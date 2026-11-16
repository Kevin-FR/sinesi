//! Workbench inspector responsibilities.
use super::*;

impl Workbench {
    pub(super) fn palette(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        ui.heading(language.text("devices"));
        let mut diagram = self.art.diagram;
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut diagram, false, language.text("view_icons"));
            ui.selectable_value(&mut diagram, true, language.text("view_diagram"));
        });
        if diagram != self.art.diagram {
            self.choose_representation(diagram);
        }
        ui.add_enabled_ui(self.mode == Mode::Design, |ui| {
            let tile = Vec2::new(
                ((ui.available_width() - ui.spacing().item_spacing.x) / 2.)
                    .floor()
                    .max(64.),
                78.,
            );
            let mut clicked = None;
            ui.horizontal_wrapped(|ui| {
                for kind in PALETTE {
                    let response = self.palette_tile(ui, kind, tile);
                    response.dnd_set_drag_payload(kind);
                    if response.clicked() {
                        clicked = Some(kind);
                    }
                }
            });
            if let Some(kind) = clicked {
                {
                    let n = self.project().devices.len();
                    let (x, y) = sns_topology::free_position(self.project());
                    self.apply(Edit::AddNamed {
                        name: format!("{} {}", kind_label(language, kind), n + 1),
                        kind,
                        x,
                        y,
                    });
                    if let Some(d) = self.project().devices.last() {
                        self.select(d.id.clone(), false);
                    }
                }
            }
            let cable = egui::Button::selectable(
                self.cabling,
                egui::RichText::new(format!("{} · C", language.text("cable_tool"))).strong(),
            )
            .min_size(Vec2::new(ui.available_width(), 36.));
            if ui.add(cable).clicked() {
                self.cabling = !self.cabling;
                self.cable_notice = None;
            }
            if !self.cabling {
                self.cable = None;
            }
            if ui.button(language.text("delete_selection")).clicked() {
                self.delete();
            }
            self.topology_tools(ui);
        });
        ui.separator();
        ui.heading(language.text("selection"));
        let devices = self.project().devices.clone();
        for d in devices {
            if ui
                .selectable_label(
                    self.selected.contains(&d.id),
                    format!(
                        "{} · {}",
                        self.device_label(&d),
                        self.project()
                            .lan
                            .get(&d.id)
                            .and_then(|c| c.symbol)
                            .map(|s| language.text(s.key()))
                            .unwrap_or_else(|| kind_label(language, d.kind))
                    ),
                )
                .clicked()
            {
                self.select(d.id, ui.input(|i| i.modifiers.shift));
            }
        }
        ui.separator();
        self.ipv4_warning_panel(ui);
        if self.mode == Mode::Documentation {
            self.annotation_palette(ui);
        }
    }

    pub(super) fn inspector(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        ui.heading(language.text("inspector"));
        if self.annotation_inspector(ui) {
            return;
        }
        if let Some(link) = self
            .selected
            .iter()
            .find(|id| self.project().links.iter().any(|l| &l.id == *id))
            .cloned()
            && ui.button(language.text("link_conditions_menu")).clicked()
        {
            self.open_link_conditions(Some(link));
        }
        let Some((mut device, mut config)) = self.draft.clone() else {
            ui.label(language.text("select_hint"));
            return;
        };
        ui.label(
            language
                .format("identity", &[("id", device.id.as_str())])
                .unwrap(),
        );
        self.cloud_checkbox(ui, &device, &mut config);
        ui.add_enabled_ui(
            self.mode == Mode::Design && !self.worker.pending_commands(),
            |ui| {
                let label = ui.label(language.text("device_name"));
                ui.text_edit_singleline(&mut device.label)
                    .labelled_by(label.id);
                egui::ComboBox::new("device-display-mode", language.text("display_mode"))
                    .selected_text(match config.display {
                        DisplayName::Name => language.text("name"),
                        DisplayName::Ipv4 => "IPv4",
                        DisplayName::Mac => "MAC",
                    })
                    .show_ui(ui, |ui| {
                        for (v, label) in [
                            (DisplayName::Name, language.text("name")),
                            (DisplayName::Ipv4, "IPv4"),
                            (DisplayName::Mac, "MAC"),
                        ] {
                            ui.selectable_value(&mut config.display, v, label);
                        }
                    });
                if self.mode == Mode::Design {
                    egui::ComboBox::new("device-symbol", language.text("device_symbol"))
                        .selected_text(
                            language.text(
                                config
                                    .symbol
                                    .unwrap_or(if device.network.ssid.is_some() {
                                        DeviceSymbol::AccessPoint
                                    } else {
                                        DeviceSymbol::automatic(device.kind)
                                    })
                                    .key(),
                            ),
                        )
                        .show_ui(ui, |ui| {
                            for symbol in DeviceSymbol::ALL
                                .into_iter()
                                .filter(|s| *s != DeviceSymbol::Cloud)
                            {
                                ui.selectable_value(
                                    &mut config.symbol,
                                    Some(symbol),
                                    language.text(symbol.key()),
                                );
                            }
                        });
                    ui.label(language.text("symbol_hint"));
                }
                if device.kind == DeviceKind::Modem {
                    let modem = device.network.modem.get_or_insert_with(Default::default);
                    ui.label(language.text("bridge_external_hint"));
                    let role_label = ui.label(language.text("bridge_role"));
                    egui::ComboBox::from_id_salt("modem-role")
                        .selected_text(language.text(if modem.role == ModemRole::Listen {
                            "bridge_listen_role"
                        } else {
                            "bridge_connect_role"
                        }))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut modem.role,
                                ModemRole::Listen,
                                language.text("bridge_listen_role"),
                            );
                            ui.selectable_value(
                                &mut modem.role,
                                ModemRole::Connect,
                                language.text("bridge_connect_role"),
                            );
                        })
                        .response
                        .labelled_by(role_label.id);
                    let label = ui.label(language.text("bridge_endpoint"));
                    ui.text_edit_singleline(&mut modem.endpoint)
                        .labelled_by(label.id);
                }
                if device.kind == DeviceKind::Switch {
                    let label = ui.label(language.text("sat_retention"));
                    ui.add(
                        egui::DragValue::new(&mut config.sat_retention_us.0)
                            .range(1..=86_400_000_000_u64),
                    )
                    .labelled_by(label.id);
                }
                if device.kind == DeviceKind::Switch
                    && device
                        .interfaces
                        .iter()
                        .any(|i| i.medium == Medium::Wireless)
                {
                    let mut ssid = device.network.ssid.clone().unwrap_or_default();
                    let label = ui.label(language.text("wireless_ssid"));
                    if ui
                        .text_edit_singleline(&mut ssid)
                        .labelled_by(label.id)
                        .changed()
                    {
                        device.network.ssid = if ssid.is_empty() { None } else { Some(ssid) };
                    }
                }
                for nic in &mut device.interfaces {
                    ui.separator();
                    ui.label(
                        language
                            .format(
                                "interface",
                                &[(
                                    "id",
                                    &if let Some(g) = &device.network.gateway {
                                        format!(
                                            "{} · {}",
                                            if nic.id == g.lan { "LAN" } else { "WAN" },
                                            nic.id.as_str()
                                        )
                                    } else {
                                        nic.id.as_str().to_string()
                                    },
                                )],
                            )
                            .unwrap(),
                    );
                    egui::ComboBox::new(
                        ("interface-medium", &nic.id),
                        language.text("connection_medium"),
                    )
                    .selected_text(language.text(if nic.medium == Medium::Ethernet {
                        "ethernet_medium"
                    } else {
                        "wireless_medium"
                    }))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut nic.medium,
                            Medium::Ethernet,
                            language.text("ethernet_medium"),
                        );
                        ui.selectable_value(
                            &mut nic.medium,
                            Medium::Wireless,
                            language.text("wireless_medium"),
                        );
                    });
                    if nic.medium == Medium::Ethernet {
                        device.network.wireless.remove(&nic.id);
                    } else if !matches!(device.kind, DeviceKind::Switch | DeviceKind::Modem) {
                        let mut ssid = device
                            .network
                            .wireless
                            .get(&nic.id)
                            .cloned()
                            .unwrap_or_default();
                        let label = ui.label(language.text("wireless_ssid"));
                        if ui
                            .text_edit_singleline(&mut ssid)
                            .labelled_by(label.id)
                            .changed()
                        {
                            if ssid.is_empty() {
                                device.network.wireless.remove(&nic.id);
                            } else {
                                device.network.wireless.insert(nic.id.clone(), ssid);
                            }
                        }
                    }
                    let label = ui.label(language.text("mac_address"));
                    ui.text_edit_singleline(&mut nic.mac).labelled_by(label.id);
                    if !matches!(device.kind, DeviceKind::Switch | DeviceKind::Modem) {
                        let mut automatic = matches!(nic.ipv4, Some(Ipv4Config::Dhcp));
                        if ui
                            .checkbox(&mut automatic, language.text("dhcp_client"))
                            .changed()
                        {
                            nic.ipv4 = if automatic {
                                Some(Ipv4Config::Dhcp)
                            } else {
                                None
                            };
                        }
                        let (mut address, mut prefix) = match &nic.ipv4 {
                            Some(Ipv4Config::Static {
                                address,
                                prefix_length,
                            }) => (address.clone(), *prefix_length),
                            _ => (String::new(), 24),
                        };
                        let label = ui.label(language.text("ipv4_address"));
                        if self.field_help() {
                            help(ui, language.text("help_ipv4"));
                        }
                        let changed = ui
                            .add_enabled(!automatic, egui::TextEdit::singleline(&mut address))
                            .labelled_by(label.id)
                            .changed();
                        let changed = ui
                            .add_enabled(
                                !automatic,
                                egui::DragValue::new(&mut prefix).range(0..=32).prefix("/ "),
                            )
                            .changed()
                            || changed;
                        if changed && !automatic {
                            nic.ipv4 = if address.is_empty() {
                                None
                            } else {
                                Some(Ipv4Config::Static {
                                    address: address.clone(),
                                    prefix_length: prefix,
                                })
                            };
                        }
                        if let Ok(address) = address.parse()
                            && let Ok((net, broadcast)) = sns_protocols::subnet(address, prefix)
                        {
                            ui.label(
                                language
                                    .format(
                                        "subnet",
                                        &[
                                            ("network", &net.to_string()),
                                            ("broadcast", &broadcast.to_string()),
                                        ],
                                    )
                                    .unwrap(),
                            );
                        }
                    }
                }
                if !device
                    .interfaces
                    .iter()
                    .any(|i| i.medium == Medium::Wireless)
                {
                    device.network.ssid = None;
                }
                if device.kind != DeviceKind::Modem {
                    let help_shown = self.field_help();
                    address_field(
                        ui,
                        language.text("gateway"),
                        help_shown.then(|| language.text("help_gateway")),
                        &mut device.network.default_gateway,
                    );
                    show_gateway_advice(ui, language, &sns_topology::gateway_advice(&device));
                    address_field(
                        ui,
                        language.text("dns_server"),
                        help_shown.then(|| language.text("help_dns")),
                        &mut device.network.dns_server,
                    );
                }
                let dhcp_valid = ui
                    .push_id(("dhcp", device.id.clone()), |ui| {
                        desktop::dhcp_controls(ui, language, &mut device)
                    })
                    .inner;
                if ui
                    .add_enabled(dhcp_valid, egui::Button::new(language.text("apply")))
                    .clicked()
                {
                    self.apply(Edit::Configure(Box::new(device.clone()), config.clone()));
                }
                if ui.button(language.text("disconnect")).clicked() {
                    self.apply(Edit::Disconnect(device.id.clone()));
                }
            },
        );
        if device.kind == DeviceKind::Modem {
            ui.separator();
            ui.label(language.text("bridge_external_hint"));
            if let Some(view) = self.worker.view.bridges.get(&device.id) {
                ui.label(language.text(&view.state));
                ui.monospace(&view.endpoint);
                if let Some(code) = &view.detail
                    && code != &view.state
                {
                    ui.label(language.text(code));
                }
            }
            ui.add_enabled_ui(self.mode == Mode::Simulation, |ui| {
                if ui.button(language.text("bridge_activate")).clicked() {
                    self.worker.submit(Job::BridgeStart {
                        device: device.id.clone(),
                        config: device.network.modem.clone().unwrap_or_default(),
                    });
                }
                if ui.button(language.text("bridge_deactivate")).clicked() {
                    self.worker.submit(Job::BridgeStop {
                        device: device.id.clone(),
                    });
                }
            });
        }
        self.draft = Some((device.clone(), config));
        ui.separator();
        ui.heading(language.text("ports"));
        for nic in &device.interfaces {
            let busy = self
                .project()
                .links
                .iter()
                .any(|l| l.endpoints.contains(&nic.id));
            if ui
                .add_enabled(
                    !busy && self.mode == Mode::Design,
                    egui::Button::new(format!(
                        "{} · {}",
                        if let Some(g) = &device.network.gateway {
                            format!(
                                "{} · {}",
                                if nic.id == g.lan { "LAN" } else { "WAN" },
                                nic.id.as_str()
                            )
                        } else {
                            nic.id.as_str().to_string()
                        },
                        if busy {
                            language.text("port_busy")
                        } else {
                            language.text("port_free")
                        }
                    )),
                )
                .clicked()
            {
                self.port_click(nic.id.clone());
            }
        }
        if let Some(port) = &self.cable {
            ui.label(
                language
                    .format("cable_start", &[("port", port.as_str())])
                    .unwrap(),
            );
        }
        if device.kind == DeviceKind::Modem {
            return;
        }
        if self.mode == Mode::Simulation {
            if !matches!(device.kind, DeviceKind::Switch | DeviceKind::Modem) {
                if ui.button(language.text("open_desktop")).clicked() {
                    self.desktop.host = Some(device.id.clone());
                    self.page = Page::Desktop;
                }
                ui.heading(language.text("ping_title"));
                let label = ui.label(language.text("ping_target"));
                ui.text_edit_singleline(&mut self.target)
                    .labelled_by(label.id);
                ui.add(
                    egui::DragValue::new(&mut self.ttl)
                        .range(0..=255)
                        .prefix("TTL "),
                );
                if ui.button(language.text("send_ping")).clicked() {
                    self.worker.submit(Job::Command(Command::Ping {
                        source: device.id.clone(),
                        destination: self.target.clone(),
                        ttl: self.ttl,
                    }));
                }
                if ui.button(language.text("purge_arp")).clicked() {
                    self.worker.submit(Job::Command(Command::PurgeArp {
                        device: device.id.clone(),
                    }));
                }
            } else if ui.button(language.text("purge_sat")).clicked() {
                self.worker.submit(Job::Command(Command::PurgeSat {
                    device: device.id.clone(),
                }));
            }
            ui.label(language.text("paused_hint"));
        }
        self.routing_inspector(ui, &device);
        self.security_inspector(ui, &device);
    }
}

