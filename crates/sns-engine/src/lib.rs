//! Deterministic, bounded event engine. J0 messages are prototypes, not a TCP/IP stack.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sns_model::*;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
};

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Debug)]
struct Transit {
    source: Id,
    target: Id,
    message: Message,
    correlation: Id,
    caused_by: Option<Id>,
}
#[derive(Clone, Debug)]
enum Event {
    Dispatch(Transit),
    Arrival {
        transit: Transit,
        route: Vec<(Id, Id)>,
        hop: usize,
    },
}
#[derive(Clone, Debug)]
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

pub struct Engine {
    scenario: Scenario,
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
}

impl Engine {
    pub fn new(scenario: Scenario, seed: Option<u64>, limits: Limits) -> Result<Self, Diagnostic> {
        scenario.validate()?;
        if scenario.actions.len() > limits.queue {
            return Err(Diagnostic::new("resource.limit_exceeded", "queue"));
        }
        let seed = seed.unwrap_or(scenario.seed.0);
        let run_id = Id::new(format!("run-{seed}-0"))?;
        let mut engine = Self {
            scenario,
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
        };
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
        Ok(engine)
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
    pub fn restart(&mut self) -> Result<(), Diagnostic> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.generation_overflow", "run"))?;
        let mut next = Self::new(self.scenario.clone(), Some(self.seed), self.limits)?;
        next.generation = generation;
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
        if envelope.expected_revision.0 != 0 {
            return Err(Diagnostic::new("state.revision_conflict", "project"));
        }
        if self.receipts.len() >= self.limits.commands {
            return Err(Diagnostic::new("resource.limit_exceeded", "commands"));
        }
        if self.state == RunState::Stopped {
            return Err(Diagnostic::new("state.stopped", self.run_id.as_str()));
        }
        match &envelope.command {
            Command::Pause => self.state = RunState::Paused,
            Command::Resume => self.state = RunState::Running,
            Command::Stop => {
                self.queue.clear();
                self.state = RunState::Stopped;
            }
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
        if until_us.is_some_and(|bound| bound < self.now) {
            return Err(Diagnostic::new("simulation.past_bound", "until_us"));
        }
        if self.state == RunState::Stopped {
            return Ok(self.report(0, Termination::Stopped));
        }
        self.state = RunState::Running;
        let result = self.advance(event_budget, until_us);
        self.state = RunState::Paused;
        result
    }
    fn advance(&mut self, budget: usize, bound: Option<u64>) -> Result<RunReport, Diagnostic> {
        let mut count = 0;
        loop {
            let Some(Reverse(next)) = self.queue.peek() else {
                return Ok(self.report(count, Termination::Idle));
            };
            if bound.is_some_and(|limit| next.time > limit) {
                return Ok(self.report(count, Termination::TimeBound));
            }
            if count >= budget {
                return Ok(self.report(count, Termination::EventBudget));
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
        let Some(Reverse(scheduled)) = self.queue.peek().cloned() else {
            return Ok(false);
        };
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

    fn plan(
        &self,
        scheduled: &Scheduled,
        rng: &mut SplitMix64,
    ) -> Result<(Vec<Effect>, Vec<Fact>), Diagnostic> {
        let mut effects = Vec::new();
        let mut facts = Vec::new();
        let event_id = Id::new(format!("evt-{}", self.trace.len() + 1))?;
        match &scheduled.event {
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
