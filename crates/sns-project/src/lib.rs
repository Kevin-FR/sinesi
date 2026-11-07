//! Bounded JSON persistence. .fls execution and historical class loading are never used.
use sns_model::{Diagnostic, Project, Scenario};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const MAX_DOCUMENT_BYTES: u64 = 8 * 1_024 * 1_024;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Validation(#[from] Diagnostic),
    #[error("project.invalid_json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("project.io: {0}")]
    Io(#[from] std::io::Error),
}

pub fn parse_project(bytes: &[u8]) -> Result<Project, Error> {
    limit(bytes)?;
    let project: Project = serde_json::from_slice(bytes)?;
    project.validate()?;
    Ok(project)
}
pub fn parse_scenario(bytes: &[u8]) -> Result<Scenario, Error> {
    limit(bytes)?;
    let scenario: Scenario = serde_json::from_slice(bytes)?;
    scenario.validate()?;
    Ok(scenario)
}
fn limit(bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err(Diagnostic::new("resource.limit_exceeded", "document").into());
    }
    Ok(())
}
pub fn read_bounded(path: &Path) -> Result<Vec<u8>, Error> {
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_DOCUMENT_BYTES {
        return Err(Diagnostic::new("resource.limit_exceeded", path.display().to_string()).into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_DOCUMENT_BYTES + 1).read_to_end(&mut bytes)?;
    limit(&bytes)?;
    Ok(bytes)
}
pub fn load_project(path: &Path) -> Result<Project, Error> {
    parse_project(&read_bounded(path)?)
}
pub fn load_scenario(path: &Path) -> Result<Scenario, Error> {
    parse_scenario(&read_bounded(path)?)
}

/// Same-volume temporary file + fsync + atomic replacement. No partial target is exposed.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    limit(bytes)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}
pub fn save_project(path: &Path, project: &Project) -> Result<(), Error> {
    project.validate()?;
    let mut bytes = serde_json::to_vec_pretty(project)?;
    bytes.push(b'\n');
    atomic_write(path, &bytes)
}

/// Only the documented v0 prototype is migrated. The original is backed up before replacement.
pub fn migrate_v0(path: &Path) -> Result<PathBuf, Error> {
    let bytes = read_bounded(path)?;
    let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
    if value.get("schema_version").and_then(|v| v.as_u64()) != Some(0) {
        return Err(
            Diagnostic::new("project.unsupported_migration", path.display().to_string()).into(),
        );
    }
    let object = value
        .as_object_mut()
        .ok_or_else(|| Diagnostic::new("project.invalid_json", "root"))?;
    let title = object
        .remove("title")
        .ok_or_else(|| Diagnostic::new("project.invalid_json", "title"))?;
    if object.contains_key("name") {
        return Err(Diagnostic::new("project.invalid_json", "ambiguous name/title").into());
    }
    object.insert("name".into(), title);
    object.insert("schema_version".into(), serde_json::json!(1));
    let project: Project = serde_json::from_value(value)?;
    project.validate()?;
    let backup = path.with_extension("v0.backup.json");
    // create_new refuses to overwrite an earlier original, even on a repeated attempt.
    let mut original = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup)?;
    original.write_all(&bytes)?;
    original.sync_all()?;
    save_project(path, &project)?;
    Ok(backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    const PROJECT: &[u8] = include_bytes!("../../../fixtures/valid/project.json");
    #[test]
    fn project_roundtrip_and_failed_replacement_preserve_original() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("network.sinesi.json");
        let project = parse_project(PROJECT).unwrap();
        save_project(&path, &project).unwrap();
        assert_eq!(load_project(&path).unwrap(), project);
        let original = fs::read(&path).unwrap();
        let mut invalid = project.clone();
        invalid.schema_version = 77;
        assert!(save_project(&path, &invalid).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        // A failed atomic rename must leave the existing target directory intact.
        let occupied = directory.path().join("occupied");
        fs::create_dir(&occupied).unwrap();
        fs::write(occupied.join("sentinel"), b"keep").unwrap();
        assert!(save_project(&occupied, &project).is_err());
        assert_eq!(fs::read(occupied.join("sentinel")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }
    #[test]
    fn migration_preserves_exact_source_and_refuses_unrecognized_versions() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("legacy.json");
        let mut value: serde_json::Value = serde_json::from_slice(PROJECT).unwrap();
        let name = value.as_object_mut().unwrap().remove("name").unwrap();
        value["title"] = name;
        value["schema_version"] = 0.into();
        let original = serde_json::to_vec_pretty(&value).unwrap();
        fs::write(&path, &original).unwrap();
        let backup = migrate_v0(&path).unwrap();
        assert_eq!(fs::read(&backup).unwrap(), original);
        assert_eq!(
            load_project(&path).unwrap(),
            parse_project(PROJECT).unwrap()
        );
        assert!(migrate_v0(&path).is_err());
        assert_eq!(fs::read(&backup).unwrap(), original);
    }
    #[test]
    fn untrusted_documents_are_rejected_before_use() {
        let mut value: serde_json::Value = serde_json::from_slice(PROJECT).unwrap();
        value["devices"][0]["interfaces"][0]["ipv4"]["address"] = "999.1.1.1".into();
        assert!(parse_project(&serde_json::to_vec(&value).unwrap()).is_err());
        assert!(parse_project(&vec![b' '; MAX_DOCUMENT_BYTES as usize + 1]).is_err());
        assert!(
            parse_project(b"<!DOCTYPE x [<!ENTITY e SYSTEM 'file:///etc/passwd'>]><x>&e;</x>")
                .is_err()
        );
        assert!(parse_project(b"PK\x03\x04").is_err());
    }
}
