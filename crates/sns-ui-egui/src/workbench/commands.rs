//! Workbench commands responsibilities.
use super::*;

impl Workbench {
    pub(super) fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Comma)) {
            self.toggle_preferences();
        }
        if ctx.input_mut(|i| i.key_pressed(egui::Key::Escape)) && self.dismiss_preferences() {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            return;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F1)) {
            self.toggle_help();
        }
        if ctx.input_mut(|i| i.key_pressed(egui::Key::Escape)) && self.dismiss_help() {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            return;
        }
        if ctx.text_edit_focused() {
            return;
        }
        // Use the key event's modifiers even if they were released before this frame.
        let save_as = ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::S,
            )
        });
        if save_as || ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.save(save_as);
        }
        let keys = ctx.input(|i| {
            (
                i.modifiers.command,
                i.modifiers.shift,
                i.key_pressed(egui::Key::N),
                i.key_pressed(egui::Key::O),
                i.key_pressed(egui::Key::Z),
                i.key_pressed(egui::Key::Y),
                i.key_pressed(egui::Key::Delete),
                i.key_pressed(egui::Key::C),
                i.key_pressed(egui::Key::F2),
                i.key_pressed(egui::Key::F3),
                i.key_pressed(egui::Key::F4),
                i.key_pressed(egui::Key::F5),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::D),
            )
        });
        let (command, shift, n, o, z, y, delete, c, f2, f3, f4, f5, escape, duplicate) = keys;
        if command {
            if n {
                self.replace(Replacement::New);
            }
            if o {
                self.replace(Replacement::Open);
            }
            if self.mode != Mode::Simulation {
                if duplicate {
                    self.duplicate_selected();
                }
                if z && !shift {
                    self.editor.undo();
                    self.duplicate_omitted = 0;
                    self.refresh_draft();
                }
                if y || (z && shift) {
                    self.editor.redo();
                    self.duplicate_omitted = 0;
                    self.refresh_draft();
                }
            }
        } else {
            if delete && self.mode == Mode::Design {
                self.delete();
            }
            if c && self.mode == Mode::Design {
                self.cabling = !self.cabling;
                self.cable_notice = None;
                self.cable = None;
            }
            if f2 {
                self.set_mode(Mode::Design);
            }
            if f3 {
                self.set_mode(Mode::Simulation);
            }
            if f4 {
                self.set_mode(Mode::Documentation);
            }
            if f5 {
                self.speed = if self.speed == 100 {
                    1
                } else {
                    self.speed * 10
                };
                self.worker.submit(Job::Speed(self.speed));
            }
        }
        if escape {
            self.cabling = false;
            self.cable = None;
            self.annotations.cancel_gesture();
        }
    }
    pub(super) fn refresh_draft(&mut self) {
        self.annotations.cancel_gesture();
        self.annotations.draft = self
            .project()
            .annotations
            .iter()
            .find(|a| self.selected.contains(&a.id))
            .cloned();
        let project = self.editor.project();
        self.selected.retain(|id| {
            project.device(id).is_some()
                || project.links.iter().any(|l| &l.id == id)
                || project.annotations.iter().any(|a| &a.id == id)
        });
        self.draft = self.selected.iter().find_map(|id| {
            self.project().device(id).map(|d| {
                (
                    d.clone(),
                    self.project().lan.get(id).cloned().unwrap_or_default(),
                )
            })
        });
    }
    pub(super) fn delete(&mut self) {
        self.annotations.cancel_gesture();
        self.annotations.draft = None;
        self.apply(Edit::Delete(self.selected.clone()));
        self.selected.clear();
        self.draft = None;
    }
}
