//! Host applications, independent of windows, host I/O and wall time.
mod p2p;
pub use p2p::P2pView;
mod mail;
pub use mail::MailView;
mod dns;
mod web;
pub use web::{BrowserView, ImageResource};
mod terminal;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Serialize;
use serde_json::{Value, json};
use sns_model::*;
use sns_protocols::{Action, Fact, Network};
use sns_vfs::{Entry, Vfs};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
pub use terminal::{COMMAND_HELP, parse_words, usage};
pub const MAX_LOG_LINES: usize = 1024;
pub const MAX_RECEIVED_BYTES: usize = 1_048_576;
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Line {
    Message {
        key: String,
        params: BTreeMap<String, String>,
    },
    Text {
        value: String,
    },
    Prompt {
        cwd: String,
        value: String,
    },
    Help {
        command: Option<String>,
    },
    Error {
        diagnostic: Diagnostic,
    },
    Json {
        value: Value,
    },
    Capture {
        time_us: Decimal,
        event: String,
        correlation: Id,
        payload: BTreeMap<String, Value>,
    },
}
#[derive(Clone, Debug, Serialize)]
pub struct AppView {
    pub p2p: P2pView,
    pub mail: MailView,
    pub browser: BrowserView,
    pub id: Id,
    pub software: String,
    pub visible: bool,
    pub running: bool,
    pub busy: bool,
    pub lines: Vec<Line>,
    pub history: Vec<String>,
    pub cwd: String,
    pub received_base64: String,
    pub omitted_lines: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct HostView {
    pub device: Id,
    pub applications: Vec<AppView>,
    pub files: Vec<Entry>,
}
#[derive(Clone, serde::Serialize)]
struct AppState {
    p2p: P2pView,
    mail: MailView,
    browser: BrowserView,
    visible: bool,
    running: bool,
    lines: Vec<Line>,
    history: Vec<String>,
    cwd: String,
    received: Vec<u8>,
    omitted: u64,
    job: Option<Job>,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            p2p: P2pView::default(),
            mail: MailView::default(),
            browser: BrowserView::default(),
            visible: false,
            running: false,
            lines: vec![],
            history: vec![],
            cwd: "/".into(),
            received: vec![],
            omitted: 0,
            job: None,
        }
    }
}
#[derive(Clone, serde::Serialize)]
enum Job {
    Trace {
        token: u64,
        target: Ipv4Addr,
        max: u8,
        done: bool,
    },
    Ping {
        token: u64,
        target: Ipv4Addr,
        count: u16,
        sent: u16,
        received: u32,
        replied: BTreeSet<u16>,
        broadcast: bool,
    },
    Capture,
}
#[derive(Clone, serde::Serialize)]
struct Host {
    files: Vfs,
    apps: BTreeMap<Id, AppState>,
}
#[derive(Clone, Default, serde::Serialize)]
pub struct Runtime {
    p2p: p2p::P2p,
    mail: mail::Mail,
    dns: dns::Dns,
    web: web::Web,
    hosts: BTreeMap<Id, Host>,
    serial: u64,
}
#[derive(Clone, Debug, serde::Serialize)]
pub enum TimerKind {
    P2pSession { socket: Id },
    P2pSearch,
    MailJob { job: u64 },
    MailSession { socket: Id },
    DnsTimeout { socket: Id },
    WebServerTimeout { socket: Id },
    WebTimeout { socket: Id },
    TraceSend { ttl: u8 },
    TraceNext { ttl: u8 },
    PingSend { index: u16 },
    PingDone,
}
impl TimerKind {
    pub fn waits_for_response(&self) -> bool {
        match self {
            Self::PingSend { .. } | Self::TraceSend { .. } => false,
            Self::P2pSession { .. }
            | Self::P2pSearch
            | Self::MailJob { .. }
            | Self::MailSession { .. }
            | Self::DnsTimeout { .. }
            | Self::WebServerTimeout { .. }
            | Self::WebTimeout { .. }
            | Self::TraceNext { .. }
            | Self::PingDone => true,
        }
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Timer {
    pub device: Id,
    pub application: Id,
    pub token: u64,
    pub kind: TimerKind,
    pub delay_us: u64,
    pub correlation: Id,
}
pub enum Effect {
    Files {
        device: Id,
        files: Vec<VirtualFile>,
    },
    ApplicationData {
        device: Id,
        application: Id,
        data: ApplicationData,
    },
    Security {
        device: Id,
        command: SecurityCommand,
        correlation: Id,
    },
    Network {
        action: Action,
        correlation: Id,
    },
    Timer(Timer),
}
#[derive(Default)]
pub struct Output {
    pub effects: Vec<Effect>,
    pub facts: Vec<Fact>,
}
fn diag(code: &str, target: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, target)
}
fn fact(out: &mut Output, device: &Id, flow: &Id, kind: &'static str, value: Value) {
    out.facts.push(Fact {
        event_schema_version: if kind.starts_with("p2p.")
            || kind.starts_with("mail.")
            || kind.starts_with("smtp.")
            || kind.starts_with("pop3.")
        {
            8
        } else if kind.starts_with("admin.") {
            7
        } else if kind.starts_with("dns.") || kind.starts_with("http.") {
            6
        } else {
            4
        },
        correlation: flow.clone(),
        kind,
        device: device.clone(),
        payload: serde_json::from_value(value).unwrap(),
    });
}
fn network(out: &mut Output, flow: &Id, action: Action) {
    out.effects.push(Effect::Network {
        action,
        correlation: flow.clone(),
    });
}
fn socket(out: &mut Output, device: &Id, flow: &Id, command: SocketCommand) {
    network(
        out,
        flow,
        Action::Socket {
            device: device.clone(),
            command,
        },
    );
}
fn message(key: &str, params: &[(&str, String)]) -> Line {
    Line::Message {
        key: key.into(),
        params: params
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    }
}
fn push(state: &mut AppState, line: Line) {
    if state.lines.len() == MAX_LOG_LINES {
        state.lines.remove(0);
        state.omitted += 1;
    }
    state.lines.push(line);
}
pub fn resolve_name(project: &Project, device: &Id, target: &str) -> Result<Ipv4Addr, Diagnostic> {
    if target.eq_ignore_ascii_case("localhost") {
        return Ok(Ipv4Addr::LOCALHOST);
    }
    if let Ok(address) = target.parse() {
        return Ok(address);
    }
    let candidates: Vec<_> = project
        .devices
        .iter()
        .filter(|d| {
            d.id.as_str().eq_ignore_ascii_case(target) || d.label.eq_ignore_ascii_case(target)
        })
        .collect();
    if candidates.len() != 1 {
        return Err(diag("resolver.name_not_found", target));
    }
    candidates[0]
        .interfaces
        .iter()
        .find_map(|i| match &i.ipv4 {
            Some(Ipv4Config::Static { address, .. }) => address.parse().ok(),
            _ => None,
        })
        .ok_or_else(|| diag("routing.no_route", device.as_str()))
}
impl Runtime {
    pub fn new(project: &Project) -> Result<Self, Diagnostic> {
        let mut hosts = BTreeMap::new();
        for d in &project.devices {
            if matches!(
                d.kind,
                DeviceKind::Workstation
                    | DeviceKind::Notebook
                    | DeviceKind::Router
                    | DeviceKind::Gateway
            ) {
                let apps = d
                    .applications
                    .iter()
                    .map(|a| (a.id.clone(), AppState::default()))
                    .collect();
                hosts.insert(
                    d.id.clone(),
                    Host {
                        files: Vfs::from_files(&d.files)?,
                        apps,
                    },
                );
            }
        }
        Ok(Self {
            p2p: p2p::P2p::default(),
            hosts,
            serial: 0,
            mail: Default::default(),
            dns: Default::default(),
            web: Default::default(),
        })
    }
    pub fn startup(&mut self, project: &Project) -> Result<Output, Diagnostic> {
        let mut out = Output::default();
        for d in &project.devices {
            for a in &d.applications {
                if let Some(ServiceConfig::Http(config)) = &a.service
                    && config.administration
                {
                    self.state(&d.id, &a.id)?.running = true;
                    socket(
                        &mut out,
                        &d.id,
                        &Id::new("administration-boot")?,
                        SocketCommand::Bind {
                            socket: a.id.clone(),
                            owner: a.id.clone(),
                            protocol: TransportProtocol::Tcp,
                            interface: None,
                            address: None,
                            port: config.port,
                        },
                    );
                }
            }
        }
        Ok(out)
    }
    pub fn views(&self, project: &Project) -> Vec<HostView> {
        self.hosts
            .iter()
            .filter_map(|(id, host)| {
                project.device(id).map(|d| HostView {
                    device: id.clone(),
                    files: host.files.list("/").unwrap_or_default(),
                    applications: d
                        .applications
                        .iter()
                        .filter_map(|a| {
                            host.apps.get(&a.id).map(|s| AppView {
                                p2p: s.p2p.clone(),
                                mail: s.mail.clone(),
                                browser: s.browser.clone(),
                                id: a.id.clone(),
                                software: a.kind.clone(),
                                visible: s.visible,
                                running: if a.kind == "firewall" {
                                    d.network.firewall.as_ref().is_some_and(|f| f.enabled)
                                } else {
                                    s.running
                                },
                                busy: s.job.is_some() || s.mail.busy,
                                lines: s.lines.clone(),
                                history: s.history.clone(),
                                cwd: s.cwd.clone(),
                                received_base64: STANDARD.encode(&s.received),
                                omitted_lines: s.omitted,
                            })
                        })
                        .collect(),
                })
            })
            .collect()
    }
    pub fn files(&self, device: &Id) -> Option<&Vfs> {
        self.hosts.get(device).map(|h| &h.files)
    }
    fn state(&mut self, device: &Id, app: &Id) -> Result<&mut AppState, Diagnostic> {
        self.hosts
            .get_mut(device)
            .and_then(|h| h.apps.get_mut(app))
            .ok_or_else(|| diag("application.not_installed", app.as_str()))
    }
    pub fn command(
        &mut self,
        project: &mut Project,
        net: &Network,
        now: u64,
        device: &Id,
        command: &AppCommand,
        flow: &Id,
    ) -> Result<Output, Diagnostic> {
        if !self.hosts.contains_key(device) {
            return Err(diag("device.not_host", device.as_str()));
        }
        let mut out = Output::default();
        if let AppCommand::Install {
            application,
            software,
        } = command
        {
            if project
                .device(device)
                .unwrap()
                .applications
                .iter()
                .any(|a| a.kind == software.id())
            {
                return Err(diag("application.already_installed", software.id()));
            }
            let host = project
                .devices
                .iter_mut()
                .find(|d| &d.id == device)
                .unwrap();
            host.applications.push(Application {
                data: None,
                service: None,
                id: application.clone(),
                kind: software.id().into(),
                config: BTreeMap::new(),
            });
            self.hosts
                .get_mut(device)
                .unwrap()
                .apps
                .insert(application.clone(), AppState::default());
            upgrade(project);
            if matches!(
                software,
                Software::MailServer | Software::MailClient | Software::P2p
            ) {
                advanced_profile(project);
            }
            if *software == Software::Firewall {
                private_profile(project);
            }
            if matches!(
                software,
                Software::DnsServer | Software::WebServer | Software::WebBrowser
            ) && project.schema_version < 5
            {
                project.schema_version = 5;
                project.semantic_profile = "sns-educational-services-v1".into();
            }
            project.validate()?;
            fact(
                &mut out,
                device,
                flow,
                "application.installed",
                json!({"application":application,"software":software}),
            );
            return Ok(out);
        }
        let application = match command {
            AppCommand::P2p { application, .. }
            | AppCommand::Mail { application, .. }
            | AppCommand::ConfigureService { application, .. }
            | AppCommand::Navigate { application, .. }
            | AppCommand::PurgeDns { application }
            | AppCommand::Uninstall { application }
            | AppCommand::OpenView { application }
            | AppCommand::CloseView { application }
            | AppCommand::StartServer { application, .. }
            | AppCommand::Stop { application }
            | AppCommand::Connect { application, .. }
            | AppCommand::Send { application, .. }
            | AppCommand::ClearReceived { application }
            | AppCommand::File { application, .. }
            | AppCommand::Terminal { application, .. }
            | AppCommand::Interrupt { application } => application,
            AppCommand::Install { .. } => unreachable!(),
        };
        let metadata = project
            .device(device)
            .unwrap()
            .applications
            .iter()
            .find(|a| &a.id == application)
            .cloned()
            .ok_or_else(|| diag("application.not_installed", application.as_str()))?;
        let software = Software::parse(&metadata.kind);
        match command {
            AppCommand::P2p { command, .. } => {
                advanced_profile(project);
                self.p2p_command(project, net, device, application, command, flow, &mut out)?;
            }
            AppCommand::Mail { command, .. } => {
                advanced_profile(project);
                self.mail_command(
                    mail::Context { project, net, now },
                    device,
                    application,
                    command,
                    flow,
                    &mut out,
                )?;
            }
            AppCommand::ConfigureService { config, .. } => {
                config.validate(&metadata.kind)?;
                if matches!(config, ServiceConfig::P2p(_)) {
                    if self.state(device, application)?.running {
                        return Err(diag("application.already_running", application.as_str()));
                    }
                    advanced_profile(project);
                }
                if matches!(
                    config,
                    ServiceConfig::MailServer(_) | ServiceConfig::MailClient(_)
                ) {
                    if self.state(device, application)?.mail.busy {
                        return Err(diag("mail.busy", application.as_str()));
                    }
                    if self.state(device, application)?.running
                        && matches!((&metadata.service,config),(Some(ServiceConfig::MailServer(old)),ServiceConfig::MailServer(new)) if old.domain!=new.domain||old.smtp_port!=new.smtp_port||old.pop3_port!=new.pop3_port)
                    {
                        return Err(diag("application.already_running", application.as_str()));
                    }
                    advanced_profile(project);
                    self.mail_reconfigure(project, device, application, config)?;
                }
                if matches!(config,ServiceConfig::Http(c) if c.administration) {
                    private_profile(project);
                }
                if self.state(device, application)?.running
                    && matches!(config, ServiceConfig::Http(_))
                {
                    return Err(diag("application.already_running", application.as_str()));
                }
                if project.schema_version < 5 {
                    project.schema_version = 5;
                    project.semantic_profile = "sns-educational-services-v1".into();
                }
                project
                    .devices
                    .iter_mut()
                    .find(|d| &d.id == device)
                    .unwrap()
                    .applications
                    .iter_mut()
                    .find(|a| &a.id == application)
                    .unwrap()
                    .service = Some(config.clone());
            }
            AppCommand::Navigate { url, fields, .. } => {
                if software != Some(Software::WebBrowser) {
                    return Err(diag("application.invalid_operation", application.as_str()));
                }
                self.navigate(
                    project,
                    net,
                    now,
                    device,
                    application,
                    url,
                    fields.clone(),
                    flow,
                    &mut out,
                )?;
            }
            AppCommand::PurgeDns { .. } => {
                self.purge_dns(device);
                fact(&mut out, device, flow, "dns.cache.purged", json!({}));
            }
            AppCommand::Uninstall { .. } => {
                self.p2p_cancel(device, application);
                self.mail_cancel(device, application, true);
                if software == Some(Software::Firewall)
                    && let Some(mut config) = project
                        .device(device)
                        .unwrap()
                        .network
                        .firewall
                        .as_deref()
                        .cloned()
                {
                    config.enabled = false;
                    out.effects.push(Effect::Security {
                        device: device.clone(),
                        command: SecurityCommand::ConfigureFirewall { config },
                        correlation: flow.clone(),
                    });
                }

                self.cancel_service_queries(device, application);
                self.cancel_web(device, application);
                socket(
                    &mut out,
                    device,
                    flow,
                    SocketCommand::CloseOwner {
                        owner: application.clone(),
                    },
                );
                self.hosts.get_mut(device).unwrap().apps.remove(application);
                project
                    .devices
                    .iter_mut()
                    .find(|d| &d.id == device)
                    .unwrap()
                    .applications
                    .retain(|a| &a.id != application);
                upgrade(project);
                fact(
                    &mut out,
                    device,
                    flow,
                    "application.uninstalled",
                    json!({"application":application}),
                );
            }
            AppCommand::CloseView { .. } => self.state(device, application)?.visible = false,
            AppCommand::Stop { .. } => {
                self.p2p_cancel(device, application);
                self.mail_cancel(device, application, false);
                if software == Some(Software::Firewall)
                    && let Some(mut config) = project
                        .device(device)
                        .unwrap()
                        .network
                        .firewall
                        .as_deref()
                        .cloned()
                {
                    config.enabled = false;
                    out.effects.push(Effect::Security {
                        device: device.clone(),
                        command: SecurityCommand::ConfigureFirewall { config },
                        correlation: flow.clone(),
                    });
                }

                self.cancel_service_queries(device, application);
                self.cancel_web(device, application);
                let state = self.state(device, application)?;
                state.running = false;
                state.job = None;
                socket(
                    &mut out,
                    device,
                    flow,
                    SocketCommand::CloseOwner {
                        owner: application.clone(),
                    },
                );
            }
            AppCommand::OpenView { .. } => {
                if software.is_none_or(|s| !s.available()) {
                    return Err(diag("application.unavailable", metadata.kind));
                }
                let state = self.state(device, application)?;
                state.visible = true;
                if software == Some(Software::Terminal) && state.lines.is_empty() {
                    push(state, message("terminal_welcome", &[]));
                }
            }
            AppCommand::StartServer { port, .. } => {
                if !matches!(
                    software,
                    Some(Software::TcpServer | Software::DnsServer | Software::WebServer)
                ) {
                    return Err(diag("application.invalid_operation", application.as_str()));
                }
                if self.state(device, application)?.running {
                    return Err(diag("application.already_running", application.as_str()));
                }
                let service_port = match (&software, &metadata.service) {
                    (Some(Software::DnsServer), Some(ServiceConfig::Dns(_))) => 53,
                    (Some(Software::WebServer), Some(ServiceConfig::Http(c))) => c.port,
                    (Some(Software::TcpServer), _) => *port,
                    _ => return Err(diag("service.invalid_config", application.as_str())),
                };
                self.state(device, application)?.running = true;
                set_config(
                    project,
                    device,
                    application,
                    "port",
                    service_port.to_string(),
                );
                socket(
                    &mut out,
                    device,
                    flow,
                    SocketCommand::Bind {
                        socket: application.clone(),
                        owner: application.clone(),
                        protocol: if software == Some(Software::DnsServer) {
                            TransportProtocol::Udp
                        } else {
                            TransportProtocol::Tcp
                        },
                        interface: None,
                        address: None,
                        port: service_port,
                    },
                );
            }
            AppCommand::Connect { target, port, .. } => {
                if software != Some(Software::TcpClient) {
                    return Err(diag("application.invalid_operation", application.as_str()));
                }
                if needs_dns(project, net, device, target) {
                    self.state(device, application)?.running = true;
                    self.dns_query(
                        project,
                        net,
                        now,
                        device,
                        application,
                        target,
                        DnsType::A,
                        None,
                        dns::Purpose::Connect { port: *port },
                        flow,
                        &mut out,
                    )?;
                    return Ok(out);
                }
                let destination = resolve_name(&net.effective_project(project), device, target)?;
                self.state(device, application)?.running = true;
                set_config(project, device, application, "target", target.clone());
                set_config(project, device, application, "port", port.to_string());
                socket(
                    &mut out,
                    device,
                    flow,
                    SocketCommand::Connect {
                        socket: application.clone(),
                        owner: application.clone(),
                        destination,
                        port: *port,
                        local_address: None,
                        local_port: None,
                    },
                );
            }
            AppCommand::Send { text, .. } => {
                if software != Some(Software::TcpClient) {
                    return Err(diag("application.invalid_operation", application.as_str()));
                }
                if text.len() > MAX_RECEIVED_BYTES {
                    return Err(diag("resource.limit_exceeded", application.as_str()));
                }
                socket(
                    &mut out,
                    device,
                    flow,
                    SocketCommand::Send {
                        socket: application.clone(),
                        data_base64: STANDARD.encode(text.as_bytes()),
                    },
                );
            }
            AppCommand::ClearReceived { .. } => {
                let s = self.state(device, application)?;
                s.received.clear();
                s.lines.clear();
                s.omitted = 0;
            }
            AppCommand::File { command, .. } => {
                if !matches!(
                    software,
                    Some(Software::FileExplorer | Software::TextEditor | Software::ImageViewer)
                ) {
                    return Err(diag("application.invalid_operation", application.as_str()));
                }
                self.file(project, device, command)?;
                fact(
                    &mut out,
                    device,
                    flow,
                    "filesystem.changed",
                    json!({"application":application,"command":command}),
                );
            }
            AppCommand::Terminal { line, .. } => {
                if software != Some(Software::Terminal) {
                    return Err(diag("application.invalid_operation", application.as_str()));
                }
                if line.len() > 4096 {
                    return Err(diag("resource.limit_exceeded", "terminal_input"));
                }
                let s = self.state(device, application)?;
                s.visible = true;
                if s.history.len() == 256 {
                    s.history.remove(0);
                }
                if !line.trim().is_empty() {
                    s.history.push(line.clone());
                }
                push(
                    s,
                    Line::Prompt {
                        cwd: s.cwd.clone(),
                        value: line.clone(),
                    },
                );
                if let Err(error) = self.evaluate(
                    terminal::Request {
                        project,
                        net,
                        now,
                        device,
                        application,
                        line,
                        flow,
                    },
                    &mut out,
                ) {
                    push(
                        self.state(device, application)?,
                        Line::Error { diagnostic: error },
                    );
                }
            }
            AppCommand::Interrupt { .. } => {
                self.interrupt(device, application)?;
                socket(
                    &mut out,
                    device,
                    flow,
                    SocketCommand::CloseOwner {
                        owner: application.clone(),
                    },
                );
            }
            AppCommand::Install { .. } => unreachable!(),
        }
        fact(
            &mut out,
            device,
            flow,
            "application.command",
            json!({"application":application,"command":command}),
        );
        Ok(out)
    }
    fn file(
        &mut self,
        project: &mut Project,
        device: &Id,
        command: &FileCommand,
    ) -> Result<(), Diagnostic> {
        let files = &mut self.hosts.get_mut(device).unwrap().files;
        match command {
            FileCommand::Mkdir { path } => files.mkdir(path)?,
            FileCommand::Write {
                path,
                media_type,
                data_base64,
                overwrite,
            } => {
                if data_base64.len() > 1_398_104 {
                    return Err(diag("resource.limit_exceeded", path));
                }
                let bytes = STANDARD
                    .decode(data_base64)
                    .map_err(|_| diag("validation.invalid_base64", path))?;
                files.write(path, &bytes, media_type, *overwrite)?;
            }
            FileCommand::Copy {
                source,
                destination,
            } => files.copy(source, destination)?,
            FileCommand::Move {
                source,
                destination,
            } => files.move_entry(source, destination)?,
            FileCommand::Remove { path, recursive } => files.remove(path, *recursive)?,
        }
        project
            .devices
            .iter_mut()
            .find(|d| &d.id == device)
            .unwrap()
            .files = files.to_files();
        upgrade(project);
        Ok(())
    }
    pub fn notices(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        notices: Vec<SocketNotice>,
    ) -> Result<Output, Diagnostic> {
        let mut out = Output::default();
        for notice in notices {
            if self.p2p_notice(project, &notice, &mut out)? {
                continue;
            }
            if self.mail_notice(mail::Context { project, net, now }, &notice, &mut out)? {
                continue;
            }
            if self.dns_notice(project, net, now, &notice, &mut out)?
                || self.web_notice(project, net, now, &notice, &mut out)?
            {
                continue;
            }
            let Some(host) = self.hosts.get_mut(&notice.device) else {
                continue;
            };
            let Some(state) = host.apps.get_mut(&notice.owner) else {
                continue;
            };
            let software = project
                .device(&notice.device)
                .and_then(|d| d.applications.iter().find(|a| a.id == notice.owner))
                .and_then(|a| Software::parse(&a.kind));
            match notice.kind {
                SocketNoticeKind::Data { data_base64 } => {
                    let bytes = STANDARD
                        .decode(&data_base64)
                        .map_err(|_| diag("validation.invalid_base64", "socket_notice"))?;
                    if state.received.len() + bytes.len() > MAX_RECEIVED_BYTES {
                        return Err(diag("resource.limit_exceeded", notice.owner.as_str()));
                    }
                    state.received.extend_from_slice(&bytes);
                    push(
                        state,
                        message("app_bytes_received", &[("bytes", bytes.len().to_string())]),
                    );
                    fact(
                        &mut out,
                        &notice.device,
                        &notice.correlation,
                        "application.data.received",
                        json!({"application":notice.owner,"socket":notice.socket,"data_base64":data_base64}),
                    );
                    if software == Some(Software::TcpServer) && state.running {
                        socket(
                            &mut out,
                            &notice.device,
                            &notice.correlation,
                            SocketCommand::Send {
                                socket: notice.socket,
                                data_base64,
                            },
                        );
                    }
                }
                SocketNoticeKind::Connected | SocketNoticeKind::Accepted => {
                    push(state, message("app_connected", &[]))
                }
                SocketNoticeKind::Closed { reason } => {
                    if software == Some(Software::TcpClient) || notice.socket == notice.owner {
                        state.running = false;
                    }
                    if reason == "socket.closed" || reason == "socket.aborted" {
                        push(state, message("app_closed", &[]));
                    } else {
                        push(
                            state,
                            Line::Error {
                                diagnostic: diag(&reason, notice.socket.as_str()),
                            },
                        );
                    }
                }
                _ => {}
            }
        }
        Ok(out)
    }
    pub fn observe(&mut self, now: u64, facts: &[Fact]) {
        for fact in facts {
            for (device, host) in &mut self.hosts {
                for state in host.apps.values_mut() {
                    if matches!(state.job, Some(Job::Capture)) && fact.device == *device {
                        push(
                            state,
                            Line::Capture {
                                time_us: Decimal(now),
                                event: fact.kind.into(),
                                correlation: fact.correlation.clone(),
                                payload: fact.payload.clone(),
                            },
                        );
                    }
                    if fact.device == *device
                        && let Some(Job::Trace { token, done, .. }) = &mut state.job
                    {
                        let prefix = format!("trace-{token}-");
                        if let Some(ttl) = fact.correlation.as_str().strip_prefix(&prefix) {
                            if fact.kind == "icmp.echo.reply" || fact.kind == "icmp.error" {
                                let address = fact
                                    .payload
                                    .get("source")
                                    .and_then(Value::as_str)
                                    .unwrap_or("?")
                                    .to_owned();
                                let rtt = fact
                                    .payload
                                    .get("rtt_us")
                                    .and_then(Value::as_str)
                                    .unwrap_or("0")
                                    .to_owned();
                                *done = fact.kind == "icmp.echo.reply"
                                    || fact.payload.get("icmp_type").and_then(Value::as_u64)
                                        == Some(3);
                                let terminal_error = fact.kind == "icmp.error" && *done;
                                push(
                                    state,
                                    message(
                                        "trace_hop",
                                        &[
                                            ("hop", ttl.to_owned()),
                                            ("source", address),
                                            ("rtt", rtt),
                                        ],
                                    ),
                                );
                                if terminal_error {
                                    let reason = fact
                                        .payload
                                        .get("reason")
                                        .and_then(Value::as_str)
                                        .unwrap_or("routing.no_route");
                                    push(
                                        state,
                                        Line::Error {
                                            diagnostic: diag(reason, device.as_str()),
                                        },
                                    );
                                }
                            } else if fact.kind == "icmp.timeout" {
                                push(state, message("trace_timeout", &[("hop", ttl.to_owned())]));
                            }
                        }
                    }
                    if let Some(Job::Ping { token, .. }) = &state.job {
                        let prefix = format!("ping-{token}-");
                        if fact.correlation.as_str().starts_with(&prefix) {
                            if fact.kind == "icmp.timeout" {
                                push(
                                    state,
                                    message(
                                        "ping_timeout",
                                        &[("probe", fact.correlation.as_str().to_owned())],
                                    ),
                                );
                            } else if fact.kind == "icmp.error" {
                                let reason = fact
                                    .payload
                                    .get("reason")
                                    .and_then(Value::as_str)
                                    .unwrap_or("routing.no_route");
                                push(
                                    state,
                                    Line::Error {
                                        diagnostic: diag(reason, device.as_str()),
                                    },
                                );
                            }
                        }
                    }
                    if let Some(Job::Ping {
                        token,
                        replied,
                        received,
                        ..
                    }) = &mut state.job
                    {
                        let prefix = format!("ping-{token}-");
                        if let Some(index) = fact
                            .correlation
                            .as_str()
                            .strip_prefix(&prefix)
                            .and_then(|s| s.parse::<u16>().ok())
                            && fact.kind == "icmp.echo.reply"
                        {
                            replied.insert(index);
                            *received += 1;
                            let source = fact
                                .payload
                                .get("source")
                                .and_then(Value::as_str)
                                .unwrap_or("?")
                                .to_owned();
                            let rtt = fact
                                .payload
                                .get("rtt_us")
                                .and_then(Value::as_str)
                                .unwrap_or("0")
                                .to_owned();
                            push(
                                state,
                                message("ping_reply", &[("source", source), ("rtt", rtt)]),
                            );
                        }
                    }
                }
            }
        }
    }
    pub fn timer(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        timer: &Timer,
    ) -> Result<Output, Diagnostic> {
        if matches!(
            timer.kind,
            TimerKind::P2pSession { .. } | TimerKind::P2pSearch
        ) {
            return self.p2p_timer(timer);
        }
        if matches!(
            timer.kind,
            TimerKind::MailJob { .. } | TimerKind::MailSession { .. }
        ) {
            return self.mail_timer(mail::Context { project, net, now }, timer);
        }
        match &timer.kind {
            TimerKind::DnsTimeout { socket } => {
                return self.dns_timeout(project, net, now, timer, socket);
            }
            TimerKind::WebTimeout { socket } => return self.web_timeout(timer, socket),
            TimerKind::WebServerTimeout { socket } => {
                return self.web_server_timeout(timer, socket);
            }
            _ => {}
        }
        let mut out = Output::default();
        let Some(state) = self
            .hosts
            .get_mut(&timer.device)
            .and_then(|h| h.apps.get_mut(&timer.application))
        else {
            return Ok(out);
        };
        if let Some(Job::Trace {
            token,
            target,
            max,
            done,
        }) = state.job.as_ref()
        {
            if *token != timer.token {
                return Ok(out);
            }
            let (target, max, done) = (*target, *max, *done);
            match timer.kind {
                TimerKind::TraceSend { ttl } => {
                    let flow = Id::new(format!("trace-{}-{ttl}", timer.token))?;
                    network(
                        &mut out,
                        &flow,
                        Action::Ping {
                            source: timer.device.clone(),
                            destination: target,
                            ttl,
                        },
                    );
                    out.effects.push(Effect::Timer(Timer {
                        kind: TimerKind::TraceNext { ttl },
                        delay_us: sns_protocols::PING_TIMEOUT_US,
                        ..timer.clone()
                    }));
                }
                TimerKind::TraceNext { ttl } => {
                    if done || ttl >= max {
                        state.job = None;
                        push(
                            state,
                            message(
                                if done {
                                    "trace_finished"
                                } else {
                                    "trace_limit"
                                },
                                &[],
                            ),
                        );
                    } else {
                        out.effects.push(Effect::Timer(Timer {
                            kind: TimerKind::TraceSend { ttl: ttl + 1 },
                            delay_us: 0,
                            ..timer.clone()
                        }));
                    }
                }
                _ => {}
            }
            return Ok(out);
        }
        let Some(Job::Ping {
            token,
            target,
            count,
            sent,
            broadcast,
            ..
        }) = state.job.as_mut()
        else {
            return Ok(out);
        };
        if *token != timer.token {
            return Ok(out);
        }
        match timer.kind {
            TimerKind::PingSend { index } => {
                if index >= *count {
                    return Ok(out);
                }
                *sent += 1;
                let flow = Id::new(format!("ping-{}-{index}", timer.token))?;
                network(
                    &mut out,
                    &flow,
                    if *broadcast {
                        Action::PingBroadcast {
                            source: timer.device.clone(),
                            destination: *target,
                            ttl: 64,
                        }
                    } else {
                        Action::Ping {
                            source: timer.device.clone(),
                            destination: *target,
                            ttl: 64,
                        }
                    },
                );
                let (kind, delay) = if index + 1 < *count {
                    (TimerKind::PingSend { index: index + 1 }, 1_000_000)
                } else {
                    (TimerKind::PingDone, 3_000_000)
                };
                out.effects.push(Effect::Timer(Timer {
                    kind,
                    delay_us: delay,
                    ..timer.clone()
                }));
            }
            TimerKind::PingDone => {
                self.finish_ping(&timer.device, &timer.application)?;
            }
            TimerKind::TraceSend { .. }
            | TimerKind::TraceNext { .. }
            | TimerKind::DnsTimeout { .. }
            | TimerKind::WebTimeout { .. }
            | TimerKind::WebServerTimeout { .. }
            | TimerKind::P2pSession { .. }
            | TimerKind::P2pSearch
            | TimerKind::MailJob { .. }
            | TimerKind::MailSession { .. } => {}
        }
        let _ = project;
        Ok(out)
    }
    fn finish_ping(&mut self, device: &Id, app: &Id) -> Result<(), Diagnostic> {
        let state = self.state(device, app)?;
        if let Some(Job::Ping {
            sent,
            replied,
            received,
            ..
        }) = state.job.take()
        {
            push(
                state,
                message(
                    "ping_summary",
                    &[
                        ("sent", sent.to_string()),
                        ("received", received.to_string()),
                        ("lost", (usize::from(sent) - replied.len()).to_string()),
                    ],
                ),
            );
        }
        Ok(())
    }
    fn interrupt(&mut self, device: &Id, app: &Id) -> Result<(), Diagnostic> {
        self.cancel_service_queries(device, app);
        self.finish_ping(device, app)?;
        let s = self.state(device, app)?;
        s.job = None;
        push(s, message("interrupted", &[]));
        Ok(())
    }
}
fn upgrade(project: &mut Project) {
    if project.schema_version < 3 {
        project.schema_version = 3;
        project.semantic_profile = "sns-educational-hosts-v1".into();
    }
}
fn set_config(project: &mut Project, device: &Id, app: &Id, key: &str, value: String) {
    project
        .devices
        .iter_mut()
        .find(|d| &d.id == device)
        .unwrap()
        .applications
        .iter_mut()
        .find(|a| &a.id == app)
        .unwrap()
        .config
        .insert(key.into(), value);
    upgrade(project);
}

fn needs_dns(project: &Project, net: &Network, device: &Id, target: &str) -> bool {
    target.parse::<Ipv4Addr>().is_err()
        && !target.eq_ignore_ascii_case("localhost")
        && net
            .effective_project(project)
            .device(device)
            .is_some_and(|d| d.network.dns_server.is_some())
}

/// Resolve a document resource without performing I/O. Only simulated HTTP URLs are accepted.
pub fn resource_url(input: &str, base: &str) -> Result<String, Diagnostic> {
    Ok(web::Url::parse(input, Some(base))?.full())
}
