//! Workbench chrome responsibilities.
use super::*;

impl Workbench {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        annotations::install_fonts(ui.ctx());
        self.apply_display_style(ui.ctx());
        self.poll_preferences(ui.ctx());
        self.poll_document_export(ui.ctx());
        self.poll_session(ui.ctx());
        if ui.input(|i| i.viewport().close_requested())
            || self.session_ui.close_requested
            || self.close_confirmed
            || matches!(self.pending, Some(Replacement::Close))
        {
            self.document_load = None;
        }
        if ui.input(|i| i.viewport().close_requested())
            && (self.session_enabled() || self.document_export_busy() || self.preferences_busy())
            && !self.close_confirmed
        {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.session_ui.close_requested = true;
        }
        if self.session_ui.loading {
            ui.spinner();
            ui.label(self.language.text("session_saving"));
            return;
        }
        self.poll_document_load(ui.ctx());
        if self.document_load.is_some() {
            ui.spinner();
            ui.label(self.language.text("document_opening"));
            return;
        }
        self.poll_runtime(ui.ctx());
        if self.runtime_busy() {
            ui.spinner();
            ui.label(self.language.text("runtime_working"));
            return;
        }
        self.runtime_window(ui.ctx());
        self.poll_import(ui.ctx());
        let language = self.language;
        if self.import_ui.receiver.is_some() {
            ui.spinner();
            ui.label(language.text("import_working"));
            return;
        }
        self.import_window(ui.ctx());
        self.document_export_window(ui.ctx());
        self.worker.attach_context(ui.ctx());
        self.worker.poll();
        let failed = self.worker.view.error.is_some();
        if failed && !self.simulation_error_visible {
            self.journal.record(TechnicalEvent::SimulationFailed);
        }
        self.simulation_error_visible = failed;
        self.sync_applications();
        if self.close_confirmed {
            if self.session_close_ready(ui.ctx()) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                ui.spinner();
                ui.label(self.language.text("session_saving"));
            }
            return;
        }
        self.link_conditions_window(ui.ctx());
        self.update_activity(ui.ctx());
        if (ui.input(|i| i.viewport().close_requested()) || self.session_ui.close_requested)
            && !self.close_confirmed
        {
            self.session_ui.close_requested = false;
            if self.editor.dirty() || self.desktop.dirty() || self.worker.pending_commands() {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.pending = Some(Replacement::Close);
            } else if self.session_enabled()
                || self.document_export_busy()
                || self.preferences_busy()
            {
                self.close_confirmed = true;
            }
        }
        if self.pending.is_none() {
            self.shortcuts(ui.ctx());
        }
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            self.header(ui);
            self.hint_strip(ui);
            self.update_mission();
            self.mission_card(ui);
            if self.mode == Mode::Simulation {
                self.simulation_bar(ui);
            }
            ui.horizontal(|ui| {
                for (page, label) in [
                    (Page::Topology, language.text("topology")),
                    (Page::Trace, language.text("exchanges")),
                    (Page::Tables, language.text("tables")),
                    (Page::Desktop, language.text("desktop")),
                ] {
                    ui.selectable_value(&mut self.page, page, label);
                }
            });
            if let Some(error) = self.error.as_ref().or(self.worker.view.error.as_ref()) {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(
                        Color32::LIGHT_RED,
                        language
                            .format("error", &[("message", &language.describe_error(error))])
                            .unwrap(),
                    );
                });
            }
            if let Some(error) = &self.settings_error {
                ui.label(
                    language
                        .format(
                            "settings_error",
                            &[("error", &language.describe_error(error))],
                        )
                        .unwrap(),
                );
            }
            ui.separator();
            self.desktop.top = ui.cursor().top();
            if self.page == Page::Trace {
                self.trace_ui(ui);
            } else if self.page == Page::Desktop {
                self.desktop_ui(ui);
            } else if self.page == Page::Tables {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.monospace(&self.worker.view.tables);
                });
            } else {
                self.topology_layout(ui);
            }
        });
        self.application_windows(ui.ctx());
        self.journey_window(ui.ctx());
        self.session_window(ui.ctx());
        self.help_window(ui.ctx());
        self.missions_window(ui.ctx());
        self.preferences_window(ui.ctx());
        self.configuration_window(ui.ctx());
        if self.close_confirmed && self.session_close_ready(ui.ctx()) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if let Some(action) = self.pending {
            egui::Modal::new(ui.scope_id().with("unsaved-project")).show(ui.ctx(), |ui| {
                ui.heading(language.text("unsaved_title"));
                ui.label(language.text("unsaved_prompt"));
                if ui
                    .add_enabled(
                        !self.desktop.dirty(),
                        egui::Button::new(language.text("save_continue")),
                    )
                    .clicked()
                    && self.save(false)
                {
                    self.pending = None;
                    self.replace_now(action);
                }
                if ui.button(language.text("discard")).clicked() {
                    if matches!(action, Replacement::Close) {
                        self.session_ui.discard_on_close = true;
                    }
                    self.pending = None;
                    self.replace_now(action);
                }
                if matches!(action, Replacement::Close)
                    && self.session_enabled()
                    && ui.button(language.text("session_keep_and_close")).clicked()
                {
                    self.pending = None;
                    self.session_ui.discard_on_close = false;
                    self.close_confirmed = true;
                }
                if ui.button(language.text("continue_editing")).clicked() {
                    self.pending = None;
                }
            });
        }
        if !self.close_confirmed {
            self.tick_session(ui.ctx());
        }
        if self.document_load.is_some() {
            ui.ctx().request_repaint();
        }
    }
    pub fn request_close(&mut self) {
        self.replace(Replacement::Close);
    }
}
