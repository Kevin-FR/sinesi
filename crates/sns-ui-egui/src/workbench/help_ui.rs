//! Contextual offline help. Opening or navigating it never dispatches a network job.
use super::*;
use sns_settings::help::{self, Topic};
#[derive(Default)]
pub(super) struct HelpUi {
    open: bool,
    topic: Topic,
}
impl Workbench {
    pub(super) fn toggle_help(&mut self) {
        self.help_ui.open = !self.help_ui.open;
        if self.help_ui.open {
            self.help_ui.topic = match self.mode {
                Mode::Design => Topic::Design,
                Mode::Simulation => Topic::Simulation,
                Mode::Documentation => Topic::Documentation,
            };
        }
    }
    pub(super) fn dismiss_help(&mut self) -> bool {
        let was_open = self.help_ui.open;
        self.help_ui.open = false;
        was_open
    }
    pub(super) fn help_window(&mut self, ctx: &egui::Context) {
        if !self.help_ui.open {
            return;
        }
        let l = self.language;
        let mut open = true;
        let mut example = None;
        let window = egui::Window::new(l.text("help_title"))
            .id(egui::Id::unique("workbench-help"))
            .default_pos([290., 140.])
            .open(&mut open);
        let response = self
            .configured_window(ctx, "help", window, Vec2::new(760., 590.))
            .min_width(200.)
            .hscroll(true)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for topic in Topic::ALL {
                        ui.selectable_value(
                            &mut self.help_ui.topic,
                            topic,
                            l.text(topic.label_key()),
                        );
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("help-content")
                    .show(ui, |ui| {
                        ui.heading(l.text(self.help_ui.topic.label_key()));
                        for key in self.help_ui.topic.paragraph_keys() {
                            ui.label(l.text(key));
                            ui.add_space(10.);
                        }
                        match self.help_ui.topic {
                            Topic::Software => {
                                for software in Software::ALL {
                                    ui.strong(l.text(software.label_key()));
                                }
                            }
                            Topic::Examples => {
                                for lesson in Example::ALL {
                                    ui.push_id(lesson.label_key(), |ui| {
                                        ui.heading(l.text(lesson.label_key()));
                                        ui.label(l.text(lesson.instructions_key()));
                                        if ui
                                            .button(format!(
                                                "{} · {}",
                                                l.text("help_load_example"),
                                                l.text(lesson.label_key())
                                            ))
                                            .clicked()
                                        {
                                            example = Some(lesson);
                                        }
                                        ui.add_space(12.);
                                    });
                                }
                            }
                            Topic::About => {
                                egui::Grid::new("product-information")
                                    .num_columns(2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for (key, value) in [
                                            (
                                                "help_product_version",
                                                help::PRODUCT_VERSION.to_owned(),
                                            ),
                                            (
                                                "help_loaded_format",
                                                self.project().schema_version.to_string(),
                                            ),
                                            (
                                                "help_latest_format",
                                                DOCUMENTATION_SCHEMA_VERSION.to_string(),
                                            ),
                                            (
                                                "help_supported_formats",
                                                format!("1 - {DOCUMENTATION_SCHEMA_VERSION}"),
                                            ),
                                        ] {
                                            ui.label(l.text(key));
                                            ui.monospace(value);
                                            ui.end_row();
                                        }
                                    });
                                ui.add_space(12.);
                                ui.heading(l.text("help_credits"));
                                for key in [
                                    "help_credit_code",
                                    "help_credit_symbols",
                                    "help_credit_fonts",
                                    "help_credit_dependencies",
                                    "help_credit_reference",
                                ] {
                                    ui.label(l.text(key));
                                    ui.add_space(6.);
                                }
                                for (id, key, text) in [
                                    (
                                        "project-license",
                                        "help_code_license",
                                        help::PROJECT_LICENSE,
                                    ),
                                    ("font-license", "help_font_license", help::FONT_LICENSE),
                                ] {
                                    ui.push_id(id, |ui| {
                                        ui.collapsing(l.text(key), |ui| {
                                            ui.add(egui::Label::new(text).selectable(true).wrap());
                                        });
                                    });
                                }
                            }
                            _ => {}
                        }
                    });
            });
        if let Some(response) = response {
            self.remember_window(ctx, "help", &response.response);
        }
        self.help_ui.open = open;
        if let Some(example) = example {
            self.help_ui.open = false;
            self.replace(Replacement::Example(example));
        }
    }
}
