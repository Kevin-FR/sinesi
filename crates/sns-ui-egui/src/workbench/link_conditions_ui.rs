//! Runtime link conditions, kept outside the persistent topology document.
use super::*;
#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConditionsUi {
    pub open: bool,
    link: Option<Id>,
    run: Option<Id>,
    base: Option<LinkConditions>,
    delay_ms: f64,
    jitter_ms: f64,
    loss_percent: f64,
}
impl Workbench {
    pub fn link_conditions(&self) -> &std::collections::BTreeMap<Id, LinkConditions> {
        &self.worker.view.link_conditions
    }
    pub(super) fn open_link_conditions(&mut self, link: Option<Id>) {
        self.conditions_ui.open = true;
        if self.conditions_ui.link != link {
            self.conditions_ui.link = link;
            self.conditions_ui.base = None;
        }
    }
    pub(super) fn link_conditions_window(&mut self, ctx: &egui::Context) {
        if !self.conditions_ui.open {
            return;
        }
        let l = self.language;
        let links = self.project().links.clone();
        let mut open = true;
        let mut close = false;
        let mut apply = false;
        let mut reload = false;
        if !links
            .iter()
            .any(|v| Some(&v.id) == self.conditions_ui.link.as_ref())
        {
            self.conditions_ui.link = links.first().map(|v| v.id.clone());
            self.conditions_ui.base = None;
        }
        let selected = self.conditions_ui.link.clone();
        let label = |link: &sns_model::Link| {
            let endpoint = |port: &Id| {
                self.project()
                    .device_for_interface(port)
                    .map(|d| {
                        format!(
                            "{} [{}]",
                            d.label,
                            d.interfaces.iter().position(|i| &i.id == port).unwrap() + 1
                        )
                    })
                    .unwrap_or_default()
            };
            format!(
                "{} ↔ {}",
                endpoint(&link.endpoints[0]),
                endpoint(&link.endpoints[1])
            )
        };
        let entries: Vec<_> = links.iter().map(|v| (v.id.clone(), label(v))).collect();
        let current = links
            .iter()
            .find(|v| Some(&v.id) == selected.as_ref())
            .map(|link| {
                self.worker
                    .view
                    .link_conditions
                    .get(&link.id)
                    .cloned()
                    .unwrap_or(LinkConditions {
                        delay_us: link.latency_us,
                        jitter_us: link.jitter_us,
                        loss_ppm: 0,
                    })
            });
        if self.conditions_ui.run.as_ref() != Some(&self.worker.view.run_id)
            || self.conditions_ui.base.is_none()
        {
            self.conditions_ui.run = Some(self.worker.view.run_id.clone());
            self.conditions_ui.base = current.clone();
            if let Some(c) = &current {
                self.conditions_ui.delay_ms = c.delay_us.0 as f64 / 1000.;
                self.conditions_ui.jitter_ms = c.jitter_us.0 as f64 / 1000.;
                self.conditions_ui.loss_percent = c.loss_ppm as f64 / 10000.;
            }
        }
        let stale = current != self.conditions_ui.base && !self.worker.pending_commands();
        egui::Window::new(l.text("link_conditions_menu"))
            .id(egui::Id::unique("link-conditions"))
            .default_width(640.)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(l.text("link_conditions_scope"));
                ui.label(format!(
                    "{}: {}",
                    l.text("link_conditions_seed"),
                    self.worker.view.seed
                ));
                if entries.is_empty() {
                    ui.label(l.text("link_conditions_empty"));
                }
                egui::ComboBox::new("conditions-link", l.text("link_conditions_link"))
                    .width(500.)
                    .selected_text(
                        entries
                            .iter()
                            .find(|(id, _)| Some(id) == self.conditions_ui.link.as_ref())
                            .map(|(_, s)| s.as_str())
                            .unwrap_or("—"),
                    )
                    .show_ui(ui, |ui| {
                        for (id, label) in &entries {
                            ui.selectable_value(
                                &mut self.conditions_ui.link,
                                Some(id.clone()),
                                label,
                            );
                        }
                    });
                if selected != self.conditions_ui.link {
                    self.conditions_ui.base = None;
                    return;
                }
                numeric(
                    ui,
                    l.text("link_conditions_delay"),
                    &mut self.conditions_ui.delay_ms,
                    60000.,
                    " ms",
                    3,
                );
                numeric(
                    ui,
                    l.text("link_conditions_jitter"),
                    &mut self.conditions_ui.jitter_ms,
                    60000.,
                    " ms",
                    3,
                );
                numeric(
                    ui,
                    l.text("link_conditions_loss"),
                    &mut self.conditions_ui.loss_percent,
                    100.,
                    " %",
                    4,
                );
                if stale {
                    ui.label(l.text("link_conditions_changed"));
                    reload = ui.button(l.text("link_conditions_reload")).clicked();
                }
                apply = ui
                    .add_enabled(
                        self.mode == Mode::Simulation
                            && !stale
                            && selected.is_some()
                            && !self.worker.pending_commands(),
                        egui::Button::new(l.text("link_conditions_apply")),
                    )
                    .clicked();
                close = ui.button(l.text("close_view")).clicked();
            });
        self.conditions_ui.open = open && !close;
        if reload {
            self.conditions_ui.base = None;
        }
        if apply && let Some(link) = selected {
            let conditions = LinkConditions {
                delay_us: Decimal((self.conditions_ui.delay_ms * 1000.).round() as u64),
                jitter_us: Decimal((self.conditions_ui.jitter_ms * 1000.).round() as u64),
                loss_ppm: (self.conditions_ui.loss_percent * 10000.).round() as u32,
            };
            match conditions.validate() {
                Err(e) => self.error = Some(e.to_string()),
                Ok(()) => {
                    self.worker.submit(Job::Command(Command::ConfigureLink {
                        link,
                        conditions: conditions.clone(),
                    }));
                    self.conditions_ui.base = Some(conditions);
                }
            }
        }
    }
}

fn numeric(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    max: f64,
    suffix: &str,
    decimals: usize,
) {
    ui.horizontal(|ui| {
        let label = ui.label(label);
        ui.add(
            egui::DragValue::new(value)
                .range(0.0..=max)
                .speed(1.)
                .max_decimals(decimals)
                .suffix(suffix),
        )
        .labelled_by(label.id);
    });
}
