use sns_engine::{Engine, Limits, TraceEvent};
use sns_model::*;
use std::{collections::BTreeMap, sync::mpsc, time::Duration};

pub enum Job {
    Step,
    Run,
    Reset,
    Stress,
    Http {
        method: HttpMethod,
        path: String,
        fields: BTreeMap<String, String>,
    },
}
pub struct HttpResult {
    pub path: String,
    pub status: u16,
    pub media_type: String,
    pub body_base64: String,
}
pub struct View {
    pub project: Project,
    pub trace: Vec<TraceEvent>,
    pub now: u64,
    pub pending: usize,
    pub trace_hash: String,
    pub results: Vec<HttpResult>,
    pub error: Option<String>,
}
pub struct Controller {
    sender: mpsc::SyncSender<(Job, Option<egui::Context>)>,
    receiver: mpsc::Receiver<View>,
    pub view: View,
    pub in_flight: usize,
}
impl Controller {
    pub fn new() -> Result<Self, String> {
        let scenario = sns_project::parse_scenario(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/valid/scenario.json"
        )))
        .map_err(|e| e.to_string())?;
        let engine =
            Engine::new(scenario.clone(), None, Limits::default()).map_err(|e| e.to_string())?;
        let view = snapshot(&engine, Vec::new(), None);
        let (sender, jobs) = mpsc::sync_channel::<(Job, Option<egui::Context>)>(8);
        let (responses, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("sinesi-engine".into())
            .spawn(move || {
                let mut engine = engine;
                let mut command_sequence = 0_u64;
                let mut seen = 0;
                let mut paths = initial_paths(&scenario);
                while let Ok((job, wake)) = jobs.recv() {
                    let result = (|| -> Result<(), Diagnostic> {
                        match job {
                            Job::Step => {
                                engine.step()?;
                            }
                            Job::Run => {
                                engine.run(65_536, None)?;
                            }
                            Job::Reset => {
                                engine.restart()?;
                                seen = 0;
                                paths = initial_paths(&scenario);
                            }
                            Job::Stress => {
                                let mut stress = scenario.clone();
                                stress.actions = (0..5_000)
                                    .map(|index| ScheduledMessage {
                                        at_us: Decimal(0),
                                        source: stress.project.devices[0].id.clone(),
                                        target: stress.project.devices[1].id.clone(),
                                        message: Message::Signal {
                                            text: format!("Échange de charge {index}"),
                                        },
                                    })
                                    .collect();
                                engine = Engine::new(stress, None, Limits::default())?;
                                seen = 0;
                                paths.clear();
                                engine.run(20_000, None)?;
                            }
                            Job::Http {
                                method,
                                path,
                                fields,
                            } => {
                                command_sequence += 1;
                                let id = Id::new(format!("ui-{command_sequence}"))?;
                                engine.command(CommandEnvelope {
                                    api_version: 1,
                                    command_id: id.clone(),
                                    project_id: engine.project().project_id.clone(),
                                    expected_revision: Decimal(0),
                                    run_id: engine.run_id().clone(),
                                    command: Command::Send {
                                        source: Id::new("client")?,
                                        target: Id::new("server")?,
                                        message: Message::HttpRequest {
                                            method,
                                            path: path.clone(),
                                            fields,
                                        },
                                    },
                                })?;
                                paths.insert(id, path);
                                engine.run(65_536, None)?;
                            }
                        }
                        Ok(())
                    })();
                    let mut results = Vec::new();
                    for event in &engine.trace()[seen.min(engine.trace().len())..] {
                        if event.kind == "prototype.message.delivered"
                            && event.device_id.as_str() == "client"
                            && let Some(path) = paths.get(&event.correlation_id)
                            && let Ok(Message::HttpResponse {
                                status,
                                media_type,
                                body_base64,
                            }) = serde_message(event)
                        {
                            results.push(HttpResult {
                                path: path.clone(),
                                status,
                                media_type,
                                body_base64,
                            });
                        }
                    }
                    seen = engine.trace().len();
                    if responses
                        .send(snapshot(
                            &engine,
                            results,
                            result.err().map(|e| e.to_string()),
                        ))
                        .is_err()
                    {
                        break;
                    }
                    if let Some(context) = wake {
                        context.request_repaint();
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            sender,
            receiver,
            view,
            in_flight: 0,
        })
    }
    pub fn submit(&mut self, job: Job, context: Option<egui::Context>) {
        match self.sender.try_send((job, context)) {
            Ok(()) => self.in_flight += 1,
            Err(error) => self.view.error = Some(format!("Commande non acceptée : {error}")),
        }
    }
    pub fn poll(&mut self) {
        let mut results = std::mem::take(&mut self.view.results);
        while let Ok(mut view) = self.receiver.try_recv() {
            self.in_flight = self.in_flight.saturating_sub(1);
            results.append(&mut view.results);
            self.view = view;
        }
        self.view.results = results;
    }
    /// Synchronizes tests or a caller explicitly waiting for a submitted job; not used by UI frames.
    pub fn wait(&mut self) -> Result<(), String> {
        let mut results = std::mem::take(&mut self.view.results);
        while self.in_flight > 0 {
            let mut view = self
                .receiver
                .recv_timeout(Duration::from_secs(10))
                .map_err(|e| e.to_string())?;
            self.in_flight -= 1;
            results.append(&mut view.results);
            self.view = view;
        }
        self.view.results = results;
        Ok(())
    }
}
fn serde_message(event: &TraceEvent) -> Result<Message, serde_json::Error> {
    serde_json::from_value(event.payload["message"].clone())
}
fn initial_paths(scenario: &Scenario) -> BTreeMap<Id, String> {
    scenario
        .actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| {
            if let Message::HttpRequest { path, .. } = &action.message {
                Some((
                    Id::new(format!("flow-{index}")).expect("generated ID"),
                    path.clone(),
                ))
            } else {
                None
            }
        })
        .collect()
}
fn snapshot(engine: &Engine, results: Vec<HttpResult>, error: Option<String>) -> View {
    View {
        project: engine.project().clone(),
        trace: engine.trace().to_vec(),
        now: engine.now(),
        pending: engine.pending(),
        trace_hash: engine.trace_hash(),
        results,
        error,
    }
}
