use sns_engine::{Engine, Limits};
use sns_i18n::Language;
use sns_model::*;
use sns_report::{Report, ReportOptions, Snapshot, render_pdf};
use std::sync::Arc;
fn project() -> Project {
    sns_project::parse_project(include_bytes!(
        "../../../fixtures/valid/report-network.json"
    ))
    .unwrap()
}
fn snapshot() -> Snapshot {
    let mut e = Engine::from_project(project(), 42, Limits::default()).unwrap();
    e.run(100_000, Some(1_000_000)).unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: Id::new("report-ping").unwrap(),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::Ping {
            source: Id::new("a").unwrap(),
            destination: "192.168.30.2".into(),
            ttl: 64,
        },
    })
    .unwrap();
    e.run(100_000, Some(2_000_000)).unwrap();
    let capture = Snapshot::capture(&e);
    assert!(
        capture.routes[&Id::new("a").unwrap()]
            .iter()
            .any(|r| r.origin == sns_protocols::routing::Origin::Default)
    );
    capture
}
#[test]
fn routed_dns_report_is_deterministic_frozen_and_localized_in_both_views() {
    use sha2::{Digest, Sha256};
    let snapshot = snapshot();
    let before = serde_json::to_vec(snapshot.project.as_ref()).unwrap();
    let expected = snapshot
        .events
        .iter()
        .filter(|e| matches!(e.kind.as_str(), "ethernet.sent" | "ethernet.received"))
        .count();
    assert!(expected > 10);
    for language in Language::ALL {
        for diagram in [false, true] {
            let report = Report::build(&snapshot, ReportOptions { language, diagram }).unwrap();
            assert_eq!(report.packet_rows, expected);
            assert!(report.interface_groups >= 6);
            assert!(!report.trace_partial);
            let first = render_pdf(&report).unwrap();
            let second = render_pdf(&report).unwrap();
            assert_eq!(first.bytes, second.bytes);
            assert!(first.pages >= 4);
            assert!(first.bytes.starts_with(b"%PDF-"));
            if let Ok(dir) = std::env::var("SNS_RECORD_J7_REPORT") {
                let name = format!(
                    "j7-report-{}-{}",
                    language.code(),
                    if diagram { "diagram" } else { "icons" }
                );
                let base = std::path::Path::new(&dir).join(name);
                std::fs::write(base.with_extension("pdf"), &first.bytes).unwrap();
                std::fs::write(base.with_extension("json"),serde_json::to_vec_pretty(&serde_json::json!({"language":language.code(),"diagram":diagram,"pages":first.pages,"packet_rows":first.packet_rows,"interface_groups":first.interface_groups,"trace_partial":first.trace_partial,"bytes":first.bytes.len(),"sha256":format!("{:x}",Sha256::digest(&first.bytes)),"required_text":[language.text("report_overview"),language.text("report_configuration"),language.text("report_interfaces"),language.text("software"),language.text("report_dns"),language.text("report_routes"),language.text("report_traffic"),"ecole.test","192.168.30.2","report-ping","DHCP","192.168.10.20/24"],"forbidden_text":["secret-password-sentinel"]})).unwrap()).unwrap();
            }
        }
    }
    assert_eq!(
        serde_json::to_vec(snapshot.project.as_ref()).unwrap(),
        before
    );
}
#[test]
fn packet_tables_paginate_without_dropping_rows_and_partial_history_is_explicit() {
    let mut snapshot = snapshot();
    let template = snapshot
        .events
        .iter()
        .find(|e| e.kind == "ethernet.sent")
        .unwrap()
        .clone();
    let mut rows = Vec::new();
    for i in 0..1200 {
        let mut event = template.clone();
        event.sequence = Decimal(i + 50);
        event.time_us = Decimal(i);
        event.event_id = Id::new(format!("packet-{i}")).unwrap();
        rows.push(event);
    }
    snapshot.events = Arc::new(rows);
    let report = Report::build(&snapshot, ReportOptions::default()).unwrap();
    assert_eq!(report.packet_rows, 1200);
    assert_eq!(report.interface_groups, 1);
    assert!(report.trace_partial);
    assert!(report.page_count() > 30);
    let pdf = render_pdf(&report).unwrap();
    assert!(pdf.bytes.len() < 64 * 1024 * 1024);
    if let Ok(dir) = std::env::var("SNS_RECORD_J7_REPORT") {
        use sha2::{Digest, Sha256};
        let base = std::path::Path::new(&dir).join("j7-report-pagination");
        std::fs::write(base.with_extension("pdf"), &pdf.bytes).unwrap();
        std::fs::write(base.with_extension("json"),serde_json::to_vec_pretty(&serde_json::json!({"language":"fr","diagram":false,"pages":pdf.pages,"packet_rows":pdf.packet_rows,"interface_groups":pdf.interface_groups,"trace_partial":true,"bytes":pdf.bytes.len(),"sha256":format!("{:x}",Sha256::digest(&pdf.bytes)),"required_text":[Language::Fr.text("report_traffic"),Language::Fr.text("report_partial_trace"),"#50","#1249"],"forbidden_text":["secret-password-sentinel"]})).unwrap()).unwrap();
    }
}
#[test]
fn invalid_input_and_quota_are_errors_before_pdf_output() {
    let mut s = snapshot();
    s.events = Arc::new(s.events.iter().rev().cloned().collect());
    assert_eq!(
        Report::build(&s, ReportOptions::default())
            .err()
            .unwrap()
            .code,
        "report.invalid_snapshot"
    );
    let mut s = snapshot();
    s.events = Arc::new(vec![s.events[0].clone(); 100_001]);
    assert!(Report::build(&s, ReportOptions::default()).is_err());
    let mut p = project();
    p.annotations[0].text = "\u{10ffff}".into();
    assert!(Report::build(&Snapshot::configured(p), ReportOptions::default()).is_err());
}
