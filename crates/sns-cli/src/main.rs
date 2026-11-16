mod mission_cli;
mod render_cli;
mod report_cli;
mod runtime_cli;
use sns_engine::{Engine, Limits, Termination};
use sns_project::{atomic_write, load_project, migrate_v0};
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
            "Sinesi — Simple network simulator\n\nUsage:\n  sinesi run <scenario.json> [--seed N] [--rtt-factor 1..5] [--until-us N] [--max-events N] [--trace FILE] [--checkpoint FILE] [--recording FILE]\n  sinesi resume <checkpoint.json> [--continuation recording.json] [--until-us N] [--max-events N] [--trace FILE] [--checkpoint FILE] [--recording FILE]\n  sinesi replay <recording.json> [--trace FILE] [--checkpoint FILE]\n  sinesi ping <project.json> --source ID --target IPv4 [--count N] [--ttl N] [--rtt-factor 1..5] [--trace FILE]\n  sinesi render-png <project.json> --output <network.png> [--view icons|diagram] [--size 256..4096] [--background transparent|canvas]\n  sinesi report-pdf <input.json> --output <report.pdf> [--input project|checkpoint] [--view icons|diagram] [--language fr|en|es|it|pt|de]\n  sinesi validate <project.json>\n  sinesi migrate <v0-project.json>\n  sinesi import-filius <source.fls> --output <project.json> --report <report.json>\n  sinesi mission-check <mission.json> [--project P] [--trace FILE] [--hints N] [--language fr|en|es|it|pt|de] [--output report.json]\n\nrun: scenario 1 = isolated J0 prototype; scenario 2 = scheduled simulated network commands. Replay is offline."
        );
        return Ok(0);
    }
    let path = args
        .get(1)
        .map(PathBuf::from)
        .ok_or("missing document path")?;
    if args[0] == "import-filius" {
        return run_import(&path, &args[2..]);
    }
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
        "ping" => run_ping(&path, &args[2..]),
        "mission-check" => mission_cli::execute(&path, &args[2..]),
        "render-png" => render_cli::execute(&path, &args[2..]),
        "report-pdf" => report_cli::execute(&path, &args[2..]),
        "run" | "resume" | "replay" => runtime_cli::execute(&args[0], &path, &args[2..]),
        _ => Err("unknown command or unexpected arguments; use --help".into()),
    }
}

fn run_ping(path: &std::path::Path, args: &[String]) -> Result<u8, String> {
    use sns_model::*;
    let mut source = None;
    let mut target = None;
    let mut count = 1_usize;
    let mut ttl = 64_u8;
    let mut wait_factor = sns_model::WaitFactor::default();
    let mut trace = None;
    let mut seen = std::collections::BTreeSet::new();
    let mut flags = args.chunks_exact(2);
    for pair in &mut flags {
        if !seen.insert(&pair[0]) {
            return Err(format!("duplicate option: {}", pair[0]));
        }
        match pair[0].as_str() {
            "--rtt-factor" => {
                wait_factor =
                    WaitFactor::try_from(pair[1].parse::<u8>().map_err(|_| "invalid --rtt-factor")?)
                        .map_err(|e| e.to_string())?
            }
            "--source" => source = Some(Id::new(&pair[1]).map_err(|e| e.to_string())?),
            "--target" => target = Some(pair[1].clone()),
            "--count" => count = pair[1].parse().map_err(|_| "invalid --count")?,
            "--ttl" => ttl = pair[1].parse().map_err(|_| "invalid --ttl")?,
            "--trace" => trace = Some(PathBuf::from(&pair[1])),
            flag => return Err(format!("unknown option: {flag}")),
        }
    }
    if !flags.remainder().is_empty() {
        return Err("missing option value".into());
    }
    if !(1..=1024).contains(&count) {
        return Err("--count must be 1..1024".into());
    }
    let source = source.ok_or("missing --source")?;
    let destination = target.ok_or("missing --target")?;
    let project = load_project(path).map_err(|e| e.to_string())?;
    let mut e = Engine::from_project_with_wait_factor(project, 42, Limits::default(), wait_factor)
        .map_err(|e| e.to_string())?;
    for index in 0..count {
        e.command(CommandEnvelope {
            api_version: 1,
            command_id: Id::new(format!("ping-{index}")).unwrap(),
            project_id: e.project().project_id.clone(),
            expected_revision: Decimal(0),
            run_id: e.run_id().clone(),
            command: Command::Ping {
                source: source.clone(),
                destination: destination.clone(),
                ttl,
            },
        })
        .map_err(|e| e.to_string())?;
        let report = e
            .run(
                100_000,
                Some(
                    e.now()
                        .checked_add(wait_factor.scale(3_000_000).map_err(|e| e.to_string())?)
                        .ok_or("time overflow")?,
                ),
            )
            .map_err(|e| e.to_string())?;
        if report.termination == Termination::EventBudget {
            return Err("event budget exceeded".into());
        }
    }
    if let Some(path) = trace {
        atomic_write(&path, &e.canonical_trace()).map_err(|e| e.to_string())?;
    }
    let replies = e
        .trace()
        .iter()
        .filter(|f| f.kind == "icmp.echo.reply")
        .count();
    println!(
        "{}",
        serde_json::json!({"sent":count,"replies":replies,"arp_requests":e.trace().iter().filter(|f|f.kind=="arp.request").count(),"time_us":Decimal(e.now()),"trace_sha256":e.trace_hash()})
    );
    Ok(if replies == count { 0 } else { 1 })
}

fn run_import(source: &std::path::Path, args: &[String]) -> Result<u8, String> {
    use std::io::Read as _;
    let mut output = None;
    let mut report = None;
    let mut pairs = args.chunks_exact(2);
    for pair in &mut pairs {
        match pair[0].as_str() {
            "--output" if output.is_none() => output = Some(PathBuf::from(&pair[1])),
            "--report" if report.is_none() => report = Some(PathBuf::from(&pair[1])),
            _ => return Err("invalid or duplicate import option".into()),
        }
    }
    if !pairs.remainder().is_empty() {
        return Err("missing import option value".into());
    }
    let output = output.ok_or("missing --output")?;
    let report = report.ok_or("missing --report")?;
    let absolute = |p: &std::path::Path| -> Result<PathBuf, String> {
        let parent = p
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(std::path::Path::new("."));
        Ok(std::fs::canonicalize(parent)
            .map_err(|e| e.to_string())?
            .join(p.file_name().ok_or("invalid output path")?))
    };
    let original = std::fs::canonicalize(source).map_err(|e| e.to_string())?;
    if absolute(&output)? == original
        || absolute(&report)? == original
        || absolute(&output)? == absolute(&report)?
    {
        return Err("import paths must be distinct".into());
    }
    if !std::fs::metadata(source)
        .map_err(|e| e.to_string())?
        .is_file()
    {
        return Err("import source must be a regular file".into());
    }
    let mut bytes = vec![];
    std::fs::File::open(source)
        .map_err(|e| e.to_string())?
        .take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let result = sns_filius_import::import(
        &bytes,
        source
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("import.fls"),
    )
    .map_err(|e| e.to_string())?;
    // A report failure cannot replace the project; the source archive is never a destination.
    atomic_write(
        &report,
        &serde_json::to_vec_pretty(&result.report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    sns_project::save_project(&output, &result.project).map_err(|e| e.to_string())?;
    println!(
        "imported: {} devices, {} links; {} unconverted fields; report: {}",
        result.report.counts.devices,
        result.report.counts.links,
        result.report.unconverted_fields.len(),
        report.display()
    );
    Ok(0)
}
