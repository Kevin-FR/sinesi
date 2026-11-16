use egui_kittest::{Harness, kittest::Queryable as _};
use sns_ui_egui::prototype::{SinesiUi, Tab};

fn harness() -> Harness<'static, SinesiUi> {
    Harness::builder()
        .with_size((1200.0, 800.0))
        .build_ui_state(|ui, state| state.show(ui), SinesiUi::new().unwrap())
}
fn settle(harness: &mut Harness<'_, SinesiUi>) {
    for _ in 0..4 {
        harness.step();
        harness.state_mut().wait_for_worker().unwrap();
    }
    harness.run();
}
#[test]
fn controls_drive_the_real_engine_and_restart_clears_state() {
    let mut harness = harness();
    harness.run();
    harness.get_by_label("Pas à pas").click();
    settle(&mut harness);
    assert_eq!(harness.state().trace_len(), 1);
    harness.get_by_label("Exécuter").click();
    settle(&mut harness);
    assert!(harness.state().trace_len() >= 6);
    assert!(harness.state().now() > 0);
    harness.get_by_label("Réinitialiser").click();
    settle(&mut harness);
    assert_eq!(harness.state().trace_len(), 0);
    assert_eq!(harness.state().now(), 0);
}
#[test]
fn canvas_has_a_keyboard_reachable_equipment_selection() {
    let mut harness = harness();
    harness.run();
    harness.get_by_label("Poste élève").focus();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().selected().unwrap().as_str(), "client");
}
#[test]
fn native_browser_gets_page_image_link_and_post_via_engine() {
    let mut harness = harness();
    harness.run();
    harness.get_by_label("Navigateur simulé").click();
    harness.run();
    assert_eq!(harness.state().tab, Tab::Browser);
    harness.get_by_label("Ouvrir la page").click();
    settle(&mut harness);
    assert_eq!(harness.state().image_count(), 1);
    harness.get_by_label("Envoyer au serveur").click();
    settle(&mut harness);
    assert_eq!(harness.state().last_post(), Some("message=Bonjour"));
    harness.get_by_label("Voir les détails").click();
    settle(&mut harness);
    harness.get_by_label("Retour").click();
    settle(&mut harness);
    assert!(harness.state().trace_len() > 10);
}
#[test]
fn large_trace_uses_the_virtualized_table() {
    let mut harness = harness();
    harness.run();
    harness.get_by_label("Charger 10 000 événements").click();
    settle(&mut harness);
    assert_eq!(harness.state().trace_len(), 10_000);
    assert_eq!(harness.state().tab, Tab::Trace);
}
#[test]
#[cfg(feature = "ui-snapshots")]
fn rendered_workbench_reference() {
    let mut harness = harness();
    harness.run();
    harness.snapshot("j0-topology");
    harness.get_by_label("Navigateur simulé").click();
    harness.run();
    harness.get_by_label("Ouvrir la page").click();
    settle(&mut harness);
    harness.snapshot("j0-browser");
}
