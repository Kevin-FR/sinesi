//! Read-only projections of immutable simulation events; no clock, I/O or GUI.
mod journey;
pub use journey::{Journey, layer_parts};
use serde_json::Value;
use sns_engine::TraceEvent;
use sns_model::{Diagnostic, Id};

pub const MAX_EXPORT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    pub device: Option<Id>,
    pub from_us: Option<u64>,
    pub through_us: Option<u64>,
    pub protocol: String,
    pub address: String,
    pub correlation: String,
}
impl Query {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if self
            .from_us
            .zip(self.through_us)
            .is_some_and(|(a, b)| a > b)
            || self.protocol.len() > 64
            || self.address.len() > 128
            || self.correlation.len() > 128
        {
            return Err(Diagnostic::new("trace.invalid_filter", "query"));
        }
        Ok(())
    }
    /// Bounds are inclusive; protocol ignores ASCII case, other fields are exact.
    pub fn matches(&self, e: &TraceEvent) -> bool {
        self.validate().is_ok()
            && self.device.as_ref().is_none_or(|d| *d == e.device_id)
            && self.from_us.is_none_or(|t| e.time_us.0 >= t)
            && self.through_us.is_none_or(|t| e.time_us.0 <= t)
            && (self.correlation.trim().is_empty()
                || self.correlation.trim() == e.correlation_id.as_str())
            && (self.protocol.trim().is_empty() || has_protocol(e, self.protocol.trim()))
            && (self.address.trim().is_empty()
                || e.payload
                    .iter()
                    .any(|(key, v)| has_address(key, v, self.address.trim())))
    }
}
fn has_address(key: &str, value: &Value, address: &str) -> bool {
    match value {
        Value::String(s) => {
            matches!(
                key,
                "address"
                    | "source"
                    | "destination"
                    | "sender_ip"
                    | "target_ip"
                    | "source_ip"
                    | "destination_ip"
                    | "next_hop"
                    | "gateway"
                    | "mac"
                    | "source_mac"
                    | "destination_mac"
                    | "sender_mac"
                    | "target_mac"
            ) && s.eq_ignore_ascii_case(address)
        }
        Value::Object(o) => o.iter().any(|(k, v)| has_address(k, v, address)),
        Value::Array(a) => a.iter().any(|v| has_address(key, v, address)),
        _ => false,
    }
}
fn value_has_protocol(v: &Value, protocol: &str) -> bool {
    match v {
        Value::Object(o) => {
            o.get("protocol").and_then(Value::as_str).is_some_and(|p| {
                p.eq_ignore_ascii_case(protocol)
                    || (p == "ipv4_transport" && protocol.eq_ignore_ascii_case("ipv4"))
            }) || (o.contains_key("icmp") && protocol.eq_ignore_ascii_case("icmp"))
                || o.values().any(|v| value_has_protocol(v, protocol))
        }
        Value::Array(a) => a.iter().any(|v| value_has_protocol(v, protocol)),
        _ => false,
    }
}
pub fn has_protocol(e: &TraceEvent, protocol: &str) -> bool {
    e.kind
        .split('.')
        .next()
        .is_some_and(|p| p.eq_ignore_ascii_case(protocol))
        || (e.payload.contains_key("icmp") && protocol.eq_ignore_ascii_case("icmp"))
        || e.payload.values().any(|v| value_has_protocol(v, protocol))
        || e.payload
            .get("protocol")
            .and_then(Value::as_str)
            .is_some_and(|p| p.eq_ignore_ascii_case(protocol))
}
pub fn layer(e: &TraceEvent) -> usize {
    let p = e.kind.split('.').next().unwrap_or("");
    match p {
        "application" | "filesystem" | "dhcp" | "dns" | "http" | "admin" | "mail" | "smtp"
        | "pop3" | "p2p" => 3,
        "tcp" | "udp" | "socket" => 2,
        "icmp" | "ipv4" | "routing" | "nat" | "firewall" | "security" => 1,
        _ => 0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Jsonl,
    Csv,
}

/// Export precisely the supplied iterator, in its order, without renumbering.
pub fn export<'a>(
    events: impl IntoIterator<Item = &'a TraceEvent>,
    format: ExportFormat,
) -> Result<Vec<u8>, Diagnostic> {
    let mut out = Vec::new();
    if format == ExportFormat::Csv {
        out.extend_from_slice(b"run_id,event_id,event_schema_version,sequence,time_us,type,device_id,caused_by,correlation_id,payload_json\r\n");
    }
    for e in events {
        let record = match format {
            ExportFormat::Jsonl => {
                let mut line = serde_json::to_vec(e).expect("event serialization");
                line.push(b'\n');
                line
            }
            ExportFormat::Csv => {
                let fields = [
                    e.run_id.as_str().to_owned(),
                    e.event_id.as_str().to_owned(),
                    e.event_schema_version.to_string(),
                    e.sequence.0.to_string(),
                    e.time_us.0.to_string(),
                    e.kind.clone(),
                    e.device_id.as_str().into(),
                    e.caused_by.as_ref().map(Id::as_str).unwrap_or("").into(),
                    e.correlation_id.as_str().into(),
                    serde_json::to_string(&e.payload).expect("payload serialization"),
                ];
                let mut line = fields
                    .iter()
                    .map(|s| format!("\"{}\"", s.replace('"', "\"\"")))
                    .collect::<Vec<_>>()
                    .join(",");
                line.push_str("\r\n");
                line.into_bytes()
            }
        };
        if out.len().saturating_add(record.len()) > MAX_EXPORT_BYTES {
            return Err(Diagnostic::new("resource.limit_exceeded", "trace_export"));
        }
        out.extend_from_slice(&record);
    }
    Ok(out)
}
