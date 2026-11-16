//! Endpoint-dependent NAPT. No historical packet is modified in place.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Token {
    Port(u16),
    Icmp(Id),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NatEndpoint {
    pub address: Ipv4Addr,
    pub token: Token,
}
#[derive(Clone, Debug, Serialize)]
pub struct Mapping {
    pub id: u64,
    pub device: Id,
    pub protocol: Protocol,
    pub internal: NatEndpoint,
    pub external: NatEndpoint,
    pub remote: Ipv4Addr,
    pub remote_port: Option<u16>,
    pub forwarded: bool,
    pub expires_us: Decimal,
}
#[derive(Clone, Default, serde::Serialize)]
pub struct Nat {
    pub entries: BTreeMap<u64, Mapping>,
    serial: u64,
}
fn tuple(p: &Payload) -> Option<(Protocol, NatEndpoint, NatEndpoint)> {
    let (source, destination, a, b, protocol) = match p {
        Payload::Arp { .. } => return None,
        Payload::Ipv4 {
            source,
            destination,
            icmp,
            ..
        } if matches!(icmp.message_type, 0 | 8) => (
            *source,
            *destination,
            Token::Icmp(icmp.identifier.clone()),
            Token::Icmp(icmp.identifier.clone()),
            Protocol::Icmp,
        ),
        Payload::Ipv4 { .. } => return None,
        Payload::Ipv4Transport {
            source,
            destination,
            packet,
            ..
        } => match packet {
            TransportPacket::Tcp(t) => (
                *source,
                *destination,
                Token::Port(t.source_port),
                Token::Port(t.destination_port),
                Protocol::Tcp,
            ),
            TransportPacket::Udp {
                source_port,
                destination_port,
                ..
            } => (
                *source,
                *destination,
                Token::Port(*source_port),
                Token::Port(*destination_port),
                Protocol::Udp,
            ),
        },
    };
    Some((
        protocol,
        NatEndpoint {
            address: source,
            token: a,
        },
        NatEndpoint {
            address: destination,
            token: b,
        },
    ))
}
fn port(e: &NatEndpoint) -> Option<u16> {
    if let Token::Port(p) = e.token {
        Some(p)
    } else {
        None
    }
}
fn addresses(p: &Payload) -> Option<(Ipv4Addr, Ipv4Addr)> {
    match p {
        Payload::Ipv4 {
            source,
            destination,
            ..
        }
        | Payload::Ipv4Transport {
            source,
            destination,
            ..
        } => Some((*source, *destination)),
        _ => None,
    }
}
fn rewrite(p: &mut Payload, source: bool, e: &NatEndpoint) {
    match p {
        Payload::Ipv4 {
            source: s,
            destination: d,
            icmp,
            ..
        } => {
            if source {
                *s = e.address
            } else {
                *d = e.address
            };
            if let Token::Icmp(id) = &e.token {
                icmp.identifier = id.clone();
            }
        }
        Payload::Ipv4Transport {
            source: s,
            destination: d,
            packet,
            ..
        } => {
            if source {
                *s = e.address
            } else {
                *d = e.address
            };
            let Token::Port(v) = e.token else { return };
            match packet {
                TransportPacket::Tcp(t) => {
                    if source {
                        t.source_port = v
                    } else {
                        t.destination_port = v
                    }
                }
                TransportPacket::Udp {
                    source_port,
                    destination_port,
                    ..
                } => {
                    if source {
                        *source_port = v
                    } else {
                        *destination_port = v
                    }
                }
            }
        }
        _ => {}
    }
}
fn in_lan(host: &Device, g: &GatewayConfig, a: Ipv4Addr) -> bool {
    host.interfaces
        .iter()
        .find(|i| i.id == g.lan)
        .and_then(ip)
        .is_some_and(|(n, p)| u32::from(n) & prefix_mask(p) == u32::from(a) & prefix_mask(p))
}
fn initial(p: &Payload) -> bool {
    match p {
        Payload::Ipv4 { icmp, .. } => icmp.message_type == 8,
        Payload::Ipv4Transport {
            packet: TransportPacket::Tcp(t),
            ..
        } => t.flags.syn && !t.flags.ack,
        Payload::Ipv4Transport {
            packet: TransportPacket::Udp { .. },
            ..
        } => true,
        _ => false,
    }
}
impl std::fmt::Display for NatEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.token {
            Token::Port(p) => write!(f, "{}:{p}", self.address),
            Token::Icmp(id) => write!(f, "{} · ICMP {}", self.address, id.as_str()),
        }
    }
}
impl Network {
    pub(super) fn nat_original(&self, host: &Device, payload: &Payload) -> Option<Payload> {
        let (protocol, external, remote) = tuple(payload)?;
        let e = self.nat.entries.values().find(|e| {
            e.device == host.id
                && e.protocol == protocol
                && e.external == external
                && e.remote == remote.address
                && e.remote_port == port(&remote)
        })?;
        let mut original = payload.clone();
        rewrite(&mut original, true, &e.internal);
        Some(original)
    }
    pub fn invalidate_configuration(&mut self) {
        self.dhcp.effective = None;
    }
    pub(super) fn nat_clear(&mut self, c: &mut Context<'_>, device: &Id, reason: &str) {
        let count = self.nat.entries.len();
        self.nat.entries.retain(|_, e| &e.device != device);
        c.fact(
            "nat.table.cleared",
            device,
            json!({"removed":count-self.nat.entries.len(),"reason":reason}),
        );
    }
    pub(super) fn nat_expire(&mut self, c: &mut Context<'_>, id: u64) -> Result<(), Diagnostic> {
        if let Some(e) = self.nat.entries.get(&id).cloned() {
            if e.expires_us.0 <= c.now {
                self.nat.entries.remove(&id);
                c.fact("nat.mapping.expired", &e.device, json!({"mapping":e}));
            } else {
                c.timer(e.expires_us.0 - c.now, Action::NatExpire { entry: id })?;
            }
        }
        Ok(())
    }
    fn nat_touch(
        &mut self,
        c: &mut Context<'_>,
        entry: u64,
        timeout: u64,
    ) -> Result<Mapping, Diagnostic> {
        let expires = c
            .now
            .checked_add(timeout)
            .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "nat"))?;
        let e = self.nat.entries.get_mut(&entry).unwrap();
        e.expires_us = Decimal(expires);
        Ok(e.clone())
    }
    fn nat_create(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        mut e: Mapping,
        timeout: u64,
    ) -> Result<Option<Mapping>, Diagnostic> {
        if self.nat.entries.len() >= 1024 {
            c.fact("nat.dropped", &host.id, json!({"reason":"nat.table_full"}));
            return Ok(None);
        }
        self.nat.serial = self
            .nat
            .serial
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "nat"))?;
        e.id = self.nat.serial;
        e.expires_us = Decimal(
            c.now
                .checked_add(timeout)
                .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "nat"))?,
        );
        self.nat.entries.insert(e.id, e.clone());
        c.fact("nat.mapping.created", &host.id, json!({"mapping":e}));
        c.timer(timeout, Action::NatExpire { entry: e.id })?;
        Ok(Some(e))
    }
    pub(super) fn nat_inbound(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        incoming: &Id,
        frame: &Frame,
    ) -> Result<Option<Frame>, Diagnostic> {
        let Some(g) = &host.network.gateway else {
            return Ok(Some(frame.clone()));
        };
        let Some((source, destination)) = addresses(&frame.payload) else {
            return Ok(Some(frame.clone()));
        };
        if incoming != &g.wan {
            return Ok(Some(frame.clone()));
        }
        if in_lan(host, g, destination) || in_lan(host, g, source) {
            c.fact(
                "nat.dropped",
                &host.id,
                json!({"reason":"nat.wan_private_scope","interface":incoming,"frame":frame}),
            );
            return Ok(None);
        }
        let Some((wan, _)) = host.interfaces.iter().find(|i| i.id == g.wan).and_then(ip) else {
            return Ok(Some(frame.clone()));
        };
        if destination != wan || !g.nat_enabled {
            return Ok(Some(frame.clone()));
        }
        let mut result = frame.clone();
        if let Payload::Ipv4 { icmp, .. } = &frame.payload
            && matches!(icmp.message_type, 3 | 11)
            && let Some(quote) = &icmp.quoted
        {
            if let Some((protocol, external, remote)) = tuple(quote) {
                let found = self
                    .nat
                    .entries
                    .values()
                    .find(|e| {
                        e.device == host.id
                            && e.protocol == protocol
                            && e.external == external
                            && e.remote == remote.address
                            && e.remote_port == port(&remote)
                    })
                    .map(|e| e.id);
                if let Some(id) = found {
                    let e = self.nat_touch(c, id, g.idle_timeout_us.0)?;
                    if let Payload::Ipv4 {
                        destination, icmp, ..
                    } = &mut result.payload
                    {
                        *destination = e.internal.address;
                        rewrite(icmp.quoted.as_deref_mut().unwrap(), true, &e.internal);
                        if let Token::Icmp(id) = &e.internal.token {
                            icmp.identifier = id.clone();
                        }
                    }
                    c.fact("nat.translated",&host.id,json!({"direction":"inbound_error","mapping":e,"before":frame.payload,"after":result.payload}));
                    return Ok(Some(result));
                }
            }
            c.fact(
                "nat.dropped",
                &host.id,
                json!({"reason":"nat.error_unmapped","frame":frame}),
            );
            return Ok(None);
        }
        let Some((protocol, remote, external)) = tuple(&frame.payload) else {
            return Ok(Some(result));
        };
        let found = self
            .nat
            .entries
            .values()
            .find(|e| {
                e.device == host.id
                    && e.protocol == protocol
                    && e.external == external
                    && e.remote == remote.address
                    && e.remote_port == port(&remote)
            })
            .map(|e| e.id);
        let e = if let Some(id) = found {
            Some(self.nat_touch(c, id, g.idle_timeout_us.0)?)
        } else if let Some(f) = g.forwards.iter().find(|f| match protocol {
            Protocol::Tcp => {
                f.protocol == TransportProtocol::Tcp && Some(f.external_port) == port(&external)
            }
            Protocol::Udp => {
                f.protocol == TransportProtocol::Udp && Some(f.external_port) == port(&external)
            }
            _ => false,
        }) {
            if !initial(&frame.payload) {
                c.fact(
                    "nat.dropped",
                    &host.id,
                    json!({"reason":"nat.tcp_not_initial","frame":frame}),
                );
                return Ok(None);
            }
            self.nat_create(
                c,
                host,
                Mapping {
                    id: 0,
                    device: host.id.clone(),
                    protocol,
                    internal: NatEndpoint {
                        address: f.internal_address,
                        token: Token::Port(f.internal_port),
                    },
                    external,
                    remote: remote.address,
                    remote_port: port(&remote),
                    forwarded: true,
                    expires_us: Decimal(0),
                },
                g.idle_timeout_us.0,
            )?
        } else {
            None
        };
        if let Some(e) = e {
            rewrite(&mut result.payload, false, &e.internal);
            c.fact("nat.translated",&host.id,json!({"direction":"inbound","mapping":e,"before":frame.payload,"after":result.payload}));
        }
        Ok(Some(result))
    }
    pub(super) fn nat_outbound(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        incoming: &Id,
        outgoing: &Id,
        payload: &mut Payload,
    ) -> Result<bool, Diagnostic> {
        let Some(g) = &host.network.gateway else {
            return Ok(true);
        };
        if incoming != &g.lan || outgoing != &g.wan || !g.nat_enabled {
            return Ok(true);
        }
        let Some((source, _)) = addresses(payload) else {
            return Ok(true);
        };
        if !in_lan(host, g, source) {
            c.fact(
                "nat.dropped",
                &host.id,
                json!({"reason":"nat.lan_source_scope","before":payload}),
            );
            return Ok(false);
        }
        let Some((wan, _)) = host.interfaces.iter().find(|i| i.id == g.wan).and_then(ip) else {
            c.fact(
                "nat.dropped",
                &host.id,
                json!({"reason":"nat.wan_unconfigured"}),
            );
            return Ok(false);
        };
        let before = payload.clone();
        if let Payload::Ipv4 { icmp, .. } = payload
            && matches!(icmp.message_type, 3 | 11)
            && let Some(quote) = &icmp.quoted
            && let Some((protocol, remote, internal)) = tuple(quote)
        {
            let found = self
                .nat
                .entries
                .values()
                .find(|e| {
                    e.device == host.id
                        && e.protocol == protocol
                        && e.internal == internal
                        && e.remote == remote.address
                        && e.remote_port == port(&remote)
                })
                .map(|e| e.id);
            if let Some(id) = found {
                let e = self.nat_touch(c, id, g.idle_timeout_us.0)?;
                if let Payload::Ipv4 { source, icmp, .. } = payload {
                    *source = e.external.address;
                    rewrite(icmp.quoted.as_deref_mut().unwrap(), false, &e.external);
                }
                c.fact("nat.translated",&host.id,json!({"direction":"outbound_error","mapping":e,"before":before,"after":payload}));
                return Ok(true);
            }
            c.fact(
                "nat.dropped",
                &host.id,
                json!({"reason":"nat.error_unmapped","before":before}),
            );
            return Ok(false);
        }
        let Some((protocol, internal, remote)) = tuple(payload) else {
            return Ok(true);
        };
        let found = self
            .nat
            .entries
            .values()
            .find(|e| {
                e.device == host.id
                    && e.protocol == protocol
                    && e.internal == internal
                    && e.remote == remote.address
                    && e.remote_port == port(&remote)
                    && e.external.address == wan
            })
            .map(|e| e.id);
        let e = if let Some(id) = found {
            Some(self.nat_touch(c, id, g.idle_timeout_us.0)?)
        } else {
            if !initial(payload) {
                c.fact(
                    "nat.dropped",
                    &host.id,
                    json!({"reason":"nat.tcp_not_initial","before":before}),
                );
                return Ok(false);
            }
            let token = if protocol == Protocol::Icmp {
                Token::Icmp(Id::new(format!(
                    "nat-icmp-{}",
                    self.nat
                        .serial
                        .checked_add(1)
                        .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "nat"))?
                ))?)
            } else {
                let mut used_ports: BTreeSet<u16> = g
                    .forwards
                    .iter()
                    .filter(|f| {
                        matches!(
                            (protocol, f.protocol),
                            (Protocol::Tcp, TransportProtocol::Tcp)
                                | (Protocol::Udp, TransportProtocol::Udp)
                        )
                    })
                    .map(|f| f.external_port)
                    .collect();
                used_ports.extend(
                    self.nat
                        .entries
                        .values()
                        .filter(|e| e.device == host.id && e.protocol == protocol)
                        .filter_map(|e| port(&e.external)),
                );
                used_ports.extend(
                    self.transport
                        .views()
                        .iter()
                        .filter(|s| {
                            s.device == host.id
                                && matches!(
                                    (protocol, s.protocol),
                                    (Protocol::Tcp, TransportProtocol::Tcp)
                                        | (Protocol::Udp, TransportProtocol::Udp)
                                )
                        })
                        .map(|s| s.local.port),
                );
                let used = |p: u16| used_ports.contains(&p);
                let preferred =
                    port(&internal).filter(|p| *p >= g.port_start && *p <= g.port_end && !used(*p));
                let available =
                    preferred.or_else(|| (g.port_start..=g.port_end).find(|p| !used(*p)));
                let Some(port) = available else {
                    c.fact(
                        "nat.dropped",
                        &host.id,
                        json!({"reason":"nat.ports_exhausted","before":before}),
                    );
                    return Ok(false);
                };
                Token::Port(port)
            };
            self.nat_create(
                c,
                host,
                Mapping {
                    id: 0,
                    device: host.id.clone(),
                    protocol,
                    internal,
                    external: NatEndpoint {
                        address: wan,
                        token,
                    },
                    remote: remote.address,
                    remote_port: port(&remote),
                    forwarded: false,
                    expires_us: Decimal(0),
                },
                g.idle_timeout_us.0,
            )?
        };
        let Some(e) = e else { return Ok(false) };
        rewrite(payload, true, &e.external);
        c.fact(
            "nat.translated",
            &host.id,
            json!({"direction":"outbound","mapping":e,"before":before,"after":payload}),
        );
        Ok(true)
    }
}
