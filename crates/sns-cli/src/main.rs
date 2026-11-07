use sns_engine::{Engine, Limits, Termination};
use sns_project::{atomic_write, load_project, load_scenario, migrate_v0};
use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    match execute(env::args().skip(1).collect()) {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}
fn execute(args: Vec<String>) -> Result<u8, String> {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h" | "help") {
        println!(
            "Sinesi — Simple network simulator\n\nUsage:\n  sinesi run <scenario.json> [--seed N] [--until-us N] [--max-events N] [--trace FILE]\n  sinesi validate <project.json>\n  sinesi migrate <v0-project.json>\n\nJ0: deterministic prototype messages; TCP/IP is not implemented yet."
        );
        return Ok(0);
    }
    let path = args
        .get(1)
        .map(PathBuf::from)
        .ok_or("missing document path")?;
    match args[0].as_str() {
        "validate" if args.len() == 2 => {
            let project = load_project(&path).map_err(|e| e.to_string())?;
            println!("valid: {}", project.name);
            Ok(0)
        }
        "migrate" if args.len() == 2 => {
            let backup = migrate_v0(&path).map_err(|e| e.to_string())?;
            println!("backup: {}", backup.display());
            Ok(0)
        }
        "run" => {
            let mut seed = None;
            let mut until = None;
            let mut max_events = 100_000;
            let mut trace = None;
            let mut seen = std::collections::BTreeSet::new();
            let mut flags = args[2..].chunks_exact(2);
            for pair in &mut flags {
                if !seen.insert(&pair[0]) {
                    return Err(format!("duplicate option: {}", pair[0]));
                }
                match pair[0].as_str() {
                    "--seed" => seed = Some(pair[1].parse::<u64>().map_err(|_| "invalid --seed")?),
                    "--until-us" => {
                        until = Some(pair[1].parse::<u64>().map_err(|_| "invalid --until-us")?)
                    }
                    "--max-events" => {
                        max_events = pair[1]
                            .parse::<usize>()
                            .map_err(|_| "invalid --max-events")?
                    }
                    "--trace" => trace = Some(PathBuf::from(&pair[1])),
                    other => return Err(format!("unknown option: {other}")),
                }
            }
            if !flags.remainder().is_empty() {
                return Err("missing option value".into());
            }
            let scenario = load_scenario(&path).map_err(|e| e.to_string())?;
            let mut engine =
                Engine::new(scenario, seed, Limits::default()).map_err(|e| e.to_string())?;
            let report = engine.run(max_events, until).map_err(|e| e.to_string())?;
            if let Some(path) = trace {
                atomic_write(&path, &engine.canonical_trace()).map_err(|e| e.to_string())?;
            }
            println!(
                "{}",
                serde_json::to_string(&report).map_err(|e| e.to_string())?
            );
            Ok(if report.termination == Termination::EventBudget {
                2
            } else {
                0
            })
        }
        _ => Err("unknown command or unexpected arguments; use --help".into()),
    }
}
