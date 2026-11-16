//! R-P2P: actual simulated TCP, including cycles and cancellation generations.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sns_engine::{Engine, Limits};
use sns_model::*;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn engine() -> Engine {
    Engine::from_project(
        serde_json::from_str(include_str!("../../../fixtures/valid/p2p.json")).unwrap(),
        42,
        Limits::default(),
    )
    .unwrap()
}
fn cmd(e: &mut Engine, host: &str, c: P2pCommand) -> Result<(), Diagnostic> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let cid = format!(
        "p2p-cmd-{}",
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(&cid),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::App {
            device: id(host),
            command: AppCommand::P2p {
                application: id(&format!("{host}-p2p")),
                command: c,
            },
        },
    })
    .map(|_| ())
}
fn run(e: &mut Engine) {
    let until = e.now() + 2_000_000;
    e.run(100_000, Some(until)).unwrap();
}
fn view(e: &Engine, h: &str) -> sns_apps::P2pView {
    e.apps()
        .views(e.project())
        .into_iter()
        .find(|x| x.device == id(h))
        .unwrap()
        .applications[0]
        .p2p
        .clone()
}
fn peer(ip: &str) -> Endpoint {
    Endpoint {
        address: ip.parse().unwrap(),
        port: 6346,
    }
}
fn join(e: &mut Engine, h: &str, p: Option<Endpoint>) {
    cmd(e, h, P2pCommand::Join { peer: p }).unwrap();
    run(e);
}
#[test]
fn cyclic_overlay_discovers_searches_downloads_exact_bytes_and_resets() {
    let mut e = engine();
    join(&mut e, "host-a", None);
    join(&mut e, "host-b", Some(peer("192.168.1.2")));
    join(&mut e, "host-c", Some(peer("192.168.1.3")));
    join(&mut e, "host-a", Some(peer("192.168.1.4")));
    for h in ["host-a", "host-b", "host-c"] {
        assert_eq!(view(&e, h).neighbors.len(), 2);
    }
    cmd(&mut e, "host-a", P2pCommand::Discover { ttl: 4 }).unwrap();
    run(&mut e);
    assert_eq!(view(&e, "host-a").discovered.len(), 2);
    cmd(
        &mut e,
        "host-a",
        P2pCommand::Search {
            text: "reseau".into(),
            ttl: 4,
        },
    )
    .unwrap();
    run(&mut e);
    let results = view(&e, "host-a").results;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].source, peer("192.168.1.4"));
    cmd(
        &mut e,
        "host-a",
        P2pCommand::Download {
            result: results[0].id.clone(),
            destination: "/copie.txt".into(),
        },
    )
    .unwrap();
    run(&mut e);
    let downloaded = e
        .project()
        .device(&id("host-a"))
        .unwrap()
        .files
        .iter()
        .find(|f| f.path == "/copie.txt")
        .unwrap();
    let original = e
        .project()
        .device(&id("host-c"))
        .unwrap()
        .files
        .iter()
        .find(|f| f.path == "/shared/reseau.txt")
        .unwrap();
    assert_eq!(
        STANDARD.decode(&downloaded.data_base64).unwrap(),
        STANDARD.decode(&original.data_base64).unwrap()
    );
    assert!(e.trace().iter().any(|t| t.kind == "p2p.message.duplicate"));
    assert!(e.trace().len() < 10_000);
    cmd(&mut e, "host-a", P2pCommand::Reset).unwrap();
    run(&mut e);
    assert!(view(&e, "host-a").neighbors.is_empty());
    assert!(
        view(&e, "host-b")
            .neighbors
            .iter()
            .all(|p| *p != peer("192.168.1.2"))
    );
    join(&mut e, "host-a", Some(peer("192.168.1.3")));
    assert_eq!(view(&e, "host-a").neighbors.len(), 1);
    if let Some(dir) = std::env::var_os("SNS_J6_ARTIFACT_DIR") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("j6-p2p.jsonl"), e.canonical_trace()).unwrap();
        std::fs::write(
            dir.join("j6-p2p-project.json"),
            serde_json::to_vec_pretty(e.project()).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn cancelled_and_superseded_searches_ignore_in_flight_results() {
    let mut e = engine();
    join(&mut e, "host-a", None);
    join(&mut e, "host-b", Some(peer("192.168.1.2")));
    join(&mut e, "host-c", Some(peer("192.168.1.3")));
    cmd(
        &mut e,
        "host-a",
        P2pCommand::Search {
            text: "reseau".into(),
            ttl: 1,
        },
    )
    .unwrap();
    run(&mut e);
    assert!(view(&e, "host-a").results.is_empty());
    cmd(
        &mut e,
        "host-a",
        P2pCommand::Search {
            text: "reseau".into(),
            ttl: 4,
        },
    )
    .unwrap();
    cmd(&mut e, "host-a", P2pCommand::CancelSearch).unwrap();
    run(&mut e);
    assert!(view(&e, "host-a").results.is_empty());
    assert!(!view(&e, "host-a").searching);
    cmd(
        &mut e,
        "host-a",
        P2pCommand::Search {
            text: "reseau".into(),
            ttl: 4,
        },
    )
    .unwrap();
    cmd(
        &mut e,
        "host-a",
        P2pCommand::Search {
            text: "absent".into(),
            ttl: 4,
        },
    )
    .unwrap();
    run(&mut e);
    assert!(view(&e, "host-a").results.is_empty());
    cmd(
        &mut e,
        "host-a",
        P2pCommand::Search {
            text: "reseau".into(),
            ttl: 4,
        },
    )
    .unwrap();
    run(&mut e);
    assert_eq!(view(&e, "host-a").results.len(), 1);
    cmd(&mut e, "host-a", P2pCommand::ClearResults).unwrap();
    run(&mut e);
    assert!(view(&e, "host-a").results.is_empty());
}
#[test]
fn limits_reject_extra_neighbors_and_invalid_ttl() {
    let mut e = engine();
    let mut p = e.project().clone();
    p.devices[0].applications[0].service = Some(ServiceConfig::P2p(P2pConfig {
        max_neighbors: 1,
        ..Default::default()
    }));
    e = Engine::from_project(p, 42, Limits::default()).unwrap();
    join(&mut e, "host-a", None);
    join(&mut e, "host-b", Some(peer("192.168.1.2")));
    join(&mut e, "host-c", Some(peer("192.168.1.2")));
    assert_eq!(view(&e, "host-a").neighbors.len(), 1);
    assert!(view(&e, "host-c").neighbors.is_empty());
    assert!(cmd(&mut e, "host-a", P2pCommand::Discover { ttl: 0 }).is_err());
}
