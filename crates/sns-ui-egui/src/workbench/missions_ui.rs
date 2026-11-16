//! Missions: a map of the builtin exercises and the card of the running one.
//! Objectives are checked by `sns-missions` on the applied project and the
//! current trace; the card never edits the project or drives the engine.
use super::*;
use sns_missions::{Mission, Report};

const GOLD: Color32 = Color32::from_rgb(0xF5, 0x9E, 0x0B);

pub(super) struct MissionRun {
    index: usize,
    hints: usize,
    /// Trace objectives already met during this mission, kept across runs.
    observed: BTreeSet<String>,
    recorded: bool,
}

#[derive(Default)]
pub(super) struct MissionsUi {
    pub open: bool,
    catalog: Vec<Mission>,
    /// Missions before this index are shipped with Sinesi; the rest come from the class.
    builtin: usize,
    pub active: Option<MissionRun>,
}

fn star(painter: &egui::Painter, center: Pos2, radius: f32, filled: bool) {
    let points: Vec<Pos2> = (0..10)
        .map(|i| {
            let angle = std::f32::consts::PI * (i as f32) / 5. - std::f32::consts::FRAC_PI_2;
            let r = if i % 2 == 0 { radius } else { radius * 0.45 };
            center + Vec2::new(angle.cos(), angle.sin()) * r
        })
        .collect();
    let fill = if filled { GOLD } else { Color32::TRANSPARENT };
    painter.add(egui::Shape::convex_polygon(
        points.clone(),
        fill,
        Stroke::NONE,
    ));
    let mut outline = points;
    outline.push(outline[0]);
    painter.add(egui::Shape::line(outline, Stroke::new(1.5, GOLD)));
}

/// Three stars with an accessible label such as "2 étoiles sur 3".
fn stars(ui: &mut egui::Ui, language: Language, earned: u8, size: f32) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(size * 3.3, size), Sense::hover());
    let label = language
        .format("mission_stars", &[("count", &earned.to_string())])
        .unwrap();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Image, true, &label));
    for i in 0..3 {
        star(
            ui.painter(),
            Pos2::new(rect.left() + size * (0.5 + 1.1 * i as f32), rect.center().y),
            size * 0.48,
            i < earned,
        );
    }
}

