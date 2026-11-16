use egui_kittest::{Harness, kittest::Queryable as _};
use sns_model::*;
use sns_project::session::*;
use sns_ui_egui::{Mode, Workbench};
use std::{path::Path, time::Duration};
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn seed(root: &Path) -> (Project, std::path::PathBuf) {
    let lease = SessionLease::create(root).unwrap();
    let mut project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    project.name = "Réseau à récupérer".into();
    let record = SessionRecord {
        session_version: 1,
        session_id: lease.id().clone(),
        saved_at_unix_ms: Decimal(1),
        revision: Decimal(4),
        origin_display: Some("/cours/reseau.sinesi.json".into()),
        project: project.clone(),
        view: SessionView {
            diagram: true,
            zoom: 1.,
            pan: [20., 40.],
            selected: vec![id("host-a")],
        },
        edit_state: None,
    };
    lease.save(&record).unwrap();
    (project, lease.path())
}
fn harness(app: Workbench) -> Harness<'static, Workbench> {
    Harness::builder()
        .with_max_steps(40)
        .with_size((1800., 1200.))
        .build_ui_state(|ui, state| state.show(ui), app)
}
fn storage(h: &mut Harness<'_, Workbench>) {
    for _ in 0..300 {
        h.step();
        if !h.state().session_busy() {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    h.state_mut().wait_for_worker().unwrap();
    h.run();
}
#[test]
fn recovery_offer_restores_language_document_selection_and_two_views_in_all_six_languages() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for language in sns_i18n::Language::ALL {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("sessions");
        let prefs = dir.path().join("settings.json");
        sns_settings::save(
            &prefs,
            &sns_settings::Settings {
                language,
                ..Default::default()
            },
        )
        .unwrap();
        let (project, old) = seed(&root);
        let mut h = harness(
            Workbench::with_storage(Some(prefs.clone()), Some(root.clone()), None).unwrap(),
        );
        storage(&mut h);
        assert_eq!(h.state().language(), language);
        assert_eq!(h.state().recovery_count(), 1);
        h.get_by_role_and_label(egui::Role::Button, language.text("session_restore"));
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-session-offer-{}", language.code()));
        h.get_by_role_and_label(egui::Role::Button, language.text("session_restore"))
            .click();
        storage(&mut h);
        // Wait for the immediate publication that makes retiring the input safe.
        storage(&mut h);
        assert_eq!(h.state().project(), &project);
        assert_eq!(h.state().mode, Mode::Design);
        assert!(h.state().project_path().is_none());
        assert!(h.state().recovery_view().diagram);
        assert!(h.state().selected().contains(&id("host-a")));
        assert_eq!(h.state().recovery_view().zoom, 1.);
        assert_eq!(h.state().recovery_view().pan, [20., 40.]);
        assert!(h.state().session_error().is_none());
        assert!(h.state().session_saved());
        assert!(!old.exists());
        #[cfg(feature = "ui-snapshots")]
        h.snapshot(format!("j7-session-restored-{}", language.code()));
        h.get_by_label(language.text("view_icons")).click();
        h.run();
        assert!(!h.state().recovery_view().diagram);
        assert_eq!(h.state().project(), &project);
        let new = dir.path().join("recovered.json");
        h.state_mut().save_project_to(new.clone()).unwrap();
        assert_eq!(sns_project::load_project(&new).unwrap(), project);
        let protected = root.join("user-project.json");
        assert!(h.state_mut().save_project_to(protected.clone()).is_err());
        assert!(!protected.exists());
        assert_eq!(sns_settings::load(&prefs).unwrap().language, language);
        #[cfg(feature = "ui-snapshots")]
        snapshots.extend_harness(&mut h);
    }
    #[cfg(feature = "ui-snapshots")]
    snapshots.unwrap();
}
#[test]
fn explicit_launch_file_has_priority_and_failed_restore_preserves_document_and_cache() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("sessions");
    let (_old_project, old) = seed(&root);
    let mut explicit =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    explicit.name = "Projet fourni au lancement".into();
    let source = dir.path().join("explicit.json");
    sns_project::save_project(&source, &explicit).unwrap();
    let bytes = std::fs::read(&source).unwrap();
    let mut h =
        harness(Workbench::with_storage(None, Some(root.clone()), Some(source.clone())).unwrap());
    storage(&mut h);
    assert_eq!(h.state().project(), &explicit);
    assert_eq!(h.state().project_path(), Some(source.as_path()));
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
    assert!(
        h.query_by_role_and_label(
            egui::Role::Button,
            sns_i18n::Language::Fr.text("session_restore")
        )
        .is_none()
    );
    h.state_mut().request_autosave();
    storage(&mut h);
    assert!(h.state().session_saved());
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
    let lease =
        SessionLease::acquire(&root, &id(old.file_stem().unwrap().to_str().unwrap())).unwrap();
    let mut record = lease.read().unwrap();
    record.edit_state = Some(serde_json::json!({"version":999}));
    lease.save(&record).unwrap();
    drop(lease);
    h.get_by_role_and_label(
        egui::Role::Button,
        sns_i18n::Language::Fr.text("recover_session"),
    )
    .click();
    storage(&mut h);
    h.get_by_role_and_label(
        egui::Role::Button,
        sns_i18n::Language::Fr.text("session_restore"),
    )
    .click();
    storage(&mut h);
    assert_eq!(h.state().project(), &explicit);
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
    assert!(old.exists());
    assert!(h.state().session_error().is_some());
}

