//! Core types for AETHER blockchain

use serde::{Deserialize, Serialize};

/// AETHER block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block height
    pub height: u64,
    /// Previous block hash
    pub prev_hash: Hash,
    /// Timestamp
    pub timestamp: u64,
    /// PoH hash
    pub poh_hash: Hash,
    /// State root
    pub state_root: Hash,
}

/// AETHER transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction signature
    pub signature: Signature,
    /// Sender address
    pub from: Address,
    /// Recipient address
    pub to: Address,
    /// Amount in lamports
    pub amount: u64,
    /// Transaction data
    pub data: Vec<u8>,
    /// AI priority score (0-100)
    pub priority_score: u8,
}

/// Hash type
pub type Hash = [u8; 32];

/// Signature type
pub type Signature = [u8; 64];

/// Address type
pub type Address = [u8; 32];

/// Block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// Transactions in block
    pub transactions: Vec<Transaction>,
}

/// Validator info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator address
    pub address: Address,
    /// Stake amount
    pub stake: u64,
    /// Commission rate (0-10000, representing 0-100%)
    pub commission: u16,
}