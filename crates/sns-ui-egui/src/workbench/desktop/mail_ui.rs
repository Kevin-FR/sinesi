//! Native mail windows; commands and message storage remain owned by the engine.
use super::*;
#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MailDraftUi {
    pub initialized: bool,
    pub loaded: Option<ServiceConfig>,
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    pub body: String,
    #[serde(skip_serializing, default)]
    pub generation: u64,
    pub selected: Option<Id>,
    #[serde(skip_serializing, default)]
    pub sent: bool,
}
fn list(s: &str) -> Vec<String> {
    s.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}
impl Workbench {
    pub(super) fn mail_ui(
        &mut self,
        ui: &mut egui::Ui,
        host: &Id,
        app: &AppView,
        draft: &mut Draft,
        software: Software,
    ) {
        let l = self.language;
        let metadata = self
            .worker
            .view
            .project
            .device(host)
            .and_then(|h| h.applications.iter().find(|a| a.id == app.id))
            .cloned();
        let Some(metadata) = metadata else { return };
        let mut stale = false;
        if draft.mail.initialized && draft.mail.loaded != metadata.service {
            if draft.service == draft.mail.loaded || draft.service == metadata.service {
                draft.service = metadata.service.clone();
                draft.mail.loaded = metadata.service.clone();
            } else {
                stale = true;
            }
        }
        if !draft.mail.initialized {
            draft.mail.initialized = true;
            draft.mail.loaded = metadata.service.clone();
            draft.service = None;
        }
        if draft.service.is_none() {
            draft.service = metadata.service.clone().or_else(|| {
                Some(if software == Software::MailServer {
                    ServiceConfig::MailServer(Default::default())
                } else {
                    ServiceConfig::MailClient(Default::default())
                })
            });
        }
        ui.label(l.text("mail_simulated_hint"));
        if stale {
            ui.label(l.text("admin_stale"));
            if ui.button(l.text("reload_configuration")).clicked() {
                draft.mail.initialized = false;
            }
        }
        let mut apply = false;
        egui::CollapsingHeader::new(l.text("mail_account_settings"))
            .id_salt("mail-settings")
            .default_open(software == Software::MailServer || metadata.service.is_none())
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("mail-settings-scroll")
                    .max_height(320.)
                    .show(ui, |ui| match draft.service.as_mut().unwrap() {
                        ServiceConfig::MailServer(c) => {
                            field(ui, l.text("mail_domain"), &mut c.domain);
                            let label = ui.label(l.text("mail_smtp_port"));
                            ui.add(egui::DragValue::new(&mut c.smtp_port).range(1..=65535))
                                .labelled_by(label.id);
                            let label = ui.label(l.text("mail_pop3_port"));
                            ui.add(egui::DragValue::new(&mut c.pop3_port).range(1..=65535))
                                .labelled_by(label.id);
                            let mut remove = None;
                            for (i, a) in c.accounts.iter_mut().enumerate() {
                                ui.push_id(i, |ui| {
                                    ui.separator();
                                    field(ui, l.text("mail_username"), &mut a.username);
                                    field(ui, l.text("mail_first_name"), &mut a.first_name);
                                    field(ui, l.text("mail_last_name"), &mut a.last_name);
                                    let label = ui.label(l.text("mail_password"));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut a.password)
                                            .password(true)
                                            .char_limit(128),
                                    )
                                    .labelled_by(label.id);
                                    if ui.button(l.text("mail_delete_account")).clicked() {
                                        remove = Some(i);
                                    }
                                });
                            }
                            if let Some(i) = remove {
                                c.accounts.remove(i);
                            }
                            if c.accounts.len() < 64
                                && ui.button(l.text("mail_add_account")).clicked()
                            {
                                c.accounts.push(MailAccount {
                                    username: "eleve".into(),
                                    password: "classe".into(),
                                    first_name: String::new(),
                                    last_name: String::new(),
                                });
                            }
                        }
                        ServiceConfig::MailClient(c) => {
                            field(ui, l.text("mail_address"), &mut c.address);
                            field(ui, l.text("mail_display_name"), &mut c.display_name);
                            field(ui, l.text("mail_smtp_server"), &mut c.smtp_server);
                            let label = ui.label(l.text("mail_smtp_port"));
                            ui.add(egui::DragValue::new(&mut c.smtp_port).range(1..=65535))
                                .labelled_by(label.id);
                            field(ui, l.text("mail_pop3_server"), &mut c.pop3_server);
                            let label = ui.label(l.text("mail_pop3_port"));
                            ui.add(egui::DragValue::new(&mut c.pop3_port).range(1..=65535))
                                .labelled_by(label.id);
                            field(ui, l.text("mail_username"), &mut c.username);
                            let label = ui.label(l.text("mail_password"));
                            ui.add(
                                egui::TextEdit::singleline(&mut c.password)
                                    .password(true)
                                    .char_limit(128),
                            )
                            .labelled_by(label.id);
                        }
                        _ => {}
                    });
                let valid = draft.service.as_ref().unwrap().validate(software.id());
                if let Err(e) = &valid {
                    error(ui, l, e);
                }
                apply = ui
                    .add_enabled(
                        valid.is_ok() && !app.busy && !stale,
                        egui::Button::new(l.text("mail_apply_settings")),
                    )
                    .clicked();
            });
        if apply {
            self.app_command(
                host,
                AppCommand::ConfigureService {
                    application: app.id.clone(),
                    config: draft.service.clone().unwrap(),
                },
            );
        }
        if software == Software::MailServer {
            if ui
                .add_enabled(
                    metadata.service.is_some(),
                    egui::Button::new(l.text(if app.running { "stop" } else { "start_service" })),
                )
                .clicked()
            {
                if app.running {
                    self.app_command(
                        host,
                        AppCommand::Stop {
                            application: app.id.clone(),
                        },
                    );
                } else {
                    self.app_command(
                        host,
                        AppCommand::Mail {
                            application: app.id.clone(),
                            command: MailCommand::Start,
                        },
                    );
                }
            }
            ui.label(l.text(if app.running {
                "running"
            } else {
                "service_stopped"
            }));
            if let Some(ApplicationData::MailServer(data)) = &metadata.data {
                for (name, messages) in &data.mailboxes {
                    ui.label(format!(
                        "{name} · {} {}",
                        messages.len(),
                        l.text("mail_messages")
                    ));
                }
            }
        } else {
            if draft.mail.generation != app.mail.draft_generation {
                draft.mail.generation = app.mail.draft_generation;
                if let Some(m) = &app.mail.draft {
                    draft.mail.to = m.to.join(", ");
                    draft.mail.cc = m.cc.join(", ");
                    draft.mail.bcc = m.bcc.join(", ");
                    draft.mail.subject = m.subject.clone();
                    draft.mail.body = m.body.clone();
                }
            }
            if ui
                .add_enabled(!app.busy, egui::Button::new(l.text("mail_receive")))
                .clicked()
            {
                self.app_command(
                    host,
                    AppCommand::Mail {
                        application: app.id.clone(),
                        command: MailCommand::Receive,
                    },
                );
            }
            if app.busy {
                ui.label(l.text("mail_working"));
            }
            egui::CollapsingHeader::new(l.text("mail_compose"))
                .id_salt("mail-compose")
                .default_open(true)
                .show(ui, |ui| {
                    field(ui, l.text("mail_to"), &mut draft.mail.to);
                    field(ui, l.text("mail_cc"), &mut draft.mail.cc);
                    field(ui, l.text("mail_bcc"), &mut draft.mail.bcc);
                    field(ui, l.text("mail_subject"), &mut draft.mail.subject);
                    let label = ui.label(l.text("mail_body"));
                    ui.add(
                        egui::TextEdit::multiline(&mut draft.mail.body)
                            .char_limit(32768)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    )
                    .labelled_by(label.id);
                    if ui
                        .add_enabled(!app.busy, egui::Button::new(l.text("mail_send")))
                        .clicked()
                    {
                        self.app_command(
                            host,
                            AppCommand::Mail {
                                application: app.id.clone(),
                                command: MailCommand::Send {
                                    draft: MailDraft {
                                        to: list(&draft.mail.to),
                                        cc: list(&draft.mail.cc),
                                        bcc: list(&draft.mail.bcc),
                                        subject: draft.mail.subject.clone(),
                                        body: draft.mail.body.clone(),
                                    },
                                },
                            },
                        );
                    }
                });
            let data = match &metadata.data {
                Some(ApplicationData::MailClient(data)) => data.clone(),
                _ => MailClientData::default(),
            };
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(!draft.mail.sent, l.text("mail_inbox"))
                    .clicked()
                {
                    draft.mail.sent = false;
                    draft.mail.selected = None;
                }
                if ui
                    .selectable_label(draft.mail.sent, l.text("mail_sent"))
                    .clicked()
                {
                    draft.mail.sent = true;
                    draft.mail.selected = None;
                }
            });
            let messages = if draft.mail.sent {
                &data.sent
            } else {
                &data.inbox
            };
            egui::ScrollArea::vertical()
                .id_salt("mail-message-list")
                .max_height(140.)
                .show(ui, |ui| {
                    for m in messages {
                        if ui
                            .selectable_label(
                                draft.mail.selected.as_ref() == Some(&m.id),
                                format!("{} · {}", m.from, m.subject),
                            )
                            .clicked()
                        {
                            draft.mail.selected = Some(m.id.clone());
                        }
                    }
                });
            if let Some(m) = messages
                .iter()
                .find(|m| Some(&m.id) == draft.mail.selected.as_ref())
            {
                ui.label(format!(
                    "{}: {} <{}>",
                    l.text("mail_from"),
                    m.sender_name,
                    m.from
                ));
                ui.label(format!("{}: {}", l.text("mail_to"), m.to.join(", ")));
                ui.label(format!("{}: {}", l.text("mail_cc"), m.cc.join(", ")));
                if draft.mail.sent {
                    ui.label(format!("{}: {}", l.text("mail_bcc"), m.bcc.join(", ")));
                }
                egui::ScrollArea::vertical()
                    .id_salt("mail-message-body")
                    .max_height(160.)
                    .show(ui, |ui| {
                        ui.label(&m.body);
                    });
                ui.horizontal(|ui| {
                    if !draft.mail.sent && ui.button(l.text("mail_reply")).clicked() {
                        self.app_command(
                            host,
                            AppCommand::Mail {
                                application: app.id.clone(),
                                command: MailCommand::Reply {
                                    message: m.id.clone(),
                                },
                            },
                        );
                    }
                    if ui.button(l.text("mail_delete_message")).clicked() {
                        self.app_command(
                            host,
                            AppCommand::Mail {
                                application: app.id.clone(),
                                command: MailCommand::Delete {
                                    folder: if draft.mail.sent {
                                        MailFolder::Sent
                                    } else {
                                        MailFolder::Inbox
                                    },
                                    message: m.id.clone(),
                                },
                            },
                        );
                        draft.mail.selected = None;
                    }
                });
            }
            if !data.contacts.is_empty() {
                ui.collapsing(l.text("mail_contacts"), |ui| {
                    for contact in &data.contacts {
                        if ui
                            .button(format!(
                                "{} {} · {}",
                                contact.first_name, contact.last_name, contact.email
                            ))
                            .clicked()
                        {
                            if !draft.mail.to.is_empty() {
                                draft.mail.to.push_str(", ");
                            }
                            draft.mail.to.push_str(&contact.email);
                        }
                    }
                });
            }
        }
        for entry in app.lines.iter().rev().take(3).rev() {
            ui.label(line(l, entry));
        }
    }
}
