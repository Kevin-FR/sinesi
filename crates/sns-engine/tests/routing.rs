use sns_engine::{Engine, Limits};
use sns_model::*;
use sns_protocols::routing::{self, Origin};
thread_local! {static REQUESTS:std::cell::RefCell<Vec<serde_json::Value>>=const{std::cell::RefCell::new(Vec::new())};}
fn record(e: &Engine, name: &str, bounds: &[u64]) {
    if let Some(directory) = std::env::var_os("SNS_RECORD_J3") {
        let root = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(format!("j3-{name}.jsonl")), e.canonical_trace()).unwrap();
        std::fs::write(
            root.join(format!("j3-{name}-project.json")),
            serde_json::to_vec_pretty(e.project()).unwrap(),
        )
        .unwrap();
        let routes: std::collections::BTreeMap<_, _> = e
            .project()
            .devices
            .iter()
            .map(|h| (h.id.clone(), e.network().routing.views(h, e.now())))
            .collect();
        REQUESTS.with(|requests|std::fs::write(root.join(format!("j3-{name}-recipe.json")),serde_json::to_vec_pretty(&serde_json::json!({"seed":42,"bounds_us":bounds,"commands":requests.borrow().clone(),"final_time_us":e.now().to_string(),"routes":routes})).unwrap()).unwrap());
    }
}
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn project(rip: bool) -> Project {
    serde_json::from_str(if rip {
        include_str!("../../../fixtures/valid/rip.json")
    } else {
        include_str!("../../../fixtures/valid/routed.json")
    })
    .unwrap()
}
fn engine(p: Project) -> Engine {
    REQUESTS.with(|r| r.borrow_mut().clear());
    Engine::from_project(p, 42, Limits::default()).unwrap()
}
fn command(e: &mut Engine, name: &str, command: Command) {
    let request = CommandEnvelope {
        api_version: 1,
        command_id: id(name),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command,
    };
    let time = Decimal(e.now());
    e.command(request.clone()).unwrap();
    REQUESTS.with(|r| {
        r.borrow_mut()
            .push(serde_json::json!({"at_us":time,"request":request}))
    });
}

