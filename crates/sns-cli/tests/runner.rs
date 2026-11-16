use std::{fs, process::Command};

#[test]
fn response_wait_factor_changes_slow_network_outcome_and_survives_offline_replay_and_resume() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = tempfile::tempdir().unwrap();
    let path = |name: &str| dir.path().join(name).to_str().unwrap().to_owned();
    let call = |args: Vec<&str>| {
        let output = Command::new(env!("CARGO_BIN_EXE_sinesi"))
            .current_dir(&root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    let ordinary = path("ordinary.jsonl");
    let full = path("full.jsonl");
    let replay = path("replay.jsonl");
    let resumed = path("resumed.jsonl");
    let point = path("point.json");
    let recording = path("recording.json");
    let source = "fixtures/valid/response-wait-scenario.json";
    call(vec![
        "run",
        source,
        "--until-us",
        "6000000",
        "--trace",
        &ordinary,
    ]);
    call(vec![
        "run",
        source,
        "--rtt-factor",
        "5",
        "--until-us",
        "300000",
        "--checkpoint",
        &point,
    ]);
    call(vec![
        "run",
        source,
        "--rtt-factor",
        "5",
        "--until-us",
        "6000000",
        "--trace",
        &full,
        "--recording",
        &recording,
    ]);
    call(vec!["replay", &recording, "--trace", &replay]);
    call(vec![
        "resume",
        &point,
        "--continuation",
        &recording,
        "--until-us",
        "6000000",
        "--trace",
        &resumed,
    ]);
    let bytes = fs::read(&full).unwrap();
    assert_eq!(bytes, fs::read(&replay).unwrap());
    assert_eq!(bytes, fs::read(&resumed).unwrap());
    let has_reply = |bytes: &[u8]| {
        std::str::from_utf8(bytes).unwrap().lines().any(|line| {
            serde_json::from_str::<serde_json::Value>(line).unwrap()["type"] == "icmp.echo.reply"
        })
    };
    assert!(has_reply(&bytes));
    assert!(!has_reply(&fs::read(&ordinary).unwrap()));
    for args in [
        vec!["run", source, "--rtt-factor", "6", "--trace", &full],
        vec!["resume", &point, "--rtt-factor", "2", "--trace", &full],
        vec!["replay", &recording, "--rtt-factor", "2", "--trace", &full],
    ] {
        assert!(
            !Command::new(env!("CARGO_BIN_EXE_sinesi"))
                .current_dir(&root)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
        assert_eq!(fs::read(&full).unwrap(), bytes);
    }
}

#[test]
fn independent_processes_emit_identical_traces_and_honor_bounds() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let directory = tempfile::tempdir().unwrap();
    let mut hashes = Vec::new();
    for index in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_sinesi"))
            .current_dir(&root)
            .args([
                "run",
                "fixtures/valid/scenario.json",
                "--seed",
                "42",
                "--trace",
            ])
            .arg(directory.path().join(format!("trace-{index}.jsonl")))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        hashes.push(output.stdout);
    }
    assert_eq!(hashes[0], hashes[1]);
    assert_eq!(
        fs::read(directory.path().join("trace-0.jsonl")).unwrap(),
        fs::read(directory.path().join("trace-1.jsonl")).unwrap()
    );
    let bounded = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .current_dir(&root)
        .args(["run", "fixtures/valid/scenario.json", "--until-us", "999"])
        .output()
        .unwrap();
    assert!(bounded.status.success());
    assert!(String::from_utf8_lossy(&bounded.stdout).contains("time_bound"));
    let exhausted = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .current_dir(&root)
        .args(["run", "fixtures/valid/scenario.json", "--max-events", "1"])
        .output()
        .unwrap();
    assert_eq!(exhausted.status.code(), Some(2));
    let invalid = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .current_dir(&root)
        .args(["validate", "fixtures/invalid/dangling-link.json"])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("project.invalid_reference"));
}
#[test]
fn independent_lan_processes_ping_and_reuse_arp_cache() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = tempfile::tempdir().unwrap();
    let mut outputs = Vec::new();
    for n in 0..2 {
        let path = dir.path().join(format!("lan-{n}.jsonl"));
        let output = Command::new(env!("CARGO_BIN_EXE_sinesi"))
            .current_dir(&root)
            .args([
                "ping",
                "fixtures/valid/lan.json",
                "--source",
                "host-a",
                "--target",
                "192.168.1.3",
                "--count",
                "2",
                "--trace",
            ])
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["replies"], 2);
        assert_eq!(report["arp_requests"], 1);
        outputs.push(fs::read(path).unwrap());
    }
    assert_eq!(outputs[0], outputs[1]);
    let fail = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .current_dir(&root)
        .args([
            "ping",
            "fixtures/valid/lan.json",
            "--source",
            "host-a",
            "--target",
            "10.0.0.1",
        ])
        .output()
        .unwrap();
    assert_eq!(fail.status.code(), Some(1));
}
#[test]
fn filius_import_cli_emits_report_and_keeps_source_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("source.fls");
    let out = dir.path().join("converted.json");
    let report = dir.path().join("report.json");
    let bytes = include_bytes!("../../../fixtures/legacy/lan.fls");
    std::fs::write(&src, bytes).unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .arg("import-filius")
        .arg(&src)
        .arg("--output")
        .arg(&out)
        .arg("--report")
        .arg(&report)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let p = sns_project::load_project(&out).unwrap();
    assert_eq!(p.devices.len(), 3);
    let r: serde_json::Value = serde_json::from_slice(&std::fs::read(report).unwrap()).unwrap();
    assert_eq!(r["execution_performed"], false);
    assert_eq!(std::fs::read(&src).unwrap(), bytes);
    let bad = std::process::Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .arg("import-filius")
        .arg(&src)
        .arg("--output")
        .arg(&src)
        .arg("--report")
        .arg(&out)
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert_eq!(std::fs::read(&src).unwrap(), bytes);
}

