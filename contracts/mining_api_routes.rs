// Mining API Routes - AeTHer Chain
// API endpoints for FLUX mining operations, device management, and reward tracking

use serde::{Deserialize, Serialize};

/// Mining API version
pub const MINING_API_VERSION: &str = "v1";

// ============================================================================
// MINING ENDPOINTS
// ============================================================================

/// Base mining routes
pub mod mining {
    pub const REGISTER: &str = "/mining/register";
    pub const DEREGISTER: &str = "/mining/deregister";
    pub const STATUS: &str = "/mining/status/{miner_id}";
    pub const REWARDS: &str = "/mining/rewards/{miner_id}";
    pub const CLAIM: &str = "/mining/claim/{miner_id}";
    pub const HISTORY: &str = "/mining/history/{miner_id}";
    pub const LEADERBOARD: &str = "/mining/leaderboard";
    pub const METRICS: &str = "/mining/metrics";
    pub const POOL_STATS: &str = "/mining/pool-stats";
    pub const DEVICE_INFO: &str = "/mining/device/{miner_id}";
    pub const UPDATE_UPTIME: &str = "/mining/uptime/{miner_id}";
    pub const OPTIMIZE: &str = "/mining/optimize/{miner_id}";
}

/// FLUX token endpoints (for reward distribution)
pub mod flux {
    pub const BALANCE: &str = "/flux/balance/{address}";
    pub const STAKE_FOR_FLUX: &str = "/flux/stake";
    pub const UNSTAKE_FLUX: &str = "/flux/unstake";
    pub const FLUX_REWARDS: &str = "/flux/rewards/{address}";
    pub const SWAP_STATUS: &str = "/flux/swap-status";
}

/// Mining pool endpoints
pub mod pools {
    pub const LIST: &str = "/mining/pools";
    pub const JOIN: &str = "/mining/pools/{pool_id}/join";
    pub const LEAVE: &str = "/mining/pools/{pool_id}/leave";
    pub const POOL_MINERS: &str = "/mining/pools/{pool_id}/miners";
    pub const POOL_REWARDS: &str = "/mining/pools/{pool_id}/rewards";
    pub const POOL_METRICS: &str = "/mining/pools/{pool_id}/metrics";
}

/// Mining statistics and analytics
pub mod stats {
    pub const NETWORK_HASH: &str = "/mining/stats/network-hashrate";
    pub const DIFFICULTY: &str = "/mining/stats/difficulty";
    pub const EMISSION: &str = "/mining/stats/emission";
    pub const DISTRIBUTION: &str = "/mining/stats/distribution";
    pub const PROJECTION: &str = "/mining/stats/projection/{miner_id}";
    pub const COMPARISON: &str = "/mining/stats/comparison/{miner_id}";
    pub const TRENDS: &str = "/mining/stats/trends";
    pub const FORECAST: &str = "/mining/stats/forecast/{miner_id}";
}

// ============================================================================
// COMBINED FLUX + STAKING DASHBOARD ENDPOINTS (Sprint 49)
// ============================================================================

/// Combined dashboard endpoints for unified FLUX staking + mining view
pub mod dashboard {
    pub const OVERVIEW: &str = "/dashboard/overview/{address}";
    pub const COMBINEDREWARDS: &str = "/dashboard/combined-rewards/{address}";
    pub const PORTFOLIO: &str = "/dashboard/portfolio/{address}";
    pub const ACTIVITY: &str = "/dashboard/activity/{address}";
    pub const NETWORTH: &str = "/dashboard/net-worth/{address}";
    pub const ALLOCATION: &str = "/dashboard/allocation/{address}";
    pub const PERFORMANCE: &str = "/dashboard/performance/{address}";
    pub const PROJECTIONS: &str = "/dashboard/projections/{address}";
}

/// Mining contribution tiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MiningTier {
    Bronze,    // Entry level miners
    Silver,    // Active miners
    Gold,      // Power miners
    Diamond,   // Elite miners (dedicated servers)
}

impl MiningTier {
    /// Get multiplier for mining tier
    pub fn multiplier(&self) -> f64 {
        match self {
            MiningTier::Bronze => 1.0,
            MiningTier::Silver => 1.25,
            MiningTier::Gold => 1.5,
            MiningTier::Diamond => 2.0,
        }
    }
    
    /// Get minimum uptime for tier
    pub fn min_uptime(&self) -> f64 {
        match self {
            MiningTier::Bronze => 50.0,
            MiningTier::Silver => 70.0,
            MiningTier::Gold => 85.0,
            MiningTier::Diamond => 95.0,
        }
    }
    