fn ping(e: &mut Engine, name: &str, ttl: u8) {
    command(
        e,
        name,
        Command::Ping {
            source: id("a"),
            destination: "192.168.30.2".into(),
            ttl,
        },
    );
}
fn run(e: &mut Engine, bound: u64) {
    e.run(100_000, Some(bound)).unwrap();
}
fn terminal(e: &mut Engine, name: &str, line: &str) {
    command(
        e,
        name,
        Command::App {
            device: id("a"),
            command: AppCommand::Terminal {
                application: id("a-terminal"),
                line: line.into(),
            },
        },
    );
}
#[test]
fn three_lans_forward_original_addresses_and_ttl_and_return_real_icmp() {
    let mut e = engine(project(false));
    ping(&mut e, "probe", 64);
    run(&mut e, 5_000_000);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "icmp.echo.reply" && f.device_id == id("a"))
    );
    let rows: Vec<_> = e
        .trace()
        .iter()
        .filter(|f| f.kind == "ipv4.forwarded" && f.payload["source"] == "192.168.10.2")
        .collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].payload["ttl"], 63);
    assert_eq!(rows[1].payload["ttl"], 62);
    ping(&mut e, "ttl-one", 1);
    run(&mut e, 10_000_000);
    let err = e
        .trace()
        .iter()
        .find(|f| f.kind == "icmp.error" && f.correlation_id == id("ttl-one"))
        .unwrap();
    assert_eq!(err.payload["source"], "192.168.10.1");
    assert_eq!(err.payload["reason"], "ipv4.ttl_expired");
    record(&e, "routes", &[5_000_000, 10_000_000]);
}
#[test]
fn routes_specific_prefix_default_delete_and_host_forwarding_switch() {
    let mut p = project(false);
    let r = p.devices.iter_mut().find(|d| d.id == id("r1")).unwrap();
    r.network.default_gateway = Some("192.168.20.2".into());
    let special = StaticRoute {
        network: "192.168.30.2".parse().unwrap(),
        prefix: 32,
        gateway: None,
        interface: id("r1-p0"),
    };
    r.network.routes.push(special);
    let target = "192.168.30.2".parse().unwrap();
    let table = routing::configured(r);
    assert_eq!(routing::select(&table, target).unwrap().prefix, 32);
    r.network.routes.clear();
    let table = routing::configured(r);
    assert_eq!(
        routing::select(&table, target).unwrap().origin,
        Origin::Default
    );
    r.kind = DeviceKind::Workstation;
    r.network.forwarding = Some(false);
    let mut blocked = engine(p.clone());
    ping(&mut blocked, "disabled", 64);
    run(&mut blocked, 5_000_000);
    assert!(blocked.trace().iter().any(
        |f| f.kind == "routing.dropped" && f.payload["reason"] == "routing.forwarding_disabled"
    ));
    p.devices
        .iter_mut()
        .find(|d| d.id == id("r1"))
        .unwrap()
        .network
        .forwarding = Some(true);
    let mut enabled = engine(p);
    ping(&mut enabled, "enabled", 64);
    run(&mut enabled, 5_000_000);
    assert!(enabled.trace().iter().any(|f| f.kind == "icmp.echo.reply"));
}
#[test]
fn traceroute_starts_at_one_orders_hops_and_finishes_without_os_calls() {
    let mut e = engine(project(false));
    terminal(&mut e, "trace", "traceroute -m 8 192.168.30.2");
    run(&mut e, 15_000_000);
    let probes: Vec<_> = e
        .trace()
        .iter()
        .filter(|f| f.kind == "icmp.echo.requested")
        .map(|f| f.payload["ttl"].as_u64().unwrap())
        .collect();
    assert_eq!(probes, vec![1, 2, 3]);
    let view = e
        .apps()
        .views(e.project())
        .into_iter()
        .find(|h| h.device == id("a"))
        .unwrap();
    let lines = serde_json::to_string(&view.applications[0].lines).unwrap();
    assert!(!view.applications[0].busy);
    for address in ["192.168.10.1", "192.168.20.2", "192.168.30.2"] {
        assert!(lines.contains(address), "{lines}");
    }
    terminal(&mut e, "table", "route");
    record(&e, "traceroute", &[15_000_000]);
}
#[test]
fn rip_three_routers_shortest_path_poisoning_and_expiration() {
    let mut e = engine(project(true));
    run(&mut e, 2_000_000);
    let host = e.project().device(&id("r1")).unwrap();
    let table = e.network().routing.views(host, e.now());
    let row = routing::select(&table, "192.168.30.2".parse().unwrap()).unwrap();
    assert_eq!(row.origin, Origin::Rip);
    assert_eq!(row.metric, 1);
    assert_eq!(row.gateway.unwrap().to_string(), "192.168.50.2");
    ping(&mut e, "learned-ping", 64);
    run(&mut e, 6_000_000);
    assert!(e.trace().iter().any(|f| f.kind == "icmp.echo.reply"));
    command(
        &mut e,
        "mute",
        Command::RipMute {
            device: id("r3"),
            muted: true,
        },
    );
    run(&mut e, 160_000_000);
    let table = e
        .network()
        .routing
        .views(e.project().device(&id("r1")).unwrap(), e.now());
    assert!(routing::select(&table, "192.168.30.2".parse().unwrap()).is_none());
    assert!(e.trace().iter().any(|f| f.kind == "rip.route.expired"));
    assert!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "rip.advertisement.sent")
            .all(|f| f.payload["poison_reverse"] == true)
    );
    command(
        &mut e,
        "unmute",
        Command::RipMute {
            device: id("r3"),
            muted: false,
        },
    );
    run(&mut e, 163_000_000);
    assert!(
        routing::select(
            &e.network()
                .routing
                .views(e.project().device(&id("r1")).unwrap(), e.now()),
            "192.168.30.2".parse().unwrap()
        )
        .is_some()
    );
    record(&e, "rip", &[2_000_000, 6_000_000, 160_000_000, 163_000_000]);
}
#[test]
fn rip_deterministic_with_reordered_entities_and_step_execution() {
    let p = project(true);
    let mut reversed = p.clone();
    reversed.devices.reverse();
    reversed.links.reverse();
    for d in &mut reversed.devices {
        d.interfaces.reverse();
    }
    let mut a = engine(p);
    let mut b = engine(reversed);
    run(&mut a, 2_000_000);
    while b.pump(1, Some(2_000_000)).unwrap().0 > 0 {}
    assert_eq!(a.canonical_trace(), b.canonical_trace());
}

