use base64::{Engine as _, engine::general_purpose::STANDARD};
use sns_filius_import::import;
use std::{
    io::{Cursor, Write},
    path::Path,
};
fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut w = zip::ZipWriter::new(Cursor::new(vec![]));
    for (name, bytes) in entries {
        w.start_file(
            *name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored),
        )
        .unwrap();
        w.write_all(bytes).unwrap();
    }
    w.finish().unwrap().into_inner()
}
#[test]
#[ignore = "requires the local read-only Filius corpus; run explicitly with --ignored"]
fn corpus_imports_as_data_preserving_every_byte_and_reports_all_losses() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../_archive/filius-master");
    let mut files = vec![];
    fn collect(p: &Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(p).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                collect(&p, out);
            } else if p.extension().is_some_and(|e| e == "fls") {
                out.push(p);
            }
        }
    }
    collect(&root, &mut files);
    files.sort();
    assert_eq!(files.len(), 20);
    let mut summaries = vec![];
    for path in files {
        let bytes = std::fs::read(&path).unwrap();
        let result = import(&bytes, path.file_name().unwrap().to_str().unwrap())
            .unwrap_or_else(|e| panic!("{}: {e:?}", path.display()));
        result.project.validate().unwrap();
        validate_contracts(&result);
        let mut original_xml = zip::ZipArchive::new(Cursor::new(bytes.clone())).unwrap();
        let config = original_xml
            .file_names()
            .map(Result::unwrap)
            .find(|n| n.ends_with("/konfiguration.xml"))
            .unwrap()
            .to_string();
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut original_xml.by_name(&config).unwrap(), &mut xml)
            .unwrap();
        let expected = roxmltree::Document::parse(&xml)
            .unwrap()
            .descendants()
            .filter(|n| n.attribute("class") == Some("filius.gui.netzwerksicht.GUIKnotenItem"))
            .count();
        assert_eq!(result.project.devices.len(), expected, "{}", path.display());
        if expected > 0 {
            assert!(!result.report.converted.is_empty());
        }
        assert!(!result.report.execution_performed);
        assert!(!result.report.functional_equivalence_verified);
        assert!(result.project.extensions.iter().all(|e| !e.enabled));
        let mut original = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(result.report.preserved.len(), original.len());
        if let Some(extension) = match path.file_name().and_then(|n| n.to_str()) {
            Some("anwendung_komplett.fls") => Some("Accumulator"),
            Some("multicast_server.fls") => Some("Broadcast"),
            _ => None,
        } {
            for suffix in ["java", "class"] {
                assert!(result.report.preserved.iter().any(|entry| {
                    entry.executable && entry.name.ends_with(&format!("/{extension}.{suffix}"))
                }));
            }
            assert!(
                result
                    .report
                    .issues
                    .iter()
                    .any(|i| i.code == "import.inactive_extension")
            );
            assert!(!result.project.extensions.is_empty());
        }
        for item in &result.report.preserved {
            let mut restored = vec![];
            for rid in &item.resources {
                let r = result
                    .project
                    .resources
                    .iter()
                    .find(|r| &r.id == rid)
                    .unwrap();
                restored.extend(STANDARD.decode(&r.data_base64).unwrap());
            }
            let mut entry = original.by_name(&item.name).unwrap();
            let mut data = vec![];
            std::io::Read::read_to_end(&mut entry, &mut data).unwrap();
            assert_eq!(restored, data, "{}:{}", path.display(), item.name);
        }
        let project = serde_json::to_vec(&result.project).unwrap();
        assert_eq!(
            sns_project::parse_project(&project).unwrap(),
            result.project
        );
        let mut mail = std::collections::BTreeMap::<&str, usize>::new();
        use sns_model::{ApplicationData, ServiceConfig};
        for app in result
            .project
            .devices
            .iter()
            .flat_map(|device| &device.applications)
        {
            match &app.service {
                Some(ServiceConfig::MailClient(_)) => {
                    *mail.entry("configured_clients").or_default() += 1
                }
                Some(ServiceConfig::MailServer(config)) => {
                    *mail.entry("server_accounts").or_default() += config.accounts.len()
                }
                _ => {}
            }
            match &app.data {
                Some(ApplicationData::MailClient(data)) => {
                    *mail.entry("inbox").or_default() += data.inbox.len();
                    *mail.entry("sent").or_default() += data.sent.len();
                    *mail.entry("contacts").or_default() += data.contacts.len();
                }
                Some(ApplicationData::MailServer(data)) => {
                    *mail.entry("mailboxes").or_default() +=
                        data.mailboxes.values().map(Vec::len).sum::<usize>();
                }
                _ => {}
            }
        }
        summaries.push(serde_json::json!({"path":path.strip_prefix(&root).unwrap().to_string_lossy(),"report":&result.report,"mail":mail}));
    }
    if let Some(path) = std::env::var_os("SNS_RECORD_J7_IMPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&summaries).unwrap()).unwrap();
    }
}
#[test]
fn unsafe_archives_and_entity_xml_and_dangling_aliases_are_rejected() {
    for name in [
        "../outside",
        "/absolute",
        "C:/system",
        "projekt/../outside",
        "projekt//bad",
    ] {
        let bytes = archive(&[("projekt/konfiguration.xml", b"<java/>"), (name, b"x")]);
        assert!(import(&bytes, "unsafe.fls").is_err(), "{name}");
    }
    for xml in [
        "<!DOCTYPE java [<!ENTITY x SYSTEM 'file:///etc/passwd'>]><java>&x;</java>",
        "<java><object idref='missing'/></java>",
        "<java><object id='x'/><object id='x'/></java>",
    ] {
        assert!(
            import(
                &archive(&[("projekt/konfiguration.xml", xml.as_bytes())]),
                "unsafe.fls"
            )
            .is_err()
        );
    }
    assert!(import(b"not a zip", "invalid.fls").is_err());
    let bomb = archive(&[(
        "projekt/konfiguration.xml",
        &vec![b'x'; 8 * 1024 * 1024 + 1],
    )]);
    assert!(import(&bomb, "bomb.fls").is_err());
}
#[test]
fn arbitrary_embedded_code_is_preserved_and_disabled() {
    let xml=b"<java><object class='java.lang.ProcessBuilder'><string>must never execute</string></object></java>";
    let bytes = archive(&[
        ("projekt/konfiguration.xml", xml),
        ("projekt/anwendungen/Unsafe.class", b"class-data"),
        ("projekt/anwendungen/Unsafe.java", b"source-data"),
    ]);
    let r = import(&bytes, "unknown.fls").unwrap();
    assert_eq!(r.project.extensions.len(), 2);
    assert!(r.project.extensions.iter().all(|e| !e.enabled));
    assert!(
        r.report
            .unknown_classes
            .contains(&"java.lang.ProcessBuilder".into())
    );
}
#[test]
#[ignore = "requires the local read-only Filius LAN example; run explicitly with --ignored"]
fn imported_historical_lan_uses_production_arp_and_icmp() {
    use sns_model::*;
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../_archive/filius-master/beispiele/einfaches_rechnernetz_komplett.fls");
    let r = import(
        &std::fs::read(path).unwrap(),
        "einfaches_rechnernetz_komplett.fls",
    )
    .unwrap();
    let source = r
        .project
        .devices
        .iter()
        .find(|d| d.label == "Rechner 1")
        .unwrap()
        .id
        .clone();
    let mut e =
        sns_engine::Engine::from_project(r.project, 42, sns_engine::Limits::default()).unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: Id::new("imported-ping").unwrap(),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(0),
        command: Command::Ping {
            source,
            destination: "192.168.0.12".into(),
            ttl: 64,
        },
    })
    .unwrap();
    e.run(10000, Some(3_000_000)).unwrap();
    assert!(e.trace().iter().any(|t| t.kind == "icmp.echo.reply"));
    assert!(e.trace().iter().any(|t| t.kind == "arp.request"));
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_IMPORT_DIR") {
        let dir = Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("j7-import-lan.jsonl"), e.canonical_trace()).unwrap();
    }
}
#[test]
fn duplicate_names_bad_crc_and_unsafe_virtual_tree_are_rejected() {
    let bytes = archive(&[
        ("projekt/konfiguration.xml", b"<java/>"),
        ("projekt/aaa", b"unchanged"),
        ("projekt/bbb", b"bad"),
    ]);
    let mut duplicate = bytes.clone();
    let pattern = b"projekt/bbb";
    for i in 0..duplicate.len() - pattern.len() {
        if &duplicate[i..i + pattern.len()] == pattern {
            duplicate[i..i + pattern.len()].copy_from_slice(b"projekt/aaa");
        }
    }
    assert!(import(&duplicate, "duplicate.fls").is_err());
    let mut corrupt = bytes;
    let i = corrupt.windows(9).position(|w| w == b"unchanged").unwrap();
    corrupt[i] ^= 1;
    assert!(import(&corrupt, "corrupt.fls").is_err());
    let xml=b"<java><object class='filius.gui.netzwerksicht.GUIKnotenItem'><void property='knoten'><object class='filius.hardware.knoten.Rechner'><void property='systemSoftware'><void property='dateisystem'><void property='arbeitsVerzeichnis'><void method='add'><object class='javax.swing.tree.DefaultMutableTreeNode'><void property='userObject'><string>..</string></void></object></void></void></void></void></object></void></object></java>";
    assert!(
        import(
            &archive(&[("projekt/konfiguration.xml", xml)]),
            "unsafe-tree.fls"
        )
        .is_err()
    );
}
#[test]
fn portable_synthetic_import_roundtrips_and_exchanges_real_simulated_frames() {
    use sns_model::*;
    let r = import(
        include_bytes!("../../../fixtures/legacy/lan.fls"),
        "lan.fls",
    )
    .unwrap();
    validate_contracts(&r);
    assert_eq!(r.report.counts.devices, 3);
    assert_eq!(r.report.counts.links, 2);
    assert!(
        r.report
            .unconverted_fields
            .iter()
            .any(|f| f.contains("unmappedTeachingNote"))
    );
    let mut e =
        sns_engine::Engine::from_project(r.project, 42, sns_engine::Limits::default()).unwrap();
    let id = |s: &str| Id::new(s).unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("import-ping"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(0),
        command: Command::Ping {
            source: id("legacy-device-0"),
            destination: "192.168.8.3".into(),
            ttl: 64,
        },
    })
    .unwrap();
    e.run(10000, Some(3_000_000)).unwrap();
    assert!(e.trace().iter().any(|t| t.kind == "icmp.echo.reply"));
}

