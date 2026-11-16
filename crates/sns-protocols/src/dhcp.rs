//! Bounded, LAN-only DHCP model carried by simulated UDP broadcasts.
use super::*;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Deserialize;
use std::sync::Arc;
const RETRY: u64 = 2_000_000;
const PROBE: u64 = 250_000;
#[derive(Clone, Debug, Serialize)]
pub struct Assignment {
    pub address: Ipv4Addr,
    pub prefix_length: u8,
    pub gateway: Option<Ipv4Addr>,
    pub dns: Option<Ipv4Addr>,
    pub expires_us: Decimal,
    pub link_local: bool,
}
#[derive(Clone, Default, serde::Serialize)]
pub struct Dhcp {
    pub assigned: BTreeMap<Id, Assignment>,
    clients: BTreeMap<Id, Client>,
    offers: BTreeMap<(Id, String), Lease>,
    leases: BTreeMap<(Id, String), Lease>,
    refused: BTreeMap<(Id, Ipv4Addr), u64>,
    pub(super) effective: Option<Arc<Project>>,
}
#[derive(Clone, serde::Serialize)]
struct Lease {
    address: Ipv4Addr,
    expires: u64,
}
#[derive(Clone, serde::Serialize)]
struct Client {
    token: u64,
    attempt: u8,
    phase: Phase,
    offer: Option<Wire>,
}
#[derive(Clone, PartialEq, serde::Serialize)]
enum Phase {
    Discover,
    Probe,
    Request,
    Bound,
    Failed,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    kind: String,
    mac: String,
    xid: u64,
    server: Option<Ipv4Addr>,
    address: Option<Ipv4Addr>,
    prefix: u8,
    gateway: Option<Ipv4Addr>,
    dns: Option<Ipv4Addr>,
    lease: u64,
}
impl Dhcp {
    pub(super) fn uses_interface(&self, id: &Id) -> bool {
        self.clients.contains_key(id)
    }
}
impl Network {
    /// The document retains DHCP mode. Only this projection contains acquired addresses.
    pub fn effective_project(&self, base: &Project) -> Project {
        let mut p = base.clone();
        for d in &mut p.devices {
            for i in &mut d.interfaces {
                if matches!(i.ipv4, Some(Ipv4Config::Dhcp))
                    && let Some(a) = self.dhcp.assigned.get(&i.id)
                {
                    i.ipv4 = Some(Ipv4Config::Static {
                        address: a.address.to_string(),
                        prefix_length: a.prefix_length,
                    });
                    d.network.default_gateway = a.gateway.map(|a| a.to_string());
                    d.network.dns_server = a.dns.map(|a| a.to_string());
                }
            }
        }
        p
    }
    pub(super) fn dhcp_start(
        &mut self,
        c: &mut Context<'_>,
        port: &Id,
        attempt: u8,
    ) -> Result<(), Diagnostic> {
        let host = c
            .project
            .device_for_interface(port)
            .ok_or_else(|| Diagnostic::new("project.invalid_reference", port.as_str()))?;
        let nic = host.interfaces.iter().find(|i| &i.id == port).unwrap();
        self.serial += 1;
        let token = self.serial;
        if host
            .network
            .gateway
            .as_ref()
            .is_some_and(|g| &g.wan == port)
        {
            self.nat_clear(c, &host.id, "wan_address_changed");
        }
        self.dhcp.assigned.remove(port);
        self.dhcp.effective = None;
        let w = Wire {
            kind: "discover".into(),
            mac: nic.mac.to_ascii_lowercase(),
            xid: token,
            server: None,
            address: None,
            prefix: 16,
            gateway: None,
            dns: None,
            lease: 0,
        };
        self.dhcp.clients.insert(
            port.clone(),
            Client {
                token,
                attempt,
                phase: Phase::Discover,
                offer: None,
            },
        );
        self.dhcp_emit(c, port, &w, false)?;
        c.timer(
            RETRY,
            Action::DhcpTick {
                interface: port.clone(),
                token,
            },
        )
    }
    fn dhcp_emit(
        &mut self,
        c: &mut Context<'_>,
        port: &Id,
        w: &Wire,
        server: bool,
    ) -> Result<(), Diagnostic> {
        let host = c.project.device_for_interface(port).unwrap();
        let nic = host.interfaces.iter().find(|i| &i.id == port).unwrap();
        let data_base64 = STANDARD.encode(serde_json::to_vec(w).unwrap());
        let packet = TransportPacket::Udp {
            source_port: if server { 67 } else { 68 },
            destination_port: if server { 68 } else { 67 },
            length: (8 + STANDARD.decode(&data_base64).unwrap().len()) as u16,
            data_base64,
        };
        let frame = self.frame(
            nic.mac.to_ascii_lowercase(),
            BROADCAST.into(),
            Payload::Ipv4Transport {
                source: if server {
                    ip(nic).unwrap().0
                } else {
                    Ipv4Addr::UNSPECIFIED
                },
                destination: Ipv4Addr::BROADCAST,
                ttl: 64,
                packet,
            },
        )?;
        let kind = match w.kind.as_str() {
            "discover" => "dhcp.discover",
            "offer" => "dhcp.offer",
            "request" => "dhcp.request",
            "ack" => "dhcp.ack",
            "nack" => "dhcp.nack",
            _ => "dhcp.decline",
        };
        c.fact(kind,&host.id,json!({"interface":port,"transaction":w.xid,"mac":w.mac,"address":w.address,"server":w.server}));
        self.send(c, port, frame);
        Ok(())
    }
    pub(super) fn dhcp_receive(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        port: &Id,
        frame: &Frame,
    ) -> Result<bool, Diagnostic> {
        // An ARP response or announcement for the candidate invalidates the probe.
        if let Payload::Arp {
            sender_ip,
            sender_mac,
            target_ip,
            ..
        } = &frame.payload
        {
            let conflict = self
                .dhcp
                .clients
                .get(port)
                .and_then(|cl| cl.offer.as_ref().map(|w| (cl, w)))
                .is_some_and(|(cl, w)| {
                    cl.phase == Phase::Probe
                        && (w.address == Some(*sender_ip)
                            || (sender_ip.is_unspecified() && w.address == Some(*target_ip)))
                        && !sender_mac.eq_ignore_ascii_case(&w.mac)
                });
            if conflict {
                self.dhcp_conflict(c, host, port)?;
            }
            return Ok(false);
        }
        let Payload::Ipv4Transport {
            packet:
                TransportPacket::Udp {
                    source_port,
                    destination_port,
                    data_base64,
                    ..
                },
            ..
        } = &frame.payload
        else {
            return Ok(false);
        };
        if !matches!((*source_port, *destination_port), (67, 68) | (68, 67)) {
            return Ok(false);
        }
        if host
            .network
            .dhcp_server
            .as_ref()
            .is_none_or(|cfg| &cfg.interface != port)
            && !self.dhcp.clients.contains_key(port)
        {
            return Ok(false);
        }
        let Ok(data) = STANDARD.decode(data_base64) else {
            return Ok(true);
        };
        if data.len() > 4096 {
            return Ok(true);
        }
        let Ok(w) = serde_json::from_slice::<Wire>(&data) else {
            return Ok(true);
        };
        if *destination_port == 67 {
            let Some(cfg) = host
                .network
                .dhcp_server
                .as_ref()
                .filter(|cfg| &cfg.interface == port)
            else {
                return Ok(true);
            };
            self.dhcp_sweep(c, &host.id);
            let server = ip(host.interfaces.iter().find(|i| &i.id == port).unwrap())
                .unwrap()
                .0;
            let key = (host.id.clone(), w.mac.to_ascii_lowercase());
            let mut response = w.clone();
            response.server = Some(server);
            response.prefix = cfg.prefix_length;
            response.lease = cfg.lease_us.0;
            response.gateway = cfg.gateway.or_else(|| {
                cfg.use_host_options
                    .then(|| {
                        host.network
                            .default_gateway
                            .as_ref()
                            .and_then(|s| s.parse().ok())
                    })
                    .flatten()
            });
            response.dns = cfg.dns.or_else(|| {
                cfg.use_host_options
                    .then(|| {
                        host.network
                            .dns_server
                            .as_ref()
                            .and_then(|s| s.parse().ok())
                    })
                    .flatten()
            });
            match w.kind.as_str() {
                "discover" => {
                    let occupied = |a: Ipv4Addr| {
                        a == server
                            || self.dhcp.refused.contains_key(&(host.id.clone(), a))
                            || self
                                .dhcp
                                .leases
                                .iter()
                                .chain(self.dhcp.offers.iter())
                                .any(|(k, l)| k.0 == host.id && *k != key && l.address == a)
                    };
                    let reserved = cfg
                        .reservations
                        .iter()
                        .find(|(m, _)| m.eq_ignore_ascii_case(&w.mac))
                        .map(|(_, a)| *a);
                    let address = reserved.filter(|a| !occupied(*a)).or_else(|| {
                        if reserved.is_some() {
                            None
                        } else {
                            (u32::from(cfg.pool_start)..=u32::from(cfg.pool_end))
                                .map(Ipv4Addr::from)
                                .find(|a| {
                                    !cfg.reservations.values().any(|r| r == a) && !occupied(*a)
                                })
                        }
                    });
                    if let Some(a) = address {
                        self.dhcp.offers.insert(
                            key,
                            Lease {
                                address: a,
                                expires: deadline(c.now, c.wait_factor.scale(3_000_000)?)?,
                            },
                        );
                        response.address = Some(a);
                        response.kind = "offer".into();
                        self.dhcp_emit(c, port, &response, true)?;
                        c.timer(
                            c.wait_factor.scale(3_000_000)?,
                            Action::DhcpSweep {
                                device: host.id.clone(),
                            },
                        )?;
                    } else {
                        c.fact("dhcp.pool.exhausted", &host.id, json!({"mac":w.mac}));
                    }
                }
                "request" if w.server == Some(server) => {
                    let valid = self
                        .dhcp
                        .offers
                        .get(&key)
                        .is_some_and(|l| Some(l.address) == w.address);
                    response.kind = if valid { "ack" } else { "nack" }.into();
                    if valid {
                        let offer = self.dhcp.offers.remove(&key).unwrap();
                        self.dhcp.leases.insert(
                            key,
                            Lease {
                                address: offer.address,
                                expires: deadline(c.now, cfg.lease_us.0)?,
                            },
                        );
                        c.timer(
                            cfg.lease_us.0,
                            Action::DhcpSweep {
                                device: host.id.clone(),
                            },
                        )?;
                    }
                    self.dhcp_emit(c, port, &response, true)?;
                }
                "decline"
                    if w.server == Some(server)
                        && self
                            .dhcp
                            .offers
                            .get(&key)
                            .is_some_and(|l| Some(l.address) == w.address) =>
                {
                    self.dhcp.offers.remove(&key);
                    self.dhcp.refused.insert(
                        (host.id.clone(), w.address.unwrap()),
                        deadline(c.now, 30_000_000)?,
                    );
                    c.timer(
                        30_000_000,
                        Action::DhcpSweep {
                            device: host.id.clone(),
                        },
                    )?;
                }
                _ => {}
            }
        } else {
            let Some(cl) = self.dhcp.clients.get(port).cloned() else {
                return Ok(true);
            };
            let nic = host.interfaces.iter().find(|i| &i.id == port).unwrap();
            if !w.mac.eq_ignore_ascii_case(&nic.mac) || w.xid != cl.token {
                return Ok(true);
            }
            match w.kind.as_str() {
                "offer" if cl.phase == Phase::Discover && w.address.is_some() => {
                    self.dhcp_probe(c, port, w)?;
                }
                "ack"
                    if cl.phase == Phase::Request
                        && cl
                            .offer
                            .as_ref()
                            .is_some_and(|o| o.address == w.address && o.server == w.server) =>
                {
                    self.dhcp_assign(c, port, &w, false)?;
                }
                "nack" if cl.phase == Phase::Request => {
                    self.dhcp_retry(c, host, port, cl.attempt + 1)?;
                }
                _ => {}
            }
        }
        Ok(true)
    }
    fn dhcp_probe(&mut self, c: &mut Context<'_>, port: &Id, w: Wire) -> Result<(), Diagnostic> {
        let cl = self.dhcp.clients.get_mut(port).unwrap();
        cl.phase = Phase::Probe;
        cl.offer = Some(w.clone());
        let frame = self.frame(
            w.mac.clone(),
            BROADCAST.into(),
            Payload::Arp {
                operation: 1,
                sender_ip: Ipv4Addr::UNSPECIFIED,
                sender_mac: w.mac.clone(),
                target_ip: w.address.unwrap(),
                target_mac: "00:00:00:00:00:00".into(),
            },
        )?;
        c.fact(
            "dhcp.address.probing",
            &c.project.device_for_interface(port).unwrap().id,
            json!({"interface":port,"address":w.address}),
        );
        self.send(c, port, frame);
        c.timer(
            PROBE,
            Action::DhcpTick {
                interface: port.clone(),
                token: w.xid,
            },
        )
    }
    fn dhcp_conflict(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        port: &Id,
    ) -> Result<(), Diagnostic> {
        let cl = self.dhcp.clients.get(port).unwrap().clone();
        let mut w = cl.offer.unwrap();
        c.fact(
            "dhcp.address.conflict",
            &host.id,
            json!({"interface":port,"address":w.address}),
        );
        if w.server.is_some() {
            w.kind = "decline".into();
            self.dhcp_emit(c, port, &w, false)?;
        }
        self.dhcp_retry(c, host, port, cl.attempt + 1)
    }
    fn dhcp_retry(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        port: &Id,
        attempt: u8,
    ) -> Result<(), Diagnostic> {
        if attempt < 3 {
            return self.dhcp_start(c, port, attempt);
        }
        if attempt >= 6 {
            self.dhcp.clients.get_mut(port).unwrap().phase = Phase::Failed;
            c.fact(
                "dhcp.failed",
                &host.id,
                json!({"interface":port,"reason":"dhcp.link_local_conflict"}),
            );
            return Ok(());
        }
        self.serial += 1;
        let token = self.serial;
        let mac = host
            .interfaces
            .iter()
            .find(|i| &i.id == port)
            .unwrap()
            .mac
            .to_ascii_lowercase();
        let hash = mac
            .bytes()
            .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
        let n = (hash + (attempt - 3) as u32) % 65024 + 256;
        let address = Ipv4Addr::new(169, 254, (n >> 8) as u8, n as u8);
        let w = Wire {
            kind: "link_local".into(),
            mac,
            xid: token,
            server: None,
            address: Some(address),
            prefix: 16,
            gateway: None,
            dns: None,
            lease: 0,
        };
        self.dhcp.clients.insert(
            port.clone(),
            Client {
                token,
                attempt,
                phase: Phase::Probe,
                offer: None,
            },
        );
        self.dhcp_probe(c, port, w)
    }
    pub(super) fn dhcp_tick(
        &mut self,
        c: &mut Context<'_>,
        port: &Id,
        token: u64,
    ) -> Result<(), Diagnostic> {
        let Some(cl) = self
            .dhcp
            .clients
            .get(port)
            .filter(|cl| cl.token == token)
            .cloned()
        else {
            return Ok(());
        };
        let host = c.project.device_for_interface(port).unwrap();
        match cl.phase {
            Phase::Discover | Phase::Request => self.dhcp_retry(c, host, port, cl.attempt + 1)?,
            Phase::Probe => {
                let mut w = cl.offer.unwrap();
                if w.server.is_none() {
                    self.dhcp_assign(c, port, &w, true)?;
                } else {
                    w.kind = "request".into();
                    self.dhcp.clients.get_mut(port).unwrap().phase = Phase::Request;
                    self.dhcp_emit(c, port, &w, false)?;
                    c.timer(
                        RETRY,
                        Action::DhcpTick {
                            interface: port.clone(),
                            token,
                        },
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn dhcp_assign(
        &mut self,
        c: &mut Context<'_>,
        port: &Id,
        w: &Wire,
        link_local: bool,
    ) -> Result<(), Diagnostic> {
        let host = c.project.device_for_interface(port).unwrap();
        if let Some(g) = &host.network.gateway
            && &g.wan == port
            && let Some((lan, prefix)) = host.interfaces.iter().find(|i| i.id == g.lan).and_then(ip)
        {
            let mask = prefix_mask(prefix.min(w.prefix));
            if w.address
                .is_some_and(|a| u32::from(a) & mask == u32::from(lan) & mask)
            {
                self.dhcp.clients.get_mut(port).unwrap().phase = Phase::Failed;
                self.dhcp.assigned.remove(port);
                self.dhcp.effective = None;
                c.fact("dhcp.failed",&host.id,json!({"interface":port,"reason":"nat.overlapping_networks","address":w.address}));
                return Ok(());
            }
        }
        self.dhcp.clients.get_mut(port).unwrap().phase = Phase::Bound;
        let expires = if link_local {
            u64::MAX
        } else {
            deadline(c.now, w.lease)?
        };
        let a = Assignment {
            address: w.address.unwrap(),
            prefix_length: w.prefix,
            gateway: w.gateway,
            dns: w.dns,
            expires_us: Decimal(expires),
            link_local,
        };
        self.dhcp.assigned.insert(port.clone(), a.clone());
        self.dhcp.effective = None;
        c.fact(
            if link_local {
                "dhcp.link_local"
            } else {
                "dhcp.lease.acquired"
            },
            &c.project.device_for_interface(port).unwrap().id,
            json!({"interface":port,"assignment":a}),
        );
        if !link_local {
            c.timer(
                w.lease,
                Action::DhcpExpire {
                    interface: port.clone(),
                    token: w.xid,
                },
            )?;
        }
        Ok(())
    }
    pub(super) fn dhcp_expire(
        &mut self,
        c: &mut Context<'_>,
        port: &Id,
        token: u64,
    ) -> Result<(), Diagnostic> {
        if self
            .dhcp
            .clients
            .get(port)
            .is_some_and(|cl| cl.token == token && cl.phase == Phase::Bound)
        {
            let host = c.project.device_for_interface(port).unwrap();
            let expired = self.dhcp.assigned.get(port).map(|a| a.address);
            let sockets: Vec<_> = self
                .transport
                .views()
                .into_iter()
                .filter(|s| {
                    s.device == host.id
                        && (Some(s.local.address) == expired
                            || s.interface.as_ref() == Some(port)
                            || (s.protocol == TransportProtocol::Udp
                                && s.socket != s.owner
                                && s.local.port >= 49152))
                })
                .map(|s| s.socket)
                .collect();
            for socket in sockets {
                let out = self.transport.command(
                    c.project,
                    &host.id,
                    &SocketCommand::Close {
                        socket,
                        force: true,
                    },
                    &c.flow,
                    &self.routing.views(host, c.now),
                )?;
                self.apply_transport(c, out)?;
            }
            self.arp.retain(|(i, _), _| i != port);
            c.fact("dhcp.lease.expired", &host.id, json!({"interface":port}));
            self.dhcp_start(c, port, 0)?;
        }
        Ok(())
    }
    pub(super) fn dhcp_sweep(&mut self, c: &mut Context<'_>, server: &Id) {
        let before = self.dhcp.offers.len() + self.dhcp.leases.len() + self.dhcp.refused.len();
        self.dhcp.offers.retain(|_, l| l.expires > c.now);
        self.dhcp.leases.retain(|_, l| l.expires > c.now);
        self.dhcp.refused.retain(|_, t| *t > c.now);
        let expired =
            before - self.dhcp.offers.len() - self.dhcp.leases.len() - self.dhcp.refused.len();
        if expired > 0 {
            c.fact("dhcp.entries.expired", server, json!({"count":expired}));
        }
    }
}

fn deadline(now: u64, delay: u64) -> Result<u64, Diagnostic> {
    now.checked_add(delay)
        .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "dhcp"))
}
