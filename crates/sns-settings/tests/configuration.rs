use serde_json::{Value, json};
use sns_i18n::Language;
use sns_settings::{
    Settings,
    configuration::{self, Overrides, Source, Sources},
    display::{DisplayPreferences, UiFont},
};
fn write(path: &std::path::Path, value: Value) {
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}
#[test]
fn precedence_is_per_field_and_command_line_overrides_are_temporary() {
    let dir = tempfile::tempdir().unwrap();
    let admin = dir.path().join("admin.json");
    let user = dir.path().join("user.json");
    write(
        &admin,
        json!({"configuration_version":1,"language":"en","display":{"scale_percent":125,"font":"noto_sans","inspector_width":360}}),
    );
    write(&user, json!({"settings_version":1,"language":"de"}));
    let old_admin = std::fs::read(&admin).unwrap();
    let old_user = std::fs::read(&user).unwrap();
    let resolved = configuration::resolve(Sources {
        administrator: Some(admin.clone()),
        user: Some(user.clone()),
        overrides: Overrides {
            language: Some(json!("es")),
            display: serde_json::from_value(json!({"scale_percent":180})).unwrap(),
        },
        ..Default::default()
    });
    assert!(resolved.diagnostics.is_empty());
    assert!(resolved.user_writable);
    assert_eq!(resolved.effective.language, Language::Es);
    assert_eq!(resolved.effective.display.scale_percent, 180);
    assert_eq!(resolved.effective.display.font, UiFont::NotoSans);
    assert_eq!(resolved.persistent.language, Language::De);
    assert_eq!(resolved.persistent.display.scale_percent, 125);
    for (field, source) in [
        ("language", Source::CommandLine),
        ("display.scale_percent", Source::CommandLine),
        ("display.font", Source::Administrator),
        ("display.layout", Source::Builtin),
    ] {
        assert_eq!(resolved.origins[field], source);
    }
    assert_eq!(std::fs::read(&admin).unwrap(), old_admin);
    assert_eq!(std::fs::read(&user).unwrap(), old_user);
    let complete = Settings {
        language: Language::It,
        ..Default::default()
    };
    sns_settings::save(&user, &complete).unwrap();
    let r = configuration::resolve(Sources {
        administrator: Some(admin),
        user: Some(user),
        ..Default::default()
    });
    assert_eq!(r.effective, complete);
    assert_eq!(r.origins["display.font"], Source::User);
}
#[test]
fn invalid_fields_retain_lower_values_preserve_original_and_never_echo_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let admin = dir.path().join("admin.json");
    let user = dir.path().join("user.json");
    write(
        &admin,
        json!({"configuration_version":1,"language":"pt","display":{"scale_percent":120,"inspector_width":340}}),
    );
    let mut profile = serde_json::to_value(Settings::default()).unwrap();
    profile["language"] = json!("it");
    profile["display"]["scale_percent"] = json!(201);
    profile["display"]["font"] = json!("PASSWORD_DO_NOT_ECHO");
    profile["MAIL_BODY_DO_NOT_ECHO"] = json!("PRIVATE_CONTENT");
    write(&user, profile);
    let original = std::fs::read(&user).unwrap();
    let r = configuration::resolve(Sources {
        administrator: Some(admin),
        user: Some(user.clone()),
        overrides: Overrides {
            language: Some(json!("de")),
            display: serde_json::from_value(json!({"scale_percent":999999,"inspector_width":380}))
                .unwrap(),
        },
        ..Default::default()
    });
    assert_eq!(r.effective.language, Language::De);
    assert_eq!(r.effective.display.scale_percent, 120);
    assert_eq!(r.effective.display.inspector_width, 380);
    assert_eq!(r.effective.display.font, UiFont::Default);
    assert!(!r.user_writable);
    assert_eq!(r.persistent.language, Language::It);
    assert_eq!(r.persistent.display.scale_percent, 120);
    assert_eq!(r.origins["display.scale_percent"], Source::Administrator);
    let encoded = serde_json::to_string(&r).unwrap();
    for secret in [
        "PASSWORD_DO_NOT_ECHO",
        "MAIL_BODY_DO_NOT_ECHO",
        "PRIVATE_CONTENT",
    ] {
        assert!(!encoded.contains(secret));
    }
    for l in Language::ALL {
        for issue in &r.diagnostics {
            let text = issue.localized(l);
            assert!(!text.contains("configuration.invalid_value"));
            assert!(!text.contains("DO_NOT_ECHO"));
        }
    }
    assert_eq!(std::fs::read(user).unwrap(), original);
}
#[test]
fn missing_malformed_future_and_oversized_files_do_not_block_safe_resolution() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let missing = configuration::resolve(Sources {
        administrator: Some(path.clone()),
        administrator_required: true,
        ..Default::default()
    });
    assert_eq!(missing.diagnostics[0].code, "configuration.missing_file");
    assert_eq!(missing.effective, Settings::default());
    assert!(
        configuration::resolve(Sources {
            administrator: Some(path.clone()),
            ..Default::default()
        })
        .diagnostics
        .is_empty()
    );
    for bytes in [
        b"not JSON SECRET".to_vec(),
        b"{\"settings_version\":99,\"language\":\"de\"}".to_vec(),
        vec![b' '; 65537],
        b"[]".to_vec(),
    ] {
        std::fs::write(&path, &bytes).unwrap();
        let r = configuration::resolve(Sources {
            user: Some(path.clone()),
            ..Default::default()
        });
        assert_eq!(r.effective, Settings::default());
        assert!(!r.user_writable);
        assert!(!r.diagnostics.is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    let mut bad = serde_json::Map::new();
    bad.insert("settings_version".into(), json!(1));
    bad.insert("language".into(), json!("fr"));
    for n in 0..200 {
        bad.insert(format!("unknown{n}"), json!("SECRET"));
    }
    write(&path, Value::Object(bad));
    let r = configuration::resolve(Sources {
        user: Some(path),
        ..Default::default()
    });
    assert_eq!(r.diagnostics.len(), 32);
    assert_eq!(
        r.diagnostics.last().unwrap().code,
        "configuration.too_many_issues"
    );
}
#[test]
fn every_numeric_limit_uses_safe_fallback_and_window_maps_are_atomic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("admin.json");
    for (field, min, max) in [
        ("scale_percent", 80, 200),
        ("palette_width", 160, 360),
        ("inspector_width", 240, 480),
        ("window_width", 840, 3840),
        ("window_height", 600, 2160),
        ("application_scale_percent", 75, 150),
    ] {
        for number in [min - 1, max + 1] {
            let mut display = serde_json::Map::new();
            display.insert(field.into(), json!(number));
            write(&path, json!({"configuration_version":1,"display":display}));
            let r = configuration::resolve(Sources {
                administrator: Some(path.clone()),
                ..Default::default()
            });
            assert_eq!(r.effective, Settings::default(), "{field}");
            assert_eq!(r.diagnostics.len(), 1, "{field}");
            assert_eq!(r.diagnostics[0].field, format!("display.{field}"));
        }
    }
    write(
        &path,
        json!({"configuration_version":1,"display":{"font":"noto_sans","windows":{"not_a_window":{"x":0,"y":0,"width":600,"height":400}}}}),
    );
    let r = configuration::resolve(Sources {
        administrator: Some(path),
        ..Default::default()
    });
    assert_eq!(r.effective.display.font, UiFont::NotoSans);
    assert!(r.effective.display.windows.is_empty());
    assert_eq!(r.diagnostics[0].field, "display.windows");
}
#[test]
fn edits_copy_only_changed_fields_without_leaking_temporary_values() {
    let mut persistent = DisplayPreferences::default();
    let before = DisplayPreferences {
        scale_percent: 180,
        ..Default::default()
    };
    let after = DisplayPreferences {
        font: UiFont::NotoSans,
        ..before.clone()
    };
    persistent.copy_user_edits(&before, &after);
    assert_eq!(persistent.scale_percent, 100);
    assert_eq!(persistent.font, UiFont::NotoSans);
}
#[test]
fn portable_administrator_contract_matches_resolver_and_rejects_invalid_fields() {
    let value: Value = serde_json::from_slice(include_bytes!(
        "../../../contracts/configuration.schema.json"
    ))
    .unwrap();
    let schema = jsonschema::validator_for(&value).unwrap();
    let valid: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/configuration.json"))
            .unwrap();
    assert!(schema.is_valid(&valid));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("admin.json");
    write(&path, valid.clone());
    let resolved = configuration::resolve(Sources {
        administrator: Some(path.clone()),
        ..Default::default()
    });
    assert!(resolved.diagnostics.is_empty());
    assert_eq!(resolved.effective.display.scale_percent, 125);
    for (pointer, bad) in [
        ("/configuration_version", json!(99)),
        ("/language", json!("unknown")),
        ("/display/scale_percent", json!(201)),
        ("/display/font", json!("download")),
    ] {
        let mut value = valid.clone();
        *value.pointer_mut(pointer).unwrap() = bad;
        assert!(!schema.is_valid(&value));
        write(&path, value);
        assert!(
            !configuration::resolve(Sources {
                administrator: Some(path.clone()),
                ..Default::default()
            })
            .diagnostics
            .is_empty()
        );
    }
}

