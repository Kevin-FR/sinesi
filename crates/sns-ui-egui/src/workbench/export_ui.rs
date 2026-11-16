//! Native destination selection and CPU export both run outside the UI thread.
use super::*;
use sns_render::{PngOptions, Scene};
use std::sync::mpsc;
#[derive(Clone, Debug)]
pub struct PngReceipt {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub revision: u64,
    pub diagram: bool,
    pub clipped_texts: usize,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Png,
    Pdf,
    Diagnostic,
}
#[derive(Clone, Debug)]
pub struct PdfReceipt {
    pub path: PathBuf,
    pub pages: usize,
    pub packet_rows: usize,
    pub revision: u64,
    pub diagram: bool,
    pub trace_partial: bool,
    pub clipped_texts: usize,
}
enum Outcome {
    Chosen(Option<PathBuf>, Format),
    PdfWritten(PdfReceipt),
    DiagnosticWritten,
    Written(PngReceipt),
}
#[derive(Default)]
pub(super) struct ExportUi {
    pub open: bool,
    options: PngOptions,
    task: Option<mpsc::Receiver<Result<Outcome, String>>>,
    destination: Option<PathBuf>,
    saved: Option<PngReceipt>,
    saved_pdf: Option<PdfReceipt>,
    error: Option<String>,
    pub(super) diagnostic_saved: bool,
}
impl ExportUi {
    fn start(
        &mut self,
        name: &str,
        job: impl FnOnce() -> Result<Outcome, String> + Send + 'static,
    ) -> Result<(), String> {
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name(name.into())
            .spawn(move || {
                let _ = tx.send(job());
            })
            .map_err(|e| e.to_string())?;
        self.task = Some(rx);
        self.error = None;
        self.saved = None;
        self.saved_pdf = None;
        self.diagnostic_saved = false;
        Ok(())
    }
}
impl Workbench {
    pub fn document_export_busy(&self) -> bool {
        self.export_ui.task.is_some()
    }
    pub fn document_export_error(&self) -> Option<&str> {
        self.export_ui.error.as_deref()
    }
    pub fn png_export_receipt(&self) -> Option<&PngReceipt> {
        self.export_ui.saved.as_ref()
    }
    pub fn png_export_options(&self) -> PngOptions {
        self.export_ui.options
    }
    pub(super) fn export_destination_matches(&self, path: &std::path::Path) -> bool {
        self.export_ui
            .destination
            .as_ref()
            .is_some_and(|p| sns_project::destination::same_destination(p, path))
    }
    pub fn pdf_export_receipt(&self) -> Option<&PdfReceipt> {
        self.export_ui.saved_pdf.as_ref()
    }
    pub fn begin_pdf_export(&mut self, path: PathBuf) -> Result<(), String> {
        self.worker.poll();
        self.sync_applications();
        if self.document_export_busy()
            || self.worker.pending_commands()
            || self.runtime_busy()
            || self.import_ui.receiver.is_some()
        {
            return Err("state.pending_commands".into());
        }
        self.export_output_allowed(&path)?;
        let mut snapshot = sns_report::Snapshot::configured(self.project().clone());
        snapshot.revision = self.editor.revision();
        if self.mode == Mode::Simulation && self.worker.current() {
            snapshot.effective = Some(self.worker.view.effective.clone());
            snapshot.events = self.worker.view.trace.clone();
            snapshot.routes = self.worker.view.routes.clone();
            snapshot.time_us = self.worker.view.now;
            snapshot.run_id = Some(self.worker.view.run_id.clone());
        }
        let options = sns_report::ReportOptions {
            language: self.language,
            diagram: self.art.diagram,
        };
        let destination = path.clone();
        self.export_ui.start("network-pdf", move || {
            let report =
                sns_report::Report::build(&snapshot, options).map_err(|e| e.to_string())?;
            let pdf = sns_report::render_pdf(&report).map_err(|e| e.to_string())?;
            sns_project::atomic_write_runtime(&path, &pdf.bytes).map_err(|e| e.to_string())?;
            Ok(Outcome::PdfWritten(PdfReceipt {
                path,
                pages: pdf.pages,
                packet_rows: pdf.packet_rows,
                revision: snapshot.revision,
                diagram: options.diagram,
                trace_partial: pdf.trace_partial,
                clipped_texts: pdf.clipped_scene_texts,
            }))
        })?;
        self.export_ui.destination = Some(destination);
        Ok(())
    }
    pub fn begin_png_export(&mut self, path: PathBuf) -> Result<(), String> {
        self.worker.poll();
        self.sync_applications();
        if self.document_export_busy()
            || self.worker.pending_commands()
            || (self.mode == Mode::Simulation && !self.worker.current())
            || self.runtime_busy()
            || self.import_ui.receiver.is_some()
        {
            return Err("state.pending_commands".into());
        }
        self.export_output_allowed(&path)?;
        let project = self.project().clone();
        let labels = project
            .devices
            .iter()
            .map(|d| (d.id.clone(), self.device_label(d)))
            .collect();
        let diagram = self.art.diagram;
        let revision = self.editor.revision();
        let options = self.export_ui.options;
        let destination = path.clone();
        self.export_ui.start("network-png", move || {
            let scene =
                Scene::build_with_labels(&project, diagram, &labels).map_err(|e| e.to_string())?;
            let image = sns_render::render_png(&scene, options).map_err(|e| e.to_string())?;
            sns_project::atomic_write_runtime(&path, &image.bytes).map_err(|e| e.to_string())?;
            Ok(Outcome::Written(PngReceipt {
                path,
                width: image.width,
                height: image.height,
                revision,
                diagram,
                clipped_texts: scene.counts().clipped_texts,
            }))
        })?;
        self.export_ui.destination = Some(destination);
        Ok(())
    }
    pub fn begin_diagnostic_export(&mut self, path: PathBuf) -> Result<(), String> {
        if self.document_export_busy() || self.runtime_busy() || self.import_ui.receiver.is_some() {
            return Err("state.pending_commands".into());
        }
        self.export_output_allowed(&path)?;
        let snapshot = self.journal.snapshot();
        let destination = path.clone();
        self.export_ui.start("diagnostic-export", move || {
            snapshot
                .write_to(&path)
                .map_err(|_| "diagnostic.export_failed".to_owned())?;
            Ok(Outcome::DiagnosticWritten)
        })?;
        self.export_ui.destination = Some(destination);
        Ok(())
    }