    /// Get minimum contribution score for tier
    pub fn min_contribution(&self) -> f64 {
        match self {
            MiningTier::Bronze => 0.3,
            MiningTier::Silver => 0.5,
            MiningTier::Gold => 0.7,
            MiningTier::Diamond => 0.9,
        }
    }
}

/// Device specification for mining tier calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSpec {
    pub device_type: String,      // "mobile", "laptop", "desktop", "server"
    pub ram_gb: u32,
    pub cpu_cores: u32,
    pub cpu_model: String,
    pub gpu_available: bool,
    pub gpu_model: Option<String>,
    pub os: String,                // "android", "ios", "windows", "macos", "linux"
    pub architecture: String,      // "arm64", "x86_64"
}

impl DeviceSpec {
    /// Calculate estimated tier based on device spec
    pub fn estimate_tier(&self) -> MiningTier {
        let mut score = 0.0;
        
        // RAM contributes heavily
        score += (self.ram_gb as f64 / 64.0).min(1.0) * 30.0;
        
        // CPU cores
        score += (self.cpu_cores as f64 / 32.0).min(1.0) * 20.0;
        
        // GPU bonus
        if self.gpu_available {
            score += 25.0;
        }
        
        // OS stability bonus (servers typically run Linux)
        match self.os.as_str() {
            "linux" => score += 15.0,
            "windows" | "macos" => score += 10.0,
            "android" | "ios" => score += 5.0,
            _ => {}
        }
        
        // Device type bonus
        match self.device_type.as_str() {
            "server" => score += 10.0,
            "desktop" => score += 5.0,
            "laptop" => score += 2.0,
            _ => {}
        }
        
        // Map score to tier
        if score >= 75.0 {
            MiningTier::Diamond
        } else if score >= 55.0 {
            MiningTier::Gold
        } else if score >= 35.0 {
            MiningTier::Silver
        } else {
            MiningTier::Bronze
        }
    }
}

/// Mining leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningLeaderboardEntry {
    pub rank: u32,
    pub miner_id: String,
    pub device_tier: MiningTier,
    pub total_rewards: u64,
    pub epochs_mined: u64,
    pub uptime_percentage: f64,
    pub contribution_score: f64,
    pub recent_rewards: u64,       // Last 7 days
    pub is_active: bool,
}

/// Network-wide mining statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMiningStats {
    pub total_miners: u64,
    pub active_miners: u64,
    pub total_network_hashrate: f64,  // GH/s equivalent
    pub average_uptime: f64,
    pub average_contribution: f64,
    pub total_rewards_distributed: u64,
    pub rewards_by_tier: TierRewards,
    pub device_distribution: DeviceDistribution,
    pub emission_rate: EmissionStats,
}

/// Rewards breakdown by tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRewards {
    pub bronze: u64,
    pub silver: u64,
    pub gold: u64,
    pub diamond: u64,
}

/// Device type distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDistribution {
    pub mobile: u32,
    pub laptop: u32,
    pub desktop: u32,
    pub server: u32,
}

/// Emission and inflation stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionStats {
    pub current_epoch_rewards: u64,
    pub daily_emission: u64,
    pub annual_emission: u64,
    pub inflation_rate: f64,
    pub halving_count: u64,
    pub next_halving_epoch: u64,
    pub supply_remaining: u64,
}

/// Miner reward projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerProjection {
    pub miner_id: String,
    pub current_tier: MiningTier,
    pub estimated_tier: MiningTier,
    pub hourly_rewards: u64,
    pub daily_rewards: u64,
    pub weekly_rewards: u64,
    pub monthly_rewards: u64,
    pub yearly_rewards: u64,
    pub confidence: f64,
    pub factors: ProjectionFactors,
}

/// Factors affecting projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionFactors {
    pub device_mult: f64,
    pub uptime_mult: f64,
    pub contribution_mult: f64,
    pub tier_mult: f64,
    pub stake_mult: f64,
    pub loyalty_mult: f64,
    pub halving_factor: f64,
}

/// Combined FLUX staking + mining position for unified dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedPosition {
    pub address: String,
    pub flux_staked: u64,
    pub flux_pending_rewards: u64,
    pub flux_total_earned: u64,
    pub mining_active: bool,
    pub mining_device_tier: Option<MiningTier>,
    pub mining_rewards_pending: u64,
    pub mining_total_earned: u64,
    pub mining_epochs: u64,
    pub total_combined_value: u64,
    pub total_combined_rewards: u64,
    pub combined_apy_estimate: f64,
    pub tier_boost: f64,
    pub auto_compound_enabled: bool,
}

