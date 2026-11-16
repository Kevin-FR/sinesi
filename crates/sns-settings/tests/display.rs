use serde_json::{Value, json};
use sns_i18n::Language;
use sns_settings::{
    Settings,
    display::{DisplayPreferences, PanelLayout, Representation, UiFont},
};
fn schema() -> jsonschema::Validator {
    let value: Value =
        serde_json::from_slice(include_bytes!("../../../contracts/preferences.schema.json"))
            .unwrap();
    jsonschema::validator_for(&value).unwrap()
}
#[test]
fn version_one_migrates_in_memory_without_touching_original_and_all_preferences_roundtrip() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("settings.json");
    for language in Language::ALL {
        let old = serde_json::to_vec(&json!({"settings_version":1,"language":language})).unwrap();
        std::fs::write(&path, &old).unwrap();
        let mut settings = sns_settings::load(&path).unwrap();
        assert_eq!(settings.language, language);
        assert_eq!(settings.settings_version, 2);
        assert_eq!(settings.display, DisplayPreferences::default());
        assert_eq!(std::fs::read(&path).unwrap(), old);
        for layout in [
            PanelLayout::InspectorLeft,
            PanelLayout::Tabs,
            PanelLayout::InspectorRight,
        ] {
            settings.display = DisplayPreferences {
                scale_percent: 200,
                font: UiFont::NotoSans,
                representation: Representation::Diagram,
                layout,
                ..Default::default()
            };
            sns_settings::save(&path, &settings).unwrap();
            assert_eq!(sns_settings::load(&path).unwrap(), settings);
            let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            assert!(schema().is_valid(&saved));
        }
    }
}
#[test]
fn portable_contract_and_reader_reject_invalid_preferences_and_preserve_file() {
    let contract = schema();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    for bytes in [
        include_bytes!("../../../fixtures/valid/preferences-v1.json").as_slice(),
        include_bytes!("../../../fixtures/valid/preferences-v2.json").as_slice(),
    ] {
        let value: Value = serde_json::from_slice(bytes).unwrap();
        assert!(contract.is_valid(&value));
        std::fs::write(&path, bytes).unwrap();
        assert!(sns_settings::load(&path).is_ok());
    }
    let valid: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/valid/preferences-v2.json"
    ))
    .unwrap();
    for (pointer, bad) in [
        ("/display/scale_percent", json!(201)),
        ("/display/font", json!("system_download")),
        ("/display/window_width", json!(10)),
        ("/display/inspector_width", json!(10000)),
        ("/display/windows/terminal/x", json!(1e100)),
        ("/display/windows/terminal/height", json!(0)),
        ("/display/layout", json!("unknown")),
        ("/settings_version", json!(99)),
        ("/language", json!("xx")),
    ] {
        let mut v = valid.clone();
        *v.pointer_mut(pointer).unwrap() = bad;
        assert!(!contract.is_valid(&v), "{pointer}");
        let bytes = serde_json::to_vec(&v).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(sns_settings::load(&path).is_err(), "{pointer}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    for extra in [
        json!({"display":{"unexpected":true}}),
        json!({"display":{"windows":{"arbitrary_name":{"x":0,"y":0,"width":200,"height":100}}}}),
    ] {
        let mut v = valid.clone();
        let patch = extra["display"].as_object().unwrap();
        for (k, val) in patch {
            v["display"][k] = val.clone();
        }
        assert!(!contract.is_valid(&v));
        std::fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(sns_settings::load(&path).is_err());
    }
}
#[test]
fn invalid_save_never_changes_existing_destination() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, b"preserve-me").unwrap();
    let mut settings = Settings::default();
    settings.display.scale_percent = 0;
    assert!(sns_settings::save(&path, &settings).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"preserve-me");
}
#[test]
fn every_application_window_category_has_the_same_portable_contract() {
    use sns_settings::display::WindowGeometry;
    let contract = schema();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let mut settings = Settings::default();
    for key in sns_model::Software::ALL
        .into_iter()
        .map(|software| software.id())
        .chain(["help"])
    {
        settings.display.windows.insert(
            key.to_owned(),
            WindowGeometry {
                x: 24,
                y: 40,
                width: 620,
                height: 420,
            },
        );
    }
    let value = serde_json::to_value(&settings).unwrap();
    assert!(contract.is_valid(&value));
    sns_settings::save(&path, &settings).unwrap();
    assert_eq!(sns_settings::load(&path).unwrap(), settings);
}
#[test]
fn level_is_optional_in_version_two_and_expert_roundtrips_within_the_contract() {
    use sns_settings::display::Level;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let without = include_bytes!("../../../fixtures/valid/preferences-v2.json");
    std::fs::write(&path, without).unwrap();
    let mut settings = sns_settings::load(&path).unwrap();
    assert_eq!(settings.display.level, Level::Student);
    let resolved = sns_settings::configuration::resolve(sns_settings::configuration::Sources {
        user: Some(path.clone()),
        ..Default::default()
    });
    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    assert!(resolved.user_writable);
    assert_eq!(std::fs::read(&path).unwrap(), without);
    settings.display.level = Level::Expert;
    sns_settings::save(&path, &settings).unwrap();
    let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["display"]["level"], "expert");
    assert!(schema().is_valid(&saved));
    assert_eq!(
        sns_settings::load(&path).unwrap().display.level,
        Level::Expert
    );
    let mut invalid = saved.clone();
    invalid["display"]["level"] = json!("teacher");
    assert!(!schema().is_valid(&invalid));
}
#[test]
fn mission_stars_are_optional_bounded_and_kept_by_the_user_profile() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let mut settings = Settings::default();
    // An empty progress keeps the previous profile bytes.
    sns_settings::save(&path, &settings).unwrap();
    assert!(!std::fs::read_to_string(&path).unwrap().contains("missions"));
    settings.missions.insert("forgotten-gateway".into(), 2);
    sns_settings::save(&path, &settings).unwrap();
    let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert!(schema().is_valid(&saved));
    assert_eq!(sns_settings::load(&path).unwrap(), settings);
    let resolved = sns_settings::configuration::resolve(sns_settings::configuration::Sources {
        user: Some(path.clone()),
        ..Default::default()
    });
    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    assert_eq!(resolved.persistent.missions, settings.missions);
    for bad in [json!({"forgotten-gateway": 4}), json!({"Bad Id": 1})] {
        let mut invalid = saved.clone();
        invalid["missions"] = bad;
        assert!(!schema().is_valid(&invalid));
        std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(sns_settings::load(&path).is_err());
    }
}
