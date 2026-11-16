use super::*;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use routing::*;
pub(super) struct RipDatagram<'a> {
    pub source: Ipv4Addr,
    pub ttl: u8,
    pub length: u16,
    pub data: &'a str,
}
impl Network {
    pub(super) fn rip_tick(
        &mut self,
        c: &mut Context<'_>,
        device: &Id,
        periodic: bool,
    ) -> Result<(), Diagnostic> {
        let host = c
            .project
            .device(device)
            .ok_or_else(|| Diagnostic::new("project.invalid_reference", device.as_str()))?;
        if !host.network.rip {
            return Ok(());
        }
        if self.routing.started.insert(device.clone()) {
            c.timer(
                1_000_000,
                Action::RipSweep {
                    device: device.clone(),
                },
            )?;
        }
        if periodic {
            c.timer(
                RIP_PERIOD_US,
                Action::RipTick {
                    device: device.clone(),
                    periodic: true,
                },
            )?;
        } else {
            self.routing.triggered.remove(device);
        }
        if self.routing.muted.contains(device) {
            return Ok(());
        }
        let rows = self.routing.views(host, c.now);
        let mut ports: Vec<_> = host
            .interfaces
            .iter()
            .filter(|i| i.medium == Medium::Ethernet && ip(i).is_some())
            .collect();
        ports.sort_by_key(|i| &i.id);
        for port in ports {
            let mut entries: BTreeMap<(Ipv4Addr, u8), Advertisement> = BTreeMap::new();
            for route in rows
                .iter()
                .filter(|r| matches!(r.origin, Origin::Direct | Origin::Rip))
            {
                let metric =
                    if route.origin == Origin::Rip && route.interface.as_ref() == Some(&port.id) {
                        16
                    } else {
                        route.metric
                    };
                let key = (route.network, route.prefix);
                let entry = Advertisement {
                    network: route.network,
                    prefix: route.prefix,
                    metric,
                };
                if entries
                    .get(&key)
                    .is_none_or(|old| entry.metric < old.metric)
                {
                    entries.insert(key, entry);
                }
            }
            let entries: Vec<_> = entries.into_values().collect();
            for chunk in entries.chunks(25) {
                let bytes = serde_json::to_vec(&RipMessage {
                    version: 2,
                    entries: chunk.to_vec(),
                })
                .unwrap();
                let source = ip(port).unwrap().0;
                let destination = Ipv4Addr::new(224, 0, 0, 9);
                let packet = TransportPacket::Udp {
                    source_port: 520,
                    destination_port: 520,
                    length: (bytes.len() + 8) as u16,
                    data_base64: STANDARD.encode(bytes),
                };
                let payload = Payload::Ipv4Transport {
                    source,
                    destination,
                    ttl: 1,
                    packet,
                };
                c.fact("rip.advertisement.sent",device,json!({"interface":port.id,"source":source,"destination":destination,"entries":chunk,"poison_reverse":true}));
                let frame = self.frame(port.mac.clone(), BROADCAST.into(), payload)?;
                self.send(c, &port.id, frame);
            }
        }
        Ok(())
    }
    fn rip_trigger(&mut self, c: &mut Context<'_>, device: &Id) -> Result<(), Diagnostic> {
        if self.routing.triggered.insert(device.clone()) {
            c.timer(
                1_000,
                Action::RipTick {
                    device: device.clone(),
                    periodic: false,
                },
            )?;
        }
        Ok(())
    }
    pub(super) fn rip_receive(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        port: &Id,
        datagram: RipDatagram<'_>,
    ) -> Result<(), Diagnostic> {
        let RipDatagram {
            source,
            ttl,
            length: wire_length,
            data,
        } = datagram;
        if !host.network.rip {
            return Ok(());
        }
        let nic = host.interfaces.iter().find(|i| &i.id == port).unwrap();
        let valid = ip(nic).is_some_and(|(a, p)| {
            source != a
                && !source.is_unspecified()
                && !source.is_multicast()
                && subnet(a, p).is_ok_and(|(n, b)| {
                    subnet(source, p).is_ok_and(|(s, _)| s == n)
                        && (p >= 31 || (source != n && source != b))
                })
        });
        let decoded = if data.len() <= 16384 {
            STANDARD
                .decode(data)
                .ok()
                .filter(|b| b.len() + 8 == usize::from(wire_length))
                .and_then(|b| serde_json::from_slice::<RipMessage>(&b).ok())
        } else {
            None
        };
        let Some(message) = decoded.filter(|m| {
            valid
                && ttl == 1
                && m.version == 2
                && m.entries.len() <= 25
                && m.entries.iter().all(|e| {
                    e.prefix <= 32
                        && e.metric <= 16
                        && !e.network.is_loopback()
                        && !e.network.is_multicast()
                        && subnet(e.network, e.prefix).is_ok_and(|(n, _)| n == e.network)
                })
        }) else {
            c.fact(
                "rip.advertisement.rejected",
                &host.id,
                json!({"reason":"routing.invalid_rip","source":source,"interface":port}),
            );
            return Ok(());
        };
        c.fact(
            "rip.advertisement.received",
            &host.id,
            json!({"source":source,"interface":port,"entries":message.entries}),
        );
        let local = configured(host);
        let mut changed = false;
        for entry in message.entries {
            if local.iter().any(|r| {
                r.origin == Origin::Direct && r.network == entry.network && r.prefix == entry.prefix
            }) {
                continue;
            }
            let metric = entry.metric.saturating_add(1).min(16);
            let key = (
                host.id.clone(),
                entry.network,
                entry.prefix,
                port.clone(),
                source,
            );
            let old = self.routing.learned.get(&key);
            if old.is_none() && metric == 16 {
                continue;
            }
            if old.is_none() && self.routing.learned.len() >= 4096 {
                return Err(Diagnostic::new("resource.limit_exceeded", "rip_routes"));
            }
            let differs = old.is_none_or(|r| r.entry.metric != metric);
            changed |= differs;
            let expires = c
                .now
                .checked_add(RIP_TIMEOUT_US)
                .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "rip"))?;
            let garbage = if metric == 16 {
                old.and_then(|r| r.garbage).or(Some(
                    c.now
                        .checked_add(RIP_GARBAGE_US)
                        .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "rip"))?,
                ))
            } else {
                None
            };
            let updated = Learned {
                device: host.id.clone(),
                interface: port.clone(),
                neighbor: source,
                entry: Advertisement { metric, ..entry },
                expires,
                garbage,
            };
            if differs {
                c.fact("rip.route.changed",&host.id,json!({"source":source,"interface":port,"route":updated.entry,"reason":if metric==16 {"routing.poisoned"}else{"routing.learned"},"expires_us":expires.to_string()}));
            }
            self.routing.learned.insert(key, updated);
        }
        if changed {
            self.rip_trigger(c, &host.id)?;
        }
        Ok(())
    }
    pub(super) fn rip_sweep(&mut self, c: &mut Context<'_>, device: &Id) -> Result<(), Diagnostic> {
        if !c.project.device(device).is_some_and(|h| h.network.rip) {
            return Ok(());
        }
        let garbage = c
            .now
            .checked_add(RIP_GARBAGE_US)
            .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "rip"))?;
        let mut changed = false;
        for route in self
            .routing
            .learned
            .values_mut()
            .filter(|r| &r.device == device)
        {
            if route.expires <= c.now && route.entry.metric < 16 {
                route.entry.metric = 16;
                route.garbage = Some(garbage);
                changed = true;
                c.fact("rip.route.expired",device,json!({"route":route.entry,"neighbor":route.neighbor,"interface":route.interface,"reason":"routing.expired"}));
            }
        }
        self.routing
            .learned
            .retain(|_, r| r.garbage.is_none_or(|g| g > c.now));
        if changed {
            self.rip_trigger(c, device)?;
        }
        c.timer(
            1_000_000,
            Action::RipSweep {
                device: device.clone(),
            },
        )?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "rip_tests.rs"]
mod tests;