impl Workbench {
    fn mission_catalog(&mut self) -> &[Mission] {
        if self.missions.catalog.is_empty() {
            self.missions.catalog = sns_missions::builtin();
            self.missions.builtin = self.missions.catalog.len();
        }
        &self.missions.catalog
    }
    /// Missions shipped with Sinesi, in map order.
    pub fn missions(&mut self) -> Vec<String> {
        self.mission_catalog()
            .iter()
            .map(|m| m.id.clone())
            .collect()
    }
    pub(super) fn mission_project(&mut self, index: usize) -> Option<Project> {
        self.mission_catalog().get(index).map(|m| m.project.clone())
    }
    pub(super) fn begin_mission(&mut self, index: usize) {
        self.missions.active = Some(MissionRun {
            index,
            hints: 0,
            observed: BTreeSet::new(),
            recorded: false,
        });
        self.missions.open = false;
    }
    /// Stars saved for a mission in the user profile (0 when never completed).
    pub fn mission_stars(&self, id: &str) -> u8 {
        self.preferences_ui
            .persistent
            .missions
            .get(id)
            .copied()
            .unwrap_or(0)
    }
    /// Add (or refresh, by id) a teacher's mission for this session; returns its index.
    pub(super) fn add_teacher_mission(&mut self, mission: Mission) -> usize {
        self.mission_catalog();
        let builtin = self.missions.builtin;
        let catalog = &mut self.missions.catalog;
        match catalog
            .iter()
            .skip(builtin)
            .position(|m| m.id == mission.id)
        {
            Some(offset) => {
                catalog[builtin + offset] = mission;
                builtin + offset
            }
            None => {
                catalog.push(mission);
                catalog.len() - 1
            }
        }
    }
    pub(super) fn unlocked(&mut self, index: usize) -> bool {
        self.mission_catalog();
        // Class missions are chosen by the teacher: never locked.
        index == 0
            || index >= self.missions.builtin
            || self
                .mission_catalog()
                .get(index - 1)
                .map(|m| m.id.clone())
                .is_some_and(|id| self.mission_stars(&id) > 0)
    }
    /// The running mission's correction, as exported by `sinesi mission-check`.
    pub fn mission_report(&mut self) -> Option<Report> {
        let language = self.language.code().to_owned();
        let run = self.missions.active.as_ref()?;
        let (index, hints, observed) = (run.index, run.hints, run.observed.clone());
        let mission = self.mission_catalog().get(index)?.clone();
        let trace: &[sns_engine::TraceEvent] = if self.worker.current() {
            self.trace()
        } else {
            &[]
        };
        Some(sns_missions::report(
            &mission,
            self.project(),
            trace,
            &observed,
            hints,
            &language,
        ))
    }
    /// Latch trace objectives and save stars once the mission is complete.
    pub(super) fn update_mission(&mut self) {
        let Some(report) = self.mission_report() else {
            return;
        };
        let Some(index) = self.missions.active.as_ref().map(|r| r.index) else {
            return;
        };
        let mission = self.mission_catalog()[index].clone();
        let Some(run) = self.missions.active.as_mut() else {
            return;
        };
        for (objective, result) in mission.objectives.iter().zip(&report.objectives) {
            if result.done && objective.check.uses_trace() {
                run.observed.insert(objective.id.clone());
            }
        }
        if let (true, Some(earned), false) = (report.complete, report.stars, run.recorded) {
            run.recorded = true;
            let best = self.mission_stars(&mission.id).max(earned);
            self.preferences_ui
                .persistent
                .missions
                .insert(mission.id.clone(), best);
            self.queue_preferences();
        }
    }
    pub(super) fn missions_window(&mut self, ctx: &egui::Context) {
        if !self.missions.open {
            return;
        }
        let l = self.language;
        let code = l.code();
        let mut open = true;
        let mut start = None;
        let mut open_file = false;
        let catalog = self.mission_catalog().to_vec();
        egui::Window::new(l.text("missions"))
            .id(egui::Id::unique("missions-map"))
            .default_pos([120., 120.])
            .default_width(520.)
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.label(l.text("missions_intro"));
                if ui.button(l.text("mission_open_file")).clicked() {
                    open_file = true;
                }
                let builtin = self.missions.builtin;
                let mut chapter = 0;
                for (index, mission) in catalog.iter().enumerate() {
                    if index == builtin {
                        ui.add_space(6.);
                        ui.heading(l.text("mission_class"));
                    } else if index < builtin && mission.chapter != chapter {
                        chapter = mission.chapter;
                        ui.add_space(6.);
                        ui.heading(
                            l.format("mission_chapter", &[("number", &chapter.to_string())])
                                .unwrap(),
                        );
                    }
                    let unlocked = self.unlocked(index);
                    let earned = self.mission_stars(&mission.id);
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            stars(ui, l, earned, 18.);
                            ui.label(
                                egui::RichText::new(sns_missions::text(&mission.title, code))
                                    .strong()
                                    .size(16.),
                            );
                        });
                        ui.label(sns_missions::text(&mission.brief, code));
                        let label = l.text(if earned > 0 {
                            "mission_replay"
                        } else {
                            "mission_start"
                        });
                        let button = egui::Button::new(format!(
                            "{label} · {}",
                            sns_missions::text(&mission.title, code)
                        ));
                        if ui.add_enabled(unlocked, button).clicked() {
                            start = Some(index);
                        }
                        if !unlocked {
                            ui.label(egui::RichText::new(l.text("mission_locked")).weak());
                        }
                    });
                }
            });
        self.missions.open = open;
        if let Some(index) = start {
            self.replace(Replacement::Mission(index));
        }
        if open_file {
            self.replace(Replacement::OpenMission);
        }
    }
    /// The running mission under the instruction strip.
    pub(super) fn mission_card(&mut self, ui: &mut egui::Ui) {
        let Some(report) = self.mission_report() else {
            return;
        };
        let l = self.language;
        let code = l.code();
        let Some((index, hints)) = self.missions.active.as_ref().map(|r| (r.index, r.hints)) else {
            return;
        };
        let mission = self.mission_catalog()[index].clone();
        let next = (index + 1 < self.mission_catalog().len()).then_some(index + 1);
        let mut quit = false;
        let mut hint = false;
        let mut go_next = false;
        let accent = if report.complete {
            Color32::from_rgb(0x15, 0x80, 0x3D)
        } else {
            GOLD
        };
        egui::Frame::new()
            .fill(accent.gamma_multiply(0.12))
            .stroke(Stroke::new(1.5, accent))
            .corner_radius(10.)
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(
                            l.format(
                                "mission_heading",
                                &[("title", sns_missions::text(&mission.title, code))],
                            )
                            .unwrap(),
                        )
                        .strong()
                        .size(17.),
                    );
                    if let Some(earned) = report.stars {
                        stars(ui, l, earned, 20.);
                    }
                });
                if report.complete {
                    ui.label(
                        egui::RichText::new(l.text("mission_complete"))
                            .strong()
                            .size(16.)
                            .color(Color32::from_rgb(0x22, 0xC5, 0x5E)),
                    );
                } else {
                    ui.label(sns_missions::text(&mission.brief, code));
                }
                for objective in &report.objectives {
                    let state = l.text(if objective.done {
                        "mission_done"
                    } else {
                        "mission_todo"
                    });
                    let text = egui::RichText::new(format!("{state} · {}", objective.text));
                    ui.label(if objective.done {
                        text.color(Color32::from_rgb(0x22, 0xC5, 0x5E))
                    } else {
                        text.strong()
                    });
                }
                for text in mission.hints.iter().take(hints) {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} · {}",
                            l.text("mission_hint"),
                            sns_missions::text(text, code)
                        ))
                        .italics(),
                    );
                }
                ui.horizontal_wrapped(|ui| {
                    if !report.complete && hints < mission.hints.len() {
                        hint = ui.button(l.text("mission_hint_button")).clicked();
                    }
                    if !report.complete && ui.button(l.text("mission_check")).clicked() {
                        self.missions_feedback(&report);
                    }
                    if report.complete
                        && let Some(_) = next
                    {
                        go_next = ui.button(l.text("mission_next")).clicked();
                    }
                    quit = ui.button(l.text("mission_quit")).clicked();
                });
                // The explanation lasts while its objective stays unmet.
                if let Some(feedback) = self.missions_feedback_text.as_ref().and_then(|id| {
                    report
                        .objectives
                        .iter()
                        .find(|o| &o.id == id && !o.done)
                        .and_then(|o| o.explanation.as_ref())
                }) {
                    ui.label(
                        egui::RichText::new(feedback).color(Color32::from_rgb(0xF5, 0x9E, 0x0B)),
                    );
                }
            });
        if hint && let Some(run) = self.missions.active.as_mut() {
            run.hints += 1;
        }
        if quit {
            self.missions.active = None;
            self.missions_feedback_text = None;
        }
        if go_next && let Some(next) = next {
            self.replace(Replacement::Mission(next));
        }
    }
    fn missions_feedback(&mut self, report: &Report) {
        self.missions_feedback_text = report
            .objectives
            .iter()
            .find(|o| !o.done)
            .map(|o| o.id.clone());
    }
}
