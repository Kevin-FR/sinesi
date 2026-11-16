//! J2 application recipes run through the real virtual-time Ethernet/transport stack.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sns_engine::{Engine, Limits};
use sns_model::*;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn engine() -> Engine {
    Engine::from_project(
        serde_json::from_str(include_str!("../../../fixtures/valid/lan.json")).unwrap(),
        42,
        Limits::default(),
    )
    .unwrap()
}
fn app(e: &mut Engine, host: &str, command: AppCommand) -> Result<(), Diagnostic> {
    let n = e.trace().len();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(&format!("app-{}-{n}", e.revision())),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::App {
            device: id(host),
            command,
        },
    })
}
fn install(e: &mut Engine, host: &str, name: &str, software: Software) {
    app(
        e,
        host,
        AppCommand::Install {
            application: id(name),
            software,
        },
    )
    .unwrap();
}
fn terminal(e: &mut Engine, line: &str) {
    app(
        e,
        "host-a",
        AppCommand::Terminal {
            application: id("term"),
            line: line.into(),
        },
    )
    .unwrap();
}
fn received(e: &Engine, host: &str, name: &str) -> Vec<u8> {
    STANDARD
        .decode(
            &e.apps()
                .views(e.project())
                .into_iter()
                .find(|h| h.device == id(host))
                .unwrap()
                .applications
                .into_iter()
                .find(|a| a.id == id(name))
                .unwrap()
                .received_base64,
        )
        .unwrap()
}
fn pump(e: &mut Engine) {
    e.run(10000, Some(e.now() + 2_000_000)).unwrap();
}
fn record(e: &Engine, name: &str) {
    if let Some(directory) = std::env::var_os("SNS_RECORD_J2") {
        let root = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(format!("j2-{name}.jsonl")), e.canonical_trace()).unwrap();
        std::fs::write(
            root.join(format!("j2-{name}-project.json")),
            serde_json::to_vec_pretty(e.project()).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn echo_two_clients_window_service_lifecycle_and_transactional_ports() {
    let mut e = engine();
    install(&mut e, "host-b", "server", Software::TcpServer);
    install(&mut e, "host-a", "client-a", Software::TcpClient);
    install(&mut e, "host-c", "client-c", Software::TcpClient);
    app(
        &mut e,
        "host-b",
        AppCommand::StartServer {
            application: id("server"),
            port: 7777,
        },
    )
    .unwrap();
    for (h, a) in [("host-a", "client-a"), ("host-c", "client-c")] {
        app(
            &mut e,
            h,
            AppCommand::Connect {
                application: id(a),
                target: "192.168.1.3".into(),
                port: 7777,
            },
        )
        .unwrap();
    }
    pump(&mut e);
    app(
        &mut e,
        "host-b",
        AppCommand::CloseView {
            application: id("server"),
        },
    )
    .unwrap();
    for (h, a, s) in [
        ("host-a", "client-a", "Bonjour électricité"),
        ("host-c", "client-c", "Hola mundo"),
    ] {
        app(
            &mut e,
            h,
            AppCommand::Send {
                application: id(a),
                text: s.into(),
            },
        )
        .unwrap();
    }
    pump(&mut e);
    assert_eq!(
        received(&e, "host-a", "client-a"),
        "Bonjour électricité".as_bytes()
    );
    assert_eq!(received(&e, "host-c", "client-c"), b"Hola mundo");
    assert!(e.trace().iter().any(|f| f.kind == "ethernet.sent"));
    assert!(
        e.network()
            .transport
            .views()
            .iter()
            .filter(|s| s.owner == id("server"))
            .count()
            >= 3
    );
    app(
        &mut e,
        "host-b",
        AppCommand::Stop {
            application: id("server"),
        },
    )
    .unwrap();
    pump(&mut e);
    assert!(
        !e.network()
            .transport
            .views()
            .iter()
            .any(|s| s.owner == id("server"))
    );
    app(
        &mut e,
        "host-b",
        AppCommand::StartServer {
            application: id("server"),
            port: 7777,
        },
    )
    .unwrap();
    app(
        &mut e,
        "host-b",
        AppCommand::Uninstall {
            application: id("server"),
        },
    )
    .unwrap();
    assert!(
        !e.network()
            .transport
            .views()
            .iter()
            .any(|s| s.owner == id("server"))
    );
    let p = e.project().clone();
    let r = e.revision();
    assert!(
        app(
            &mut e,
            "host-c",
            AppCommand::Install {
                application: id("client-a"),
                software: Software::Terminal
            }
        )
        .is_err()
    );
    assert_eq!(*e.project(), p);
    assert_eq!(e.revision(), r);
}
#[test]
fn virtual_files_terminal_aliases_roundtrip_and_unknown_commands_are_isolated() {
    let mut e = engine();
    install(&mut e, "host-a", "term", Software::Terminal);
    install(&mut e, "host-a", "editor", Software::TextEditor);
    for s in [
        "mkdir docs",
        "cd docs",
        "pwd",
        "touch empty",
        "copy empty copied",
        "move copied renamed",
        "ls",
        "dir",
    ] {
        terminal(&mut e, s);
    }
    app(
        &mut e,
        "host-a",
        AppCommand::File {
            application: id("editor"),
            command: FileCommand::Write {
                path: "/docs/index.html".into(),
                media_type: "text/html".into(),
                data_base64: STANDARD.encode("héllo <p>é</p>"),
                overwrite: false,
            },
        },
    )
    .unwrap();
    for s in [
        "cat index.html",
        "type index.html",
        "cp index.html a",
        "mv a b",
        "rm b",
        "del empty",
        "del renamed",
        "ipconfig",
        "netstat",
        "arp",
        "arp -a 192.168.1.3",
        "help ping",
    ] {
        terminal(&mut e, s);
    }
    let old = e.project().clone();
    terminal(&mut e, "unknown dangerous");
    terminal(&mut e, "cd absent");
    assert_eq!(*e.project(), old);
    assert_eq!(
        e.apps()
            .files(&id("host-a"))
            .unwrap()
            .read_text("/docs/index.html")
            .unwrap(),
        "héllo <p>é</p>"
    );
    assert!(!e.apps().files(&id("host-b")).unwrap().exists("/docs"));
    let p: Project = serde_json::from_slice(&serde_json::to_vec(e.project()).unwrap()).unwrap();
    p.validate().unwrap();
    let reopened = Engine::from_project(p, 42, Limits::default()).unwrap();
    assert_eq!(
        reopened
            .apps()
            .files(&id("host-a"))
            .unwrap()
            .read("/docs/index.html")
            .unwrap(),
        "héllo <p>é</p>".as_bytes()
    );
    for s in ["cls", "reset", "exit"] {
        terminal(&mut e, s);
    }
    assert!(
        !e.apps().views(e.project())[0]
            .applications
            .iter()
            .find(|a| a.id == id("term"))
            .unwrap()
            .visible
    );
    app(
        &mut e,
        "host-a",
        AppCommand::OpenView {
            application: id("term"),
        },
    )
    .unwrap();
    assert!(
        e.apps()
            .files(&id("host-a"))
            .unwrap()
            .exists("/docs/index.html")
    );
    record(&e, "files");
}
#[test]
fn ping_default_four_statistics_capture_interrupt_and_broadcast() {
    let mut e = engine();
    install(&mut e, "host-a", "term", Software::Terminal);
    terminal(&mut e, "ping 192.168.1.3");
    e.run(10000, Some(10_000_000)).unwrap();
    assert_eq!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "icmp.echo.requested")
            .count(),
        4
    );
    let view = e
        .apps()
        .views(e.project())
        .into_iter()
        .find(|h| h.device == id("host-a"))
        .unwrap()
        .applications
        .remove(0);
    assert!(!view.busy);
    assert!(
        serde_json::to_string(&view.lines)
            .unwrap()
            .contains("ping_summary")
    );
    terminal(&mut e, "tcpdump");
    let start = e.trace().len();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("capture-ping"),
        project_id: e.project().project_id.clone(),
        expected_revision: Decimal(e.revision()),
        run_id: e.run_id().clone(),
        command: Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.3".parse().unwrap(),
            ttl: 64,
        },
    })
    .unwrap();
    pump(&mut e);
    assert!(e.trace().len() > start);
    app(
        &mut e,
        "host-a",
        AppCommand::Interrupt {
            application: id("term"),
        },
    )
    .unwrap();
    terminal(&mut e, "ping -n 2 -b 192.168.1.255");
    e.run(10000, Some(e.now() + 10_000_000)).unwrap();
    assert!(
        e.trace()
            .iter()
            .any(|f| f.kind == "icmp.broadcast.completed")
    );
    terminal(&mut e, "ping -c 100 192.168.1.3");
    e.step().unwrap();
    app(
        &mut e,
        "host-a",
        AppCommand::Interrupt {
            application: id("term"),
        },
    )
    .unwrap();
    let start = e
        .trace()
        .iter()
        .filter(|f| f.kind == "icmp.echo.requested")
        .count();
    e.run(10000, Some(e.now() + 10_000_000)).unwrap();
    assert_eq!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "icmp.echo.requested")
            .count(),
        start
    );
    terminal(&mut e, "arp -d 192.168.1.3");
    assert!(
        !e.network()
            .arp
            .keys()
            .any(|(i, ip)| i.as_str() == "if-a" && ip.to_string() == "192.168.1.3")
    );
    terminal(&mut e, "arp -d");
}
#[test]
fn registry_all_thirteen_entries_are_available() {
    let mut e = engine();
    for s in Software::ALL {
        install(&mut e, "host-a", s.id(), s);
    }
    assert_eq!(
        e.project()
            .device(&id("host-a"))
            .unwrap()
            .applications
            .len(),
        13
    );
    assert!(
        app(
            &mut e,
            "host-a",
            AppCommand::OpenView {
                application: id("p2p")
            }
        )
        .is_ok()
    );
    assert!(
        app(
            &mut e,
            "host-a",
            AppCommand::OpenView {
                application: id("terminal")
            }
        )
        .is_ok()
    );
}

