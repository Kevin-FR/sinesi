//! Product workbench state and coordination; panels own presentation only.
mod annotations;
mod canvas;
mod chrome;
mod commands;
mod configuration_ui;
mod document;
mod export_ui;
mod help_ui;
mod inspector;
mod trace_table;
mod worker;
pub use export_ui::{PdfReceipt, PngReceipt};
mod desktop;
mod import_ui;
mod link_conditions_ui;
mod missions_ui;
mod packets;
mod preferences_ui;
mod presentation_ui;
mod routing_ui;
mod runtime_ui;
mod security_ui;
mod session_ui;
mod story_ui;
mod toolbar;
mod topology_ui;
mod trace_ui;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use sns_i18n::Language;
use sns_model::*;
use sns_settings::{
    diagnostics::{Event as TechnicalEvent, Journal},
    help::Example,
};
use sns_topology::{Edit, Editor, display_label};
use std::{collections::BTreeSet, path::PathBuf};
use worker::{Job, Worker};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Design,
    Simulation,
    Documentation,
}
#[derive(Clone, Copy, PartialEq)]
enum Page {
    Topology,
    Trace,
    Tables,
    Desktop,
}
#[derive(Clone, Copy)]
enum Replacement {
    RecoverSession,
    New,
    Open,
    ImportFilius,
    LoadScenario,
    RestoreCheckpoint,
    ReplayRecording,
    Example(Example),
    Mission(usize),
    OpenMission,
    Close,
}
struct Drag {
    origin: Pos2,
    positions: Vec<Position>,
}
pub struct Workbench {
    document_load: Option<document::DocumentLoad>,
    export_ui: export_ui::ExportUi,
    annotations: annotations::AnnotationUi,
    session_ui: session_ui::SessionUi,
    observation: trace_ui::ObservationUi,
    duplicate_omitted: usize,
    conditions_ui: link_conditions_ui::ConditionsUi,
    runtime_ui: runtime_ui::RuntimeUi,
    import_ui: import_ui::ImportUi,
    desktop: desktop::DesktopState,
    synced_revision: u64,
    art: crate::network_art::NetworkArt,
    routing_ui: routing_ui::RoutingUi,
    security_ui: security_ui::SecurityUi,
    configuration_ui: configuration_ui::ConfigurationUi,
    journal: Journal,
    log_path: Option<PathBuf>,
    simulation_error_visible: bool,
    preferences_ui: preferences_ui::PreferencesUi,
    language: Language,
    settings_path: Option<PathBuf>,
    settings_writable: bool,
    settings_error: Option<String>,
    editor: Editor,
    worker: Worker,
    pub mode: Mode,
    page: Page,
    selected: BTreeSet<Id>,
    draft: Option<(Device, LanConfig)>,
    cable: Option<Id>,
    cabling: bool,
    cable_notice: Option<String>,
    packets: packets::Packets,
    missions: missions_ui::MissionsUi,
    missions_feedback_text: Option<String>,
    zoom: f32,
    pan: Vec2,
    drag: Option<Drag>,
    preview: Vec<Position>,
    rectangle: Option<Pos2>,
    target: String,
    ttl: u8,
    speed: u32,
    launch_wait_factor: WaitFactor,
    manual_loss: bool,
    path: Option<PathBuf>,
    error: Option<String>,
    traffic: crate::traffic::TrafficFilter,
    observed_trace: usize,
    active_links: std::collections::BTreeMap<Id, f64>,
    filter: String,
    event_index: Option<usize>,
    help_ui: help_ui::HelpUi,
    annotation: String,
    pending: Option<Replacement>,
    fit: bool,
    close_confirmed: bool,
}
impl Workbench {
    pub fn new() -> Result<Self, String> {
        Self::new_with_wait_factor(WaitFactor::default())
    }
    fn new_with_wait_factor(wait_factor: WaitFactor) -> Result<Self, String> {
        let project = sns_project::parse_project(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/valid/lan.json"
        )))
        .map_err(|e| e.to_string())?;
        Self::from_project_with_wait_factor(project, wait_factor)
    }
    pub fn from_project(project: Project) -> Result<Self, String> {
        Self::from_project_with_wait_factor(project, WaitFactor::default())
    }
    fn from_project_with_wait_factor(
        project: Project,
        wait_factor: WaitFactor,
    ) -> Result<Self, String> {
        Ok(Self {
            document_load: None,
            export_ui: Default::default(),
            annotations: Default::default(),
            session_ui: Default::default(),
            observation: Default::default(),
            duplicate_omitted: 0,
            conditions_ui: Default::default(),
            runtime_ui: Default::default(),
            import_ui: Default::default(),
            desktop: Default::default(),
            synced_revision: 0,
            art: Default::default(),
            routing_ui: Default::default(),
            security_ui: Default::default(),
            configuration_ui: Default::default(),
            journal: Journal::default(),
            log_path: None,
            simulation_error_visible: false,
            preferences_ui: Default::default(),
            language: Language::Fr,
            settings_path: None,
            settings_writable: true,
            settings_error: None,
            editor: Editor::new(project.clone()).map_err(|e| e.to_string())?,
            worker: Worker::new(project, wait_factor)?,
            launch_wait_factor: wait_factor,
            mode: Mode::Design,
            page: Page::Topology,
            selected: BTreeSet::new(),
            draft: None,
            cable: None,
            cabling: false,
            cable_notice: None,
            packets: Default::default(),
            missions: Default::default(),
            missions_feedback_text: None,
            zoom: 1.,
            pan: Vec2::ZERO,
            drag: None,
            preview: vec![],
            rectangle: None,
            target: "192.168.1.3".into(),
            ttl: 64,
            speed: 1,
            manual_loss: false,
            path: None,
            error: None,
            traffic: Default::default(),
            observed_trace: 0,
            active_links: Default::default(),
            filter: String::new(),
            event_index: None,
            help_ui: Default::default(),
            annotation: String::new(),
            pending: None,
            fit: true,
            close_confirmed: false,
        })
    }
    pub fn native() -> Result<Self, String> {
        Self::native_with_project(None)
    }
    pub fn native_with_project(project: Option<PathBuf>) -> Result<Self, String> {
        Self::with_configuration(
            sns_settings::configuration::resolve(sns_settings::configuration::Sources::native()),
            sns_settings::default_session_directory(),
            project,
        )
    }
    pub fn with_preferences(path: Option<PathBuf>) -> Result<Self, String> {
        Self::with_storage(path, None, None)
    }

    pub fn language(&self) -> Language {
        self.language
    }
    pub fn choose_language(&mut self, language: Language) {
        self.language = language;
        self.preferences_ui.persistent.language = language;
        self.queue_preferences();
    }

    fn device_label(&self, device: &Device) -> String {
        if self.mode == Mode::Simulation
            && self.worker.current()
            && let Some(effective) = self.worker.view.effective.device(&device.id)
        {
            return display_label(&self.worker.view.effective, effective);
        }
        display_label(self.project(), device)
    }
    pub fn project(&self) -> &Project {
        self.editor.project()
    }
    pub fn selected(&self) -> &BTreeSet<Id> {
        &self.selected
    }
    pub fn trace(&self) -> &[sns_engine::TraceEvent] {
        &self.worker.view.trace
    }
    pub fn run_id(&self) -> &Id {
        &self.worker.view.run_id
    }
    pub fn now(&self) -> u64 {
        self.worker.view.now
    }
    pub fn attach_diagnostics(&mut self, journal: Journal, log_path: Option<PathBuf>) {
        self.journal = journal;
        self.log_path = log_path;
    }
    pub fn runtime_wait_factor(&self) -> WaitFactor {
        self.worker.view.wait_factor
    }
    pub fn wait_for_worker(&mut self) -> Result<(), String> {
        self.worker.wait()?;
        self.sync_applications();
        Ok(())
    }
    fn sync_applications(&mut self) {
        if self.worker.current() && self.worker.view.revision > self.synced_revision {
            let start = if self.worker.view.trace.len() < self.observed_trace {
                0
            } else {
                self.observed_trace
            };
            let presentation_devices = self
                .worker
                .view
                .trace
                .iter()
                .skip(start)
                .filter(|e| e.kind == "presentation.symbol.changed")
                .map(|e| e.device_id.clone())
                .collect();
            match self
                .editor
                .accept_runtime((*self.worker.view.project).clone(), &presentation_devices)
            {
                Ok(()) => {
                    self.synced_revision = self.worker.view.revision;
                    if let Some((d, _)) = &self.draft
                        && let Some(updated) = self.editor.project().device(&d.id)
                    {
                        self.draft = Some((
                            updated.clone(),
                            self.editor
                                .project()
                                .lan
                                .get(&d.id)
                                .cloned()
                                .unwrap_or_default(),
                        ));
                    }
                }
                Err(e) => self.error = Some(e.to_string()),
            }
        }
    }
    fn apply(&mut self, edit: Edit) {
        if self.mode == Mode::Simulation {
            self.error = Some("state.editing_locked".into());
            return;
        }
        if self.mode == Mode::Documentation
            && !match &edit {
                Edit::Annotate(_) | Edit::AddAnnotation { .. } => true,
                Edit::Delete(ids) => ids
                    .iter()
                    .all(|id| self.project().annotations.iter().any(|a| &a.id == id)),
                _ => false,
            }
        {
            self.error = Some("state.editing_locked".into());
            return;
        }
        if self.worker.pending_commands() {
            self.error = Some("state.pending_commands".into());
            return;
        }
        match self.editor.apply(self.editor.revision(), edit) {
            Ok(outcome) => {
                self.duplicate_omitted = outcome.omitted_links.len();
                if !outcome.created.is_empty() {
                    self.selected = outcome.created;
                    self.refresh_draft();
                    self.fit = true;
                }
                self.error = None;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
    fn select(&mut self, id: Id, extend: bool) {
        self.annotations.draft = self
            .project()
            .annotations
            .iter()
            .find(|a| a.id == id)
            .cloned();
        self.routing_ui = Default::default();
        if !extend {
            self.selected.clear();
        }
        if extend && self.selected.contains(&id) {
            self.selected.remove(&id);
        } else {
            self.selected.insert(id.clone());
        }
        self.draft = self.project().device(&id).map(|d| {
            (
                d.clone(),
                self.project().lan.get(&id).cloned().unwrap_or_default(),
            )
        });
    }
    fn set_mode(&mut self, mode: Mode) {
        if self.mode == mode {
            return;
        }
        if mode == Mode::Simulation {
            self.worker
                .restart(self.project().clone(), self.launch_wait_factor);
            self.synced_revision = 0;
            self.manual_loss = false;
            self.traffic.after = 0;
            self.observed_trace = 0;
            self.active_links.clear();
        } else if self.mode == Mode::Simulation {
            self.worker.submit(Job::Stop);
        }
        self.mode = mode;
        self.cable = None;
        self.cabling = false;
        self.preview.clear();
        self.drag = None;
        self.annotations.cancel_gesture();
    }
}
fn kind_label(language: Language, kind: DeviceKind) -> &'static str {
    language.text(match kind {
        DeviceKind::Workstation => "workstation",
        DeviceKind::Notebook => "notebook",
        DeviceKind::Switch => "switch",
        DeviceKind::Router => "router",
        DeviceKind::Gateway => "gateway_device",
        DeviceKind::Modem => "modem",
    })
}

#[cfg(test)]
mod measurements {
    use super::*;
    mod load {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/lan_load.rs"
        ));
    }
    #[test]
    #[ignore = "reference capacity measurement; run explicitly with --nocapture"]
    fn reference_load_100_nodes_150_links_10000_events() {
        let project = load::project();
        let mut e = sns_engine::Engine::from_project(
            project.clone(),
            42,
            sns_engine::Limits {
                trace: 250_000,
                ..Default::default()
            },
        )
        .unwrap();
        for n in 0..40 {
            e.command(CommandEnvelope {
                api_version: 1,
                command_id: Id::new(format!("bench-{n}")).unwrap(),
                project_id: project.project_id.clone(),
                expected_revision: Decimal(0),
                run_id: e.run_id().clone(),
                command: Command::Ping {
                    source: Id::new(format!("node-{n}")).unwrap(),
                    destination: format!("192.168.1.{}", (n + 1) % 50 + 2),
                    ttl: 64,
                },
            })
            .unwrap();
        }
        let start = std::time::Instant::now();
        let (processed, termination) = e.pump(10_000, None).unwrap();
        let engine_ms = start.elapsed().as_secs_f64() * 1000.;
        assert_eq!(processed, 10_000);
        let trace = e.trace().to_vec();
        let trace_len = trace.len();
        let mut state = Workbench::new().unwrap();
        state.editor = Editor::new(project).unwrap();
        state.worker.view.trace = std::sync::Arc::new(trace);
        let mut h = egui_kittest::Harness::builder()
            .with_size((1320., 880.))
            .build_ui_state(|ui, state: &mut Workbench| state.show(ui), state);
        h.run();
        let mut topology = Vec::new();
        for _ in 0..100 {
            let now = std::time::Instant::now();
            h.step();
            topology.push(now.elapsed().as_secs_f64() * 1000.);
        }
        h.state_mut().page = Page::Trace;
        h.run();
        let mut table = Vec::new();
        for _ in 0..100 {
            let now = std::time::Instant::now();
            h.step();
            table.push(now.elapsed().as_secs_f64() * 1000.);
        }
        topology.sort_by(f64::total_cmp);
        table.sort_by(f64::total_cmp);
        println!(
            "{}",
            serde_json::json!({"devices":100,"links":150,"processed_events":processed,"trace_facts":trace_len,"termination":termination,"engine_ms":engine_ms,"ui_topology_p95_ms":topology[94],"ui_trace_p95_ms":table[94],"ui_method":"egui CPU layout + accessibility, 1320x880, 100 frames; excludes GPU/presentation"})
        );
    }
}
