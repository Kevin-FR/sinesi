//! R-REPRISE: real engine state, reachable-state verification and new-generation command isolation.
use sns_engine::{Checkpoint, Engine, Limits};
use sns_model::*;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn engine() -> Engine {
    Engine::from_project(
        serde_json::from_str(include_str!("../../../fixtures/valid/lan.json")).unwrap(),
        42,
        Limits::default(),
    )
    .unwrap()
}
fn cmd(e: &mut Engine, name: &str, command: Command) -> Result<(), Diagnostic> {
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(name),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command,
    })
}
#[test]
fn checkpoint_preserves_in_flight_frames_suffix_and_rejects_old_generation() {
    let mut original = engine();
    cmd(
        &mut original,
        "ping-before-checkpoint",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    )
    .unwrap();
    original.step().unwrap();
    let state = original.checkpoint_state().unwrap();
    assert!(
        !state["frame_transit_budgets"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let point = original.checkpoint().unwrap();
    let bytes = serde_json::to_vec(&point).unwrap();
    let point = Checkpoint::from_bytes(&bytes).unwrap();
    let mut restored = point.restore().unwrap();
    assert_ne!(restored.run_id(), original.run_id());
    assert_eq!(restored.canonical_trace(), original.canonical_trace());
    let before = restored.checkpoint_state().unwrap();
    let err = restored
        .command(CommandEnvelope {
            api_version: 1,
            command_id: id("late-input"),
            project_id: restored.project().project_id.clone(),
            run_id: original.run_id().clone(),
            expected_revision: Decimal(restored.revision()),
            command: Command::ManualLoss { enabled: true },
        })
        .err()
        .unwrap();
    assert_eq!(err.code, "state.stale_run");
    assert_eq!(before, restored.checkpoint_state().unwrap());
    original.run(10000, Some(4_000_000)).unwrap();
    restored.run(10000, Some(4_000_000)).unwrap();
    assert_eq!(restored.canonical_trace(), original.canonical_trace());
    assert_eq!(
        restored.checkpoint_state().unwrap()["network"]["arp"],
        original.checkpoint_state().unwrap()["network"]["arp"]
    );
    // A checkpoint after a previous restoration retains its original provenance and generation operations.
    let point = restored.checkpoint().unwrap();
    let again = point.restore().unwrap();
    assert_eq!(again.canonical_trace(), restored.canonical_trace());
}
#[test]
fn recording_replays_control_commands_idempotency_and_external_output_drains() {
    let mut e = engine();
    cmd(&mut e, "resume", Command::Resume).unwrap();
    e.pump(0, None).unwrap();
    cmd(
        &mut e,
        "ping",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    )
    .unwrap();
    for _ in 0..4 {
        e.step().unwrap();
    }
    e.take_bridge_output();
    let recording = e.recording().unwrap();
    let replayed = recording.replay().unwrap();
    assert_eq!(
        e.checkpoint_state().unwrap(),
        replayed.checkpoint_state().unwrap()
    );
    e.stop().unwrap();
    let replayed = e.recording().unwrap().replay().unwrap();
    assert_eq!(
        e.checkpoint_state().unwrap(),
        replayed.checkpoint_state().unwrap()
    );
}
#[test]
fn corrupt_incompatible_and_forged_reachable_state_are_rejected() {
    use sha2::{Digest, Sha256};
    let e = engine();
    let mut p = e.checkpoint().unwrap();
    p.checkpoint_version = 99;
    assert_eq!(p.restore().err().unwrap().code, "checkpoint.incompatible");
    let mut p = e.checkpoint().unwrap();
    p.runtime_state["virtual_time_us"] = "12".into();
    assert_eq!(p.restore().err().unwrap().code, "checkpoint.corrupt");
    p.runtime_sha256 = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&p.runtime_state).unwrap())
    );
    assert_eq!(p.restore().err().unwrap().code, "checkpoint.incompatible");
    assert!(Checkpoint::from_bytes(b"{}").is_err());
}
fn app(e: &mut Engine, name: &str, host: &str, command: AppCommand) {
    cmd(
        e,
        name,
        Command::App {
            device: id(host),
            command,
        },
    )
    .unwrap();
}
fn rich_engine() -> Engine {
    let mut p: Project =
        serde_json::from_str(include_str!("../../../fixtures/valid/private.json")).unwrap();
    p.devices
        .iter_mut()
        .find(|d| d.id == id("host-a"))
        .unwrap()
        .interfaces[0]
        .ipv4 = Some(Ipv4Config::Dhcp);
    let gateway = p
        .devices
        .iter_mut()
        .find(|d| d.id == id("gateway"))
        .unwrap();
    gateway.network.dhcp_server = Some(Box::new(DhcpConfig {
        interface: id("gateway-lan"),
        pool_start: "192.168.10.20".parse().unwrap(),
        pool_end: "192.168.10.30".parse().unwrap(),
        prefix_length: 24,
        gateway: Some("192.168.10.1".parse().unwrap()),
        dns: Some("198.51.100.10".parse().unwrap()),
        use_host_options: false,
        reservations: Default::default(),
        lease_us: Decimal(30_000_000),
    }));
    p.devices
        .iter_mut()
        .find(|d| d.id == id("uplink"))
        .unwrap()
        .network
        .rip = true;
    p.devices
        .iter_mut()
        .find(|d| d.id == id("host-a"))
        .unwrap()
        .applications
        .push(Application {
            id: id("checkpoint-client"),
            kind: "tcp_client".into(),
            config: Default::default(),
            service: None,
            data: None,
        });
    p.devices
        .iter_mut()
        .find(|d| d.id == id("host-c"))
        .unwrap()
        .applications
        .push(Application {
            id: id("checkpoint-dns"),
            kind: "dns_server".into(),
            config: Default::default(),
            service: Some(ServiceConfig::Dns(DnsConfig {
                records: vec![DnsRecord {
                    name: "lesson.test".into(),
                    record_type: DnsType::A,
                    value: "198.51.100.10".into(),
                    ttl_seconds: 5,
                    priority: 0,
                }],
                recursive: false,
                upstream: None,
            })),
            data: None,
        });
    p.validate().unwrap();
    let mut e = Engine::from_project(p, 72, Limits::default()).unwrap();
    e.run(10000, Some(6_000_000)).unwrap();
    assert!(e.network().dhcp.assigned.contains_key(&id("host-a-p0")));
    app(
        &mut e,
        "start-echo",
        "host-c",
        AppCommand::StartServer {
            application: id("echo"),
            port: 7777,
        },
    );
    app(
        &mut e,
        "start-dns",
        "host-c",
        AppCommand::StartServer {
            application: id("checkpoint-dns"),
            port: 53,
        },
    );
    app(
        &mut e,
        "resolve",
        "host-a",
        AppCommand::Terminal {
            application: id("term-0"),
            line: "ping -n 1 lesson.test".into(),
        },
    );
    app(
        &mut e,
        "connect",
        "host-a",
        AppCommand::Connect {
            application: id("checkpoint-client"),
            target: "198.51.100.10".into(),
            port: 7777,
        },
    );
    let until = e.now() + 2_000_000;
    e.run(10000, Some(until)).unwrap();
    e
}
#[test]
fn checkpoint_restores_active_tcp_rip_dhcp_dns_nat_and_all_expiration_suffixes() {
    let mut original = rich_engine();
    assert!(
        original
            .network()
            .transport
            .views()
            .iter()
            .any(|s| s.state == "ESTABLISHED")
    );
    assert!(!original.network().nat.entries.is_empty());
    assert!(original.network().routing.started.contains(&id("uplink")));
    let state = original.checkpoint_state().unwrap();
    assert!(
        !state["applications"]["dns"]["cache"]["$entries"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        state["queue"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.pointer("/event/Network/event/action/RipTick").is_some())
    );
    let point = original.checkpoint().unwrap();
    let checkpoint_bytes = serde_json::to_vec(&point).unwrap();
    let mut restored = Checkpoint::from_bytes(&checkpoint_bytes)
        .unwrap()
        .restore()
        .unwrap();
    let prefix = original.canonical_trace();
    assert_eq!(restored.canonical_trace(), prefix);
    original.run(100000, Some(95_000_000)).unwrap();
    restored.run(100000, Some(95_000_000)).unwrap();
    let queries_before = original
        .trace()
        .iter()
        .filter(|t| t.kind == "dns.query.sent")
        .count();
    let hits_before = original
        .trace()
        .iter()
        .filter(|t| t.kind == "dns.cache.hit")
        .count();
    app(
        &mut original,
        "resolve-expired",
        "host-a",
        AppCommand::Terminal {
            application: id("term-0"),
            line: "ping -n 1 lesson.test".into(),
        },
    );
    app(
        &mut restored,
        "resolve-expired",
        "host-a",
        AppCommand::Terminal {
            application: id("term-0"),
            line: "ping -n 1 lesson.test".into(),
        },
    );
    let until = original.now() + 2_000_000;
    original.run(10000, Some(until)).unwrap();
    restored.run(10000, Some(until)).unwrap();
    assert_eq!(original.canonical_trace(), restored.canonical_trace());
    assert!(
        original
            .trace()
            .iter()
            .filter(|t| t.kind == "dns.query.sent")
            .count()
            > queries_before
    );
    assert_eq!(
        original
            .trace()
            .iter()
            .filter(|t| t.kind == "dns.cache.hit")
            .count(),
        hits_before
    );
    assert!(
        original
            .trace()
            .iter()
            .any(|t| t.kind == "nat.mapping.expired")
    );
    let a = original.checkpoint_state().unwrap();
    let b = restored.checkpoint_state().unwrap();
    assert_eq!(a["network"], b["network"]);
    assert_eq!(a["applications"], b["applications"]);
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_CHECKPOINT") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("j7-active-checkpoint.json"), checkpoint_bytes).unwrap();
        std::fs::write(
            dir.join("j7-reprise-original.jsonl"),
            original.canonical_trace(),
        )
        .unwrap();
        std::fs::write(
            dir.join("j7-reprise-restored.jsonl"),
            restored.canonical_trace(),
        )
        .unwrap();
    }
}

