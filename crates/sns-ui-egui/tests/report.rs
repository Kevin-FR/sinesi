use egui_kittest::{Harness, kittest::Queryable as _};
use sns_i18n::Language;
use sns_report::{Report, ReportOptions, Snapshot, render_pdf};
use sns_ui_egui::Workbench;
fn harness() -> Harness<'static, Workbench> {
    let project = sns_project::parse_project(include_bytes!(
        "../../../fixtures/valid/report-network.json"
    ))
    .unwrap();
    Harness::builder()
        .with_max_steps(40)
        .with_size((1800., 1200.))
        .build_ui_state(
            |ui, state| state.show(ui),
            Workbench::from_project(project).unwrap(),
        )
}
fn wait(h: &mut Harness<'_, Workbench>) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(12);
    while h.state().document_export_busy() {
        h.step();
        assert!(std::time::Instant::now() < until, "report export timeout");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    h.run();
}
#[test]
fn pdf_buttons_and_background_export_preserve_six_languages_views_and_frozen_document() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness();
    h.state_mut()
        .choose_level(sns_settings::display::Level::Expert);
    h.run();
    let original = h.state().project().clone();
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for language in Language::ALL {
        for diagram in [false, true] {
            h.state_mut().choose_language(language);
            h.run();
            h.get_by_label(language.text(if diagram {
                "view_diagram"
            } else {
                "view_icons"
            }))
            .click();
            h.run();
            h.get_by_role_and_label(egui::Role::Button, language.text("export_network"))
                .click();
            h.run();
            h.get_by_label(language.text("export_pdf"));
            let path = dir.path().join("network-report.pdf");
            let snapshot = Snapshot::configured(h.state().project().clone());
            let expected =
                render_pdf(&Report::build(&snapshot, ReportOptions { language, diagram }).unwrap())
                    .unwrap();
            let run = h.state().run_id().clone();
            let now = h.state().now();
            let trace = h.state().trace().to_vec();
            h.state_mut().begin_pdf_export(path.clone()).unwrap();
            assert!(
                h.state_mut()
                    .begin_png_export(dir.path().join("concurrent.png"))
                    .is_err()
            );
            assert!(h.state_mut().save_project_to(path.clone()).is_err());
            wait(&mut h);
            assert!(
                h.state().document_export_error().is_none(),
                "{:?}",
                h.state().document_export_error()
            );
            assert_eq!(std::fs::read(&path).unwrap(), expected.bytes);
            let receipt = h.state().pdf_export_receipt().unwrap();
            assert_eq!(receipt.diagram, diagram);
            assert_eq!(receipt.packet_rows, 0);
            assert_eq!(receipt.pages, expected.pages);
            assert_eq!(h.state().project(), &original);
            assert_eq!(h.state().run_id(), &run);
            assert_eq!(h.state().now(), now);
            assert_eq!(h.state().trace(), trace);
            #[cfg(feature = "ui-snapshots")]
            h.snapshot(format!(
                "j7-display-report-pdf-{}-{}",
                if diagram { "diagram" } else { "icons" },
                language.code()
            ));
        }
    }
    #[cfg(feature = "ui-snapshots")]
    {
        snapshots.extend_harness(&mut h);
        snapshots.unwrap();
    }
}
#[test]
fn pdf_export_keeps_previous_file_on_failure_and_protects_project() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("school.json");
    let mut h = harness();
    h.run();
    h.state_mut().save_project_to(source.clone()).unwrap();
    let bytes = std::fs::read(&source).unwrap();
    assert!(h.state_mut().begin_pdf_export(source.clone()).is_err());
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
    // A glyph outside the supported font set fails before replacing an existing file.
    let mut project = h.state().project().clone();
    project.annotations[0].text = "\u{10ffff}".into();
    let mut app = Workbench::from_project(project).unwrap();
    let output = dir.path().join("previous.pdf");
    std::fs::write(&output, b"sentinel").unwrap();
    app.begin_pdf_export(output.clone()).unwrap();
    let mut h = Harness::builder().build_ui_state(|ui, state| state.show(ui), app);
    wait(&mut h);
    assert!(h.state().document_export_error().is_some());
    assert_eq!(std::fs::read(output).unwrap(), b"sentinel");
}

#[test]
fn simulation_pdf_captures_recorded_packets_without_advancing_the_network() {
    use sns_model::{Id, Ipv4Config};
    let mut h = harness();
    h.run();
    h.get_by_label("Tester · F3").click();
    h.step();
    h.state_mut().wait_for_worker().unwrap();
    h.run();
    let host = Id::new("a").unwrap();
    for _ in 0..300 {
        if h.state()
            .trace()
            .iter()
            .any(|e| e.kind == "dhcp.lease.acquired" && e.device_id == host)
        {
            break;
        }
        h.get_by_label("Un événement").click();
        h.step();
        h.state_mut().wait_for_worker().unwrap();
        h.run();
    }
    assert!(
        h.state()
            .trace()
            .iter()
            .any(|e| e.kind == "dhcp.lease.acquired" && e.device_id == host)
    );
    assert_eq!(
        h.state().project().device(&host).unwrap().interfaces[0].ipv4,
        Some(Ipv4Config::Dhcp)
    );
    let trace = h.state().trace().to_vec();
    let now = h.state().now();
    let run = h.state().run_id().clone();
    let project = h.state().project().clone();
    let expected = trace
        .iter()
        .filter(|e| matches!(e.kind.as_str(), "ethernet.sent" | "ethernet.received"))
        .count();
    assert!(expected > 0);
    let dir = tempfile::tempdir().unwrap();
    h.state_mut()
        .begin_pdf_export(dir.path().join("simulation.pdf"))
        .unwrap();
    wait(&mut h);
    assert!(h.state().document_export_error().is_none());
    assert_eq!(
        h.state().pdf_export_receipt().unwrap().packet_rows,
        expected
    );
    assert_eq!(h.state().now(), now);
    assert_eq!(h.state().run_id(), &run);
    assert_eq!(h.state().trace(), trace);
    assert_eq!(h.state().project(), &project);
    assert_eq!(h.state().mode, sns_ui_egui::Mode::Simulation);
}
