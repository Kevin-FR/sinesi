//! Headless report creation from a project or a verified checkpoint.
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
pub(super) fn execute(source: &Path, args: &[String]) -> Result<u8, String> {
    let mut output = None;
    let mut options = sns_report::ReportOptions::default();
    let mut checkpoint = false;
    let mut seen = BTreeSet::new();
    let mut pairs = args.chunks_exact(2);
    for pair in &mut pairs {
        if !seen.insert(&pair[0]) {
            return Err(format!("duplicate option: {}", pair[0]));
        }
        match pair[0].as_str() {
            "--output" => output = Some(PathBuf::from(&pair[1])),
            "--view" => {
                options.diagram = match pair[1].as_str() {
                    "icons" => false,
                    "diagram" => true,
                    _ => return Err("--view must be icons or diagram".into()),
                }
            }
            "--language" => {
                options.language = sns_i18n::Language::ALL
                    .into_iter()
                    .find(|l| l.code() == pair[1])
                    .ok_or("--language must be fr, en, es, it, pt or de")?
            }
            "--input" => {
                checkpoint = match pair[1].as_str() {
                    "project" => false,
                    "checkpoint" => true,
                    _ => return Err("--input must be project or checkpoint".into()),
                }
            }
            _ => return Err(format!("unknown option: {}", pair[0])),
        }
    }
    if !pairs.remainder().is_empty() {
        return Err("missing report option value".into());
    }
    let output = output.ok_or("missing --output")?;
    super::runtime_cli::distinct_paths(&[source, &output])?;
    let snapshot = if checkpoint {
        let bytes = sns_project::read_runtime_document(source).map_err(|e| e.to_string())?;
        let point = sns_engine::Checkpoint::from_bytes(&bytes).map_err(|e| e.to_string())?;
        let engine = point.restore().map_err(|e| e.to_string())?;
        sns_report::Snapshot::capture(&engine)
    } else {
        sns_report::Snapshot::configured(
            sns_project::load_project(source).map_err(|e| e.to_string())?,
        )
    };
    let report = sns_report::Report::build(&snapshot, options).map_err(|e| e.to_string())?;
    let pdf = sns_report::render_pdf(&report).map_err(|e| e.to_string())?;
    sns_project::atomic_write_runtime(&output, &pdf.bytes).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({"pages":pdf.pages,"bytes":pdf.bytes.len(),"packet_rows":pdf.packet_rows,"interface_groups":pdf.interface_groups,"clipped_texts":pdf.clipped_scene_texts,"trace_partial":pdf.trace_partial,"view":if options.diagram{"diagram"}else{"icons"},"language":options.language.code(),"revision":snapshot.revision,"time_us":snapshot.time_us.to_string()})
    );
    Ok(0)
}
