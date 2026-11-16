//! Portable private-network, translation and pedagogical filtering configuration.
use crate::*;
use std::net::Ipv4Addr;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Accept,
    Drop,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterProtocol {
    Any,
    Tcp,
    Udp,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "match", rename_all = "snake_case", deny_unknown_fields)]
pub enum IpMatch {
    #[default]
    Any,
    LocalNetwork,
    Subnet {
        network: Ipv4Addr,
        prefix: u8,
    },
}
impl IpMatch {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if let Self::Subnet { network, prefix } = self
            && (*prefix > 32 || u32::from(*network) & !prefix_mask(*prefix) != 0)
        {
            return Err(Diagnostic::new("firewall.invalid_rule", "network"));
        }
        Ok(())
    }
}
pub fn prefix_mask(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX
            .checked_shl(u32::from(32u8.saturating_sub(prefix)))
            .unwrap_or(0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirewallRule {
    pub source: IpMatch,
    pub destination: IpMatch,
    pub protocol: FilterProtocol,
    pub destination_port: Option<u16>,
    pub action: Verdict,
}
impl Default for FirewallRule {
    fn default() -> Self {
        Self {
            source: IpMatch::Any,
            destination: IpMatch::Any,
            protocol: FilterProtocol::Tcp,
            destination_port: None,
            action: Verdict::Drop,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirewallConfig {
    pub enabled: bool,
    pub default_policy: Verdict,
    pub syn_only: bool,
    pub filter_udp: bool,
    pub drop_icmp: bool,
    pub rules: Vec<FirewallRule>,
}
impl Default for FirewallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_policy: Verdict::Drop,
            syn_only: true,
            filter_udp: true,
            drop_icmp: false,
            rules: vec![],
        }
    }
}
impl FirewallConfig {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if self.rules.len() > 128 {
            return Err(Diagnostic::new("resource.limit_exceeded", "firewall_rules"));
        }
        for r in &self.rules {
            r.source.validate()?;
            r.destination.validate()?;
            if r.destination_port == Some(0) {
                return Err(Diagnostic::new("firewall.invalid_rule", "port"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortForward {
    pub protocol: TransportProtocol,
    pub external_port: u16,
    pub internal_address: Ipv4Addr,
    pub internal_port: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayConfig {
    pub lan: Id,
    pub wan: Id,
    pub nat_enabled: bool,
    pub port_start: u16,
    pub port_end: u16,
    pub idle_timeout_us: Decimal,
    pub forwards: Vec<PortForward>,
}
impl GatewayConfig {
    pub fn validate(&self, host: &Device) -> Result<(), Diagnostic> {
        let bad = || Diagnostic::new("nat.invalid_config", host.id.as_str());
        if host.kind != DeviceKind::Gateway
            || host.interfaces.len() != 2
            || self.lan == self.wan
            || self.port_start == 0
            || self.port_end < self.port_start
            || self.idle_timeout_us.0 < 1_000_000
            || self.idle_timeout_us.0 > 604_800_000_000
            || self.forwards.len() > 128
        {
            return Err(bad());
        }
        let lan = host
            .interfaces
            .iter()
            .find(|i| i.id == self.lan && i.medium == Medium::Ethernet)
            .ok_or_else(bad)?;
        let wan = host
            .interfaces
            .iter()
            .find(|i| i.id == self.wan && i.medium == Medium::Ethernet)
            .ok_or_else(bad)?;
        let Some(Ipv4Config::Static {
            address,
            prefix_length,
        }) = &lan.ipv4
        else {
            return Err(bad());
        };
        let local: Ipv4Addr = address.parse().map_err(|_| bad())?;
        let mask = prefix_mask(*prefix_length);
        if *prefix_length == 0 || *prefix_length > 30 {
            return Err(bad());
        }
        if let Some(Ipv4Config::Static {
            address,
            prefix_length,
        }) = &wan.ipv4
        {
            let a: Ipv4Addr = address.parse().map_err(|_| bad())?;
            let common = mask & prefix_mask(*prefix_length);
            if u32::from(a) & common == u32::from(local) & common {
                return Err(bad());
            }
        }
        if let Some(c) = &host.network.dhcp_server
            && c.interface != self.lan
        {
            return Err(bad());
        }
        let mut ports = BTreeSet::new();
        for f in &self.forwards {
            let addr = u32::from(f.internal_address);
            if f.external_port == 0
                || f.internal_port == 0
                || !ports.insert((f.protocol, f.external_port))
                || addr & mask != u32::from(local) & mask
                || f.internal_address == local
                || addr & !mask == 0
                || addr & !mask == !mask
            {
                return Err(bad());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum SecurityCommand {
    ConfigureFirewall { config: FirewallConfig },
    ConfigureGateway { config: GatewayConfig },
    PurgeNat,
}
pub fn private_profile(project: &mut Project) {
    if project.schema_version < 6 {
        project.schema_version = 6;
        project.semantic_profile = "sns-educational-private-v1".into();
    }
}
