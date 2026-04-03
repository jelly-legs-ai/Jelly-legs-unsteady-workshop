// Unified Rewards Bridge - AeTHer Chain
// Bridges FLUX mining rewards with staking rewards for combined APY calculations
// Enables cross-rewards optimization and combined portfolio views

use serde::{Deserialize, Serialize};
use crate::staking_reward_calculator::{StakingRewardCalculator, StakePositionCalc, TierInfo, PoolInfo};
use crate::mining_rewards::{Miner, MiningRewardConfig, DeviceTier};

/// Unified rewards position combining staking and mining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPosition {
    pub position_id: String,
    pub wallet_address: String,
    pub staking_position: Option<StakePositionCalc>,
    pub mining_position: Option<MinerPosition>,
    pub combined_value_usd: f64,
    pub rewards_source: Vec<RewardSource>,
}

/// Miner position for unified view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerPosition {
    pub miner_id: String,
    pub device_tier: DeviceTier,
    pub total_rewards_earned: u64,
    pub pending_rewards: u64,
    pub epochs_mined: u64,
    pub uptime_percentage: f64,
    pub last_claim_epoch: u64,
}

/// Source of rewards in unified position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardSource {
    pub source_type: RewardSourceType,
    pub source_id: String,
    pub daily_rewards: u64,
    pub weekly_rewards: u64,
    pub monthly_rewards: u64,
    pub yearly_rewards: u64,
    pub current_balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RewardSourceType {
    Staking,
    Mining,
    Farming,
    Liquidity,
    Governance,
}

/// Combined rewards summary for a wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRewardsSummary {
    pub wallet_address: String,
    pub total_value_locked: u64,
    pub total_pending_rewards: u64,
    pub total_claimed_rewards: u64,
    pub combined_daily_rewards: u64,
    pub combined_weekly_rewards: u64,
    pub combined_monthly_rewards: u64,
    pub combined_yearly_rewards: u64,
    pub effective_combined_apy: f64,
    pub positions_count: usize,
    pub sources_breakdown: Vec<SourceBreakdown>,
    pub optimization_suggestions: Vec<RewardOptimization>,
}

/// Breakdown of rewards by source type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceBreakdown {
    pub source_type: RewardSourceType,
    pub position_count: usize,
    pub total_value: u64,
    pub daily_rewards: u64,
    pub percentage_of_total: f64,
}

/// Optimization suggestion for rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardOptimization {
    pub suggestion_type: OptimizationType,
    pub title: String,
    pub description: String,
    pub potential_gain_daily: u64,
    pub potential_gain_yearly: u64,
    pub effort: EffortLevel,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationType {
    RestakeRewards,
    UpgradeTier,
    CompoundMining,
    DiversifyPortfolio,
    ClaimPendingRewards,
    MigrateToBetterPool,
    LockForBonus,
    SwitchDeviceTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
}

/// Bridge between staking and mining reward systems
pub struct UnifiedRewardsBridge;

impl UnifiedRewardsBridge {
    /// Calculate combined APY from multiple reward sources
    pub fn calculate_combined_apy(
        staking_positions: &[StakePositionCalc],
        mining_positions: &[Miner],
        staking_pool_apy: f64,
        mining_config: &MiningRewardConfig,
        current_time: u64,
    ) -> f64 {
        let mut total_value: f64 = 0.0;
        let mut total_daily_rewards: f64 = 0.0;

        // Calculate staking rewards
        for pos in staking_positions {
            let tier_bonus = StakingRewardCalculator::get_tier_for_amount(pos.amount, &StakingRewardCalculator::default_tiers())
                .map(|t| {
                    StakingRewardCalculator::default_tiers()
                        .iter()
                        .find(|ti| ti.name == t)
                        .map(|ti| ti.apy_base)
                        .unwrap_or(0.0)
                })
                .unwrap_or(0.0);
            
            let daily_rate = staking_pool_apy / 365.0 * (1.0 + tier_bonus);
            let daily_rewards = pos.amount as f64 * daily_rate;
            
            total_value += pos.amount as f64;
            total_daily_rewards += daily_rewards;
        }

        // Calculate mining rewards (daily estimate)
        for miner in mining_positions {
            if !miner.is_active {
                continue;
            }
            
            let device_mult = miner.device_tier.multiplier();
            let base_daily = mining_config.base_reward_per_epoch as f64 * 24.0 * device_mult;
            let uptime_factor = miner.uptime_percentage.min(1.0);
            let contribution_factor = mining_config.contribution_factor;
            
            let daily_rewards = base_daily * uptime_factor * contribution_factor;
            
            // Estimate device value (rough approximation)
            let device_value = match miner.device_tier {
                DeviceTier::Mobile => 500.0,
                DeviceTier::Laptop => 1500.0,
                DeviceTier::Desktop => 3000.0,
                DeviceTier::Server => 10000.0,
            };
            
            total_value += device_value;
            total_daily_rewards += daily_rewards;
        }

        if total_value <= 0.0 {
            return 0.0;
        }

        // Annualize daily rate
        (total_daily_rewards / total_value) * 365.0
    }

