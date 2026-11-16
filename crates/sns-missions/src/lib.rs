//! Assessable missions: a portable exercise format, objective checks over a
//! project and its simulation trace, stars and an exportable report.
//! Checks only read data; they never run, advance or edit a simulation.
use serde::{Deserialize, Serialize};
use sns_engine::TraceEvent;
use sns_model::{Id, Project};
use std::collections::{BTreeMap, BTreeSet};

pub const MISSION_VERSION: u32 = 1;
pub const REPORT_VERSION: u32 = 1;
const LANGUAGES: [&str; 6] = ["fr", "en", "es", "it", "pt", "de"];
const MAX_BYTES: usize = 4 * 1024 * 1024;

/// Learner-facing text by language code; French is required as the fallback.
pub type Text = BTreeMap<String, String>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mission {
    pub mission_version: u32,
    pub id: String,
    pub chapter: u8,
    pub order: u8,
    pub title: Text,
    pub brief: Text,
    pub project: Project,
    pub objectives: Vec<Objective>,
    pub hints: Vec<Text>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Objective {
    pub id: String,
    pub text: Text,
    /// Shown when the objective is checked and not met.
    pub explain: Text,
    pub check: Check,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Check {
    /// Saved configuration: the device's default gateway.
    Gateway { device: Id, equals: String },
    /// Saved configuration: the device's DNS server.
    DnsServer { device: Id, equals: String },
    /// Saved topology: a cable joins the two devices.
    Connected { a: Id, b: Id },
    /// Trace: the device received an echo reply from this address.
    EchoReply { device: Id, from: String },
    /// Trace: the device received a successful DNS answer for this name.
    DnsResolved { device: Id, name: String },
    /// Trace: the application opened its listening socket on the device.
    ServiceListening { device: Id, application: Id },
}

impl Check {
    fn devices(&self) -> Vec<&Id> {
        match self {
            Check::Gateway { device, .. }
            | Check::DnsServer { device, .. }
            | Check::EchoReply { device, .. }
            | Check::DnsResolved { device, .. }
            | Check::ServiceListening { device, .. } => vec![device],
            Check::Connected { a, b } => vec![a, b],
        }
    }
    /// Checks read from the trace hold once observed during a run.
    pub fn uses_trace(&self) -> bool {
        matches!(
            self,
            Check::EchoReply { .. } | Check::DnsResolved { .. } | Check::ServiceListening { .. }
        )
    }
}

fn error(code: &str, target: &str) -> String {
    format!("{code}:{target}")
}

fn valid_text(text: &Text) -> bool {
    text.get("fr").is_some_and(|t| !t.trim().is_empty())
        && text
            .iter()
            .all(|(k, v)| LANGUAGES.contains(&k.as_str()) && v.len() <= 2000)
}

/// Read and validate a mission file.
pub fn parse(bytes: &[u8]) -> Result<Mission, String> {
    if bytes.len() > MAX_BYTES {
        return Err(error("resource.limit_exceeded", "mission"));
    }
    let mission: Mission = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    mission.validate()?;
    Ok(mission)
}

impl Mission {
    pub fn validate(&self) -> Result<(), String> {
        if self.mission_version != MISSION_VERSION {
            return Err(error("mission.unsupported_version", &self.id));
        }
        let id_ok = |id: &str| {
            !id.is_empty()
                && id.len() <= 64
                && id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        };
        if !id_ok(&self.id) {
            return Err(error("mission.invalid_id", &self.id));
        }
        if !valid_text(&self.title) || !valid_text(&self.brief) {
            return Err(error("mission.invalid_text", &self.id));
        }
        if self.objectives.is_empty() || self.objectives.len() > 12 || self.hints.len() > 5 {
            return Err(error("mission.invalid_size", &self.id));
        }
        self.project.validate().map_err(|e| e.to_string())?;
        let mut seen = BTreeSet::new();
        for objective in &self.objectives {
            if !id_ok(&objective.id) || !seen.insert(&objective.id) {
                return Err(error("mission.invalid_objective", &objective.id));
            }
            if !valid_text(&objective.text) || !valid_text(&objective.explain) {
                return Err(error("mission.invalid_text", &objective.id));
            }
            for device in objective.check.devices() {
                if self.project.device(device).is_none() {
                    return Err(error("mission.unknown_device", device.as_str()));
                }
            }
        }
        if !self.hints.iter().all(valid_text) {
            return Err(error("mission.invalid_text", "hints"));
        }
        Ok(())
    }
}

/// Text in the requested language, French otherwise.
pub fn text<'a>(text: &'a Text, language: &str) -> &'a str {
    text.get(language)
        .or_else(|| text.get("fr"))
        .map(String::as_str)
        .unwrap_or("")
}

