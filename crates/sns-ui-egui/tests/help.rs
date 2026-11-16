use egui_kittest::{Harness, kittest::Queryable as _};
use sns_i18n::Language;
use sns_settings::help::{PRODUCT_VERSION, Topic};
use sns_ui_egui::Workbench;
fn harness() -> Harness<'static, Workbench> {
    Harness::builder()
        .with_max_steps(40)
        .with_size((1600., 1050.))
        .build_ui_state(|ui, state| state.show(ui), Workbench::new().unwrap())
}
fn settle(h: &mut Harness<'_, Workbench>) {
    h.step();
    h.state_mut().wait_for_worker().unwrap();
    h.run();
}
#[test]
fn f1_opens_context_and_about_in_six_languages_both_views_without_mutation() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for language in Language::ALL {
        let mut h = harness();
        h.state_mut().choose_language(language);
        h.run();
        for diagram in [false, true] {
            h.get_by_label(language.text(if diagram {
                "view_diagram"
            } else {
                "view_icons"
            }))
            .click();
            h.run();
            for (key, shortcut, topic, name) in [
                ("mode_build", "F2", Topic::Design, "design"),
                ("mode_test", "F3", Topic::Simulation, "simulation"),
                ("mode_annotate", "F4", Topic::Documentation, "documentation"),
            ] {
                h.get_by_label(&format!("{} · {shortcut}", language.text(key)))
                    .click();
                settle(&mut h);
                let project = h.state().project().clone();
                let trace = h.state().trace().to_vec();
                let now = h.state().now();
                let run = h.state().run_id().clone();
                let selected = h.state().selected().clone();
                if diagram {
                    h.get_by_label(&format!("{} · F1", language.text("help")))
                        .click();
                } else {
                    h.key_press(egui::Key::F1);
                }
                h.run();
                h.get_by_label(language.text(topic.paragraph_keys()[0]));
                assert_eq!(h.state().project(), &project);
                assert_eq!(h.state().now(), now);
                assert_eq!(h.state().run_id(), &run);
                assert_eq!(h.state().trace(), trace);
                assert_eq!(h.state().selected(), &selected);
                #[cfg(feature = "ui-snapshots")]
                h.snapshot(format!(
                    "j7-display-help-{name}-{}-{}",
                    if diagram { "diagram" } else { "icons" },
                    language.code()
                ));
                #[cfg(not(feature = "ui-snapshots"))]
                let _ = name;
                h.key_press(egui::Key::F1);
                h.run();
            }
        }
        h.key_press(egui::Key::F1);
        h.run();
        h.get_by_label(language.text("help_about")).click();
        h.run();
        h.get_by_label(PRODUCT_VERSION);
        h.get_by_label(language.text("help_loaded_format"));
        h.get_by_label("2");
        h.get_by_label("9");
        h.get_by_label(language.text("help_credit_code"));
        h.get_by_label(language.text("help_code_license"));
        h.get_by_label(language.text("help_font_license"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-display-help-about-{}", language.code()));
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn focused_draft_stays_intact_while_f1_and_escape_open_and_close_help() {
    let mut h = harness();
    h.run();
    h.get_by_label("Équipement Poste A").click();
    h.run();
    let project = h.state().project().clone();
    h.get_by_label("Nom de l’équipement").focus();
    h.run();
    h.get_by_label("Nom de l’équipement")
        .type_text(" brouillon");
    h.run();
    let draft = h
        .get_by_label("Nom de l’équipement")
        .value()
        .unwrap()
        .to_owned();
    h.key_press(egui::Key::F1);
    h.run();
    h.get_by_label(Language::Fr.text("help_design_intro"));
    h.key_press(egui::Key::Escape);
    h.run();
    assert_eq!(
        h.get_by_label("Nom de l’équipement").value().unwrap(),
        draft
    );
    assert_eq!(h.state().project(), &project);
}
#[test]
fn guided_example_reuses_the_unsaved_document_confirmation() {
    let mut h = harness();
    h.run();
    h.get_by_label("Ajouter un portable").click();
    h.run();
    let edited = h.state().project().clone();
    assert_eq!(edited.devices.len(), 5);
    let open = |h: &mut Harness<'_, Workbench>| {
        h.key_press(egui::Key::F1);
        h.run();
        h.get_by_label(Language::Fr.text("help_examples")).click();
        h.run();
        h.get_by_label(&format!(
            "{} · {}",
            Language::Fr.text("help_load_example"),
            Language::Fr.text("example")
        ))
        .click();
        h.run();
    };
    open(&mut h);
    h.get_by_label(Language::Fr.text("unsaved_title"));
    assert_eq!(h.state().project(), &edited);
    h.get_by_label(Language::Fr.text("continue_editing"))
        .click();
    h.run();
    assert_eq!(h.state().project(), &edited);
    open(&mut h);
    h.get_by_label(Language::Fr.text("discard")).click();
    settle(&mut h);
    assert_eq!(h.state().project().devices.len(), 4);
    assert_eq!(h.state().mode, sns_ui_egui::Mode::Design);
    assert_eq!(h.state().now(), 0);
}

#[test]
fn direct_arp_and_modem_examples_open_from_the_localized_network_menu() {
    use sns_settings::help::Example;
    for language in Language::ALL {
        let mut h = harness();
        h.state_mut().choose_language(language);
        h.run();
        for example in [
            Example::Direct,
            Example::Arp,
            Example::ModemListen,
            Example::ModemConnect,
        ] {
            h.get_by_label(language.text("examples_menu")).click();
            h.run();
            h.get_by_label(language.text(example.label_key())).click();
            settle(&mut h);
            assert_eq!(
                h.state().project(),
                &sns_project::parse_project(example.project_bytes()).unwrap()
            );
            assert!(h.state().trace().is_empty());
            assert_eq!(h.state().now(), 0);
        }
    }
}
