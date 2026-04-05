//! AETHER Core Types
//!
//! Core blockchain types for Aether transactions, blocks, and accounts.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Address type alias (32 bytes)
pub type Address = [u8; 32];

/// Hash type alias (32 bytes)
pub type Hash = [u8; 32];

/// Account in the Aether state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account address (public key)
    pub address: Address,
    /// Lamports (native tokens)
    pub lamports: u64,
    /// Account owner program
    pub owner: Address,
    /// Account data
    pub data: Vec<u8>,
    /// Epoch at which this account will next owe rent
    pub rent_epoch: u64,
    /// Whether this account exists (deleted accounts are marked)
    pub exists: bool,
    /// Whether this account is executable (program account)
    pub executable: bool,
}

impl Account {
    pub fn new(address: Address, lamports: u64) -> Self {
        Self {
            address,
            lamports,
            owner: [0u8; 32],
            data: Vec::new(),
            rent_epoch: 0,
            exists: true,
            executable: false,
        }
    }

    /// Compute the address hash for this account (for state root)
    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(&self.address);
        hasher.update(self.lamports.to_le_bytes());
        hasher.update(&self.owner);
        hasher.update(&self.data);
        hasher.update(self.rent_epoch.to_le_bytes());
        hasher.update([self.exists as u8]);
        hasher.update([self.executable as u8]);
        hasher.finalize().into()
    }
}

impl Default for Account {
    fn default() -> Self {
        Self {
            address: [0u8; 32],
            lamports: 0,
            owner: [0u8; 32],
            data: Vec::new(),
            rent_epoch: 0,
            exists: true,
            executable: false,
        }
    }
}

/// Genesis account for initialization
#[derive(Debug, Clone)]
pub struct GenesisAccount {
    pub address: Address,
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
    Delegate,
    Vote,
    /// Generic/native program instruction
    Native(u32),
}

impl TransactionType {
    /// Serialize to bytes
    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
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
    /// Transfer tokens
    Transfer {
        recipient: String,
        amount: u64,
        nonce: u64,
    },
    /// Stake tokens for consensus participation
    Stake {
        validator: String,
        amount: u64,
        tier: String,
    },
    /// Unstake tokens (initiate unlock period)
    Unstake {
        position_index: usize,
        amount: u64,
    },
    /// Claim accumulated staking rewards
    ClaimRewards {
        position_index: usize,
    },
    /// Create a new NFT
    CreateNFT {
        metadata_uri: String,
        supply: u64,
        name: String,
    },
    /// Mint additional supply of an existing NFT
    MintNFT {
        nft_id: String,
        amount: u64,
    },
    /// Transfer an NFT to another account
    TransferNFT {
        nft_id: String,
        recipient: String,
    },
    /// Update NFT metadata
    UpdateMetadata {
        nft_id: String,
        metadata_uri: String,
    },
    /// Delegate tokens to a validator
    Delegate {
        validator: String,
        amount: u64,
    },
    /// Vote on a block
    Vote {
        slot: u64,
        block_hash: String,
    },
}

impl TransactionPayload {
    /// Serialize the payload to bytes for signing
    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Get the transaction type for this payload
    pub fn tx_type(&self) -> TransactionType {
        match self {
            TransactionPayload::Transfer { .. } => TransactionType::Transfer,
            TransactionPayload::Stake { .. } => TransactionType::Stake,
            TransactionPayload::Unstake { .. } => TransactionType::Unstake,
            TransactionPayload::ClaimRewards { .. } => TransactionType::ClaimRewards,
            TransactionPayload::CreateNFT { .. } => TransactionType::CreateNFT,
            TransactionPayload::MintNFT { .. } => TransactionType::MintNFT,
            TransactionPayload::TransferNFT { .. } => TransactionType::TransferNFT,
            TransactionPayload::UpdateMetadata { .. } => TransactionType::UpdateMetadata,
            TransactionPayload::Delegate { .. } => TransactionType::Delegate,
            TransactionPayload::Vote { .. } => TransactionType::Vote,
        }
    }
}

impl Default for TransactionPayload {
    fn default() -> Self {
        TransactionPayload::Transfer {
            recipient: String::new(),
            amount: 0,
            nonce: 0,
        }
    }
}

