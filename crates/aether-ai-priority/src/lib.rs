//! AI Transaction Priority Module
//!
//! Implements the 3-tier AI transaction priority system:
//! - Critical: AI governance, emergency operations
//! - High: AI agent transactions, MEV protection
//! - Standard: Regular user transactions
//!
//! Fee Distribution Model:
//! - Critical lane: 10x base fee (100% to treasury)
//! - High lane: 5x base fee (50% treasury, 50% validators)
//! - Standard lane: base fee (100% to validators)
//!
//! Treasury funds: network development, audits, airdrops, validator subsidies

pub mod priority;
pub mod oracle;
pub mod classifier;
pub mod fees;
pub mod fee_distribution;

#[cfg(test)]
mod tests;

pub use priority::*;
pub use oracle::*;
pub use classifier::*;
pub use fees::*;
pub use fee_distribution::*;

use thiserror::Error;

/// AI priority errors
#[derive(Error, Debug, Clone)]
pub enum AIPriorityError {
    #[error("Invalid AI signature: {0}")]
    InvalidSignature(String),
    
    #[error("Oracle not authorized: {0:?}")]
    UnauthorizedOracle([u8; 32]),
    
    #[error("Classification failed: {0}")]
    ClassificationFailed(String),
    
    #[error("Fee calculation error: {0}")]
    FeeCalculationError(String),
    
    #[error("Priority lane full: {0:?}")]
    LaneFull(AIPriorityLane),
}

/// Result type for AI priority operations
pub type AIPriorityResult<T> = Result<T, AIPriorityError>;

use aether_common::types::AIPriorityLane;
