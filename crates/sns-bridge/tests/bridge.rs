//! R-PONT exercises two OS processes and the production adapter, never real ICMP.
use sns_bridge::{Bridge, BridgeEvent, Driver};
use sns_engine::{Engine, Limits};
use sns_model::*;
use std::{
    io::Write,
    net::TcpStream,
    path::Path,
    time::{Duration, Instant},
};
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn project(remote: bool) -> Project {
    let bytes = if remote {
        include_bytes!("../../../fixtures/valid/modem-listen.json").as_slice()
    } else {
        include_bytes!("../../../fixtures/valid/modem-connect.json").as_slice()
    };
    serde_json::from_slice(bytes).unwrap()
}
fn engine(remote: bool) -> Engine {
    Engine::from_project(project(remote), 42, Limits::default()).unwrap()
}
fn tick(d: &mut Driver, e: &mut Engine) {
    d.poll(e);
    e.pump(128, Some(e.now() + 5000)).unwrap();
    std::thread::sleep(Duration::from_millis(5));
}
fn wait(timeout: u64, mut f: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(timeout);
    while Instant::now() < until {
        if f() {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("bridge test deadline");
}
#[test]
fn bridge_child_process() {
    let Some(dir) = std::env::var_os("SNS_BRIDGE_CHILD") else {
        return;
    };
    let dir = Path::new(&dir);
    let mut e = engine(true);
    let mut d = Driver::default();
    d.start(
        &mut e,
        &id("modem"),
        ModemConfig {
            role: ModemRole::Listen,
            endpoint: "127.0.0.1:0".into(),
        },
    )
    .unwrap();
    wait(3, || {
        tick(&mut d, &mut e);
        if let Some(v) = d.views().get(&id("modem"))
            && v.state == "bridge.listening"
        {
            std::fs::write(dir.join("endpoint"), &v.endpoint).unwrap();
            return true;
        }
        false
    });
    wait(8, || {
        tick(&mut d, &mut e);
        dir.join("stop").exists()
    });
    std::fs::write(
        dir.join("remote-trace.json"),
        serde_json::to_vec_pretty(e.trace()).unwrap(),
    )
    .unwrap();
    assert!(e.trace().iter().any(|t| t.kind == "bridge.input"));
}
struct Child(std::process::Child);
impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
fn two_real_instances_exchange_ping_record_external_inputs_and_disconnect() {
    let dir = tempfile::tempdir().unwrap();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["bridge_child_process", "--exact", "--nocapture"])
        .env("SNS_BRIDGE_CHILD", dir.path())
        .spawn()
        .unwrap();
    let mut child = Child(child);
    wait(3, || dir.path().join("endpoint").exists());
    let endpoint = std::fs::read_to_string(dir.path().join("endpoint")).unwrap();
    let mut e = engine(false);
    let mut d = Driver::default();
    d.start(
        &mut e,
        &id("modem"),
        ModemConfig {
            role: ModemRole::Connect,
            endpoint,
        },
    )
    .unwrap();
    wait(3, || {
        tick(&mut d, &mut e);
        d.views()
            .get(&id("modem"))
            .is_some_and(|v| v.state == "bridge.connected")
    });
    let before_arrival = e.checkpoint().unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("ping-over-real-tunnel"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(0),
        command: Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    })
    .unwrap();
    wait(4, || {
        tick(&mut d, &mut e);
        e.trace().iter().any(|t| t.kind == "icmp.echo.reply")
    });
    assert!(
        e.trace()
            .iter()
            .any(|t| t.kind == "bridge.input" && t.payload.contains_key("packet"))
    );
    let input = e.trace().iter().find(|t| t.kind == "bridge.input").unwrap();
    let packet: BridgePacket = serde_json::from_value(input.payload["packet"].clone()).unwrap();
    // Captured external arrivals replay offline; no bridge adapter is constructed by the engine.
    assert_eq!(packet.version, 1);
    assert!(packet.budget > 0);
    let continued = before_arrival
        .replay_remaining(&e.recording().unwrap())
        .unwrap();
    assert_eq!(continued.canonical_trace(), e.canonical_trace());
    assert_eq!(
        continued.checkpoint_state().unwrap()["network"],
        e.checkpoint_state().unwrap()["network"]
    );
    let active = e.checkpoint().unwrap();
    let recovered = active.restore().unwrap();
    assert_eq!(recovered.canonical_trace(), e.canonical_trace());
    d.stop(&mut e, &id("modem"));
    assert_eq!(d.views()[&id("modem")].state, "bridge.disconnected");
    assert!(!d.active());
    std::fs::write(dir.path().join("stop"), b"stop").unwrap();
    assert!(child.0.wait().unwrap().success());
    if let Some(path) = std::env::var_os("SNS_J6_ARTIFACT_DIR") {
        let path = Path::new(&path);
        std::fs::create_dir_all(path).unwrap();
        std::fs::write(
            path.join("j6-bridge-local.json"),
            serde_json::to_vec_pretty(e.trace()).unwrap(),
        )
        .unwrap();
        std::fs::copy(
            dir.path().join("remote-trace.json"),
            path.join("j6-bridge-remote.json"),
        )
        .unwrap();
    }
    let offline = e.recording().unwrap().replay().unwrap();
    assert_eq!(offline.canonical_trace(), e.canonical_trace());
    assert_eq!(
        offline.checkpoint_state().unwrap(),
        e.checkpoint_state().unwrap()
    );
    let before = e
        .trace()
        .iter()
        .filter(|t| t.kind == "bridge.input")
        .count();
    for _ in 0..10 {
        tick(&mut d, &mut e);
    }
    assert_eq!(
        e.trace()
            .iter()
            .filter(|t| t.kind == "bridge.input")
            .count(),
        before
    );
}
fn listening() -> (Bridge, String) {
    let b = Bridge::start(ModemConfig {
        role: ModemRole::Listen,
        endpoint: "127.0.0.1:0".into(),
    })
    .unwrap();
    let mut endpoint = None;
    wait(2, || {
        for e in b.poll() {
            if let BridgeEvent::Listening(a) = e {
                endpoint = Some(a.to_string());
            }
        }
        endpoint.is_some()
    });
    (b, endpoint.unwrap())
}
#[test]
fn absent_endpoint_bad_version_oversize_and_mid_frame_cut_have_explicit_states() {
    // Reserve then close a local port, making refusal deterministic on the test host.
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = l.local_addr().unwrap();
    drop(l);
    let b = Bridge::start(ModemConfig {
        role: ModemRole::Connect,
        endpoint: address.to_string(),
    })
    .unwrap();
    wait(2, || {
        b.poll()
            .iter()
            .any(|e| matches!(e,BridgeEvent::Disconnected(c)if c=="bridge.endpoint_unavailable"))
    });
    for case in 0..3 {
        let (b, endpoint) = listening();
        let mut raw = TcpStream::connect(endpoint).unwrap();
        if case == 0 {
            raw.write_all(b"SNS-BRIDGE/2\n").unwrap();
        } else {
            raw.write_all(b"SNS-BRIDGE/1\n").unwrap();
            if case == 1 {
                raw.write_all(&((BRIDGE_MAX_PACKET + 1) as u32).to_be_bytes())
                    .unwrap();
            } else {
                raw.write_all(&1000u32.to_be_bytes()).unwrap();
                raw.write_all(b"{\"version\":1").unwrap();
            }
        }
        drop(raw);
        let expected = if case == 0 {
            "bridge.incompatible_version"
        } else if case == 1 {
            "bridge.message_too_large"
        } else {
            "bridge.disconnected"
        };
        wait(2, || {
            let events = b.poll();
            assert!(!events.iter().any(|e| matches!(e, BridgeEvent::Packet(_))));
            events
                .iter()
                .any(|e| matches!(e,BridgeEvent::Disconnected(c)if c==expected))
        });
    }
}
