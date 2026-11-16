//! Configuration advice for learners: plain checks on saved or draft data.
//! Advice never edits a project and never replaces the engine's behaviour.
use sns_model::*;
use std::net::Ipv4Addr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GatewayAdvice {
    /// Not a host, or nothing to say.
    NotApplicable,
    /// No gateway: only the local network is reachable.
    Missing,
    /// The text is not an IPv4 address.
    Invalid,
    /// The gateway lies outside every network of the device.
    OutsideNetwork {
        network: String,
    },
    /// The gateway is one of the device's own addresses.
    OwnAddress,
    Reachable,
}

fn network(address: Ipv4Addr, prefix: u8) -> Option<(u32, u32)> {
    let mask = match prefix {
        0 => 0,
        1..=32 => u32::MAX << (32 - u32::from(prefix)),
        _ => return None,
    };
    Some((u32::from(address) & mask, mask))
}

/// Advice about a host's default gateway relative to its static IPv4 interfaces.
pub fn gateway_advice(device: &Device) -> GatewayAdvice {
    if !matches!(device.kind, DeviceKind::Workstation | DeviceKind::Notebook) {
        return GatewayAdvice::NotApplicable;
    }
    let statics: Vec<(Ipv4Addr, u8)> = device
        .interfaces
        .iter()
        .filter_map(|i| match &i.ipv4 {
            Some(Ipv4Config::Static {
                address,
                prefix_length,
            }) => address.parse().ok().map(|a| (a, *prefix_length)),
            _ => None,
        })
        .collect();
    let gateway = match device.network.default_gateway.as_deref().map(str::trim) {
        None | Some("") => {
            return if statics.is_empty() {
                GatewayAdvice::NotApplicable
            } else {
                GatewayAdvice::Missing
            };
        }
        Some(text) => match text.parse::<Ipv4Addr>() {
            Ok(gateway) => gateway,
            Err(_) => return GatewayAdvice::Invalid,
        },
    };
    // A DHCP-only host learns its network later: nothing reliable to compare yet.
    let Some(&(first, first_prefix)) = statics.first() else {
        return GatewayAdvice::NotApplicable;
    };
    if statics.iter().any(|(a, _)| *a == gateway) {
        return GatewayAdvice::OwnAddress;
    }
    if statics
        .iter()
        .any(|(a, p)| network(*a, *p).is_some_and(|(net, mask)| u32::from(gateway) & mask == net))
    {
        return GatewayAdvice::Reachable;
    }
    let net = network(first, first_prefix).map_or(0, |(net, _)| net);
    GatewayAdvice::OutsideNetwork {
        network: format!("{}/{first_prefix}", Ipv4Addr::from(net)),
    }
}