#[test]
fn checkpoint_replays_new_inputs_and_merged_event_tail_in_fresh_generation() {
    let mut e = engine();
    cmd(
        &mut e,
        "initial",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    )
    .unwrap();
    e.step().unwrap();
    let point = e.checkpoint().unwrap();
    e.run(10000, Some(4_000_000)).unwrap();
    cmd(
        &mut e,
        "following",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    )
    .unwrap();
    e.run(10000, Some(8_000_000)).unwrap();
    let full = e.recording().unwrap();
    let restored = point.replay_remaining(&full).unwrap();
    assert_ne!(e.run_id(), restored.run_id());
    assert_eq!(e.canonical_trace(), restored.canonical_trace());
    assert_eq!(
        e.checkpoint_state().unwrap()["network"],
        restored.checkpoint_state().unwrap()["network"]
    );
    let offline = restored.recording().unwrap().replay().unwrap();
    assert_eq!(
        offline.checkpoint_state().unwrap(),
        restored.checkpoint_state().unwrap()
    );
    let mut wrong = full;
    wrong.seed = Decimal(99);
    assert_eq!(
        point.replay_remaining(&wrong).err().unwrap().code,
        "checkpoint.invalid_recording"
    );
}

#[test]
fn scheduled_network_actions_survive_checkpoint_and_invalid_actions_are_atomic() {
    let scenario: Scenario = serde_json::from_str(include_str!(
        "../../../fixtures/valid/network-scenario.json"
    ))
    .unwrap();
    let mut original = Engine::new(scenario.clone(), None, Limits::default()).unwrap();
    original.run(1000, Some(1500)).unwrap();
    let point = original.checkpoint().unwrap();
    assert!(
        serde_json::to_string(&point.runtime_state["queue"])
            .unwrap()
            .contains("lesson-ping-second")
    );
    let mut recovered = point.restore().unwrap();
    original.run(10000, Some(10_000_000)).unwrap();
    recovered.run(10000, Some(10_000_000)).unwrap();
    assert_eq!(original.canonical_trace(), recovered.canonical_trace());
    assert_eq!(
        original
            .trace()
            .iter()
            .filter(|t| t.kind == "icmp.echo.reply")
            .count(),
        2
    );
    let offline = original.recording().unwrap().replay().unwrap();
    assert_eq!(
        offline.checkpoint_state().unwrap(),
        original.checkpoint_state().unwrap()
    );
    let mut bad = scenario.clone();
    bad.network_actions[1].id = bad.network_actions[0].id.clone();
    assert_eq!(bad.validate().unwrap_err().code, "scenario.invalid_action");
    let mut bad = scenario.clone();
    bad.network_actions[1].at_us = Decimal(0);
    assert!(bad.validate().is_err());
    let mut bad = scenario.clone();
    bad.network_actions[0].command = Command::Stop;
    assert!(bad.validate().is_err());
    let mut bad = scenario;
    bad.network_actions[0].command = Command::Ping {
        source: id("missing-host"),
        destination: "192.168.1.3".into(),
        ttl: 64,
    };
    let mut e = Engine::new(bad, None, Limits::default()).unwrap();
    let before = e.checkpoint_state().unwrap();
    assert!(e.step().is_err());
    assert_eq!(before, e.checkpoint_state().unwrap());
}
