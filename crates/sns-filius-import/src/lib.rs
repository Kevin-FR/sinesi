//! Bounded Filius ZIP/XML data importer. Never evaluates JavaBeans, classes or embedded code.
mod annotations;
mod archive;
mod convert;
mod mail;
mod mail_content;
mod network;
mod security;
mod xml;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sns_model::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldConversion {
    pub source: String,
    pub target: String,
    pub rule: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportIssue {
    pub code: String,
    pub source: String,
    pub detail: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreservedEntry {
    pub name: String,
    pub bytes: Decimal,
    pub sha256: String,
    pub resources: Vec<Id>,
    pub executable: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub devices: usize,
    pub interfaces: usize,
    pub links: usize,
    pub applications: usize,
    pub files: usize,
    pub annotations: usize,
    pub resources: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportReport {
    pub format_version: u32,
    pub source_name: String,
    pub source_sha256: String,
    pub counts: Counts,
    pub converted: Vec<FieldConversion>,
    pub unconverted_fields: Vec<String>,
    pub issues: Vec<ImportIssue>,
    pub unknown_classes: Vec<String>,
    pub preserved: Vec<PreservedEntry>,
    pub execution_performed: bool,
    pub functional_equivalence_verified: bool,
}
#[derive(Clone, Debug)]
pub struct ImportResult {
    pub project: Project,
    pub report: ImportReport,
}
fn error(code: &str, target: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, target)
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn id(s: impl Into<String>) -> Id {
    Id::new(s).expect("generated opaque identifier")
}
/// Mapped fields are converted; every remaining property and original byte stays auditable.
pub fn import(bytes: &[u8], source_name: &str) -> Result<ImportResult, Diagnostic> {
    if source_name.len() > 1024 || source_name.chars().any(char::is_control) {
        return Err(error("import.invalid_name", "source"));
    }
    let entries = archive::read(bytes)?;
    let configs = entries
        .keys()
        .filter(|n| n.ends_with("/konfiguration.xml"))
        .collect::<Vec<_>>();
    if configs.len() != 1 {
        return Err(error("import.invalid_xml", "configuration_count"));
    }
    let text = std::str::from_utf8(&entries[configs[0]])
        .map_err(|_| error("import.invalid_xml", "encoding"))?;
    let mut source = xml::Source::parse(text)?;
    let digest = hash(bytes);
    let mut report = ImportReport {
        format_version: 1,
        source_name: source_name.into(),
        source_sha256: digest.clone(),
        counts: Counts::default(),
        converted: vec![],
        unconverted_fields: vec![],
        issues: vec![],
        unknown_classes: vec![],
        preserved: vec![],
        execution_performed: false,
        functional_equivalence_verified: false,
    };
    let mut project = convert::project(&mut source, &mut report, &digest)?;
    report.converted.extend(source.converted());
    report.unconverted_fields = source.unconverted();
    report.unknown_classes = source.unknown_classes();
    for (i, (name, data)) in entries.iter().enumerate() {
        let executable = name.to_ascii_lowercase().ends_with(".java")
            || name.to_ascii_lowercase().ends_with(".class")
            || name.to_ascii_lowercase().ends_with(".jar");
        let mut refs = vec![];
        for (part, chunk) in data.chunks(MAX_BODY_BYTES).enumerate() {
            let rid = id(format!("legacy-resource-{i}-{part}"));
            refs.push(rid.clone());
            project.resources.push(Resource {
                id: rid.clone(),
                media_type: "application/octet-stream".into(),
                data_base64: STANDARD.encode(chunk),
            });
            if executable {
                project.extensions.push(Extension {
                    id: id(format!("legacy-extension-{i}-{part}")),
                    api_version: 0,
                    resource_id: rid,
                    enabled: false,
                });
            }
        }
        report.preserved.push(PreservedEntry {
            name: name.clone(),
            bytes: Decimal(data.len() as u64),
            sha256: hash(data),
            resources: refs,
            executable,
        });
        if executable {
            report.issues.push(ImportIssue {
                code: "import.inactive_extension".into(),
                source: name.clone(),
                detail: "embedded_executable_inactive".into(),
            });
        }
    }
    report.counts = Counts {
        devices: project.devices.len(),
        interfaces: project.devices.iter().map(|d| d.interfaces.len()).sum(),
        links: project.links.len(),
        applications: project.devices.iter().map(|d| d.applications.len()).sum(),
        files: project.devices.iter().map(|d| d.files.len()).sum(),
        annotations: project.annotations.len(),
        resources: project.resources.len() + 1,
    };
    if !report.unconverted_fields.is_empty() {
        report.issues.push(ImportIssue {
            code: "import.unconverted_fields".into(),
            source: "konfiguration.xml".into(),
            detail: report.unconverted_fields.len().to_string(),
        });
    }
    let data = serde_json::to_vec(&report).map_err(|_| error("import.invalid_xml", "report"))?;
    if data.len() > MAX_BODY_BYTES {
        return Err(error("resource.limit_exceeded", "import_report"));
    }
    project.resources.push(Resource {
        id: id("legacy-import-report"),
        media_type: "application/vnd.sinesi.import-report+json".into(),
        data_base64: STANDARD.encode(data),
    });
    project.validate()?;
    if serde_json::to_vec(&project).map_or(true, |b| b.len() > 8 * 1024 * 1024) {
        return Err(error("resource.limit_exceeded", "imported_project"));
    }
    Ok(ImportResult { project, report })
}
