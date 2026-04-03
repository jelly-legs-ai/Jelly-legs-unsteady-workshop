// Mining Analytics - AeTHer Chain
// Advanced analytics, projections, and forecasting for FLUX mining operations

use serde::{Deserialize, Serialize};

/// Mining analytics for a miner's performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningAnalytics {
    pub miner_id: String,
    pub period_start: i64,
    pub period_end: i64,
    pub total_epochs_mined: u64,
    pub total_rewards: u64,
    pub average_uptime: f64,
    pub average_contribution: f64,
    pub tier_changes: Vec<TierChange>,
    pub performance_score: f64,
    pub rank: u32,
    pub percentile: f64,
}

/// Record of tier upgrades/downgrades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierChange {
    pub epoch: u64,
    pub from_tier: String,
    pub to_tier: String,
    pub reason: String,
}

/// Mining projection for future rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningProjection {
    pub miner_id: String,
    pub projection_days: u32,
    pub conservative_estimate: u64,
    pub expected_estimate: u64,
    pub optimistic_estimate: u64,
    pub daily_rates: Vec<DailyRate>,
    pub milestone_dates: Vec<MilestoneDate>,
}

/// Daily reward rate with date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRate {
    pub date: String,
    pub reward: u64,
    pub uptime: f64,
}

/// Projected milestone for reaching reward goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneDate {
    pub target_rewards: u64,
    pub estimated_date: String,
    pub days_remaining: u32,
}

/// Network mining statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMiningStats {
    pub total_active_miners: u64,
    pub total_devices: u64,
    pub total_hashrate: u64,
    pub average_miner_rewards: u64,
    pub median_miner_rewards: u64,
    pub top_earners: Vec<MinerEarning>,
    pub device_tier_distribution: TierDistribution,
    pub reward_distribution_by_tier: Vec<TierRewardStats>,
    pub mining_difficulty: f64,
    pub next_halving_epoch: u64,
    pub epochs_until_halving: u64,
}

/// Miner earning summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerEarning {
    pub miner_id: String,
    pub rank: u32,
    pub total_rewards: u64,
    pub device_tier: String,
}

/// Distribution of miners by device tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDistribution {
    pub mobile: u64,
    pub laptop: u64,
    pub desktop: u64,
    pub server: u64,
}

/// Reward statistics per tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRewardStats {
    pub tier: String,
    pub miner_count: u64,
    pub total_rewards: u64,
    pub average_rewards: u64,
    pub percentage_of_network: f64,
}

/// Comparison of miner against network averages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerComparison {
    pub miner_id: String,
    pub vs_average: ComparisonMetrics,
    pub vs_tier_average: ComparisonMetrics,
    pub vs_top_performer: ComparisonMetrics,
}

/// Comparison metrics for benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    pub uptime_delta: f64,
    pub rewards_delta: f64,
    pub efficiency_delta: f64,
    pub percentile: f64,
}

/// Mining efficiency score calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyScore {
    pub raw_score: f64,
    pub normalized_score: f64,
    pub factors: EfficiencyFactors,
    pub recommendations: Vec<String>,
}

/// Factors contributing to efficiency score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyFactors {
    pub uptime_factor: f64,
    pub contribution_factor: f64,
    pub tier_bonus: f64,
    pub consistency_bonus: f64,
    pub penalty: f64,
}

impl MiningAnalytics {
    /// Calculate performance score based on uptime and contribution
    pub fn calculate_performance_score(uptime: f64, contribution: f64, tier_multiplier: f64) -> f64 {
        let base_score = (uptime * 0.6) + (contribution * 0.4);
        base_score * tier_multiplier
    }
}

impl MiningProjection {
    /// Generate reward projections based on current rates and historical data
    pub fn generate_projections(
        miner_id: &str,
        current_daily_rate: u64,
        uptime_history: &[f64],
        days: u32,
    ) -> Self {
        let avg_uptime = if !uptime_history.is_empty() {
            uptime_history.iter().sum::<f64>() / uptime_history.len() as f64
        } else {
            0.95
        };

        // Apply uptime factor to reward rate
        let effective_rate = (current_daily_rate as f64 * avg_uptime) as u64;

        // Calculate estimates with different scenarios
        let conservative_estimate = effective_rate * 90 / 100; // 10% buffer
        let expected_estimate = effective_rate;
        let optimistic_estimate = effective_rate * 115 / 100; // 15% upside

        // Generate daily rates for chart
        let daily_rates: Vec<DailyRate> = (0..days)
            .map(|i| {
                let date = chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::days(i))
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();
                DailyRate {
                    date,
                    reward: effective_rate,
                    uptime: avg_uptime,
                }
            })
            .collect();

