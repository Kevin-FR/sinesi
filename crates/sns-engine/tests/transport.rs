//! Transport recipes use actual Ethernet/ARP/IP deliveries, never the J0 shortcut.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sns_engine::{Engine, Limits};
use sns_model::*;
use std::net::Ipv4Addr;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn project() -> Project {
    serde_json::from_str(include_str!("../../../fixtures/valid/lan.json")).unwrap()
}
fn engine() -> Engine {
    Engine::from_project(project(), 42, Limits::default()).unwrap()
}
fn command(
    e: &mut Engine,
    name: &str,
    device: &str,
    request: SocketCommand,
) -> Result<(), Diagnostic> {
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(name),
        project_id: e.project().project_id.clone(),
        expected_revision: Decimal(0),
        run_id: e.run_id().clone(),
        command: Command::Socket {
            device: id(device),
            command: request,
        },
    })
}
fn bind(
    e: &mut Engine,
    name: &str,
    device: &str,
    socket: &str,
    protocol: TransportProtocol,
    port: u16,
) -> Result<(), Diagnostic> {
    command(
        e,
        name,
        device,
        SocketCommand::Bind {
            socket: id(socket),
            owner: id(socket),
            protocol,
            interface: None,
            address: None,
            port,
        },
    )
}
fn connect(e: &mut Engine, name: &str, device: &str, socket: &str, target: &str, port: u16) {
    command(
        e,
        name,
        device,
        SocketCommand::Connect {
            socket: id(socket),
            owner: id(socket),
            destination: target.parse().unwrap(),
            port,
            local_address: None,
            local_port: None,
        },
    )
    .unwrap();
}
fn loss(e: &mut Engine, name: &str, enabled: bool) {
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(name),
        project_id: e.project().project_id.clone(),
        expected_revision: Decimal(0),
        run_id: e.run_id().clone(),
        command: Command::ManualLoss { enabled },
    })
    .unwrap();
}
fn send(e: &mut Engine, name: &str, device: &str, socket: &str, data: &[u8]) {
    command(
        e,
        name,
        device,
        SocketCommand::Send {
            socket: id(socket),
            data_base64: STANDARD.encode(data),
        },
    )
    .unwrap();
}
fn settle(e: &mut Engine) {
    e.run(10000, Some(e.now() + 2_000_000)).unwrap();
}
fn received(e: &Engine, device: &str, socket: &Id) -> Vec<u8> {
    e.trace()
        .iter()
        .filter(|f| {
            f.kind == "socket.event"
                && f.device_id == id(device)
                && f.payload["socket"] == serde_json::json!(socket)
                && f.payload["kind"]["event"] == "data"
        })
        .flat_map(|f| {
            STANDARD
                .decode(f.payload["kind"]["data_base64"].as_str().unwrap())
                .unwrap()
        })
        .collect()
}
fn peer_socket(e: &Engine, device: &str, peer: &str) -> Id {
    e.network()
        .transport
        .views()
        .into_iter()
        .find(|v| {
            v.device == id(device)
                && v.peer
                    .is_some_and(|p| p.address == peer.parse::<Ipv4Addr>().unwrap())
        })
        .unwrap()
        .socket
}
fn pair() -> Engine {
    let mut e = engine();
    bind(
        &mut e,
        "listen",
        "host-b",
        "server",
        TransportProtocol::Tcp,
        9000,
    )
    .unwrap();
    connect(&mut e, "connect", "host-a", "client", "192.168.1.3", 9000);
    settle(&mut e);
    e
}
#[test]
fn tcp_two_clients_have_distinct_endpoints_and_segmented_bidirectional_streams() {
    let mut e = pair();
    connect(
        &mut e,
        "connect-c",
        "host-c",
        "client-c",
        "192.168.1.3",
        9000,
    );
    settle(&mut e);
    let clients: Vec<_> = e
        .network()
        .transport
        .views()
        .into_iter()
        .filter(|v| v.device == id("host-b") && v.state == "ESTABLISHED")
        .collect();
    assert_eq!(clients.len(), 2);
    assert_ne!(clients[0].peer, clients[1].peer);
    assert!(
        e.network()
            .transport
            .views()
            .iter()
            .filter(|v| v.device != id("host-b"))
            .all(|v| (49152..=65535).contains(&v.local.port))
    );
    let text = "Bonjour · Hello · Español · Italiano · Português · Deutsch 🦀\n"
        .repeat(35)
        .into_bytes();
    send(&mut e, "send-a", "host-a", "client", &text);
    send(&mut e, "send-c", "host-c", "client-c", b"second client");
    settle(&mut e);
    let sa = peer_socket(&e, "host-b", "192.168.1.2");
    let sc = peer_socket(&e, "host-b", "192.168.1.4");
    assert_eq!(received(&e, "host-b", &sa), text);
    assert_eq!(received(&e, "host-b", &sc), b"second client");
    send(&mut e, "echo-a", "host-b", sa.as_str(), &text);
    send(&mut e, "echo-c", "host-b", sc.as_str(), b"second client");
    settle(&mut e);
    assert_eq!(received(&e, "host-a", &id("client")), text);
    assert_eq!(received(&e, "host-c", &id("client-c")), b"second client");
    let flags: Vec<_> = e
        .trace()
        .iter()
        .filter(|f| f.kind == "tcp.segment.sent")
        .take(3)
        .map(|f| f.payload["packet"]["flags"].clone())
        .collect();
    assert_eq!(flags.len(), 3);
    assert_eq!(flags[0]["syn"], true);
    assert_eq!(flags[0]["ack"], false);
    assert_eq!(flags[1]["syn"], true);
    assert_eq!(flags[1]["ack"], true);
    assert_eq!(flags[2]["syn"], false);
    assert_eq!(flags[2]["ack"], true);
}
#[test]
fn loss_of_data_and_ack_retransmits_without_duplicate_application_bytes() {
    for ack_loss in [false, true] {
        let mut e = pair();
        let server = peer_socket(&e, "host-b", "192.168.1.2");
        let data = if ack_loss {
            vec![42; 200]
        } else {
            vec![71; 1700]
        };
        send(&mut e, "data", "host-a", "client", &data);
        if ack_loss {
            e.step().unwrap();
            e.step().unwrap();
        }
        loss(&mut e, "loss-on", true);
        e.step().unwrap();
        loss(&mut e, "loss-off", false);
        settle(&mut e);
        assert_eq!(received(&e, "host-b", &server), data);
        assert!(e.trace().iter().any(|f| f.kind == "tcp.retransmit"));
        if ack_loss {
            assert!(
                e.trace()
                    .iter()
                    .any(|f| f.kind == "tcp.segment.duplicate_or_gap")
            );
        }
    }
}
#[test]
fn final_handshake_ack_loss_does_not_reaccept_or_reconnect() {
    let mut e = engine();
    bind(
        &mut e,
        "listen",
        "host-b",
        "server",
        TransportProtocol::Tcp,
        9000,
    )
    .unwrap();
    connect(&mut e, "connect", "host-a", "client", "192.168.1.3", 9000);
    while !e
        .network()
        .transport
        .views()
        .iter()
        .any(|v| v.device == id("host-a") && v.state == "ESTABLISHED")
    {
        e.step().unwrap();
    }
    loss(&mut e, "loss-on", true);
    e.step().unwrap();
    loss(&mut e, "loss-off", false);
    settle(&mut e);
    assert_eq!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "socket.event" && f.payload["kind"]["event"] == "connected")
            .count(),
        1
    );
    assert_eq!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "socket.event" && f.payload["kind"]["event"] == "accepted")
            .count(),
        1
    );
}
#[test]
fn ports_conflicts_and_invalid_commands_are_atomic_then_owner_stop_frees_everything() {
    let mut e = pair();
    let before = e.trace_hash();
    let views = e.network().transport.views();
    assert_eq!(
        bind(
            &mut e,
            "conflict",
            "host-b",
            "collision",
            TransportProtocol::Tcp,
            9000
        )
        .unwrap_err()
        .code,
        "socket.port_in_use"
    );
    assert_eq!(e.trace_hash(), before);
    assert_eq!(e.network().transport.views(), views);
    assert_eq!(
        command(
            &mut e,
            "invalid",
            "host-a",
            SocketCommand::Send {
                socket: id("client"),
                data_base64: "invalid!".into()
            }
        )
        .unwrap_err()
        .code,
        "validation.invalid_base64"
    );
    assert_eq!(e.network().transport.views(), views);
    command(
        &mut e,
        "stop-server",
        "host-b",
        SocketCommand::CloseOwner {
            owner: id("server"),
        },
    )
    .unwrap();
    assert!(
        !e.network()
            .transport
            .views()
            .iter()
            .any(|v| v.device == id("host-b"))
    );
    bind(
        &mut e,
        "reinstall",
        "host-b",
        "new-server",
        TransportProtocol::Tcp,
        9000,
    )
    .unwrap();
    settle(&mut e);
    assert!(
        !e.network()
            .transport
            .views()
            .iter()
            .any(|v| v.device == id("host-a"))
    );
    bind(
        &mut e,
        "same-port-udp",
        "host-b",
        "udp",
        TransportProtocol::Udp,
        9000,
    )
    .unwrap();
}
#[test]
fn graceful_fin_ack_closes_sessions_and_timers_do_not_resurrect_them() {
    let mut e = pair();
    send(&mut e, "data", "host-a", "client", b"flush before FIN");
    command(
        &mut e,
        "close",
        "host-a",
        SocketCommand::Close {
            socket: id("client"),
            force: false,
        },
    )
    .unwrap();
    e.run(10000, None).unwrap();
    assert!(
        e.network()
            .transport
            .views()
            .iter()
            .all(|v| v.state == "LISTEN")
    );
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "tcp.segment.sent" && f.payload["packet"]["flags"]["fin"] == true)
    );
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "tcp.state" && f.payload["state"] == "TIME_WAIT")
    );
    assert_eq!(e.pending(), 0);
}
#[test]
fn mute_peer_times_out_boundedly_and_loopback_never_uses_ethernet() {
    let mut e = engine();
    connect(
        &mut e,
        "mute",
        "host-a",
        "mute-client",
        "192.168.1.99",
        9000,
    );
    e.run(1000, None).unwrap();
    assert!(e.network().transport.views().is_empty());
    assert_eq!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "tcp.retransmit")
            .count(),
        4
    );
    assert!(
        e.trace()
            .iter()
            .any(|f| f.payload.get("reason") == Some(&serde_json::json!("tcp.timeout")))
    );
    let mut p = project();
    p.links.clear();
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    bind(
        &mut e,
        "listen",
        "host-a",
        "local",
        TransportProtocol::Tcp,
        9000,
    )
    .unwrap();
    connect(&mut e, "connect", "host-a", "client", "127.0.0.1", 9000);
    settle(&mut e);
    assert_eq!(
        e.network()
            .transport
            .views()
            .iter()
            .filter(|v| v.state == "ESTABLISHED")
            .count(),
        2
    );
    assert!(!e.trace().iter().any(|f| f.kind == "ethernet.sent"));
}
#[test]
fn udp_lost_datagram_has_no_transport_retry_and_broadcast_stays_on_its_lan() {
    let mut p = project();
    p.links.retain(|l| l.id != id("cable-host-c"));
    p.devices[0].interfaces[0].ipv4 = None;
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    for device in ["host-b", "host-c"] {
        bind(
            &mut e,
            &format!("bind-{device}"),
            device,
            "dhcp-like",
            TransportProtocol::Udp,
            67,
        )
        .unwrap();
    }
    command(
        &mut e,
        "bind-source",
        "host-a",
        SocketCommand::Bind {
            socket: id("sender"),
            owner: id("sender"),
            protocol: TransportProtocol::Udp,
            interface: Some(id("host-a-p0")),
            address: None,
            port: 68,
        },
    )
    .unwrap();
    command(
        &mut e,
        "broadcast",
        "host-a",
        SocketCommand::SendTo {
            socket: id("sender"),
            destination: Ipv4Addr::BROADCAST,
            port: 67,
            data_base64: STANDARD.encode([1, 2, 3, 4]),
        },
    )
    .unwrap();
    settle(&mut e);
    let datagrams: Vec<_> = e
        .trace()
        .iter()
        .filter(|f| f.kind == "socket.event" && f.payload["kind"]["event"] == "datagram")
        .collect();
    assert_eq!(datagrams.len(), 1);
    assert_eq!(datagrams[0].device_id, id("host-b"));
    assert_eq!(datagrams[0].payload["kind"]["source"]["address"], "0.0.0.0");
    loss(&mut e, "loss-on", true);
    command(
        &mut e,
        "lost-broadcast",
        "host-a",
        SocketCommand::SendTo {
            socket: id("sender"),
            destination: Ipv4Addr::BROADCAST,
            port: 67,
            data_base64: STANDARD.encode([5, 6]),
        },
    )
    .unwrap();
    e.step().unwrap();
    loss(&mut e, "loss-off", false);
    e.run(10000, None).unwrap();
    assert_eq!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "udp.datagram.sent")
            .count(),
        2
    );
    assert!(!e.trace().iter().any(|f| f.kind == "tcp.retransmit"));
}
#[test]
fn transport_step_and_run_preserve_canonical_order_for_fixed_input() {
    let mut a = engine();
    let mut b = engine();
    for e in [&mut a, &mut b] {
        bind(
            e,
            "listen",
            "host-b",
            "server",
            TransportProtocol::Tcp,
            9000,
        )
        .unwrap();
        connect(e, "connect", "host-a", "client", "192.168.1.3", 9000);
    }
    a.run(1000, Some(100000)).unwrap();
    while b.now() < 100000 && b.pending() > 0 {
        if b.network()
            .transport
            .views()
            .iter()
            .filter(|v| v.state == "ESTABLISHED")
            .count()
            == 2
        {
            break;
        }
        b.step().unwrap();
    }
    for e in [&mut a, &mut b] {
        send(e, "send", "host-a", "client", b"stable");
    }
    a.run(1000, None).unwrap();
    while b.step().unwrap() {}
    assert_eq!(a.canonical_trace(), b.canonical_trace());
}
#[test]
fn udp_interface_scopes_and_closed_port_are_explicit() {
    let mut p = project();
    let mut nic = p.devices[0].interfaces[0].clone();
    nic.id = id("a-extra");
    nic.mac = "02:00:00:00:07:01".into();
    nic.ipv4 = Some(Ipv4Config::Static {
        address: "10.0.0.2".into(),
        prefix_length: 24,
    });
    p.devices[0].interfaces.push(nic);
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    for (name, interface) in [("scope-one", "host-a-p0"), ("scope-two", "a-extra")] {
        command(
            &mut e,
            name,
            "host-a",
            SocketCommand::Bind {
                socket: id(name),
                owner: id(name),
                protocol: TransportProtocol::Udp,
                interface: Some(id(interface)),
                address: None,
                port: 7000,
            },
        )
        .unwrap();
    }
    let before = e.trace_hash();
    assert_eq!(
        bind(
            &mut e,
            "wildcard",
            "host-a",
            "wildcard",
            TransportProtocol::Udp,
            7000
        )
        .unwrap_err()
        .code,
        "socket.port_in_use"
    );
    assert_eq!(e.trace_hash(), before);
    command(
        &mut e,
        "unbound-destination",
        "host-a",
        SocketCommand::SendTo {
            socket: id("scope-one"),
            destination: "192.168.1.3".parse().unwrap(),
            port: 9999,
            data_base64: STANDARD.encode("no listener"),
        },
    )
    .unwrap();
    settle(&mut e);
    assert!(
        e.trace().iter().any(|f| f.kind == "udp.datagram.dropped"
            && f.payload["reason"] == "socket.port_unreachable")
    );
}
#[test]
fn transport_trace_quota_and_invalid_broadcast_connect_do_not_reserve_ports() {
    let mut e = Engine::from_project(
        project(),
        42,
        Limits {
            trace: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        bind(
            &mut e,
            "quota",
            "host-b",
            "server",
            TransportProtocol::Tcp,
            9000
        )
        .unwrap_err()
        .code,
        "resource.limit_exceeded"
    );
    assert!(e.network().transport.views().is_empty());
    assert!(e.trace().is_empty());
    assert_eq!(e.pending(), 0);
    let mut e = engine();
    let before = e.trace_hash();
    let error = command(
        &mut e,
        "broadcast-connect",
        "host-a",
        SocketCommand::Connect {
            socket: id("client"),
            owner: id("client"),
            destination: "192.168.1.255".parse().unwrap(),
            port: 9000,
            local_address: None,
            local_port: None,
        },
    )
    .unwrap_err();
    assert_eq!(error.code, "socket.invalid_endpoint");
    assert!(e.network().transport.views().is_empty());
    assert_eq!(e.trace_hash(), before);
}
