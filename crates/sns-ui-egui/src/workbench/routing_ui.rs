use super::*;
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoutingUi {
    network: String,
    prefix: u8,
    gateway: String,
    interface: Option<Id>,
    editing: Option<usize>,
    swap_a: Option<Id>,
    swap_b: Option<Id>,
    count: usize,
}
impl Default for RoutingUi {
    fn default() -> Self {
        Self {
            network: String::new(),
            prefix: 24,
            gateway: String::new(),
            interface: None,
            editing: None,
            swap_a: None,
            swap_b: None,
            count: 0,
        }
    }
}
impl Workbench {
    pub(super) fn routing_inspector(&mut self, ui: &mut egui::Ui, device: &Device) {
        if device.kind == DeviceKind::Switch {
            return;
        }
        let l = self.language;
        ui.separator();
        ui.heading(l.text("routing"));
        let design = self.mode == Mode::Design;
        let mut edited = device.clone();
        let config = self
            .project()
            .lan
            .get(&device.id)
            .cloned()
            .unwrap_or_default();
        ui.add_enabled_ui(design, |ui| {
            let mut forwarding = edited.network.forwarding.unwrap_or(matches!(
                device.kind,
                DeviceKind::Router | DeviceKind::Gateway
            ));
            if ui
                .checkbox(&mut forwarding, l.text("ip_forwarding"))
                .changed()
            {
                edited.network.forwarding = Some(forwarding);
                self.apply(Edit::Configure(Box::new(edited.clone()), config.clone()));
                self.draft = Some((edited.clone(), config.clone()));
            }
            if device.kind == DeviceKind::Router
                && ui
                    .checkbox(&mut edited.network.rip, l.text("enable_rip"))
                    .changed()
            {
                self.apply(Edit::Configure(Box::new(edited.clone()), config.clone()));
                self.draft = Some((edited.clone(), config.clone()));
            }
        });
        if self.mode == Mode::Simulation && device.network.rip {
            let mut muted = self.worker.view.rip_muted.contains(&device.id);
            if ui.checkbox(&mut muted, l.text("mute_rip")).changed() {
                self.worker.submit(Job::Command(Command::RipMute {
                    device: device.id.clone(),
                    muted,
                }));
            }
        }
        if device.network.rip {
            ui.label(l.text("rip_timers"));
        }
        let views = if self.mode == Mode::Simulation {
            self.worker
                .view
                .routes
                .get(&device.id)
                .cloned()
                .unwrap_or_default()
        } else {
            sns_protocols::routing::configured(device)
        };
        egui::ScrollArea::horizontal()
            .id_salt(("routing-table", &device.id))
            .show(ui, |ui| {
                egui::Grid::new(("routes", &device.id))
                    .striped(true)
                    .show(ui, |ui| {
                        for key in [
                            "route_network",
                            "route_gateway",
                            "interface_label",
                            "route_origin",
                            "route_metric",
                            "route_expiry",
                        ] {
                            ui.strong(l.text(key));
                        }
                        ui.end_row();
                        for route in views {
                            ui.label(format!("{}/{}", route.network, route.prefix));
                            ui.label(
                                route
                                    .gateway
                                    .map(|a| a.to_string())
                                    .unwrap_or_else(|| "—".into()),
                            );
                            ui.label(route.interface.as_ref().map(Id::as_str).unwrap_or("lo"));
                            ui.label(l.text(match route.origin {
                                sns_protocols::routing::Origin::Loopback => "route_loopback",
                                sns_protocols::routing::Origin::Direct => "route_direct",
                                sns_protocols::routing::Origin::Static => "route_static",
                                sns_protocols::routing::Origin::Rip => "route_rip",
                                sns_protocols::routing::Origin::Default => "route_default",
                            }));
                            if route.metric == 16 {
                                ui.colored_label(
                                    Color32::LIGHT_RED,
                                    format!("16 · {}", l.text("route_unreachable")),
                                );
                            } else if matches!(
                                route.origin,
                                sns_protocols::routing::Origin::Static
                                    | sns_protocols::routing::Origin::Default
                            ) {
                                ui.label("—");
                            } else {
                                ui.label(route.metric.to_string());
                            }
                            ui.label(
                                route
                                    .expires_us
                                    .map(|t| t.0.to_string())
                                    .unwrap_or_else(|| "—".into()),
                            );
                            ui.end_row();
                        }
                    });
            });
        ui.add_enabled_ui(design, |ui| {
            for (index, route) in device.network.routes.iter().enumerate() {
                ui.push_id(("static-route", index), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{}/{}", route.network, route.prefix));
                        if ui.button(l.text("edit_route")).clicked() {
                            self.routing_ui.network = route.network.to_string();
                            self.routing_ui.prefix = route.prefix;
                            self.routing_ui.gateway =
                                route.gateway.map(|a| a.to_string()).unwrap_or_default();
                            self.routing_ui.interface = Some(route.interface.clone());
                            self.routing_ui.editing = Some(index);
                        }
                        if ui.button(l.text("delete_route")).clicked() {
                            let mut next = device.clone();
                            next.network.routes.remove(index);
                            self.apply(Edit::Configure(Box::new(next.clone()), config.clone()));
                            self.draft = Some((next, config.clone()));
                            self.routing_ui.editing = None;
                        }
                    })
                });
            }
            let field = ui.label(l.text("route_network"));
            ui.text_edit_singleline(&mut self.routing_ui.network)
                .labelled_by(field.id);
            let field = ui.label(l.text("prefix"));
            ui.add(egui::DragValue::new(&mut self.routing_ui.prefix).range(0..=32))
                .labelled_by(field.id);
            let field = ui.label(l.text("route_gateway"));
            ui.text_edit_singleline(&mut self.routing_ui.gateway)
                .labelled_by(field.id);
            if self
                .routing_ui
                .interface
                .as_ref()
                .is_none_or(|id| !device.interfaces.iter().any(|i| &i.id == id))
            {
                self.routing_ui.interface = device.interfaces.first().map(|i| i.id.clone());
            }
            egui::ComboBox::new("route-interface", l.text("route_interface"))
                .selected_text(
                    self.routing_ui
                        .interface
                        .as_ref()
                        .map(Id::as_str)
                        .unwrap_or("—"),
                )
                .show_ui(ui, |ui| {
                    for nic in &device.interfaces {
                        ui.selectable_value(
                            &mut self.routing_ui.interface,
                            Some(nic.id.clone()),
                            nic.id.as_str(),
                        );
                    }
                });
            if ui.button(l.text("save_route")).clicked() {
                let result =
                    (|| -> Result<StaticRoute, Diagnostic> {
                        Ok(StaticRoute {
                            network: self
                                .routing_ui
                                .network
                                .parse()
                                .map_err(|_| Diagnostic::new("routing.invalid_route", "network"))?,
                            prefix: self.routing_ui.prefix,
                            gateway: if self.routing_ui.gateway.is_empty() {
                                None
                            } else {
                                Some(self.routing_ui.gateway.parse().map_err(|_| {
                                    Diagnostic::new("routing.invalid_route", "gateway")
                                })?)
                            },
                            interface: self.routing_ui.interface.clone().ok_or_else(|| {
                                Diagnostic::new("routing.invalid_route", "interface")
                            })?,
                        })
                    })();
                match result {
                    Ok(route) => {
                        let mut next = device.clone();
                        if let Some(index) = self
                            .routing_ui
                            .editing
                            .filter(|i| *i < next.network.routes.len())
                        {
                            next.network.routes[index] = route;
                        } else {
                            next.network.routes.push(route);
                        }
                        self.apply(Edit::Configure(Box::new(next.clone()), config.clone()));
                        if self.error.is_none() {
                            self.draft = Some((next, config.clone()));
                            self.routing_ui.editing = None;
                        }
                    }
                    Err(e) => self.error = Some(e.to_string()),
                }
            }
            if self.routing_ui.editing.is_some() && ui.button(l.text("cancel_route_edit")).clicked()
            {
                self.routing_ui.editing = None;
            }
            if device.kind == DeviceKind::Router {
                ui.separator();
                ui.label(l.text("router_connections"));
                if self.routing_ui.count < 2 {
                    self.routing_ui.count = device.interfaces.len();
                }
                let field = ui.label(l.text("interface_count"));
                ui.add(egui::DragValue::new(&mut self.routing_ui.count).range(2..=32))
                    .labelled_by(field.id);
                if ui.button(l.text("resize_interfaces")).clicked() {
                    self.apply(Edit::ResizeInterfaces {
                        device: device.id.clone(),
                        count: self.routing_ui.count,
                    });
                    self.refresh_draft();
                }
                for (key, value) in [
                    ("swap_from", &mut self.routing_ui.swap_a),
                    ("swap_to", &mut self.routing_ui.swap_b),
                ] {
                    if value
                        .as_ref()
                        .is_none_or(|id| !device.interfaces.iter().any(|i| &i.id == id))
                    {
                        *value = device.interfaces.first().map(|i| i.id.clone());
                    }
                    egui::ComboBox::new(key, l.text(key))
                        .selected_text(value.as_ref().map(Id::as_str).unwrap_or("—"))
                        .show_ui(ui, |ui| {
                            for nic in &device.interfaces {
                                ui.selectable_value(value, Some(nic.id.clone()), nic.id.as_str());
                            }
                        });
                }
                for nic in &device.interfaces {
                    let link = self
                        .project()
                        .links
                        .iter()
                        .find(|l| l.endpoints.contains(&nic.id));
                    ui.label(format!(
                        "{}: {}",
                        nic.id.as_str(),
                        link.map(|l| l.id.as_str()).unwrap_or("—")
                    ));
                }
                if ui.button(l.text("swap_connections")).clicked()
                    && let (Some(a), Some(b)) = (
                        self.routing_ui.swap_a.clone(),
                        self.routing_ui.swap_b.clone(),
                    )
                {
                    self.apply(Edit::SwapInterfaces {
                        device: device.id.clone(),
                        a,
                        b,
                    });
                    self.refresh_draft();
                }
            }
        });
    }
}