fn same(value: Option<&str>, expected: &str) -> bool {
    value.is_some_and(|v| v.trim() == expected.trim())
}

/// Whether one objective holds for this project and trace.
pub fn met(check: &Check, project: &Project, trace: &[TraceEvent]) -> bool {
    match check {
        Check::Gateway { device, equals } => project
            .device(device)
            .is_some_and(|d| same(d.network.default_gateway.as_deref(), equals)),
        Check::DnsServer { device, equals } => project
            .device(device)
            .is_some_and(|d| same(d.network.dns_server.as_deref(), equals)),
        Check::Connected { a, b } => project.links.iter().any(|link| {
            let owners: Vec<_> = link
                .endpoints
                .iter()
                .filter_map(|port| project.device_for_interface(port).map(|d| &d.id))
                .collect();
            owners.contains(&a) && owners.contains(&b)
        }),
        Check::EchoReply { device, from } => trace.iter().any(|e| {
            e.kind == "icmp.echo.reply"
                && &e.device_id == device
                && e.payload.get("source").and_then(|v| v.as_str()) == Some(from.as_str())
        }),
        Check::DnsResolved { device, name } => trace.iter().any(|e| {
            let response = e.payload.get("response");
            e.kind == "dns.response.received"
                && &e.device_id == device
                && response
                    .and_then(|r| r.get("code"))
                    .and_then(|v| v.as_str())
                    == Some("ok")
                && response
                    .and_then(|r| r.get("name"))
                    .and_then(|v| v.as_str())
                    == Some(name.as_str())
                && response
                    .and_then(|r| r.get("records"))
                    .and_then(|v| v.as_array())
                    .is_some_and(|records| !records.is_empty())
        }),
        Check::ServiceListening {
            device,
            application,
        } => trace.iter().any(|e| {
            e.kind == "socket.bound"
                && &e.device_id == device
                && e.payload.get("owner").and_then(|v| v.as_str()) == Some(application.as_str())
        }),
    }
}

/// Stars for a completed mission: three, minus one per hint, at least one.
pub fn stars(hints_used: usize) -> u8 {
    3usize.saturating_sub(hints_used).max(1) as u8
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectiveResult {
    pub id: String,
    pub done: bool,
    pub text: String,
    /// The author's explanation, present only for an unmet objective.
    pub explanation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub report_version: u32,
    pub mission: String,
    pub language: String,
    pub complete: bool,
    pub hints_used: usize,
    pub stars: Option<u8>,
    pub objectives: Vec<ObjectiveResult>,
}

/// The exportable correction for a learner's project and trace.
/// `observed` lists objectives already met earlier in the session.
pub fn report(
    mission: &Mission,
    project: &Project,
    trace: &[TraceEvent],
    observed: &BTreeSet<String>,
    hints_used: usize,
    language: &str,
) -> Report {
    let objectives: Vec<_> = mission
        .objectives
        .iter()
        .map(|o| {
            let done =
                (o.check.uses_trace() && observed.contains(&o.id)) || met(&o.check, project, trace);
            ObjectiveResult {
                id: o.id.clone(),
                done,
                text: text(&o.text, language).to_owned(),
                explanation: (!done).then(|| text(&o.explain, language).to_owned()),
            }
        })
        .collect();
    let complete = objectives.iter().all(|o| o.done);
    Report {
        report_version: REPORT_VERSION,
        mission: mission.id.clone(),
        language: language.to_owned(),
        complete,
        hints_used: hints_used.min(mission.hints.len()),
        stars: complete.then(|| stars(hints_used.min(mission.hints.len()))),
        objectives,
    }
}

/// Missions shipped with Sinesi, ordered by chapter then order.
pub fn builtin() -> Vec<Mission> {
    let mut missions: Vec<Mission> = [
        include_bytes!("../missions/forgotten-gateway.json").as_slice(),
        include_bytes!("../missions/forgotten-cable.json").as_slice(),
        include_bytes!("../missions/broken-directory.json").as_slice(),
    ]
    .into_iter()
    .map(|bytes| parse(bytes).expect("checked builtin mission"))
    .collect();
    missions.sort_by_key(|m| (m.chapter, m.order));
    missions
}
