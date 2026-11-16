//! Educational Ethernet/ARP/IPv4/ICMP state machines. No OS clock or sockets.
pub mod bridge;
mod dhcp;
pub mod firewall;
mod forwarding;
mod link_conditions;
pub mod nat;
pub use forwarding::arp_injection_route;
mod rip;
pub mod routing;
pub mod transport;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sns_model::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};

pub const ARP_RETENTION_US: u64 = 60_000_000;
pub const PING_TIMEOUT_US: u64 = 3_000_000;
pub const ARP_TIMEOUT_US: u64 = 1_000_000;
const BROADCAST: &str = "ff:ff:ff:ff:ff:ff";
const MAX_PENDING: usize = 4096;

pub fn subnet(address: Ipv4Addr, prefix: u8) -> Result<(Ipv4Addr, Ipv4Addr), Diagnostic> {
    if prefix > 32 {
        return Err(Diagnostic::new("validation.invalid_mask", "prefix"));
    }
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    Ok((
        (u32::from(address) & mask).into(),
        (u32::from(address) | !mask).into(),
    ))
}
pub fn mask_prefix(mask: &str) -> Result<u8, Diagnostic> {
    let mask = u32::from(
        mask.parse::<Ipv4Addr>()
            .map_err(|_| Diagnostic::new("validation.invalid_mask", mask))?,
    );
    let prefix = mask.leading_ones();
    if mask
        != if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        }
    {
        return Err(Diagnostic::new("validation.invalid_mask", "mask"));
    }
    Ok(prefix as u8)
}
fn ip(interface: &Interface) -> Option<(Ipv4Addr, u8)> {
    match &interface.ipv4 {
        Some(Ipv4Config::Static {
            address,
            prefix_length,
        }) => address.parse().ok().map(|ip| (ip, *prefix_length)),
        _ => None,
    }
}
pub fn next_hop(device: &Device, target: Ipv4Addr) -> Result<(Id, Ipv4Addr, Ipv4Addr), Diagnostic> {
    routing::hop(device, &routing::configured(device), target)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "protocol", rename_all = "snake_case")]
