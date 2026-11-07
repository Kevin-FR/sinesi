use std::{fs, process::Command};

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
