// FLUX Token Contract - AeTHer Chain
// Mining rewards and utility token for network participation

use serde::{Deserialize, Serialize};

/// FLUX Token Configuration
pub const FLUX_TOKEN_NAME: &str = "FLUX";
pub const FLUX_TOKEN_SYMBOL: &str = "FLUX";
pub const FLUX_TOKEN_DECIMALS: u8 = 18;
pub const FLUX_MAX_SUPPLY: u64 = 10_000_000_000_u64; // 10 billion FLUX

/// FLUX Token State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxToken {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub mining_reward_per_epoch: u64,
    pub last_reward_distribution: u64,
}

/// Initialize FLUX token
pub fn init_flux_token() -> FluxToken {
    FluxToken {
        total_supply: FLUX_MAX_SUPPLY,
        circulating_supply: 0,
        mining_reward_per_epoch: 1000, // 1000 FLUX per epoch
        last_reward_distribution: 0,
    }
}

/// Calculate mining reward based on epoch and device tier
pub fn calculate_mining_reward(
    epoch: u64,
    device_tier: DeviceTier,
    uptime_hours: u64,
    network_participation: f64,
) -> u64 {
    let base_reward = 1000_u64;
    let tier_multiplier = match device_tier {
        DeviceTier::Mobile => 1.0,
        DeviceTier::Laptop => 1.5,
        DeviceTier::Desktop => 2.0,
    };
    let uptime_factor = (uptime_hours as f64 / 24.0).min(1.0);
    let participation_factor = network_participation.min(1.0);
    
    (base_reward as f64 * tier_multiplier * uptime_factor * participation_factor) as u64
}

/// Device tier for mining rewards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceTier {
    Mobile,
    Laptop,
    Desktop,
}

/// Reward distribution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    pub epoch: u64,
    pub recipient: String,
    pub amount: u64,
    pub device_tier: DeviceTier,
    pub timestamp: u64,
}

/// Distribute mining rewards for an epoch
pub fn distribute_epoch_rewards(
    token: &mut FluxToken,
    epoch: u64,
    recipients: Vec<(String, DeviceTier, u64, f64)>, // (address, tier, uptime, participation)
) -> Vec<RewardDistribution> {
    let mut distributions = Vec::new();
    
    for (recipient, tier, uptime, participation) in recipients {
        let reward = calculate_mining_reward(epoch, tier, uptime, participation);
        token.circulating_supply += reward;
        
        distributions.push(RewardDistribution {
            epoch,
            recipient,
            amount: reward,
            device_tier: tier,
            timestamp: epoch * 3600, // epoch duration in seconds
        });
    }
    
    token.last_reward_distribution = epoch;
    distributions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flux_token_init() {
        let token = init_flux_token();
        assert_eq!(token.total_supply, FLUX_MAX_SUPPLY);
        assert_eq!(token.circulating_supply, 0);
    }

    #[test]
    fn test_mobile_mining_reward() {
        let reward = calculate_mining_reward(1, DeviceTier::Mobile, 24, 1.0);
        assert_eq!(reward, 1000);
    }

    #[test]
    fn test_laptop_mining_reward() {
        let reward = calculate_mining_reward(1, DeviceTier::Laptop, 24, 1.0);
        assert_eq!(reward, 1500);
    }

    #[test]
    fn test_desktop_mining_reward() {
        let reward = calculate_mining_reward(1, DeviceTier::Desktop, 24, 1.0);
        assert_eq!(reward, 2000);
    }

    #[test]
    fn test_partial_uptime_reward() {
        let reward = calculate_mining_reward(1, DeviceTier::Mobile, 12, 1.0);
        assert_eq!(reward, 500); // 50% uptime = 50% reward
    }
}
