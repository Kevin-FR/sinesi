//! `sinesi mission-check`: the exportable correction of a mission, without a window.
use sns_project::{atomic_write, load_project};
use std::{collections::BTreeSet, path::Path};

const LANGUAGES: [&str; 6] = ["fr", "en", "es", "it", "pt", "de"];
const MAX_TRACE_BYTES: u64 = 256 * 1024 * 1024;

/// Exit 0 when every objective is met, 2 when the mission is incomplete.
pub fn execute(mission: &Path, args: &[String]) -> Result<u8, String> {
    let mut project = None;
    let mut trace = None;
    let mut output = None;
    let mut hints = 0_usize;
    let mut language = "fr".to_owned();
    let mut seen = BTreeSet::new();
    let mut flags = args.chunks_exact(2);
    for pair in &mut flags {
        if !seen.insert(&pair[0]) {
            return Err(format!("duplicate option: {}", pair[0]));
        }
        match pair[0].as_str() {
            "--project" => project = Some(pair[1].clone()),
            "--trace" => trace = Some(pair[1].clone()),
            "--output" => output = Some(pair[1].clone()),
            "--hints" => hints = pair[1].parse().map_err(|_| "invalid --hints")?,
            "--language" if LANGUAGES.contains(&pair[1].as_str()) => language = pair[1].clone(),
            "--language" => return Err("--language must be fr|en|es|it|pt|de".into()),
            flag => return Err(format!("unknown option: {flag}")),
        }
    }
    if !flags.remainder().is_empty() {
        return Err("missing option value".into());
    }
    let bytes = std::fs::read(mission).map_err(|e| e.to_string())?;
    let mission = sns_missions::parse(&bytes)?;
    // Without --project, the mission's own starting network is checked.
    let project = match project {
        Some(path) => load_project(Path::new(&path)).map_err(|e| e.to_string())?,
        None => mission.project.clone(),
    };
    let events = match trace {
        Some(path) => read_trace(Path::new(&path))?,
        None => vec![],
    };
    let report = sns_missions::report(
        &mission,
        &project,
        &events,
        &BTreeSet::new(),
        hints,
        &language,
    );
    let mut json = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    json.push(b'\n');
    match output {
        Some(path) => atomic_write(Path::new(&path), &json).map_err(|e| e.to_string())?,
        None => print!("{}", String::from_utf8_lossy(&json)),
    }
    Ok(if report.complete { 0 } else { 2 })
}

fn read_trace(path: &Path) -> Result<Vec<sns_engine::TraceEvent>, String> {
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX_TRACE_BYTES {
        return Err("resource.limit_exceeded:trace".into());
    }
    std::fs::read_to_string(path)
        .map_err(|e| e.to_string())?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(n, line)| {
            serde_json::from_str(line).map_err(|e| format!("trace line {}: {e}", n + 1))
        })
        .collect()
}
