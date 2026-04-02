//! Validator configuration
//!
//! YAML configuration for validator node settings.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    pub validator: ValidatorSettings,
    pub rpc: RpcConfig,
    pub p2p: P2PConfig,
    pub consensus: ConsensusSettings,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSettings {
    pub identity: PathBuf,
    pub vote_account: PathBuf,
    pub stake_account: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub bind: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    pub bind: String,
    pub port: u16,
    pub peers: u32,
    pub max_peers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusSettings {
    pub slot_time_ms: u64,
    pub tower_finality: u64,
    pub min_stake: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            validator: ValidatorSettings {
                identity: PathBuf::from("validator-identity.json"),
                vote_account: PathBuf::from("vote-account.json"),
                stake_account: None,
            },
            rpc: RpcConfig {
                bind: "0.0.0.0".to_string(),
                port: 8899,
            },
            p2p: P2PConfig {
                bind: "0.0.0.0".to_string(),
                port: 8001,
                peers: 10,
                max_peers: 100,
            },
            consensus: ConsensusSettings {
                slot_time_ms: 400,
                tower_finality: 12,
                min_stake: 100,
            },
            metrics: MetricsConfig {
                enabled: true,
                port: 9320,
            },
        }
    }
}

impl ValidatorConfig {
    /// Load configuration from a YAML file
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        serde_yaml::from_str(&content)
            .context("Failed to parse config file")
    }

    /// Save configuration to a YAML file
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = serde_yaml::to_string(self)
            .context("Failed to serialize config")?;
        
        std::fs::write(path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }
}
