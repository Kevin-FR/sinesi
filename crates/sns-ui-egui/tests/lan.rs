use egui_kittest::{Harness, kittest::Queryable as _};
use sns_ui_egui::{Mode, Workbench};
fn harness() -> Harness<'static, Workbench> {
    Harness::builder()
        .with_max_steps(20)
        .with_size((1320., 880.))
        .build_ui_state(|ui, state| state.show(ui), Workbench::new().unwrap())
}
fn settle(h: &mut Harness<'_, Workbench>) {
    h.step();
    h.state_mut().wait_for_worker().unwrap();
    h.run();
}
#[test]
fn palette_delete_and_undo_drive_transactional_editor() {
    let mut h = harness();
    h.run();
    h.get_by_label("Ajouter un portable").click();
    h.run();
    assert_eq!(h.state().project().devices.len(), 5);
    h.get_by_label("Supprimer la sélection").click();
    h.run();
    assert_eq!(h.state().project().devices.len(), 4);
    h.get_by_label("Annuler").click();
    h.run();
    assert_eq!(h.state().project().devices.len(), 5);
    h.get_by_label("Rétablir").click();
    h.run();
    assert_eq!(h.state().project().devices.len(), 4);
}
#[test]
fn keyboard_equipment_ping_pause_step_and_modes_preserve_project() {
    let mut h = harness();
    h.run();
    let original = h.state().project().clone();
    h.get_by_label("Équipement Poste A").focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert_eq!(h.state().selected().len(), 1);
    h.get_by_label("Tester · F3").click();
    settle(&mut h);
    assert_eq!(h.state().mode, Mode::Simulation);
    // The inspector also explains the empty gateway; bring the command into view.
    h.get_by_label("Envoyer un ping").scroll_to_me();
    h.run();
    h.get_by_label("Envoyer un ping").click();
    settle(&mut h);
    assert!(h.state().trace().is_empty());
    assert_eq!(h.state().now(), 0);
    for _ in 0..20 {
        h.get_by_label("Un événement").click();
        settle(&mut h);
        if h.state()
            .trace()
            .iter()
            .any(|e| e.kind == "icmp.echo.reply")
        {
            break;
        }
    }
    assert_eq!(
        h.state()
            .trace()
            .iter()
            .filter(|e| e.kind == "icmp.echo.reply")
            .count(),
        1
    );
    h.get_by_label("Tables ARP / SAT").click();
    h.run();
    h.get_by_label("Annoter · F4").click();
    settle(&mut h);
    assert_eq!(h.state().project(), &original);
    assert_eq!(h.state().mode, Mode::Documentation);
    h.get_by_label("Tester · F3").click();
    settle(&mut h);
    assert!(h.state().trace().is_empty());
    assert_eq!(h.state().now(), 0);
}
#[test]
fn inspector_rename_preserves_ids_and_shortcuts_respect_text_entry() {
    let mut h = harness();
    h.run();
    h.get_by_label("Équipement Poste A").click();
    h.run();
    let name = h.get_by_label("Nom de l’équipement");
    name.focus();
    h.run();
    h.key_press(egui::Key::F3);
    h.run();
    assert_eq!(h.state().mode, Mode::Design);
    h.get_by_label("Nom de l’équipement").type_text(" renommé");
    h.run();
    h.get_by_label("Appliquer les paramètres").click();
    h.run();
    assert!(h.state().project().devices[0].label.contains("renommé"));
    assert_eq!(h.state().project().devices[0].id.as_str(), "host-a");
}
#[test]
#[cfg(feature = "ui-snapshots")]
fn rendered_lan_editor_reference() {
    let mut h = harness();
    h.run();
    h.get_by_label("Équipement Poste A").click();
    h.run();
    h.snapshot("j6-editor");
}
#[test]
fn palette_drag_drop_and_rectangle_group_move_keep_cables_attached() {
    let mut h = harness();
    h.run();
    let a = h.get_by_label("Équipement Poste A").rect();
    let b = h.get_by_label("Équipement Portable B").rect();
    let start = a.min - egui::vec2(12., 12.);
    let end = b.max + egui::vec2(12., 12.);
    h.hover_at(start);
    h.run();
    h.drag_at(start);
    h.run();
    h.hover_at(start + egui::vec2(5., 5.));
    h.run();
    h.hover_at(end);
    h.run();
    h.drop_at(end);
    h.run();
    assert_eq!(h.state().selected().len(), 2);
    let before = h.state().project().clone();
    let origin = a.center();
    let delta = egui::vec2(30., 20.);
    h.hover_at(origin);
    h.run();
    h.drag_at(origin);
    h.run();
    h.hover_at(origin + delta);
    h.run();
    h.drop_at(origin + delta);
    h.run();
    assert_ne!(h.state().project().layout, before.layout);
    assert_eq!(h.state().project().links, before.links);
    assert_eq!(h.state().selected().len(), 2);
    h.get_by_label("Annuler").click();
    h.run();
    assert_eq!(h.state().project().layout, before.layout);
    let palette = h.get_by_label("Ajouter un portable").rect().center();
    let destination = egui::pos2(600., 800.);
    h.hover_at(palette);
    h.run();
    h.drag_at(palette);
    h.run();
    h.hover_at(palette + egui::vec2(8., 8.));
    h.run();
    h.hover_at(destination);
    h.run();
    h.drop_at(destination);
    h.run();
    assert_eq!(h.state().project().devices.len(), 5);
}
#[test]
fn keyboard_connects_two_hosts_and_inspects_real_echo() {
    fn activate(h: &mut Harness<'_, Workbench>, label: &str) {
        h.get_by_label(label).focus();
        h.run();
        h.key_press(egui::Key::Enter);
        h.run();
    }
    let mut h = harness();
    h.run();
    activate(&mut h, "Nouveau");
    // New replaces the worker generation. Wait for its acknowledgement before
    // trying the palette, which correctly refuses edits while that is pending.
    settle(&mut h);
    activate(&mut h, "Ajouter un ordinateur");
    let first = h.state().project().devices[0].clone();
    h.get_by_label("Adresse IPv4 statique").focus();
    h.run();
    h.get_by_label("Adresse IPv4 statique")
        .type_text("192.168.10.2");
    h.run();
    activate(&mut h, "Appliquer les paramètres");
    activate(&mut h, "Ajouter un portable");
    let second = h.state().project().devices[1].clone();
    h.get_by_label("Adresse IPv4 statique").focus();
    h.run();
    h.get_by_label("Adresse IPv4 statique")
        .type_text("192.168.10.3");
    h.run();
    activate(&mut h, "Appliquer les paramètres");
    activate(
        &mut h,
        &format!("{} · libre", second.interfaces[0].id.as_str()),
    );
    activate(&mut h, &format!("{} · Ordinateur", first.label));
    activate(
        &mut h,
        &format!("{} · libre", first.interfaces[0].id.as_str()),
    );
    assert_eq!(h.state().project().links.len(), 1);
    activate(&mut h, "Tester · F3");
    settle(&mut h);
    h.get_by_label("Destination du ping")
        .set_text_value_accesskit("192.168.10.3");
    h.run();
    activate(&mut h, "Envoyer un ping");
    settle(&mut h);
    for _ in 0..12 {
        activate(&mut h, "Un événement");
        settle(&mut h);
        if h.state()
            .trace()
            .iter()
            .any(|f| f.kind == "icmp.echo.reply")
        {
            break;
        }
    }
    assert!(
        h.state()
            .trace()
            .iter()
            .any(|f| f.kind == "icmp.echo.reply")
    );
    activate(&mut h, "Échanges LAN");
    activate(&mut h, "Tables ARP / SAT");
}
#[test]
fn documentation_and_unsaved_replacement_preserve_project_until_decision() {
    let mut h = harness();
    h.run();
    h.get_by_label("Annoter · F4").click();
    h.run();
    h.get_by_label("Texte de l’annotation").focus();
    h.run();
    h.get_by_label("Texte de l’annotation")
        .type_text("Exercice : observer ARP");
    h.run();
    h.get_by_label("Ajouter l’annotation").click();
    h.run();
    assert_eq!(h.state().project().annotations.len(), 1);
    let annotated = h.state().project().clone();
    h.get_by_label("Construire · F2").click();
    h.run();
    h.get_by_label("Nouveau").click();
    h.run();
    h.get_by_label("Continuer l’édition").click();
    h.run();
    assert_eq!(h.state().project(), &annotated);
    h.get_by_label("Nouveau").click();
    h.run();
    h.get_by_label("Abandonner les modifications").click();
    settle(&mut h);
    assert!(h.state().project().devices.is_empty());
    assert!(h.state().trace().is_empty());
}
#[test]
fn six_languages_change_visible_controls_without_modifying_project() {
    use sns_i18n::Language;
    let mut h = harness();
    h.run();
    let project = h.state().project().clone();
    for language in [
        Language::En,
        Language::Es,
        Language::It,
        Language::Pt,
        Language::De,
        Language::Fr,
    ] {
        let previous = h.state().language();
        h.get_by_label(previous.text("language")).click();
        h.run();
        h.get_by_label(language.label()).click();
        h.run();
        assert_eq!(h.state().language(), language);
        h.get_by_label(language.text("new"));
        h.get_by_label(language.text("save"));
        h.get_by_label(language.text("add_workstation"));
        h.get_by_label(language.text("exchanges"));
        assert_eq!(h.state().project(), &project);
    }
}
#[test]
fn preferred_language_survives_new_ui_instance_and_new_documents_use_it() {
    use sns_i18n::Language;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let mut state = Workbench::with_preferences(Some(path.clone())).unwrap();
    let project = state.project().clone();
    state.choose_language(Language::De);
    state.wait_for_preferences().unwrap();
    assert_eq!(state.project(), &project);
    let state = Workbench::with_preferences(Some(path)).unwrap();
    assert_eq!(state.language(), Language::De);
    let mut h = Harness::builder()
        .with_max_steps(20)
        .with_size((1320., 880.))
        .build_ui_state(|ui, state: &mut Workbench| state.show(ui), state);
    h.run();
    h.get_by_label(Language::De.text("new")).click();
    h.run();
    assert_eq!(h.state().project().name, Language::De.text("new_network"));
}
#[test]
#[cfg(feature = "ui-snapshots")]
fn rendered_six_languages_reference() {
    let mut h = harness();
    h.run();
    h.get_by_label("Équipement Poste A").click();
    h.run();
    for language in sns_i18n::Language::ALL {
        h.state_mut().choose_language(language);
        h.run();
        h.snapshot(format!("j6-editor-{}", language.code()));
    }
}

