//! Transactional topology commands, independent of rendering and simulation.
mod advice;
mod duplication;
pub use advice::*;
pub use duplication::*;
use sns_model::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub enum Edit {
    SetSymbol {
        device: Id,
        symbol: Option<DeviceSymbol>,
    },
    Duplicate {
        selection: BTreeSet<Id>,
        dx: f32,
        dy: f32,
    },
    Align {
        selection: BTreeSet<Id>,
        alignment: Alignment,
    },
    Add {
        kind: DeviceKind,
        x: f32,
        y: f32,
    },
    AddNamed {
        kind: DeviceKind,
        name: String,
        x: f32,
        y: f32,
    },
    Move(Vec<Position>),
    ResizeInterfaces {
        device: Id,
        count: usize,
    },
    SwapInterfaces {
        device: Id,
        a: Id,
        b: Id,
    },
    Configure(Box<Device>, LanConfig),
    Connect {
        a: Id,
        b: Id,
    },
    Delete(BTreeSet<Id>),
    Disconnect(Id),
    Annotate(Annotation),
    AddAnnotation {
        text: String,
        x: f32,
        y: f32,
        kind: AnnotationKind,
    },
}
#[derive(Clone, Debug, Default)]
pub struct EditResult {
    pub created: BTreeSet<Id>,
    pub identities: BTreeMap<Id, Id>,
    pub omitted_links: Vec<Id>,
}
pub struct Editor {
    project: Project,
    undo: Vec<Project>,
    redo: Vec<Project>,
    revision: u64,
    serial: u64,
    saved: Project,
    unsaved: bool,
}
/// Shared initial placement, without cloning host applications or files.
pub fn complete_positions(project: &Project) -> Vec<Position> {
    let mut positions = project.layout.devices.clone();
    let placed: BTreeSet<_> = positions.iter().map(|p| p.device_id.clone()).collect();
    for (index, device) in project.devices.iter().enumerate() {
        if !placed.contains(&device.id) {
            positions.push(Position {
                device_id: device.id.clone(),
                x: 80. + (index % 5) as f32 * 150.,
                y: 80. + (index / 5) as f32 * 100.,
            });
        }
    }
    positions
}
/// First free cell of a 160×130 grid aligned on the network, scanning rows of
/// five; a new device never covers an existing one or an annotation's origin.
pub fn free_position(project: &Project) -> (f32, f32) {
    const STEP_X: f32 = 160.;
    const STEP_Y: f32 = 130.;
    let positions = complete_positions(project);
    let origin = positions
        .iter()
        .fold(None, |acc: Option<(f32, f32)>, p| {
            Some(acc.map_or((p.x, p.y), |(x, y)| (x.min(p.x), y.min(p.y))))
        })
        .unwrap_or((80., 80.));
    let taken = |x: f32, y: f32| {
        positions
            .iter()
            .any(|p| (p.x - x).abs() < 130. && (p.y - y).abs() < 100.)
    };
    for cell in 0..1000 {
        let (x, y) = (
            origin.0 + (cell % 5) as f32 * STEP_X,
            origin.1 + (cell / 5) as f32 * STEP_Y,
        );
        if !taken(x, y) {
            return (x, y);
        }
    }
    (origin.0, origin.1 + 1000. * STEP_Y / 5.)
}
impl Editor {
    pub fn new(mut project: Project) -> Result<Self, Diagnostic> {
        project.validate()?;
        let saved = project.clone();
        project.layout.devices = complete_positions(&project);
        Ok(Self {
            saved,
            unsaved: false,
            project,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            serial: 0,
        })
    }
    pub fn empty() -> Self {
        Self::new(Project {
            schema_version: 2,
            semantic_profile: "sns-educational-lan-v1".into(),
            project_id: Id::new("project-lan").unwrap(),
            name: "Nouveau réseau".into(),
            devices: vec![],
            links: vec![],
            layout: Layout::default(),
            annotations: vec![],
            resources: vec![],
            extensions: vec![],
            lan: BTreeMap::new(),
        })
        .unwrap()
    }
    pub fn project(&self) -> &Project {
        &self.project
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn dirty(&self) -> bool {
        self.unsaved || self.saved != self.project
    }
    /// Merge a newer application document through the same undo/dirty history.
    pub fn accept_applications(&mut self, source: Project) -> Result<(), Diagnostic> {
        self.accept_runtime(source, &BTreeSet::new())
    }
    pub fn accept_runtime(
        &mut self,
        source: Project,
        presentation_devices: &BTreeSet<Id>,
    ) -> Result<(), Diagnostic> {
        source.validate()?;
        if source.project_id != self.project.project_id {
            return Err(Diagnostic::new("state.revision_conflict", "project"));
        }
        let mut project = self.project.clone();
        for host in &mut project.devices {
            if let Some(updated) = source.device(&host.id) {
                host.applications = updated.applications.clone();
                host.files = updated.files.clone();
                host.network.gateway = updated.network.gateway.clone();
                host.network.firewall = updated.network.firewall.clone();
            }
        }
        for device in presentation_devices {
            if project.device(device).is_some() {
                let symbol = source.lan.get(device).and_then(|c| c.symbol);
                project = project.with_symbol(device, symbol)?;
            }
        }
        if source.schema_version > project.schema_version {
            project.schema_version = source.schema_version;
            project.semantic_profile = source.semantic_profile;
        }
        project.validate()?;
        if project != self.project {
            self.undo.push(self.project.clone());
            self.redo.clear();
            self.project = project;
            self.revision += 1;
        }
        Ok(())
    }
    pub fn mark_unsaved(&mut self) {
        self.unsaved = true;
    }
    pub fn mark_saved(&mut self) {
        self.unsaved = false;
        self.saved = self.project.clone();
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    fn id(&mut self, prefix: &str) -> Id {
        loop {
            self.serial += 1;
            let id = Id::new(format!("{prefix}-{}", self.serial)).unwrap();
            let p = &self.project;
            if id != p.project_id
                && !p.devices.iter().any(|d| {
                    d.id == id
                        || d.interfaces.iter().any(|i| i.id == id)
                        || d.applications.iter().any(|a| a.id == id)
                })
                && !p.links.iter().any(|l| l.id == id)
                && !p.annotations.iter().any(|a| a.id == id)
                && !p.resources.iter().any(|r| r.id == id)
                && !p.extensions.iter().any(|e| e.id == id)
            {
                return id;
            }
        }
    }
    pub fn apply(&mut self, expected_revision: u64, edit: Edit) -> Result<EditResult, Diagnostic> {
        if expected_revision != self.revision {
            return Err(Diagnostic::new("state.revision_conflict", "project"));
        }
        let initial_serial = self.serial;
        let mut next = self.project.clone();
        if next.schema_version < 2 {
            next.schema_version = 2;
            next.semantic_profile = "sns-educational-lan-v1".into();
        }
        let requested_name = match &edit {
            Edit::AddNamed { name, .. } => Some(name.clone()),
            _ => None,
        };
        let mut result = EditResult::default();
        let edited: Result<(), Diagnostic> = (|| {
            match edit {
                Edit::SetSymbol { device, symbol } => {
                    next = next.with_symbol(&device, symbol)?;
                }
                Edit::Duplicate { selection, dx, dy } => {
                    let copy = duplicate_selection(&next, &selection, dx, dy)?;
                    result = EditResult {
                        created: copy.created,
                        identities: copy.identities,
                        omitted_links: copy.omitted_links,
                    };
                    next = copy.project;
                }
                Edit::Align {
                    selection,
                    alignment,
                } => {
                    for p in aligned_positions(&next, &selection, alignment)? {
                        if let Some(old) = next
                            .layout
                            .devices
                            .iter_mut()
                            .find(|v| v.device_id == p.device_id)
                        {
                            *old = p;
                        }
                    }
                }
                Edit::Add { kind, x, y } | Edit::AddNamed { kind, x, y, .. } => {
                    let id = self.id("device");
                    let ports = if kind == DeviceKind::Switch {
                        8
                    } else if kind == DeviceKind::Gateway {
                        2
                    } else if kind == DeviceKind::Router {
                        4
                    } else {
                        1
                    };
                    let mut interfaces = Vec::new();
                    for _ in 0..ports {
                        let port = self.id("port");
                        let mut mac_number = self.serial;
                        let mac = loop {
                            let mac = format!(
                                "02:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                (mac_number >> 32) & 255,
                                (mac_number >> 24) & 255,
                                (mac_number >> 16) & 255,
                                (mac_number >> 8) & 255,
                                mac_number & 255
                            );
                            if !next
                                .devices
                                .iter()
                                .flat_map(|d| &d.interfaces)
                                .any(|i| i.mac.eq_ignore_ascii_case(&mac))
                                && !interfaces
                                    .iter()
                                    .any(|i: &Interface| i.mac.eq_ignore_ascii_case(&mac))
                            {
                                break mac;
                            }
                            mac_number += 1;
                        };
                        interfaces.push(Interface {
                            id: port,
                            mac,
                            medium: Medium::Ethernet,
                            ipv4: None,
                        });
                    }
                    if kind == DeviceKind::Router && next.schema_version < 4 {
                        next.schema_version = 4;
                        next.semantic_profile = "sns-educational-routing-v1".into();
                    }
                    next.devices.push(Device {
                        id: id.clone(),
                        kind,
                        label: requested_name
                            .unwrap_or_else(|| format!("{kind:?} {}", id.as_str())),
                        interfaces,
                        network: Network::default(),
                        applications: vec![],
                        files: vec![],
                    });
                    if kind == DeviceKind::Gateway {
                        let admin_id = self.id("app");
                        let firewall_id = self.id("app");
                        let host = next.devices.last_mut().unwrap();
                        let lan = host.interfaces[0].id.clone();
                        let wan = host.interfaces[1].id.clone();
                        host.interfaces[0].ipv4 = Some(Ipv4Config::Static {
                            address: "192.168.0.1".into(),
                            prefix_length: 24,
                        });
                        host.interfaces[1].ipv4 = Some(Ipv4Config::Dhcp);
                        host.network.gateway = Some(Box::new(GatewayConfig {
                            lan: lan.clone(),
                            wan,
                            nat_enabled: true,
                            port_start: 40000,
                            port_end: 60000,
                            idle_timeout_us: Decimal(60_000_000),
                            forwards: vec![],
                        }));
                        host.network.dhcp_server = Some(Box::new(DhcpConfig {
                            interface: lan,
                            pool_start: "192.168.0.100".parse().unwrap(),
                            pool_end: "192.168.0.200".parse().unwrap(),
                            prefix_length: 24,
                            gateway: Some("192.168.0.1".parse().unwrap()),
                            dns: None,
                            use_host_options: true,
                            reservations: Default::default(),
                            lease_us: Decimal(86_400_000_000),
                        }));
                        host.applications.push(Application {
                            data: None,
                            id: admin_id,
                            kind: "web_server".into(),
                            config: Default::default(),
                            service: Some(ServiceConfig::Http(HttpConfig {
                                administration: true,
                                ..Default::default()
                            })),
                        });
                        host.applications.push(Application {
                            data: None,
                            id: firewall_id,
                            kind: "firewall".into(),
                            config: Default::default(),
                            service: None,
                        });
                        private_profile(&mut next);
                    }

                    next.layout.devices.push(Position {
                        device_id: id.clone(),
                        x,
                        y,
                    });
                    next.lan.insert(id, LanConfig::default());
                }
                Edit::ResizeInterfaces { device, count } => {
                    let host = next
                        .devices
                        .iter_mut()
                        .find(|d| d.id == device)
                        .ok_or_else(|| {
                            Diagnostic::new("project.invalid_reference", device.as_str())
                        })?;
                    if !(2..=32).contains(&count) || host.kind != DeviceKind::Router {
                        return Err(Diagnostic::new(
                            "routing.router_interfaces",
                            device.as_str(),
                        ));
                    }
                    if count < host.interfaces.len() {
                        let removed = &host.interfaces[count..];
                        if next
                            .links
                            .iter()
                            .any(|l| removed.iter().any(|p| l.endpoints.contains(&p.id)))
                            || host
                                .network
                                .routes
                                .iter()
                                .any(|r| removed.iter().any(|p| p.id == r.interface))
                        {
                            return Err(Diagnostic::new("topology.port_busy", device.as_str()));
                        }
                        host.interfaces.truncate(count);
                    } else {
                        while next.device(&device).unwrap().interfaces.len() < count {
                            let id = self.id("port");
                            let n = self.serial;
                            let mut value = n;
                            let mac = loop {
                                let mac = format!(
                                    "06:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                    (value >> 32) & 255,
                                    (value >> 24) & 255,
                                    (value >> 16) & 255,
                                    (value >> 8) & 255,
                                    value & 255
                                );
                                if !next
                                    .devices
                                    .iter()
                                    .flat_map(|d| &d.interfaces)
                                    .any(|i| i.mac.eq_ignore_ascii_case(&mac))
                                {
                                    break mac;
                                }
                                value += 1;
                            };
                            next.devices
                                .iter_mut()
                                .find(|d| d.id == device)
                                .unwrap()
                                .interfaces
                                .push(Interface {
                                    id,
                                    mac,
                                    medium: Medium::Ethernet,
                                    ipv4: None,
                                });
                        }
                    }
                    if next.schema_version < 4 {
                        next.schema_version = 4;
                        next.semantic_profile = "sns-educational-routing-v1".into();
                    }
                }
                Edit::SwapInterfaces { device, a, b } => {
                    let host = next.device(&device).ok_or_else(|| {
                        Diagnostic::new("project.invalid_reference", device.as_str())
                    })?;
                    if host.kind != DeviceKind::Router
                        || a == b
                        || ![&a, &b]
                            .iter()
                            .all(|id| host.interfaces.iter().any(|i| &i.id == *id))
                    {
                        return Err(Diagnostic::new("topology.invalid_link", device.as_str()));
                    }
                    for link in &mut next.links {
                        for endpoint in &mut link.endpoints {
                            if *endpoint == a {
                                *endpoint = b.clone();
                            } else if *endpoint == b {
                                *endpoint = a.clone();
                            }
                        }
                    }
                    if next.schema_version < 4 {
                        next.schema_version = 4;
                        next.semantic_profile = "sns-educational-routing-v1".into();
                    }
                }
                Edit::Move(positions) => {
                    for p in positions {
                        next.layout
                            .devices
                            .retain(|old| old.device_id != p.device_id);
                        next.layout.devices.push(p);
                    }
                }
                Edit::Configure(device, config) => {
                    let device = *device;
                    let old = next
                        .devices
                        .iter_mut()
                        .find(|d| d.id == device.id)
                        .ok_or_else(|| {
                            Diagnostic::new("project.invalid_reference", device.id.as_str())
                        })?;
                    if (config.symbol.is_some()
                        || device.network.forwarding.is_some()
                        || device.network.rip
                        || !device.network.routes.is_empty()
                        || device.kind == DeviceKind::Router)
                        && next.schema_version < 4
                    {
                        next.schema_version = 4;
                        next.semantic_profile = "sns-educational-routing-v1".into();
                    }
                    if (device.network.dhcp_server.is_some()
                        || device
                            .interfaces
                            .iter()
                            .any(|i| matches!(i.ipv4, Some(Ipv4Config::Dhcp))))
                        && next.schema_version < 5
                    {
                        next.schema_version = 5;
                        next.semantic_profile = "sns-educational-services-v1".into();
                    }
                    if (device.network.gateway.is_some() || device.network.firewall.is_some())
                        && next.schema_version < 6
                    {
                        next.schema_version = 6;
                        next.semantic_profile = "sns-educational-private-v1".into();
                    }
                    if (matches!(
                        config.symbol,
                        Some(DeviceSymbol::AccessPoint | DeviceSymbol::Modem)
                    ) || device.network.modem.is_some()
                        || device.network.ssid.is_some()
                        || !device.network.wireless.is_empty())
                        && next.schema_version < 7
                    {
                        next.schema_version = 7;
                        next.semantic_profile = "sns-educational-applications-v1".into();
                    }
                    next.links.retain(|l| {
                        !device
                            .interfaces
                            .iter()
                            .any(|i| i.medium == Medium::Wireless && l.endpoints.contains(&i.id))
                    });
                    if config.symbol == Some(DeviceSymbol::Cloud) && next.schema_version < 8 {
                        next.schema_version = 8;
                        next.semantic_profile = PRESENTATION_PROFILE.into();
                    }
                    next.lan.insert(device.id.clone(), config);
                    *old = device;
                }
                Edit::Connect { a, b } => {
                    let da = next
                        .device_for_interface(&a)
                        .ok_or_else(|| Diagnostic::new("project.invalid_reference", a.as_str()))?;
                    let db = next
                        .device_for_interface(&b)
                        .ok_or_else(|| Diagnostic::new("project.invalid_reference", b.as_str()))?;
                    if da.id == db.id
                        || [(&a, da), (&b, db)].iter().any(|(id, d)| {
                            d.interfaces
                                .iter()
                                .any(|i| &i.id == *id && i.medium != Medium::Ethernet)
                        })
                    {
                        return Err(Diagnostic::new("topology.incompatible_ports", "link"));
                    }
                    next.links.push(Link {
                        id: self.id("link"),
                        endpoints: [a, b],
                        latency_us: Decimal(1_000),
                        jitter_us: Decimal(0),
                    });
                }
                Edit::Delete(ids) => {
                    let ports: BTreeSet<_> = next
                        .devices
                        .iter()
                        .filter(|d| ids.contains(&d.id))
                        .flat_map(|d| d.interfaces.iter().map(|i| i.id.clone()))
                        .collect();
                    next.devices.retain(|d| !ids.contains(&d.id));
                    next.links.retain(|l| {
                        !ids.contains(&l.id) && !l.endpoints.iter().any(|i| ports.contains(i))
                    });
                    next.layout.devices.retain(|p| !ids.contains(&p.device_id));
                    next.annotations.retain(|a| !ids.contains(&a.id));
                    next.lan.retain(|id, _| !ids.contains(id));
                }
                Edit::Disconnect(id) => {
                    let d = next
                        .device(&id)
                        .ok_or_else(|| Diagnostic::new("project.invalid_reference", id.as_str()))?;
                    let ports: BTreeSet<_> = d.interfaces.iter().map(|i| i.id.clone()).collect();
                    next.links
                        .retain(|l| !l.endpoints.iter().any(|i| ports.contains(i)));
                }
                Edit::Annotate(a) => {
                    next = next.with_annotation(a)?;
                }
                Edit::AddAnnotation { text, x, y, kind } => {
                    let id = self.id("note");
                    next = next.with_annotation(Annotation {
                        id: id.clone(),
                        text,
                        x,
                        y,
                        style: Some(AnnotationStyle::for_kind(kind)),
                    })?;
                    result.created.insert(id);
                }
            }
            Ok(())
        })();
        if let Err(error) = edited {
            self.serial = initial_serial;
            return Err(error);
        }
        if let Err(error) = next.validate() {
            self.serial = initial_serial;
            return Err(error);
        }
        if next != self.project {
            if self.undo.len() == 100 {
                self.undo.remove(0);
            }
            self.undo.push(std::mem::replace(&mut self.project, next));
            self.redo.clear();
            self.revision += 1;
        }
        Ok(result)
    }
    pub fn undo(&mut self) {
        if let Some(previous) = self.undo.pop() {
            self.redo
                .push(std::mem::replace(&mut self.project, previous));
            self.revision += 1;
        }
    }
    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.project, next));
            self.revision += 1;
        }
    }
}

pub fn display_label(project: &Project, device: &Device) -> String {
    match project
        .lan
        .get(&device.id)
        .map(|c| c.display)
        .unwrap_or(DisplayName::Name)
    {
        DisplayName::Name => device.label.clone(),
        DisplayName::Mac => device
            .interfaces
            .first()
            .map(|i| i.mac.clone())
            .unwrap_or_else(|| device.label.clone()),
        DisplayName::Ipv4 => device
            .interfaces
            .iter()
            .find_map(|i| {
                if let Some(Ipv4Config::Static { address, .. }) = &i.ipv4 {
                    Some(address.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| device.label.clone()),
    }
}
