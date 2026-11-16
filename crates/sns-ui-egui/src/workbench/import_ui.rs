//! Bounded background import. Only a complete validated project replaces the editor.
use super::*;
use sns_filius_import::{ImportReport, ImportResult};
use std::sync::mpsc;
#[derive(Default)]
pub(super) struct ImportUi {
    pub(super) receiver: Option<mpsc::Receiver<Result<ImportResult, Diagnostic>>>,
    pub(super) report: Option<ImportReport>,
    pub(super) open: bool,
}
impl ImportUi {
    fn start(
        &mut self,
        job: impl FnOnce() -> Result<ImportResult, Diagnostic> + Send + 'static,
    ) -> Result<(), Diagnostic> {
        if self.receiver.is_some() {
            return Err(Diagnostic::new("import.busy", "import"));
        }
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("sinesi-import".into())
            .spawn(move || {
                let _ = tx.send(job());
            })
            .map_err(|_| Diagnostic::new("import.worker_failed", "import"))?;
        self.receiver = Some(rx);
        Ok(())
    }
    pub(super) fn file(&mut self, path: PathBuf) -> Result<(), Diagnostic> {
        self.start(move || {
            use std::io::Read as _;
            if !std::fs::metadata(&path)
                .map_err(|_| Diagnostic::new("project.io", "import"))?
                .is_file()
            {
                return Err(Diagnostic::new(
                    "import.invalid_archive",
                    "not_regular_file",
                ));
            }
            let mut bytes = vec![];
            std::fs::File::open(&path)
                .map_err(|_| Diagnostic::new("project.io", "import"))?
                .take(32 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| Diagnostic::new("project.io", "import"))?;
            sns_filius_import::import(
                &bytes,
                path.file_name()
                    .and_then(|p| p.to_str())
                    .unwrap_or("import.fls"),
            )
        })
    }
}
impl Workbench {
    /// Explicit replacement API, also used to exercise the background path without native dialogs.
    pub fn begin_filius_import(
        &mut self,
        bytes: Vec<u8>,
        source_name: String,
    ) -> Result<(), Diagnostic> {
        if self.runtime_busy() || self.document_load.is_some() {
            return Err(Diagnostic::new("state.pending_commands", "runtime"));
        }
        self.import_ui
            .start(move || sns_filius_import::import(&bytes, &source_name))
    }
    pub fn import_report(&self) -> Option<&ImportReport> {
        self.import_ui.report.as_ref()
    }
    pub(super) fn poll_import(&mut self, ctx: &egui::Context) {
        let outcome = self
            .import_ui
            .receiver
            .as_ref()
            .and_then(|rx| match rx.try_recv() {
                Ok(r) => Some(r),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(_) => Some(Err(Diagnostic::new("import.worker_failed", "import"))),
            });
        if let Some(result) = outcome {
            self.import_ui.receiver = None;
            match result {
                Err(e) => {
                    self.journal.record(TechnicalEvent::ImportFailed);
                    self.error = Some(e.to_string());
                }
                Ok(result) => match Editor::new(result.project) {
                    Err(e) => self.error = Some(e.to_string()),
                    Ok(mut editor) => {
                        editor.mark_unsaved();
                        self.install_document(editor, None);
                        self.import_ui.report = Some(result.report);
                        self.import_ui.open = true;
                    }
                },
            }
        }
        if self.import_ui.receiver.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(20));
        }
    }
    pub(super) fn import_window(&mut self, ctx: &egui::Context) {
        let l = self.language;
        let Some(report) = &self.import_ui.report else {
            return;
        };
        let mut close_report = false;
        let mut export_path = None;
        egui::Window::new(l.text("import_report"))
            .id(egui::Id::unique("filius-import-report"))
            .default_size((850., 620.))
            .vscroll(true)
            .open(&mut self.import_ui.open)
            .show(ctx, |ui| {
                close_report = ui.button(l.text("close_view")).clicked();
                ui.heading(&report.source_name);
                ui.label(l.text("import_data_only"));
                let c = &report.counts;
                ui.monospace(format!(
                    "{}: {} · {}: {} · {}: {}",
                    l.text("devices"),
                    c.devices,
                    l.text("import_links"),
                    c.links,
                    l.text("software"),
                    c.applications
                ));
                ui.label(
                    l.format(
                        "import_coverage",
                        &[
                            ("converted", &report.converted.len().to_string()),
                            ("remaining", &report.unconverted_fields.len().to_string()),
                            ("entries", &report.preserved.len().to_string()),
                        ],
                    )
                    .unwrap(),
                );
                if ui.button(l.text("import_export_report")).clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .set_file_name("import-report.json")
                        .save_file()
                {
                    export_path = Some(path);
                }
                egui::CollapsingHeader::new(l.text("import_issues"))
                    .default_open(true)
                    .show(ui, |ui| {
                        for issue in &report.issues {
                            ui.label(format!(
                                "{} · {} · {}",
                                l.text(&issue.code),
                                issue.source,
                                issue.detail
                            ));
                        }
                    });
                egui::CollapsingHeader::new(l.text("import_remaining")).show(ui, |ui| {
                    for field in &report.unconverted_fields {
                        ui.monospace(field);
                    }
                });
                egui::CollapsingHeader::new(l.text("import_converted")).show(ui, |ui| {
                    for field in &report.converted {
                        ui.monospace(format!(
                            "{} → {} ({})",
                            field.source, field.target, field.rule
                        ));
                    }
                });
                egui::CollapsingHeader::new(l.text("import_preserved")).show(ui, |ui| {
                    for e in &report.preserved {
                        ui.monospace(format!(
                            "{} · {} B · SHA-256 {}",
                            e.name, e.bytes.0, e.sha256
                        ));
                    }
                });
                egui::CollapsingHeader::new(l.text("import_unknown_classes")).show(ui, |ui| {
                    for class in &report.unknown_classes {
                        ui.monospace(class);
                    }
                });
            });
        if let Some(path) = export_path {
            let result = self
                .export_output_allowed(&path)
                .and_then(|()| serde_json::to_vec_pretty(report).map_err(|e| e.to_string()))
                .and_then(|bytes| {
                    sns_project::atomic_write(&path, &bytes).map_err(|e| e.to_string())
                });
            if let Err(e) = result {
                self.error = Some(e);
            }
        }
        if close_report {
            self.import_ui.open = false;
        }
    }
}
