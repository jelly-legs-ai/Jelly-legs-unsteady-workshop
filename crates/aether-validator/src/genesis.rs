//! Genesis block creation and management
//!
//! Handles generation and loading of testnet genesis blocks.

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use anyhow::Context;

// ============================================================================
// Validator Tier System
// ============================================================================

/// Validator tier classification determining consensus participation level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorTier {
    /// Full Validator — 10K AETH min stake, 1.0x consensus weight, block production
    Full,
    /// Lite Validator — 1K AETH min stake, stake-based weight, no solo block production
    Lite,
    /// Observer Node — 0 AETH, relay-only, earns FLUX via data relay
    Observer,
}

impl Default for ValidatorTier {
    fn default() -> Self {
        ValidatorTier::Full
    }
}

/// Per-tier configuration used in ConsensusConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    pub tier: ValidatorTier,
    /// Consensus weight: 1.0 for full, stake/10000 for lite, 0.0 for observer
    pub consensus_weight: f64,
    /// Whether this tier can produce blocks (full validators only)
    pub can_produce_blocks: bool,
    /// Whether this tier can vote on consensus (full and lite)
    pub can_vote: bool,
    /// Minimum stake required for this tier in motes (1 AETH = 100_000_000 motes)
    /// Full: 10_000 AETH, Lite: 1_000 AETH, Observer: 0
    pub min_stake: u64,
    /// FLUX per byte relayed per epoch (observer only)
    pub relay_reward_rate: f64,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            tier: ValidatorTier::Full,
            consensus_weight: 1.0,
            can_produce_blocks: true,
            can_vote: true,
            min_stake: 10_000 * 100_000_000, // 10,000 AETH in motes
            relay_reward_rate: 0.0,
        }
    }
}

/// Genesis block configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GenesisBlock {
    pub chain_id: String,
    pub timestamp: i64,
    pub genesis_hash: String,
    pub bootstrap_validators: Vec<GenesisValidator>,
    pub consensus: ConsensusConfig,
    pub rewards: RewardsConfig,
    // Optional top-level fields from older genesis files
    pub slot_time_ms: Option<u64>,
    pub slots_per_epoch: Option<u64>,
    pub min_stake: Option<u64>,
    pub bootstrap_multiplier: Option<u64>,
}

impl Default for GenesisBlock {
    fn default() -> Self {
        Self {
            chain_id: "aether-testnet-1".to_string(),
            timestamp: 0,
            genesis_hash: String::new(),
            bootstrap_validators: Vec::new(),
            consensus: ConsensusConfig::default(),
            rewards: RewardsConfig::default(),
            slot_time_ms: None,
            slots_per_epoch: None,
            min_stake: None,
            bootstrap_multiplier: None,
        }
    }
}

/// Bootstrap validator in genesis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GenesisValidator {
    pub identity_pubkey: String,
    #[serde(alias = "activated_stake", default)]
    pub stake: u64,
    pub commission: u8,
    #[serde(default)]
    pub active: bool,
}

impl Default for GenesisValidator {
    fn default() -> Self {
        Self {
            identity_pubkey: String::new(),
            stake: 0,
            commission: 10,
            active: true,
        }
    }
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConsensusConfig {
    pub slot_time_ms: u64,
    pub tower_finality: u64,
    pub min_stake: u64,
    pub target_stake: u64,
    /// Optional per-tier configuration (defaults to Full validator if None)
    #[serde(default)]
    pub tier_config: Option<TierConfig>,
    // Extra fields from older configs (ignored)
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub poh_target_ticks_per_sec: Option<u64>,
    #[serde(default)]
    pub poh_ticks_per_slot: Option<u64>,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            slot_time_ms: 400,
            tower_finality: 12,
            min_stake: 100,
            target_stake: 1_000_000,
            tier_config: Some(TierConfig::default()),
            mode: None,
            poh_target_ticks_per_sec: None,
            poh_ticks_per_slot: None,
        }
    }
}

