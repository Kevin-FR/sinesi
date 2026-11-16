//! Shared projections for host filtering and the private-network router.
use super::*;
use std::{collections::BTreeMap, net::Ipv4Addr};
#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SecurityUi {
    firewalls: BTreeMap<Id, FirewallDraft>,
    gateways: BTreeMap<Id, GatewayDraft>,
}
impl SecurityUi {
    pub(super) fn recovery_valid(&self, project: &Project) -> bool {
        self.firewalls
            .iter()
            .all(|(id, d)| project.device(id).is_some() && d.rows.len() <= 256)
            && self
                .gateways
                .iter()
                .all(|(id, d)| project.device(id).is_some() && d.rows.len() <= 256)
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleDraft {
    source: String,
    destination: String,
    port: String,
    protocol: FilterProtocol,
    action: Verdict,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FirewallDraft {
    loaded: Option<Box<FirewallConfig>>,
    config: FirewallConfig,
    rows: Vec<RuleDraft>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GatewayDraft {
    loaded: Box<GatewayConfig>,
    config: GatewayConfig,
    rows: Vec<(TransportProtocol, u16, String, u16)>,
}
fn text_match(m: &IpMatch) -> String {
    match m {
        IpMatch::Any => "*".into(),
        IpMatch::LocalNetwork => "LAN".into(),
        IpMatch::Subnet { network, prefix } => format!("{network}/{prefix}"),
    }
}
fn parse_match(s: &str) -> Result<IpMatch, Diagnostic> {
    if s == "*" || s.is_empty() {
        return Ok(IpMatch::Any);
    }
    if s.eq_ignore_ascii_case("LAN") {
        return Ok(IpMatch::LocalNetwork);
    }
    let (a, p) = s.split_once('/').unwrap_or((s, "32"));
    let m = IpMatch::Subnet {
        network: a
            .parse()
            .map_err(|_| Diagnostic::new("firewall.invalid_rule", s))?,
        prefix: p
            .parse()
            .map_err(|_| Diagnostic::new("firewall.invalid_rule", s))?,
    };
    m.validate()?;
    Ok(m)
}
fn field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    let label = ui.label(label);
    ui.add(egui::TextEdit::singleline(value).char_limit(256))
        .labelled_by(label.id);
}
impl FirewallDraft {
    fn new(value: Option<Box<FirewallConfig>>) -> Self {
        let config = value.as_deref().cloned().unwrap_or_else(|| FirewallConfig {
            enabled: false,
            ..Default::default()
        });
        let rows = config
            .rules
            .iter()
            .map(|r| RuleDraft {
                source: text_match(&r.source),
                destination: text_match(&r.destination),
                port: r
                    .destination_port
                    .map(|p| p.to_string())
                    .unwrap_or("*".into()),
                protocol: r.protocol,
                action: r.action,
            })
            .collect();
        Self {
            loaded: value,
            config,
            rows,
        }
    }
    fn parsed(&self) -> Result<FirewallConfig, Diagnostic> {
        let mut cfg = self.config.clone();
        cfg.rules = self
            .rows
            .iter()
            .map(|r| {
                Ok(FirewallRule {
                    source: parse_match(&r.source)?,
                    destination: parse_match(&r.destination)?,
                    protocol: r.protocol,
                    destination_port: if r.port == "*" || r.port.is_empty() {
                        None
                    } else {
                        Some(
                            r.port
                                .parse::<u16>()
                                .ok()
                                .filter(|p| *p > 0)
                                .ok_or_else(|| Diagnostic::new("firewall.invalid_rule", "port"))?,
                        )
                    },
                    action: r.action,
                })
            })
            .collect::<Result<_, Diagnostic>>()?;
        cfg.validate()?;
        Ok(cfg)
    }
}
impl GatewayDraft {
    fn new(value: Box<GatewayConfig>) -> Self {
        let config = (*value).clone();
        let rows = config
            .forwards
            .iter()
            .map(|f| {
                (
                    f.protocol,
                    f.external_port,
                    f.internal_address.to_string(),
                    f.internal_port,
                )
            })
            .collect();
        Self {
            loaded: value,
            config,
            rows,
        }
    }
    fn parsed(&self, host: &Device) -> Result<GatewayConfig, Diagnostic> {
        let mut c = self.config.clone();
        c.forwards = self
            .rows
            .iter()
            .map(|(protocol, external_port, address, internal_port)| {
                Ok(PortForward {
                    protocol: *protocol,
                    external_port: *external_port,
                    internal_address: address
                        .parse::<Ipv4Addr>()
                        .map_err(|_| Diagnostic::new("nat.invalid_config", address))?,
                    internal_port: *internal_port,
                })
            })
            .collect::<Result<_, Diagnostic>>()?;
        c.validate(host)?;
        Ok(c)
    }
}
impl Workbench {
    fn security_submit(&mut self, device: &Id, command: SecurityCommand) {
        if self.mode == Mode::Simulation {
            self.worker.submit(Job::Command(Command::Security {
                device: device.clone(),
                command,
            }));
        } else if let Some(mut host) = self.project().device(device).cloned() {
            match command {
                SecurityCommand::ConfigureFirewall { config } => {
                    host.network.firewall = Some(Box::new(config))
                }
                SecurityCommand::ConfigureGateway { config } => {
                    host.network.gateway = Some(Box::new(config))
                }
                SecurityCommand::PurgeNat => return,
            }
            let config = self.project().lan.get(device).cloned().unwrap_or_default();
            self.apply(Edit::Configure(Box::new(host.clone()), config.clone()));
            self.draft = Some((host, config));
        }
    }
    pub(super) fn security_inspector(&mut self, ui: &mut egui::Ui, device: &Device) {
        if matches!(device.kind, DeviceKind::Switch | DeviceKind::Modem) {
            return;
        }
        // Always read the current worker document after web administration has edited it.
        let host = self
            .project()
            .device(&device.id)
            .cloned()
            .unwrap_or_else(|| device.clone());
        if host.network.gateway.is_some() {
            egui::CollapsingHeader::new(self.language.text("private_network"))
                .id_salt(("private-network", &host.id))
                .show(ui, |ui| self.gateway_panel(ui, &host));
        }
        egui::CollapsingHeader::new(self.language.text("firewall"))
            .id_salt(("firewall-panel", &host.id))
            .show(ui, |ui| self.firewall_panel(ui, &host));
    }
    pub(super) fn firewall_panel(&mut self, ui: &mut egui::Ui, host: &Device) {
        let l = self.language;
        let mut draft = self
            .security_ui
            .firewalls
            .remove(&host.id)
            .unwrap_or_else(|| FirewallDraft::new(host.network.firewall.clone()));
        if draft.loaded != host.network.firewall
            && draft.parsed().ok()
                == Some(
                    draft
                        .loaded
                        .as_deref()
                        .cloned()
                        .unwrap_or_else(|| FirewallConfig {
                            enabled: false,
                            ..Default::default()
                        }),
                )
        {
            draft = FirewallDraft::new(host.network.firewall.clone());
        }
        let stale = draft.loaded != host.network.firewall;
        let mut save = false;
        let mut reset = false;
        ui.push_id(("firewall", &host.id), |ui| {
            if stale {
                ui.label(l.text("admin_stale"));
                reset = ui.button(l.text("reload_configuration")).clicked();
            }
            ui.label(l.text("firewall_hint"));
            ui.checkbox(&mut draft.config.enabled, l.text("firewall_enable"));
            egui::ComboBox::new("policy", l.text("firewall_default"))
                .selected_text(match draft.config.default_policy {
                    Verdict::Accept => "ACCEPT",
                    Verdict::Drop => "DROP",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut draft.config.default_policy,
                        Verdict::Accept,
                        "ACCEPT",
                    );
                    ui.selectable_value(&mut draft.config.default_policy, Verdict::Drop, "DROP");
                });
            ui.checkbox(&mut draft.config.syn_only, l.text("firewall_syn_only"));
            ui.checkbox(&mut draft.config.filter_udp, l.text("firewall_udp_enable"));
            ui.checkbox(&mut draft.config.drop_icmp, l.text("firewall_icmp_drop"));
            let mut remove = None;
            let mut movement = None;
            egui::ScrollArea::vertical()
                .id_salt("firewall-rules")
                .max_height(270.)
                .show(ui, |ui| {
                    let len = draft.rows.len();
                    for (i, r) in draft.rows.iter_mut().enumerate() {
                        ui.push_id(i, |ui| {
                            ui.separator();
                            ui.strong(format!("{} {}", l.text("firewall_rule"), i + 1));
                            field(ui, l.text("firewall_source"), &mut r.source);
                            field(ui, l.text("firewall_destination"), &mut r.destination);
                            egui::ComboBox::new("protocol", l.text("protocol"))
                                .selected_text(match r.protocol {
                                    FilterProtocol::Any => "*",
                                    FilterProtocol::Tcp => "TCP",
                                    FilterProtocol::Udp => "UDP",
                                })
                                .show_ui(ui, |ui| {
                                    for (p, t) in [
                                        (FilterProtocol::Any, "*"),
                                        (FilterProtocol::Tcp, "TCP"),
                                        (FilterProtocol::Udp, "UDP"),
                                    ] {
                                        ui.selectable_value(&mut r.protocol, p, t);
                                    }
                                });
                            field(ui, l.text("port"), &mut r.port);
                            egui::ComboBox::new("action", l.text("firewall_action"))
                                .selected_text(format!("{:?}", r.action).to_uppercase())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut r.action, Verdict::Accept, "ACCEPT");
                                    ui.selectable_value(&mut r.action, Verdict::Drop, "DROP");
                                });
                            ui.horizontal_wrapped(|ui| {
                                if ui.button(l.text("delete")).clicked() {
                                    remove = Some(i);
                                }
                                if ui
                                    .add_enabled(i > 0, egui::Button::new(l.text("move_up")))
                                    .clicked()
                                {
                                    movement = Some((i, i - 1));
                                }
                                if ui
                                    .add_enabled(
                                        i + 1 < len,
                                        egui::Button::new(l.text("move_down")),
                                    )
                                    .clicked()
                                {
                                    movement = Some((i, i + 1));
                                }
                            });
                        });
                    }
                });
            if let Some(i) = remove {
                draft.rows.remove(i);
            } else if let Some((a, b)) = movement {
                draft.rows.swap(a, b);
            }
            if draft.rows.len() < 128 && ui.button(l.text("firewall_add_rule")).clicked() {
                draft.rows.push(RuleDraft {
                    source: "*".into(),
                    destination: "*".into(),
                    port: "80".into(),
                    protocol: FilterProtocol::Tcp,
                    action: Verdict::Accept,
                });
            }
            let config = draft.parsed();
            if let Err(e) = &config {
                ui.colored_label(Color32::LIGHT_RED, l.describe_error(&e.to_string()));
            }
            save = ui
                .add_enabled(
                    !stale && config.is_ok() && self.mode != Mode::Documentation,
                    egui::Button::new(l.text("apply_firewall")),
                )
                .clicked();
            if let Some(rows) = self.worker.view.firewall.get(&host.id) {
                ui.collapsing(l.text("firewall_log"), |ui| {
                    for r in rows.iter().rev().take(32) {
                        ui.monospace(format!(
                            "{} µs · {} → {} · {} · {:?} · #{}",
                            r.time_us.0,
                            r.source,
                            r.destination,
                            r.protocol,
                            r.action,
                            r.rule.map(|i| i.to_string()).unwrap_or("—".into())
                        ));
                    }
                });
            }
        });
        if save {
            if let Ok(config) = draft.parsed() {
                self.security_submit(&host.id, SecurityCommand::ConfigureFirewall { config });
            }
        } else if !reset {
            self.security_ui.firewalls.insert(host.id.clone(), draft);
        }
    }
    fn gateway_panel(&mut self, ui: &mut egui::Ui, host: &Device) {
        let l = self.language;
        let g = host.network.gateway.clone().unwrap();
        let mut draft = self
            .security_ui
            .gateways
            .remove(&host.id)
            .unwrap_or_else(|| GatewayDraft::new(g.clone()));
        if draft.loaded != g && draft.parsed(host).ok() == Some((*draft.loaded).clone()) {
            draft = GatewayDraft::new(g.clone());
        }
        let stale = draft.loaded != g;
        let mut save = false;
        let mut reset = false;
        ui.push_id(("gateway", &host.id), |ui| {
            ui.label(l.text("nat_hint"));
            ui.label(format!(
                "LAN: {} · WAN: {}",
                draft.config.lan.as_str(),
                draft.config.wan.as_str()
            ));
            if stale {
                ui.label(l.text("admin_stale"));
                reset = ui.button(l.text("reload_configuration")).clicked();
            }
            ui.checkbox(&mut draft.config.nat_enabled, l.text("nat_enable"));
            let label = ui.label(l.text("nat_port_start"));
            ui.add(egui::DragValue::new(&mut draft.config.port_start).range(1..=65535))
                .labelled_by(label.id);
            let label = ui.label(l.text("nat_port_end"));
            ui.add(egui::DragValue::new(&mut draft.config.port_end).range(1..=65535))
                .labelled_by(label.id);
            let mut seconds = draft.config.idle_timeout_us.0 / 1_000_000;
            let label = ui.label(l.text("nat_timeout"));
            if ui
                .add(egui::DragValue::new(&mut seconds).range(1..=604800))
                .labelled_by(label.id)
                .changed()
            {
                draft.config.idle_timeout_us = Decimal(seconds * 1_000_000);
            }
            let mut remove = None;
            egui::ScrollArea::vertical()
                .id_salt("port-forwards")
                .max_height(240.)
                .show(ui, |ui| {
                    for (i, (proto, external, address, internal)) in
                        draft.rows.iter_mut().enumerate()
                    {
                        ui.push_id(i, |ui| {
                            ui.separator();
                            egui::ComboBox::new("forward-protocol", l.text("protocol"))
                                .selected_text(format!("{proto:?}").to_uppercase())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(proto, TransportProtocol::Tcp, "TCP");
                                    ui.selectable_value(proto, TransportProtocol::Udp, "UDP");
                                });
                            let label = ui.label(l.text("nat_external_port"));
                            ui.add(egui::DragValue::new(external).range(1..=65535))
                                .labelled_by(label.id);
                            field(ui, l.text("nat_internal_address"), address);
                            let label = ui.label(l.text("nat_internal_port"));
                            ui.add(egui::DragValue::new(internal).range(1..=65535))
                                .labelled_by(label.id);
                            if ui.button(l.text("delete")).clicked() {
                                remove = Some(i);
                            }
                        });
                    }
                });
            if let Some(i) = remove {
                draft.rows.remove(i);
            }
            if draft.rows.len() < 128 && ui.button(l.text("nat_add_forward")).clicked() {
                draft
                    .rows
                    .push((TransportProtocol::Tcp, 8080, "192.168.10.2".into(), 80));
            }
            let parsed = draft.parsed(host);
            if let Err(e) = &parsed {
                ui.colored_label(Color32::LIGHT_RED, l.describe_error(&e.to_string()));
            }
            save = ui
                .add_enabled(
                    !stale && parsed.is_ok(),
                    egui::Button::new(l.text("apply_nat")),
                )
                .clicked();
            if save {
                draft.config = parsed.unwrap();
            }
            if self.mode == Mode::Simulation {
                if ui.button(l.text("nat_purge")).clicked() {
                    self.security_submit(&host.id, SecurityCommand::PurgeNat);
                }
                ui.heading(l.text("nat_table"));
                egui::ScrollArea::horizontal()
                    .id_salt("nat-table")
                    .show(ui, |ui| {
                        for m in self.worker.view.nat.iter().filter(|m| m.device == host.id) {
                            ui.monospace(format!(
                                "{} · {} → {} · {}{} · {} · {} µs",
                                format!("{:?}", m.protocol).to_uppercase(),
                                m.internal,
                                m.external,
                                m.remote,
                                m.remote_port.map(|p| format!(":{p}")).unwrap_or_default(),
                                l.text(if m.forwarded {
                                    "nat_static"
                                } else {
                                    "nat_dynamic"
                                }),
                                m.expires_us.0.saturating_sub(self.worker.view.now)
                            ));
                        }
                    });
            }
        });
        if save {
            self.security_submit(
                &host.id,
                SecurityCommand::ConfigureGateway {
                    config: draft.config,
                },
            );
        } else if !reset {
            self.security_ui.gateways.insert(host.id.clone(), draft);
        }
    }
}
