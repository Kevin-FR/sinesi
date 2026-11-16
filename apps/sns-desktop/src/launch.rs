//! Native paths stay OsString/PathBuf; option values are validated by the shared resolver.
use eframe::egui;
use serde_json::{Value, json};
use sns_model::WaitFactor;
use sns_settings::configuration::{
    ConfigurationDiagnostic, Overrides, ResolvedConfiguration, Source, Sources,
};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Default, Debug, PartialEq)]
pub(super) struct Launch {
    pub prototype: bool,
    pub help: bool,
    pub version: bool,
    pub verbose: bool,
    log: bool,
    native_theme: bool,
    small_screen: bool,
    log_file: Option<PathBuf>,
    pub check_config: bool,
    pub project: Option<PathBuf>,
    config: Option<PathBuf>,
    settings: Option<PathBuf>,
    session: Option<PathBuf>,
    working_directory: Option<PathBuf>,
    values: Vec<(&'static str, String)>,
}
pub(super) struct Prepared {
    small_screen: bool,
    pub theme: egui::ThemePreference,
    pub log_file: Option<PathBuf>,
    pub wait_factor: WaitFactor,
    pub configuration: ResolvedConfiguration,
    pub project: Option<PathBuf>,
    pub session: Option<PathBuf>,
}
impl Prepared {
    pub fn viewport(&self) -> egui::ViewportBuilder {
        let display = &self.configuration.effective.display;
        egui::ViewportBuilder::default()
            .with_title("Sinesi — Simple network simulator")
            .with_inner_size([
                f32::from(display.window_width),
                f32::from(display.window_height),
            ])
            .with_min_inner_size(if self.small_screen {
                [640., 480.]
            } else {
                [840., 600.]
            })
    }
}
struct OptionSpec {
    names: &'static [&'static str],
    metavar: &'static str,
    description: &'static str,
}
const OPTIONS: &[OptionSpec] = &[
    OptionSpec {
        names: &["-n", "--native-theme"],
        metavar: "",
        description: "launch_native_theme",
    },
    OptionSpec {
        names: &["-s", "--small-screen"],
        metavar: "",
        description: "launch_small_screen",
    },
    OptionSpec {
        names: &["-v", "--verbose"],
        metavar: "",
        description: "launch_verbose",
    },
    OptionSpec {
        names: &["-l", "--log"],
        metavar: "",
        description: "launch_log",
    },
    OptionSpec {
        names: &["--log-file"],
        metavar: "PATH",
        description: "launch_log_file",
    },
    OptionSpec {
        names: &["-r", "--rtt-factor"],
        metavar: "1..5",
        description: "launch_rtt_factor",
    },
    OptionSpec {
        names: &["-h", "--help"],
        metavar: "",
        description: "launch_help",
    },
    OptionSpec {
        names: &["--version"],
        metavar: "",
        description: "launch_version",
    },
    OptionSpec {
        names: &["--check-config"],
        metavar: "",
        description: "launch_check_config",
    },
    OptionSpec {
        names: &["--config"],
        metavar: "PATH",
        description: "launch_config",
    },
    OptionSpec {
        names: &["--settings"],
        metavar: "PATH",
        description: "launch_settings",
    },
    OptionSpec {
        names: &["--session-dir"],
        metavar: "PATH",
        description: "launch_session_dir",
    },
    OptionSpec {
        names: &["-w", "--workdir"],
        metavar: "PATH",
        description: "launch_workdir",
    },
    OptionSpec {
        names: &["--language"],
        metavar: "fr|en|es|it|pt|de",
        description: "launch_language",
    },
    OptionSpec {
        names: &["--ui-scale"],
        metavar: "80..200",
        description: "launch_ui_scale",
    },
    OptionSpec {
        names: &["--font"],
        metavar: "default|noto_sans",
        description: "launch_font",
    },
    OptionSpec {
        names: &["--view"],
        metavar: "icons|diagram",
        description: "launch_view",
    },
    OptionSpec {
        names: &["--layout"],
        metavar: "inspector_right|inspector_left|tabs",
        description: "launch_layout",
    },
    OptionSpec {
        names: &["--window-width"],
        metavar: "840..3840",
        description: "launch_window_width",
    },
    OptionSpec {
        names: &["--window-height"],
        metavar: "600..2160",
        description: "launch_window_height",
    },
    OptionSpec {
        names: &["--prototype-j0"],
        metavar: "",
        description: "launch_prototype",
    },
];
pub(super) fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Launch, &'static str> {
    let mut result = Launch::default();
    let mut positional = false;
    let mut arguments = arguments.into_iter();
    let mut count = 0;
    while let Some(argument) = arguments.next() {
        count += 1;
        if count > 128 {
            return Err("launch.too_many_arguments");
        }
        if !positional && argument == "--" {
            positional = true;
            continue;
        }
        if !positional && argument.to_string_lossy().starts_with('-') {
            let Some(spec) = OPTIONS
                .iter()
                .find(|s| s.names.iter().any(|name| argument == *name))
            else {
                return Err("launch.unknown_option");
            };
            if spec.metavar.is_empty() {
                match spec.names.last().copied() {
                    Some("--help") => result.help = true,
                    Some("--verbose") => result.verbose = true,
                    Some("--native-theme") => result.native_theme = true,
                    Some("--small-screen") => result.small_screen = true,
                    Some("--log") => result.log = true,
                    Some("--version") => result.version = true,
                    Some("--check-config") => result.check_config = true,
                    Some("--prototype-j0") => result.prototype = true,
                    _ => return Err("launch.unknown_option"),
                }
                continue;
            }
            let value = arguments.next().ok_or("launch.missing_value")?;
            count += 1;
            if count > 128 {
                return Err("launch.too_many_arguments");
            }
            if value == "--"
                || OPTIONS
                    .iter()
                    .any(|s| s.names.iter().any(|name| value == *name))
            {
                return Err("launch.missing_value");
            }
            match spec.names.last().copied() {
                Some("--config") => result.config = Some(value.into()),
                Some("--log-file") => result.log_file = Some(value.into()),
                Some("--settings") => result.settings = Some(value.into()),
                Some("--session-dir") => result.session = Some(value.into()),
                Some("--workdir") => result.working_directory = Some(value.into()),
                Some(key) => result
                    .values
                    .push((key, value.to_string_lossy().into_owned())),
                None => return Err("launch.unknown_option"),
            }
        } else if result.project.replace(argument.into()).is_some() {
            return Err("launch.multiple_projects");
        }
    }
    if result.prototype
        && (result.project.is_some()
            || result.check_config
            || result.small_screen
            || result.values.iter().any(|(key, _)| *key == "--rtt-factor"))
    {
        return Err("launch.prototype_conflict");
    }
    Ok(result)
}
impl Launch {
    pub fn prepare(
        &self,
        cwd: &Path,
        mut sources: Sources,
        default_session: Option<PathBuf>,
    ) -> Prepared {
        let mut issues = vec![];
        let base = match &self.working_directory {
            Some(path) => {
                let path = absolute(cwd, path);
                if path.is_dir() {
                    path
                } else {
                    issues.push(ConfigurationDiagnostic {
                        source: Source::CommandLine,
                        field: "working_directory".into(),
                        code: "configuration.invalid_directory",
                    });
                    cwd.to_owned()
                }
            }
            None => cwd.to_owned(),
        };
        if let Some(path) = &self.config {
            sources.administrator = Some(absolute(&base, path));
            sources.administrator_required = true;
        }
        if let Some(path) = &self.settings {
            sources.user = Some(absolute(&base, path));
        }
        let mut overrides = Overrides::default();
        let mut wait_factor = WaitFactor::default();
        for (key, value) in &self.values {
            let (name, numeric) = match *key {
                "--language" => {
                    overrides.language = Some(Value::String(value.clone()));
                    continue;
                }
                "--rtt-factor" => {
                    wait_factor = match value
                        .parse::<u8>()
                        .ok()
                        .and_then(|v| WaitFactor::try_from(v).ok())
                    {
                        Some(factor) => factor,
                        None => {
                            issues.push(ConfigurationDiagnostic {
                                source: Source::CommandLine,
                                field: "wait_factor".into(),
                                code: "simulation.invalid_wait_factor",
                            });
                            WaitFactor::default()
                        }
                    };
                    continue;
                }
                "--ui-scale" => ("scale_percent", true),
                "--font" => ("font", false),
                "--view" => ("representation", false),
                "--layout" => ("layout", false),
                "--window-width" => ("window_width", true),
                "--window-height" => ("window_height", true),
                _ => continue,
            };
            let value = if numeric {
                serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.clone()))
            } else {
                Value::String(value.clone())
            };
            overrides.display.insert(name.into(), value);
        }
        sources.small_screen = self.small_screen;
        sources.overrides = overrides;
        sources.administrator = sources.administrator.map(|p| absolute(&base, &p));
        sources.user = sources.user.map(|p| absolute(&base, &p));
        let mut configuration = sns_settings::configuration::resolve(sources);
        for issue in issues {
            configuration.record_diagnostic(issue.source, &issue.field, issue.code);
        }
        let log_file = self
            .log_file
            .as_ref()
            .map(|p| absolute(&base, p))
            .or_else(|| {
                self.log.then(|| {
                    let stamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos();
                    base.join(format!("sinesi-{}-{stamp}.log.jsonl", std::process::id()))
                })
            });
        Prepared {
            small_screen: self.small_screen,
            theme: if self.native_theme {
                egui::ThemePreference::System
            } else {
                egui::ThemePreference::Dark
            },
            log_file,
            wait_factor,
            configuration,
            project: self.project.as_ref().map(|p| absolute(&base, p)),
            session: self
                .session
                .as_ref()
                .map(|p| absolute(&base, p))
                .or(default_session.map(|p| absolute(&base, &p))),
        }
    }
    pub fn help_text(&self, configuration: &ResolvedConfiguration) -> String {
        let l = configuration.effective.language;
        let mut lines = vec![
            l.text("launch_usage").to_owned(),
            l.text("configuration_priority").to_owned(),
            String::new(),
        ];
        lines.extend(
            OPTIONS
                .iter()
                .filter(|s| !s.names.contains(&"--prototype-j0"))
                .map(|s| {
                    format!(
                        "  {} {}\n      {}",
                        s.names.join(", "),
                        s.metavar,
                        l.text(s.description)
                    )
                }),
        );
        lines.push(String::new());
        lines.push(l.text("launch_temporary_hint").into());
        lines.join("\n")
    }
}
fn absolute(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    }
}
pub(super) fn report(prepared: &Prepared, app: &sns_ui_egui::Workbench) -> Value {
    let mut value = json!({"startup_report_version":1,"configuration":app.startup_configuration().unwrap_or(&prepared.configuration),"project":{
        "schema_version":app.project().schema_version,"devices":app.project().devices.len(),"links":app.project().links.len(),
    }});
    if !app.runtime_wait_factor().is_default() {
        value["startup_report_version"] = json!(2);
        value["runtime"] = json!({"wait_factor":app.runtime_wait_factor()});
    }
    value
}
#[cfg(test)]
mod tests {
    use super::*;
    fn args(v: &[&str]) -> Vec<OsString> {
        v.iter().map(OsString::from).collect()
    }
    #[test]
    fn native_paths_option_boundary_and_duplicate_values_are_explicit() {
        assert_eq!(
            parse(args(&["école réseau.json"])).unwrap().project,
            Some("école réseau.json".into())
        );
        assert_eq!(
            parse(args(&["--", "-classe.json"])).unwrap().project,
            Some("-classe.json".into())
        );
        for bad in [
            vec!["a.json", "b.json"],
            vec!["--prototype-j0", "a.json"],
            vec!["--unknown"],
            vec!["--settings"],
            vec!["--prototype-j0", "--check-config"],
        ] {
            assert!(parse(args(&bad)).is_err());
        }
        let dir = tempfile::tempdir().unwrap();
        let parsed = parse(args(&[
            "--language",
            "fr",
            "--language",
            "de",
            "-w",
            dir.path().to_str().unwrap(),
            "--settings",
            "profils/école.json",
            "--",
            "classe.json",
        ]))
        .unwrap();
        let ready = parsed.prepare(dir.path(), Sources::default(), None);
        assert_eq!(ready.configuration.effective.language.label(), "Deutsch");
        assert_eq!(
            ready.configuration.user_path,
            Some(dir.path().join("profils/école.json"))
        );
        assert_eq!(ready.project, Some(dir.path().join("classe.json")));
    }
}

