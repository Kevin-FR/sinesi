//! Documentation controls and canvas interactions; all commits use editor transactions.
use super::*;
#[derive(Default)]
pub(super) struct AnnotationUi {
    pub draft: Option<Annotation>,
    drag: Option<(Annotation, Pos2, bool)>,
    preview: Option<Annotation>,
}
impl AnnotationUi {
    pub(super) fn cancel_gesture(&mut self) {
        self.drag = None;
        self.preview = None;
    }
}
fn color(value: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(value[0], value[1], value[2], value[3])
}
pub(super) fn install_fonts(ctx: &egui::Context) {
    let key = egui::Id::unique("sns-document-fonts-v1");
    if ctx.data(|d| d.get_temp::<bool>(key)) == Some(true) {
        return;
    }
    // A valid imported Unicode string must remain editable even if our bundled
    // fonts cannot draw it. Document exports still reject unsupported glyphs.
    ctx.set_missing_glyph_policy(egui::MissingGlyphPolicy::Tofu);
    for (name, bytes) in sns_fonts::FONTS {
        ctx.add_font(egui::FontInsert::new(
            name,
            egui::FontData::from_static(bytes),
            vec![egui::InsertFontFamily {
                family: egui::FontFamily::Name(name.into()),
                priority: egui::FontPriority::Highest,
            }],
        ));
    }
    let defaults = egui::FontDefinitions::default();
    let fallback = defaults
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let family = egui::FontFamily::Name("sns-ui-noto".into());
    for name in fallback.into_iter().rev() {
        if let Some(data) = defaults.font_data.get(&name) {
            ctx.add_font(egui::FontInsert::new(
                &name,
                (**data).clone(),
                vec![egui::InsertFontFamily {
                    family: family.clone(),
                    priority: egui::FontPriority::Highest,
                }],
            ));
        }
    }
    ctx.add_font(egui::FontInsert::new(
        "sns-ui-noto-regular",
        egui::FontData::from_static(sns_fonts::SANS),
        vec![egui::InsertFontFamily {
            family,
            priority: egui::FontPriority::Highest,
        }],
    ));
    ctx.data_mut(|d| d.insert_temp(key, true));
}
impl Workbench {
    pub(super) fn annotation_palette(&mut self, ui: &mut egui::Ui) {
        let l = self.language;
        ui.heading(l.text("annotation_tools"));
        let label = ui.label(l.text("annotation_text"));
        ui.add(
            egui::TextEdit::multiline(&mut self.annotation)
                .desired_rows(3)
                .desired_width(ui.available_width())
                .char_limit(65_536),
        )
        .labelled_by(label.id);
        for (kind, key) in [
            (AnnotationKind::Text, "add_annotation"),
            (AnnotationKind::Rectangle, "add_annotation_rectangle"),
        ] {
            if ui.button(l.text(key)).clicked() {
                self.apply(Edit::AddAnnotation {
                    text: self.annotation.clone(),
                    x: 80.,
                    y: 400.,
                    kind,
                });
                self.refresh_draft();
            }
        }
        let notes = self.project().annotations.clone();
        for note in notes {
            let title = note
                .text
                .lines()
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or(note.id.as_str());
            if ui
                .selectable_label(self.selected.contains(&note.id), title)
                .clicked()
            {
                self.select(note.id, false);
            }
        }
        ui.label(l.text("annotation_mode_hint"));
    }
    pub(super) fn annotation_inspector(&mut self, ui: &mut egui::Ui) -> bool {
        let Some(id) = self
            .selected
            .iter()
            .find(|id| self.project().annotations.iter().any(|a| &a.id == *id))
            .cloned()
        else {
            return false;
        };
        let Some(original) = self
            .project()
            .annotations
            .iter()
            .find(|a| a.id == id)
            .cloned()
        else {
            return false;
        };
        let l = self.language;
        let mut draft = self
            .annotations
            .draft
            .take()
            .filter(|d| d.id == id)
            .unwrap_or(original.clone());
        ui.label(id.as_str());
        ui.add_enabled_ui(
            self.mode == Mode::Documentation && !self.worker.pending_commands(),
            |ui| {
                let (mut apply, mut revert, mut delete) = (false, false, false);
                ui.horizontal_wrapped(|ui| {
                    apply = ui.button(l.text("annotation_apply")).clicked();
                    revert = ui.button(l.text("annotation_revert")).clicked();
                    delete = ui.button(l.text("annotation_delete")).clicked();
                });
                let label = ui.label(l.text("annotation_text"));
                ui.add(
                    egui::TextEdit::multiline(&mut draft.text)
                        .desired_rows(5)
                        .desired_width(ui.available_width())
                        .char_limit(65_536),
                )
                .labelled_by(label.id);
                let effective = draft.effective_style();
                let style = draft.style.get_or_insert(effective);
                for (key, value, range) in [
                    ("annotation_x", &mut draft.x, -1_000_000.0..=1_000_000.),
                    ("annotation_y", &mut draft.y, -1_000_000.0..=1_000_000.),
                    ("annotation_width", &mut style.width, 24.0..=10_000.),
                    ("annotation_height", &mut style.height, 24.0..=10_000.),
                    ("annotation_font_size", &mut style.font_size, 8.0..=96.),
                    (
                        "annotation_border_width",
                        &mut style.border_width,
                        0.0..=20.,
                    ),
                ] {
                    ui.horizontal(|ui| {
                        let label = ui.label(l.text(key));
                        ui.add(egui::DragValue::new(value).range(range).speed(1.))
                            .labelled_by(label.id);
                    });
                }
                egui::ComboBox::new("annotation-kind", l.text("annotation_kind"))
                    .selected_text(l.text(if style.kind == AnnotationKind::Rectangle {
                        "annotation_rectangle"
                    } else {
                        "annotation_note"
                    }))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut style.kind,
                            AnnotationKind::Text,
                            l.text("annotation_note"),
                        );
                        ui.selectable_value(
                            &mut style.kind,
                            AnnotationKind::Rectangle,
                            l.text("annotation_rectangle"),
                        );
                    });
                ui.checkbox(&mut style.italic, l.text("annotation_italic"));
                ui.checkbox(&mut style.bold, l.text("annotation_bold"));
                ui.checkbox(&mut style.underline, l.text("annotation_underline"));
                let mut mono = style.font == AnnotationFont::Monospace;
                ui.checkbox(&mut mono, l.text("annotation_monospace"));
                style.font = if mono {
                    AnnotationFont::Monospace
                } else {
                    AnnotationFont::Proportional
                };
                for (key, value) in [
                    ("annotation_text_color", &mut style.text_color),
                    ("annotation_fill_color", &mut style.fill_color),
                    ("annotation_border_color", &mut style.border_color),
                ] {
                    ui.horizontal(|ui| {
                        let label = ui.label(l.text(key));
                        ui.color_edit_button_srgba_unmultiplied(value)
                            .labelled_by(label.id);
                    });
                }
                ui.label(l.text("annotation_clip_hint"));
                if apply {
                    self.apply(Edit::Annotate(draft.clone()));
                }
                if revert {
                    draft = original;
                }
                if delete {
                    self.apply(Edit::Delete(BTreeSet::from([id.clone()])));
                    self.selected.remove(&id);
                }
            },
        );
        self.annotations.draft = self
            .project()
            .annotations
            .iter()
            .any(|a| a.id == draft.id)
            .then_some(draft);
        true
    }
    pub(super) fn paint_annotations(&mut self, ui: &mut egui::Ui, canvas: Rect) {
        let l = self.language;
        let pan = self.pan;
        let zoom = self.zoom;
        let editing = self.mode == Mode::Documentation
            && !self.worker.pending_commands()
            && !ui.input(|i| i.modifiers.alt || i.pointer.button_down(egui::PointerButton::Middle));
        if !editing {
            self.annotations.cancel_gesture();
        }
        let mut selected = None;
        let mut commit = None;
        for stored in self.project().annotations.clone() {
            let note = self
                .annotations
                .preview
                .as_ref()
                .filter(|a| a.id == stored.id)
                .unwrap_or(&stored);
            let style = note.effective_style();
            let area = Rect::from_min_size(
                canvas.min + pan + Vec2::new(note.x, note.y) * zoom,
                Vec2::new(style.width, style.height) * zoom,
            );
            let painter = ui.painter_at(canvas.intersect(area));
            if note.style.is_some() {
                painter.rect_filled(area, 0., color(style.fill_color));
                painter.rect_stroke(
                    area,
                    0.,
                    Stroke::new(style.border_width * zoom, color(style.border_color)),
                    egui::StrokeKind::Inside,
                );
            }
            let text_rect = if note.style.is_some() {
                area.shrink(6. * zoom)
            } else {
                area
            };
            let ink = if note.style.is_some() {
                color(style.text_color)
            } else if self.art.diagram {
                Color32::DARK_BLUE
            } else {
                Color32::LIGHT_YELLOW
            };
            let mut job = egui::text::LayoutJob::single_section(
                note.text.clone(),
                egui::TextFormat {
                    font_id: egui::FontId::new(
                        style.font_size * zoom,
                        if note.style.is_none() {
                            egui::FontFamily::Proportional
                        } else {
                            egui::FontFamily::Name(
                                match (style.font, style.bold) {
                                    (AnnotationFont::Monospace, true) => "sns-annotation-mono-bold",
                                    (AnnotationFont::Monospace, false) => "sns-annotation-mono",
                                    (AnnotationFont::Proportional, true) => {
                                        "sns-annotation-sans-bold"
                                    }
                                    (AnnotationFont::Proportional, false) => "sns-annotation-sans",
                                }
                                .into(),
                            )
                        },
                    ),
                    color: ink,
                    italics: style.italic,
                    underline: if style.underline {
                        Stroke::new(zoom, ink)
                    } else {
                        Stroke::NONE
                    },
                    ..Default::default()
                },
            );
            job.wrap.max_width = text_rect.width().max(1.);
            job.wrap.max_rows = 1024;
            painter.with_clip_rect(canvas.intersect(area)).galley(
                text_rect.min,
                painter.layout_job(job),
                ink,
            );
            if !editing {
                continue;
            }
            let active = self.selected.contains(&note.id);
            let r = ui.interact(
                area.intersect(canvas),
                ui.scope_id().with(("annotation", note.id.as_str())),
                Sense::click_and_drag(),
            );
            r.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::Role::Button,
                    true,
                    active,
                    l.format("annotation_canvas", &[("id", note.id.as_str())])
                        .unwrap(),
                )
            });
            if r.clicked() || (r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                selected = Some(note.id.clone());
            }
            let handle = Rect::from_min_max(area.max - Vec2::splat(12.), area.max);
            if active {
                painter.rect_stroke(
                    area,
                    0.,
                    Stroke::new(2., Color32::from_rgb(240, 150, 20)),
                    egui::StrokeKind::Inside,
                );
            }
            let resize = if active && handle.intersects(canvas) {
                painter.rect_filled(handle, 0., Color32::from_rgb(240, 150, 20));
                let response = ui
                    .interact(
                        handle.intersect(canvas),
                        ui.scope_id().with(("annotation-resize", note.id.as_str())),
                        Sense::click_and_drag(),
                    )
                    .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::Button,
                        true,
                        l.format("annotation_resize", &[("id", note.id.as_str())])
                            .unwrap(),
                    )
                });
                Some(response)
            } else {
                None
            };
            if r.drag_started() || resize.as_ref().is_some_and(|r| r.drag_started()) {
                self.annotations.drag = Some((
                    stored.clone(),
                    ui.input(|i| i.pointer.press_origin()).unwrap_or(area.min),
                    resize.as_ref().is_some_and(|r| r.drag_started()),
                ));
                selected = Some(stored.id.clone());
            }
            if (r.dragged() || resize.as_ref().is_some_and(|r| r.dragged()))
                && let Some((start, origin, resizing)) = &self.annotations.drag
                && start.id == stored.id
                && let Some(pointer) = ui.ctx().pointer_interact_pos()
            {
                let delta = (pointer - *origin) / zoom;
                let mut next = start.clone();
                if *resizing {
                    let effective = next.effective_style();
                    let style = next.style.get_or_insert(effective);
                    style.width = (style.width + delta.x).clamp(24., 10_000.);
                    style.height = (style.height + delta.y).clamp(24., 10_000.);
                } else {
                    next.x = (start.x + delta.x).clamp(-1_000_000., 1_000_000.);
                    next.y = (start.y + delta.y).clamp(-1_000_000., 1_000_000.);
                }
                self.annotations.preview = Some(next);
            }
            if (r.drag_stopped() || resize.as_ref().is_some_and(|r| r.drag_stopped()))
                && self
                    .annotations
                    .drag
                    .as_ref()
                    .is_some_and(|(start, _, _)| start.id == stored.id)
            {
                self.annotations.drag = None;
                commit = self.annotations.preview.take();
            }
        }
        if let Some(id) = selected {
            self.select(id, false);
        }
        if let Some(note) = commit {
            self.apply(Edit::Annotate(note));
            self.refresh_draft();
        }
    }
}
