//! R-NAT/R-FILTRE run on the real virtual LAN/WAN stack.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sns_engine::{Engine, Limits};
use sns_model::*;
fn record(e: &Engine, name: &str) {
    if let Some(directory) = std::env::var_os("SNS_RECORD_J5") {
        let root = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(format!("j5-{name}.jsonl")), e.canonical_trace()).unwrap();
        std::fs::write(
            root.join(format!("j5-{name}-project.json")),
            serde_json::to_vec_pretty(e.project()).unwrap(),
        )
        .unwrap();
    }
}
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn project() -> Project {
    serde_json::from_str(include_str!("../../../fixtures/valid/private.json")).unwrap()
}
thread_local! {static SERIAL:std::cell::Cell<u64>=const{std::cell::Cell::new(0)};}
fn engine(p: Project) -> Engine {
    SERIAL.with(|s| s.set(0));
    Engine::from_project(p, 42, Limits::default()).unwrap()
}
fn command(e: &mut Engine, command: Command) -> Result<(), Diagnostic> {
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(&format!(
            "j5-{}",
            SERIAL.with(|s| {
                let n = s.get();
                s.set(n + 1);
                n
            })
        )),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command,
    })
}
fn app(e: &mut Engine, host: &str, c: AppCommand) {
    command(
        e,
        Command::App {
            device: id(host),
            command: c,
        },
    )
    .unwrap();
}
fn socket(e: &mut Engine, host: &str, c: SocketCommand) {
    command(
        e,
        Command::Socket {
            device: id(host),
            command: c,
        },
    )
    .unwrap();
}
fn run(e: &mut Engine, delta: u64) {
    e.run(100_000, Some(e.now() + delta)).unwrap();
}
fn count(e: &Engine, kind: &str) -> usize {
    e.trace().iter().filter(|f| f.kind == kind).count()
}
fn received(e: &Engine, host: &str, app: &str) -> Vec<u8> {
    STANDARD
        .decode(
            &e.apps()
                .views(e.project())
                .into_iter()
                .find(|h| h.device == id(host))
                .unwrap()
                .applications
                .into_iter()
                .find(|a| a.id == id(app))
                .unwrap()
                .received_base64,
        )
        .unwrap()
}
fn start_echo(e: &mut Engine) {
    app(
        e,
        "host-c",
        AppCommand::StartServer {
            application: id("echo"),
            port: 7777,
        },
    );
}
fn clients(e: &mut Engine) {
    for (h, a) in [("host-a", "client-a"), ("host-b", "client-b")] {
        app(
            e,
            h,
            AppCommand::Install {
                application: id(a),
                software: Software::TcpClient,
            },
        );
        socket(
            e,
            h,
            SocketCommand::Connect {
                socket: id(a),
                owner: id(a),
                destination: "198.51.100.10".parse().unwrap(),
                port: 7777,
                local_address: None,
                local_port: Some(40000),
            },
        );
    }
}
#[test]
fn nat_tcp_same_ports_restore_to_correct_hosts_and_keep_original_trace() {
    let mut e = engine(project());
    start_echo(&mut e);
    clients(&mut e);
    run(&mut e, 1_000_000);
    assert_eq!(e.network().nat.entries.len(), 2);
    let ports: std::collections::BTreeSet<_> = e
        .network()
        .nat
        .entries
        .values()
        .map(|m| serde_json::to_string(&m.external).unwrap())
        .collect();
    assert_eq!(ports.len(), 2);
    let before = e.canonical_trace();
    for (h, a, text) in [
        ("host-a", "client-a", "Bonjour A"),
        ("host-b", "client-b", "Bonjour B"),
    ] {
        app(
            &mut e,
            h,
            AppCommand::Send {
                application: id(a),
                text: text.into(),
            },
        );
    }
    run(&mut e, 1_000_000);
    assert_eq!(received(&e, "host-a", "client-a"), b"Bonjour A");
    assert_eq!(received(&e, "host-b", "client-b"), b"Bonjour B");
    assert!(e.canonical_trace().starts_with(&before));
    assert!(count(&e, "nat.translated") >= 8);
    record(&e, "nat-tcp");
}
#[test]
fn nat_udp_same_ports_isolate_reply_endpoints() {
    let mut e = engine(project());
    for h in ["host-a", "host-b", "host-c"] {
        socket(
            &mut e,
            h,
            SocketCommand::Bind {
                socket: id(h),
                owner: id(h),
                protocol: TransportProtocol::Udp,
                interface: None,
                address: None,
                port: if h == "host-c" { 9000 } else { 40000 },
            },
        );
    }
    for h in ["host-a", "host-b"] {
        socket(
            &mut e,
            h,
            SocketCommand::SendTo {
                socket: id(h),
                destination: "198.51.100.10".parse().unwrap(),
                port: 9000,
                data_base64: STANDARD.encode(h),
            },
        );
    }
    run(&mut e, 1_000_000);
    let mappings: Vec<_> = e.network().nat.entries.values().cloned().collect();
    assert_eq!(mappings.len(), 2);
    for m in mappings {
        let sns_protocols::nat::Token::Port(port) = m.external.token else {
            panic!()
        };
        socket(
            &mut e,
            "host-c",
            SocketCommand::SendTo {
                socket: id("host-c"),
                destination: m.external.address,
                port,
                data_base64: STANDARD.encode(m.internal.address.to_string()),
            },
        );
    }
    run(&mut e, 1_000_000);
    for (h, ip) in [("host-a", "192.168.10.2"), ("host-b", "192.168.10.3")] {
        assert!(e.trace().iter().any(|f| f.kind == "socket.event"
            && f.device_id == id(h)
            && f.payload["kind"]["data_base64"] == STANDARD.encode(ip)));
    }
}
#[test]
fn static_forward_and_icmp_traceroute_quotes_traverse_nat() {
    let mut e = engine(project());
    app(
        &mut e,
        "host-a",
        AppCommand::StartServer {
            application: id("lan-web"),
            port: 80,
        },
    );
    app(
        &mut e,
        "host-c",
        AppCommand::Install {
            application: id("wan-browser"),
            software: Software::WebBrowser,
        },
    );
    app(
        &mut e,
        "host-c",
        AppCommand::Navigate {
            application: id("wan-browser"),
            url: "http://203.0.113.2:8080/".into(),
            fields: None,
        },
    );
    run(&mut e, 1_000_000);
    assert!(count(&e, "http.response.received") > 0);
    assert!(e.network().nat.entries.values().any(|m| m.forwarded));
    for h in ["host-a", "host-b"] {
        command(
            &mut e,
            Command::Ping {
                source: id(h),
                destination: "198.51.100.10".into(),
                ttl: 64,
            },
        )
        .unwrap();
    }
    run(&mut e, 1_000_000);
    assert_eq!(count(&e, "icmp.echo.reply"), 2);
    for (h, a) in [("host-a", "term-0"), ("host-b", "term-1")] {
        app(
            &mut e,
            h,
            AppCommand::Terminal {
                application: id(a),
                line: "traceroute -m 3 198.51.100.10".into(),
            },
        );
    }
    run(&mut e, 12_000_000);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "nat.translated" && f.payload["direction"] == "inbound_error")
    );
    assert!(count(&e, "icmp.echo.reply") >= 4);
    record(&e, "icmp-forward");
}
#[test]
fn firewall_order_policy_icmp_and_udp_reverse_matching() {
    let mut p = project();
    let rule = FirewallRule {
        source: IpMatch::Subnet {
            network: "192.168.10.0".parse().unwrap(),
            prefix: 24,
        },
        destination: IpMatch::Any,
        protocol: FilterProtocol::Tcp,
        destination_port: Some(7777),
        action: Verdict::Drop,
    };
    p.devices
        .iter_mut()
        .find(|d| d.id == id("gateway"))
        .unwrap()
        .network
        .firewall = Some(Box::new(FirewallConfig {
        rules: vec![
            rule.clone(),
            FirewallRule {
                action: Verdict::Accept,
                ..rule
            },
        ],
        ..Default::default()
    }));
    let mut e = engine(p);
    start_echo(&mut e);
    clients(&mut e);
    run(&mut e, 1_000_000);
    assert_eq!(e.network().nat.entries.len(), 0);
    assert!(
        e.network().firewall.journal[&id("gateway")]
            .iter()
            .any(|d| d.rule == Some(1) && d.action == Verdict::Drop)
    );
    let mut cfg = *e
        .project()
        .device(&id("gateway"))
        .unwrap()
        .network
        .firewall
        .clone()
        .unwrap();
    cfg.rules.swap(0, 1);
    command(
        &mut e,
        Command::Security {
            device: id("gateway"),
            command: SecurityCommand::ConfigureFirewall { config: cfg },
        },
    )
    .unwrap();
    run(&mut e, 2_000_000);
    assert_eq!(e.network().nat.entries.len(), 2);
}
fn browser(
    e: &mut Engine,
    host: &str,
    app_id: &str,
    url: &str,
    fields: Option<std::collections::BTreeMap<String, String>>,
) -> sns_apps::BrowserView {
    app(
        e,
        host,
        AppCommand::Navigate {
            application: id(app_id),
            url: url.into(),
            fields,
        },
    );
    run(e, 700_000);
    e.apps()
        .views(e.project())
        .into_iter()
        .find(|h| h.device == id(host))
        .unwrap()
        .applications
        .into_iter()
        .find(|a| a.id == id(app_id))
        .unwrap()
        .browser
}
#[test]
fn administration_over_real_http_updates_shared_configuration_and_rejects_stale_or_wan_forms() {
    let mut e = engine(project());
    let b = browser(
        &mut e,
        "host-a",
        "browser-0",
        "http://192.168.10.1/admin/firewall?lang=fr",
        None,
    );
    assert_eq!(b.status, 200);
    let doc = sns_html::parse_network(&b.html).unwrap();
    let action = doc
        .blocks
        .iter()
        .find_map(|b| {
            if let sns_html::Block::Form { action, .. } = b {
                action.contains("/accept?").then_some(action.clone())
            } else {
                None
            }
        })
        .unwrap();
    let b = browser(
        &mut e,
        "host-a",
        "browser-0",
        &action,
        Some(Default::default()),
    );
    assert_eq!(b.status, 200);
    assert_eq!(
        e.project()
            .device(&id("gateway"))
            .unwrap()
            .network
            .firewall
            .as_ref()
            .unwrap()
            .default_policy,
        Verdict::Accept
    );
    assert_eq!(
        browser(
            &mut e,
            "host-a",
            "browser-0",
            &action,
            Some(Default::default())
        )
        .status,
        409
    );
    let b = browser(
        &mut e,
        "host-a",
        "browser-0",
        "http://192.168.10.1/admin/routes?lang=de",
        None,
    );
    assert!(b.html.contains("Routen und Weiterleitung"));
    assert!(b.html.contains("203.0.113.1"));
    app(
        &mut e,
        "host-c",
        AppCommand::Install {
            application: id("wan-browser"),
            software: Software::WebBrowser,
        },
    );
    assert_eq!(
        browser(
            &mut e,
            "host-c",
            "wan-browser",
            "http://203.0.113.2/admin",
            None
        )
        .status,
        403
    );
    let b = browser(
        &mut e,
        "host-a",
        "browser-0",
        "http://192.168.10.1/admin/nat",
        None,
    );
    let doc = sns_html::parse_network(&b.html).unwrap();
    let action = doc
        .blocks
        .iter()
        .find_map(|b| {
            if let sns_html::Block::Form { action, .. } = b {
                action.contains("/nat/add?").then_some(action.clone())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        browser(
            &mut e,
            "host-a",
            "browser-0",
            &action,
            Some(
                [
                    ("protocol".into(), "UDP".into()),
                    ("external".into(), "8081".into()),
                    ("address".into(), "192.168.10.3".into()),
                    ("internal".into(), "9000".into())
                ]
                .into()
            )
        )
        .status,
        200
    );
    assert_eq!(
        e.project()
            .device(&id("gateway"))
            .unwrap()
            .network
            .gateway
            .as_ref()
            .unwrap()
            .forwards
            .len(),
        2
    );
}
#[test]
fn wan_dhcp_and_lan_distribution_are_independent_and_expiry_keeps_lan_administration() {
    let mut p = project();
    for h in p
        .devices
        .iter_mut()
        .filter(|h| matches!(h.id.as_str(), "host-a" | "host-b"))
    {
        h.interfaces[0].ipv4 = Some(Ipv4Config::Dhcp);
        h.network.default_gateway = None;
        h.network.dns_server = None;
    }
    let g = p
        .devices
        .iter_mut()
        .find(|d| d.id == id("gateway"))
        .unwrap();
    g.interfaces[1].ipv4 = Some(Ipv4Config::Dhcp);
    g.network.default_gateway = None;
    g.network.dhcp_server = Some(Box::new(DhcpConfig {
        interface: id("gateway-lan"),
        pool_start: "192.168.10.100".parse().unwrap(),
        pool_end: "192.168.10.101".parse().unwrap(),
        prefix_length: 24,
        gateway: Some("192.168.10.1".parse().unwrap()),
        dns: Some("198.51.100.10".parse().unwrap()),
        use_host_options: false,
        reservations: Default::default(),
        lease_us: Decimal(86_400_000_000),
    }));
    p.devices
        .iter_mut()
        .find(|d| d.id == id("uplink"))
        .unwrap()
        .network
        .dhcp_server = Some(Box::new(DhcpConfig {
        interface: id("uplink-a"),
        pool_start: "203.0.113.2".parse().unwrap(),
        pool_end: "203.0.113.3".parse().unwrap(),
        prefix_length: 24,
        gateway: Some("203.0.113.1".parse().unwrap()),
        dns: Some("198.51.100.10".parse().unwrap()),
        use_host_options: false,
        reservations: Default::default(),
        lease_us: Decimal(2_000_000),
    }));
    let mut e = engine(p);
    run(&mut e, 1_000_000);
    let effective = e.network().effective_project(e.project());
    assert_eq!(e.network().dhcp.assigned.len(), 3);
    assert_eq!(
        effective
            .device(&id("gateway"))
            .unwrap()
            .network
            .default_gateway
            .as_deref(),
        Some("203.0.113.1")
    );
    assert_eq!(
        effective
            .device(&id("host-a"))
            .unwrap()
            .network
            .default_gateway
            .as_deref(),
        Some("192.168.10.1")
    );
    start_echo(&mut e);
    clients(&mut e);
    run(&mut e, 700_000);
    assert_eq!(e.network().nat.entries.len(), 2);
    run(&mut e, 3_000_000);
    assert!(count(&e, "dhcp.lease.expired") > 0);
    assert!(
        e.network()
            .transport
            .views()
            .iter()
            .any(|s| s.device == id("gateway") && s.socket == id("admin"))
    );
    assert_eq!(
        browser(
            &mut e,
            "host-a",
            "browser-0",
            "http://192.168.10.1/admin",
            None
        )
        .status,
        200
    );
    record(&e, "wan-dhcp");
}
#[test]
fn mapping_purge_expiry_and_port_exhaustion_are_observable() {
    let mut p = project();
    let g = p
        .devices
        .iter_mut()
        .find(|d| d.id == id("gateway"))
        .unwrap()
        .network
        .gateway
        .as_mut()
        .unwrap();
    g.port_end = g.port_start;
    g.idle_timeout_us = Decimal(1_000_000);
    let mut e = engine(p);
    start_echo(&mut e);
    clients(&mut e);
    run(&mut e, 200_000);
    assert_eq!(e.network().nat.entries.len(), 1);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "nat.dropped" && f.payload["reason"] == "nat.ports_exhausted")
    );
    command(
        &mut e,
        Command::Security {
            device: id("gateway"),
            command: SecurityCommand::PurgeNat,
        },
    )
    .unwrap();
    assert!(e.network().nat.entries.is_empty());
    run(&mut e, 10_000_000);
    assert!(count(&e, "nat.mapping.expired") > 0);
    record(&e, "expiration");
}
#[test]
fn filter_udp_reverse_rule_icmp_switch_and_syn_only_are_explicit() {
    let mut p = project();
    let g = p
        .devices
        .iter_mut()
        .find(|d| d.id == id("gateway"))
        .unwrap();
    g.network.firewall = Some(Box::new(FirewallConfig {
        default_policy: Verdict::Drop,
        rules: vec![FirewallRule {
            source: IpMatch::Any,
            destination: IpMatch::Subnet {
                network: "198.51.100.10".parse().unwrap(),
                prefix: 32,
            },
            protocol: FilterProtocol::Udp,
            destination_port: Some(9000),
            action: Verdict::Accept,
        }],
        drop_icmp: true,
        ..Default::default()
    }));
    let mut e = engine(p);
    socket(
        &mut e,
        "host-a",
        SocketCommand::Bind {
            socket: id("u"),
            owner: id("u"),
            protocol: TransportProtocol::Udp,
            interface: None,
            address: None,
            port: 40000,
        },
    );
    socket(
        &mut e,
        "host-a",
        SocketCommand::SendTo {
            socket: id("u"),
            destination: "198.51.100.10".parse().unwrap(),
            port: 9000,
            data_base64: STANDARD.encode(b"UDP"),
        },
    );
    run(&mut e, 100_000);
    assert_eq!(e.network().nat.entries.len(), 1);
    socket(
        &mut e,
        "host-c",
        SocketCommand::Bind {
            socket: id("u-server"),
            owner: id("u-server"),
            protocol: TransportProtocol::Udp,
            interface: None,
            address: None,
            port: 9000,
        },
    );
    socket(
        &mut e,
        "host-c",
        SocketCommand::SendTo {
            socket: id("u-server"),
            destination: "203.0.113.2".parse().unwrap(),
            port: 40000,
            data_base64: STANDARD.encode(b"reverse"),
        },
    );
    run(&mut e, 100_000);
    assert!(e.trace().iter().any(|f| f.kind == "socket.event"
        && f.device_id == id("host-a")
        && f.payload["kind"]["data_base64"] == STANDARD.encode(b"reverse")));
    command(
        &mut e,
        Command::Ping {
            source: id("host-a"),
            destination: "198.51.100.10".into(),
            ttl: 64,
        },
    )
    .unwrap();
    run(&mut e, 4_000_000);
    assert_eq!(count(&e, "icmp.echo.reply"), 0);
    assert!(
        e.network().firewall.journal[&id("gateway")]
            .iter()
            .any(|d| d.reason == "icmp_policy" && d.action == Verdict::Drop)
    );
    // SYN-only permits return SYN/ACK and ACK segments; all-segment mode applies the default DROP.
    for syn_only in [true, false] {
        let mut p = project();
        p.devices
            .iter_mut()
            .find(|d| d.id == id("gateway"))
            .unwrap()
            .network
            .firewall = Some(Box::new(FirewallConfig {
            syn_only,
            rules: vec![FirewallRule {
                protocol: FilterProtocol::Tcp,
                destination_port: Some(7777),
                action: Verdict::Accept,
                ..Default::default()
            }],
            ..Default::default()
        }));
        let mut e = engine(p);
        start_echo(&mut e);
        clients(&mut e);
        run(&mut e, 1_000_000);
        let established = e
            .network()
            .transport
            .views()
            .into_iter()
            .filter(|s| {
                matches!(s.device.as_str(), "host-a" | "host-b") && s.state == "ESTABLISHED"
            })
            .count();
        assert_eq!(established, if syn_only { 2 } else { 0 });
    }
}
#[test]
fn host_filter_blocks_one_service_and_preserves_http_when_icmp_is_blocked() {
    let mut p = project();
    p.devices
        .iter_mut()
        .find(|d| d.id == id("host-c"))
        .unwrap()
        .network
        .firewall = Some(Box::new(FirewallConfig {
        drop_icmp: true,
        rules: vec![FirewallRule {
            source: IpMatch::Subnet {
                network: "203.0.113.0".parse().unwrap(),
                prefix: 24,
            },
            protocol: FilterProtocol::Tcp,
            destination_port: Some(80),
            action: Verdict::Accept,
            ..Default::default()
        }],
        ..Default::default()
    }));
    let mut e = engine(p);
    start_echo(&mut e);
    app(
        &mut e,
        "host-c",
        AppCommand::StartServer {
            application: id("wan-web"),
            port: 80,
        },
    );
    let page = browser(&mut e, "host-a", "browser-0", "http://198.51.100.10/", None);
    assert_eq!(page.status, 200);
    clients(&mut e);
    command(
        &mut e,
        Command::Ping {
            source: id("host-a"),
            destination: "198.51.100.10".into(),
            ttl: 64,
        },
    )
    .unwrap();
    run(&mut e, 4_000_000);
    assert_eq!(count(&e, "icmp.echo.reply"), 0);
    assert!(
        e.network().firewall.journal[&id("host-c")]
            .iter()
            .any(|d| d.destination_port == Some(7777) && d.action == Verdict::Drop)
    );
    app(
        &mut e,
        "host-c",
        AppCommand::Install {
            application: id("fw-host"),
            software: Software::Firewall,
        },
    );
    app(
        &mut e,
        "host-c",
        AppCommand::Stop {
            application: id("fw-host"),
        },
    );
    assert!(
        !e.project()
            .device(&id("host-c"))
            .unwrap()
            .network
            .firewall
            .as_ref()
            .unwrap()
            .enabled
    );
}
#[test]
fn static_udp_forward_and_wan_private_scope_guard() {
    let mut p = project();
    p.devices
        .iter_mut()
        .find(|d| d.id == id("gateway"))
        .unwrap()
        .network
        .gateway
        .as_mut()
        .unwrap()
        .forwards
        .push(PortForward {
            protocol: TransportProtocol::Udp,
            external_port: 8080,
            internal_address: "192.168.10.3".parse().unwrap(),
            internal_port: 9001,
        });
    let mut e = engine(p);
    for (h, port) in [("host-b", 9001), ("host-c", 40000)] {
        socket(
            &mut e,
            h,
            SocketCommand::Bind {
                socket: id(h),
                owner: id(h),
                protocol: TransportProtocol::Udp,
                interface: None,
                address: None,
                port,
            },
        );
    }
    socket(
        &mut e,
        "host-c",
        SocketCommand::SendTo {
            socket: id("host-c"),
            destination: "203.0.113.2".parse().unwrap(),
            port: 8080,
            data_base64: STANDARD.encode(b"published"),
        },
    );
    run(&mut e, 100_000);
    assert!(e.trace().iter().any(|f| f.kind == "socket.event"
        && f.device_id == id("host-b")
        && f.payload["kind"]["data_base64"] == STANDARD.encode(b"published")));
    socket(
        &mut e,
        "host-b",
        SocketCommand::SendTo {
            socket: id("host-b"),
            destination: "198.51.100.10".parse().unwrap(),
            port: 40000,
            data_base64: STANDARD.encode(b"reply"),
        },
    );
    run(&mut e, 100_000);
    assert!(e.trace().iter().any(|f| f.kind == "socket.event"
        && f.device_id == id("host-c")
        && f.payload["kind"]["source"]["port"] == 8080));
    // The uplink may know a route to the private subnet; the WAN boundary still rejects it.
    let mut p = project();
    p.devices
        .iter_mut()
        .find(|d| d.id == id("uplink"))
        .unwrap()
        .network
        .routes
        .push(StaticRoute {
            network: "192.168.10.0".parse().unwrap(),
            prefix: 24,
            gateway: Some("203.0.113.2".parse().unwrap()),
            interface: id("uplink-a"),
        });
    let mut e = engine(p);
    command(
        &mut e,
        Command::Ping {
            source: id("host-c"),
            destination: "192.168.10.1".into(),
            ttl: 64,
        },
    )
    .unwrap();
    run(&mut e, 4_000_000);
    assert_eq!(count(&e, "icmp.echo.reply"), 0);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "nat.dropped" && f.payload["reason"] == "nat.wan_private_scope")
    );
}
#[test]
fn idle_udp_mapping_expires_then_port_can_be_reused_and_invalid_edits_are_atomic() {
    let mut p = project();
    p.devices
        .iter_mut()
        .find(|d| d.id == id("gateway"))
        .unwrap()
        .network
        .gateway
        .as_mut()
        .unwrap()
        .idle_timeout_us = Decimal(1_000_000);
    let mut e = engine(p);
    socket(
        &mut e,
        "host-a",
        SocketCommand::Bind {
            socket: id("udp"),
            owner: id("udp"),
            protocol: TransportProtocol::Udp,
            interface: None,
            address: None,
            port: 40000,
        },
    );
    socket(
        &mut e,
        "host-a",
        SocketCommand::SendTo {
            socket: id("udp"),
            destination: "198.51.100.10".parse().unwrap(),
            port: 9000,
            data_base64: STANDARD.encode("hello"),
        },
    );
    run(&mut e, 100_000);
    assert_eq!(e.network().nat.entries.len(), 1);
    run(&mut e, 2_000_000);
    assert!(e.network().nat.entries.is_empty());
    assert_eq!(count(&e, "nat.mapping.expired"), 1);
    let before = e.project().clone();
    let revision = e.revision();
    let trace = e.canonical_trace();
    let mut cfg = *e
        .project()
        .device(&id("gateway"))
        .unwrap()
        .network
        .gateway
        .clone()
        .unwrap();
    cfg.wan = cfg.lan.clone();
    assert!(
        command(
            &mut e,
            Command::Security {
                device: id("gateway"),
                command: SecurityCommand::ConfigureGateway { config: cfg }
            }
        )
        .is_err()
    );
    assert_eq!(e.project(), &before);
    assert_eq!(e.revision(), revision);
    assert_eq!(e.canonical_trace(), trace);
}
#[test]
fn web_rule_changes_are_applied_and_the_journal_reports_the_rule_and_packet() {
    let mut e = engine(project());
    let mut config = FirewallConfig {
        enabled: true,
        default_policy: Verdict::Accept,
        ..Default::default()
    };
    command(
        &mut e,
        Command::Security {
            device: id("gateway"),
            command: SecurityCommand::ConfigureFirewall {
                config: config.clone(),
            },
        },
    )
    .unwrap();
    let page = browser(
        &mut e,
        "host-a",
        "browser-0",
        "http://192.168.10.1/admin/firewall",
        None,
    );
    let document = sns_html::parse_network(&page.html).unwrap();
    let action = document
        .blocks
        .iter()
        .find_map(|b| {
            if let sns_html::Block::Form { action, .. } = b {
                action.contains("/firewall/add?").then_some(action.clone())
            } else {
                None
            }
        })
        .unwrap();
    let response = browser(
        &mut e,
        "host-a",
        "browser-0",
        &action,
        Some(
            [
                ("source".into(), "192.168.10.0/24".into()),
                ("destination".into(), "*".into()),
                ("protocol".into(), "TCP".into()),
                ("port".into(), "7777".into()),
                ("action".into(), "DROP".into()),
            ]
            .into(),
        ),
    );
    assert_eq!(response.status, 200);
    config.rules = e
        .project()
        .device(&id("gateway"))
        .unwrap()
        .network
        .firewall
        .as_ref()
        .unwrap()
        .rules
        .clone();
    assert_eq!(config.rules.len(), 1);
    start_echo(&mut e);
    clients(&mut e);
    run(&mut e, 200_000);
    assert!(
        e.network().firewall.journal[&id("gateway")]
            .iter()
            .any(|d| d.rule == Some(1) && d.action == Verdict::Drop)
    );
    let page = browser(
        &mut e,
        "host-a",
        "browser-0",
        "http://192.168.10.1/admin/log",
        None,
    );
    assert_eq!(page.status, 200);
    assert!(page.html.contains("Règle"));
    assert!(page.html.contains("Trame"));
    assert!(page.html.contains("Drop"));
    record(&e, "web-filter");
}
#[test]
fn wan_arp_failure_quotes_the_original_private_probe() {
    let mut p = project();
    p.devices
        .iter_mut()
        .find(|d| d.id == id("uplink"))
        .unwrap()
        .interfaces[0]
        .ipv4 = Some(Ipv4Config::Static {
        address: "203.0.113.9".into(),
        prefix_length: 24,
    });
    let mut e = engine(p);
    for h in ["host-a", "host-b"] {
        command(
            &mut e,
            Command::Ping {
                source: id(h),
                destination: "198.51.100.10".into(),
                ttl: 64,
            },
        )
        .unwrap();
    }
    run(&mut e, 4_000_000);
    assert_eq!(count(&e, "routing.arp.failed"), 2);
    assert_eq!(
        e.trace()
            .iter()
            .filter(
                |f| f.kind == "icmp.error" && matches!(f.device_id.as_str(), "host-a" | "host-b")
            )
            .count(),
        2
    );
    assert!(e.trace().iter().any(|f| {
        f.kind == "ipv4.delivered"
            && f.payload
                .get("icmp")
                .is_some_and(|i| i["quoted"]["source"] == "192.168.10.2")
    }));
}
#[test]
fn private_network_workflows_have_identical_canonical_traces() {
    let execute = || {
        let mut e = engine(project());
        start_echo(&mut e);
        clients(&mut e);
        run(&mut e, 1_000_000);
        for (h, a) in [("host-a", "client-a"), ("host-b", "client-b")] {
            app(
                &mut e,
                h,
                AppCommand::Send {
                    application: id(a),
                    text: "deterministic".into(),
                },
            );
        }
        run(&mut e, 1_000_000);
        e.canonical_trace()
    };
    assert_eq!(execute(), execute());
}
#[test]
fn wan_dhcp_rejects_an_overlapping_lan_assignment_instead_of_swapping_roles() {
    let mut p = project();
    let gateway = p
        .devices
        .iter_mut()
        .find(|d| d.id == id("gateway"))
        .unwrap();
    gateway.interfaces[1].ipv4 = Some(Ipv4Config::Dhcp);
    gateway.network.default_gateway = None;
    let provider = p.devices.iter_mut().find(|d| d.id == id("uplink")).unwrap();
    provider.interfaces[0].ipv4 = Some(Ipv4Config::Static {
        address: "192.168.10.50".into(),
        prefix_length: 24,
    });
    provider.network.dhcp_server = Some(Box::new(DhcpConfig {
        interface: id("uplink-a"),
        pool_start: "192.168.10.100".parse().unwrap(),
        pool_end: "192.168.10.101".parse().unwrap(),
        prefix_length: 24,
        gateway: Some("192.168.10.50".parse().unwrap()),
        dns: None,
        use_host_options: false,
        reservations: Default::default(),
        lease_us: Decimal(10_000_000),
    }));
    let mut e = engine(p);
    run(&mut e, 1_000_000);
    assert!(!e.network().dhcp.assigned.contains_key(&id("gateway-wan")));
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "dhcp.failed" && f.payload["reason"] == "nat.overlapping_networks")
    );
    let gateway = e.project().device(&id("gateway")).unwrap();
    assert_eq!(
        gateway.network.gateway.as_ref().unwrap().lan,
        id("gateway-lan")
    );
    assert_eq!(
        gateway.interfaces[0].ipv4,
        Some(Ipv4Config::Static {
            address: "192.168.10.1".into(),
            prefix_length: 24
        })
    );
}
