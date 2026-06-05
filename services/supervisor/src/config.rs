//! TOML supervisor-tree configuration loader.

use crate::{ChildSpec, SupError, SupResult, Strategy};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Top-level config file format. See `examples/supervisor.toml` for a sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorConfig {
    /// Logical name (root, network, storage, ...).
    pub name: String,
    /// Restart strategy.
    pub strategy: Strategy,
    /// Max restarts allowed inside the window before giving up.
    #[serde(default = "default_max_restarts")]
    pub max_restarts: u32,
    /// Window size, in seconds.
    #[serde(default = "default_within_seconds")]
    pub within_seconds: u64,
    /// Optional nested supervisors.
    #[serde(default)]
    pub supervisors: Vec<SupervisorConfig>,
    /// Worker children supervised at this level.
    #[serde(default)]
    pub children: Vec<ChildSpec>,
}

fn default_max_restarts() -> u32 {
    5
}
fn default_within_seconds() -> u64 {
    60
}

impl SupervisorConfig {
    /// Load and parse a config file.
    pub fn load(path: impl AsRef<Path>) -> SupResult<Self> {
        let text = std::fs::read_to_string(path.as_ref())
            .map_err(|e| SupError::Config(format!("read: {e}")))?;
        Self::from_str(&text)
    }

    /// Parse from a TOML string.
    pub fn from_str(text: &str) -> SupResult<Self> {
        toml::from_str(text).map_err(|e| SupError::Config(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_tree() {
        let text = r#"
            name = "root"
            strategy = "one-for-one"
            max_restarts = 5
            within_seconds = 60

            [[supervisors]]
            name = "storage"
            strategy = "one-for-all"
            max_restarts = 3
            within_seconds = 30

                [[supervisors.children]]
                name = "fs_go"
                command = "services/fs_go/fs_go"
                restart = "permanent"

                [[supervisors.children]]
                name = "ngfs"
                command = "services/ngfs/target/release/ngfs"
                restart = "permanent"
        "#;
        let cfg: SupervisorConfig = SupervisorConfig::from_str(text).unwrap();
        assert_eq!(cfg.name, "root");
        assert_eq!(cfg.supervisors.len(), 1);
        assert_eq!(cfg.supervisors[0].children.len(), 2);
    }
}
