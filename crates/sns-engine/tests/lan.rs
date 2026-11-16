use sns_engine::{Engine, Limits, Termination};
use sns_model::*;
use sns_protocols::{mask_prefix, next_hop, subnet};
use std::net::Ipv4Addr;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn project() -> Project {
    serde_json::from_str(include_str!("../../../fixtures/valid/lan.json")).unwrap()
}
fn engine() -> Engine {
    Engine::from_project(project(), 42, Limits::default()).unwrap()
}
fn cmd(e: &mut Engine, name: &str, command: Command) {
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(name),
        project_id: e.project().project_id.clone(),
        expected_revision: Decimal(0),
        run_id: e.run_id().clone(),
        command,
    })
    .unwrap();
}
fn ping(e: &mut Engine, name: &str, target: &str) {
    cmd(
        e,
        name,
        Command::Ping {
            source: id("host-a"),
            destination: target.into(),
            ttl: 64,
        },
    );
}
fn count(e: &Engine, kind: &str) -> usize {
    e.trace().iter().filter(|f| f.kind == kind).count()
}
fn finish(e: &mut Engine) {
    e.run(10_000, Some(e.now() + 3_000_000)).unwrap();
    assert_eq!(e.network().pending_probes(), 0);
}
#[test]
fn r_lan_arp_learning_unicast_cache_and_purge() {
    let mut e = engine();
    ping(&mut e, "ping-1", "192.168.1.3");
    finish(&mut e);
    assert_eq!(count(&e, "arp.request"), 1);
    assert_eq!(count(&e, "arp.reply"), 1);
    assert_eq!(count(&e, "icmp.echo.reply"), 1);
    assert!(
        !e.trace()
            .iter()
            .any(|f| f.device_id == id("host-c") && f.kind == "ipv4.delivered")
    );
    assert_eq!(e.network().sat.len(), 2);
    assert_eq!(e.network().arp.len(), 2);
    let before = e.trace().len();
    ping(&mut e, "ping-2", "192.168.1.3");
    finish(&mut e);
    assert_eq!(count(&e, "arp.request"), 1);
    assert_eq!(count(&e, "icmp.echo.reply"), 2);
    assert!(
        !e.trace()[before..]
            .iter()
            .any(|f| f.device_id == id("host-c") && f.kind == "ethernet.received")
    );
    cmd(
        &mut e,
        "arp-clear",
        Command::PurgeArp {
            device: id("host-a"),
        },
    );
    assert_eq!(e.network().arp.len(), 1);
    cmd(
        &mut e,
        "sat-clear",
        Command::PurgeSat {
            device: id("switch"),
        },
    );
    assert!(e.network().sat.is_empty());
    ping(&mut e, "ping-3", "192.168.1.3");
    finish(&mut e);
    assert_eq!(count(&e, "arp.request"), 2);
    e.run(10_000, None).unwrap();
    assert!(e.network().arp.is_empty());
    assert!(e.network().sat.is_empty());
    assert!(count(&e, "switch.sat.expired") > 0);
    assert!(count(&e, "arp.cache.expired") > 0);
}
#[test]
fn deterministic_step_chunks_and_speeds_are_identical() {
    let mut a = engine();
    let mut b = engine();
    let mut c = engine();
    for e in [&mut a, &mut b, &mut c] {
        ping(e, "ping", "192.168.1.3");
    }
    a.run(10000, None).unwrap();
    while b.step().unwrap() {}
    for bound in (0..=301_000_000).step_by(1_000_000) {
        c.run(10000, Some(bound)).unwrap();
    }
    assert_eq!(a.canonical_trace(), b.canonical_trace());
    assert_eq!(a.canonical_trace(), c.canonical_trace());
    let ids: std::collections::BTreeSet<_> = a.trace().iter().map(|f| f.event_id.clone()).collect();
    assert!(
        a.trace()
            .iter()
            .all(|f| f.caused_by.as_ref().is_none_or(|id| ids.contains(id)))
    );
}
#[test]
fn paused_step_loss_timeouts_and_ten_clean_restarts() {
    let mut e = engine();
    ping(&mut e, "drop-ping", "192.168.1.3");
    let pending = e.pending();
    assert_eq!(e.now(), 0);
    assert_eq!(e.pending(), pending);
    assert!(e.step().unwrap());
    assert_eq!(e.now(), 0);
    assert!(e.pending() > 1);
    cmd(&mut e, "drop-on", Command::ManualLoss { enabled: true });
    finish(&mut e);
    assert_eq!(count(&e, "icmp.echo.reply"), 0);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.payload.get("reason") == Some(&serde_json::json!("simulation.manual_loss")))
    );
    cmd(&mut e, "drop-off", Command::ManualLoss { enabled: false });
    ping(&mut e, "retry", "192.168.1.3");
    finish(&mut e);
    assert_eq!(count(&e, "icmp.echo.reply"), 1);
    for _ in 0..10 {
        let old = e.run_id().clone();
        cmd(&mut e, "stop", Command::Stop);
        assert_eq!(e.pending(), 0);
        assert!(e.network().arp.is_empty());
        assert!(e.network().sat.is_empty());
        e.restart().unwrap();
        assert_ne!(e.run_id(), &old);
        assert_eq!(e.now(), 0);
        assert!(e.trace().is_empty());
        assert_eq!(e.pending(), 0);
        ping(&mut e, "ping", "192.168.1.3");
        finish(&mut e);
        assert_eq!(count(&e, "icmp.echo.reply"), 1);
    }
}
#[test]
fn local_subnets_gateway_masks_and_errors() {
    let addr: Ipv4Addr = "192.168.10.2".parse().unwrap();
    assert_eq!(
        subnet(addr, 24).unwrap(),
        (
            "192.168.10.0".parse().unwrap(),
            "192.168.10.255".parse().unwrap()
        )
    );
    assert_eq!(mask_prefix("255.255.255.0").unwrap(), 24);
    assert_eq!(mask_prefix("0.0.0.0").unwrap(), 0);
    assert_eq!(mask_prefix("255.255.255.255").unwrap(), 32);
    assert!(mask_prefix("255.0.255.0").is_err());
    let mut p = project();
    p.links.clear();
    let mut e = Engine::from_project(p.clone(), 42, Limits::default()).unwrap();
    for (n, target) in [("loop", "127.0.0.1"), ("own", "192.168.1.2")] {
        ping(&mut e, n, target);
        finish(&mut e);
    }
    assert_eq!(count(&e, "icmp.echo.reply"), 2);
    assert_eq!(count(&e, "ethernet.sent"), 0);
    ping(&mut e, "no-route", "10.0.0.1");
    finish(&mut e);
    cmd(
        &mut e,
        "ttl",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 0,
        },
    );
    finish(&mut e);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.payload.get("reason") == Some(&serde_json::json!("routing.no_route")))
    );
    assert!(
        e.trace()
            .iter()
            .any(|f| f.payload.get("reason") == Some(&serde_json::json!("ipv4.ttl_expired")))
    );
    p.devices[0].network.default_gateway = Some("192.168.1.1".into());
    assert_eq!(
        next_hop(&p.devices[0], "10.0.0.1".parse().unwrap())
            .unwrap()
            .2,
        "192.168.1.1".parse::<Ipv4Addr>().unwrap()
    );
}
#[test]
fn loop_protection_is_bounded_and_explained() {
    let mut p = project();
    let first = p.devices[3].clone();
    for n in 1..=2 {
        let mut d = first.clone();
        d.id = id(&format!("sw-{n}"));
        for (j, port) in d.interfaces.iter_mut().enumerate() {
            port.id = id(&format!("sw-{n}-p{j}"));
            port.mac = format!("02:00:00:{n:02x}:01:{j:02x}");
        }
        p.devices.push(d);
    }
    for (n, a, b) in [
        (0, "switch-p3", "sw-1-p0"),
        (1, "sw-1-p1", "sw-2-p0"),
        (2, "sw-2-p1", "switch-p4"),
    ] {
        p.links.push(Link {
            id: id(&format!("loop-{n}")),
            endpoints: [id(a), id(b)],
            latency_us: Decimal(1000),
            jitter_us: Decimal(0),
        });
    }
    let mut e = Engine::from_project(p, 1, Limits::default()).unwrap();
    ping(&mut e, "ping", "192.168.1.3");
    let report = e.run(2000, None).unwrap();
    assert_eq!(report.termination, Termination::Idle);
    assert!(report.processed_events < 200);
    assert!(
        e.trace()
            .iter()
            .any(|f| f.payload.get("reason") == Some(&serde_json::json!("switch.loop_detected")))
    );
}
#[test]
fn lan_quota_failure_rolls_back_state_rng_queue_and_trace() {
    let mut e = Engine::from_project(
        project(),
        1,
        Limits {
            trace: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    ping(&mut e, "ping", "192.168.1.3");
    let before = e.trace_hash();
    assert!(e.step().is_err());
    assert_eq!(e.now(), 0);
    assert_eq!(e.pending(), 1);
    assert_eq!(e.trace_hash(), before);
    assert_eq!(e.network().pending_probes(), 0);
    assert!(e.network().arp.is_empty());
}
#[test]
fn gateway_emits_wire_icmp_time_exceeded_and_unreachable() {
    let mut p = project();
    p.devices[1].kind = DeviceKind::Router;
    p.devices[0].network.default_gateway = Some("192.168.1.3".into());
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    for (name, ttl, expected) in [("expired", 1, 11), ("unreachable", 64, 3)] {
        cmd(
            &mut e,
            name,
            Command::Ping {
                source: id("host-a"),
                destination: "10.0.0.1".into(),
                ttl,
            },
        );
        finish(&mut e);
        assert!(e.trace().iter().any(|f| f.kind == "icmp.error"
            && f.correlation_id == id(name)
            && f.payload["icmp_type"] == expected));
    }
    assert_eq!(count(&e, "icmp.error.sent"), 2);
    assert_eq!(count(&e, "icmp.echo.reply"), 0);
}
#[test]
fn configurable_sat_retention_and_arp_coalescing_keep_correlations() {
    let mut p = project();
    p.lan.get_mut(&id("switch")).unwrap().sat_retention_us = Decimal(10_000);
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    ping(&mut e, "first", "192.168.1.3");
    ping(&mut e, "second", "192.168.1.3");
    e.run(1000, Some(9_000)).unwrap();
    assert_eq!(count(&e, "arp.request"), 1);
    assert_eq!(count(&e, "icmp.echo.reply"), 2);
    assert_eq!(e.network().sat.len(), 2);
    let flows: std::collections::BTreeSet<_> = e
        .trace()
        .iter()
        .filter(|f| f.kind == "icmp.echo.reply")
        .map(|f| f.correlation_id.clone())
        .collect();
    assert_eq!(
        flows,
        std::collections::BTreeSet::from([id("first"), id("second")])
    );
    e.run(1000, Some(30_000)).unwrap();
    assert!(e.network().sat.is_empty());
    assert!(!e.network().arp.is_empty());
}
#[test]
fn all_three_hosts_exchange_and_shuffled_storage_preserves_seeded_trace() {
    let mut p = project();
    for link in &mut p.links {
        link.jitter_us = Decimal(500);
    }
    let mut reordered = p.clone();
    reordered.devices.reverse();
    reordered.links.reverse();
    for d in &mut reordered.devices {
        d.interfaces.reverse();
    }
    let mut a = Engine::from_project(p, 91, Limits::default()).unwrap();
    let mut b = Engine::from_project(reordered, 91, Limits::default()).unwrap();
    for e in [&mut a, &mut b] {
        for (name, source, target) in [
            ("a-b", "host-a", "192.168.1.3"),
            ("b-c", "host-b", "192.168.1.4"),
            ("c-a", "host-c", "192.168.1.2"),
        ] {
            cmd(
                e,
                name,
                Command::Ping {
                    source: id(source),
                    destination: target.into(),
                    ttl: 64,
                },
            );
        }
        e.run(1000, None).unwrap();
        assert_eq!(count(e, "icmp.echo.reply"), 3);
    }
    assert_eq!(a.canonical_trace(), b.canonical_trace());
    for receive in a.trace().iter().filter(|f| f.kind == "ethernet.received") {
        let sent = a
            .trace()
            .iter()
            .find(|f| Some(&f.event_id) == receive.caused_by.as_ref())
            .unwrap();
        assert_eq!(sent.kind, "ethernet.sent");
        assert_eq!(
            sent.payload["frame"]["frame_id"],
            receive.payload["frame"]["frame_id"]
        );
    }
}
#[test]
fn lan_time_overflow_leaves_failing_delivery_unconsumed() {
    let mut p = project();
    for link in &mut p.links {
        link.latency_us = Decimal(u64::MAX);
    }
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    ping(&mut e, "ping", "192.168.1.3");
    for _ in 0..3 {
        e.step().unwrap();
    }
    let before = e.trace_hash();
    let pending = e.pending();
    let now = e.now();
    assert_eq!(e.step().unwrap_err().code, "simulation.time_overflow");
    assert_eq!(e.now(), now);
    assert_eq!(e.pending(), pending);
    assert_eq!(e.trace_hash(), before);
    assert!(e.network().sat.is_empty());
}

#[test]
fn direct_example_resolves_arp_and_replies_over_its_only_cable() {
    let project: Project =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/direct.json")).unwrap();
    assert_eq!(project.devices.len(), 2);
    assert_eq!(project.links.len(), 1);
    let mut e = Engine::from_project(project, 42, Limits::default()).unwrap();
    ping(&mut e, "direct-ping", "192.168.1.3");
    finish(&mut e);
    assert_eq!(count(&e, "arp.request"), 1);
    assert_eq!(count(&e, "arp.reply"), 1);
    assert_eq!(count(&e, "icmp.echo.reply"), 1);
    assert!(e.network().sat.is_empty());
}
