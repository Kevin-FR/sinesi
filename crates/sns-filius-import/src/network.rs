//! Static configuration from literal Filius data. Runtime state is left in the archive.
use super::*;
use crate::{convert, xml::Source};
use roxmltree::Node;
use std::net::Ipv4Addr;

type Xml<'s, 'a> = Node<'s, 'a>;

pub(super) fn literal<'s, 'a>(
    s: &'s Source<'a>,
    n: Xml<'s, 'a>,
    field: &str,
) -> Result<Option<(Xml<'s, 'a>, String)>, Diagnostic> {
    let Some(property) = s.property(n, field)? else {
        return Ok(None);
    };
    Ok(Some((property, s.value(property)?.unwrap_or_default())))
}

pub(super) fn dhcp(
    s: &Source<'_>,
    system: Node<'_, '_>,
    device: &mut Device,
    report: &mut ImportReport,
) -> Result<(), Diagnostic> {
    let Some(server) = s.property(system, "DHCPServer")? else {
        return Ok(());
    };
    let Some((active, value)) = literal(s, server, "aktiv")? else {
        return Ok(());
    };
    let target = format!("{}.network.dhcp_server", device.id.as_str());
    match value.as_str() {
        "true" => {}
        "false" => {
            s.note(active, &target);
            return Ok(());
        }
        _ => {
            convert::issue(
                report,
                s,
                active,
                "import.invalid_value",
                "dhcp_enabled_flag",
            );
            return Ok(());
        }
    }

    let gateway_device = device.kind == DeviceKind::Gateway;
    let own = if gateway_device {
        None
    } else {
        literal(s, server, "ownSettings")?
    };
    let own_settings = match own.as_ref().map(|(_, value)| value.as_str()) {
        None | Some("false") => false,
        Some("true") => true,
        _ => {
            convert::issue(
                report,
                s,
                server,
                "import.invalid_value",
                "dhcp_option_mode",
            );
            return Ok(());
        }
    };
    let start = literal(s, server, "untergrenze")?;
    let end = literal(s, server, "obergrenze")?;
    let gateway = if !gateway_device && own_settings {
        literal(s, server, "gatewayip")?
    } else {
        None
    };
    let dns = if gateway_device || own_settings {
        literal(s, server, "dnsserverip")?
    } else {
        None
    };
    let address = |value: &Option<(_, String)>| value.as_ref()?.1.trim().parse::<Ipv4Addr>().ok();
    let option = |value: &Option<(_, String)>| match value {
        Some((_, text)) if !text.trim().is_empty() => {
            address(value).map(|ip| (!ip.is_unspecified()).then_some(ip))
        }
        _ => Some(None),
    };
    let config = (|| {
        // GatewayFirmware binds its server to interface 1 (LAN), and the
        // default firmware advertises the primary interface's subnet mask.
        let nic = device.interfaces.get(usize::from(gateway_device))?;
        let Some(Ipv4Config::Static {
            address: ip,
            prefix_length,
        }) = &nic.ipv4
        else {
            return None;
        };
        let config = DhcpConfig {
            interface: nic.id.clone(),
            pool_start: address(&start)?,
            pool_end: address(&end)?,
            prefix_length: *prefix_length,
            gateway: if gateway_device {
                Some(ip.parse().ok()?)
            } else {
                option(&gateway)?
            },
            dns: option(&dns)?,
            use_host_options: !gateway_device && !own_settings,
            reservations: BTreeMap::new(),
            lease_us: Decimal(86_400_000_000),
        };
        config.validate(device).ok()?;
        Some(config)
    })();
    let Some(mut config) = config else {
        convert::issue(
            report,
            s,
            server,
            "import.unsupported_data",
            "dhcp_pool_interface_or_options",
        );
        return Ok(());
    };

    for (value, field) in [
        (start, "pool_start"),
        (end, "pool_end"),
        (own, "use_host_options"),
        (gateway, "gateway"),
        (dns, "dns"),
    ] {
        if let Some((property, _)) = value {
            s.note(property, &format!("{target}.{field}"));
        }
    }
    if let Some(list) = s.property(server, "staticAssignedAddresses")? {
        let list_view = s.view(list)?;
        let mut complete = true;
        for row in list_view.children().filter(Node::is_element) {
            let value = s.value(row)?;
            let entry = value.as_deref().and_then(|value| {
                let mut fields = value.split_whitespace();
                let mac = fields.next()?.to_ascii_lowercase();
                let ip = fields.next()?.parse::<Ipv4Addr>().ok()?;
                (fields.next().is_none() && row.attribute("method") == Some("add"))
                    .then_some((mac, ip))
            });
            if let Some((mac, ip)) = entry
                && !config.reservations.contains_key(&mac)
            {
                config.reservations.insert(mac.clone(), ip);
                if config.validate(device).is_ok() {
                    s.note(row, &format!("{target}.reservations"));
                    continue;
                }
                config.reservations.remove(&mac);
            }
            complete = false;
            convert::issue(
                report,
                s,
                row,
                "import.unsupported_data",
                "dhcp_reservation",
            );
        }
        if complete {
            s.note(list, &format!("{target}.reservations"));
        }
    }
    device.network.dhcp_server = Some(Box::new(config));
    s.note(server, &target);
    s.note(active, &target);
    Ok(())
}