    /// Generate unified rewards summary for a wallet
    pub fn generate_unified_summary(
        wallet_address: &str,
        staking_positions: Vec<StakePositionCalc>,
        mining_positions: Vec<Miner>,
        staking_pools: &[PoolInfo],
        mining_config: &MiningRewardConfig,
        current_time: u64,
    ) -> UnifiedRewardsSummary {
        let mut total_tvl: u64 = 0;
        let mut total_pending: u64 = 0;
        let mut total_claimed: u64 = 0;
        let mut daily_total: u64 = 0;
        let mut weekly_total: u64 = 0;
        let mut monthly_total: u64 = 0;
        let mut yearly_total: u64 = 0;

        let mut staking_daily: u64 = 0;
        let mut mining_daily: u64 = 0;
        let mut staking_count = 0;
        let mut mining_count = 0;

        // Process staking positions
        for pos in &staking_positions {
            total_tvl += pos.amount;
            total_pending += pos.accumulated_rewards;
            total_claimed += pos.accumulated_rewards; // Simplified
            
            let pool = staking_pools.iter().find(|p| p.pool_id == pos.pool_id);
            let pool_apy = pool.map(|p| p.apy).unwrap_or(0.12);
            
            let tier_name = StakingRewardCalculator::get_tier_for_amount(pos.amount, &StakingRewardCalculator::default_tiers());
            let tier_bonus = tier_name
                .and_then(|t| StakingRewardCalculator::default_tiers().iter().find(|ti| ti.name == t))
                .map(|ti| ti.apy_base)
                .unwrap_or(0.0);
            
            let daily_rate = (pool_apy / 365.0) * (1.0 + tier_bonus);
            let daily = (pos.amount as f64 * daily_rate) as u64;
            
            staking_daily += daily;
            daily_total += daily;
            weekly_total += daily * 7;
            monthly_total += daily * 30;
            yearly_total += daily * 365;
            staking_count += 1;
        }

        // Process mining positions
        for miner in &mining_positions {
            if !miner.is_active {
                continue;
            }

            total_pending += miner.pending_rewards;
            total_claimed += miner.total_rewards_earned;

            let device_mult = miner.device_tier.multiplier();
            let base_epoch = mining_config.base_reward_per_epoch as f64;
            let uptime_factor = miner.uptime_percentage.min(1.0);
            
            let daily = (base_epoch * 24.0 * device_mult * uptime_factor * mining_config.contribution_factor) as u64;
            
            mining_daily += daily;
            daily_total += daily;
            weekly_total += daily * 7;
            monthly_total += daily * 30;
            yearly_total += daily * 365;
            mining_count += 1;
        }

        // Calculate effective combined APY
        let effective_apy = if total_tvl > 0 {
            (daily_total as f64 / total_tvl as f64) * 365.0
        } else {
            0.0
        };

        // Build source breakdown
        let mut sources_breakdown = Vec::new();
        
        if staking_count > 0 {
            sources_breakdown.push(SourceBreakdown {
                source_type: RewardSourceType::Staking,
                position_count: staking_count,
                total_value: total_tvl,
                daily_rewards: staking_daily,
                percentage_of_total: if daily_total > 0 { (staking_daily as f64 / daily_total as f64) * 100.0 } else { 0.0 },
            });
        }
        
        if mining_count > 0 {
            sources_breakdown.push(SourceBreakdown {
                source_type: RewardSourceType::Mining,
                position_count: mining_count,
                total_value: 0, // Mining doesn't lock value like staking
                daily_rewards: mining_daily,
                percentage_of_total: if daily_total > 0 { (mining_daily as f64 / daily_total as f64) * 100.0 } else { 0.0 },
            });
        }

        // Generate optimization suggestions
        let optimization_suggestions = Self::generate_optimizations(
            &staking_positions,
            &mining_positions,
            staking_pools,
            mining_config,
            daily_total,
        );

        UnifiedRewardsSummary {
            wallet_address: wallet_address.to_string(),
            total_value_locked: total_tvl,
            total_pending_rewards: total_pending,
            total_claimed_rewards: total_claimed,
            combined_daily_rewards: daily_total,
            combined_weekly_rewards: weekly_total,
            combined_monthly_rewards: monthly_total,
            combined_yearly_rewards: yearly_total,
            effective_combined_apy: effective_apy,
            positions_count: staking_count + mining_count,
            sources_breakdown,
            optimization_suggestions,
        }
    }

