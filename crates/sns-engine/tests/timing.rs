//! Response waits on real simulated links, with durable and replayable policy.
use sns_engine::{Checkpoint, Engine, Limits, Recording};
use sns_model::*;

fn id(name: &str) -> Id {
    Id::new(name).unwrap()
}
fn project() -> Project {
    serde_json::from_slice(include_bytes!("../../../fixtures/valid/lan.json")).unwrap()
}
fn engine(project: Project, factor: u8) -> Engine {
    Engine::from_project_with_wait_factor(
        project,
        42,
        Limits::default(),
        WaitFactor::try_from(factor).unwrap(),
    )
    .unwrap()
}
fn command(e: &mut Engine, name: &str, command: Command) {
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(name),
        project_id: e.project().project_id.clone(),
        expected_revision: Decimal(e.revision()),
        run_id: e.run_id().clone(),
        command,
    })
    .unwrap();
}
fn number(value: &serde_json::Value) -> u64 {
    let value = value.get("$u64").unwrap_or(value);
    value
        .as_u64()
        .unwrap_or_else(|| value.as_str().unwrap().parse().unwrap())
}
fn times(e: &Engine, pointer: &str) -> Vec<u64> {
    e.checkpoint_state().unwrap()["queue"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|q| q.pointer(pointer).is_some())
        .map(|q| number(&q["time"]))
        .collect()
}
fn slow_ping(factor: u8) -> Engine {
    let mut e = engine(project(), factor);
    for link in e.project().links.clone() {
        command(
            &mut e,
            &format!("delay-{}", link.id.as_str()),
            Command::ConfigureLink {
                link: link.id,
                conditions: LinkConditions {
                    delay_us: Decimal(300_000),
                    jitter_us: Decimal(0),
                    loss_ppm: 0,
                },
            },
        );
    }
    command(
        &mut e,
        "ping",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    );
    e
}

#[test]
fn factor_is_bounded_and_default_keeps_the_previous_checkpoint_encoding() {
    for valid in ["2", "2.0", "2e0"] {
        assert_eq!(
            serde_json::from_str::<WaitFactor>(valid).unwrap(),
            WaitFactor::try_from(2).unwrap()
        );
    }
    for invalid in ["0", "6", "256", "-1", "1.5", "\"2\"", "null"] {
        assert!(
            serde_json::from_str::<WaitFactor>(invalid).is_err(),
            "{invalid}"
        );
    }
    assert!(WaitFactor::try_from(5).unwrap().scale(u64::MAX).is_err());
    let e = engine(project(), 1);
    let point = serde_json::to_value(e.checkpoint().unwrap()).unwrap();
    assert_eq!(point["checkpoint_version"], 1);
    assert_eq!(
        point["runtime_state"]["runtime_profile"],
        "sns-runtime-state-v1"
    );
    assert!(point["recording"].get("wait_factor").is_none());
    assert!(
        point["runtime_state"]["network"]
            .get("wait_factor")
            .is_none()
    );
    Checkpoint::from_bytes(include_bytes!(
        "../../../fixtures/valid/checkpoint-v1-unconfigured.json"
    ))
    .unwrap()
    .restore()
    .unwrap();
}

