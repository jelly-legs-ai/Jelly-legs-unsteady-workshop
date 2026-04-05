// Unified Rewards API Routes - AeTHer Chain
// REST API endpoints for combined staking + mining rewards

use serde::{Deserialize, Serialize};
use crate::unified_rewards_bridge::{
    UnifiedRewardsBridge, UnifiedRewardsSummary, UnifiedPosition, 
    RewardSource, RewardSourceType, MinerPosition
};
use crate::staking_reward_calculator::{StakingRewardCalculator, StakePositionCalc, PoolInfo, TierInfo};
use crate::mining_rewards::{MiningRewardConfig, Miner, DeviceTier};

/// API version
pub const UNIFIED_API_VERSION: &str = "v1";

// ============================================================================
// UNIFIED REWARDS ENDPOINTS
// ============================================================================

pub mod unified_rewards {
    pub const SUMMARY: &str = "/unified-rewards/summary/{address}";
    pub const POSITIONS: &str = "/unified-rewards/positions/{address}";
    pub const OPTIMIZE: &str = "/unified-rewards/optimize/{address}";
    pub const PROJECTIONS: &str = "/unified-rewards/projections/{address}";
    pub const COMBINED_APY: &str = "/unified-rewards/combined-apy/{address}";
    pub const CLAIM_ALL: &str = "/unified-rewards/claim-all/{address}";
    pub const HISTORY: &str = "/unified-rewards/history/{address}";
}

// ============================================================================
// CROSS-ASSET REWARDS ENDPOINTS
// ============================================================================

pub mod cross_asset {
    pub const STAKE_FOR_MINING: &str = "/cross-asset/stake-for-mining";
    pub const MINING_TO_STAKING: &str = "/cross-asset/mining-to-staking";
    pub const COMBINED_PORTFOLIO: &str = "/cross-asset/portfolio/{address}";
    pub const REBALANCE: &str = "/cross-asset/rebalance";
}

// ============================================================================
// REWARD AGGREGATION ENDPOINTS
// ============================================================================

pub mod aggregation {
    pub const DAILY_SUMMARY: &str = "/aggregation/daily/{address}";
    pub const WEEKLY_SUMMARY: &str = "/aggregation/weekly/{address}";
    pub const MONTHLY_SUMMARY: &str = "/aggregation/monthly/{address}";
    pub const YEARLY_SUMMARY: &str = "/aggregation/yearly/{address}";
    pub const TAX_REPORT: &str = "/aggregation/tax-report/{address}";
    pub const EXPORT: &str = "/aggregation/export/{address}";
}