#[test]
fn six_language_desktops_terminal_history_and_persistent_document() {
    use sns_i18n::Language;
    use sns_model::{Id, Software};
    for language in Language::ALL {
        let mut app = Workbench::new().unwrap();
        app.choose_language(language);
        let mut h = Harness::builder()
            .with_max_steps(20)
            .with_size((1440., 1000.))
            .build_ui_state(|ui, state| state.show(ui), app);
        h.run();
        h.get_by_label(&format!("{} · F3", language.text("mode_test")))
            .click();
        settle(&mut h);
        h.get_by_label(language.text("desktop")).click();
        h.run();
        h.get_by_label(&format!(
            "{} · {}",
            language.text("install"),
            language.text("terminal")
        ))
        .click();
        settle(&mut h);
        assert_eq!(
            h.state()
                .project()
                .device(&Id::new("host-a").unwrap())
                .unwrap()
                .applications[0]
                .kind,
            Software::Terminal.id()
        );
        h.get_by_label(&format!(
            "{} · {}",
            language.text("open"),
            language.text("terminal")
        ))
        .click();
        settle(&mut h);
        h.get_by_label(language.text("terminal_input")).focus();
        h.run();
        h.get_by_label(language.text("terminal_input"))
            .type_text("mkdir notes");
        h.run();
        h.key_press(egui::Key::Enter);
        settle(&mut h);
        assert!(
            h.state()
                .project()
                .device(&Id::new("host-a").unwrap())
                .unwrap()
                .files
                .iter()
                .any(|f| f.path == "/notes")
        );
        h.key_press(egui::Key::ArrowUp);
        h.run();
        assert_eq!(
            h.get_by_label(language.text("terminal_input"))
                .value()
                .as_deref(),
            Some("mkdir notes")
        );
        h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::C);
        settle(&mut h);
        h.get_by_label(language.text("terminal_input"))
            .type_text("help ping");
        h.run();
        h.key_press(egui::Key::Enter);
        settle(&mut h);
        h.get_by_label_contains(language.text("terminal_help"));
        h.get_by_label(&format!("{} · F2", language.text("mode_build")))
            .click();
        settle(&mut h);
        assert!(
            h.state()
                .project()
                .device(&Id::new("host-a").unwrap())
                .unwrap()
                .files
                .iter()
                .any(|f| f.path == "/notes")
        );
    }
}
#[test]
fn traffic_filters_export_and_clear_preserve_immutable_trace() {
    use sns_engine::{Engine, Limits};
    use sns_model::*;
    use sns_ui_egui::traffic::TrafficFilter;
    let mut e = Engine::from_project(
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap(),
        42,
        Limits::default(),
    )
    .unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: Id::new("ping").unwrap(),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(0),
        command: Command::Ping {
            source: Id::new("host-a").unwrap(),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    })
    .unwrap();
    e.run(10000, Some(5_000_000)).unwrap();
    let original = e.canonical_trace();
    let mut filter = TrafficFilter::default();
    let all = filter.export_text(e.trace(), "", sns_i18n::Language::Fr);
    assert_eq!(all.lines().count(), e.trace().len() + 1);
    filter.interface = Some(
        e.project()
            .device(&Id::new("host-a").unwrap())
            .unwrap()
            .interfaces[0]
            .id
            .clone(),
    );
    let count = e.trace().iter().filter(|f| filter.matches(f, "")).count();
    assert!(count > 0 && count < e.trace().len());
    filter.layers[0] = false;
    assert!(e.trace().iter().filter(|f| filter.matches(f, "")).count() < count);
    filter.layers = [false; 4];
    assert!(!e.trace().iter().any(|f| filter.matches(f, "")));
    filter = Default::default();
    filter.after = e.trace().last().unwrap().sequence.0;
    assert_eq!(
        filter
            .export_text(e.trace(), "", sns_i18n::Language::Fr)
            .lines()
            .count(),
        1
    );
    assert_eq!(e.canonical_trace(), original);
}

fn replace_text(h: &mut Harness<'_, Workbench>, label: &str, text: &str) {
    h.get_by_label(label).focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label(label).type_text(text);
    h.run();
}
fn hosts_harness(language: sns_i18n::Language) -> Harness<'static, Workbench> {
    let mut app = Workbench::from_project(
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/hosts.json")).unwrap(),
    )
    .unwrap();
    app.choose_language(language);
    let mut h = Harness::builder()
        .with_max_steps(20)
        .with_size((1440., 1000.))
        .build_ui_state(|ui, state| state.show(ui), app);
    h.run();
    h.get_by_label(&format!("{} · F3", language.text("mode_test")))
        .click();
    settle(&mut h);
    h.get_by_label(language.text("desktop")).click();
    h.run();
    h
}
#[test]
fn virtual_editor_saves_and_image_viewer_preserves_binary_in_six_languages() {
    use sns_i18n::Language;
    use sns_model::Id;
    for l in Language::ALL {
        let mut h = hosts_harness(l);
        let original = h.state().project().clone();
        h.get_by_label(&format!("{} · {}", l.text("open"), l.text("text_editor")))
            .click();
        settle(&mut h);
        replace_text(&mut h, l.text("path"), "/index.html");
        h.get_by_label(l.text("open_file")).click();
        h.run();
        replace_text(&mut h, l.text("content"), "<h1>Édition — UTF-8</h1>");
        assert_eq!(h.state().project(), &original);
        h.get_by_label(l.text("virtual_save")).click();
        settle(&mut h);
        let content = &h
            .state()
            .project()
            .device(&Id::new("host-a").unwrap())
            .unwrap()
            .files
            .iter()
            .find(|f| f.path == "/index.html")
            .unwrap()
            .data_base64;
        use base64::Engine as _;
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(content)
                .unwrap(),
            "<h1>Édition — UTF-8</h1>".as_bytes()
        );
        replace_text(&mut h, l.text("destination"), "/copie.txt");
        h.get_by_label(l.text("virtual_save_as")).click();
        settle(&mut h);
        assert!(
            h.state()
                .project()
                .device(&Id::new("host-a").unwrap())
                .unwrap()
                .files
                .iter()
                .any(|f| f.path == "/copie.txt")
        );
        h.get_by_label(l.text("close_view")).click();
        settle(&mut h);
        h.get_by_label(&format!("{} · {}", l.text("open"), l.text("image_viewer")))
            .click();
        settle(&mut h);
        replace_text(&mut h, l.text("path"), "/pixel.png");
        let before = h.state().project().clone();
        h.get_by_label(l.text("open_file")).click();
        h.run();
        h.get_by_role(egui::Role::Image);
        assert_eq!(h.state().project(), &before);
    }
}