const PALETTE: [DeviceKind; 6] = [
    DeviceKind::Workstation,
    DeviceKind::Notebook,
    DeviceKind::Switch,
    DeviceKind::Router,
    DeviceKind::Gateway,
    DeviceKind::Modem,
];
/// Hue of a palette family; the active state also differs in lightness.
pub(super) fn kind_accent(kind: DeviceKind) -> Color32 {
    match kind {
        DeviceKind::Workstation | DeviceKind::Notebook => Color32::from_rgb(0x3B, 0x82, 0xF6),
        DeviceKind::Switch => Color32::from_rgb(0x14, 0xB8, 0xA6),
        DeviceKind::Router | DeviceKind::Gateway => Color32::from_rgb(0x8B, 0x5C, 0xF6),
        DeviceKind::Modem => Color32::from_rgb(0xEC, 0x48, 0x99),
    }
}
impl Workbench {
    /// One illustrated palette entry; its accessible name is the full add command.
    fn palette_tile(&mut self, ui: &mut egui::Ui, kind: DeviceKind, size: Vec2) -> egui::Response {
        let language = self.language;
        let full = language.text(match kind {
            DeviceKind::Workstation => "add_workstation",
            DeviceKind::Notebook => "add_notebook",
            DeviceKind::Switch => "add_switch",
            DeviceKind::Router => "add_router",
            DeviceKind::Gateway => "add_gateway",
            DeviceKind::Modem => "add_modem",
        });
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let enabled = ui.is_enabled();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, full));
        if ui.is_rect_visible(rect) {
            let accent = kind_accent(kind);
            let hovered = response.hovered() || response.has_focus();
            let fill = accent.gamma_multiply(if hovered { 0.32 } else { 0.18 });
            let stroke = Stroke::new(if hovered { 2.5 } else { 1.5 }, accent);
            let painter = ui.painter();
            painter.rect(rect, 12., fill, stroke, egui::StrokeKind::Inside);
            if response.has_focus() {
                painter.rect_stroke(
                    rect.expand(2.),
                    14.,
                    Stroke::new(2., ui.visuals().selection.stroke.color),
                    egui::StrokeKind::Outside,
                );
            }
            let texture = self
                .art
                .texture(ui.ctx(), sns_model::DeviceSymbol::automatic(kind));
            let icon =
                Rect::from_center_size(rect.center_top() + Vec2::new(0., 26.), Vec2::splat(36.));
            painter.image(
                texture,
                icon,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                Color32::WHITE,
            );
            let galley = painter.layout(
                kind_label(language, kind).to_owned(),
                egui::FontId::proportional(12.5),
                ui.visuals().text_color(),
                rect.width() - 8.,
            );
            let at = Pos2::new(
                rect.center().x - galley.size().x / 2.,
                rect.bottom() - 6. - galley.size().y,
            );
            painter.galley(at, galley, ui.visuals().text_color());
        }
        response.on_hover_text(full)
    }
}

