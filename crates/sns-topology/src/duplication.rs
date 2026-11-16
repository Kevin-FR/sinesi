//! Whole-selection copies with structural remapping; warnings never silently renumber IPv4.
use super::*;
use std::net::Ipv4Addr;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct InterfaceLocation {
    pub device: Id,
    pub interface: Id,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Ipv4Conflict {
    pub address: Ipv4Addr,
    pub interfaces: Vec<InterfaceLocation>,
}
#[derive(Clone, Debug)]
pub struct Duplication {
    pub project: Project,
    pub identities: BTreeMap<Id, Id>,
    pub created: BTreeSet<Id>,
    pub omitted_links: Vec<Id>,
    pub potential_conflicts: Vec<Ipv4Conflict>,
}
pub fn potential_ipv4_conflicts(project: &Project) -> Vec<Ipv4Conflict> {
    let mut addresses: BTreeMap<Ipv4Addr, Vec<InterfaceLocation>> = BTreeMap::new();
    for d in &project.devices {
        for i in &d.interfaces {
            if let Some(Ipv4Config::Static { address, .. }) = &i.ipv4
                && let Ok(address) = address.parse()
            {
                addresses
                    .entry(address)
                    .or_default()
                    .push(InterfaceLocation {
                        device: d.id.clone(),
                        interface: i.id.clone(),
                    });
            }
        }
    }
    addresses
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(address, mut interfaces)| {
            interfaces.sort_by(|a, b| (&a.device, &a.interface).cmp(&(&b.device, &b.interface)));
            Ipv4Conflict {
                address,
                interfaces,
            }
        })
        .collect()
}
fn selection_valid(project: &Project, ids: &BTreeSet<Id>) -> Result<(), Diagnostic> {
    for id in ids {
        if project.device(id).is_none()
            && !project.links.iter().any(|l| &l.id == id)
            && !project.annotations.iter().any(|a| &a.id == id)
        {
            return Err(Diagnostic::new("project.invalid_reference", id.as_str()));
        }
    }
    Ok(())
}
struct Names {
    serial: u64,
    ids: BTreeSet<String>,
    macs: BTreeSet<String>,
    labels: BTreeSet<String>,
    ssids: BTreeSet<String>,
}
impl Names {
    fn new(p: &Project) -> Self {
        let mut ids = BTreeSet::from([p.project_id.as_str().to_string()]);
        for d in &p.devices {
            ids.insert(d.id.as_str().to_string());
            for i in &d.interfaces {
                ids.insert(i.id.as_str().to_string());
            }
            for a in &d.applications {
                ids.insert(a.id.as_str().to_string());
                if let Some(data) = &a.data {
                    match data {
                        ApplicationData::MailServer(s) => {
                            for m in s.mailboxes.values().flatten() {
                                ids.insert(m.id.as_str().to_string());
                            }
                        }
                        ApplicationData::MailClient(c) => {
                            for m in c.inbox.iter().chain(&c.sent) {
                                ids.insert(m.id.as_str().to_string());
                            }
                            for contact in &c.contacts {
                                ids.insert(contact.id.clone());
                            }
                        }
                    }
                }
            }
        }
        for id in p
            .links
            .iter()
            .map(|v| &v.id)
            .chain(p.annotations.iter().map(|v| &v.id))
            .chain(p.resources.iter().map(|v| &v.id))
            .chain(p.extensions.iter().map(|v| &v.id))
        {
            ids.insert(id.as_str().to_string());
        }
        Self {
            serial: 0,
            ids,
            macs: p
                .devices
                .iter()
                .flat_map(|d| &d.interfaces)
                .map(|i| i.mac.to_ascii_lowercase())
                .collect(),
            labels: p.devices.iter().map(|d| d.label.clone()).collect(),
            ssids: p
                .devices
                .iter()
                .filter_map(|d| d.network.ssid.clone())
                .collect(),
        }
    }
    fn next(&mut self) -> Result<u64, Diagnostic> {
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("resource.limit_exceeded", "copy_id"))?;
        Ok(self.serial)
    }
    fn id(&mut self, kind: &str) -> Result<Id, Diagnostic> {
        loop {
            let n = self.next()?;
            let s = format!("copy-{kind}-{n}");
            if self.ids.insert(s.clone()) {
                return Id::new(s);
            }
        }
    }
    fn mac(&mut self) -> Result<String, Diagnostic> {
        loop {
            let n = self.next()?;
            let s = format!(
                "02:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                (n >> 32) & 255,
                (n >> 24) & 255,
                (n >> 16) & 255,
                (n >> 8) & 255,
                n & 255
            );
            if self.macs.insert(s.clone()) {
                return Ok(s);
            }
        }
    }
    fn text(base: &str, suffix: &str, maximum: usize) -> String {
        let mut b = base.to_string();
        while b.len() + suffix.len() > maximum {
            b.pop();
        }
        b + suffix
    }
    fn label(&mut self, base: &str) -> String {
        for n in 2.. {
            let s = Self::text(base, &format!(" ({n})"), 4096);
            if self.labels.insert(s.clone()) {
                return s;
            }
        }
        unreachable!()
    }
    fn ssid(&mut self, base: &str) -> String {
        for n in 2.. {
            let s = Self::text(base, &format!("-{n}"), 32);
            if self.ssids.insert(s.clone()) {
                return s;
            }
        }
        unreachable!()
    }
}
fn renew_message(
    m: &mut MailMessage,
    names: &mut Names,
    mapping: &mut BTreeMap<Id, Id>,
) -> Result<(), Diagnostic> {
    let fresh = if let Some(id) = mapping.get(&m.id) {
        id.clone()
    } else {
        let id = names.id("message")?;
        mapping.insert(m.id.clone(), id.clone());
        id
    };
    m.id = fresh;
    Ok(())
}
fn renew_mail(
    data: &mut ApplicationData,
    names: &mut Names,
    mapping: &mut BTreeMap<Id, Id>,
) -> Result<(), Diagnostic> {
    match data {
        ApplicationData::MailServer(s) => {
            for m in s.mailboxes.values_mut().flatten() {
                renew_message(m, names, mapping)?;
            }
        }
        ApplicationData::MailClient(c) => {
            for m in c.inbox.iter_mut().chain(&mut c.sent) {
                renew_message(m, names, mapping)?;
            }
            for contact in &mut c.contacts {
                contact.id = names.id("contact")?.as_str().to_string();
            }
        }
    }
    Ok(())
}
/// Copies internal cables automatically. Selected external cables are reported and never attached to occupied original ports.
pub fn duplicate_selection(
    source: &Project,
    selection: &BTreeSet<Id>,
    dx: f32,
    dy: f32,
) -> Result<Duplication, Diagnostic> {
    source.validate()?;
    selection_valid(source, selection)?;
    if !dx.is_finite() || !dy.is_finite() {
        return Err(Diagnostic::new(
            "validation.invalid_position",
            "copy_offset",
        ));
    }
    let mut devices: Vec<_> = source
        .devices
        .iter()
        .filter(|d| selection.contains(&d.id))
        .cloned()
        .collect();
    devices.sort_by(|a, b| a.id.cmp(&b.id));
    let mut notes: Vec<_> = source
        .annotations
        .iter()
        .filter(|a| selection.contains(&a.id))
        .cloned()
        .collect();
    notes.sort_by(|a, b| a.id.cmp(&b.id));
    if devices.is_empty() && notes.is_empty() {
        return Err(Diagnostic::new("topology.selection_empty", "duplicate"));
    }
    if source.devices.len() + devices.len() > MAX_DEVICES {
        return Err(Diagnostic::new("resource.limit_exceeded", "devices"));
    }
    let mut names = Names::new(source);
    let mut identities = BTreeMap::new();
    let mut macs = BTreeMap::new();
    let mut ssids = BTreeMap::new();
    for d in &devices {
        identities.insert(d.id.clone(), names.id("device")?);
        for i in &d.interfaces {
            identities.insert(i.id.clone(), names.id("port")?);
            macs.insert(i.mac.to_ascii_lowercase(), names.mac()?);
        }
        for a in &d.applications {
            identities.insert(a.id.clone(), names.id("app")?);
        }
        if let Some(s) = &d.network.ssid {
            ssids.insert(s.clone(), names.ssid(s));
        }
    }
    let mut next = source.clone();
    let mut created = BTreeSet::new();
    let mut messages = BTreeMap::new();
    for mut d in devices {
        let original = d.id.clone();
        d.id = identities[&original].clone();
        d.label = names.label(&d.label);
        for i in &mut d.interfaces {
            i.id = identities[&i.id].clone();
            i.mac = macs[&i.mac.to_ascii_lowercase()].clone();
        }
        for a in &mut d.applications {
            a.id = identities[&a.id].clone();
            if let Some(data) = &mut a.data {
                renew_mail(data, &mut names, &mut messages)?;
            }
        }
        if let Some(gateway) = &mut d.network.gateway {
            gateway.lan = identities[&gateway.lan].clone();
            gateway.wan = identities[&gateway.wan].clone();
        }
        for route in &mut d.network.routes {
            route.interface = identities[&route.interface].clone();
        }
        if let Some(dhcp) = &mut d.network.dhcp_server {
            dhcp.interface = identities[&dhcp.interface].clone();
            dhcp.reservations = dhcp
                .reservations
                .iter()
                .map(|(mac, ip)| {
                    (
                        macs.get(&mac.to_ascii_lowercase())
                            .cloned()
                            .unwrap_or_else(|| mac.clone()),
                        *ip,
                    )
                })
                .collect();
        }
        if let Some(ssid) = &mut d.network.ssid {
            *ssid = ssids[ssid].clone();
        }
        d.network.wireless = d
            .network
            .wireless
            .iter()
            .map(|(id, ssid)| {
                (
                    identities[id].clone(),
                    ssids.get(ssid).cloned().unwrap_or_else(|| ssid.clone()),
                )
            })
            .collect();
        let index = source
            .devices
            .iter()
            .position(|v| v.id == original)
            .unwrap();
        let position = source
            .layout
            .devices
            .iter()
            .find(|v| v.device_id == original)
            .cloned()
            .unwrap_or(Position {
                device_id: original.clone(),
                x: 80. + (index % 5) as f32 * 150.,
                y: 80. + (index / 5) as f32 * 100.,
            });
        next.layout.devices.push(Position {
            device_id: d.id.clone(),
            x: position.x + dx,
            y: position.y + dy,
        });
        if let Some(c) = source.lan.get(&original) {
            next.lan.insert(d.id.clone(), c.clone());
        }
        created.insert(d.id.clone());
        next.devices.push(d);
    }
    let mut links = source.links.clone();
    links.sort_by(|a, b| a.id.cmp(&b.id));
    let mut omitted_links = Vec::new();
    for link in links {
        if let (Some(a), Some(b)) = (
            identities.get(&link.endpoints[0]).cloned(),
            identities.get(&link.endpoints[1]).cloned(),
        ) {
            let id = names.id("link")?;
            identities.insert(link.id.clone(), id.clone());
            created.insert(id.clone());
            next.links.push(Link {
                id,
                endpoints: [a, b],
                latency_us: link.latency_us,
                jitter_us: link.jitter_us,
            });
        } else if selection.contains(&link.id) {
            omitted_links.push(link.id);
        }
    }
    for mut note in notes {
        let id = names.id("annotation")?;
        identities.insert(note.id.clone(), id.clone());
        note.id = id.clone();
        note.x += dx;
        note.y += dy;
        created.insert(id);
        next.annotations.push(note);
    }
    next.validate()?;
    if serde_json::to_vec(&next).map_or(true, |v| v.len() > MAX_PROJECT_DOCUMENT_BYTES) {
        return Err(Diagnostic::new(
            "resource.limit_exceeded",
            "duplicated_project",
        ));
    }
    let potential_conflicts = potential_ipv4_conflicts(&next);
    Ok(Duplication {
        project: next,
        identities,
        created,
        omitted_links,
        potential_conflicts,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Left,
    HorizontalCenter,
    Right,
    Top,
    VerticalCenter,
    Bottom,
    DistributeHorizontal,
    DistributeVertical,
}
impl Alignment {
    pub const ALL: [Self; 8] = [
        Self::Left,
        Self::HorizontalCenter,
        Self::Right,
        Self::Top,
        Self::VerticalCenter,
        Self::Bottom,
        Self::DistributeHorizontal,
        Self::DistributeVertical,
    ];
}
pub fn aligned_positions(
    project: &Project,
    selection: &BTreeSet<Id>,
    alignment: Alignment,
) -> Result<Vec<Position>, Diagnostic> {
    selection_valid(project, selection)?;
    project.validate()?;
    let mut positions = Vec::new();
    for d in project.devices.iter().filter(|d| selection.contains(&d.id)) {
        positions.push(
            project
                .layout
                .devices
                .iter()
                .find(|p| p.device_id == d.id)
                .cloned()
                .ok_or_else(|| Diagnostic::new("validation.invalid_position", d.id.as_str()))?,
        );
    }
    let distribute = matches!(
        alignment,
        Alignment::DistributeHorizontal | Alignment::DistributeVertical
    );
    if positions.len() < if distribute { 3 } else { 2 } {
        return Err(Diagnostic::new("topology.alignment_selection", "selection"));
    }
    let horizontal = matches!(
        alignment,
        Alignment::Left
            | Alignment::HorizontalCenter
            | Alignment::Right
            | Alignment::DistributeHorizontal
    );
    let value = |p: &Position| if horizontal { p.x } else { p.y };
    positions.sort_by(|a, b| {
        value(a)
            .total_cmp(&value(b))
            .then(a.device_id.cmp(&b.device_id))
    });
    let min = value(&positions[0]);
    let max = value(positions.last().unwrap());
    let step = (max - min) / (positions.len() - 1) as f32;
    for (index, p) in positions.iter_mut().enumerate() {
        let target = match alignment {
            Alignment::Left | Alignment::Top => min,
            Alignment::Right | Alignment::Bottom => max,
            Alignment::HorizontalCenter | Alignment::VerticalCenter => (min + max) / 2.,
            _ => min + step * index as f32,
        };
        if horizontal {
            p.x = target;
        } else {
            p.y = target;
        }
    }
    positions.sort_by(|a, b| a.device_id.cmp(&b.device_id));
    Ok(positions)
}