pub enum Payload {
    Arp {
        operation: u16,
        sender_ip: Ipv4Addr,
        sender_mac: String,
        target_ip: Ipv4Addr,
        target_mac: String,
    },
    Ipv4Transport {
        source: Ipv4Addr,
        destination: Ipv4Addr,
        ttl: u8,
        packet: TransportPacket,
    },
    Ipv4 {
        source: Ipv4Addr,
        destination: Ipv4Addr,
        ttl: u8,
        icmp: Icmp,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Icmp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted: Option<Box<Payload>>,
    pub message_type: u8,
    pub code: u8,
    pub identifier: Id,
    pub sequence: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub frame_id: Decimal,
    pub source_mac: String,
    pub destination_mac: String,
    pub ether_type: u16,
    pub payload: Payload,
    #[serde(skip)]
    remaining_hops: u8,
}
impl Frame {
    pub fn transit_budget(&self) -> u8 {
        self.remaining_hops
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Event {
    pub correlation: Id,
    pub action: Action,
}
#[derive(Clone, Debug, serde::Serialize)]
pub enum Action {
    ConfigureLink {
        link: Id,
        conditions: LinkConditions,
        seed: u64,
    },
    Bridge {
        device: Id,
        command: BridgeCommand,
    },
    NatExpire {
        entry: u64,
    },
    SecurityChanged {
        device: Id,
        purge_nat: bool,
    },
    PurgeNat {
        device: Id,
    },
    DhcpStart {
        interface: Id,
    },
    DhcpTick {
        interface: Id,
        token: u64,
    },
    DhcpExpire {
        interface: Id,
        token: u64,
    },
    DhcpSweep {
        device: Id,
    },
    RipTick {
        device: Id,
        periodic: bool,
    },
    RipSweep {
        device: Id,
    },
    RipMute {
        device: Id,
        muted: bool,
    },
    ArpSend {
        device: Id,
        source: Ipv4Addr,
        target: Ipv4Addr,
    },
    PurgeArpAddress {
        device: Id,
        address: Ipv4Addr,
    },
    PingBroadcast {
        source: Id,
        destination: Ipv4Addr,
        ttl: u8,
    },
    Socket {
        device: Id,
        command: SocketCommand,
    },
    TransportTimeout {
        device: Id,
        socket: Id,
        token: u64,
    },
    LocalTransport {
        device: Id,
        source: Ipv4Addr,
        destination: Ipv4Addr,
        packet: TransportPacket,
    },
    Ping {
        source: Id,
        destination: Ipv4Addr,
        ttl: u8,
    },
    Receive {
        interface: Id,
        link: Id,
        frame: Frame,
    },
    ArpExpire {
        interface: Id,
        address: Ipv4Addr,
        expires: u64,
    },
    SatExpire {
        device: Id,
        mac: String,
        expires: u64,
    },
    ArpTimeout {
        interface: Id,
        address: Ipv4Addr,
        token: u64,
    },
    PingTimeout {
        source: Id,
    },
    PurgeArp {
        device: Id,
    },
    PurgeSat {
        device: Id,
    },
}
#[derive(Clone, serde::Serialize)]
pub enum Delay {
    Link(Id),
    Timer(u64),
}
pub struct Effect {
    pub cause_index: Option<usize>,
    pub delay: Delay,
    pub event: Event,
}
pub struct Fact {
    pub event_schema_version: u32,
    pub correlation: Id,
    pub kind: &'static str,
    pub device: Id,
    pub payload: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ArpEntry {
    pub mac: String,
    pub expires_us: Decimal,
}
#[derive(Clone, Debug, Serialize)]
pub struct SatEntry {
    pub interface: Id,
    pub expires_us: Decimal,
}
#[derive(Clone, serde::Serialize)]
struct Waiting {
    packets: Vec<(Payload, Id)>,
    token: u64,
}
#[derive(Clone, Default, serde::Serialize)]
pub struct Network {
    #[serde(skip_serializing_if = "WaitFactor::is_default")]
    wait_factor: WaitFactor,
    pub bridge_output: Vec<(Id, BridgePacket)>,
    bridge_sessions: BTreeMap<Id, (Id, u64, u64)>,
    pub nat: nat::Nat,
    pub firewall: firewall::Firewall,
    pub dhcp: dhcp::Dhcp,
    pub routing: routing::Routing,
    pub arp: BTreeMap<(Id, Ipv4Addr), ArpEntry>,
    pub sat: BTreeMap<(Id, String), SatEntry>,
    waiting: BTreeMap<(Id, Ipv4Addr), Waiting>,
    probes: BTreeMap<Id, (Id, u64)>,
    broadcast_probes: BTreeMap<Id, BTreeSet<Ipv4Addr>>,
    seen: BTreeSet<(Id, u64)>,
    serial: u64,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    link_conditions: BTreeMap<Id, link_conditions::LinkState>,
    pub manual_loss: bool,
    pub transport: transport::Transport,
    pub socket_events: Vec<SocketNotice>,
}
struct Context<'a> {
    wait_factor: WaitFactor,
    project: &'a Project,
    now: u64,
    flow: Id,
    effects: Vec<Effect>,
    facts: Vec<Fact>,
}
impl Context<'_> {
    fn fact(&mut self, kind: &'static str, device: &Id, value: Value) {
        let event_schema_version = if kind.starts_with("simulation.link.")
            || value.get("reason").and_then(Value::as_str) == Some("simulation.link_loss")
        {
            9
        } else if kind.starts_with("bridge.") || kind.starts_with("wireless.") {
            8
        } else if kind.starts_with("nat.")
            || kind.starts_with("firewall.")
            || kind.starts_with("security.")
            || contains_quote(&value)
        {
            7
        } else if kind.starts_with("dhcp.") {
            6
        } else if kind.starts_with("routing.")
            || kind.starts_with("rip.")
            || kind == "ipv4.forwarded"
        {
            5
        } else if kind == "icmp.broadcast.completed" {
            4
        } else if kind.starts_with("tcp.")
            || kind.starts_with("udp.")
            || kind.starts_with("socket.")
            || value.get("packet").is_some()
            || value
                .get("frame")
                .and_then(|f| f.get("payload"))
                .and_then(|p| p.get("protocol"))
                .and_then(Value::as_str)
                == Some("ipv4_transport")
        {
            3
        } else {
            2
        };
        self.facts.push(Fact {
            event_schema_version,
            correlation: self.flow.clone(),
            kind,
            device: device.clone(),
            payload: serde_json::from_value(value).expect("object payload"),
        });
    }
    fn timer(&mut self, delay: u64, action: Action) -> Result<(), Diagnostic> {
        let delay = if matches!(
            action,
            Action::ArpTimeout { .. } | Action::PingTimeout { .. } | Action::DhcpTick { .. }
        ) {
            self.wait_factor.scale(delay)?
        } else {
            delay
        };
        self.now
            .checked_add(delay)
            .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "timer"))?;
        self.effects.push(Effect {
            cause_index: self.facts.len().checked_sub(1),
            delay: Delay::Timer(delay),
            event: Event {
                correlation: self.flow.clone(),
                action,
            },
        });
        Ok(())
    }
}
impl Network {
    pub fn with_wait_factor(wait_factor: WaitFactor) -> Self {
        Self {
            wait_factor,
            ..Self::default()
        }
    }
    pub fn wait_factor(&self) -> WaitFactor {
        self.wait_factor
    }
    pub fn pending_probes(&self) -> usize {
        self.probes.len()
    }
    pub fn process(
        &mut self,
        project: &Project,
        now: u64,
        event: &Event,
    ) -> Result<(Vec<Effect>, Vec<Fact>), Diagnostic> {
        self.socket_events.clear();
        if !self.dhcp.assigned.is_empty() && self.dhcp.effective.is_none() {
            self.dhcp.effective = Some(std::sync::Arc::new(self.effective_project(project)));
        }
        let effective = self.dhcp.effective.clone();
        let project = effective.as_deref().unwrap_or(project);
        let mut c = Context {
            wait_factor: self.wait_factor,
            project,
            now,
            flow: event.correlation.clone(),
            effects: vec![],
            facts: vec![],
        };
        match &event.action {
            Action::ConfigureLink {
                link,
                conditions,
                seed,
            } => self.configure_link(&mut c, link, conditions, *seed)?,
            Action::Bridge { device, command } => self.bridge_command(&mut c, device, command)?,
            Action::NatExpire { entry } => self.nat_expire(&mut c, *entry)?,
            Action::PurgeNat { device } => self.nat_clear(&mut c, device, "user"),
            Action::SecurityChanged { device, purge_nat } => {
                if *purge_nat {
                    self.nat_clear(&mut c, device, "configuration_changed");
                }
                c.fact("security.configuration.changed",device,json!({"gateway":project.device(device).unwrap().network.gateway,"firewall":project.device(device).unwrap().network.firewall}));
            }
            Action::DhcpStart { interface } => self.dhcp_start(&mut c, interface, 0)?,
            Action::DhcpTick { interface, token } => self.dhcp_tick(&mut c, interface, *token)?,
            Action::DhcpExpire { interface, token } => {
                self.dhcp_expire(&mut c, interface, *token)?
            }
            Action::DhcpSweep { device } => self.dhcp_sweep(&mut c, device),
            Action::RipTick { device, periodic } => self.rip_tick(&mut c, device, *periodic)?,
            Action::RipSweep { device } => self.rip_sweep(&mut c, device)?,
            Action::RipMute { device, muted } => {
                if !project.device(device).is_some_and(|h| h.network.rip) {
                    return Err(Diagnostic::new("routing.rip_disabled", device.as_str()));
                }
                if *muted {
                    self.routing.muted.insert(device.clone());
                } else {
                    self.routing.muted.remove(device);
                    if self.routing.triggered.insert(device.clone()) {
                        c.timer(
                            1_000,
                            Action::RipTick {
                                device: device.clone(),
                                periodic: false,
                            },
                        )?;
                    }
                }
                c.fact("rip.announcements.changed", device, json!({"muted":muted}));
            }
            Action::ArpSend {
                device,
                source,
                target,
            } => self.arp_send(&mut c, device, *source, *target)?,
            Action::Socket { device, command } => {
                let host = project
                    .device(device)
                    .ok_or_else(|| Diagnostic::new("project.invalid_reference", device.as_str()))?;
                if matches!(
                    command,
                    SocketCommand::Bind {
                        protocol: TransportProtocol::Udp,
                        port: 67,
                        ..
                    }
                ) && host.network.dhcp_server.is_some()
                    || matches!(
                        command,
                        SocketCommand::Bind {
                            protocol: TransportProtocol::Udp,
                            port: 68,
                            ..
                        }
                    ) && host.interfaces.iter().any(|i| {
                        self.dhcp.uses_interface(&i.id) || matches!(i.ipv4, Some(Ipv4Config::Dhcp))
                    })
                {
                    return Err(Diagnostic::new("socket.port_in_use", "DHCP"));
                }
                if host.network.rip
                    && matches!(
                        command,
                        SocketCommand::Bind {
                            protocol: TransportProtocol::Udp,
                            port: 520,
                            ..
                        }
                    )
                {
                    return Err(Diagnostic::new("socket.port_in_use", "RIP"));
                }
                let routes = self.routing.views(host, now);
                let out = self
                    .transport
                    .command(project, device, command, &c.flow, &routes)?;
                self.apply_transport(&mut c, out)?;
            }
            Action::TransportTimeout {
                device,
                socket,
                token,
            } => {
                let out = self.transport.timeout(device, socket, *token, &c.flow)?;
                self.apply_transport(&mut c, out)?;
            }
            Action::LocalTransport {
                device,
                source,
                destination,
                packet,
            } => {
                let host = project
                    .device(device)
                    .ok_or_else(|| Diagnostic::new("project.invalid_reference", device.as_str()))?;
                c.fact("ipv4.delivered",device,json!({"source":source,"destination":destination,"ttl":64,"packet":packet,"local":true}));
                let interface = host
                    .interfaces
                    .iter()
                    .find(|i| ip(i).is_some_and(|(a, _)| a == *destination))
                    .map(|i| &i.id);
                let out = self.transport.receive(
                    transport::Delivery {
                        device: host,
                        interface,
                        local_address: *destination,
                        source: *source,
                        destination: *destination,
                        flow: &c.flow,
                    },
                    packet,
                )?;
                self.apply_transport(&mut c, out)?;
            }
            Action::PingBroadcast {
                source,
                destination,
                ttl,
            }
            | Action::Ping {
                source,
                destination,
                ttl,
            } => {
                let device = project
                    .device(source)
                    .ok_or_else(|| Diagnostic::new("project.invalid_reference", source.as_str()))?;
                if !matches!(
                    device.kind,
                    DeviceKind::Workstation
                        | DeviceKind::Notebook
                        | DeviceKind::Router
                        | DeviceKind::Gateway
                ) {
                    return Err(Diagnostic::new("device.not_host", source.as_str()));
                }
                if !matches!(event.action, Action::PingBroadcast { .. })
                    && (destination.is_broadcast()
                        || device.interfaces.iter().any(|i| {
                            ip(i).is_some_and(|(a, p)| {
                                p < 31 && subnet(a, p).is_ok_and(|(_, b)| b == *destination)
                            })
                        }))
                {
                    return Err(Diagnostic::new(
                        "ipv4.broadcast_requires_opt_in",
                        source.as_str(),
                    ));
                }
                if self.probes.len() >= MAX_PENDING {
                    return Err(Diagnostic::new("resource.limit_exceeded", "probes"));
                }
                self.probes.insert(c.flow.clone(), (source.clone(), now));
                c.fact(
                    "icmp.echo.requested",
                    source,
                    json!({"destination":destination,"ttl":ttl,"icmp_type":8,"sequence":1}),
                );
                if destination.is_loopback()
                    || device
                        .interfaces
                        .iter()
                        .any(|i| ip(i).is_some_and(|(a, _)| a == *destination))
                {
                    c.fact("ipv4.local", source, json!({"destination":destination}));
                    self.finish(
                        &mut c,
                        "icmp.echo.reply",
                        source,
                        json!({"icmp_type":0,"source":destination,"rtt_us":"0","sequence":1}),
                    );
                } else if *ttl == 0 {
                    self.finish(
                        &mut c,
                        "icmp.error",
                        source,
                        json!({"reason":"ipv4.ttl_expired","icmp_type":11,"icmp_code":0}),
                    );
                } else if matches!(&event.action, Action::PingBroadcast { .. }) {
                    let mut interfaces: Vec<_> = device
                        .interfaces
                        .iter()
                        .filter(|i| ip(i).is_some())
                        .collect();
                    interfaces.sort_by_key(|i| &i.id);
                    let nic = interfaces
                        .into_iter()
                        .find(|i| {
                            destination.is_broadcast()
                                || ip(i).is_some_and(|(a, p)| {
                                    p < 31 && subnet(a, p).is_ok_and(|(_, b)| b == *destination)
                                })
                        })
                        .ok_or_else(|| {
                            Diagnostic::new("validation.invalid_broadcast", source.as_str())
                        })?;
                    self.broadcast_probes
                        .insert(c.flow.clone(), BTreeSet::new());
                    let payload = Payload::Ipv4 {
                        source: ip(nic).unwrap().0,
                        destination: *destination,
                        ttl: *ttl,
                        icmp: Icmp {
                            quoted: None,
                            message_type: 8,
                            code: 0,
                            identifier: c.flow.clone(),
                            sequence: 1,
                        },
                    };
                    let frame =
                        self.frame(nic.mac.to_ascii_lowercase(), BROADCAST.into(), payload)?;
                    self.send(&mut c, &nic.id, frame);
                    c.timer(
                        PING_TIMEOUT_US,
                        Action::PingTimeout {
                            source: source.clone(),
                        },
                    )?;
                } else {
                    match self.routing.next_hop(device, *destination, c.now) {
                        Ok((port, address, hop)) => {
                            c.fact(
                                "ipv4.route.selected",
                                source,
                                json!({"interface":port,"next_hop":hop,"destination":destination}),
                            );
                            let packet = Payload::Ipv4 {
                                source: address,
                                destination: *destination,
                                ttl: *ttl,
                                icmp: Icmp {
                                    quoted: None,
                                    message_type: 8,
                                    code: 0,
                                    identifier: c.flow.clone(),
                                    sequence: 1,
                                },
                            };
                            self.resolve(&mut c, &port, hop, packet)?;
                            c.timer(
                                PING_TIMEOUT_US,
                                Action::PingTimeout {
                                    source: source.clone(),
                                },
                            )?;
                        }
                        Err(_) => self.finish(
                            &mut c,
                            "icmp.error",
                            source,
                            json!({"reason":"routing.no_route","icmp_type":3,"icmp_code":0}),
                        ),
                    }
                }
            }
            Action::Receive {
                interface,
                link,
                frame,
            } => {
                let device = project.device_for_interface(interface).ok_or_else(|| {
                    Diagnostic::new("project.invalid_reference", interface.as_str())
                })?;
                if self.manual_loss {
                    c.fact("ethernet.dropped",&device.id,json!({"reason":"simulation.manual_loss","interface":interface,"link":link,"frame":frame}));
                } else {
                    c.fact(
                        "ethernet.received",
                        &device.id,
                        json!({"interface":interface,"link":link,"frame":frame}),
                    );
                    if device.kind == DeviceKind::Modem {
                        if let Some((_, _, sequence)) = self.bridge_sessions.get_mut(&device.id) {
                            if frame.remaining_hops == 0 {
                                c.fact(
                                    "bridge.frame.dropped",
                                    &device.id,
                                    json!({"reason":"bridge.hop_limit"}),
                                );
                            } else if self.bridge_output.len() >= 256 {
                                c.fact(
                                    "bridge.frame.dropped",
                                    &device.id,
                                    json!({"reason":"resource.limit_exceeded"}),
                                );
                            } else {
                                *sequence += 1;
                                self.bridge_output.push((
                                    device.id.clone(),
                                    BridgePacket {
                                        version: 1,
                                        sequence: Decimal(*sequence),
                                        budget: frame.remaining_hops,
                                        frame: serde_json::to_value(frame).unwrap(),
                                    },
                                ));
                                c.fact(
                                    "bridge.frame.queued",
                                    &device.id,
                                    json!({"sequence":sequence.to_string()}),
                                );
                            }
                        } else {
                            c.fact(
                                "bridge.frame.dropped",
                                &device.id,
                                json!({"reason":"bridge.not_connected"}),
                            );
                        }
                    } else if device.kind == DeviceKind::Switch {
                        self.switch(&mut c, device, interface, frame)?;
                    } else {
                        self.receive_host(&mut c, device, interface, frame)?;
                    }
                }
            }
            Action::ArpExpire {
                interface,
                address,
                expires,
            } => {
                let key = (interface.clone(), *address);
                if self
                    .arp
                    .get(&key)
                    .is_some_and(|e| e.expires_us.0 == *expires)
                {
                    self.arp.remove(&key);
                    c.fact(
                        "arp.cache.expired",
                        &project.device_for_interface(interface).unwrap().id,
                        json!({"interface":interface,"address":address}),
                    );
                }
            }
            Action::SatExpire {
                device,
                mac,
                expires,
            } => {
                let key = (device.clone(), mac.clone());
                if self
                    .sat
                    .get(&key)
                    .is_some_and(|e| e.expires_us.0 == *expires)
                {
                    self.sat.remove(&key);
                    c.fact("switch.sat.expired", device, json!({"mac":mac}));
                }
            }
            Action::ArpTimeout {
                interface,
                address,
                token,
            } => {
                let key = (interface.clone(), *address);
                if self.waiting.get(&key).is_some_and(|w| w.token == *token) {
                    let waiting = self.waiting.remove(&key).unwrap();
                    for (mut packet, flow) in waiting.packets {
                        let old = std::mem::replace(&mut c.flow, flow);
                        let device = project.device_for_interface(interface).unwrap();
                        if let Some(original) = self.nat_original(device, &packet) {
                            packet = original;
                        }
                        let source = match &packet {
                            Payload::Ipv4 { source, .. }
                            | Payload::Ipv4Transport { source, .. } => Some(*source),
                            _ => None,
                        };
                        if let Some(source) = source.filter(|source| {
                            !device
                                .interfaces
                                .iter()
                                .any(|i| ip(i).is_some_and(|(a, _)| a == *source))
                        }) {
                            c.fact("routing.arp.failed",&device.id,json!({"reason":"arp.timeout","interface":interface,"next_hop":address}));
                            self.icmp_failure(&mut c, device, source, &packet, "arp.timeout")?;
                        } else {
                            self.finish(
                                &mut c,
                                "icmp.error",
                                &device.id,
                                json!({"reason":"arp.timeout","icmp_type":3,"icmp_code":1}),
                            );
                        }
                        c.flow = old;
                    }
                }
            }
            Action::PingTimeout { source } => {
                if self
                    .broadcast_probes
                    .get(&c.flow)
                    .is_some_and(|set| !set.is_empty())
                {
                    self.finish(&mut c, "icmp.broadcast.completed", source, json!({}));
                } else if self.probes.contains_key(&c.flow) {
                    self.finish(
                        &mut c,
                        "icmp.timeout",
                        source,
                        json!({"reason":"icmp.timeout"}),
                    );
                }
            }
            Action::PurgeArpAddress { device, address } => {
                self.arp.retain(|(port, ip), _| {
                    *ip != *address
                        || project
                            .device_for_interface(port)
                            .is_none_or(|d| &d.id != device)
                });
                c.fact("arp.cache.cleared", device, json!({"address":address}));
            }
            Action::PurgeArp { device } => {
                self.arp.retain(|(port, _), _| {
                    project
                        .device_for_interface(port)
                        .is_none_or(|d| &d.id != device)
                });
                c.fact("arp.cache.cleared", device, json!({}));
            }
            Action::PurgeSat { device } => {
                self.sat.retain(|(id, _), _| id != device);
                c.fact("switch.sat.cleared", device, json!({}));
            }
        }
        Ok((c.effects, c.facts))
    }
    fn apply_transport(
        &mut self,
        c: &mut Context<'_>,
        out: transport::Output,
    ) -> Result<(), Diagnostic> {
        let initial = c.flow.clone();
        for (device, flow, kind, payload) in out.facts {
            c.flow = flow;
            c.fact(kind, &device, payload);
        }
        for notice in out.notices {
            c.flow = notice.correlation.clone();
            c.fact(
                "socket.event",
                &notice.device,
                serde_json::to_value(&notice).unwrap(),
            );
            self.socket_events.push(notice);
        }
        for timer in out.timers {
            c.flow = timer.flow;
            c.timer(
                if timer.waits_for_response {
                    c.wait_factor.scale(timer.delay)?
                } else {
                    timer.delay
                },
                Action::TransportTimeout {
                    device: timer.device,
                    socket: timer.socket,
                    token: timer.token,
                },
            )?;
        }
        for packet in out.packets {
            c.flow = packet.flow;
            self.send_transport(
                c,
                &packet.device,
                packet.source,
                packet.destination,
                packet.packet,
                packet.interface,
            )?;
        }
        c.flow = initial;
        Ok(())
    }
    fn send_transport(
        &mut self,
        c: &mut Context<'_>,
        device: &Id,
        source: Ipv4Addr,
        destination: Ipv4Addr,
        packet: TransportPacket,
        interface: Option<Id>,
    ) -> Result<(), Diagnostic> {
        let host = c
            .project
            .device(device)
            .ok_or_else(|| Diagnostic::new("project.invalid_reference", device.as_str()))?;
        let kind = if matches!(packet, TransportPacket::Tcp(_)) {
            "tcp.segment.sent"
        } else {
            "udp.datagram.sent"
        };
        c.fact(
            kind,
            device,
            json!({"source":source,"destination":destination,"packet":packet}),
        );
        if destination.is_loopback()
            || host
                .interfaces
                .iter()
                .any(|i| ip(i).is_some_and(|(a, _)| a == destination))
        {
            c.timer(
                0,
                Action::LocalTransport {
                    device: device.clone(),
                    source,
                    destination,
                    packet,
                },
            )?;
            return Ok(());
        }
        let payload = Payload::Ipv4Transport {
            source,
            destination,
            ttl: 64,
            packet,
        };
        let mut ports: Vec<_> = host
            .interfaces
            .iter()
            .filter(|i| interface.as_ref().is_none_or(|id| &i.id == id))
            .collect();
        ports.sort_by_key(|i| &i.id);
        let nic = ports
            .iter()
            .find(|i| ip(i).is_some_and(|(a, _)| a == source))
            .copied()
            .or_else(|| {
                if source.is_unspecified() {
                    ports.first().copied()
                } else {
                    None
                }
            })
            .ok_or_else(|| Diagnostic::new("socket.address_not_local", device.as_str()))?;
        let broadcast = destination.is_broadcast()
            || ip(nic).is_some_and(|(address, prefix)| {
                prefix < 31 && subnet(address, prefix).is_ok_and(|(_, b)| b == destination)
            });
        if broadcast {
            let frame = self.frame(nic.mac.to_ascii_lowercase(), BROADCAST.into(), payload)?;
            self.send(c, &nic.id, frame);
        } else {
            match self.routing.next_hop(host, destination, c.now) {
                Ok((port, _, hop)) => {
                    if port != nic.id {
                        c.fact(
                            "ipv4.dropped",
                            device,
                            json!({"reason":"routing.bound_interface_mismatch"}),
                        );
                        return Ok(());
                    }
                    self.resolve(c, &port, hop, payload)?;
                }
                Err(_) => c.fact("ipv4.dropped", device, json!({"reason":"routing.no_route"})),
            }
        }
        Ok(())
    }
    fn finish(&mut self, c: &mut Context<'_>, kind: &'static str, device: &Id, value: Value) {
        self.probes.remove(&c.flow);
        self.broadcast_probes.remove(&c.flow);
        c.fact(kind, device, value);
    }
    fn frame(
        &mut self,
        source: String,
        destination: String,
        payload: Payload,
    ) -> Result<Frame, Diagnostic> {
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "frame"))?;
        Ok(Frame {
            frame_id: Decimal(self.serial),
            source_mac: source,
            destination_mac: destination,
            ether_type: if matches!(payload, Payload::Arp { .. }) {
                0x0806
            } else {
                0x0800
            },
            payload,
            remaining_hops: 64,
        })
    }
    fn send(&mut self, c: &mut Context<'_>, port: &Id, frame: Frame) {
        let device = &c.project.device_for_interface(port).unwrap().id;
        c.fact(
            "ethernet.sent",
            device,
            json!({"interface":port,"frame":frame}),
        );
        let nic = c
            .project
            .device_for_interface(port)
            .unwrap()
            .interfaces
            .iter()
            .find(|i| &i.id == port)
            .unwrap();
        if nic.medium == Medium::Wireless {
            let pairs = c.project.wireless_links();
            let peers: Vec<_> = pairs
                .into_iter()
                .filter_map(|(host, ap)| {
                    if &host == port {
                        Some(ap)
                    } else if &ap == port {
                        Some(host)
                    } else {
                        None
                    }
                })
                .filter(|p| {
                    c.project.device_for_interface(p).is_some_and(|d| {
                        d.interfaces
                            .iter()
                            .any(|i| &i.id == p && !i.mac.eq_ignore_ascii_case(&frame.source_mac))
                    })
                })
                .collect();
            if peers.is_empty() {
                c.fact("wireless.disconnected", device, json!({"interface":port}));
            }
            for peer in peers {
                c.fact(
                    "wireless.frame.sent",
                    device,
                    json!({"interface":port,"peer":peer,"frame":frame}),
                );
                c.effects.push(Effect {
                    cause_index: c.facts.len().checked_sub(1),
                    delay: Delay::Timer(1_000),
                    event: Event {
                        correlation: c.flow.clone(),
                        action: Action::Receive {
                            interface: peer,
                            link: port.clone(),
                            frame: frame.clone(),
                        },
                    },
                });
            }
            return;
        }
        if let Some(link) = c.project.links.iter().find(|l| l.endpoints.contains(port)) {
            let other = if link.endpoints[0] == *port {
                &link.endpoints[1]
            } else {
                &link.endpoints[0]
            };
            let delay = match self.conditioned_transit(c, &link.id, port, &frame) {
                Some((true, _)) => return,
                Some((false, delay)) => Delay::Timer(delay),
                None => Delay::Link(link.id.clone()),
            };
            c.effects.push(Effect {
                cause_index: c.facts.len().checked_sub(1),
                delay,
                event: Event {
                    correlation: c.flow.clone(),
                    action: Action::Receive {
                        interface: other.clone(),
                        link: link.id.clone(),
                        frame,
                    },
                },
            });
        } else {
            c.fact(
                "ethernet.dropped",
                device,
                json!({"reason":"topology.disconnected","interface":port,"frame":frame}),
            );
        }
    }
    fn resolve(
        &mut self,
        c: &mut Context<'_>,
        port: &Id,
        hop: Ipv4Addr,
        packet: Payload,
    ) -> Result<(), Diagnostic> {
        let device = c.project.device_for_interface(port).unwrap();
        let nic = device.interfaces.iter().find(|i| &i.id == port).unwrap();
        let key = (port.clone(), hop);
        if let Some(entry) = self.arp.get(&key).filter(|e| e.expires_us.0 > c.now) {
            c.fact(
                "arp.cache.hit",
                &device.id,
                json!({"interface":port,"address":hop,"mac":entry.mac}),
            );
            let frame = self.frame(nic.mac.to_ascii_lowercase(), entry.mac.clone(), packet)?;
            self.send(c, port, frame);
        } else {
            if self
                .waiting
                .values()
                .map(|w| w.packets.len())
                .sum::<usize>()
                >= MAX_PENDING
            {
                return Err(Diagnostic::new("resource.limit_exceeded", "arp_pending"));
            }
            if let Some(waiting) = self.waiting.get_mut(&key) {
                waiting.packets.push((packet, c.flow.clone()));
                return Ok(());
            }
            self.serial = self
                .serial
                .checked_add(1)
                .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "arp"))?;
            let token = self.serial;
            self.waiting.insert(
                key,
                Waiting {
                    packets: vec![(packet, c.flow.clone())],
                    token,
                },
            );
            let frame = self.frame(
                nic.mac.to_ascii_lowercase(),
                BROADCAST.into(),
                Payload::Arp {
                    operation: 1,
                    sender_ip: ip(nic).unwrap().0,
                    sender_mac: nic.mac.to_ascii_lowercase(),
                    target_ip: hop,
                    target_mac: "00:00:00:00:00:00".into(),
                },
            )?;
            c.fact(
                "arp.request",
                &device.id,
                json!({"interface":port,"target_ip":hop}),
            );
            self.send(c, port, frame);
            c.timer(
                ARP_TIMEOUT_US,
                Action::ArpTimeout {
                    interface: port.clone(),
                    address: hop,
                    token,
                },
            )?;
        }
        Ok(())
    }
    fn switch(
        &mut self,
        c: &mut Context<'_>,
        device: &Device,
        port: &Id,
        frame: &Frame,
    ) -> Result<(), Diagnostic> {
        if frame.remaining_hops == 0 || !self.seen.insert((device.id.clone(), frame.frame_id.0)) {
            c.fact(
                "ethernet.dropped",
                &device.id,
                json!({"reason":"switch.loop_detected","interface":port,"frame":frame}),
            );
            return Ok(());
        }
        // Deduplication memory is bounded; the per-transit hop budget bounds even evicted frames.
        if self.seen.len() > 65_536 {
            self.seen.pop_first();
        }
        let retention = c
            .project
            .lan
            .get(&device.id)
            .map(|v| v.sat_retention_us.0)
            .unwrap_or(300_000_000);
        let expires = c
            .now
            .checked_add(retention)
            .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "sat"))?;
        let key = (device.id.clone(), frame.source_mac.clone());
        if self.sat.len() >= 65_536 && !self.sat.contains_key(&key) {
            return Err(Diagnostic::new("resource.limit_exceeded", "sat"));
        }
        self.sat.insert(
            key,
            SatEntry {
                interface: port.clone(),
                expires_us: Decimal(expires),
            },
        );
        c.fact(
            "switch.sat.learned",
            &device.id,
            json!({"mac":frame.source_mac,"interface":port,"expires_us":expires.to_string()}),
        );
        c.timer(
            retention,
            Action::SatExpire {
                device: device.id.clone(),
                mac: frame.source_mac.clone(),
                expires,
            },
        )?;
        let known = self
            .sat
            .get(&(device.id.clone(), frame.destination_mac.clone()))
            .filter(|entry| entry.expires_us.0 > c.now)
            .map(|e| e.interface.clone());
        let mut ports: Vec<_> = device
            .interfaces
            .iter()
            .filter(|i| {
                (&i.id != port || i.medium == Medium::Wireless)
                    && known.as_ref().is_none_or(|p| p == &i.id)
                    && (i.medium == Medium::Wireless
                        || c.project
                            .links
                            .iter()
                            .any(|link| link.endpoints.contains(&i.id)))
            })
            .map(|i| i.id.clone())
            .collect();
        ports.sort();
        c.fact(
            if known.is_some() {
                "switch.forward"
            } else {
                "switch.flood"
            },
            &device.id,
            json!({"ingress":port,"egress":ports,"frame_id":frame.frame_id}),
        );
        for port in ports {
            let mut frame = frame.clone();
            frame.remaining_hops -= 1;
            self.send(c, &port, frame);
        }
        Ok(())
    }
    fn learn_arp(
        &mut self,
        c: &mut Context<'_>,
        port: &Id,
        address: Ipv4Addr,
        mac: &str,
    ) -> Result<(), Diagnostic> {
        let expires = c
            .now
            .checked_add(ARP_RETENTION_US)
            .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "arp"))?;
        if self.arp.len() >= 65_536 && !self.arp.contains_key(&(port.clone(), address)) {
            return Err(Diagnostic::new("resource.limit_exceeded", "arp"));
        }
        self.arp.insert(
            (port.clone(), address),
            ArpEntry {
                mac: mac.into(),
                expires_us: Decimal(expires),
            },
        );
        c.fact(
            "arp.cache.learned",
            &c.project.device_for_interface(port).unwrap().id,
            json!({"interface":port,"address":address,"mac":mac,"expires_us":expires.to_string()}),
        );
        c.timer(
            ARP_RETENTION_US,
            Action::ArpExpire {
                interface: port.clone(),
                address,
                expires,
            },
        )?;
        if let Some(waiting) = self.waiting.remove(&(port.clone(), address)) {
            let nic = c
                .project
                .device_for_interface(port)
                .unwrap()
                .interfaces
                .iter()
                .find(|i| &i.id == port)
                .unwrap();
            for (packet, flow) in waiting.packets {
                let old = std::mem::replace(&mut c.flow, flow);
                let frame = self.frame(nic.mac.to_ascii_lowercase(), mac.into(), packet)?;
                self.send(c, port, frame);
                c.flow = old;
            }
        }
        Ok(())
    }
    fn receive_host(
        &mut self,
        c: &mut Context<'_>,
        device: &Device,
        port: &Id,
        frame: &Frame,
    ) -> Result<(), Diagnostic> {
        let nic = device.interfaces.iter().find(|i| &i.id == port).unwrap();
        if !frame.destination_mac.eq_ignore_ascii_case(&nic.mac)
            && frame.destination_mac != BROADCAST
        {
            c.fact(
                "ethernet.filtered",
                &device.id,
                json!({"reason":"ethernet.other_destination","frame_id":frame.frame_id}),
            );
            return Ok(());
        }
        if !self.filter_ingress(c, device, port, frame) {
            return Ok(());
        }
        if self.dhcp_receive(c, device, port, frame)? {
            return Ok(());
        }
        let Some(translated) = self.nat_inbound(c, device, port, frame)? else {
            return Ok(());
        };
        let frame = &translated;
        if self.forward_or_control(c, device, port, frame)? {
            return Ok(());
        }
        match &frame.payload {
            Payload::Ipv4Transport {
                source,
                destination,
                ttl,
                packet,
            } => {
                let local = ip(nic)
                    .map(|(address, _)| address)
                    .unwrap_or(Ipv4Addr::UNSPECIFIED);
                let ours = device
                    .interfaces
                    .iter()
                    .any(|i| ip(i).is_some_and(|(a, _)| a == *destination));
                let broadcast = destination.is_broadcast()
                    || ip(nic).is_some_and(|(address, prefix)| {
                        prefix < 31 && subnet(address, prefix).is_ok_and(|(_, b)| b == *destination)
                    });
                if *ttl == 0
                    || (!ours && !broadcast)
                    || (broadcast && matches!(packet, TransportPacket::Tcp(_)))
                {
                    c.fact("ipv4.dropped",&device.id,json!({"reason":if *ttl==0 {"ipv4.ttl_expired"}else{"ipv4.forwarding_unavailable"},"packet":packet}));
                    return Ok(());
                }
                c.fact("ipv4.delivered",&device.id,json!({"source":source,"destination":destination,"ttl":ttl,"packet":packet,"interface":port}));
                let out = self.transport.receive(
                    transport::Delivery {
                        device,
                        interface: Some(port),
                        local_address: if broadcast { local } else { *destination },
                        source: *source,
                        destination: *destination,
                        flow: &c.flow,
                    },
                    packet,
                )?;
                self.apply_transport(c, out)?;
            }
            Payload::Arp {
                operation,
                sender_ip,
                sender_mac,
                target_ip,
                ..
            } => {
                if !sender_mac.eq_ignore_ascii_case(&frame.source_mac) {
                    c.fact(
                        "ethernet.dropped",
                        &device.id,
                        json!({"reason":"arp.sender_mismatch"}),
                    );
                    return Ok(());
                }
                let ours = ip(nic).is_some_and(|(a, _)| a == *target_ip);
                if ours || self.arp.contains_key(&(port.clone(), *sender_ip)) {
                    self.learn_arp(c, port, *sender_ip, sender_mac)?;
                }
                if ours && *operation == 1 {
                    c.fact(
                        "arp.reply",
                        &device.id,
                        json!({"interface":port,"target_ip":sender_ip}),
                    );
                    let reply = self.frame(
                        nic.mac.to_ascii_lowercase(),
                        sender_mac.clone(),
                        Payload::Arp {
                            operation: 2,
                            sender_ip: *target_ip,
                            sender_mac: nic.mac.to_ascii_lowercase(),
                            target_ip: *sender_ip,
                            target_mac: sender_mac.clone(),
                        },
                    )?;
                    self.send(c, port, reply);
                }
            }
            Payload::Ipv4 {
                source,
                destination,
                ttl,
                icmp,
            } => {
                let broadcast = destination.is_broadcast()
                    || ip(nic).is_some_and(|(a, p)| {
                        p < 31 && subnet(a, p).is_ok_and(|(_, b)| b == *destination)
                    });
                if !(broadcast && icmp.message_type == 8
                    || device
                        .interfaces
                        .iter()
                        .any(|i| ip(i).is_some_and(|(a, _)| a == *destination)))
                {
                    c.fact(
                        "ipv4.dropped",
                        &device.id,
                        json!({"reason":"ipv4.forwarding_unavailable","destination":destination}),
                    );
                    return Ok(());
                }
                if *ttl == 0 {
                    c.fact(
                        "ipv4.dropped",
                        &device.id,
                        json!({"reason":"ipv4.ttl_expired"}),
                    );
                    return Ok(());
                }
                c.fact(
                    "ipv4.delivered",
                    &device.id,
                    json!({"source":source,"destination":destination,"ttl":ttl,"icmp":icmp}),
                );
                if icmp.message_type == 8 {
                    c.fact(
                        "icmp.echo.received",
                        &device.id,
                        json!({"source":source,"icmp_type":8,"sequence":icmp.sequence}),
                    );
                    if let Ok((port, address, hop)) = self.routing.next_hop(device, *source, c.now)
                    {
                        let reply = Payload::Ipv4 {
                            source: address,
                            destination: *source,
                            ttl: 64,
                            icmp: Icmp {
                                message_type: 0,
                                ..icmp.clone()
                            },
                        };
                        self.resolve(c, &port, hop, reply)?;
                    } else {
                        c.fact(
                            "ipv4.dropped",
                            &device.id,
                            json!({"reason":"routing.no_route"}),
                        );
                    }
                } else if matches!(icmp.message_type, 3 | 11)
                    && let Some((owner, _)) = self.probes.get(&icmp.identifier)
                    && owner == &device.id
                {
                    let (_, start) = self.probes.remove(&icmp.identifier).unwrap();
                    c.fact("icmp.error",&device.id,json!({"rtt_us":(c.now-start).to_string(),"source":source,"icmp_type":icmp.message_type,"icmp_code":icmp.code,"probe":icmp.identifier,"reason":if icmp.message_type==11 {"ipv4.ttl_expired"} else if icmp.code==13 {"routing.forwarding_disabled"} else if icmp.code==1 {"arp.timeout"} else {"routing.no_route"}}));
                } else if icmp.message_type == 0
                    && let Some((owner, start)) = self.probes.get(&icmp.identifier).cloned()
                    && owner == device.id
                {
                    if let Some(responders) = self.broadcast_probes.get_mut(&icmp.identifier) {
                        if !responders.insert(*source) {
                            return Ok(());
                        }
                    } else {
                        self.probes.remove(&icmp.identifier);
                    }
                    c.fact("icmp.echo.reply",&device.id,json!({"source":source,"icmp_type":0,"sequence":icmp.sequence,"probe":icmp.identifier,"rtt_us":(c.now-start).to_string()}));
                }
            }
        }
        Ok(())
    }
}

fn contains_quote(v: &Value) -> bool {
    match v {
        Value::Object(o) => o
            .iter()
            .any(|(k, v)| (k == "quoted" && !v.is_null()) || contains_quote(v)),
        Value::Array(a) => a.iter().any(contains_quote),
        _ => false,
    }
}