/// Rewards configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RewardsConfig {
    pub epoch_duration: u64,
    pub base_reward_rate: u64,
    #[serde(default)]
    pub bootstrap_bonus: Option<u64>,
}

impl Default for RewardsConfig {
    fn default() -> Self {
        Self {
            epoch_duration: 432_000,
            base_reward_rate: 6,
            bootstrap_bonus: None,
        }
    }
}

/// Load genesis block from file (JSON)
pub fn load_genesis_from_file(path: &Path) -> anyhow::Result<GenesisBlock> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read genesis file: {}", path.display()))?;
    
    let genesis: GenesisBlock = serde_json::from_str(&content)
        .with_context(|| "Failed to parse genesis JSON")?;
    
    Ok(genesis)
}

/// Load genesis block from JSON string
pub fn load_genesis_from_json(json: &str) -> anyhow::Result<GenesisBlock> {
    let genesis: GenesisBlock = serde_json::from_str(json)
        .with_context(|| "Failed to parse genesis JSON")?;
    Ok(genesis)
}

/// Generate genesis hash from configuration bytes
pub fn compute_genesis_hash(genesis: &GenesisBlock) -> String {
    let mut hasher = Sha256::new();
    hasher.update(genesis.chain_id.as_bytes());
    hasher.update(genesis.timestamp.to_le_bytes());
    for v in &genesis.bootstrap_validators {
        hasher.update(v.identity_pubkey.as_bytes());
        hasher.update(v.stake.to_le_bytes());
    }
    let result = hasher.finalize();
    bs58::encode(result).into_string()
}

/// Verify genesis hash matches
pub fn verify_genesis_hash(genesis: &GenesisBlock) -> bool {
    let computed = compute_genesis_hash(genesis);
    computed == genesis.genesis_hash
}

/// Generate genesis hash from raw bytes (for in-memory genesis)
pub fn generate_genesis_hash() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut hasher = Sha256::new();
    hasher.update(b"aether-genesis-v1");
    hasher.update(timestamp.to_le_bytes());
    hasher.update(b"aether-testnet-1");
    
    let result = hasher.finalize();
    bs58::encode(result).into_string()
}

/// Create a testnet genesis block with default settings
pub fn create_testnet_genesis() -> GenesisBlock {
    GenesisBlock {
        chain_id: "aether-testnet-1".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        genesis_hash: generate_genesis_hash(),
        bootstrap_validators: Vec::new(),
        consensus: ConsensusConfig {
            slot_time_ms: 400,
            tower_finality: 12,
            min_stake: 100,
            target_stake: 1_000_000,
            tier_config: Some(TierConfig::default()),
            ..Default::default()
        },
        rewards: RewardsConfig {
            epoch_duration: 432_000,
            base_reward_rate: 6,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Create genesis with specific chain ID and validators
pub fn create_genesis_with(
    chain_id: &str,
    validators: Vec<GenesisValidator>,
) -> GenesisBlock {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let mut genesis = GenesisBlock {
        chain_id: chain_id.to_string(),
        timestamp,
        genesis_hash: String::new(),
        bootstrap_validators: validators,
        consensus: ConsensusConfig {
            slot_time_ms: 400,
            tower_finality: 12,
            min_stake: 100,
            target_stake: 1_000_000,
            tier_config: Some(TierConfig::default()),
            ..Default::default()
        },
        rewards: RewardsConfig {
            epoch_duration: 432_000,
            base_reward_rate: 6,
            ..Default::default()
        },
        ..Default::default()
    };
    
    genesis.genesis_hash = compute_genesis_hash(&genesis);
    genesis
}

/// Bootstrap validator keypair generation (for testnet setup)
pub fn generate_bootstrap_keypair() -> (String, Vec<u8>) {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let signing_key = SigningKey::from_bytes(&bytes);
    let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    
    (pubkey, signing_key.to_bytes().to_vec())
}
