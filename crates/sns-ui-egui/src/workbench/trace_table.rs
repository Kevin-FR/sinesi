//! Workbench trace table responsibilities.
use super::*;

impl Workbench {
    pub(super) fn update_activity(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if self.mode != Mode::Simulation {
            self.active_links.clear();
            self.clear_flights();
            return;
        }
        if !self.worker.current() {
            return;
        }
        if self.trace().len() < self.observed_trace {
            self.observed_trace = 0;
            self.active_links.clear();
            self.clear_flights();
        }
        for index in self.observed_trace..self.trace().len() {
            self.queue_flight(index, now);
        }
        for event in &self.worker.view.trace[self.observed_trace..] {
            if event.kind == "ethernet.sent" || event.kind == "ethernet.received" {
                let port = crate::traffic::interface(event);
                for link in &self.worker.view.project.links {
                    if link.endpoints.iter().any(|p| p.as_str() == port) {
                        self.active_links.insert(link.id.clone(), now + 0.45);
                    }
                }
            }
        }
        self.observed_trace = self.trace().len();
        self.active_links.retain(|_, expires| *expires > now);
        if !self.active_links.is_empty() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
    pub(super) fn trace_ui(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        self.trace_search_ui(ui);
        ui.horizontal_wrapped(|ui| {
            let label = ui.label(language.text("filter"));
            ui.text_edit_singleline(&mut self.filter)
                .labelled_by(label.id);
            egui::ComboBox::new("trace-interface", language.text("interface_label"))
                .selected_text(
                    self.traffic
                        .interface
                        .as_ref()
                        .map(Id::as_str)
                        .unwrap_or(language.text("all_interfaces")),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.traffic.interface,
                        None,
                        language.text("all_interfaces"),
                    );
                    for device in &self.worker.view.project.devices {
                        for port in &device.interfaces {
                            ui.selectable_value(
                                &mut self.traffic.interface,
                                Some(port.id.clone()),
                                format!("{} · {}", device.label, port.id.as_str()),
                            );
                        }
                    }
                });
            ui.checkbox(&mut self.traffic.follow, language.text("follow"));
            if ui.button(language.text("clear_trace")).clicked() {
                self.traffic.after = self.trace().last().map(|e| e.sequence.0).unwrap_or(0);
                self.event_index = None;
            }
            if ui.button(language.text("export_text")).clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .set_file_name("traffic.txt")
                    .save_file()
            {
                let text = self
                    .traffic
                    .export_text(self.trace(), &self.filter, language);
                if let Err(e) = self.export_output_allowed(&path).and_then(|()| {
                    sns_project::atomic_write(&path, text.as_bytes()).map_err(|e| e.to_string())
                }) {
                    self.error = Some(e);
                }
            }
            if ui.button(language.text("export_jsonl")).clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .set_file_name("trace-lan.jsonl")
                    .save_file()
            {
                self.export_filtered_trace(&path, sns_trace::ExportFormat::Jsonl);
            }
            if ui.button(language.text("export_csv")).clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .set_file_name("trace-lan.csv")
                    .save_file()
            {
                self.export_filtered_trace(&path, sns_trace::ExportFormat::Csv);
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.traffic.layered, false, language.text("aggregate"));
            ui.selectable_value(&mut self.traffic.layered, true, language.text("layers"));
            for (index, key) in [
                "link_layer",
                "network_layer",
                "transport_layer",
                "application_layer",
            ]
            .iter()
            .enumerate()
            {
                ui.checkbox(&mut self.traffic.layers[index], language.text(key));
            }
        });
        let events = self.worker.view.trace.clone();
        let rows: Vec<_> = events
            .iter()
            .enumerate()
            .filter(|(_, e)| self.traffic.matches(e, &self.filter))
            .collect();
        if self
            .event_index
            .is_some_and(|i| !rows.iter().any(|(index, _)| *index == i))
        {
            self.event_index = None;
        }
        let height = (ui.available_height() * 0.55).max(80.);
        if rows.is_empty() {
            ui.label(language.text("no_events"));
        }
        egui_extras::TableBuilder::new(ui)
            .id_salt("network-trace-table")
            .resizable(true)
            .animate_scrolling(false)
            .striped(true)
            .max_scroll_height(height)
            .stick_to_bottom(self.traffic.follow)
            .column(egui_extras::Column::auto().at_least(65.).clip(true))
            .column(egui_extras::Column::auto().at_least(160.).clip(true))
            .column(egui_extras::Column::remainder().at_least(200.).clip(true))
            .column(egui_extras::Column::auto().at_least(105.).clip(true))
            .column(egui_extras::Column::auto().at_least(120.).clip(true))
            .column(egui_extras::Column::auto().at_least(90.).clip(true))
            .column(egui_extras::Column::auto().at_least(85.).clip(true))
            .header(24., |mut row| {
                for key in [
                    "number",
                    "time_us",
                    "event",
                    "device",
                    "interface_label",
                    "direction",
                    "protocol",
                ] {
                    row.col(|ui| {
                        ui.strong(language.text(key));
                    });
                }
            })
            .body(|body| {
                body.rows(23., rows.len(), |mut row| {
                    let (index, f) = rows[row.index()];
                    for value in [
                        f.sequence.0.to_string(),
                        f.time_us.0.to_string(),
                        f.kind.clone(),
                        f.device_id.as_str().into(),
                        crate::traffic::interface(f).into(),
                        crate::traffic::direction(f, language).into(),
                        crate::traffic::protocol(f).into(),
                    ] {
                        row.col(|ui| {
                            if ui
                                .selectable_label(self.event_index == Some(index), value)
                                .clicked()
                            {
                                self.event_index = Some(index);
                            }
                        });
                    }
                });
            });
        ui.separator();
        ui.heading(language.text("traffic_detail"));
        if let Some(event) = self.event_index.and_then(|i| events.get(i)) {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "{} · {}",
                    event.run_id.as_str(),
                    event.event_id.as_str()
                ));
                if ui.button(language.text("message_journey")).clicked()
                    && let Err(e) = self.open_message_journey(&event.event_id)
                {
                    self.error = Some(e.to_string());
                }
            });
            if let Some(reason) = event.payload.get("reason").and_then(|v| v.as_str()) {
                ui.label(format!(
                    "{}: {} · {}",
                    language.text("why_packet"),
                    language.describe_error(reason),
                    event.device_id.as_str()
                ));
            }
            if let Some(route) = event.payload.get("route") {
                ui.label(format!("{}: {}", language.text("selected_route"), route));
            }
            egui::ScrollArea::both()
                .id_salt("trace-details")
                .show(ui, |ui| {
                    if self.traffic.layered {
                        let value = serde_json::to_value(event).unwrap();
                        let frame = value.pointer("/payload/frame");
                        let packet = frame.and_then(|v| v.get("payload"));
                        for (key, part) in [
                            ("link_layer", frame),
                            ("network_layer", packet),
                            (
                                "transport_layer",
                                packet.and_then(|v| v.get("packet").or_else(|| v.get("icmp"))),
                            ),
                            (
                                "application_layer",
                                event.payload.get("data_base64").or_else(|| {
                                    packet.and_then(|v| v.pointer("/packet/data_base64"))
                                }),
                            ),
                        ] {
                            ui.collapsing(language.text(key), |ui| {
                                ui.monospace(
                                    part.map(|v| serde_json::to_string_pretty(v).unwrap())
                                        .unwrap_or_else(|| "—".into()),
                                );
                            });
                        }
                    } else {
                        ui.monospace(serde_json::to_string_pretty(event).unwrap());
                    }
                });
        } else {
            ui.label(language.text("traffic_hint"));
        }
    }
}