fn validate_contracts(result: &sns_filius_import::ImportResult) {
    for (schema, value) in [
        (
            include_str!("../../../contracts/import-report.schema.json"),
            serde_json::to_value(&result.report).unwrap(),
        ),
        (
            match result.project.schema_version {
                9 => include_str!("../../../contracts/project-documentation.schema.json"),
                8 => include_str!("../../../contracts/project-presentation.schema.json"),
                _ => include_str!("../../../contracts/project-advanced.schema.json"),
            },
            serde_json::to_value(&result.project).unwrap(),
        ),
    ] {
        let schema: serde_json::Value = serde_json::from_str(schema).unwrap();
        jsonschema::options()
            .build(&schema)
            .unwrap()
            .validate(&value)
            .unwrap();
    }
}
#[test]
fn import_is_deterministic_and_mac_repairs_are_unique() {
    let bytes = include_bytes!("../../../fixtures/legacy/lan.fls");
    let a = import(bytes, "lan.fls").unwrap();
    let b = import(bytes, "lan.fls").unwrap();
    assert_eq!(a.project, b.project);
    assert_eq!(
        serde_json::to_vec(&a.report).unwrap(),
        serde_json::to_vec(&b.report).unwrap()
    );
    let xml = include_str!("../../../fixtures/legacy/lan.xml")
        .replace("02:08:00:00:00:01", "02:7f:00:00:00:02")
        .replace("02:08:00:00:00:02", "02:7f:00:00:00:02");
    let r = import(
        &archive(&[("projekt/konfiguration.xml", xml.as_bytes())]),
        "duplicate-mac.fls",
    )
    .unwrap();
    let mut macs = std::collections::BTreeSet::new();
    assert!(
        r.project
            .devices
            .iter()
            .flat_map(|d| &d.interfaces)
            .all(|i| macs.insert(&i.mac))
    );
    assert!(
        r.report
            .issues
            .iter()
            .any(|i| i.code == "import.generated_value")
    );
}

