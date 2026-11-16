//! Host preferences adapter. Language never changes the simulated project.
pub mod configuration;
pub mod diagnostics;
pub mod display;
pub mod help;
use serde::{Deserialize, Serialize};
use sns_i18n::Language;
use sns_model::Diagnostic;
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};
pub const MAX_SETTINGS_BYTES: u64 = 65536;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub settings_version: u32,
    pub language: Language,
    pub display: display::DisplayPreferences,
    /// Best stars per completed mission; optional in version 2 and omitted when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub missions: BTreeMap<String, u8>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            settings_version: 2,
            language: Language::Fr,
            display: Default::default(),
            missions: BTreeMap::new(),
        }
    }
}
pub const MAX_MISSIONS: usize = 64;
/// Mission progress keeps identifiers of the portable mission format and 1–3 stars.
pub fn validate_missions(missions: &BTreeMap<String, u8>) -> Result<(), &'static str> {
    let valid = missions.len() <= MAX_MISSIONS
        && missions.iter().all(|(id, stars)| {
            !id.is_empty()
                && id.len() <= 64
                && id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                && (1..=3).contains(stars)
        });
    if valid {
        Ok(())
    } else {
        Err("settings.invalid_missions")
    }
}
pub fn load(path: &Path) -> Result<Settings, String> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Settings::default()),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_SETTINGS_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_SETTINGS_BYTES {
        return Err(Diagnostic::new("resource.limit_exceeded", "settings").to_string());
    }
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    normalize_integer_numbers(&mut value);
    let settings: Settings = match value.get("settings_version").and_then(|v| v.as_u64()) {
        Some(1) => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct VersionOne {
                #[serde(rename = "settings_version")]
                _version: u32,
                language: Language,
            }
            let old: VersionOne = serde_json::from_value(value).map_err(|e| e.to_string())?;
            Settings {
                language: old.language,
                ..Default::default()
            }
        }
        Some(2) => serde_json::from_value(value).map_err(|e| e.to_string())?,
        _ => return Err(Diagnostic::new("settings.unsupported_version", "settings").to_string()),
    };
    settings.display.validate().map_err(str::to_owned)?;
    validate_missions(&settings.missions).map_err(str::to_owned)?;
    Ok(settings)
}
// JSON Schema integers include integral decimal/exponent spellings such as 125.0.
// Every supported numeric preference is well within this exact conversion range.
pub(crate) fn normalize_integer_numbers(value: &mut serde_json::Value) {
    use serde_json::Value;
    match value {
        Value::Number(number) if number.is_f64() => {
            if let Some(number) = number.as_f64()
                && number.fract() == 0.
                && (i32::MIN as f64..=i32::MAX as f64).contains(&number)
            {
                *value = Value::from(number as i64);
            }
        }
        Value::Array(values) => {
            for value in values {
                normalize_integer_numbers(value);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                normalize_integer_numbers(value);
            }
        }
        _ => {}
    }
}
pub fn save(path: &Path, settings: &Settings) -> Result<(), String> {
    if settings.settings_version != 2 {
        return Err(Diagnostic::new("settings.unsupported_version", "settings").to_string());
    }
    settings.display.validate().map_err(str::to_owned)?;
    validate_missions(&settings.missions).map_err(str::to_owned)?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    sns_project::atomic_write(
        path,
        &serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
pub fn default_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("SINESI_SETTINGS_PATH") {
        return Some(path.into());
    }
    if cfg!(target_os = "windows") {
        return std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Sinesi/settings.json"));
    }
    if cfg!(target_os = "macos") {
        return std::env::var_os("HOME")
            .map(|p| PathBuf::from(p).join("Library/Application Support/Sinesi/settings.json"));
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
        .map(|p| p.join("sinesi/settings.json"))
}
/// Recovery is local state, separate from preferences and user project destinations.
pub fn default_session_directory() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("SINESI_SESSION_DIR") {
        return Some(path.into());
    }
    if cfg!(target_os = "windows") {
        return std::env::var_os("LOCALAPPDATA")
            .or_else(|| std::env::var_os("APPDATA"))
            .map(|p| PathBuf::from(p).join("Sinesi/sessions"));
    }
    if cfg!(target_os = "macos") {
        return std::env::var_os("HOME")
            .map(|p| PathBuf::from(p).join("Library/Application Support/Sinesi/sessions"));
    }
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/state")))
        .map(|p| p.join("sinesi/sessions"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn language_survives_restart_outside_the_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences/settings.json");
        assert_eq!(load(&path).unwrap(), Settings::default());
        for language in Language::ALL {
            save(
                &path,
                &Settings {
                    language,
                    ..Settings::default()
                },
            )
            .unwrap();
            assert_eq!(load(&path).unwrap().language, language);
        }
    }
    #[test]
    fn malformed_and_oversized_settings_do_not_get_overwritten_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        for bytes in [
            b"{\"settings_version\":99,\"language\":\"de\"}".to_vec(),
            vec![0; 65537],
            b"{\"settings_version\":1,\"language\":\"xx\"}".to_vec(),
        ] {
            std::fs::write(&path, &bytes).unwrap();
            assert!(load(&path).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
    }
}
