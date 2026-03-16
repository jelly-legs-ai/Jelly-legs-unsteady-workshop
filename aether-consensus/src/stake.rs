//! Staking operations

/// Stake pool
pub struct StakePool {
    /// Total staked amount
    pub total_stake: u64,
}

impl StakePool {
    /// Create new stake pool
    pub fn new() -> Self {
        Self { total_stake: 0 }
    }
}