#[test]
fn cloud_flag_imports_as_presentation_data_without_changing_ports_or_wires() {
    let source = include_str!("../../../fixtures/legacy/lan.xml");
    let source=source.replace("class=\"filius.hardware.knoten.Switch\">","class=\"filius.hardware.knoten.Switch\"><void property=\"cloud\"><boolean>true</boolean></void>");
    let base = sns_filius_import::import(
        include_bytes!("../../../fixtures/legacy/lan.fls"),
        "base.fls",
    )
    .unwrap();
    let bytes = archive(&[("cloud/konfiguration.xml", source.as_bytes())]);
    let imported = sns_filius_import::import(&bytes, "cloud.fls").unwrap();
    assert_eq!(imported.project.schema_version, 8);
    let switch = imported
        .project
        .devices
        .iter()
        .find(|d| d.kind == sns_model::DeviceKind::Switch)
        .unwrap();
    assert_eq!(
        imported.project.lan[&switch.id].symbol,
        Some(sns_model::DeviceSymbol::Cloud)
    );
    let original = base
        .project
        .devices
        .iter()
        .find(|d| d.kind == sns_model::DeviceKind::Switch)
        .unwrap();
    assert_eq!(switch.interfaces.len(), original.interfaces.len());
    assert_eq!(imported.project.links.len(), base.project.links.len());
    assert!(
        imported
            .report
            .converted
            .iter()
            .any(|f| f.source.ends_with(":cloud"))
    );
    assert!(!imported.report.execution_performed);
    validate_contracts(&imported);
}

#[test]
#[ignore = "requires the local read-only Filius corpus; run explicitly with --ignored"]
fn imported_historical_dhcp_and_manual_routes_drive_the_engine() {
    use sns_model::*;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../_archive/filius-master/beispiele");
    let load = |name| import(&std::fs::read(root.join(name)).unwrap(), name).unwrap();

    let dhcp = load("dhcp_test_scenario.fls");
    let servers: Vec<_> = dhcp
        .project
        .devices
        .iter()
        .filter_map(|device| {
            device
                .network
                .dhcp_server
                .as_ref()
                .map(|config| (device, config))
        })
        .collect();
    assert_eq!(servers.len(), 2);
    let (gateway, pool) = servers
        .iter()
        .find(|(device, _)| device.kind == DeviceKind::Gateway)
        .unwrap();
    assert_eq!(pool.interface, gateway.interfaces[1].id);
    assert_eq!(pool.gateway, Some("192.168.0.10".parse().unwrap()));
    assert_eq!(pool.lease_us, Decimal(86_400_000_000));
    let mut dhcp =
        sns_engine::Engine::from_project(dhcp.project, 42, sns_engine::Limits::default()).unwrap();
    dhcp.run(10000, Some(5_000_000)).unwrap();
    let leases: Vec<_> = dhcp
        .trace()
        .iter()
        .filter(|event| event.kind == "dhcp.lease.acquired")
        .collect();
    assert_eq!(leases.len(), 2, "{leases:?}");
    assert!(
        leases
            .iter()
            .any(|event| event.payload["assignment"]["gateway"] == "192.168.0.10")
    );
    assert!(
        leases
            .iter()
            .any(|event| event.payload["assignment"]["address"] == "192.168.0.101")
    );

    let routes = load("internetwork_komplett.fls");
    let source = routes
        .project
        .devices
        .iter()
        .find(|device| device.label == "Notebook 1.10")
        .unwrap()
        .id
        .clone();
    assert_eq!(
        routes
            .project
            .devices
            .iter()
            .map(|d| d.network.routes.len())
            .sum::<usize>(),
        2
    );
    let mut routes =
        sns_engine::Engine::from_project(routes.project, 42, sns_engine::Limits::default())
            .unwrap();
    routes
        .command(CommandEnvelope {
            api_version: 1,
            command_id: Id::new("imported-routed-ping").unwrap(),
            project_id: routes.project().project_id.clone(),
            run_id: routes.run_id().clone(),
            expected_revision: Decimal(0),
            command: Command::Ping {
                source,
                destination: "141.99.5.10".into(),
                ttl: 64,
            },
        })
        .unwrap();
    routes.run(10000, Some(5_000_000)).unwrap();
    assert!(routes.trace().iter().any(|t| t.kind == "icmp.echo.reply"));
    for next_hop in ["141.99.4.2", "141.99.2.1"] {
        assert!(routes.trace().iter().any(|t| {
            t.kind == "ipv4.forwarded"
                && t.payload["next_hop"] == next_hop
                && t.payload["route"]["origin"] == "static"
        }));
    }
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_IMPORT_DIR") {
        let dir = Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("j7-import-dhcp.jsonl"), dhcp.canonical_trace()).unwrap();
        std::fs::write(
            dir.join("j7-import-static-route.jsonl"),
            routes.canonical_trace(),
        )
        .unwrap();
    }
}

