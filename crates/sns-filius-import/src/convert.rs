//! Explicit mappings from historical data to the current educational network profile.
use super::*;
use crate::xml::Source;
use roxmltree::Node;
fn kind(class: &str) -> Option<DeviceKind> {
    Some(match class.rsplit('.').next()? {
        "Rechner" => DeviceKind::Workstation,
        "Notebook" => DeviceKind::Notebook,
        "Switch" => DeviceKind::Switch,
        "Vermittlungsrechner" => DeviceKind::Router,
        "Gateway" => DeviceKind::Gateway,
        "Modem" => DeviceKind::Modem,
        _ => return None,
    })
}
fn software(class: &str) -> Option<Software> {
    Some(match class.rsplit('.').next()? {
        "Terminal" => Software::Terminal,
        "FileExplorer" | "DateiExplorer" | "DateiManager" => Software::FileExplorer,
        "TextEditor" => Software::TextEditor,
        "ImageViewer" | "BildBetrachter" => Software::ImageViewer,
        "ServerBaustein" | "EchoServer" => Software::TcpServer,
        "ClientBaustein" | "EinfacherClient" => Software::TcpClient,
        "WebServer" => Software::WebServer,
        "WebBrowser" => Software::WebBrowser,
        "DNSServer" => Software::DnsServer,
        "EmailServer" => Software::MailServer,
        "EmailAnwendung" => Software::MailClient,
        "PeerToPeerAnwendung" => Software::P2p,
        "Firewall" => Software::Firewall,
        _ => return None,
    })
}
pub(super) fn scalar(
    s: &Source<'_>,
    n: Node<'_, '_>,
    p: &str,
    t: &str,
    default: &str,
) -> Result<String, Diagnostic> {
    Ok(s.string(n, p, t)?.unwrap_or_else(|| default.into()))
}
pub(super) fn issue(
    report: &mut ImportReport,
    s: &Source<'_>,
    n: Node<'_, '_>,
    code: &str,
    detail: &str,
) {
    report.issues.push(ImportIssue {
        code: code.into(),
        source: s.location(n),
        detail: detail.into(),
    });
}
fn ip(s: &str) -> Option<String> {
    s.parse::<std::net::Ipv4Addr>()
        .ok()
        .filter(|a| !a.is_unspecified())
        .map(|a| a.to_string())
}
pub(super) fn prefix(mask: &str) -> Option<u8> {
    let bits = u32::from(mask.parse::<std::net::Ipv4Addr>().ok()?);
    let p = bits.leading_ones() as u8;
    if bits == if p == 0 { 0 } else { u32::MAX << (32 - p) } {
        Some(p)
    } else {
        None
    }
}
fn identity(n: Node<'_, '_>) -> Option<String> {
    n.attribute("idref")
        .or(n.attribute("id"))
        .map(str::to_string)
}
fn coord(s: &Source<'_>, n: Node<'_, '_>, name: &str) -> Option<f32> {
    n.children()
        .find(|c| {
            c.attribute("method") == Some("getField")
                && c.children()
                    .any(|c| c.has_tag_name("string") && c.text() == Some(name))
        })
        .and_then(|c| c.descendants().find(|n| n.has_tag_name("int")))
        .and_then(|c| s.value(c).ok().flatten())
        .and_then(|v| v.parse().ok())
        .or_else(|| {
            let values = n
                .children()
                .filter(|n| n.has_tag_name("int"))
                .filter_map(|n| n.text()?.parse().ok())
                .collect::<Vec<f32>>();
            if values.len() == 4 {
                Some(values[usize::from(name == "y")])
            } else {
                None
            }
        })
}
pub(super) fn project(
    s: &mut Source<'_>,
    report: &mut ImportReport,
    digest: &str,
) -> Result<Project, Diagnostic> {
    let mut p = Project {
        schema_version: 7,
        semantic_profile: "sns-educational-applications-v1".into(),
        project_id: id(format!("import-{}", &digest[..24])),
        name: report.source_name.clone(),
        devices: vec![],
        links: vec![],
        layout: Layout { devices: vec![] },
        annotations: vec![],
        resources: vec![],
        extensions: vec![],
        lan: BTreeMap::new(),
    };
    let mut ports = BTreeMap::new();
    let mut macs = BTreeSet::new();
    let mut serial = 0u64;
    let mut ssids = BTreeSet::new();
    let nodes = s
        .doc
        .descendants()
        .filter(|n| n.attribute("class") == Some("filius.gui.netzwerksicht.GUIKnotenItem"))
        .collect::<Vec<_>>();
    for (index, item) in nodes.iter().enumerate() {
        let Some(hardware) = s.prop(*item, "knoten", "devices")? else {
            issue(
                report,
                s,
                *item,
                "import.unsupported_data",
                "device_without_hardware",
            );
            continue;
        };
        let Some(device_kind) = hardware.attribute("class").and_then(kind) else {
            issue(
                report,
                s,
                hardware,
                "import.unsupported_data",
                "hardware_class",
            );
            continue;
        };
        let did = id(format!("legacy-device-{index}"));
        let target = did.as_str();
        let label = scalar(
            s,
            hardware,
            "name",
            target,
            &format!(
                "{} {}",
                hardware
                    .attribute("class")
                    .unwrap()
                    .rsplit('.')
                    .next()
                    .unwrap(),
                index + 1
            ),
        )?;
        let mut d = Device {
            id: did.clone(),
            kind: device_kind,
            label,
            interfaces: vec![],
            network: Network::default(),
            applications: vec![],
            files: vec![],
        };
        let system = s.prop(hardware, "systemSoftware", target)?;
        if matches!(device_kind, DeviceKind::Router | DeviceKind::Gateway) {
            d.network.forwarding = Some(true);
        }
        if let Some(system) = system {
            crate::network::routing_flags(s, system, &mut d, report)?;
        }
        let selected_ssid = if let Some(system) = system {
            s.string(system, "SSID", target)?
        } else {
            None
        };
        let interfaces = s.prop(hardware, "netzwerkInterfaces", target)?;
        if let Some(interfaces) = interfaces {
            for entry in interfaces.children().filter(Node::is_element) {
                if entry.attribute("index").is_none() && entry.attribute("method") != Some("add") {
                    continue;
                }
                let nic = s.view(entry)?;
                serial += 1;
                let iid = id(format!("legacy-interface-{serial}"));
                let target = iid.as_str();
                let first = d.interfaces.is_empty();
                let (default_address, default_mask) = if device_kind == DeviceKind::Gateway && first
                {
                    ("42.0.0.10", "255.0.0.0")
                } else {
                    ("192.168.0.10", "255.255.255.0")
                };
                let address = scalar(s, nic, "ip", target, default_address)?;
                let mask = scalar(s, nic, "subnetzMaske", target, default_mask)?;
                let mut ipv4 = ip(&address)
                    .zip(prefix(&mask))
                    .map(|(address, prefix_length)| Ipv4Config::Static {
                        address,
                        prefix_length,
                    });
                if ipv4.is_none() {
                    issue(report, s, nic, "import.invalid_value", "ipv4_or_netmask");
                }
                if first
                    && let Some(sys) = system
                    && scalar(s, sys, "DHCPKonfiguration", target, "false")? == "true"
                {
                    ipv4 = Some(Ipv4Config::Dhcp);
                }
                let wireless = scalar(s, nic, "wireless", target, "false")? == "true";
                let medium = if wireless {
                    Medium::Wireless
                } else {
                    Medium::Ethernet
                };
                let proposed = scalar(s, nic, "mac", target, "")?.to_lowercase();
                let mac = if proposed.len() == 17
                    && proposed.split(':').count() == 6
                    && proposed
                        .split(':')
                        .all(|v| v.len() == 2 && u8::from_str_radix(v, 16).is_ok())
                    && macs.insert(proposed.clone())
                {
                    proposed
                } else {
                    issue(
                        report,
                        s,
                        nic,
                        "import.generated_value",
                        "mac_generated_or_collision_repaired",
                    );

                    fresh_mac(&mut serial, &mut macs)
                };
                if d.network.rip {
                    if let Some((property, gateway)) = crate::network::literal(s, nic, "gateway")?
                        && !gateway.trim().is_empty()
                    {
                        issue(
                            report,
                            s,
                            property,
                            "import.unsupported_data",
                            "rip_inactive_default_gateway",
                        );
                    }
                } else {
                    d.network.default_gateway = d
                        .network
                        .default_gateway
                        .or(ip(&scalar(s, nic, "gateway", target, "")?));
                }
                d.network.dns_server = d
                    .network
                    .dns_server
                    .or(ip(&scalar(s, nic, "dns", target, "")?));
                if let Some(port) = s.prop(nic, "port", target)?
                    && let Some(key) = identity(port)
                    && ports.insert(key, iid.clone()).is_some()
                {
                    return Err(error("import.invalid_reference", "shared_port"));
                }
                if wireless
                    && let Some(ssid) = &selected_ssid
                    && ssid_valid(ssid)
                {
                    d.network.wireless.insert(iid.clone(), ssid.clone());
                }
                d.interfaces.push(Interface {
                    id: iid,
                    mac,
                    medium,
                    ipv4,
                });
            }
        }
        if matches!(device_kind, DeviceKind::Switch | DeviceKind::Modem)
            && let Some(list) = s.prop(hardware, "anschluesse", target)?
        {
            for entry in list.children().filter(Node::is_element) {
                if entry.attribute("index").is_none() && entry.attribute("method") != Some("add") {
                    continue;
                }
                let port = s.view(entry)?;
                serial += 1;
                let iid = id(format!("legacy-interface-{serial}"));
                if let Some(key) = identity(port).or_else(|| identity(entry))
                    && ports.insert(key, iid.clone()).is_some()
                {
                    return Err(error("import.invalid_reference", "shared_port"));
                }
                d.interfaces.push(Interface {
                    id: iid,
                    mac: fresh_mac(&mut serial, &mut macs),
                    medium: Medium::Ethernet,
                    ipv4: None,
                });
            }
        }
        // Missing constructor defaults are explicit data rules, never reflected into Java.
        let minimum = if matches!(device_kind, DeviceKind::Router | DeviceKind::Gateway) {
            2
        } else {
            1
        };
        while d.interfaces.len() < minimum {
            serial += 1;
            issue(
                report,
                s,
                hardware,
                "import.generated_value",
                "unconnected_constructor_interface",
            );
            d.interfaces.push(Interface {
                id: id(format!("legacy-interface-{serial}")),
                mac: fresh_mac(&mut serial, &mut macs),
                medium: Medium::Ethernet,
                ipv4: None,
            });
        }
        if device_kind == DeviceKind::Switch
            && let Some(ssid) = selected_ssid
        {
            if ssid_valid(&ssid) && ssids.insert(ssid.clone()) {
                serial += 1;
                d.interfaces.push(Interface {
                    id: id(format!("legacy-interface-{serial}")),
                    mac: fresh_mac(&mut serial, &mut macs),
                    medium: Medium::Wireless,
                    ipv4: None,
                });
                d.network.ssid = Some(ssid);
            } else {
                issue(
                    report,
                    s,
                    hardware,
                    "import.invalid_value",
                    "duplicate_or_invalid_ssid",
                );
            }
        }
        if device_kind == DeviceKind::Modem {
            d.network.modem = Some(ModemConfig::default());
            issue(
                report,
                s,
                hardware,
                "import.inactive_extension",
                "modem_endpoint_requires_explicit_configuration",
            );
        }
        if let Some(system) = system {
            crate::network::dhcp(s, system, &mut d, report)?;
            crate::network::routes(s, system, &mut d, report)?;
            crate::security::gateway(s, system, &mut d, report)?;
            crate::security::firewall(s, system, &mut d, report)?;
            host_data(s, system, &mut d, report)?;
        }
        let mut display = DisplayName::Name;
        if scalar(s, hardware, "useIPAsName", target, "false")? == "true" {
            display = DisplayName::Ipv4;
        }
        let cloud = if device_kind == DeviceKind::Switch {
            match scalar(s, hardware, "cloud", target, "false")?.as_str() {
                "true" => true,
                "false" => false,
                _ => {
                    issue(
                        report,
                        s,
                        hardware,
                        "import.unsupported_data",
                        "invalid_cloud_flag",
                    );
                    false
                }
            }
        } else {
            false
        };
        if cloud {
            p.schema_version = 8;
            p.semantic_profile = PRESENTATION_PROFILE.into();
        }
        p.lan.insert(
            did.clone(),
            LanConfig {
                symbol: cloud.then_some(DeviceSymbol::Cloud),
                display,
                ..Default::default()
            },
        );
        let bounds = s
            .prop(*item, "imageLabel", target)?
            .map(|button| s.prop(button, "bounds", target))
            .transpose()?
            .flatten();
        let (x, y) = if let Some(bounds) = bounds {
            (
                coord(s, bounds, "x").unwrap_or(100.0 + (index % 5) as f32 * 160.0),
                coord(s, bounds, "y").unwrap_or(100.0 + (index / 5) as f32 * 120.0),
            )
        } else {
            (
                100.0 + (index % 5) as f32 * 160.0,
                100.0 + (index / 5) as f32 * 120.0,
            )
        };
        p.layout.devices.push(Position {
            device_id: did,
            x,
            y,
        });
        p.devices.push(d);
    }
    let mut occupied = BTreeSet::new();
    for cable in s
        .doc
        .descendants()
        .filter(|n| n.attribute("class") == Some("filius.hardware.Kabel"))
    {
        let Some(list) = s.prop(cable, "anschluesse", "links")? else {
            continue;
        };
        let endpoints = list
            .descendants()
            .filter_map(|n| n.attribute("idref").or(n.attribute("id")))
            .filter_map(|key| ports.get(key).cloned())
            .collect::<Vec<_>>();
        if endpoints.len() != 2 {
            return Err(error("import.invalid_reference", s.location(cable)));
        }
        let endpoints: [Id; 2] = endpoints.try_into().unwrap();
        if endpoints.iter().any(|id| {
            p.device_for_interface(id)
                .unwrap()
                .interfaces
                .iter()
                .any(|i| &i.id == id && i.medium == Medium::Wireless)
        }) {
            issue(
                report,
                s,
                cable,
                "import.unsupported_data",
                "historical_wireless_cable_replaced_by_ssid_association",
            );
            continue;
        }
        if endpoints[0] == endpoints[1] || endpoints.iter().any(|id| !occupied.insert(id.clone())) {
            return Err(error("import.invalid_reference", "occupied_port"));
        }
        p.links.push(Link {
            id: id(format!("legacy-link-{}", p.links.len())),
            endpoints,
            latency_us: Decimal(1000),
            jitter_us: Decimal(0),
        });
    }
    p.annotations = crate::annotations::read(s, report)?;
    if p.annotations.iter().any(|a| a.style.is_some()) {
        p.schema_version = DOCUMENTATION_SCHEMA_VERSION;
        p.semantic_profile = DOCUMENTATION_PROFILE.into();
    }
    Ok(p)
}
fn host_data(
    s: &Source<'_>,
    system: Node<'_, '_>,
    d: &mut Device,
    report: &mut ImportReport,
) -> Result<(), Diagnostic> {
    if matches!(d.kind, DeviceKind::Switch | DeviceKind::Modem) {
        return Ok(());
    }
    if let Some(fs) = s.prop(system, "dateisystem", d.id.as_str())?
        && let Some(root) = s.prop(fs, "arbeitsVerzeichnis", d.id.as_str())?
    {
        let mut paths = BTreeSet::new();
        files(s, root, "", d, report, &mut paths)?;
    }
    sns_vfs::Vfs::from_files(&d.files)?;
    if let Some(apps) = s.prop(system, "installierteAnwendungen", d.id.as_str())? {
        let mut seen = BTreeSet::new();
        for entry in apps.children().filter(Node::is_element) {
            let class = entry
                .children()
                .find(|n| n.has_tag_name("string"))
                .and_then(|n| n.text())
                .unwrap_or("");
            let Some(software) = software(class) else {
                issue(
                    report,
                    s,
                    entry,
                    "import.unsupported_data",
                    "application_class_preserved_in_source",
                );
                continue;
            };
            if !seen.insert(software) {
                continue;
            }
            let bean = entry
                .children()
                .find(|n| n.has_tag_name("object"))
                .map(|n| s.view(n))
                .transpose()?
                .unwrap_or(entry);
            let aid = id(format!("{}-{}", d.id.as_str(), software.id()));
            let target = aid.as_str();
            let mut data = None;
            let mut service = match software {
                Software::WebServer => Some(ServiceConfig::Http(HttpConfig {
                    root: "/webserver".into(),
                    form_path: None,
                    ..Default::default()
                })),
                Software::DnsServer => Some(ServiceConfig::Dns(DnsConfig {
                    recursive: scalar(s, bean, "recursiveResolutionEnabled", target, "false")?
                        == "true",
                    ..Default::default()
                })),
                Software::MailServer | Software::MailClient => {
                    let (configuration, content) =
                        crate::mail::configuration(s, bean, software, target, &d.files, report)?;
                    data = content;
                    configuration
                }
                Software::P2p => Some(ServiceConfig::P2p(P2pConfig::default())),
                _ => None,
            };
            if software == Software::DnsServer
                && let Some(ServiceConfig::Dns(c)) = &mut service
                && let Some(file) = d.files.iter().find(|f| f.path == "/dns/hosts")
            {
                let bytes = STANDARD.decode(&file.data_base64).unwrap_or_default();
                for line in String::from_utf8_lossy(&bytes).lines() {
                    let v = line.split_whitespace().collect::<Vec<_>>();
                    if v.is_empty() {
                        continue;
                    }
                    let record = (|| -> Option<DnsRecord> {
                        let [name, kind, ttl, value] = v.as_slice() else {
                            return None;
                        };
                        let record_type = match *kind {
                            "A" => DnsType::A,
                            "MX" => DnsType::Mx,
                            "NS" => DnsType::Ns,
                            _ => return None,
                        };
                        Some(DnsRecord {
                            name: dns_name(name).ok()?,
                            record_type,
                            value: if record_type == DnsType::A {
                                ip(value)?
                            } else {
                                dns_name(value).ok()?
                            },
                            ttl_seconds: ttl.parse().ok()?,
                            priority: 0,
                        })
                    })();
                    if let Some(r) = record {
                        c.records.push(r);
                    } else {
                        issue(
                            report,
                            s,
                            bean,
                            "import.unsupported_data",
                            "dns_record_in_original_file",
                        );
                    }
                }
            }
            if let Some(config) = &service
                && config.validate(software.id()).is_err()
            {
                issue(
                    report,
                    s,
                    bean,
                    "import.invalid_value",
                    "service_configuration_requires_review",
                );
                service = None;
            }
            d.applications.push(Application {
                id: aid,
                kind: software.id().into(),
                config: BTreeMap::new(),
                service,
                data,
            });
        }
    }
    Ok(())
}
fn files(
    s: &Source<'_>,
    node: Node<'_, '_>,
    parent: &str,
    d: &mut Device,
    report: &mut ImportReport,
    paths: &mut BTreeSet<String>,
) -> Result<(), Diagnostic> {
    for add in node
        .children()
        .filter(|n| n.attribute("method") == Some("add"))
    {
        let tree = s.view(add)?;
        let Some(user) = s.prop(tree, "userObject", d.id.as_str())? else {
            continue;
        };
        let object = s.view(user)?;
        let file = object.attribute("class") == Some("filius.software.system.Datei");
        let name = if file {
            scalar(s, object, "name", d.id.as_str(), "unnamed")?
        } else {
            s.value(user)?.unwrap_or_default()
        };
        if name.is_empty() || name.contains('/') || name.contains('\\') {
            return Err(error("import.unsafe_path", name));
        }
        let path = format!("{parent}/{name}");
        if !virtual_path_valid(&path) || !paths.insert(path.clone()) {
            return Err(error("import.unsafe_path", path));
        }
        if file {
            let body = scalar(s, object, "dateiInhalt", d.id.as_str(), "")?;
            let typ = scalar(s, object, "dateiTyp", d.id.as_str(), "")?.to_lowercase();
            let ext = if typ.is_empty() {
                name.rsplit('.').next().unwrap_or("").to_lowercase()
            } else {
                typ
            };
            let (media, data) = match ext.as_str() {
                "png" | "jpg" | "jpeg" | "gif" | "bmp" => {
                    let bytes = STANDARD
                        .decode(body.split_whitespace().collect::<String>())
                        .map_err(|_| error("import.invalid_xml", "image_base64"))?;
                    (
                        format!("image/{}", if ext == "jpg" { "jpeg" } else { &ext }),
                        bytes,
                    )
                }
                "html" | "htm" => ("text/html".into(), body.into_bytes()),
                _ => ("text/plain".into(), body.into_bytes()),
            };
            d.files.push(VirtualFile {
                path,
                media_type: media,
                data_base64: STANDARD.encode(data),
            });
        } else {
            d.files.push(VirtualFile {
                path: path.clone(),
                media_type: "inode/directory".into(),
                data_base64: String::new(),
            });
            files(s, tree, &path, d, report, paths)?;
        }
    }
    let _ = report;
    Ok(())
}

fn fresh_mac(serial: &mut u64, used: &mut BTreeSet<String>) -> String {
    loop {
        let mac = format!(
            "02:7f:{:02x}:{:02x}:{:02x}:{:02x}",
            (*serial >> 24) & 255,
            (*serial >> 16) & 255,
            (*serial >> 8) & 255,
            *serial & 255
        );
        if used.insert(mac.clone()) {
            return mac;
        }
        *serial += 1;
    }
}
