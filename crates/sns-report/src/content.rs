//! Project data and packet facts become an immutable paginated document.
use crate::limit;
use sns_engine::TraceEvent;
use sns_fonts::{Face, TextLayout};
use sns_i18n::Language;
use sns_model::*;
use sns_protocols::routing::{self, RouteView};
use sns_render::Scene;
use std::{collections::BTreeMap, sync::Arc};

pub const MAX_PAGES: usize = 512;
pub const MAX_EVENTS: usize = 100_000;
const BODY_WIDTH: f32 = 523.;
const BODY_BOTTOM: f32 = 784.;
#[derive(Clone)]
pub struct Snapshot {
    pub project: Arc<Project>,
    pub effective: Option<Arc<Project>>,
    pub routes: BTreeMap<Id, Vec<RouteView>>,
    pub events: Arc<Vec<TraceEvent>>,
    pub revision: u64,
    pub time_us: u64,
    pub run_id: Option<Id>,
}
impl Snapshot {
    /// Captures the current state without stepping, pausing or commanding the engine.
    pub fn capture(engine: &sns_engine::Engine) -> Self {
        let effective = engine.network().effective_project(engine.project());
        let routes = effective
            .devices
            .iter()
            .map(|d| {
                (
                    d.id.clone(),
                    engine.network().routing.views(d, engine.now()),
                )
            })
            .collect();
        Self {
            project: Arc::new(engine.project().clone()),
            effective: Some(Arc::new(effective)),
            routes,
            events: Arc::new(engine.trace().to_vec()),
            revision: engine.revision(),
            time_us: engine.now(),
            run_id: Some(engine.run_id().clone()),
        }
    }
    pub fn configured(project: Project) -> Self {
        Self {
            project: Arc::new(project),
            effective: None,
            routes: BTreeMap::new(),
            events: Arc::new(Vec::new()),
            revision: 0,
            time_us: 0,
            run_id: None,
        }
    }
}
#[derive(Clone, Copy)]
pub struct ReportOptions {
    pub language: Language,
    pub diagram: bool,
}
impl Default for ReportOptions {
    fn default() -> Self {
        Self {
            language: Language::Fr,
            diagram: false,
        }
    }
}
pub(crate) struct Text {
    pub x: f32,
    pub y: f32,
    pub layout: TextLayout,
    pub color: [u8; 4],
}
pub(crate) struct Page {
    pub section: String,
    pub text: Vec<Text>,
    pub rules: Vec<f32>,
    pub scene: Option<(Scene, f32, f32, f32)>,
}
pub struct Report {
    pub(crate) pages: Vec<Page>,
    pub(crate) title: String,
    pub(crate) options: ReportOptions,
    pub packet_rows: usize,
    pub interface_groups: usize,
    pub trace_partial: bool,
    pub clipped_scene_texts: usize,
}
impl Report {
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
    pub fn build(snapshot: &Snapshot, options: ReportOptions) -> Result<Self, Diagnostic> {
        snapshot.project.validate()?;
        if snapshot.events.len() > MAX_EVENTS {
            return Err(limit("report_events"));
        }
        if let Some(effective) = &snapshot.effective {
            effective.validate()?;
            if effective.project_id != snapshot.project.project_id
                || effective.devices.len() != snapshot.project.devices.len()
                || effective
                    .devices
                    .iter()
                    .zip(&snapshot.project.devices)
                    .any(|(a, b)| {
                        a.id != b.id
                            || a.interfaces.iter().map(|i| &i.id).collect::<Vec<_>>()
                                != b.interfaces.iter().map(|i| &i.id).collect::<Vec<_>>()
                    })
            {
                return Err(Diagnostic::new(
                    "report.invalid_snapshot",
                    "effective_project",
                ));
            }
        }
        let l = options.language;
        let mut flow = Flow::new(snapshot.project.name.clone(), options);
        flow.section(l.text("report_overview"))?;
        flow.paragraph(&snapshot.project.name, 18., true)?;
        let revision = l
            .format("revision", &[("revision", &snapshot.revision.to_string())])
            .map_err(|_| Diagnostic::new("report.pdf_failed", "revision_label"))?;
        flow.paragraph(
            &format!(
                "{} · {} · {} {}",
                if snapshot.run_id.is_some() {
                    l.text("report_runtime")
                } else {
                    l.text("report_configured")
                },
                revision,
                l.text("time_us"),
                snapshot.time_us
            ),
            10.,
            false,
        )?;
        if let Some(run) = &snapshot.run_id {
            flow.paragraph(run.as_str(), 9., false)?;
        }
        let mut clipped_scene_texts = 0;
        let mut labels = BTreeMap::new();
        if let Some(p) = &snapshot.effective {
            for device in &p.devices {
                labels.insert(device.id.clone(), sns_topology_label(p, device));
            }
        }
        if snapshot.project.devices.is_empty() && snapshot.project.annotations.is_empty() {
            flow.paragraph(l.text("report_no_data"), 11., false)?;
        } else {
            let scene = Scene::build_with_labels(&snapshot.project, options.diagram, &labels)?;
            clipped_scene_texts = scene.counts().clipped_texts;
            let bounds = scene.bounds();
            let scale =
                (BODY_WIDTH / bounds.width).min((BODY_BOTTOM - flow.y - 20.) / bounds.height);
            let x = 36. + (BODY_WIDTH - bounds.width * scale) / 2.;
            flow.current().scene = Some((scene, x, flow.y, scale));
        }
        if snapshot.project.devices.is_empty() {
            flow.section(l.text("report_configuration"))?;
            flow.paragraph(l.text("report_no_data"), 10., false)?;
        }
        for device in &snapshot.project.devices {
            flow.section(&format!(
                "{} / {}",
                l.text("report_configuration"),
                device.label
            ))?;
            flow.heading(&format!("{} · {}", device.label, device.id.as_str()))?;
            let configuration: Vec<(String, String)> = vec![
                (
                    l.text("gateway").into(),
                    device
                        .network
                        .default_gateway
                        .clone()
                        .unwrap_or_else(|| "-".into()),
                ),
                (
                    l.text("dns_server").into(),
                    device
                        .network
                        .dns_server
                        .clone()
                        .unwrap_or_else(|| "-".into()),
                ),
                (
                    l.text("ip_forwarding").into(),
                    if device.network.forwarding.unwrap_or(matches!(
                        device.kind,
                        DeviceKind::Router | DeviceKind::Gateway
                    )) {
                        l.text("report_yes")
                    } else {
                        l.text("report_no")
                    }
                    .into(),
                ),
                (
                    "RIP".into(),
                    if device.network.rip {
                        l.text("report_yes")
                    } else {
                        l.text("report_no")
                    }
                    .into(),
                ),
            ];
            if let Some(effective) = snapshot
                .effective
                .as_ref()
                .and_then(|p| p.device(&device.id))
            {
                let actual = [
                    effective
                        .network
                        .default_gateway
                        .clone()
                        .unwrap_or_else(|| "-".into()),
                    effective
                        .network
                        .dns_server
                        .clone()
                        .unwrap_or_else(|| "-".into()),
                    if effective.network.forwarding.unwrap_or(matches!(
                        effective.kind,
                        DeviceKind::Router | DeviceKind::Gateway
                    )) {
                        l.text("report_yes")
                    } else {
                        l.text("report_no")
                    }
                    .into(),
                    if effective.network.rip {
                        l.text("report_yes")
                    } else {
                        l.text("report_no")
                    }
                    .into(),
                ];
                flow.table(
                    &[
                        l.text("name"),
                        l.text("report_configured"),
                        l.text("report_runtime"),
                    ],
                    &[160., 181., 182.],
                    &configuration
                        .iter()
                        .zip(actual)
                        .map(|((name, value), effective)| {
                            vec![name.clone(), value.clone(), effective]
                        })
                        .collect::<Vec<_>>(),
                )?;
            } else {
                flow.pairs(&configuration)?;
            }
            flow.heading(l.text("report_interfaces"))?;
            flow.table(
                &[
                    l.text("interface_label"),
                    l.text("mac_address"),
                    l.text("report_configured"),
                    l.text("report_runtime"),
                ],
                &[105., 125., 145., 148.],
                &device
                    .interfaces
                    .iter()
                    .map(|nic| {
                        let actual = snapshot
                            .effective
                            .as_ref()
                            .and_then(|p| p.device(&device.id))
                            .and_then(|d| d.interfaces.iter().find(|i| i.id == nic.id));
                        vec![
                            nic.id.as_str().into(),
                            nic.mac.clone(),
                            ipv4(nic),
                            actual.map(ipv4).unwrap_or_else(|| "-".into()),
                        ]
                    })
                    .collect::<Vec<_>>(),
            )?;
            flow.heading(l.text("software"))?;
            if device.applications.is_empty() {
                flow.paragraph(l.text("report_no_data"), 10., false)?;
            }
            for app in &device.applications {
                let kind = Software::parse(&app.kind)
                    .map(|s| l.text(s.label_key()))
                    .unwrap_or(&app.kind);
                flow.paragraph(&format!("{kind} · {}", app.id.as_str()), 10., false)?;
            }
            flow.heading(l.text("report_dns"))?;
            let rows: Vec<_> = device
                .applications
                .iter()
                .flat_map(|app| match &app.service {
                    Some(ServiceConfig::Dns(config)) => config
                        .records
                        .iter()
                        .map(|r| {
                            vec![
                                r.name.clone(),
                                match r.record_type {
                                    DnsType::A => "A",
                                    DnsType::Mx => "MX",
                                    DnsType::Ns => "NS",
                                }
                                .into(),
                                r.value.clone(),
                                format!("{} / {}", r.ttl_seconds, r.priority),
                            ]
                        })
                        .collect(),
                    _ => Vec::new(),
                })
                .collect();
            flow.table(
                &[
                    l.text("dns_record_name"),
                    l.text("dns_record_type"),
                    l.text("dns_record_value"),
                    "TTL / MX",
                ],
                &[150., 45., 230., 98.],
                &rows,
            )?;
            flow.heading(l.text("report_routes"))?;
            let routes = snapshot
                .routes
                .get(&device.id)
                .cloned()
                .unwrap_or_else(|| routing::configured(device));
            let rows: Vec<_> = routes
                .iter()
                .map(|r| {
                    vec![
                        format!("{}/{}", r.network, r.prefix),
                        r.gateway
                            .map(|a| a.to_string())
                            .unwrap_or_else(|| "-".into()),
                        r.interface
                            .as_ref()
                            .map(|i| i.as_str())
                            .unwrap_or("-")
                            .into(),
                        format!(
                            "{} / {}",
                            l.text(match r.origin {
                                routing::Origin::Loopback => "route_loopback",
                                routing::Origin::Direct => "route_direct",
                                routing::Origin::Static => "route_static",
                                routing::Origin::Default => "route_default",
                                routing::Origin::Rip => "route_rip",
                            }),
                            r.metric
                        ),
                    ]
                })
                .collect();
            flow.table(
                &[
                    l.text("route_network"),
                    l.text("route_gateway"),
                    l.text("interface_label"),
                    l.text("route_origin"),
                ],
                &[140., 120., 120., 143.],
                &rows,
            )?;
            if let Some(gateway) = &device.network.gateway {
                flow.heading(l.text("report_nat"))?;
                flow.table(
                    &[
                        l.text("protocol"),
                        l.text("report_external_port"),
                        l.text("destination"),
                    ],
                    &[100., 140., 283.],
                    &gateway
                        .forwards
                        .iter()
                        .map(|f| {
                            vec![
                                format!("{:?}", f.protocol).to_uppercase(),
                                f.external_port.to_string(),
                                format!("{}:{}", f.internal_address, f.internal_port),
                            ]
                        })
                        .collect::<Vec<_>>(),
                )?;
            }
        }
        let mut previous = None;
        let mut groups: BTreeMap<(String, String, String), Vec<Vec<String>>> = BTreeMap::new();
        let mut packet_rows = 0;
        let mut partial = false;
        for event in snapshot.events.iter() {
            if previous.is_some_and(|(seq, time)| event.sequence.0 <= seq || event.time_us.0 < time)
            {
                return Err(Diagnostic::new("report.invalid_snapshot", "trace_order"));
            }
            partial |= previous.is_some_and(|(seq, _)| event.sequence.0 > seq + 1)
                || (previous.is_none() && event.sequence.0 > 1);
            previous = Some((event.sequence.0, event.time_us.0));
            if !matches!(event.kind.as_str(), "ethernet.sent" | "ethernet.received") {
                continue;
            }
            let interface = event
                .payload
                .get("interface")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Diagnostic::new("report.invalid_snapshot", "packet_interface"))?;
            let device = snapshot
                .project
                .device(&event.device_id)
                .ok_or_else(|| Diagnostic::new("report.invalid_snapshot", "packet_device"))?;
            let nic = device
                .interfaces
                .iter()
                .find(|i| i.id.as_str() == interface)
                .ok_or_else(|| Diagnostic::new("report.invalid_snapshot", "packet_interface"))?;
            let frame = event
                .payload
                .get("frame")
                .ok_or_else(|| Diagnostic::new("report.invalid_snapshot", "packet_frame"))?;
            let string = |key| frame.get(key).and_then(|v| v.as_str()).unwrap_or("-");
            let row = vec![
                format!("#{}\n{} µs", event.sequence.0, event.time_us.0),
                l.text(if event.kind.ends_with(".sent") {
                    "sent"
                } else {
                    "received_direction"
                })
                .into(),
                format!("{}\n{}", string("source_mac"), string("destination_mac")),
                format!(
                    "{} · {} · {}\n{}",
                    event.kind,
                    event.run_id.as_str(),
                    event.correlation_id.as_str(),
                    packet_summary(frame)
                ),
            ];
            groups
                .entry((
                    nic.mac.to_ascii_lowercase(),
                    device.id.as_str().into(),
                    interface.into(),
                ))
                .or_default()
                .push(row);
            packet_rows += 1;
        }
        flow.section(l.text("report_traffic"))?;
        if partial {
            flow.paragraph(l.text("report_partial_trace"), 11., true)?;
        }
        if groups.is_empty() {
            flow.paragraph(l.text("report_no_packets"), 11., false)?;
        }
        let interface_groups = groups.len();
        for ((mac, device, interface), rows) in groups {
            let context = format!("{device} / {interface} / {mac}");
            flow.heading(&context)?;
            flow.repeat = Some(context);
            flow.table(
                &[
                    l.text("time_us"),
                    l.text("direction"),
                    l.text("report_mac_pair"),
                    l.text("report_packet"),
                ],
                &[72., 67., 130., 254.],
                &rows,
            )?;
            flow.repeat = None;
        }
        Ok(Report {
            pages: flow.pages,
            title: flow.title,
            options,
            packet_rows,
            interface_groups,
            trace_partial: partial,
            clipped_scene_texts,
        })
    }
}
fn sns_topology_label(project: &Project, device: &Device) -> String {
    // Labels are shared with the scene; effective configuration remains a separate input.
    sns_topology::display_label(project, device)
}
fn ipv4(nic: &Interface) -> String {
    match &nic.ipv4 {
        Some(Ipv4Config::Static {
            address,
            prefix_length,
        }) => format!("{address}/{prefix_length}"),
        Some(Ipv4Config::Dhcp) => "DHCP".into(),
        None => "-".into(),
    }
}
fn packet_summary(frame: &serde_json::Value) -> String {
    let payload = &frame["payload"];
    let packet = payload.get("packet").unwrap_or(payload);
    let mut text = Vec::new();
    for key in [
        "protocol",
        "source",
        "destination",
        "source_ip",
        "destination_ip",
        "sender_ip",
        "target_ip",
        "ttl",
        "source_port",
        "destination_port",
    ] {
        if let Some(v) = packet.get(key) {
            text.push(format!(
                "{key}={}",
                v.as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| v.to_string())
            ));
        }
    }
    if text.is_empty() {
        text.push(format!(
            "ether_type={}",
            frame
                .get("ether_type")
                .map(|v| v.to_string())
                .unwrap_or_default()
        ));
    }
    text.join(" · ")
}
struct Flow {
    pages: Vec<Page>,
    title: String,
    options: ReportOptions,
    y: f32,
    text_bytes: usize,
    glyphs: usize,
    repeat: Option<String>,
}
impl Flow {
    fn new(title: String, options: ReportOptions) -> Self {
        Self {
            pages: Vec::new(),
            title,
            options,
            y: 0.,
            text_bytes: 0,
            glyphs: 0,
            repeat: None,
        }
    }
    fn current(&mut self) -> &mut Page {
        self.pages
            .last_mut()
            .expect("flow always starts with a section")
    }
    fn page(&mut self, section: String) -> Result<(), Diagnostic> {
        if self.pages.len() >= MAX_PAGES {
            return Err(limit("report_pages"));
        }
        self.pages.push(Page {
            section,
            text: Vec::new(),
            rules: Vec::new(),
            scene: None,
        });
        self.y = 78.;
        Ok(())
    }
    fn section(&mut self, section: &str) -> Result<(), Diagnostic> {
        self.page(section.into())
    }
    fn ensure(&mut self, height: f32) -> Result<bool, Diagnostic> {
        if self.y + height > BODY_BOTTOM {
            let title = self.current().section.clone();
            self.page(title)?;
            if let Some(label) = self.repeat.take() {
                self.paragraph(&label, 10., true)?;
                self.repeat = Some(label);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn layout(
        &mut self,
        text: &str,
        width: f32,
        size: f32,
        bold: bool,
    ) -> Result<TextLayout, Diagnostic> {
        self.text_bytes = self
            .text_bytes
            .checked_add(text.len())
            .ok_or_else(|| limit("report_text"))?;
        if self.text_bytes > 8 * 1024 * 1024 {
            return Err(limit("report_text"));
        }
        let layout = sns_fonts::layout(
            text,
            if bold { Face::SansBold } else { Face::Sans },
            size,
            width,
            20_000.,
            4096,
            false,
        )
        .map_err(|e| {
            Diagnostic::new(
                e.to_string()
                    .split(':')
                    .next()
                    .unwrap_or("report.pdf_failed"),
                "report_text",
            )
        })?;
        if layout.clipped {
            return Err(limit("report_cell"));
        }
        self.glyphs += layout.lines.iter().map(|l| l.glyphs.len()).sum::<usize>();
        if self.glyphs > 2_000_000 {
            return Err(limit("report_glyphs"));
        }
        Ok(layout)
    }
    fn paragraph(&mut self, text: &str, size: f32, bold: bool) -> Result<(), Diagnostic> {
        let layout = self.layout(text, BODY_WIDTH, size, bold)?;
        for line in &layout.lines {
            self.ensure(size * 1.45)?;
            let y = self.y;
            self.current().text.push(Text {
                x: 36.,
                y,
                layout: TextLayout {
                    face: layout.face,
                    size,
                    lines: vec![sns_fonts::Line {
                        baseline: size * 1.05,
                        ..line.clone()
                    }],
                    height: size * 1.35,
                    clipped: false,
                },
                color: [28, 42, 58, 255],
            });
            self.y += size * 1.45;
        }
        self.y += 7.;
        Ok(())
    }
    fn heading(&mut self, text: &str) -> Result<(), Diagnostic> {
        self.ensure(70.)?;
        self.y += 10.;
        self.paragraph(text, 12., true)
    }
    fn pairs(&mut self, rows: &[(String, String)]) -> Result<(), Diagnostic> {
        self.table(
            &[],
            &[160., 363.],
            &rows
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect::<Vec<_>>(),
        )
    }
    fn table(
        &mut self,
        headers: &[&str],
        widths: &[f32],
        rows: &[Vec<String>],
    ) -> Result<(), Diagnostic> {
        let heading: Vec<String> = headers.iter().map(|s| (*s).into()).collect();
        if !heading.is_empty() {
            self.row(&heading, widths, true)?;
        }
        if rows.is_empty() {
            self.paragraph(self.options.language.text("report_no_data"), 10., false)?;
        }
        for row in rows {
            let layouts: Vec<_> = row
                .iter()
                .zip(widths)
                .map(|(text, width)| self.layout(text, width - 10., 9., false))
                .collect::<Result<_, _>>()?;
            let lines = layouts
                .iter()
                .map(|l| l.lines.len())
                .max()
                .unwrap_or(1)
                .max(1);
            let mut offset = 0;
            while offset < lines {
                if self.y + 30. > BODY_BOTTOM {
                    self.ensure(31.)?;
                    if !heading.is_empty() {
                        self.row(&heading, widths, true)?;
                    }
                }
                let capacity = ((BODY_BOTTOM - self.y - 12.) / 13.).floor().max(1.) as usize;
                let count = (lines - offset).min(capacity);
                let y = self.y;
                let mut x = 36.;
                for (layout, width) in layouts.iter().zip(widths) {
                    let selected = layout
                        .lines
                        .iter()
                        .skip(offset)
                        .take(count)
                        .enumerate()
                        .map(|(index, line)| sns_fonts::Line {
                            baseline: 10. + index as f32 * 13.,
                            ..line.clone()
                        })
                        .collect::<Vec<_>>();
                    self.current().text.push(Text {
                        x: x + 5.,
                        y: y + 4.,
                        layout: TextLayout {
                            lines: selected,
                            height: count as f32 * 13.,
                            face: layout.face,
                            size: layout.size,
                            clipped: false,
                        },
                        color: [28, 42, 58, 255],
                    });
                    x += width;
                }
                self.y += count as f32 * 13. + 10.;
                let y = self.y;
                self.current().rules.push(y);
                offset += count;
                if offset < lines {
                    self.ensure(BODY_BOTTOM)?;
                    if !heading.is_empty() {
                        self.row(&heading, widths, true)?;
                    }
                }
            }
        }
        self.y += 7.;
        Ok(())
    }
    fn row(&mut self, cells: &[String], widths: &[f32], bold: bool) -> Result<(), Diagnostic> {
        let layouts: Vec<_> = cells
            .iter()
            .zip(widths)
            .map(|(text, width)| self.layout(text, width - 10., 9., bold))
            .collect::<Result<_, _>>()?;
        let h = layouts.iter().map(|l| l.height).fold(13., f32::max) + 10.;
        self.ensure(h)?;
        let mut x = 36.;
        let y = self.y;
        for (layout, width) in layouts.into_iter().zip(widths) {
            self.current().text.push(Text {
                x: x + 5.,
                y: y + 4.,
                layout,
                color: [31, 83, 122, 255],
            });
            x += width;
        }
        self.y += h;
        let y = self.y;
        self.current().rules.push(y);
        Ok(())
    }
}