#[test]
fn gateway_dhcp_keeps_the_lan_static_and_reports_rejected_reservations() {
    use sns_model::Ipv4Config;
    let xml = r#"<java>
      <object class="filius.gui.netzwerksicht.GUIKnotenItem">
        <void property="knoten"><object class="filius.hardware.knoten.Gateway">
          <void property="netzwerkInterfaces">
            <void index="0"><object class="filius.hardware.NetzwerkInterface"/></void>
            <void index="1"><object class="filius.hardware.NetzwerkInterface"/></void>
          </void>
          <void property="systemSoftware">
            <void property="DHCPKonfiguration"><object idref="enabled"/></void>
            <void property="DHCPServer">
              <void property="aktiv"><object idref="enabled"/></void>
              <void property="untergrenze"><string>192.168.0.100</string></void>
              <void property="obergrenze"><string>192.168.0.150</string></void>
              <void property="dnsserverip"><string>192.168.0.2</string></void>
              <void property="staticAssignedAddresses">
                <void method="add"><string>02:00:00:00:00:02 192.168.0.80</string></void>
                <void method="add"><string>02:00:00:00:00:03 192.168.1.80</string></void>
              </void>
            </void>
          </void>
        </object></void>
      </object>
      <void id="enabled" method="isEnabled"/>
    </java>"#;
    let result = import(
        &archive(&[("projekt/konfiguration.xml", xml.as_bytes())]),
        "gateway-dhcp.fls",
    )
    .unwrap();
    let device = &result.project.devices[0];
    assert_eq!(device.interfaces[0].ipv4, Some(Ipv4Config::Dhcp));
    assert_eq!(
        device.interfaces[1].ipv4,
        Some(Ipv4Config::Static {
            address: "192.168.0.10".into(),
            prefix_length: 24
        })
    );
    let config = device.network.dhcp_server.as_ref().unwrap();
    assert_eq!(config.interface, device.interfaces[1].id);
    assert_eq!(config.dns, Some("192.168.0.2".parse().unwrap()));
    assert_eq!(config.reservations.len(), 1);
    assert_eq!(
        config.reservations["02:00:00:00:00:02"],
        "192.168.0.80".parse::<std::net::Ipv4Addr>().unwrap()
    );
    assert!(
        result
            .report
            .issues
            .iter()
            .any(|issue| issue.detail == "dhcp_reservation")
    );
    assert!(
        result
            .report
            .unconverted_fields
            .iter()
            .any(|field| field.ends_with(":staticAssignedAddresses"))
    );
    validate_contracts(&result);

    let xml = xml.replace(
        r#"property="aktiv"><object idref="enabled"/>"#,
        r#"property="aktiv"><boolean>false</boolean>"#,
    );
    let disabled = import(
        &archive(&[("projekt/konfiguration.xml", xml.as_bytes())]),
        "disabled.fls",
    )
    .unwrap();
    assert!(disabled.project.devices[0].network.dhcp_server.is_none());
    assert!(
        disabled
            .report
            .unconverted_fields
            .iter()
            .any(|field| field.ends_with(":untergrenze"))
    );
}

