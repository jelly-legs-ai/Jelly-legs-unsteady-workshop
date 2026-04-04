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

    /// Calculate rewards across multiple positions (portfolio view)
    pub fn calculate_portfolio_rewards(
        positions: &[StakePositionCalc],
        pools: &[PoolInfo],
        current_time: u64,
    ) -> Vec<RewardProjection> {
        let tiers = Self::default_tiers();
        let mut projections = Vec::new();

        for position in positions {
            if let Some(pool) = pools.iter().find(|p| p.pool_id == position.pool_id) {
                let tier_info = tiers.iter().find(|t| t.name == position.tier);
                let tier_bonus = tier_info.map(|t| t.apy_base).unwrap_or(0.0);
                let projection = Self::calculate_rewards(position, pool.apy, tier_bonus, current_time);
                projections.push(projection);
            }
        }

        projections
    }

    /// Summary of entire staking portfolio
    pub fn calculate_portfolio_summary(
        positions: &[StakePositionCalc],
        pools: &[PoolInfo],
        current_time: u64,
    ) -> PortfolioSummary {
        let projections = Self::calculate_portfolio_rewards(positions, pools, current_time);
        let total_staked: u64 = positions.iter().map(|p| p.amount).sum();
        let total_accrued: u64 = projections.iter().map(|p| p.current_rewards).sum();
        let total_daily: u64 = projections.iter().map(|p| p.daily_rewards).sum();

        PortfolioSummary {
            total_staked,
            total_accrued,
            total_daily,
            position_count: positions.len(),
        }
    }
}

/// Portfolio summary struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub total_staked: u64,
    pub total_accrued: u64,
    pub total_daily: u64,
    pub position_count: usize,
}

/// Auto-compounding configuration and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundConfig {
    pub enabled: bool,
    pub frequency_hours: u32,      // How often to compound (e.g., 24 = daily)
    pub reinvest_percentage: f64, // % of rewards to reinvest (0.0 to 1.0)
    pub min_reinvest_amount: u64,   // Minimum reward before reinvest triggers
    pub compound_until_epoch: u64, // Stop auto-compound at this epoch (0 = no limit)
}

impl Default for AutoCompoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency_hours: 24,
            reinvest_percentage: 0.80, // 80% auto-compound
            min_reinvest_amount: 100,  // Min 100 units to trigger
            compound_until_epoch: 0,
        }
    }
}

/// Auto-compounding state for a position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundState {
    pub position_id: String,
    pub last_compound_epoch: u64,
    pub total_compounds: u32,
    pub total_reinvested: u64,
    pub compound_countdown: u32,  // Epochs until next compound
}

impl StakingRewardCalculator {
    /// Calculate optimal compound frequency for maximum APY
    pub fn calculate_optimal_compound_frequency(
        amount: u64,
        apy: f64,
        lock_period_days: u32,
    ) -> u32 {
        if lock_period_days == 0 {
            // No lock - compound daily for best results
            return 24;
        }
        
        // For locked positions, less frequent compounding may be optimal
        // due to lock-up constraints. Weekly is usually good.
        if lock_period_days < 30 {
            return 24; // Daily
        } else if lock_period_days < 90 {
            return 72; // Every 3 days
        } else {
            return 168; // Weekly (24 * 7)
        }
    }
    
    /// Simulate auto-compounding over time and return projection
    pub fn simulate_auto_compound(
        principal: u64,
        apy: f64,
        days: u32,
        compound_frequency_hours: u32,
        reinvest_percentage: f64,
    ) -> AutoCompoundResult {
        let compounds_per_day = 24 / compound_frequency_hours;
        let total_compounds = (compounds_per_day * days) as u64;
        
        let n = compounds_per_day as f64;
        let t = days as f64 / 365.0;
        let r = apy * reinvest_percentage; // Only the reinvested portion compounds
        
        // Calculate compound growth on reinvested portion
        let reinvest_amount = (principal as f64 * reinvest_percentage);
        let compounded_growth = (1.0 + r / n).powf(n * t);
        let final_from_compound = reinvest_amount * compounded_growth;
        
        // Non-reinvested portion goes to wallet
        let non_reinvest = principal as f64 * (1.0 - reinvest_percentage);
        
        // Total final amount
        let total_final = non_reinvest + final_from_compound;
        let total_rewards = total_final - principal as f64;
        
        AutoCompoundResult {
            initial_stake: principal,
            final_amount: total_final as u64,
            total_rewards: total_rewards as u64,
            total_compounds,
            effective_apy: (total_rewards as f64 / principal as f64) * 365.0 / days as f64 * 100.0,
            compound_frequency_hours,
            reinvest_percentage,
        }
    }
    
    /// Process an auto-compound event for a position
    pub fn process_auto_compound(
        position: &mut StakePositionCalc,
        config: &AutoCompoundConfig,
        current_epoch: u64,
        pool_apy: f64,
        tier_bonus: f64,
    ) -> Option<AutoCompoundResultDetail> {
        if !config.enabled {
            return None;
        }
        
        // Check if we've reached the compound limit
        if config.compound_until_epoch > 0 && current_epoch >= config.compound_until_epoch {
            return None;
        }
        
        // Calculate pending rewards since last compound
        let epoch_hours = 1u64; // Assuming 1 hour per epoch
        let hours_since_last = (current_epoch - position.start_time) % config.frequency_hours as u64;
        
        if hours_since_last < epoch_hours {
            return None; // Not time to compound yet
        }
        
        // Create a temporary position for reward calculation
        let temp_position = StakePositionCalc {
            position_id: position.position_id.clone(),
            pool_id: position.pool_id.clone(),
            amount: position.amount,
            tier: position.tier.clone(),
            start_time: position.start_time,
            lock_end_time: position.lock_end_time,
            accumulated_rewards: position.accumulated_rewards,
        };
        
        // Calculate rewards
        let projection = Self::calculate_rewards(
            &temp_position,
            pool_apy,
            tier_bonus,
            current_epoch,
        );
        
        let rewards_to_reinvest = (projection.daily_rewards as f64 * config.reinvest_percentage) as u64;
        
        // Check minimum threshold
        if rewards_to_reinvest < config.min_reinvest_amount {
            return None;
        }
        
        // Compound the rewards
        position.amount += rewards_to_reinvest;
        position.accumulated_rewards += rewards_to_reinvest;
        
        Some(AutoCompoundResultDetail {
            position_id: position.position_id.clone(),
            epoch: current_epoch,
            rewards_reinvested: rewards_to_reinvest,
            new_stake_amount: position.amount,
            total_compounds: 1,
        })
    }
    
