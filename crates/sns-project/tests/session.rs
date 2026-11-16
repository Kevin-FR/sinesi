//! Real process leases and forced-exit recovery, separate from project files.
use sns_model::*;
use sns_project::session::*;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    time::Duration,
};
fn project() -> Project {
    sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap()
}
fn record(lease: &SessionLease) -> SessionRecord {
    SessionRecord {
        session_version: 1,
        session_id: lease.id().clone(),
        saved_at_unix_ms: Decimal(unix_time_ms()),
        revision: Decimal(1),
        origin_display: None,
        project: project(),
        view: SessionView::default(),
        edit_state: None,
    }
}

#[test]
fn leases_isolate_live_instances_and_invalid_snapshots_preserve_previous_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let a = SessionLease::create(dir.path()).unwrap();
    let b = SessionLease::create(dir.path()).unwrap();
    assert_ne!(a.id(), b.id());
    let original = record(&a);
    a.save(&original).unwrap();
    b.save(&record(&b)).unwrap();
    assert!(scan(dir.path()).unwrap().recoveries.is_empty());
    let bytes = std::fs::read(a.path()).unwrap();
    let mut invalid = original.clone();
    invalid.view.zoom = f32::NAN;
    assert!(a.save(&invalid).is_err());
    assert_eq!(std::fs::read(a.path()).unwrap(), bytes);
    invalid = original.clone();
    invalid.project.schema_version = 999;
    assert!(a.save(&invalid).is_err());
    assert_eq!(std::fs::read(a.path()).unwrap(), bytes);
    let a_path = a.path();
    drop(a);
    let first = scan(dir.path()).unwrap();
    assert_eq!(first.recoveries.len(), 1);
    assert_eq!(first.recoveries[0].record, original);
    assert!(scan(dir.path()).unwrap().recoveries.is_empty());
    assert_eq!(first.recoveries[0].lease.path(), a_path);
    let b_path = b.path();
    drop(b);
    std::fs::write(&b_path, b"{broken").unwrap();
    let bad = scan(dir.path()).unwrap();
    assert!(bad.recoveries.is_empty());
    assert_eq!(bad.diagnostics.len(), 1);
    assert_eq!(std::fs::read(&b_path).unwrap(), b"{broken");
    for recovery in first.recoveries {
        recovery.lease.discard().unwrap();
    }
    assert!(!a_path.exists());
    assert!(b_path.exists());
}

/// Invoked by the parent test in a distinct process, never as a desktop command.
#[test]
#[ignore = "helper invoked only by the forced-exit parent test"]
fn crash_writer_child() {
    let Some(root) = std::env::var_os("SINESI_SESSION_CRASH_CHILD") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let lease = SessionLease::create(&root.join("sessions")).unwrap();
    let mut record = record(&lease);
    record.project = sns_project::load_project(&root.join("original.json")).unwrap();
    record.project.name = "Modifié avant arrêt forcé".into();
    record.view.diagram = true;
    record.view.zoom = 1.5;
    record.view.pan = [20., 30.];
    record.view.selected = vec![Id::new("host-a").unwrap()];
    record.revision = Decimal(2);
    record.origin_display = Some(root.join("original.json").display().to_string());
    lease.save(&record).unwrap();
    println!("SNS_SESSION_READY:{}", lease.id().as_str());
    std::io::stdout().flush().unwrap();
    loop {
        std::thread::park_timeout(Duration::from_secs(30));
    }
}
struct KillOnDrop(Child);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
fn forced_child_exit_releases_lease_and_recovers_latest_document_without_touching_saved_project() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("original.json");
    sns_project::save_project(&original, &project()).unwrap();
    let saved = std::fs::read(&original).unwrap();
    let mut child = KillOnDrop(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "crash_writer_child", "--ignored", "--nocapture"])
            .env("SINESI_SESSION_CRASH_CHILD", dir.path())
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
            if let Some((_, id)) = line.split_once("SNS_SESSION_READY:") {
                let _ = tx.send(id.trim().to_owned());
                return;
            }
        }
    });
    let session = rx
        .recv_timeout(Duration::from_secs(15))
        .expect("child publishes a durable recovery before kill");
    assert!(
        scan(&dir.path().join("sessions"))
            .unwrap()
            .recoveries
            .is_empty()
    );
    child.0.kill().unwrap();
    let exit = child.0.wait().unwrap();
    assert!(!exit.success());
    reader.join().unwrap();
    let mut recovered = scan(&dir.path().join("sessions")).unwrap();
    assert!(recovered.diagnostics.is_empty());
    assert_eq!(recovered.recoveries.len(), 1);
    let recovery = recovered.recoveries.remove(0);
    assert_eq!(recovery.record.session_id.as_str(), session);
    assert_eq!(recovery.record.project.name, "Modifié avant arrêt forcé");
    assert!(recovery.record.view.diagram);
    assert_eq!(recovery.record.revision, Decimal(2));
    assert_eq!(std::fs::read(&original).unwrap(), saved);
    let copy = dir.path().join("recovered.json");
    sns_project::save_project(&copy, &recovery.record.project).unwrap();
    assert_eq!(
        sns_project::load_project(&copy).unwrap(),
        recovery.record.project
    );
    if let Some(out) = std::env::var_os("SNS_RECORD_J7_SESSION") {
        let out = std::path::Path::new(&out);
        std::fs::create_dir_all(out).unwrap();
        std::fs::write(out.join("j7-session-crash.json"),serde_json::to_vec_pretty(&serde_json::json!({"child_process":true,"forced_exit_success":exit.success(),"exit_code":exit.code(),"lease_live_hidden":true,"saved_original_unchanged":std::fs::read(&original).unwrap()==saved,"recovered_revision":recovery.record.revision,"recovered_name":recovery.record.project.name,"recovered_diagram":recovery.record.view.diagram})).unwrap()).unwrap();
        std::fs::write(
            out.join("j7-session-recovered.json"),
            serde_json::to_vec_pretty(&recovery.record).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn unpublished_slots_are_cleaned_but_live_writers_and_nonempty_corruption_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let unused = SessionLease::create(dir.path()).unwrap();
    let empty = unused.path();
    assert_eq!(std::fs::metadata(&empty).unwrap().len(), 0);
    assert!(summaries(dir.path()).unwrap().0.is_empty());
    assert!(empty.exists());
    drop(unused);
    assert!(summaries(dir.path()).unwrap().0.is_empty());
    assert!(!empty.exists());
    let guard = SessionLease::create(dir.path()).unwrap();
    let path = guard.path();
    std::fs::write(&path, b"broken-data-preserved").unwrap();
    drop(guard);
    let (_, errors) = summaries(dir.path()).unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(std::fs::read(path).unwrap(), b"broken-data-preserved");
}
