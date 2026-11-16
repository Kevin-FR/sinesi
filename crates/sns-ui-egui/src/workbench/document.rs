//! Workbench document responsibilities.
use super::*;
use std::{sync::mpsc, time::Duration};

const PROJECT_EXTENSIONS: &[&str] = &["sinesi", "json"];

enum DocumentInput {
    Project(Box<Editor>, PathBuf),
    /// A teacher's mission, read and validated off the frame.
    Mission(Box<Editor>, Box<sns_missions::Mission>),
    File(PathBuf, Replacement),
}

pub(super) struct DocumentLoad {
    outcome: mpsc::Receiver<Result<Option<DocumentInput>, String>>,
    epoch: u64,
    revision: u64,
    mode: Mode,
}

impl Workbench {
    fn choose_document(&mut self, action: Replacement) {
        let (filter, extensions): (&str, &[&str]) = match action {
            Replacement::Open => (self.language.text("project_file"), PROJECT_EXTENSIONS),
            Replacement::ImportFilius => ("Filius", &["fls"]),
            Replacement::OpenMission => (self.language.text("mission_file"), &["json"]),
            _ => ("JSON", &["json"]),
        };
        if let Err(error) = self.begin_document_load(action, move || {
            pollster::block_on(
                rfd::AsyncFileDialog::new()
                    .add_filter(filter, extensions)
                    .pick_file(),
            )
            .map(|file| file.path().to_path_buf())
        }) {
            self.error = Some(error);
        }
    }

    fn begin_document_load(
        &mut self,
        action: Replacement,
        pick: impl FnOnce() -> Option<PathBuf> + Send + 'static,
    ) -> Result<(), String> {
        if self.document_load.is_some()
            || self.document_export_busy()
            || self.runtime_busy()
            || self.import_ui.receiver.is_some()
            || self.worker.pending_commands()
        {
            return Err("state.pending_commands".into());
        }
        let (send, outcome) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("sinesi-document-open".into())
            .spawn(move || {
                let result = pick()
                    .map(|path| {
                        if matches!(action, Replacement::OpenMission) {
                            std::fs::read(&path)
                                .map_err(|error| error.to_string())
                                .and_then(|bytes| sns_missions::parse(&bytes))
                                .and_then(|mission| {
                                    Editor::new(mission.project.clone())
                                        .map(|editor| {
                                            DocumentInput::Mission(
                                                Box::new(editor),
                                                Box::new(mission),
                                            )
                                        })
                                        .map_err(|error| error.to_string())
                                })
                        } else if matches!(action, Replacement::Open) {
                            sns_project::load_project(&path)
                                .and_then(|project| Editor::new(project).map_err(Into::into))
                                .map(|editor| DocumentInput::Project(Box::new(editor), path))
                                .map_err(|error| error.to_string())
                        } else {
                            Ok(DocumentInput::File(path, action))
                        }
                    })
                    .transpose();
                let _ = send.send(result);
            })
            .map_err(|_| "project.io".to_owned())?;
        self.document_load = Some(DocumentLoad {
            outcome,
            epoch: self.worker.epoch,
            revision: self.editor.revision(),
            mode: self.mode,
        });
        Ok(())
    }