#[test]
fn extended_wait_succeeds_on_a_slow_lan_and_checkpoint_replay_preserve_its_suffix() {
    let mut ordinary = slow_ping(1);
    ordinary.run(10000, Some(6_000_000)).unwrap();
    assert!(!ordinary.trace().iter().any(|f| f.kind == "icmp.echo.reply"));
    let mut extended = slow_ping(5);
    extended.step().unwrap();
    let point = extended.checkpoint().unwrap();
    assert_eq!(point.checkpoint_version, 2);
    assert_eq!(
        point.recording.wait_factor,
        WaitFactor::try_from(5).unwrap()
    );
    let mut restored = point.restore().unwrap();
    extended.run(10000, Some(6_000_000)).unwrap();
    restored.run(10000, Some(6_000_000)).unwrap();
    assert!(extended.trace().iter().any(|f| f.kind == "icmp.echo.reply"));
    assert_eq!(restored.canonical_trace(), extended.canonical_trace());
    let recording = extended.recording().unwrap();
    assert_eq!(
        recording.replay().unwrap().canonical_trace(),
        extended.canonical_trace()
    );
    assert_eq!(
        point
            .replay_remaining(&recording)
            .unwrap()
            .canonical_trace(),
        extended.canonical_trace()
    );
    for fact in extended
        .trace()
        .iter()
        .filter(|f| f.kind == "arp.cache.learned")
    {
        assert_eq!(
            number(&fact.payload["expires_us"]) - fact.time_us.0,
            sns_protocols::ARP_RETENTION_US
        );
    }
    for fact in extended
        .trace()
        .iter()
        .filter(|f| f.kind == "simulation.link.transit")
    {
        assert_eq!(number(&fact.payload["delay_us"]), 300_000);
    }
    let mut other_policy = recording.clone();
    other_policy.wait_factor = WaitFactor::try_from(4).unwrap();
    assert!(point.replay_remaining(&other_policy).is_err());
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_TIMING") {
        let root = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&root).unwrap();
        for (name, bytes) in [
            (
                "checkpoint.json",
                serde_json::to_vec_pretty(&point).unwrap(),
            ),
            (
                "recording.json",
                serde_json::to_vec_pretty(&recording).unwrap(),
            ),
            ("default.jsonl", ordinary.canonical_trace()),
            ("extended.jsonl", extended.canonical_trace()),
            ("restored.jsonl", restored.canonical_trace()),
        ] {
            std::fs::write(root.join(format!("j7-timing-{name}")), bytes).unwrap();
        }
    }
}

#[test]
fn tcp_dns_and_dhcp_waits_scale_but_ping_cadence_and_dhcp_leases_do_not() {
    let mut tcp = engine(project(), 5);
    command(
        &mut tcp,
        "tcp",
        Command::Socket {
            device: id("host-a"),
            command: SocketCommand::Connect {
                socket: id("client"),
                owner: id("client"),
                destination: "192.168.1.254".parse().unwrap(),
                port: 80,
                local_address: None,
                local_port: None,
            },
        },
    );
    assert_eq!(
        times(&tcp, "/event/Network/event/action/TransportTimeout"),
        vec![2_500_000]
    );
    assert_eq!(
        times(&tcp, "/event/Network/event/action/ArpTimeout"),
        vec![5_000_000]
    );
    let services: Project =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/services.json")).unwrap();
    let lease = services
        .devices
        .iter()
        .find_map(|h| h.network.dhcp_server.as_ref())
        .unwrap()
        .lease_us
        .0;
    let mut dhcp = engine(services.clone(), 5);
    dhcp.run(10000, Some(0)).unwrap();
    assert_eq!(
        times(&dhcp, "/event/Network/event/action/DhcpTick"),
        vec![10_000_000, 10_000_000]
    );
    dhcp.run(10000, Some(4_000_000)).unwrap();
    let acquired: Vec<_> = dhcp
        .trace()
        .iter()
        .filter(|f| f.kind == "dhcp.lease.acquired")
        .collect();
    assert_eq!(acquired.len(), 2);
    for fact in acquired {
        assert_eq!(
            number(&fact.payload["assignment"]["expires_us"]) - fact.time_us.0,
            lease
        );
    }
    let mut static_services = services;
    for (index, h) in static_services
        .devices
        .iter_mut()
        .filter(|h| matches!(h.id.as_str(), "host-a" | "host-c"))
        .enumerate()
    {
        h.interfaces[0].ipv4 = Some(Ipv4Config::Static {
            address: format!("192.168.1.{}", 20 + index),
            prefix_length: 24,
        });
        h.network.dns_server = Some("192.168.1.254".into());
    }
    let mut apps = engine(static_services, 5);
    command(
        &mut apps,
        "dns",
        Command::App {
            device: id("host-a"),
            command: AppCommand::Terminal {
                application: id("host-a-terminal"),
                line: "nslookup absent.test".into(),
            },
        },
    );
    assert_eq!(
        times(&apps, "/event/Application/timer/kind/DnsTimeout"),
        vec![15_000_000]
    );
    command(
        &mut apps,
        "terminal-ping",
        Command::App {
            device: id("host-c"),
            command: AppCommand::Terminal {
                application: id("host-c-terminal"),
                line: "ping 192.168.1.254".into(),
            },
        },
    );
    apps.run(10000, Some(0)).unwrap();
    assert_eq!(
        times(&apps, "/event/Application/timer/kind/PingSend"),
        vec![1_000_000]
    );
}

