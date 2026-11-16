//! F-TOP-009: copies remain complete, independently addressable and reversible.
use sns_model::*;
use sns_topology::*;
use std::collections::BTreeSet;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn project(name: &str) -> Project {
    let bytes = match name {
        "private" => include_str!("../../../fixtures/valid/private.json"),
        "wireless" => include_str!("../../../fixtures/valid/wireless.json"),
        "mail" => include_str!("../../../fixtures/valid/mail.json"),
        _ => include_str!("../../../fixtures/valid/lan.json"),
    };
    serde_json::from_str(bytes).unwrap()
}
#[test]
fn repeated_whole_graph_copies_have_unique_identities_macs_and_work_over_their_own_cables() {
    let mut p = project("lan");
    p.annotations.push(Annotation {
        style: None,
        id: id("lesson-note"),
        text: "Réseau pédagogique".into(),
        x: 60.,
        y: 450.,
    });
    p.resources.push(Resource {
        id: id("copy-device-1"),
        media_type: "application/octet-stream".into(),
        data_base64: "AA==".into(),
    });
    p.resources.push(Resource {
        id: id("copy-port-3"),
        media_type: "application/octet-stream".into(),
        data_base64: "AA==".into(),
    });
    let selection: BTreeSet<_> = p
        .devices
        .iter()
        .map(|d| d.id.clone())
        .chain([id("lesson-note")])
        .collect();
    let source = p.clone();
    let copy = duplicate_selection(&p, &selection, 600., 40.).unwrap();
    assert_eq!(p, source);
    assert_eq!(copy.created.len(), 8);
    assert_eq!(copy.project.devices.len(), 8);
    assert_eq!(copy.project.links.len(), 6);
    assert_eq!(copy.project.annotations[1].text, p.annotations[0].text);
    assert_ne!(copy.project.annotations[1].id, p.annotations[0].id);
    assert_eq!(copy.potential_conflicts.len(), 3);
    assert!(
        copy.potential_conflicts
            .iter()
            .all(|v| v.interfaces.len() == 2)
    );
    let cloned_nodes: BTreeSet<_> = p
        .devices
        .iter()
        .map(|d| copy.identities[&d.id].clone())
        .collect();
    let second = duplicate_selection(&copy.project, &cloned_nodes, 600., 40.).unwrap();
    second.project.validate().unwrap();
    assert_eq!(second.project.devices.len(), 12);
    assert_eq!(second.project.links.len(), 9);
    let macs: Vec<_> = second
        .project
        .devices
        .iter()
        .flat_map(|d| &d.interfaces)
        .map(|i| i.mac.to_ascii_lowercase())
        .collect();
    assert_eq!(macs.len(), macs.iter().collect::<BTreeSet<_>>().len());
    let mut e =
        sns_engine::Engine::from_project(copy.project.clone(), 42, Default::default()).unwrap();
    let a = copy.identities[&id("host-a")].clone();
    let b = copy.identities[&id("host-b")].clone();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("copy-ping"),
        project_id: e.project().project_id.clone(),
        expected_revision: Decimal(0),
        run_id: e.run_id().clone(),
        command: Command::Ping {
            source: a.clone(),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    })
    .unwrap();
    e.run(10000, Some(4_000_000)).unwrap();
    assert!(
        e.trace()
            .iter()
            .any(|t| t.kind == "icmp.echo.reply" && t.device_id == a)
    );
    assert!(
        e.trace()
            .iter()
            .any(|t| t.kind == "icmp.echo.received" && t.device_id == b)
    );
    assert!(
        e.trace()
            .iter()
            .filter(|t| t.kind == "ethernet.received")
            .all(|t| cloned_nodes.contains(&t.device_id))
    );
    assert_eq!(copy.project.resources, p.resources);
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_TOPOLOGY") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("j7-topology-copy.json"),
            serde_json::to_vec_pretty(&copy.project).unwrap(),
        )
        .unwrap();
        std::fs::write(dir.join("j7-topology-copy-ping.jsonl"), e.canonical_trace()).unwrap();
        std::fs::write(
            dir.join("j7-topology-conflicts.json"),
            serde_json::to_vec_pretty(&copy.potential_conflicts).unwrap(),
        )
        .unwrap();
    }
    let mut editor = Editor::new(p).unwrap();
    let before = editor.project().clone();
    let outcome = editor
        .apply(
            0,
            Edit::Duplicate {
                selection,
                dx: 600.,
                dy: 40.,
            },
        )
        .unwrap();
    assert_eq!(outcome.created, copy.created);
    let after = editor.project().clone();
    editor.undo();
    assert_eq!(editor.project(), &before);
    editor.redo();
    assert_eq!(editor.project(), &after);
    let reopened: Project = serde_json::from_slice(&serde_json::to_vec(&after).unwrap()).unwrap();
    reopened.validate().unwrap();
    assert_eq!(reopened, after);
}
#[test]
fn gateway_routes_dhcp_reservations_and_wireless_selection_remap_to_the_copies() {
    let mut p = project("private");
    let mac = p.device(&id("host-a")).unwrap().interfaces[0].mac.clone();
    p.devices
        .iter_mut()
        .find(|d| d.id == id("gateway"))
        .unwrap()
        .network
        .dhcp_server = Some(Box::new(DhcpConfig {
        interface: id("gateway-lan"),
        pool_start: "192.168.10.20".parse().unwrap(),
        pool_end: "192.168.10.30".parse().unwrap(),
        prefix_length: 24,
        gateway: Some("192.168.10.1".parse().unwrap()),
        dns: None,
        use_host_options: false,
        reservations: std::collections::BTreeMap::from([(
            mac.clone(),
            "192.168.10.22".parse().unwrap(),
        )]),
        lease_us: Decimal(30_000_000),
    }));
    p.validate().unwrap();
    let copied = duplicate_selection(
        &p,
        &BTreeSet::from([id("gateway"), id("host-a")]),
        500.,
        40.,
    )
    .unwrap();
    let gateway = copied
        .project
        .device(&copied.identities[&id("gateway")])
        .unwrap();
    let host = copied
        .project
        .device(&copied.identities[&id("host-a")])
        .unwrap();
    let roles = gateway.network.gateway.as_ref().unwrap();
    assert_eq!(roles.lan, copied.identities[&id("gateway-lan")]);
    assert_eq!(roles.wan, copied.identities[&id("gateway-wan")]);
    for route in &gateway.network.routes {
        assert!(gateway.interfaces.iter().any(|i| i.id == route.interface));
    }
    let dhcp = gateway.network.dhcp_server.as_ref().unwrap();
    assert_eq!(dhcp.interface, roles.lan);
    assert_eq!(
        dhcp.reservations[&host.interfaces[0].mac],
        "192.168.10.22".parse::<std::net::Ipv4Addr>().unwrap()
    );
    assert_ne!(host.interfaces[0].mac, mac);
    let mut p = project("wireless");
    let ssid = "é".repeat(16);
    p.devices
        .iter_mut()
        .find(|d| d.id == id("switch"))
        .unwrap()
        .network
        .ssid = Some(ssid.clone());
    for d in &mut p.devices {
        for s in d.network.wireless.values_mut() {
            *s = ssid.clone();
        }
    }
    p.validate().unwrap();
    let copy =
        duplicate_selection(&p, &BTreeSet::from([id("switch"), id("host-b")]), 500., 40.).unwrap();
    let ap = copy
        .project
        .device(&copy.identities[&id("switch")])
        .unwrap();
    let host = copy
        .project
        .device(&copy.identities[&id("host-b")])
        .unwrap();
    let new_ssid = ap.network.ssid.as_ref().unwrap();
    assert_ne!(new_ssid, &ssid);
    assert!(ssid_valid(new_ssid));
    assert!(host.network.wireless.values().all(|s| s == new_ssid));
    assert!(copy.project.wireless_links().iter().any(|(client, radio)| {
        host.interfaces.iter().any(|i| &i.id == client)
            && ap.interfaces.iter().any(|i| &i.id == radio)
    }));
    assert_eq!(
        copy.project
            .device(&id("switch"))
            .unwrap()
            .network
            .ssid
            .as_ref(),
        Some(&ssid)
    );
}
#[test]
fn mail_files_and_unicode_labels_survive_with_fresh_consistent_message_identities() {
    let mut p = project("mail");
    let message = MailMessage {
        sender_name: "École".into(),
        relay_path: vec![],
        id: id("existing-message"),
        from: "teacher@alpha.test".into(),
        to: vec!["alice@alpha.test".into()],
        cc: vec![],
        bcc: vec![],
        subject: "Exercice".into(),
        body: "Bonjour aux élèves".into(),
        received_us: Decimal(123),
    };
    let client_id = p
        .devices
        .iter()
        .find(|d| d.applications.iter().any(|a| a.kind == "mail_client"))
        .unwrap()
        .id
        .clone();
    let server_id = p
        .devices
        .iter()
        .find(|d| d.applications.iter().any(|a| a.kind == "mail_server"))
        .unwrap()
        .id
        .clone();
    let client = p.devices.iter_mut().find(|d| d.id == client_id).unwrap();
    client.label = "é".repeat(2048);
    client.files.push(VirtualFile {
        path: "/notes.txt".into(),
        media_type: "text/plain".into(),
        data_base64: "Y2xhc3NlCg==".into(),
    });
    client
        .applications
        .iter_mut()
        .find(|a| a.kind == "mail_client")
        .unwrap()
        .data = Some(ApplicationData::MailClient(MailClientData {
        next_id: Decimal(10),
        inbox: vec![message.clone()],
        sent: vec![],
        contacts: vec![Contact {
            id: "original-contact".into(),
            email: "teacher@alpha.test".into(),
            ..Default::default()
        }],
    }));
    let server = p.devices.iter_mut().find(|d| d.id == server_id).unwrap();
    let app = server
        .applications
        .iter_mut()
        .find(|a| a.kind == "mail_server")
        .unwrap();
    let Some(ServiceConfig::MailServer(config)) = &app.service else {
        panic!()
    };
    let user = config.accounts[0].username.clone();
    app.data = Some(ApplicationData::MailServer(MailServerData {
        mailboxes: std::collections::BTreeMap::from([(user, vec![message.clone()])]),
    }));
    p.validate().unwrap();
    let copy = duplicate_selection(
        &p,
        &BTreeSet::from([client_id.clone(), server_id.clone()]),
        500.,
        40.,
    )
    .unwrap();
    let client = copy.project.device(&copy.identities[&client_id]).unwrap();
    assert_eq!(client.files, p.device(&client_id).unwrap().files);
    assert!(client.label.len() <= 4096);
    let Some(ApplicationData::MailClient(data)) = &client
        .applications
        .iter()
        .find(|a| a.kind == "mail_client")
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(data.inbox[0].body, message.body);
    assert_ne!(data.inbox[0].id, message.id);
    assert_ne!(data.contacts[0].id, "original-contact");
    let server = copy.project.device(&copy.identities[&server_id]).unwrap();
    let Some(ApplicationData::MailServer(server)) = &server
        .applications
        .iter()
        .find(|a| a.kind == "mail_server")
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(
        server.mailboxes.values().next().unwrap()[0].id,
        data.inbox[0].id
    );
}
#[test]
fn every_alignment_is_reversible_and_preserves_all_network_configuration() {
    let selected = BTreeSet::from([id("host-a"), id("host-b"), id("host-c")]);
    for alignment in Alignment::ALL {
        let mut e = Editor::new(project("lan")).unwrap();
        let before = e.project().clone();
        e.apply(
            0,
            Edit::Align {
                selection: selected.clone(),
                alignment,
            },
        )
        .unwrap();
        let after = e.project().clone();
        assert_eq!(after.devices, before.devices);
        assert_eq!(after.links, before.links);
        assert_eq!(after.lan, before.lan);
        let points: Vec<_> = after
            .layout
            .devices
            .iter()
            .filter(|p| selected.contains(&p.device_id))
            .collect();
        let horizontal = matches!(
            alignment,
            Alignment::Left
                | Alignment::Right
                | Alignment::HorizontalCenter
                | Alignment::DistributeHorizontal
        );
        let mut values: Vec<_> = points
            .iter()
            .map(|p| if horizontal { p.x } else { p.y })
            .collect();
        values.sort_by(f32::total_cmp);
        if matches!(
            alignment,
            Alignment::DistributeHorizontal | Alignment::DistributeVertical
        ) {
            assert_eq!(values[1] - values[0], values[2] - values[1]);
        } else {
            assert!(values.iter().all(|v| *v == values[0]));
        }
        e.undo();
        assert_eq!(e.project(), &before);
        e.redo();
        assert_eq!(e.project(), &after);
    }
    let mut e = Editor::new(project("lan")).unwrap();
    e.apply(
        0,
        Edit::Align {
            selection: BTreeSet::from([id("host-a"), id("host-b")]),
            alignment: Alignment::Top,
        },
    )
    .unwrap();
    assert_eq!(e.revision(), 0);
    assert!(!e.can_undo());
}
#[test]
fn invalid_selection_offsets_and_document_quota_refuse_the_whole_edit() {
    let mut e = Editor::new(project("lan")).unwrap();
    let before = e.project().clone();
    for edit in [
        Edit::Duplicate {
            selection: BTreeSet::from([id("cable-host-a")]),
            dx: 40.,
            dy: 40.,
        },
        Edit::Duplicate {
            selection: BTreeSet::from([id("host-a")]),
            dx: f32::NAN,
            dy: 40.,
        },
        Edit::Duplicate {
            selection: BTreeSet::from([id("missing")]),
            dx: 40.,
            dy: 40.,
        },
        Edit::Align {
            selection: BTreeSet::from([id("host-a")]),
            alignment: Alignment::Left,
        },
    ] {
        assert!(e.apply(e.revision(), edit).is_err());
        assert_eq!(e.project(), &before);
        assert_eq!(e.revision(), 0);
    }
    let mut p = project("lan");
    p.devices[0].files = (0..5)
        .map(|n| VirtualFile {
            path: format!("/large-{n}.bin"),
            media_type: "application/octet-stream".into(),
            data_base64: "A".repeat(1_000_000),
        })
        .collect();
    assert!(serde_json::to_vec(&p).unwrap().len() < MAX_PROJECT_DOCUMENT_BYTES);
    let mut e = Editor::new(p).unwrap();
    let before = e.project().clone();
    assert_eq!(
        e.apply(
            0,
            Edit::Duplicate {
                selection: BTreeSet::from([id("host-a")]),
                dx: 500.,
                dy: 40.
            }
        )
        .err()
        .unwrap()
        .code,
        "resource.limit_exceeded"
    );
    assert_eq!(e.project(), &before);
    assert!(!e.can_undo());
}