fn settle(h: &mut Harness<'_, Workbench>) {
    h.step();
    h.state_mut().wait_for_worker().unwrap();
    h.run();
}
fn replace_text(h: &mut Harness<'_, Workbench>, label: &str, text: &str) {
    h.get_by_label(label).focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label(label).type_text(text);
    h.run();
}
fn published(root: &Path) -> Option<SessionRecord> {
    std::fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .find_map(|e| {
            std::fs::read(e.path())
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        })
}
#[test]
#[ignore = "helper invoked only by the automatic-recovery parent test"]
fn automatic_ui_writer_child() {
    let Some(root) = std::env::var_os("SINESI_UI_SESSION_CHILD") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let l = sns_i18n::Language::De;
    let mut app = Workbench::with_storage(
        Some(root.join("settings.json")),
        Some(root.join("sessions")),
        Some(root.join("original.json")),
    )
    .unwrap();
    app.choose_language(l);
    let mut h = harness(app);
    storage(&mut h);
    h.get_by_label(&format!("{} · F3", l.text("mode_test")))
        .click();
    settle(&mut h);
    h.get_by_label(l.text("desktop")).click();
    h.run();
    h.get_by_label(&format!("{} · {}", l.text("open"), l.text("text_editor")))
        .click();
    settle(&mut h);
    replace_text(&mut h, l.text("content"), "Brouillon de l’élève — Grüße");
    h.get_by_label(&format!("{} · {}", l.text("open"), l.text("terminal")))
        .click_accesskit();
    settle(&mut h);
    h.get_by_label(l.text("terminal_input")).focus();
    h.run();
    h.get_by_label(l.text("terminal_input"))
        .type_text("touch /interdit.txt");
    h.run();
    h.get_by_label(&format!("{} · F2", l.text("mode_build")))
        .click();
    settle(&mut h);
    h.get_by_label(l.text("topology")).click();
    h.run();
    h.get_by_label(&l.format("canvas_device", &[("label", "Poste A")]).unwrap())
        .click();
    h.run();
    replace_text(&mut h, l.text("device_name"), "Poste A — travail en cours");
    h.get_by_label(l.text("apply")).click();
    h.run();
    replace_text(
        &mut h,
        l.text("device_name"),
        "Poste A — travail en cours non appliqué",
    );
    h.get_by_label(l.text("view_diagram")).click();
    h.run();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    // No forced-save API: the production two-second timer must publish all edits.
    let snapshot = loop {
        h.step();
        if let Some(point) = published(&root.join("sessions"))
            && point.project.device(&id("host-a")).unwrap().label == "Poste A — travail en cours"
            && point.view.diagram
            && point
                .edit_state
                .as_ref()
                .and_then(|s| s.pointer("/inspector/0/label"))
                .and_then(serde_json::Value::as_str)
                == Some("Poste A — travail en cours non appliqué")
            && point.edit_state.as_ref().is_some_and(|s| {
                serde_json::to_string(s)
                    .unwrap()
                    .contains("Brouillon de l’élève")
            })
        {
            break point;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "automatic snapshot was not published: {:?}",
            h.state().session_error()
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    use std::io::Write;
    println!("SNS_UI_SESSION_READY:{}", snapshot.session_id.as_str());
    std::io::stdout().flush().unwrap();
    loop {
        std::thread::park_timeout(Duration::from_secs(30));
    }
}
struct KillOnDrop(std::process::Child);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
fn automatic_gui_snapshot_survives_forced_exit_with_unapplied_buffers_and_explicit_file_preserved()
{
    use std::{
        io::{BufRead, BufReader},
        process::{Command, Stdio},
    };
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("original.json");
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/hosts.json")).unwrap();
    sns_project::save_project(&source, &project).unwrap();
    let before = std::fs::read(&source).unwrap();
    let mut child = KillOnDrop(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "automatic_ui_writer_child",
                "--ignored",
                "--nocapture",
            ])
            .env("SINESI_UI_SESSION_CHILD", dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let line = line.unwrap();
            if let Some((_, id)) = line.split_once("SNS_UI_SESSION_READY:") {
                let _ = tx.send(id.trim().to_owned());
                return;
            }
        }
    });
    let _session = rx
        .recv_timeout(Duration::from_secs(15))
        .expect("the UI child automatically publishes a durable session");
    child.0.kill().unwrap();
    let exit = child.0.wait().unwrap();
    reader.join().unwrap();
    assert!(!exit.success());
    assert_eq!(std::fs::read(&source).unwrap(), before);
    let l = sns_i18n::Language::De;
    let mut h = harness(
        Workbench::with_storage(
            Some(dir.path().join("settings.json")),
            Some(dir.path().join("sessions")),
            None,
        )
        .unwrap(),
    );
    storage(&mut h);
    assert_eq!(h.state().language(), l);
    assert_eq!(h.state().recovery_count(), 1);
    h.get_by_role_and_label(egui::Role::Button, l.text("session_restore"))
        .click();
    storage(&mut h);
    storage(&mut h);
    assert_eq!(h.state().mode, Mode::Design);
    assert!(h.state().project_path().is_none());
    assert!(h.state().recovery_view().diagram);
    assert_eq!(
        h.state().project().device(&id("host-a")).unwrap().label,
        "Poste A — travail en cours"
    );
    assert_eq!(
        h.get_by_label(l.text("device_name")).value().as_deref(),
        Some("Poste A — travail en cours non appliqué")
    );
    h.get_by_label(&format!("{} · F3", l.text("mode_test")))
        .click();
    settle(&mut h);
    h.get_by_label(l.text("desktop")).click();
    h.run();
    h.get_by_label(&format!("{} · {}", l.text("open"), l.text("text_editor")))
        .click();
    settle(&mut h);
    assert_eq!(
        h.get_by_label(l.text("content")).value().as_deref(),
        Some("Brouillon de l’élève — Grüße")
    );
    h.get_by_label(&format!("{} · {}", l.text("open"), l.text("terminal")))
        .click_accesskit();
    settle(&mut h);
    assert_eq!(
        h.get_by_label(l.text("terminal_input")).value().as_deref(),
        Some("touch /interdit.txt")
    );
    assert!(
        !h.state()
            .project()
            .device(&id("host-a"))
            .unwrap()
            .files
            .iter()
            .any(|f| f.path == "/interdit.txt")
    );
    assert_eq!(std::fs::read(&source).unwrap(), before);
    if let Some(out) = std::env::var_os("SNS_RECORD_J7_SESSION") {
        let out = std::path::Path::new(&out);
        std::fs::create_dir_all(out).unwrap();
        std::fs::write(out.join("j7-session-ui-crash.json"),serde_json::to_vec_pretty(&serde_json::json!({"forced_child_exit_success":exit.success(),"automatic_interval_seconds":2,"saved_original_unchanged":std::fs::read(&source).unwrap()==before,"language":l.code(),"diagram":h.state().recovery_view().diagram,"document_label":h.state().project().device(&id("host-a")).unwrap().label,"unapplied_inspector_restored":true,"text_editor_draft_restored":true,"terminal_input_restored_without_execution":true})).unwrap()).unwrap();
    }
}

