//! Portable data contracts. No UI, operating-system I/O or simulation clock lives here.
mod timing;
pub use timing::WaitFactor;
mod annotations;
pub use annotations::*;
mod presentation;
pub use presentation::*;
mod link_conditions;
pub use link_conditions::*;
mod modem;
pub use modem::*;
mod wireless;
pub use wireless::*;
mod p2p;
pub use p2p::*;
mod mail;
pub use mail::*;
mod security;
pub use security::*;
mod services;
pub use services::*;
mod routing;
pub use routing::*;
pub mod apps;
pub use apps::*;
pub mod transport;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};
pub use transport::*;

pub const PROFILE: &str = "sns-educational-v1";
pub const MAX_DEVICES: usize = 1_024;
pub const MAX_PROJECT_DOCUMENT_BYTES: usize = 8 * 1_024 * 1_024;
pub const MAX_LINKS: usize = 4_096;
pub const MAX_ACTIONS: usize = 32_768;
pub const MAX_BODY_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{code}: {message_key}")]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub message_key: String,
    pub target: Option<String>,
    pub details: BTreeMap<String, String>,
}

impl Diagnostic {
    pub fn new(code: &str, target: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message_key: code.into(),
            target: Some(target.into()),
            details: BTreeMap::new(),
        }
    }
}

/// Exact, canonical decimal encoding at JSON boundaries (including values above 2^53).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(pub u64);

impl Serialize for Decimal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        if value.is_empty()
            || (value.len() > 1 && value.starts_with('0'))
            || !value.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(serde::de::Error::custom(
                "expected a canonical unsigned decimal string",
            ));
        }
        value.parse().map(Self).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Id(String);
