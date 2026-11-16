//! Read known NAT tables and firewall rules as data.
use super::*;
use crate::{
    convert,
    network::{literal, string_array_row},
    xml::Source,
};
use roxmltree::Node;
use std::net::Ipv4Addr;

pub(super) fn gateway(
    s: &Source<'_>,
    system: Node<'_, '_>,
    device: &mut Device,
    report: &mut ImportReport,
) -> Result<(), Diagnostic> {
    if device.kind != DeviceKind::Gateway {
        return Ok(());
    }
    let retention = literal(s, system, "retentionTime")?;
    let lease = retention.as_ref().map_or(Some(300_000_000), |(_, millis)| {
        millis.parse::<u64>().ok()?.checked_mul(1000)
    });
    let Some(mut config) = lease.and_then(|lease| {
        let [wan, lan] = device.interfaces.as_slice() else {
            return None;
        };
        let config = GatewayConfig {
            lan: lan.id.clone(),
            wan: wan.id.clone(),
            nat_enabled: true,
            port_start: 40000,
            port_end: 60000,
            idle_timeout_us: Decimal(lease),
            forwards: vec![],
        };
        config.validate(device).ok()?;
        wan.ipv4.as_ref()?;
        Some(config)
    }) else {
        convert::issue(
            report,
            s,
            system,
            "import.unsupported_data",
            "nat_interfaces_or_timeout",
        );
        return Ok(());
    };
    let target = format!("{}.network.gateway", device.id.as_str());
    if let Some((retention, _)) = retention {
        s.note(retention, &format!("{target}.idle_timeout_us"));
    }
    if let Some(table) = s.property(system, "staticNAT")? {
        let mut rows = Vec::new();
        let mut complete = true;
        for row in s.view(table)?.children().filter(Node::is_element) {
            let Some(forward) = string_array_row(s, row)?.and_then(port_forward) else {
                convert::issue(
                    report,
                    s,
                    row,
                    "import.unsupported_data",
                    "nat_port_forward",
                );
                complete = false;
                continue;
            };
            let previous = config.forwards.iter().position(|f| {
                f.protocol == forward.protocol && f.external_port == forward.external_port
            });
            let prior = previous.map(|index| config.forwards.remove(index));
            config.forwards.push(forward);
            if config.validate(device).is_err() {
                config.forwards.pop();
                if let Some((index, prior)) = previous.zip(prior) {
                    config.forwards.insert(index, prior);
                }
                convert::issue(
                    report,
                    s,
                    row,
                    "import.unsupported_data",
                    "nat_port_forward",
                );
                complete = false;
                continue;
            }
            if let Some(index) = previous {
                let replaced = rows.remove(index);
                convert::issue(
                    report,
                    s,
                    replaced,
                    "import.unsupported_data",
                    "nat_superseded_forward",
                );
            }
            rows.push(row);
        }
        for (index, row) in rows.into_iter().enumerate() {
            s.note(row, &format!("{target}.forwards[{index}]"));
        }
        if complete {
            s.note(table, &format!("{target}.forwards"));
        }
    }
    device.network.gateway = Some(Box::new(config));
    Ok(())
}

fn port_forward([protocol, external, address, internal]: [String; 4]) -> Option<PortForward> {
    let protocol = match protocol.to_ascii_uppercase().as_str() {
        "TCP" | "6" => TransportProtocol::Tcp,
        "UDP" | "17" => TransportProtocol::Udp,
        _ => return None,
    };
    Some(PortForward {
        protocol,
        external_port: external.parse().ok()?,
        internal_address: address.parse().ok()?,
        internal_port: internal.parse().ok()?,
    })
}

const FIREWALL: &str = "filius.software.firewall.Firewall";
const NAT_GATEWAY: &str = "filius.software.nat.NatGateway";
const WEB_SERVER: &str = "filius.software.www.WebServer";
const FIREWALL_RULE: &str = "filius.software.firewall.FirewallRule";