    pub(super) fn poll_document_load(&mut self, ctx: &egui::Context) {
        let Some(load) = &self.document_load else {
            return;
        };
        let result = match load.outcome.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Disconnected) => Err("project.io".into()),
            Err(mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(20));
                return;
            }
        };
        let (epoch, revision, mode) = (load.epoch, load.revision, load.mode);
        self.worker.poll();
        self.sync_applications();
        let current = epoch == self.worker.epoch
            && revision == self.editor.revision()
            && mode == self.mode
            && !self.worker.pending_commands()
            && !self.session_ui.loading
            && !self.runtime_busy()
            && self.import_ui.receiver.is_none();
        self.document_load = None;
        if !current {
            return;
        }
        let result = match result {
            Ok(Some(DocumentInput::Project(editor, path))) => {
                self.install_document(*editor, Some(path));
                return;
            }
            Ok(Some(DocumentInput::Mission(editor, mission))) => {
                self.install_document(*editor, None);
                let index = self.add_teacher_mission(*mission);
                self.begin_mission(index);
                return;
            }
            Ok(Some(DocumentInput::File(path, Replacement::ImportFilius))) => {
                self.import_ui.file(path).map_err(|error| error.to_string())
            }
            Ok(Some(DocumentInput::File(path, action))) => self.begin_runtime_file(path, action),
            Ok(None) => return,
            Err(error) => {
                self.journal.record(TechnicalEvent::ProjectLoadFailed);
                Err(error)
            }
        };
        if let Err(error) = result {
            self.error = Some(error);
        }
    }

    pub(super) fn replace(&mut self, action: Replacement) {
        if self.worker.pending_commands() {
            self.error = Some("state.pending_commands".into());
            return;
        }
        if self.editor.dirty() || self.desktop.dirty() {
            self.pending = Some(action);
        } else {
            self.replace_now(action);
        }
    }
    pub(super) fn replace_now(&mut self, action: Replacement) {
        let language = self.language;
        if matches!(action, Replacement::Close) {
            self.close_confirmed = true;
            return;
        }
        let result = match action {
            Replacement::RecoverSession => {
                self.load_requested_session();
                Ok(None)
            }
            Replacement::Close => unreachable!(),
            Replacement::Open
            | Replacement::OpenMission
            | Replacement::ImportFilius
            | Replacement::LoadScenario
            | Replacement::RestoreCheckpoint
            | Replacement::ReplayRecording => {
                self.choose_document(action);
                Ok(None)
            }
            Replacement::New => {
                let mut project = Editor::empty().project().clone();
                project.name = language.text("new_network").into();
                Editor::new(project)
                    .map_err(sns_project::Error::from)
                    .map(Some)
            }
            Replacement::Example(example) => sns_project::parse_project(example.project_bytes())
                .and_then(|p| Editor::new(p).map_err(sns_project::Error::from))
                .map(Some),
            Replacement::Mission(index) => match self.mission_project(index) {
                Some(project) => Editor::new(project)
                    .map_err(sns_project::Error::from)
                    .map(Some),
                None => Ok(None),
            },
        };
        match result {
            Ok(Some(editor)) => {
                self.install_document(editor, None);
                if let Replacement::Mission(index) = action {
                    self.begin_mission(index);
                }
            }
            Ok(None) => {}
            Err(e) => {
                self.journal.record(TechnicalEvent::ProjectLoadFailed);
                self.error = Some(e.to_string());
            }
        }
    }

    pub(super) fn install_document(&mut self, editor: Editor, path: Option<PathBuf>) {
        self.session_ui.can_save = true;
        self.set_mode(Mode::Design);
        self.editor = editor;
        self.duplicate_omitted = 0;
        self.desktop = Default::default();
        self.worker
            .restart(self.project().clone(), self.launch_wait_factor);
        self.synced_revision = 0;
        self.page = Page::Topology;
        self.event_index = None;
        self.filter.clear();
        self.path = path;
        self.selected.clear();
        self.draft = None;
        self.annotations = Default::default();
        self.fit = true;
        self.error = None;
        // Another document ends the running mission; Mission installs begin a new one.
        self.missions.active = None;
        self.missions_feedback_text = None;
    }
    pub(super) fn save(&mut self, save_as: bool) -> bool {
        if self.worker.pending_commands() {
            self.error = Some("state.pending_commands".into());
            return false;
        }
        let language = self.language;
        let path = if save_as || self.path.is_none() {
            rfd::FileDialog::new()
                .add_filter(language.text("project_file"), PROJECT_EXTENSIONS)
                .set_file_name(language.text("project_filename"))
                .save_file()
        } else {
            self.path.clone()
        };
        let Some(path) = path else {
            return false;
        };
        match self.save_project_to(path) {
            Ok(()) => true,
            Err(e) => {
                self.error = Some(e);
                false
            }
        }
    }
    pub fn save_project_to(&mut self, path: PathBuf) -> Result<(), String> {
        if self.worker.pending_commands() {
            return Err("state.pending_commands".into());
        }
        self.document_output_allowed(&path)?;
        match sns_project::save_project(&path, self.project()) {
            Ok(()) => {
                self.path = Some(path);
                self.editor.mark_saved();
                Ok(())
            }
            Err(e) => {
                self.journal.record(TechnicalEvent::ProjectSaveFailed);
                self.error = Some(e.to_string());
                Err(e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn finish_open(workbench: &mut Workbench) {
        let context = egui::Context::default();
        let deadline = Instant::now() + Duration::from_secs(5);
        while workbench.document_load.is_some() {
            assert!(Instant::now() < deadline, "document worker did not finish");
            workbench.poll_document_load(&context);
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn document_open_waits_without_blocking_and_installs_a_validated_project() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("network.json");
        let mut workbench = Workbench::new().unwrap();
        let mut project = workbench.project().clone();
        project.name = "Opened project".into();
        sns_project::save_project(&path, &project).unwrap();
        let (release, wait) = mpsc::channel();
        let selected = path.clone();
        workbench
            .begin_document_load(Replacement::Open, move || {
                wait.recv().unwrap();
                Some(selected)
            })
            .unwrap();
        workbench.poll_document_load(&egui::Context::default());
        assert!(workbench.document_load.is_some());
        assert_ne!(workbench.project().name, project.name);
        assert!(
            workbench
                .begin_document_load(Replacement::Open, || None)
                .is_err()
        );
        release.send(()).unwrap();
        finish_open(&mut workbench);
        assert_eq!(workbench.project(), &project);
        assert_eq!(workbench.path, Some(path));
        assert!(!workbench.editor.dirty());
    }

    #[test]
    fn document_open_cancellation_errors_and_stale_results_preserve_edits() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("network.json");
        let invalid = directory.path().join("invalid.json");
        let mut workbench = Workbench::new().unwrap();
        sns_project::save_project(&path, workbench.project()).unwrap();
        std::fs::write(&invalid, "{").unwrap();
        workbench
            .editor
            .apply(
                0,
                Edit::Add {
                    kind: DeviceKind::Workstation,
                    x: 10.,
                    y: 10.,
                },
            )
            .unwrap();
        let edited = workbench.project().clone();
        workbench
            .begin_document_load(Replacement::Open, || None)
            .unwrap();
        finish_open(&mut workbench);
        assert!(workbench.error.is_none());
        assert_eq!(workbench.project(), &edited);
        workbench
            .begin_document_load(Replacement::Open, || Some(invalid))
            .unwrap();
        finish_open(&mut workbench);
        assert!(workbench.error.is_some());
        assert_eq!(workbench.project(), &edited);

        workbench
            .begin_document_load(Replacement::Open, || Some(path))
            .unwrap();
        workbench.editor.undo();
        let newer = workbench.project().clone();
        finish_open(&mut workbench);
        assert_eq!(workbench.project(), &newer);
        assert!(workbench.path.is_none(), "stale open must not set the path");
    }

    #[test]
    fn teacher_mission_is_validated_off_frame_installed_and_started_in_the_class_section() {
        let directory = tempfile::tempdir().unwrap();
        let mut mission = sns_missions::builtin().remove(1);
        mission.id = "class-gateway".into();
        mission
            .title
            .insert("fr".into(), "Mission de la classe".into());
        let path = directory.path().join("mission.json");
        std::fs::write(&path, serde_json::to_vec(&mission).unwrap()).unwrap();
        let mut workbench = Workbench::new().unwrap();
        let builtin = workbench.missions().len();
        let selected = path.clone();
        workbench
            .begin_document_load(Replacement::OpenMission, move || Some(selected))
            .unwrap();
        finish_open(&mut workbench);
        assert!(workbench.error.is_none(), "{:?}", workbench.error);
        assert_eq!(workbench.project(), &mission.project);
        assert!(
            workbench.path.is_none(),
            "a mission is not the saved document"
        );
        let ids = workbench.missions();
        assert_eq!(ids.len(), builtin + 1);
        assert_eq!(ids[builtin], "class-gateway");
        assert!(workbench.unlocked(builtin));
        let report = workbench.mission_report().unwrap();
        assert_eq!(report.mission, "class-gateway");
        assert!(!report.complete);
        // Reopening the same mission refreshes it instead of duplicating it,
        // once the restarted engine has acknowledged the previous document.
        workbench.wait_for_worker().unwrap();
        let selected = path.clone();
        workbench
            .begin_document_load(Replacement::OpenMission, move || Some(selected))
            .unwrap();
        finish_open(&mut workbench);
        assert_eq!(workbench.missions().len(), builtin + 1);
    }

    #[test]
    fn invalid_or_stale_teacher_missions_preserve_the_current_work() {
        let directory = tempfile::tempdir().unwrap();
        let invalid = directory.path().join("invalid.json");
        std::fs::write(&invalid, r#"{"mission_version":2}"#).unwrap();
        let mut workbench = Workbench::new().unwrap();
        let original = workbench.project().clone();
        workbench
            .begin_document_load(Replacement::OpenMission, move || Some(invalid))
            .unwrap();
        finish_open(&mut workbench);
        assert!(workbench.error.is_some());
        assert_eq!(workbench.project(), &original);
        assert!(workbench.mission_report().is_none());

        let valid = directory.path().join("valid.json");
        std::fs::write(
            &valid,
            serde_json::to_vec(&sns_missions::builtin()[0]).unwrap(),
        )
        .unwrap();
        workbench.error = None;
        workbench
            .begin_document_load(Replacement::OpenMission, move || Some(valid))
            .unwrap();
        // An edit made while the file is read makes the result stale.
        workbench
            .editor
            .apply(
                workbench.editor.revision(),
                Edit::Add {
                    kind: DeviceKind::Workstation,
                    x: 10.,
                    y: 10.,
                },
            )
            .unwrap();
        let edited = workbench.project().clone();
        finish_open(&mut workbench);
        assert_eq!(workbench.project(), &edited);
        assert!(workbench.mission_report().is_none());
    }

    #[test]
    fn document_open_dispatches_filius_and_scenario_to_their_existing_workers() {
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
        let mut imported = Workbench::new().unwrap();
        let legacy = fixtures.join("legacy/lan.fls");
        imported
            .begin_document_load(Replacement::ImportFilius, || Some(legacy))
            .unwrap();
        finish_open(&mut imported);
        assert!(imported.import_ui.receiver.is_some());

        let mut scenario = Workbench::new().unwrap();
        let file = fixtures.join("valid/network-scenario.json");
        scenario
            .begin_document_load(Replacement::LoadScenario, || Some(file))
            .unwrap();
        finish_open(&mut scenario);
        assert!(scenario.runtime_busy());

        let context = egui::Context::default();
        let deadline = Instant::now() + Duration::from_secs(5);
        while imported.import_report().is_none() || scenario.runtime_busy() {
            assert!(
                Instant::now() < deadline,
                "import or scenario did not finish"
            );
            imported.poll_import(&context);
            scenario.poll_runtime(&context);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(imported.editor.dirty());
        assert!(imported.path.is_none());
        assert!(matches!(scenario.mode, Mode::Simulation));
        assert!(scenario.error.is_none());
    }
}
