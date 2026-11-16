//! Native projections of host applications. No application runs on the GUI thread.
mod mail_ui;
mod p2p_ui;
mod services_ui;
mod terminal_text;
use super::*;
use base64::{Engine as _, engine::general_purpose::STANDARD};
pub(super) use services_ui::dhcp_controls;
use sns_apps::{AppView, Line};
use std::collections::BTreeMap;
#[derive(Default)]
pub(super) struct DesktopState {
    pub(super) host: Option<Id>,
    pub(super) top: f32,
    serial: u64,
    drafts: BTreeMap<(Id, Id), Draft>,
}
impl DesktopState {
    pub(super) fn dirty(&self) -> bool {
        self.drafts.values().any(|d| d.dirty)
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Draft {
    p2p: p2p_ui::P2pDraft,
    mail: mail_ui::MailDraftUi,
    service: Option<ServiceConfig>,
    vhosts: Option<Vec<(String, String)>>,
    #[serde(skip)]
    web_generation: u64,
    #[serde(skip)]
    web_images: BTreeMap<String, egui::TextureHandle>,
    #[serde(with = "field_pairs")]
    web_fields: BTreeMap<(String, String), String>,
    input: String,
    #[serde(skip)]
    history: usize,
    path: String,
    destination: String,
    text: String,
    dirty: bool,
    #[serde(skip)]
    close_requested: bool,
    #[serde(skip)]
    pending_save: Option<String>,
    target: String,
    port: u16,
    #[serde(skip)]
    image: Option<(String, egui::TextureHandle)>,
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            p2p: Default::default(),
            mail: Default::default(),
            service: None,
            vhosts: None,
            web_generation: 0,
            web_images: BTreeMap::new(),
            web_fields: BTreeMap::new(),
            input: String::new(),
            history: 0,
            path: "/".into(),
            destination: "/new.txt".into(),
            text: String::new(),
            dirty: false,
            close_requested: false,
            pending_save: None,
            target: "192.168.1.3".into(),
            port: 7777,
            image: None,
        }
    }
}
fn field(ui: &mut egui::Ui, label: &str, value: &mut String) -> egui::Response {
    let label = ui.label(label);
    ui.add(egui::TextEdit::singleline(value).char_limit(4096))
        .labelled_by(label.id)
}
fn error(ui: &mut egui::Ui, language: Language, e: impl ToString) {
    ui.colored_label(Color32::LIGHT_RED, language.describe_error(&e.to_string()));
}
fn line(language: Language, line: &Line) -> String {
    match line {
        Line::Message { key, params } => language
            .format(
                key,
                &params
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect::<Vec<_>>(),
            )
            .unwrap_or_else(|e| e),
        Line::Text { value } => value.clone(),
        Line::Prompt { cwd, value } => format!("{cwd} > {value}"),
        Line::Error { diagnostic } => format!(
            "{} · {}",
            language.describe_error(&diagnostic.to_string()),
            diagnostic.target.as_deref().unwrap_or("")
        ),
        Line::Help { command } => match command {
            Some(command) => format!(
                "{}\n{}",
                sns_apps::usage(command).unwrap_or(command),
                language.text("terminal_help")
            ),
            None => format!(
                "{}\n{}",
                language.text("terminal_help"),
                sns_apps::COMMAND_HELP
            ),
        },
        Line::Json { value } => serde_json::to_string_pretty(value).unwrap(),
        Line::Capture {
            time_us,
            event,
            correlation,
            payload,
        } => format!(
            "{} µs · {event} · {} · {}",
            time_us.0,
            correlation.as_str(),
            serde_json::to_string(payload).unwrap()
        ),
    }
}
impl Workbench {
    fn app_command(&mut self, host: &Id, command: AppCommand) {
        if self.mode == Mode::Simulation && self.worker.current() {
            self.worker.submit(Job::Command(Command::App {
                device: host.clone(),
                command,
            }));
        }
    }
    pub(super) fn desktop_ui(&mut self, ui: &mut egui::Ui) {
        let l = self.language;
        if self.mode != Mode::Simulation {
            ui.label(l.text("desktop_hint"));
            return;
        }
        if !self.worker.current() {
            ui.spinner();
            return;
        }
        let hosts = self.worker.view.hosts.clone();
        if self.desktop.host.is_none()
            || !hosts
                .iter()
                .any(|h| Some(&h.device) == self.desktop.host.as_ref())
        {
            self.desktop.host = self
                .selected
                .iter()
                .find(|id| hosts.iter().any(|h| &h.device == *id))
                .cloned()
                .or_else(|| hosts.first().map(|h| h.device.clone()));
        }
        ui.horizontal_wrapped(|ui| {
            for host in hosts.iter() {
                let name = self
                    .project()
                    .device(&host.device)
                    .map(|d| self.device_label(d))
                    .unwrap_or_default();
                ui.selectable_value(&mut self.desktop.host, Some(host.device.clone()), name);
            }
        });
        let Some(host) = hosts
            .iter()
            .find(|h| Some(&h.device) == self.desktop.host.as_ref())
        else {
            return;
        };
        if let Some(device) = self.worker.view.effective.device(&host.device) {
            ui.collapsing(l.text("state"), |ui| {
                for nic in &device.interfaces {
                    ui.monospace(format!(
                        "{} · {} · {}",
                        nic.id.as_str(),
                        nic.mac.as_str(),
                        serde_json::to_string(&nic.ipv4).unwrap()
                    ));
                }
                ui.label(format!(
                    "{}: {}",
                    l.text("gateway"),
                    device.network.default_gateway.as_deref().unwrap_or("—")
                ));
                ui.label(format!(
                    "{}: {}",
                    l.text("dns_server"),
                    device.network.dns_server.as_deref().unwrap_or("—")
                ));
                for socket in self
                    .worker
                    .view
                    .sockets
                    .iter()
                    .filter(|s| s.device == host.device)
                {
                    ui.monospace(format!(
                        "{}:{} · {}",
                        socket.local.address, socket.local.port, socket.state
                    ));
                }
            });
        }
        ui.heading(l.text("software"));
        egui::ScrollArea::vertical()
            .id_salt("software-catalogue")
            .show(ui, |ui| {
                for software in Software::ALL {
                    ui.push_id(software.id(), |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.strong(l.text(software.label_key()));
                            let installed = host
                                .applications
                                .iter()
                                .find(|a| a.software == software.id());
                            if let Some(app) = installed {
                                ui.label(l.text("installed"));
                                if ui
                                    .add_enabled(
                                        software.available(),
                                        egui::Button::new(format!(
                                            "{} · {}",
                                            l.text("open"),
                                            l.text(software.label_key())
                                        )),
                                    )
                                    .clicked()
                                {
                                    self.app_command(
                                        &host.device,
                                        AppCommand::OpenView {
                                            application: app.id.clone(),
                                        },
                                    );
                                }
                                if ui
                                    .button(format!(
                                        "{} · {}",
                                        l.text("uninstall"),
                                        l.text(software.label_key())
                                    ))
                                    .clicked()
                                {
                                    if self
                                        .desktop
                                        .drafts
                                        .get(&(host.device.clone(), app.id.clone()))
                                        .is_some_and(|d| d.dirty)
                                    {
                                        self.error = Some("application.unsaved_draft".into());
                                    } else {
                                        self.app_command(
                                            &host.device,
                                            AppCommand::Uninstall {
                                                application: app.id.clone(),
                                            },
                                        );
                                    }
                                }
                                ui.label(l.text(if app.running { "active" } else { "inactive" }));
                            } else if ui
                                .button(format!(
                                    "{} · {}",
                                    l.text("install"),
                                    l.text(software.label_key())
                                ))
                                .clicked()
                            {
                                let application = loop {
                                    self.desktop.serial += 1;
                                    let id =
                                        Id::new(format!("app-ui-{}", self.desktop.serial)).unwrap();
                                    let p = self.project();
                                    if p.project_id != id
                                        && !p.devices.iter().any(|d| {
                                            d.id == id
                                                || d.interfaces.iter().any(|i| i.id == id)
                                                || d.applications.iter().any(|a| a.id == id)
                                        })
                                        && !p.links.iter().any(|l| l.id == id)
                                        && !p.annotations.iter().any(|a| a.id == id)
                                        && !p.resources.iter().any(|r| r.id == id)
                                    {
                                        break id;
                                    }
                                };
                                self.app_command(
                                    &host.device,
                                    AppCommand::Install {
                                        application,
                                        software,
                                    },
                                );
                            }
                            if !software.available() {
                                ui.label(format!(
                                    "{} · J{}",
                                    l.text("unavailable"),
                                    software.milestone()
                                ));
                            }
                        })
                    });
                }
                for unknown in host
                    .applications
                    .iter()
                    .filter(|a| Software::parse(&a.software).is_none())
                {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} · {}", unknown.software, l.text("unavailable")));
                        if ui.button(l.text("uninstall")).clicked() {
                            self.app_command(
                                &host.device,
                                AppCommand::Uninstall {
                                    application: unknown.id.clone(),
                                },
                            );
                        }
                    });
                }
            });
    }
    pub(super) fn application_windows(&mut self, ctx: &egui::Context) {
        if self.mode != Mode::Simulation || !self.worker.current() {
            return;
        }
        let l = self.language;
        let hosts = self.worker.view.hosts.clone();
        self.desktop.drafts.retain(|(host, app), _| {
            hosts
                .iter()
                .any(|h| &h.device == host && h.applications.iter().any(|a| &a.id == app))
        });
        for host in hosts.iter() {
            for app in host.applications.iter().filter(|a| a.visible) {
                let Some(software) = Software::parse(&app.software) else {
                    continue;
                };
                let name = self
                    .project()
                    .device(&host.device)
                    .map(|d| self.device_label(d))
                    .unwrap_or_default();
                let key = (host.device.clone(), app.id.clone());
                let project = self.worker.view.project.clone();
                let mut draft = self.desktop.drafts.remove(&key).unwrap_or_else(|| {
                    let mut draft = Draft::default();
                    if let Some(metadata) = project
                        .device(&host.device)
                        .and_then(|d| d.applications.iter().find(|a| a.id == app.id))
                    {
                        if let Some(port) = metadata.config.get("port").and_then(|p| p.parse().ok())
                        {
                            draft.port = port;
                        }
                        if let Some(target) = metadata.config.get("target") {
                            draft.target = target.clone();
                        }
                    }
                    draft
                });
                let mut open = true;
                let size = if matches!(
                    software,
                    Software::MailClient | Software::MailServer | Software::P2p
                ) {
                    Vec2::new(760., 780.)
                } else if software == Software::WebBrowser {
                    Vec2::new(760., 600.)
                } else {
                    Vec2::new(620., 420.)
                };
                let bounds = ctx.content_rect();
                let top = (self.desktop.top + 44.)
                    .min(bounds.max.y - 100.)
                    .max(bounds.min.y);
                let bounds = Rect::from_min_max(Pos2::new(bounds.min.x, top), bounds.max);
                let window =
                    egui::Window::new(format!("{} · {name}", l.text(software.label_key())))
                        .id(egui::Id::unique(("application", &host.device, &app.id)))
                        .open(&mut open);
                let response = self
                    .configured_window(ctx, software.id(), window, size)
                    .max_size(bounds.size().max(Vec2::splat(100.)))
                    .hscroll(true)
                    .vscroll(true)
                    .constrain_to(bounds)
                    .show(ctx, |ui| {
                        // Full-width editors follow the window instead of overflowing it.
                        ui.set_max_width(ui.available_width().min(860.));
                        if ui.button(l.text("close_view")).clicked() {
                            if draft.dirty {
                                draft.close_requested = true;
                            } else {
                                self.app_command(
                                    &host.device,
                                    AppCommand::CloseView {
                                        application: app.id.clone(),
                                    },
                                );
                            }
                        }
                        match software {
                            Software::P2p => self.p2p_ui(ui, &host.device, app, &mut draft),
                            Software::MailServer | Software::MailClient => {
                                self.mail_ui(ui, &host.device, app, &mut draft, software)
                            }
                            Software::Firewall => {
                                if let Some(device) = self.project().device(&host.device).cloned() {
                                    self.firewall_panel(ui, &device);
                                }
                            }
                            Software::DnsServer | Software::WebServer => {
                                self.service_ui(ui, &host.device, app, &mut draft, software)
                            }
                            Software::WebBrowser => {
                                self.browser_ui(ui, &host.device, app, &mut draft)
                            }
                            Software::Terminal => {
                                self.terminal_ui(ui, &host.device, app, &mut draft)
                            }
                            Software::TcpServer | Software::TcpClient => {
                                self.tcp_ui(ui, &host.device, app, &mut draft, software)
                            }
                            Software::FileExplorer
                            | Software::TextEditor
                            | Software::ImageViewer => {
                                if let Some(device) = project.device(&host.device) {
                                    match sns_vfs::Vfs::from_files(&device.files) {
                                        Ok(files) => self.files_ui(
                                            ui,
                                            &host.device,
                                            app,
                                            &mut draft,
                                            software,
                                            &files,
                                        ),
                                        Err(e) => error(ui, l, e),
                                    }
                                }
                            }
                        }
                    });
                if let Some(response) = response {
                    self.remember_window(ctx, software.id(), &response.response);
                }
                if !open {
                    if draft.dirty {
                        draft.close_requested = true;
                    } else {
                        self.app_command(
                            &host.device,
                            AppCommand::CloseView {
                                application: app.id.clone(),
                            },
                        );
                    }
                }
                if draft.close_requested {
                    egui::Window::new(l.text("unsaved_title"))
                        .id(egui::Id::unique(("draft-close", &app.id)))
                        .show(ctx, |ui| {
                            ui.label(l.text("unsaved_prompt"));
                            if ui.button(l.text("discard")).clicked() {
                                draft.dirty = false;
                                draft.close_requested = false;
                                self.app_command(
                                    &host.device,
                                    AppCommand::CloseView {
                                        application: app.id.clone(),
                                    },
                                );
                            }
                            if ui.button(l.text("continue_editing")).clicked() {
                                draft.close_requested = false;
                            }
                        });
                }
                self.desktop.drafts.insert(key, draft);
            }
        }
    }
    fn terminal_ui(&mut self, ui: &mut egui::Ui, host: &Id, app: &AppView, draft: &mut Draft) {
        // Horizontal scrolling gives its child a wide layout. Keep terminal actions
        // within the actual viewport so enlarged controls wrap before clipping.
        ui.set_max_width((ui.clip_rect().right() - ui.min_rect().left() - 16.).clamp(100., 860.));
        let l = self.language;
        egui::ScrollArea::vertical()
            .id_salt(("terminal-log", &app.id))
            .max_height(300.)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let expert = self.expert();
                let mut command: Option<&str> = None;
                for entry in &app.lines {
                    if let Line::Prompt { value, .. } = entry {
                        command = value.split_whitespace().next();
                    }
                    let readable = if expert {
                        None
                    } else {
                        terminal_text::readable(l, command, entry)
                    };
                    ui.monospace(readable.unwrap_or_else(|| line(l, entry)));
                }
                if app.omitted_lines > 0 {
                    ui.label(
                        l.format(
                            "omitted_lines",
                            &[("count", &app.omitted_lines.to_string())],
                        )
                        .unwrap(),
                    );
                }
            });
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("{} >", app.cwd));
            let response = field(ui, l.text("terminal_input"), &mut draft.input);
            let focused = response.has_focus() || response.lost_focus();
            if focused && ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                draft.history = draft.history.saturating_add(1).min(app.history.len());
                if draft.history > 0 {
                    draft.input = app.history[app.history.len() - draft.history].clone();
                }
            }
            if focused && ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                draft.history = draft.history.saturating_sub(1);
                draft.input = if draft.history == 0 {
                    String::new()
                } else {
                    app.history[app.history.len() - draft.history].clone()
                };
            }
            if focused && ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) {
                self.app_command(
                    host,
                    AppCommand::Interrupt {
                        application: app.id.clone(),
                    },
                );
                draft.input.clear();
                response.request_focus();
            } else if (focused && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                || ui.button(l.text("send")).clicked()
            {
                self.app_command(
                    host,
                    AppCommand::Terminal {
                        application: app.id.clone(),
                        line: std::mem::take(&mut draft.input),
                    },
                );
                draft.history = 0;
                response.request_focus();
            }
            if ui
                .add_enabled(app.busy, egui::Button::new(l.text("interrupt")))
                .clicked()
            {
                self.app_command(
                    host,
                    AppCommand::Interrupt {
                        application: app.id.clone(),
                    },
                );
            }
        });
    }
    fn tcp_ui(
        &mut self,
        ui: &mut egui::Ui,
        host: &Id,
        app: &AppView,
        draft: &mut Draft,
        software: Software,
    ) {
        let l = self.language;
        ui.horizontal_wrapped(|ui| {
            if software == Software::TcpClient {
                field(ui, l.text("connection_target"), &mut draft.target);
            }
            let port_label = ui.label(l.text("port"));
            ui.add(egui::DragValue::new(&mut draft.port).range(1..=65535))
                .labelled_by(port_label.id);
            if ui
                .add_enabled(
                    !app.running,
                    egui::Button::new(l.text(if software == Software::TcpServer {
                        "start_service"
                    } else {
                        "connect"
                    })),
                )
                .clicked()
            {
                self.app_command(
                    host,
                    if software == Software::TcpServer {
                        AppCommand::StartServer {
                            application: app.id.clone(),
                            port: draft.port,
                        }
                    } else {
                        AppCommand::Connect {
                            application: app.id.clone(),
                            target: draft.target.clone(),
                            port: draft.port,
                        }
                    },
                );
            }
            if ui
                .add_enabled(app.running, egui::Button::new(l.text("stop_service")))
                .clicked()
            {
                self.app_command(
                    host,
                    AppCommand::Stop {
                        application: app.id.clone(),
                    },
                );
            }
        });
        for socket in self
            .worker
            .view
            .sockets
            .iter()
            .filter(|s| s.device == *host && s.owner == app.id)
        {
            ui.monospace(format!(
                "{}:{} <-> {} · {}",
                socket.local.address,
                socket.local.port,
                socket
                    .peer
                    .as_ref()
                    .map(|p| format!("{}:{}", p.address, p.port))
                    .unwrap_or_else(|| "*".into()),
                socket.state
            ));
        }
        if software == Software::TcpClient {
            ui.horizontal(|ui| {
                field(ui, l.text("message"), &mut draft.input);
                if ui
                    .add_enabled(app.running, egui::Button::new(l.text("send")))
                    .clicked()
                {
                    self.app_command(
                        host,
                        AppCommand::Send {
                            application: app.id.clone(),
                            text: std::mem::take(&mut draft.input),
                        },
                    );
                }
            });
        }
        ui.strong(l.text("received"));
        egui::ScrollArea::vertical()
            .id_salt("tcp-log")
            .max_height(240.)
            .show(ui, |ui| {
                for entry in &app.lines {
                    ui.monospace(line(l, entry));
                }
                if let Ok(bytes) = STANDARD.decode(&app.received_base64) {
                    ui.monospace(String::from_utf8_lossy(&bytes));
                }
            });
        if ui.button(l.text("clear_trace")).clicked() {
            self.app_command(
                host,
                AppCommand::ClearReceived {
                    application: app.id.clone(),
                },
            );
        }
    }
    fn file_command(&mut self, host: &Id, app: &AppView, command: FileCommand) {
        self.app_command(
            host,
            AppCommand::File {
                application: app.id.clone(),
                command,
            },
        );
    }
    fn files_ui(
        &mut self,
        ui: &mut egui::Ui,
        host: &Id,
        app: &AppView,
        draft: &mut Draft,
        software: Software,
        files: &sns_vfs::Vfs,
    ) {
        let l = self.language;
        if let Some(path) = draft.pending_save.as_ref()
            && files.read_text(path).is_ok_and(|text| text == draft.text)
        {
            draft.path = path.clone();
            draft.pending_save = None;
            draft.dirty = false;
        }
        field(ui, l.text("path"), &mut draft.path);
        if software == Software::FileExplorer {
            ui.horizontal(|ui| {
                if ui
                    .button(l.text("parent_folder"))
                    .on_hover_text(l.text("parent_folder"))
                    .clicked()
                {
                    draft.path = sns_vfs::resolve(&draft.path, "..").unwrap_or_else(|_| "/".into());
                }
                if ui.button(l.text("import_file")).clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .set_title(l.text("import_file"))
                        .pick_file()
                {
                    let result = (|| -> Result<(), String> {
                        let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
                        use std::io::Read;
                        let mut bytes = Vec::new();
                        file.take((sns_vfs::MAX_HOST_BYTES + 1) as u64)
                            .read_to_end(&mut bytes)
                            .map_err(|e| e.to_string())?;
                        if bytes.len() > sns_vfs::MAX_HOST_BYTES {
                            return Err("resource.limit_exceeded".into());
                        }
                        let name = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .ok_or("fs.invalid_path")?;
                        let destination =
                            sns_vfs::resolve(&draft.path, name).map_err(|e| e.to_string())?;
                        let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                            "image/png"
                        } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
                            "image/jpeg"
                        } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
                            "image/gif"
                        } else if bytes.starts_with(b"BM") {
                            "image/bmp"
                        } else if std::str::from_utf8(&bytes).is_ok() {
                            "text/plain"
                        } else {
                            "application/octet-stream"
                        };
                        self.file_command(
                            host,
                            app,
                            FileCommand::Write {
                                path: destination,
                                media_type: mime.into(),
                                data_base64: STANDARD.encode(bytes),
                                overwrite: false,
                            },
                        );
                        Ok(())
                    })();
                    if let Err(e) = result {
                        self.error = Some(e);
                    }
                }
            });
            match files.list(&draft.path) {
                Ok(entries) => {
                    egui::ScrollArea::vertical()
                        .id_salt("file-list")
                        .max_height(190.)
                        .show(ui, |ui| {
                            for entry in entries {
                                ui.horizontal(|ui| {
                                    if ui
                                        .selectable_label(
                                            draft.destination == entry.path,
                                            format!(
                                                "{} {}",
                                                if entry.directory { "[+]" } else { "·" },
                                                entry.path
                                            ),
                                        )
                                        .clicked()
                                    {
                                        draft.destination = entry.path.clone();
                                    }
                                    if ui.button(l.text("open_file")).clicked() {
                                        if entry.directory {
                                            draft.path = entry.path.clone();
                                        } else {
                                            let kind = if entry.media_type.starts_with("image/") {
                                                Software::ImageViewer
                                            } else {
                                                Software::TextEditor
                                            };
                                            let installed = self
                                                .worker
                                                .view
                                                .hosts
                                                .iter()
                                                .find(|h| h.device == *host)
                                                .and_then(|h| {
                                                    h.applications
                                                        .iter()
                                                        .find(|a| a.software == kind.id())
                                                })
                                                .cloned();
                                            if let Some(target) = installed {
                                                let target_draft = self
                                                    .desktop
                                                    .drafts
                                                    .entry((host.clone(), target.id.clone()))
                                                    .or_default();
                                                if !target_draft.dirty {
                                                    target_draft.path = entry.path.clone();
                                                    if kind == Software::TextEditor {
                                                        match files.read_text(&entry.path) {
                                                            Ok(s) => target_draft.text = s.into(),
                                                            Err(e) => {
                                                                self.error = Some(e.to_string());
                                                                return;
                                                            }
                                                        }
                                                    }
                                                    self.app_command(
                                                        host,
                                                        AppCommand::OpenView {
                                                            application: target.id,
                                                        },
                                                    );
                                                } else {
                                                    self.error =
                                                        Some("application.unsaved_draft".into());
                                                }
                                            } else {
                                                self.error =
                                                    Some("application.not_installed".into());
                                            }
                                        }
                                    }
                                });
                            }
                        });
                }
                Err(e) => error(ui, l, e),
            }
            field(ui, l.text("destination"), &mut draft.destination);
            ui.horizontal_wrapped(|ui| {
                if ui.button(l.text("new_folder")).clicked() {
                    self.file_command(
                        host,
                        app,
                        FileCommand::Mkdir {
                            path: draft.destination.clone(),
                        },
                    );
                }
                if ui.button(l.text("new_file")).clicked() {
                    self.file_command(
                        host,
                        app,
                        FileCommand::Write {
                            path: draft.destination.clone(),
                            media_type: "text/plain".into(),
                            data_base64: String::new(),
                            overwrite: false,
                        },
                    );
                }
                if ui.button(l.text("delete")).clicked() {
                    self.file_command(
                        host,
                        app,
                        FileCommand::Remove {
                            path: draft.destination.clone(),
                            recursive: false,
                        },
                    );
                }
            });
            ui.horizontal_wrapped(|ui| {
                field(ui, l.text("source_path"), &mut draft.input);
                if ui.button(l.text("copy")).clicked() {
                    self.file_command(
                        host,
                        app,
                        FileCommand::Copy {
                            source: draft.input.clone(),
                            destination: draft.destination.clone(),
                        },
                    );
                }
                if ui.button(l.text("rename_move")).clicked() {
                    self.file_command(
                        host,
                        app,
                        FileCommand::Move {
                            source: draft.input.clone(),
                            destination: draft.destination.clone(),
                        },
                    );
                }
            });
        } else if software == Software::TextEditor {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(!draft.dirty, egui::Button::new(l.text("open_file")))
                    .clicked()
                {
                    match files.read_text(&draft.path) {
                        Ok(s) => draft.text = s.into(),
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
                if ui.button(l.text("virtual_save")).clicked() {
                    self.file_command(
                        host,
                        app,
                        FileCommand::Write {
                            path: draft.path.clone(),
                            media_type: if draft.path.ends_with(".html") {
                                "text/html"
                            } else {
                                "text/plain"
                            }
                            .into(),
                            data_base64: STANDARD.encode(draft.text.as_bytes()),
                            overwrite: true,
                        },
                    );
                    draft.pending_save = Some(draft.path.clone());
                }
            });
            field(ui, l.text("destination"), &mut draft.destination);
            if ui.button(l.text("virtual_save_as")).clicked() {
                self.file_command(
                    host,
                    app,
                    FileCommand::Write {
                        path: draft.destination.clone(),
                        media_type: "text/plain".into(),
                        data_base64: STANDARD.encode(draft.text.as_bytes()),
                        overwrite: false,
                    },
                );
                draft.pending_save = Some(draft.destination.clone());
            }
            let content_label = ui.label(l.text("content"));
            if ui
                .add(
                    egui::TextEdit::multiline(&mut draft.text)
                        .char_limit(sns_vfs::MAX_HOST_BYTES)
                        .desired_width(f32::INFINITY)
                        .desired_rows(14)
                        .code_editor(),
                )
                .labelled_by(content_label.id)
                .changed()
            {
                draft.dirty = true;
            }
            if draft.dirty {
                ui.label(l.text("modified"));
            }
        } else {
            ui.label(l.text("image_hint"));
            if ui.button(l.text("open_file")).clicked() {
                let result = (|| -> Result<egui::TextureHandle, String> {
                    let bytes = files.read(&draft.path).map_err(|e| e.to_string())?;
                    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
                        .with_guessed_format()
                        .map_err(|_| "image.invalid_data")?;
                    let mut limits = image::Limits::default();
                    limits.max_image_width = Some(4096);
                    limits.max_image_height = Some(4096);
                    limits.max_alloc = Some(64 * 1024 * 1024);
                    reader.limits(limits);
                    let image = reader
                        .decode()
                        .map_err(|_| "image.invalid_data")?
                        .into_rgba8();
                    let size = [image.width() as usize, image.height() as usize];
                    Ok(ui.ctx().load_texture(
                        format!("{}:{}", host.as_str(), draft.path),
                        egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw()),
                        Default::default(),
                    ))
                })();
                match result {
                    Ok(texture) => draft.image = Some((draft.path.clone(), texture)),
                    Err(e) => self.error = Some(e),
                }
            }
            if let Some((path, texture)) = &draft.image {
                ui.label(path);
                ui.add(
                    egui::Image::new(texture)
                        .max_size(Vec2::new(640., 400.))
                        .shrink_to_fit(),
                );
            }
        }
    }
}