#[test]
#[cfg(feature = "ui-snapshots")]
fn rendered_desktop_and_terminal_six_languages() {
    let mut h = hosts_harness(sns_i18n::Language::Fr);
    let l = sns_i18n::Language::Fr;
    h.get_by_label(&format!("{} · {}", l.text("open"), l.text("terminal")))
        .click();
    settle(&mut h);
    h.get_by_label(l.text("terminal_input")).focus();
    h.run();
    h.get_by_label(l.text("terminal_input"))
        .type_text("ipconfig");
    h.run();
    h.key_press(egui::Key::Enter);
    settle(&mut h);
    for l in sns_i18n::Language::ALL {
        h.state_mut().choose_language(l);
        h.run();
        h.snapshot(format!("j6-desktop-{}", l.code()));
    }
}

#[test]
fn tcp_windows_use_actual_transport_and_close_view_keeps_server_active() {
    use sns_i18n::Language;
    let l = Language::Fr;
    let mut h = hosts_harness(l);
    h.get_by_label("Portable B").click();
    h.run();
    h.get_by_label("Ouvrir… · Serveur echo").click();
    settle(&mut h);
    h.get_by_label(l.text("start_service")).click();
    settle(&mut h);
    h.get_by_label(l.text("close_view")).click();
    settle(&mut h);
    h.get_by_label("Poste A").click();
    h.run();
    h.get_by_label("Ouvrir… · Client TCP").click();
    settle(&mut h);
    h.get_by_label(l.text("connect")).click();
    settle(&mut h);
    for _ in 0..30 {
        h.get_by_label(l.text("step")).click();
        settle(&mut h);
    }
    h.get_by_label(l.text("message")).focus();
    h.run();
    h.get_by_label(l.text("message")).type_text("Écho réel");
    h.run();
    h.get_by_label(l.text("send")).click();
    settle(&mut h);
    for _ in 0..30 {
        h.get_by_label(l.text("step")).click();
        settle(&mut h);
    }
    assert!(
        h.state()
            .trace()
            .iter()
            .any(|f| f.kind == "application.data.received" && f.device_id.as_str() == "host-a")
    );
    h.get_by_label("Écho réel");
}
#[test]
fn explorer_copy_move_delete_and_host_switch_use_isolated_documents() {
    use sns_i18n::Language;
    use sns_model::Id;
    let l = Language::Fr;
    let mut h = hosts_harness(l);
    h.get_by_label("Ouvrir… · Explorateur de fichiers").click();
    settle(&mut h);
    replace_text(&mut h, l.text("destination"), "/docs");
    h.get_by_label(l.text("new_folder")).click();
    settle(&mut h);
    replace_text(&mut h, l.text("source_path"), "/index.html");
    replace_text(&mut h, l.text("destination"), "/docs/copied.html");
    h.get_by_label(l.text("copy")).click();
    settle(&mut h);
    replace_text(&mut h, l.text("source_path"), "/docs/copied.html");
    replace_text(&mut h, l.text("destination"), "/docs/renamed.html");
    h.get_by_label(l.text("rename_move")).click();
    settle(&mut h);
    let files = &h
        .state()
        .project()
        .device(&Id::new("host-a").unwrap())
        .unwrap()
        .files;
    assert!(files.iter().any(|f| f.path == "/docs/renamed.html"));
    assert!(!files.iter().any(|f| f.path == "/docs/copied.html"));
    h.get_by_label(l.text("delete")).click();
    settle(&mut h);
    assert!(
        !h.state()
            .project()
            .device(&Id::new("host-a").unwrap())
            .unwrap()
            .files
            .iter()
            .any(|f| f.path == "/docs/renamed.html")
    );
    h.get_by_label(l.text("close_view")).click();
    settle(&mut h);
    h.get_by_label("Portable B").click();
    h.run();
    assert!(
        h.state()
            .project()
            .device(&Id::new("host-b").unwrap())
            .unwrap()
            .files
            .is_empty()
    );
}

#[test]
fn failed_file_save_preserves_draft_and_close_requires_explicit_discard() {
    let l = sns_i18n::Language::Fr;
    let mut h = hosts_harness(l);
    h.get_by_label("Ouvrir… · Éditeur de texte").click();
    settle(&mut h);
    replace_text(&mut h, l.text("path"), "/missing/note.txt");
    replace_text(&mut h, l.text("content"), "Brouillon à conserver");
    h.get_by_label(l.text("virtual_save")).click();
    settle(&mut h);
    assert_eq!(
        h.get_by_label(l.text("content")).value().as_deref(),
        Some("Brouillon à conserver")
    );
    h.get_by_label(l.text("close_view")).click();
    h.run();
    h.get_by_label(l.text("unsaved_prompt"));
    h.get_by_label(l.text("continue_editing")).click();
    h.run();
    assert_eq!(
        h.get_by_label(l.text("content")).value().as_deref(),
        Some("Brouillon à conserver")
    );
    replace_text(&mut h, l.text("destination"), "/note.txt");
    h.get_by_label(l.text("virtual_save_as")).click();
    settle(&mut h);
    h.get_by_label(l.text("close_view")).click();
    settle(&mut h);
    assert!(h.query_by_label(l.text("content")).is_none());
}

