//! "The packet's journey" panel shown under the canvas while testing.
use super::*;
use crate::story::Tone;

const STEPS: usize = 40;

impl Workbench {
    pub(super) fn story_panel(&mut self, ui: &mut egui::Ui, height: f32) {
        let l = self.language;
        let expert = self.expert();
        let steps = if self.worker.current() {
            crate::story::recent(&self.worker.view.project, l, self.trace(), STEPS)
        } else {
            vec![]
        };
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height((height - 24.).max(60.));
                ui.label(
                    egui::RichText::new(l.text("story_title"))
                        .strong()
                        .size(16.),
                );
                if steps.is_empty() {
                    ui.label(l.text("story_empty"));
                    return;
                }
                egui::ScrollArea::vertical()
                    .id_salt("packet-story")
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (n, step) in steps.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let color = match step.tone {
                                    Tone::Go => Color32::from_rgb(0xC2, 0x41, 0x0C),
                                    Tone::Info => Color32::from_rgb(0x6D, 0x28, 0xD9),
                                    Tone::Success => Color32::from_rgb(0x15, 0x80, 0x3D),
                                    Tone::Failure => Color32::from_rgb(0xB9, 0x1C, 0x1C),
                                };
                                let (badge, _) =
                                    ui.allocate_exact_size(Vec2::splat(22.), Sense::hover());
                                ui.painter().circle_filled(badge.center(), 11., color);
                                ui.painter().text(
                                    badge.center(),
                                    egui::Align2::CENTER_CENTER,
                                    (n + 1).to_string(),
                                    egui::FontId::proportional(11.),
                                    Color32::WHITE,
                                );
                                ui.vertical(|ui| {
                                    let mut text = egui::RichText::new(&step.text).size(14.5);
                                    if matches!(step.tone, Tone::Success | Tone::Failure) {
                                        text = text.strong();
                                    }
                                    ui.add(egui::Label::new(text).wrap());
                                    if expert {
                                        ui.label(
                                            egui::RichText::new(&step.detail)
                                                .monospace()
                                                .small()
                                                .weak(),
                                        );
                                    }
                                });
                            });
                        }
                    });
            });
    }
}
