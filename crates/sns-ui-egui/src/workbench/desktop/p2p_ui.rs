//! P2P projections; all networking and file writes are engine commands.
use super::*;
#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct P2pDraft {
    config: Option<P2pConfig>,
    peer: String,
    port: u16,
    query: String,
    ttl: u8,
    selected: Option<Id>,
}
impl Workbench {
    pub(super) fn p2p_ui(
        &mut self,
        ui: &mut egui::Ui,
        host: &Id,
        app: &AppView,
        draft: &mut Draft,
    ) {
        let l = self.language;
        let metadata = self
            .project()
            .device(host)
            .and_then(|d| d.applications.iter().find(|a| a.id == app.id))
            .cloned();
        let Some(metadata) = metadata else { return };
        let d = &mut draft.p2p;
        let cfg = d.config.get_or_insert_with(|| match metadata.service {
            Some(ServiceConfig::P2p(c)) => c,
            _ => Default::default(),
        });
        ui.label(l.text("p2p_simulated_hint"));
        egui::CollapsingHeader::new(l.text("p2p_settings"))
            .default_open(!app.running)
            .show(ui, |ui| {
                ui.add_enabled_ui(!app.running, |ui| {
                    let label = ui.label(l.text("port"));
                    ui.add(egui::DragValue::new(&mut cfg.port).range(1..=65535))
                        .labelled_by(label.id);
                    let label = ui.label(l.text("p2p_max_neighbors"));
                    ui.add(egui::DragValue::new(&mut cfg.max_neighbors).range(1..=32))
                        .labelled_by(label.id);
                    field(
                        ui,
                        l.text("p2p_shared_directory"),
                        &mut cfg.shared_directory,
                    );
                    if ui.button(l.text("apply")).clicked() {
                        self.app_command(
                            host,
                            AppCommand::ConfigureService {
                                application: app.id.clone(),
                                config: ServiceConfig::P2p(cfg.clone()),
                            },
                        );
                    }
                });
            });
        if d.port == 0 {
            d.port = 6346;
        }
        if d.ttl == 0 {
            d.ttl = 4;
        }
        field(ui, l.text("p2p_bootstrap"), &mut d.peer);
        let label = ui.label(l.text("p2p_peer_port"));
        ui.add(egui::DragValue::new(&mut d.port).range(1..=65535))
            .labelled_by(label.id);
        let mut command = None;
        ui.horizontal(|ui| {
            if ui.button(l.text("p2p_join")).clicked() {
                if d.peer.trim().is_empty() {
                    command = Some(P2pCommand::Join { peer: None });
                } else {
                    match d.peer.trim().parse() {
                        Ok(address) => {
                            command = Some(P2pCommand::Join {
                                peer: Some(Endpoint {
                                    address,
                                    port: d.port,
                                }),
                            })
                        }
                        Err(_) => self.error = Some(l.text("p2p.invalid_peer").into()),
                    }
                }
            }
            if ui
                .add_enabled(app.running, egui::Button::new(l.text("p2p_reset")))
                .clicked()
            {
                command = Some(P2pCommand::Reset);
            }
        });
        ui.label(l.text("p2p_neighbors"));
        for p in &app.p2p.neighbors {
            ui.monospace(format!("{}:{}", p.address, p.port));
        }
        ui.add_enabled_ui(app.running, |ui| {
            let label = ui.label("TTL");
            ui.add(egui::DragValue::new(&mut d.ttl).range(1..=16))
                .labelled_by(label.id);
            if ui.button(l.text("p2p_discover")).clicked() {
                command = Some(P2pCommand::Discover { ttl: d.ttl });
            }
            egui::CollapsingHeader::new(l.text("p2p_discovered")).show(ui, |ui| {
                for p in &app.p2p.discovered {
                    ui.monospace(format!("{}:{}", p.address, p.port));
                }
            });
            field(ui, l.text("p2p_query"), &mut d.query);
            ui.horizontal(|ui| {
                if ui.button(l.text("p2p_search")).clicked() {
                    command = Some(P2pCommand::Search {
                        text: d.query.clone(),
                        ttl: d.ttl,
                    });
                }
                if ui
                    .add_enabled(app.p2p.searching, egui::Button::new(l.text("p2p_cancel")))
                    .clicked()
                {
                    command = Some(P2pCommand::CancelSearch);
                }
                if ui.button(l.text("p2p_clear")).clicked() {
                    command = Some(P2pCommand::ClearResults);
                    d.selected = None;
                }
            });
            ui.label(l.text("p2p_results"));
            egui::ScrollArea::vertical()
                .id_salt("p2p-results")
                .max_height(160.)
                .show(ui, |ui| {
                    for r in &app.p2p.results {
                        if ui
                            .selectable_label(
                                d.selected.as_ref() == Some(&r.id),
                                format!(
                                    "{} · {}:{} · {} B",
                                    r.path, r.source.address, r.source.port, r.bytes
                                ),
                            )
                            .clicked()
                        {
                            d.selected = Some(r.id.clone());
                        }
                    }
                });
            field(ui, l.text("destination"), &mut draft.destination);
            if ui
                .add_enabled(
                    d.selected.is_some() && !app.p2p.downloading,
                    egui::Button::new(l.text("p2p_download")),
                )
                .clicked()
            {
                command = Some(P2pCommand::Download {
                    result: d.selected.clone().unwrap(),
                    destination: draft.destination.clone(),
                });
            }
            if app.p2p.downloading {
                ui.spinner();
            }
        });
        if let Some(command) = command {
            self.app_command(
                host,
                AppCommand::P2p {
                    application: app.id.clone(),
                    command,
                },
            );
        }
        for value in &app.lines {
            ui.label(line(l, value));
        }
    }
}
