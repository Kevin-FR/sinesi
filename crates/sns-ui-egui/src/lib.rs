//! Native J0 workbench. Rendering reads snapshots; only the worker owns the engine.
mod controller;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use controller::{Controller, Job};
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use egui_extras::{Column, TableBuilder};
use sns_html::{Block, Document};
use sns_model::{HttpMethod, Id};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Topology,
    Trace,
    Browser,
}

pub struct SinesiUi {
    controller: Controller,
    pub tab: Tab,
    selected: Option<Id>,
    zoom: f32,
    pan: Vec2,
    filter: String,
    document: Option<Document>,
    fields: BTreeMap<String, String>,
    images: BTreeMap<String, egui::TextureHandle>,
    requested_images: BTreeSet<String>,
    page_path: String,
    browser_status: String,
    last_post: Option<String>,
    command_error: Option<String>,
}

impl SinesiUi {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            controller: Controller::new()?,
            tab: Tab::Topology,
            selected: None,
            zoom: 1.0,
            pan: Vec2::ZERO,
            filter: String::new(),
            document: None,
            fields: BTreeMap::new(),
            images: BTreeMap::new(),
            requested_images: BTreeSet::new(),
            page_path: "/".into(),
            browser_status: "Ouvrir la page pour lancer la requête simulée.".into(),
            last_post: None,
            command_error: None,
        })
    }
    pub fn trace_len(&self) -> usize {
        self.controller.view.trace.len()
    }
    pub fn now(&self) -> u64 {
        self.controller.view.now
    }
    pub fn image_count(&self) -> usize {
        self.images.len()
    }
    pub fn last_post(&self) -> Option<&str> {
        self.last_post.as_deref()
    }
    pub fn selected(&self) -> Option<&Id> {
        self.selected.as_ref()
    }
    pub fn wait_for_worker(&mut self) -> Result<(), String> {
        self.controller.wait()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.controller.poll();
        self.consume_http(ui.ctx());
        if self.controller.in_flight > 0 {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(20));
        }
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.horizontal(|ui| {
                ui.heading("Sinesi");
                ui.label(
                    egui::RichText::new("Simple network simulator")
                        .color(Color32::from_rgb(112, 190, 180)),
                );
                ui.label("• Atelier J0");
            });
            ui.label(
                "Prototype de fondation — les échanges ci-dessous ne simulent pas encore TCP/IP.",
            );
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                let enabled = self.controller.in_flight == 0;
                if ui
                    .add_enabled(enabled, egui::Button::new("Pas à pas"))
                    .clicked()
                {
                    self.controller.submit(Job::Step, Some(ui.ctx().clone()));
                }
                if ui
                    .add_enabled(enabled, egui::Button::new("Exécuter"))
                    .clicked()
                {
                    self.controller.submit(Job::Run, Some(ui.ctx().clone()));
                }
                if ui
                    .add_enabled(enabled, egui::Button::new("Réinitialiser"))
                    .clicked()
                {
                    self.clear_browser();
                    self.controller.submit(Job::Reset, Some(ui.ctx().clone()));
                }
                if ui
                    .add_enabled(enabled, egui::Button::new("Charger 10 000 événements"))
                    .clicked()
                {
                    self.clear_browser();
                    self.tab = Tab::Trace;
                    self.controller.submit(Job::Stress, Some(ui.ctx().clone()));
                }
                ui.separator();
                ui.label(format!("Temps virtuel : {} µs", self.now()));
                ui.label(format!(
                    "{} en attente · {} faits",
                    self.controller.view.pending,
                    self.trace_len()
                ));
                if !enabled {
                    ui.spinner();
                }
            });
            if let Some(error) = self
                .controller
                .view
                .error
                .as_ref()
                .or(self.command_error.as_ref())
            {
                ui.colored_label(Color32::LIGHT_RED, error);
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Topology, "Topologie");
                ui.selectable_value(&mut self.tab, Tab::Trace, "Échanges");
                ui.selectable_value(&mut self.tab, Tab::Browser, "Navigateur simulé");
            });
            ui.add_space(8.0);
            match self.tab {
                Tab::Topology => self.topology(ui),
                Tab::Trace => self.trace(ui),
                Tab::Browser => self.browser(ui),
            }
        });
    }
    fn clear_browser(&mut self) {
        self.document = None;
        self.images.clear();
        self.fields.clear();
        self.requested_images.clear();
        self.last_post = None;
        self.browser_status = "Ouvrir la page pour lancer la requête simulée.".into();
    }
    fn topology(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let label=ui.label("Zoom");ui.add(egui::Slider::new(&mut self.zoom,0.4..=2.5).text("Échelle du canevas")).labelled_by(label.id);
            if ui.button("Cadrer le réseau").clicked(){self.pan=Vec2::ZERO;self.zoom=1.0;}
            ui.label("Glisser le fond pour naviguer; sélectionner un équipement par bouton ou sur le canevas.");
        });
        ui.horizontal(|ui| {
            for device in &self.controller.view.project.devices {
                if ui
                    .selectable_label(self.selected.as_ref() == Some(&device.id), &device.label)
                    .clicked()
                {
                    self.selected = Some(device.id.clone());
                }
            }
        });
        let size = Vec2::new(
            ui.available_width(),
            (ui.available_height() - 90.0).max(160.0),
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::drag());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Image, true, "Canevas du réseau")
        });
        if response.dragged() {
            self.pan += response.drag_delta();
        }
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 10.0, Color32::from_rgb(15, 25, 36));
        let offset = rect.min.to_vec2() + Vec2::new(20.0, 5.0) + self.pan;
        let positions: BTreeMap<_, _> = self
            .controller
            .view
            .project
            .layout
            .devices
            .iter()
            .map(|p| {
                (
                    p.device_id.clone(),
                    Pos2::new(p.x * self.zoom, p.y * self.zoom) + offset,
                )
            })
            .collect();
        for link in &self.controller.view.project.links {
            let project = &self.controller.view.project;
            if let (Some(a), Some(b)) = (
                project.device_for_interface(&link.endpoints[0]),
                project.device_for_interface(&link.endpoints[1]),
            ) && let (Some(start), Some(end)) = (positions.get(&a.id), positions.get(&b.id))
            {
                painter.line_segment(
                    [*start, *end],
                    Stroke::new(3.0, Color32::from_rgb(50, 170, 150)),
                );
                painter.text(
                    start.lerp(*end, 0.5) + Vec2::new(0.0, -20.0),
                    egui::Align2::CENTER_CENTER,
                    "Liaison virtuelle",
                    egui::FontId::proportional(12.0),
                    Color32::GRAY,
                );
            }
        }
        for device in &self.controller.view.project.devices {
            if let Some(center) = positions.get(&device.id) {
                let node = Rect::from_center_size(*center, Vec2::new(160.0, 82.0) * self.zoom);
                let click =
                    ui.interact(node, ui.scope_id().with(device.id.as_str()), Sense::click());
                click.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::Button,
                        true,
                        format!("Sélectionner {}", device.label),
                    )
                });
                if click.clicked() {
                    self.selected = Some(device.id.clone());
                }
                let selected = self.selected.as_ref() == Some(&device.id);
                painter.rect_filled(
                    node,
                    10.0,
                    if selected {
                        Color32::from_rgb(28, 98, 90)
                    } else {
                        Color32::from_rgb(34, 49, 66)
                    },
                );
                painter.text(
                    *center,
                    egui::Align2::CENTER_CENTER,
                    &device.label,
                    egui::FontId::proportional(16.0 * self.zoom),
                    Color32::WHITE,
                );
                painter.text(
                    *center + Vec2::new(0.0, 26.0 * self.zoom),
                    egui::Align2::CENTER_CENTER,
                    device.id.as_str(),
                    egui::FontId::monospace(11.0 * self.zoom),
                    Color32::LIGHT_GRAY,
                );
            }
        }
        if let Some(device) = self
            .selected
            .as_ref()
            .and_then(|id| self.controller.view.project.device(id))
        {
            ui.label(format!(
                "Équipement sélectionné : {} · {} interface(s)",
                device.label,
                device.interfaces.len()
            ));
            for interface in &device.interfaces {
                ui.monospace(format!("{}  MAC {}", interface.id.as_str(), interface.mac));
            }
        }
    }
    fn trace(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let label = ui.label("Filtrer les échanges");
            ui.text_edit_singleline(&mut self.filter)
                .labelled_by(label.id);
            if ui.button("Effacer le filtre").clicked() {
                self.filter.clear();
            }
        });
        let rows: Vec<_> = self
            .controller
            .view
            .trace
            .iter()
            .filter(|event| {
                self.filter.is_empty()
                    || event.kind.contains(&self.filter)
                    || event.device_id.as_str().contains(&self.filter)
            })
            .collect();
        ui.label(format!(
            "{} lignes affichées — colonnes redimensionnables",
            rows.len()
        ));
        ui.monospace(format!("SHA-256 : {}", self.controller.view.trace_hash));
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::initial(80.0))
            .column(Column::initial(100.0))
            .column(Column::initial(300.0))
            .column(Column::remainder())
            .header(24.0, |mut header| {
                for title in ["N°", "Temps (µs)", "Événement", "Équipement"] {
                    header.col(|ui| {
                        ui.strong(title);
                    });
                }
            })
            .body(|body| {
                body.rows(24.0, rows.len(), |mut row| {
                    let event = rows[row.index()];
                    row.col(|ui| {
                        ui.monospace(event.sequence.0.to_string());
                    });
                    row.col(|ui| {
                        ui.monospace(event.time_us.0.to_string());
                    });
                    row.col(|ui| {
                        ui.label(&event.kind);
                    });
                    row.col(|ui| {
                        ui.label(event.device_id.as_str());
                    });
                })
            });
    }
    fn request(
        &mut self,
        method: HttpMethod,
        path: String,
        fields: BTreeMap<String, String>,
        ctx: &egui::Context,
    ) {
        if !sns_model::virtual_path_valid(&path) {
            self.command_error = Some("Cette adresse sort du réseau simulé.".into());
            return;
        }
        self.browser_status = format!("Requête simulée vers {path}…");
        self.controller.submit(
            Job::Http {
                method,
                path,
                fields,
            },
            Some(ctx.clone()),
        );
    }
    fn consume_http(&mut self, ctx: &egui::Context) {
        for result in std::mem::take(&mut self.controller.view.results) {
            self.browser_status = format!("Réponse {} · {}", result.status, result.path);
            let decoded = STANDARD.decode(&result.body_base64);
            let Ok(bytes) = decoded else {
                self.command_error = Some("Réponse binaire invalide.".into());
                continue;
            };
            if bytes.len() > sns_model::MAX_BODY_BYTES {
                self.command_error = Some("Ressource trop volumineuse.".into());
                continue;
            }
            match result.media_type.as_str() {
                "text/html" => {
                    match std::str::from_utf8(&bytes)
                        .map_err(|e| e.to_string())
                        .and_then(|html| sns_html::parse(html).map_err(|e| e.to_string()))
                    {
                        Ok(document) => {
                            self.fields.clear();
                            self.last_post = None;
                            for block in &document.blocks {
                                match block {
                                    Block::Form { action, fields, .. } => {
                                        for field in fields {
                                            self.fields.insert(
                                                format!("{action}:{}", field.name),
                                                field.value.clone(),
                                            );
                                        }
                                    }
                                    Block::Image { source, .. }
                                        if self.requested_images.insert(source.clone()) =>
                                    {
                                        self.request(
                                            HttpMethod::Get,
                                            source.clone(),
                                            BTreeMap::new(),
                                            ctx,
                                        )
                                    }
                                    _ => {}
                                }
                            }
                            self.page_path = result.path;
                            self.document = Some(document);
                        }
                        Err(error) => self.command_error = Some(error),
                    }
                }
                "image/png" => {
                    let reader = image::ImageReader::with_format(
                        std::io::Cursor::new(&bytes),
                        image::ImageFormat::Png,
                    );
                    let mut reader = reader;
                    let mut limits = image::Limits::default();
                    limits.max_image_width = Some(1024);
                    limits.max_image_height = Some(1024);
                    limits.max_alloc = Some(8 * 1024 * 1024);
                    reader.limits(limits);
                    match reader.decode() {
                        Ok(image) => {
                            let rgba = image.to_rgba8();
                            let size = [rgba.width() as usize, rgba.height() as usize];
                            let pixels =
                                egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                            self.images.insert(
                                result.path,
                                ctx.load_texture(
                                    "prototype-server-image",
                                    pixels,
                                    egui::TextureOptions::LINEAR,
                                ),
                            );
                        }
                        Err(error) => self.command_error = Some(format!("Image refusée : {error}")),
                    }
                }
                "text/plain" => self.last_post = Some(String::from_utf8_lossy(&bytes).into_owned()),
                _ => {
                    self.command_error =
                        Some("Type de contenu non pris en charge dans le prototype.".into())
                }
            }
        }
    }
    fn browser(&mut self, ui: &mut egui::Ui) {
        ui.label(
            "HTTP applicatif de démonstration, transporté par les événements du prototype J0.",
        );
        ui.horizontal(|ui| {
            let label = ui.label("Chemin du serveur simulé");
            ui.text_edit_singleline(&mut self.page_path)
                .labelled_by(label.id);
            if ui.button("Ouvrir la page").clicked() {
                self.request(
                    HttpMethod::Get,
                    self.page_path.clone(),
                    BTreeMap::new(),
                    ui.ctx(),
                );
            }
        });
        ui.label(&self.browser_status);
        ui.separator();
        if let Some(document) = self.document.clone() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, block) in document.blocks.iter().enumerate() {
                    ui.push_id(index, |ui| match block {
                        Block::Text { text, heading } => {
                            if *heading {
                                ui.heading(text);
                            } else {
                                ui.label(text);
                            }
                        }
                        Block::Link { text, target } => {
                            if ui.button(text).clicked() {
                                self.request(
                                    HttpMethod::Get,
                                    target.clone(),
                                    BTreeMap::new(),
                                    ui.ctx(),
                                );
                            }
                        }
                        Block::Image { alt, source } => {
                            if let Some(texture) = self.images.get(source) {
                                ui.add(
                                    egui::Image::new(texture)
                                        .fit_to_exact_size(Vec2::splat(72.0))
                                        .alt_text(alt),
                                );
                            } else {
                                ui.label(format!("Chargement de l’image : {alt}"));
                            }
                        }
                        Block::Form {
                            action,
                            fields,
                            submit,
                        } => {
                            for field in fields {
                                ui.horizontal(|ui| {
                                    let label = ui.label(&field.label);
                                    ui.text_edit_singleline(
                                        self.fields
                                            .entry(format!("{action}:{}", field.name))
                                            .or_default(),
                                    )
                                    .labelled_by(label.id);
                                });
                            }
                            if ui.button(submit).clicked() {
                                let values = fields
                                    .iter()
                                    .map(|field| {
                                        (
                                            field.name.clone(),
                                            self.fields
                                                .get(&format!("{action}:{}", field.name))
                                                .cloned()
                                                .unwrap_or_default(),
                                        )
                                    })
                                    .collect();
                                self.request(HttpMethod::Post, action.clone(), values, ui.ctx());
                            }
                        }
                    });
                }
                if let Some(result) = &self.last_post {
                    ui.separator();
                    ui.strong("Réponse du formulaire");
                    ui.monospace(result);
                }
            });
        }
    }
}
