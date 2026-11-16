//! Deterministic forwarding table and bounded distance-vector state.
use super::*;
use serde::{Deserialize, Serialize};
pub const RIP_PERIOD_US: u64 = 30_000_000;
pub const RIP_TIMEOUT_US: u64 = 75_000_000;
pub const RIP_GARBAGE_US: u64 = 30_000_000;
pub const RIP_INFINITY: u8 = 16;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Loopback,
    Direct,
    Static,
    Default,
    Rip,
}
#[derive(Clone, Debug, Serialize)]
pub struct RouteView {
    pub network: Ipv4Addr,
    pub prefix: u8,
    pub gateway: Option<Ipv4Addr>,
    pub interface: Option<Id>,
    pub origin: Origin,
    pub metric: u8,
    pub expires_us: Option<Decimal>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Advertisement {
    pub network: Ipv4Addr,
    pub prefix: u8,
    pub metric: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RipMessage {
    pub version: u8,
    pub entries: Vec<Advertisement>,
}
#[derive(Clone, serde::Serialize)]
pub struct Learned {
    pub device: Id,
    pub interface: Id,
    pub neighbor: Ipv4Addr,
    pub entry: Advertisement,
    pub expires: u64,
    pub garbage: Option<u64>,
}
#[derive(Clone, Default, serde::Serialize)]
pub struct Routing {
    pub learned: BTreeMap<(Id, Ipv4Addr, u8, Id, Ipv4Addr), Learned>,
    pub started: BTreeSet<Id>,
    pub muted: BTreeSet<Id>,
    pub triggered: BTreeSet<Id>,
}
fn matches(address: Ipv4Addr, network: Ipv4Addr, prefix: u8) -> bool {
    subnet(address, prefix).is_ok_and(|(n, _)| n == network)
}
pub fn configured(host: &Device) -> Vec<RouteView> {
    let mut rows = vec![RouteView {
        network: Ipv4Addr::new(127, 0, 0, 0),
        prefix: 8,
        gateway: None,
        interface: None,
        origin: Origin::Loopback,
        metric: 0,
        expires_us: None,
    }];
    for nic in host.interfaces.iter() {
        if let Some((address, prefix)) = ip(nic) {
            rows.push(RouteView {
                network: subnet(address, prefix).unwrap().0,
                prefix,
                gateway: None,
                interface: Some(nic.id.clone()),
                origin: Origin::Direct,
                metric: 0,
                expires_us: None,
            });
        }
    }
    rows.extend(host.network.routes.iter().map(|r| RouteView {
        network: r.network,
        prefix: r.prefix,
        gateway: r.gateway,
        interface: Some(r.interface.clone()),
        origin: Origin::Static,
        metric: 0,
        expires_us: None,
    }));
    if let Some(gateway) = host
        .network
        .default_gateway
        .as_ref()
        .and_then(|s| s.parse::<Ipv4Addr>().ok())
    {
        let mut ports: Vec<_> = host
            .interfaces
            .iter()
            .filter_map(|i| ip(i).map(|(a, p)| (i, a, p)))
            .filter(|(_, a, p)| subnet(*a, *p).is_ok_and(|(n, _)| matches(gateway, n, *p)))
            .collect();
        ports.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.id.cmp(&b.0.id)));
        if let Some((nic, _, _)) = ports.first() {
            rows.push(RouteView {
                network: Ipv4Addr::UNSPECIFIED,
                prefix: 0,
                gateway: Some(gateway),
                interface: Some(nic.id.clone()),
                origin: Origin::Default,
                metric: 0,
                expires_us: None,
            });
        }
    }
    rows
}
pub fn select(rows: &[RouteView], target: Ipv4Addr) -> Option<&RouteView> {
    rows.iter()
        .filter(|r| r.metric < RIP_INFINITY && matches(target, r.network, r.prefix))
        .min_by(|a, b| {
            b.prefix
                .cmp(&a.prefix)
                .then(a.origin.cmp(&b.origin))
                .then(a.metric.cmp(&b.metric))
                .then(a.gateway.cmp(&b.gateway))
                .then(a.interface.cmp(&b.interface))
        })
}
pub fn hop(
    host: &Device,
    rows: &[RouteView],
    target: Ipv4Addr,
) -> Result<(Id, Ipv4Addr, Ipv4Addr), Diagnostic> {
    let route = select(rows, target)
        .ok_or_else(|| Diagnostic::new("routing.no_route", host.id.as_str()))?;
    let port = route
        .interface
        .as_ref()
        .ok_or_else(|| Diagnostic::new("routing.no_route", host.id.as_str()))?;
    let address = host
        .interfaces
        .iter()
        .find(|i| &i.id == port)
        .and_then(ip)
        .ok_or_else(|| Diagnostic::new("routing.no_route", host.id.as_str()))?
        .0;
    Ok((port.clone(), address, route.gateway.unwrap_or(target)))
}
impl Routing {
    pub fn views(&self, host: &Device, now: u64) -> Vec<RouteView> {
        let mut rows = configured(host);
        if host.network.rip {
            rows.extend(
                self.learned
                    .values()
                    .filter(|r| r.device == host.id)
                    .map(|r| RouteView {
                        network: r.entry.network,
                        prefix: r.entry.prefix,
                        gateway: Some(r.neighbor),
                        interface: Some(r.interface.clone()),
                        origin: Origin::Rip,
                        metric: if r.expires <= now { 16 } else { r.entry.metric },
                        expires_us: Some(Decimal(r.expires)),
                    }),
            );
        }
        rows.sort_by(|a, b| {
            a.network
                .cmp(&b.network)
                .then(a.prefix.cmp(&b.prefix))
                .then(a.origin.cmp(&b.origin))
                .then(a.metric.cmp(&b.metric))
                .then(a.gateway.cmp(&b.gateway))
                .then(a.interface.cmp(&b.interface))
        });
        rows
    }
    pub fn next_hop(
        &self,
        host: &Device,
        target: Ipv4Addr,
        now: u64,
    ) -> Result<(Id, Ipv4Addr, Ipv4Addr), Diagnostic> {
        hop(host, &self.views(host, now), target)
    }
}
