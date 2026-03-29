// Staking API Routes - AeTHer Chain
// API endpoints for staking operations, tier management, and auto-compound

use serde::{Deserialize, Serialize};

/// Staking API version
pub const STAKING_API_VERSION: &str = "v1";

// ============================================================================
// STAKING ENDPOINTS
// ============================================================================

/// Base staking routes
pub mod staking {
    pub const STAKE: &str = "/staking/stake";
    pub const UNSTAKE: &str = "/staking/unstake";
    pub const CLAIM: &str = "/staking/claim";
    pub const DELEGATE: &str = "/staking/delegate";
    pub const UNDELEGATE: &str = "/staking/undelegate";
    pub const REDELEGATE: &str = "/staking/redelegate";
    pub const POSITION: &str = "/staking/position/{stake_id}";
    pub const POSITIONS: &str = "/staking/positions/{user_id}";
    pub const REWARDS: &str = "/staking/rewards/{user_id}";
    pub const HISTORY: &str = "/staking/history/{user_id}";
    pub const POOLS: &str = "/staking/pools";
    pub const POOL: &str = "/staking/pools/{pool_id}";
    pub const VALIDATORS: &str = "/staking/validators";
    pub const VALIDATOR: &str = "/staking/validators/{validator_id}";
    pub const COMMISSION: &str = "/staking/validators/{validator_id}/commission";
    pub const METRICS: &str = "/staking/validators/{validator_id}/metrics";
}

// ============================================================================
// TIERED REWARDS ENDPOINTS (Sprint 18 Enhancement)
// ============================================================================

/// Tier management endpoints
pub mod tiers {
    pub const GET_TIER: &str = "/staking/tier/{user_id}";
    pub const TIER_BENEFITS: &str = "/staking/tiers/{tier}/benefits";
    pub const ALL_TIERS: &str = "/staking/tiers";
    pub const UPGRADE_TIER: &str = "/staking/tier/upgrade/{user_id}";
    pub const DOWNGRADE_TIER: &str = "/staking/tier/downgrade/{user_id}";
    pub const TIER_PROGRESS: &str = "/staking/tier/{user_id}/progress";
    pub const ESTIMATE_REWARDS: &str = "/staking/tier/estimate";
}

/// Tier levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TierLevel {
    Bronze,    // 100 - 10K stake, 1.0x multiplier
    Silver,    // 10K - 100K stake, 1.05x multiplier (5% bonus)
    Gold,      // 100K - 1M stake, 1.12x multiplier (12% bonus)
    Diamond,   // 1M+ stake, 1.20x multiplier (20% bonus)
}

impl TierLevel {
    /// Get multiplier for tier
    pub fn multiplier(&self) -> f64 {
        match self {
            TierLevel::Bronze => 1.0,
            TierLevel::Silver => 1.05,
            TierLevel::Gold => 1.12,
            TierLevel::Diamond => 1.20,
        }
    }
    
    /// Get minimum stake for tier
    pub fn min_stake(&self) -> u64 {
        match self {
            TierLevel::Bronze => 100,
            TierLevel::Silver => 10_000,
            TierLevel::Gold => 100_000,
            TierLevel::Diamond => 1_000_000,
        }
    }
    
    /// Get lockup reduction percentage
    pub fn lockup_reduction(&self) -> f64 {
        match self {
            TierLevel::Bronze => 0.0,
            TierLevel::Silver => 0.10,
            TierLevel::Gold => 0.20,
            TierLevel::Diamond => 0.30,
        }
    }
    
    /// Get governance power multiplier
    pub fn governance_multiplier(&self) -> f64 {
        match self {
            TierLevel::Diamond => 1.5,
            _ => 1.0,
        }
    }
}

/// Tier benefits response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierBenefits {
    pub tier: TierLevel,
    pub multiplier: f64,
    pub bonus_percentage: f64,
    pub lockup_reduction: f64,
    pub governance_multiplier: f64,
    pub priority_withdrawal: bool,
    pub reduced_commission: bool,
    pub exclusive_features: Vec<String>,
}

