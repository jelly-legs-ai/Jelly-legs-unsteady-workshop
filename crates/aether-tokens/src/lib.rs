//! AeTHer Chain Token Contracts (FLUX & AETH)
//! 
//! This module implements the dual-token economy:
//! - FLUX: Utility token for AI agents, mining rewards, and network fees
//! - AETH: Governance token for staking, validation, and DAO voting

pub mod flux;
pub mod aeth;
pub mod error;

pub use flux::FluxToken;
pub use aeth::AethToken;
pub use error::TokenError;

/// Token decimals (matches Solana convention)
pub const TOKEN_DECIMALS: u8 = 9;

/// Initial supply for FLUX (in lamports-like units)
pub const FLUX_INITIAL_SUPPLY: u64 = 1_000_000_000_000_000; // 1B FLUX with 9 decimals

/// Initial supply for AETH (in lamports-like units)
pub const AETH_INITIAL_SUPPLY: u64 = 100_000_000_000_000; // 100M AETH with 9 decimals

/// Maximum supply caps
pub const FLUX_MAX_SUPPLY: u64 = 10_000_000_000_000_000; // 10B FLUX
pub const AETH_MAX_SUPPLY: u64 = 1_000_000_000_000_000; // 1B AETH

/// Mining reward per epoch (FLUX)
pub const MINING_REWARD_PER_EPOCH: u64 = 1_000_000_000; // 1 FLUX per epoch

/// Validator staking minimum
pub const VALIDATOR_MIN_STAKE: u64 = 100_000_000_000; // 100 AETH

/// Staking reward rate (APY in basis points)
pub const STAKING_APY_BASIS_POINTS: u32 = 1250; // 12.5%
