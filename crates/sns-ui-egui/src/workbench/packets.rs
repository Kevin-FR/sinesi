//! Packets drawn travelling along cables. Each flight replays one recorded
//! `ethernet.sent` fact; the clock here only paces the drawing, never the engine.
use super::*;

/// Delay between successive departures, so a multi-hop journey reads in order.
const STAGGER: f64 = 0.35;
const DURATION: f64 = 0.6;
/// Older departures are skipped rather than queued forever at high speed.
const MAX_BACKLOG: f64 = 3.;
const MAX_FLIGHTS: usize = 48;

#[derive(Clone)]
pub(super) struct Flight {
    from: Id,
    to: Id,
    start: f64,
    color: Color32,
}

/// Request orange, reply blue, address resolution purple, everything else teal.
fn frame_color(event: &sns_engine::TraceEvent) -> Color32 {
    let payload = event.payload.get("frame").and_then(|f| f.get("payload"));
    let protocol = payload
        .and_then(|p| p.get("protocol"))
        .and_then(|p| p.as_str());
    let icmp = payload
        .and_then(|p| p.get("icmp"))
        .and_then(|i| i.get("message_type"))
        .and_then(|t| t.as_u64());
    match (protocol, icmp) {
        (Some("arp"), _) => Color32::from_rgb(0x8B, 0x5C, 0xF6),
        (_, Some(8)) => Color32::from_rgb(0xEA, 0x58, 0x0C),
        (_, Some(0)) => Color32::from_rgb(0x25, 0x63, 0xEB),
        _ => Color32::from_rgb(0x14, 0xB8, 0xA6),
    }
}

impl Workbench {
    pub(super) fn queue_flight(&mut self, event_index: usize, now: f64) {
        let event = &self.worker.view.trace[event_index];
        if event.kind != "ethernet.sent" {
            return;
        }
        let port = crate::traffic::interface(event);
        let project = &self.worker.view.project;
        let Some(link) = project
            .links
            .iter()
            .find(|l| l.endpoints.iter().any(|p| p.as_str() == port))
        else {
            return;
        };
        let Some(other) = link.endpoints.iter().find(|p| p.as_str() != port) else {
            return;
        };
        let Some(to) = project
            .devices
            .iter()
            .find(|d| d.interfaces.iter().any(|i| &i.id == other))
            .map(|d| d.id.clone())
        else {
            return;
        };
        let start = (self.packets.last_start + STAGGER).max(now);
        if start - now > MAX_BACKLOG {
            return;
        }
        self.packets.last_start = start;
        let flight = Flight {
            from: event.device_id.clone(),
            to,
            start,
            color: frame_color(event),
        };
        self.packets.flights.push(flight);
        if self.packets.flights.len() > MAX_FLIGHTS {
            self.packets.flights.remove(0);
        }
    }
    pub(super) fn clear_flights(&mut self) {
        self.packets = Default::default();
    }
    /// Number of packets currently drawn or waiting to depart.
    pub fn packets_in_flight(&self) -> usize {
        self.packets.flights.len()
    }
    pub(super) fn paint_flights(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        point: impl Fn(&Id) -> Option<Pos2>,
    ) {
        let now = ui.input(|i| i.time);
        self.packets.flights.retain(|f| now < f.start + DURATION);
        if self.packets.flights.is_empty() {
            return;
        }
        let painter = ui.painter_at(rect);
        let radius = (9. * self.zoom).clamp(5., 12.);
        for flight in &self.packets.flights {
            let t = ((now - flight.start) / DURATION) as f32;
            if !(0. ..=1.).contains(&t) {
                continue;
            }
            let (Some(a), Some(b)) = (point(&flight.from), point(&flight.to)) else {
                continue;
            };
            let at = a + (b - a) * t;
            painter.circle_filled(at, radius * 1.8, flight.color.gamma_multiply(0.25));
            painter.circle(at, radius, flight.color, Stroke::new(2., Color32::WHITE));
        }
        ui.ctx().request_repaint();
    }
}

#[derive(Default)]
pub(super) struct Packets {
    flights: Vec<Flight>,
    last_start: f64,
}