/// An Aether transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherTransaction {
    /// 64-byte signature (base58 encoded in JSON)
    #[serde(with = "serde_bytes_64")]
    pub signature: [u8; 64],
    /// Signer public key (base58 encoded in JSON)
    #[serde(with = "serde_bytes_32")]
    pub signer: Address,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Instruction payload
    pub payload: TransactionPayload,
    /// Fee paid (in lamports)
    pub fee: u64,
    /// Slot at which this was included (0 if pending)
    pub slot: u64,
    /// Unix timestamp
    pub timestamp: u64,
}

impl AetherTransaction {
    /// Create a new AetherTransaction
    pub fn new(
        signature: [u8; 64],
        signer: Address,
        payload: TransactionPayload,
        slot: u64,
    ) -> Self {
        let tx_type = payload.tx_type();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            signature,
            signer,
            tx_type,
            payload,
            fee: 0,
            slot,
            timestamp,
        }
    }
}

/// Result of executing a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// State changes caused by this transaction
    pub state_changes: Vec<StateChange>,
    /// Gas used
    pub gas_used: u64,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            success: false,
            error: Some("Not executed".to_string()),
            state_changes: vec![],
            gas_used: 0,
        }
    }
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success() -> Self {
        Self {
            success: true,
            error: None,
            state_changes: vec![],
            gas_used: 0,
        }
    }

    /// Create a successful result with state changes and gas
    pub fn success_with(state_changes: Vec<StateChange>, gas_used: u64) -> Self {
        Self {
            success: true,
            error: None,
            state_changes,
            gas_used,
        }
    }

    /// Create a failed result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(error.into()),
            state_changes: vec![],
            gas_used: 0,
        }
    }

    /// Create a failed result with gas used
    pub fn failure_with(error: impl Into<String>, gas_used: u64) -> Self {
        Self {
            success: false,
            error: Some(error.into()),
            state_changes: vec![],
            gas_used,
        }
    }
}

/// A state change caused by transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// Account that was modified
    #[serde(with = "serde_bytes_32")]
    pub account: Address,
    /// Field that was changed
    pub field: String,
    /// Previous value (as bytes)
    pub old_value: Vec<u8>,
    /// New value (as bytes)
    pub new_value: Vec<u8>,
}

impl StateChange {
    /// Create a lamports state change
    pub fn lamports(account: Address, old_balance: u64, new_balance: u64) -> Self {
        Self {
            account,
            field: "lamports".to_string(),
            old_value: old_balance.to_le_bytes().to_vec(),
            new_value: new_balance.to_le_bytes().to_vec(),
        }
    }

    /// Create a data state change
    pub fn data(account: Address, old_data: Vec<u8>, new_data: Vec<u8>) -> Self {
        Self {
            account,
            field: "data".to_string(),
            old_value: old_data,
            new_value: new_data,
        }
    }
}

/// Transaction receipt — proof of execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction signature (base58 encoded in JSON)
    #[serde(with = "serde_bytes_64")]
    pub signature: [u8; 64],
    /// Slot where transaction was executed
    pub slot: u64,
    /// Block hash (set after block is created)
    pub block_hash: String,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Signer public key (base58 encoded in JSON)
    #[serde(with = "serde_bytes_32")]
    pub signer: Address,
    /// Execution result
    pub result: ExecutionResult,
    /// Unix timestamp
    pub timestamp: u64,
}

impl TransactionReceipt {
    /// Compute the receipt hash for merkle inclusion proof
    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(&self.signature);
        hasher.update(self.slot.to_le_bytes());
        hasher.update(self.block_hash.as_bytes());
        hasher.update(serde_json::to_vec(&self.tx_type).unwrap_or_default());
        hasher.update(&self.signer);
        hasher.update(serde_json::to_vec(&self.result).unwrap_or_default());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.finalize().into()
    }
}

// Custom serde for base58-encoded 64-byte arrays
mod serde_bytes_64 {
    use serde::{Deserialize, Deserializer, Serializer};
    
    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&bs58::encode(bytes).into_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
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
    where
        S: Serializer,
    {
        serializer.serialize_str(&bs58::encode(bytes).into_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let decoded = bs58::decode(&s).into_vec().map_err(serde::de::Error::custom)?;
        let mut arr = [0u8; 32];
        let len = decoded.len().min(32);
        arr[..len].copy_from_slice(&decoded[..len]);
        Ok(arr)
    }
}