fn named_beans<'s, 'a>(
    s: &'s Source<'a>,
    map: Node<'s, 'a>,
    name: &str,
    class: &str,
) -> Result<Vec<Node<'s, 'a>>, Diagnostic> {
    let mut beans = Vec::new();
    for entry in s
        .view(map)?
        .children()
        .filter(|entry| matches!(entry.attribute("method"), Some("get" | "put")))
        .filter(|entry| {
            entry
                .children()
                .find(|n| n.has_tag_name("string"))
                .and_then(|n| n.text())
                == Some(name)
        })
    {
        let bean = entry
            .children()
            .find(|n| n.has_tag_name("object"))
            .unwrap_or(entry);
        let bean = s.view(bean)?;
        if (!bean.has_tag_name("object") || bean.attribute("class") == Some(class))
            && !beans.contains(&bean)
        {
            beans.push(bean);
        }
    }
    Ok(beans)
}

fn firewall_beans<'s, 'a>(
    s: &'s Source<'a>,
    system: Node<'s, 'a>,
    kind: DeviceKind,
) -> Result<Vec<Node<'s, 'a>>, Diagnostic> {
    let Some(apps) = s.property(system, "installierteAnwendungen")? else {
        return Ok(vec![]);
    };
    let mut beans = named_beans(s, apps, FIREWALL, FIREWALL)?;
    if beans.is_empty() && kind == DeviceKind::Gateway {
        beans = named_beans(s, apps, NAT_GATEWAY, NAT_GATEWAY)?;
    }
    if beans.is_empty() && matches!(kind, DeviceKind::Gateway | DeviceKind::Router) {
        // The built-in configuration plug-in serializes the existing firewall
        // via its getter. Read its saved fields without instantiating the plug-in.
        for web in named_beans(s, apps, WEB_SERVER, WEB_SERVER)? {
            if let Some(plugins) = s.property(web, "plugins")? {
                for plugin in named_beans(
                    s,
                    plugins,
                    "konfig",
                    "filius.software.firewall.FirewallWebKonfig",
                )? {
                    if let Some(bean) = s.property(plugin, "firewall")? {
                        let bean = s.view(bean)?;
                        if (!bean.has_tag_name("object")
                            || matches!(bean.attribute("class"), Some(FIREWALL | NAT_GATEWAY)))
                            && !beans.contains(&bean)
                        {
                            beans.push(bean);
                        }
                    }
                }
            }
        }
    }
    Ok(beans)
}

fn verdict(value: &str) -> Option<Verdict> {
    match value {
        "0" => Some(Verdict::Drop),
        "1" => Some(Verdict::Accept),
        _ => None,
    }
}

