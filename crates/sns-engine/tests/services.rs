//! J4 recipes use the production event loop and actual simulated LAN sockets.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sns_engine::{Engine, Limits};
use sns_model::*;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn project() -> Project {
    serde_json::from_str(include_str!("../../../fixtures/valid/services.json")).unwrap()
}
fn engine(p: Project) -> Engine {
    Engine::from_project(p, 42, Limits::default()).unwrap()
}
fn app(e: &mut Engine, h: &str, c: AppCommand) {
    let n = e.trace().len();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(&format!("j4-{}-{n}", e.revision())),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::App {
            device: id(h),
            command: c,
        },
    })
    .unwrap();
}
fn run(e: &mut Engine, delta: u64) {
    e.run(100_000, Some(e.now() + delta)).unwrap();
}
fn start(e: &mut Engine) {
    for a in ["dns", "web"] {
        app(
            e,
            "host-b",
            AppCommand::StartServer {
                application: id(a),
                port: 80,
            },
        );
    }
    run(e, 1_000_000);
}
fn browse(
    e: &mut Engine,
    h: &str,
    url: &str,
    fields: Option<std::collections::BTreeMap<String, String>>,
) {
    app(
        e,
        h,
        AppCommand::Navigate {
            application: id(&format!("{h}-browser")),
            url: url.into(),
            fields,
        },
    );
    run(e, 1_000_000);
}
fn view(e: &Engine, h: &str) -> sns_apps::BrowserView {
    e.apps()
        .views(e.project())
        .into_iter()
        .find(|a| a.device == id(h))
        .unwrap()
        .applications
        .into_iter()
        .find(|a| a.software == "web_browser")
        .unwrap()
        .browser
}
fn count(e: &Engine, kind: &str) -> usize {
    e.trace().iter().filter(|f| f.kind == kind).count()
}
fn query(e: &mut Engine, h: &str, line: &str) {
    app(
        e,
        h,
        AppCommand::Terminal {
            application: id(&format!("{h}-terminal")),
            line: line.into(),
        },
    );
    run(e, 500_000);
}
fn record(e: &Engine, name: &str) {
    if let Some(dir) = std::env::var_os("SNS_RECORD_J4") {
        let root = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(format!("j4-{name}.jsonl")), e.canonical_trace()).unwrap();
        std::fs::write(
            root.join(format!("j4-{name}-project.json")),
            serde_json::to_vec_pretty(e.project()).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn r_dhcp_two_clients_reservation_effective_settings_and_document() {
    let mut e = engine(project());
    run(&mut e, 1_000_000);
    let p = e.network().effective_project(e.project());
    assert_eq!(e.network().dhcp.assigned.len(), 2);
    assert_eq!(
        p.devices[0].interfaces[0].ipv4,
        Some(Ipv4Config::Static {
            address: "192.168.1.10".into(),
            prefix_length: 24
        })
    );
    assert_eq!(
        p.devices[2].network.dns_server.as_deref(),
        Some("192.168.1.3")
    );
    assert_eq!(
        p.devices[2].network.default_gateway.as_deref(),
        Some("192.168.1.1")
    );
    assert_eq!(
        e.project().devices[0].interfaces[0].ipv4,
        Some(Ipv4Config::Dhcp)
    );
    for k in [
        "dhcp.discover",
        "dhcp.offer",
        "dhcp.request",
        "dhcp.ack",
        "dhcp.lease.acquired",
    ] {
        assert_eq!(count(&e, k), 2, "{k}");
    }
    query(&mut e, "host-a", "ipconfig");
    record(&e, "dhcp");
}
#[test]
fn r_web_offline_dns_http_images_forms_virtual_hosts_and_isolation() {
    let mut e = engine(project());
    start(&mut e);
    browse(&mut e, "host-a", "http://www.ecole.test/", None);
    let a = view(&e, "host-a");
    assert_eq!(a.status, 200);
    assert!(a.html.contains("École"));
    assert_eq!(a.images.len(), 1);
    assert_eq!(count(&e, "dns.query.sent"), 1);
    assert!(count(&e, "tcp.segment.sent") > 0);
    let image = &a.images.values().next().unwrap().data_base64;
    assert!(STANDARD.decode(image).unwrap().starts_with(b"\x89PNG"));
    browse(&mut e, "host-c", "http://autre.ecole.test/", None);
    assert!(view(&e, "host-c").html.contains("Autre classe"));
    assert!(view(&e, "host-c").images.is_empty());
    assert_eq!(view(&e, "host-a").images.len(), 1);
    browse(
        &mut e,
        "host-a",
        "/submit",
        Some([("name".into(), "<script>& élève".into())].into()),
    );
    assert_eq!(view(&e, "host-a").status, 200);
    assert!(
        view(&e, "host-a")
            .html
            .contains("&lt;script&gt;&amp; élève")
    );
    browse(&mut e, "host-a", "/missing", None);
    assert_eq!(view(&e, "host-a").status, 404);
    record(&e, "web");
}
#[test]
fn dns_ttl_cache_is_per_host_and_delete_survives_reload() {
    let mut p = project();
    if let Some(ServiceConfig::Dns(c)) = &mut p.devices[1].applications[0].service {
        for r in &mut c.records {
            r.ttl_seconds = 2;
        }
    }
    let mut e = engine(p);
    start(&mut e);
    query(&mut e, "host-a", "host WWW.ECOLE.TEST.");
    query(&mut e, "host-a", "host www.ecole.test");
    assert_eq!(count(&e, "dns.query.sent"), 1);
    query(&mut e, "host-c", "nslookup www.ecole.test");
    assert_eq!(count(&e, "dns.query.sent"), 2);
    let mut cfg = match e.project().devices[1].applications[0]
        .service
        .clone()
        .unwrap()
    {
        ServiceConfig::Dns(c) => c,
        _ => unreachable!(),
    };
    cfg.records.retain(|r| r.name != "www.ecole.test");
    app(
        &mut e,
        "host-b",
        AppCommand::ConfigureService {
            application: id("dns"),
            config: ServiceConfig::Dns(cfg),
        },
    );
    run(&mut e, 3_000_000);
    query(&mut e, "host-a", "host www.ecole.test");
    assert_eq!(count(&e, "dns.query.sent"), 3);
    assert!(
        String::from_utf8(e.canonical_trace())
            .unwrap()
            .contains("nxdomain")
    );
    let saved = serde_json::to_string(e.project()).unwrap();
    let reloaded: Project = serde_json::from_str(&saved).unwrap();
    assert!(
        !serde_json::to_string(&reloaded.devices[1].applications[0])
            .unwrap()
            .contains("\"name\":\"www.ecole.test\"")
    );
}
#[test]
fn dhcp_conflict_decline_pool_exhaustion_and_link_local() {
    let mut p = project();
    p.devices[2].interfaces[0].ipv4 = Some(Ipv4Config::Static {
        address: "192.168.1.10".into(),
        prefix_length: 24,
    });
    let cfg = p.devices[1].network.dhcp_server.as_mut().unwrap();
    cfg.reservations.clear();
    cfg.pool_end = cfg.pool_start;
    let mut e = engine(p);
    run(&mut e, 10_000_000);
    assert!(count(&e, "dhcp.decline") > 0);
    assert!(count(&e, "dhcp.pool.exhausted") > 0);
    assert!(e.network().dhcp.assigned.values().all(|a| a.link_local));
    record(&e, "conflict");
}
#[test]
fn dhcp_expiry_releases_and_reacquires_virtual_lease() {
    let mut p = project();
    p.devices[1].network.dhcp_server.as_mut().unwrap().lease_us = Decimal(1_000_000);
    let mut e = engine(p);
    run(&mut e, 2_000_000);
    assert!(count(&e, "dhcp.lease.expired") >= 2);
    assert!(count(&e, "dhcp.lease.acquired") >= 4);
    assert_eq!(
        e.project().devices[0].interfaces[0].ipv4,
        Some(Ipv4Config::Dhcp)
    );
}
#[test]
fn dns_mx_nodata_timeout_and_explicit_purge() {
    let mut e = engine(project());
    start(&mut e);
    query(&mut e, "host-a", "host -t MX ecole.test");
    assert!(
        String::from_utf8(e.canonical_trace())
            .unwrap()
            .contains("\"priority\":10")
    );
    query(&mut e, "host-a", "host -t MX www.ecole.test");
    assert!(
        String::from_utf8(e.canonical_trace())
            .unwrap()
            .contains("nodata")
    );
    query(&mut e, "host-a", "dnscache -d");
    app(
        &mut e,
        "host-b",
        AppCommand::Stop {
            application: id("dns"),
        },
    );
    query(&mut e, "host-a", "nslookup www.ecole.test");
    run(&mut e, 4_000_000);
    assert_eq!(count(&e, "dns.timeout"), 1);
}
#[test]
fn dns_parent_delegation_recursive_and_iterative_queries_and_bounded_loop() {
    for recursive in [false, true] {
        let mut p = project();
        p.devices[2].interfaces[0].ipv4 = Some(Ipv4Config::Static {
            address: "192.168.1.4".into(),
            prefix_length: 24,
        });
        let ns = DnsRecord {
            name: "remote.test".into(),
            record_type: DnsType::Ns,
            value: "ns.remote.test".into(),
            ttl_seconds: 30,
            priority: 0,
        };
        let glue = DnsRecord {
            name: "ns.remote.test".into(),
            record_type: DnsType::A,
            value: "192.168.1.4".into(),
            ttl_seconds: 30,
            priority: 0,
        };
        p.devices[1].applications[0].service = Some(ServiceConfig::Dns(DnsConfig {
            records: vec![ns, glue],
            recursive,
            upstream: None,
        }));
        p.devices[2].applications.push(Application {
            data: None,
            id: id("dns-child"),
            kind: "dns_server".into(),
            config: Default::default(),
            service: Some(ServiceConfig::Dns(DnsConfig {
                records: vec![DnsRecord {
                    name: "www.remote.test".into(),
                    record_type: DnsType::A,
                    value: "192.168.1.3".into(),
                    ttl_seconds: 30,
                    priority: 0,
                }],
                recursive: true,
                upstream: None,
            })),
        });
        let mut e = engine(p);
        start(&mut e);
        app(
            &mut e,
            "host-c",
            AppCommand::StartServer {
                application: id("dns-child"),
                port: 53,
            },
        );
        query(&mut e, "host-a", "host www.remote.test");
        assert_eq!(count(&e, "dns.query.sent"), 2);
        assert!(
            e.trace()
                .iter()
                .any(|f| f.kind == "dns.response.sent" && f.device_id == id("host-c"))
        );
        assert!(
            String::from_utf8(e.canonical_trace())
                .unwrap()
                .contains("www.remote.test")
        );
        // A parent/child cycle must return a finite loop result without filling the queue.
        app(
            &mut e,
            "host-c",
            AppCommand::ConfigureService {
                application: id("dns-child"),
                config: ServiceConfig::Dns(DnsConfig {
                    records: vec![],
                    recursive: true,
                    upstream: Some("192.168.1.3".parse().unwrap()),
                }),
            },
        );
        query(&mut e, "host-a", "host missing.remote.test");
        run(&mut e, 10_000_000);
        assert!(
            String::from_utf8(e.canonical_trace())
                .unwrap()
                .contains("\"code\":\"loop\"")
        );
        assert!(count(&e, "dns.query.sent") < 12);
        record(&e, if recursive { "recursive" } else { "iterative" });
    }
}
#[test]
fn link_local_conflicts_have_a_bounded_final_failure() {
    let mut p = project();
    p.devices[1].network.dhcp_server = None;
    let mac = p.devices[0].interfaces[0].mac.to_ascii_lowercase();
    let hash = mac
        .bytes()
        .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
    for k in 0..3 {
        let n = (hash + k) % 65024 + 256;
        let mut host = p.devices[2].clone();
        host.id = id(&format!("conflict-{k}"));
        host.label = format!("Conflict {k}");
        host.applications.clear();
        host.interfaces[0].id = id(&format!("conflict-{k}-p0"));
        host.interfaces[0].mac = format!("02:00:00:04:00:0{k}");
        host.interfaces[0].ipv4 = Some(Ipv4Config::Static {
            address: format!("169.254.{}.{}", n >> 8, n as u8),
            prefix_length: 16,
        });
        p.links.push(Link {
            id: id(&format!("conflict-link-{k}")),
            endpoints: [
                host.interfaces[0].id.clone(),
                id(&format!("switch-p{}", k + 3)),
            ],
            latency_us: Decimal(1000),
            jitter_us: Decimal(0),
        });
        p.devices.push(host);
    }
    let mut e = engine(p);
    run(&mut e, 12_000_000);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "dhcp.failed" && f.device_id == id("host-a"))
    );
    assert!(!e.network().dhcp.assigned.contains_key(&id("host-a-p0")));
    record(&e, "link-local-failure");
}
#[test]
fn same_named_images_from_two_origins_and_navigation_are_isolated() {
    let mut p = project();
    p.devices[2].interfaces[0].ipv4 = Some(Ipv4Config::Static {
        address: "192.168.1.4".into(),
        prefix_length: 24,
    });
    p.devices[2].network.dns_server = Some("192.168.1.3".into());
    p.devices[2].applications.push(Application {
        data: None,
        id: id("web-c"),
        kind: "web_server".into(),
        config: Default::default(),
        service: Some(ServiceConfig::Http(HttpConfig::default())),
    });
    let image = p.devices[1]
        .files
        .iter()
        .find(|f| f.path.ends_with(".png"))
        .unwrap()
        .clone();
    p.devices[2].files=vec![image,VirtualFile{path:"/www/index.html".into(),media_type:"text/html".into(),data_base64:STANDARD.encode(b"<h1>Second server</h1><img src=\"/logo.png\" alt=\"Local\"><img src=\"http://192.168.1.3/logo.png\" alt=\"Remote\">")}];
    let mut e = engine(p);
    start(&mut e);
    app(
        &mut e,
        "host-c",
        AppCommand::StartServer {
            application: id("web-c"),
            port: 80,
        },
    );
    browse(&mut e, "host-a", "http://192.168.1.4/", None);
    assert_eq!(view(&e, "host-a").images.len(), 2);
    assert!(view(&e, "host-c").images.is_empty());
    browse(&mut e, "host-c", "http://192.168.1.3/", None);
    assert_eq!(view(&e, "host-c").images.len(), 1);
    browse(&mut e, "host-a", "http://192.168.1.3/suite.html", None);
    assert!(view(&e, "host-a").images.is_empty());
    assert_eq!(view(&e, "host-c").images.len(), 1);
}
#[test]
fn http_methods_encoded_paths_segmented_body_and_stop_with_pending_queries() {
    let mut e = engine(project());
    start(&mut e);
    app(
        &mut e,
        "host-a",
        AppCommand::Install {
            application: id("raw"),
            software: Software::TcpClient,
        },
    );
    app(
        &mut e,
        "host-a",
        AppCommand::Connect {
            application: id("raw"),
            target: "192.168.1.3".into(),
            port: 80,
        },
    );
    run(&mut e, 500_000);
    app(
        &mut e,
        "host-a",
        AppCommand::Send {
            application: id("raw"),
            text: "DELETE / HTTP/1.1\r\nHost: www.ecole.test\r\nContent-Length: 0\r\n\r\n".into(),
        },
    );
    run(&mut e, 500_000);
    let raw = e
        .apps()
        .views(e.project())
        .into_iter()
        .find(|h| h.device == id("host-a"))
        .unwrap()
        .applications
        .into_iter()
        .find(|a| a.id == id("raw"))
        .unwrap();
    assert!(
        STANDARD
            .decode(raw.received_base64)
            .unwrap()
            .starts_with(b"HTTP/1.1 501")
    );
    browse(&mut e, "host-a", "http://192.168.1.3/%2e%2e/secret", None);
    assert_eq!(view(&e, "host-a").status, 403);
    let large = "é".repeat(1200);
    browse(
        &mut e,
        "host-a",
        "http://www.ecole.test/submit",
        Some([("message".into(), large.clone())].into()),
    );
    assert!(view(&e, "host-a").html.contains(&large));
    query(&mut e, "host-a", "nslookup missing.test 192.168.1.99");
    app(
        &mut e,
        "host-a",
        AppCommand::Stop {
            application: id("host-a-terminal"),
        },
    );
    run(&mut e, 20_000_000);
    assert!(
        !e.network()
            .transport
            .views()
            .iter()
            .any(|s| s.owner == id("host-a-terminal"))
    );
}
#[test]
fn dns_and_http_cross_routed_networks_with_dhcp_options_inherited() {
    let mut p: Project =
        serde_json::from_str(include_str!("../../../fixtures/valid/routed.json")).unwrap();
    p.schema_version = 5;
    p.semantic_profile = "sns-educational-services-v1".into();
    let service = project().devices[1].clone();
    p.devices[0].network.dns_server = Some("192.168.30.2".into());
    p.devices[0].applications = project().devices[0].applications.clone();
    p.devices[3].files = service.files;
    p.devices[3].applications = service.applications;
    if let Some(ServiceConfig::Dns(c)) = &mut p.devices[3].applications[0].service {
        for r in &mut c.records {
            if r.record_type == DnsType::A {
                r.value = "192.168.30.2".into();
            }
        }
    }
    let mut e = engine(p);
    for a in ["dns", "web"] {
        app(
            &mut e,
            "b",
            AppCommand::StartServer {
                application: id(a),
                port: 80,
            },
        );
    }
    app(
        &mut e,
        "a",
        AppCommand::Navigate {
            application: id("host-a-browser"),
            url: "http://www.ecole.test/".into(),
            fields: None,
        },
    );
    run(&mut e, 2_000_000);
    let view = e
        .apps()
        .views(e.project())
        .into_iter()
        .find(|h| h.device == id("a"))
        .unwrap()
        .applications
        .into_iter()
        .find(|a| a.software == "web_browser")
        .unwrap()
        .browser;
    assert_eq!(view.status, 200);
    assert_eq!(view.images.len(), 1);
    assert!(count(&e, "ipv4.forwarded") > 10);
    let mut p = project();
    let c = p.devices[1].network.dhcp_server.as_mut().unwrap();
    c.gateway = None;
    c.dns = None;
    c.use_host_options = true;
    p.devices[1].network.default_gateway = Some("192.168.1.1".into());
    p.devices[1].network.dns_server = Some("192.168.1.3".into());
    let mut e = engine(p);
    run(&mut e, 1_000_000);
    let a = e.network().dhcp.assigned.values().next().unwrap();
    assert_eq!(a.gateway, Some("192.168.1.1".parse().unwrap()));
    assert_eq!(a.dns, Some("192.168.1.3".parse().unwrap()));
}
#[test]
fn dhcp_nack_unoffered_address_and_expiration_of_refused_entries() {
    let mut p = project();
    p.devices[2].interfaces[0].ipv4 = Some(Ipv4Config::Static {
        address: "192.168.1.10".into(),
        prefix_length: 24,
    });
    p.devices[1]
        .network
        .dhcp_server
        .as_mut()
        .unwrap()
        .reservations
        .clear();
    let mut e = engine(p);
    run(&mut e, 1_000_000);
    let commands=[SocketCommand::Bind{socket:id("dhcp-probe"),owner:id("dhcp-probe"),protocol:TransportProtocol::Udp,interface:None,address:None,port:68},SocketCommand::SendTo{socket:id("dhcp-probe"),destination:"255.255.255.255".parse().unwrap(),port:67,data_base64:STANDARD.encode(serde_json::to_vec(&serde_json::json!({"kind":"request","mac":"02:00:00:00:02:01","xid":99,"server":"192.168.1.3","address":"192.168.1.42","prefix":24,"gateway":null,"dns":null,"lease":1000000})).unwrap())}];
    for (i, command) in commands.into_iter().enumerate() {
        e.command(CommandEnvelope {
            api_version: 1,
            command_id: id(&format!("negative-dhcp-{i}")),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::Socket {
                device: id("host-c"),
                command,
            },
        })
        .unwrap();
    }
    run(&mut e, 500_000);
    assert_eq!(count(&e, "dhcp.nack"), 1);
    assert!(count(&e, "dhcp.decline") >= 1);
    run(&mut e, 35_000_000);
    assert!(count(&e, "dhcp.entries.expired") >= 1);
}
