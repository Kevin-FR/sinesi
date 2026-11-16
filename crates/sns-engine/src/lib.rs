//! Deterministic bounded engine; real LAN events and isolated J0 prototype messages.
mod applications;
mod checkpoint;
pub use checkpoint::{Checkpoint, MAX_CHECKPOINT_BYTES};
mod recording;
pub use recording::{RecordedOperation, Recording};
mod state;
mod state_value;
use applications::ApplicationPlan;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sns_model::*;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
    sync::Arc,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub queue: usize,
    pub trace: usize,
    pub commands: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            queue: 65_536,
            trace: 100_000,
            commands: 4_096,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceEvent {
    pub event_schema_version: u32,
    pub run_id: Id,
    pub event_id: Id,
    pub time_us: Decimal,
    pub sequence: Decimal,
    #[serde(rename = "type")]
    pub kind: String,
    pub device_id: Id,
    pub caused_by: Option<Id>,
    pub correlation_id: Id,
    pub payload: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Termination {
    Idle,
    TimeBound,
    EventBudget,
    Stopped,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RunReport {
    pub time_us: Decimal,
    pub processed_events: usize,
    pub pending_events: usize,
    pub termination: Termination,
    pub trace_sha256: String,
}

#[derive(Clone, Copy)]
struct SplitMix64(u64);
impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

#[derive(Clone, Debug, Serialize)]
struct Transit {
    source: Id,
    target: Id,
    message: Message,
    correlation: Id,
    caused_by: Option<Id>,
}
#[derive(Clone, Debug, Serialize)]
enum Event {
    NetworkCommand {
        action: NetworkAction,
    },
    Application {
        timer: sns_apps::Timer,
        cause: Option<Id>,
    },
    Network {
        event: sns_protocols::Event,
        cause: Option<Id>,
    },
    Dispatch(Transit),
    Arrival {
        transit: Transit,
        route: Vec<(Id, Id)>,
        hop: usize,
    },
}
#[derive(Clone, Debug, Serialize)]
struct Scheduled {
    time: u64,
    priority: u8,
    sequence: u64,
    event: Event,
}
impl PartialEq for Scheduled {
    fn eq(&self, other: &Self) -> bool {
        (self.time, self.priority, self.sequence) == (other.time, other.priority, other.sequence)
    }
}
impl Eq for Scheduled {}
impl Ord for Scheduled {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.time, self.priority, self.sequence).cmp(&(other.time, other.priority, other.sequence))
    }
}
impl PartialOrd for Scheduled {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Effect {
    time: u64,
    priority: u8,
    event: Event,
}
struct Fact {
    kind: &'static str,
    device: Id,
    transit: Transit,
    reason: Option<&'static str>,
}

#[derive(Clone)]
pub struct Engine {
    recording: Recording,
    scenario: Arc<Scenario>,
    seed: u64,
    generation: u64,
    run_id: Id,
    now: u64,
    state: RunState,
    next_sequence: u64,
    rng: SplitMix64,
    queue: BinaryHeap<Reverse<Scheduled>>,
    trace: Vec<TraceEvent>,
    limits: Limits,
    receipts: BTreeMap<Id, CommandEnvelope>,
    network: sns_protocols::Network,
    apps: sns_apps::Runtime,
    revision: u64,
}

impl Engine {
    pub fn new(scenario: Scenario, seed: Option<u64>, limits: Limits) -> Result<Self, Diagnostic> {
        Self::new_with_wait_factor(scenario, seed, limits, WaitFactor::default())
    }
    pub fn new_with_wait_factor(
        scenario: Scenario,
        seed: Option<u64>,
        limits: Limits,
        wait_factor: WaitFactor,
    ) -> Result<Self, Diagnostic> {
        scenario.validate()?;
        if scenario.actions.len() + scenario.network_actions.len() > limits.queue {
            return Err(Diagnostic::new("resource.limit_exceeded", "queue"));
        }
        let apps = sns_apps::Runtime::new(&scenario.project)?;
        let seed = seed.unwrap_or(scenario.seed.0);
        let run_id = Id::new(format!("run-{seed}-0"))?;
        let scenario = Arc::new(scenario);
        let recording = Recording::new(scenario.clone(), seed, limits, wait_factor);
        let mut engine = Self {
            recording,
            scenario,
            apps,
            revision: 0,
            seed,
            generation: 0,
            run_id,
            now: 0,
            state: RunState::Paused,
            next_sequence: 0,
            rng: SplitMix64(seed),
            queue: BinaryHeap::new(),
            trace: Vec::new(),
            limits,
            receipts: BTreeMap::new(),
            network: sns_protocols::Network::with_wait_factor(wait_factor),
        };
        let mut routers: Vec<_> = engine
            .project()
            .devices
            .iter()
            .filter(|h| h.network.rip)
            .map(|h| h.id.clone())
            .collect();
        routers.sort();
        for (index, device) in routers.into_iter().enumerate() {
            engine.enqueue(Effect {
                time: 0,
                priority: 15,
                event: Event::Network {
                    event: sns_protocols::Event {
                        correlation: Id::new(format!("rip-init-{index}"))?,
                        action: sns_protocols::Action::RipTick {
                            device,
                            periodic: true,
                        },
                    },
                    cause: None,
                },
            })?;
        }
        let clients: Vec<_> = engine
            .project()
            .devices
            .iter()
            .flat_map(|d| &d.interfaces)
            .filter(|i| matches!(i.ipv4, Some(Ipv4Config::Dhcp)))
            .map(|i| i.id.clone())
            .collect();
        for (index, interface) in clients.into_iter().enumerate() {
            engine.enqueue(Effect {
                time: 0,
                priority: 15,
                event: Event::Network {
                    event: sns_protocols::Event {
                        correlation: Id::new(format!("dhcp-init-{index}"))?,
                        action: sns_protocols::Action::DhcpStart { interface },
                    },
                    cause: None,
                },
            })?;
        }
        for action in engine.scenario.network_actions.clone() {
            engine.enqueue(Effect {
                time: action.at_us.0,
                priority: 0,
                event: Event::NetworkCommand { action },
            })?;
        }
        let actions = engine.scenario.actions.clone();
        for (index, action) in actions.into_iter().enumerate() {
            let transit = Transit {
                source: action.source,
                target: action.target,
                message: action.message,
                correlation: Id::new(format!("flow-{index}"))?,
                caused_by: None,
            };
            engine.enqueue(Effect {
                time: action.at_us.0,
                priority: 20,
                event: Event::Dispatch(transit),
            })?;
        }
        engine.boot_applications()?;
        Ok(engine)
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn apps(&self) -> &sns_apps::Runtime {
        &self.apps
    }
    pub fn stop(&mut self) -> Result<(), Diagnostic> {
        let record = self.state != RunState::Stopped;
        let apps = sns_apps::Runtime::new(self.project())?;
        self.queue.clear();
        self.network = sns_protocols::Network::with_wait_factor(self.network.wait_factor());
        self.apps = apps;
        self.state = RunState::Stopped;
        if record {
            self.recording.push(RecordedOperation::Stop);
        }
        Ok(())
    }
    pub fn project(&self) -> &Project {
        &self.scenario.project
    }
    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }
    pub fn run_id(&self) -> &Id {
        &self.run_id
    }
    /// Command identities remain reserved after checkpoint restoration.
    pub fn command_id_taken(&self, id: &Id) -> bool {
        self.receipts.contains_key(id) || self.scenario.network_actions.iter().any(|a| &a.id == id)
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    /// Adopt recorded state as a new live execution without reusing an earlier UI generation.
    pub fn renew_generation_after(&mut self, floor: u64) -> Result<(), Diagnostic> {
        let generation = self
            .generation
            .max(floor)
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.generation_overflow", "restore"))?;
        let run_id = Id::new(format!("run-{}-{generation}", self.seed))?;
        let mut recording = self.recording()?;
        recording.push(RecordedOperation::Rebase {
            generation: Decimal(generation),
        });
        if recording.overflow {
            return Err(Diagnostic::new("checkpoint.history_limit", "generation"));
        }
        self.generation = generation;
        self.run_id = run_id;
        self.recording = recording;
        Ok(())
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn now(&self) -> u64 {
        self.now
    }
    pub fn state(&self) -> RunState {
        self.state
    }
    pub fn trace(&self) -> &[TraceEvent] {
        &self.trace
    }
    pub fn pending(&self) -> usize {
        self.queue.len()
    }
    pub fn take_bridge_output(&mut self) -> Vec<(Id, BridgePacket)> {
        if !self.network.bridge_output.is_empty() {
            self.recording.push(RecordedOperation::DrainBridgeOutput);
        }
        std::mem::take(&mut self.network.bridge_output)
    }
    pub fn network(&self) -> &sns_protocols::Network {
        &self.network
    }
    pub fn from_project(project: Project, seed: u64, limits: Limits) -> Result<Self, Diagnostic> {
        Self::from_project_with_wait_factor(project, seed, limits, WaitFactor::default())
    }
    pub fn from_project_with_wait_factor(
        project: Project,
        seed: u64,
        limits: Limits,
        wait_factor: WaitFactor,
    ) -> Result<Self, Diagnostic> {
        Self::new_with_wait_factor(
            Scenario {
                scenario_schema_version: 1,
                project,
                seed: Decimal(seed),
                actions: vec![],
                prototype_servers: vec![],
                network_actions: vec![],
            },
            None,
            limits,
            wait_factor,
        )
    }
    pub fn replace_project(&mut self, project: Project) -> Result<(), Diagnostic> {
        self.replace_project_with_wait_factor(project, self.network.wait_factor())
    }
    pub fn replace_project_with_wait_factor(
        &mut self,
        project: Project,
        wait_factor: WaitFactor,
    ) -> Result<(), Diagnostic> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.generation_overflow", "run"))?;
        let mut next =
            Self::from_project_with_wait_factor(project, self.seed, self.limits, wait_factor)?;
        next.generation = generation;
        next.recording.initial_generation = Decimal(generation);
        next.run_id = Id::new(format!("run-{}-{generation}", self.seed))?;
        *self = next;
        Ok(())
    }
    pub fn restart(&mut self) -> Result<(), Diagnostic> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.generation_overflow", "run"))?;
        let mut next = Self::new_with_wait_factor(
            self.scenario.as_ref().clone(),
            Some(self.seed),
            self.limits,
            self.network.wait_factor(),
        )?;
        next.generation = generation;
        next.recording.initial_generation = Decimal(generation);
        next.run_id = Id::new(format!("run-{}-{generation}", self.seed))?;
        *self = next;
        Ok(())
    }

