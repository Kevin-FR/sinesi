use serde_json::Value;
use sns_engine::{Engine, Limits};
use sns_model::{CommandEnvelope, Diagnostic};

#[test]
fn response_wait_recording_and_checkpoint_versions_agree_and_old_data_remains_readable() {
    use sns_model::WaitFactor;
    let p = sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    let e = Engine::from_project_with_wait_factor(
        p,
        42,
        Limits::default(),
        WaitFactor::try_from(3).unwrap(),
    )
    .unwrap();
    let recording = serde_json::to_value(e.recording().unwrap()).unwrap();
    let checkpoint = serde_json::to_value(e.checkpoint().unwrap()).unwrap();
    schema("recording").validate(&recording).unwrap();
    schema("checkpoint").validate(&checkpoint).unwrap();
    let old: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/valid/checkpoint-v1-unconfigured.json"
    ))
    .unwrap();
    schema("checkpoint").validate(&old).unwrap();
    for (pointer, value) in [
        ("/recording_version", serde_json::json!(1)),
        (
            "/engine_profile",
            serde_json::json!("sns-engine-semantics-v1"),
        ),
        ("/wait_factor", serde_json::json!(0)),
        ("/wait_factor", serde_json::json!(6)),
    ] {
        let mut bad = recording.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(!schema("recording").is_valid(&bad));
        assert!(
            sns_engine::Recording::from_bytes(&serde_json::to_vec(&bad).unwrap())
                .and_then(|r| r.replay())
                .is_err()
        );
    }
    for pointer in ["/checkpoint_version", "/recording/recording_version"] {
        let mut bad = checkpoint.clone();
        *bad.pointer_mut(pointer).unwrap() = serde_json::json!(1);
        assert!(!schema("checkpoint").is_valid(&bad));
        assert!(
            sns_engine::Checkpoint::from_bytes(&serde_json::to_vec(&bad).unwrap())
                .unwrap()
                .restore()
                .is_err()
        );
    }
    let mut bad = checkpoint;
    bad["recording"]["wait_factor"] = serde_json::json!(4);
    assert!(
        sns_engine::Checkpoint::from_bytes(&serde_json::to_vec(&bad).unwrap())
            .unwrap()
            .restore()
            .is_err()
    );
    let mut integral = recording;
    integral["wait_factor"] = serde_json::json!(3.0);
    schema("recording").validate(&integral).unwrap();
    sns_engine::Recording::from_bytes(&serde_json::to_vec(&integral).unwrap())
        .unwrap()
        .replay()
        .unwrap();
    let scenario: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/valid/response-wait-scenario.json"
    ))
    .unwrap();
    schema("network-scenario").validate(&scenario).unwrap();
}

fn schema(name: &str) -> jsonschema::Validator {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../contracts/{name}.schema.json"));
    let value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    jsonschema::options()
        .should_validate_formats(true)
        .build(&value)
        .unwrap()
}