#[test]
#[ignore = "requires the local read-only Filius corpus; run explicitly with --ignored"]
fn imported_historical_nat_and_firewall_handle_real_packets() {
    use sns_engine::{Engine, Limits};
    use sns_model::*;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../_archive/filius-master/beispiele");
    let load = |name| {
        import(&std::fs::read(root.join(name)).unwrap(), name)
            .unwrap()
            .project
    };
    let find = |p: &Project, label: &str| {
        p.devices
            .iter()
            .find(|d| d.label == label)
            .unwrap()
            .id
            .clone()
    };
    let app = |e: &mut Engine, device: &Id, command: AppCommand| {
        e.command(CommandEnvelope {
            api_version: 1,
            command_id: Id::new(format!("import-security-{}", e.trace().len())).unwrap(),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::App {
                device: device.clone(),
                command,
            },
        })
        .unwrap();
    };
    let browse = |device: &Id, url: &str| AppCommand::Navigate {
        application: Id::new(format!("{}-web_browser", device.as_str())).unwrap(),
        url: url.into(),
        fields: None,
    };
    let start_http = |device: &Id| AppCommand::StartServer {
        application: Id::new(format!("{}-web_server", device.as_str())).unwrap(),
        port: 80,
    };

    let nat = load("nat_with_mapped_port.fls");
    let historical_timeout = load("nat.fls");
    assert_eq!(
        historical_timeout.devices[0]
            .network
            .gateway
            .as_ref()
            .unwrap()
            .idle_timeout_us,
        Decimal(30_000_000)
    );
    let gateway = find(&nat, "Heimrouter Vincent");
    let cfg = nat
        .device(&gateway)
        .unwrap()
        .network
        .gateway
        .as_ref()
        .unwrap();
    assert_eq!(
        cfg.forwards[0].internal_address,
        "192.168.0.10".parse::<std::net::Ipv4Addr>().unwrap()
    );
    let server = find(&nat, "Webserver - 192.168.0.10");
    let client = find(&nat, "Notebook 1.10");
    let mut nat = Engine::from_project(nat, 42, Limits::default()).unwrap();
    app(&mut nat, &server, start_http(&server));
    app(&mut nat, &client, browse(&client, "http://141.99.5.10/"));
    nat.run(10000, Some(3_000_000)).unwrap();
    assert!(nat.trace().iter().any(|t| t.device_id == gateway
        && t.kind == "nat.translated"
        && t.payload["direction"] == "inbound"
        && t.payload["after"]["destination"] == "192.168.0.10"));
    assert!(nat.trace().iter().any(|t| t.device_id == client
        && t.kind == "http.response.received"
        && t.payload["status"] == 200));

    let mut firewall = load(
        "Internet_Complete_with_e-Mail_Webserver_Intranet_Portforwarding_Firewall_DHCP_EN.fls",
    );
    let gateway = find(&firewall, "Firewall");
    let campus = find(&firewall, "Notebook 1");
    let reserved = find(&firewall, "Notebook 3");
    let provider = find(&firewall, "Provider 1");
    let server = find(&firewall, "myschool.org (10.0.0.1)");
    // A replacement card on the original cable exercises the dynamic pool.
    firewall
        .devices
        .iter_mut()
        .find(|d| d.id == campus)
        .unwrap()
        .interfaces[0]
        .mac = "02:00:00:00:fe:01".into();
    let cfg = firewall
        .device(&gateway)
        .unwrap()
        .network
        .firewall
        .as_ref()
        .unwrap();
    assert!(cfg.enabled);
    assert_eq!(cfg.rules.len(), 6);
    assert_eq!(cfg.rules[1].action, Verdict::Drop);
    let mut firewall = Engine::from_project(firewall, 42, Limits::default()).unwrap();
    app(&mut firewall, &server, start_http(&server));
    app(
        &mut firewall,
        &provider,
        AppCommand::Install {
            application: Id::new(format!("{}-web_browser", provider.as_str())).unwrap(),
            software: Software::WebBrowser,
        },
    );
    firewall.run(10000, Some(1_000_000)).unwrap();
    app(
        &mut firewall,
        &provider,
        browse(&provider, "http://42.0.0.10/"),
    );
    app(&mut firewall, &campus, browse(&campus, "http://10.0.0.2/"));
    app(
        &mut firewall,
        &reserved,
        browse(&reserved, "http://10.0.0.2/"),
    );
    firewall.run(10000, Some(4_000_000)).unwrap();
    let decisions = &firewall.network().firewall.journal[&gateway];
    assert!(
        decisions
            .iter()
            .any(|d| d.rule == Some(6) && d.action == Verdict::Accept),
        "{decisions:?}"
    );
    assert!(
        decisions
            .iter()
            .any(|d| d.rule == Some(2) && d.action == Verdict::Drop && d.protocol == "tcp"),
        "{decisions:?}"
    );
    assert!(
        decisions
            .iter()
            .any(|d| d.rule == Some(1) && d.action == Verdict::Accept && d.protocol == "tcp")
    );
    assert!(firewall.trace().iter().any(|t| t.device_id == provider
        && t.kind == "http.response.received"
        && t.payload["status"] == 200));
    for engine in [&nat, &firewall] {
        let checkpoint = engine.checkpoint().unwrap().restore().unwrap();
        let replay = engine.recording().unwrap().replay().unwrap();
        assert_eq!(checkpoint.canonical_trace(), engine.canonical_trace());
        assert_eq!(replay.canonical_trace(), engine.canonical_trace());
    }
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_IMPORT_DIR") {
        let dir = Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("j7-import-nat.jsonl"), nat.canonical_trace()).unwrap();
        std::fs::write(
            dir.join("j7-import-firewall.jsonl"),
            firewall.canonical_trace(),
        )
        .unwrap();
    }
}

