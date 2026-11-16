use egui_kittest::{Harness, kittest::Queryable as _};
use sns_model::*;
use sns_ui_egui::Workbench;
fn harness(app: Workbench) -> Harness<'static, Workbench> {
    Harness::builder()
        .with_max_steps(40)
        .with_size((1800., 1200.))
        .build_ui_state(|ui, state| state.show(ui), app)
}
fn drag(h: &mut Harness<'_, Workbench>, start: egui::Pos2, delta: egui::Vec2) {
    h.hover_at(start);
    h.run();
    h.drag_at(start);
    h.run();
    h.hover_at(start + delta);
    h.run();
    h.drop_at(start + delta);
    h.run();
}
fn replace_note(h: &mut Harness<'_, Workbench>, language: sns_i18n::Language, text: &str) {
    let key = language.text("annotation_text");
    h.get_all_by_label(key).last().unwrap().focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_all_by_label(key).last().unwrap().type_text(text);
    h.run();
}
#[test]
fn notes_and_zones_edit_move_resize_reopen_and_preserve_network_in_six_languages_and_two_views() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let project =
            sns_project::parse_project(include_bytes!("../../../fixtures/valid/annotations.json"))
                .unwrap();
        let mut app = Workbench::from_project(project.clone()).unwrap();
        app.choose_language(l);
        let mut h = harness(app);
        h.run();
        h.get_by_label(&format!("{} · F4", l.text("mode_annotate")))
            .click();
        h.run();
        let id = "note-exercise";
        let name = l.format("annotation_canvas", &[("id", id)]).unwrap();
        h.get_by_label(&name).click();
        h.run();
        replace_note(
            &mut h,
            l,
            "Note de cours — Grüße, ação, perché\nObserver DHCP puis DNS et HTTP.",
        );
        for (key, value) in [
            ("annotation_width", 340.),
            ("annotation_height", 160.),
            ("annotation_font_size", 19.),
        ] {
            h.get_by_role_and_label(egui::Role::SpinButton, l.text(key))
                .set_value_accesskit(value);
            h.run();
        }
        h.get_by_label(l.text("annotation_bold")).click();
        h.run();
        h.get_by_label(l.text("annotation_apply")).click();
        h.run();
        let applied = h.state().project().annotations[1].clone();
        assert!(applied.style.as_ref().unwrap().bold);
        assert_eq!(applied.style.as_ref().unwrap().width, 340.);
        assert_eq!(applied.style.as_ref().unwrap().height, 160.);
        assert_eq!(applied.style.as_ref().unwrap().font_size, 19.);
        for view in ["view_icons", "view_diagram"] {
            h.get_by_label(l.text(view)).click();
            h.run();
            h.get_by_label(l.text("fit")).click();
            h.run();
            let before = h.state().project().annotations[1].clone();
            let zoom = h.state().recovery_view().zoom;
            let area = h.get_by_label(&name).rect();
            // Escape cancels the preview without publishing an edit.
            let origin = area.min + egui::vec2(24., 24.);
            h.hover_at(origin);
            h.run();
            h.drag_at(origin);
            h.run();
            h.hover_at(origin + egui::vec2(30., 20.));
            h.run();
            h.key_press(egui::Key::Escape);
            h.run();
            h.drop_at(origin + egui::vec2(30., 20.));
            h.run();
            assert_eq!(h.state().project().annotations[1], before);
            drag(
                &mut h,
                area.min + egui::vec2(24., 24.),
                egui::vec2(18., 14.),
            );
            let moved = h.state().project().annotations[1].clone();
            assert!((moved.x - before.x - 18. / zoom).abs() < 0.02);
            assert!((moved.y - before.y - 14. / zoom).abs() < 0.02);
            h.get_by_label(l.text("fit")).click();
            h.run();
            let resize_zoom = h.state().recovery_view().zoom;
            let handle = l.format("annotation_resize", &[("id", id)]).unwrap();
            let start = h.get_by_label(&handle).rect().center();
            drag(&mut h, start, egui::vec2(26., 18.));
            let resized = h.state().project().annotations[1].clone();
            let size = resized.style.as_ref().unwrap();
            assert!(
                (size.width - moved.style.as_ref().unwrap().width - 26. / resize_zoom).abs() < 0.02,
                "{view}: {size:?}; before={:?}, zoom={zoom}, pos=({},{})",
                moved.style,
                resized.x,
                resized.y
            );
            assert!(
                (size.height - moved.style.as_ref().unwrap().height - 18. / resize_zoom).abs()
                    < 0.02
            );
            h.get_by_label(l.text("undo")).click();
            h.run();
            assert_eq!(h.state().project().annotations[1], moved);
            h.get_by_label(l.text("redo")).click();
            h.run();
            assert_eq!(h.state().project().annotations[1], resized);
            h.get_by_label(l.text("fit")).click();
            h.run();
            #[cfg(feature = "ui-snapshots")]
            h.snapshot(format!(
                "j7-annotations-{}-{}",
                view.trim_start_matches("view_"),
                l.code()
            ));
        }
        let saved = h.state().project().clone();
        assert_eq!(saved.devices, project.devices);
        assert_eq!(saved.links, project.links);
        assert_eq!(saved.lan, project.lan);
        assert_eq!(saved.layout, project.layout);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Cours réseau.json");
        h.state_mut().save_project_to(path.clone()).unwrap();
        let reopened = sns_project::load_project(&path).unwrap();
        assert_eq!(reopened, saved);
        let mut fresh = harness(Workbench::from_project(reopened).unwrap());
        fresh.run();
        assert_eq!(fresh.state().project().annotations, saved.annotations);
        // Creation allocates a fresh identity; deleting is reversible.
        h.get_by_label(l.text("add_annotation_rectangle"))
            .click_accesskit();
        h.run();
        assert_eq!(h.state().project().annotations.len(), 3);
        assert_eq!(
            h.state().project().annotations[2]
                .style
                .as_ref()
                .unwrap()
                .kind,
            AnnotationKind::Rectangle
        );
        h.get_by_label(l.text("annotation_delete")).click();
        h.run();
        assert_eq!(h.state().project().annotations.len(), 2);
        h.get_by_label(l.text("undo")).click();
        h.run();
        assert_eq!(h.state().project().annotations.len(), 3);
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}