/// Portfolio allocation breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAllocation {
    pub address: String,
    pub aeth_staked: u64,
    pub flux_staked: u64,
    pub flux_locked: u64,
    pub ath_holding: u64,
    pub mining_positions: u64,
    pub total_value_usd: f64,
    pub allocation_percentages: AllocationPercentages,
    pub risk_score: f64,
    pub diversification_tier: String,
}

/// Allocation percentages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPercentages {
    pub aeth_stake: f64,
    pub flux_stake: f64,
    pub flux_lock: f64,
    pub ath_holdings: f64,
    pub mining: f64,
}

// ============================================================================
// MINER TIER PROGRESSION & PERFORMANCE TRACKING (Sprint 51)
// ============================================================================

/// Historical tier progression record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerTierHistory {
    pub miner_id: String,
    pub previous_tier: MiningTier,
    pub new_tier: MiningTier,
    pub reason: TierChangeReason,
    pub timestamp: u64,
    pub effective_epoch: u64,
    pub threshold_met: u64,
    pub next_tier_threshold: Option<u64>,
}

/// Reason for tier change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TierChangeReason {
    HashrateIncrease,
    UptimeMilestone,
    StakeBonusApplied,
    LoyaltyBonusApplied,
    CombinedScoreBoost,
    HalvingAdjustment,
    ManualUpgrade,
}

/// Miner performance metrics (rolling averages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerPerformanceMetrics {
    pub miner_id: String,
    pub current_epoch: u64,
    // Rolling 7-day metrics
    pub avg_hashrate_7d: f64,
    pub avg_uptime_7d: f64,
    pub avg_rewards_7d: u64,
    pub share_rate_7d: f64,
    // Rolling 30-day metrics
    pub avg_hashrate_30d: f64,
    pub avg_uptime_30d: f64,
    pub avg_rewards_30d: u64,
    pub share_rate_30d: f64,
    // Lifetime totals
    pub total_blocks_found: u64,
    pub total_rewards_earned: u64,
    pub total_epochs_mined: u64,
    pub best_epoch_hashrate: f64,
    pub longest_uptime_streak: u64,
    // Performance scores
    pub efficiency_score: f64,      // rewards / hashrate ratio
    pub consistency_score: f64,      // how stable the hashrate is
    pub reliability_score: f64,       // uptime percentage
    pub tier_score: f64,            // combined tier bonus factor
}

/// Epoch-by-epoch reward summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochRewardsSummary {
    pub epoch: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub total_network_rewards: u64,
    pub miner_reward: u64,
    pub pool_reward: Option<u64>,
    pub base_reward: u64,
    pub tier_bonus: u64,
    pub stake_bonus: u64,
    pub loyalty_bonus: u64,
    pub penalty: u64,
    pub net_reward: u64,
    pub shares_submitted: u64,
    pub effective_hashrate: f64,
    pub miner_participation_rate: f64,
}

/// Miner streak tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStreakData {
    pub miner_id: String,
    pub current_uptime_streak: u64,     // consecutive epochs with >99% uptime
    pub current_hashrate_streak: u64,   // consecutive epochs meeting hashrate quota
    pub current_consistency_streak: u64, // consecutive epochs with <5% variance
    pub longest_uptime_streak_ever: u64,
    pub longest_hashrate_streak_ever: u64,
    pub longest_consistency_streak_ever: u64,
    pub streak_multiplier: f64,         // bonus applied for active streaks
    pub streak_tier: StreakTier,
}

/// Streak tiers for bonuses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreakTier {
    None,       // No active streak
    Bronze,     // 7+ epochs
    Silver,     // 30+ epochs
    Gold,       // 100+ epochs
    Diamond,    // 365+ epochs
}

/// Miner projected position at next epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerProjectedPosition {
    pub miner_id: String,
    pub current_tier: MiningTier,
    pub projected_tier: MiningTier,
    pub current_score: f64,
    pub projected_score: f64,
    pub score_delta: f64,
    pub epochs_to_upgrade: Option<u64>,
    pub epochs_to_downgrade: Option<u64>,
    pub upgrade_requirements: Vec<String>,
    pub projected_rewards_delta: u64,
    pub next_halving_impact: f64,
}
