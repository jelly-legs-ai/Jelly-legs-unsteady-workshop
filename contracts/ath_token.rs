// ATH Token Contract - AeTHer Chain Governance Token
// stub implementation for ATH token used in staking/governance

use serde::{Deserialize, Serialize};

/// ATH Token Configuration
pub const ATH_TOKEN_NAME: &str = "Aether";
pub const ATH_TOKEN_SYMBOL: &str = "ATH";
pub const ATH_TOKEN_DECIMALS: u8 = 18;
pub const ATH_MAX_SUPPLY: u64 = 1_000_000_000_u64; // 1 billion ATH

/// ATH Token State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthToken {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub reserve_pool: u64,
    pub governance_treasury: u64,
}

/// Initialize ATH token
pub fn init_ath_token() -> AthToken {
    AthToken {
        total_supply: ATH_MAX_SUPPLY,
        circulating_supply: 0,
        reserve_pool: ATH_MAX_SUPPLY / 5,       // 20% reserved
        governance_treasury: ATH_MAX_SUPPLY / 5, // 20% to treasury
    }
}

/// Calculate staking rewards based on amount and duration
pub fn calculate_staking_reward(
    staked_amount: u64,
    lock_days: u64,
    network_tier: NetworkTier,
) -> u64 {
    let base_apy = match network_tier {
        NetworkTier::Bronze => 0.05,   // 5% APY
        NetworkTier::Silver => 0.08,   // 8% APY
        NetworkTier::Gold => 0.12,      // 12% APY
        NetworkTier::Platinum => 0.15,  // 15% APY
    };
    
    // Duration bonus: extra APY for longer locks
    let duration_multiplier = (lock_days as f64 / 365.0).min(2.0);
    
    (staked_amount as f64 * base_apy * duration_multiplier) as u64
}

/// Network tier for staking calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

/// Staking position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    pub owner: String,
    pub amount: u64,
    pub start_epoch: u64,
    pub lock_end_epoch: u64,
    pub tier: NetworkTier,
    pub pending_rewards: u64,
}

/// Create a new staking position
pub fn create_staking_position(
    owner: String,
    amount: u64,
    start_epoch: u64,
    lock_days: u64,
    tier: NetworkTier,
) -> StakingPosition {
    StakingPosition {
        owner,
        amount,
        start_epoch,
        lock_end_epoch: start_epoch + (lock_days * 24), // epochs are hourly
        tier,
        pending_rewards: 0,
    }
}

/// Check if a position can be unlocked
pub fn can_unlock(position: &StakingPosition, current_epoch: u64) -> bool {
    current_epoch >= position.lock_end_epoch
}

/// Calculate pending rewards for a staking position
pub fn calculate_pending_rewards(position: &StakingPosition, current_epoch: u64) -> u64 {
    if current_epoch < position.start_epoch {
        return 0;
    }
    
    let epochs_staked = current_epoch - position.start_epoch;
    calculate_staking_reward(position.amount, epochs_staked / 24, position.tier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ath_token_init() {
        let token = init_ath_token();
        assert_eq!(token.total_supply, ATH_MAX_SUPPLY);
        assert_eq!(token.reserve_pool, ATH_MAX_SUPPLY / 5);
    }

    #[test]
    fn test_bronze_staking_reward() {
        let reward = calculate_staking_reward(1000, 365, NetworkTier::Bronze);
        assert_eq!(reward, 50); // 5% of 1000
    }

    #[test]
    fn test_platinum_staking_reward() {
        let reward = calculate_staking_reward(1000, 365, NetworkTier::Platinum);
        assert_eq!(reward, 150); // 15% of 1000
    }

    #[test]
    fn test_staking_position_creation() {
        let position = create_staking_position(
            "0x1234".to_string(),
            1000,
            100,
            30,
            NetworkTier::Silver,
        );
        assert_eq!(position.amount, 1000);
        assert!(!can_unlock(&position, 101));
    }
}