#[test]
fn j3_router_palette_and_pedagogical_views_are_localized_without_changing_topology() {
    use sns_i18n::Language;
    use sns_model::*;
    for l in Language::ALL {
        let mut app = Workbench::new().unwrap();
        app.choose_language(l);
        let mut h = Harness::builder()
            .with_max_steps(20)
            .with_size((1440., 1000.))
            .build_ui_state(|ui, state| state.show(ui), app);
        h.run();
        h.get_by_label(l.text("add_router")).click();
        h.run();
        assert_eq!(
            h.state().project().devices.last().unwrap().kind,
            DeviceKind::Router
        );
        assert_eq!(
            h.state().project().devices.last().unwrap().interfaces.len(),
            4
        );
        let before = h.state().project().clone();
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        assert_eq!(h.state().project(), &before);
        h.get_by_label(l.text("view_icons")).click();
        h.run();
        assert_eq!(h.state().project(), &before);
        h.get_by_label(&l.format("canvas_device", &[("label", "Poste A")]).unwrap())
            .click();
        h.run();
        h.get_by_label(l.text("device_symbol")).click();
        h.run();
        h.get_by_label(l.text("server_device")).click();
        h.run();
        h.get_by_label(l.text("apply")).scroll_to_me();
        h.run();
        h.get_by_label(l.text("apply")).click();
        h.run();
        assert_eq!(
            h.state().project().lan[&Id::new("host-a").unwrap()].symbol,
            Some(DeviceSymbol::Server)
        );
        let reopened =
            sns_project::parse_project(&serde_json::to_vec(h.state().project()).unwrap()).unwrap();
        assert_eq!(&reopened, h.state().project());
    }
}
#[test]
fn j3_terminal_traceroute_is_usable_in_each_language() {
    for l in sns_i18n::Language::ALL {
        let p = sns_project::parse_project(include_bytes!("../../../fixtures/valid/routed.json"))
            .unwrap();
        let mut app = Workbench::from_project(p).unwrap();
        app.choose_language(l);
        let mut h = Harness::builder()
            .with_max_steps(20)
            .with_size((1440., 1000.))
            .build_ui_state(|ui, state| state.show(ui), app);
        h.run();
        h.get_by_label(&format!("{} · F3", l.text("mode_test")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("desktop")).click();
        h.run();
        h.get_by_label(&format!("{} · {}", l.text("open"), l.text("terminal")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("terminal_input")).focus();
        h.run();
        h.get_by_label(l.text("terminal_input"))
            .type_text("traceroute -m 1 192.168.30.2");
        h.run();
        h.key_press(egui::Key::Enter);
        settle(&mut h);
        for _ in 0..18 {
            h.get_by_label(l.text("step")).click();
            settle(&mut h);
        }
        assert!(
            h.state()
                .trace()
                .iter()
                .any(|e| e.kind == "icmp.error" && e.payload["reason"] == "ipv4.ttl_expired")
        );
        h.get_by_label_contains(
            &l.format(
                "trace_hop",
                &[("hop", "1"), ("source", "192.168.10.1"), ("rtt", "4000")],
            )
            .unwrap(),
        );
    }
}

#[test]
#[cfg(feature = "ui-snapshots")]
fn rendered_j3_icons_diagram_and_rip_routing_table() {
    let p = sns_project::parse_project(include_bytes!("../../../fixtures/valid/rip.json")).unwrap();
    let app = Workbench::from_project(p).unwrap();
    let mut h = Harness::builder()
        .with_max_steps(20)
        .with_size((1440., 1000.))
        .build_ui_state(|ui, state| state.show(ui), app);
    h.run();
    h.snapshot("j6-network-icons");
    h.get_by_label(sns_i18n::Language::Fr.text("view_diagram"))
        .click();
    h.run();
    h.snapshot("j6-network-diagram");
    h.get_by_label("Tester · F3").click();
    settle(&mut h);
    for _ in 0..70 {
        h.get_by_label("Un événement").click();
        settle(&mut h);
    }
    h.get_by_label("Équipement R1").click();
    h.run();
    h.get_by_label(sns_i18n::Language::Fr.text("enable_rip"))
        .scroll_to_me();
    h.run();
    h.snapshot("j6-rip-table");
}

#[test]
fn static_route_inspector_creates_edits_and_removes_a_persistent_route() {
    use sns_model::Id;
    let app = Workbench::from_project(
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/routed.json")).unwrap(),
    )
    .unwrap();
    let mut h = Harness::builder()
        .with_max_steps(20)
        .with_size((1800., 1600.))
        .build_ui_state(|ui, state| state.show(ui), app);
    h.run();
    h.get_by_label("Équipement R1").click();
    h.run();
    fn field(h: &mut Harness<'_, Workbench>, label: &str, value: &str) {
        h.get_all_by_label(label).last().unwrap().scroll_to_me();
        h.run();
        h.get_all_by_label(label).last().unwrap().focus();
        h.run();
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.run();
        h.get_all_by_label(label).last().unwrap().type_text(value);
        h.run();
    }
    let l = sns_i18n::Language::Fr;
    field(&mut h, l.text("route_network"), "198.51.100.0");
    h.get_by_label(l.text("save_route")).click();
    h.run();
    assert_eq!(
        h.state()
            .project()
            .device(&Id::new("r1").unwrap())
            .unwrap()
            .network
            .routes
            .len(),
        2
    );
    h.get_all_by_label(l.text("edit_route"))
        .last()
        .unwrap()
        .click();
    h.run();
    field(&mut h, l.text("route_network"), "198.51.101.0");
    h.get_by_label(l.text("save_route")).click();
    h.run();
    assert_eq!(
        h.state()
            .project()
            .device(&Id::new("r1").unwrap())
            .unwrap()
            .network
            .routes
            .last()
            .unwrap()
            .network
            .to_string(),
        "198.51.101.0"
    );
    h.get_all_by_label(l.text("delete_route"))
        .last()
        .unwrap()
        .click();
    h.run();
    assert_eq!(
        h.state()
            .project()
            .device(&Id::new("r1").unwrap())
            .unwrap()
            .network
            .routes
            .len(),
        1
    );
}
#[test]
#[cfg(feature = "ui-snapshots")]
fn rendered_classroom_uses_the_supplied_school_symbols() {
    let app = Workbench::from_project(
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/classroom.json"))
            .unwrap(),
    )
    .unwrap();
    let mut h = Harness::builder()
        .with_max_steps(20)
        .with_size((1440., 1000.))
        .build_ui_state(|ui, state| state.show(ui), app);
    h.run();
    h.snapshot("j6-classroom-icons");
    h.get_by_label("Schéma").click();
    h.run();
    h.snapshot("j6-classroom-diagram");
}
fn services_harness() -> Harness<'static, Workbench> {
    let p = sns_project::parse_project(include_bytes!("../../../fixtures/valid/services.json"))
        .unwrap();
    let mut h = Harness::builder()
        .with_max_steps(20)
        .with_size((1500., 1100.))
        .build_ui_state(
            |ui, state| state.show(ui),
            Workbench::from_project(p).unwrap(),
        );
    h.run();
    h
}
#[test]
fn j4_dhcp_controls_and_service_panels_preserve_six_languages_and_views() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut h = services_harness();
        h.state_mut().choose_language(l);
        h.run();
        h.get_by_label(&l.format("canvas_device", &[("label", "Poste A")]).unwrap())
            .click();
        h.run();
        h.get_by_label(l.text("dhcp_client"));
        let original = h.state().project().clone();
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        assert_eq!(h.state().project(), &original);
        h.get_by_label(&format!("{} · F3", l.text("mode_test")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("desktop")).click();
        h.run();
        h.get_by_label("Serveur école").click();
        h.run();
        h.get_by_label(&format!("{} · {}", l.text("open"), l.text("dns_software")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("dns_records_hint"));
        h.get_by_label(l.text("dns_recursion"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-dns-{}", l.code()));
        h.get_by_label(l.text("close_view")).click();
        settle(&mut h);
        h.get_by_label(&format!("{} · {}", l.text("open"), l.text("web_server")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("http_form_path"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-http-{}", l.code()));
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn j4_browser_ui_generates_http_for_page_link_image_and_form() {
    let mut h = services_harness();
    let l = sns_i18n::Language::Fr;
    h.get_by_label("Tester · F3").click();
    settle(&mut h);
    h.get_by_label(l.text("desktop")).click();
    h.run();
    h.get_by_label("Serveur école").click();
    h.run();
    for software in ["dns_software", "web_server"] {
        h.get_by_label(&format!("{} · {}", l.text("open"), l.text(software)))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("start_service")).click();
        settle(&mut h);
        h.get_by_label(l.text("close_view")).click();
        settle(&mut h);
    }
    for _ in 0..80 {
        h.get_by_label(l.text("step")).click();
        settle(&mut h);
        if h.state()
            .trace()
            .iter()
            .filter(|e| e.kind == "dhcp.lease.acquired")
            .count()
            == 2
        {
            break;
        }
    }
    h.get_by_label("Poste A").click();
    h.run();
    h.get_by_label(l.text("state")).click();
    h.run();
    h.get_by_label_contains("192.168.1.10");
    h.get_by_label_contains("DNS: 192.168.1.3");
    h.get_by_label(l.text("state")).click();
    h.run();
    h.get_by_label(&format!("{} · {}", l.text("open"), l.text("web_browser")))
        .click();
    settle(&mut h);
    h.get_by_label(l.text("http_go")).click();
    settle(&mut h);
    for _ in 0..180 {
        h.get_by_label(l.text("step")).click();
        settle(&mut h);
        if h.state()
            .trace()
            .iter()
            .filter(|e| e.kind == "http.response.received")
            .count()
            >= 2
        {
            break;
        }
    }
    assert!(
        h.state()
            .trace()
            .iter()
            .filter(|e| e.kind.starts_with("dhcp.")
                || e.kind.starts_with("dns.")
                || e.kind.starts_with("http."))
            .all(|e| sns_ui_egui::traffic::layer(e) == 3)
    );
    h.get_by_label("École");
    h.get_by_label("Suite");
    h.get_by_label("Prénom");
    #[cfg(feature = "ui-snapshots")]
    for lang in sns_i18n::Language::ALL {
        h.state_mut().choose_language(lang);
        h.run();
        h.snapshot(format!("j6-browser-{}", lang.code()));
    }
    h.state_mut().choose_language(l);
    h.run();
    for link in ["Suite", "Accueil"] {
        let before = h
            .state()
            .trace()
            .iter()
            .filter(|e| e.kind == "http.response.received")
            .count();
        h.get_by_label(link).scroll_to_me();
        h.run();
        h.get_by_label(link).click();
        settle(&mut h);
        for _ in 0..160 {
            h.get_by_label(l.text("step")).click();
            settle(&mut h);
            if h.state()
                .trace()
                .iter()
                .filter(|e| e.kind == "http.response.received")
                .count()
                > before
            {
                break;
            }
        }
    }
    h.get_by_label("Envoyer").scroll_to_me();
    h.run();
    h.get_by_label("Envoyer").click();
    settle(&mut h);
    let before = h
        .state()
        .trace()
        .iter()
        .filter(|e| e.kind == "http.response.received")
        .count();
    for _ in 0..180 {
        h.get_by_label(l.text("step")).click();
        settle(&mut h);
        if h.state()
            .trace()
            .iter()
            .filter(|e| e.kind == "http.response.received")
            .count()
            > before
        {
            break;
        }
    }
    h.get_by_label("POST");
    assert!(
        h.state()
            .trace()
            .iter()
            .any(|e| e.kind == "http.request.sent")
    );
}
fn private_harness(l: sns_i18n::Language) -> Harness<'static, Workbench> {
    let p =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/private.json")).unwrap();
    let mut app = Workbench::from_project(p).unwrap();
    app.choose_language(l);
    Harness::builder()
        .with_max_steps(20)
        .with_size((1600., 1150.))
        .build_ui_state(|ui, state| state.show(ui), app)
}
#[test]
fn j5_home_router_roles_views_and_six_languages_are_preserved() {
    for l in sns_i18n::Language::ALL {
        let mut h = private_harness(l);
        h.run();
        h.get_by_label(l.text("add_gateway")).click();
        h.run();
        let p = h.state().project().clone();
        let g = p.devices.last().unwrap();
        let roles = g.network.gateway.as_ref().unwrap();
        assert_eq!(g.interfaces.len(), 2);
        h.get_by_label(&format!(
            "LAN · {} · {}",
            roles.lan.as_str(),
            l.text("port_free")
        ));
        h.get_by_label(&format!(
            "WAN · {} · {}",
            roles.wan.as_str(),
            l.text("port_free")
        ));
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        assert_eq!(h.state().project(), &p);
        h.get_by_label(l.text("view_icons")).click();
        h.run();
        assert_eq!(h.state().project(), &p);
    }
}
#[test]
fn j5_firewall_native_window_edits_adds_and_deletes_rules_in_all_languages() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut h = private_harness(l);
        h.run();
        h.get_by_label(&format!("{} · F3", l.text("mode_test")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("desktop")).click();
        h.run();
        h.get_by_label("Routeur domestique").click();
        h.run();
        h.get_by_label(&format!("{} · {}", l.text("open"), l.text("firewall")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("firewall_enable")).click();
        h.run();
        h.get_by_label(l.text("firewall_add_rule")).click();
        h.run();
        h.get_by_label(l.text("apply_firewall")).scroll_to_me();
        h.run();
        h.get_by_label(l.text("apply_firewall")).click();
        settle(&mut h);
        let config = h
            .state()
            .project()
            .devices
            .iter()
            .find(|d| d.id.as_str() == "gateway")
            .unwrap()
            .network
            .firewall
            .as_ref()
            .unwrap();
        assert!(config.enabled);
        assert_eq!(config.rules.len(), 1);
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-firewall-{}", l.code()));
        h.get_by_label(l.text("delete")).scroll_to_me();
        h.run();
        h.get_by_label(l.text("delete")).click();
        h.run();
        h.get_by_label(l.text("apply_firewall")).click();
        settle(&mut h);
        assert!(
            h.state()
                .project()
                .devices
                .iter()
                .find(|d| d.id.as_str() == "gateway")
                .unwrap()
                .network
                .firewall
                .as_ref()
                .unwrap()
                .rules
                .is_empty()
        );
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
fn load_private_admin(h: &mut Harness<'_, Workbench>, url: &str) {
    let l = h.state().language();
    h.get_by_label(l.text("http_url")).focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.get_by_label(l.text("http_url")).type_text(url);
    h.run();
    h.get_by_label(l.text("http_go")).click();
    settle(h);
    let before = h
        .state()
        .trace()
        .iter()
        .filter(|e| e.kind == "http.response.received")
        .count();
    for _ in 0..160 {
        h.get_by_label(l.text("step")).click();
        settle(h);
        if h.state()
            .trace()
            .iter()
            .filter(|e| e.kind == "http.response.received")
            .count()
            > before
        {
            break;
        }
    }
}
#[test]
fn j5_web_administration_changes_the_same_model_and_localizes_six_languages() {
    let l = sns_i18n::Language::Fr;
    let mut h = private_harness(l);
    h.run();
    h.get_by_label("Tester · F3").click();
    settle(&mut h);
    h.get_by_label(l.text("desktop")).click();
    h.run();
    h.get_by_label("Poste A").click();
    h.run();
    h.get_by_label(&format!("{} · {}", l.text("open"), l.text("web_browser")))
        .click();
    settle(&mut h);
    load_private_admin(&mut h, "http://192.168.10.1/admin/firewall?lang=fr");
    h.get_by_label(l.text("firewall_default_accept"))
        .scroll_to_me();
    h.run();
    h.get_by_label(l.text("firewall_default_accept")).click();
    settle(&mut h);
    let before = h
        .state()
        .trace()
        .iter()
        .filter(|e| e.kind == "http.response.received")
        .count();
    for _ in 0..180 {
        h.get_by_label(l.text("step")).click();
        settle(&mut h);
        if h.state()
            .trace()
            .iter()
            .filter(|e| e.kind == "http.response.received")
            .count()
            > before
        {
            break;
        }
    }
    assert_eq!(
        h.state()
            .project()
            .devices
            .iter()
            .find(|d| d.id.as_str() == "gateway")
            .unwrap()
            .network
            .firewall
            .as_ref()
            .unwrap()
            .default_policy,
        sns_model::Verdict::Accept
    );
    for lang in sns_i18n::Language::ALL {
        h.state_mut().choose_language(lang);
        h.run();
        load_private_admin(
            &mut h,
            &format!("http://192.168.10.1/admin?lang={}", lang.code()),
        );
        h.get_by_label(lang.text("admin_title"));
        h.get_by_label(lang.text("admin_routes"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-admin-{}", lang.code()));
    }
}
#[test]
fn j5_nat_table_updates_and_purges_without_reopening_the_panel() {
    let l = sns_i18n::Language::Fr;
    let mut h = private_harness(l);
    h.run();
    h.get_by_label(&l.format("canvas_device", &[("label", "Poste A")]).unwrap())
        .click();
    h.run();
    h.get_by_label("Tester · F3").click();
    settle(&mut h);
    h.get_by_label(l.text("ping_target")).focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.get_by_label(l.text("ping_target"))
        .type_text("198.51.100.10");
    h.run();
    h.get_by_label(l.text("send_ping")).click();
    settle(&mut h);
    for _ in 0..140 {
        h.get_by_label(l.text("step")).click();
        settle(&mut h);
        if h.state()
            .trace()
            .iter()
            .any(|f| f.kind == "icmp.echo.reply")
        {
            break;
        }
    }
    assert!(
        h.state()
            .trace()
            .iter()
            .any(|f| f.kind == "nat.mapping.created")
    );
    h.get_by_label(
        &l.format("canvas_device", &[("label", "Routeur domestique")])
            .unwrap(),
    )
    .click();
    h.run();
    h.get_by_label(l.text("private_network")).scroll_to_me();
    h.run();
    h.get_by_label(l.text("private_network")).click();
    h.run();
    h.get_by_label(l.text("nat_table")).scroll_to_me();
    h.run();
    h.get_by_label_contains("192.168.10.2 · ICMP");
    #[cfg(feature = "ui-snapshots")]
    for lang in sns_i18n::Language::ALL {
        h.state_mut().choose_language(lang);
        h.run();
        h.get_by_label(lang.text("nat_table")).scroll_to_me();
        h.run();
        h.snapshot(format!("j6-nat-{}", lang.code()));
    }
    h.state_mut().choose_language(l);
    h.run();
    h.get_by_label(l.text("nat_purge")).scroll_to_me();
    h.run();
    h.get_by_label(l.text("nat_purge")).click();
    settle(&mut h);
    assert!(
        h.state()
            .trace()
            .iter()
            .any(|f| f.kind == "nat.table.cleared")
    );
}
#[test]
#[cfg(feature = "ui-snapshots")]
fn rendered_private_topology_preserves_icons_and_diagram() {
    let mut h = private_harness(sns_i18n::Language::Fr);
    h.run();
    h.snapshot("j6-private-icons");
    h.get_by_label("Schéma").click();
    h.run();
    h.snapshot("j6-private-diagram");
}
fn advanced_harness(fixture: &str, l: sns_i18n::Language) -> Harness<'static, Workbench> {
    let bytes = match fixture {
        "mail" => include_bytes!("../../../fixtures/valid/mail.json").as_slice(),
        "p2p" => include_bytes!("../../../fixtures/valid/p2p.json").as_slice(),
        "modem" => include_bytes!("../../../fixtures/valid/modem.json").as_slice(),
        _ => include_bytes!("../../../fixtures/valid/wireless.json").as_slice(),
    };
    let mut app = Workbench::from_project(sns_project::parse_project(bytes).unwrap()).unwrap();
    app.choose_language(l);
    Harness::builder()
        .with_max_steps(30)
        .with_size((1650., 1200.))
        .build_ui_state(|ui, state| state.show(ui), app)
}
fn advanced_desktop(h: &mut Harness<'_, Workbench>, host: &str, software: &str) {
    let l = h.state().language();
    h.get_by_label(&format!("{} · F3", l.text("mode_test")))
        .click();
    settle(h);
    h.get_by_label(l.text("desktop")).click();
    h.run();
    h.get_by_label(host).click();
    h.run();
    h.get_by_label(&format!("{} · {}", l.text("open"), l.text(software)))
        .click();
    settle(h);
}
#[test]
fn j6_mail_windows_and_p2p_controls_preserve_all_six_languages() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut h = advanced_harness("mail", l);
        h.run();
        advanced_desktop(&mut h, "Serveur Alpha", "mail_server");
        h.get_by_label(l.text("mail_domain"));
        h.get_by_label(l.text("start_service"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-mail-server-{}", l.code()));
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
        let mut h = advanced_harness("mail", l);
        h.run();
        advanced_desktop(&mut h, "Poste A", "mail_client");
        h.get_by_label(l.text("mail_to"));
        h.get_by_label(l.text("mail_cc"));
        h.get_by_label(l.text("mail_bcc"));
        h.get_by_label(l.text("mail_receive"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-mail-client-{}", l.code()));
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
        let mut h = advanced_harness("p2p", l);
        h.run();
        advanced_desktop(&mut h, "Poste A", "p2p");
        h.get_by_label(l.text("p2p_join")).click();
        settle(&mut h);
        h.get_by_label(l.text("p2p_discover"));
        h.get_by_label(l.text("p2p_search"));
        h.get_by_label(l.text("p2p_reset"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-p2p-{}", l.code()));
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn j6_wireless_inspector_and_both_views_preserve_associations() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut h = advanced_harness("wireless", l);
        h.run();
        let original = h.state().project().clone();
        h.get_by_label(
            &l.format("canvas_device", &[("label", "Portable B")])
                .unwrap(),
        )
        .click();
        h.run();
        h.get_by_label(l.text("wireless_ssid"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-wireless-icons-{}", l.code()));
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        assert_eq!(h.state().project(), &original);
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-wireless-diagram-{}", l.code()));
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn j6_modem_activation_is_explicit_localized_and_disconnects() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut h = advanced_harness("modem", l);
        h.run();
        h.get_by_label(
            &l.format("canvas_device", &[("label", "Modem pont")])
                .unwrap(),
        )
        .click();
        h.run();
        h.get_by_label(l.text("bridge_endpoint"));
        h.get_by_label(l.text("bridge_role"));
        h.get_by_label(&format!("{} · F3", l.text("mode_test")))
            .click();
        settle(&mut h);
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j6-modem-{}", l.code()));
        h.get_by_label(l.text("bridge_activate")).click();
        settle(&mut h);
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(5));
            settle(&mut h);
        }
        h.get_by_label(l.text("bridge.listening"));
        h.get_by_label(l.text("bridge_deactivate")).click();
        settle(&mut h);
        h.get_by_label(l.text("bridge.disconnected"));
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn j6_mail_native_buttons_send_receive_reply_and_delete() {
    let l = sns_i18n::Language::Fr;
    let mut h = advanced_harness("mail", l);
    h.run();
    advanced_desktop(&mut h, "Serveur Alpha", "mail_server");
    h.get_by_label(l.text("start_service")).click();
    settle(&mut h);
    assert!(
        h.state()
            .trace()
            .iter()
            .any(|t| t.kind == "mail.server.started")
    );
    h.get_by_label(l.text("close_view")).click();
    settle(&mut h);
    h.get_by_label("Poste A").click();
    h.run();
    h.get_by_label(&format!("{} · {}", l.text("open"), l.text("mail_client")))
        .click();
    settle(&mut h);
    replace_text(&mut h, l.text("mail_to"), "alice@alpha.test");
    replace_text(&mut h, l.text("mail_subject"), "Exercice J6");
    replace_text(&mut h, l.text("mail_body"), "Bonjour classe");
    h.get_by_label(l.text("mail_send")).click();
    settle(&mut h);
    for _ in 0..400 {
        if h.state()
            .trace()
            .iter()
            .any(|t| t.kind == "mail.operation.completed")
        {
            break;
        }
        h.get_by_label(l.text("step")).click();
        settle(&mut h);
    }
    assert!(
        h.state()
            .trace()
            .iter()
            .any(|t| t.kind == "mail.operation.completed"),
        "{:?}",
        h.state()
            .trace()
            .iter()
            .filter(|t| t.kind.starts_with("mail."))
            .collect::<Vec<_>>()
    );
    h.get_by_label(l.text("mail_receive")).click();
    settle(&mut h);
    for _ in 0..400 {
        if h.state().project().devices[0].applications[0]
            .data
            .as_ref()
            .is_some_and(
                |d| matches!(d,sns_model::ApplicationData::MailClient(d)if !d.inbox.is_empty()),
            )
        {
            break;
        }
        h.get_by_label(l.text("step")).click();
        settle(&mut h);
    }
    h.get_by_label("alice@alpha.test · Exercice J6").click();
    h.run();
    h.get_by_label(l.text("mail_reply")).click();
    settle(&mut h);
    #[cfg(feature = "ui-snapshots")]
    h.snapshot("j6-mail-reply-fr");
    h.get_by_label(l.text("mail_delete_message")).click();
    settle(&mut h);
    assert!(
        matches!(&h.state().project().devices[0].applications[0].data,Some(sns_model::ApplicationData::MailClient(d))if d.inbox.is_empty()&&d.sent.len()==1)
    );
}
#[test]
fn j7_background_import_preserves_six_languages_views_and_previous_project_on_failure() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut h = harness();
        h.state_mut().choose_language(l);
        h.run();
        h.state_mut()
            .begin_filius_import(
                include_bytes!("../../../fixtures/legacy/lan.fls").to_vec(),
                "lan.fls".into(),
            )
            .unwrap();
        for _ in 0..100 {
            h.step();
            if h.state().import_report().is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        h.run();
        assert_eq!(h.state().project().devices.len(), 3);
        assert_eq!(h.state().import_report().unwrap().counts.links, 2);
        h.get_by_label(l.text("import_data_only"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-import-{}", l.code()));
        h.get_by_label(l.text("close_view")).click();
        h.run();
        let imported = h.state().project().clone();
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        assert_eq!(h.state().project(), &imported);
        h.get_by_label(l.text("view_icons")).click();
        h.run();
        assert_eq!(h.state().project(), &imported);
        h.state_mut()
            .begin_filius_import(b"invalid archive".to_vec(), "bad.fls".into())
            .unwrap();
        for _ in 0..15 {
            h.step();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        h.run();
        assert_eq!(h.state().project(), &imported);
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}

#[test]
fn j7_runtime_restore_export_replay_preserves_languages_views_and_failed_load() {
    use sns_model::*;
    let id = |s: &str| Id::new(s).unwrap();
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    let mut e = sns_engine::Engine::from_project(project.clone(), 42, Default::default()).unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("lan-ui-1"),
        project_id: project.project_id.clone(),
        expected_revision: Decimal(0),
        run_id: e.run_id().clone(),
        command: Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    })
    .unwrap();
    e.step().unwrap();
    let bytes = serde_json::to_vec(&e.checkpoint().unwrap()).unwrap();
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut app = Workbench::from_project(project.clone()).unwrap();
        app.choose_language(l);
        let mut h = Harness::builder()
            .with_max_steps(30)
            .with_size((1650., 1200.))
            .build_ui_state(|ui, state| state.show(ui), app);
        h.run();
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        h.state_mut()
            .begin_checkpoint_restore(bytes.clone())
            .unwrap();
        for _ in 0..200 {
            h.step();
            if !h.state().runtime_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        h.run();
        assert!(!h.state().runtime_busy());
        assert_eq!(h.state().run_id().as_str(), "run-42-2");
        h.get_by_label(l.text("runtime_loaded"));
        h.get_by_label(l.text("runtime_save_checkpoint"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-runtime-restored-{}", l.code()));
        h.get_by_label(l.text("close_view")).click();
        h.run();
        h.get_by_label(l.text("view_icons")).click();
        h.run();
        assert_eq!(h.state().project(), &project);
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-runtime-icons-{}", l.code()));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("point.json");
        h.state_mut().export_runtime_to(path.clone(), true).unwrap();
        for _ in 0..200 {
            h.step();
            if !h.state().runtime_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let point = sns_engine::Checkpoint::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
        point.restore().unwrap();
        let previous = h.state().trace().to_vec();
        let run = h.state().run_id().clone();
        h.state_mut()
            .begin_checkpoint_restore(b"invalid".to_vec())
            .unwrap();
        for _ in 0..100 {
            h.step();
            if !h.state().runtime_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(h.state().project(), &project);
        assert_eq!(h.state().run_id(), &run);
        assert_eq!(h.state().trace(), previous);
        let recording = serde_json::to_vec(&e.recording().unwrap()).unwrap();
        h.state_mut().begin_recording_replay(recording).unwrap();
        for _ in 0..200 {
            h.step();
            if !h.state().runtime_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        h.run();
        assert_eq!(h.state().trace(), e.trace());
        h.get_by_label(l.text("close_view")).click();
        h.run();
        // A newly created worker skips command identities already present in the checkpoint.
        h.get_by_label(l.text("drop_hold")).focus();
        h.key_press(egui::Key::Space);
        settle(&mut h);
        assert!(!h.state().runtime_busy());
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}

#[test]
fn j7_link_conditions_apply_and_repair_in_all_languages_and_views_without_editing_project() {
    let plan: sns_model::Scenario = serde_json::from_str(include_str!(
        "../../../fixtures/valid/link-conditions-scenario.json"
    ))
    .unwrap();
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut app = Workbench::from_project(plan.project.clone()).unwrap();
        app.choose_language(l);
        app.choose_level(sns_settings::display::Level::Expert);
        let mut h = Harness::builder()
            .with_max_steps(30)
            .with_size((1650., 1200.))
            .build_ui_state(|ui, state| state.show(ui), app);
        h.run();
        h.get_by_label(&format!("{} · F3", l.text("mode_test")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("runtime_menu")).click();
        h.run();
        h.get_by_label(l.text("link_conditions_menu")).click();
        h.run();
        h.get_by_label(l.text("link_conditions_loss")).click();
        h.run();
        replace_text(&mut h, l.text("link_conditions_loss"), "100");
        h.get_by_label(l.text("link_conditions_delay")).click();
        h.run();
        replace_text(&mut h, l.text("link_conditions_delay"), "10");
        h.get_by_label(l.text("link_conditions_jitter")).click();
        h.run();
        replace_text(&mut h, l.text("link_conditions_jitter"), "2");
        h.get_by_label(l.text("link_conditions_apply")).click();
        settle(&mut h);
        let configured = &h.state().link_conditions()[&sns_model::Id::new("link-ab").unwrap()];
        assert_eq!(configured.loss_ppm, 1000000);
        assert_eq!(configured.delay_us.0, 10000);
        assert_eq!(configured.jitter_us.0, 2000);
        assert_eq!(h.state().link_conditions().len(), 1);
        assert_eq!(h.state().project(), &plan.project);
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-links-blocked-{}", l.code()));
        h.get_by_label(l.text("close_view")).click();
        h.run();
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        h.get_by_label(l.text("runtime_menu")).click();
        h.run();
        h.get_by_label(l.text("link_conditions_menu")).click();
        h.run();
        h.get_by_label(l.text("link_conditions_loss")).click();
        h.run();
        replace_text(&mut h, l.text("link_conditions_loss"), "0");
        h.get_by_label(l.text("link_conditions_apply")).click();
        settle(&mut h);
        assert_eq!(
            h.state().link_conditions()[&sns_model::Id::new("link-ab").unwrap()].loss_ppm,
            0
        );
        assert_eq!(h.state().project(), &plan.project);
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-links-repaired-{}", l.code()));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("point.json");
        h.state_mut().export_runtime_to(path.clone(), true).unwrap();
        for _ in 0..200 {
            h.step();
            if !h.state().runtime_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let restored = sns_engine::Checkpoint::from_bytes(&std::fs::read(path).unwrap())
            .unwrap()
            .restore()
            .unwrap();
        assert_eq!(
            restored.network().configured_links(),
            *h.state().link_conditions()
        );
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}

#[test]
fn j7_duplicate_align_undo_and_ipv4_warnings_preserve_all_languages_and_views() {
    use sns_i18n::Language;
    let p = sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in Language::ALL {
        let mut app = Workbench::from_project(p.clone()).unwrap();
        app.choose_language(l);
        let mut h = Harness::builder()
            .with_max_steps(30)
            .with_size((1800., 1280.))
            .build_ui_state(|ui, state| state.show(ui), app);
        h.run();
        let rects: Vec<_> = p
            .devices
            .iter()
            .map(|d| {
                h.get_by_label(&l.format("canvas_device", &[("label", &d.label)]).unwrap())
                    .rect()
            })
            .collect();
        let area = rects.into_iter().reduce(|a, b| a.union(b)).unwrap();
        let start = area.min - egui::vec2(12., 12.);
        let end = area.max + egui::vec2(12., 12.);
        h.hover_at(start);
        h.run();
        h.drag_at(start);
        h.run();
        h.hover_at(start + egui::vec2(5., 5.));
        h.run();
        h.hover_at(end);
        h.run();
        h.drop_at(end);
        h.run();
        assert_eq!(h.state().selected().len(), 4);
        h.get_by_label(&format!("{} · Ctrl/⌘D", l.text("topology_duplicate")))
            .click();
        h.run();
        assert_eq!(h.state().project().devices.len(), 8);
        assert_eq!(h.state().project().links.len(), 6);
        assert_eq!(h.state().selected().len(), 7);
        h.state().project().validate().unwrap();
        h.get_by_label(l.text("topology_ipv4_title"));
        let warning = l
            .format(
                "topology_ipv4_group",
                &[("address", "192.168.1.2"), ("count", "2")],
            )
            .unwrap();
        h.get_by_label(&warning);
        let copied = h.state().project().clone();
        let selection = h.state().selected().clone();
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-topology-copy-{}", l.code()));
        h.get_by_label(l.text("topology_align")).click();
        h.run();
        h.get_by_label(l.text("distribute_x")).click();
        h.run();
        let aligned = h.state().project().clone();
        assert_ne!(aligned.layout, copied.layout);
        assert_eq!(aligned.devices, copied.devices);
        assert_eq!(aligned.links, copied.links);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
        h.run();
        assert_eq!(h.state().project(), &copied);
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::Z,
        );
        h.run();
        assert_eq!(h.state().project(), &aligned);
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        assert_eq!(h.state().selected(), &selection);
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-topology-aligned-{}", l.code()));
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::D);
        h.run();
        assert_eq!(h.state().project().devices.len(), 12);
        assert_eq!(h.state().project().links.len(), 9);
        h.get_by_label(&format!("{} · F3", l.text("mode_test")))
            .click();
        settle(&mut h);
        let running = h.state().project().clone();
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::D);
        h.run();
        assert_eq!(h.state().project(), &running);
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}

#[test]
fn j7_cloud_switch_changes_live_with_ports_sat_and_run_preserved_in_six_languages() {
    use sns_model::*;
    let id = |s: &str| Id::new(s).unwrap();
    let p = sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    let mut e = sns_engine::Engine::from_project(p.clone(), 42, Default::default()).unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("warm-sat"),
        project_id: p.project_id.clone(),
        expected_revision: Decimal(0),
        run_id: e.run_id().clone(),
        command: Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    })
    .unwrap();
    e.run(10000, Some(3_000_000)).unwrap();
    assert!(e.network().sat.len() >= 2);
    let point = e.checkpoint().unwrap();
    let bytes = serde_json::to_vec(&point).unwrap();
    let interfaces = p.device(&id("switch")).unwrap().interfaces.clone();
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut app = Workbench::from_project(p.clone()).unwrap();
        app.choose_language(l);
        let mut h = Harness::builder()
            .with_max_steps(30)
            .with_size((1650., 1200.))
            .build_ui_state(|ui, state| state.show(ui), app);
        h.run();
        h.state_mut()
            .begin_checkpoint_restore(bytes.clone())
            .unwrap();
        for _ in 0..200 {
            h.step();
            if !h.state().runtime_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        h.run();
        h.get_by_label(l.text("close_view")).click();
        h.run();
        h.get_by_label(
            &l.format("canvas_device", &[("label", "Commutateur")])
                .unwrap(),
        )
        .click();
        h.run();
        let run = h.state().run_id().clone();
        let now = h.state().now();
        h.get_by_label(l.text("switch_cloud")).click();
        settle(&mut h);
        assert_eq!(
            h.state().project().lan[&id("switch")].symbol,
            Some(DeviceSymbol::Cloud)
        );
        assert_eq!(h.state().project().schema_version, 8);
        assert_eq!(h.state().run_id(), &run);
        assert_eq!(h.state().now(), now);
        assert_eq!(
            h.state()
                .project()
                .device(&id("switch"))
                .unwrap()
                .interfaces,
            interfaces
        );
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-cloud-icons-{}", l.code()));
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-cloud-diagram-{}", l.code()));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cloud-point.json");
        h.state_mut().export_runtime_to(path.clone(), true).unwrap();
        for _ in 0..200 {
            h.step();
            if !h.state().runtime_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let saved = sns_engine::Checkpoint::from_bytes(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(
            saved.runtime_state["network"],
            point.runtime_state["network"]
        );
        assert_eq!(saved.runtime_state["queue"], point.runtime_state["queue"]);
        let restored = saved.restore().unwrap();
        assert_eq!(
            restored.project().lan[&id("switch")].symbol,
            Some(DeviceSymbol::Cloud)
        );
        h.get_by_label(l.text("switch_cloud")).click();
        settle(&mut h);
        assert_eq!(h.state().project().lan[&id("switch")].symbol, None);
        assert_eq!(h.state().run_id(), &run);
        assert_eq!(
            h.state()
                .project()
                .device(&id("switch"))
                .unwrap()
                .interfaces,
            interfaces
        );
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}

#[test]
fn j7_trace_queries_exports_and_local_global_animation_preserve_six_languages_and_views() {
    use sns_model::*;
    let id = |s: &str| Id::new(s).unwrap();
    let p = sns_project::parse_project(include_bytes!("../../../fixtures/valid/services.json"))
        .unwrap();
    let mut e = sns_engine::Engine::from_project(p.clone(), 42, Default::default()).unwrap();
    for (host, app, operation) in [
        ("host-b", "dns", 0),
        ("host-b", "web", 0),
        ("host-a", "host-a-browser", 1),
    ] {
        if operation == 1 {
            e.run(100000, Some(1_000_000)).unwrap();
        }
        let command = if operation == 0 {
            AppCommand::StartServer {
                application: id(app),
                port: 80,
            }
        } else {
            AppCommand::Navigate {
                application: id(app),
                url: "http://www.ecole.test/".into(),
                fields: None,
            }
        };
        e.command(CommandEnvelope {
            api_version: 1,
            command_id: id(&format!("observation-{}", e.trace().len())),
            project_id: p.project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::App {
                device: id(host),
                command,
            },
        })
        .unwrap();
    }
    e.run(100000, Some(4_000_000)).unwrap();
    let anchor = e
        .trace()
        .iter()
        .rev()
        .find(|e| {
            e.device_id == id("host-a")
                && e.kind == "ethernet.received"
                && sns_trace::has_protocol(e, "TCP")
                && sns_trace::layer_parts(e)[3].is_some()
        })
        .unwrap()
        .clone();
    let point = serde_json::to_vec(&e.checkpoint().unwrap()).unwrap();
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let mut app = Workbench::from_project(p.clone()).unwrap();
        app.choose_language(l);
        let mut h = Harness::builder()
            .with_max_steps(40)
            .with_size((1800., 1150.))
            .build_ui_state(|ui, state| state.show(ui), app);
        h.run();
        h.state_mut()
            .begin_checkpoint_restore(point.clone())
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        loop {
            h.step();
            if !h.state().runtime_busy() {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Checkpoint restore timeout"
            );
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        h.run();
        h.get_by_label(l.text("close_view")).click();
        h.run();
        let original = h.state().trace().to_vec();
        let original_project = h.state().project().clone();
        let now = h.state().now();
        let run = h.state().run_id().clone();
        h.get_by_label(l.text("exchanges")).click();
        h.run();
        for (key, value) in [
            ("trace_protocol", "TCP".to_owned()),
            ("trace_address", "192.168.1.3".to_owned()),
            (
                "trace_correlation",
                anchor.correlation_id.as_str().to_owned(),
            ),
            ("trace_from_us", anchor.time_us.0.to_string()),
            ("trace_through_us", anchor.time_us.0.to_string()),
        ] {
            h.get_by_label(l.text(key)).focus();
            h.run();
            h.get_by_label(l.text(key)).type_text(&value);
            h.run();
        }
        h.get_by_role_and_label(egui::Role::ComboBox, l.text("device"))
            .click();
        h.run();
        h.get_by_label("Poste A · host-a").click();
        h.run();
        let ids = h.state().filtered_event_ids();
        assert!(!ids.is_empty());
        assert!(ids.contains(&anchor.event_id));
        assert!(ids.len() < original.len());
        let expected_jsonl = h
            .state()
            .filtered_trace_bytes(sns_trace::ExportFormat::Jsonl)
            .unwrap();
        let expected_csv = h
            .state()
            .filtered_trace_bytes(sns_trace::ExportFormat::Csv)
            .unwrap();
        let decoded: Vec<sns_engine::TraceEvent> = std::str::from_utf8(&expected_jsonl)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(
            decoded
                .iter()
                .map(|e| e.event_id.clone())
                .collect::<Vec<_>>(),
            ids
        );
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("selection.csv");
        h.state_mut()
            .begin_trace_export(file.clone(), sns_trace::ExportFormat::Csv)
            .unwrap();
        h.get_by_label(l.text("reset_trace_filters")).click();
        for _ in 0..200 {
            h.step();
            if !h.state().trace_export_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        h.run();
        assert_eq!(std::fs::read(&file).unwrap(), expected_csv);
        h.get_by_label(l.text("trace_from_us")).focus();
        h.run();
        h.get_by_label(l.text("trace_from_us"))
            .type_text("not-a-time");
        h.run();
        assert!(h.state().filtered_event_ids().is_empty());
        assert!(
            h.state_mut()
                .begin_trace_export(file.clone(), sns_trace::ExportFormat::Jsonl)
                .is_err()
        );
        assert_eq!(std::fs::read(&file).unwrap(), expected_csv);
        h.get_by_label(l.text("reset_trace_filters")).click();
        h.run();
        // Select the real message through the same stable-identity operation used by integrations.
        assert!(h.state_mut().select_trace_event(&anchor.event_id));
        h.run();
        h.get_by_label(l.text("message_journey")).click();
        h.run();
        let first = h.state().journey_event().unwrap().event_id.clone();
        h.get_by_label(l.text("journey_next")).click();
        h.run();
        assert_ne!(h.state().journey_event().unwrap().event_id, first);
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-observation-local-{}", l.code()));
        h.get_by_label(l.text("journey_global")).click();
        h.run();
        let first = h.state().journey_event().unwrap().event_id.clone();
        h.get_by_label(l.text("journey_play")).click();
        h.step();
        h.run_steps(55);
        h.get_by_label(l.text("journey_pause")).click();
        h.step();
        h.run();
        assert_ne!(h.state().journey_event().unwrap().event_id, first);
        h.get_by_label(l.text("journey_show_network")).click();
        h.run();
        h.get_by_label(l.text("view_diagram")).click();
        h.run();
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-observation-global-{}", l.code()));
        assert_eq!(h.state().trace(), original);
        assert_eq!(h.state().project(), &original_project);
        assert_eq!(h.state().now(), now);
        assert_eq!(h.state().run_id(), &run);
        // Playback does not change the selected representation.
        assert!(h.state().journey_event().is_some());
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}

#[test]
fn enlarged_keyboard_only_project_roundtrip_and_ping_match_in_all_views() {
    use egui_kittest::kittest::NodeT as _;
    use sns_i18n::Language;
    use sns_settings::display::{DisplayPreferences, Representation};

    fn activate(h: &mut Harness<'_, Workbench>, label: &str) {
        for _ in 0..160 {
            if h.get_by_label(label).accesskit_node().is_focused() {
                h.key_press(egui::Key::Enter);
                h.run();
                return;
            }
            h.key_press(egui::Key::Tab);
            h.run();
        }
        panic!("not reachable with Tab: {label}");
    }

    let baseline =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    let source = sns_model::Id::new("host-a").unwrap();
    let mut expected_trace = None;
    for language in Language::ALL {
        for representation in [Representation::Icons, Representation::Diagram] {
            let directory = tempfile::tempdir().unwrap();
            let project = directory.path().join("- réseau de classe.sinesi");
            sns_project::save_project(&project, &baseline).unwrap();
            let mut app = Workbench::with_storage(None, None, Some(project.clone())).unwrap();
            app.choose_language(language);
            app.set_display_preferences(DisplayPreferences {
                scale_percent: 150,
                representation,
                ..Default::default()
            })
            .unwrap();
            let mut h = Harness::builder()
                .with_max_steps(60)
                .with_size((1600., 1050.))
                .build_ui_state(|ui, state| state.show(ui), app);
            h.run();
            activate(&mut h, language.text("add_notebook"));
            assert_eq!(
                h.state().project().devices.len(),
                baseline.devices.len() + 1
            );
            // A native backend can deliver the whole chord before the next frame.
            for pressed in [true, false] {
                h.event(egui::Event::Key {
                    key: egui::Key::S,
                    physical_key: Some(egui::Key::S),
                    pressed,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                });
            }
            h.run();
            let saved = sns_project::load_project(&project).unwrap();
            assert_eq!(&saved, h.state().project());
            assert_eq!(saved.links, baseline.links);

            h.key_press(egui::Key::F3);
            settle(&mut h);
            assert_eq!(h.state().mode, Mode::Simulation);
            let label = language
                .format(
                    "canvas_device",
                    &[("label", &baseline.device(&source).unwrap().label)],
                )
                .unwrap();
            activate(&mut h, &label);
            assert!(h.state().selected().contains(&source));
            activate(&mut h, language.text("send_ping"));
            settle(&mut h);
            for _ in 0..20 {
                activate(&mut h, language.text("step"));
                settle(&mut h);
                if h.state()
                    .trace()
                    .iter()
                    .any(|event| event.kind == "icmp.echo.reply")
                {
                    break;
                }
            }
            let trace = h.state().trace();
            assert!(trace.iter().any(|event| event.kind == "icmp.echo.reply"));
            if let Some(expected) = &expected_trace {
                assert_eq!(trace, expected);
            } else {
                expected_trace = Some(trace.to_vec());
            }
            assert_eq!(sns_project::load_project(&project).unwrap(), saved);
        }
    }
}
