//! Selection editing and potential-address warnings; never touches the running engine.
use super::*;
use sns_topology::{Alignment, potential_ipv4_conflicts};
impl Workbench {
    pub(super) fn duplicate_selected(&mut self) {
        let annotations_only = !self.selected.is_empty()
            && self
                .selected
                .iter()
                .all(|id| self.project().annotations.iter().any(|a| &a.id == id));
        if self.mode != Mode::Design && !(self.mode == Mode::Documentation && annotations_only) {
            self.error = Some("state.editing_locked".into());
            return;
        }
        let selected_x = self
            .project()
            .layout
            .devices
            .iter()
            .filter(|p| self.selected.contains(&p.device_id))
            .map(|p| p.x)
            .chain(
                self.project()
                    .annotations
                    .iter()
                    .filter(|a| self.selected.contains(&a.id))
                    .map(|a| a.x),
            )
            .reduce(f32::min)
            .unwrap_or(0.);
        let right = self
            .project()
            .layout
            .devices
            .iter()
            .map(|p| p.x)
            .chain(self.project().annotations.iter().map(|a| a.x))
            .reduce(f32::max)
            .unwrap_or(0.);
        self.apply(Edit::Duplicate {
            selection: self.selected.clone(),
            dx: right + 200. - selected_x,
            dy: 40.,
        });
    }
    pub(super) fn topology_tools(&mut self, ui: &mut egui::Ui) {
        let l = self.language;
        let any = self.selected.iter().any(|id| {
            self.project().device(id).is_some()
                || self.project().annotations.iter().any(|a| &a.id == id)
        });
        if ui
            .add_enabled(
                any,
                egui::Button::new(format!("{} · Ctrl/⌘D", l.text("topology_duplicate"))),
            )
            .clicked()
        {
            self.duplicate_selected();
        }
        let count = self
            .project()
            .devices
            .iter()
            .filter(|d| self.selected.contains(&d.id))
            .count();
        ui.menu_button(l.text("topology_align"), |ui| {
            for alignment in Alignment::ALL {
                let required = if matches!(
                    alignment,
                    Alignment::DistributeHorizontal | Alignment::DistributeVertical
                ) {
                    3
                } else {
                    2
                };
                if ui
                    .add_enabled(
                        count >= required,
                        egui::Button::new(l.text(alignment_key(alignment))),
                    )
                    .clicked()
                {
                    self.apply(Edit::Align {
                        selection: self.selected.clone(),
                        alignment,
                    });
                    ui.close();
                }
            }
        });
    }
    pub(super) fn ipv4_warning_panel(&mut self, ui: &mut egui::Ui) {
        let l = self.language;
        if self.duplicate_omitted > 0 {
            ui.label(
                l.format(
                    "topology_omitted_links",
                    &[("count", &self.duplicate_omitted.to_string())],
                )
                .unwrap(),
            );
        }
        let conflicts = potential_ipv4_conflicts(self.project());
        if conflicts.is_empty() {
            return;
        }
        egui::CollapsingHeader::new(l.text("topology_ipv4_title"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(l.text("topology_ipv4_note"));
                egui::ScrollArea::vertical()
                    .id_salt("ipv4-address-warnings")
                    .max_height(180.)
                    .show(ui, |ui| {
                        for conflict in conflicts {
                            let title = l
                                .format(
                                    "topology_ipv4_group",
                                    &[
                                        ("address", &conflict.address.to_string()),
                                        ("count", &conflict.interfaces.len().to_string()),
                                    ],
                                )
                                .unwrap();
                            ui.collapsing(title, |ui| {
                                for nic in conflict.interfaces {
                                    if let Some(d) = self.project().device(&nic.device) {
                                        let label = format!(
                                            "{} [{}]",
                                            d.label,
                                            d.interfaces
                                                .iter()
                                                .position(|i| i.id == nic.interface)
                                                .unwrap()
                                                + 1
                                        );
                                        if ui
                                            .selectable_label(
                                                self.selected.contains(&nic.device),
                                                label,
                                            )
                                            .clicked()
                                        {
                                            self.select(nic.device, false);
                                        }
                                    }
                                }
                            });
                        }
                    });
            });
    }
}
fn alignment_key(a: Alignment) -> &'static str {
    match a {
        Alignment::Left => "align_left",
        Alignment::HorizontalCenter => "align_center_x",
        Alignment::Right => "align_right",
        Alignment::Top => "align_top",
        Alignment::VerticalCenter => "align_center_y",
        Alignment::Bottom => "align_bottom",
        Alignment::DistributeHorizontal => "distribute_x",
        Alignment::DistributeVertical => "distribute_y",
    }
}
