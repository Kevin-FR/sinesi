//! Portable configuration for the educational application protocols.
use crate::*;
use std::net::Ipv4Addr;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DhcpConfig {
    pub interface: Id,
    pub pool_start: Ipv4Addr,
    pub pool_end: Ipv4Addr,
    pub prefix_length: u8,
    #[serde(default)]
    pub gateway: Option<Ipv4Addr>,
    #[serde(default)]
    pub dns: Option<Ipv4Addr>,
    #[serde(default)]
    pub use_host_options: bool,
    #[serde(default)]
    pub reservations: BTreeMap<String, Ipv4Addr>,
    pub lease_us: Decimal,
}
impl DhcpConfig {
    pub fn validate(&self, host: &Device) -> Result<(), Diagnostic> {
        let Some(nic) = host.interfaces.iter().find(|i| i.id == self.interface) else {
            return Err(Diagnostic::new("dhcp.invalid_config", "interface"));
        };
        let Some(Ipv4Config::Static {
            address,
            prefix_length,
        }) = &nic.ipv4
        else {
            return Err(Diagnostic::new("dhcp.invalid_config", "server_address"));
        };
        let server: Ipv4Addr = address
            .parse()
            .map_err(|_| Diagnostic::new("dhcp.invalid_config", "server_address"))?;
        let prefix = self.prefix_length;
        if prefix == 0
            || prefix > 30
            || *prefix_length != prefix
            || self.lease_us.0 < 1_000_000
            || self.lease_us.0 > 604_800_000_000
            || self.reservations.len() > 256
        {
            return Err(Diagnostic::new("dhcp.invalid_config", "limits"));
        }
        let mask = u32::MAX << (32 - prefix);
        let net = u32::from(server) & mask;
        let valid = |a: Ipv4Addr| {
            u32::from(a) & mask == net
                && u32::from(a) != net
                && u32::from(a) != (net | !mask)
                && a != server
        };
        if !valid(self.pool_start)
            || !valid(self.pool_end)
            || self.pool_start > self.pool_end
            || u32::from(self.pool_end) - u32::from(self.pool_start) > 4095
        {
            return Err(Diagnostic::new("dhcp.invalid_config", "pool"));
        }
        let mut seen = BTreeSet::new();
        let mut macs = BTreeSet::new();
        for (mac, address) in &self.reservations {
            if !macs.insert(mac.to_ascii_lowercase())
                || mac.len() != 17
                || !mac
                    .split(':')
                    .all(|v| v.len() == 2 && u8::from_str_radix(v, 16).is_ok())
                || !valid(*address)
                || !seen.insert(address)
            {
                return Err(Diagnostic::new("dhcp.invalid_config", "reservation"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DnsType {
    A,
    Mx,
    Ns,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: DnsType,
    pub value: String,
    pub ttl_seconds: u32,
    #[serde(default)]
    pub priority: u16,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsConfig {
    pub records: Vec<DnsRecord>,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub upstream: Option<Ipv4Addr>,
}
/// ASCII teaching names, case insensitive; the root is supported for referrals.
pub fn dns_name(name: &str) -> Result<String, Diagnostic> {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    if name.len() > 253
        || (!name.is_empty()
            && name.split('.').any(|label| {
                label.is_empty()
                    || label.len() > 63
                    || label.starts_with('-')
                    || label.ends_with('-')
                    || !label
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            }))
    {
        return Err(Diagnostic::new("dns.invalid_name", name));
    }
    Ok(name)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub administration: bool,
    pub port: u16,
    pub root: String,
    #[serde(default)]
    pub virtual_hosts: BTreeMap<String, String>,
    /// Declarative dynamic route: HTML-escaped form values, never executable code.
    #[serde(default)]
    pub form_path: Option<String>,
}
impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            administration: false,
            port: 80,
            root: "/www".into(),
            virtual_hosts: BTreeMap::new(),
            form_path: Some("/submit".into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "service",
    content = "settings",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ServiceConfig {
    P2p(crate::P2pConfig),
    MailServer(crate::MailServerConfig),
    MailClient(crate::MailClientConfig),
    Dns(DnsConfig),
    Http(HttpConfig),
}
impl ServiceConfig {
    pub fn validate(&self, kind: &str) -> Result<(), Diagnostic> {
        let bad = || Diagnostic::new("service.invalid_config", kind);
        match self {
            Self::P2p(c) => {
                if kind != "p2p" {
                    return Err(bad());
                }
                c.validate()?;
            }
            Self::MailServer(c) => {
                if kind != "mail_server" {
                    return Err(bad());
                }
                c.validate()?;
            }
            Self::MailClient(c) => {
                if kind != "mail_client" {
                    return Err(bad());
                }
                c.validate()?;
            }
            Self::Dns(c) => {
                if kind != "dns_server" || c.records.len() > 256 {
                    return Err(bad());
                }
                let mut unique = BTreeSet::new();
                for r in &c.records {
                    let name = dns_name(&r.name)?;
                    if !unique.insert((name, r.record_type, r.value.to_ascii_lowercase()))
                        || r.ttl_seconds > 604800
                    {
                        return Err(bad());
                    }
                    if r.record_type == DnsType::A {
                        r.value.parse::<Ipv4Addr>().map_err(|_| bad())?;
                    } else {
                        dns_name(&r.value)?;
                    }
                }
            }
            Self::Http(c) => {
                if kind != "web_server"
                    || c.port == 0
                    || !virtual_path_valid(&c.root)
                    || c.virtual_hosts.len() > 64
                    || c.form_path.as_ref().is_some_and(|p| !virtual_path_valid(p))
                {
                    return Err(bad());
                }
                for (name, root) in &c.virtual_hosts {
                    dns_name(name)?;
                    if !virtual_path_valid(root) {
                        return Err(bad());
                    }
                }
            }
        }
        Ok(())
    }
}
