use sns_engine::{Engine, Limits, TraceEvent};
use sns_model::*;
use sns_trace::*;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn app(e: &mut Engine, host: &str, command: AppCommand) {
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(&format!("obs-{}-{}", e.revision(), e.trace().len())),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::App {
            device: id(host),
            command,
        },
    })
    .unwrap();
}
fn web() -> Engine {
    let p = serde_json::from_str(include_str!("../../../fixtures/valid/services.json")).unwrap();
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    for name in ["dns", "web"] {
        app(
            &mut e,
            "host-b",
            AppCommand::StartServer {
                application: id(name),
                port: 80,
            },
        );
    }
    e.run(100_000, Some(1_000_000)).unwrap();
    app(
        &mut e,
        "host-a",
        AppCommand::Navigate {
            application: id("host-a-browser"),
            url: "http://www.ecole.test/".into(),
            fields: None,
        },
    );
    e.run(100_000, Some(4_000_000)).unwrap();
    assert!(
        e.trace().iter().any(|e| e.kind.starts_with("http.")),
        "HTTP ran"
    );
    e
}
#[test]
fn combined_queries_export_exact_stable_ids_without_mutating_http_engine() {
    let e = web();
    let before = e.checkpoint_state().unwrap();
    let anchor = e
        .trace()
        .iter()
        .find(|e| e.kind == "ethernet.received" && has_protocol(e, "TCP"))
        .unwrap();
    let query = Query {
        device: Some(anchor.device_id.clone()),
        from_us: Some(anchor.time_us.0),
        through_us: Some(anchor.time_us.0),
        protocol: "TCP".into(),
        address: "192.168.1.3".into(),
        correlation: anchor.correlation_id.as_str().into(),
    };
    query.validate().unwrap();
    let selection: Vec<_> = e.trace().iter().filter(|e| query.matches(e)).collect();
    assert!(!selection.is_empty());
    assert!(selection.contains(&anchor));
    let jsonl = export(selection.iter().copied(), ExportFormat::Jsonl).unwrap();
    let decoded: Vec<TraceEvent> = std::str::from_utf8(&jsonl)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(
        decoded,
        selection.iter().map(|&e| e.clone()).collect::<Vec<_>>()
    );
    let mut excluded = query.clone();
    excluded.address = "192.168.1.30".into();
    assert!(!excluded.matches(anchor));
    excluded = query.clone();
    excluded.protocol = "UDP".into();
    assert!(!excluded.matches(anchor));
    excluded = query.clone();
    excluded.correlation.push('0');
    assert!(!excluded.matches(anchor));
    excluded = query.clone();
    excluded.from_us = Some(anchor.time_us.0 + 1);
    assert!(excluded.validate().is_err());
    assert!(!excluded.matches(anchor));
    assert_eq!(before, e.checkpoint_state().unwrap());
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_OBSERVATION") {
        let p = std::path::Path::new(&dir);
        std::fs::create_dir_all(p).unwrap();
        std::fs::write(p.join("j7-observation-selected.jsonl"), jsonl).unwrap();
        std::fs::write(
            p.join("j7-observation-selected.csv"),
            export(selection.iter().copied(), ExportFormat::Csv).unwrap(),
        )
        .unwrap();
        std::fs::write(
            p.join("j7-observation-web.jsonl"),
            export(e.trace(), ExportFormat::Jsonl).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn causal_journey_spans_four_layers_and_devices_and_preserves_snapshot() {
    let e = web();
    let before = e.checkpoint_state().unwrap();
    let selected = e
        .trace()
        .iter()
        .rev()
        .find(|e| e.kind == "http.response.received")
        .unwrap();
    let j = Journey::build(e.trace(), e.run_id(), &selected.event_id).unwrap();
    assert!(!j.incomplete);
    assert!(j.indices.len() > 10);
    let layers: std::collections::BTreeSet<_> =
        j.indices.iter().map(|&i| layer(&e.trace()[i])).collect();
    assert_eq!(layers, std::collections::BTreeSet::from([0, 1, 2, 3]));
    let devices: std::collections::BTreeSet<_> =
        j.indices.iter().map(|&i| &e.trace()[i].device_id).collect();
    assert!(devices.len() >= 3);
    let local = j.local_indices(e.trace(), &selected.device_id);
    assert!(local.len() < j.indices.len());
    assert!(
        local
            .iter()
            .all(|&i| e.trace()[i].device_id == selected.device_id)
    );
    assert!(
        j.indices
            .windows(2)
            .all(|w| e.trace()[w[0]].sequence.0 < e.trace()[w[1]].sequence.0)
    );
    let frame = e
        .trace()
        .iter()
        .find(|e| {
            e.kind == "ethernet.sent" && has_protocol(e, "TCP") && layer_parts(e)[3].is_some()
        })
        .unwrap();
    assert!(layer_parts(frame).iter().all(Option::is_some));
    assert_eq!(before, e.checkpoint_state().unwrap());
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_OBSERVATION") {
        std::fs::write(std::path::Path::new(&dir).join("j7-observation-path.json"),serde_json::to_vec_pretty(&serde_json::json!({"run_id":j.run_id,"selected":j.anchor,"indices":j.indices,"event_ids":j.indices.iter().map(|&i|&e.trace()[i].event_id).collect::<Vec<_>>(),"local_indices":local,"layers":layers})).unwrap()).unwrap();
    }
}
#[test]
fn journey_excludes_causal_siblings_other_runs_and_marks_missing_ancestors() {
    let e = web();
    let mut facts = e.trace()[0..4].to_vec();
    for (i, f) in facts.iter_mut().enumerate() {
        f.event_id = id(&format!("e{i}"));
        f.sequence = Decimal(i as u64 + 1);
        f.caused_by = None;
        f.correlation_id = id("same-flow");
    }
    facts[1].caused_by = Some(id("e0"));
    facts[2].caused_by = Some(id("e0"));
    facts[3].caused_by = Some(id("e1"));
    facts[3].run_id = id("resumed-generation");
    let mut other = facts[0].clone();
    other.run_id = id("other-run");
    facts.push(other);
    let j = Journey::build(&facts, e.run_id(), &id("e1")).unwrap();
    assert_eq!(j.indices, vec![0, 1, 3]);
    facts[0].caused_by = Some(id("missing"));
    assert!(
        Journey::build(&facts, e.run_id(), &id("e1"))
            .unwrap()
            .incomplete
    );
    facts[0].caused_by = Some(id("e3"));
    assert!(Journey::build(&facts, e.run_id(), &id("e1")).is_err());
    assert!(Journey::build(&facts, e.run_id(), &id("unknown")).is_err());
    let empty: Vec<TraceEvent> = vec![];
    assert_eq!(
        export(&empty, ExportFormat::Jsonl).unwrap(),
        Vec::<u8>::new()
    );
    assert_eq!(export(&empty,ExportFormat::Csv).unwrap(),b"run_id,event_id,event_schema_version,sequence,time_us,type,device_id,caused_by,correlation_id,payload_json\r\n");
}

#[test]
fn structured_exports_quote_unicode_and_reject_oversized_results() {
    let e = web();
    let mut fact = e.trace()[0].clone();
    fact.payload.insert(
        "note".into(),
        serde_json::json!("élève, \"réseau\"\r\n日本語"),
    );
    let jsonl = export([&fact], ExportFormat::Jsonl).unwrap();
    let csv = export([&fact], ExportFormat::Csv).unwrap();
    assert_eq!(serde_json::from_slice::<TraceEvent>(&jsonl).unwrap(), fact);
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_OBSERVATION") {
        let p = std::path::Path::new(&dir);
        std::fs::create_dir_all(p).unwrap();
        std::fs::write(p.join("j7-observation-escaping.jsonl"), &jsonl).unwrap();
        std::fs::write(p.join("j7-observation-escaping.csv"), &csv).unwrap();
    }
    fact.payload
        .insert("note".into(), serde_json::json!("x".repeat(8192)));
    for format in [ExportFormat::Jsonl, ExportFormat::Csv] {
        assert_eq!(
            export(std::iter::repeat_n(&fact, 10_000), format)
                .unwrap_err()
                .code,
            "resource.limit_exceeded"
        );
    }
}