impl Workbench {
    /// Field explanations accompany configuration: Student level, Build workspace.
    fn field_help(&self) -> bool {
        !self.expert() && self.mode == Mode::Design
    }
}
fn help(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(egui::RichText::new(text).size(13.).weak()).wrap());
}
fn address_field(ui: &mut egui::Ui, label: &str, hint: Option<&str>, value: &mut Option<String>) {
    let label = ui.label(label);
    if let Some(hint) = hint {
        help(ui, hint);
    }
    let mut text = value.clone().unwrap_or_default();
    if ui
        .text_edit_singleline(&mut text)
        .labelled_by(label.id)
        .changed()
    {
        *value = if text.is_empty() { None } else { Some(text) };
    }
}
/// Learner-facing check of the draft gateway; colour also differs in lightness.
fn show_gateway_advice(
    ui: &mut egui::Ui,
    language: Language,
    advice: &sns_topology::GatewayAdvice,
) {
    use sns_topology::GatewayAdvice as A;
    let (color, text) = match advice {
        A::NotApplicable => return,
        A::Reachable => (
            Color32::from_rgb(0x22, 0xC5, 0x5E),
            language.text("advice_gateway_ok").to_owned(),
        ),
        A::Missing => (
            Color32::from_rgb(0xF5, 0x9E, 0x0B),
            language.text("advice_gateway_missing").to_owned(),
        ),
        A::Invalid => (
            Color32::from_rgb(0xF8, 0x71, 0x71),
            language.text("advice_gateway_invalid").to_owned(),
        ),
        A::OwnAddress => (
            Color32::from_rgb(0xF8, 0x71, 0x71),
            language.text("advice_gateway_own").to_owned(),
        ),
        A::OutsideNetwork { network } => (
            Color32::from_rgb(0xF8, 0x71, 0x71),
            language
                .format("advice_gateway_outside", &[("network", network)])
                .unwrap(),
        ),
    };
    ui.add(egui::Label::new(egui::RichText::new(text).color(color)).wrap());
}