#[test]
fn unsafe_or_unrepresentable_security_rows_remain_visible() {
    let forward = |protocol: &str, port: &str, ip: &str| {
        let cells = [protocol, port, ip, "80"]
            .iter()
            .enumerate()
            .map(|(index, field)| {
                format!(r#"<void index="{index}"><string>{field}</string></void>"#)
            })
            .collect::<String>();
        format!(
            r#"<void method="add"><array class="java.lang.String" length="4">{cells}</array></void>"#
        )
    };
    let first = forward("TCP", "8080", "192.168.0.20");
    let replacement = forward("6", "8080", "192.168.0.21");
    let invalid = forward("UDP", "53", "192.0.2.80");
    let xml = format!(
        r#"<java>
      <object class="filius.gui.netzwerksicht.GUIKnotenItem">
        <void property="knoten"><object class="filius.hardware.knoten.Gateway">
          <void property="netzwerkInterfaces">
            <void index="0"><object class="filius.hardware.NetzwerkInterface"/></void>
            <void index="1"><object class="filius.hardware.NetzwerkInterface"/></void>
          </void>
          <void property="systemSoftware">
            <void property="staticNAT">{first}{replacement}{invalid}</void>
            <void property="installierteAnwendungen">
              <void method="get"><string>filius.software.nat.NatGateway</string>
                <void property="activated"><boolean>true</boolean></void>
                <void property="defaultPolicy"><short>1</short></void>
                <void property="ruleset">
                  <void method="add"><object class="filius.software.firewall.FirewallRule"/></void>
                  <void method="add"><object class="custom.ExtensionRule"/></void>
                </void>
              </void>
            </void>
          </void>
        </object></void>
      </object>
    </java>"#
    );
    let result = import(
        &archive(&[("projekt/konfiguration.xml", xml.as_bytes())]),
        "invalid-security.fls",
    )
    .unwrap();
    let net = &result.project.devices[0].network;
    let forwards = &net.gateway.as_ref().unwrap().forwards;
    assert_eq!(forwards.len(), 1);
    assert_eq!(forwards[0].internal_address.to_string(), "192.168.0.21");
    assert!(net.firewall.is_none());
    for detail in [
        "nat_superseded_forward",
        "nat_port_forward",
        "firewall_rule",
    ] {
        assert!(
            result
                .report
                .issues
                .iter()
                .any(|issue| issue.detail == detail)
        );
    }
    for field in [":staticNAT", ":ruleset"] {
        assert!(
            result
                .report
                .unconverted_fields
                .iter()
                .any(|source| source.ends_with(field))
        );
    }
    validate_contracts(&result);
}

#[test]
#[ignore = "requires the local read-only Filius corpus; run explicitly with --ignored"]
fn imported_historical_rip_converges_forwards_and_replays() {
    use sns_model::*;
    let name =
        "Internet_Complete_with_e-Mail_Webserver_Intranet_Portforwarding_Firewall_DHCP_EN.fls";
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../_archive/filius-master/beispiele")
        .join(name);
    let imported = import(&std::fs::read(path).unwrap(), name).unwrap();
    assert_eq!(
        imported
            .project
            .devices
            .iter()
            .filter(|d| d.network.rip)
            .count(),
        7
    );
    assert_eq!(
        imported
            .project
            .devices
            .iter()
            .find(|d| d.label == "myschool.org (10.0.0.1)")
            .unwrap()
            .network
            .forwarding,
        Some(true)
    );
    let source = imported
        .project
        .devices
        .iter()
        .find(|d| d.label == "mail.gmx.com (130.100.200.25)")
        .unwrap()
        .id
        .clone();
    let mut engine =
        sns_engine::Engine::from_project(imported.project, 42, Default::default()).unwrap();
    engine.run(100_000, Some(5_000_000)).unwrap();
    assert!(
        engine
            .trace()
            .iter()
            .any(|t| t.kind == "rip.route.changed"
                && t.payload["route"]["network"] == "140.100.200.0")
    );
    let checkpoint = engine.checkpoint().unwrap();
    engine
        .command(CommandEnvelope {
            api_version: 1,
            command_id: Id::new("import-rip-ping").unwrap(),
            project_id: engine.project().project_id.clone(),
            run_id: engine.run_id().clone(),
            expected_revision: Decimal(engine.revision()),
            command: Command::Ping {
                source,
                destination: "140.100.200.25".into(),
                ttl: 64,
            },
        })
        .unwrap();
    engine.run(100_000, Some(10_000_000)).unwrap();
    for destination in ["140.100.200.25", "130.100.200.25"] {
        assert!(engine.trace().iter().any(|t| t.kind == "ipv4.forwarded"
            && t.payload["destination"] == destination
            && t.payload["route"]["origin"] == "rip"));
    }
    assert!(
        engine
            .trace()
            .iter()
            .any(|t| t.kind == "icmp.echo.reply" && t.payload["source"] == "140.100.200.25")
    );
    let recording = engine.recording().unwrap();
    assert_eq!(
        recording.replay().unwrap().canonical_trace(),
        engine.canonical_trace()
    );
    assert_eq!(
        checkpoint
            .replay_remaining(&recording)
            .unwrap()
            .canonical_trace(),
        engine.canonical_trace()
    );

    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_IMPORT_DIR") {
        let dir = Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("j7-import-rip.jsonl"), engine.canonical_trace()).unwrap();
    }
}

#[test]
fn explicit_rip_keeps_legacy_static_choices_inactive() {
    let xml = r#"<java>
      <object class="filius.gui.netzwerksicht.GUIKnotenItem">
        <void property="knoten"><object class="filius.hardware.knoten.Vermittlungsrechner">
          <void property="netzwerkInterfaces"><void index="0">
            <void property="gateway"><string>192.168.0.1</string></void>
          </void></void>
          <void property="systemSoftware">
            <void property="ipForwardingEnabled"><boolean>false</boolean></void>
            <void property="ripEnabled"><boolean>true</boolean></void>
            <void property="weiterleitungstabelle"><void property="manuelleTabelle">
              <void method="add"><array class="java.lang.String" length="4">
                <void index="0"><string>203.0.113.0</string></void>
                <void index="1"><string>255.255.255.0</string></void>
                <void index="2"><string>192.168.0.1</string></void>
                <void index="3"><string>192.168.0.10</string></void>
              </array></void>
            </void></void>
          </void>
        </object></void>
      </object>
    </java>"#;
    let load = |source: &str| {
        import(
            &archive(&[("projekt/konfiguration.xml", source.as_bytes())]),
            "routing.fls",
        )
        .unwrap()
    };
    let rip = load(xml);
    let net = &rip.project.devices[0].network;
    assert!(net.rip);
    assert_eq!(net.forwarding, Some(false));
    assert!(net.default_gateway.is_none());
    assert!(net.routes.is_empty());
    for detail in [
        "rip_inactive_default_gateway",
        "static_table_inactive_or_unsupported",
    ] {
        assert!(rip.report.issues.iter().any(|issue| issue.detail == detail));
    }
    let static_mode = load(&xml.replace("<boolean>true</boolean>", "<boolean>false</boolean>"));
    let net = &static_mode.project.devices[0].network;
    assert!(!net.rip);
    assert_eq!(net.default_gateway.as_deref(), Some("192.168.0.1"));
    assert_eq!(net.routes.len(), 1);
}

