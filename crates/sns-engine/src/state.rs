//! Complete explicit simulation state for versioned checkpoints; no platform adapter handles.
use super::*;
#[derive(Serialize)]
struct State<'a> {
    runtime_profile: &'static str,
    project_and_scenario: &'a Scenario,
    run_id: &'a Id,
    generation: u64,
    initial_seed: u64,
    virtual_time_us: Decimal,
    revision: u64,
    next_sequence: u64,
    run_state: RunState,
    limits: Limits,
    rng_algorithm: &'static str,
    rng_state: u64,
    queue: Vec<&'a Scheduled>,
    frame_transit_budgets: Vec<(u64, u8)>,
    network: &'a sns_protocols::Network,
    applications: &'a sns_apps::Runtime,
    command_receipts: &'a BTreeMap<Id, CommandEnvelope>,
    trace: &'a [TraceEvent],
    recording_position: RecordingPosition,
}
#[derive(Serialize)]
struct RecordingPosition {
    operations: usize,
    recorded_scheduler_events: u64,
    estimated_bytes: usize,
    overflow: bool,
}
impl Engine {
    /// Contains pedagogical data and credentials: reserved for explicitly requested checkpoint files.
    pub fn checkpoint_state(&self) -> Result<serde_json::Value, Diagnostic> {
        let mut queue = self.queue.iter().map(|e| &e.0).collect::<Vec<_>>();
        queue.sort_by_key(|e| (e.time, e.priority, e.sequence));
        let frame_transit_budgets = queue
            .iter()
            .filter_map(|e| match &e.event {
                Event::Network {
                    event:
                        sns_protocols::Event {
                            action: sns_protocols::Action::Receive { frame, .. },
                            ..
                        },
                    ..
                } => Some((e.sequence, frame.transit_budget())),
                _ => None,
            })
            .collect();
        state_value::value(&State {
            runtime_profile: if self.network.wait_factor().is_default() {
                "sns-runtime-state-v1"
            } else {
                "sns-runtime-state-v2"
            },
            project_and_scenario: &self.scenario,
            run_id: &self.run_id,
            generation: self.generation,
            initial_seed: self.seed,
            virtual_time_us: Decimal(self.now),
            revision: self.revision,
            next_sequence: self.next_sequence,
            run_state: self.state,
            limits: self.limits,
            rng_algorithm: "splitmix64-v1",
            rng_state: self.rng.0,
            queue,
            frame_transit_budgets,
            network: &self.network,
            applications: &self.apps,
            command_receipts: &self.receipts,
            trace: &self.trace,
            recording_position: RecordingPosition {
                operations: self.recording.operations.len(),
                recorded_scheduler_events: self.recording.events,
                estimated_bytes: self.recording.bytes,
                overflow: self.recording.overflow,
            },
        })
        .map_err(|_| Diagnostic::new("checkpoint.invalid_state", "state"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_contains_queue_budget_and_deterministic_protocol_and_application_tables() {
        let project: Project =
            serde_json::from_str(include_str!("../../../fixtures/valid/lan.json")).unwrap();
        let engine = Engine::from_project(project, 42, Limits::default()).unwrap();
        let a = engine.checkpoint_state().unwrap();
        let b = engine.checkpoint_state().unwrap();
        assert_eq!(a, b);
        for key in [
            "queue",
            "frame_transit_budgets",
            "network",
            "applications",
            "rng_state",
            "command_receipts",
            "trace",
        ] {
            assert!(a.get(key).is_some(), "{key}");
        }
        assert_eq!(a["virtual_time_us"], "0");
        assert_eq!(a["rng_algorithm"], "splitmix64-v1");
    }
}
