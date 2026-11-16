//! Workbench toolbar: workspaces, level, file commands and the current instruction.
//! Every button posts the same transaction as the headless path; the level only
//! chooses which commands are offered.
use super::*;
use sns_settings::display::Level;

const BUILD: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);
const TEST: Color32 = Color32::from_rgb(0xC2, 0x41, 0x0C);
const ANNOTATE: Color32 = Color32::from_rgb(0x15, 0x80, 0x3D);

impl Workbench {
    pub(super) fn header(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        ui.horizontal_wrapped(|ui| {
            ui.set_row_height(40.);
            ui.heading(egui::RichText::new("Sinesi").strong());
            ui.add_space(8.);
            for (mode, key, accent) in [
                (Mode::Design, "mode_build", BUILD),
                (Mode::Simulation, "mode_test", TEST),
                (Mode::Documentation, "mode_annotate", ANNOTATE),
            ] {
                let shortcut = match mode {
                    Mode::Design => "F2",
                    Mode::Simulation => "F3",
                    Mode::Documentation => "F4",
                };
                if mode_tab(
                    ui,
                    self.mode == mode,
                    &format!("{} · {shortcut}", language.text(key)),
                    accent,
                )
                .clicked()
                {
                    self.set_mode(mode);
                }
            }
            let missions = egui::Button::selectable(
                self.missions.open || self.missions.active.is_some(),
                egui::RichText::new(language.text("missions"))
                    .size(17.)
                    .strong(),
            )
            .stroke(Stroke::new(2., Color32::from_rgb(0xF5, 0x9E, 0x0B)))
            .min_size(Vec2::new(0., 40.))
            .corner_radius(10.);
            if ui.add(missions).clicked() {
                self.missions.open = !self.missions.open;
            }
            ui.separator();
            self.level_switch(ui);
            ui.separator();
            egui::ComboBox::new("ui-language", language.text("language"))
                .selected_text(self.language.label())
                .show_ui(ui, |ui| {
                    for value in Language::ALL {
                        if ui
                            .selectable_label(self.language == value, value.label())
                            .clicked()
                        {
                            self.choose_language(value);
                        }
                    }
                });
            if ui.button(language.text("preferences")).clicked() {
                self.toggle_preferences();
            }
            if ui
                .button(format!("{} · F1", language.text("help")))
                .clicked()
            {
                self.toggle_help();
            }
        });
        // Row 2: document commands; Expert adds the advanced transactions.
        ui.horizontal_wrapped(|ui| {
            if self.mode != Mode::Simulation {
                if ui
                    .add_enabled(
                        self.editor.can_undo(),
                        egui::Button::new(language.text("undo")),
                    )
                    .clicked()
                {
                    self.editor.undo();
                    self.refresh_draft();
                }
                if ui
                    .add_enabled(
                        self.editor.can_redo(),
                        egui::Button::new(language.text("redo")),
                    )
                    .clicked()
                {
                    self.editor.redo();
                    self.refresh_draft();
                }
            }
            if ui.button(language.text("new")).clicked() {
                self.replace(Replacement::New);
            }
            if ui.button(language.text("open")).clicked() {
                self.replace(Replacement::Open);
            }
            if ui.button(language.text("save")).clicked() {
                self.save(false);
            }
            if ui.button(language.text("save_as")).clicked() {
                self.save(true);
            }
            self.examples_menu(ui);
            self.session_toolbar(ui);
            if self.expert() {
                if ui.button(language.text("import_filius")).clicked() {
                    self.replace(Replacement::ImportFilius);
                }
                if self.import_ui.report.is_some()
                    && ui.button(language.text("import_report")).clicked()
                {
                    self.import_ui.open = true;
                }
                if ui.button(language.text("runtime_menu")).clicked() {
                    self.runtime_ui.open = true;
                }
                if ui.button(language.text("export_network")).clicked() {
                    self.export_ui.open = true;
                }
            }
        });
    }
    fn examples_menu(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        egui::ComboBox::from_id_salt("routing-examples")
            .height(320.)
            .selected_text(language.text("examples_menu"))
            .show_ui(ui, |ui| {
                for example in [Example::Lan, Example::Hosts].into_iter().chain(
                    Example::ALL
                        .into_iter()
                        .filter(|e| !matches!(e, Example::Lan | Example::Hosts)),
                ) {
                    let key = match example {
                        Example::Lan => "example",
                        Example::Hosts => "software_example",
                        _ => example.label_key(),
                    };
                    if ui.button(language.text(key)).clicked() {
                        self.replace(Replacement::Example(example));
                        ui.close();
                    }
                }
            })
            .response
            .widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::Role::ComboBox,
                    true,
                    language.text("examples_menu"),
                )
            });
    }
    fn level_switch(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        ui.label(language.text("level"));
        for (level, key) in [
            (Level::Student, "level_student"),
            (Level::Expert, "level_expert"),
        ] {
            if ui
                .add(
                    egui::Button::selectable(self.level() == level, language.text(key))
                        .min_size(Vec2::new(0., 28.)),
                )
                .clicked()
            {
                self.choose_level(level);
            }
        }
    }
    pub(super) fn hint_strip(&self, ui: &mut egui::Ui) {
        let (key, accent) = match self.mode {
            Mode::Design => ("hint_build", BUILD),
            Mode::Simulation => ("hint_test", TEST),
            Mode::Documentation => ("hint_annotate", ANNOTATE),
        };
        egui::Frame::new()
            .fill(accent.gamma_multiply(0.14))
            .stroke(Stroke::new(1., accent.gamma_multiply(0.5)))
            .corner_radius(8.)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new(self.language.text(key)).size(15.));
            });
    }
    pub(super) fn simulation_bar(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.worker.view.running {
                    language.text("pause")
                } else {
                    language.text("resume")
                })
                .clicked()
            {
                self.worker.submit(Job::Running(!self.worker.view.running));
            }
            if ui
                .add_enabled(
                    !self.worker.view.running,
                    egui::Button::new(language.text("step")),
                )
                .clicked()
            {
                self.worker.submit(Job::Step);
            }
            if ui.button(language.text("stop")).clicked() {
                self.set_mode(Mode::Design);
            }
            ui.label(language.text("speed"));
            if ui
                .add(
                    egui::Slider::new(&mut self.speed, 1..=100)
                        .suffix("×")
                        .text(language.text("speed")),
                )
                .changed()
            {
                self.worker.submit(Job::Speed(self.speed));
            }
            let loss = ui.button(language.text("drop_hold"));
            if loss.clicked() {
                self.runtime_ui.preserve_manual_loss = false;
            }
            let held = self.runtime_ui.preserve_manual_loss
                || loss.is_pointer_button_down_on()
                || (loss.has_focus() && ui.input(|i| i.key_down(egui::Key::Space)));
            if held != self.manual_loss {
                self.manual_loss = held;
                self.worker
                    .submit(Job::Command(Command::ManualLoss { enabled: held }));
            }
            ui.label(if self.manual_loss {
                language.text("loss_on")
            } else if self
                .worker
                .view
                .link_conditions
                .values()
                .any(|c| c.loss_ppm > 0)
            {
                language.text("link_conditions_active")
            } else {
                language.text("loss_off")
            });
        });
        let state = language.text(if self.worker.view.running {
            "running"
        } else {
            "paused"
        });
        if self.expert() {
            ui.label(format!(
                "{} · {}",
                self.worker.view.run_id.as_str(),
                language
                    .format(
                        "runtime",
                        &[
                            ("time", &self.worker.view.now.to_string()),
                            ("pending", &self.worker.view.pending.to_string()),
                            ("facts", &self.trace().len().to_string()),
                            ("state", state),
                        ],
                    )
                    .unwrap()
            ));
        } else {
            ui.label(
                language
                    .format("simulation_state", &[("state", state)])
                    .unwrap(),
            );
        }
    }
}

fn mode_tab(ui: &mut egui::Ui, selected: bool, label: &str, accent: Color32) -> egui::Response {
    let text = egui::RichText::new(label).size(17.).strong();
    let button = if selected {
        egui::Button::selectable(true, text.color(Color32::WHITE)).fill(accent)
    } else {
        egui::Button::selectable(false, text)
    };
    ui.add(button.min_size(Vec2::new(0., 40.)).corner_radius(10.))
}
