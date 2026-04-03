//! Aether Tokens Module
//! 
//! Contains FLUX (utility) and ATH (governance) token implementations

pub mod utils;
pub mod flux;
pub mod ath;
pub mod staking;

pub use flux::FluxToken;
pub use ath::AthToken;
pub use staking::StakingContract;