        // Calculate milestone dates
        let milestones = vec![
            MilestoneDate {
                target_rewards: effective_rate * 30,
                estimated_date: "30 days".to_string(),
                days_remaining: 30,
            },
            MilestoneDate {
                target_rewards: effective_rate * 365,
                estimated_date: "1 year".to_string(),
                days_remaining: 365,
            },
        ];

        MiningProjection {
            miner_id: miner_id.to_string(),
            projection_days: days,
            conservative_estimate,
            expected_estimate,
            optimistic_estimate,
            daily_rates,
            milestone_dates: milestones,
        }
    }
}

impl NetworkMiningStats {
    /// Create mock network stats for testing
    pub fn mock() -> Self {
        NetworkMiningStats {
            total_active_miners: 48291,
            total_devices: 67834,
            total_hashrate: 1250000000000,
            average_miner_rewards: 2500000,
            median_miner_rewards: 1800000,
            top_earners: vec![
                MinerEarning {
                    miner_id: "flux_top_001".to_string(),
                    rank: 1,
                    total_rewards: 15000000000,
                    device_tier: "server".to_string(),
                },
                MinerEarning {
                    miner_id: "flux_elite_002".to_string(),
                    rank: 2,
                    total_rewards: 12500000000,
                    device_tier: "server".to_string(),
                },
                MinerEarning {
                    miner_id: "flux_pro_003".to_string(),
                    rank: 3,
                    total_rewards: 9800000000,
                    device_tier: "desktop".to_string(),
                },
            ],
            device_tier_distribution: TierDistribution {
                mobile: 25000,
                laptop: 22000,
                desktop: 15000,
                server: 5834,
            },
            reward_distribution_by_tier: vec![
                TierRewardStats {
                    tier: "mobile".to_string(),
                    miner_count: 25000,
                    total_rewards: 50000000000,
                    average_rewards: 2000000,
                    percentage_of_network: 0.08,
                },
                TierRewardStats {
                    tier: "laptop".to_string(),
                    miner_count: 22000,
                    total_rewards: 88000000000,
                    average_rewards: 4000000,
                    percentage_of_network: 0.15,
                },
                TierRewardStats {
                    tier: "desktop".to_string(),
                    miner_count: 15000,
                    total_rewards: 120000000000,
                    average_rewards: 8000000,
                    percentage_of_network: 0.25,
                },
                TierRewardStats {
                    tier: "server".to_string(),
                    miner_count: 5834,
                    total_rewards: 210000000000,
                    average_rewards: 36000000,
                    percentage_of_network: 0.52,
                },
            ],
            mining_difficulty: 1.25,
            next_halving_epoch: 43800,
            epochs_until_halving: 12450,
        }
    }
}

impl EfficiencyScore {
    /// Calculate efficiency score for a miner
    pub fn calculate(
        uptime: f64,
        contribution: f64,
        tier_multiplier: f64,
        consecutive_epochs: u64,
        has_penalty: bool,
    ) -> Self {
        let uptime_factor = uptime * 0.4;
        let contribution_factor = contribution * 0.3;
        let tier_bonus = (tier_multiplier - 1.0) * 0.2;
        let consistency_bonus = if consecutive_epochs > 1000 {
            0.1
        } else if consecutive_epochs > 500 {
            0.05
        } else {
            0.0
        };
        let penalty = if has_penalty { 0.2 } else { 0.0 };

        let raw_score =
            uptime_factor + contribution_factor + tier_bonus + consistency_bonus - penalty;
        let normalized_score = (raw_score * 100.0).min(100.0).max(0.0);

        let mut recommendations = Vec::new();
        if uptime < 0.9 {
            recommendations.push("Improve device uptime to 90%+ for better rewards".to_string());
        }
        if contribution < 0.7 {
            recommendations.push("Increase contribution score by running more tasks".to_string());
        }
        if tier_multiplier < 2.0 {
            recommendations.push("Consider upgrading to a higher-tier device".to_string());
        }

        EfficiencyScore {
            raw_score,
            normalized_score,
            factors: EfficiencyFactors {
                uptime_factor,
                contribution_factor,
                tier_bonus,
                consistency_bonus,
                penalty,
            },
            recommendations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_score_calculation() {
        let score = MiningAnalytics::calculate_performance_score(0.95, 0.8, 2.0);
        assert!(score > 0.0);
        assert!(score <= 2.0);
    }

    #[test]
    fn test_projection_generation() {
        let projection = MiningProjection::generate_projections(
            "test_miner",
            1000000,
            &[0.95, 0.92, 0.98],
            30,
        );
        assert_eq!(projection.miner_id, "test_miner");
        assert!(projection.expected_estimate >= projection.conservative_estimate);
    }

    #[test]
    fn test_efficiency_score_calculation() {
        let score =
            EfficiencyScore::calculate(0.95, 0.8, 2.0, 1500, false);
        assert!(score.normalized_score > 50.0);
        assert!(!score.recommendations.is_empty());
    }
}
