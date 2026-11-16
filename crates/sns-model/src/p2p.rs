//! Portable configuration and commands for the bounded educational P2P overlay.
use crate::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct P2pConfig {
    pub port: u16,
    pub max_neighbors: u8,
    pub shared_directory: String,
    #[serde(default)]
    pub bootstrap: Option<Endpoint>,
}
impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            port: 6346,
            max_neighbors: 8,
            shared_directory: "/shared".into(),
            bootstrap: None,
        }
    }
}
impl P2pConfig {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if self.port == 0
            || !(1..=32).contains(&self.max_neighbors)
            || !virtual_path_valid(&self.shared_directory)
            || self.bootstrap.is_some_and(|e| {
                e.port == 0
                    || e.address.is_unspecified()
                    || e.address.is_multicast()
                    || e.address.is_broadcast()
            })
        {
            return Err(Diagnostic::new("p2p.invalid_config", "p2p"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum P2pCommand {
    Join {
        #[serde(default)]
        peer: Option<Endpoint>,
    },
    Discover {
        ttl: u8,
    },
    Search {
        text: String,
        ttl: u8,
    },
    CancelSearch,
    ClearResults,
    Reset,
    Download {
        result: Id,
        destination: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct P2pResult {
    pub id: Id,
    pub source: Endpoint,
    pub path: String,
    pub media_type: String,
    pub bytes: u32,
    pub sha256: String,
}