#[test]
fn an_unapplied_annotation_draft_is_recovered_without_applying_it() {
    use sns_project::session::*;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("sessions");
    let source = dir.path().join("original.json");
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/annotations.json"))
            .unwrap();
    sns_project::save_project(&source, &project).unwrap();
    let bytes = std::fs::read(&source).unwrap();
    let mut h =
        harness(Workbench::with_storage(None, Some(root.clone()), Some(source.clone())).unwrap());
    fn storage(h: &mut Harness<'_, Workbench>) {
        for _ in 0..300 {
            h.step();
            if !h.state().session_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        h.run();
    }
    storage(&mut h);
    h.get_by_label("Annoter · F4").click();
    h.run();
    h.get_by_label("Annotation note-exercise").click();
    h.run();
    replace_note(&mut h, sns_i18n::Language::Fr, "Brouillon non appliqué");
    h.state_mut().request_autosave();
    storage(&mut h);
    storage(&mut h);
    assert!(h.state().session_saved());
    drop(h);
    let mut recovered = harness(Workbench::with_storage(None, Some(root.clone()), None).unwrap());
    storage(&mut recovered);
    recovered.get_by_label("Reprendre la session").click();
    storage(&mut recovered);
    storage(&mut recovered);
    assert_eq!(recovered.state().project(), &project);
    assert_eq!(std::fs::read(source).unwrap(), bytes);
    recovered.get_by_label("Annoter · F4").click();
    recovered.run();
    let text = recovered
        .get_all_by_label("Texte de l’annotation")
        .last()
        .unwrap()
        .value()
        .unwrap();
    assert_eq!(text, "Brouillon non appliqué");
    recovered.get_by_label("Appliquer l’annotation").click();
    recovered.run();
    assert_eq!(
        recovered.state().project().annotations[1].text,
        "Brouillon non appliqué"
    );
    drop(recovered);
    let records = scan(&root).unwrap();
    assert!(!records.recoveries.is_empty());
}
