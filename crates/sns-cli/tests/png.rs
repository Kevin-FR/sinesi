use std::{fs, process::Command};
#[test]
fn cli_png_exports_both_views_and_preserves_source_and_previous_output_on_failure() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("Cours réseau.json");
    fs::write(
        &source,
        include_bytes!("../../../fixtures/valid/annotations.json"),
    )
    .unwrap();
    let original = fs::read(&source).unwrap();
    for view in ["icons", "diagram"] {
        let output = dir.path().join(format!("réseau-{view}.png"));
        let run = || {
            Command::new(env!("CARGO_BIN_EXE_sinesi"))
                .arg("render-png")
                .arg(&source)
                .arg("--output")
                .arg(&output)
                .args(["--view", view, "--size", "512"])
                .output()
                .unwrap()
        };
        let result = run();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["devices"], 4);
        assert_eq!(report["annotations"], 2);
        assert_eq!(report["clipped_texts"], 0);
        assert_eq!(report["transparent"], true);
        let first = fs::read(&output).unwrap();
        assert_eq!(&first[..8], b"\x89PNG\r\n\x1a\n");
        let width = u32::from_be_bytes(first[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(first[20..24].try_into().unwrap());
        assert_eq!(width.max(height), 512);
        assert!(run().status.success());
        assert_eq!(fs::read(output).unwrap(), first);
    }
    let overwrite = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .arg("render-png")
        .arg(&source)
        .arg("--output")
        .arg(&source)
        .output()
        .unwrap();
    assert!(!overwrite.status.success());
    assert_eq!(fs::read(&source).unwrap(), original);
    let output = dir.path().join("keep.png");
    fs::write(&output, b"previous image").unwrap();
    let mut invalid: serde_json::Value = serde_json::from_slice(&original).unwrap();
    invalid["annotations"][0]["text"] = serde_json::json!("\u{10ffff}");
    fs::write(&source, serde_json::to_vec(&invalid).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_sinesi"))
        .arg("render-png")
        .arg(&source)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("render.unsupported_glyph"));
    assert_eq!(fs::read(&output).unwrap(), b"previous image");
    for arguments in [
        vec!["--size", "0"],
        vec!["--view", "bad"],
        vec!["--size", "512", "--size", "512"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_sinesi"))
            .arg("render-png")
            .arg(&source)
            .arg("--output")
            .arg(&output)
            .args(arguments)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert_eq!(fs::read(&output).unwrap(), b"previous image");
    }
}
