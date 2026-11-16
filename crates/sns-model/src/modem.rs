//! Explicit bridge adapter configuration and recorded external input contract.
use crate::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModemRole {
    Listen,
    Connect,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModemConfig {
    pub role: ModemRole,
    pub endpoint: String,
}
impl Default for ModemConfig {
    fn default() -> Self {
        Self {
            role: ModemRole::Listen,
            endpoint: "127.0.0.1:50555".into(),
        }
    }
}
impl ModemConfig {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        self.endpoint
            .parse::<std::net::SocketAddr>()
            .map_err(|_| Diagnostic::new("bridge.invalid_endpoint", &self.endpoint))?;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgePacket {
    pub version: u8,
    pub sequence: Decimal,
    pub budget: u8,
    pub frame: serde_json::Value,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum BridgeCommand {
    Activate { session: Id },
    Disconnect { session: Id, reason: String },
    Input { session: Id, packet: BridgePacket },
}
pub const BRIDGE_MAX_PACKET: usize = 131_072;