#[test]
fn missing_route_arp_failure_and_routing_loop_have_distinct_structured_causes() {
    let mut p = project(false);
    p.devices
        .iter_mut()
        .find(|h| h.id == id("r1"))
        .unwrap()
        .network
        .routes
        .clear();
    let mut e = engine(p);
    ping(&mut e, "missing", 64);
    run(&mut e, 5_000_000);
    assert!(e.trace().iter().any(|f| f.kind == "icmp.error"
        && f.device_id == id("a")
        && f.payload["reason"] == "routing.no_route"));
    let mut e = engine(project(false));
    command(
        &mut e,
        "absent",
        Command::Ping {
            source: id("a"),
            destination: "192.168.30.99".into(),
            ttl: 64,
        },
    );
    run(&mut e, 5_000_000);
    assert!(
        e.trace().iter().any(|f| f.kind == "icmp.error"
            && f.device_id == id("a")
            && f.payload["reason"] == "arp.timeout"),
        "{:?}",
        e.trace()
            .iter()
            .filter(|f| f.kind.contains("error"))
            .map(|f| (&f.kind, &f.payload))
            .collect::<Vec<_>>()
    );
    let mut p = project(false);
    for (h, gw) in [("r1", "192.168.20.2"), ("r2", "192.168.20.1")] {
        p.devices
            .iter_mut()
            .find(|d| d.id == id(h))
            .unwrap()
            .network
            .default_gateway = Some(gw.into());
    }
    let mut e = engine(p);
    command(
        &mut e,
        "loop",
        Command::Ping {
            source: id("a"),
            destination: "198.51.100.2".into(),
            ttl: 6,
        },
    );
    run(&mut e, 5_000_000);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "icmp.error" && f.payload["reason"] == "ipv4.ttl_expired")
    );
    assert_eq!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "ipv4.forwarded" && f.payload["source"] == "192.168.10.2")
            .count(),
        5
    );
}
#[test]
fn arp_injection_changes_only_simulated_cache_and_broadcast_stays_on_lan() {
    let mut e = engine(project(false));
    terminal(&mut e, "inject", "arpsend 192.168.10.99 192.168.10.1");
    run(&mut e, 100_000);
    let entry = &e.network().arp[&(id("r1-p0"), "192.168.10.99".parse().unwrap())];
    assert_eq!(
        entry.mac,
        e.project().device(&id("a")).unwrap().interfaces[0].mac
    );
    assert!(e.trace().iter().any(|f| f.kind == "routing.arp.injected"));
    terminal(&mut e, "bad-inject", "arpsend 224.0.0.1 192.168.10.1");
    terminal(&mut e, "broadcast", "ping -c 1 -b 192.168.10.255");
    run(&mut e, 5_000_000);
    assert!(!e.trace().iter().any(|f| f.kind == "ipv4.forwarded"));
    assert!(e.trace().iter().any(|f| f.kind == "icmp.echo.reply"));
    let before = e.trace().len();
    terminal(&mut e, "reject", "ping 192.168.10.255");
    assert!(
        !e.trace()[before..]
            .iter()
            .any(|f| f.kind == "icmp.echo.requested")
    );
}
#[test]
fn tcp_echo_uses_multiple_router_hops_and_preserves_routing_profile() {
    let mut e = engine(project(false));
    for (host, app, software) in [
        ("a", "client", Software::TcpClient),
        ("b", "server", Software::TcpServer),
    ] {
        command(
            &mut e,
            &format!("install-{host}"),
            Command::App {
                device: id(host),
                command: AppCommand::Install {
                    application: id(app),
                    software,
                },
            },
        );
    }
    command(
        &mut e,
        "serve",
        Command::App {
            device: id("b"),
            command: AppCommand::StartServer {
                application: id("server"),
                port: 7777,
            },
        },
    );
    command(
        &mut e,
        "connect",
        Command::App {
            device: id("a"),
            command: AppCommand::Connect {
                application: id("client"),
                target: "b".into(),
                port: 7777,
            },
        },
    );
    run(&mut e, 2_000_000);
    command(
        &mut e,
        "send",
        Command::App {
            device: id("a"),
            command: AppCommand::Send {
                application: id("client"),
                text: "Bonjour réseau routé".into(),
            },
        },
    );
    run(&mut e, 5_000_000);
    use base64::Engine as _;
    let host = e
        .apps()
        .views(e.project())
        .into_iter()
        .find(|h| h.device == id("a"))
        .unwrap();
    let app = host
        .applications
        .iter()
        .find(|a| a.id == id("client"))
        .unwrap();
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(&app.received_base64)
            .unwrap(),
        "Bonjour réseau routé".as_bytes()
    );
    assert_eq!(e.project().schema_version, 4);
}
#[test]
fn rip_chain_uses_two_hops_and_restarts_have_no_old_generation_state() {
    let p: Project =
        serde_json::from_str(include_str!("../../../fixtures/valid/rip-chain.json")).unwrap();
    let mut e = engine(p);
    run(&mut e, 2_000_000);
    let table = e
        .network()
        .routing
        .views(e.project().device(&id("r1")).unwrap(), e.now());
    assert_eq!(
        routing::select(&table, "192.168.30.2".parse().unwrap())
            .unwrap()
            .metric,
        2
    );
    e.stop().unwrap();
    assert!(e.network().routing.learned.is_empty());
    e.restart().unwrap();
    assert!(e.trace().is_empty());
    assert!(e.network().routing.learned.is_empty());
    run(&mut e, 2_000_000);
    assert!(!e.network().routing.learned.is_empty());
}
#[test]
fn interrupted_traceroute_stops_future_probes_and_malformed_routes_do_not_panic() {
    let mut e = engine(project(false));
    terminal(&mut e, "trace-cancel", "traceroute 192.168.30.2");
    e.step().unwrap();
    command(
        &mut e,
        "interrupt",
        Command::App {
            device: id("a"),
            command: AppCommand::Interrupt {
                application: id("a-terminal"),
            },
        },
    );
    run(&mut e, 10_000_000);
    assert_eq!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "icmp.echo.requested")
            .count(),
        1
    );
    let mut p = project(false);
    let host = p.devices.iter_mut().find(|h| h.id == id("r1")).unwrap();
    if let Some(Ipv4Config::Static { prefix_length, .. }) = &mut host.interfaces[1].ipv4 {
        *prefix_length = 33;
    }
    assert!(p.validate().is_err());
}