#[test]
fn documentation_layers_colors_and_font_flags_are_portable() {
    let xml = r#"<java>
      <object class="filius.gui.netzwerksicht.GUIDocuItem">
        <void property="type"><int>2</int></void>
        <void property="text"><string>École</string></void>
        <void property="color"><object class="java.awt.Color">
          <object class="javax.swing.SwingConstants" field="CENTER"/>
          <int>80</int><int>125</int><int>255</int>
        </object></void>
        <void property="font"><object class="java.awt.Font">
          <string>Monospaced</string><int>3</int><int>12</int>
        </object></void>
        <void property="width"><int>88</int></void>
        <void property="height"><int>20</int></void>
      </object>
      <object class="filius.gui.netzwerksicht.GUIDocuItem">
        <void property="type"><int>1</int></void>
        <void property="color"><object class="java.awt.Color">
          <int>204</int><int>204</int><int>204</int>
        </object></void>
        <void property="width"><int>428</int></void>
        <void property="height"><int>292</int></void>
      </object>
    </java>"#;
    let result = import(
        &archive(&[("projekt/konfiguration.xml", xml.as_bytes())]),
        "notes.fls",
    )
    .unwrap();
    use sns_model::{AnnotationFont, AnnotationKind, DOCUMENTATION_SCHEMA_VERSION};
    assert_eq!(result.project.schema_version, DOCUMENTATION_SCHEMA_VERSION);
    let area = result.project.annotations[0].style.as_ref().unwrap();
    assert_eq!(area.kind, AnnotationKind::Rectangle);
    assert_eq!([area.width, area.height], [428., 292.]);
    assert_eq!(area.fill_color, [215, 215, 215, 255]);
    assert_eq!(area.border_color, [163, 163, 163, 255]);
    let note = &result.project.annotations[1];
    assert_eq!(note.text, "École");
    let style = note.style.as_ref().unwrap();
    assert_eq!(style.kind, AnnotationKind::Text);
    assert_eq!(style.font, AnnotationFont::Monospace);
    assert!(style.bold && style.italic);
    assert_eq!(style.text_color, [0, 80, 125, 255]);
    assert_eq!(style.fill_color, [0; 4]);
    assert_eq!(style.height, 27.);
    assert!(
        result
            .report
            .issues
            .iter()
            .any(|issue| issue.detail == "annotation_minimum_size")
    );
    validate_contracts(&result);
}

#[test]
fn imports_mail_folders_and_contacts_with_private_bcc() {
    use sns_model::*;
    let message = r#"<void method="add"><object class="filius.software.email.Email">
        <void property="absender"><object class="filius.software.email.AddressEntry">
          <void property="mailAddress"><string>alice@example.test</string></void>
          <void property="name"><string>Alice</string></void>
        </object></void>
        <void property="empfaenger"><void method="add"><string>bob@example.test</string></void></void>
        <void property="cc"><void method="add"><object class="filius.software.email.AddressEntry">
          <void property="mailAddress"><string>copie@example.test</string></void>
        </object></void></void>
        <void property="bcc"><void method="add"><string>discret@example.test</string></void></void>
        <void property="betreff"><string>Réseau</string></void>
        <void property="text"><string>corps-confidentiel</string></void>
        <void property="dateReceived"><string>date-locale-confidentielle</string></void>
      </object></void>"#;
    let xml = format!(
        r#"<java>
      <object class="filius.gui.netzwerksicht.GUIKnotenItem">
        <void property="knoten"><object class="filius.hardware.knoten.Rechner">
          <void property="systemSoftware"><void property="installierteAnwendungen">
            <void method="get"><string>filius.software.email.EmailAnwendung</string>
              <void property="kontoListe"><void method="put"><string>alice</string>
                <object class="filius.software.email.EmailKonto">
                  <void property="benutzername"><string>alice</string></void>
                  <void property="passwort"><string>mot-de-passe-confidentiel</string></void>
                  <void property="emailAdresse"><string>alice@example.test</string></void>
                  <void property="smtpserver"><string>mail.example.test</string></void>
                  <void property="pop3server"><string>mail.example.test</string></void>
                </object>
              </void></void>
              <void property="empfangeneNachrichten">{message}</void>
              <void property="gesendeteNachrichten">{message}</void>
              <void property="adressbuch">
                <void method="add"><object class="filius.software.email.Kontakt">
                  <void property="vorname"><string>Élodie</string></void>
                  <void property="name"><string>Test</string></void>
                  <void property="hausnr"><int>12</int></void>
                  <void property="plz"><int>75000</int></void>
                  <void property="email"><string>contact@example.test</string></void>
                </object></void>
                <void method="add"><object class="filius.software.email.Kontakt">
                  <void property="email"><string>invalid</string></void>
                </object></void>
              </void>
            </void>
            <void method="get"><string>filius.software.email.EmailServer</string>
              <void property="listeBenutzerkonten"><void method="add">
                <object class="filius.software.email.EmailKonto">
                  <void property="benutzername"><string>bob</string></void>
                  <void property="passwort"><string>mot-de-passe-confidentiel</string></void>
                  <void property="nachrichten">{message}</void>
                </object>
              </void></void>
            </void>
          </void></void>
        </object></void>
      </object>
    </java>"#
    );
    let imported = import(
        &archive(&[("project/konfiguration.xml", xml.as_bytes())]),
        "mail.fls",
    )
    .unwrap();
    let app = &imported.project.devices[0].applications[0];
    assert!(matches!(app.service, Some(ServiceConfig::MailClient(_))));
    let Some(ApplicationData::MailClient(data)) = &app.data else {
        panic!("missing content");
    };
    assert_eq!(data.inbox.len(), 1);
    assert_eq!(data.sent.len(), 1);
    assert!(data.inbox[0].bcc.is_empty());
    assert_eq!(data.sent[0].bcc, ["discret@example.test"]);
    assert_eq!(data.inbox[0].cc, ["copie@example.test"]);
    assert_eq!(data.inbox[0].sender_name, "Alice");
    assert_eq!(data.inbox[0].body, "corps-confidentiel");
    assert_eq!(data.inbox[0].received_us, Decimal(0));
    assert_eq!(data.contacts.len(), 1);
    assert_eq!(data.contacts[0].first_name, "Élodie");
    assert_eq!(data.contacts[0].house_number, "12");
    let server = &imported.project.devices[0].applications[1];
    let Some(ApplicationData::MailServer(server)) = &server.data else {
        panic!("missing server mailbox");
    };
    assert_eq!(server.mailboxes["bob"].len(), 1);
    assert!(server.mailboxes["bob"][0].bcc.is_empty());
    assert_ne!(server.mailboxes["bob"][0].id, data.inbox[0].id);
    let report = serde_json::to_string(&imported.report).unwrap();
    for secret in [
        "corps-confidentiel",
        "mot-de-passe-confidentiel",
        "discret@example.test",
        "date-locale-confidentielle",
    ] {
        assert!(!report.contains(secret));
    }
    for detail in [
        "mail_contact",
        "mail_bcc_preserved_in_original",
        "mail_wall_clock_date",
    ] {
        assert!(imported.report.issues.iter().any(|i| i.detail == detail));
    }
    validate_contracts(&imported);
}