#[test]
fn stop_restart_and_replacement_keep_policy_and_reject_incompatible_headers() {
    let mut e = engine(project(), 3);
    e.stop().unwrap();
    assert_eq!(e.network().wait_factor(), WaitFactor::try_from(3).unwrap());
    e.recording().unwrap().replay().unwrap();
    e.restart().unwrap();
    assert_eq!(
        e.recording().unwrap().wait_factor,
        WaitFactor::try_from(3).unwrap()
    );
    e.replace_project(project()).unwrap();
    assert_eq!(e.network().wait_factor(), WaitFactor::try_from(3).unwrap());
    let mut point = e.checkpoint().unwrap();
    point.checkpoint_version = 1;
    assert!(point.restore().is_err());
    let recording = e.recording().unwrap();
    for (version, profile, factor) in [
        (1, "sns-engine-semantics-v1", 3),
        (2, "sns-engine-semantics-v1", 3),
        (2, "sns-engine-semantics-v2", 1),
    ] {
        let mut bad = serde_json::to_value(&recording).unwrap();
        bad["recording_version"] = version.into();
        bad["engine_profile"] = profile.into();
        bad["wait_factor"] = factor.into();
        assert!(
            Recording::from_bytes(&serde_json::to_vec(&bad).unwrap())
                .unwrap()
                .replay()
                .is_err()
        );
    }
}

#[test]
fn tcp_time_wait_and_rip_cadence_remain_protocol_lifetimes() {
    let mut e = engine(project(), 5);
    command(
        &mut e,
        "listen",
        Command::Socket {
            device: id("host-b"),
            command: SocketCommand::Bind {
                socket: id("server"),
                owner: id("server"),
                protocol: TransportProtocol::Tcp,
                interface: None,
                address: None,
                port: 9000,
            },
        },
    );
    command(
        &mut e,
        "connect",
        Command::Socket {
            device: id("host-a"),
            command: SocketCommand::Connect {
                socket: id("client"),
                owner: id("client"),
                destination: "192.168.1.3".parse().unwrap(),
                port: 9000,
                local_address: None,
                local_port: None,
            },
        },
    );
    e.run(10000, Some(100_000)).unwrap();
    command(
        &mut e,
        "close",
        Command::Socket {
            device: id("host-a"),
            command: SocketCommand::Close {
                socket: id("client"),
                force: false,
            },
        },
    );
    e.run(10000, Some(200_000)).unwrap();
    let closed_at = e
        .trace()
        .iter()
        .find(|f| f.kind == "tcp.state" && f.payload["state"] == "TIME_WAIT")
        .unwrap()
        .time_us
        .0;
    assert!(
        times(&e, "/event/Network/event/action/TransportTimeout")
            .contains(&(closed_at + sns_protocols::transport::TIME_WAIT_US))
    );
    let rip: Project =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/rip.json")).unwrap();
    let mut rip = engine(rip, 5);
    rip.run(10000, Some(0)).unwrap();
    let periodic = times(&rip, "/event/Network/event/action/RipTick");
    assert!(!periodic.is_empty());
    assert!(
        periodic
            .iter()
            .all(|t| *t == sns_protocols::routing::RIP_PERIOD_US)
    );
}
