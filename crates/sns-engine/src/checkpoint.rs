//! Complete checkpoint data, verified by deterministic reconstruction before restoring a fresh generation.
use super::*;
pub const MAX_CHECKPOINT_BYTES: usize = 64 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub checkpoint_version: u32,
    pub state_encoding: String,
    pub recording: Recording,
    pub runtime_state: serde_json::Value,
    pub runtime_sha256: String,
}
impl Engine {
    pub fn recording(&self) -> Result<Recording, Diagnostic> {
        if self.recording.overflow
            || self.limits.queue > 65_536
            || self.limits.trace > 100_000
            || self.limits.commands > 4096
        {
            return Err(Diagnostic::new("checkpoint.history_limit", "recording"));
        }
        Ok(self.recording.clone())
    }
    pub fn checkpoint(&self) -> Result<Checkpoint, Diagnostic> {
        if self.state != RunState::Paused {
            return Err(Diagnostic::new("state.not_paused", "checkpoint"));
        }
        let recording = self.recording()?;
        let runtime_state = self.checkpoint_state()?;
        let runtime_sha256 = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&canonical_json(runtime_state.clone()))
                    .map_err(|_| Diagnostic::new("checkpoint.invalid_state", "state"))?
            )
        );
        let result = Checkpoint {
            checkpoint_version: recording.recording_version,
            state_encoding: if recording.wait_factor.is_default() {
                "structural-json-v1"
            } else {
                "structural-json-v2"
            }
            .into(),
            recording,
            runtime_state,
            runtime_sha256,
        };
        if serde_json::to_vec(&result).map_or(true, |v| v.len() > MAX_CHECKPOINT_BYTES) {
            return Err(Diagnostic::new("resource.limit_exceeded", "checkpoint"));
        }
        Ok(result)
    }
}
impl Checkpoint {
    /// The untrusted state tree is never deserialized into internal protocol objects.
    /// Validated inputs reconstruct reachable state; every exported state field must then match.
    pub fn restore(&self) -> Result<Engine, Diagnostic> {
        if self.checkpoint_version != self.recording.recording_version
            || !matches!(
                (self.checkpoint_version, self.state_encoding.as_str()),
                (1, "structural-json-v1") | (2, "structural-json-v2")
            )
        {
            return Err(Diagnostic::new("checkpoint.incompatible", "checkpoint"));
        }
        if serde_json::to_vec(self).map_or(true, |v| v.len() > MAX_CHECKPOINT_BYTES) {
            return Err(Diagnostic::new("resource.limit_exceeded", "checkpoint"));
        }
        let hash = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&canonical_json(self.runtime_state.clone()))
                    .map_err(|_| Diagnostic::new("checkpoint.invalid_state", "state"))?
            )
        );
        if hash != self.runtime_sha256 {
            return Err(Diagnostic::new("checkpoint.corrupt", "state_hash"));
        }
        let mut e = self.recording.replay()?;
        let reconstructed_hash = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&canonical_json(e.checkpoint_state()?))
                    .map_err(|_| Diagnostic::new("checkpoint.invalid_state", "state"))?
            )
        );
        if e.state != RunState::Paused || reconstructed_hash != self.runtime_sha256 {
            return Err(Diagnostic::new(
                "checkpoint.incompatible",
                "reconstructed_state",
            ));
        }
        e.renew_generation_after(e.generation)?;
        Ok(e)
    }
    /// Replay an explicitly supplied continuation; ordinary stale commands remain rejected.
    /// The complete recording must validate and extend this checkpoint's exact history.
    pub fn replay_remaining(&self, complete: &Recording) -> Result<Engine, Diagnostic> {
        let header = |r: &Recording| {
            serde_json::to_value((
                r.recording_version,
                &r.engine_profile,
                &r.initial_scenario,
                r.seed,
                r.initial_generation,
                r.limits,
                r.wait_factor,
            ))
            .map_err(|_| Diagnostic::new("checkpoint.invalid_recording", "header"))
        };
        if header(&self.recording)? != header(complete)? {
            return Err(Diagnostic::new("checkpoint.invalid_recording", "origin"));
        }
        // Validate all commands and boundaries in their recorded generations before remapping.
        complete.replay()?;
        let prefix = &self.recording.operations;
        if prefix.len() > complete.operations.len() {
            return Err(Diagnostic::new("checkpoint.invalid_recording", "prefix"));
        }
        let mut tail = Vec::new();
        for (i, expected) in prefix.iter().enumerate() {
            let actual = &complete.operations[i];
            match (expected, actual) {
                (
                    RecordedOperation::Events { count: a },
                    RecordedOperation::Events { count: b },
                ) if i + 1 == prefix.len() && b.0 >= a.0 => {
                    if b.0 > a.0 {
                        tail.push(RecordedOperation::Events {
                            count: Decimal(b.0 - a.0),
                        });
                    }
                }
                _ if serde_json::to_value(expected).ok() == serde_json::to_value(actual).ok() => {}
                _ => return Err(Diagnostic::new("checkpoint.invalid_recording", "prefix")),
            }
        }
        tail.extend_from_slice(&complete.operations[prefix.len()..]);
        let mut e = self.restore()?;
        for op in tail {
            match op {
                RecordedOperation::Command { mut envelope } => {
                    envelope.run_id = e.run_id.clone();
                    e.command(*envelope)?;
                }
                RecordedOperation::Events { count } => {
                    let (n, _) = e.pump(count.0 as usize, None)?;
                    if n as u64 != count.0 {
                        return Err(Diagnostic::new(
                            "checkpoint.invalid_recording",
                            "continuation",
                        ));
                    }
                }
                RecordedOperation::DrainBridgeOutput => {
                    e.take_bridge_output();
                }
                RecordedOperation::Stop => e.stop()?,
                RecordedOperation::Pause => {
                    e.state = RunState::Paused;
                    e.recording.push(RecordedOperation::Pause);
                }
                RecordedOperation::Rebase { .. } => {
                    e.generation = e.generation.checked_add(1).ok_or_else(|| {
                        Diagnostic::new("simulation.generation_overflow", "checkpoint")
                    })?;
                    e.run_id = Id::new(format!("run-{}-{}", e.seed, e.generation))?;
                    e.recording.push(RecordedOperation::Rebase {
                        generation: Decimal(e.generation),
                    });
                }
            }
        }
        e.recording()?;
        Ok(e)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(Diagnostic::new("resource.limit_exceeded", "checkpoint"));
        }
        serde_json::from_slice(bytes).map_err(|_| Diagnostic::new("checkpoint.corrupt", "json"))
    }
}