mod field_pairs {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeMap;
    pub fn serialize<S: Serializer>(
        map: &BTreeMap<(String, String), String>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        map.iter().collect::<Vec<_>>().serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<BTreeMap<(String, String), String>, D::Error> {
        let entries = Vec::<((String, String), String)>::deserialize(d)?;
        let mut map = BTreeMap::new();
        for (key, value) in entries {
            if map.insert(key, value).is_some() {
                return Err(serde::de::Error::custom("duplicate form field"));
            }
        }
        Ok(map)
    }
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedDesktop {
    host: Option<Id>,
    drafts: Vec<SavedAppDraft>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedAppDraft {
    device: Id,
    application: Id,
    state: Draft,
}
impl DesktopState {
    pub(super) fn recovery_value(&self) -> serde_json::Value {
        serde_json::json!({"host":self.host,"drafts":self.drafts.iter().map(|((device,application),state)|serde_json::json!({"device":device,"application":application,"state":state})).collect::<Vec<_>>()})
    }
    pub(super) fn restore_recovery(
        value: serde_json::Value,
        project: &Project,
    ) -> Result<Self, String> {
        let saved: SavedDesktop =
            serde_json::from_value(value).map_err(|_| "session.invalid_document".to_owned())?;
        if saved.drafts.len() > 4096
            || saved
                .host
                .as_ref()
                .is_some_and(|h| project.device(h).is_none())
        {
            return Err("session.invalid_document".into());
        }
        let mut drafts = BTreeMap::new();
        for mut draft in saved.drafts {
            draft.state.mail.generation = 0;
            draft.state.mail.sent = false;
            if !project
                .device(&draft.device)
                .is_some_and(|h| h.applications.iter().any(|a| a.id == draft.application))
                || drafts
                    .insert((draft.device, draft.application), draft.state)
                    .is_some()
            {
                return Err("session.invalid_document".into());
            }
        }
        Ok(Self {
            host: saved.host,
            drafts,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    #[test]
    fn restored_mail_composer_keeps_new_draft_and_forgets_old_send_acknowledgements() {
        let project = sns_project::parse_project(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/valid/mail.json"
        )))
        .unwrap();
        let (host, application) = project
            .devices
            .iter()
            .find_map(|h| {
                h.applications
                    .iter()
                    .find(|a| a.kind == Software::MailClient.id())
                    .map(|a| (&h.id, &a.id))
            })
            .unwrap();
        let mut desktop = DesktopState::default();
        let mut draft = Draft::default();
        draft.mail.body = "Nouveau brouillon après l’envoi".into();
        draft.mail.sent = true;
        draft.mail.generation = 99;
        desktop
            .drafts
            .insert((host.clone(), application.clone()), draft);
        let mut encoded = desktop.recovery_value();
        assert!(encoded.pointer("/drafts/0/state/mail/sent").is_none());
        // Older prototype data may still contain these procedural fields.
        encoded["drafts"][0]["state"]["mail"]["sent"] = serde_json::json!(true);
        encoded["drafts"][0]["state"]["mail"]["generation"] = serde_json::json!(99);
        let restored = DesktopState::restore_recovery(encoded, &project).unwrap();
        let mail = &restored.drafts[&(host.clone(), application.clone())].mail;
        assert_eq!(mail.body, "Nouveau brouillon après l’envoi");
        assert!(!mail.sent);
        assert_eq!(mail.generation, 0);
    }
}