    /// Generate reward optimization suggestions
    fn generate_optimizations(
        staking_positions: &[StakePositionCalc],
        mining_positions: &[Miner],
        staking_pools: &[PoolInfo],
        mining_config: &MiningRewardConfig,
        current_daily: u64,
    ) -> Vec<RewardOptimization> {
        let mut suggestions = Vec::new();

        // Suggest claiming if pending rewards are significant
        let total_pending: u64 = staking_positions.iter().map(|p| p.accumulated_rewards).sum();
        let mining_pending: u64 = mining_positions.iter().map(|m| m.pending_rewards).sum();
        let total_pending_all = total_pending + mining_pending;

        if total_pending_all > 100_000_000 { // > 1 FLUX
            suggestions.push(RewardOptimization {
                suggestion_type: OptimizationType::ClaimPendingRewards,
                title: "Claim Pending Rewards".to_string(),
                description: format!("You have {} FLUX in pending rewards. Claiming and restaking can accelerate compounding.", 
                    total_pending_all as f64 / 100_000_000.0),
                potential_gain_daily: total_pending_all / 30, // Rough daily additional from compounding
                potential_gain_yearly: total_pending_all / 12,
                effort: EffortLevel::Low,
                risk: RiskLevel::None,
            });
        }

        // Suggest tier upgrade if staking significant amount
        let total_staked: u64 = staking_positions.iter().map(|p| p.amount).sum();
        if total_staked >= 10_000 && total_staked < 100_000 {
            suggestions.push(RewardOptimization {
                suggestion_type: OptimizationType::UpgradeTier,
                title: "Upgrade to Gold Tier".to_string(),
                description: "Your stake qualifies for Gold tier (100K+). Gold tier provides 1.5x multiplier vs 1.25x for Silver.".to_string(),
                potential_gain_daily: current_daily * 20 / 100, // ~20% increase
                potential_gain_yearly: current_daily * 20 / 100 * 365,
                effort: EffortLevel::Medium,
                risk: RiskLevel::Low,
            });
        }

        // Suggest device upgrade for miners
        let active_miners: Vec<&Miner> = mining_positions.iter().filter(|m| m.is_active).collect();
        if !active_miners.is_empty() {
            let has_mobile = active_miners.iter().any(|m| m.device_tier == DeviceTier::Mobile);
            if has_mobile {
                suggestions.push(RewardOptimization {
                    suggestion_type: OptimizationType::SwitchDeviceTier,
                    title: "Upgrade Mining Device".to_string(),
                    description: "Mobile mining has 2.5-5x lower yield than Desktop. Consider upgrading for significantly higher rewards.".to_string(),
                    potential_gain_daily: current_daily * 50 / 100, // 50% potential increase
                    potential_gain_yearly: current_daily * 50 / 100 * 365,
                    effort: EffortLevel::High,
                    risk: RiskLevel::Medium,
                });
            }
        }

        // Suggest restaking rewards
        if total_pending_all > 1_000_000_000 { // > 10 FLUX
            suggestions.push(RewardOptimization {
                suggestion_type: OptimizationType::RestakeRewards,
                title: "Automate Reward Restaking".to_string(),
                description: "Enable auto-compound to automatically restake rewards. This can increase yearly yields by 5-15% through compounding.".to_string(),
                potential_gain_daily: current_daily * 10 / 100,
                potential_gain_yearly: current_daily * 10 / 100 * 365,
                effort: EffortLevel::Low,
                risk: RiskLevel::None,
            });
        }

        // Diversification suggestions
        if staking_positions.len() <= 1 && total_staked > 50_000 {
            suggestions.push(RewardOptimization {
                suggestion_type: OptimizationType::DiversifyPortfolio,
                title: "Diversify Staking Positions".to_string(),
                description: "Splitting your stake across multiple validators reduces risk and can improve overall APY through competitive commissions.".to_string(),
                potential_gain_daily: current_daily * 5 / 100,
                potential_gain_yearly: current_daily * 5 / 100 * 365,
                effort: EffortLevel::Medium,
                risk: RiskLevel::Low,
            });
        }

        suggestions.sort_by(|a, b| b.potential_gain_yearly.cmp(&a.potential_gain_yearly));
        suggestions.truncate(5); // Top 5 suggestions
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combined_apy_calculation() {
        let staking_positions = vec![
            StakePositionCalc {
                position_id: "stake_1".to_string(),
                pool_id: "flux".to_string(),
                amount: 50_000,
                tier: "Gold".to_string(),
                start_time: 1000000000,
                lock_end_time: 1000000000 + (90 * 86400),
                accumulated_rewards: 500_000_000,
            },
        ];

        let mining_positions = vec![
            Miner {
                miner_id: "miner_1".to_string(),
                device_tier: DeviceTier::Laptop,
                ram_gb: 16,
                cpu_cores: 8,
                uptime_percentage: 0.95,
                contribution_score: 0.8,
                epochs_mined: 1000,
                total_rewards_earned: 5_000_000_000,
                pending_rewards: 500_000_000,
                last_claim_epoch: 500,
                is_active: true,
            },
        ];

        let mining_config = MiningRewardConfig::default();
        
        let combined_apy = UnifiedRewardsBridge::calculate_combined_apy(
            &staking_positions,
            &mining_positions,
            0.12, // 12% staking pool APY
            &mining_config,
            1000000000,
        );

        assert!(combined_apy > 0.0);
        assert!(combined_apy < 1.0); // Should be a reasonable APY percentage
    }

    #[test]
    fn test_unified_summary_generation() {
        let staking_positions = vec![
            StakePositionCalc {
                position_id: "stake_1".to_string(),
                pool_id: "flux".to_string(),
                amount: 100_000,
                tier: "Gold".to_string(),
                start_time: 1000000000,
                lock_end_time: 1000000000 + (180 * 86400),
                accumulated_rewards: 1_000_000_000,
            },
        ];

        let mining_positions = vec![
            Miner {
                miner_id: "miner_1".to_string(),
                device_tier: DeviceTier::Desktop,
                ram_gb: 32,
                cpu_cores: 16,
                uptime_percentage: 0.98,
                contribution_score: 0.9,
                epochs_mined: 5000,
                total_rewards_earned: 10_000_000_000,
                pending_rewards: 1_000_000_000,
                last_claim_epoch: 2000,
                is_active: true,
            },
        ];

        let pools = vec![
            PoolInfo {
                pool_id: "flux".to_string(),
                name: "FLUX Staking".to_string(),
                token_symbol: "FLUX".to_string(),
                current_tvl: 10_000_000_000,
                apy: 0.14,
                is_active: true,
            },
        ];

        let mining_config = MiningRewardConfig::default();
        
        let summary = UnifiedRewardsBridge::generate_unified_summary(
            "0x1234...abcd",
            staking_positions,
            mining_positions,
            &pools,
            &mining_config,
            1000000000,
        );

        assert_eq!(summary.wallet_address, "0x1234...abcd");
        assert!(summary.total_value_locked > 0);
        assert!(summary.combined_daily_rewards > 0);
        assert!(summary.effective_combined_apy > 0.0);
        assert!(!summary.optimization_suggestions.is_empty() || summary.total_value_locked < 10_000); // No suggestions for small stakes
    }
}