#[test]
fn echo_retransmission_of_segmented_unicode_is_delivered_once() {
    let mut e = engine();
    install(&mut e, "host-b", "server", Software::TcpServer);
    install(&mut e, "host-a", "client", Software::TcpClient);
    app(
        &mut e,
        "host-b",
        AppCommand::StartServer {
            application: id("server"),
            port: 7777,
        },
    )
    .unwrap();
    app(
        &mut e,
        "host-a",
        AppCommand::Connect {
            application: id("client"),
            target: "192.168.1.3".into(),
            port: 7777,
        },
    )
    .unwrap();
    pump(&mut e);
    let message = "Réseau ⚙ fiable — ".repeat(180);
    let loss = |e: &mut Engine, enabled, name: &str| {
        e.command(CommandEnvelope {
            api_version: 1,
            command_id: id(name),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::ManualLoss { enabled },
        })
        .unwrap();
    };
    loss(&mut e, true, "loss-on");
    app(
        &mut e,
        "host-a",
        AppCommand::Send {
            application: id("client"),
            text: message.clone(),
        },
    )
    .unwrap();
    e.run(10000, Some(e.now() + 100_000)).unwrap();
    assert!(received(&e, "host-a", "client").is_empty());
    loss(&mut e, false, "loss-off");
    pump(&mut e);
    assert_eq!(received(&e, "host-b", "server"), message.as_bytes());
    assert_eq!(received(&e, "host-a", "client"), message.as_bytes());
    assert!(e.trace().iter().any(|f| f.kind == "tcp.retransmit"));
    record(&e, "echo");
}
#[test]
fn failed_application_commands_keep_document_revision_and_runtime_atomic() {
    let mut e = engine();
    install(&mut e, "host-b", "server", Software::TcpServer);
    let command = CommandEnvelope {
        api_version: 1,
        command_id: id("reserve"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::Socket {
            device: id("host-b"),
            command: SocketCommand::Bind {
                socket: id("occupied"),
                owner: id("other"),
                protocol: TransportProtocol::Tcp,
                interface: None,
                address: None,
                port: 7777,
            },
        },
    };
    e.command(command).unwrap();
    let project = e.project().clone();
    let revision = e.revision();
    let trace = e.canonical_trace();
    assert!(
        app(
            &mut e,
            "host-b",
            AppCommand::StartServer {
                application: id("server"),
                port: 7777
            }
        )
        .is_err()
    );
    assert_eq!(*e.project(), project);
    assert_eq!(e.revision(), revision);
    assert_eq!(e.canonical_trace(), trace);
    assert!(
        !e.apps()
            .views(e.project())
            .into_iter()
            .find(|h| h.device == id("host-b"))
            .unwrap()
            .applications[0]
            .running
    );
    let stale = CommandEnvelope {
        api_version: 1,
        command_id: id("stale"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(0),
        command: Command::App {
            device: id("host-b"),
            command: AppCommand::Uninstall {
                application: id("server"),
            },
        },
    };
    assert_eq!(
        e.command(stale).unwrap_err().code,
        "state.revision_conflict"
    );
    assert_eq!(*e.project(), project);
}

#[test]
fn j7_historical_file_probe_is_diagnostic_only_and_student_test_commands_are_rejected() {
    let mut e = engine();
    install(&mut e, "host-a", "term", Software::Terminal);
    assert!(sns_apps::usage("test").is_none());
    assert!(
        !sns_apps::COMMAND_HELP
            .split(|c: char| !c.is_ascii_alphanumeric())
            .any(|w| w == "test")
    );
    for command in sns_apps::COMMAND_HELP
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
    {
        assert!(sns_apps::usage(command).is_some(), "{command}");
    }
    let files = e.apps().files(&id("host-a")).unwrap().clone();
    let network = e.checkpoint_state().unwrap()["network"].clone();
    for line in ["test", "test ignored", "help test"] {
        terminal(&mut e, line);
    }
    assert_eq!(e.apps().files(&id("host-a")).unwrap(), &files);
    assert_eq!(e.checkpoint_state().unwrap()["network"], network);
    assert!(
        e.trace()
            .iter()
            .all(|f| !f.kind.starts_with("filesystem.") && !f.kind.starts_with("ethernet."))
    );
    let view = e
        .apps()
        .views(e.project())
        .into_iter()
        .find(|h| h.device == id("host-a"))
        .unwrap()
        .applications
        .into_iter()
        .find(|a| a.id == id("term"))
        .unwrap();
    assert_eq!(view.lines.iter().filter(|line|matches!(line,sns_apps::Line::Error{diagnostic} if diagnostic.code=="terminal.unknown_command")).count(),3);
    // Developer probe: direct, isolated VFS data. It is never registered in the parser.
    let mut diagnostic = files.clone();
    diagnostic
        .write("/test.txt", b"blaaa", "text/plain", false)
        .unwrap();
    assert_eq!(diagnostic.read_text("/test.txt").unwrap(), "blaaa");
    assert!(!e.apps().files(&id("host-a")).unwrap().exists("/test.txt"));
    if let Some(dir) = std::env::var_os("SNS_RECORD_J7_TERMINAL") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("j7-terminal-diagnostic.json"),serde_json::to_vec_pretty(&serde_json::json!({"student_commands":sns_apps::COMMAND_HELP,"rejected":["test","test ignored","help test"],"unknown_command_errors":3,"diagnostic":{"path":"/test.txt","content":diagnostic.read_text("/test.txt").unwrap(),"isolated_clone":true},"student_file_created":false,"network_unchanged":true})).unwrap()).unwrap();
    }
}