// ============================================================================
// REQUEST/RESPONSE STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRewardsSummaryResponse {
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
    pub staking_positions_count: usize,
    pub mining_positions_count: usize,
    pub sources_breakdown: Vec<SourceBreakdownResponse>,
    pub optimization_suggestions: Vec<OptimizationSuggestionResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceBreakdownResponse {
    pub source_type: String,
    pub source_label: String,
    pub position_count: usize,
    pub total_value: u64,
    pub daily_rewards: u64,
    pub percentage_of_total: f64,
    pub icon: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestionResponse {
    pub suggestion_type: String,
    pub suggestion_id: String,
    pub title: String,
    pub description: String,
    pub potential_gain_daily: u64,
    pub potential_gain_weekly: u64,
    pub potential_gain_monthly: u64,
    pub potential_gain_yearly: u64,
    pub effort: String,
    pub effort_label: String,
    pub risk: String,
    pub risk_label: String,
    pub action_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedApyResponse {
    pub wallet_address: String,
    pub staking_apy: f64,
    pub mining_apy: f64,
    pub combined_apy: f64,
    pub weighted_by_value: bool,
    pub breakdown: ApyBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApyBreakdown {
    pub staking_contribution: f64,
    pub mining_contribution: f64,
    pub compounding_bonus: f64,
    pub tier_bonus: f64,
    pub loyalty_bonus: f64,
    pub total_components: Vec<ApyComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApyComponent {
    pub name: String,
    pub value: f64,
    pub is_bonus: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardProjectionsRequest {
    pub projection_days: Option<u32>, // Default 365
    pub include_staking: Option<bool>, // Default true
    pub include_mining: Option<bool>, // Default true
    pub compound_frequency_days: Option<u32>, // Default 1 (daily)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardProjectionResponse {
    pub wallet_address: String,
    pub projections: Vec<DailyProjection>,
    pub tier_progression: Vec<TierProgression>,
    pub milestones: Vec<RewardMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyProjection {
    pub day: u32,
    pub date: String,
    pub total_rewards: u64,
    pub staking_rewards: u64,
    pub mining_rewards: u64,
    pub cumulative_rewards: u64,
    pub total_value_locked: u64,
    pub combined_apy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierProgression {
    pub current_tier: String,
    pub next_tier: String,
    pub amount_needed: u64,
    pub days_to_reach: Option<u32>,
    pub bonus_attainment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardMilestone {
    pub milestone_type: String,
    pub description: String,
    pub reward_amount: u64,
    pub reached_at_day: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimAllRequest {
    pub claim_staking: Option<bool>,
    pub claim_mining: Option<bool>,
    pub restake_percentage: Option<u32>, // 0-100, auto-restake portion
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimAllResponse {
    pub success: bool,
    pub staking_claimed: u64,
    pub mining_claimed: u64,
    pub total_claimed: u64,
    pub restaked_amount: u64,
    pub transaction_hash: Option<String>,
    pub new_tier: Option<String>,
    pub tier_upgraded: bool,
}

// ============================================================================
// MOCK DATA GENERATORS (for demo/testing)
// ============================================================================

pub fn generate_mock_staking_positions(wallet: &str) -> Vec<StakePositionCalc> {
    vec![
        StakePositionCalc {
            position_id: format!("stake_{}_1", &wallet[..8]),
            pool_id: "flux_pool".to_string(),
            amount: 75_000,
            tier: "Silver".to_string(),
            start_time: 1000000000,
            lock_end_time: 1000000000 + (30 * 86400),
            accumulated_rewards: 750_000_000,
        },
        StakePositionCalc {
            position_id: format!("stake_{}_2", &wallet[..8]),
            pool_id: "ath_pool".to_string(),
            amount: 150_000,
            tier: "Gold".to_string(),
            start_time: 1000000000,
            lock_end_time: 1000000000 + (90 * 86400),
            accumulated_rewards: 2_250_000_000,
        },
    ]
}

pub fn generate_mock_mining_positions(wallet: &str) -> Vec<Miner> {
    vec![
        Miner {
            miner_id: format!("miner_{}_1", &wallet[..8]),
            device_tier: DeviceTier::Laptop,
            ram_gb: 16,
            cpu_cores: 8,
            uptime_percentage: 0.97,
            contribution_score: 0.85,
            epochs_mined: 2340,
            total_rewards_earned: 12_500_000_000,
            pending_rewards: 850_000_000,
            last_claim_epoch: 2000,
            is_active: true,
        },
    ]
}

pub fn generate_mock_staking_pools() -> Vec<PoolInfo> {
    vec![
        PoolInfo {
            pool_id: "flux_pool".to_string(),
            name: "FLUX Staking Pool".to_string(),
            token_symbol: "FLUX".to_string(),
            current_tvl: 125_000_000_000_000,
            apy: 0.14,
            is_active: true,
        },
        PoolInfo {
            pool_id: "ath_pool".to_string(),
            name: "ATH Staking Pool".to_string(),
            token_symbol: "ATH".to_string(),
            current_tvl: 89_000_000_000_000,
            apy: 0.12,
            is_active: true,
        },
    ]
}

pub fn generate_mock_mining_config() -> MiningRewardConfig {
    MiningRewardConfig::default()
}

// ============================================================================
// API RESPONSE BUILDERS
// ============================================================================

pub fn build_summary_response(summary: &UnifiedRewardsSummary) -> UnifiedRewardsSummaryResponse {
    let staking_count = summary.sources_breakdown.iter()
        .filter(|s| s.source_type == RewardSourceType::Staking)
        .map(|s| s.position_count)
        .sum();
    
    let mining_count = summary.sources_breakdown.iter()
        .filter(|s| s.source_type == RewardSourceType::Mining)
        .map(|s| s.position_count)
        .sum();

    UnifiedRewardsSummaryResponse {
        wallet_address: summary.wallet_address.clone(),
        total_value_locked: summary.total_value_locked,
        total_pending_rewards: summary.total_pending_rewards,
        total_claimed_rewards: summary.total_claimed_rewards,
        combined_daily_rewards: summary.combined_daily_rewards,
        combined_weekly_rewards: summary.combined_weekly_rewards,
        combined_monthly_rewards: summary.combined_monthly_rewards,
        combined_yearly_rewards: summary.combined_yearly_rewards,
        effective_combined_apy: summary.effective_combined_apy,
        positions_count: summary.positions_count,
        staking_positions_count: staking_count,
        mining_positions_count: mining_count,
        sources_breakdown: summary.sources_breakdown.iter().map(|s| {
            let (label, icon, color) = match s.source_type {
                RewardSourceType::Staking => ("Staking", "🔗", "#5668f5"),
                RewardSourceType::Mining => ("Mining", "⛏️", "#f59e0b"),
                RewardSourceType::Farming => ("Farming", "🌾", "#22c55e"),
                RewardSourceType::Liquidity => ("Liquidity", "💧", "#06b6d4"),
                RewardSourceType::Governance => ("Governance", "🏛️", "#8b5cf6"),
            };
            SourceBreakdownResponse {
                source_type: format!("{:?}", s.source_type),
                source_label: label.to_string(),
                position_count: s.position_count,
                total_value: s.total_value,
                daily_rewards: s.daily_rewards,
                percentage_of_total: s.percentage_of_total,
                icon: icon.to_string(),
                color: color.to_string(),
            }
        }).collect(),
        optimization_suggestions: summary.optimization_suggestions.iter().map(|o| {
            let (effort_label, action_hint) = match o.effort {
                crate::unified_rewards_bridge::EffortLevel::Low => ("One-click", "Click to apply"),
                crate::unified_rewards_bridge::EffortLevel::Medium => ("Setup required", "Configure and apply"),
                crate::unified_rewards_bridge::EffortLevel::High => ("Complex", "Manual action needed"),
            };
            
            let risk_label = match o.risk {
                crate::unified_rewards_bridge::RiskLevel::None => ("No risk", "Safe action"),
                crate::unified_rewards_bridge::RiskLevel::Low => ("Low risk", "Minimal impact"),
                crate::unified_rewards_bridge::RiskLevel::Medium => ("Medium risk", "Moderate changes"),
                crate::unified_rewards_bridge::RiskLevel::High => ("High risk", "Significant changes"),
            };

            OptimizationSuggestionResponse {
                suggestion_type: format!("{:?}", o.suggestion_type),
                suggestion_id: format!("{:?}_{}", o.suggestion_type, &o.title[..3].to_lowercase()),
                title: o.title.clone(),
                description: o.description.clone(),
                potential_gain_daily: o.potential_gain_daily,
                potential_gain_weekly: o.potential_gain_daily * 7,
                potential_gain_monthly: o.potential_gain_daily * 30,
                potential_gain_yearly: o.potential_gain_yearly,
                effort: format!("{:?}", o.effort),
                effort_label: effort_label.to_string(),
                risk: format!("{:?}", o.risk),
                risk_label: risk_label.0.to_string(),
                action_hint: action_hint.to_string(),
            }
        }).collect(),
    }
}

// ============================================================================
// API ROUTER IMPLEMENTATION
// ============================================================================

pub struct UnifiedRewardsRouter;

impl UnifiedRewardsRouter {
    /// Generate combined rewards summary for a wallet
    pub fn get_summary(
        wallet_address: &str,
        staking_positions: Vec<StakePositionCalc>,
        mining_positions: Vec<Miner>,
    ) -> UnifiedRewardsSummaryResponse {
        let pools = generate_mock_staking_pools();
        let mining_config = generate_mock_mining_config();
        let current_time = 1000000000; // Would be actual time in production

        let summary = UnifiedRewardsBridge::generate_unified_summary(
            wallet_address,
            staking_positions,
            mining_positions,
            &pools,
            &mining_config,
            current_time,
        );

        build_summary_response(&summary)
    }

    /// Calculate combined APY for a wallet
    pub fn get_combined_apy(
        wallet_address: &str,
        staking_positions: Vec<StakePositionCalc>,
        mining_positions: Vec<Miner>,
    ) -> CombinedApyResponse {
        let pools = generate_mock_staking_pools();
        let mining_config = generate_mock_mining_config();
        let current_time = 1000000000;

        let staking_apy = if !staking_positions.is_empty() {
            pools.iter()
                .find(|p| p.pool_id == staking_positions[0].pool_id)
                .map(|p| p.apy)
                .unwrap_or(0.12)
        } else {
            0.0
        };

        let combined_apy = UnifiedRewardsBridge::calculate_combined_apy(
            &staking_positions,
            &mining_positions,
            staking_apy,
            &mining_config,
            current_time,
        );

        // Estimate mining APY
        let mining_value: f64 = mining_positions.iter().map(|m| {
            match m.device_tier {
                DeviceTier::Mobile => 500.0,
                DeviceTier::Laptop => 1500.0,
                DeviceTier::Desktop => 3000.0,
                DeviceTier::Server => 10000.0,
            }
        }).sum();

        let mining_daily: f64 = mining_positions.iter().map(|m| {
            if !m.is_active { return 0.0; }
            let mult = m.device_tier.multiplier();
            (mining_config.base_reward_per_epoch as f64 * 24.0 * mult * m.uptime_percentage.min(1.0) * mining_config.contribution_factor) / 100_000_000.0
        }).sum();

        let mining_apy = if mining_value > 0.0 {
            (mining_daily / mining_value) * 365.0
        } else {
            0.0
        };

        CombinedApyResponse {
            wallet_address: wallet_address.to_string(),
            staking_apy,
            mining_apy,
            combined_apy,
            weighted_by_value: true,
            breakdown: ApyBreakdown {
                staking_contribution: staking_apy * 0.7, // Rough estimate
                mining_contribution: mining_apy * 0.2,
                compounding_bonus: 0.02, // ~2% from compounding
                tier_bonus: 0.03, // Tier bonuses average
                loyalty_bonus: 0.01, // Loyalty programs
                total_components: vec![
                    ApyComponent { name: "Base Staking".to_string(), value: staking_apy * 0.6, is_bonus: false },
                    ApyComponent { name: "Pool Rewards".to_string(), value: staking_apy * 0.1, is_bonus: false },
                    ApyComponent { name: "Mining Rewards".to_string(), value: mining_apy, is_bonus: false },
                    ApyComponent { name: "Compounding".to_string(), value: 0.02, is_bonus: true },
                    ApyComponent { name: "Tier Bonus".to_string(), value: 0.03, is_bonus: true },
                    ApyComponent { name: "Loyalty".to_string(), value: 0.01, is_bonus: true },
                ],
            },
        }
    }

    /// Generate reward projections for upcoming days
    pub fn get_projections(
        wallet_address: &str,
        staking_positions: Vec<StakePositionCalc>,
        mining_positions: Vec<Miner>,
        days: u32,
        compound_days: u32,
    ) -> RewardProjectionResponse {
        let pools = generate_mock_staking_pools();
        let mining_config = generate_mock_mining_config();
        let current_time = 1000000000;

        let summary = UnifiedRewardsBridge::generate_unified_summary(
            wallet_address,
            staking_positions.clone(),
            mining_positions.clone(),
            &pools,
            &mining_config,
            current_time,
        );

        let mut projections = Vec::new();
        let mut cumulative = summary.total_claimed_rewards;
        let base_daily = summary.combined_daily_rewards;
        let base_staking = staking_positions.iter().map(|p| {
            let pool = pools.iter().find(|pl| pl.pool_id == p.pool_id);
            let apy = pool.map(|pl| pl.apy).unwrap_or(0.12);
            p.amount as f64 * apy / 365.0
        }).sum::<f64>() as u64;
        
        let base_mining = mining_positions.iter().filter(|m| m.is_active).map(|m| {
            (mining_config.base_reward_per_epoch * 24 * m.device_tier.multiplier() as u64 * (m.uptime_percentage as u64).min(1)) as f64 * mining_config.contribution_factor
        }).sum::<f64>() as u64;

        for day in 1..=days {
            let compounding_factor = if compound_days > 0 && day % compound_days == 0 {
                1.0 + (base_daily as f64 * 0.1 / 100.0) // 0.1% compound bonus
            } else {
                1.0
            };

            let day_rewards = ((base_staking + base_mining) as f64 * compounding_factor) as u64;
            cumulative += day_rewards;

            projections.push(DailyProjection {
                day,
                date: format!("Day {}", day),
                total_rewards: day_rewards,
                staking_rewards: (base_staking as f64 * compounding_factor) as u64,
                mining_rewards: (base_mining as f64 * compounding_factor) as u64,
                cumulative_rewards: cumulative,
                total_value_locked: summary.total_value_locked,
                combined_apy: summary.effective_combined_apy,
            });
        }

        // Calculate tier progression
        let current_stake: u64 = staking_positions.iter().map(|p| p.amount).sum();
        let tiers = StakingRewardCalculator::default_tiers();
        
        let mut tier_progression = Vec::new();
        for tier in &tiers {
            if tier.min_stake > current_stake {
                tier_progression.push(TierProgression {
                    current_tier: tiers.iter().find(|t| t.min_stake <= current_stake).map(|t| t.name.clone()).unwrap_or_default(),
                    next_tier: tier.name.clone(),
                    amount_needed: tier.min_stake - current_stake,
                    days_to_reach: if summary.combined_daily_rewards > 0 {
                        Some(((tier.min_stake - current_stake) / summary.combined_daily_rewards).max(1))
                    } else {
                        None
                    },
                    bonus_attainment: format!("+{:.0}% APY", tier.apy_base * 100),
                });
                break;
            }
        }

        // Generate milestones
        let milestones = vec![
            RewardMilestone {
                milestone_type: "first_100".to_string(),
                description: "Earn 100 FLUX".to_string(),
                reward_amount: 100_000_000_00,
                reached_at_day: if summary.combined_daily_rewards > 0 { Some(100_000_000_00 / summary.combined_daily_rewards) } else { None },
            },
            RewardMilestone {
                milestone_type: "first_1000".to_string(),
                description: "Earn 1,000 FLUX".to_string(),
                reward_amount: 100_000_000_000,
                reached_at_day: if summary.combined_daily_rewards > 0 { Some(100_000_000_000 / summary.combined_daily_rewards) } else { None },
            },
            RewardMilestone {
                milestone_type: "tier_gold".to_string(),
                description: "Reach Gold tier (100K stake)".to_string(),
                reward_amount: 0,
                reached_at_day: tier_progression.first().and_then(|t| t.days_to_reach),
            },
        ];

        RewardProjectionResponse {
            wallet_address: wallet_address.to_string(),
            projections,
            tier_progression,
            milestones,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_response_builder() {
        let positions = generate_mock_staking_positions("0x1234abcd");
        let miners = generate_mock_mining_positions("0x1234abcd");

        let response = UnifiedRewardsRouter::get_summary(
            "0x1234abcd",
            positions,
            miners,
        );

        assert_eq!(response.wallet_address, "0x1234abcd");
        assert!(response.total_value_locked > 0);
        assert!(response.combined_daily_rewards > 0);
    }

    #[test]
    fn test_combined_apy_calculation() {
        let positions = generate_mock_staking_positions("0x5678efgh");
        let miners = generate_mock_mining_positions("0x5678efgh");

        let response = UnifiedRewardsRouter::get_combined_apy(
            "0x5678efgh",
            positions,
            miners,
        );

        assert!(response.staking_apy > 0.0);
        assert!(response.combined_apy > 0.0);
    }

    #[test]
    fn test_projection_generation() {
        let positions = generate_mock_staking_positions("0xabcd1234");
        let miners = generate_mock_mining_positions("0xabcd1234");

        let response = UnifiedRewardsRouter::get_projections(
            "0xabcd1234",
            positions,
            miners,
            30,
            7, // Weekly compounding
        );

        assert_eq!(response.projections.len(), 30);
        assert!(response.projections[0].cumulative_rewards < response.projections[29].cumulative_rewards);
    }
}