#[test]
fn network_scenario_checkpoint_continuation_and_offline_replay_match_across_processes() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = tempfile::tempdir().unwrap();
    let source = root.join("fixtures/valid/network-scenario.json");
    let original = fs::read(&source).unwrap();
    let invoke = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_sinesi"))
            .current_dir(dir.path())
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    invoke(&[
        "run",
        source.to_str().unwrap(),
        "--until-us",
        "1500",
        "--checkpoint",
        "point.json",
    ]);
    invoke(&[
        "resume",
        "point.json",
        "--until-us",
        "10000000",
        "--trace",
        "resumed.jsonl",
    ]);
    invoke(&[
        "run",
        source.to_str().unwrap(),
        "--until-us",
        "10000000",
        "--recording",
        "full.json",
        "--trace",
        "original.jsonl",
    ]);
    invoke(&["replay", "full.json", "--trace", "offline.jsonl"]);
    invoke(&[
        "resume",
        "point.json",
        "--continuation",
        "full.json",
        "--until-us",
        "10000000",
        "--trace",
        "continued.jsonl",
    ]);
    for name in ["resumed.jsonl", "offline.jsonl", "continued.jsonl"] {
        assert_eq!(
            fs::read(dir.path().join(name)).unwrap(),
            fs::read(dir.path().join("original.jsonl")).unwrap()
        );
    }
    let bad = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .args([
            "run",
            source.to_str().unwrap(),
            "--checkpoint",
            source.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert_eq!(fs::read(&source).unwrap(), original);
    let mut corrupt: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.path().join("point.json")).unwrap()).unwrap();
    corrupt["checkpoint_version"] = 99.into();
    fs::write(
        dir.path().join("bad.json"),
        serde_json::to_vec(&corrupt).unwrap(),
    )
    .unwrap();
    fs::write(dir.path().join("preserved.jsonl"), b"keep").unwrap();
    let bad = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .current_dir(dir.path())
        .args(["resume", "bad.json", "--trace", "preserved.jsonl"])
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert_eq!(
        fs::read(dir.path().join("preserved.jsonl")).unwrap(),
        b"keep"
    );
}