pub(super) fn string_array_row<'s, 'a>(
    s: &'s Source<'a>,
    row: Xml<'s, 'a>,
) -> Result<Option<[String; 4]>, Diagnostic> {
    let mut children = row.children().filter(Node::is_element);
    let Some(array) = children.next() else {
        return Ok(None);
    };
    let array = s.view(array)?;
    if row.attribute("method") != Some("add")
        || children.next().is_some()
        || !array.has_tag_name("array")
        || array.attribute("class") != Some("java.lang.String")
        || array.attribute("length") != Some("4")
    {
        return Ok(None);
    }
    let mut slots = [None, None, None, None];
    for cell in array.children().filter(Node::is_element) {
        let index = cell
            .attribute("index")
            .and_then(|value| value.parse::<usize>().ok());
        let Some(slot) = index.and_then(|index| slots.get_mut(index)) else {
            return Ok(None);
        };
        let mut children = cell.children().filter(Node::is_element);
        let Some(value) = children.next() else {
            return Ok(None);
        };
        let value = s.view(value)?;
        if slot.is_some() || children.next().is_some() || !value.has_tag_name("string") {
            return Ok(None);
        }
        *slot = Some(value.text().unwrap_or_default().trim().to_owned());
    }
    let [Some(first), Some(second), Some(third), Some(fourth)] = slots else {
        return Ok(None);
    };
    Ok(Some([first, second, third, fourth]))
}

fn static_route(
    [network, mask, gateway, interface]: [String; 4],
    device: &Device,
) -> Option<StaticRoute> {
    let network: Ipv4Addr = network.parse().ok()?;
    let prefix = convert::prefix(&mask)?;
    let gateway = gateway.parse().ok()?;
    let interface_ip: Ipv4Addr = interface.parse().ok()?;
    let mut interfaces = device.interfaces.iter().filter_map(|nic| match &nic.ipv4 {
        Some(Ipv4Config::Static {
            address,
            prefix_length,
        }) if address.parse().ok() == Some(interface_ip) => Some((nic, *prefix_length)),
        _ => None,
    });
    let (nic, interface_prefix) = interfaces.next()?;
    if interfaces.next().is_some() {
        return None;
    }
    let interface_mask = u32::MAX
        .checked_shl(32 - u32::from(interface_prefix))
        .unwrap_or(0);
    let on_link = prefix >= interface_prefix
        && u32::from(network) & interface_mask == u32::from(interface_ip) & interface_mask;
    let route = StaticRoute {
        network,
        prefix,
        gateway: (!on_link).then_some(gateway),
        interface: nic.id.clone(),
    };
    route.validate(device).ok()?;
    Some(route)
}

pub(super) fn routes(
    s: &Source<'_>,
    system: Node<'_, '_>,
    device: &mut Device,
    report: &mut ImportReport,
) -> Result<(), Diagnostic> {
    let Some(table) = s.property(system, "weiterleitungstabelle")? else {
        return Ok(());
    };
    let Some(list) = s.property(table, "manuelleTabelle")? else {
        return Ok(());
    };
    let list_view = s.view(list)?;
    let mut rows = list_view.children().filter(Node::is_element).peekable();
    if rows.peek().is_none() {
        return Ok(());
    }
    if device.kind == DeviceKind::Switch
        || literal(s, system, "ripEnabled")?.is_some_and(|(_, value)| value != "false")
    {
        convert::issue(
            report,
            s,
            list,
            "import.unsupported_data",
            "static_table_inactive_or_unsupported",
        );
        return Ok(());
    }

    let target = format!("{}.network.routes", device.id.as_str());
    let mut destinations = BTreeSet::new();
    let mut complete = true;
    for row in rows {
        if let Some(route) =
            string_array_row(s, row)?.and_then(|fields| static_route(fields, device))
            && device.network.routes.len() < 256
            && destinations.insert((route.network, route.prefix))
        {
            s.note(row, &format!("{target}[{}]", device.network.routes.len()));
            device.network.routes.push(route);
        } else {
            complete = false;
            convert::issue(
                report,
                s,
                row,
                "import.unsupported_data",
                "manual_ipv4_route",
            );
        }
    }
    if complete {
        s.note(table, &target);
        s.note(list, &target);
    }
    Ok(())
}

pub(super) fn routing_flags(
    s: &Source<'_>,
    system: Node<'_, '_>,
    device: &mut Device,
    report: &mut ImportReport,
) -> Result<(), Diagnostic> {
    let target = device.id.as_str();
    for (name, allowed) in [
        ("ripEnabled", device.kind == DeviceKind::Router),
        (
            "ipForwardingEnabled",
            !matches!(device.kind, DeviceKind::Switch | DeviceKind::Modem),
        ),
    ] {
        let Some((property, value)) = literal(s, system, name)? else {
            continue;
        };
        let enabled = match value.as_str() {
            "true" => true,
            "false" => false,
            _ => {
                convert::issue(
                    report,
                    s,
                    property,
                    "import.invalid_value",
                    "routing_boolean",
                );
                continue;
            }
        };
        if !allowed {
            convert::issue(
                report,
                s,
                property,
                "import.unsupported_data",
                "routing_mode_device",
            );
            continue;
        }
        let field = if name == "ripEnabled" {
            device.network.rip = enabled;
            "rip"
        } else {
            device.network.forwarding = Some(enabled);
            "forwarding"
        };
        s.note(property, &format!("{target}.network.{field}"));
    }
    Ok(())
}