#[test]
#[ignore = "requires the local read-only Filius corpus; run explicitly with --ignored"]
fn historical_mailbox_is_retrieved_over_simulated_pop3_and_replays() {
    use sha2::{Digest, Sha256};
    use sns_model::*;
    let name = "man-in-the-middle.fls";
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../_archive/filius-master/beispiele")
        .join(name);
    let imported = import(&std::fs::read(path).unwrap(), name).unwrap();
    let bob = imported
        .project
        .devices
        .iter()
        .find(|d| d.label == "Bob")
        .unwrap();
    let client = bob
        .applications
        .iter()
        .find(|a| a.kind == "mail_client")
        .unwrap();
    assert!(matches!(client.service, Some(ServiceConfig::MailClient(_))));
    let Some(ApplicationData::MailClient(before)) = &client.data else {
        panic!("missing inbox");
    };
    assert_eq!((before.inbox.len(), before.sent.len()), (1, 3));
    let host = imported
        .project
        .devices
        .iter()
        .find(|d| d.label == "Mail-Server (filius.de 141.99.50.232)")
        .unwrap();
    let server = host
        .applications
        .iter()
        .find(|a| a.kind == "mail_server")
        .unwrap();
    let Some(ApplicationData::MailServer(mail)) = &server.data else {
        panic!("missing mailbox");
    };
    assert_eq!(mail.mailboxes.values().map(Vec::len).sum::<usize>(), 2);
    let (bob, client, host, server) = (
        bob.id.clone(),
        client.id.clone(),
        host.id.clone(),
        server.id.clone(),
    );

    let mut engine =
        sns_engine::Engine::from_project(imported.project, 42, Default::default()).unwrap();
    let command = |e: &mut sns_engine::Engine, key: &str, device: Id, application: Id, command| {
        e.command(CommandEnvelope {
            api_version: 1,
            command_id: Id::new(key).unwrap(),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::App {
                device,
                command: AppCommand::Mail {
                    application,
                    command,
                },
            },
        })
        .unwrap();
    };
    command(
        &mut engine,
        "import-mail-start",
        host,
        server,
        MailCommand::Start,
    );
    engine.run(10000, Some(1_000_000)).unwrap();
    let checkpoint = engine.checkpoint().unwrap();
    command(
        &mut engine,
        "import-mail-receive",
        bob.clone(),
        client.clone(),
        MailCommand::Receive,
    );
    engine.run(10000, Some(10_000_000)).unwrap();
    let app = engine
        .project()
        .device(&bob)
        .unwrap()
        .applications
        .iter()
        .find(|a| a.id == client)
        .unwrap();
    let Some(ApplicationData::MailClient(after)) = &app.data else {
        panic!("missing retrieved inbox");
    };
    assert_eq!((after.inbox.len(), after.sent.len()), (3, 3));
    assert!(after.inbox.iter().all(|m| m.bcc.is_empty()));
    let recorded = engine.recording().unwrap();
    let trace = engine.canonical_trace();
    assert!(recorded.replay().unwrap().canonical_trace() == trace);
    assert!(
        checkpoint
            .replay_remaining(&recorded)
            .unwrap()
            .canonical_trace()
            == trace
    );

    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_IMPORT_DIR") {
        let dir = Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        // Keep only counts and a digest. Wire records can contain credentials or bodies.
        let summary = serde_json::json!({
            "source": name, "inbox_before": 1, "inbox_after": after.inbox.len(),
            "sent_after": after.sent.len(), "trace_events": engine.trace().len(),
            "trace_sha256": format!("{:x}", Sha256::digest(&trace)),
            "bcc_received": after.inbox.iter().map(|m| m.bcc.len()).sum::<usize>(),
            "checkpoint_and_replay_match": true
        });
        std::fs::write(
            dir.join("j7-import-mail-pop3.json"),
            serde_json::to_vec_pretty(&summary).unwrap(),
        )
        .unwrap();
    }
}
