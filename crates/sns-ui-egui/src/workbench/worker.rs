//! Single engine owner. Wall time only determines requested virtual bounds.
use sns_engine::{Engine, Limits, TraceEvent};
use sns_model::*;
use std::{
    sync::{Arc, OnceLock, mpsc},
    time::{Duration, Instant},
};
#[derive(Clone)]
pub struct Snapshot {
    pub wait_factor: WaitFactor,
    pub bridges: std::collections::BTreeMap<Id, sns_bridge::BridgeView>,
    pub epoch: u64,
    pub generation: u64,
    pub seed: u64,
    pub link_conditions: std::collections::BTreeMap<Id, LinkConditions>,
    pub command_serial: u64,
    pub revision: u64,
    pub project: Arc<Project>,
    pub effective: Arc<Project>,
    pub hosts: Arc<Vec<sns_apps::HostView>>,
    pub sockets: Vec<SocketView>,
    pub nat: Vec<sns_protocols::nat::Mapping>,
    pub firewall: std::collections::BTreeMap<Id, Vec<sns_protocols::firewall::Decision>>,
    pub routes: std::collections::BTreeMap<Id, Vec<sns_protocols::routing::RouteView>>,
    pub rip_muted: std::collections::BTreeSet<Id>,
    pub trace: Arc<Vec<TraceEvent>>,
    pub now: u64,
    pub pending: usize,
    pub running: bool,
    pub tables: String,
    pub error: Option<String>,
    pub run_id: Id,
}
pub enum Job {
    Start {
        project: Project,
        epoch: u64,
        wait_factor: WaitFactor,
    },
    Stop,
    BridgeStart {
        device: Id,
        config: ModemConfig,
    },
    BridgeStop {
        device: Id,
    },
    Step,
    Running(bool),
    Speed(u32),
    Command(Command),
    Capture {
        checkpoint: bool,
        reply: mpsc::SyncSender<Result<Vec<u8>, String>>,
    },
    Fence(mpsc::SyncSender<Snapshot>),
}
pub struct Worker {
    pub epoch: u64,
    submitted: u64,
    sender: mpsc::SyncSender<Job>,
    receiver: mpsc::Receiver<Snapshot>,
    pub view: Snapshot,
    wake: Arc<OnceLock<egui::Context>>,
}
impl Worker {
    pub fn new(project: Project, wait_factor: WaitFactor) -> Result<Self, String> {
        let engine =
            Engine::from_project_with_wait_factor(project, 42, Limits::default(), wait_factor)
                .map_err(|e| e.to_string())?;
        Self::from_engine(engine)
    }
    pub(super) fn from_engine(engine: Engine) -> Result<Self, String> {
        let view = snapshot(&engine, false, None, 0, 0, Default::default());
        let (sender, jobs) = mpsc::sync_channel(16);
        // One replaceable projection in transit. A slow GUI never grows the queue.
        let (updates, receiver) = mpsc::sync_channel(1);
        let wake = Arc::new(OnceLock::<egui::Context>::new());
        let worker_wake = wake.clone();
        std::thread::Builder::new()
            .name("sinesi-lan".into())
            .spawn(move || {
                let mut e = engine;
                let mut bridges = sns_bridge::Driver::default();
                let mut epoch = 0;
                let mut running = false;
                let mut speed = 1_u32;
                let mut serial = 0;
                let mut command_identity = 0;
                let mut horizon = 0;
                let mut last = Instant::now();
                let mut unpublished = None;
                let mut last_error = None;
                loop {
                    if let Some(view) = unpublished.take() {
                        match updates.try_send(view) {
                            Ok(()) => {
                                if let Some(ctx) = worker_wake.get() {
                                    ctx.request_repaint();
                                }
                            }
                            Err(mpsc::TrySendError::Full(view)) => unpublished = Some(view),
                            Err(mpsc::TrySendError::Disconnected(_)) => break,
                        }
                    }
                    let job = if running || unpublished.is_some() || bridges.active() {
                        match jobs.recv_timeout(Duration::from_millis(20)) {
                            Ok(j) => Some(j),
                            Err(mpsc::RecvTimeoutError::Timeout) => None,
                            Err(_) => break,
                        }
                    } else {
                        match jobs.recv() {
                            Ok(j) => Some(j),
                            Err(_) => break,
                        }
                    };
                    let fence = matches!(&job, Some(Job::Fence(_)));
                    let mut changed = job.is_some();
                    let result = (|| -> Result<(), Diagnostic> {
                        if let Some(job) = job {
                            match job {
                                Job::Start {
                                    project,
                                    epoch: next_epoch,
                                    wait_factor,
                                } => {
                                    bridges.stop_all(&mut e);
                                    e.replace_project_with_wait_factor(project, wait_factor)?;
                                    bridges = sns_bridge::Driver::default();
                                    epoch = next_epoch;
                                    running = false;
                                    horizon = 0;
                                }
                                Job::BridgeStart { device, config } => {
                                    bridges.start(&mut e, &device, config)?
                                }
                                Job::BridgeStop { device } => bridges.stop(&mut e, &device),
                                Job::Stop => {
                                    bridges.stop_all(&mut e);
                                    running = false;
                                    e.stop()?;
                                }
                                Job::Step => {
                                    running = false;
                                    e.step()?;
                                    horizon = e.now();
                                }
                                Job::Running(value) => {
                                    running = value;
                                    horizon = e.now();
                                }
                                Job::Speed(value) => speed = value.clamp(1, 100),
                                Job::Command(command) => {
                                    serial += 1;
                                    send(&mut e, &mut command_identity, command)?;
                                }
                                Job::Capture { checkpoint, reply } => {
                                    running = false;
                                    horizon = e.now();
                                    let result = if checkpoint {
                                        e.checkpoint().map_err(|d| d.to_string()).and_then(|p| {
                                            serde_json::to_vec(&p).map_err(|d| d.to_string())
                                        })
                                    } else {
                                        e.recording().map_err(|d| d.to_string()).and_then(|r| {
                                            serde_json::to_vec(&r).map_err(|d| d.to_string())
                                        })
                                    };
                                    let _ = reply.send(result);
                                }
                                Job::Fence(reply) => {
                                    let _ = reply.send(snapshot(
                                        &e,
                                        running,
                                        last_error.clone(),
                                        epoch,
                                        serial,
                                        bridges.views(),
                                    ));
                                }
                            }
                            last = Instant::now();
                        } else if running {
                            let elapsed = last.elapsed().as_micros().min(100_000) as u64;
                            last = Instant::now();
                            horizon = horizon.saturating_add(elapsed.saturating_mul(speed as u64));
                            let (processed, _) = e.pump(128, Some(horizon))?;
                            changed = processed > 0;
                            if e.pending() == 0 && !bridges.active() {
                                running = false;
                                changed = true;
                            }
                        }
                        Ok(())
                    })();
                    changed |= bridges.poll(&mut e);
                    let failed = result.is_err();
                    if let Err(error) = result {
                        running = false;
                        last_error = Some(error.to_string());
                    } else if changed && !fence {
                        last_error = None;
                    }
                    if changed || failed {
                        unpublished = Some(snapshot(
                            &e,
                            running,
                            last_error.clone(),
                            epoch,
                            serial,
                            bridges.views(),
                        ));
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            epoch: 0,
            submitted: 0,
            sender,
            receiver,
            view,
            wake,
        })
    }
    pub(super) fn capture(
        &self,
        checkpoint: bool,
        reply: mpsc::SyncSender<Result<Vec<u8>, String>>,
    ) -> Result<(), String> {
        self.sender
            .try_send(Job::Capture { checkpoint, reply })
            .map_err(|_| "command.queue_full".into())
    }
    pub fn restart(&mut self, project: Project, wait_factor: WaitFactor) {
        let epoch = self.epoch + 1;
        match self.sender.try_send(Job::Start {
            project,
            epoch,
            wait_factor,
        }) {
            Ok(()) => self.epoch = epoch,
            Err(e) => self.view.error = Some(format!("command.queue_full:{e}")),
        }
    }
    pub fn pending_commands(&self) -> bool {
        !self.current() || self.submitted > self.view.command_serial
    }
    pub fn current(&self) -> bool {
        self.epoch == self.view.epoch
    }
    pub fn attach_context(&self, context: &egui::Context) {
        let _ = self.wake.set(context.clone());
    }
    pub fn submit(&mut self, job: Job) {
        let command = matches!(&job, Job::Command(_));
        match self.sender.try_send(job) {
            Ok(()) => {
                self.submitted += u64::from(command);
            }
            Err(e) => self.view.error = Some(format!("command.queue_full:{e}")),
        }
    }
    pub fn poll(&mut self) {
        while let Ok(view) = self.receiver.try_recv() {
            self.view = view;
        }
    }
    pub fn wait(&mut self) -> Result<(), String> {
        // Tests fence the worker; the GUI never blocks on this method.
        let (reply, result) = mpsc::sync_channel(1);
        self.sender
            .send(Job::Fence(reply))
            .map_err(|e| e.to_string())?;
        self.view = result
            .recv_timeout(Duration::from_secs(10))
            .map_err(|e| e.to_string())?;
        while self.receiver.try_recv().is_ok() {}
        Ok(())
    }
}
fn send(e: &mut Engine, serial: &mut u64, command: Command) -> Result<(), Diagnostic> {
    let command_id = loop {
        *serial = serial
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "command"))?;
        let candidate = Id::new(format!("lan-ui-{serial}"))?;
        if !e.command_id_taken(&candidate) {
            break candidate;
        }
    };
    e.command(CommandEnvelope {
        api_version: 1,
        command_id,
        project_id: e.project().project_id.clone(),
        expected_revision: Decimal(e.revision()),
        run_id: e.run_id().clone(),
        command,
    })
}
fn snapshot(
    e: &Engine,
    running: bool,
    error: Option<String>,
    epoch: u64,
    command_serial: u64,
    bridges: std::collections::BTreeMap<Id, sns_bridge::BridgeView>,
) -> Snapshot {
    let arp:Vec<_>=e.network().arp.iter().map(|((port,ip),entry)|serde_json::json!({"interface":port,"ip":ip,"mac":entry.mac,"expires_us":entry.expires_us})).collect();
    let sat:Vec<_>=e.network().sat.iter().map(|((switch,mac),entry)|serde_json::json!({"switch":switch,"mac":mac,"interface":entry.interface,"expires_us":entry.expires_us})).collect();
    Snapshot {
        wait_factor: e.network().wait_factor(),
        generation: e.generation(),
        seed:e.seed(),
        link_conditions:e.network().configured_links(),
        bridges,
        epoch,
        command_serial,
        revision: e.revision(),
        project: Arc::new(e.project().clone()),
        effective: Arc::new(e.network().effective_project(e.project())),
        hosts: Arc::new(e.apps().views(e.project())),
        sockets: e.network().transport.views(),
        nat:e.network().nat.entries.values().cloned().collect(),
        firewall:e.network().firewall.journal.clone(),
        routes: e
            .network()
            .effective_project(e.project())
            .devices
            .iter()
            .filter(|d| d.kind != DeviceKind::Switch)
            .map(|d| (d.id.clone(), e.network().routing.views(d, e.now())))
            .collect(),
        rip_muted: e.network().routing.muted.clone(),
        trace: Arc::new(e.trace().to_vec()),
        now: e.now(),
        pending: e.pending(),
        running,
        tables: serde_json::to_string_pretty(&serde_json::json!({"ARP":arp,"SAT":sat,"NAT":e.network().nat.entries.values().collect::<Vec<_>>()})).unwrap(),
        error,
        run_id: e.run_id().clone(),
    }
}

