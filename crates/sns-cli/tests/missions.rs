//! `sinesi mission-check` exports the same correction as the workbench.
use std::process::Command;

fn check(args: &[&str]) -> (i32, serde_json::Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .args(args)
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);
    (output.status.code().unwrap(), report)
}

#[test]
fn mission_check_reports_unmet_objectives_then_a_completed_repair() {
    let dir = tempfile::tempdir().unwrap();
    let mission = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../sns-missions/missions/forgotten-gateway.json"
    );
    // The starting network: both objectives unmet and explained in English.
    let (code, report) = check(&["mission-check", mission, "--language", "en"]);
    assert_eq!(code, 2);
    assert_eq!(report["complete"], false);
    assert_eq!(report["language"], "en");
    assert!(
        report["objectives"]
            .as_array()
            .unwrap()
            .iter()
            .all(|o| o["done"] == false && o["explanation"].is_string())
    );
    // Repair the project, record a real ping trace, then check again.
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(mission).unwrap()).unwrap();
    value["project"]["devices"][0]["network"]["default_gateway"] = "192.168.10.1".into();
    let project = dir.path().join("fixed.json");
    std::fs::write(&project, serde_json::to_vec(&value["project"]).unwrap()).unwrap();
    let trace = dir.path().join("trace.jsonl");
    let ping = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .args([
            "ping",
            project.to_str().unwrap(),
            "--source",
            "a",
            "--target",
            "192.168.30.2",
        ])
        .args(["--trace", trace.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        ping.status.success(),
        "{}",
        String::from_utf8_lossy(&ping.stderr)
    );
    let output = dir.path().join("report.json");
    let (code, _) = check(&[
        "mission-check",
        mission,
        "--project",
        project.to_str().unwrap(),
        "--trace",
        trace.to_str().unwrap(),
        "--hints",
        "1",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    assert_eq!(report["complete"], true);
    assert_eq!(report["stars"], 2);
    assert_eq!(report["report_version"], 1);
    // Bad input is refused without a report.
    let (code, _) = check(&["mission-check", mission, "--language", "xx"]);
    assert_eq!(code, 1);
}