    fn enqueue(&mut self, effect: Effect) -> Result<(), Diagnostic> {
        if self.queue.len() >= self.limits.queue {
            return Err(Diagnostic::new("resource.limit_exceeded", "queue"));
        }
        let next = self
            .next_sequence
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "queue"))?;
        self.queue.push(Reverse(Scheduled {
            time: effect.time,
            priority: effect.priority,
            sequence: self.next_sequence,
            event: effect.event,
        }));
        self.next_sequence = next;
        Ok(())
    }

    /// Accepted command IDs are remembered for this run; reusing an ID with other data fails.
    pub fn command(&mut self, envelope: CommandEnvelope) -> Result<(), Diagnostic> {
        let suppressed = self.recording.suppressed;
        self.recording.suppressed = true;
        let repeated = self.receipts.contains_key(&envelope.command_id);
        let result = self.command_inner(envelope.clone());
        self.recording.suppressed = suppressed;
        if result.is_ok() && !repeated {
            self.recording.push(RecordedOperation::Command {
                envelope: Box::new(envelope),
            });
        }
        result
    }
    fn command_inner(&mut self, envelope: CommandEnvelope) -> Result<(), Diagnostic> {
        if envelope.api_version != 1 || envelope.project_id != self.project().project_id {
            return Err(Diagnostic::new(
                "command.invalid_envelope",
                envelope.command_id.as_str(),
            ));
        }
        if &envelope.run_id != self.run_id() {
            return Err(Diagnostic::new("state.stale_run", envelope.run_id.as_str()));
        }
        if let Some(previous) = self.receipts.get(&envelope.command_id) {
            return if previous == &envelope {
                Ok(())
            } else {
                Err(Diagnostic::new(
                    "command.id_conflict",
                    envelope.command_id.as_str(),
                ))
            };
        }
        if envelope.expected_revision.0 != self.revision {
            return Err(Diagnostic::new("state.revision_conflict", "project"));
        }
        if self.receipts.len() >= self.limits.commands {
            return Err(Diagnostic::new("resource.limit_exceeded", "commands"));
        }
        if self.state == RunState::Stopped {
            return Err(Diagnostic::new("state.stopped", self.run_id.as_str()));
        }
        match &envelope.command {
            Command::SetSymbol { device, symbol } => {
                let project = self.project().with_symbol(device, *symbol)?;
                if project != *self.project() {
                    if self.trace.len() >= self.limits.trace {
                        return Err(Diagnostic::new("resource.limit_exceeded", "trace"));
                    }
                    let revision = self
                        .revision
                        .checked_add(1)
                        .ok_or_else(|| Diagnostic::new("state.revision_overflow", "project"))?;
                    let sequence = self.trace.len() as u64 + 1;
                    let fact = TraceEvent {
                        event_schema_version: 10,
                        run_id: self.run_id.clone(),
                        event_id: Id::new(format!("evt-{sequence}"))?,
                        time_us: Decimal(self.now),
                        sequence: Decimal(sequence),
                        kind: "presentation.symbol.changed".into(),
                        device_id: device.clone(),
                        caused_by: None,
                        correlation_id: envelope.command_id.clone(),
                        payload: BTreeMap::from([
                            ("symbol".into(), serde_json::json!(symbol)),
                            (
                                "project_schema_version".into(),
                                serde_json::json!(project.schema_version),
                            ),
                        ]),
                    };
                    Arc::make_mut(&mut self.scenario).project = project;
                    self.revision = revision;
                    self.trace.push(fact);
                }
            }
            Command::ConfigureLink { link, conditions } => {
                let event = sns_protocols::Event {
                    correlation: envelope.command_id.clone(),
                    action: sns_protocols::Action::ConfigureLink {
                        link: link.clone(),
                        conditions: conditions.clone(),
                        seed: self.seed,
                    },
                };
                let scheduled = Scheduled {
                    time: self.now,
                    priority: 0,
                    sequence: 0,
                    event: Event::Network {
                        event: event.clone(),
                        cause: None,
                    },
                };
                self.tick_network(&scheduled, &event, None, false)?;
            }
            Command::Bridge { device, command } => {
                let event = sns_protocols::Event {
                    correlation: envelope.command_id.clone(),
                    action: sns_protocols::Action::Bridge {
                        device: device.clone(),
                        command: command.clone(),
                    },
                };
                let scheduled = Scheduled {
                    time: self.now,
                    priority: 20,
                    sequence: 0,
                    event: Event::Network {
                        event: event.clone(),
                        cause: None,
                    },
                };
                self.tick_network(&scheduled, &event, None, false)?;
            }
            Command::Security { device, command } => {
                self.security_command(device, command, &envelope.command_id)?
            }
            Command::RipMute { device, muted } => {
                let event = sns_protocols::Event {
                    correlation: envelope.command_id.clone(),
                    action: sns_protocols::Action::RipMute {
                        device: device.clone(),
                        muted: *muted,
                    },
                };
                let scheduled = Scheduled {
                    time: self.now,
                    priority: 20,
                    sequence: 0,
                    event: Event::Network {
                        event: event.clone(),
                        cause: None,
                    },
                };
                self.tick_network(&scheduled, &event, None, false)?;
            }
            Command::App { device, command } => {
                self.application_command(device, command, &envelope.command_id)?
            }
            Command::Socket { device, command } => {
                let event = sns_protocols::Event {
                    correlation: envelope.command_id.clone(),
                    action: sns_protocols::Action::Socket {
                        device: device.clone(),
                        command: command.clone(),
                    },
                };
                let scheduled = Scheduled {
                    time: self.now,
                    priority: 20,
                    sequence: 0,
                    event: Event::Network {
                        event: event.clone(),
                        cause: None,
                    },
                };
                self.tick_network(&scheduled, &event, None, false)?;
            }
            Command::Ping {
                source,
                destination,
                ttl,
            } => {
                if !self.project().device(source).is_some_and(|d| {
                    matches!(
                        d.kind,
                        DeviceKind::Workstation | DeviceKind::Notebook | DeviceKind::Router
                    )
                }) {
                    return Err(Diagnostic::new("device.not_host", source.as_str()));
                }
                let destination = destination
                    .parse()
                    .map_err(|_| Diagnostic::new("validation.invalid_ipv4", destination))?;
                let target: std::net::Ipv4Addr = destination;
                let host = self.project().device(source).unwrap();
                if target.is_broadcast() || host.interfaces.iter().any(|i|matches!(&i.ipv4,Some(Ipv4Config::Static{address,prefix_length}) if *prefix_length<31 && address.parse().ok().is_some_and(|a|sns_protocols::subnet(a,*prefix_length).is_ok_and(|(_,b)|b==target)))) {
                    return Err(Diagnostic::new("ipv4.broadcast_requires_opt_in",source.as_str()));
                }
                self.enqueue(Effect {
                    time: self.now,
                    priority: 20,
                    event: Event::Network {
                        event: sns_protocols::Event {
                            correlation: envelope.command_id.clone(),
                            action: sns_protocols::Action::Ping {
                                source: source.clone(),
                                destination,
                                ttl: *ttl,
                            },
                        },
                        cause: None,
                    },
                })?;
            }
            Command::ManualLoss { enabled } => self.network.manual_loss = *enabled,
            Command::PurgeArp { device } | Command::PurgeSat { device } => {
                if self.project().device(device).is_none() {
                    return Err(Diagnostic::new(
                        "project.invalid_reference",
                        device.as_str(),
                    ));
                }
                let action = if matches!(&envelope.command, Command::PurgeArp { .. }) {
                    sns_protocols::Action::PurgeArp {
                        device: device.clone(),
                    }
                } else {
                    sns_protocols::Action::PurgeSat {
                        device: device.clone(),
                    }
                };
                let event = sns_protocols::Event {
                    correlation: envelope.command_id.clone(),
                    action,
                };
                let scheduled = Scheduled {
                    time: self.now,
                    priority: 0,
                    sequence: 0,
                    event: Event::Network {
                        event: event.clone(),
                        cause: None,
                    },
                };
                self.tick_network(&scheduled, &event, None, false)?;
            }
            Command::Pause => self.state = RunState::Paused,
            Command::Resume => self.state = RunState::Running,
            Command::Stop => self.stop()?,
            Command::Step => {
                if self.state != RunState::Paused {
                    return Err(Diagnostic::new("state.not_paused", "simulation"));
                }
                self.tick()?;
            }
            Command::Send {
                source,
                target,
                message,
            } => {
                message.validate()?;
                if self.project().device(source).is_none()
                    || self.project().device(target).is_none()
                {
                    return Err(Diagnostic::new("project.invalid_reference", "command"));
                }
                self.enqueue(Effect {
                    time: self.now,
                    priority: 20,
                    event: Event::Dispatch(Transit {
                        source: source.clone(),
                        target: target.clone(),
                        message: message.clone(),
                        correlation: envelope.command_id.clone(),
                        caused_by: None,
                    }),
                })?;
            }
        }
        self.receipts.insert(envelope.command_id.clone(), envelope);
        Ok(())
    }

    /// Execute at most one event, transactionally, even when a quota/overflow is hit.
    pub fn step(&mut self) -> Result<bool, Diagnostic> {
        if self.state != RunState::Paused {
            return Err(Diagnostic::new("state.not_paused", "simulation"));
        }
        self.tick()
    }

    pub fn run(
        &mut self,
        event_budget: usize,
        until_us: Option<u64>,
    ) -> Result<RunReport, Diagnostic> {
        let (count, termination) = self.pump(event_budget, until_us)?;
        Ok(self.report(count, termination))
    }
    /// Bounded execution without serializing the entire trace (for UI projections).
    pub fn pump(
        &mut self,
        budget: usize,
        bound: Option<u64>,
    ) -> Result<(usize, Termination), Diagnostic> {
        if bound.is_some_and(|bound| bound < self.now) {
            return Err(Diagnostic::new("simulation.past_bound", "until_us"));
        }
        if self.state == RunState::Stopped {
            return Ok((0, Termination::Stopped));
        }
        let before = self.state;
        let recorded = self.recording.events;
        self.state = RunState::Running;
        let result = self.advance(budget, bound);
        self.state = RunState::Paused;
        if self.recording.events == recorded && before != RunState::Paused {
            self.recording.push(RecordedOperation::Pause);
        }
        result
    }
    fn advance(
        &mut self,
        budget: usize,
        bound: Option<u64>,
    ) -> Result<(usize, Termination), Diagnostic> {
        let mut count = 0;
        loop {
            let Some(Reverse(next)) = self.queue.peek() else {
                return Ok((count, Termination::Idle));
            };
            if bound.is_some_and(|limit| next.time > limit) {
                return Ok((count, Termination::TimeBound));
            }
            if count >= budget {
                return Ok((count, Termination::EventBudget));
            }
            self.tick()?;
            count += 1;
        }
    }
    fn report(&self, count: usize, termination: Termination) -> RunReport {
        RunReport {
            time_us: Decimal(self.now),
            processed_events: count,
            pending_events: self.queue.len(),
            termination,
            trace_sha256: self.trace_hash(),
        }
    }
    pub fn trace_hash(&self) -> String {
        format!("{:x}", Sha256::digest(self.canonical_trace()))
    }
    pub fn canonical_trace(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for event in &self.trace {
            // Struct fields have fixed order; nested payload objects are recursively sorted.
            let value = serde_json::to_value(event).expect("trace consists of valid JSON data");
            let mut canonical = canonical_json(value);
            canonical["run_id"] = serde_json::json!("run");
            serde_json::to_writer(&mut result, &canonical).expect("writing to Vec cannot fail");
            result.push(b'\n');
        }
        result
    }

    fn tick(&mut self) -> Result<bool, Diagnostic> {
        let result = self.tick_inner();
        if matches!(result, Ok(true)) {
            self.recording
                .push(RecordedOperation::Events { count: Decimal(1) });
        }
        result
    }
    fn tick_inner(&mut self) -> Result<bool, Diagnostic> {
        let Some(Reverse(scheduled)) = self.queue.peek().cloned() else {
            return Ok(false);
        };
        if let Event::NetworkCommand { action } = &scheduled.event {
            // A command and its consumed queue entry commit together. Invalid actions leave the engine unchanged.
            let mut next = self.clone();
            next.queue.pop();
            next.now = scheduled.time;
            next.command_inner(CommandEnvelope {
                api_version: 1,
                command_id: action.id.clone(),
                project_id: next.project().project_id.clone(),
                expected_revision: Decimal(next.revision),
                run_id: next.run_id.clone(),
                command: action.command.clone(),
            })?;
            *self = next;
            return Ok(true);
        }
        if let Event::Application { timer, cause } = &scheduled.event {
            return self.tick_application(&scheduled, timer, cause.clone());
        }
        if let Event::Network { event, cause } = &scheduled.event {
            return self.tick_network(&scheduled, event, cause.clone(), true);
        }
        let mut rng = self.rng;
        let (effects, facts) = self.plan(&scheduled, &mut rng)?;
        if self.queue.len() - 1 + effects.len() > self.limits.queue
            || self.trace.len() + facts.len() > self.limits.trace
        {
            return Err(Diagnostic::new("resource.limit_exceeded", "engine"));
        }
        self.next_sequence
            .checked_add(effects.len() as u64)
            .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "queue"))?;
        self.queue.pop();
        self.now = scheduled.time;
        self.rng = rng;
        for fact in facts {
            let sequence = self.trace.len() as u64 + 1;
            let mut payload = BTreeMap::new();
            payload.insert("source".into(), serde_json::json!(fact.transit.source));
            payload.insert("target".into(), serde_json::json!(fact.transit.target));
            payload.insert("message".into(), serde_json::json!(fact.transit.message));
            if let Some(reason) = fact.reason {
                payload.insert("reason".into(), serde_json::json!(reason));
            }
            self.trace.push(TraceEvent {
                event_schema_version: 1,
                run_id: self.run_id.clone(),
                event_id: Id::new(format!("evt-{sequence}"))?,
                time_us: Decimal(self.now),
                sequence: Decimal(sequence),
                kind: fact.kind.into(),
                device_id: fact.device,
                caused_by: fact.transit.caused_by,
                correlation_id: fact.transit.correlation,
                payload,
            });
        }
        for effect in effects {
            self.enqueue(effect)?;
        }
        Ok(true)
    }

    fn tick_network(
        &mut self,
        scheduled: &Scheduled,
        event: &sns_protocols::Event,
        mut cause: Option<Id>,
        consume: bool,
    ) -> Result<bool, Diagnostic> {
        let scenario = self.scenario.clone();
        let mut plan =
            ApplicationPlan::new(&scenario.project, self.network.clone(), self.apps.clone());
        plan.network(scheduled.time, event.clone())?;
        self.commit_application_plan(scheduled.time, plan, cause.take(), consume)?;
        Ok(true)
    }

    fn plan(
        &self,
        scheduled: &Scheduled,
        rng: &mut SplitMix64,
    ) -> Result<(Vec<Effect>, Vec<Fact>), Diagnostic> {
        let mut effects = Vec::new();
        let mut facts = Vec::new();
        let event_id = Id::new(format!("evt-{}", self.trace.len() + 1))?;
        match &scheduled.event {
            Event::NetworkCommand { .. } | Event::Application { .. } | Event::Network { .. } => {
                unreachable!("network events have their own transactional planner")
            }
            Event::Dispatch(transit) => {
                let Some(route) = self.route(&transit.source, &transit.target) else {
                    facts.push(Fact {
                        kind: "prototype.message.dropped",
                        device: transit.source.clone(),
                        transit: transit.clone(),
                        reason: Some("routing.no_route"),
                    });
                    return Ok((effects, facts));
                };
                facts.push(Fact {
                    kind: "prototype.message.sent",
                    device: transit.source.clone(),
                    transit: transit.clone(),
                    reason: None,
                });
                let mut next = transit.clone();
                next.caused_by = Some(event_id);
                let time = if route.is_empty() {
                    scheduled.time
                } else {
                    self.arrival_time(scheduled.time, &route[0].0, rng)?
                };
                effects.push(Effect {
                    time,
                    priority: 10,
                    event: Event::Arrival {
                        transit: next,
                        route,
                        hop: 0,
                    },
                });
            }
            Event::Arrival {
                transit,
                route,
                hop,
            } => {
                let device = route
                    .get(*hop)
                    .map(|(_, node)| node.clone())
                    .unwrap_or_else(|| transit.target.clone());
                let arrived = *hop + 1 >= route.len();
                facts.push(Fact {
                    kind: if arrived {
                        "prototype.message.delivered"
                    } else {
                        "prototype.message.forwarded"
                    },
                    device: device.clone(),
                    transit: transit.clone(),
                    reason: None,
                });
                let mut next = transit.clone();
                next.caused_by = Some(event_id);
                if !arrived {
                    let time = self.arrival_time(scheduled.time, &route[*hop + 1].0, rng)?;
                    effects.push(Effect {
                        time,
                        priority: 10,
                        event: Event::Arrival {
                            transit: next,
                            route: route.clone(),
                            hop: hop + 1,
                        },
                    });
                } else if let Message::HttpRequest {
                    method,
                    path,
                    fields,
                } = &transit.message
                    && let Some(server) = self
                        .scenario
                        .prototype_servers
                        .iter()
                        .find(|s| s.device_id == device)
                {
                    let (status, media_type, body_base64) = match method {
                        HttpMethod::Get => server
                            .get_routes
                            .iter()
                            .find(|r| &r.path == path)
                            .map(|r| (200, r.media_type.clone(), r.body_base64.clone()))
                            .unwrap_or_else(|| {
                                (404, "text/plain".into(), STANDARD.encode("Not found"))
                            }),
                        HttpMethod::Post if path == &server.post_echo_path => {
                            let text = fields
                                .iter()
                                .map(|(k, v)| format!("{k}={v}"))
                                .collect::<Vec<_>>()
                                .join("\n");
                            (200, "text/plain".into(), STANDARD.encode(text))
                        }
                        HttpMethod::Post => {
                            (404, "text/plain".into(), STANDARD.encode("Not found"))
                        }
                    };
                    next.source = transit.target.clone();
                    next.target = transit.source.clone();
                    next.message = Message::HttpResponse {
                        status,
                        media_type,
                        body_base64,
                    };
                    effects.push(Effect {
                        time: scheduled.time,
                        priority: 20,
                        event: Event::Dispatch(next),
                    });
                }
            }
        }
        Ok((effects, facts))
    }
    fn arrival_time(
        &self,
        now: u64,
        link_id: &Id,
        rng: &mut SplitMix64,
    ) -> Result<u64, Diagnostic> {
        let link = self
            .project()
            .links
            .iter()
            .find(|l| &l.id == link_id)
            .ok_or_else(|| Diagnostic::new("project.invalid_reference", link_id.as_str()))?;
        let jitter = if link.jitter_us.0 == 0 {
            0
        } else if link.jitter_us.0 == u64::MAX {
            rng.next()
        } else {
            rng.next() % (link.jitter_us.0 + 1)
        };
        now.checked_add(link.latency_us.0)
            .and_then(|v| v.checked_add(jitter))
            .ok_or_else(|| Diagnostic::new("simulation.time_overflow", link_id.as_str()))
    }
    /// Sorted breadth-first prototype path, independent of project array ordering.
    fn route(&self, source: &Id, target: &Id) -> Option<Vec<(Id, Id)>> {
        let mut queue = VecDeque::from([(source.clone(), Vec::new())]);
        let mut visited = BTreeSet::from([source.clone()]);
        while let Some((current, route)) = queue.pop_front() {
            if &current == target {
                return Some(route);
            }
            let mut neighbors = Vec::new();
            for link in &self.project().links {
                let a = self.project().device_for_interface(&link.endpoints[0])?;
                let b = self.project().device_for_interface(&link.endpoints[1])?;
                if a.id == current {
                    neighbors.push((link.id.clone(), b.id.clone()));
                }
                if b.id == current {
                    neighbors.push((link.id.clone(), a.id.clone()));
                }
            }
            neighbors.sort();
            for (link, node) in neighbors {
                if visited.insert(node.clone()) {
                    let mut next = route.clone();
                    next.push((link, node.clone()));
                    queue.push_back((node, next));
                }
            }
        }
        None
    }
}

fn canonical_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(object) => {
            let sorted: BTreeMap<_, _> = object
                .into_iter()
                .map(|(k, v)| (k, canonical_json(v)))
                .collect();
            serde_json::Value::Object(sorted.into_iter().collect())
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonical_json).collect())
        }
        value => value,
    }
}
