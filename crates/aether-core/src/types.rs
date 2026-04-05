//! AETHER Core Types
//!
//! Core blockchain types for Aether transactions, blocks, and accounts.

use serde::{Deserialize, Serialize};

/// Address type alias (32 bytes)
pub type Address = [u8; 32];

/// Account in the Aether state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub lamports: u64,
    pub owner: [u8; 32],
    pub data: Vec<u8>,
    pub rent_epoch: u64,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            lamports: 0,
            owner: [0u8; 32],
            data: Vec::new(),
            rent_epoch: 0,
        }
    }
}

/// Genesis account for initialization
#[derive(Debug, Clone)]
pub struct GenesisAccount {
    pub address: [u8; 32],
    pub lamports: u64,
    pub data: Option<Vec<u8>>,
}

/// Transaction type variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Transfer,
    Stake,
    Unstake,
    ClaimRewards,
    CreateNFT,
    MintNFT,
    TransferNFT,
    UpdateMetadata,
}

impl Default for TransactionType {
    fn default() -> Self {
        TransactionType::Transfer
    }
}

/// Transaction payload (instruction data)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TransactionPayload {
    Transfer { recipient: String, amount: u64, nonce: u64 },
    Stake { validator: String, amount: u64 },
    Unstake { stake_account: String, amount: u64 },
    ClaimRewards { stake_account: String },
    CreateNFT { metadata_url: String, royalties: u16 },
    MintNFT { nft_id: String, amount: u64 },
    TransferNFT { nft_id: String, recipient: String },
    UpdateMetadata { nft_id: String, metadata_url: String },
}

impl Default for TransactionPayload {
    fn default() -> Self {
        TransactionPayload::Transfer { recipient: String::new(), amount: 0, nonce: 0 }
    }
}

/// An Aether transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherTransaction {
    #[serde(with = "serde_bytes_64")]
    pub signature: [u8; 64],
    #[serde(with = "serde_bytes_32")]
    pub signer: [u8; 32],
    pub tx_type: TransactionType,
    pub payload: TransactionPayload,
    pub fee: u64,
    pub slot: u64,
    pub timestamp: u64,
}

/// Result of executing a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub error: Option<String>,
    pub gas_used: u64,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self { success: false, error: Some("Not executed".to_string()), gas_used: 0 }
    }
}

impl ExecutionResult {
    pub fn success() -> Self {
        Self { success: true, error: None, gas_used: 0 }
    }
    pub fn failure(error: impl Into<String>) -> Self {
        Self { success: false, error: Some(error.into()), gas_used: 0 }
    }
}

/// Transaction receipt — proof of execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    #[serde(with = "serde_bytes_64")]
    pub signature: [u8; 64],
    pub slot: u64,
    pub block_hash: String,
    pub tx_type: TransactionType,
    #[serde(with = "serde_bytes_32")]
    pub signer: [u8; 32],
    pub result: ExecutionResult,
    pub timestamp: u64,
}

// Custom serde for base58-encoded 64-byte arrays
mod serde_bytes_64 {
    use serde::{Deserialize, Deserializer, Serializer};
    
    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&bs58::encode(bytes).into_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let decoded = bs58::decode(&s).into_vec().map_err(serde::de::Error::custom)?;
        let mut arr = [0u8; 64];
        let len = decoded.len().min(64);
        arr[..len].copy_from_slice(&decoded[..len]);
        Ok(arr)
    }
}

// Custom serde for base58-encoded 32-byte arrays  
mod serde_bytes_32 {
    use serde::{Deserialize, Deserializer, Serializer};
    
    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&bs58::encode(bytes).into_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let decoded = bs58::decode(&s).into_vec().map_err(serde::de::Error::custom)?;
        let mut arr = [0u8; 32];
        let len = decoded.len().min(32);
        arr[..len].copy_from_slice(&decoded[..len]);
        Ok(arr)
    }
}
