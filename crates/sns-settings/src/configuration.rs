//! Startup layering. Administrator values are defaults, not enforced restrictions.
use crate::{Settings, display::DisplayPreferences};
use serde::Serialize;
use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};

pub const MAX_BYTES: u64 = crate::MAX_SETTINGS_BYTES;
const MAX_DIAGNOSTICS: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Builtin,
    Administrator,
    User,
    CommandLine,
}
impl Source {
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Builtin => "configuration_builtin",
            Self::Administrator => "configuration_administrator",
            Self::User => "configuration_user",
            Self::CommandLine => "configuration_command_line",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConfigurationDiagnostic {
    pub source: Source,
    pub field: String,
    pub code: &'static str,
}
impl ConfigurationDiagnostic {
    pub fn localized(&self, language: sns_i18n::Language) -> String {
        language
            .format(
                "configuration_diagnostic",
                &[
                    ("source", language.text(self.source.label_key())),
                    ("field", &self.field),
                    (
                        "message",
                        language
                            .try_text(self.code)
                            .unwrap_or_else(|| language.text("configuration.invalid_value")),
                    ),
                ],
            )
            .unwrap_or_else(|_| self.code.to_owned())
    }
}
#[derive(Clone, Debug, Default)]
pub struct Overrides {
    pub language: Option<Value>,
    pub display: Map<String, Value>,
}
#[derive(Clone, Debug, Default)]
pub struct Sources {
    pub small_screen: bool,
    pub administrator: Option<PathBuf>,
    pub administrator_required: bool,
    pub user: Option<PathBuf>,
    pub overrides: Overrides,
}
impl Sources {
    pub fn native() -> Self {
        Self {
            small_screen: false,
            administrator: default_administrator_path(),
            user: crate::default_path(),
            ..Default::default()
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct ResolvedConfiguration {
    pub effective: Settings,
    /// Snapshot before command-line overrides, used for user edits and persistence.
    pub persistent: Settings,
    pub origins: BTreeMap<String, Source>,
    pub diagnostics: Vec<ConfigurationDiagnostic>,
    pub user_writable: bool,
    #[serde(skip)]
    pub administrator_path: Option<PathBuf>,
    #[serde(skip)]
    pub user_path: Option<PathBuf>,
}
impl Default for ResolvedConfiguration {
    fn default() -> Self {
        let mut origins = BTreeMap::from([("language".into(), Source::Builtin)]);
        if let Ok(Value::Object(display)) = serde_json::to_value(DisplayPreferences::default()) {
            origins.extend(
                display
                    .keys()
                    .map(|key| (format!("display.{key}"), Source::Builtin)),
            );
        }
        Self {
            effective: Settings::default(),
            persistent: Settings::default(),
            origins,
            diagnostics: vec![],
            user_writable: true,
            administrator_path: None,
            user_path: None,
        }
    }
}
impl ResolvedConfiguration {
    pub fn record_diagnostic(&mut self, source: Source, field: &str, code: &'static str) {
        if source == Source::User {
            self.user_writable = false;
        }
        if self.diagnostics.len() < MAX_DIAGNOSTICS {
            self.diagnostics.push(ConfigurationDiagnostic {
                source,
                field: field.into(),
                code,
            });
        } else if let Some(last) = self.diagnostics.last_mut() {
            *last = ConfigurationDiagnostic {
                source,
                field: "configuration".into(),
                code: "configuration.too_many_issues",
            };
        }
    }
    fn language(&mut self, value: &Value, source: Source) {
        match serde_json::from_value(value.clone()) {
            Ok(language) => {
                self.effective.language = language;
                self.origins.insert("language".into(), source);
            }
            Err(_) => self.record_diagnostic(source, "language", "configuration.invalid_value"),
        }
    }
    fn display(&mut self, patch: &Map<String, Value>, source: Source, complete: bool) {
        let Ok(Value::Object(mut current)) = serde_json::to_value(&self.effective.display) else {
            return;
        };
        let known: Vec<_> = current.keys().cloned().collect();
        if complete {
            // `level` was added to profile version 2 as an optional field (D-015).
            for key in known.iter().filter(|key| key.as_str() != "level") {
                if !patch.contains_key(key) {
                    self.record_diagnostic(
                        source,
                        &format!("display.{key}"),
                        "configuration.missing_field",
                    );
                }
            }
        }
        for (key, value) in patch {
            if !current.contains_key(key) {
                self.record_diagnostic(source, "display", "configuration.unknown_field");
                continue;
            }
            let mut candidate = current.clone();
            candidate.insert(key.clone(), value.clone());
            let mut candidate = Value::Object(candidate);
            crate::normalize_integer_numbers(&mut candidate);
            let Some(candidate) = candidate.as_object().cloned() else {
                continue;
            };
            match serde_json::from_value::<DisplayPreferences>(Value::Object(candidate.clone())) {
                Ok(display) if display.validate().is_ok() => {
                    self.effective.display = display;
                    current = candidate;
                    self.origins.insert(format!("display.{key}"), source);
                }
                _ => self.record_diagnostic(
                    source,
                    &format!("display.{key}"),
                    "configuration.invalid_value",
                ),
            }
        }
    }
    fn file(&mut self, path: &Path, source: Source, required: bool) {
        let value = match read_json(path) {
            Ok(Some(value)) => value,
            Ok(None) => {
                if required {
                    self.record_diagnostic(source, "configuration", "configuration.missing_file");
                }
                return;
            }
            Err(code) => {
                self.record_diagnostic(source, "configuration", code);
                return;
            }
        };
        let Some(object) = value.as_object() else {
            self.record_diagnostic(source, "configuration", "configuration.invalid_object");
            return;
        };
        let version_key = if source == Source::User {
            "settings_version"
        } else {
            "configuration_version"
        };
        let version = object.get(version_key).and_then(Value::as_u64);
        if !matches!(
            (source, version),
            (Source::User, Some(1 | 2)) | (Source::Administrator, Some(1))
        ) {
            self.record_diagnostic(source, version_key, "configuration.unsupported_version");
            return;
        }
        for key in object.keys() {
            let progress = source == Source::User && key == "missions";
            if key != version_key && key != "language" && key != "display" && !progress {
                // Never reflect unknown keys or input values into diagnostics.
                self.record_diagnostic(source, "configuration", "configuration.unknown_field");
            }
        }
        if let Some(language) = object.get("language") {
            self.language(language, source);
        } else if source == Source::User {
            self.record_diagnostic(source, "language", "configuration.missing_field");
        }
        if source == Source::User
            && let Some(missions) = object.get("missions")
        {
            match serde_json::from_value(missions.clone()) {
                Ok(missions) if crate::validate_missions(&missions).is_ok() => {
                    self.effective.missions = missions;
                }
                _ => self.record_diagnostic(source, "missions", "configuration.invalid_value"),
            }
        }
        match object.get("display") {
            Some(Value::Object(display)) if source != Source::User || version == Some(2) => {
                self.display(display, source, source == Source::User)
            }
            Some(_) => self.record_diagnostic(source, "display", "configuration.invalid_value"),
            None if source == Source::User && version == Some(2) => {
                self.record_diagnostic(source, "display", "configuration.missing_field")
            }
            None => {}
        }
    }
}
pub fn resolve(sources: Sources) -> ResolvedConfiguration {
    let mut resolved = ResolvedConfiguration {
        administrator_path: sources.administrator.clone(),
        user_path: sources.user.clone(),
        ..Default::default()
    };
    if let Some(path) = sources.administrator {
        resolved.file(&path, Source::Administrator, sources.administrator_required);
    }
    if let Some(path) = sources.user {
        resolved.file(&path, Source::User, false);
    }
    resolved.persistent = resolved.effective.clone();
    if sources.small_screen {
        resolved.display(
            &Map::from_iter([
                ("window_width".into(), Value::from(840)),
                ("window_height".into(), Value::from(600)),
                ("scale_percent".into(), Value::from(90)),
                ("application_scale_percent".into(), Value::from(85)),
                ("layout".into(), Value::from("tabs")),
            ]),
            Source::CommandLine,
            false,
        );
    }
    if let Some(language) = sources.overrides.language {
        resolved.language(&language, Source::CommandLine);
    }
    resolved.display(&sources.overrides.display, Source::CommandLine, false);
    resolved
}
fn read_json(path: &Path) -> Result<Option<Value>, &'static str> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("configuration.read_failed"),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "configuration.read_failed")?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("configuration.too_large");
    }
    serde_json::from_slice(&bytes)
        .map(|mut value| {
            crate::normalize_integer_numbers(&mut value);
            Some(value)
        })
        .map_err(|_| "configuration.invalid_json")
}
pub fn default_administrator_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("SINESI_ADMIN_CONFIG") {
        return Some(path.into());
    }
    if cfg!(target_os = "windows") {
        std::env::var_os("PROGRAMDATA").map(|p| PathBuf::from(p).join("Sinesi/config.json"))
    } else if cfg!(target_os = "macos") {
        Some(PathBuf::from(
            "/Library/Application Support/Sinesi/config.json",
        ))
    } else {
        Some(PathBuf::from("/etc/sinesi/config.json"))
    }
}
