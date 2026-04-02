//! Genesis block creation and management
//!
//! Handles generation of testnet genesis blocks with bootstrap validators.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Genesis block configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlock {
    pub chain_id: String,
    pub timestamp: i64,
    pub genesis_hash: String,
    pub bootstrap_validators: Vec<GenesisValidator>,
    pub consensus: ConsensusConfig,
    pub rewards: RewardsConfig,
}

/// Bootstrap validator in genesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisValidator {
    pub identity_pubkey: String,
    pub stake: u64,
    pub commission: u8,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub slot_time_ms: u64,
    pub tower_finality: u64,
    pub min_stake: u64,
    pub target_stake: u64,
}

/// Rewards configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsConfig {
    pub epoch_duration: u64,
    pub base_reward_rate: u64,
}

/// Generate genesis hash from configuration
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
        },
        rewards: RewardsConfig {
            epoch_duration: 432_000,
            base_reward_rate: 6,
        },
    }
}

/// Bootstrap validator keypair generation (for testnet setup)
pub fn generate_bootstrap_keypair() -> (String, Vec<u8>) {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    let signing_key = SigningKey::generate(&mut OsRng);
    let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    
    (pubkey, signing_key.to_bytes().to_vec())
}