#[cfg(test)]
mod checkpoint_tests {
    use super::*;
    #[test]
    fn restored_worker_reserves_old_and_future_ids_without_losing_command_acknowledgements() {
        let id = |s: &str| Id::new(s).unwrap();
        let mut scenario: Scenario = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/valid/network-scenario.json"
        )))
        .unwrap();
        scenario.network_actions[0].id = id("lan-ui-2");
        let mut e = Engine::new(scenario, None, Limits::default()).unwrap();
        e.command(CommandEnvelope {
            api_version: 1,
            command_id: id("lan-ui-1"),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::PurgeArp {
                device: id("host-a"),
            },
        })
        .unwrap();
        let mut worker = Worker::from_engine(e.checkpoint().unwrap().restore().unwrap()).unwrap();
        worker.submit(Job::Command(Command::PurgeArp {
            device: id("host-a"),
        }));
        assert!(worker.pending_commands());
        worker.wait().unwrap();
        assert!(!worker.pending_commands());
        assert_eq!(worker.view.command_serial, 1);
        assert_eq!(worker.view.error, None);
        let (tx, rx) = mpsc::sync_channel(1);
        worker.capture(false, tx).unwrap();
        let r = sns_engine::Recording::from_bytes(
            &rx.recv_timeout(Duration::from_secs(10)).unwrap().unwrap(),
        )
        .unwrap();
        let replay = r.replay().unwrap();
        assert!(replay.command_id_taken(&id("lan-ui-3")));
    }
}
