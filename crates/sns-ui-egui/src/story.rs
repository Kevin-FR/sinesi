//! Plain-language journey of packets, derived from the canonical trace.
//! Presentation only: it never advances or changes the simulation.
use sns_engine::TraceEvent;
use sns_i18n::Language;
use sns_model::Project;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// A request leaves or travels forward.
    Go,
    /// Address resolution or switching, needed for the journey.
    Info,
    /// The expected answer arrived.
    Success,
    /// The packet could not get through.
    Failure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub event_index: usize,
    pub tone: Tone,
    pub text: String,
    /// Technical identifiers, shown at the Expert level only.
    pub detail: String,
}

fn text<'a>(event: &'a TraceEvent, key: &str) -> &'a str {
    event
        .payload
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("—")
}

fn device_name(project: &Project, id: &str) -> String {
    project
        .devices
        .iter()
        .find(|d| d.id.as_str() == id)
        .map(|d| sns_topology::display_label(project, d))
        .unwrap_or_else(|| id.to_owned())
}

/// One sentence for an event that matters to a learner; `None` for internal facts.
pub fn step(
    project: &Project,
    language: Language,
    index: usize,
    event: &TraceEvent,
) -> Option<Step> {
    let device = device_name(project, event.device_id.as_str());
    let reason = text(event, "reason");
    let (tone, key, params, detail): (Tone, &str, Vec<(&str, String)>, String) =
        match event.kind.as_str() {
            "icmp.echo.requested" => (
                Tone::Go,
                "story_ping_sent",
                vec![("target", text(event, "destination").to_owned())],
                format!(
                    "TTL {}",
                    event
                        .payload
                        .get("ttl")
                        .map_or("—".into(), |v| v.to_string())
                ),
            ),
            "arp.request" => (
                Tone::Info,
                "story_arp_request",
                vec![("target", text(event, "target_ip").to_owned())],
                text(event, "interface").to_owned(),
            ),
            "arp.reply" => (
                Tone::Info,
                "story_arp_reply",
                vec![],
                text(event, "interface").to_owned(),
            ),
            "switch.forward" => (Tone::Info, "story_switch_forward", vec![], ports(event)),
            "switch.flood" => (Tone::Info, "story_switch_flood", vec![], ports(event)),
            "ipv4.forwarded" => (
                Tone::Go,
                "story_forwarded",
                vec![("next_hop", text(event, "next_hop").to_owned())],
                format!("{} → {}", text(event, "incoming"), text(event, "interface")),
            ),
            "icmp.echo.received" => (
                Tone::Go,
                "story_echo_received",
                vec![("source", text(event, "source").to_owned())],
                String::new(),
            ),
            "icmp.echo.reply" => {
                let micros: u64 = text(event, "rtt_us").parse().unwrap_or(0);
                (
                    Tone::Success,
                    "story_echo_reply",
                    vec![
                        ("source", text(event, "source").to_owned()),
                        ("ms", format!("{}", micros.div_ceil(1000))),
                    ],
                    format!("rtt {micros} µs"),
                )
            }
            "icmp.error" if reason == "routing.no_route" => {
                (Tone::Failure, "story_no_route", vec![], reason.to_owned())
            }
            "icmp.error" if reason == "arp.timeout" => {
                (Tone::Failure, "story_arp_failed", vec![], reason.to_owned())
            }
            "icmp.error" => (Tone::Failure, "story_error", vec![], reason.to_owned()),
            "icmp.timeout" => (Tone::Failure, "story_timeout", vec![], reason.to_owned()),
            "dns.query.sent" => (
                Tone::Info,
                "story_dns_query",
                vec![
                    ("name", text(event, "name").to_owned()),
                    ("server", text(event, "server").to_owned()),
                ],
                text(event, "record_type").to_owned(),
            ),
            "dns.response.received" => {
                let response = event.payload.get("response");
                let field = |key: &str| {
                    response
                        .and_then(|r| r.get(key))
                        .and_then(|v| v.as_str())
                        .unwrap_or("—")
                        .to_owned()
                };
                let value = response
                    .and_then(|r| r.get("records"))
                    .and_then(|r| r.as_array())
                    .and_then(|r| r.first())
                    .and_then(|r| r.get("value"))
                    .and_then(|v| v.as_str());
                match (field("code").as_str(), value) {
                    ("ok", Some(value)) => (
                        Tone::Success,
                        "story_dns_answer",
                        vec![("name", field("name")), ("address", value.to_owned())],
                        "code ok".into(),
                    ),
                    (code, _) => (
                        Tone::Failure,
                        "story_dns_failed",
                        vec![("name", field("name"))],
                        format!("code {code}"),
                    ),
                }
            }
            "routing.dropped" if reason == "ipv4.ttl_expired" => {
                (Tone::Failure, "story_ttl", vec![], reason.to_owned())
            }
            "routing.dropped" => (Tone::Failure, "story_dropped", vec![], reason.to_owned()),
            _ => return None,
        };
    let mut values: Vec<(&str, &str)> = vec![("device", device.as_str())];
    values.extend(params.iter().map(|(k, v)| (*k, v.as_str())));
    let text = language.format(key, &values).ok()?;
    let detail = if detail.is_empty() {
        event.kind.clone()
    } else {
        format!("{} · {detail}", event.kind)
    };
    Some(Step {
        event_index: index,
        tone,
        text,
        detail,
    })
}

fn ports(event: &TraceEvent) -> String {
    let egress = event
        .payload
        .get("egress")
        .and_then(|v| v.as_array())
        .map(|ports| {
            ports
                .iter()
                .filter_map(|p| p.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    format!("{} → {egress}", text(event, "ingress"))
}

/// The most recent narrated steps, oldest first; scans backwards so a long trace stays cheap.
pub fn recent(
    project: &Project,
    language: Language,
    trace: &[TraceEvent],
    limit: usize,
) -> Vec<Step> {
    let mut steps: Vec<_> = trace
        .iter()
        .enumerate()
        .rev()
        .filter_map(|(i, e)| step(project, language, i, e))
        .take(limit)
        .collect();
    steps.reverse();
    steps
}