impl Id {
    pub fn new(value: impl Into<String>) -> Result<Self, Diagnostic> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        {
            return Err(Diagnostic::new("validation.invalid_id", value));
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for Id {
    type Error = Diagnostic;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub schema_version: u32,
    pub semantic_profile: String,
    pub project_id: Id,
    pub name: String,
    pub devices: Vec<Device>,
    pub links: Vec<Link>,
    pub layout: Layout,
    pub annotations: Vec<Annotation>,
    pub resources: Vec<Resource>,
    pub extensions: Vec<Extension>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lan: BTreeMap<Id, LanConfig>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub id: Id,
    pub kind: DeviceKind,
    pub label: String,
    pub interfaces: Vec<Interface>,
    pub network: Network,
    pub applications: Vec<Application>,
    pub files: Vec<VirtualFile>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Workstation,
    Notebook,
    Switch,
    Router,
    Gateway,
    Modem,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    pub id: Id,
    pub mac: String,
    pub medium: Medium,
    pub ipv4: Option<Ipv4Config>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Medium {
    Ethernet,
    Wireless,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Ipv4Config {
    Static { address: String, prefix_length: u8 },
    Dhcp,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modem: Option<ModemConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub wireless: BTreeMap<Id, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<Box<GatewayConfig>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firewall: Option<Box<FirewallConfig>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dhcp_server: Option<Box<DhcpConfig>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forwarding: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<StaticRoute>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rip: bool,
    pub default_gateway: Option<String>,
    pub dns_server: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Application {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<ApplicationData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<ServiceConfig>,
    pub id: Id,
    pub kind: String,
    pub config: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VirtualFile {
    pub path: String,
    pub media_type: String,
    pub data_base64: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub id: Id,
    pub endpoints: [Id; 2],
    pub latency_us: Decimal,
    pub jitter_us: Decimal,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub devices: Vec<Position>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub device_id: Id,
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    pub id: Id,
    pub text: String,
    pub x: f32,
    pub y: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<AnnotationStyle>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: Id,
    pub media_type: String,
    pub data_base64: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extension {
    pub id: Id,
    pub api_version: u32,
    pub resource_id: Id,
    pub enabled: bool,
}

pub fn virtual_path_valid(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains(['\\', ':', '\0', '%', '#'])
        && !path.split('/').any(|part| part == ".." || part == ".")
        && path.len() <= 2_048
}

fn validate_base64(data: &str) -> Result<(), Diagnostic> {
    if data.len() > MAX_BODY_BYTES * 2 {
        return Err(Diagnostic::new("resource.limit_exceeded", "binary content"));
    }
    let bytes = STANDARD
        .decode(data)
        .map_err(|_| Diagnostic::new("validation.invalid_base64", "binary content"))?;
    if bytes.len() > MAX_BODY_BYTES {
        return Err(Diagnostic::new("resource.limit_exceeded", "binary content"));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceSymbol {
    AccessPoint,
    Modem,
    Desktop,
    Laptop,
    Server,
    Printer,
    Tablet,
    Router,
    Switch,
    Cloud,
}
impl DeviceSymbol {
    pub const ALL: [Self; 10] = [
        Self::AccessPoint,
        Self::Modem,
        Self::Desktop,
        Self::Laptop,
        Self::Server,
        Self::Printer,
        Self::Tablet,
        Self::Router,
        Self::Switch,
        Self::Cloud,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::AccessPoint => "access_point_device",
            Self::Modem => "modem",
            Self::Desktop => "desktop_device",
            Self::Laptop => "notebook",
            Self::Server => "server_device",
            Self::Printer => "printer_device",
            Self::Tablet => "tablet_device",
            Self::Router => "router",
            Self::Switch => "switch",
            Self::Cloud => "cloud_device",
        }
    }
    pub fn automatic(kind: DeviceKind) -> Self {
        match kind {
            DeviceKind::Workstation => Self::Desktop,
            DeviceKind::Notebook => Self::Laptop,
            DeviceKind::Switch => Self::Switch,
            DeviceKind::Modem => Self::Modem,
            _ => Self::Router,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LanConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<DeviceSymbol>,
    pub display: DisplayName,
    pub sat_retention_us: Decimal,
}
impl Default for LanConfig {
    fn default() -> Self {
        Self {
            symbol: None,
            display: DisplayName::Name,
            sat_retention_us: Decimal(300_000_000),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayName {
    Name,
    Ipv4,
    Mac,
}

impl Project {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if !((self.schema_version == 1 && self.semantic_profile == PROFILE && self.lan.is_empty())
            || (self.schema_version == 2 && self.semantic_profile == "sns-educational-lan-v1")
            || (self.schema_version == 3 && self.semantic_profile == "sns-educational-hosts-v1")
            || (self.schema_version == 4 && self.semantic_profile == "sns-educational-routing-v1")
            || (self.schema_version == 5 && self.semantic_profile == "sns-educational-services-v1")
            || (self.schema_version == 6 && self.semantic_profile == "sns-educational-private-v1")
            || (self.schema_version == 7
                && self.semantic_profile == "sns-educational-applications-v1")
            || (self.schema_version == 8 && self.semantic_profile == PRESENTATION_PROFILE)
            || (self.schema_version == DOCUMENTATION_SCHEMA_VERSION
                && self.semantic_profile == DOCUMENTATION_PROFILE))
        {
            return Err(Diagnostic::new(
                "project.unsupported_version",
                self.project_id.as_str(),
            ));
        }
        if self.devices.len() > MAX_DEVICES
            || self.links.len() > MAX_LINKS
            || self.name.len() > 4_096
        {
            return Err(Diagnostic::new("resource.limit_exceeded", "project"));
        }
        for (id, config) in &self.lan {
            if self.device(id).is_none()
                || config.sat_retention_us.0 == 0
                || config.sat_retention_us.0 > 86_400_000_000
            {
                return Err(Diagnostic::new(
                    "validation.invalid_lan_config",
                    id.as_str(),
                ));
            }
        }
        for (id, c) in &self.lan {
            if c.symbol == Some(DeviceSymbol::Cloud) {
                if self.schema_version < 8 {
                    return Err(Diagnostic::new(
                        "project.unsupported_version",
                        "cloud_symbol",
                    ));
                }
                if !self
                    .device(id)
                    .is_some_and(|d| d.kind == DeviceKind::Switch)
                {
                    return Err(Diagnostic::new("presentation.invalid_symbol", id.as_str()));
                }
            }
        }
        if self.schema_version < 4 && self.lan.values().any(|c| c.symbol.is_some()) {
            return Err(Diagnostic::new(
                "project.unsupported_version",
                "device_symbols",
            ));
        }
        if self.schema_version < 7
            && self.lan.values().any(|c| {
                matches!(
                    c.symbol,
                    Some(DeviceSymbol::AccessPoint | DeviceSymbol::Modem)
                )
            })
        {
            return Err(Diagnostic::new(
                "project.unsupported_version",
                "connection_symbols",
            ));
        }
        let mut ids = BTreeSet::new();
        let mut insert = |id: &Id| {
            if ids.insert(id.clone()) {
                Ok(())
            } else {
                Err(Diagnostic::new("project.duplicate_id", id.as_str()))
            }
        };
        insert(&self.project_id)?;
        let mut interfaces = BTreeMap::new();
        let mut macs = BTreeSet::new();
        let mut ssids = BTreeSet::new();
        for device in &self.devices {
            insert(&device.id)?;
            if let Some(config) = &device.network.modem {
                config.validate()?;
                if self.schema_version < 7
                    || device.kind != DeviceKind::Modem
                    || device.interfaces.len() != 1
                    || device.interfaces[0].medium != Medium::Ethernet
                {
                    return Err(Diagnostic::new("bridge.invalid_config", device.id.as_str()));
                }
            }
            if device.network.ssid.is_some() || !device.network.wireless.is_empty() {
                if self.schema_version < 7 {
                    return Err(Diagnostic::new("project.unsupported_version", "wireless"));
                }
                if let Some(ssid) = &device.network.ssid
                    && (device.kind != DeviceKind::Switch
                        || !ssid_valid(ssid)
                        || !ssids.insert(ssid)
                        || device
                            .interfaces
                            .iter()
                            .filter(|i| i.medium == Medium::Wireless)
                            .count()
                            != 1)
                {
                    return Err(Diagnostic::new(
                        "wireless.invalid_config",
                        device.id.as_str(),
                    ));
                }
                if device.network.wireless.iter().any(|(id, ssid)| {
                    device.kind == DeviceKind::Switch
                        || !ssid_valid(ssid)
                        || !device
                            .interfaces
                            .iter()
                            .any(|i| &i.id == id && i.medium == Medium::Wireless)
                }) {
                    return Err(Diagnostic::new(
                        "wireless.invalid_config",
                        device.id.as_str(),
                    ));
                }
            }
            if device.network.routes.len() > 256 {
                return Err(Diagnostic::new(
                    "resource.limit_exceeded",
                    device.id.as_str(),
                ));
            }
            if self.schema_version < 4
                && (device.network.forwarding.is_some()
                    || device.network.rip
                    || !device.network.routes.is_empty())
            {
                return Err(Diagnostic::new("project.unsupported_version", "routing"));
            }
            if self.schema_version >= 4
                && device.kind == DeviceKind::Router
                && !(2..=32).contains(&device.interfaces.len())
            {
                return Err(Diagnostic::new(
                    "routing.router_interfaces",
                    device.id.as_str(),
                ));
            }
            if device.kind == DeviceKind::Switch
                && (device.network.forwarding.is_some() || !device.network.routes.is_empty())
            {
                return Err(Diagnostic::new("device.not_host", device.id.as_str()));
            }
            if device.network.rip && device.kind != DeviceKind::Router {
                return Err(Diagnostic::new(
                    "routing.rip_requires_router",
                    device.id.as_str(),
                ));
            }
            let mut routes = BTreeSet::new();
            for route in &device.network.routes {
                route.validate(device)?;
                if !routes.insert((route.network, route.prefix, route.interface.clone())) {
                    return Err(Diagnostic::new(
                        "routing.duplicate_route",
                        device.id.as_str(),
                    ));
                }
            }

            if device.label.len() > 4_096 || device.interfaces.len() > 128 {
                return Err(Diagnostic::new(
                    "resource.limit_exceeded",
                    device.id.as_str(),
                ));
            }
            for interface in &device.interfaces {
                insert(&interface.id)?;
                let parts: Vec<_> = interface.mac.split(':').collect();
                if parts.len() != 6
                    || parts
                        .iter()
                        .any(|p| p.len() != 2 || !p.bytes().all(|b| b.is_ascii_hexdigit()))
                {
                    return Err(Diagnostic::new(
                        "validation.invalid_mac",
                        interface.id.as_str(),
                    ));
                }
                if !macs.insert(interface.mac.to_ascii_lowercase()) {
                    return Err(Diagnostic::new(
                        "validation.duplicate_mac",
                        interface.id.as_str(),
                    ));
                }
                if let Some(Ipv4Config::Static {
                    address,
                    prefix_length,
                }) = &interface.ipv4
                    && (address.parse::<std::net::Ipv4Addr>().is_err() || *prefix_length > 32)
                {
                    return Err(Diagnostic::new(
                        "validation.invalid_ipv4",
                        interface.id.as_str(),
                    ));
                }
                interfaces.insert(interface.id.clone(), device.id.clone());
            }
            for address in [&device.network.default_gateway, &device.network.dns_server]
                .into_iter()
                .flatten()
            {
                if address.parse::<std::net::Ipv4Addr>().is_err() {
                    return Err(Diagnostic::new(
                        "validation.invalid_ipv4",
                        device.id.as_str(),
                    ));
                }
            }
            if device.network.gateway.is_some() || device.network.firewall.is_some() {
                if self.schema_version < 6 {
                    return Err(Diagnostic::new("project.unsupported_version", "security"));
                }
                if let Some(c) = &device.network.gateway {
                    c.validate(device)?;
                }
                if let Some(c) = &device.network.firewall {
                    if device.kind == DeviceKind::Switch {
                        return Err(Diagnostic::new("device.not_host", device.id.as_str()));
                    }
                    c.validate()?;
                }
            }
            if let Some(config) = &device.network.dhcp_server {
                if self.schema_version < 5 {
                    return Err(Diagnostic::new("project.unsupported_version", "dhcp"));
                }
                config.validate(device)?;
            }
            for app in &device.applications {
                if let Some(data) = &app.data {
                    if self.schema_version < 7 {
                        return Err(Diagnostic::new("project.unsupported_version", "mail_data"));
                    }
                    data.validate(&app.kind)?;
                }
                if matches!(
                    &app.service,
                    Some(
                        ServiceConfig::MailServer(_)
                            | ServiceConfig::MailClient(_)
                            | ServiceConfig::P2p(_)
                    )
                ) && self.schema_version < 7
                {
                    return Err(Diagnostic::new("project.unsupported_version", "mail"));
                }
                if let Some(config) = &app.service {
                    if self.schema_version < 5 {
                        return Err(Diagnostic::new("project.unsupported_version", "service"));
                    }
                    if matches!(config,ServiceConfig::Http(c) if c.administration)
                        && self.schema_version < 6
                    {
                        return Err(Diagnostic::new(
                            "project.unsupported_version",
                            "administration",
                        ));
                    }
                    config.validate(&app.kind)?;
                }
                insert(&app.id)?;
                if app.kind.len() > 128
                    || app.config.len() > 128
                    || app
                        .config
                        .iter()
                        .any(|(key, value)| key.len() > 128 || value.len() > 4_096)
                {
                    return Err(Diagnostic::new("resource.limit_exceeded", app.id.as_str()));
                }
            }
            let mut paths = BTreeSet::new();
            for file in &device.files {
                validate_base64(&file.data_base64)?;
                if !virtual_path_valid(&file.path)
                    || !paths.insert(&file.path)
                    || file.data_base64.len() > MAX_BODY_BYTES * 2
                {
                    return Err(Diagnostic::new("project.invalid_file", &file.path));
                }
            }
        }
        let mut occupied = BTreeSet::new();
        for link in &self.links {
            insert(&link.id)?;
            if link.endpoints[0] == link.endpoints[1]
                || link.latency_us.0 == 0
                || link.latency_us.0.checked_add(link.jitter_us.0).is_none()
            {
                return Err(Diagnostic::new("topology.invalid_link", link.id.as_str()));
            }
            for endpoint in &link.endpoints {
                if !interfaces.contains_key(endpoint) {
                    return Err(Diagnostic::new(
                        "project.invalid_reference",
                        endpoint.as_str(),
                    ));
                }
                if self.device_for_interface(endpoint).is_some_and(|d| {
                    d.interfaces
                        .iter()
                        .any(|i| &i.id == endpoint && i.medium != Medium::Ethernet)
                }) {
                    return Err(Diagnostic::new(
                        "topology.incompatible_ports",
                        endpoint.as_str(),
                    ));
                }
                if !occupied.insert(endpoint) {
                    return Err(Diagnostic::new("topology.port_busy", endpoint.as_str()));
                }
            }
        }
        let devices: BTreeSet<_> = self.devices.iter().map(|d| &d.id).collect();
        let mut positions = BTreeSet::new();
        for position in &self.layout.devices {
            if !devices.contains(&position.device_id) || !positions.insert(&position.device_id) {
                return Err(Diagnostic::new(
                    "project.invalid_reference",
                    position.device_id.as_str(),
                ));
            }
            if !coordinate_valid(position.x, position.y) {
                return Err(Diagnostic::new(
                    "validation.invalid_position",
                    position.device_id.as_str(),
                ));
            }
        }
        if self.annotations.len() > MAX_ANNOTATIONS {
            return Err(Diagnostic::new("resource.limit_exceeded", "annotations"));
        }
        for annotation in &self.annotations {
            insert(&annotation.id)?;
            if !coordinate_valid(annotation.x, annotation.y)
                || annotation.text.len() > 65_536
                || annotation
                    .style
                    .as_ref()
                    .is_some_and(|s| self.schema_version < 9 || !s.validate())
            {
                return Err(Diagnostic::new(
                    "validation.invalid_annotation",
                    annotation.id.as_str(),
                ));
            }
        }
        for resource in &self.resources {
            insert(&resource.id)?;
            validate_base64(&resource.data_base64)?;
            if resource.data_base64.len() > MAX_BODY_BYTES * 2 {
                return Err(Diagnostic::new(
                    "resource.limit_exceeded",
                    resource.id.as_str(),
                ));
            }
        }
        for extension in &self.extensions {
            insert(&extension.id)?;
            if extension.enabled {
                return Err(Diagnostic::new(
                    "extension.not_supported",
                    extension.id.as_str(),
                ));
            }
            if !self.resources.iter().any(|r| r.id == extension.resource_id) {
                return Err(Diagnostic::new(
                    "project.invalid_reference",
                    extension.resource_id.as_str(),
                ));
            }
        }
        Ok(())
    }
    pub fn device(&self, id: &Id) -> Option<&Device> {
        self.devices.iter().find(|d| d.id == *id)
    }
    pub fn device_for_interface(&self, id: &Id) -> Option<&Device> {
        self.devices
            .iter()
            .find(|d| d.interfaces.iter().any(|i| i.id == *id))
    }
}
fn coordinate_valid(x: f32, y: f32) -> bool {
    x.is_finite() && y.is_finite() && x.abs() <= 1_000_000.0 && y.abs() <= 1_000_000.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Message {
    Signal {
        text: String,
    },
    HttpRequest {
        method: HttpMethod,
        path: String,
        fields: BTreeMap<String, String>,
    },
    HttpResponse {
        status: u16,
        media_type: String,
        body_base64: String,
    },
}
impl Message {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        let encoded = serde_json::to_vec(self)
            .map_err(|_| Diagnostic::new("validation.invalid_message", "message"))?;
        if encoded.len() > MAX_BODY_BYTES * 2 {
            return Err(Diagnostic::new("resource.limit_exceeded", "message"));
        }
        if let Self::HttpRequest { path, .. } = self
            && !virtual_path_valid(path)
        {
            return Err(Diagnostic::new("validation.invalid_virtual_path", path));
        }
        if let Self::HttpResponse {
            status,
            body_base64,
            ..
        } = self
        {
            if !(100..=599).contains(status) {
                return Err(Diagnostic::new("validation.invalid_status", "response"));
            }
            validate_base64(body_base64)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduledMessage {
    pub at_us: Decimal,
    pub source: Id,
    pub target: Id,
    pub message: Message,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRoute {
    pub path: String,
    pub media_type: String,
    pub body_base64: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeServer {
    pub device_id: Id,
    pub get_routes: Vec<HttpRoute>,
    pub post_echo_path: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub scenario_schema_version: u32,
    pub project: Project,
    pub seed: Decimal,
    pub actions: Vec<ScheduledMessage>,
    pub prototype_servers: Vec<PrototypeServer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub network_actions: Vec<NetworkAction>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkAction {
    pub id: Id,
    pub at_us: Decimal,
    pub command: Command,
}
impl Scenario {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if !matches!(self.scenario_schema_version, 1 | 2) {
            return Err(Diagnostic::new("scenario.unsupported_version", "scenario"));
        }
        self.project.validate()?;
        if (self.scenario_schema_version == 1 && !self.network_actions.is_empty())
            || (self.scenario_schema_version == 2
                && (!self.actions.is_empty() || !self.prototype_servers.is_empty()))
        {
            return Err(Diagnostic::new(
                "scenario.unsupported_version",
                "mixed_profiles",
            ));
        }
        if self.network_actions.len() > 4096 {
            return Err(Diagnostic::new(
                "resource.limit_exceeded",
                "network_actions",
            ));
        }
        let mut identities = BTreeSet::new();
        let mut previous = 0;
        for action in &self.network_actions {
            if action.at_us.0 < previous
                || !identities.insert(&action.id)
                || matches!(
                    action.command,
                    Command::Send { .. }
                        | Command::Step
                        | Command::Stop
                        | Command::Resume
                        | Command::Pause
                )
            {
                return Err(Diagnostic::new(
                    "scenario.invalid_action",
                    action.id.as_str(),
                ));
            }
            previous = action.at_us.0;
        }
        if self.actions.len() > MAX_ACTIONS {
            return Err(Diagnostic::new("resource.limit_exceeded", "actions"));
        }
        for action in &self.actions {
            if self.project.device(&action.source).is_none()
                || self.project.device(&action.target).is_none()
            {
                return Err(Diagnostic::new("project.invalid_reference", "action"));
            }
            action.message.validate()?;
        }
        let mut servers = BTreeSet::new();
        for server in &self.prototype_servers {
            if self.project.device(&server.device_id).is_none()
                || !servers.insert(&server.device_id)
                || !virtual_path_valid(&server.post_echo_path)
                || server.get_routes.len() > 128
            {
                return Err(Diagnostic::new(
                    "validation.invalid_server",
                    server.device_id.as_str(),
                ));
            }
            let mut paths = BTreeSet::new();
            for route in &server.get_routes {
                validate_base64(&route.body_base64)?;
                if !virtual_path_valid(&route.path)
                    || !paths.insert(&route.path)
                    || route.body_base64.len() > MAX_BODY_BYTES * 2
                {
                    return Err(Diagnostic::new("validation.invalid_route", &route.path));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Paused,
    Running,
    Stopped,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Command {
    #[serde(rename = "presentation.symbol.set")]
    SetSymbol {
        device: Id,
        symbol: Option<DeviceSymbol>,
    },
    #[serde(rename = "simulation.link.configure")]
    ConfigureLink {
        link: Id,
        conditions: LinkConditions,
    },
    #[serde(rename = "bridge.command")]
    Bridge { device: Id, command: BridgeCommand },
    #[serde(rename = "security.command")]
    Security {
        device: Id,
        command: SecurityCommand,
    },
    #[serde(rename = "routing.rip.mute")]
    RipMute { device: Id, muted: bool },
    #[serde(rename = "app.command")]
    App { device: Id, command: AppCommand },
    #[serde(rename = "socket.command")]
    Socket { device: Id, command: SocketCommand },
    #[serde(rename = "icmp.ping")]
    Ping {
        source: Id,
        destination: String,
        ttl: u8,
    },
    #[serde(rename = "simulation.manual_loss")]
    ManualLoss { enabled: bool },
    #[serde(rename = "arp.clear")]
    PurgeArp { device: Id },
    #[serde(rename = "switch.sat.clear")]
    PurgeSat { device: Id },
    #[serde(rename = "simulation.pause")]
    Pause,
    #[serde(rename = "simulation.resume")]
    Resume,
    #[serde(rename = "simulation.step")]
    Step,
    #[serde(rename = "simulation.stop")]
    Stop,
    #[serde(rename = "prototype.send")]
    Send {
        source: Id,
        target: Id,
        message: Message,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandEnvelope {
    pub api_version: u32,
    pub command_id: Id,
    pub project_id: Id,
    pub expected_revision: Decimal,
    pub run_id: Id,
    pub command: Command,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decimal_is_exact_and_canonical() {
        let value = Decimal(u64::MAX);
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            "\"18446744073709551615\""
        );
        assert_eq!(
            serde_json::from_str::<Decimal>("\"18446744073709551615\"").unwrap(),
            value
        );
        for bad in [
            "42",
            "\"01\"",
            "\"+1\"",
            "\"-1\"",
            "\"18446744073709551616\"",
        ] {
            assert!(serde_json::from_str::<Decimal>(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn paths_cannot_escape_into_host_resources() {
        for path in [
            "https://example.org/a",
            "file:///etc/passwd",
            "//host/a",
            "/../a",
            "/a/./b",
            "/%2e%2e/a",
            "/a\\b",
        ] {
            assert!(!virtual_path_valid(path), "{path}");
        }
        assert!(virtual_path_valid("/images/logo.png"));
    }
}
