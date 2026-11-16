//! Portable input/event-boundary journal. Replaying this module never constructs an OS adapter.
use super::*;
pub const MAX_RECORDED_OPERATIONS: usize = 65_536;
pub const MAX_REPLAY_EVENTS: u64 = 1_000_000;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecordedOperation {
    Command { envelope: Box<CommandEnvelope> },
    Events { count: Decimal },
    DrainBridgeOutput,
    Stop,
    Pause,
    Rebase { generation: Decimal },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recording {
    pub recording_version: u32,
    pub engine_profile: String,
    #[serde(default, skip_serializing_if = "WaitFactor::is_default")]
    pub wait_factor: WaitFactor,
    pub initial_scenario: Arc<Scenario>,
    pub seed: Decimal,
    pub initial_generation: Decimal,
    pub limits: Limits,
    pub operations: Vec<RecordedOperation>,
    #[serde(skip)]
    pub(crate) suppressed: bool,
    #[serde(skip)]
    pub(crate) overflow: bool,
    #[serde(skip)]
    pub(crate) events: u64,
    #[serde(skip)]
    pub(crate) bytes: usize,
}
impl Recording {
    pub(crate) fn new(
        origin: Arc<Scenario>,
        seed: u64,
        limits: Limits,
        wait_factor: WaitFactor,
    ) -> Self {
        let bytes = serde_json::to_vec(&origin).map_or(usize::MAX, |v| v.len().saturating_add(512));
        Self {
            recording_version: if wait_factor.is_default() { 1 } else { 2 },
            engine_profile: if wait_factor.is_default() {
                "sns-engine-semantics-v1"
            } else {
                "sns-engine-semantics-v2"
            }
            .into(),
            wait_factor,
            initial_scenario: origin,
            seed: Decimal(seed),
            initial_generation: Decimal(0),
            limits,
            operations: vec![],
            suppressed: false,
            overflow: bytes > MAX_CHECKPOINT_BYTES,
            events: 0,
            bytes,
        }
    }
    pub(crate) fn push(&mut self, op: RecordedOperation) {
        if self.suppressed || self.overflow {
            return;
        }
        if let RecordedOperation::Events { count } = &op {
            self.events = self.events.saturating_add(count.0);
            if self.events > MAX_REPLAY_EVENTS {
                self.overflow = true;
                return;
            }
            if let Some(RecordedOperation::Events { count: previous }) = self.operations.last_mut()
            {
                previous.0 += count.0;
                return;
            }
        }
        let size = Self::operation_size(&op);
        self.bytes = self.bytes.saturating_add(size);
        if self.bytes > MAX_CHECKPOINT_BYTES {
            self.overflow = true;
            return;
        }
        if self.operations.len() >= MAX_RECORDED_OPERATIONS {
            self.overflow = true;
            return;
        }
        self.operations.push(op);
    }
    fn operation_size(op: &RecordedOperation) -> usize {
        if matches!(op, RecordedOperation::Events { .. }) {
            96
        } else {
            serde_json::to_vec(op).map_or(usize::MAX, |v| v.len().saturating_add(32))
        }
    }
    fn estimated_bytes(&self) -> usize {
        self.operations.iter().fold(
            serde_json::to_vec(&self.initial_scenario)
                .map_or(usize::MAX, |v| v.len().saturating_add(512)),
            |n, op| n.saturating_add(Self::operation_size(op)),
        )
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(Diagnostic::new("resource.limit_exceeded", "recording"));
        }
        serde_json::from_slice(bytes)
            .map_err(|_| Diagnostic::new("checkpoint.corrupt", "recording_json"))
    }
    pub fn replay(&self) -> Result<Engine, Diagnostic> {
        if serde_json::to_vec(self).map_or(true, |v| v.len() > MAX_CHECKPOINT_BYTES) {
            return Err(Diagnostic::new("resource.limit_exceeded", "recording"));
        }
        if !matches!(
            (
                self.recording_version,
                self.engine_profile.as_str(),
                self.wait_factor.is_default()
            ),
            (1, "sns-engine-semantics-v1", true) | (2, "sns-engine-semantics-v2", false)
        ) {
            return Err(Diagnostic::new("checkpoint.incompatible", "recording"));
        }
        if self.operations.len() > MAX_RECORDED_OPERATIONS
            || self.estimated_bytes() > MAX_CHECKPOINT_BYTES
            || self.limits.queue > 65_536
            || self.limits.trace > 100_000
            || self.limits.commands > 4096
        {
            return Err(Diagnostic::new("resource.limit_exceeded", "recording"));
        }
        self.initial_scenario.validate()?;
        let mut e = Engine::new_with_wait_factor(
            self.initial_scenario.as_ref().clone(),
            Some(self.seed.0),
            self.limits,
            self.wait_factor,
        )?;
        e.generation = self.initial_generation.0;
        e.run_id = Id::new(format!("run-{}-{}", e.seed, e.generation))?;
        e.recording.suppressed = true;
        let mut total = 0u64;
        for op in &self.operations {
            match op {
                RecordedOperation::Command { envelope } => e.command(*envelope.clone())?,
                RecordedOperation::Events { count } => {
                    total = total
                        .checked_add(count.0)
                        .filter(|n| *n <= MAX_REPLAY_EVENTS)
                        .ok_or_else(|| {
                            Diagnostic::new("resource.limit_exceeded", "replay_events")
                        })?;
                    if count.0 == 0 {
                        return Err(Diagnostic::new(
                            "checkpoint.invalid_recording",
                            "zero_event_boundary",
                        ));
                    }
                    let (processed, _) = e.pump(count.0 as usize, None)?;
                    if processed as u64 != count.0 {
                        return Err(Diagnostic::new(
                            "checkpoint.invalid_recording",
                            "event_boundary",
                        ));
                    }
                }
                RecordedOperation::DrainBridgeOutput => {
                    e.take_bridge_output();
                }
                RecordedOperation::Stop => e.stop()?,
                RecordedOperation::Pause => e.state = RunState::Paused,
                RecordedOperation::Rebase { generation } => {
                    if generation.0 <= e.generation {
                        return Err(Diagnostic::new(
                            "checkpoint.invalid_recording",
                            "generation",
                        ));
                    }
                    e.generation = generation.0;
                    e.run_id = Id::new(format!("run-{}-{}", e.seed, e.generation))?;
                }
            }
        }
        e.recording = self.clone();
        e.recording.bytes = self.estimated_bytes();
        e.recording.events = total;
        e.recording.suppressed = false;
        e.recording.overflow = false;
        Ok(e)
    }
}
