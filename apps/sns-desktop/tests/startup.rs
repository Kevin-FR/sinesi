use serde_json::{Value, json};

#[test]
fn response_wait_factor_is_applied_to_the_worker_and_help_is_available_in_six_languages() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(&root.join("admin.json"), json!({"configuration_version":1}));
    write(
        &root.join("profile.json"),
        json!({"settings_version":1,"language":"fr"}),
    );
    let original = std::fs::read(root.join("profile.json")).unwrap();
    for language in ["fr", "en", "es", "it", "pt", "de"] {
        for view in ["icons", "diagram"] {
            let out = invoke(
                root,
                &[
                    "--check-config",
                    "--language",
                    language,
                    "--view",
                    view,
                    "-r",
                    "3",
                ],
            );
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let report: Value = serde_json::from_slice(&out.stdout).unwrap();
            contract().validate(&report).unwrap();
            assert_eq!(report["startup_report_version"], 2);
            assert_eq!(report["runtime"]["wait_factor"], 3);
            assert_eq!(report["configuration"]["effective"]["language"], language);
            assert_eq!(
                report["configuration"]["effective"]["display"]["representation"],
                view
            );
            assert!(
                report["configuration"]["persistent"]
                    .get("wait_factor")
                    .is_none()
            );
        }
        let help = invoke(root, &["--help", "--language", language]);
        assert!(help.status.success());
        let help = String::from_utf8(help.stdout).unwrap();
        assert!(help.contains("-r, --rtt-factor 1..5"));
        assert!(!help.contains("launch_rtt_factor"));
        for value in ["0", "6", "-1", "2.5", "PRIVATE_INVALID_VALUE"] {
            let out = invoke(
                root,
                &[
                    "--check-config",
                    "--language",
                    language,
                    "--rtt-factor",
                    value,
                ],
            );
            assert!(out.status.success());
            let report: Value = serde_json::from_slice(&out.stdout).unwrap();
            contract().validate(&report).unwrap();
            assert_eq!(report["startup_report_version"], 1);
            assert!(
                report["configuration"]["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|d| d["code"] == "simulation.invalid_wait_factor")
            );
            assert!(!String::from_utf8_lossy(&out.stderr).contains("PRIVATE_INVALID_VALUE"));
        }
    }
    assert_eq!(std::fs::read(root.join("profile.json")).unwrap(), original);
    assert!(!root.join("sessions").exists());
}
use std::{
    path::Path,
    process::{Command, Output},
};
fn invoke(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sns-desktop"))
        .current_dir(root)
        .env_remove("SINESI_ADMIN_CONFIG")
        .env_remove("SINESI_SETTINGS_PATH")
        .env_remove("SINESI_SESSION_DIR")
        .args([
            "--config",
            "admin.json",
            "--settings",
            "profile.json",
            "--session-dir",
            "sessions",
        ])
        .args(args)
        .output()
        .unwrap()
}
fn write(path: &Path, value: Value) {
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}
fn contract() -> jsonschema::Validator {
    jsonschema::validator_for(
        &serde_json::from_slice::<Value>(include_bytes!(
            "../../../contracts/startup-report.schema.json"
        ))
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn real_binary_resolves_layers_and_checks_the_explicit_project_without_writing_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        &root.join("admin.json"),
        json!({"configuration_version":1,"language":"en","display":{"scale_percent":125,"font":"noto_sans"}}),
    );
    write(
        &root.join("profile.json"),
        json!({"settings_version":1,"language":"de"}),
    );
    let input = include_bytes!("../../../fixtures/valid/services.json");
    std::fs::write(root.join("école réseau.json"), input).unwrap();
    let old_profile = std::fs::read(root.join("profile.json")).unwrap();
    let output = invoke(
        root,
        &[
            "--check-config",
            "--language",
            "pt",
            "--ui-scale",
            "180",
            "--view",
            "diagram",
            "école réseau.json",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(contract().is_valid(&report));
    let cfg = &report["configuration"];
    assert_eq!(cfg["effective"]["language"], "pt");
    assert_eq!(cfg["persistent"]["language"], "de");
    assert_eq!(cfg["effective"]["display"]["scale_percent"], 180);
    assert_eq!(cfg["persistent"]["display"]["scale_percent"], 125);
    assert_eq!(cfg["origins"]["display.font"], "administrator");
    assert_eq!(cfg["origins"]["display.representation"], "command_line");
    let source: Value = serde_json::from_slice(input).unwrap();
    assert_eq!(
        report["project"]["devices"],
        source["devices"].as_array().unwrap().len()
    );
    assert_eq!(
        report["project"]["schema_version"],
        source["schema_version"]
    );
    assert_eq!(
        report["project"]["links"],
        source["links"].as_array().unwrap().len()
    );
    assert_eq!(
        std::fs::read(root.join("école réseau.json")).unwrap(),
        input
    );
    assert_eq!(
        std::fs::read(root.join("profile.json")).unwrap(),
        old_profile
    );
    assert!(!root.join("sessions").exists());
}
#[test]
fn help_version_and_safe_fallback_work_in_all_six_languages_without_a_window() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(&root.join("admin.json"), json!({"configuration_version":1}));
    for language in ["fr", "en", "es", "it", "pt", "de"] {
        let output = invoke(root, &["--language", language, "--help"]);
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        for option in [
            "--help",
            "--version",
            "--check-config",
            "--config",
            "--settings",
            "--workdir",
            "--language",
            "--ui-scale",
            "--view",
            "--layout",
        ] {
            assert!(help.contains(option));
        }
        let output = invoke(
            root,
            &[
                "--language",
                language,
                "--check-config",
                "--ui-scale",
                "NOT_A_NUMBER_SECRET",
            ],
        );
        assert!(output.status.success());
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(contract().is_valid(&report));
        assert_eq!(
            report["configuration"]["effective"]["display"]["scale_percent"],
            100
        );
        assert_eq!(report["configuration"]["effective"]["language"], language);
        assert!(!String::from_utf8_lossy(&output.stderr).contains("NOT_A_NUMBER_SECRET"));
    }
    let version = invoke(root, &["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        format!("Sinesi {}", env!("CARGO_PKG_VERSION"))
    );
    let unknown = invoke(root, &["--unknown-secret"]);
    assert!(!unknown.status.success());
    assert!(!String::from_utf8_lossy(&unknown.stderr).contains("unknown-secret"));
    assert!(!root.join("profile.json").exists());
    assert!(!root.join("sessions").exists());
}
#[test]
fn working_directory_and_invalid_project_are_explicit_and_preserve_the_source() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("répertoire de travail");
    std::fs::create_dir(&base).unwrap();
    write(&base.join("admin.json"), json!({"configuration_version":1}));
    let input = include_bytes!("../../../fixtures/valid/lan.json");
    std::fs::write(base.join("-classe.json"), input).unwrap();
    let good = invoke(
        dir.path(),
        &[
            "-w",
            base.to_str().unwrap(),
            "--check-config",
            "--",
            "-classe.json",
        ],
    );
    assert!(
        good.status.success(),
        "{}",
        String::from_utf8_lossy(&good.stderr)
    );
    assert!(!base.join("sessions").exists());
    let conflict = invoke(
        dir.path(),
        &[
            "-w",
            base.to_str().unwrap(),
            "--settings",
            "missing/../-classe.json",
            "--check-config",
            "--",
            "-classe.json",
        ],
    );
    assert!(conflict.status.success());
    let report: Value = serde_json::from_slice(&conflict.stdout).unwrap();
    assert!(contract().is_valid(&report));
    assert_eq!(report["configuration"]["user_writable"], false);
    assert!(
        String::from_utf8_lossy(&conflict.stderr).contains(
            sns_settings::Settings::default()
                .language
                .text("configuration.project_path_conflict")
        )
    );
    assert!(!base.join("missing").exists());

    std::fs::write(base.join("broken.json"), b"SECRET_DO_NOT_ECHO").unwrap();
    let bad = invoke(
        dir.path(),
        &[
            "-w",
            base.to_str().unwrap(),
            "--check-config",
            "broken.json",
        ],
    );
    assert!(!bad.status.success());
    assert!(!String::from_utf8_lossy(&bad.stderr).contains("SECRET_DO_NOT_ECHO"));
    assert_eq!(
        std::fs::read(base.join("broken.json")).unwrap(),
        b"SECRET_DO_NOT_ECHO"
    );
}
#[test]
#[cfg(target_os = "linux")]
fn native_non_utf8_project_and_profile_paths_are_not_lossily_converted() {
    use std::os::unix::ffi::OsStringExt;
    let dir = tempfile::tempdir().unwrap();
    let project = dir
        .path()
        .join(std::ffi::OsString::from_vec(b"classe-\xff.json".to_vec()));
    let profile = dir
        .path()
        .join(std::ffi::OsString::from_vec(b"profil-\xfe.json".to_vec()));
    std::fs::write(&project, include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    write(&profile, json!({"settings_version":1,"language":"it"}));
    let output = Command::new(env!("CARGO_BIN_EXE_sns-desktop"))
        .args([
            std::ffi::OsStr::new("--settings"),
            profile.as_os_str(),
            std::ffi::OsStr::new("--check-config"),
            project.as_os_str(),
        ])
        .env("SINESI_ADMIN_CONFIG", dir.path().join("missing-admin.json"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(contract().is_valid(&report));
    assert_eq!(report["configuration"]["effective"]["language"], "it");
}

#[test]
#[cfg(target_os = "macos")]
fn non_utf8_path_metadata_does_not_prevent_a_startup_report_on_apfs() {
    use std::os::unix::ffi::OsStringExt;
    let dir = tempfile::tempdir().unwrap();
    let profile = dir
        .path()
        .join(std::ffi::OsString::from_vec(b"profil-\xfe.json".to_vec()));
    let project = dir.path().join("classe.json");
    std::fs::write(&project, include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sns-desktop"))
        .args([
            std::ffi::OsStr::new("--settings"),
            profile.as_os_str(),
            std::ffi::OsStr::new("--check-config"),
            project.as_os_str(),
        ])
        .env("SINESI_ADMIN_CONFIG", dir.path().join("missing-admin.json"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(contract().is_valid(&report));
    assert_eq!(report["project"]["devices"], 4);
    assert!(!profile.exists());
}

#[test]
fn technical_log_is_announced_and_verbose_preserves_machine_readable_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let work = root.join("salle de cours");
    std::fs::create_dir(&work).unwrap();
    write(&work.join("admin.json"), json!({"configuration_version":1}));
    let out = invoke(
        root,
        &[
            "--workdir",
            "salle de cours",
            "--check-config",
            "-v",
            "--log-file",
            "essai technique.jsonl",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    contract()
        .validate(&serde_json::from_slice::<Value>(&out.stdout).unwrap())
        .unwrap();
    let log = work.join("essai technique.jsonl");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains(&format!("{:?}", log.canonicalize().unwrap())));
    let lines = std::fs::read_to_string(&log).unwrap();
    assert!(stderr.contains("project_ready"));
    assert!(lines.contains("configuration_resolved"));
    assert!(lines.contains("project_ready"));
    for (i, line) in lines.lines().enumerate() {
        let entry: Value = serde_json::from_str(line).unwrap();
        assert_eq!(entry["sequence"], i + 1);
    }
    let normal = invoke(
        root,
        &["--workdir", "salle de cours", "--check-config", "-l"],
    );
    assert!(normal.status.success());
    let default_log = std::fs::read_dir(&work)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("sinesi-")
        })
        .unwrap();
    assert!(
        String::from_utf8(normal.stderr)
            .unwrap()
            .contains(&format!("{:?}", default_log.canonicalize().unwrap()))
    );
    assert!(
        !std::fs::read_to_string(default_log)
            .unwrap()
            .contains("project_ready")
    );
    let failure = invoke(
        root,
        &[
            "--workdir",
            "salle de cours",
            "--check-config",
            "--log-file",
            "essai technique.jsonl",
        ],
    );
    assert!(!failure.status.success());
    assert_eq!(std::fs::read_to_string(log).unwrap(), lines);
    assert!(!work.join("profile.json").exists());
}

#[test]
fn log_creation_protects_launch_roles_and_records_only_the_failed_stage() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(&root.join("admin.json"), json!({"configuration_version":1}));
    let project = root.join("PRIVATE_PROJECT_PATH.json");
    let input = br#"{"secret":"PRIVATE_MAIL_BODY","password":"PRIVATE_PASSWORD"}"#;
    std::fs::write(&project, input).unwrap();
    std::fs::create_dir(root.join("sessions")).unwrap();
    for path in [
        "admin.json",
        "profile.json",
        "sessions/journal.jsonl",
        "PRIVATE_PROJECT_PATH.json",
    ] {
        let out = invoke(
            root,
            &[
                "--check-config",
                "--log-file",
                path,
                "PRIVATE_PROJECT_PATH.json",
            ],
        );
        assert!(!out.status.success());
    }
    assert!(!root.join("profile.json").exists());
    assert!(!root.join("sessions/journal.jsonl").exists());
    let out = invoke(
        root,
        &[
            "--check-config",
            "--log-file",
            "failed.jsonl",
            "PRIVATE_PROJECT_PATH.json",
        ],
    );
    assert!(!out.status.success());
    let log = std::fs::read_to_string(root.join("failed.jsonl")).unwrap();
    assert!(log.contains("project_load_failed"));
    assert!(!log.contains("PRIVATE_"));
    assert_eq!(std::fs::read(project).unwrap(), input);
    for language in ["fr", "en", "es", "it", "pt", "de"] {
        let output = invoke(root, &["--help", "--language", language]);
        let help = String::from_utf8(output.stdout).unwrap();
        for flag in ["-v, --verbose", "-l, --log", "--log-file PATH"] {
            assert!(help.contains(flag));
        }
        assert!(!help.contains("launch_log"));
    }
}

#[test]
fn compact_launch_is_temporary_and_explicit_valid_options_override_its_baseline() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(&root.join("admin.json"), json!({"configuration_version":1}));
    let mut profile = serde_json::to_value(sns_settings::Settings::default()).unwrap();
    profile["display"]["scale_percent"] = json!(150);
    profile["display"]["window_width"] = json!(1600);
    let path = root.join("profile.json");
    write(&path, profile.clone());
    let before = std::fs::read(&path).unwrap();
    for language in ["fr", "en", "es", "it", "pt", "de"] {
        let help = invoke(root, &["--help", "--language", language]);
        assert!(help.status.success());
        let help = String::from_utf8(help.stdout).unwrap();
        assert!(help.contains("-n, --native-theme"));
        assert!(help.contains("-s, --small-screen"));
        assert!(!help.contains("launch_small_screen"));
        for (view, flags) in [
            ("icons", ["-n", "-s"]),
            ("diagram", ["--native-theme", "--small-screen"]),
        ] {
            let output = invoke(
                root,
                &[
                    "--check-config",
                    "--language",
                    language,
                    "--view",
                    view,
                    flags[0],
                    flags[1],
                ],
            );
            assert!(output.status.success());
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            contract().validate(&report).unwrap();
            let effective = &report["configuration"]["effective"];
            assert_eq!(effective["language"], language);
            assert_eq!(effective["display"]["representation"], view);
            assert_eq!(effective["display"]["layout"], "tabs");
            assert_eq!(effective["display"]["scale_percent"], 90);
            assert_eq!(effective["display"]["application_scale_percent"], 85);
            assert_eq!(effective["display"]["window_width"], 840);
            assert_eq!(effective["display"]["window_height"], 600);
            assert_eq!(
                report["configuration"]["origins"]["display.window_width"],
                "command_line"
            );
            assert_eq!(report["configuration"]["persistent"], profile);
        }
    }
    let output = invoke(
        root,
        &[
            "--check-config",
            "--window-width",
            "960",
            "--window-height",
            "oops",
            "-s",
        ],
    );
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let effective = &report["configuration"]["effective"]["display"];
    assert_eq!(effective["window_width"], 960);
    assert_eq!(effective["window_height"], 600);
    assert!(
        report["configuration"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["field"] == "display.window_height")
    );
    assert_eq!(std::fs::read(path).unwrap(), before);
    assert!(!root.join("sessions").exists());
}
