//! Explicit offline run/checkpoint/replay commands. No host bridge is constructed here.
use sns_engine::{Checkpoint, Engine, Limits, Recording, Termination};
use sns_project::{atomic_write_runtime, load_scenario, read_runtime_document};
use std::path::{Path, PathBuf};

pub(super) fn execute(mode: &str, source: &Path, args: &[String]) -> Result<u8, String> {
    let mut seed = None;
    let mut wait_factor = sns_model::WaitFactor::default();
    let mut until = None;
    let mut max_events = 100_000;
    let mut trace = None;
    let mut checkpoint = None;
    let mut recording = None;
    let mut continuation = None;
    let mut seen = std::collections::BTreeSet::new();
    let mut pairs = args.chunks_exact(2);
    for pair in &mut pairs {
        if !seen.insert(pair[0].as_str()) {
            return Err(format!("duplicate option: {}", pair[0]));
        }
        match pair[0].as_str() {
            "--rtt-factor" if mode == "run" => {
                wait_factor = sns_model::WaitFactor::try_from(
                    pair[1].parse::<u8>().map_err(|_| "invalid --rtt-factor")?,
                )
                .map_err(|e| e.to_string())?;
            }
            "--seed" if mode == "run" => {
                seed = Some(pair[1].parse::<u64>().map_err(|_| "invalid --seed")?)
            }
            "--until-us" if mode != "replay" => {
                until = Some(pair[1].parse::<u64>().map_err(|_| "invalid --until-us")?)
            }
            "--max-events" if mode != "replay" => {
                max_events = pair[1]
                    .parse::<usize>()
                    .map_err(|_| "invalid --max-events")?
            }
            "--trace" => trace = Some(PathBuf::from(&pair[1])),
            "--checkpoint" => checkpoint = Some(PathBuf::from(&pair[1])),
            "--recording" if mode != "replay" => recording = Some(PathBuf::from(&pair[1])),
            "--continuation" if mode == "resume" => continuation = Some(PathBuf::from(&pair[1])),
            _ => return Err(format!("unknown option: {}", pair[0])),
        }
    }
    if !pairs.remainder().is_empty() {
        return Err("missing option value".into());
    }
    let mut paths = vec![source];
    paths.extend(continuation.as_deref());
    paths.extend(trace.as_deref());
    paths.extend(checkpoint.as_deref());
    paths.extend(recording.as_deref());
    distinct_paths(&paths)?;
    let mut engine = match mode {
        "run" => Engine::new_with_wait_factor(
            load_scenario(source).map_err(|e| e.to_string())?,
            seed,
            Limits::default(),
            wait_factor,
        ),
        "replay" => {
            Recording::from_bytes(&read_runtime_document(source).map_err(|e| e.to_string())?)
                .and_then(|r| r.replay())
        }
        "resume" => {
            let point =
                Checkpoint::from_bytes(&read_runtime_document(source).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if let Some(path) = continuation {
                let full = Recording::from_bytes(
                    &read_runtime_document(&path).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                point.replay_remaining(&full)
            } else {
                point.restore()
            }
        }
        _ => return Err("invalid runtime command".into()),
    }
    .map_err(|e| e.to_string())?;
    let (summary, exhausted) = if mode == "replay" {
        (
            serde_json::json!({"replayed":true,"time_us":sns_model::Decimal(engine.now()),"pending_events":engine.pending(),"trace_sha256":engine.trace_hash()}),
            false,
        )
    } else {
        let report = engine.run(max_events, until).map_err(|e| e.to_string())?;
        let exhausted = report.termination == Termination::EventBudget;
        (
            serde_json::to_value(report).map_err(|e| e.to_string())?,
            exhausted,
        )
    };
    // Validate and encode every requested export before replacing any destination.
    let point_bytes = checkpoint
        .as_ref()
        .map(|_| {
            engine
                .checkpoint()
                .map_err(|e| e.to_string())
                .and_then(|v| serde_json::to_vec(&v).map_err(|e| e.to_string()))
        })
        .transpose()?;
    let recording_bytes = recording
        .as_ref()
        .map(|_| {
            engine
                .recording()
                .map_err(|e| e.to_string())
                .and_then(|v| serde_json::to_vec(&v).map_err(|e| e.to_string()))
        })
        .transpose()?;
    for (path, bytes) in [
        (checkpoint, point_bytes),
        (recording, recording_bytes),
        (trace, Some(engine.canonical_trace())),
    ] {
        if let (Some(path), Some(bytes)) = (path, bytes) {
            atomic_write_runtime(&path, &bytes).map_err(|e| e.to_string())?;
        }
    }
    println!(
        "{}",
        serde_json::to_string(&summary).map_err(|e| e.to_string())?
    );
    Ok(if exhausted { 2 } else { 0 })
}

pub(super) fn distinct_paths(paths: &[&Path]) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for path in paths {
        let resolved = if path.exists() {
            path.canonicalize()
        } else {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            parent
                .canonicalize()
                .map(|p| p.join(path.file_name().unwrap_or_default()))
        }
        .map_err(|e| e.to_string())?;
        if !seen.insert(resolved) {
            return Err("runtime inputs and outputs must use distinct paths".into());
        }
    }
    Ok(())
}
