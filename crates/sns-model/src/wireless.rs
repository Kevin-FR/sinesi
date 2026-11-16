//! Logical SSID associations. Radio propagation and encryption are outside this profile.
use crate::*;
impl Project {
    /// One access-point radio per SSID; clients retain their selected SSID when it disappears.
    pub fn wireless_links(&self) -> Vec<(Id, Id)> {
        let mut links = Vec::new();
        for host in &self.devices {
            if host.kind == DeviceKind::Switch {
                continue;
            }
            for (port, ssid) in &host.network.wireless {
                if !host
                    .interfaces
                    .iter()
                    .any(|i| &i.id == port && i.medium == Medium::Wireless)
                {
                    continue;
                }
                if let Some(radio) = self
                    .devices
                    .iter()
                    .filter(|d| {
                        d.kind == DeviceKind::Switch && d.network.ssid.as_ref() == Some(ssid)
                    })
                    .flat_map(|d| &d.interfaces)
                    .find(|i| i.medium == Medium::Wireless)
                {
                    links.push((port.clone(), radio.id.clone()));
                }
            }
        }
        links.sort();
        links
    }
}
pub fn ssid_valid(ssid: &str) -> bool {
    !ssid.is_empty() && ssid.len() <= 32 && !ssid.chars().any(char::is_control)
}
