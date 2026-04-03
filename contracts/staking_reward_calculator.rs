// Staking Reward Calculator - AeTHer Chain
// Calculates staking rewards across different pools and tiers

use serde::{Deserialize, Serialize};

/// Tier info for reward calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierInfo {
    pub name: String,
    pub min_stake: u64,
    pub apy_base: f64,
    pub lock_period_days: u32,
}

/// Pool info for reward calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub pool_id: String,
    pub name: String,
    pub token_symbol: String,
    pub current_tvl: u64,
    pub apy: f64,
    pub is_active: bool,
}

/// Stake position for calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakePositionCalc {
    pub position_id: String,
    pub pool_id: String,
    pub amount: u64,
    pub tier: String,
    pub start_time: u64,
    pub lock_end_time: u64,
    pub accumulated_rewards: u64,
}

/// Reward calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardProjection {
    pub position_id: String,
    pub current_rewards: u64,
    pub daily_rewards: u64,
    pub weekly_rewards: u64,
    pub monthly_rewards: u64,
    pub yearly_rewards: u64,
    pub projected_total: u64,
    pub apy_effective: f64,
    pub tier_bonus: f64,
    pub lock_bonus: f64,
}

/// Calculator for staking rewards
pub struct StakingRewardCalculator;

impl StakingRewardCalculator {
    /// Default tier configurations
    pub fn default_tiers() -> Vec<TierInfo> {
        vec![
            TierInfo {
                name: "Bronze".to_string(),
                min_stake: 1_000,
                apy_base: 0.05,  // 5%
                lock_period_days: 0,
            },
            TierInfo {
                name: "Silver".to_string(),
                min_stake: 10_000,
                apy_base: 0.08,  // 8%
                lock_period_days: 30,
            },
            TierInfo {
                name: "Gold".to_string(),
                min_stake: 50_000,
                apy_base: 0.12,  // 12%
                lock_period_days: 90,
            },
            TierInfo {
                name: "Platinum".to_string(),
                min_stake: 100_000,
                apy_base: 0.18,  // 18%
                lock_period_days: 180,
            },
            TierInfo {
                name: "Diamond".to_string(),
                min_stake: 500_000,
                apy_base: 0.25,  // 25%
                lock_period_days: 365,
            },
        ]
    }

    /// Get tier for a given stake amount
    pub fn get_tier_for_amount(amount: u64, tiers: &[TierInfo]) -> Option<String> {
        let mut best_tier = None;
        for tier in tiers {
            if amount >= tier.min_stake {
                best_tier = Some(tier.name.clone());
            }
        }
        best_tier
    }

    /// Calculate lock bonus based on lock period remaining
    pub fn calculate_lock_bonus(lock_end_time: u64, current_time: u64) -> f64 {
        let seconds_in_day = 86400u64;
        let remaining_days = if lock_end_time > current_time {
            (lock_end_time - current_time) / seconds_in_day
        } else {
            0
        };

        // Lock bonus scales with remaining lock period
        if remaining_days >= 365 {
            0.15  // 15% bonus for 1+ year lock
        } else if remaining_days >= 180 {
            0.10  // 10% bonus for 6+ months
        } else if remaining_days >= 90 {
            0.05  // 5% bonus for 3+ months
        } else if remaining_days >= 30 {
            0.02  // 2% bonus for 1+ month
        } else {
            0.0   // No bonus for no lock
        }
    }

    /// Calculate projected rewards for a stake position
    pub fn calculate_rewards(
        position: &StakePositionCalc,
        pool_apy: f64,
        tier_bonus: f64,
        current_time: u64,
    ) -> RewardProjection {
        // Base daily rate
        let daily_rate = pool_apy / 365.0;
        
        // Apply tier bonus
        let effective_daily_rate = daily_rate * (1.0 + tier_bonus);
        
        // Apply lock bonus
        let lock_bonus = Self::calculate_lock_bonus(position.lock_end_time, current_time);
        let final_daily_rate = effective_daily_rate * (1.0 + lock_bonus);
        
        // Calculate rewards
        let amount_float = position.amount as f64;
        let daily_rewards = (amount_float * final_daily_rate) as u64;
        let weekly_rewards = daily_rewards * 7;
        let monthly_rewards = daily_rewards * 30;
        let yearly_rewards = daily_rewards * 365;
        
        let effective_apy = final_daily_rate * 365.0;
        
        RewardProjection {
            position_id: position.position_id.clone(),
            current_rewards: position.accumulated_rewards,
            daily_rewards,
            weekly_rewards,
            monthly_rewards,
            yearly_rewards,
            projected_total: position.accumulated_rewards + yearly_rewards,
            apy_effective: effective_apy,
            tier_bonus,
            lock_bonus,
        }
    }

    /// Calculate compound rewards over time
    pub fn calculate_compound_rewards(
        principal: u64,
        apy: f64,
        days: u32,
        compounding_frequency: u32,
    ) -> u64 {
        if compounding_frequency == 0 {
            return principal;
        }
        
        let n = compounding_frequency as f64;
        let t = days as f64 / 365.0;
        let r = apy;
        
        // A = P(1 + r/n)^(nt)
        let compound_factor = (1.0 + r / n).powf(n * t);
        let final_amount = principal as f64 * compound_factor;
        
        (final_amount - principal as f64) as u64
    }

    /// Find optimal staking pool for given amount
    pub fn find_optimal_pool(
        amount: u64,
        pools: &[PoolInfo],
        tiers: &[TierInfo],
    ) -> Option<(String, String)> {
        let mut best_option: Option<(String, String)> = None;
        let mut highest_apy = 0.0;
        
        for pool in pools {
            if !pool.is_active {
                continue;
            }
            
            let tier_name = Self::get_tier_for_amount(amount, tiers)?;
            let tier_info = tiers.iter().find(|t| t.name == tier_name)?;
            let effective_apy = pool.apy * (1.0 + tier_info.apy_base);
            
            if effective_apy > highest_apy {
                highest_apy = effective_apy;
                best_option = Some((pool.pool_id.clone(), tier_name));
            }
        }
        
        best_option
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_assignment() {
        let tiers = StakingRewardCalculator::default_tiers();
        assert_eq!(StakingRewardCalculator::get_tier_for_amount(5_000, &tiers), Some("Bronze".to_string()));
        assert_eq!(StakingRewardCalculator::get_tier_for_amount(50_000, &tiers), Some("Gold".to_string()));
        assert_eq!(StakingRewardCalculator::get_tier_for_amount(500_000, &tiers), Some("Diamond".to_string()));
    }

    #[test]
    fn test_lock_bonus() {
        let current = 1000000000u64;
        let lock_1_year = current + (365 * 86400);
        let lock_6_months = current + (180 * 86400);
        let lock_none = current - 1;
        
        assert!((StakingRewardCalculator::calculate_lock_bonus(lock_1_year, current) - 0.15).abs() < 0.001);
        assert!((StakingRewardCalculator::calculate_lock_bonus(lock_6_months, current) - 0.10).abs() < 0.001);
        assert!((StakingRewardCalculator::calculate_lock_bonus(lock_none, current) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compound_calculation() {
        let rewards = StakingRewardCalculator::calculate_compound_rewards(10_000, 0.10, 365, 12);
        assert!(rewards > 0);
    }
}
