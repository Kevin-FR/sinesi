//! F-ADV-004: missions are validated, checked on real simulation traces and
//! explain unmet objectives. The engine runs here; the checks only read.
use sns_engine::{Engine, Limits};
use sns_missions::*;
use sns_model::*;
use std::collections::BTreeSet;

fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn mission(name: &str) -> Mission {
    builtin().into_iter().find(|m| m.id == name).unwrap()
}
fn send(e: &mut Engine, command: Command) {
    let n = e.trace().len();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(&format!("mission-{}-{n}", e.revision())),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command,
    })
    .unwrap();
}
fn run(project: &Project, command: Command) -> Vec<sns_engine::TraceEvent> {
    run_all(project, vec![command])
}
fn run_all(project: &Project, commands: Vec<Command>) -> Vec<sns_engine::TraceEvent> {
    let mut e = Engine::from_project(project.clone(), 42, Limits::default()).unwrap();
    for command in commands {
        send(&mut e, command);
        e.run(20_000, Some(e.now() + 1_000_000)).unwrap();
    }
    e.run(20_000, Some(e.now() + 10_000_000)).unwrap();
    e.trace().to_vec()
}
fn report_for(m: &Mission, p: &Project, trace: &[sns_engine::TraceEvent], hints: usize) -> Report {
    report(m, p, trace, &BTreeSet::new(), hints, "fr")
}

#[test]
fn builtin_missions_are_valid_ordered_and_translated() {
    let missions = builtin();
    assert_eq!(
        missions.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
        ["forgotten-cable", "forgotten-gateway", "broken-directory"]
    );
    for m in &missions {
        m.validate().unwrap();
        for text in [&m.title, &m.brief]
            .into_iter()
            .chain(m.hints.iter())
            .chain(m.objectives.iter().flat_map(|o| [&o.text, &o.explain]))
        {
            assert_eq!(text.len(), 6, "{}: six languages", m.id);
        }
        // The starting network is broken: no mission is complete before work.
        assert!(!report_for(m, &m.project, &[], 0).complete, "{}", m.id);
    }
}

#[test]
fn invalid_missions_are_refused() {
    let mut m = mission("forgotten-gateway");
    m.mission_version = 2;
    assert!(m.validate().is_err());
    let mut m = mission("forgotten-gateway");
    m.objectives[0].check = Check::Gateway {
        device: id("absent"),
        equals: "1.2.3.4".into(),
    };
    assert!(
        m.validate()
            .unwrap_err()
            .starts_with("mission.unknown_device")
    );
    let mut m = mission("forgotten-gateway");
    m.title.remove("fr");
    assert!(m.validate().is_err());
    let mut m = mission("forgotten-gateway");
    m.objectives[1].id = m.objectives[0].id.clone();
    assert!(m.validate().is_err());
    assert!(parse(br#"{"mission_version":1}"#).is_err());
}

#[test]
fn missing_gateway_is_explained_then_repaired_with_a_real_ping() {
    let m = mission("forgotten-gateway");
    let ping = |p: &Project| {
        run(
            p,
            Command::Ping {
                source: id("a"),
                destination: "192.168.30.2".into(),
                ttl: 64,
            },
        )
    };
    let broken = ping(&m.project);
    let before = report_for(&m, &m.project, &broken, 0);
    assert!(!before.complete);
    assert!(
        before
            .objectives
            .iter()
            .all(|o| !o.done && o.explanation.is_some())
    );
    assert!(broken.iter().any(|e| e.kind == "icmp.error"));

    let mut fixed = m.project.clone();
    fixed.devices[0].network.default_gateway = Some("192.168.10.1".into());
    let trace = ping(&fixed);
    let after = report_for(&m, &fixed, &trace, 1);
    assert!(after.complete, "{after:?}");
    assert_eq!(after.stars, Some(2));
    assert!(after.objectives.iter().all(|o| o.explanation.is_none()));
}

#[test]
fn repaired_dns_fault_is_validated_and_the_failure_is_explained() {
    let m = mission("broken-directory");
    let lookup = |p: &Project, start: bool| {
        let mut commands = vec![];
        if start {
            commands.push(Command::App {
                device: id("host-b"),
                command: AppCommand::StartServer {
                    application: id("dns"),
                    port: 53,
                },
            });
        }
        commands.push(Command::App {
            device: id("host-a"),
            command: AppCommand::Terminal {
                application: id("host-a-terminal"),
                line: "host www.ecole.test".into(),
            },
        });
        run_all(p, commands)
    };
    // Nothing repaired: every objective is unmet and explained.
    let broken = lookup(&m.project, false);
    assert!(broken.iter().any(|e| e.kind == "dns.query.sent"));
    let before = report_for(&m, &m.project, &broken, 0);
    assert!(
        before
            .objectives
            .iter()
            .all(|o| !o.done && o.explanation.is_some())
    );
    let resolve = before
        .objectives
        .iter()
        .find(|o| o.id == "resolve")
        .unwrap();
    assert!(
        resolve
            .explanation
            .as_deref()
            .unwrap()
            .contains("host www.ecole.test")
    );
    // Server started but the client still asks the wrong directory.
    let half = lookup(&m.project, true);
    let partial = report_for(&m, &m.project, &half, 0);
    let done: Vec<_> = partial.objectives.iter().map(|o| o.done).collect();
    assert_eq!(done, [true, false, false]);

    let mut fixed = m.project.clone();
    for d in &mut fixed.devices {
        if d.id == id("host-a") {
            d.network.dns_server = Some("192.168.1.3".into());
        }
    }
    let trace = lookup(&fixed, true);
    let after = report_for(&m, &fixed, &trace, 0);
    assert!(after.complete, "{after:?}");
    assert_eq!(after.stars, Some(3));
}

#[test]
fn forgotten_cable_needs_the_link_and_a_reply() {
    let m = mission("forgotten-cable");
    let mut fixed = m.project.clone();
    fixed.links.push(Link {
        id: id("cable-host-c"),
        endpoints: [id("host-c-p0"), id("switch-p2")],
        latency_us: Decimal(1000),
        jitter_us: Decimal(0),
    });
    let trace = run(
        &fixed,
        Command::Ping {
            source: id("host-a"),
            destination: "192.168.1.4".into(),
            ttl: 64,
        },
    );
    let r = report_for(&m, &fixed, &trace, 5);
    assert!(r.complete, "{r:?}");
    assert_eq!(r.stars, Some(1));
    assert_eq!(r.hints_used, 2, "hints are bounded by the mission");
    // A reply observed earlier still counts when the trace was reset.
    let observed = BTreeSet::from(["ping".to_owned()]);
    assert!(report(&m, &fixed, &[], &observed, 0, "en").complete);
    assert!(!report(&m, &m.project, &[], &observed, 0, "en").complete);
}
