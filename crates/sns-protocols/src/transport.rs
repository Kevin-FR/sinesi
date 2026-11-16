//! Bounded event-driven educational TCP/UDP. One MSS in flight per direction.
use crate::routing;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::json;
use sns_model::*;
use std::{
    collections::{BTreeMap, VecDeque},
    net::Ipv4Addr,
};
pub const MSS: usize = 512;
pub const RETRY_US: u64 = 500_000;
pub const MAX_RETRIES: u8 = 4;
pub const TIME_WAIT_US: u64 = 2_000_000;
const CLOSE_TIMEOUT_US: u64 = 5_000_000;
const MAX_SOCKETS: usize = 1024;
const MAX_BUFFER: usize = 1_048_576;
const MAX_TOTAL_BUFFER: usize = 8_388_608;
#[derive(Clone, serde::Serialize)]
struct Binding {
    device: Id,
    socket: Id,
    owner: Id,
    protocol: TransportProtocol,
    interface: Option<Id>,
    address: Option<Ipv4Addr>,
    port: u16,
}
#[derive(Clone, serde::Serialize)]
struct Write {
    data: VecDeque<u8>,
    flow: Id,
}
#[derive(Clone, serde::Serialize)]
struct Outstanding {
    packet: TcpSegment,
    end: u32,
    token: u64,
    retries: u8,
    flow: Id,
}
#[derive(Clone, serde::Serialize)]
struct Session {
    device: Id,
    socket: Id,
    owner: Id,
    local: Endpoint,
    peer: Endpoint,
    state: TcpState,
    send_next: u32,
    send_unacked: u32,
    receive_next: u32,
    outstanding: Option<Outstanding>,
    writes: VecDeque<Write>,
    closing: bool,
    flow: Id,
    close_token: u64,
}
#[derive(Clone, Default, serde::Serialize)]
pub struct Transport {
    bindings: BTreeMap<(Id, Id), Binding>,
    sessions: BTreeMap<(Id, Id), Session>,
    serial: u64,
    ephemeral: BTreeMap<(Id, TransportProtocol), u16>,
}
#[derive(Clone, Copy)]
pub(crate) struct Delivery<'a> {
    pub device: &'a Device,
    pub interface: Option<&'a Id>,
    pub local_address: Ipv4Addr,
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub flow: &'a Id,
}
pub(crate) struct Outbound {
    pub interface: Option<Id>,
    pub device: Id,
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub packet: TransportPacket,
    pub flow: Id,
}
pub(crate) struct Timer {
    pub waits_for_response: bool,
    pub device: Id,
    pub socket: Id,
    pub token: u64,
    pub delay: u64,
    pub flow: Id,
}
#[derive(Default)]
pub(crate) struct Output {
    pub packets: Vec<Outbound>,
    pub timers: Vec<Timer>,
    pub facts: Vec<(Id, Id, &'static str, serde_json::Value)>,
    pub notices: Vec<SocketNotice>,
}
impl Output {
    fn fact(&mut self, s: &Session, flow: &Id, kind: &'static str, value: serde_json::Value) {
        self.facts
            .push((s.device.clone(), flow.clone(), kind, value));
    }
    fn notice(&mut self, s: &Session, flow: &Id, kind: SocketNoticeKind) {
        self.notices.push(SocketNotice {
            device: s.device.clone(),
            socket: s.socket.clone(),
            owner: s.owner.clone(),
            correlation: flow.clone(),
            kind,
        });
    }
    fn packet(&mut self, s: &Session, flow: &Id, packet: TcpSegment) {
        self.packets.push(Outbound {
            interface: None,
            device: s.device.clone(),
            source: s.local.address,
            destination: s.peer.address,
            packet: TransportPacket::Tcp(packet),
            flow: flow.clone(),
        });
    }
}
fn failure(code: &str, id: &Id) -> Diagnostic {
    Diagnostic::new(code, id.as_str())
}
fn bytes(data: &str, limit: usize) -> Result<Vec<u8>, Diagnostic> {
    if data.len() > limit.div_ceil(3) * 4 {
        return Err(Diagnostic::new("resource.limit_exceeded", "socket_data"));
    }
    let data = STANDARD
        .decode(data)
        .map_err(|_| Diagnostic::new("validation.invalid_base64", "socket_data"))?;
    if data.len() > limit {
        return Err(Diagnostic::new("resource.limit_exceeded", "socket_data"));
    }
    Ok(data)
}
fn is_local(device: &Device, address: Ipv4Addr) -> bool {
    address.is_loopback()||device.interfaces.iter().any(|i|matches!(&i.ipv4,Some(Ipv4Config::Static {address:a,..}) if a.parse::<Ipv4Addr>()==Ok(address)))
}
fn local_address(
    device: &Device,
    target: Ipv4Addr,
    requested: Option<Ipv4Addr>,
    routes: &[routing::RouteView],
) -> Result<Ipv4Addr, Diagnostic> {
    if let Some(address) = requested
        && !address.is_unspecified()
    {
        return if is_local(device, address) {
            Ok(address)
        } else {
            Err(failure("socket.address_not_local", &device.id))
        };
    }
    if target.is_loopback() {
        return Ok(Ipv4Addr::LOCALHOST);
    }
    if is_local(device, target) {
        return Ok(target);
    }
    if target.is_broadcast() {
        let mut ports: Vec<_> = device.interfaces.iter().collect();
        ports.sort_by_key(|i| &i.id);
        return ports
            .into_iter()
            .find_map(|i| match &i.ipv4 {
                Some(Ipv4Config::Static { address, .. }) => address.parse().ok(),
                _ => None,
            })
            .map_or(Ok(Ipv4Addr::UNSPECIFIED), Ok);
    }
    routing::hop(device, routes, target).map(|(_, address, _)| address)
}
impl Transport {
    fn token(&mut self) -> Result<u64, Diagnostic> {
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "socket"))?;
        Ok(self.serial)
    }
    fn contains(&self, device: &Id, socket: &Id) -> bool {
        let k = (device.clone(), socket.clone());
        self.bindings.contains_key(&k) || self.sessions.contains_key(&k)
    }
    fn ensure_room(&self) -> Result<(), Diagnostic> {
        if self.bindings.len() + self.sessions.len() >= MAX_SOCKETS {
            Err(Diagnostic::new("resource.limit_exceeded", "sockets"))
        } else {
            Ok(())
        }
    }
    fn busy(
        &self,
        device: &Id,
        protocol: TransportProtocol,
        interface: Option<&Id>,
        address: Option<Ipv4Addr>,
        port: u16,
    ) -> bool {
        self.bindings.values().any(|b| {
            b.device == *device
                && b.protocol == protocol
                && b.port == port
                && (interface.is_none()
                    || b.interface.is_none()
                    || interface == b.interface.as_ref())
                && (address.is_none() || b.address.is_none() || address == b.address)
        }) || (protocol == TransportProtocol::Tcp
            && self.sessions.values().any(|s| {
                s.device == *device
                    && s.local.port == port
                    && address.is_none_or(|a| a == s.local.address)
            }))
    }
    fn allocate(
        &mut self,
        device: &Id,
        protocol: TransportProtocol,
        interface: Option<&Id>,
        address: Option<Ipv4Addr>,
        requested: u16,
    ) -> Result<u16, Diagnostic> {
        if requested != 0 {
            return if self.busy(device, protocol, interface, address, requested) {
                Err(failure("socket.port_in_use", device))
            } else {
                Ok(requested)
            };
        }
        let start = *self
            .ephemeral
            .get(&(device.clone(), protocol))
            .unwrap_or(&49152);
        for offset in 0..16384_u32 {
            let port = 49152 + ((u32::from(start) - 49152 + offset) % 16384) as u16;
            if !self.busy(device, protocol, interface, address, port) {
                self.ephemeral.insert(
                    (device.clone(), protocol),
                    if port == 65535 { 49152 } else { port + 1 },
                );
                return Ok(port);
            }
        }
        Err(failure("socket.no_ephemeral_port", device))
    }
    pub fn views(&self) -> Vec<SocketView> {
        let mut views: Vec<_> = self
            .bindings
            .values()
            .map(|b| SocketView {
                device: b.device.clone(),
                socket: b.socket.clone(),
                owner: b.owner.clone(),
                protocol: b.protocol,
                interface: b.interface.clone(),
                local: Endpoint {
                    address: b.address.unwrap_or(Ipv4Addr::UNSPECIFIED),
                    port: b.port,
                },
                peer: None,
                state: if b.protocol == TransportProtocol::Tcp {
                    "LISTEN"
                } else {
                    "BOUND"
                }
                .into(),
            })
            .collect();
        views.extend(self.sessions.values().map(|s| {
            SocketView {
                device: s.device.clone(),
                socket: s.socket.clone(),
                owner: s.owner.clone(),
                protocol: TransportProtocol::Tcp,
                interface: None,
                local: s.local,
                peer: Some(s.peer),
                state: serde_json::to_value(s.state)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .into(),
            }
        }));
        views.sort_by(|a, b| (&a.device, &a.socket).cmp(&(&b.device, &b.socket)));
        views
    }
    pub(crate) fn command(
        &mut self,
        project: &Project,
        device: &Id,
        command: &SocketCommand,
        flow: &Id,
        routes: &[routing::RouteView],
    ) -> Result<Output, Diagnostic> {
        let host = project
            .device(device)
            .filter(|d| {
                matches!(
                    d.kind,
                    DeviceKind::Workstation
                        | DeviceKind::Notebook
                        | DeviceKind::Router
                        | DeviceKind::Gateway
                )
            })
            .ok_or_else(|| failure("device.not_host", device))?;
        let mut out = Output::default();
        match command {
            SocketCommand::Bind {
                socket,
                owner,
                protocol,
                interface,
                address,
                port,
            } => {
                self.ensure_room()?;
                if self.contains(device, socket) {
                    return Err(failure("socket.id_in_use", socket));
                }
                let nic = interface
                    .as_ref()
                    .map(|id| {
                        host.interfaces
                            .iter()
                            .find(|i| &i.id == id)
                            .ok_or_else(|| failure("socket.invalid_interface", id))
                    })
                    .transpose()?;
                let mut address = address.filter(|a| !a.is_unspecified());
                if let Some(nic) = nic {
                    let nic_ip = match &nic.ipv4 {
                        Some(Ipv4Config::Static { address, .. }) => address.parse().ok(),
                        _ => None,
                    };
                    if address.is_some() && address != nic_ip {
                        return Err(failure("socket.address_not_local", socket));
                    }
                    if *protocol == TransportProtocol::Tcp {
                        address = Some(
                            nic_ip.ok_or_else(|| failure("socket.address_not_local", socket))?,
                        );
                    }
                }
                if address.is_some_and(|a| !is_local(host, a)) {
                    return Err(failure("socket.address_not_local", device));
                }
                let port = self.allocate(device, *protocol, interface.as_ref(), address, *port)?;
                self.bindings.insert(
                    (device.clone(), socket.clone()),
                    Binding {
                        device: device.clone(),
                        socket: socket.clone(),
                        owner: owner.clone(),
                        protocol: *protocol,
                        interface: interface.clone(),
                        address,
                        port,
                    },
                );
                out.facts.push((device.clone(),flow.clone(),"socket.bound",json!({"socket":socket,"owner":owner,"protocol":protocol,"address":address.unwrap_or(Ipv4Addr::UNSPECIFIED),"port":port})));
                out.notices.push(SocketNotice {
                    device: device.clone(),
                    socket: socket.clone(),
                    owner: owner.clone(),
                    correlation: flow.clone(),
                    kind: SocketNoticeKind::Listening,
                });
            }
            SocketCommand::Connect {
                socket,
                owner,
                destination,
                port,
                local_address: requested,
                local_port,
            } => {
                self.ensure_room()?;
                if self.contains(device, socket) {
                    return Err(failure("socket.id_in_use", socket));
                }
                if *port == 0
                    || destination.is_broadcast()
                    || host.interfaces.iter().any(|i| match &i.ipv4 {
                        Some(Ipv4Config::Static {
                            address,
                            prefix_length,
                        }) if *prefix_length < 31 => address.parse().ok().is_some_and(|a| {
                            crate::subnet(a, *prefix_length)
                                .is_ok_and(|(_, broadcast)| broadcast == *destination)
                        }),
                        _ => false,
                    })
                    || destination.is_multicast()
                    || destination.is_unspecified()
                {
                    return Err(failure("socket.invalid_endpoint", socket));
                }
                let address = local_address(host, *destination, *requested, routes)?;
                let local_port = self.allocate(
                    device,
                    TransportProtocol::Tcp,
                    None,
                    Some(address),
                    local_port.unwrap_or(0),
                )?;
                let serial = self.token()?;
                let isn = initial_sequence(socket, serial);
                let mut s = Session {
                    device: device.clone(),
                    socket: socket.clone(),
                    owner: owner.clone(),
                    local: Endpoint {
                        address,
                        port: local_port,
                    },
                    peer: Endpoint {
                        address: *destination,
                        port: *port,
                    },
                    state: TcpState::SynSent,
                    send_next: isn,
                    send_unacked: isn,
                    receive_next: 0,
                    outstanding: None,
                    writes: VecDeque::new(),
                    closing: false,
                    flow: flow.clone(),
                    close_token: 0,
                };
                out.fact(
                    &s,
                    flow,
                    "tcp.state",
                    json!({"socket":socket,"state":"SYN_SENT","local":s.local,"peer":s.peer}),
                );
                let packet = Self::segment(
                    &s,
                    TcpFlags {
                        syn: true,
                        ..Default::default()
                    },
                    vec![],
                );
                self.transmit(&mut s, packet, flow, &mut out)?;
                self.sessions.insert((device.clone(), socket.clone()), s);
            }
            SocketCommand::Send {
                socket,
                data_base64,
            } => {
                let data = bytes(data_base64, MAX_BUFFER)?;
                let total: usize = self.sessions.values().map(buffered).sum();
                if total.saturating_add(data.len()) > MAX_TOTAL_BUFFER {
                    return Err(failure("resource.limit_exceeded", socket));
                }
                let key = (device.clone(), socket.clone());
                let mut s = self
                    .sessions
                    .remove(&key)
                    .ok_or_else(|| failure("socket.not_connected", socket))?;
                if s.state != TcpState::Established || s.closing {
                    return Err(failure("socket.not_connected", socket));
                }
                if buffered(&s).saturating_add(data.len()) > MAX_BUFFER {
                    return Err(failure("resource.limit_exceeded", socket));
                }
                if !data.is_empty() {
                    s.writes.push_back(Write {
                        data: data.into(),
                        flow: flow.clone(),
                    });
                    self.pump(&mut s, &mut out)?;
                }
                self.sessions.insert(key, s);
            }
            SocketCommand::SendTo {
                socket,
                destination,
                port,
                data_base64,
            } => {
                let data = bytes(data_base64, 65507)?;
                if *port == 0 || destination.is_unspecified() || destination.is_multicast() {
                    return Err(failure("socket.invalid_endpoint", socket));
                }
                let b = self
                    .bindings
                    .get(&(device.clone(), socket.clone()))
                    .filter(|b| b.protocol == TransportProtocol::Udp)
                    .ok_or_else(|| failure("socket.not_bound", socket))?;
                let scoped_address = b
                    .interface
                    .as_ref()
                    .and_then(|id| host.interfaces.iter().find(|i| &i.id == id))
                    .and_then(|nic| match &nic.ipv4 {
                        Some(Ipv4Config::Static { address, .. }) => address.parse().ok(),
                        _ => None,
                    });
                let source =
                    local_address(host, *destination, b.address.or(scoped_address), routes)?;
                out.packets.push(Outbound {
                    interface: b.interface.clone(),
                    device: device.clone(),
                    source,
                    destination: *destination,
                    packet: TransportPacket::Udp {
                        source_port: b.port,
                        destination_port: *port,
                        length: (data.len() + 8) as u16,
                        data_base64: STANDARD.encode(data),
                    },
                    flow: flow.clone(),
                });
            }
            SocketCommand::Close { socket, force } => {
                self.close(device, socket, *force, flow, &mut out)?;
            }
            SocketCommand::CloseOwner { owner } => {
                let sockets: Vec<_> = self
                    .bindings
                    .values()
                    .filter(|b| b.device == *device && b.owner == *owner)
                    .map(|b| b.socket.clone())
                    .chain(
                        self.sessions
                            .values()
                            .filter(|s| s.device == *device && s.owner == *owner)
                            .map(|s| s.socket.clone()),
                    )
                    .collect();
                for socket in sockets {
                    self.close(device, &socket, true, flow, &mut out)?;
                }
            }
        }
        Ok(out)
    }
    fn close(
        &mut self,
        device: &Id,
        socket: &Id,
        force: bool,
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let key = (device.clone(), socket.clone());
        if let Some(b) = self.bindings.remove(&key) {
            out.facts.push((
                device.clone(),
                flow.clone(),
                "socket.closed",
                json!({"socket":socket,"reason":"socket.closed"}),
            ));
            out.notices.push(SocketNotice {
                device: device.clone(),
                socket: socket.clone(),
                owner: b.owner,
                correlation: flow.clone(),
                kind: SocketNoticeKind::Closed {
                    reason: "socket.closed".into(),
                },
            });
            return Ok(());
        }
        let mut s = self
            .sessions
            .remove(&key)
            .ok_or_else(|| failure("socket.not_found", socket))?;
        if force || matches!(s.state, TcpState::SynSent | TcpState::SynReceived) {
            out.packet(
                &s,
                flow,
                Self::segment(
                    &s,
                    TcpFlags {
                        rst: true,
                        ack: true,
                        ..Default::default()
                    },
                    vec![],
                ),
            );
            Self::closed(&s, flow, "socket.aborted", out);
        } else {
            s.closing = true;
            self.pump(&mut s, out)?;
            self.sessions.insert(key, s);
        }
        Ok(())
    }
    fn segment(s: &Session, flags: TcpFlags, data: Vec<u8>) -> TcpSegment {
        TcpSegment {
            source_port: s.local.port,
            destination_port: s.peer.port,
            sequence: s.send_next,
            acknowledgment: s.receive_next,
            flags,
            window: MSS as u16,
            data_base64: STANDARD.encode(data),
        }
    }
    fn transmit(
        &mut self,
        s: &mut Session,
        packet: TcpSegment,
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let size = bytes(&packet.data_base64, MSS)?.len() as u32
            + u32::from(packet.flags.syn)
            + u32::from(packet.flags.fin);
        let end = packet.sequence.wrapping_add(size);
        s.send_next = end;
        let token = self.token()?;
        s.outstanding = Some(Outstanding {
            packet: packet.clone(),
            end,
            token,
            retries: 0,
            flow: flow.clone(),
        });
        out.packet(s, flow, packet);
        out.timers.push(Timer {
            waits_for_response: true,
            device: s.device.clone(),
            socket: s.socket.clone(),
            token,
            delay: RETRY_US,
            flow: flow.clone(),
        });
        Ok(())
    }
    fn ack(s: &Session, flow: &Id, out: &mut Output) {
        out.packet(
            s,
            flow,
            Self::segment(
                s,
                TcpFlags {
                    ack: true,
                    ..Default::default()
                },
                vec![],
            ),
        );
    }
    fn pump(&mut self, s: &mut Session, out: &mut Output) -> Result<(), Diagnostic> {
        if s.outstanding.is_some()
            || !matches!(s.state, TcpState::Established | TcpState::CloseWait)
        {
            return Ok(());
        }
        if let Some(write) = s.writes.front_mut() {
            let data: Vec<_> = write.data.drain(..write.data.len().min(MSS)).collect();
            let flow = write.flow.clone();
            if write.data.is_empty() {
                s.writes.pop_front();
            }
            let packet = Self::segment(
                s,
                TcpFlags {
                    ack: true,
                    ..Default::default()
                },
                data,
            );
            self.transmit(s, packet, &flow, out)?;
        } else if s.closing {
            s.state = if s.state == TcpState::CloseWait {
                TcpState::LastAck
            } else {
                TcpState::FinWait1
            };
            let flow = s.flow.clone();
            out.fact(
                s,
                &flow,
                "tcp.state",
                json!({"socket":s.socket,"state":s.state,"local":s.local,"peer":s.peer}),
            );
            let fin = Self::segment(
                s,
                TcpFlags {
                    fin: true,
                    ack: true,
                    ..Default::default()
                },
                vec![],
            );
            self.transmit(s, fin, &flow, out)?;
        }
        Ok(())
    }
    fn closed(s: &Session, flow: &Id, reason: &str, out: &mut Output) {
        out.fact(s,flow,"tcp.state",json!({"socket":s.socket,"state":"CLOSED","reason":reason,"local":s.local,"peer":s.peer}));
        out.notice(
            s,
            flow,
            SocketNoticeKind::Closed {
                reason: reason.into(),
            },
        );
    }
    pub(crate) fn timeout(
        &mut self,
        device: &Id,
        socket: &Id,
        token: u64,
        flow: &Id,
    ) -> Result<Output, Diagnostic> {
        let mut out = Output::default();
        let key = (device.clone(), socket.clone());
        let Some(mut s) = self.sessions.remove(&key) else {
            return Ok(out);
        };
        if s.close_token == token && matches!(s.state, TcpState::TimeWait | TcpState::FinWait2) {
            Self::closed(
                &s,
                flow,
                if s.state == TcpState::TimeWait {
                    "socket.closed"
                } else {
                    "tcp.close_timeout"
                },
                &mut out,
            );
            return Ok(out);
        }
        if let Some(mut outstanding) = s.outstanding.take() {
            if outstanding.token == token {
                if outstanding.retries >= MAX_RETRIES {
                    Self::closed(&s, &outstanding.flow, "tcp.timeout", &mut out);
                    return Ok(out);
                }
                outstanding.retries += 1;
                out.fact(&s,&outstanding.flow,"tcp.retransmit",json!({"socket":s.socket,"sequence":outstanding.packet.sequence,"attempt":outstanding.retries}));
                out.packet(&s, &outstanding.flow, outstanding.packet.clone());
                out.timers.push(Timer {
                    waits_for_response: true,
                    device: device.clone(),
                    socket: socket.clone(),
                    token,
                    delay: RETRY_US << outstanding.retries,
                    flow: outstanding.flow.clone(),
                });
            }
            s.outstanding = Some(outstanding);
        }
        self.sessions.insert(key, s);
        Ok(out)
    }
    pub(crate) fn receive(
        &mut self,
        delivery: Delivery<'_>,
        packet: &TransportPacket,
    ) -> Result<Output, Diagnostic> {
        let Delivery {
            device,
            interface,
            local_address,
            source,
            destination,
            flow,
        } = delivery;
        let mut out = Output::default();
        match packet {
            TransportPacket::Udp {
                source_port,
                destination_port,
                length,
                data_base64,
            } => {
                let data = bytes(data_base64, 65507)?;
                if *length as usize != data.len() + 8 {
                    return Err(failure("udp.invalid_length", &device.id));
                }
                out.facts.push((device.id.clone(),flow.clone(),"udp.datagram.received",json!({"source":source,"destination":destination,"packet":packet,"interface":interface})));
                let mut delivered = false;
                for b in self.bindings.values().filter(|b| {
                    b.device == device.id
                        && b.protocol == TransportProtocol::Udp
                        && b.port == *destination_port
                        && b.address.is_none_or(|a| a == local_address)
                        && b.interface.as_ref().is_none_or(|id| Some(id) == interface)
                }) {
                    delivered = true;
                    out.notices.push(SocketNotice {
                        device: device.id.clone(),
                        socket: b.socket.clone(),
                        owner: b.owner.clone(),
                        correlation: flow.clone(),
                        kind: SocketNoticeKind::Datagram {
                            source: Endpoint {
                                address: source,
                                port: *source_port,
                            },
                            destination: Endpoint {
                                address: destination,
                                port: *destination_port,
                            },
                            data_base64: data_base64.clone(),
                        },
                    });
                }
                if !delivered {
                    out.facts.push((
                        device.id.clone(),
                        flow.clone(),
                        "udp.datagram.dropped",
                        json!({"reason":"socket.port_unreachable","packet":packet}),
                    ));
                }
            }
            TransportPacket::Tcp(segment) => self.receive_tcp(delivery, segment, &mut out)?,
        }
        Ok(out)
    }
    fn receive_tcp(
        &mut self,
        delivery: Delivery<'_>,
        packet: &TcpSegment,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let Delivery {
            device,
            interface,
            source,
            destination,
            flow,
            ..
        } = delivery;
        let data = bytes(&packet.data_base64, MSS)?;
        out.facts.push((device.id.clone(),flow.clone(),"tcp.segment.received",json!({"source":source,"destination":destination,"packet":TransportPacket::Tcp(packet.clone()),"interface":interface})));
        let local = Endpoint {
            address: destination,
            port: packet.destination_port,
        };
        let peer = Endpoint {
            address: source,
            port: packet.source_port,
        };
        let key = self
            .sessions
            .iter()
            .find(|(_, s)| s.device == device.id && s.local == local && s.peer == peer)
            .map(|(k, _)| k.clone());
        let Some(key) = key else {
            if packet.flags.rst {
                return Ok(());
            }
            if packet.flags.syn
                && !packet.flags.ack
                && let Some(b) = self
                    .bindings
                    .values()
                    .find(|b| {
                        b.device == device.id
                            && b.protocol == TransportProtocol::Tcp
                            && b.port == local.port
                            && b.address.is_none_or(|a| a == local.address)
                            && b.interface.as_ref().is_none_or(|id| Some(id) == interface)
                    })
                    .cloned()
            {
                self.ensure_room()?;
                let socket = loop {
                    let serial = self.token()?;
                    let id = Id::new(format!("tcp-accepted-{serial}"))?;
                    if !self.contains(&device.id, &id) {
                        break id;
                    }
                };
                let isn = initial_sequence(&socket, self.serial);
                let mut s = Session {
                    device: device.id.clone(),
                    socket: socket.clone(),
                    owner: b.owner,
                    local,
                    peer,
                    state: TcpState::SynReceived,
                    send_next: isn,
                    send_unacked: isn,
                    receive_next: packet.sequence.wrapping_add(1),
                    outstanding: None,
                    writes: VecDeque::new(),
                    closing: false,
                    flow: flow.clone(),
                    close_token: 0,
                };
                out.fact(
                    &s,
                    flow,
                    "tcp.state",
                    json!({"socket":s.socket,"state":"SYN_RECEIVED","local":local,"peer":peer}),
                );
                let reply = Self::segment(
                    &s,
                    TcpFlags {
                        syn: true,
                        ack: true,
                        ..Default::default()
                    },
                    vec![],
                );
                self.transmit(&mut s, reply, flow, out)?;
                self.sessions.insert((device.id.clone(), socket), s);
            } else {
                let next = packet
                    .sequence
                    .wrapping_add(data.len() as u32)
                    .wrapping_add(u32::from(packet.flags.syn))
                    .wrapping_add(u32::from(packet.flags.fin));
                out.packets.push(Outbound {
                    interface: None,
                    device: device.id.clone(),
                    source: destination,
                    destination: source,
                    packet: TransportPacket::Tcp(TcpSegment {
                        source_port: local.port,
                        destination_port: peer.port,
                        sequence: packet.acknowledgment,
                        acknowledgment: next,
                        flags: TcpFlags {
                            rst: true,
                            ack: true,
                            ..Default::default()
                        },
                        window: 0,
                        data_base64: String::new(),
                    }),
                    flow: flow.clone(),
                });
            }
            return Ok(());
        };
        let mut s = self.sessions.remove(&key).unwrap();
        if packet.flags.rst {
            Self::closed(&s, flow, "tcp.reset", out);
            return Ok(());
        }
        if s.state == TcpState::SynSent {
            if packet.flags.syn && packet.flags.ack && packet.acknowledgment == s.send_next {
                s.send_unacked = s.send_next;
                s.outstanding = None;
                s.receive_next = packet.sequence.wrapping_add(1);
                s.state = TcpState::Established;
                Self::ack(&s, flow, out);
                out.fact(
                    &s,
                    flow,
                    "tcp.state",
                    json!({"socket":s.socket,"state":"ESTABLISHED","local":s.local,"peer":s.peer}),
                );
                out.notice(&s, flow, SocketNoticeKind::Connected);
            }
            self.sessions.insert(key, s);
            return Ok(());
        }
        if packet.flags.syn {
            if s.state == TcpState::SynReceived {
                if let Some(p) = &s.outstanding {
                    out.packet(&s, &p.flow, p.packet.clone());
                }
            } else if packet.sequence.wrapping_add(1) == s.receive_next {
                Self::ack(&s, flow, out);
            }
            self.sessions.insert(key, s);
            return Ok(());
        }
        if packet.flags.ack {
            let acked = s
                .outstanding
                .as_ref()
                .is_some_and(|p| p.end == packet.acknowledgment);
            if acked {
                s.send_unacked = packet.acknowledgment;
                s.outstanding = None;
                match s.state {
                    TcpState::SynReceived => {
                        s.state = TcpState::Established;
                        out.fact(&s,flow,"tcp.state",json!({"socket":s.socket,"state":"ESTABLISHED","local":s.local,"peer":s.peer}));
                        out.notice(&s, flow, SocketNoticeKind::Accepted);
                    }
                    TcpState::FinWait1 => {
                        s.state = TcpState::FinWait2;
                        s.close_token = self.token()?;
                        out.timers.push(Timer {
                            waits_for_response: true,
                            device: s.device.clone(),
                            socket: s.socket.clone(),
                            token: s.close_token,
                            delay: CLOSE_TIMEOUT_US,
                            flow: flow.clone(),
                        });
                    }
                    TcpState::Closing => self.time_wait(&mut s, flow, out)?,
                    TcpState::LastAck => {
                        Self::closed(&s, flow, "socket.closed", out);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
        if matches!(
            s.state,
            TcpState::Established | TcpState::FinWait1 | TcpState::FinWait2 | TcpState::CloseWait
        ) {
            if !data.is_empty() {
                if packet.sequence == s.receive_next {
                    s.receive_next = s.receive_next.wrapping_add(data.len() as u32);
                    out.notice(
                        &s,
                        flow,
                        SocketNoticeKind::Data {
                            data_base64: packet.data_base64.clone(),
                        },
                    );
                } else {
                    out.fact(&s,flow,"tcp.segment.duplicate_or_gap",json!({"socket":s.socket,"expected":s.receive_next,"sequence":packet.sequence}));
                }
                Self::ack(&s, flow, out);
            }
            if packet.flags.fin {
                let fin_sequence = packet.sequence.wrapping_add(data.len() as u32);
                if fin_sequence == s.receive_next {
                    s.receive_next = s.receive_next.wrapping_add(1);
                    Self::ack(&s, flow, out);
                    out.notice(&s, flow, SocketNoticeKind::PeerClosed);
                    match s.state {
                        TcpState::FinWait1 => s.state = TcpState::Closing,
                        TcpState::FinWait2 => self.time_wait(&mut s, flow, out)?,
                        TcpState::Established => {
                            s.state = TcpState::CloseWait;
                            s.closing = true;
                        }
                        _ => {}
                    }
                } else {
                    Self::ack(&s, flow, out);
                }
            }
        } else if s.state == TcpState::TimeWait && packet.flags.fin {
            Self::ack(&s, flow, out);
        }
        self.pump(&mut s, out)?;
        self.sessions.insert(key, s);
        Ok(())
    }
    fn time_wait(
        &mut self,
        s: &mut Session,
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        s.state = TcpState::TimeWait;
        s.close_token = self.token()?;
        out.fact(
            s,
            flow,
            "tcp.state",
            json!({"socket":s.socket,"state":"TIME_WAIT","local":s.local,"peer":s.peer}),
        );
        out.timers.push(Timer {
            waits_for_response: false,
            device: s.device.clone(),
            socket: s.socket.clone(),
            token: s.close_token,
            delay: TIME_WAIT_US,
            flow: flow.clone(),
        });
        Ok(())
    }
}
fn initial_sequence(id: &Id, serial: u64) -> u32 {
    let mut value = 2166136261_u32;
    for byte in id.as_str().bytes() {
        value = (value ^ u32::from(byte)).wrapping_mul(16777619);
    }
    value.wrapping_add((serial as u32).wrapping_mul(104729))
}
fn buffered(s: &Session) -> usize {
    s.writes.iter().map(|w| w.data.len()).sum::<usize>()
        + s.outstanding
            .as_ref()
            .map(|p| p.packet.data_base64.len().div_ceil(4) * 3)
            .unwrap_or(0)
}
