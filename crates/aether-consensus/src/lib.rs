//! AetherFlow Consensus Module
//!
//! Implements the modified Proof of History + Proof of Stake consensus
//! with AI transaction priority lanes.
//!
//! # AetherFlow Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    AetherFlow Consensus                     │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Layer 1: Proof of History (PoH) - Verifiable Delay Function │
//! │  Layer 2: Proof of Stake (PoS) - Stake-weighted leader election
//! │  Layer 3: AI Priority Queue - 3-tier transaction lanes     │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Slot Time: 400ms                                          │
//! │  Block Time: 400-800ms (variable based on load)             │
//! │  Finality: ~12-16 slots (4.8-6.4s)                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod poh;
pub mod pos;
pub mod aetherflow;
pub mod validator;
pub mod fork_choice;
pub mod tower;
pub mod staking;

#[cfg(test)]
mod tests;

pub use poh::*;
pub use pos::*;
pub use aetherflow::*;
pub use validator::*;
pub use fork_choice::*;
pub use tower::*;

use thiserror::Error;

/// Consensus errors
#[derive(Error, Debug, Clone)]
pub enum ConsensusError {
    #[error("Invalid proof of history: {0}")]
    InvalidPoH(String),
    
    #[error("Invalid proof of stake: {0}")]
    InvalidPoS(String),
    
    #[error("Validator not found: {0:?}")]
    ValidatorNotFound([u8; 32]),
    
    #[error("Insufficient stake: required {required}, got {actual}")]
    InsufficientStake { required: u64, actual: u64 },
    
    #[error("Invalid block producer: expected {expected:?}, got {actual:?}")]
    InvalidBlockProducer { expected: [u8; 32], actual: [u8; 32] },
    
    #[error("Fork choice rule violation: {0}")]
    ForkChoiceViolation(String),
    
    #[error("Tower consensus error: {0}")]
    TowerError(String),
    
    #[error("AI priority error: {0}")]
    AIPriorityError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for consensus operations
pub type ConsensusResult<T> = Result<T, ConsensusError>;
