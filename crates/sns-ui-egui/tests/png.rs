use egui_kittest::{Harness, kittest::Queryable as _};
use sns_render::{PngOptions, Scene, render_png};
use sns_ui_egui::Workbench;
fn harness(app: Workbench) -> Harness<'static, Workbench> {
    Harness::builder()
        .with_max_steps(40)
        .with_size((1800., 1200.))
        .build_ui_state(|ui, state| state.show(ui), app)
}
fn wait(h: &mut Harness<'_, Workbench>) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(8);
    loop {
        h.step();
        if !h.state().document_export_busy() {
            break;
        }
        assert!(std::time::Instant::now() < until, "PNG export timeout");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    h.run();
}
#[test]
fn png_controls_and_background_exports_preserve_six_languages_two_views_and_document() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in sns_i18n::Language::ALL {
        let original =
            sns_project::parse_project(include_bytes!("../../../fixtures/valid/annotations.json"))
                .unwrap();
        let mut app = Workbench::from_project(original.clone()).unwrap();
        app.choose_language(l);
        app.choose_level(sns_settings::display::Level::Expert);
        let mut h = harness(app);
        h.run();
        let dir = tempfile::tempdir().unwrap();
        for diagram in [false, true] {
            h.get_by_label(l.text(if diagram {
                "view_diagram"
            } else {
                "view_icons"
            }))
            .click();
            h.run();
            h.get_by_role_and_label(egui::Role::Button, l.text("export_network"))
                .click();
            h.run();
            h.get_by_label(l.text("export_png"));
            h.get_by_role_and_label(egui::Role::SpinButton, l.text("png_longest_side"))
                .set_value_accesskit(512.);
            h.run();
            assert!(h.state().png_export_options().transparent);
            let project = h.state().project().clone();
            let time = h.state().now();
            let run = h.state().run_id().clone();
            let trace = h.state().trace().to_vec();
            let output = dir
                .path()
                .join(if diagram { "schema.png" } else { "icones.png" });
            let expected = render_png(
                &Scene::build(&project, diagram).unwrap(),
                PngOptions {
                    longest_side: 512,
                    transparent: true,
                },
            )
            .unwrap();
            h.state_mut().begin_png_export(output.clone()).unwrap();
            assert!(
                h.state_mut()
                    .begin_png_export(dir.path().join("second.png"))
                    .is_err()
            );
            assert!(h.state_mut().save_project_to(output.clone()).is_err());
            wait(&mut h);
            assert!(h.state().document_export_error().is_none());
            assert_eq!(std::fs::read(&output).unwrap(), expected.bytes);
            assert_eq!(h.state().png_export_receipt().unwrap().diagram, diagram);
            assert_eq!(h.state().png_export_receipt().unwrap().clipped_texts, 0);
            assert_eq!(h.state().project(), &project);
            assert_eq!(h.state().now(), time);
            assert_eq!(h.state().run_id(), &run);
            assert_eq!(h.state().trace(), trace);
            #[cfg(feature = "ui-snapshots")]
            h.snapshot(format!(
                "j7-display-report-png-{}-{}",
                if diagram { "diagram" } else { "icons" },
                l.code()
            ));
        }
        assert_eq!(h.state().project(), &original);
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn png_export_freezes_applied_document_and_protects_project_preferences_and_session_storage() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("project.json");
    let prefs = dir.path().join("prefs.json");
    let root = dir.path().join("sessions");
    let original =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/annotations.json"))
            .unwrap();
    sns_project::save_project(&source, &original).unwrap();
    sns_settings::save(&prefs, &Default::default()).unwrap();
    let bytes = std::fs::read(&source).unwrap();
    let mut h = harness(
        Workbench::with_storage(
            Some(prefs.clone()),
            Some(root.clone()),
            Some(source.clone()),
        )
        .unwrap(),
    );
    for _ in 0..100 {
        h.step();
        if !h.state().session_busy() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    h.run();
    for path in [&source, &prefs, &root.join("network.png")] {
        assert!(h.state_mut().begin_png_export(path.clone()).is_err());
    }
    let output = dir.path().join("frozen.png");
    let expected = render_png(
        &Scene::build(&original, false).unwrap(),
        PngOptions::default(),
    )
    .unwrap();
    h.state_mut().begin_png_export(output.clone()).unwrap();
    // Edits remain usable during export; the output belongs to the snapshot taken above.
    h.get_by_label("Annoter · F4").click();
    h.run();
    h.get_by_label("Annotation note-exercise").click();
    h.run();
    h.get_all_by_label("Texte de l’annotation")
        .last()
        .unwrap()
        .set_text_value_accesskit("Texte changé pendant l’export");
    h.run();
    #[cfg(feature = "ui-snapshots")]
    h.render()
        .unwrap()
        .save("/tmp/sns-display-annotation-diagnostic.png")
        .unwrap();
    h.get_by_label("Appliquer l’annotation").click();
    h.run();
    wait(&mut h);
    assert!(h.state().document_export_error().is_none());
    assert_eq!(std::fs::read(output).unwrap(), expected.bytes);
    assert_ne!(h.state().project().annotations, original.annotations);
    assert_eq!(std::fs::read(source).unwrap(), bytes);
}
#[test]
fn simulation_png_freezes_effective_dhcp_labels_without_advancing_or_stopping_the_network() {
    use sns_model::*;
    let mut project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/services.json"))
            .unwrap();
    let host = Id::new("host-a").unwrap();
    project.lan.entry(host.clone()).or_default().display = DisplayName::Ipv4;
    let mut h = harness(Workbench::from_project(project).unwrap());
    h.run();
    h.get_by_label("Tester · F3").click();
    h.step();
    h.state_mut().wait_for_worker().unwrap();
    h.run();
    let mut address = None;
    for _ in 0..300 {
        if let Some(fact) = h
            .state()
            .trace()
            .iter()
            .find(|f| f.kind == "dhcp.lease.acquired" && f.device_id == host)
        {
            address = Some(
                fact.payload["assignment"]["address"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            );
            break;
        }
        h.get_by_label("Un événement").click();
        h.step();
        h.state_mut().wait_for_worker().unwrap();
        h.run();
    }
    let address = address.expect("DHCP must bind through the production stack");
    assert_eq!(
        h.state().project().device(&host).unwrap().interfaces[0].ipv4,
        Some(Ipv4Config::Dhcp)
    );
    let labels = std::collections::BTreeMap::from([(host, address)]);
    let scene = Scene::build_with_labels(h.state().project(), false, &labels).unwrap();
    let expected = render_png(&scene, PngOptions::default()).unwrap();
    let now = h.state().now();
    let run = h.state().run_id().clone();
    let trace = h.state().trace().to_vec();
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("lease.png");
    h.state_mut().begin_png_export(output.clone()).unwrap();
    wait(&mut h);
    assert!(h.state().document_export_error().is_none());
    assert_eq!(std::fs::read(output).unwrap(), expected.bytes);
    assert_eq!(h.state().now(), now);
    assert_eq!(h.state().run_id(), &run);
    assert_eq!(h.state().trace(), trace);
    assert_eq!(h.state().mode, sns_ui_egui::Mode::Simulation);
}