impl TierBenefits {
    pub fn from_tier(tier: TierLevel) -> Self {
        let mut features = vec![];
        
        if tier == TierLevel::Silver {
            features.push("5% rewards bonus".to_string());
        }
        if tier == TierLevel::Gold {
            features.push("12% rewards bonus".to_string());
            features.push("Priority withdrawal queue".to_string());
            features.push("Reduced validator commission".to_string());
        }
        if tier == TierLevel::Diamond {
            features.push("20% rewards bonus".to_string());
            features.push("Priority withdrawal queue".to_string());
            features.push("Reduced validator commission".to_string());
            features.push("1.5x governance voting power".to_string());
            features.push("Exclusive Diamond staking pools".to_string());
        }
        
        TierBenefits {
            tier: tier.clone(),
            multiplier: tier.multiplier(),
            bonus_percentage: (tier.multiplier() - 1.0) * 100.0,
            lockup_reduction: tier.lockup_reduction(),
            governance_multiplier: tier.governance_multiplier(),
            priority_withdrawal: tier == TierLevel::Gold || tier == TierLevel::Diamond,
            reduced_commission: tier == TierLevel::Gold || tier == TierLevel::Diamond,
            exclusive_features: features,
        }
    }
}

// ============================================================================
// AUTO-COMPOUND ENDPOINTS (Sprint 18 Enhancement)
// ============================================================================

/// Auto-compound management endpoints
pub mod autocompound {
    pub const ENABLE: &str = "/staking/autocompound/enable/{stake_id}";
    pub const DISABLE: &str = "/staking/autocompound/disable/{stake_id}";
    pub const STATUS: &str = "/staking/autocompound/status/{stake_id}";
    pub const CONFIGURE: &str = "/staking/autocompound/configure/{stake_id}";
    pub const HISTORY: &str = "/staking/autocompound/history/{stake_id}";
    pub const BATCH_ENABLE: &str = "/staking/autocompound/batch/enable";
    pub const BATCH_DISABLE: &str = "/staking/autocompound/batch/disable";
    pub const ESTIMATE_YIELD: &str = "/staking/autocompound/estimate";
}

/// Auto-compound configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundConfig {
    pub enabled: bool,
    pub compound_threshold: u64,      // Minimum rewards before auto-compound
    pub compound_frequency_epochs: u64, // How often to compound (in epochs)
    pub reinvest_percentage: f64,      // Percentage to reinvest (0.0 - 100.0)
    pub gas_optimization: bool,        // Enable gas-optimized batching
}

impl Default for AutoCompoundConfig {
    fn default() -> Self {
        AutoCompoundConfig {
            enabled: false,
            compound_threshold: 1000,
            compound_frequency_epochs: 24, // Once per day (assuming 1hr epochs)
            reinvest_percentage: 100.0,
            gas_optimization: true,
        }
    }
}

/// Auto-compound status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundStatus {
    pub stake_id: String,
    pub enabled: bool,
    pub pending_rewards: u64,
    pub last_compound_epoch: u64,
    pub next_compound_epoch: u64,
    pub total_compounded: u64,
    pub compound_count: u64,
    pub config: AutoCompoundConfig,
}

// ============================================================================
// VALIDATOR BONDING ENDPOINTS (Sprint 18 Enhancement)
// ============================================================================

/// Validator bonding endpoints
pub mod bonding {
    pub const BOND: &str = "/staking/validator/{validator_id}/bond";
    pub const UNBOND: &str = "/staking/validator/{validator_id}/unbond";
    pub const STATUS: &str = "/staking/validator/{validator_id}/bond-status";
    pub const SLASHING_COVERAGE: &str = "/staking/validator/{validator_id}/slashing-coverage";
    pub const SELF_DELEGATION: &str = "/staking/validator/{validator_id}/self-delegation";
    pub const VERIFY_BOND: &str = "/staking/validator/{validator_id}/verify-bond";
    pub const BOND_HISTORY: &str = "/staking/validator/{validator_id}/bond-history";
}

/// Validator bond information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorBond {
    pub validator_address: String,
    pub bond_amount: u64,
    pub bond_percentage: f64,          // Bond as % of total delegated
    pub slashing_coverage: f64,        // How much slashing the bond can cover
    pub self_delegation_percentage: f64,
    pub is_bonded: bool,
    pub can_accept_delegations: bool,
    pub last_bond_update_epoch: u64,
}

