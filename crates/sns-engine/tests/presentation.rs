//! F-EQU-008: presentation changes preserve an already running switch and its queue.
use sns_engine::{Engine, Limits};
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
fn envelope(e: &Engine, name: &str, command: Command) -> CommandEnvelope {
    CommandEnvelope {
        api_version: 1,
        command_id: id(name),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command,
    }
}
#[test]
fn cloud_toggle_preserves_ports_sat_timers_and_live_communication_then_replays() {
    let mut e = engine();
    e.command(envelope(
        &e,
        "warm-switch",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    ))
    .unwrap();
    e.run(10000, Some(3_000_000)).unwrap();
    assert!(e.network().sat.len() >= 2);
    e.command(envelope(
        &e,
        "queued-ping",
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    ))
    .unwrap();
    e.step().unwrap();
    assert!(
        !e.checkpoint_state().unwrap()["frame_transit_budgets"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let ports = e
        .project()
        .device(&id("switch"))
        .unwrap()
        .interfaces
        .clone();
    let before = e.checkpoint_state().unwrap();
    let run = e.run_id().clone();
    let command = envelope(
        &e,
        "cloud",
        Command::SetSymbol {
            device: id("switch"),
            symbol: Some(DeviceSymbol::Cloud),
        },
    );
    e.command(command.clone()).unwrap();
    let after = e.checkpoint_state().unwrap();
    for field in [
        "network",
        "applications",
        "queue",
        "rng_state",
        "virtual_time_us",
    ] {
        assert_eq!(after[field], before[field], "{field}");
    }
    assert_eq!(e.project().schema_version, 8);
    assert_eq!(e.project().device(&id("switch")).unwrap().interfaces, ports);
    assert_eq!(e.run_id(), &run);
    assert_eq!(
        e.project().lan[&id("switch")].symbol,
        Some(DeviceSymbol::Cloud)
    );
    let duplicate = e.checkpoint_state().unwrap();
    e.command(command).unwrap();
    assert_eq!(duplicate, e.checkpoint_state().unwrap());
    let recovered = e.checkpoint().unwrap().restore().unwrap();
    assert_eq!(recovered.project(), e.project());
    assert_eq!(
        recovered.checkpoint_state().unwrap()["network"],
        after["network"]
    );
    let until = e.now() + 3_000_000;
    e.run(10000, Some(until)).unwrap();
    assert_eq!(
        e.trace()
            .iter()
            .filter(|t| t.kind == "icmp.echo.reply")
            .count(),
        2
    );
    let before = e.checkpoint_state().unwrap();
    e.command(envelope(
        &e,
        "switch-again",
        Command::SetSymbol {
            device: id("switch"),
            symbol: None,
        },
    ))
    .unwrap();
    assert_eq!(e.checkpoint_state().unwrap()["network"], before["network"]);
    assert_eq!(e.project().lan[&id("switch")].symbol, None);
    assert_eq!(e.project().schema_version, 8);
    let replay = e.recording().unwrap().replay().unwrap();
    assert_eq!(
        replay.checkpoint_state().unwrap(),
        e.checkpoint_state().unwrap()
    );
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_CLOUD") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("j7-cloud-trace.jsonl"), e.canonical_trace()).unwrap();
        std::fs::write(
            dir.join("j7-cloud-recording.json"),
            serde_json::to_vec(&e.recording().unwrap()).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn invalid_target_and_full_trace_leave_the_entire_state_unchanged() {
    let mut e = engine();
    let before = e.checkpoint_state().unwrap();
    assert_eq!(
        e.command(envelope(
            &e,
            "bad-host",
            Command::SetSymbol {
                device: id("host-a"),
                symbol: Some(DeviceSymbol::Cloud)
            }
        ))
        .unwrap_err()
        .code,
        "presentation.invalid_symbol"
    );
    assert_eq!(before, e.checkpoint_state().unwrap());
    assert!(
        e.command(envelope(
            &e,
            "missing",
            Command::SetSymbol {
                device: id("missing"),
                symbol: None
            }
        ))
        .is_err()
    );
    assert_eq!(before, e.checkpoint_state().unwrap());
    let p = e.project().clone();
    let mut e = Engine::from_project(
        p,
        42,
        Limits {
            trace: 0,
            ..Limits::default()
        },
    )
    .unwrap();
    let before = e.checkpoint_state().unwrap();
    assert_eq!(
        e.command(envelope(
            &e,
            "no-space",
            Command::SetSymbol {
                device: id("switch"),
                symbol: Some(DeviceSymbol::Cloud)
            }
        ))
        .unwrap_err()
        .code,
        "resource.limit_exceeded"
    );
    assert_eq!(before, e.checkpoint_state().unwrap());
}
