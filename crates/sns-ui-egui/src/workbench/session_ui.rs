//! Local recovery I/O and proposals. Restoring editing data never dispatches a command.
use super::*;
use sns_project::session::{
    self, Recovery, SessionLease, SessionRecord, SessionSummary, SessionView,
};
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
const INTERVAL: Duration = Duration::from_secs(2);
use sns_project::destination::{resolve_destination, same_destination};

type Listings = Result<(Vec<SessionSummary>, Vec<String>), String>;
enum Outcome {
    Initialized {
        lease: Option<SessionLease>,
        listings: Listings,
        error: Option<String>,
    },
    Listed {
        lease: Option<SessionLease>,
        listings: Listings,
    },
    Saved {
        lease: SessionLease,
        record: Arc<SessionRecord>,
        retire: Option<SessionLease>,
        result: Result<(), String>,
    },
    Loaded {
        lease: Option<SessionLease>,
        result: Result<Box<(Recovery, EditingState)>, String>,
    },
    Deleted {
        lease: Option<SessionLease>,
        result: Result<(), String>,
    },
    Closed(Result<(), String>),
}
pub(super) struct SessionUi {
    root: Option<PathBuf>,
    lease: Option<SessionLease>,
    task: Option<mpsc::Receiver<Outcome>>,
    candidates: Vec<SessionSummary>,
    diagnostics: Vec<String>,
    open: bool,
    prompt: bool,
    selected: Option<Id>,
    pub requested: Option<Id>,
    saved: Option<Arc<SessionRecord>>,
    retire: Option<SessionLease>,
    next: Instant,
    pub can_save: bool,
    pub discard_on_close: bool,
    pub close_requested: bool,
    pub loading: bool,
    closed: bool,
    error: Option<String>,
}
impl Default for SessionUi {
    fn default() -> Self {
        Self {
            root: None,
            lease: None,
            task: None,
            candidates: Vec::new(),
            diagnostics: Vec::new(),
            open: false,
            prompt: false,
            selected: None,
            requested: None,
            saved: None,
            retire: None,
            next: Instant::now() + INTERVAL,
            can_save: false,
            discard_on_close: false,
            close_requested: false,
            loading: false,
            closed: false,
            error: None,
        }
    }
}
impl SessionUi {
    fn start(&mut self, job: impl FnOnce() -> Outcome + Send + 'static) {
        let (tx, rx) = mpsc::sync_channel(1);
        match std::thread::Builder::new()
            .name("session-storage".into())
            .spawn(move || {
                let _ = tx.send(job());
            }) {
            Ok(_) => self.task = Some(rx),
            Err(e) => {
                self.error = Some(e.to_string());
                self.loading = false;
            }
        }
    }
    fn initialize(&mut self) {
        let Some(root) = self.root.clone() else {
            return;
        };
        self.start(move || {
            let listings = session::summaries(&root).map_err(|e| e.to_string());
            match SessionLease::create(&root) {
                Ok(lease) => Outcome::Initialized {
                    lease: Some(lease),
                    listings,
                    error: None,
                },
                Err(e) => Outcome::Initialized {
                    lease: None,
                    listings,
                    error: Some(e.to_string()),
                },
            }
        });
    }
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EditingState {
    version: u32,
    inspector: Option<(Device, LanConfig)>,
    desktop: serde_json::Value,
    routing: routing_ui::RoutingUi,
    security: security_ui::SecurityUi,
    conditions: link_conditions_ui::ConditionsUi,
    annotation: String,
    #[serde(default)]
    annotation_draft: Option<Annotation>,
    target: String,
    ttl: u8,
}
impl EditingState {
    fn read(record: &SessionRecord) -> Result<Self, String> {
        let Some(value) = record.edit_state.clone() else {
            return Ok(Self {
                version: 1,
                inspector: None,
                desktop: desktop::DesktopState::default().recovery_value(),
                routing: Default::default(),
                security: Default::default(),
                conditions: Default::default(),
                annotation: String::new(),
                annotation_draft: None,
                target: "192.168.1.3".into(),
                ttl: 64,
            });
        };
        let mut state: Self =
            serde_json::from_value(value).map_err(|_| "session.invalid_document".to_owned())?;
        if state.version != 1
            || state.annotation.len() > 4096
            || state.target.len() > 4096
            || !state.security.recovery_valid(&record.project)
        {
            return Err("session.invalid_document".into());
        }
        if let Some((d, _)) = &state.inspector {
            let original = record
                .project
                .device(&d.id)
                .ok_or("session.invalid_document")?;
            if d.kind != original.kind
                || d.interfaces.iter().map(|i| &i.id).collect::<Vec<_>>()
                    != original
                        .interfaces
                        .iter()
                        .map(|i| &i.id)
                        .collect::<Vec<_>>()
            {
                return Err("session.invalid_document".into());
            }
        }
        if let Some(note) = &state.annotation_draft {
            if !record.project.annotations.iter().any(|a| a.id == note.id) {
                return Err("session.invalid_document".into());
            }
            record
                .project
                .with_annotation(note.clone())
                .map_err(|_| "session.invalid_document".to_owned())?;
        }
        // Validate host/application ownership before any document is replaced.
        let _ = desktop::DesktopState::restore_recovery(state.desktop.clone(), &record.project)?;
        state.conditions.open = false;
        Ok(state)
    }
}
impl Workbench {
    pub(super) fn session_enabled(&self) -> bool {
        self.session_ui.root.is_some()
    }
    pub fn with_storage(
        preferences: Option<PathBuf>,
        root: Option<PathBuf>,
        project: Option<PathBuf>,
    ) -> Result<Self, String> {
        Self::with_configuration(
            sns_settings::configuration::resolve(sns_settings::configuration::Sources {
                user: preferences,
                ..Default::default()
            }),
            root,
            project,
        )
    }
    pub fn with_configuration(
        configuration: sns_settings::configuration::ResolvedConfiguration,
        root: Option<PathBuf>,
        project: Option<PathBuf>,
    ) -> Result<Self, String> {
        Self::with_configuration_and_wait_factor(
            configuration,
            root,
            project,
            WaitFactor::default(),
        )
    }
    pub fn with_configuration_and_wait_factor(
        configuration: sns_settings::configuration::ResolvedConfiguration,
        root: Option<PathBuf>,
        project: Option<PathBuf>,
        wait_factor: WaitFactor,
    ) -> Result<Self, String> {
        if configuration.effective.settings_version != 2
            || configuration.persistent.settings_version != 2
        {
            return Err("settings.unsupported_version".into());
        }
        configuration
            .effective
            .display
            .validate()
            .map_err(str::to_owned)?;
        configuration
            .persistent
            .display
            .validate()
            .map_err(str::to_owned)?;
        let preferences = configuration.user_path.clone();
        let mut app = if let Some(path) = &project {
            Self::from_project_with_wait_factor(
                sns_project::load_project(path).map_err(|e| e.to_string())?,
                wait_factor,
            )?
        } else {
            Self::new_with_wait_factor(wait_factor)?
        };
        let mut configuration = configuration;
        app.path = project.clone();
        let conflict = preferences
            .as_ref()
            .zip(app.path.as_ref())
            .is_some_and(|(a, b)| same_destination(a, b));
        if conflict {
            configuration.user_writable = false;
            configuration.diagnostics.truncate(31);
            configuration
                .diagnostics
                .push(sns_settings::configuration::ConfigurationDiagnostic {
                    source: sns_settings::configuration::Source::User,
                    field: "configuration".into(),
                    code: "configuration.project_path_conflict",
                });
            app.settings_error = Some("settings.project_path_conflict".into());
            app.settings_path = None;
        } else {
            app.settings_path = preferences;
        }
        app.load_startup_configuration(configuration);
        app.session_ui.root = root;
        app.session_ui.can_save = project.is_some();
        app.session_ui.prompt = project.is_none();
        app.session_ui.initialize();
        Ok(app)
    }
    pub fn session_busy(&self) -> bool {
        self.session_ui.task.is_some()
    }
    pub fn session_error(&self) -> Option<&str> {
        self.session_ui.error.as_deref()
    }
    pub fn request_autosave(&mut self) {
        self.session_ui.can_save = true;
        self.session_ui.next = Instant::now();
    }
    pub fn recovery_count(&self) -> usize {
        self.session_ui.candidates.len()
    }
    pub fn session_saved(&self) -> bool {
        self.session_ui.saved.is_some() && !self.session_busy() && self.session_ui.error.is_none()
    }
    pub fn recovery_view(&self) -> SessionView {
        SessionView {
            diagram: self.art.diagram,
            zoom: self.zoom,
            pan: [self.pan.x, self.pan.y],
            selected: self.selected.iter().cloned().collect(),
        }
    }
    pub fn project_path(&self) -> Option<&std::path::Path> {
        self.path.as_deref()
    }
    pub(super) fn session_output_allowed(&self, path: &std::path::Path) -> Result<(), String> {
        let Some(root) = &self.session_ui.root else {
            return Ok(());
        };
        let root = resolve_destination(root)?;
        let target = resolve_destination(path)?;
        if target.starts_with(root) {
            return Err("session.protected_path".into());
        }
        Ok(())
    }
    pub(super) fn preferences_output_allowed(&self, path: &std::path::Path) -> Result<(), String> {
        let target = resolve_destination(path)?;
        self.configuration_output_allowed(path)?;
        if self
            .path
            .as_ref()
            .is_some_and(|project| resolve_destination(project).ok().as_ref() == Some(&target))
        {
            return Err("settings.project_path_conflict".into());
        }
        self.session_output_allowed(path)
    }
    pub(super) fn document_output_allowed(&self, path: &std::path::Path) -> Result<(), String> {
        self.configuration_output_allowed(path)?;
        if self.export_destination_matches(path) {
            return Err("render.path_in_use".into());
        }
        self.session_output_allowed(path)?;
        if self
            .settings_path
            .as_ref()
            .is_some_and(|prefs| same_destination(prefs, path))
        {
            return Err("settings.project_path_conflict".into());
        }
        Ok(())
    }
    pub(super) fn export_output_allowed(&self, path: &std::path::Path) -> Result<(), String> {
        self.document_output_allowed(path)?;
        if self
            .path
            .as_ref()
            .is_some_and(|project| same_destination(project, path))
        {
            return Err("runtime.project_path_conflict".into());
        }
        Ok(())
    }
    fn editing_value(&self) -> serde_json::Value {
        serde_json::json!({"version":1,"inspector":self.draft,"desktop":self.desktop.recovery_value(),"routing":self.routing_ui,"security":self.security_ui,"conditions":self.conditions_ui,"annotation":self.annotation,"annotation_draft":self.annotations.draft,"target":self.target,"ttl":self.ttl})
    }
    fn session_snapshot(&self) -> Result<Option<Arc<SessionRecord>>, String> {
        let Some(lease) = &self.session_ui.lease else {
            return Ok(None);
        };
        let edit_state = self.editing_value();
        if serde_json::to_vec(&edit_state)
            .map_err(|e| e.to_string())?
            .len()
            > session::MAX_EDIT_STATE_BYTES
        {
            return Err("resource.limit_exceeded: session_edit_state".into());
        }
        let view = SessionView {
            diagram: self.art.diagram,
            zoom: self.zoom,
            pan: [self.pan.x, self.pan.y],
            selected: self.selected.iter().cloned().collect(),
        };
        let origin = self.path.as_ref().map(|p| p.display().to_string());
        if self.session_ui.saved.as_ref().is_some_and(|old| {
            old.project == *self.project()
                && old.view == view
                && old.edit_state.as_ref() == Some(&edit_state)
                && old.origin_display == origin
        }) {
            return Ok(None);
        }
        Ok(Some(Arc::new(SessionRecord {
            session_version: 1,
            session_id: lease.id().clone(),
            saved_at_unix_ms: Decimal(session::unix_time_ms()),
            revision: Decimal(self.editor.revision()),
            origin_display: origin,
            project: self.project().clone(),
            view,
            edit_state: Some(edit_state),
        })))
    }
    fn publish_session(&mut self) -> bool {
        let record = match self.session_snapshot() {
            Ok(Some(record)) => record,
            Ok(None) => return true,
            Err(e) => {
                self.session_ui.error = Some(e);
                return false;
            }
        };
        let Some(lease) = self.session_ui.lease.take() else {
            return false;
        };
        let mut retire = self.session_ui.retire.take();
        self.session_ui.start(move || {
            let mut result = EditingState::read(&record)
                .and_then(|_| lease.save(&record).map_err(|e| e.to_string()));
            if result.is_ok()
                && let Some(old) = retire.take()
            {
                result = old.discard().map_err(|e| e.to_string());
            }
            Outcome::Saved {
                lease,
                record,
                retire,
                result,
            }
        });
        false
    }
    pub(super) fn poll_session(&mut self, ctx: &egui::Context) {
        let result = self
            .session_ui
            .task
            .as_ref()
            .and_then(|rx| match rx.try_recv() {
                Ok(o) => Some(Ok(o)),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(_) => Some(Err("runtime.worker_failed".to_owned())),
            });
        if let Some(outcome) = result {
            self.session_ui.task = None;
            match outcome {
                Err(e) => {
                    self.session_ui.error = Some(e);
                    self.session_ui.loading = false;
                }
                Ok(Outcome::Initialized {
                    lease,
                    listings,
                    error,
                }) => {
                    self.session_ui.lease = lease;
                    self.session_ui.error = error;
                    self.accept_session_list(listings);
                    if self.session_ui.prompt && !self.session_ui.candidates.is_empty() {
                        self.session_ui.open = true;
                    }
                    self.session_ui.prompt = false;
                }
                Ok(Outcome::Listed { lease, listings }) => {
                    self.session_ui.lease = lease;
                    self.accept_session_list(listings);
                }
                Ok(Outcome::Saved {
                    lease,
                    record,
                    retire,
                    result,
                }) => {
                    self.session_ui.lease = Some(lease);
                    self.session_ui.retire = retire;
                    match result {
                        Ok(()) => {
                            self.session_ui.saved = Some(record);
                            self.session_ui.error = None;
                        }
                        Err(e) => {
                            self.session_ui.error = Some(e);
                            if self.close_confirmed {
                                self.close_confirmed = false;
                                self.pending = Some(Replacement::Close);
                            }
                        }
                    }
                }
                Ok(Outcome::Loaded { lease, result }) => {
                    self.session_ui.loading = false;
                    self.session_ui.lease = lease;
                    match result {
                        Err(e) => self.session_ui.error = Some(e),
                        Ok(pair) => {
                            let (recovery, editing) = *pair;
                            if let Err(e) = self.adopt_session(recovery, editing) {
                                self.session_ui.error = Some(e);
                            }
                        }
                    }
                }
                Ok(Outcome::Deleted { lease, result }) => {
                    self.session_ui.lease = lease;
                    if let Err(e) = result {
                        self.session_ui.error = Some(e);
                    }
                    if self.session_ui.lease.is_some() {
                        self.scan_sessions();
                    } else {
                        self.session_ui.initialize();
                    }
                }
                Ok(Outcome::Closed(result)) => {
                    self.session_ui.closed = true;
                    if let Err(e) = result {
                        self.session_ui.error = Some(e);
                    }
                }
            }
            ctx.request_repaint();
        }
        if self.session_busy() {
            ctx.request_repaint_after(Duration::from_millis(30));
        }
    }
    fn accept_session_list(&mut self, listings: Listings) {
        match listings {
            Ok((entries, diagnostics)) => {
                self.session_ui.selected = entries.first().map(|s| s.id.clone());
                self.session_ui.candidates = entries;
                self.session_ui.diagnostics = diagnostics;
            }
            Err(e) => self.session_ui.error = Some(e),
        }
    }
    fn scan_sessions(&mut self) {
        let lease = self.session_ui.lease.take();
        let Some(root) = self.session_ui.root.clone() else {
            return;
        };
        self.session_ui.start(move || Outcome::Listed {
            lease,
            listings: session::summaries(&root).map_err(|e| e.to_string()),
        });
    }
    pub(super) fn load_requested_session(&mut self) {
        let Some(id) = self.session_ui.requested.take() else {
            return;
        };
        let lease = self.session_ui.lease.take();
        let Some(root) = self.session_ui.root.clone() else {
            return;
        };
        self.session_ui.loading = true;
        self.session_ui.start(move || {
            let result = (|| {
                let recovered = SessionLease::acquire(&root, &id).map_err(|e| e.to_string())?;
                let record = recovered.read().map_err(|e| e.to_string())?;
                let editing = EditingState::read(&record)?;
                Ok(Box::new((
                    Recovery {
                        lease: recovered,
                        record,
                    },
                    editing,
                )))
            })();
            Outcome::Loaded { lease, result }
        });
    }
    fn adopt_session(&mut self, recovery: Recovery, editing: EditingState) -> Result<(), String> {
        let mut editor = Editor::new(recovery.record.project.clone()).map_err(|e| e.to_string())?;
        let desktop =
            desktop::DesktopState::restore_recovery(editing.desktop, &recovery.record.project)?;
        editor.mark_unsaved();
        self.set_mode(Mode::Design);
        self.editor = editor;
        self.desktop = desktop;
        self.worker
            .restart(self.project().clone(), self.launch_wait_factor);
        self.synced_revision = 0;
        self.path = None;
        self.page = Page::Topology;
        self.event_index = None;
        self.filter.clear();
        self.traffic = Default::default();
        self.art.diagram = recovery.record.view.diagram;
        self.zoom = recovery.record.view.zoom;
        self.pan = Vec2::new(recovery.record.view.pan[0], recovery.record.view.pan[1]);
        self.selected = recovery.record.view.selected.iter().cloned().collect();
        self.draft = editing.inspector;
        if self.draft.is_none()
            && let Some(device) = self
                .selected
                .iter()
                .find_map(|id| self.project().device(id))
        {
            self.draft = Some((
                device.clone(),
                self.project()
                    .lan
                    .get(&device.id)
                    .cloned()
                    .unwrap_or_default(),
            ));
        }
        self.routing_ui = editing.routing;
        self.security_ui = editing.security;
        self.conditions_ui = editing.conditions;
        self.annotation = editing.annotation;
        self.annotations.draft = editing.annotation_draft;
        self.target = editing.target;
        self.ttl = editing.ttl;
        self.fit = false;
        self.preferences_ui.follow_canvas_size = false;
        self.error = None;
        self.observation = Default::default();
        self.duplicate_omitted = 0;
        self.observed_trace = 0;
        self.active_links.clear();
        if self.session_ui.lease.is_none() {
            self.session_ui.lease = Some(recovery.lease);
        } else {
            self.session_ui.retire = Some(recovery.lease);
        }
        self.session_ui.open = false;
        self.session_ui.candidates.clear();
        self.session_ui.can_save = true;
        self.session_ui.saved = None;
        self.session_ui.error = None;
        self.session_ui.next = Instant::now();
        Ok(())
    }
    pub(super) fn session_toolbar(&mut self, ui: &mut egui::Ui) {
        if self.session_ui.root.is_none() {
            return;
        }
        let l = self.language;
        if ui
            .add_enabled(
                !self.session_busy(),
                egui::Button::new(l.text("recover_session")),
            )
            .clicked()
        {
            self.session_ui.open = true;
            self.scan_sessions();
        }
        ui.label(l.text(if self.session_busy() {
            "session_saving"
        } else if self.session_saved() {
            "session_saved"
        } else {
            "session_enabled"
        }));
        if let Some(error) = &self.session_ui.error {
            ui.colored_label(Color32::LIGHT_RED, l.describe_error(error));
        }
    }
    pub(super) fn session_window(&mut self, ctx: &egui::Context) {
        if !self.session_ui.open {
            return;
        }
        let l = self.language;
        let mut open = true;
        let mut close = false;
        let mut restore = false;
        let mut discard = false;
        egui::Window::new(l.text("recover_session"))
            .id(egui::Id::unique("session-recovery"))
            .default_pos([430., 210.])
            .default_width(650.)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(l.text("session_recovery_hint"));
                if self.session_busy() {
                    ui.spinner();
                }
                if self.session_ui.candidates.is_empty() && !self.session_busy() {
                    ui.label(l.text("session_none"));
                }
                egui::ScrollArea::vertical()
                    .max_height(320.)
                    .show(ui, |ui| {
                        for entry in &self.session_ui.candidates {
                            ui.push_id(&entry.id, |ui| {
                                if ui
                                    .selectable_label(
                                        self.session_ui.selected.as_ref() == Some(&entry.id),
                                        &entry.name,
                                    )
                                    .clicked()
                                {
                                    self.session_ui.selected = Some(entry.id.clone());
                                }
                                if let Some(origin) = &entry.origin_display {
                                    ui.label(origin);
                                }
                            });
                        }
                    });
                if self
                    .session_ui
                    .diagnostics
                    .iter()
                    .any(|d| d != "session.more_available")
                {
                    ui.label(l.text("session_unreadable"));
                }
                if self
                    .session_ui
                    .diagnostics
                    .iter()
                    .any(|d| d == "session.more_available")
                {
                    ui.label(l.text("session.more_available"));
                }
                ui.horizontal_wrapped(|ui| {
                    let ready = !self.session_busy() && self.session_ui.selected.is_some();
                    restore = ui
                        .add_enabled(ready, egui::Button::new(l.text("session_restore")))
                        .clicked();
                    discard = ui
                        .add_enabled(ready, egui::Button::new(l.text("session_delete")))
                        .clicked();
                    close = ui.button(l.text("session_later")).clicked();
                });
                if let Some(e) = &self.session_ui.error {
                    ui.colored_label(Color32::LIGHT_RED, l.describe_error(e));
                }
            });
        if restore {
            self.session_ui.requested = self.session_ui.selected.clone();
            self.replace(Replacement::RecoverSession);
        }
        if discard
            && let (Some(id), lease, Some(root)) = (
                self.session_ui.selected.clone(),
                self.session_ui.lease.take(),
                self.session_ui.root.clone(),
            )
        {
            self.session_ui.start(move || Outcome::Deleted {
                lease,
                result: SessionLease::acquire(&root, &id)
                    .and_then(SessionLease::discard)
                    .map_err(|e| e.to_string()),
            });
        }
        if close || !open {
            self.session_ui.open = false;
        }
    }
    pub(super) fn tick_session(&mut self, ctx: &egui::Context) {
        if self.session_ui.root.is_none() || self.session_ui.closed {
            return;
        }
        if self.session_busy()
            || self.worker.pending_commands()
            || self.runtime_busy()
            || self.import_ui.receiver.is_some()
        {
            return;
        }
        if self.session_ui.open {
            return;
        }
        self.session_ui.can_save |= self.editor.dirty()
            || self.desktop.dirty()
            || self.draft.is_some()
            || self.annotations.draft.is_some()
            || self.art.diagram;
        if Instant::now() >= self.session_ui.next {
            self.session_ui.next = Instant::now() + INTERVAL;
            if self.session_ui.can_save {
                self.publish_session();
            }
        }
        ctx.request_repaint_after(INTERVAL);
    }
    pub(super) fn session_close_ready(&mut self, ctx: &egui::Context) -> bool {
        if self.document_export_busy() || self.preferences_busy() {
            ctx.request_repaint_after(Duration::from_millis(30));
            return false;
        }
        if self.session_ui.root.is_none() || self.session_ui.closed {
            return true;
        }
        if self.session_busy() || self.worker.pending_commands() {
            ctx.request_repaint_after(Duration::from_millis(30));
            return false;
        }
        if self.session_ui.discard_on_close || !self.session_ui.can_save {
            let Some(lease) = self.session_ui.lease.take() else {
                return true;
            };
            let retire = self.session_ui.retire.take();
            self.session_ui.start(move || {
                Outcome::Closed(lease.discard().map_err(|e| e.to_string()).and_then(|()| {
                    retire
                        .map(SessionLease::discard)
                        .transpose()
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }))
            });
            return false;
        }
        if self.session_ui.lease.is_none() {
            if !self.editor.dirty() && !self.desktop.dirty() {
                return true;
            }
            self.close_confirmed = false;
            self.pending = Some(Replacement::Close);
            return false;
        }
        self.session_ui.error = None;
        let ready = self.publish_session();
        if self.session_ui.error.is_some() {
            self.close_confirmed = false;
            self.pending = Some(Replacement::Close);
        }
        ready
    }
}
