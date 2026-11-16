use super::*;
impl Network {
    pub(super) fn forward_or_control(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        incoming: &Id,
        frame: &Frame,
    ) -> Result<bool, Diagnostic> {
        let (source, destination, ttl) = match &frame.payload {
            Payload::Arp { .. } => return Ok(false),
            Payload::Ipv4 {
                source,
                destination,
                ttl,
                ..
            }
            | Payload::Ipv4Transport {
                source,
                destination,
                ttl,
                ..
            } => (*source, *destination, *ttl),
        };
        if let Payload::Ipv4Transport {
            packet:
                TransportPacket::Udp {
                    source_port: 520,
                    destination_port: 520,
                    data_base64,
                    length,
                },
            ..
        } = &frame.payload
            && destination == Ipv4Addr::new(224, 0, 0, 9)
        {
            self.rip_receive(
                c,
                host,
                incoming,
                rip::RipDatagram {
                    source,
                    ttl,
                    length: *length,
                    data: data_base64,
                },
            )?;
            return Ok(true);
        }
        if host
            .interfaces
            .iter()
            .any(|i| ip(i).is_some_and(|(a, _)| a == destination))
        {
            return Ok(false);
        }
        let broadcast = destination.is_broadcast()
            || host.interfaces.iter().any(|i| {
                ip(i).is_some_and(|(a, p)| {
                    p < 31 && subnet(a, p).is_ok_and(|(_, b)| b == destination)
                })
            });
        if broadcast {
            return Ok(false);
        }
        let reason = if destination.is_multicast() {
            Some("routing.multicast_not_forwarded")
        } else if !host.network.forwarding.unwrap_or(matches!(
            host.kind,
            DeviceKind::Router | DeviceKind::Gateway
        )) {
            Some("routing.forwarding_disabled")
        } else if ttl <= 1 {
            Some("ipv4.ttl_expired")
        } else {
            None
        };
        if let Some(reason) = reason {
            c.fact("routing.dropped",&host.id,json!({"reason":reason,"source":source,"destination":destination,"interface":incoming,"ttl":ttl,"frame":frame}));
            self.icmp_failure(c, host, source, &frame.payload, reason)?;
            return Ok(true);
        }
        let rows = self.routing.views(host, c.now);
        let Some(route) = routing::select(&rows, destination) else {
            c.fact("routing.dropped",&host.id,json!({"reason":"routing.no_route","source":source,"destination":destination,"interface":incoming,"ttl":ttl,"frame":frame}));
            self.icmp_failure(c, host, source, &frame.payload, "routing.no_route")?;
            return Ok(true);
        };
        if destination.is_loopback()
            || source.is_loopback()
            || source.is_unspecified()
            || (route.prefix < 31
                && subnet(route.network, route.prefix).is_ok_and(|(_, b)| b == destination))
        {
            let reason =
                if destination.is_loopback() || source.is_loopback() || source.is_unspecified() {
                    "routing.invalid_scope"
                } else {
                    "routing.broadcast_not_forwarded"
                };
            c.fact(
                "routing.dropped",
                &host.id,
                json!({"reason":reason,"destination":destination,"route":route}),
            );
            return Ok(true);
        }
        let (port, _, hop) = routing::hop(host, &rows, destination)?;
        let mut payload = frame.payload.clone();
        if !self.nat_outbound(c, host, incoming, &port, &mut payload)? {
            return Ok(true);
        }
        match &mut payload {
            Payload::Ipv4 { ttl, .. } | Payload::Ipv4Transport { ttl, .. } => *ttl -= 1,
            _ => unreachable!(),
        }
        c.fact("ipv4.forwarded",&host.id,json!({"source":source,"destination":destination,"incoming":incoming,"interface":port,"next_hop":hop,"ttl_before":ttl,"ttl":ttl-1,"route":route,"reason":"routing.longest_prefix","original_frame":frame}));
        self.resolve(c, &port, hop, payload)?;
        Ok(true)
    }
    pub(super) fn icmp_failure(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        source: Ipv4Addr,
        original: &Payload,
        reason: &str,
    ) -> Result<(), Diagnostic> {
        let identifier = match original {
            Payload::Ipv4 { icmp, .. } if icmp.message_type == 8 => icmp.identifier.clone(),
            Payload::Ipv4Transport { .. } => c.flow.clone(),
            _ => return Ok(()),
        };
        if source.is_unspecified()
            || source.is_broadcast()
            || source.is_multicast()
            || source.is_loopback()
        {
            return Ok(());
        }
        if let Ok((port, address, hop)) = self.routing.next_hop(host, source, c.now) {
            let kind = if reason == "ipv4.ttl_expired" { 11 } else { 3 };
            let code = if reason == "routing.forwarding_disabled" {
                13
            } else if reason == "arp.timeout" {
                1
            } else {
                0
            };
            let payload = Payload::Ipv4 {
                source: address,
                destination: source,
                ttl: 64,
                icmp: Icmp {
                    quoted: (c.project.schema_version >= 6).then(|| Box::new(original.clone())),
                    message_type: kind,
                    code,
                    identifier,
                    sequence: 1,
                },
            };
            c.fact(
                "icmp.error.sent",
                &host.id,
                json!({"icmp_type":kind,"icmp_code":code,"reason":reason,"destination":source}),
            );
            self.resolve(c, &port, hop, payload)?;
        } else {
            c.fact(
                "routing.icmp.unreturnable",
                &host.id,
                json!({"reason":"routing.no_return_route","destination":source}),
            );
        }
        Ok(())
    }
    pub(super) fn arp_send(
        &mut self,
        c: &mut Context<'_>,
        device: &Id,
        source: Ipv4Addr,
        target: Ipv4Addr,
    ) -> Result<(), Diagnostic> {
        let host = c
            .project
            .device(device)
            .ok_or_else(|| Diagnostic::new("project.invalid_reference", device.as_str()))?;
        let (port, _, _) = arp_injection_route(host, source, target)?;
        let nic = host.interfaces.iter().find(|i| i.id == port).unwrap();
        let dest = self
            .arp
            .get(&(port.clone(), target))
            .filter(|e| e.expires_us.0 > c.now)
            .map(|e| e.mac.clone())
            .unwrap_or_else(|| BROADCAST.into());
        let payload = Payload::Arp {
            operation: 2,
            sender_ip: source,
            sender_mac: nic.mac.clone(),
            target_ip: target,
            target_mac: dest.clone(),
        };
        let frame = self.frame(nic.mac.clone(), dest, payload)?;
        c.fact(
            "routing.arp.injected",
            device,
            json!({"source":source,"target":target,"interface":port,"simulated":true}),
        );
        self.send(c, &port, frame);
        Ok(())
    }
}

pub fn arp_injection_route(
    host: &Device,
    source: Ipv4Addr,
    target: Ipv4Addr,
) -> Result<(Id, Ipv4Addr, Ipv4Addr), Diagnostic> {
    let (port, address, hop) = next_hop(host, target)?;
    let nic = host.interfaces.iter().find(|i| i.id == port).unwrap();
    let valid = ip(nic).is_some_and(|(a, p)| {
        subnet(a, p).is_ok_and(|(n, b)| {
            subnet(target, p).is_ok_and(|(t, _)| t == n)
                && (p >= 31 || (target != n && target != b))
        })
    });
    if !valid
        || hop != target
        || source.is_unspecified()
        || source.is_broadcast()
        || source.is_multicast()
        || source.is_loopback()
    {
        return Err(Diagnostic::new(
            "routing.invalid_arp_injection",
            host.id.as_str(),
        ));
    }
    Ok((port, address, hop))
}
