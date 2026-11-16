//! Bounded SMTP/POP3 sessions. Every protocol octet uses the simulated TCP stack.
use super::*;
use sha2::{Digest, Sha256};
#[derive(Clone, Copy)]
pub(super) struct Context<'a> {
    pub project: &'a Project,
    pub net: &'a Network,
    pub now: u64,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct MailView {
    pub busy: bool,
    pub draft: Option<MailDraft>,
    pub draft_generation: u64,
}
type Key = (Id, Id);
#[derive(Clone, Default, serde::Serialize)]
pub(super) struct Mail {
    connections: BTreeMap<Key, Connection>,
    jobs: BTreeMap<u64, Operation>,
    deliveries: BTreeMap<u64, Delivery>,
    stores: BTreeMap<Key, ApplicationData>,
    locks: BTreeMap<(Id, Id, String), Id>,
}
#[derive(Clone, serde::Serialize)]
struct Connection {
    app: Id,
    flow: Id,
    token: u64,
    buffer: Vec<u8>,
    kind: ConnectionKind,
}
#[derive(Clone, serde::Serialize)]
enum ConnectionKind {
    SmtpServer(SmtpServer),
    PopServer(PopServer),
    SmtpClient {
        job: u64,
        stage: u8,
        index: usize,
    },
    PopClient {
        job: u64,
        stage: u8,
        index: usize,
        count: usize,
        multiline: Option<Vec<String>>,
    },
}
#[derive(Clone, Default, serde::Serialize)]
struct SmtpServer {
    hello: bool,
    authenticated: Option<String>,
    from: Option<String>,
    recipients: Vec<String>,
    data: Option<Vec<String>>,
    waiting: bool,
}
#[derive(Clone, Default, serde::Serialize)]
struct PopServer {
    username: Option<String>,
    authenticated: bool,
    messages: Vec<MailMessage>,
    deleted: BTreeSet<Id>,
}
#[derive(Clone, serde::Serialize)]
struct Operation {
    device: Id,
    app: Id,
    flow: Id,
    kind: OperationKind,
    completion: Completion,
}
#[derive(Clone, serde::Serialize)]
enum OperationKind {
    Smtp {
        message: Box<MailMessage>,
        envelope: Vec<String>,
        auth: Option<(String, String)>,
    },
    Pop {
        username: String,
        password: String,
    },
}
#[derive(Clone, serde::Serialize)]
enum Completion {
    Client,
    Relay(u64),
}
#[derive(Clone, serde::Serialize)]
struct Delivery {
    device: Id,
    app: Id,
    flow: Id,
    socket: Id,
    message: MailMessage,
    local: Vec<String>,
    remaining: usize,
    failed: bool,
}
#[derive(Clone, serde::Serialize)]
pub(super) enum Resolve {
    Address { port: u16 },
    Mx,
}
fn metadata<'a>(p: &'a Project, d: &Id, a: &Id) -> Result<&'a Application, Diagnostic> {
    p.device(d)
        .and_then(|h| h.applications.iter().find(|s| &s.id == a))
        .ok_or_else(|| diag("application.not_installed", a.as_str()))
}
fn server_config(c: Context<'_>, d: &Id, a: &Id) -> Result<MailServerConfig, Diagnostic> {
    if let Some(ServiceConfig::MailServer(s)) = &metadata(c.project, d, a)?.service {
        Ok(s.clone())
    } else {
        Err(diag("mail.invalid_config", a.as_str()))
    }
}
fn client_config(c: Context<'_>, d: &Id, a: &Id) -> Result<MailClientConfig, Diagnostic> {
    if let Some(ServiceConfig::MailClient(s)) = &metadata(c.project, d, a)?.service {
        Ok(s.clone())
    } else {
        Err(diag("mail.invalid_config", a.as_str()))
    }
}
fn send(out: &mut Output, d: &Id, s: &Id, flow: &Id, text: &str) {
    socket(
        out,
        d,
        flow,
        SocketCommand::Send {
            socket: s.clone(),
            data_base64: STANDARD.encode(text.as_bytes()),
        },
    );
}
fn close(out: &mut Output, d: &Id, s: &Id, flow: &Id) {
    socket(
        out,
        d,
        flow,
        SocketCommand::Close {
            socket: s.clone(),
            force: false,
        },
    );
}
fn envelope(m: &MailMessage) -> Vec<String> {
    m.to.iter()
        .chain(&m.cc)
        .chain(&m.bcc)
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn domain(address: &str) -> &str {
    address.split_once('@').map(|(_, d)| d).unwrap_or("")
}
fn message_text(m: &MailMessage) -> String {
    format!(
        "Message-ID: <{}@simulator.invalid>\r\nFrom: {}\r\nTo: {}\r\nCc: {}\r\nSubject: {}\r\nX-Sinesi-Route: {}\r\n\r\n{}",
        m.id.as_str(),
        if m.sender_name.is_empty() {
            m.from.clone()
        } else {
            format!(
                "\"{}\" <{}>",
                m.sender_name.replace('\\', "\\\\").replace('"', "\\\""),
                m.from
            )
        },
        m.to.join(", "),
        m.cc.join(", "),
        m.subject,
        m.relay_path.join(","),
        m.body
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .replace('\n', "\r\n")
    )
}
fn block(text: &str) -> String {
    let text = text.replace("\r\n", "\n");
    let mut result = text
        .split('\n')
        .map(|line| {
            if line.starts_with('.') {
                format!(".{line}")
            } else {
                line.into()
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n");
    result.push_str("\r\n.\r\n");
    result
}
fn parse_message(text: &str, fallback: Id, now: u64) -> Result<MailMessage, Diagnostic> {
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| diag("mail.invalid_message", "headers"))?;
    let mut headers = BTreeMap::new();
    for h in head.split("\r\n") {
        let (k, v) = h
            .split_once(':')
            .ok_or_else(|| diag("mail.invalid_message", "header"))?;
        if headers
            .insert(k.to_ascii_lowercase(), v.trim().to_string())
            .is_some()
        {
            return Err(diag("mail.invalid_message", "duplicate_header"));
        }
    }
    let addresses = |key: &str| -> Result<Vec<String>, Diagnostic> {
        headers
            .get(key)
            .map(|s| {
                s.split(',')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| mail_address(s.trim()))
                    .collect()
            })
            .unwrap_or(Ok(vec![]))
    };
    let id = headers
        .get("message-id")
        .and_then(|s| s.strip_prefix('<'))
        .and_then(|s| s.strip_suffix("@simulator.invalid>"))
        .and_then(|s| Id::new(s).ok())
        .unwrap_or(fallback);
    let from = headers
        .get("from")
        .ok_or_else(|| diag("mail.invalid_message", "from"))?;
    let (sender_name, sender_address) = if let Some((name, address)) = from.rsplit_once('<') {
        (
            name.trim()
                .trim_matches('"')
                .replace("\\\"", "\"")
                .replace("\\\\", "\\"),
            address.trim_end_matches('>'),
        )
    } else {
        (String::new(), from.as_str())
    };
    let m = MailMessage {
        id,
        sender_name,
        from: mail_address(sender_address)?,
        to: addresses("to")?,
        cc: addresses("cc")?,
        bcc: vec![],
        subject: headers.get("subject").cloned().unwrap_or_default(),
        body: body.replace("\r\n", "\n"),
        received_us: Decimal(now),
        relay_path: headers
            .get("x-sinesi-route")
            .map(|s| {
                s.split(',')
                    .filter(|s| !s.is_empty())
                    .map(dns_name)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default(),
    };
    m.validate()?;
    Ok(m)
}
impl Runtime {
    fn mail_data(
        &mut self,
        c: Context<'_>,
        d: &Id,
        a: &Id,
    ) -> Result<&mut ApplicationData, Diagnostic> {
        let key = (d.clone(), a.clone());
        if !self.mail.stores.contains_key(&key) {
            let m = metadata(c.project, d, a)?;
            let data = m.data.clone().unwrap_or_else(|| {
                if m.kind == "mail_server" {
                    ApplicationData::MailServer(Default::default())
                } else {
                    ApplicationData::MailClient(Default::default())
                }
            });
            self.mail.stores.insert(key.clone(), data);
        }
        Ok(self.mail.stores.get_mut(&key).unwrap())
    }
    fn mail_persist(
        &mut self,
        c: Context<'_>,
        d: &Id,
        a: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let data = self.mail_data(c, d, a)?.clone();
        data.validate(&metadata(c.project, d, a)?.kind)?;
        out.effects.push(Effect::ApplicationData {
            device: d.clone(),
            application: a.clone(),
            data,
        });
        Ok(())
    }
    pub(super) fn mail_reconfigure(
        &mut self,
        project: &mut Project,
        d: &Id,
        a: &Id,
        config: &ServiceConfig,
    ) -> Result<(), Diagnostic> {
        let app = project
            .devices
            .iter_mut()
            .find(|h| &h.id == d)
            .unwrap()
            .applications
            .iter_mut()
            .find(|m| &m.id == a)
            .unwrap();
        if let ServiceConfig::MailServer(cfg) = config {
            let mut data = match app.data.clone() {
                Some(ApplicationData::MailServer(data)) => data,
                _ => Default::default(),
            };
            data.mailboxes.retain(|name, _| {
                cfg.accounts
                    .iter()
                    .any(|a| a.username.eq_ignore_ascii_case(name))
            });
            app.data = Some(ApplicationData::MailServer(data.clone()));
            self.mail
                .stores
                .insert((d.clone(), a.clone()), ApplicationData::MailServer(data));
        }
        Ok(())
    }
    pub(super) fn mail_cancel(&mut self, d: &Id, a: &Id, uninstall: bool) {
        let jobs: Vec<_> = self
            .mail
            .jobs
            .iter()
            .filter(|(_, j)| &j.device == d && &j.app == a)
            .map(|(id, _)| *id)
            .collect();
        for id in jobs {
            self.mail.jobs.remove(&id);
        }
        self.mail
            .connections
            .retain(|(host, _), s| host != d || &s.app != a);
        self.mail
            .deliveries
            .retain(|_, p| &p.device != d || &p.app != a);
        self.mail
            .locks
            .retain(|(host, app, _), _| host != d || app != a);
        if uninstall {
            self.mail.stores.remove(&(d.clone(), a.clone()));
        }
        if let Ok(s) = self.state(d, a) {
            s.mail.busy = false;
        }
    }
    pub(super) fn mail_command(
        &mut self,
        c: Context<'_>,
        d: &Id,
        a: &Id,
        cmd: &MailCommand,
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let m = metadata(c.project, d, a)?;
        let server = m.kind == "mail_server";
        if !server && m.kind != "mail_client" {
            return Err(diag("application.invalid_operation", a.as_str()));
        }
        if self.state(d, a)?.mail.busy && matches!(cmd, MailCommand::UpsertContact { .. }) {
            return Err(diag("mail.busy", a.as_str()));
        }
        match cmd {
            MailCommand::Start => {
                if !server {
                    return Err(diag("application.invalid_operation", a.as_str()));
                }
                let cfg = server_config(c, d, a)?;
                cfg.validate()?;
                if self.state(d, a)?.running {
                    return Err(diag("application.already_running", a.as_str()));
                }
                self.state(d, a)?.running = true;
                for port in [cfg.smtp_port, cfg.pop3_port] {
                    self.serial += 1;
                    socket(
                        out,
                        d,
                        flow,
                        SocketCommand::Bind {
                            socket: Id::new(format!("mail-listener-{}", self.serial))?,
                            owner: a.clone(),
                            protocol: TransportProtocol::Tcp,
                            interface: None,
                            address: None,
                            port,
                        },
                    );
                }
                fact(
                    out,
                    d,
                    flow,
                    "mail.server.started",
                    json!({"application":a,"smtp_port":cfg.smtp_port,"pop3_port":cfg.pop3_port}),
                );
            }
            MailCommand::Send { draft } => {
                let cfg = client_config(c, d, a)?;
                cfg.validate()?;
                draft.validate()?;
                if self.state(d, a)?.mail.busy {
                    return Err(diag("mail.busy", a.as_str()));
                }
                let ApplicationData::MailClient(data) = self.mail_data(c, d, a)? else {
                    return Err(diag("mail.invalid_config", a.as_str()));
                };
                data.next_id.0 = data
                    .next_id
                    .0
                    .checked_add(1)
                    .ok_or_else(|| diag("simulation.sequence_overflow", "mail"))?;
                let nonce = data.next_id.0;
                let hash = Sha256::digest(format!(
                    "{}:{}:{}:{nonce}",
                    c.project.project_id.as_str(),
                    d.as_str(),
                    a.as_str()
                ));
                let mid = Id::new(format!("mail-{hash:x}"))?;
                let message = MailMessage {
                    sender_name: cfg.display_name.clone(),
                    id: mid,
                    from: mail_address(&cfg.address)?,
                    to: draft
                        .to
                        .iter()
                        .map(|v| mail_address(v))
                        .collect::<Result<_, _>>()?,
                    cc: draft
                        .cc
                        .iter()
                        .map(|v| mail_address(v))
                        .collect::<Result<_, _>>()?,
                    bcc: draft
                        .bcc
                        .iter()
                        .map(|v| mail_address(v))
                        .collect::<Result<_, _>>()?,
                    subject: draft.subject.clone(),
                    body: draft.body.replace("\r\n", "\n"),
                    received_us: Decimal(c.now),
                    relay_path: vec![],
                };
                let mut reserved = self.mail_data(c, d, a)?.clone();
                if let ApplicationData::MailClient(data) = &mut reserved {
                    data.sent.push(message.clone());
                }
                reserved.validate("mail_client")?;
                self.mail_persist(c, d, a, out)?;
                self.state(d, a)?.mail.busy = true;
                self.serial += 1;
                let job = self.serial;
                let op = Operation {
                    device: d.clone(),
                    app: a.clone(),
                    flow: flow.clone(),
                    kind: OperationKind::Smtp {
                        envelope: envelope(&message),
                        message: Box::new(message),
                        auth: Some((cfg.username, cfg.password)),
                    },
                    completion: Completion::Client,
                };
                self.mail_begin(
                    c,
                    job,
                    op,
                    &cfg.smtp_server,
                    Resolve::Address {
                        port: cfg.smtp_port,
                    },
                    out,
                )?;
            }
            MailCommand::Receive => {
                let cfg = client_config(c, d, a)?;
                cfg.validate()?;
                if self.state(d, a)?.mail.busy {
                    return Err(diag("mail.busy", a.as_str()));
                }
                self.state(d, a)?.mail.busy = true;
                self.serial += 1;
                let job = self.serial;
                self.mail_begin(
                    c,
                    job,
                    Operation {
                        device: d.clone(),
                        app: a.clone(),
                        flow: flow.clone(),
                        kind: OperationKind::Pop {
                            username: cfg.username,
                            password: cfg.password,
                        },
                        completion: Completion::Client,
                    },
                    &cfg.pop3_server,
                    Resolve::Address {
                        port: cfg.pop3_port,
                    },
                    out,
                )?;
            }
            MailCommand::Reply { message } => {
                let ApplicationData::MailClient(data) = self.mail_data(c, d, a)? else {
                    return Err(diag("application.invalid_operation", a.as_str()));
                };
                let msg = data
                    .inbox
                    .iter()
                    .find(|m| &m.id == message)
                    .cloned()
                    .ok_or_else(|| diag("mail.message_not_found", message.as_str()))?;
                self.serial += 1;
                let generation = self.serial;
                let state = self.state(d, a)?;
                state.mail.draft = Some(MailDraft {
                    to: vec![msg.from],
                    cc: vec![],
                    bcc: vec![],
                    subject: if msg.subject.to_ascii_lowercase().starts_with("re:") {
                        msg.subject
                    } else {
                        format!("Re: {}", msg.subject)
                    },
                    body: String::new(),
                });
                state.mail.draft_generation = generation;
            }
            MailCommand::Delete { folder, message } => {
                let ApplicationData::MailClient(data) = self.mail_data(c, d, a)? else {
                    return Err(diag("application.invalid_operation", a.as_str()));
                };
                let list = match folder {
                    MailFolder::Inbox => &mut data.inbox,
                    MailFolder::Sent => &mut data.sent,
                };
                if !list.iter().any(|m| &m.id == message) {
                    return Err(diag("mail.message_not_found", message.as_str()));
                }
                list.retain(|m| &m.id != message);
                self.mail_persist(c, d, a, out)?;
            }
            MailCommand::UpsertContact { contact } => {
                let ApplicationData::MailClient(data) = self.mail_data(c, d, a)? else {
                    return Err(diag("application.invalid_operation", a.as_str()));
                };
                data.contacts.retain(|v| v.id != contact.id);
                data.contacts.push(contact.clone());
                self.mail_persist(c, d, a, out)?;
            }
            MailCommand::DeleteContact { id } => {
                let ApplicationData::MailClient(data) = self.mail_data(c, d, a)? else {
                    return Err(diag("application.invalid_operation", a.as_str()));
                };
                data.contacts.retain(|v| &v.id != id);
                self.mail_persist(c, d, a, out)?;
            }
        }
        Ok(())
    }
    fn mail_begin(
        &mut self,
        c: Context<'_>,
        job: u64,
        op: Operation,
        target: &str,
        resolve: Resolve,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        if self.mail.jobs.len() >= 128 {
            return Err(diag("resource.limit_exceeded", "mail_jobs"));
        }
        self.mail.jobs.insert(job, op.clone());
        out.effects.push(Effect::Timer(Timer {
            device: op.device.clone(),
            application: op.app.clone(),
            token: job,
            kind: TimerKind::MailJob { job },
            delay_us: 30_000_000,
            correlation: op.flow.clone(),
        }));
        let result = if let Resolve::Address { port } = resolve
            && let Ok(address) = target.parse::<Ipv4Addr>()
        {
            self.mail_connect(c, job, address, port, out)
        } else {
            let kind = if matches!(resolve, Resolve::Mx) {
                DnsType::Mx
            } else {
                DnsType::A
            };
            self.dns_query(
                c.project,
                c.net,
                c.now,
                &op.device,
                &op.app,
                target,
                kind,
                None,
                dns::Purpose::Mail { job, resolve },
                &op.flow,
                out,
            )
        };
        if let Err(e) = result {
            self.mail_finish(c, job, Err(e), out)?;
        }
        Ok(())
    }
    pub(super) fn mail_dns(
        &mut self,
        c: Context<'_>,
        job: u64,
        resolve: Resolve,
        code: &str,
        records: &[DnsRecord],
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let Some(op) = self.mail.jobs.get(&job).cloned() else {
            return Ok(());
        };
        if code != "ok" {
            return self.mail_finish(c, job, Err(diag("mail.resolution_failed", code)), out);
        }
        match resolve {
            Resolve::Mx => {
                let target = records
                    .iter()
                    .filter(|r| r.record_type == DnsType::Mx)
                    .min_by_key(|r| (r.priority, &r.value))
                    .map(|r| r.value.clone());
                if let Some(target) = target {
                    if let Err(e) = self.dns_query(
                        c.project,
                        c.net,
                        c.now,
                        &op.device,
                        &op.app,
                        &target,
                        DnsType::A,
                        None,
                        dns::Purpose::Mail {
                            job,
                            resolve: Resolve::Address { port: 25 },
                        },
                        &op.flow,
                        out,
                    ) {
                        self.mail_finish(c, job, Err(e), out)?;
                    }
                } else {
                    self.mail_finish(c, job, Err(diag("mail.resolution_failed", "MX")), out)?;
                }
            }
            Resolve::Address { port } => {
                if let Some(address) = records
                    .iter()
                    .find(|r| r.record_type == DnsType::A)
                    .and_then(|r| r.value.parse().ok())
                {
                    self.mail_connect(c, job, address, port, out)?;
                } else {
                    self.mail_finish(c, job, Err(diag("mail.resolution_failed", "A")), out)?;
                }
            }
        }
        Ok(())
    }
    fn mail_connect(
        &mut self,
        _c: Context<'_>,
        job: u64,
        address: Ipv4Addr,
        port: u16,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let Some(op) = self.mail.jobs.get(&job).cloned() else {
            return Ok(());
        };
        if self.mail.connections.len() >= 256 {
            return Err(diag("resource.limit_exceeded", "mail_connections"));
        }
        let socket_id = Id::new(format!("mail-client-{job}"))?;
        let kind = match op.kind {
            OperationKind::Smtp { .. } => ConnectionKind::SmtpClient {
                job,
                stage: 0,
                index: 0,
            },
            OperationKind::Pop { .. } => ConnectionKind::PopClient {
                job,
                stage: 0,
                index: 1,
                count: 0,
                multiline: None,
            },
        };
        self.mail.connections.insert(
            (op.device.clone(), socket_id.clone()),
            Connection {
                app: op.app.clone(),
                flow: op.flow.clone(),
                token: job,
                buffer: vec![],
                kind,
            },
        );
        socket(
            out,
            &op.device,
            &op.flow,
            SocketCommand::Connect {
                socket: socket_id,
                owner: op.app,
                destination: address,
                port,
                local_address: None,
                local_port: None,
            },
        );
        Ok(())
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "SMTP transaction context and durable delivery envelope"
    )]
    fn mail_deliver(
        &mut self,
        c: Context<'_>,
        d: &Id,
        a: &Id,
        message: &MailMessage,
        users: &[String],
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let cfg = server_config(c, d, a)?;
        let mut candidate = self.mail_data(c, d, a)?.clone();
        let ApplicationData::MailServer(data) = &mut candidate else {
            return Err(diag("mail.invalid_config", a.as_str()));
        };
        for user in users {
            if !cfg
                .accounts
                .iter()
                .any(|a| a.username.eq_ignore_ascii_case(user))
            {
                return Err(diag("mail.unknown_recipient", user));
            }
            let list = data.mailboxes.entry(user.to_ascii_lowercase()).or_default();
            if !list.iter().any(|m| m.id == message.id) {
                let mut received = message.clone();
                received.bcc.clear();
                received.received_us = Decimal(c.now);
                list.push(received);
            }
        }
        candidate.validate("mail_server")?;
        self.mail.stores.insert((d.clone(), a.clone()), candidate);
        self.mail_persist(c, d, a, out)?;
        fact(
            out,
            d,
            flow,
            "mail.delivered",
            json!({"application":a,"message":message.id,"recipients":users}),
        );
        Ok(())
    }
    fn mail_finish(
        &mut self,
        c: Context<'_>,
        job: u64,
        result: Result<(), Diagnostic>,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let Some(op) = self.mail.jobs.remove(&job) else {
            return Ok(());
        };
        match op.completion {
            Completion::Client => {
                self.state(&op.device, &op.app)?.mail.busy = false;
                match result {
                    Ok(()) => {
                        if let OperationKind::Smtp { message, .. } = op.kind {
                            if let ApplicationData::MailClient(data) =
                                self.mail_data(c, &op.device, &op.app)?
                                && !data.sent.iter().any(|m| m.id == message.id)
                            {
                                data.sent.push(*message);
                            }
                            self.mail_persist(c, &op.device, &op.app, out)?;
                        }
                        push(self.state(&op.device, &op.app)?, message("mail_done", &[]));
                        fact(
                            out,
                            &op.device,
                            &op.flow,
                            "mail.operation.completed",
                            json!({"application":op.app}),
                        );
                    }
                    Err(e) => {
                        push(
                            self.state(&op.device, &op.app)?,
                            Line::Error {
                                diagnostic: e.clone(),
                            },
                        );
                        fact(
                            out,
                            &op.device,
                            &op.flow,
                            "mail.operation.failed",
                            json!({"application":op.app,"code":e.code}),
                        );
                    }
                }
            }
            Completion::Relay(parent) => {
                if let Some(mut delivery) = self.mail.deliveries.remove(&parent) {
                    delivery.remaining -= 1;
                    delivery.failed |= result.is_err();
                    if delivery.remaining > 0 {
                        self.mail.deliveries.insert(parent, delivery);
                    } else {
                        let result = if delivery.failed {
                            Err(diag("mail.relay_failed", "recipient"))
                        } else {
                            self.mail_deliver(
                                c,
                                &delivery.device,
                                &delivery.app,
                                &delivery.message,
                                &delivery.local,
                                &delivery.flow,
                                out,
                            )
                        };
                        send(
                            out,
                            &delivery.device,
                            &delivery.socket,
                            &delivery.flow,
                            if result.is_ok() {
                                "250 delivered\r\n"
                            } else {
                                "550 relay failed\r\n"
                            },
                        );
                        if let Some(conn) = self
                            .mail
                            .connections
                            .get_mut(&(delivery.device.clone(), delivery.socket.clone()))
                            && let ConnectionKind::SmtpServer(s) = &mut conn.kind
                        {
                            s.waiting = false;
                            s.from = None;
                            s.recipients.clear();
                        }
                        fact(
                            out,
                            &delivery.device,
                            &delivery.flow,
                            "mail.relay.completed",
                            json!({"message":delivery.message.id,"success":result.is_ok()}),
                        );
                    }
                }
            }
        }
        Ok(())
    }
    pub(super) fn mail_notice(
        &mut self,
        c: Context<'_>,
        n: &SocketNotice,
        out: &mut Output,
    ) -> Result<bool, Diagnostic> {
        let key = (n.device.clone(), n.socket.clone());
        if matches!(n.kind, SocketNoticeKind::Accepted)
            && metadata(c.project, &n.device, &n.owner).is_ok_and(|m| m.kind == "mail_server")
        {
            if self.mail.connections.len() >= 256 {
                close(out, &n.device, &n.socket, &n.correlation);
                return Ok(true);
            }
            let cfg = server_config(c, &n.device, &n.owner)?;
            let port = c
                .net
                .transport
                .views()
                .into_iter()
                .find(|s| s.device == n.device && s.socket == n.socket)
                .map(|s| s.local.port)
                .unwrap_or(0);
            let (kind, greeting) = if port == cfg.smtp_port {
                (
                    ConnectionKind::SmtpServer(Default::default()),
                    "220 simulated SMTP ready\r\n",
                )
            } else if port == cfg.pop3_port {
                (
                    ConnectionKind::PopServer(Default::default()),
                    "+OK simulated POP3 ready\r\n",
                )
            } else {
                return Ok(false);
            };
            self.serial += 1;
            let token = self.serial;
            self.mail.connections.insert(
                key,
                Connection {
                    app: n.owner.clone(),
                    flow: n.correlation.clone(),
                    token,
                    buffer: vec![],
                    kind,
                },
            );
            send(out, &n.device, &n.socket, &n.correlation, greeting);
            out.effects.push(Effect::Timer(Timer {
                device: n.device.clone(),
                application: n.owner.clone(),
                token,
                kind: TimerKind::MailSession {
                    socket: n.socket.clone(),
                },
                delay_us: 30_000_000,
                correlation: n.correlation.clone(),
            }));
            return Ok(true);
        }
        let Some(mut conn) = self.mail.connections.remove(&key) else {
            return Ok(metadata(c.project, &n.device, &n.owner)
                .is_ok_and(|m| matches!(m.kind.as_str(), "mail_server" | "mail_client"))
                && matches!(
                    n.kind,
                    SocketNoticeKind::Data { .. } | SocketNoticeKind::PeerClosed
                ));
        };
        match &n.kind {
            SocketNoticeKind::Data { data_base64 } => {
                conn.buffer.extend(
                    STANDARD
                        .decode(data_base64)
                        .map_err(|_| diag("validation.invalid_base64", "mail"))?,
                );
                if conn.buffer.len() > 65_536 {
                    self.mail_connection_failed(
                        c,
                        n,
                        &conn,
                        diag("mail.message_too_large", "session"),
                        out,
                    )?;
                    return Ok(true);
                }
                while let Some(end) = conn.buffer.windows(2).position(|s| s == b"\r\n") {
                    let bytes = conn.buffer.drain(..end + 2).take(end).collect();
                    let Ok(line) = String::from_utf8(bytes) else {
                        self.mail_connection_failed(
                            c,
                            n,
                            &conn,
                            diag("mail.invalid_message", "UTF-8"),
                            out,
                        )?;
                        return Ok(true);
                    };
                    let protocol = if matches!(
                        conn.kind,
                        ConnectionKind::SmtpClient { .. } | ConnectionKind::SmtpServer(_)
                    ) {
                        "smtp"
                    } else {
                        "pop3"
                    };
                    fact(
                        out,
                        &n.device,
                        &conn.flow,
                        if protocol == "smtp" {
                            "smtp.line.received"
                        } else {
                            "pop3.line.received"
                        },
                        json!({"application":n.owner,"socket":n.socket,"line":if line.starts_with("PASS ")||line.starts_with("AUTH "){line.split_whitespace().next().unwrap_or("").to_string()+" [credentials]"}else{line.clone()}}),
                    );
                    let keep = match &mut conn.kind {
                        ConnectionKind::SmtpServer(s) => {
                            self.smtp_server_line(c, n, s, &line, out)?
                        }
                        ConnectionKind::PopServer(s) => {
                            self.pop_server_line(c, n, s, &line, out)?
                        }
                        ConnectionKind::SmtpClient { job, stage, index } => {
                            self.smtp_client_line(c, n, *job, stage, index, &line, out)?
                        }
                        ConnectionKind::PopClient {
                            job,
                            stage,
                            index,
                            count,
                            multiline,
                        } => self.pop_client_line(
                            c, n, *job, stage, index, count, multiline, &line, out,
                        )?,
                    };
                    if !keep {
                        self.mail_unlock(&n.device, &n.owner, &n.socket);
                        return Ok(true);
                    }
                }
            }
            SocketNoticeKind::PeerClosed | SocketNoticeKind::Closed { .. } => {
                self.mail_connection_failed(
                    c,
                    n,
                    &conn,
                    diag("mail.connection_closed", n.socket.as_str()),
                    out,
                )?;
                return Ok(true);
            }
            _ => {}
        }
        self.mail.connections.insert(key, conn);
        Ok(true)
    }
    fn mail_unlock(&mut self, d: &Id, a: &Id, s: &Id) {
        self.mail
            .locks
            .retain(|(host, app, _), socket| host != d || app != a || socket != s);
    }
    fn mail_connection_failed(
        &mut self,
        c: Context<'_>,
        n: &SocketNotice,
        conn: &Connection,
        error: Diagnostic,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        self.mail_unlock(&n.device, &n.owner, &n.socket);
        if let ConnectionKind::SmtpClient { job, .. } | ConnectionKind::PopClient { job, .. } =
            conn.kind
        {
            self.mail_finish(c, job, Err(error), out)?;
        }
        if !matches!(n.kind, SocketNoticeKind::Closed { .. }) {
            close(out, &n.device, &n.socket, &conn.flow);
        }
        Ok(())
    }
    fn smtp_server_line(
        &mut self,
        c: Context<'_>,
        n: &SocketNotice,
        s: &mut SmtpServer,
        line: &str,
        out: &mut Output,
    ) -> Result<bool, Diagnostic> {
        let cfg = server_config(c, &n.device, &n.owner)?;
        let own = dns_name(&cfg.domain)?;
        if let Some(lines) = &mut s.data {
            if line != "." {
                lines.push(line.strip_prefix('.').unwrap_or(line).to_string());
                if lines.iter().map(String::len).sum::<usize>() > 60_000 {
                    send(
                        out,
                        &n.device,
                        &n.socket,
                        &n.correlation,
                        "552 message too large\r\n",
                    );
                    s.data = None;
                    s.recipients.clear();
                }
                return Ok(true);
            }
            let text = lines.join("\r\n");
            s.data = None;
            self.serial += 1;
            let fallback = Id::new(format!("received-{}-{}", c.now, self.serial))?;
            let result = parse_message(&text, fallback, c.now);
            let Ok(mut msg) = result else {
                send(
                    out,
                    &n.device,
                    &n.socket,
                    &n.correlation,
                    "554 invalid message\r\n",
                );
                return Ok(true);
            };
            if s.from.as_ref() != Some(&msg.from)
                || msg.relay_path.contains(&own)
                || msg.relay_path.len() >= 8
            {
                send(
                    out,
                    &n.device,
                    &n.socket,
                    &n.correlation,
                    "554 invalid sender or relay loop\r\n",
                );
                return Ok(true);
            }
            msg.relay_path.push(own.clone());
            let mut local = vec![];
            let mut remote: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for recipient in &s.recipients {
                if domain(recipient) == own {
                    local.push(recipient.split('@').next().unwrap().to_string());
                } else {
                    remote
                        .entry(domain(recipient).to_string())
                        .or_default()
                        .push(recipient.clone());
                }
            }
            if remote.is_empty() {
                let result =
                    self.mail_deliver(c, &n.device, &n.owner, &msg, &local, &n.correlation, out);
                send(
                    out,
                    &n.device,
                    &n.socket,
                    &n.correlation,
                    if result.is_ok() {
                        "250 delivered\r\n"
                    } else {
                        "550 recipient or storage unavailable\r\n"
                    },
                );
                s.from = None;
                s.recipients.clear();
            } else {
                self.serial += 1;
                let parent = self.serial;
                self.mail.deliveries.insert(
                    parent,
                    Delivery {
                        device: n.device.clone(),
                        app: n.owner.clone(),
                        flow: n.correlation.clone(),
                        socket: n.socket.clone(),
                        message: msg.clone(),
                        local,
                        remaining: remote.len(),
                        failed: false,
                    },
                );
                s.waiting = true;
                for (domain, recipients) in remote {
                    self.serial += 1;
                    let job = self.serial;
                    fact(
                        out,
                        &n.device,
                        &n.correlation,
                        "mail.relay.started",
                        json!({"message":msg.id,"domain":domain}),
                    );
                    self.mail_begin(
                        c,
                        job,
                        Operation {
                            device: n.device.clone(),
                            app: n.owner.clone(),
                            flow: n.correlation.clone(),
                            kind: OperationKind::Smtp {
                                message: Box::new(msg.clone()),
                                envelope: recipients,
                                auth: None,
                            },
                            completion: Completion::Relay(parent),
                        },
                        &domain,
                        Resolve::Mx,
                        out,
                    )?;
                }
            }
            return Ok(true);
        }
        let upper = line.to_ascii_uppercase();
        let response = if upper == "QUIT" {
            send(out, &n.device, &n.socket, &n.correlation, "221 bye\r\n");
            close(out, &n.device, &n.socket, &n.correlation);
            return Ok(false);
        } else if s.waiting {
            "503 delivery pending\r\n"
        } else if upper.starts_with("EHLO ") || upper.starts_with("HELO ") {
            s.hello = true;
            "250 simulated SMTP\r\n"
        } else if upper.starts_with("AUTH PLAIN ") {
            let value = STANDARD
                .decode(&line[11..])
                .ok()
                .and_then(|b| String::from_utf8(b).ok());
            let valid = value
                .as_deref()
                .and_then(|s| {
                    let f: Vec<_> = s.split('\0').collect();
                    (f.len() == 3).then_some((f[1], f[2]))
                })
                .and_then(|(u, p)| {
                    cfg.accounts
                        .iter()
                        .find(|a| a.username.eq_ignore_ascii_case(u) && a.password == p)
                });
            if let Some(a) = valid {
                s.authenticated = Some(a.username.to_ascii_lowercase());
                "235 authenticated\r\n"
            } else {
                "535 authentication failed\r\n"
            }
        } else if upper.starts_with("MAIL FROM:") {
            let address = mail_address(
                line[10..]
                    .trim()
                    .trim_start_matches('<')
                    .trim_end_matches('>'),
            );
            if !s.hello {
                "503 EHLO first\r\n"
            } else if let Ok(address) = address {
                let authenticated = s.authenticated.as_ref().filter(|u| {
                    cfg.accounts
                        .iter()
                        .any(|a| a.username.eq_ignore_ascii_case(u))
                });
                if domain(&address) == own && authenticated.is_none() {
                    "530 authentication required\r\n"
                } else if authenticated.is_some_and(|u| format!("{u}@{own}") != address) {
                    "553 sender mismatch\r\n"
                } else {
                    s.from = Some(address);
                    s.recipients.clear();
                    "250 sender accepted\r\n"
                }
            } else {
                "501 invalid sender\r\n"
            }
        } else if upper.starts_with("RCPT TO:") {
            let address = mail_address(
                line[8..]
                    .trim()
                    .trim_start_matches('<')
                    .trim_end_matches('>'),
            );
            if s.from.is_none() {
                "503 MAIL first\r\n"
            } else if let Ok(address) = address {
                if s.recipients.len() >= 32 {
                    "452 too many recipients\r\n"
                } else if domain(&address) == own
                    && !cfg
                        .accounts
                        .iter()
                        .any(|a| format!("{}@{own}", a.username.to_ascii_lowercase()) == address)
                {
                    "550 unknown recipient\r\n"
                } else if domain(&address) != own && s.authenticated.is_none() {
                    "550 relay denied\r\n"
                } else {
                    if !s.recipients.contains(&address) {
                        s.recipients.push(address);
                    }
                    "250 recipient accepted\r\n"
                }
            } else {
                "501 invalid recipient\r\n"
            }
        } else if upper == "DATA" {
            if s.from.is_none() || s.recipients.is_empty() {
                "503 recipients required\r\n"
            } else {
                s.data = Some(vec![]);
                "354 end with dot\r\n"
            }
        } else if upper == "RSET" {
            s.from = None;
            s.recipients.clear();
            "250 reset\r\n"
        } else if upper == "NOOP" {
            "250 OK\r\n"
        } else {
            "502 unsupported command\r\n"
        };
        send(out, &n.device, &n.socket, &n.correlation, response);
        Ok(true)
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "SMTP client state machine context"
    )]
    fn smtp_client_line(
        &mut self,
        c: Context<'_>,
        n: &SocketNotice,
        job: u64,
        stage: &mut u8,
        index: &mut usize,
        line: &str,
        out: &mut Output,
    ) -> Result<bool, Diagnostic> {
        let Some(op) = self.mail.jobs.get(&job).cloned() else {
            close(out, &n.device, &n.socket, &n.correlation);
            return Ok(false);
        };
        let OperationKind::Smtp {
            message,
            envelope,
            auth,
        } = op.kind
        else {
            return Ok(false);
        };
        let expected = match *stage {
            0 => 220,
            2 => 235,
            5 => 354,
            _ => 250,
        };
        let code = line
            .get(..3)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        if code != expected {
            self.mail_finish(
                c,
                job,
                Err(diag(
                    if code == 535 || code == 530 {
                        "mail.authentication_failed"
                    } else if code == 550 {
                        "mail.unknown_recipient"
                    } else {
                        "mail.protocol_error"
                    },
                    line,
                )),
                out,
            )?;
            send(out, &n.device, &n.socket, &n.correlation, "QUIT\r\n");
            close(out, &n.device, &n.socket, &n.correlation);
            return Ok(false);
        }
        let next = match *stage {
            0 => {
                *stage = 1;
                "EHLO simulator\r\n".into()
            }
            1 => {
                if let Some((user, password)) = auth {
                    *stage = 2;
                    format!(
                        "AUTH PLAIN {}\r\n",
                        STANDARD.encode(format!("\0{user}\0{password}"))
                    )
                } else {
                    *stage = 3;
                    format!("MAIL FROM:<{}>\r\n", message.from)
                }
            }
            2 => {
                *stage = 3;
                format!("MAIL FROM:<{}>\r\n", message.from)
            }
            3 => {
                *stage = 4;
                *index = 0;
                format!("RCPT TO:<{}>\r\n", envelope[0])
            }
            4 => {
                *index += 1;
                if *index < envelope.len() {
                    format!("RCPT TO:<{}>\r\n", envelope[*index])
                } else {
                    *stage = 5;
                    "DATA\r\n".into()
                }
            }
            5 => {
                *stage = 6;
                block(&message_text(&message))
            }
            _ => {
                self.mail_finish(c, job, Ok(()), out)?;
                send(out, &n.device, &n.socket, &n.correlation, "QUIT\r\n");
                close(out, &n.device, &n.socket, &n.correlation);
                return Ok(false);
            }
        };
        send(out, &n.device, &n.socket, &n.correlation, &next);
        Ok(true)
    }
    fn pop_server_line(
        &mut self,
        c: Context<'_>,
        n: &SocketNotice,
        s: &mut PopServer,
        line: &str,
        out: &mut Output,
    ) -> Result<bool, Diagnostic> {
        let cfg = server_config(c, &n.device, &n.owner)?;
        let (command, value) = line.split_once(' ').unwrap_or((line, ""));
        let command = command.to_ascii_uppercase();
        if s.authenticated
            && s.username.as_ref().is_none_or(|u| {
                !cfg.accounts
                    .iter()
                    .any(|a| a.username.eq_ignore_ascii_case(u))
            })
        {
            send(
                out,
                &n.device,
                &n.socket,
                &n.correlation,
                "-ERR account removed\r\n",
            );
            self.mail_unlock(&n.device, &n.owner, &n.socket);
            close(out, &n.device, &n.socket, &n.correlation);
            return Ok(false);
        }
        let response = match command.as_str() {
            "QUIT" => {
                if s.authenticated {
                    let ApplicationData::MailServer(data) =
                        self.mail_data(c, &n.device, &n.owner)?
                    else {
                        return Ok(false);
                    };
                    if let Some(list) = data.mailboxes.get_mut(s.username.as_ref().unwrap()) {
                        list.retain(|m| !s.deleted.contains(&m.id));
                    }
                    self.mail_persist(c, &n.device, &n.owner, out)?;
                }
                send(out, &n.device, &n.socket, &n.correlation, "+OK bye\r\n");
                self.mail_unlock(&n.device, &n.owner, &n.socket);
                close(out, &n.device, &n.socket, &n.correlation);
                return Ok(false);
            }
            "USER" if !s.authenticated => {
                s.username = Some(value.to_ascii_lowercase());
                "+OK password required\r\n".into()
            }
            "PASS" if !s.authenticated => {
                if let Some(a) = cfg.accounts.iter().find(|a| {
                    Some(a.username.to_ascii_lowercase()) == s.username && a.password == value
                }) {
                    let user = a.username.to_ascii_lowercase();
                    let key = (n.device.clone(), n.owner.clone(), user.clone());
                    if let std::collections::btree_map::Entry::Vacant(e) =
                        self.mail.locks.entry(key)
                    {
                        e.insert(n.socket.clone());
                        s.authenticated = true;
                        if let ApplicationData::MailServer(data) =
                            self.mail_data(c, &n.device, &n.owner)?
                        {
                            s.messages = data.mailboxes.get(&user).cloned().unwrap_or_default();
                        }
                        "+OK authenticated\r\n".into()
                    } else {
                        "-ERR mailbox locked\r\n".into()
                    }
                } else {
                    "-ERR authentication failed\r\n".into()
                }
            }
            _ if !s.authenticated => "-ERR authentication required\r\n".into(),
            "STAT" => format!(
                "+OK {} {}\r\n",
                s.messages
                    .iter()
                    .filter(|m| !s.deleted.contains(&m.id))
                    .count(),
                s.messages
                    .iter()
                    .filter(|m| !s.deleted.contains(&m.id))
                    .map(|m| message_text(m).len())
                    .sum::<usize>()
            ),
            "LIST" | "UIDL" => {
                let mut text = "+OK messages\r\n".to_string();
                for (i, m) in s
                    .messages
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| !s.deleted.contains(&m.id))
                {
                    text += &format!(
                        "{} {}\r\n",
                        i + 1,
                        if command == "UIDL" {
                            m.id.as_str().to_string()
                        } else {
                            message_text(m).len().to_string()
                        }
                    );
                }
                text + ".\r\n"
            }
            "RETR" | "DELE" => {
                let message = value
                    .parse::<usize>()
                    .ok()
                    .and_then(|i| i.checked_sub(1))
                    .and_then(|i| s.messages.get(i))
                    .filter(|m| !s.deleted.contains(&m.id));
                if let Some(m) = message {
                    if command == "DELE" {
                        s.deleted.insert(m.id.clone());
                        "+OK marked\r\n".into()
                    } else {
                        format!("+OK message\r\n{}", block(&message_text(m)))
                    }
                } else {
                    "-ERR unknown message\r\n".into()
                }
            }
            "RSET" => {
                s.deleted.clear();
                "+OK reset\r\n".into()
            }
            "NOOP" => "+OK\r\n".into(),
            _ => "-ERR unsupported command\r\n".into(),
        };
        send(out, &n.device, &n.socket, &n.correlation, &response);
        Ok(true)
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "Explicit POP3 session state stays local to the protocol connection."
    )]
    fn pop_client_line(
        &mut self,
        c: Context<'_>,
        n: &SocketNotice,
        job: u64,
        stage: &mut u8,
        index: &mut usize,
        count: &mut usize,
        multiline: &mut Option<Vec<String>>,
        line: &str,
        out: &mut Output,
    ) -> Result<bool, Diagnostic> {
        let Some(op) = self.mail.jobs.get(&job).cloned() else {
            close(out, &n.device, &n.socket, &n.correlation);
            return Ok(false);
        };
        let OperationKind::Pop { username, password } = op.kind else {
            return Ok(false);
        };
        if let Some(lines) = multiline {
            if line != "." {
                lines.push(line.strip_prefix('.').unwrap_or(line).to_string());
                if lines.iter().map(String::len).sum::<usize>() > 60_000 {
                    self.mail_finish(c, job, Err(diag("mail.message_too_large", "POP3")), out)?;
                    close(out, &n.device, &n.socket, &n.correlation);
                    return Ok(false);
                }
                return Ok(true);
            }
            let text = lines.join("\r\n");
            *multiline = None;
            self.serial += 1;
            let fallback = Id::new(format!("pop-{}-{}", c.now, self.serial))?;
            let received = (|| -> Result<(), Diagnostic> {
                let message = parse_message(&text, fallback, c.now)?;
                let mut candidate = self.mail_data(c, &n.device, &n.owner)?.clone();
                if let ApplicationData::MailClient(data) = &mut candidate
                    && !data.inbox.iter().any(|m| m.id == message.id)
                {
                    data.inbox.push(message);
                }
                candidate.validate("mail_client")?;
                self.mail
                    .stores
                    .insert((n.device.clone(), n.owner.clone()), candidate);
                self.mail_persist(c, &n.device, &n.owner, out)
            })();
            if let Err(error) = received {
                self.mail_finish(c, job, Err(error), out)?;
                close(out, &n.device, &n.socket, &n.correlation);
                return Ok(false);
            }
            *stage = 5;
            send(
                out,
                &n.device,
                &n.socket,
                &n.correlation,
                &format!("DELE {}\r\n", index),
            );
            return Ok(true);
        }
        if !line.starts_with("+OK") {
            self.mail_finish(
                c,
                job,
                Err(diag(
                    if *stage <= 2 {
                        "mail.authentication_failed"
                    } else {
                        "mail.protocol_error"
                    },
                    line,
                )),
                out,
            )?;
            close(out, &n.device, &n.socket, &n.correlation);
            return Ok(false);
        }
        let next = match *stage {
            0 => {
                *stage = 1;
                format!("USER {username}\r\n")
            }
            1 => {
                *stage = 2;
                format!("PASS {password}\r\n")
            }
            2 => {
                *stage = 3;
                "STAT\r\n".into()
            }
            3 => {
                let Some(parsed) = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse().ok())
                    .filter(|n| *n <= 128)
                else {
                    self.mail_finish(c, job, Err(diag("mail.protocol_error", "STAT")), out)?;
                    close(out, &n.device, &n.socket, &n.correlation);
                    return Ok(false);
                };
                *count = parsed;
                if *count == 0 {
                    *stage = 6;
                    "QUIT\r\n".into()
                } else {
                    *stage = 4;
                    format!("RETR {}\r\n", index)
                }
            }
            4 => {
                *multiline = Some(vec![]);
                return Ok(true);
            }
            5 => {
                *index += 1;
                if *index > *count {
                    *stage = 6;
                    "QUIT\r\n".into()
                } else {
                    *stage = 4;
                    format!("RETR {}\r\n", index)
                }
            }
            _ => {
                self.mail_finish(c, job, Ok(()), out)?;
                close(out, &n.device, &n.socket, &n.correlation);
                return Ok(false);
            }
        };
        send(out, &n.device, &n.socket, &n.correlation, &next);
        Ok(true)
    }
    pub(super) fn mail_timer(
        &mut self,
        c: Context<'_>,
        timer: &Timer,
    ) -> Result<Output, Diagnostic> {
        let mut out = Output::default();
        match &timer.kind {
            TimerKind::MailJob { job } => {
                let op = self.mail.jobs.get(job).cloned();
                self.mail_finish(c, *job, Err(diag("mail.timeout", "operation")), &mut out)?;
                if let Some(op) = op {
                    let socket_id = Id::new(format!("mail-client-{job}"))?;
                    if self
                        .mail
                        .connections
                        .remove(&(op.device.clone(), socket_id.clone()))
                        .is_some()
                    {
                        socket(
                            &mut out,
                            &op.device,
                            &op.flow,
                            SocketCommand::Close {
                                socket: socket_id,
                                force: true,
                            },
                        );
                    }
                }
            }
            TimerKind::MailSession { socket: socket_id } => {
                let key = (timer.device.clone(), socket_id.clone());
                if let Some(conn) = self
                    .mail
                    .connections
                    .get(&key)
                    .filter(|s| s.token == timer.token)
                    .cloned()
                {
                    self.mail.connections.remove(&key);
                    self.mail_unlock(&timer.device, &conn.app, socket_id);
                    socket(
                        &mut out,
                        &timer.device,
                        &conn.flow,
                        SocketCommand::Close {
                            socket: socket_id.clone(),
                            force: true,
                        },
                    );
                }
            }
            _ => {}
        }
        Ok(out)
    }
}
