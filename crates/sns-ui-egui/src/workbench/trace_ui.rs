//! Search/export and read-only playback of the recorded causal path.
use super::*;
use sns_trace::{ExportFormat, Journey};
#[derive(Default)]
pub(super) struct ObservationUi {
    from: String,
    through: String,
    filter_error: bool,
    path: Option<Journey>,
    worker_run: Option<Id>,
    global: bool,
    local_device: Option<Id>,
    cursor: usize,
    playing: bool,
    next_frame: f64,
    export: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
}
impl Workbench {
    pub fn filtered_event_ids(&self) -> Vec<Id> {
        self.trace()
            .iter()
            .filter(|e| self.traffic.matches(e, &self.filter))
            .map(|e| e.event_id.clone())
            .collect()
    }
    pub fn filtered_trace_bytes(&self, format: ExportFormat) -> Result<Vec<u8>, Diagnostic> {
        self.traffic.query.validate()?;
        sns_trace::export(
            self.trace()
                .iter()
                .filter(|e| self.traffic.matches(e, &self.filter)),
            format,
        )
    }
    pub fn select_trace_event(&mut self, event: &Id) -> bool {
        self.event_index = self
            .trace()
            .iter()
            .position(|e| &e.event_id == event && self.traffic.matches(e, &self.filter));
        self.event_index.is_some()
    }
    pub fn open_message_journey(&mut self, event: &Id) -> Result<(), Diagnostic> {
        let source = self
            .trace()
            .iter()
            .find(|e| &e.event_id == event)
            .ok_or_else(|| Diagnostic::new("trace.event_missing", event.as_str()))?;
        let path = Journey::build(self.trace(), &source.run_id, event)?;
        self.observation.worker_run = Some(self.run_id().clone());
        self.observation.local_device = Some(path.device.clone());
        self.observation.path = Some(path);
        self.observation.cursor = 0;
        self.observation.global = false;
        self.observation.playing = false;
        Ok(())
    }
    pub fn journey_event(&self) -> Option<&sns_engine::TraceEvent> {
        let path = self.observation.path.as_ref()?;
        if self.observation.worker_run.as_ref() != Some(self.run_id()) {
            return None;
        }
        let indices = if self.observation.global {
            path.indices.clone()
        } else {
            path.local_indices(self.trace(), self.observation.local_device.as_ref()?)
        };
        self.trace().get(*indices.get(self.observation.cursor)?)
    }
    pub(super) fn export_filtered_trace(&mut self, path: &std::path::Path, format: ExportFormat) {
        if let Err(e) = self.begin_trace_export(path.to_owned(), format) {
            self.error = Some(e);
        }
    }
    pub fn trace_export_busy(&self) -> bool {
        self.observation.export.is_some()
    }
    pub fn begin_trace_export(
        &mut self,
        path: PathBuf,
        format: ExportFormat,
    ) -> Result<(), String> {
        self.document_output_allowed(&path)?;
        if self.trace_export_busy() {
            return Err("state.pending_commands".into());
        }
        self.traffic.query.validate().map_err(|e| e.to_string())?;
        let resolved = |p: &std::path::Path| -> Result<PathBuf, String> {
            if p.exists() {
                p.canonicalize().map_err(|e| e.to_string())
            } else {
                p.parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(std::path::Path::new("."))
                    .canonicalize()
                    .map(|parent| parent.join(p.file_name().unwrap_or_default()))
                    .map_err(|e| e.to_string())
            }
        };
        if let Some(project) = &self.path
            && resolved(project)? == resolved(&path)?
        {
            return Err("runtime.project_path_conflict".into());
        }
        let events = self.worker.view.trace.clone();
        let filter = self.traffic.clone();
        let text = self.filter.clone();
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("trace-export".into())
            .spawn(move || {
                let result =
                    sns_trace::export(events.iter().filter(|e| filter.matches(e, &text)), format)
                        .map_err(|e| e.to_string())
                        .and_then(|bytes| {
                            sns_project::atomic_write_runtime(&path, &bytes)
                                .map_err(|e| e.to_string())
                        });
                let _ = tx.send(result);
            })
            .map_err(|e| e.to_string())?;
        self.observation.export = Some(rx);
        Ok(())
    }
    pub(super) fn trace_search_ui(&mut self, ui: &mut egui::Ui) {
        let l = self.language;
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::new("trace-device", l.text("device"))
                .selected_text(
                    self.traffic
                        .query
                        .device
                        .as_ref()
                        .map(Id::as_str)
                        .unwrap_or(l.text("all_devices")),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.traffic.query.device,
                        None,
                        l.text("all_devices"),
                    );
                    for device in &self.worker.view.project.devices {
                        ui.selectable_value(
                            &mut self.traffic.query.device,
                            Some(device.id.clone()),
                            format!("{} · {}", device.label, device.id.as_str()),
                        );
                    }
                });
            for (key, field, width) in [
                ("trace_from_us", &mut self.observation.from, 125.),
                ("trace_through_us", &mut self.observation.through, 125.),
                ("trace_protocol", &mut self.traffic.query.protocol, 90.),
                ("trace_address", &mut self.traffic.query.address, 145.),
                (
                    "trace_correlation",
                    &mut self.traffic.query.correlation,
                    130.,
                ),
            ] {
                let label = ui.label(l.text(key));
                ui.add(
                    egui::TextEdit::singleline(field)
                        .desired_width(width)
                        .char_limit(128),
                )
                .labelled_by(label.id);
            }
            if ui.button(l.text("reset_trace_filters")).clicked() {
                self.traffic.query = Default::default();
                self.observation.from.clear();
                self.observation.through.clear();
                self.filter.clear();
                self.traffic.interface = None;
                self.traffic.layers = [true; 4];
            }
        });
        let parse = |s: &str| -> Result<Option<u64>, ()> {
            if s.trim().is_empty() {
                Ok(None)
            } else {
                s.trim().parse().map(Some).map_err(|_| ())
            }
        };
        self.observation.filter_error = match (
            parse(&self.observation.from),
            parse(&self.observation.through),
        ) {
            (Ok(a), Ok(b)) => {
                self.traffic.query.from_us = a;
                self.traffic.query.through_us = b;
                self.traffic.query.validate().is_err()
            }
            _ => {
                self.traffic.query.from_us = Some(1);
                self.traffic.query.through_us = Some(0);
                true
            }
        };
        if self.observation.filter_error {
            ui.colored_label(Color32::LIGHT_RED, l.text("trace.invalid_filter"));
        }
    }
    pub(super) fn journey_window(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.observation.export {
            match rx.try_recv() {
                Ok(result) => {
                    self.observation.export = None;
                    if let Err(e) = result {
                        self.error = Some(e);
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.observation.export = None;
                    self.error = Some("runtime.worker_failed".into());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(30))
                }
            }
        }
        if self.observation.path.is_some()
            && self.observation.worker_run.as_ref() != Some(self.run_id())
        {
            self.observation.path = None;
            self.observation.playing = false;
        }
        let Some(path) = self.observation.path.clone() else {
            return;
        };
        let events = self.worker.view.trace.clone();
        let l = self.language;
        let mut open = true;
        egui::Window::new(l.text("message_journey"))
            .id(egui::Id::unique("message-journey"))
            .default_pos([900., 180.])
            .default_size([760., 620.])
            .vscroll(true)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(l.text("journey_read_only"));
                ui.label(format!(
                    "{} · {}",
                    path.run_id.as_str(),
                    path.anchor.as_str()
                ));
                if path.incomplete {
                    ui.colored_label(Color32::LIGHT_YELLOW, l.text("journey_incomplete"));
                }
                ui.horizontal_wrapped(|ui| {
                    let mut changed = ui
                        .selectable_value(
                            &mut self.observation.global,
                            false,
                            l.text("journey_local"),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut self.observation.global,
                            true,
                            l.text("journey_global"),
                        )
                        .changed();
                    if !self.observation.global {
                        let mut devices = BTreeSet::new();
                        for &i in &path.indices {
                            devices.insert(events[i].device_id.clone());
                        }
                        egui::ComboBox::new("journey-device", l.text("device"))
                            .selected_text(
                                self.observation
                                    .local_device
                                    .as_ref()
                                    .map(Id::as_str)
                                    .unwrap_or("—"),
                            )
                            .show_ui(ui, |ui| {
                                for id in devices {
                                    changed |= ui
                                        .selectable_value(
                                            &mut self.observation.local_device,
                                            Some(id.clone()),
                                            id.as_str(),
                                        )
                                        .changed();
                                }
                            });
                    }
                    if changed {
                        self.observation.cursor = 0;
                        self.observation.playing = false;
                    }
                    if ui.button(l.text("journey_show_network")).clicked() {
                        self.page = Page::Topology;
                    }
                });
                let indices = if self.observation.global {
                    path.indices.clone()
                } else {
                    path.local_indices(
                        &events,
                        self.observation
                            .local_device
                            .as_ref()
                            .unwrap_or(&path.device),
                    )
                };
                if indices.is_empty() {
                    ui.label(l.text("no_events"));
                    return;
                }
                self.observation.cursor = self.observation.cursor.min(indices.len() - 1);
                ui.horizontal_wrapped(|ui| {
                    if ui.button(l.text("journey_restart")).clicked() {
                        self.observation.cursor = 0;
                        self.observation.playing = false;
                    }
                    if ui
                        .button(if self.observation.playing {
                            l.text("journey_pause")
                        } else {
                            l.text("journey_play")
                        })
                        .clicked()
                    {
                        if self.observation.cursor == indices.len() - 1 {
                            self.observation.cursor = 0;
                        }
                        self.observation.playing = !self.observation.playing;
                        self.observation.next_frame = ctx.input(|i| i.time) + 0.6;
                    }
                    if ui
                        .add_enabled(
                            self.observation.cursor > 0,
                            egui::Button::new(l.text("journey_previous")),
                        )
                        .clicked()
                    {
                        self.observation.cursor -= 1;
                        self.observation.playing = false;
                    }
                    if ui
                        .add_enabled(
                            self.observation.cursor + 1 < indices.len(),
                            egui::Button::new(l.text("journey_next")),
                        )
                        .clicked()
                    {
                        self.observation.cursor += 1;
                        self.observation.playing = false;
                    }
                    ui.label(format!(
                        "{} / {}",
                        self.observation.cursor + 1,
                        indices.len()
                    ));
                });
                if self.observation.playing {
                    if ctx.input(|i| i.time) >= self.observation.next_frame {
                        self.observation.cursor =
                            (self.observation.cursor + 1).min(indices.len() - 1);
                        self.observation.next_frame = ctx.input(|i| i.time) + 0.6;
                        if self.observation.cursor == indices.len() - 1 {
                            self.observation.playing = false;
                        }
                    }
                    ctx.request_repaint_after(std::time::Duration::from_millis(50));
                }
                let event = &events[indices[self.observation.cursor]];
                ui.strong(format!(
                    "{} · {} · {} µs · {}",
                    event.device_id.as_str(),
                    event.event_id.as_str(),
                    event.time_us.0,
                    event.kind
                ));
                ui.label(format!(
                    "{}: {} · {}: {}",
                    l.text("correlation"),
                    event.correlation_id.as_str(),
                    l.text("journey_cause"),
                    event.caused_by.as_ref().map(Id::as_str).unwrap_or("—")
                ));
                if self.observation.global {
                    ui.label(l.text("journey_timeline"));
                    if let Some(parent) = event
                        .caused_by
                        .as_ref()
                        .and_then(|id| events.iter().find(|e| &e.event_id == id))
                    {
                        ui.label(format!(
                            "{}: {} ({}) > {} ({})",
                            l.text("journey_cause"),
                            parent.device_id.as_str(),
                            parent.event_id.as_str(),
                            event.device_id.as_str(),
                            event.event_id.as_str()
                        ));
                    }
                    // A causal route strip keeps repeated visits (forward/reply) visible.
                    let mut hops: Vec<(Id, usize)> = Vec::new();
                    let mut active = 0;
                    for (n, &i) in indices.iter().enumerate() {
                        if hops.last().map(|(id, _)| id) != Some(&events[i].device_id) {
                            hops.push((events[i].device_id.clone(), n));
                        }
                        if n == self.observation.cursor {
                            active = hops.len() - 1;
                        }
                    }
                    egui::ScrollArea::horizontal()
                        .id_salt("journey-route")
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for (n, (device, stage)) in hops
                                    .iter()
                                    .enumerate()
                                    .take(active + 5)
                                    .skip(active.saturating_sub(3))
                                {
                                    if n > 0 {
                                        ui.label("·");
                                    }
                                    if ui
                                        .add(
                                            egui::Button::new(device.as_str())
                                                .selected(n == active),
                                        )
                                        .clicked()
                                    {
                                        self.observation.cursor = *stage;
                                        self.observation.playing = false;
                                    }
                                }
                            });
                        });
                }
                let parts = sns_trace::layer_parts(event);
                for (i, key) in [
                    "link_layer",
                    "network_layer",
                    "transport_layer",
                    "application_layer",
                ]
                .iter()
                .enumerate()
                .rev()
                {
                    let active = sns_trace::layer(event) == i;
                    egui::Frame::group(ui.style())
                        .fill(if active {
                            Color32::from_rgb(25, 78, 100)
                        } else {
                            Color32::TRANSPARENT
                        })
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 16.).max(160.));
                            ui.strong(format!("{}{}", if active { "> " } else { "" }, l.text(key)));
                            if let Some(part) = &parts[i] {
                                egui::CollapsingHeader::new(l.text("journey_recorded_data"))
                                    .id_salt(("journey-layer", i))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(
                                                serde_json::to_string_pretty(part).unwrap(),
                                            )
                                            .monospace(),
                                        );
                                    });
                            } else {
                                ui.label(l.text("journey_no_data"));
                            }
                        });
                }
            });
        if !open {
            self.observation.path = None;
            self.observation.playing = false;
        }
    }
}
