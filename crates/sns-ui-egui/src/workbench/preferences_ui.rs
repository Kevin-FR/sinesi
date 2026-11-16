//! User presentation state and bounded, coalescing preference writes.
use super::*;
use sns_settings::{
    Settings,
    display::{DisplayPreferences, Level, PanelLayout, Representation, UiFont, WindowGeometry},
};
use std::{sync::mpsc, time::Duration};

#[derive(Default)]
pub(super) struct PreferencesUi {
    pub display: DisplayPreferences,
    pub persistent: Settings,
    reset_requested: bool,
    window_gesture: Option<(egui::Id, Rect, bool)>,
    draft: DisplayPreferences,
    open: bool,
    pub pane: u8,
    writer: Option<mpsc::Receiver<Result<(), String>>>,
    queued: Option<Settings>,
    applied: Option<(u16, UiFont)>,
    resize_window: bool,
    pub last_canvas_size: Option<Vec2>,
    pub follow_canvas_size: bool,
}
impl Workbench {
    pub fn display_preferences(&self) -> &DisplayPreferences {
        &self.preferences_ui.display
    }
    pub fn set_display_preferences(&mut self, display: DisplayPreferences) -> Result<(), String> {
        display.validate().map_err(str::to_owned)?;
        self.fit = true;
        self.preferences_ui.follow_canvas_size = true;
        self.art.diagram = display.representation == Representation::Diagram;
        self.preferences_ui.resize_window = self.preferences_ui.display.window_width
            != display.window_width
            || self.preferences_ui.display.window_height != display.window_height;
        self.preferences_ui
            .persistent
            .display
            .copy_user_edits(&self.preferences_ui.display, &display);
        self.preferences_ui.display = display;
        self.queue_preferences();
        Ok(())
    }
    pub(super) fn load_preferences(&mut self, settings: Settings) {
        self.preferences_ui.persistent = settings.clone();
        self.language = settings.language;
        self.art.diagram = settings.display.representation == Representation::Diagram;
        self.preferences_ui.display = settings.display;
        self.preferences_ui.resize_window = true;
        self.preferences_ui.follow_canvas_size = true;
    }
    /// Presentation level; it never changes the project or a running simulation.
    pub fn level(&self) -> Level {
        self.preferences_ui.display.level
    }
    pub(super) fn expert(&self) -> bool {
        self.level() == Level::Expert
    }
    pub fn choose_level(&mut self, level: Level) {
        self.preferences_ui.display.level = level;
        self.preferences_ui.persistent.display.level = level;
        self.queue_preferences();
    }
    pub(super) fn choose_representation(&mut self, diagram: bool) {
        self.art.diagram = diagram;
        self.preferences_ui.display.representation = if diagram {
            Representation::Diagram
        } else {
            Representation::Icons
        };
        self.preferences_ui.persistent.display.representation =
            self.preferences_ui.display.representation;
        self.queue_preferences();
    }
    pub(super) fn queue_preferences(&mut self) {
        if !self.settings_writable || self.settings_path.is_none() {
            return;
        }
        self.preferences_ui.queued = Some(self.preferences_ui.persistent.clone());
    }
    pub fn preferences_busy(&self) -> bool {
        self.preferences_ui.writer.is_some() || self.preferences_ui.queued.is_some()
    }
    /// Wait for a requested preference write outside the render loop (tools/tests).
    pub fn wait_for_preferences(&mut self) -> Result<(), String> {
        while self.preferences_busy() {
            self.poll_preferences(&egui::Context::default());
            std::thread::sleep(Duration::from_millis(1));
        }
        self.settings_error.clone().map_or(Ok(()), Err)
    }
    pub(super) fn poll_preferences(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.pointer.any_pressed()) {
            self.preferences_ui.window_gesture = None;
        }
        if let Some(rx) = &self.preferences_ui.writer {
            match rx.try_recv() {
                Ok(result) => {
                    self.settings_error = result.err();
                    if self.settings_error.is_some() {
                        self.journal.record(TechnicalEvent::PreferencesWriteFailed);
                    }
                    self.preferences_ui.writer = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.journal.record(TechnicalEvent::PreferencesWriteFailed);
                    self.settings_error = Some("settings.write_failed".into());
                    self.preferences_ui.writer = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if self.preferences_ui.writer.is_none()
            && let Some(settings) = self.preferences_ui.queued.take()
            && let Some(path) = self.settings_path.clone()
        {
            let conflict = self
                .path
                .as_ref()
                .is_some_and(|p| sns_project::destination::same_destination(&path, p));
            let guard = if conflict {
                Err("settings.project_path_conflict".to_owned())
            } else if self.export_destination_matches(&path) {
                Err("render.path_in_use".to_owned())
            } else {
                self.preferences_output_allowed(&path)
            };
            if let Err(error) = guard {
                self.settings_error = Some(error);
                return;
            }
            let (tx, rx) = mpsc::sync_channel(1);
            match std::thread::Builder::new()
                .name("sinesi-preferences".into())
                .spawn(move || {
                    let _ = tx.send(sns_settings::save(&path, &settings));
                }) {
                Ok(_) => self.preferences_ui.writer = Some(rx),
                Err(error) => self.settings_error = Some(error.to_string()),
            }
        }
        if self.preferences_busy() {
            ctx.request_repaint_after(Duration::from_millis(20));
        }
    }
    pub(super) fn apply_display_style(&mut self, ctx: &egui::Context) {
        let prefs = &self.preferences_ui.display;
        let key = (prefs.scale_percent, prefs.font);
        if self.preferences_ui.applied != Some(key) {
            ctx.set_zoom_factor(prefs.scale_percent as f32 / 100.);
            ctx.all_styles_mut(|style| {
                for font in style.text_styles.values_mut() {
                    if font.family != egui::FontFamily::Monospace {
                        font.family = match prefs.font {
                            UiFont::Default => egui::FontFamily::Proportional,
                            UiFont::NotoSans => egui::FontFamily::Name("sns-ui-noto".into()),
                        };
                    }
                }
            });
            self.preferences_ui.applied = Some(key);
        }
        if self.preferences_ui.resize_window
            && (ctx.zoom_factor() - prefs.scale_percent as f32 / 100.).abs() < 0.001
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                Vec2::new(prefs.window_width as f32, prefs.window_height as f32)
                    / ctx.zoom_factor(),
            ));
            self.preferences_ui.resize_window = false;
        }
    }
    pub(super) fn toggle_preferences(&mut self) {
        self.preferences_ui.open = !self.preferences_ui.open;
        if self.preferences_ui.open {
            self.preferences_ui.draft = self.preferences_ui.display.clone();
            self.preferences_ui.reset_requested = false;
        }
    }
    pub(super) fn dismiss_preferences(&mut self) -> bool {
        let open = self.preferences_ui.open;
        self.preferences_ui.open = false;
        open
    }
    pub(super) fn preferences_window(&mut self, ctx: &egui::Context) {
        if !self.preferences_ui.open {
            return;
        }
        let l = self.language;
        let has_issues = self
            .startup_configuration()
            .is_some_and(|c| !c.diagnostics.is_empty());
        let mut reopen_configuration = false;
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        egui::Window::new(l.text("preferences"))
            .id(egui::Id::unique("display-preferences"))
            .default_pos([80., 80.])
            .default_size([540., 560.])
            .max_size((ctx.content_rect().size() - Vec2::splat(24.)).max(Vec2::splat(100.)))
            .open(&mut open)
            .vscroll(true)
            .hscroll(true)
            .show(ctx, |ui| {
                ui.label(l.text("preferences_hint"));
                if !self.settings_writable {
                    ui.colored_label(Color32::LIGHT_RED, l.text("configuration_readonly"));
                }
                if has_issues {
                    reopen_configuration = ui.button(l.text("configuration_title")).clicked();
                }
                let d = &mut self.preferences_ui.draft;
                ui.add(
                    egui::Slider::new(&mut d.scale_percent, 80..=200)
                        .text(l.text("preferences_scale"))
                        .suffix(" %"),
                );
                egui::ComboBox::new("preferences-font", l.text("preferences_font"))
                    .selected_text(match d.font {
                        UiFont::Default => l.text("preferences_font_default"),
                        UiFont::NotoSans => "Noto Sans",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut d.font,
                            UiFont::Default,
                            l.text("preferences_font_default"),
                        );
                        ui.selectable_value(&mut d.font, UiFont::NotoSans, "Noto Sans");
                    });
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(
                        &mut d.representation,
                        Representation::Icons,
                        l.text("view_icons"),
                    );
                    ui.selectable_value(
                        &mut d.representation,
                        Representation::Diagram,
                        l.text("view_diagram"),
                    );
                });
                egui::ComboBox::new("preferences-layout", l.text("preferences_layout"))
                    .selected_text(l.text(layout_key(d.layout)))
                    .show_ui(ui, |ui| {
                        for layout in [
                            PanelLayout::InspectorRight,
                            PanelLayout::InspectorLeft,
                            PanelLayout::Tabs,
                        ] {
                            ui.selectable_value(&mut d.layout, layout, l.text(layout_key(layout)));
                        }
                    });
                ui.add(
                    egui::Slider::new(&mut d.palette_width, 160..=360)
                        .text(l.text("preferences_palette_width")),
                );
                ui.add(
                    egui::Slider::new(&mut d.inspector_width, 240..=480)
                        .text(l.text("preferences_inspector_width")),
                );
                ui.separator();
                ui.add(
                    egui::Slider::new(&mut d.window_width, 840..=3840)
                        .text(l.text("preferences_window_width")),
                );
                ui.add(
                    egui::Slider::new(&mut d.window_height, 600..=2160)
                        .text(l.text("preferences_window_height")),
                );
                ui.add(
                    egui::Slider::new(&mut d.application_scale_percent, 75..=150)
                        .text(l.text("preferences_application_scale"))
                        .suffix(" %"),
                );
                ui.checkbox(
                    &mut d.center_application_windows,
                    l.text("preferences_center_windows"),
                );
                ui.label(l.text("preferences_geometry_hint"));
                ui.horizontal_wrapped(|ui| {
                    if ui.button(l.text("preferences_defaults")).clicked() {
                        *d = Default::default();
                        self.preferences_ui.reset_requested = true;
                    }
                    apply = ui.button(l.text("apply")).clicked();
                    cancel = ui.button(l.text("preferences_cancel")).clicked();
                });
            });
        if reopen_configuration {
            self.open_configuration_diagnostics();
        }
        if apply {
            if let Err(error) = self.set_display_preferences(self.preferences_ui.draft.clone()) {
                self.settings_error = Some(error);
            } else {
                if self.preferences_ui.reset_requested {
                    self.preferences_ui.persistent.display = Default::default();
                    self.queue_preferences();
                }
                open = false;
            }
        }
        self.preferences_ui.open = open && !cancel;
    }
    pub(super) fn topology_layout(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let d = &self.preferences_ui.display;
        let (palette, inspector, layout) =
            (d.palette_width as f32, d.inspector_width as f32, d.layout);
        if layout == PanelLayout::Tabs || size.x < palette + inspector + 320. {
            ui.horizontal_wrapped(|ui| {
                for (pane, key) in [
                    (0, "preferences_canvas"),
                    (1, "preferences_tools"),
                    (2, "preferences_inspector"),
                ] {
                    ui.selectable_value(
                        &mut self.preferences_ui.pane,
                        pane,
                        self.language.text(key),
                    );
                }
            });
            match self.preferences_ui.pane {
                1 => {
                    egui::ScrollArea::both()
                        .id_salt("compact-palette")
                        .show(ui, |ui| self.palette(ui));
                }
                2 => {
                    egui::ScrollArea::both()
                        .id_salt("compact-inspector")
                        .show(ui, |ui| self.inspector(ui));
                }
                _ => self.canvas(ui),
            }
            return;
        }
        let center = (size.x - palette - inspector - 40.).max(200.);
        ui.horizontal_top(|ui| {
            for pane in if layout == PanelLayout::InspectorLeft {
                [2, 0, 1]
            } else {
                [1, 0, 2]
            } {
                let width = match pane {
                    1 => palette,
                    2 => inspector,
                    _ => center,
                };
                ui.allocate_ui_with_layout(
                    Vec2::new(width, size.y),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| match pane {
                        1 => {
                            egui::ScrollArea::both()
                                .id_salt("palette-scroll")
                                .show(ui, |ui| self.palette(ui));
                        }
                        2 => {
                            egui::ScrollArea::both()
                                .id_salt("inspector-scroll")
                                .show(ui, |ui| self.inspector(ui));
                        }
                        _ => self.canvas(ui),
                    },
                );
                if pane
                    != if layout == PanelLayout::InspectorLeft {
                        1
                    } else {
                        2
                    }
                {
                    ui.separator();
                }
            }
        });
    }
    pub(super) fn configured_window<'a>(
        &self,
        ctx: &egui::Context,
        key: &str,
        window: egui::Window<'a>,
        size: Vec2,
    ) -> egui::Window<'a> {
        let d = &self.preferences_ui.display;
        let bounds = ctx.content_rect().shrink(8.);
        let zoom = ctx.zoom_factor();
        let saved = d.windows.get(key);
        let size = saved
            .map_or(size * d.application_scale_percent as f32 / 100., |r| {
                Vec2::new(r.width as f32, r.height as f32) / zoom
            })
            .min(bounds.size().max(Vec2::splat(100.)));
        let mut window = window
            .default_size(size)
            .max_size(bounds.size().max(Vec2::splat(100.)))
            .constrain_to(bounds);
        if let Some(r) = saved {
            let p = bounds.min + Vec2::new(r.x as f32, r.y as f32) / zoom;
            window = window.default_pos(p.clamp(bounds.min, (bounds.max - size).max(bounds.min)));
        } else if d.center_application_windows {
            window = window.default_pos(bounds.center() - size / 2.);
        }
        window
    }
    pub(super) fn remember_window(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        response: &egui::Response,
    ) {
        let rect = response.rect;
        let (pressed, released, down, position, delta) = ctx.input(|i| {
            (
                i.pointer.any_pressed(),
                i.pointer.any_released(),
                i.pointer.any_down(),
                i.pointer.interact_pos(),
                i.pointer.delta(),
            )
        });
        if pressed
            && let Some(position) = position
            && ctx.layer_id_at(position) == Some(response.layer_id)
            && rect.expand(6.).contains(position)
        {
            let title_height = ctx.style_of(ctx.theme()).spacing.interact_size.y + 8.;
            let on_title = position.y < rect.top() + title_height;
            let on_border = !rect.shrink(8.).contains(position);
            if on_title || on_border {
                self.preferences_ui.window_gesture = Some((response.id, rect, false));
            }
        }
        let mut finish = false;
        let mut commit = false;
        if let Some((id, start, moved)) = &mut self.preferences_ui.window_gesture
            && *id == response.id
        {
            if (down || released)
                && delta.length_sq() > 0.01
                && (start.min.distance_sq(rect.min) > 1.
                    || (start.size() - rect.size()).length_sq() > 1.)
            {
                *moved = true;
            }
            if released {
                finish = true;
                commit = *moved;
            }
        }
        if finish {
            self.preferences_ui.window_gesture = None;
        }
        if !commit {
            return;
        }
        let zoom = ctx.zoom_factor();
        let p = (rect.min - ctx.content_rect().shrink(8.).min) * zoom;
        let geometry = WindowGeometry {
            x: p.x.round() as i32,
            y: p.y.round() as i32,
            width: (rect.width() * zoom).round() as u16,
            height: (rect.height() * zoom).round() as u16,
        };
        if self.preferences_ui.display.windows.get(key) == Some(&geometry) {
            return;
        }
        let mut next = self.preferences_ui.display.clone();
        next.windows.insert(key.to_owned(), geometry);
        if next.validate().is_ok() {
            self.preferences_ui
                .persistent
                .display
                .copy_user_edits(&self.preferences_ui.display, &next);
            self.preferences_ui.display = next;
            self.queue_preferences();
        }
    }
}
fn layout_key(layout: PanelLayout) -> &'static str {
    match layout {
        PanelLayout::InspectorRight => "preferences_right",
        PanelLayout::InspectorLeft => "preferences_left",
        PanelLayout::Tabs => "preferences_tabs",
    }
}