#[test]
fn a_full_cache_still_allows_recovery_without_deleting_other_documents() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("sessions");
    let mut newest = None;
    for n in 0..100 {
        let lease = match SessionLease::create(&root) {
            Ok(l) => l,
            Err(_) => break,
        };
        let mut record: SessionRecord =
            serde_json::from_slice(include_bytes!("../../../fixtures/valid/session.json")).unwrap();
        record.session_id = lease.id().clone();
        record.saved_at_unix_ms = Decimal(n);
        record.project.name = format!("Cache {n}");
        lease.save(&record).unwrap();
        newest = Some((record.project.clone(), lease.path()));
    }
    assert!(SessionLease::create(&root).is_err());
    let (expected, path) = newest.unwrap();
    let mut h = harness(Workbench::with_storage(None, Some(root.clone()), None).unwrap());
    storage(&mut h);
    assert_eq!(h.state().recovery_count(), MAX_RECOVERY_ENTRIES);
    assert!(h.state().session_error().is_some());
    let l = sns_i18n::Language::Fr;
    h.get_by_role_and_label(egui::Role::Button, l.text("session_restore"))
        .click();
    storage(&mut h);
    storage(&mut h);
    assert_eq!(h.state().project(), &expected);
    assert!(h.state().session_error().is_none());
    assert!(h.state().session_saved());
    assert!(path.exists());
    assert!(summaries(&root).unwrap().0.len() > 1);
}

