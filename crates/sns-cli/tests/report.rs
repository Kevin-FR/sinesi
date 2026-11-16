use std::{fs, process::Command};
fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
#[test]
fn headless_report_from_project_and_checkpoint_is_deterministic_and_atomic() {
    let dir = tempfile::tempdir().unwrap();
    let source = root().join("fixtures/valid/report-network.json");
    let original = fs::read(&source).unwrap();
    let output = dir.path().join("rapport école.pdf");
    let command = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_sinesi"))
            .args(["report-pdf"])
            .arg(&source)
            .arg("--output")
            .arg(&output)
            .args(extra)
            .output()
            .unwrap()
    };
    let first = command(&["--language", "de", "--view", "diagram"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let bytes = fs::read(&output).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    let second = command(&["--language", "de", "--view", "diagram"]);
    assert!(second.status.success());
    assert_eq!(bytes, fs::read(&output).unwrap());
    assert_eq!(first.stdout, second.stdout);
    for args in [
        &["--language", "xx"][..],
        &["--view", "other"][..],
        &["--view", "icons", "--view", "diagram"][..],
    ] {
        assert!(!command(args).status.success());
        assert_eq!(bytes, fs::read(&output).unwrap());
    }
    let protected = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .arg("report-pdf")
        .arg(&source)
        .arg("--output")
        .arg(&source)
        .output()
        .unwrap();
    assert!(!protected.status.success());
    assert_eq!(original, fs::read(source).unwrap());
    let point = dir.path().join("network-checkpoint.json");
    let run = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .current_dir(root())
        .args([
            "run",
            "fixtures/valid/network-scenario.json",
            "--until-us",
            "1000000",
            "--checkpoint",
        ])
        .arg(&point)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let report = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .arg("report-pdf")
        .arg(&point)
        .args(["--input", "checkpoint", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        report.status.success(),
        "{}",
        String::from_utf8_lossy(&report.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&report.stdout).unwrap();
    assert!(receipt["packet_rows"].as_u64().unwrap() > 0);
    let mut bad: serde_json::Value = serde_json::from_slice(&original).unwrap();
    bad["annotations"][0]["text"] = serde_json::json!("\u{10ffff}");
    let badpath = dir.path().join("bad.json");
    fs::write(&badpath, serde_json::to_vec(&bad).unwrap()).unwrap();
    let before = fs::read(&output).unwrap();
    let failed = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .arg("report-pdf")
        .arg(&badpath)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert!(!failed.status.success());
    assert_eq!(before, fs::read(output).unwrap());
}
