//! Presentation filters never mutate the canonical simulation trace.
use sns_engine::TraceEvent;
use sns_i18n::Language;
use sns_model::Id;
#[derive(Clone)]
pub struct TrafficFilter {
    pub query: sns_trace::Query,
    pub interface: Option<Id>,
    pub layers: [bool; 4],
    pub follow: bool,
    pub after: u64,
    pub layered: bool,
}
impl Default for TrafficFilter {
    fn default() -> Self {
        Self {
            query: Default::default(),
            interface: None,
            layers: [true; 4],
            follow: true,
            after: 0,
            layered: false,
        }
    }
}
pub use sns_trace::layer;
pub fn interface(event: &TraceEvent) -> &str {
    event
        .payload
        .get("interface")
        .and_then(|v| v.as_str())
        .unwrap_or("—")
}
pub fn direction(event: &TraceEvent, l: Language) -> &str {
    if event.kind.ends_with(".sent") {
        l.text("sent")
    } else if event.kind.ends_with(".received") {
        l.text("received_direction")
    } else {
        "—"
    }
}
pub fn protocol(event: &TraceEvent) -> &str {
    event
        .payload
        .get("frame")
        .and_then(|v| v.get("payload"))
        .and_then(|v| v.get("packet"))
        .and_then(|v| v.get("protocol"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            event
                .payload
                .get("frame")
                .and_then(|v| v.get("payload"))
                .and_then(|v| v.get("protocol"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or_else(|| event.kind.split('.').next().unwrap_or("—"))
}
impl TrafficFilter {
    pub fn matches(&self, event: &TraceEvent, text: &str) -> bool {
        self.query.matches(event)
            && event.sequence.0 > self.after
            && self.layers[layer(event)]
            && self
                .interface
                .as_ref()
                .is_none_or(|i| interface(event) == i.as_str())
            && (text.is_empty()
                || event.kind.contains(text)
                || event.device_id.as_str().contains(text)
                || event.correlation_id.as_str().contains(text)
                || serde_json::to_string(&event.payload)
                    .unwrap()
                    .contains(text))
    }
    pub fn export_text(&self, events: &[TraceEvent], text: &str, l: Language) -> String {
        let mut out = [
            "number",
            "time_us",
            "event",
            "device",
            "interface_label",
            "direction",
            "protocol",
            "layer",
            "correlation",
        ]
        .map(|k| l.text(k))
        .join("\t");
        out.push('\n');
        for e in events.iter().filter(|e| self.matches(e, text)) {
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                e.sequence.0,
                e.time_us.0,
                e.kind,
                e.device_id.as_str(),
                interface(e),
                direction(e, l),
                protocol(e),
                l.text(
                    [
                        "link_layer",
                        "network_layer",
                        "transport_layer",
                        "application_layer"
                    ][layer(e)]
                ),
                e.correlation_id.as_str()
            ));
        }
        out
    }
}