#[test]
fn recovery_session_contract_is_versioned_bounded_and_validates_project_and_selection() {
    use sns_project::session::SessionRecord;
    let fixture: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/session.json")).unwrap();
    let contract = schema("session");
    contract.validate(&fixture).unwrap();
    let record: SessionRecord = serde_json::from_value(fixture.clone()).unwrap();
    record.validate().unwrap();
    for (pointer, value) in [
        ("/session_version", serde_json::json!(99)),
        ("/view/zoom", serde_json::json!(0)),
        ("/view/pan", serde_json::json!([0, "NaN"])),
        ("/view/selected", serde_json::json!(["host-a", "host-a"])),
    ] {
        let mut invalid = fixture.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert!(!contract.is_valid(&invalid), "{pointer}");
        assert!(
            serde_json::from_value::<SessionRecord>(invalid)
                .map(|r| r.validate().is_err())
                .unwrap_or(true)
        );
    }
    let mut foreign = fixture.clone();
    foreign["view"]["selected"] = serde_json::json!(["unknown-device"]);
    assert!(
        serde_json::from_value::<SessionRecord>(foreign)
            .unwrap()
            .validate()
            .is_err()
    );
    let mut extension = fixture;
    extension["execute"] = serde_json::json!("anything");
    assert!(!contract.is_valid(&extension));
    assert!(serde_json::from_value::<SessionRecord>(extension).is_err());
}
#[test]
fn documentation_project_round_trips_styles_and_remains_valid_in_session_and_checkpoint() {
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/annotations.json"))
            .unwrap();
    let value = serde_json::to_value(&project).unwrap();
    let contract = schema("project-documentation");
    contract.validate(&value).unwrap();
    assert!(!schema("project-presentation").is_valid(&value));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Cours réseau.json");
    sns_project::save_project(&path, &project).unwrap();
    assert_eq!(sns_project::load_project(&path).unwrap(), project);
    for (pointer, bad) in [
        ("/annotations/0/style/width", serde_json::json!(23)),
        ("/annotations/0/style/font_size", serde_json::json!(97)),
        (
            "/annotations/0/style/fill_color",
            serde_json::json!([0, 0, 256, 255]),
        ),
        ("/annotations/0/style/kind", serde_json::json!("script")),
        ("/schema_version", serde_json::json!(8)),
    ] {
        let mut invalid = value.clone();
        *invalid.pointer_mut(pointer).unwrap() = bad;
        assert!(!contract.is_valid(&invalid), "{pointer}");
        assert!(
            sns_project::parse_project(&serde_json::to_vec(&invalid).unwrap()).is_err(),
            "{pointer}"
        );
    }
    let mut session: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/session.json")).unwrap();
    session["project"] = value;
    session["view"]["selected"] = serde_json::json!(["zone-lan"]);
    schema("session").validate(&session).unwrap();
    serde_json::from_value::<sns_project::session::SessionRecord>(session)
        .unwrap()
        .validate()
        .unwrap();
    let engine = Engine::from_project(project.clone(), 42, Limits::default()).unwrap();
    let checkpoint = engine.checkpoint().unwrap();
    schema("checkpoint")
        .validate(&serde_json::to_value(&checkpoint).unwrap())
        .unwrap();
    assert_eq!(checkpoint.restore().unwrap().project(), &project);
}
#[test]
fn neutral_schemas_accept_actual_serialized_contracts() {
    let scenario =
        sns_project::parse_scenario(include_bytes!("../../../fixtures/valid/scenario.json"))
            .unwrap();
    schema("scenario")
        .validate(&serde_json::to_value(&scenario).unwrap())
        .unwrap();
    schema("project")
        .validate(&serde_json::to_value(&scenario.project).unwrap())
        .unwrap();
    let command: CommandEnvelope =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/command.json")).unwrap();
    schema("command")
        .validate(&serde_json::to_value(command).unwrap())
        .unwrap();
    let diagnostic: Diagnostic =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/error.json")).unwrap();
    schema("error")
        .validate(&serde_json::to_value(diagnostic).unwrap())
        .unwrap();
    let mut engine = Engine::new(scenario, None, Limits::default()).unwrap();
    engine.run(100, None).unwrap();
    let event_schema = schema("event");
    for event in engine.trace() {
        event_schema
            .validate(&serde_json::to_value(event).unwrap())
            .unwrap();
    }
}
#[test]
fn schemas_reject_unknown_properties_versions_and_invalid_counters() {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/command.json")).unwrap();
    let schema = schema("command");
    for invalid in [
        serde_json::json!(42),
        serde_json::json!("01"),
        serde_json::json!("18446744073709551616"),
    ] {
        value["expected_revision"] = invalid;
        assert!(!schema.is_valid(&value));
    }
    value["expected_revision"] = serde_json::json!("18446744073709551615");
    assert!(schema.is_valid(&value));
    value["api_version"] = serde_json::json!(99);
    assert!(!schema.is_valid(&value));
    value["api_version"] = serde_json::json!(1);
    value["execute_java"] = serde_json::json!(true);
    assert!(!schema.is_valid(&value));
}
#[test]
fn semantic_reference_checks_complement_the_structural_schema() {
    for bytes in [
        include_bytes!("../../../fixtures/invalid/dangling-link.json").as_slice(),
        include_bytes!("../../../fixtures/invalid/duplicate-device.json").as_slice(),
    ] {
        let value: Value = serde_json::from_slice(bytes).unwrap();
        assert!(schema("project").is_valid(&value));
        assert!(sns_project::parse_project(bytes).is_err());
    }
    let value: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/invalid/unknown-version.json"
    ))
    .unwrap();
    assert!(!schema("project").is_valid(&value));
}
#[test]
fn lan_v2_round_trip_commands_and_actual_frames_match_neutral_contracts() {
    use sns_model::*;
    let mut project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    project.devices[0].applications.push(Application {
        data: None,
        service: None,
        id: Id::new("app-preserved").unwrap(),
        kind: "terminal".into(),
        config: Default::default(),
    });
    project.devices[0].files.push(VirtualFile {
        path: "/exercice.txt".into(),
        media_type: "text/plain".into(),
        data_base64: "Qm9uam91cg==".into(),
    });
    project.annotations.push(Annotation {
        style: None,
        id: Id::new("note-preserved").unwrap(),
        text: "Exercice".into(),
        x: 20.,
        y: 40.,
    });
    schema("project-lan")
        .validate(&serde_json::to_value(&project).unwrap())
        .unwrap();
    assert!(!schema("project").is_valid(&serde_json::to_value(&project).unwrap()));
    let mut e = Engine::from_project(project.clone(), 42, Limits::default()).unwrap();
    let envelope = CommandEnvelope {
        api_version: 1,
        command_id: Id::new("ping").unwrap(),
        project_id: project.project_id.clone(),
        expected_revision: Decimal(0),
        run_id: e.run_id().clone(),
        command: Command::Ping {
            source: Id::new("host-a").unwrap(),
            destination: "192.168.1.3".into(),
            ttl: 64,
        },
    };
    schema("command-lan")
        .validate(&serde_json::to_value(&envelope).unwrap())
        .unwrap();
    e.command(envelope).unwrap();
    e.run(1000, None).unwrap();
    let validator = schema("event-lan");
    for event in e.trace() {
        validator
            .validate(&serde_json::to_value(event).unwrap())
            .unwrap();
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reseau.sinesi.json");
    sns_project::save_project(&path, &project).unwrap();
    assert_eq!(sns_project::load_project(&path).unwrap(), project);
}
#[test]
fn actual_tcp_udp_commands_and_transport_frames_validate_as_portable_contracts() {
    use sns_model::*;
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    let mut e = Engine::from_project(project, 42, Limits::default()).unwrap();
    let command_schema = schema("transport-command");
    for (name, device, command) in [
        (
            "listen",
            "host-b",
            SocketCommand::Bind {
                socket: Id::new("server").unwrap(),
                owner: Id::new("server").unwrap(),
                protocol: TransportProtocol::Tcp,
                interface: None,
                address: None,
                port: 9000,
            },
        ),
        (
            "connect",
            "host-a",
            SocketCommand::Connect {
                socket: Id::new("client").unwrap(),
                owner: Id::new("client").unwrap(),
                destination: "192.168.1.3".parse().unwrap(),
                port: 9000,
                local_address: None,
                local_port: None,
            },
        ),
        (
            "udp-server",
            "host-b",
            SocketCommand::Bind {
                socket: Id::new("udp-server").unwrap(),
                owner: Id::new("udp-server").unwrap(),
                protocol: TransportProtocol::Udp,
                interface: Some(Id::new("host-b-p0").unwrap()),
                address: None,
                port: 9001,
            },
        ),
        (
            "udp-client",
            "host-a",
            SocketCommand::Bind {
                socket: Id::new("udp-client").unwrap(),
                owner: Id::new("udp-client").unwrap(),
                protocol: TransportProtocol::Udp,
                interface: None,
                address: None,
                port: 0,
            },
        ),
        (
            "udp-send",
            "host-a",
            SocketCommand::SendTo {
                socket: Id::new("udp-client").unwrap(),
                destination: "192.168.1.3".parse().unwrap(),
                port: 9001,
                data_base64: "AAEC/w==".into(),
            },
        ),
    ] {
        let envelope = CommandEnvelope {
            api_version: 1,
            command_id: Id::new(name).unwrap(),
            project_id: e.project().project_id.clone(),
            expected_revision: Decimal(0),
            run_id: e.run_id().clone(),
            command: Command::Socket {
                device: Id::new(device).unwrap(),
                command,
            },
        };
        command_schema
            .validate(&serde_json::to_value(&envelope).unwrap())
            .unwrap();
        e.command(envelope).unwrap();
    }
    e.run(1000, Some(100000)).unwrap();
    let event_schema = schema("transport-event");
    for event in e.trace().iter().filter(|f| f.event_schema_version == 3) {
        event_schema
            .validate(&serde_json::to_value(event).unwrap())
            .unwrap();
    }
    let mut bad = serde_json::to_value(
        e.trace()
            .iter()
            .find(|f| f.event_schema_version == 3 && f.payload.contains_key("packet"))
            .unwrap(),
    )
    .unwrap();
    bad["payload"]["packet"]["sequence"] = serde_json::json!(4294967296_u64);
    assert!(!event_schema.is_valid(&bad));
}

#[test]
fn hosts_v3_app_commands_and_events_are_portable_and_strict() {
    use sns_model::*;
    let id = |s: &str| Id::new(s).unwrap();
    let mut e = Engine::from_project(
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/lan.json")).unwrap(),
        42,
        Limits::default(),
    )
    .unwrap();
    let commands = [
        AppCommand::Install {
            application: id("term"),
            software: Software::Terminal,
        },
        AppCommand::OpenView {
            application: id("term"),
        },
        AppCommand::Terminal {
            application: id("term"),
            line: "mkdir docs".into(),
        },
        AppCommand::Terminal {
            application: id("term"),
            line: "ping -c 1 -b 192.168.1.255".into(),
        },
    ];
    for (n, command) in commands.into_iter().enumerate() {
        let request = CommandEnvelope {
            api_version: 1,
            command_id: id(&format!("command-{n}")),
            project_id: e.project().project_id.clone(),
            expected_revision: Decimal(e.revision()),
            run_id: e.run_id().clone(),
            command: Command::App {
                device: id("host-a"),
                command,
            },
        };
        let value = serde_json::to_value(&request).unwrap();
        schema("application-command").validate(&value).unwrap();
        let mut bad = value.clone();
        bad["command"]["payload"]["command"]["execute_java"] = true.into();
        assert!(!schema("application-command").is_valid(&bad));
        assert!(serde_json::from_value::<CommandEnvelope>(bad).is_err());
        e.command(request).unwrap();
    }
    e.run(10000, Some(10_000_000)).unwrap();
    schema("project-hosts")
        .validate(&serde_json::to_value(e.project()).unwrap())
        .unwrap();
    assert!(!schema("project-lan").is_valid(&serde_json::to_value(e.project()).unwrap()));
    for event in e.trace() {
        let contract = match event.event_schema_version {
            2 => "event-lan",
            3 => "transport-event",
            4 => "application-event",
            _ => panic!("unexpected version"),
        };
        schema(contract)
            .validate(&serde_json::to_value(event).unwrap())
            .unwrap();
    }
}

#[test]
fn routed_projects_and_actual_rip_forwarding_events_match_versioned_contracts() {
    use sns_model::*;
    let id = |s: &str| Id::new(s).unwrap();
    let mut project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/rip.json")).unwrap();
    project.lan.insert(
        id("a"),
        LanConfig {
            symbol: Some(DeviceSymbol::Server),
            ..Default::default()
        },
    );
    schema("project-routing")
        .validate(&serde_json::to_value(&project).unwrap())
        .unwrap();
    assert!(!schema("project-hosts").is_valid(&serde_json::to_value(&project).unwrap()));
    let mut e = Engine::from_project(project, 42, Limits::default()).unwrap();
    e.run(10000, Some(2_000_000)).unwrap();
    let request = CommandEnvelope {
        api_version: 1,
        command_id: id("mute"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(0),
        command: Command::RipMute {
            device: id("r3"),
            muted: true,
        },
    };
    schema("routing-command")
        .validate(&serde_json::to_value(&request).unwrap())
        .unwrap();
    e.command(request).unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("ping"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(0),
        command: Command::Ping {
            source: id("a"),
            destination: "192.168.30.2".into(),
            ttl: 64,
        },
    })
    .unwrap();
    e.run(10000, Some(5_000_000)).unwrap();
    for event in e.trace() {
        let name = match event.event_schema_version {
            2 => "event-lan",
            3 => "transport-event",
            4 => "application-event",
            5 => "routing-event",
            _ => panic!("unexpected schema"),
        };
        schema(name)
            .validate(&serde_json::to_value(event).unwrap())
            .unwrap();
    }
    let mut invalid = serde_json::to_value(
        e.trace()
            .iter()
            .find(|f| f.kind == "ipv4.forwarded")
            .unwrap(),
    )
    .unwrap();
    invalid["payload"]["ttl"] = 300.into();
    assert!(!schema("routing-event").is_valid(&invalid));
}
#[test]
fn service_profiles_commands_and_actual_trace_roundtrip_with_neutral_schemas() {
    use sns_model::*;
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/services.json"))
            .unwrap();
    let value = serde_json::to_value(&project).unwrap();
    schema("project-services").validate(&value).unwrap();
    let mut invalid = value.clone();
    invalid["devices"][1]["applications"][0]["service"]["settings"]["records"][0]["ttl_seconds"] =
        (-1).into();
    assert!(!schema("project-services").is_valid(&invalid));
    let mut engine = Engine::from_project(project, 42, Limits::default()).unwrap();
    let id = |s: &str| Id::new(s).unwrap();
    let commands = [
        (
            "host-b",
            AppCommand::ConfigureService {
                application: id("dns"),
                config: engine.project().devices[1].applications[0]
                    .service
                    .clone()
                    .unwrap(),
            },
        ),
        (
            "host-b",
            AppCommand::StartServer {
                application: id("dns"),
                port: 53,
            },
        ),
        (
            "host-b",
            AppCommand::StartServer {
                application: id("web"),
                port: 80,
            },
        ),
        (
            "host-a",
            AppCommand::PurgeDns {
                application: id("host-a-browser"),
            },
        ),
    ];
    for (n, (host, command)) in commands.into_iter().enumerate() {
        let request = CommandEnvelope {
            api_version: 1,
            command_id: id(&format!("service-contract-{n}")),
            project_id: engine.project().project_id.clone(),
            run_id: engine.run_id().clone(),
            expected_revision: Decimal(engine.revision()),
            command: Command::App {
                device: id(host),
                command,
            },
        };
        schema("services-command")
            .validate(&serde_json::to_value(&request).unwrap())
            .unwrap();
        engine.command(request).unwrap();
    }
    engine.run(10000, Some(1_000_000)).unwrap();
    let request = CommandEnvelope {
        api_version: 1,
        command_id: id("service-navigation"),
        project_id: engine.project().project_id.clone(),
        run_id: engine.run_id().clone(),
        expected_revision: Decimal(engine.revision()),
        command: Command::App {
            device: id("host-a"),
            command: AppCommand::Navigate {
                application: id("host-a-browser"),
                url: "http://www.ecole.test/".into(),
                fields: None,
            },
        },
    };
    schema("services-command")
        .validate(&serde_json::to_value(&request).unwrap())
        .unwrap();
    engine.command(request).unwrap();
    engine.run(10000, Some(3_000_000)).unwrap();
    let mut service_events = 0;
    for event in engine.trace() {
        let name = match event.event_schema_version {
            2 => "event-lan",
            3 => "transport-event",
            4 => "application-event",
            5 => "routing-event",
            6 => {
                service_events += 1;
                "services-event"
            }
            _ => panic!("unexpected version"),
        };
        schema(name)
            .validate(&serde_json::to_value(event).unwrap())
            .unwrap();
    }
    assert!(service_events >= 18);
    assert_eq!(
        sns_project::parse_project(&serde_json::to_vec(engine.project()).unwrap()).unwrap(),
        *engine.project()
    );
}
#[test]
fn private_network_profiles_commands_nat_and_quoted_icmp_match_neutral_contracts() {
    use sns_model::*;
    let id = |s: &str| Id::new(s).unwrap();
    let project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/private.json")).unwrap();
    schema("project-private")
        .validate(&serde_json::to_value(&project).unwrap())
        .unwrap();
    let mut e = Engine::from_project(project, 42, Limits::default()).unwrap();
    for (i, command) in [
        Command::Security {
            device: id("gateway"),
            command: SecurityCommand::ConfigureFirewall {
                config: FirewallConfig {
                    enabled: false,
                    ..Default::default()
                },
            },
        },
        Command::Security {
            device: id("gateway"),
            command: SecurityCommand::ConfigureGateway {
                config: *e
                    .project()
                    .device(&id("gateway"))
                    .unwrap()
                    .network
                    .gateway
                    .clone()
                    .unwrap(),
            },
        },
        Command::Security {
            device: id("gateway"),
            command: SecurityCommand::PurgeNat,
        },
    ]
    .into_iter()
    .enumerate()
    {
        let r = CommandEnvelope {
            api_version: 1,
            command_id: id(&format!("private-contract-{i}")),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command,
        };
        schema("security-command")
            .validate(&serde_json::to_value(&r).unwrap())
            .unwrap();
        e.command(r).unwrap();
    }
    let r = CommandEnvelope {
        api_version: 1,
        command_id: id("private-probe"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::Ping {
            source: id("host-a"),
            destination: "198.51.100.10".into(),
            ttl: 2,
        },
    };
    e.command(r).unwrap();
    e.run(10000, Some(4_000_000)).unwrap();
    let mut translations = 0;
    for event in e.trace() {
        let name = match event.event_schema_version {
            2 => "event-lan",
            3 => "transport-event",
            4 => "application-event",
            5 => "routing-event",
            6 => "services-event",
            7 => {
                translations += 1;
                "private-event"
            }
            _ => panic!("unexpected version"),
        };
        let value = serde_json::to_value(event).unwrap();
        if let Err(err) = schema(name).validate(&value) {
            panic!("{} {value}: {err}", event.kind);
        }
    }
    assert!(translations >= 6);
    let mut invalid = serde_json::to_value(e.project()).unwrap();
    invalid["devices"][3]["network"]["gateway"]["port_start"] = 0.into();
    assert!(!schema("project-private").is_valid(&invalid));
    let saved = serde_json::to_vec(e.project()).unwrap();
    let reopened = sns_project::parse_project(&saved).unwrap();
    assert_eq!(reopened, *e.project());
}
#[test]
fn j6_projects_commands_durable_mail_and_advanced_events_match_contracts() {
    use sns_model::*;
    let id = |s: &str| Id::new(s).unwrap();
    for bytes in [
        include_bytes!("../../../fixtures/valid/mail.json").as_slice(),
        include_bytes!("../../../fixtures/valid/p2p.json").as_slice(),
        include_bytes!("../../../fixtures/valid/wireless.json").as_slice(),
    ] {
        let p = sns_project::parse_project(bytes).unwrap();
        schema("project-advanced")
            .validate(&serde_json::to_value(p).unwrap())
            .unwrap();
    }
    let mut e = Engine::from_project(
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/mail.json")).unwrap(),
        42,
        Limits::default(),
    )
    .unwrap();
    let commands = [
        (
            "host-b",
            AppCommand::Mail {
                application: id("alpha-server"),
                command: MailCommand::Start,
            },
        ),
        (
            "host-a",
            AppCommand::Mail {
                application: id("alice-client"),
                command: MailCommand::Send {
                    draft: MailDraft {
                        to: vec!["alice@alpha.test".into()],
                        cc: vec![],
                        bcc: vec![],
                        subject: "Contrat".into(),
                        body: "Bonjour".into(),
                    },
                },
            },
        ),
        (
            "host-a",
            AppCommand::Mail {
                application: id("alice-client"),
                command: MailCommand::Receive,
            },
        ),
    ];
    for (i, (host, command)) in commands.into_iter().enumerate() {
        let envelope = CommandEnvelope {
            api_version: 1,
            command_id: id(&format!("j6-contract-{i}")),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::App {
                device: id(host),
                command,
            },
        };
        schema("advanced-command")
            .validate(&serde_json::to_value(&envelope).unwrap())
            .unwrap();
        e.command(envelope).unwrap();
        e.run(10000, Some(e.now() + 1_000_000)).unwrap();
    }
    let saved = serde_json::to_value(e.project()).unwrap();
    schema("project-advanced").validate(&saved).unwrap();
    let event_schema = schema("advanced-event");
    for f in e.trace().iter().filter(|f| f.event_schema_version == 8) {
        event_schema
            .validate(&serde_json::to_value(f).unwrap())
            .unwrap();
    }
    let mut bad = saved;
    bad["devices"][0]["applications"][0]["data"]["data"]["inbox"][0]["bcc"] =
        serde_json::json!(["hidden@alpha.test"]);
    assert!(!schema("project-advanced").is_valid(&bad));
    for command in [
        P2pCommand::Join { peer: None },
        P2pCommand::Search {
            text: "cours".into(),
            ttl: 4,
        },
        P2pCommand::Download {
            result: id("result-1"),
            destination: "/cours.txt".into(),
        },
    ] {
        let envelope = CommandEnvelope {
            api_version: 1,
            command_id: id("p2p-contract"),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::App {
                device: id("host-a"),
                command: AppCommand::P2p {
                    application: id("peer"),
                    command,
                },
            },
        };
        schema("advanced-command")
            .validate(&serde_json::to_value(envelope).unwrap())
            .unwrap();
    }
}

#[test]
fn j7_scheduled_network_checkpoint_and_recording_follow_portable_contracts() {
    let plan = sns_project::parse_scenario(include_bytes!(
        "../../../fixtures/valid/network-scenario.json"
    ))
    .unwrap();
    schema("network-scenario")
        .validate(&serde_json::to_value(&plan).unwrap())
        .unwrap();
    let mut e = Engine::new(plan, None, Limits::default()).unwrap();
    e.run(1000, Some(1500)).unwrap();
    let point = e.checkpoint().unwrap();
    schema("checkpoint")
        .validate(&serde_json::to_value(&point).unwrap())
        .unwrap();
    schema("recording")
        .validate(&serde_json::to_value(e.recording().unwrap()).unwrap())
        .unwrap();
    let restored = point.restore().unwrap();
    schema("recording")
        .validate(&serde_json::to_value(restored.recording().unwrap()).unwrap())
        .unwrap();
    let mut bad = serde_json::to_value(&point).unwrap();
    bad["runtime_state"]["execute_code"] = true.into();
    assert!(!schema("checkpoint").is_valid(&bad));
    let mut bad = serde_json::to_value(e.recording().unwrap()).unwrap();
    bad["operations"] = serde_json::json!([{"operation":"events","count":"0"}]);
    assert!(!schema("recording").is_valid(&bad));
}

#[test]
fn j7_link_conditions_commands_events_scenarios_and_random_state_match_contracts() {
    use sns_model::*;
    let plan = sns_project::parse_scenario(include_bytes!(
        "../../../fixtures/valid/link-conditions-scenario.json"
    ))
    .unwrap();
    schema("network-scenario")
        .validate(&serde_json::to_value(&plan).unwrap())
        .unwrap();
    let mut e = Engine::new(plan, None, Limits::default()).unwrap();
    e.run(100000, Some(5_000_000)).unwrap();
    let point = e.checkpoint().unwrap();
    schema("checkpoint")
        .validate(&serde_json::to_value(&point).unwrap())
        .unwrap();
    schema("recording")
        .validate(&serde_json::to_value(e.recording().unwrap()).unwrap())
        .unwrap();
    let mut count = 0;
    for event in e.trace().iter().filter(|t| t.event_schema_version == 9) {
        schema("link-conditions-event")
            .validate(&serde_json::to_value(event).unwrap())
            .unwrap();
        count += 1;
    }
    assert!(count > 4);
    let command = CommandEnvelope {
        api_version: 1,
        command_id: Id::new("set-link").unwrap(),
        project_id: e.project().project_id.clone(),
        expected_revision: Decimal(e.revision()),
        run_id: e.run_id().clone(),
        command: Command::ConfigureLink {
            link: Id::new("link-ab").unwrap(),
            conditions: LinkConditions {
                delay_us: Decimal(1000),
                jitter_us: Decimal(1000),
                loss_ppm: 250000,
            },
        },
    };
    let mut value = serde_json::to_value(command).unwrap();
    let contract = schema("link-conditions-command");
    contract.validate(&value).unwrap();
    value["command"]["payload"]["conditions"]["loss_ppm"] = 1000001.into();
    assert!(!contract.is_valid(&value));
}

#[test]
fn j7_cloud_project_live_commands_events_and_checkpoint_match_presentation_contracts() {
    use sns_model::*;
    let p =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/cloud.json")).unwrap();
    schema("project-presentation")
        .validate(&serde_json::to_value(&p).unwrap())
        .unwrap();
    let mut old = p.clone();
    old.schema_version = 7;
    old.semantic_profile = "sns-educational-applications-v1".into();
    assert!(old.validate().is_err());
    let old_scenario = serde_json::json!({"scenario_schema_version":2,"project":old,"seed":"42","actions":[],"prototype_servers":[]});
    assert!(!schema("network-scenario").is_valid(&old_scenario));
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    let command = CommandEnvelope {
        api_version: 1,
        command_id: Id::new("normal-switch").unwrap(),
        project_id: e.project().project_id.clone(),
        expected_revision: Decimal(e.revision()),
        run_id: e.run_id().clone(),
        command: Command::SetSymbol {
            device: Id::new("switch").unwrap(),
            symbol: None,
        },
    };
    schema("presentation-command")
        .validate(&serde_json::to_value(&command).unwrap())
        .unwrap();
    e.command(command).unwrap();
    schema("presentation-event")
        .validate(&serde_json::to_value(e.trace().last().unwrap()).unwrap())
        .unwrap();
    let point = e.checkpoint().unwrap();
    schema("checkpoint")
        .validate(&serde_json::to_value(&point).unwrap())
        .unwrap();
    point.restore().unwrap();
}
