//! R-MAIL exercises production SMTP, POP3 and DNS over the virtual LAN.
use sns_engine::{Engine, Limits};
use sns_model::*;
fn id(v: &str) -> Id {
    Id::new(v).unwrap()
}
thread_local! {static SERIAL:std::cell::Cell<u64>=const{std::cell::Cell::new(0)};}
fn engine() -> Engine {
    SERIAL.with(|s| s.set(0));
    Engine::from_project(
        serde_json::from_str(include_str!("../../../fixtures/valid/mail.json")).unwrap(),
        42,
        Limits::default(),
    )
    .unwrap()
}
fn app(e: &mut Engine, host: &str, command: AppCommand) {
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id(&format!(
            "mail-command-{}",
            SERIAL.with(|s| {
                let n = s.get();
                s.set(n + 1);
                n
            })
        )),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::App {
            device: id(host),
            command,
        },
    })
    .unwrap();
}
fn mail(e: &mut Engine, host: &str, application: &str, command: MailCommand) {
    app(
        e,
        host,
        AppCommand::Mail {
            application: id(application),
            command,
        },
    );
}
fn run(e: &mut Engine, delta: u64) {
    e.run(100_000, Some(e.now() + delta)).unwrap();
}
fn start(e: &mut Engine) {
    for (host, a) in [("host-b", "alpha-server"), ("host-c", "beta-server")] {
        mail(e, host, a, MailCommand::Start);
    }
    app(
        e,
        "host-b",
        AppCommand::StartServer {
            application: id("dns"),
            port: 53,
        },
    );
}
fn client(e: &Engine, host: &str, a: &str) -> MailClientData {
    match &e
        .project()
        .device(&id(host))
        .unwrap()
        .applications
        .iter()
        .find(|m| m.id == id(a))
        .unwrap()
        .data
    {
        Some(ApplicationData::MailClient(d)) => d.clone(),
        _ => Default::default(),
    }
}
fn server(e: &Engine, host: &str, a: &str) -> MailServerData {
    match &e
        .project()
        .device(&id(host))
        .unwrap()
        .applications
        .iter()
        .find(|m| m.id == id(a))
        .unwrap()
        .data
    {
        Some(ApplicationData::MailServer(d)) => d.clone(),
        _ => Default::default(),
    }
}
fn draft(to: &str) -> MailDraft {
    MailDraft {
        to: vec![to.into()],
        cc: vec![],
        bcc: vec![],
        subject: "Réseaux et courrier".into(),
        body: "Bonjour élève\n.ligne avec point\n.".into(),
    }
}
#[test]
fn smtp_pop3_local_delivery_and_durable_client_folders() {
    let mut e = engine();
    start(&mut e);
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::Send {
            draft: draft("alice@alpha.test"),
        },
    );
    run(&mut e, 2_000_000);
    assert_eq!(client(&e, "host-a", "alice-client").sent.len(), 1);
    assert_eq!(
        server(&e, "host-b", "alpha-server").mailboxes["alice"].len(),
        1
    );
    mail(&mut e, "host-a", "alice-client", MailCommand::Receive);
    run(&mut e, 2_000_000);
    let data = client(&e, "host-a", "alice-client");
    assert_eq!(data.inbox.len(), 1);
    assert_eq!(data.inbox[0].sender_name, "Alice");
    assert_eq!(data.inbox[0].body, draft("alice@alpha.test").body);
    assert!(server(&e, "host-b", "alpha-server").mailboxes["alice"].is_empty());
    let mid = data.inbox[0].id.clone();
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::Reply {
            message: mid.clone(),
        },
    );
    let view = e
        .apps()
        .views(e.project())
        .into_iter()
        .find(|h| h.device == id("host-a"))
        .unwrap()
        .applications
        .into_iter()
        .find(|a| a.id == id("alice-client"))
        .unwrap();
    assert_eq!(view.mail.draft.unwrap().subject, "Re: Réseaux et courrier");
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::Delete {
            folder: MailFolder::Inbox,
            message: mid,
        },
    );
    let saved = serde_json::to_vec(e.project()).unwrap();
    let reopened: sns_model::Project = serde_json::from_slice(&saved).unwrap();
    reopened.validate().unwrap();
    assert!(client(&e, "host-a", "alice-client").inbox.is_empty());
    assert!(e.trace().iter().any(|f| f.kind == "smtp.line.received"));
    assert!(e.trace().iter().any(|f| f.kind == "pop3.line.received"));
    record(&e, "mail-local");
}
#[test]
fn mx_then_a_relay_and_bcc_do_not_expose_hidden_recipients() {
    let mut e = engine();
    start(&mut e);
    let mut message = draft("carol@beta.test");
    message.cc = vec!["bob@alpha.test".into()];
    message.bcc = vec!["hidden@alpha.test".into()];
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::Send { draft: message },
    );
    run(&mut e, 5_000_000);
    assert_eq!(
        client(&e, "host-a", "alice-client").sent[0].bcc,
        vec!["hidden@alpha.test"]
    );
    let remote = server(&e, "host-c", "beta-server");
    assert_eq!(remote.mailboxes["carol"].len(), 1);
    assert!(remote.mailboxes["carol"][0].bcc.is_empty());
    let local = server(&e, "host-b", "alpha-server");
    assert!(local.mailboxes["bob"][0].bcc.is_empty());
    assert!(local.mailboxes["hidden"][0].bcc.is_empty());
    mail(&mut e, "host-c", "carol-client", MailCommand::Receive);
    run(&mut e, 2_000_000);
    assert_eq!(client(&e, "host-c", "carol-client").inbox.len(), 1);
    record(&e, "mail-relay");
    for t in ["MX", "A"] {
        assert!(
            e.trace()
                .iter()
                .any(|f| f.kind == "dns.query.sent" && f.payload["record_type"] == t)
        );
    }
}
#[test]
fn unknown_recipient_and_deleted_account_have_explicit_failures() {
    let mut e = engine();
    start(&mut e);
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::Send {
            draft: draft("absent@alpha.test"),
        },
    );
    run(&mut e, 1_000_000);
    assert!(client(&e, "host-a", "alice-client").sent.is_empty());
    assert!(e.trace().iter().any(
        |f| f.kind == "mail.operation.failed" && f.payload["code"] == "mail.unknown_recipient"
    ));
    let mut cfg = match e.project().device(&id("host-b")).unwrap().applications[0]
        .service
        .clone()
        .unwrap()
    {
        ServiceConfig::MailServer(c) => c,
        _ => panic!(),
    };
    cfg.accounts.retain(|a| a.username != "alice");
    app(
        &mut e,
        "host-b",
        AppCommand::ConfigureService {
            application: id("alpha-server"),
            config: ServiceConfig::MailServer(cfg),
        },
    );
    mail(&mut e, "host-a", "alice-client", MailCommand::Receive);
    run(&mut e, 1_000_000);
    assert!(
        e.trace().iter().any(|f| f.kind == "mail.operation.failed"
            && f.payload["code"] == "mail.authentication_failed")
    );
}
#[test]
fn contacts_persist_and_bcc_only_delivery_is_valid() {
    let mut e = engine();
    start(&mut e);
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::UpsertContact {
            contact: Contact {
                id: "carol".into(),
                first_name: "Carol".into(),
                last_name: "Élève".into(),
                street: "Rue des Écoles".into(),
                house_number: "12".into(),
                postal_code: "75000".into(),
                city: "Paris".into(),
                email: "carol@beta.test".into(),
                phone: "0102030405".into(),
            },
        },
    );
    let saved = serde_json::to_vec(e.project()).unwrap();
    let reopened: Project = serde_json::from_slice(&saved).unwrap();
    let stored = &reopened.devices[0].applications[0].data;
    assert!(
        matches!(stored,Some(ApplicationData::MailClient(c))if c.contacts[0].street=="Rue des Écoles")
    );
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::DeleteContact { id: "carol".into() },
    );
    assert!(client(&e, "host-a", "alice-client").contacts.is_empty());
    let mut msg = draft("alice@alpha.test");
    msg.bcc = msg.to.clone();
    msg.to.clear();
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::Send { draft: msg },
    );
    run(&mut e, 2_000_000);
    assert!(
        server(&e, "host-b", "alpha-server").mailboxes["alice"][0]
            .bcc
            .is_empty()
    );
}
#[test]
fn invalid_credentials_custom_ports_and_reopened_mailboxes_are_isolated() {
    let mut e = engine();
    let mut server_cfg = match e.project().devices[1].applications[0]
        .service
        .clone()
        .unwrap()
    {
        ServiceConfig::MailServer(c) => c,
        _ => panic!(),
    };
    server_cfg.smtp_port = 2525;
    server_cfg.pop3_port = 1110;
    app(
        &mut e,
        "host-b",
        AppCommand::ConfigureService {
            application: id("alpha-server"),
            config: ServiceConfig::MailServer(server_cfg),
        },
    );
    let mut cfg = match e.project().devices[0].applications[0]
        .service
        .clone()
        .unwrap()
    {
        ServiceConfig::MailClient(c) => c,
        _ => panic!(),
    };
    cfg.smtp_port = 2525;
    cfg.pop3_port = 1110;
    app(
        &mut e,
        "host-a",
        AppCommand::ConfigureService {
            application: id("alice-client"),
            config: ServiceConfig::MailClient(cfg.clone()),
        },
    );
    start(&mut e);
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::Send {
            draft: draft("bob@alpha.test"),
        },
    );
    run(&mut e, 1_000_000);
    assert_eq!(
        server(&e, "host-b", "alpha-server").mailboxes["bob"].len(),
        1
    );
    let saved = serde_json::to_vec(e.project()).unwrap();
    e = Engine::from_project(
        serde_json::from_slice(&saved).unwrap(),
        42,
        Limits::default(),
    )
    .unwrap();
    start(&mut e);
    mail(&mut e, "host-a", "alice-client", MailCommand::Receive);
    run(&mut e, 1_000_000);
    assert!(client(&e, "host-a", "alice-client").inbox.is_empty());
    cfg.address = "bob@alpha.test".into();
    cfg.username = "bob".into();
    cfg.password = "incorrect".into();
    app(
        &mut e,
        "host-a",
        AppCommand::ConfigureService {
            application: id("alice-client"),
            config: ServiceConfig::MailClient(cfg.clone()),
        },
    );
    mail(&mut e, "host-a", "alice-client", MailCommand::Receive);
    run(&mut e, 1_000_000);
    assert!(
        e.trace().iter().any(|t| t.kind == "mail.operation.failed"
            && t.payload["code"] == "mail.authentication_failed")
    );
    assert_eq!(
        server(&e, "host-b", "alpha-server").mailboxes["bob"].len(),
        1
    );
    cfg.password = "classe".into();
    app(
        &mut e,
        "host-a",
        AppCommand::ConfigureService {
            application: id("alice-client"),
            config: ServiceConfig::MailClient(cfg),
        },
    );
    mail(&mut e, "host-a", "alice-client", MailCommand::Receive);
    run(&mut e, 1_000_000);
    assert_eq!(client(&e, "host-a", "alice-client").inbox.len(), 1);
    assert_eq!(client(&e, "host-a", "alice-client").sent.len(), 1);
}
#[test]
fn interrupted_pop3_dele_does_not_commit_and_releases_mailbox_lock() {
    let mut e = engine();
    start(&mut e);
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::Send {
            draft: draft("alice@alpha.test"),
        },
    );
    run(&mut e, 1_000_000);
    app(
        &mut e,
        "host-c",
        AppCommand::Install {
            application: id("raw-pop"),
            software: Software::TcpClient,
        },
    );
    app(
        &mut e,
        "host-c",
        AppCommand::Connect {
            application: id("raw-pop"),
            target: "192.168.1.3".into(),
            port: 110,
        },
    );
    run(&mut e, 100_000);
    app(
        &mut e,
        "host-c",
        AppCommand::Send {
            application: id("raw-pop"),
            text: "USER alice\r\nPASS classe\r\nDELE 1\r\n".into(),
        },
    );
    run(&mut e, 100_000);
    assert_eq!(
        server(&e, "host-b", "alpha-server").mailboxes["alice"].len(),
        1
    );
    app(
        &mut e,
        "host-c",
        AppCommand::Stop {
            application: id("raw-pop"),
        },
    );
    run(&mut e, 100_000);
    mail(&mut e, "host-a", "alice-client", MailCommand::Receive);
    run(&mut e, 1_000_000);
    assert_eq!(client(&e, "host-a", "alice-client").inbox.len(), 1);
    assert!(server(&e, "host-b", "alpha-server").mailboxes["alice"].is_empty());
}
#[test]
fn occupied_pop3_port_rolls_back_smtp_listener_atomically() {
    let mut e = engine();
    app(
        &mut e,
        "host-b",
        AppCommand::Install {
            application: id("port-blocker"),
            software: Software::TcpServer,
        },
    );
    app(
        &mut e,
        "host-b",
        AppCommand::StartServer {
            application: id("port-blocker"),
            port: 110,
        },
    );
    let envelope = CommandEnvelope {
        api_version: 1,
        command_id: id("mail-port-conflict"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(e.revision()),
        command: Command::App {
            device: id("host-b"),
            command: AppCommand::Mail {
                application: id("alpha-server"),
                command: MailCommand::Start,
            },
        },
    };
    assert!(e.command(envelope).is_err());
    assert!(
        !e.network()
            .transport
            .views()
            .iter()
            .any(|s| s.owner == id("alpha-server"))
    );
    app(
        &mut e,
        "host-b",
        AppCommand::Stop {
            application: id("port-blocker"),
        },
    );
    mail(&mut e, "host-b", "alpha-server", MailCommand::Start);
    assert_eq!(
        e.network()
            .transport
            .views()
            .iter()
            .filter(|s| s.owner == id("alpha-server"))
            .count(),
        2
    );
}
#[test]
fn remote_unknown_recipient_fails_after_mx_lookup_without_committing_local_copy() {
    let mut e = engine();
    start(&mut e);
    let mut message = draft("absent@beta.test");
    message.cc.push("alice@alpha.test".into());
    mail(
        &mut e,
        "host-a",
        "alice-client",
        MailCommand::Send { draft: message },
    );
    run(&mut e, 5_000_000);
    assert!(client(&e, "host-a", "alice-client").sent.is_empty());
    assert!(
        server(&e, "host-b", "alpha-server")
            .mailboxes
            .values()
            .all(Vec::is_empty)
    );
    assert!(e.trace().iter().any(
        |t| t.kind == "mail.operation.failed" && t.payload["code"] == "mail.unknown_recipient"
    ));
    assert!(
        e.trace()
            .iter()
            .any(|t| t.kind == "dns.query.sent" && t.payload["record_type"] == "MX")
    );
}
fn record(e: &Engine, name: &str) {
    if let Some(dir) = std::env::var_os("SNS_J6_ARTIFACT_DIR") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(format!("j6-{name}.jsonl")), e.canonical_trace()).unwrap();
        std::fs::write(
            dir.join(format!("j6-{name}-project.json")),
            serde_json::to_vec_pretty(e.project()).unwrap(),
        )
        .unwrap();
    }
}