fn boolean(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn rule_match(address: &str, mask: &str, local_allowed: bool) -> Option<IpMatch> {
    if address.is_empty() {
        Some(IpMatch::Any)
    } else if address == "999.999.999.999" && local_allowed {
        Some(IpMatch::LocalNetwork)
    } else {
        let address: Ipv4Addr = address.parse().ok()?;
        let prefix = convert::prefix(mask)?;
        let network = Ipv4Addr::from(u32::from(address) & prefix_mask(prefix));
        Some(IpMatch::Subnet { network, prefix })
    }
}

fn firewall_rule(s: &Source<'_>, row: Node<'_, '_>, kind: DeviceKind) -> Option<FirewallRule> {
    if row.attribute("method") != Some("add") {
        return None;
    }
    let rule = s.view(row).ok()?;
    if rule.attribute("class") != Some(FIREWALL_RULE) {
        return None;
    }
    let mut fields = BTreeMap::new();
    for field in rule.children().filter(Node::is_element) {
        if field.attribute("class") != Some(FIREWALL_RULE)
            || field.attribute("method") != Some("getField")
        {
            return None;
        }
        let mut children = field.children().filter(Node::is_element);
        let name = children
            .next()
            .filter(|node| node.has_tag_name("string"))?
            .text()?;
        let setter = children.next()?;
        if setter.attribute("method") != Some("set") || children.next().is_some() {
            return None;
        }
        let mut args = setter.children().filter(Node::is_element);
        if s.view(args.next()?).ok()? != rule {
            return None;
        }
        let value = args.next()?;
        let tag = match name {
            "srcIP" | "srcMask" | "destIP" | "destMask" => "string",
            "port" => "int",
            "protocol" | "action" => "short",
            _ => return None,
        };
        if !value.has_tag_name(tag)
            || args.next().is_some()
            || fields
                .insert(name, value.text().unwrap_or_default())
                .is_some()
        {
            return None;
        }
    }
    let get = |name, default| fields.get(name).copied().unwrap_or(default);
    Some(FirewallRule {
        source: rule_match(
            get("srcIP", ""),
            get("srcMask", ""),
            kind != DeviceKind::Gateway,
        )?,
        destination: rule_match(get("destIP", ""), get("destMask", ""), false)?,
        protocol: match get("protocol", "6") {
            "-1" => FilterProtocol::Any,
            "6" => FilterProtocol::Tcp,
            "17" => FilterProtocol::Udp,
            _ => return None,
        },
        destination_port: match get("port", "-1") {
            "-1" => None,
            value => Some(value.parse().ok()?),
        },
        action: verdict(get("action", "0"))?,
    })
}

pub(super) fn firewall(
    s: &Source<'_>,
    system: Node<'_, '_>,
    device: &mut Device,
    report: &mut ImportReport,
) -> Result<(), Diagnostic> {
    if matches!(device.kind, DeviceKind::Switch | DeviceKind::Modem) {
        return Ok(());
    }
    let beans = firewall_beans(s, system, device.kind)?;
    let bean = match beans.as_slice() {
        [] => return Ok(()),
        [bean] => *bean,
        _ => {
            convert::issue(
                report,
                s,
                system,
                "import.unsupported_data",
                "multiple_firewall_configurations",
            );
            return Ok(());
        }
    };
    let built_in = matches!(device.kind, DeviceKind::Router | DeviceKind::Gateway);
    let target = format!("{}.network.firewall", device.id.as_str());
    let fields = [
        literal(s, bean, "activated")?,
        literal(s, bean, "defaultPolicy")?,
        literal(s, bean, "filterSYNSegmentsOnly")?,
        literal(s, bean, "allowRelatedPackets")?,
        literal(s, bean, "filterUdp")?,
        literal(s, bean, "dropICMP")?,
    ];
    let get = |index: usize, default| fields[index].as_ref().map_or(default, |(_, v)| v.as_str());
    let list = s.property(bean, "ruleset")?;
    if built_in && list.is_none() && fields.iter().all(Option::is_none) {
        return Ok(());
    }
    let config = (|| {
        let syn_only = boolean(get(2, get(3, "true")))?;
        if fields[2].is_some() && fields[3].is_some() && boolean(get(3, "")) != Some(syn_only) {
            return None;
        }
        Some(FirewallConfig {
            enabled: boolean(get(0, if built_in { "false" } else { "true" }))?,
            default_policy: verdict(get(1, "0"))?,
            syn_only,
            filter_udp: boolean(get(4, "true"))?,
            drop_icmp: boolean(get(5, "false"))?,
            rules: vec![],
        })
    })();
    let Some(mut config) = config else {
        convert::issue(
            report,
            s,
            bean,
            "import.unsupported_data",
            "firewall_settings",
        );
        return Ok(());
    };
    let mut rows = Vec::new();
    if let Some(list) = list {
        for row in s.view(list)?.children().filter(Node::is_element) {
            if let Some(rule) = firewall_rule(s, row, device.kind) {
                config.rules.push(rule);
                rows.push(row);
            } else {
                convert::issue(report, s, row, "import.unsupported_data", "firewall_rule");
                return Ok(());
            }
            if config.rules.len() > 128 {
                convert::issue(
                    report,
                    s,
                    list,
                    "import.unsupported_data",
                    "firewall_rule_limit",
                );
                return Ok(());
            }
        }
        if config.validate().is_err() {
            convert::issue(report, s, list, "import.unsupported_data", "firewall_rule");
            return Ok(());
        }
        s.note(list, &format!("{target}.rules"));
        for (index, row) in rows.into_iter().enumerate() {
            s.note(row, &format!("{target}.rules[{index}]"));
        }
    }
    for (value, field) in fields.into_iter().zip([
        "enabled",
        "default_policy",
        "syn_only",
        "syn_only",
        "filter_udp",
        "drop_icmp",
    ]) {
        if let Some((property, _)) = value {
            s.note(property, &format!("{target}.{field}"));
        }
    }
    s.note(bean, &target);
    device.network.firewall = Some(Box::new(config));
    Ok(())
}