#[test]
fn normal_quit_keeps_latest_applied_and_unapplied_edits_without_waiting_for_periodic_save() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("sessions");
    let source = dir.path().join("original.json");
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    sns_project::save_project(&source, &project).unwrap();
    let original = std::fs::read(&source).unwrap();
    let l = sns_i18n::Language::Fr;
    let mut h =
        harness(Workbench::with_storage(None, Some(root.clone()), Some(source.clone())).unwrap());
    storage(&mut h);
    h.get_by_label(&l.format("canvas_device", &[("label", "Poste A")]).unwrap())
        .click();
    h.run();
    replace_text(&mut h, l.text("device_name"), "Poste A conservé");
    h.get_by_label(l.text("apply")).click();
    h.run();
    replace_text(&mut h, l.text("device_name"), "Nom encore non appliqué");
    h.get_by_label(l.text("view_diagram")).click();
    h.run();
    h.state_mut().request_close();
    h.run();
    h.get_by_label(l.text("session_keep_and_close")).click();
    let deadline = std::time::Instant::now() + Duration::from_secs(4);
    loop {
        h.step();
        if h.output()
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .is_some_and(|out| {
                out.commands
                    .iter()
                    .any(|cmd| matches!(cmd, egui::ViewportCommand::Close))
            })
        {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "close did not publish recovery: {:?}",
            h.state().session_error()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!h.state().session_busy());
    let saved = published(&root).unwrap();
    assert_eq!(
        saved.project.device(&id("host-a")).unwrap().label,
        "Poste A conservé"
    );
    assert_eq!(
        saved
            .edit_state
            .unwrap()
            .pointer("/inspector/0/label")
            .and_then(serde_json::Value::as_str),
        Some("Nom encore non appliqué")
    );
    assert!(saved.view.diagram);
    assert_eq!(std::fs::read(&source).unwrap(), original);
    drop(h);
    let mut restored = harness(Workbench::with_storage(None, Some(root.clone()), None).unwrap());
    storage(&mut restored);
    restored
        .get_by_role_and_label(egui::Role::Button, l.text("session_restore"))
        .click();
    storage(&mut restored);
    assert_eq!(
        restored
            .state()
            .project()
            .device(&id("host-a"))
            .unwrap()
            .label,
        "Poste A conservé"
    );
    assert_eq!(
        restored
            .get_by_label(l.text("device_name"))
            .value()
            .as_deref(),
        Some("Nom encore non appliqué")
    );
}
#[test]
fn preferences_cannot_replace_an_explicit_project_or_be_replaced_by_project_save() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("project.json");
    let root = dir.path().join("sessions");
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    sns_project::save_project(&source, &project).unwrap();
    let original = std::fs::read(&source).unwrap();
    let mut app =
        Workbench::with_storage(Some(source.clone()), Some(root), Some(source.clone())).unwrap();
    app.choose_language(sns_i18n::Language::De);
    assert_eq!(std::fs::read(&source).unwrap(), original);
    assert_eq!(app.project(), &project);
    let prefs = dir.path().join("settings.json");
    sns_settings::save(&prefs, &Default::default()).unwrap();
    let original_prefs = std::fs::read(&prefs).unwrap();
    let mut app = Workbench::with_storage(Some(prefs.clone()), None, Some(source.clone())).unwrap();
    assert!(app.save_project_to(prefs.clone()).is_err());
    assert_eq!(std::fs::read(prefs).unwrap(), original_prefs);
}
