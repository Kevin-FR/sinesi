//! Workbench canvas responsibilities.
use super::*;

impl Workbench {
    pub(super) fn port_click(&mut self, port: Id) {
        if let Some(start) = self.cable.take() {
            let names = [&start, &port].map(|p| self.port_device_name(p));
            self.apply(Edit::Connect { a: start, b: port });
            self.cable_notice = match (&self.error, names) {
                (None, [Some(a), Some(b)]) => self
                    .language
                    .format("cable_done", &[("a", &a), ("b", &b)])
                    .ok(),
                _ => None,
            };
        } else {
            self.cable = Some(port);
            self.cabling = true;
            self.cable_notice = None;
        }
    }
    fn port_device_name(&self, port: &Id) -> Option<String> {
        self.project()
            .devices
            .iter()
            .find(|d| d.interfaces.iter().any(|i| &i.id == port))
            .map(|d| self.device_label(d))
    }
    /// Step-by-step guidance for the two-click cable gesture, drawn over the canvas.
    fn cable_banner(&self, ui: &mut egui::Ui, rect: Rect) {
        if !self.cabling || self.mode != Mode::Design {
            return;
        }
        let l = self.language;
        let text = match (&self.cable, &self.cable_notice) {
            (Some(port), _) => l
                .format(
                    "cable_step_second",
                    &[("device", &self.port_device_name(port).unwrap_or_default())],
                )
                .unwrap(),
            (None, Some(notice)) => notice.clone(),
            (None, None) => l.text("cable_step_first").to_owned(),
        };
        let painter = ui.painter_at(rect);
        let galley = painter.layout(
            text,
            egui::FontId::proportional(15.),
            Color32::WHITE,
            rect.width() - 64.,
        );
        let pill = Rect::from_center_size(
            Pos2::new(rect.center().x, rect.bottom() - 32.),
            galley.size() + Vec2::new(32., 16.),
        );
        let fill = if self.cable.is_none() && self.cable_notice.is_some() {
            Color32::from_rgb(0x15, 0x80, 0x3D)
        } else {
            Color32::from_rgb(0x0F, 0x1E, 0x33)
        };
        painter.rect(
            pill,
            999.,
            fill,
            Stroke::new(2., Color32::from_rgb(0xFD, 0xBA, 0x74)),
            egui::StrokeKind::Outside,
        );
        // A real label keeps the instruction readable by assistive technologies.
        ui.put(
            pill.shrink2(Vec2::new(16., 8.)),
            egui::Label::new(
                egui::RichText::new(galley.text())
                    .size(15.)
                    .color(Color32::WHITE),
            )
            .selectable(false),
        );
    }
    pub(super) fn canvas(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        let journey_device = self.journey_event().map(|e| e.device_id.clone());
        ui.horizontal(|ui| {
            if ui
                .add(egui::Slider::new(&mut self.zoom, 0.25..=3.).text(language.text("zoom")))
                .changed()
            {
                self.preferences_ui.follow_canvas_size = false;
            }
            if ui.button(language.text("fit")).clicked() {
                self.fit = true;
                self.preferences_ui.follow_canvas_size = true;
            }
        });
        ui.label(language.text("canvas_hint"));
        let story_height = if self.mode == Mode::Simulation {
            (ui.available_height() * 0.3).clamp(120., 220.)
        } else {
            0.
        };
        let (rect, response) = ui.allocate_exact_size(
            (ui.available_size() - Vec2::new(0., story_height)).max(Vec2::splat(10.)),
            Sense::click_and_drag(),
        );
        // Native resize/zoom commands may take effect after the first fitted frame.
        if self.preferences_ui.follow_canvas_size
            && self
                .preferences_ui
                .last_canvas_size
                .is_some_and(|old| (old - rect.size()).length_sq() > 1.)
        {
            self.fit = true;
        }
        self.preferences_ui.last_canvas_size = Some(rect.size());
        let positions = self.project().layout.devices.clone();
        if self.fit && !self.project().annotations.is_empty() {
            let mut bounds = Rect::NOTHING;
            for position in &positions {
                bounds = bounds.union(Rect::from_center_size(
                    Pos2::new(position.x, position.y),
                    Vec2::new(120., 82.),
                ));
            }
            for note in &self.project().annotations {
                let style = note.effective_style();
                bounds = bounds.union(Rect::from_min_size(
                    Pos2::new(note.x, note.y),
                    Vec2::new(style.width, style.height),
                ));
            }
            self.zoom = ((rect.width() - 40.) / bounds.width())
                .min((rect.height() - 40.) / bounds.height())
                .clamp(0.25, 2.);
            self.pan = rect.size() / 2. - bounds.center().to_vec2() * self.zoom;
            self.fit = false;
        }
        if self.fit && !positions.is_empty() {
            let min = positions
                .iter()
                .fold(Pos2::new(f32::MAX, f32::MAX), |a, p| {
                    Pos2::new(a.x.min(p.x), a.y.min(p.y))
                });
            let max = positions
                .iter()
                .fold(Pos2::new(f32::MIN, f32::MIN), |a, p| {
                    Pos2::new(a.x.max(p.x), a.y.max(p.y))
                });
            self.zoom = ((rect.width() - 130.) / (max.x - min.x + 100.))
                .min((rect.height() - 100.) / (max.y - min.y + 80.))
                .clamp(0.25, 2.);
            self.pan = rect.size() / 2.
                - Vec2::new((min.x + max.x) / 2., (min.y + max.y) / 2.) * self.zoom;
            self.fit = false;
        }
        let zoom = self.zoom;
        let pan_offset = self.pan;
        let world_to_screen = |x: f32, y: f32| rect.min + pan_offset + Vec2::new(x, y) * zoom;
        let mut rendered = positions.clone();
        for p in &mut rendered {
            if let Some(v) = self.preview.iter().find(|v| v.device_id == p.device_id) {
                *p = v.clone();
            }
        }
        let point = |id: &Id| {
            rendered
                .iter()
                .find(|p| &p.device_id == id)
                .map(|p| world_to_screen(p.x, p.y))
        };
        let painter = ui.painter_at(rect);
        painter.rect_filled(
            rect,
            8.,
            if self.art.diagram {
                Color32::from_rgb(246, 249, 251)
            } else {
                Color32::from_rgb(20, 28, 38)
            },
        );
        self.paint_annotations(ui, rect);
        let annotation_background = self
            .project()
            .annotations
            .iter()
            .any(|a| a.style.as_ref().is_some_and(|s| s.fill_color[3] > 0));
        let mut segments = Vec::new();
        for link in &self.project().links {
            let a = self
                .project()
                .device_for_interface(&link.endpoints[0])
                .and_then(|d| point(&d.id));
            let b = self
                .project()
                .device_for_interface(&link.endpoints[1])
                .and_then(|d| point(&d.id));
            if let (Some(a), Some(b)) = (a, b) {
                if annotation_background {
                    let width = if self.selected.contains(&link.id)
                        || self.active_links.contains_key(&link.id)
                    {
                        6.
                    } else {
                        4.
                    };
                    painter.line_segment([a, b], Stroke::new(width, Color32::from_rgb(15, 30, 40)));
                }
                painter.line_segment(
                    [a, b],
                    Stroke::new(
                        if self.selected.contains(&link.id)
                            || self.active_links.contains_key(&link.id)
                        {
                            4.
                        } else {
                            2.
                        },
                        if self.mode == Mode::Simulation
                            && self
                                .worker
                                .view
                                .link_conditions
                                .get(&link.id)
                                .is_some_and(|c| c.loss_ppm > 0)
                        {
                            if self.worker.view.link_conditions[&link.id].loss_ppm == 1_000_000 {
                                Color32::from_rgb(210, 65, 65)
                            } else {
                                Color32::from_rgb(210, 140, 30)
                            }
                        } else if self.active_links.contains_key(&link.id) {
                            if self.art.diagram {
                                Color32::from_rgb(210, 100, 0)
                            } else {
                                Color32::LIGHT_YELLOW
                            }
                        } else {
                            if self.art.diagram {
                                Color32::from_rgb(40, 99, 140)
                            } else {
                                Color32::from_rgb(98, 184, 183)
                            }
                        },
                    ),
                );
                if self.mode == Mode::Simulation
                    && let Some(conditions) = self.worker.view.link_conditions.get(&link.id)
                {
                    painter.text(
                        a.lerp(b, 0.5) - Vec2::new(0., 6.),
                        egui::Align2::CENTER_BOTTOM,
                        format!("{} %", conditions.loss_ppm as f64 / 10_000.),
                        egui::FontId::proportional(12. * self.zoom),
                        if self.art.diagram {
                            Color32::from_rgb(40, 60, 80)
                        } else {
                            Color32::WHITE
                        },
                    );
                }
                segments.push((link.id.clone(), a, b));
            }
        }
        for (client, ap) in self.project().wireless_links() {
            let a = self
                .project()
                .device_for_interface(&client)
                .and_then(|d| point(&d.id));
            let b = self
                .project()
                .device_for_interface(&ap)
                .and_then(|d| point(&d.id));
            if let (Some(a), Some(b)) = (a, b) {
                for i in 0..12 {
                    let t = i as f32 / 12.;
                    let u = (i as f32 + 0.55) / 12.;
                    if annotation_background {
                        painter.line_segment(
                            [a + (b - a) * t, a + (b - a) * u],
                            Stroke::new(4., Color32::from_rgb(15, 30, 40)),
                        );
                    }
                    painter.line_segment(
                        [a + (b - a) * t, a + (b - a) * u],
                        Stroke::new(2., Color32::from_rgb(32, 150, 215)),
                    );
                }
            }
        }
        if let Some(start) = self
            .cable
            .as_ref()
            .and_then(|port| self.project().device_for_interface(port))
            .and_then(|d| point(&d.id))
            && let Some(pos) = ui.ctx().pointer_hover_pos()
        {
            painter.line_segment([start, pos], Stroke::new(2., Color32::LIGHT_YELLOW));
        }
        let mut open_host = None;
        let mut selected = None;
        let mut drag_start = None;
        let mut dragging = false;
        let mut stopped = false;
        for d in self.project().devices.clone() {
            let Some(pos) = point(&d.id) else {
                continue;
            };
            let node = Rect::from_center_size(pos, Vec2::new(120., 82.) * self.zoom);
            let r = ui.interact(
                node,
                ui.scope_id().with(d.id.as_str()),
                Sense::click_and_drag(),
            );
            r.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::Role::Button,
                    true,
                    self.selected.contains(&d.id),
                    language
                        .format("canvas_device", &[("label", &d.label)])
                        .unwrap(),
                )
            });
            if r.double_clicked()
                && self.mode == Mode::Simulation
                && !matches!(d.kind, DeviceKind::Switch | DeviceKind::Modem)
            {
                open_host = Some(d.id.clone());
            }
            let active = self.selected.contains(&d.id);
            painter.rect_filled(
                node,
                6.,
                if self.art.diagram {
                    if active {
                        Color32::from_rgb(201, 235, 245)
                    } else {
                        Color32::TRANSPARENT
                    }
                } else if active {
                    Color32::from_rgb(40, 92, 106)
                } else {
                    Color32::from_rgb(40, 54, 71)
                },
            );
            painter.rect_stroke(
                node,
                6.,
                Stroke::new(
                    if r.has_focus() { 3. } else { 1. },
                    if active {
                        Color32::WHITE
                    } else {
                        if self.art.diagram {
                            Color32::TRANSPARENT
                        } else {
                            Color32::GRAY
                        }
                    },
                ),
                egui::StrokeKind::Inside,
            );
            if journey_device.as_ref() == Some(&d.id) {
                painter.rect_stroke(
                    node.expand(4.),
                    7.,
                    Stroke::new(3., Color32::from_rgb(245, 160, 45)),
                    egui::StrokeKind::Outside,
                );
            }
            let symbol = self
                .project()
                .lan
                .get(&d.id)
                .and_then(|c| c.symbol)
                .unwrap_or(if d.network.ssid.is_some() {
                    DeviceSymbol::AccessPoint
                } else {
                    DeviceSymbol::automatic(d.kind)
                });
            let texture = self.art.texture(ui.ctx(), symbol);
            let picture = Rect::from_center_size(
                pos + Vec2::new(0., -10. * self.zoom),
                Vec2::splat(52. * self.zoom),
            );
            painter.image(
                texture,
                picture,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                Color32::WHITE,
            );
            let ink = if self.art.diagram {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            let mut caption = egui::text::LayoutJob::simple(
                self.device_label(&d),
                egui::FontId::proportional(12. * self.zoom),
                ink,
                110. * self.zoom,
            );
            caption.wrap.max_rows = 1;
            let caption = painter.layout_job(caption);
            let size = caption.size();
            let inside = self.art.diagram
                && matches!(symbol, DeviceSymbol::Router | DeviceSymbol::Switch)
                && size.x <= 40. * self.zoom;
            let origin = Pos2::new(
                pos.x - size.x / 2.,
                pos.y
                    + if inside {
                        -10. * self.zoom
                    } else {
                        27. * self.zoom
                    }
                    - size.y / 2.,
            );
            if self.art.diagram && !inside {
                painter.rect_filled(
                    Rect::from_min_size(origin, size).expand(2. * self.zoom),
                    2.,
                    Color32::from_rgb(246, 249, 251),
                );
            }
            painter.galley(origin, caption, ink);
            if r.clicked() || (r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                selected = Some(d.id.clone());
            }
            if self.mode == Mode::Design {
                if r.drag_started() {
                    drag_start = Some((
                        d.id.clone(),
                        ui.input(|i| i.pointer.press_origin()).unwrap_or(pos),
                    ));
                }
                dragging |= r.dragged();
                stopped |= r.drag_stopped();
            }
        }
        if let Some(id) = open_host {
            self.desktop.host = Some(id);
            self.page = Page::Desktop;
        }
        if let Some(id) = selected {
            if self.cabling && self.mode == Mode::Design {
                let free = self
                    .project()
                    .device(&id)
                    .and_then(|d| {
                        d.interfaces.iter().find(|i| {
                            !self
                                .project()
                                .links
                                .iter()
                                .any(|l| l.endpoints.contains(&i.id))
                        })
                    })
                    .map(|i| i.id.clone());
                if let Some(port) = free {
                    self.port_click(port);
                } else {
                    self.error = Some(language.text("no_free_port").into());
                }
            } else {
                self.select(id, ui.input(|i| i.modifiers.shift));
            }
        }
        if let Some((id, origin)) = drag_start {
            if !self.selected.contains(&id) {
                self.select(id, false);
            }
            self.drag = Some(Drag {
                origin,
                positions: positions
                    .iter()
                    .filter(|p| self.selected.contains(&p.device_id))
                    .cloned()
                    .collect(),
            });
        }
        if dragging
            && let Some(drag) = &self.drag
            && let Some(pointer) = ui.ctx().pointer_interact_pos()
        {
            let delta = (pointer - drag.origin) / self.zoom;
            self.preview = drag
                .positions
                .iter()
                .map(|p| Position {
                    device_id: p.device_id.clone(),
                    x: p.x + delta.x,
                    y: p.y + delta.y,
                })
                .collect();
        }
        if stopped && self.drag.take().is_some() {
            let next = std::mem::take(&mut self.preview);
            self.apply(Edit::Move(next));
        }
        let pan =
            ui.input(|i| i.modifiers.alt || i.pointer.button_down(egui::PointerButton::Middle));
        if pan && (response.dragged() || response.dragged_by(egui::PointerButton::Middle)) {
            self.preferences_ui.follow_canvas_size = false;
            self.pan += ui.input(|i| i.pointer.delta());
        }
        if !pan && self.mode == Mode::Design {
            if response.drag_started() {
                self.rectangle = ui.input(|i| i.pointer.press_origin());
            }
            if let (Some(start), Some(end)) = (self.rectangle, ui.ctx().pointer_interact_pos()) {
                let area = Rect::from_two_pos(start, end);
                painter.rect_stroke(
                    area,
                    0.,
                    Stroke::new(1., Color32::LIGHT_BLUE),
                    egui::StrokeKind::Inside,
                );
                if response.drag_stopped() {
                    self.selected = positions
                        .iter()
                        .filter(|p| area.contains(world_to_screen(p.x, p.y)))
                        .map(|p| p.device_id.clone())
                        .collect();
                    self.rectangle = None;
                    self.refresh_draft();
                }
            }
        }
        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
            && let Some((id, _, _)) = segments
                .iter()
                .find(|(_, a, b)| distance_segment(pos, *a, *b) < 8.)
        {
            self.selected = BTreeSet::from([id.clone()]);
            self.draft = None;
        }
        if self.mode == Mode::Design
            && let Some(kind) = response.dnd_release_payload::<DeviceKind>()
            && let Some(pos) = ui.ctx().pointer_interact_pos()
        {
            let world = (pos - rect.min - self.pan) / self.zoom;
            self.apply(Edit::AddNamed {
                name: format!(
                    "{} {}",
                    kind_label(language, *kind),
                    self.project().devices.len() + 1
                ),
                kind: *kind,
                x: world.x,
                y: world.y,
            });
        }
        self.paint_flights(ui, rect, point);
        self.cable_banner(ui, rect);
        if story_height > 0. {
            self.story_panel(ui, story_height);
        }
    }
}

fn distance_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let t = if ab.length_sq() == 0. {
        0.
    } else {
        ((p - a).dot(ab) / ab.length_sq()).clamp(0., 1.)
    };
    p.distance(a + ab * t)
}
