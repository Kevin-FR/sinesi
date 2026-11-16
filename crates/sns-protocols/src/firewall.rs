//! Ingress-only pedagogical filtering. UDP reverse matching is not connection tracking.
use super::*;
#[derive(Clone, Debug, Serialize)]
pub struct Decision {
    pub time_us: Decimal,
    pub interface: Id,
    pub frame_id: Decimal,
    pub action: Verdict,
    pub rule: Option<usize>,
    pub reason: String,
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub protocol: String,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
}
#[derive(Clone, Default, serde::Serialize)]
pub struct Firewall {
    pub journal: BTreeMap<Id, Vec<Decision>>,
}
fn address_matches(m: &IpMatch, address: Ipv4Addr, host: &Device) -> bool {
    match m {
        IpMatch::Any => true,
        IpMatch::Subnet { network, prefix } => {
            u32::from(address) & prefix_mask(*prefix) == u32::from(*network)
        }
        IpMatch::LocalNetwork => host
            .interfaces
            .iter()
            .filter(|i| host.network.gateway.as_ref().is_none_or(|g| g.lan == i.id))
            .any(|i| {
                ip(i).is_some_and(|(a, p)| {
                    u32::from(a) & prefix_mask(p) == u32::from(address) & prefix_mask(p)
                })
            }),
    }
}
impl Network {
    pub(super) fn filter_ingress(
        &mut self,
        c: &mut Context<'_>,
        host: &Device,
        interface: &Id,
        frame: &Frame,
    ) -> bool {
        let Some(config) = host.network.firewall.as_ref().filter(|f| f.enabled) else {
            return true;
        };
        let (source, destination, protocol, ports, syn) = match &frame.payload {
            Payload::Arp { .. } => return true,
            Payload::Ipv4 {
                source,
                destination,
                ..
            } => (*source, *destination, "icmp", None, None),
            Payload::Ipv4Transport {
                source,
                destination,
                packet,
                ..
            } => match packet {
                TransportPacket::Tcp(t) => (
                    *source,
                    *destination,
                    "tcp",
                    Some((t.source_port, t.destination_port)),
                    Some(t.flags.syn && !t.flags.ack),
                ),
                TransportPacket::Udp {
                    source_port,
                    destination_port,
                    ..
                } => (
                    *source,
                    *destination,
                    "udp",
                    Some((*source_port, *destination_port)),
                    None,
                ),
            },
        };
        let (action, rule, reason) = if protocol == "icmp" {
            (
                if config.drop_icmp {
                    Verdict::Drop
                } else {
                    Verdict::Accept
                },
                None,
                "icmp_policy",
            )
        } else if protocol == "tcp" && config.syn_only && syn == Some(false) {
            (Verdict::Accept, None, "tcp_non_syn")
        } else if protocol == "udp" && !config.filter_udp {
            (Verdict::Accept, None, "udp_bypass")
        } else {
            let (sp, dp) = ports.unwrap();
            let matched = config.rules.iter().enumerate().find(|(_, r)| {
                let proto = match r.protocol {
                    FilterProtocol::Any => true,
                    FilterProtocol::Tcp => protocol == "tcp",
                    FilterProtocol::Udp => protocol == "udp",
                };
                let request = address_matches(&r.source, source, host)
                    && address_matches(&r.destination, destination, host)
                    && r.destination_port.is_none_or(|p| p == dp);
                let response = protocol == "udp"
                    && r.destination_port == Some(sp)
                    && address_matches(&r.source, destination, host)
                    && address_matches(&r.destination, source, host);
                proto && (request || response)
            });
            matched
                .map(|(index, r)| (r.action, Some(index + 1), "first_match"))
                .unwrap_or((config.default_policy, None, "default_policy"))
        };
        let decision = Decision {
            time_us: Decimal(c.now),
            interface: interface.clone(),
            frame_id: frame.frame_id,
            action,
            rule,
            reason: reason.into(),
            source,
            destination,
            protocol: protocol.into(),
            source_port: ports.map(|p| p.0),
            destination_port: ports.map(|p| p.1),
        };
        let rows = self.firewall.journal.entry(host.id.clone()).or_default();
        if rows.len() == 1024 {
            rows.remove(0);
        }
        rows.push(decision.clone());
        c.fact(
            "firewall.decision",
            &host.id,
            json!({"decision":decision,"frame":frame}),
        );
        action == Verdict::Accept
    }
}
