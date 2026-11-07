use sns_engine::{Engine, Limits, Termination};
use sns_model::*;

fn scenario() -> Scenario {
    serde_json::from_slice(include_bytes!("../../../fixtures/valid/scenario.json")).unwrap()
}
fn engine(seed: u64) -> Engine {
    Engine::new(scenario(), Some(seed), Limits::default()).unwrap()
}
fn envelope(engine: &Engine, id: &str, command: Command) -> CommandEnvelope {
    CommandEnvelope {
        api_version: 1,
        command_id: Id::new(id).unwrap(),
        project_id: engine.project().project_id.clone(),
        expected_revision: Decimal(0),
        run_id: engine.run_id().clone(),
        command,
    }
}

#[test]
fn fresh_executions_are_byte_identical_with_seeded_jitter() {
    let mut first = engine(42);
    let mut second = engine(42);
    let a = first.run(100, None).unwrap();
    let b = second.run(100, None).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.termination, Termination::Idle);
    assert_eq!(first.canonical_trace(), second.canonical_trace());
    assert_eq!(
        first.canonical_trace(),
        include_bytes!("../../../fixtures/valid/trace.jsonl")
    );
    let mut different = engine(43);
    different.run(100, None).unwrap();
    assert_ne!(first.canonical_trace(), different.canonical_trace());
}
#[test]
fn stepping_and_bounded_execution_match_uninterrupted_execution() {
    let mut uninterrupted = engine(42);
    uninterrupted.run(100, None).unwrap();
    let mut stepped = engine(42);
    assert_eq!(stepped.now(), 0);
    while stepped.step().unwrap() {}
    assert_eq!(uninterrupted.canonical_trace(), stepped.canonical_trace());
    let mut bounded = engine(42);
    assert_eq!(
        bounded.run(100, Some(999)).unwrap().termination,
        Termination::TimeBound
    );
    assert_eq!(bounded.trace().len(), 1);
    assert_eq!(
        bounded.run(1, None).unwrap().termination,
        Termination::EventBudget
    );
    bounded.run(100, None).unwrap();
    assert_eq!(uninterrupted.canonical_trace(), bounded.canonical_trace());
}
#[test]
fn stable_order_is_independent_of_project_container_order() {
    let mut input = scenario();
    input.project.devices.reverse();
    input.project.links.reverse();
    let mut shuffled = Engine::new(input, Some(42), Limits::default()).unwrap();
    let mut baseline = engine(42);
    shuffled.run(100, None).unwrap();
    baseline.run(100, None).unwrap();
    assert_eq!(baseline.canonical_trace(), shuffled.canonical_trace());
}
#[test]
fn delivery_priority_precedes_simultaneous_dispatch_and_fifo_breaks_ties() {
    let mut input = scenario();
    input.actions.truncate(1);
    input.project.links[0].jitter_us = Decimal(0);
    let mut action = input.actions[0].clone();
    action.at_us = Decimal(1000);
    input.actions.push(action.clone());
    input.actions.push(action);
    let mut engine = Engine::new(input, None, Limits::default()).unwrap();
    engine.run(100, None).unwrap();
    let at_1000: Vec<_> = engine
        .trace()
        .iter()
        .filter(|e| e.time_us.0 == 1000)
        .collect();
    assert_eq!(at_1000[0].kind, "prototype.message.delivered");
    assert_eq!(at_1000[1].correlation_id.as_str(), "flow-1");
    assert_eq!(at_1000[2].correlation_id.as_str(), "flow-2");
}
#[test]
fn quota_and_time_overflow_fail_without_consuming_an_event_or_randomness() {
    let mut limited = Engine::new(
        scenario(),
        None,
        Limits {
            trace: 0,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(limited.step().unwrap_err().code, "resource.limit_exceeded");
    assert_eq!(
        (limited.now(), limited.pending(), limited.trace().len()),
        (0, 2, 0)
    );
    let mut input = scenario();
    input.actions.truncate(1);
    input.actions[0].at_us = Decimal(u64::MAX);
    let mut overflow = Engine::new(input, None, Limits::default()).unwrap();
    assert_eq!(
        overflow.step().unwrap_err().code,
        "simulation.time_overflow"
    );
    assert_eq!(
        (overflow.now(), overflow.pending(), overflow.trace().len()),
        (0, 1, 0)
    );
}
#[test]
fn rejected_stale_and_duplicate_commands_are_atomic_and_stop_cancels_pending_work() {
    let mut engine = engine(42);
    let step = envelope(&engine, "step-1", Command::Step);
    engine.command(step.clone()).unwrap();
    engine.command(step.clone()).unwrap();
    assert_eq!(engine.trace().len(), 1);
    let mut conflict = step;
    conflict.command = Command::Stop;
    assert_eq!(
        engine.command(conflict).unwrap_err().code,
        "command.id_conflict"
    );
    let mut stale = envelope(&engine, "stale", Command::Step);
    stale.run_id = Id::new("old-run").unwrap();
    assert_eq!(engine.command(stale).unwrap_err().code, "state.stale_run");
    assert_eq!(engine.trace().len(), 1);
    let previous = envelope(&engine, "old-generation", Command::Step);
    engine
        .command(envelope(&engine, "stop", Command::Stop))
        .unwrap();
    assert_eq!(engine.pending(), 0);
    assert_eq!(
        engine.run(100, None).unwrap().termination,
        Termination::Stopped
    );
    engine.restart().unwrap();
    assert_eq!(
        engine.command(previous).unwrap_err().code,
        "state.stale_run"
    );
    assert_eq!(engine.trace().len(), 0);
}
#[test]
fn prototype_http_get_image_and_post_are_scheduled_messages() {
    let mut input = scenario();
    input.actions.clear();
    for path in ["/", "/logo.png", "/echo"] {
        input.actions.push(ScheduledMessage {
            at_us: Decimal(0),
            source: Id::new("client").unwrap(),
            target: Id::new("server").unwrap(),
            message: Message::HttpRequest {
                method: if path == "/echo" {
                    HttpMethod::Post
                } else {
                    HttpMethod::Get
                },
                path: path.into(),
                fields: std::collections::BTreeMap::from([("message".into(), "Bonjour".into())]),
            },
        });
    }
    let mut engine = Engine::new(input, None, Limits::default()).unwrap();
    engine.run(100, None).unwrap();
    let responses: Vec<_> = engine
        .trace()
        .iter()
        .filter(|event| {
            event.kind == "prototype.message.delivered" && event.device_id.as_str() == "client"
        })
        .collect();
    assert_eq!(responses.len(), 3);
    assert!(
        responses
            .iter()
            .any(|e| e.payload["message"]["media_type"] == "image/png")
    );
    assert!(
        responses
            .iter()
            .all(|e| e.payload["message"]["status"] == 200)
    );
}