#[test]
fn integral_json_spellings_match_the_portable_integer_contract() {
    let dir = tempfile::tempdir().unwrap();
    let admin = dir.path().join("admin.json");
    let bytes = br#"{"configuration_version":1.0,"display":{"scale_percent":1.25e2}}"#;
    let schema = jsonschema::validator_for(
        &serde_json::from_slice::<Value>(include_bytes!(
            "../../../contracts/configuration.schema.json"
        ))
        .unwrap(),
    )
    .unwrap();
    assert!(schema.is_valid(&serde_json::from_slice(bytes).unwrap()));
    std::fs::write(&admin, bytes).unwrap();
    let r = configuration::resolve(Sources {
        administrator: Some(admin),
        overrides: Overrides {
            display: serde_json::from_value(json!({"window_width":1440.0})).unwrap(),
            ..Default::default()
        },
        ..Default::default()
    });
    assert!(r.diagnostics.is_empty());
    assert_eq!(r.effective.display.scale_percent, 125);
    assert_eq!(r.effective.display.window_width, 1440);
    let user = dir.path().join("profile.json");
    let mut value = serde_json::to_value(Settings::default()).unwrap();
    value["display"]["scale_percent"] = json!(150.0);
    write(&user, value);
    assert_eq!(
        sns_settings::load(&user).unwrap().display.scale_percent,
        150
    );
}