#[cfg(all(test, unix))]
#[test]
fn parser_keeps_non_utf8_project_and_profile_path_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let filename = OsString::from_vec(b"classe-\xff.json".to_vec());
    let profile = OsString::from_vec(b"profil-\xfe.json".to_vec());
    let parsed = parse([
        OsString::from("--settings"),
        profile.clone(),
        OsString::from("--"),
        filename.clone(),
    ])
    .unwrap();
    let ready = parsed.prepare(Path::new("/tmp"), Sources::default(), None);
    assert_eq!(ready.project, Some(PathBuf::from("/tmp").join(filename)));
    assert_eq!(
        ready.configuration.user_path,
        Some(PathBuf::from("/tmp").join(profile))
    );
}

#[cfg(test)]
#[test]
fn launch_uses_system_theme_updates_and_reduces_the_compact_viewport_minimum() {
    for (flags, follow_system) in [(vec![], false), (vec!["-n", "-s"], true)] {
        let launch = parse(flags.into_iter().map(OsString::from)).unwrap();
        let ready = launch.prepare(Path::new("."), Sources::default(), None);
        let viewport = ready.viewport();
        assert_eq!(
            viewport.min_inner_size,
            Some(if follow_system {
                egui::vec2(640., 480.)
            } else {
                egui::vec2(840., 600.)
            })
        );
        let ctx = egui::Context::default();
        ctx.set_theme(ready.theme);
        for (system_theme, effective) in [
            (None, egui::Theme::Dark),
            (
                Some(egui::Theme::Light),
                if follow_system {
                    egui::Theme::Light
                } else {
                    egui::Theme::Dark
                },
            ),
            (Some(egui::Theme::Dark), egui::Theme::Dark),
        ] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    system_theme,
                    ..Default::default()
                },
                |ui| {
                    assert_eq!(ui.ctx().theme(), effective);
                    assert_eq!(ui.visuals().dark_mode, effective == egui::Theme::Dark);
                },
            );
            output.textures_delta.clear();
        }
    }
}