#[test]
fn arp_exercise_redirects_a_frame_to_the_claiming_host_only_in_the_simulation() {
    let p: Project =
        serde_json::from_str(include_str!("../../../fixtures/valid/arp.json")).unwrap();
    let mut e = engine(p);
    command(
        &mut e,
        "ordinary",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    );
    run(&mut e, 100_000);
    command(
        &mut e,
        "spoof",
        Command::App {
            device: id("host-c"),
            command: AppCommand::Terminal {
                application: id("injector"),
                line: "arpsend 192.168.1.3 192.168.1.2".into(),
            },
        },
    );
    run(&mut e, 200_000);
    command(
        &mut e,
        "redirected",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    );
    run(&mut e, 5_000_000);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.correlation_id == id("redirected")
                && f.kind == "routing.dropped"
                && f.device_id == id("host-c"))
    );
    assert!(
        !e.trace()
            .iter()
            .any(|f| f.correlation_id == id("redirected") && f.kind == "icmp.echo.reply")
    );
}

#[test]
fn rejected_broadcast_command_does_not_leave_a_failing_event_in_the_queue() {
    let mut e = engine(project(false));
    let pending = e.pending();
    let trace = e.canonical_trace();
    let result = e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("broadcast-rejected"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(0),
        command: Command::Ping {
            source: id("a"),
            destination: "192.168.10.255".into(),
            ttl: 64,
        },
    });
    assert_eq!(result.unwrap_err().code, "ipv4.broadcast_requires_opt_in");
    assert_eq!(e.pending(), pending);
    assert_eq!(e.canonical_trace(), trace);
    ping(&mut e, "next-valid", 64);
    run(&mut e, 5_000_000);
    assert!(e.trace().iter().any(|f| f.kind == "icmp.echo.reply"));
}
