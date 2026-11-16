//! F-SIM-008: real frame loss/delay, isolation, deterministic seeds, checkpoint and replay.
use sns_engine::{Checkpoint, Engine, Limits};
use sns_model::*;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn scenario(loss: u32) -> Scenario {
    let mut s: Scenario = serde_json::from_str(include_str!(
        "../../../fixtures/valid/link-conditions-scenario.json"
    ))
    .unwrap();
    if let Command::ConfigureLink { conditions, .. } = &mut s.network_actions[0].command {
        conditions.loss_ppm = loss;
    }
    s
}
fn execute(s: Scenario) -> Engine {
    let mut e = Engine::new(s, None, Limits::default()).unwrap();
    e.run(100000, Some(50_000_000)).unwrap();
    e
}
fn reference_transits(e: &Engine) -> Vec<serde_json::Value> {
    e.trace().iter().filter(|t|t.kind=="simulation.link.transit"&&t.payload["link"]=="link-cd").map(|t|serde_json::json!({"time":t.time_us,"delay":t.payload["delay_us"],"sample":t.payload["sample_ppm"],"trial":t.payload["trial"]})).collect()
}
fn replies(e: &Engine, source: &str) -> usize {
    e.trace()
        .iter()
        .filter(|t| t.kind == "icmp.echo.reply" && t.device_id == id(source))
        .count()
}
#[test]
fn hundred_percent_blocks_only_one_link_zero_restores_and_streams_are_isolated() {
    let blocked = execute(scenario(1_000_000));
    let normal = execute(scenario(0));
    assert_eq!(replies(&blocked, "host-a"), 0);
    assert_eq!(replies(&blocked, "host-c"), 16);
    assert_eq!(replies(&normal, "host-a"), 16);
    assert_eq!(replies(&normal, "host-c"), 16);
    assert_eq!(reference_transits(&blocked), reference_transits(&normal));
    assert!(blocked.trace().iter().any(|t| t.kind == "ethernet.dropped"
        && t.payload.get("reason") == Some(&"simulation.link_loss".into())));
    assert!(
        blocked
            .trace()
            .iter()
            .filter(|t| t.kind == "ethernet.dropped")
            .all(|t| t.payload["link"] == "link-ab")
    );
    for e in [&blocked, &normal] {
        for t in e
            .trace()
            .iter()
            .filter(|t| t.kind == "simulation.link.transit")
        {
            let delay = t.payload["delay_us"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap();
            assert!((2000..=3000).contains(&delay));
            if !t.payload["dropped"].as_bool().unwrap() {
                let rx = e
                    .trace()
                    .iter()
                    .find(|rx| {
                        rx.kind == "ethernet.received" && rx.caused_by.as_ref() == Some(&t.event_id)
                    })
                    .unwrap();
                assert_eq!(rx.time_us.0 - t.time_us.0, delay);
            }
        }
    }
}
#[test]
fn seeded_partial_loss_replays_and_resumes_with_future_actions_and_rng_state() {
    let mut original = Engine::new(scenario(500000), None, Limits::default()).unwrap();
    original.run(100000, Some(5_000_000)).unwrap();
    let point = original.checkpoint().unwrap();
    assert_eq!(
        point.runtime_state["network"]["link_conditions"]["$entries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let mut restored = point.restore().unwrap();
    original.run(100000, Some(50_000_000)).unwrap();
    restored.run(100000, Some(50_000_000)).unwrap();
    assert_eq!(original.canonical_trace(), restored.canonical_trace());
    assert_eq!(
        original.checkpoint_state().unwrap()["network"],
        restored.checkpoint_state().unwrap()["network"]
    );
    let replay = original.recording().unwrap().replay().unwrap();
    assert_eq!(
        replay.checkpoint_state().unwrap(),
        original.checkpoint_state().unwrap()
    );
    let continuation = point
        .replay_remaining(&original.recording().unwrap())
        .unwrap();
    assert_eq!(continuation.canonical_trace(), original.canonical_trace());
    assert_eq!(
        original.canonical_trace(),
        execute(scenario(500000)).canonical_trace()
    );
    let mut reordered = scenario(500000);
    reordered.project.devices.reverse();
    reordered.project.links.reverse();
    assert_eq!(
        original.canonical_trace(),
        execute(reordered).canonical_trace()
    );
    let mut different = scenario(500000);
    different.seed = Decimal(43);
    assert_ne!(
        reference_transits(&original),
        reference_transits(&execute(different))
    );
    assert!(
        original
            .trace()
            .iter()
            .any(|t| t.kind == "simulation.link.transit" && t.payload["dropped"] == true)
    );
    assert!(replies(&original, "host-a") > 0);
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_LINKS") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("j7-links-checkpoint.json"),
            serde_json::to_vec(&point).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("j7-links-recording.json"),
            serde_json::to_vec(&original.recording().unwrap()).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("j7-links-original.jsonl"),
            original.canonical_trace(),
        )
        .unwrap();
        std::fs::write(
            dir.join("j7-links-restored.jsonl"),
            restored.canonical_trace(),
        )
        .unwrap();
    }
}
#[test]
fn invalid_configuration_is_atomic_live_repair_works_and_earlier_checkpoints_remain_readable() {
    let point = Checkpoint::from_bytes(include_bytes!(
        "../../../fixtures/valid/checkpoint-v1-unconfigured.json"
    ))
    .unwrap();
    let mut e = point.restore().unwrap();
    let issue = |e: &mut Engine, name: &str, link: &str, loss, delay| {
        e.command(CommandEnvelope {
            api_version: 1,
            command_id: id(name),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::ConfigureLink {
                link: id(link),
                conditions: LinkConditions {
                    delay_us: Decimal(delay),
                    jitter_us: Decimal(0),
                    loss_ppm: loss,
                },
            },
        })
    };
    for (name, link, loss, delay) in [
        ("bad-loss", "cable-host-a", 1000001, 1000),
        ("bad-delay", "cable-host-a", 0, 60000001),
        ("bad-link", "missing", 0, 1000),
    ] {
        let before = e.checkpoint_state().unwrap();
        assert!(issue(&mut e, name, link, loss, delay).is_err());
        assert_eq!(before, e.checkpoint_state().unwrap());
    }
    issue(&mut e, "block", "cable-host-a", 1000000, 1000).unwrap();
    e.run(10000, Some(8_000_000)).unwrap();
    assert_eq!(replies(&e, "host-a"), 0);
    issue(&mut e, "repair", "cable-host-a", 0, 1000).unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("ping-repaired"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    })
    .unwrap();
    let bound = e.now() + 4_000_000;
    e.run(10000, Some(bound)).unwrap();
    assert_eq!(replies(&e, "host-a"), 1);
    assert_eq!(
        e.recording().unwrap().replay().unwrap().canonical_trace(),
        e.canonical_trace()
    );
}

#[test]
fn failed_event_does_not_consume_a_random_trial_or_queue_boundary() {
    let mut e = Engine::new(
        scenario(500000),
        None,
        Limits {
            trace: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    // Two configuration actions, then two scheduled ping commands.
    // The following network emission is the boundary that consumes a random trial.
    for _ in 0..4 {
        e.step().unwrap();
    }
    let before = e.checkpoint_state().unwrap();
    for _ in 0..2 {
        assert_eq!(e.step().err().unwrap().code, "resource.limit_exceeded");
        assert_eq!(before, e.checkpoint_state().unwrap());
    }
    let offline = e.recording().unwrap().replay().unwrap();
    assert_eq!(before, offline.checkpoint_state().unwrap());
}
