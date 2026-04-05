//! AETHER Core Types
//!
//! Core blockchain types and primitives for the Aether protocol.

pub mod proof_engine;
pub mod trust_score;

pub use proof_engine::*;
pub use trust_score::*;

// Re-export from aether-common for convenience
pub use aether_common::{SignatureBytes as Signature, ValidatorTier, AIPriorityLane, AITransactionMeta};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ============================================================================
// Type Aliases (aligned with aether-common)
// ============================================================================

/// Account address (public key) - 32 bytes
pub type Address = [u8; 32];

/// Hash value - 32 bytes  
pub type Hash = [u8; 32];

// ============================================================================
// Transaction Type System
// ============================================================================

/// Transaction type enum for Aether transactions
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
}

/// Unified transaction payload for all Aether transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionPayload {
    Transfer {
        recipient: Address,
        amount: u64,
        nonce: u64,
    },
    Stake {
        validator: Address,
        amount: u64,
        tier: String, // "full", "lite", "observer"
    },
    Unstake {
        position_index: usize,
        amount: u64,
    },
    ClaimRewards {
        position_index: usize,
    },
    CreateNFT {
        metadata_uri: String,
        supply: u64,
        name: String,
    },
    MintNFT {
        nft_id: [u8; 32],
        amount: u64,
    },
    TransferNFT {
        nft_id: [u8; 32],
        recipient: Address,
    },
    UpdateMetadata {
        nft_id: [u8; 32],
        metadata_uri: String,
    },
    Delegate {
        validator: Address,
        amount: u64,
    },
    Vote {
        slot: u64,
        block_hash: Hash,
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

/// AETH Transaction (full type with typed payloads)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherTransaction {
    /// Transaction signature (64 bytes)
    pub signature: Signature,
    /// Signer public key
    pub signer: Address,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Transaction payload
    pub payload: TransactionPayload,
    /// Transaction fee (in lamports)
    pub fee: u64,
    /// Slot at which this was included
    pub slot: u64,
    /// Unix timestamp
    pub timestamp: u64,
}

impl AetherTransaction {
    /// Create a new AetherTransaction
    pub fn new(
        signature: Signature,
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

/// Execution result for a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether the transaction succeeded
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// State changes caused by this transaction
    pub state_changes: Vec<StateChange>,
    /// Gas/compute units used
    pub gas_used: u64,
}

impl ExecutionResult {
    /// Create a successful execution result with no state changes
    pub fn success() -> Self {
        Self {
            success: true,
            error_message: None,
            state_changes: vec![],
            gas_used: 0,
        }
    }

    /// Create a successful execution result with state changes and gas
    pub fn success_with(state_changes: Vec<StateChange>, gas_used: u64) -> Self {
        Self {
            success: true,
            error_message: None,
            state_changes,
            gas_used,
        }
    }

    /// Create a failed execution result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            error_message: Some(error),
            state_changes: vec![],
            gas_used: 0,
        }
    }

    /// Create a failed execution result with gas used
    pub fn failure_with(error: String, gas_used: u64) -> Self {
        Self {
            success: false,
            error_message: Some(error),
            state_changes: vec![],
            gas_used,
        }
    }
}

/// A state change caused by transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// Account that was modified
    pub account: Address,
    /// Field that was changed ("lamports", "data", "owner", etc.)
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

/// Transaction receipt — immutable on-chain record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction signature
    pub signature: Signature,
    /// Slot number
    pub slot: u64,
    /// Block hash this was included in
    pub block_hash: Hash,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Signer address
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
        hasher.update(&self.signature.0);
        hasher.update(self.slot.to_le_bytes());
        hasher.update(&self.block_hash);
        hasher.update(serde_json::to_vec(&self.tx_type).unwrap_or_default());
        hasher.update(&self.signer);
        hasher.update(serde_json::to_vec(&self.result).unwrap_or_default());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.finalize().into()
    }
}

// ============================================================================
// Account Type (aligned with state_db.rs)
// ============================================================================

/// A single account in the state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account address (public key)
    pub address: Address,
    /// Lamports (1 AETH = 100_000_000 lamports)
    pub lamports: u64,
    /// Owner program (zero address = system program)
    pub owner: Address,
    /// Arbitrary account data
    pub data: Vec<u8>,
    /// Last rent epoch
    pub rent_epoch: u64,
    /// Whether this account exists (deleted accounts are marked)
    pub exists: bool,
}

impl Account {
    /// Create a new account with default values
    pub fn new(address: Address, lamports: u64) -> Self {
        Self {
            address,
            lamports,
            owner: [0u8; 32], // System program
            data: Vec::new(),
            rent_epoch: 0,
            exists: true,
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
        }
    }
}

// ============================================================================
// Genesis Account (for initialization)
// ============================================================================

/// Genesis account for initialization
#[derive(Debug, Clone)]
pub struct GenesisAccount {
    pub address: Address,
    pub lamports: u64,
    pub data: Option<Vec<u8>>,
}

// ============================================================================
// Serialization Helpers
// ============================================================================

impl TransactionType {
    /// Serialize to bytes
    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}
