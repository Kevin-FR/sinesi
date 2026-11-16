//! Explicit runtime files; reading, reconstruction and writing stay off the UI thread.
use super::*;
use std::sync::mpsc;
#[derive(Default)]
pub(super) struct RuntimeUi {
    pub open: bool,
    pub preserve_manual_loss: bool,
    receiver: Option<mpsc::Receiver<Result<Outcome, String>>>,
    loaded: bool,
    generation_floor: u64,
    saved: bool,
}
enum Outcome {
    Loaded(Box<sns_engine::Engine>),
    Saved,
}
impl RuntimeUi {
    fn start(
        &mut self,
        work: impl FnOnce() -> Result<Outcome, String> + Send + 'static,
    ) -> Result<(), String> {
        if self.receiver.is_some() {
            return Err("state.pending_commands".into());
        }
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("sinesi-runtime-file".into())
            .spawn(move || {
                let _ = tx.send(work());
            })
            .map_err(|e| e.to_string())?;
        self.receiver = Some(rx);
        self.saved = false;
        Ok(())
    }
}
impl Workbench {
    pub fn runtime_busy(&self) -> bool {
        self.runtime_ui.receiver.is_some()
    }
    /// Explicit replacement API: UI callers request unsaved-change confirmation before this operation.
    pub fn begin_checkpoint_restore(&mut self, bytes: Vec<u8>) -> Result<(), String> {
        if self.import_ui.receiver.is_some() || self.document_load.is_some() {
            return Err("state.pending_commands".into());
        }
        self.runtime_ui.start(move || {
            sns_engine::Checkpoint::from_bytes(&bytes)
                .and_then(|p| p.restore())
                .map(|e| Outcome::Loaded(Box::new(e)))
                .map_err(|e| e.to_string())
        })
    }
    pub fn begin_recording_replay(&mut self, bytes: Vec<u8>) -> Result<(), String> {
        if self.import_ui.receiver.is_some() || self.document_load.is_some() {
            return Err("state.pending_commands".into());
        }
        self.runtime_ui.start(move || {
            sns_engine::Recording::from_bytes(&bytes)
                .and_then(|r| r.replay())
                .map(|e| Outcome::Loaded(Box::new(e)))
                .map_err(|e| e.to_string())
        })
    }
    pub fn export_runtime_to(&mut self, path: PathBuf, checkpoint: bool) -> Result<(), String> {
        self.document_output_allowed(&path)?;
        if self.runtime_busy() || self.mode != Mode::Simulation {
            return Err("state.not_paused".into());
        }
        let resolved = |p: &std::path::Path| -> Result<PathBuf, String> {
            if p.exists() {
                p.canonicalize().map_err(|e| e.to_string())
            } else {
                p.parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .canonicalize()
                    .map(|p2| p2.join(p.file_name().unwrap_or_default()))
                    .map_err(|e| e.to_string())
            }
        };
        if let Some(project) = &self.path
            && resolved(project)? == resolved(&path)?
        {
            return Err("runtime.project_path_conflict".into());
        }
        let (tx, rx) = mpsc::sync_channel(1);
        self.worker.capture(checkpoint, tx)?;
        if self.import_ui.receiver.is_some() {
            return Err("state.pending_commands".into());
        }
        self.runtime_ui.start(move || {
            let bytes = rx
                .recv()
                .map_err(|_| "runtime.worker_failed".to_string())??;
            sns_project::atomic_write_runtime(&path, &bytes).map_err(|e| e.to_string())?;
            Ok(Outcome::Saved)
        })
    }
    pub(super) fn begin_runtime_file(
        &mut self,
        path: PathBuf,
        action: Replacement,
    ) -> Result<(), String> {
        if self.import_ui.receiver.is_some() {
            return Err("state.pending_commands".into());
        }
        let wait_factor = self.launch_wait_factor;
        self.runtime_ui.start(move || {
            let bytes = sns_project::read_runtime_document(&path).map_err(|e| e.to_string())?;
            let engine = match action {
                Replacement::LoadScenario => {
                    let scenario =
                        sns_project::parse_scenario(&bytes).map_err(|e| e.to_string())?;
                    sns_engine::Engine::new_with_wait_factor(
                        scenario,
                        None,
                        sns_engine::Limits::default(),
                        wait_factor,
                    )
                }
                Replacement::RestoreCheckpoint => {
                    sns_engine::Checkpoint::from_bytes(&bytes).and_then(|p| p.restore())
                }
                Replacement::ReplayRecording => {
                    sns_engine::Recording::from_bytes(&bytes).and_then(|r| r.replay())
                }
                _ => return Err("checkpoint.incompatible".into()),
            }
            .map_err(|e| e.to_string())?;
            Ok(Outcome::Loaded(Box::new(engine)))
        })
    }
    pub(super) fn poll_runtime(&mut self, ctx: &egui::Context) {
        let outcome = self
            .runtime_ui
            .receiver
            .as_ref()
            .and_then(|r| match r.try_recv() {
                Ok(v) => Some(v),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(_) => Some(Err("runtime.worker_failed".into())),
            });
        if let Some(result) = outcome {
            self.runtime_ui.receiver = None;
            match result {
                Err(e) => {
                    self.journal.record(TechnicalEvent::RuntimeFileFailed);
                    self.error = Some(e);
                }
                Ok(Outcome::Saved) => {
                    self.runtime_ui.saved = true;
                    self.error = None;
                }
                Ok(Outcome::Loaded(mut engine)) => {
                    if engine.state() == RunState::Running
                        && let Err(e) = engine.pump(0, None)
                    {
                        self.error = Some(e.to_string());
                        return;
                    }
                    self.runtime_ui.generation_floor = self
                        .runtime_ui
                        .generation_floor
                        .max(self.worker.view.generation);
                    if let Err(e) = engine.renew_generation_after(self.runtime_ui.generation_floor)
                    {
                        self.error = Some(e.to_string());
                        return;
                    }
                    let preserve_manual_loss = engine.network().manual_loss;
                    let prepared = Editor::new(engine.project().clone())
                        .map_err(|e| e.to_string())
                        .and_then(|mut editor| {
                            editor.mark_unsaved();
                            let worker = Worker::from_engine(*engine)?;
                            Ok((editor, worker))
                        });
                    match prepared {
                        Err(e) => self.error = Some(e),
                        Ok((editor, worker)) => {
                            self.runtime_ui.generation_floor = worker.view.generation;
                            self.editor = editor;
                            self.session_ui.can_save = true;
                            self.duplicate_omitted = 0;
                            self.worker = worker;
                            self.synced_revision = self.worker.view.revision;
                            self.desktop = Default::default();
                            self.conditions_ui = Default::default();
                            self.routing_ui = Default::default();
                            self.security_ui = Default::default();
                            self.traffic = Default::default();
                            self.import_ui.report = None;
                            self.import_ui.open = false;
                            self.mode = Mode::Simulation;
                            self.page = Page::Topology;
                            self.selected.clear();
                            self.draft = None;
                            self.annotations = Default::default();
                            self.cable = None;
                            self.cabling = false;
                            self.event_index = None;
                            self.filter.clear();
                            self.path = None;
                            self.fit = true;
                            self.observed_trace = 0;
                            self.active_links.clear();
                            self.manual_loss = preserve_manual_loss;
                            self.runtime_ui.preserve_manual_loss = preserve_manual_loss;
                            self.worker.submit(Job::Speed(self.speed));
                            self.error = None;
                            self.runtime_ui.loaded = true;
                            self.runtime_ui.open = true;
                        }
                    }
                }
            }
        }
        if self.runtime_busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(20));
        }
    }
    pub(super) fn runtime_window(&mut self, ctx: &egui::Context) {
        if !self.runtime_ui.open {
            return;
        }
        let l = self.language;
        let mut open = true;
        let mut close = false;
        let mut replace = None;
        let mut export = None;
        egui::Window::new(l.text("runtime_menu"))
            .id(egui::Id::unique("runtime-files"))
            .default_width(640.)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(l.text("runtime_offline"));
                if ui.button(l.text("link_conditions_menu")).clicked() {
                    self.open_link_conditions(None);
                    close = true;
                }
                for (key, action) in [
                    ("runtime_load_scenario", Replacement::LoadScenario),
                    ("runtime_restore", Replacement::RestoreCheckpoint),
                    ("runtime_replay", Replacement::ReplayRecording),
                ] {
                    if ui.button(l.text(key)).clicked() {
                        replace = Some(action);
                    }
                }
                ui.separator();
                for (key, checkpoint) in [
                    ("runtime_save_checkpoint", true),
                    ("runtime_save_recording", false),
                ] {
                    if ui
                        .add_enabled(
                            self.mode == Mode::Simulation,
                            egui::Button::new(l.text(key)),
                        )
                        .clicked()
                    {
                        export = Some(checkpoint);
                    }
                }
                ui.separator();
                if ui
                    .add_enabled(
                        !self.document_export_busy(),
                        egui::Button::new(l.text("diagnostic_export")),
                    )
                    .clicked()
                {
                    self.choose_diagnostic_export();
                }
                if self.export_ui.diagnostic_saved {
                    ui.label(l.text("diagnostic_saved"));
                }
                if self.runtime_ui.loaded {
                    ui.label(l.text("runtime_loaded"));
                }
                if self.runtime_ui.saved {
                    ui.label(l.text("runtime_saved"));
                }
                if ui.button(l.text("close_view")).clicked() {
                    close = true;
                }
            });
        self.runtime_ui.open = open && !close;
        if let Some(action) = replace {
            self.replace(action);
        }
        if let Some(checkpoint) = export
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name(if checkpoint {
                    "checkpoint.json"
                } else {
                    "recording.json"
                })
                .save_file()
            && let Err(e) = self.export_runtime_to(path, checkpoint)
        {
            self.error = Some(e);
        }
    }
}