    /// Calculate compound APY boost from auto-compounding
    pub fn calculate_compound_boost(
        base_apy: f64,
        reinvest_percentage: f64,
        compounds_per_year: u32,
    ) -> f64 {
        // Formula: A = P(1 + r/n)^(n*t) - P
        // Effective APY with compounding = (1 + r_pct/n)^n - 1
        let n = compounds_per_year as f64;
        let r = base_apy * reinvest_percentage;
        let compound_factor = (1.0 + r / n).powf(n) - 1.0;
        let simple_interest = base_apy * (1.0 - reinvest_percentage);
        base_apy + simple_interest + compound_factor * reinvest_percentage
    }
    
    /// Generate a compound schedule for a position
    pub fn generate_compound_schedule(
        position: &StakePositionCalc,
        config: &AutoCompoundConfig,
        end_epoch: u64,
    ) -> Vec<CompoundScheduleEntry> {
        let mut schedule = Vec::new();
        let mut current_epoch = position.start_time + config.frequency_hours as u64;
        let mut total_reinvested: u64 = 0;
        
        while current_epoch < end_epoch && (config.compound_until_epoch == 0 || current_epoch < config.compound_until_epoch) {
            schedule.push(CompoundScheduleEntry {
                epoch: current_epoch,
                projected_rewards: position.accumulated_rewards / 10, // Rough estimate
            });
            current_epoch += config.frequency_hours as u64;
        }
        
        schedule
    }
}

/// Result of auto-compound simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundResult {
    pub initial_stake: u64,
    pub final_amount: u64,
    pub total_rewards: u64,
    pub total_compounds: u64,
    pub effective_apy: f64,
    pub compound_frequency_hours: u32,
    pub reinvest_percentage: f64,
}

/// Detail of a processed compound event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundResultDetail {
    pub position_id: String,
    pub epoch: u64,
    pub rewards_reinvested: u64,
    pub new_stake_amount: u64,
    pub total_compounds: u32,
}

/// Entry in a compound schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundScheduleEntry {
    pub epoch: u64,
    pub projected_rewards: u64,
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

    #[test]
    fn test_multi_pool_calculation() {
        let pools = vec![
            PoolInfo {
                pool_id: "flux".to_string(),
                name: "FLUX Staking".to_string(),
                token_symbol: "FLUX".to_string(),
                current_tvl: 5_000_000,
                apy: 0.14,
                is_active: true,
            },
            PoolInfo {
                pool_id: "ath".to_string(),
                name: "ATH Staking".to_string(),
                token_symbol: "ATH".to_string(),
                current_tvl: 3_000_000,
                apy: 0.12,
                is_active: true,
            },
        ];

        let positions = vec![
            StakePositionCalc {
                position_id: "pos1".to_string(),
                pool_id: "flux".to_string(),
                amount: 50_000,
                tier: "Gold".to_string(),
                start_time: 1000000000,
                lock_end_time: 1000000000 + (90 * 86400),
                accumulated_rewards: 500,
            },
        ];

        let result = Self::calculate_portfolio_rewards(&positions, &pools, 1000000000);
        assert!(!result.is_empty());
    }
    
    #[test]
    fn test_auto_compound_simulation() {
        let result = StakingRewardCalculator::simulate_auto_compound(
            10_000,    // principal
            0.12,      // 12% APY
            365,       // 1 year
            24,        // daily compounds
            0.80,      // 80% reinvest
        );
        
        assert!(result.final_amount > result.initial_stake);
        assert!(result.total_rewards > 0);
        assert_eq!(result.compound_frequency_hours, 24);
    }
    
    #[test]
    fn test_compound_boost_calculation() {
        let base_apy = 0.12; // 12%
        let boost = StakingRewardCalculator::calculate_compound_boost(
            base_apy,
            0.80,      // 80% reinvest
            365,       // daily compounds
        );
        
        // With 80% reinvest and daily compounding, effective APY should be higher
        assert!(boost > base_apy);
    }
    
    #[test]
    fn test_optimal_compound_frequency() {
        // No lock - daily should be optimal
        let freq_no_lock = StakingRewardCalculator::calculate_optimal_compound_frequency(
            10_000, 0.12, 0
        );
        assert_eq!(freq_no_lock, 24);
        
        // Short lock - still daily
        let freq_short = StakingRewardCalculator::calculate_optimal_compound_frequency(
            10_000, 0.12, 15
        );
        assert_eq!(freq_short, 24);
        
        // Medium lock - every 3 days
        let freq_medium = StakingRewardCalculator::calculate_optimal_compound_frequency(
            10_000, 0.12, 60
        );
        assert_eq!(freq_medium, 72);
        
        // Long lock - weekly
        let freq_long = StakingRewardCalculator::calculate_optimal_compound_frequency(
            10_000, 0.12, 180
        );
        assert_eq!(freq_long, 168);
    }
}