// ============================================================================
// REWARDS TRACKER ENDPOINTS (Sprint 18 Enhancement)
// ============================================================================

/// Rewards tracker endpoints
pub mod rewards_tracker {
    pub const SUMMARY: &str = "/staking/rewards/{user_id}/summary";
    pub const ANALYTICS: &str = "/staking/rewards/{user_id}/analytics";
    pub const CLAIM_HISTORY: &str = "/staking/rewards/{user_id}/claims";
    pub const BEST_EPOCH: &str = "/staking/rewards/{user_id}/best-epoch";
    pub const AVERAGE_DAILY: &str = "/staking/rewards/{user_id}/avg-daily";
    pub const STREAK: &str = "/staking/rewards/{user_id}/streak";
    pub const PROJECTION: &str = "/staking/rewards/{user_id}/projection";
    pub const EXPORT: &str = "/staking/rewards/{user_id}/export";
}

/// Rewards summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsSummary {
    pub user_id: String,
    pub total_earned: u64,
    pub total_claimed: u64,
    pub total_compounded: u64,
    pub pending_rewards: u64,
    pub best_epoch_rewards: u64,
    pub best_epoch_number: u64,
    pub average_daily_rewards: f64,
    pub current_streak_days: u64,
    pub longest_streak_days: u64,
    pub claim_count: u64,
    pub last_claim_epoch: u64,
}

/// Claim history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimHistoryEntry {
    pub claim_id: String,
    pub epoch: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub transaction_hash: Option<String>,
    pub claim_type: ClaimType,
    pub tier_bonus_applied: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaimType {
    Manual,
    AutoCompound,
    Unstake,
    ValidatorCommission,
}

// ============================================================================
// REQUEST/RESPONSE STRUCTS
// ============================================================================

/// Stake request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeRequest {
    pub user_id: String,
    pub pool_id: String,
    pub amount: u64,
    pub validator_id: Option<String>,
    pub enable_autocompound: Option<bool>,
}

/// Unstake request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeRequest {
    pub stake_id: String,
    pub amount: u64,
    pub immediate: bool,  // Pay penalty for immediate unstake
}

/// Claim rewards request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRewardsRequest {
    pub stake_id: String,
    pub amount: Option<u64>,  // None = claim all
}

/// Delegate request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateRequest {
    pub delegator: String,
    pub validator: String,
    pub amount: u64,
}

/// Tier estimate request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierEstimateRequest {
    pub stake_amount: u64,
    pub current_tier: Option<TierLevel>,
}

/// Tier estimate response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierEstimateResponse {
    pub estimated_tier: TierLevel,
    pub current_stake: u64,
    pub next_tier: Option<TierLevel>,
    pub amount_to_next_tier: u64,
    pub potential_bonus: f64,
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl StakingError {
    pub fn insufficient_balance() -> Self {
        StakingError {
            code: "INSUFFICIENT_BALANCE".to_string(),
            message: "Insufficient balance for staking operation".to_string(),
            details: None,
        }
    }
    
    pub fn minimum_stake_not_met(minimum: u64) -> Self {
        StakingError {
            code: "MINIMUM_STAKE_NOT_MET".to_string(),
            message: format!("Minimum stake amount is {} tokens", minimum),
            details: None,
        }
    }
    
    pub fn lockup_period_active(unlock_epoch: u64) -> Self {
        StakingError {
            code: "LOCKUP_PERIOD_ACTIVE".to_string(),
            message: "Tokens are still in lockup period".to_string(),
            details: Some(format!("Unlock epoch: {}", unlock_epoch)),
        }
    }
    
    pub fn validator_not_accepting() -> Self {
        StakingError {
            code: "VALIDATOR_NOT_ACCEPTING".to_string(),
            message: "Validator is not accepting delegations".to_string(),
            details: None,
        }
    }
}

// ============================================================================
// API RESPONSE WRAPPERS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<StakingError>,
    pub timestamp: u64,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    pub fn error(error: StakingError) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
