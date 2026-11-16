use egui_kittest::{Harness, kittest::Queryable as _};
use serde_json::{Value, json};
use sns_i18n::Language;
use sns_settings::{
    Settings,
    configuration::{self, Overrides, Sources},
    display::{DisplayPreferences, Representation, UiFont},
};
use sns_ui_egui::Workbench;

#[test]
fn restored_policy_wins_for_the_run_and_new_simulation_uses_launch_policy_in_all_languages_and_views()
 {
    use sns_model::WaitFactor;
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    let engine = sns_engine::Engine::from_project_with_wait_factor(
        project,
        42,
        Default::default(),
        WaitFactor::try_from(5).unwrap(),
    )
    .unwrap();
    let point = serde_json::to_vec(&engine.checkpoint().unwrap()).unwrap();
    let recording = serde_json::to_vec(&engine.recording().unwrap()).unwrap();
    for language in Language::ALL {
        for view in ["icons", "diagram"] {
            let resolved = configuration::resolve(Sources {
                overrides: Overrides {
                    language: Some(json!(language)),
                    display: serde_json::from_value(json!({"representation":view})).unwrap(),
                },
                ..Default::default()
            });
            let app = Workbench::with_configuration_and_wait_factor(
                resolved,
                None,
                None,
                WaitFactor::try_from(3).unwrap(),
            )
            .unwrap();
            assert_eq!(u8::from(app.runtime_wait_factor()), 3);
            let mut h = harness(app);
            h.run();
            for replay in [false, true] {
                if replay {
                    h.state_mut()
                        .begin_recording_replay(recording.clone())
                        .unwrap();
                } else {
                    h.state_mut()
                        .begin_checkpoint_restore(point.clone())
                        .unwrap();
                }
                for _ in 0..300 {
                    h.step();
                    if !h.state().runtime_busy() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                h.run();
                assert!(!h.state().runtime_busy());
                assert_eq!(u8::from(h.state().runtime_wait_factor()), 5);
                assert_eq!(h.state().language(), language);
                assert_eq!(
                    h.state().display_preferences().representation,
                    if view == "icons" {
                        Representation::Icons
                    } else {
                        Representation::Diagram
                    }
                );
                h.get_by_label(language.text("close_view")).click();
                h.run();
                h.get_by_label(&format!("{} · F2", language.text("mode_build")))
                    .click();
                h.run();
                h.get_by_label(&format!("{} · F3", language.text("mode_test")))
                    .click();
                h.run();
                h.state_mut().wait_for_worker().unwrap();
                h.run();
                assert_eq!(u8::from(h.state().runtime_wait_factor()), 3);
            }
        }
    }
}
fn write(path: &std::path::Path, value: Value) {
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}
fn harness(app: Workbench) -> Harness<'static, Workbench> {
    Harness::builder()
        .with_max_steps(60)
        .with_size((1600., 1050.))
        .build_ui_state(|ui, state| state.show(ui), app)
}
#[test]
fn invalid_startup_values_are_localized_in_six_languages_both_views_and_leave_network_and_files_intact()
 {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in Language::ALL {
        for diagram in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let admin = dir.path().join("admin.json");
            let user = dir.path().join("profile.json");
            write(
                &admin,
                json!({"configuration_version":1,"display":{"scale_percent":125}}),
            );
            let mut profile = serde_json::to_value(Settings::default()).unwrap();
            profile["display"]["scale_percent"] = json!(999);
            profile["language"] = json!("PRIVATE_BAD_VALUE");
            write(&user, profile);
            let original = std::fs::read(&user).unwrap();
            let resolved = configuration::resolve(Sources {
                administrator: Some(admin),
                user: Some(user.clone()),
                overrides: Overrides {
                    language: Some(json!(l)),
                    display: serde_json::from_value(
                        json!({"representation":if diagram {"diagram"} else {"icons"}}),
                    )
                    .unwrap(),
                },
                ..Default::default()
            });
            let issues = resolved.diagnostics.clone();
            let app = Workbench::with_configuration(resolved, None, None).unwrap();
            let project = app.project().clone();
            let now = app.now();
            let trace = app.trace().to_vec();
            let run = app.run_id().clone();
            let mut h = harness(app);
            h.run();
            for issue in issues {
                h.get_by_label(&issue.localized(l));
            }
            h.get_by_label(l.text("configuration_readonly"));
            assert_eq!(h.state().language(), l);
            assert_eq!(h.state().display_preferences().scale_percent, 125);
            assert_eq!(
                h.state().display_preferences().representation,
                if diagram {
                    Representation::Diagram
                } else {
                    Representation::Icons
                }
            );
            #[cfg(feature = "ui-snapshots")]
            h.snapshot(format!(
                "j7-configuration-{}-{}",
                if diagram { "diagram" } else { "icons" },
                l.code()
            ));
            h.get_by_label(l.text("close_view")).click();
            h.run();
            h.state_mut().choose_language(l);
            let mut display = h.state().display_preferences().clone();
            display.scale_percent = 140;
            h.state_mut().set_display_preferences(display).unwrap();
            h.run();
            assert!(!h.state().preferences_busy());
            assert_eq!(std::fs::read(&user).unwrap(), original);
            assert_eq!(h.state().project(), &project);
            assert_eq!(h.state().now(), now);
            assert_eq!(h.state().trace(), trace);
            assert_eq!(h.state().run_id(), &run);
            #[cfg(feature = "ui-snapshots")]
            snapshots.extend_harness(&mut h);
        }
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn editing_one_preference_does_not_persist_other_command_line_overrides() {
    let dir = tempfile::tempdir().unwrap();
    let user = dir.path().join("profile.json");
    let settings = Settings {
        language: Language::De,
        display: DisplayPreferences {
            scale_percent: 120,
            ..Default::default()
        },
        ..Default::default()
    };
    sns_settings::save(&user, &settings).unwrap();
    let resolved = configuration::resolve(Sources {
        user: Some(user.clone()),
        overrides: Overrides {
            language: Some(json!("es")),
            display: serde_json::from_value(
                json!({"scale_percent":180,"representation":"diagram","layout":"tabs"}),
            )
            .unwrap(),
        },
        ..Default::default()
    });
    let mut h = harness(Workbench::with_configuration(resolved, None, None).unwrap());
    h.run();
    let project = h.state().project().clone();
    h.state_mut().choose_language(Language::It);
    let mut display = h.state().display_preferences().clone();
    display.font = UiFont::NotoSans;
    h.state_mut().set_display_preferences(display).unwrap();
    h.run();
    h.state_mut().wait_for_preferences().unwrap();
    let saved = sns_settings::load(&user).unwrap();
    assert_eq!(saved.language, Language::It);
    assert_eq!(saved.display.font, UiFont::NotoSans);
    assert_eq!(saved.display.scale_percent, 120);
    assert_eq!(saved.display.representation, Representation::Icons);
    assert_eq!(saved.display.layout, settings.display.layout);
    assert_eq!(h.state().display_preferences().scale_percent, 180);
    assert_eq!(h.state().project(), &project);
    let mut display = h.state().display_preferences().clone();
    display.scale_percent = 140;
    h.state_mut().set_display_preferences(display).unwrap();
    h.state_mut().wait_for_preferences().unwrap();
    assert_eq!(
        sns_settings::load(&user).unwrap().display.scale_percent,
        140
    );
    let restarted = Workbench::with_preferences(Some(user)).unwrap();
    assert_eq!(restarted.language(), Language::It);
    assert_eq!(restarted.display_preferences().scale_percent, 140);
    assert_eq!(
        restarted.display_preferences().representation,
        Representation::Icons
    );
}
#[test]
fn explicit_defaults_reset_hidden_persistent_values_and_cancel_remains_non_mutating() {
    let dir = tempfile::tempdir().unwrap();
    let user = dir.path().join("profile.json");
    let settings = Settings {
        display: DisplayPreferences {
            scale_percent: 160,
            font: UiFont::NotoSans,
            representation: Representation::Diagram,
            ..Default::default()
        },
        ..Default::default()
    };
    sns_settings::save(&user, &settings).unwrap();
    let resolved = configuration::resolve(Sources {
        user: Some(user.clone()),
        overrides: Overrides {
            display: serde_json::from_value(
                json!({"scale_percent":100,"font":"default","representation":"icons"}),
            )
            .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut h = harness(Workbench::with_configuration(resolved, None, None).unwrap());
    h.run();
    let l = Language::Fr;
    h.get_by_label(l.text("preferences")).click();
    h.run();
    h.get_by_label(l.text("preferences_defaults")).click();
    h.run();
    h.get_by_role(egui::Role::Window)
        .get_by_label(l.text("preferences_cancel"))
        .click();
    h.run();
    assert_eq!(sns_settings::load(&user).unwrap(), settings);
    h.get_by_label(l.text("preferences")).click();
    h.run();
    h.get_by_label(l.text("preferences_defaults")).click();
    h.run();
    h.get_by_label(l.text("apply")).click();
    h.run();
    h.state_mut().wait_for_preferences().unwrap();
    assert_eq!(
        sns_settings::load(&user).unwrap().display,
        DisplayPreferences::default()
    );
}
#[test]
fn administrator_and_project_destinations_are_protected_even_through_missing_parent_traversal() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("project.json");
    let admin = dir.path().join("admin.json");
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    sns_project::save_project(&source, &project).unwrap();
    write(&admin, json!({"configuration_version":1}));
    let original = std::fs::read(&source).unwrap();
    let old_admin = std::fs::read(&admin).unwrap();
    let alias = dir.path().join("missing/../project.json");
    let resolved = configuration::resolve(Sources {
        administrator: Some(admin.clone()),
        user: Some(alias),
        ..Default::default()
    });
    let mut app = Workbench::with_configuration(resolved, None, Some(source.clone())).unwrap();
    assert!(!app.startup_configuration().unwrap().user_writable);
    assert!(
        app.startup_configuration()
            .unwrap()
            .diagnostics
            .iter()
            .any(|d| d.code == "configuration.project_path_conflict")
    );
    app.choose_language(Language::En);
    assert!(!app.preferences_busy());
    assert!(!dir.path().join("missing").exists());
    assert_eq!(std::fs::read(&source).unwrap(), original);
    #[cfg(unix)]
    {
        let linked = dir.path().join("linked");
        std::fs::create_dir(&linked).unwrap();
        let linked_project = linked.join("project.json");
        sns_project::save_project(&linked_project, &project).unwrap();
        let alias = dir.path().join("shortcut");
        std::os::unix::fs::symlink(&linked, &alias).unwrap();
        let preferences = dir.path().join("missing/../shortcut/project.json");
        let resolved = configuration::resolve(Sources {
            user: Some(preferences),
            ..Default::default()
        });
        let mut protected =
            Workbench::with_configuration(resolved, None, Some(linked_project.clone())).unwrap();
        protected.choose_language(Language::En);
        assert!(!protected.preferences_busy());
        assert_eq!(std::fs::read(linked_project).unwrap(), original);
        assert!(!dir.path().join("missing").exists());
    }
    assert!(app.save_project_to(admin.clone()).is_err());
    assert!(app.begin_png_export(admin.clone()).is_err());
    assert_eq!(std::fs::read(&admin).unwrap(), old_admin);
    let resolved = configuration::resolve(Sources {
        administrator: Some(admin.clone()),
        user: Some(admin.clone()),
        ..Default::default()
    });
    let mut app = Workbench::with_configuration(resolved, None, None).unwrap();
    app.choose_language(Language::De);
    assert!(!app.preferences_busy());
    assert_eq!(std::fs::read(admin).unwrap(), old_admin);
}

#[test]
fn public_startup_boundary_refuses_invalid_typed_display_data() {
    let mut configuration = configuration::ResolvedConfiguration::default();
    configuration.effective.display.scale_percent = 0;
    assert!(Workbench::with_configuration(configuration, None, None).is_err());
    let mut configuration = configuration::ResolvedConfiguration::default();
    configuration.persistent.settings_version = 99;
    assert!(Workbench::with_configuration(configuration, None, None).is_err());
}
#[test]
fn ordinary_help_clicks_do_not_persist_geometry_derived_from_temporary_display_options() {
    let dir = tempfile::tempdir().unwrap();
    let user = dir.path().join("profile.json");
    sns_settings::save(&user, &Settings::default()).unwrap();
    let original = std::fs::read(&user).unwrap();
    let resolved = configuration::resolve(Sources {
        user: Some(user.clone()),
        overrides: Overrides {
            display: serde_json::from_value(json!({"scale_percent":150})).unwrap(),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut h = harness(Workbench::with_configuration(resolved, None, None).unwrap());
    h.run();
    h.key_press(egui::Key::F1);
    h.run();
    h.get_by_label(Language::Fr.text("help_about")).click();
    h.run();
    h.state_mut().wait_for_preferences().unwrap();
    assert_eq!(
        std::fs::read(user).unwrap(),
        original,
        "Consulting help is not an edit to the saved window layout"
    );
}

#[test]
fn technical_diagnostic_export_records_the_failed_operation_in_six_languages_and_two_views() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for language in Language::ALL {
        for view in ["icons", "diagram"] {
            let dir = tempfile::tempdir().unwrap();
            let project_path = dir.path().join("PRIVATE_PROJECT.json");
            let project_bytes = include_bytes!("../../../fixtures/valid/mail.json");
            std::fs::write(&project_path, project_bytes).unwrap();
            let profile = dir.path().join("profile.json");
            sns_settings::save(&profile, &Settings::default()).unwrap();
            let profile_bytes = std::fs::read(&profile).unwrap();
            let live_log = dir.path().join("live.jsonl");
            std::fs::write(&live_log, b"existing technical log").unwrap();
            let resolved = configuration::resolve(Sources {
                user: Some(profile.clone()),
                overrides: Overrides {
                    language: Some(json!(language)),
                    display: serde_json::from_value(
                        json!({"representation":view,"level":"expert"}),
                    )
                    .unwrap(),
                },
                ..Default::default()
            });
            let mut app =
                Workbench::with_configuration(resolved, None, Some(project_path.clone())).unwrap();
            app.attach_diagnostics(Default::default(), Some(live_log.clone()));
            for protected in [&project_path, &profile, &live_log] {
                assert!(app.begin_diagnostic_export(protected.clone()).is_err());
            }
            let mut h = harness(app);
            h.run();
            let project = h.state().project().clone();
            let trace = h.state().trace().to_vec();
            let now = h.state().now();
            h.state_mut()
                .begin_checkpoint_restore(
                    br#"{"password":"PRIVATE_PASSWORD","message":"PRIVATE_MAIL_BODY"}"#.to_vec(),
                )
                .unwrap();
            for _ in 0..300 {
                h.step();
                if !h.state().runtime_busy() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            assert!(!h.state().runtime_busy());
            h.run();
            h.get_by_label(language.text("runtime_menu")).click();
            h.run();
            h.get_by_label(language.text("diagnostic_export"));
            let target = dir.path().join("diagnostic.json");
            h.state_mut()
                .begin_diagnostic_export(target.clone())
                .unwrap();
            for _ in 0..300 {
                h.step();
                if !h.state().document_export_busy() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            assert!(!h.state().document_export_busy());
            h.run();
            h.get_by_label(language.text("diagnostic_saved"));
            let raw = std::fs::read_to_string(target).unwrap();
            if let Some(directory) = std::env::var_os("SNS_RECORD_J7_DIAGNOSTICS")
                && language == Language::Fr
                && view == "diagram"
            {
                std::fs::write(
                    std::path::PathBuf::from(directory).join("j7-diagnostic-export.json"),
                    &raw,
                )
                .unwrap();
            }
            assert!(!raw.contains("PRIVATE_"));
            assert_eq!(
                serde_json::from_str::<Value>(&raw).unwrap(),
                json!({
                    "diagnostic_version": 1,
                    "discarded": 0,
                    "entries": [{"sequence":1,"event":"runtime_file_failed"}]
                })
            );
            assert_eq!(h.state().project(), &project);
            assert_eq!(h.state().trace(), trace);
            assert_eq!(h.state().now(), now);
            assert_eq!(std::fs::read(project_path).unwrap(), project_bytes);
            assert_eq!(std::fs::read(profile).unwrap(), profile_bytes);
            assert_eq!(std::fs::read(live_log).unwrap(), b"existing technical log");
            #[cfg(feature = "ui-snapshots")]
            {
                h.snapshot(format!("j7-diagnostic-{view}-{}", language.code()));
                snapshots.extend_harness(&mut h);
            }
        }
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}

#[test]
fn compact_workbench_exposes_tools_in_six_languages_and_keeps_fonts_on_theme_changes() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for language in Language::ALL {
        for view in ["icons", "diagram"] {
            let resolved = configuration::resolve(Sources {
                small_screen: true,
                overrides: Overrides {
                    language: Some(json!(language)),
                    display: serde_json::from_value(
                        json!({"representation":view,"font":"noto_sans"}),
                    )
                    .unwrap(),
                },
                ..Default::default()
            });
            let app = Workbench::with_configuration(resolved, None, None).unwrap();
            let project = app.project().clone();
            let trace = app.trace().to_vec();
            let now = app.now();
            let mut h = Harness::builder()
                .with_max_steps(60)
                .with_size((840., 600.))
                .build_ui_state(|ui, state| state.show(ui), app);
            h.ctx.set_theme(egui::ThemePreference::System);
            for (theme, size) in [
                (egui::Theme::Light, egui::vec2(840., 600.)),
                (egui::Theme::Dark, egui::vec2(640., 480.)),
            ] {
                h.set_size(size);
                h.input_mut().system_theme = Some(theme);
                h.run();
                assert_eq!(h.ctx.theme(), theme);
                assert!((h.ctx.zoom_factor() - 0.9).abs() < 0.001);
                assert_eq!(
                    h.ctx.style_of(theme).text_styles[&egui::TextStyle::Body].family,
                    egui::FontFamily::Name("sns-ui-noto".into())
                );
                h.get_by_label(language.text("preferences_tools")).click();
                h.run();
                h.get_by_label(language.text("add_workstation"));
                h.get_by_label(language.text("preferences_inspector"))
                    .click();
                h.run();
                h.get_by_label(language.text("preferences_canvas")).click();
                h.run();
                assert_eq!(
                    h.state().display_preferences().representation,
                    if view == "icons" {
                        Representation::Icons
                    } else {
                        Representation::Diagram
                    }
                );
                assert_eq!(h.state().project(), &project);
                assert_eq!(h.state().trace(), trace);
                assert_eq!(h.state().now(), now);
                #[cfg(feature = "ui-snapshots")]
                if theme == egui::Theme::Light {
                    h.snapshot(format!("j7-compact-light-{view}-{}", language.code()));
                    snapshots.extend_harness(&mut h);
                }
            }
        }
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
