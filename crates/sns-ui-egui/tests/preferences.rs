use egui_kittest::{Harness, kittest::Queryable as _};
use sns_i18n::Language;
use sns_settings::display::{DisplayPreferences, PanelLayout, Representation, UiFont};
use sns_ui_egui::Workbench;
fn harness(app: Workbench) -> Harness<'static, Workbench> {
    Harness::builder()
        .with_max_steps(60)
        .with_size((1600., 1050.))
        .build_ui_state(|ui, state| state.show(ui), app)
}
fn settle(h: &mut Harness<'_, Workbench>) {
    h.step();
    h.state_mut().wait_for_worker().unwrap();
    h.run();
}
#[test]
fn enlarged_display_persists_in_six_languages_both_views_without_network_mutation() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in Language::ALL {
        for diagram in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let prefs = dir.path().join("settings.json");
            let source = dir.path().join("project.json");
            let project =
                sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json"))
                    .unwrap();
            sns_project::save_project(&source, &project).unwrap();
            let original = std::fs::read(&source).unwrap();
            let mut app =
                Workbench::with_storage(Some(prefs.clone()), None, Some(source.clone())).unwrap();
            app.choose_language(l);
            let mut h = harness(app);
            h.run();
            h.get_by_label(&format!("{} · F3", l.text("mode_test")))
                .click();
            settle(&mut h);
            let project = h.state().project().clone();
            let now = h.state().now();
            let run = h.state().run_id().clone();
            let trace = h.state().trace().to_vec();
            let display = DisplayPreferences {
                scale_percent: 150,
                font: UiFont::NotoSans,
                representation: if diagram {
                    Representation::Diagram
                } else {
                    Representation::Icons
                },
                layout: PanelLayout::Tabs,
                inspector_width: 320,
                window_width: 1440,
                window_height: 960,
                ..Default::default()
            };
            h.state_mut()
                .set_display_preferences(display.clone())
                .unwrap();
            h.run();
            assert!((h.ctx.zoom_factor() - 1.5).abs() < 0.001);
            h.get_by_label(l.text("preferences")).click();
            h.run();
            h.get_by_label(l.text("preferences_hint"));
            h.get_by_label(l.text("preferences_font"));
            #[cfg(feature = "ui-snapshots")]
            h.snapshot(format!(
                "j7-display-preferences-{}-{}",
                if diagram { "diagram" } else { "icons" },
                l.code()
            ));
            h.key_press(egui::Key::Escape);
            h.run();
            h.get_by_label(l.text("preferences_inspector")).click();
            h.run();
            h.get_by_label(l.text("preferences_canvas")).click();
            h.run();
            let mut largest = display.clone();
            largest.scale_percent = 200;
            largest.layout = PanelLayout::InspectorLeft;
            h.state_mut().set_display_preferences(largest).unwrap();
            h.run();
            h.get_by_label(l.text("preferences_tools"));
            #[cfg(feature = "ui-snapshots")]
            h.snapshot(format!(
                "j7-display-large-{}-{}",
                if diagram { "diagram" } else { "icons" },
                l.code()
            ));
            h.state_mut()
                .set_display_preferences(display.clone())
                .unwrap();
            h.state_mut().choose_language(l);
            h.run();
            assert_eq!(h.state().project(), &project);
            assert_eq!(h.state().now(), now);
            assert_eq!(h.state().run_id(), &run);
            assert_eq!(h.state().trace(), trace);
            assert_eq!(std::fs::read(&source).unwrap(), original);
            h.state_mut().wait_for_preferences().unwrap();
            let restarted = Workbench::with_preferences(Some(prefs)).unwrap();
            assert_eq!(restarted.language(), l);
            assert_eq!(restarted.display_preferences(), &display);
            #[cfg(feature = "ui-snapshots")]
            snapshots.extend_harness(&mut h);
        }
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn apply_cancel_and_keyboard_shortcut_preserve_the_network_and_focused_draft() {
    let mut h = harness(Workbench::new().unwrap());
    h.run();
    let l = Language::Fr;
    let project = h.state().project().clone();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Comma);
    h.run();
    h.get_by_label(l.text("preferences_font")).click();
    h.run();
    h.get_by_label("Noto Sans").click();
    h.run();
    h.get_by_role(egui::Role::Window)
        .get_by_label(l.text("preferences_cancel"))
        .click();
    h.run();
    assert_eq!(h.state().display_preferences().font, UiFont::Default);
    h.get_by_label(l.text("preferences")).click();
    h.run();
    h.get_by_label(l.text("preferences_font")).click();
    h.run();
    h.get_by_label("Noto Sans").click();
    h.run();
    h.get_by_label(l.text("apply")).click();
    h.run();
    assert_eq!(h.state().display_preferences().font, UiFont::NotoSans);
    assert_eq!(h.state().project(), &project);
    h.get_by_label("Équipement Poste A").click();
    h.run();
    h.get_by_label(l.text("device_name")).focus();
    h.get_by_label(l.text("device_name"))
        .type_text(" brouillon");
    h.run();
    let draft = h
        .get_by_label(l.text("device_name"))
        .value()
        .unwrap()
        .to_owned();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Comma);
    h.run();
    h.get_by_label(l.text("preferences_hint"));
    h.key_press(egui::Key::Escape);
    h.run();
    assert_eq!(
        h.get_by_label(l.text("device_name")).value().unwrap(),
        draft
    );
    assert_eq!(h.state().project(), &project);
}
#[test]
fn invalid_preferences_and_conflicting_project_paths_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let prefs = dir.path().join("settings.json");
    let broken = b"{\"settings_version\":99,\"language\":\"de\"}";
    std::fs::write(&prefs, broken).unwrap();
    let mut app = Workbench::with_preferences(Some(prefs.clone())).unwrap();
    app.set_display_preferences(DisplayPreferences {
        scale_percent: 180,
        ..Default::default()
    })
    .unwrap();
    app.choose_language(Language::It);
    assert!(!app.preferences_busy());
    assert_eq!(std::fs::read(&prefs).unwrap(), broken);
    let source = dir.path().join("project.json");
    sns_project::save_project(&source, app.project()).unwrap();
    let bytes = std::fs::read(&source).unwrap();
    let mut app =
        Workbench::with_storage(Some(source.clone()), None, Some(source.clone())).unwrap();
    app.choose_language(Language::De);
    app.set_display_preferences(DisplayPreferences {
        scale_percent: 160,
        ..Default::default()
    })
    .unwrap();
    assert!(!app.preferences_busy());
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
}
#[test]
fn help_geometry_is_remembered_and_clamped_after_restart_on_a_smaller_viewport() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let mut h = harness(Workbench::with_preferences(Some(path.clone())).unwrap());
    h.run();
    h.key_press(egui::Key::F1);
    h.run();
    let id = egui::Id::unique("workbench-help");
    let first = egui::AreaState::load(&h.ctx, id).unwrap().rect();
    let start = first.min + egui::vec2(180., 12.);
    let end = start + egui::vec2(90., 55.);
    h.hover_at(start);
    h.run();
    h.drag_at(start);
    h.run();
    h.hover_at(end);
    h.run();
    h.event(egui::Event::PointerButton {
        pos: end,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    h.state_mut().wait_for_preferences().unwrap();
    let saved = sns_settings::load(&path).unwrap();
    assert!(saved.display.windows.contains_key("help"));
    let moved = egui::AreaState::load(&h.ctx, id).unwrap().rect();
    assert!(moved.min.distance(first.min) > 20.);
    let mut restarted = harness(Workbench::with_preferences(Some(path)).unwrap());
    restarted.set_size(egui::vec2(900., 700.));
    restarted.run();
    restarted.key_press(egui::Key::F1);
    restarted.run();
    let rect = egui::AreaState::load(&restarted.ctx, id).unwrap().rect();
    let bounds = restarted.ctx.content_rect();
    assert!(rect.min.x >= bounds.min.x && rect.min.y >= bounds.min.y);
    assert!(
        rect.max.x <= bounds.max.x + 1. && rect.max.y <= bounds.max.y + 1.,
        "{rect:?} {bounds:?}"
    );
}
#[test]
fn terminal_remains_usable_at_200_percent_in_all_languages_without_advancing_protocols() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in Language::ALL {
        let mut app = Workbench::from_project(
            sns_project::parse_project(include_bytes!("../../../fixtures/valid/hosts.json"))
                .unwrap(),
        )
        .unwrap();
        app.choose_language(l);
        let mut h = harness(app);
        h.run();
        h.get_by_label(&format!("{} · F3", l.text("mode_test")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("desktop")).click();
        h.run();
        h.get_by_label(&format!("{} · {}", l.text("open"), l.text("terminal")))
            .click();
        settle(&mut h);
        let project = h.state().project().clone();
        let now = h.state().now();
        let trace = h.state().trace().to_vec();
        let run = h.state().run_id().clone();
        h.state_mut()
            .set_display_preferences(DisplayPreferences {
                scale_percent: 200,
                font: UiFont::NotoSans,
                application_scale_percent: 150,
                center_application_windows: true,
                ..Default::default()
            })
            .unwrap();
        h.run();
        h.get_by_label(l.text("terminal_input")).focus();
        h.get_by_label(l.text("terminal_input")).type_text("help");
        h.run();
        assert_eq!(
            h.get_by_label(l.text("terminal_input")).value().unwrap(),
            "help"
        );
        let host = sns_model::Id::new("host-a").unwrap();
        let application = sns_model::Id::new("host-a-terminal").unwrap();
        let window = egui::AreaState::load(
            &h.ctx,
            egui::Id::unique(("application", &host, &application)),
        )
        .unwrap()
        .rect();
        assert!(
            window
                .expand(1.)
                .contains_rect(h.get_by_label(l.text("interrupt")).rect()),
            "The terminal actions must remain inside the window: {:?}, {:?}, {:?}",
            l,
            window,
            h.get_by_label(l.text("interrupt")).rect()
        );
        assert_eq!(h.state().project(), &project);
        assert_eq!(h.state().now(), now);
        assert_eq!(h.state().run_id(), &run);
        assert_eq!(h.state().trace(), trace);
        #[cfg(feature = "ui-snapshots")]
        {
            h.snapshot(format!("j7-display-terminal-{}", l.code()));
            snapshots.extend_harness(&mut h);
        }
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn preference_parent_folders_can_be_created_without_writing_into_recovery_storage() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("sessions");
    let prefs = dir.path().join("new/nested/settings.json");
    let mut app = Workbench::with_storage(Some(prefs.clone()), Some(root.clone()), None).unwrap();
    app.choose_language(Language::Pt);
    app.wait_for_preferences().unwrap();
    assert_eq!(sns_settings::load(&prefs).unwrap().language, Language::Pt);
    #[cfg(unix)]
    {
        std::fs::create_dir_all(&root).unwrap();
        let alias = dir.path().join("recovery-alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        let mut linked = Workbench::with_storage(
            Some(alias.join("missing/settings.json")),
            Some(root.clone()),
            None,
        )
        .unwrap();
        linked.choose_language(Language::En);
        assert!(linked.wait_for_preferences().is_err());
        assert!(!root.join("missing/settings.json").exists());
    }
    let protected = root.join("new/nested/settings.json");
    let mut app = Workbench::with_storage(Some(protected.clone()), Some(root), None).unwrap();
    app.choose_language(Language::Es);
    assert!(app.wait_for_preferences().is_err());
    assert!(!protected.exists());
}