    pub(super) fn choose_diagnostic_export(&mut self) {
        self.choose_document_export(Format::Diagnostic);
    }

    pub(super) fn poll_document_export(&mut self, ctx: &egui::Context) {
        let outcome = self
            .export_ui
            .task
            .as_ref()
            .and_then(|rx| match rx.try_recv() {
                Ok(o) => Some(o),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(_) => Some(Err("runtime.worker_failed".into())),
            });
        if let Some(outcome) = outcome {
            self.export_ui.task = None;
            self.export_ui.destination = None;
            match outcome {
                Ok(Outcome::Chosen(Some(path), format)) => {
                    if let Err(e) = match format {
                        Format::Png => self.begin_png_export(path),
                        Format::Pdf => self.begin_pdf_export(path),
                        Format::Diagnostic => self.begin_diagnostic_export(path),
                    } {
                        self.journal.record(TechnicalEvent::ExportFailed);
                        if format == Format::Diagnostic {
                            self.error = Some(e);
                        } else {
                            self.export_ui.error = Some(e);
                        }
                    }
                }
                Ok(Outcome::Chosen(None, _)) => {}
                Ok(Outcome::PdfWritten(receipt)) => self.export_ui.saved_pdf = Some(receipt),
                Ok(Outcome::Written(receipt)) => self.export_ui.saved = Some(receipt),
                Ok(Outcome::DiagnosticWritten) => {
                    self.journal.record(TechnicalEvent::DiagnosticExported);
                    self.export_ui.diagnostic_saved = true;
                }
                Err(e) => {
                    self.journal.record(TechnicalEvent::ExportFailed);
                    if e == "diagnostic.export_failed" {
                        self.error = Some(e);
                    } else {
                        self.export_ui.error = Some(e);
                    }
                }
            }
            ctx.request_repaint();
        }
        if self.document_export_busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
    }
    pub(super) fn document_export_window(&mut self, ctx: &egui::Context) {
        if !self.export_ui.open {
            return;
        }
        let l = self.language;
        let mut open = true;
        let mut choose = None;
        egui::Window::new(l.text("export_network"))
            .id(egui::Id::unique("network-export"))
            .default_pos([430., 210.])
            .default_width(500.)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(l.text("png_content_hint"));
                ui.label(
                    l.format(
                        "png_view",
                        &[(
                            "view",
                            l.text(if self.art.diagram {
                                "view_diagram"
                            } else {
                                "view_icons"
                            }),
                        )],
                    )
                    .unwrap(),
                );
                ui.add_enabled_ui(!self.document_export_busy(), |ui| {
                    let label = ui.label(l.text("png_longest_side"));
                    ui.add(
                        egui::DragValue::new(&mut self.export_ui.options.longest_side)
                            .range(256..=4096)
                            .speed(64.),
                    )
                    .labelled_by(label.id);
                    ui.checkbox(
                        &mut self.export_ui.options.transparent,
                        l.text("png_transparent"),
                    );
                    if ui.button(l.text("export_png")).clicked() {
                        choose = Some(Format::Png);
                    }
                    ui.separator();
                    ui.label(l.text("report_hint"));
                    if ui.button(l.text("export_pdf")).clicked() {
                        choose = Some(Format::Pdf);
                    }
                });
                if self.document_export_busy() {
                    ui.spinner();
                    ui.label(l.text("document_export_working"));
                }
                if let Some(saved) = &self.export_ui.saved {
                    ui.label(
                        l.format(
                            "png_saved",
                            &[
                                ("width", &saved.width.to_string()),
                                ("height", &saved.height.to_string()),
                            ],
                        )
                        .unwrap(),
                    );
                    ui.label(
                        l.format(
                            "png_view",
                            &[(
                                "view",
                                l.text(if saved.diagram {
                                    "view_diagram"
                                } else {
                                    "view_icons"
                                }),
                            )],
                        )
                        .unwrap(),
                    );
                    if saved.clipped_texts > 0 {
                        ui.label(l.text("png_clipped_hint"));
                    }
                    ui.label(saved.path.file_name().unwrap_or_default().to_string_lossy())
                        .on_hover_text(saved.path.display().to_string());
                }
                if let Some(saved) = &self.export_ui.saved_pdf {
                    ui.label(format!(
                        "{} · {} {} · {} {}",
                        l.text("report_saved"),
                        saved.pages,
                        l.text("report_pages"),
                        saved.packet_rows,
                        l.text("report_packets")
                    ));
                    if saved.trace_partial {
                        ui.label(l.text("report_partial_trace"));
                    }
                    if saved.clipped_texts > 0 {
                        ui.label(l.text("png_clipped_hint"));
                    }
                    ui.label(saved.path.file_name().unwrap_or_default().to_string_lossy())
                        .on_hover_text(saved.path.display().to_string());
                }
                if let Some(error) = &self.export_ui.error {
                    ui.colored_label(Color32::LIGHT_RED, l.describe_error(error));
                }
            });
        self.export_ui.open = open;
        if let Some(format) = choose {
            self.choose_document_export(format);
        }
    }

    fn choose_document_export(&mut self, format: Format) {
        if self.document_export_busy() {
            return;
        }
        let (key, filter, extension, filename) = match format {
            Format::Png => ("export_png", "PNG", "png", "network.png"),
            Format::Pdf => ("export_pdf", "PDF", "pdf", "network-report.pdf"),
            Format::Diagnostic => (
                "diagnostic_export",
                "JSON",
                "json",
                "sinesi-diagnostic.json",
            ),
        };
        let title = self.language.text(key).to_owned();
        if let Err(error) = self.export_ui.start("document-destination", move || {
            let file = pollster::block_on(
                rfd::AsyncFileDialog::new()
                    .set_title(title)
                    .add_filter(filter, &[extension])
                    .set_file_name(filename)
                    .save_file(),
            );
            Ok(Outcome::Chosen(
                file.map(|f| f.path().to_path_buf()),
                format,
            ))
        }) {
            self.journal.record(TechnicalEvent::ExportFailed);
            if format == Format::Diagnostic {
                self.error = Some("diagnostic.export_failed".into());
            } else {
                self.export_ui.error = Some(error);
            }
        }
    }
}
