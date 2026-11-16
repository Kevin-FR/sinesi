//! Portable routing configuration. No operating-system routing table is consulted.
use crate::*;
use std::net::Ipv4Addr;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticRoute {
    pub network: Ipv4Addr,
    pub prefix: u8,
    pub gateway: Option<Ipv4Addr>,
    pub interface: Id,
}
impl StaticRoute {
    pub fn validate(&self, device: &Device) -> Result<(), Diagnostic> {
        let bad = || Diagnostic::new("routing.invalid_route", self.interface.as_str());
        let mask = if self.prefix == 0 {
            0
        } else {
            u32::MAX
                .checked_shl(u32::from(32_u8.saturating_sub(self.prefix)))
                .ok_or_else(bad)?
        };
        if self.prefix > 32 || u32::from(self.network) & !mask != 0 || self.network.is_multicast() {
            return Err(bad());
        }
        let nic = device
            .interfaces
            .iter()
            .find(|i| i.id == self.interface)
            .ok_or_else(bad)?;
        let Some(Ipv4Config::Static {
            address,
            prefix_length,
        }) = &nic.ipv4
        else {
            return Err(bad());
        };
        if *prefix_length > 32 {
            return Err(bad());
        }
        let address: Ipv4Addr = address.parse().map_err(|_| bad())?;
        if let Some(gateway) = self.gateway {
            let mask = if *prefix_length == 0 {
                0
            } else {
                u32::MAX << (32 - prefix_length)
            };
            if gateway.is_unspecified()
                || gateway.is_multicast()
                || gateway.is_loopback()
                || gateway == address
                || u32::from(gateway) & mask != u32::from(address) & mask
                || (*prefix_length < 31
                    && (u32::from(gateway) & !mask == 0 || u32::from(gateway) & !mask == !mask))
            {
                return Err(bad());
            }
        }
        Ok(())
    }
}
