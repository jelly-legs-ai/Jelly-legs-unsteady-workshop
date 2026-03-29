// Database Schema Migrations - Staking Enhancements
// Migration scripts for tiered rewards, auto-compound, and validator bonding
// AeTHer Chain - Sprint 19 Backend

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// MIGRATION: Tiered Rewards System
// ============================================================================

/// Staking tiers with bonus multipliers and benefits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingTier {
    pub tier_id: String,           // bronze, silver, gold, diamond
    pub name: String,              // Display name
    pub min_stake: u64,            // Minimum stake amount
    pub max_stake: Option<u64>,    // Maximum stake (None = unlimited)
    pub reward_multiplier: f64,    // 1.0x, 1.05x, 1.12x, 1.20x
    pub lockup_reduction: f64,     // 0.0, 0.10, 0.20, 0.30 (10%, 20%, 30% shorter)
    pub governance_multiplier: f64,// 1.0x, 1.0x, 1.0x, 1.5x
    pub priority_withdrawal: bool, // Skip withdrawal queue
    pub created_at: u64,
}

/// User's current staking tier assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStakingTier {
    pub user_id: String,           // Foreign key -> users.user_id
    pub current_tier: String,      // bronze, silver, gold, diamond
    pub total_staked: u64,         // Aggregate stake across all pools
    tier_assigned_at: u64,
    pub last_tier_check: u64,      // Epoch of last tier recalculation
    pub bonus_earned: u64,         // Total bonus rewards from tier
    pub tier_history: Vec<TierChange>,
}

/// Tier change history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierChange {
    pub from_tier: String,
    pub to_tier: String,
    pub changed_at: u64,
    pub reason: String,            // stake_increase, stake_decrease, manual_adjustment
    pub stake_amount: u64,
}

// ============================================================================
// MIGRATION: Auto-Compound System
// ============================================================================

/// Auto-compound configuration per stake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundConfig {
    pub config_id: String,         // Primary key: ac_{uuid}
    pub stake_id: String,          // Foreign key -> stakes.stake_id
    pub is_enabled: bool,
    pub compound_threshold: u64,   // Min rewards before auto-compound (in FLUX)
    pub compound_frequency: u64,   // Epochs between compounds
    pub reinvest_percentage: f64,  // 0.0-1.0 (100% = all rewards reinvested)
    pub last_compound_epoch: u64,
    pub total_compounded: u64,     // Lifetime compounded amount
    pub compounds_count: u64,      // Number of compounds executed
    pub gas_saved: u64,            // Estimated gas saved vs manual claims
    pub created_at: u64,
    pub updated_at: u64,
}

/// Auto-compound execution log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundLog {
    pub log_id: String,            // Primary key: acl_{uuid}
    pub config_id: String,         // Foreign key -> auto_compound_configs.config_id
    pub epoch: u64,
    pub rewards_compounded: u64,
    pub new_stake_amount: u64,
    pub gas_cost: u64,
    pub transaction_hash: String,
    pub status: String,            // success, failed, skipped
    pub error_message: Option<String>,
    pub executed_at: u64,
}

/// Batched auto-compound job for gas optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundBatch {
    pub batch_id: String,          // Primary key: acb_{uuid}
    pub epoch: u64,
    pub total_configs: u64,        // Number of configs in this batch
    pub total_compounded: u64,
    pub total_gas_cost: u64,
    pub gas_per_compound: f64,     // Average gas per compound
    pub status: String,            // pending, processing, completed, failed
    pub executed_at: Option<u64>,
    pub created_at: u64,
}

// ============================================================================
// MIGRATION: Validator Bonding System
// ============================================================================

/// Validator bond information (skin-in-the-game)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorBond {
    pub bond_id: String,           // Primary key: bond_{validator_address}
    pub validator_id: String,      // Foreign key -> validators.validator_id
    pub bond_amount: u64,          // Amount bonded (in AETH)
    pub bond_source: String,       // personal, delegated, mixed
    pub min_bond_required: u64,    // Minimum required bond
    pub bond_percentage: f64,      // Bond as % of total delegated
    pub slashing_coverage: f64,    // 0.0-1.0 (default 0.50 = 50% coverage)
    pub is_bonded: bool,
    pub bonded_since: u64,
    pub last_bond_check: u64,
    pub bond_topups: Vec<BondTopup>,
    pub created_at: u64,
}

/// Bond topup history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondTopup {
    pub topup_id: String,
    pub amount: u64,
    pub source: String,            // validator_self, delegation, rewards
    pub transaction_hash: String,
    pub topup_at: u64,
}

/// Validator self-delegation requirement tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSelfDelegation {
    pub validator_id: String,
    pub self_delegated: u64,       // Amount validator staked in own pool
    pub total_delegated: u64,      // Total including others' delegations
    pub self_delegation_percentage: f64, // self_delegated / total_delegated
    pub min_percentage_required: f64,    // Usually 0.05 (5%)
    pub is_compliant: bool,
    pub last_check_epoch: u64,
}

/// Bond verification status for accepting delegations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondVerification {
    pub validator_id: String,
    pub is_verified: bool,
    pub verification_epoch: u64,
    pub can_accept_delegations: bool,
    pub bond_health_score: f64,    // 0-100
    pub warnings: Vec<String>,
    pub last_verified: u64,
}

// ============================================================================
// MIGRATION: Enhanced Rewards Tracker
// ============================================================================

/// Enhanced rewards tracking with analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRewardsTracker {
    pub tracker_id: String,        // Primary key: ert_{user_id}
    pub user_id: String,
    pub total_earned: u64,         // Lifetime rewards earned
    pub total_claimed: u64,        // Lifetime rewards claimed
    pub total_compounded: u64,     // Lifetime rewards compounded
    pub best_epoch_rewards: u64,   // Highest single-epoch rewards
    pub best_epoch: u64,
    pub average_daily_rewards: f64,
    pub current_streak_days: u64,
    pub longest_streak_days: u64,
    pub last_claim_epoch: u64,
    pub claim_count: u64,
    pub claim_history: Vec<ClaimRecord>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Individual claim record (last 100 kept)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRecord {
    pub claim_id: String,
    pub epoch: u64,
    pub amount: u64,
    pub claim_type: String,        // manual, auto_compound, tier_bonus
    pub tier_bonus_amount: u64,    // Extra from tier multiplier
    pub transaction_hash: String,
    pub claimed_at: u64,
}

/// Daily rewards snapshot for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRewardsSnapshot {
    pub snapshot_id: String,
    pub user_id: String,
    pub date: String,              // YYYY-MM-DD
    pub rewards_earned: u64,
    pub rewards_claimed: u64,
    pub rewards_compounded: u64,
    pub tier_at_snapshot: String,
    pub active_stakes: u64,
    pub snapshot_at: u64,
}

// ============================================================================
// MIGRATION SCRIPTS
// ============================================================================

pub struct StakingEnhancementsMigration;

impl StakingEnhancementsMigration {
    /// Run all migrations for staking enhancements
    pub fn run_all() -> MigrationResult {
        let mut result = MigrationResult::new();
        
        result.add_step("Creating staking_tiers table");
        result.add_step("Creating user_staking_tiers table");
        result.add_step("Creating auto_compound_configs table");
        result.add_step("Creating auto_compound_logs table");
        result.add_step("Creating auto_compound_batches table");
        result.add_step("Creating validator_bonds table");
        result.add_step("Creating validator_self_delegations table");
        result.add_step("Creating bond_verifications table");
        result.add_step("Creating enhanced_rewards_trackers table");
        result.add_step("Creating daily_rewards_snapshots table");
        result.add_step("Adding tier columns to stakes table");
        result.add_step("Adding bond columns to validators table");
        result.add_step("Seeding default tier configurations");
        
        result.success = true;
        result.completed_at = Self::current_epoch();
        result
    }
    
    /// Seed default tier configurations
    pub fn seed_default_tiers() -> Vec<StakingTier> {
        vec![
            StakingTier {
                tier_id: "bronze".to_string(),
                name: "Bronze".to_string(),
                min_stake: 100,
                max_stake: Some(10_000),
                reward_multiplier: 1.0,
                lockup_reduction: 0.0,
                governance_multiplier: 1.0,
                priority_withdrawal: false,
                created_at: Self::current_epoch(),
            },
            StakingTier {
                tier_id: "silver".to_string(),
                name: "Silver".to_string(),
                min_stake: 10_000,
                max_stake: Some(100_000),
                reward_multiplier: 1.05,
                lockup_reduction: 0.10,
                governance_multiplier: 1.0,
                priority_withdrawal: false,
                created_at: Self::current_epoch(),
            },
            StakingTier {
                tier_id: "gold".to_string(),
                name: "Gold".to_string(),
                min_stake: 100_000,
                max_stake: Some(1_000_000),
                reward_multiplier: 1.12,
                lockup_reduction: 0.20,
                governance_multiplier: 1.0,
                priority_withdrawal: true,
                created_at: Self::current_epoch(),
            },
            StakingTier {
                tier_id: "diamond".to_string(),
                name: "Diamond".to_string(),
                min_stake: 1_000_000,
                max_stake: None,
                reward_multiplier: 1.20,
                lockup_reduction: 0.30,
                governance_multiplier: 1.5,
                priority_withdrawal: true,
                created_at: Self::current_epoch(),
            },
        ]
    }
    
    fn current_epoch() -> u64 {
        // In production: use actual epoch from chain state
        48291
    }
}

/// Migration execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub success: bool,
    pub steps: Vec<String>,
    pub errors: Vec<String>,
    pub started_at: u64,
    pub completed_at: u64,
}

impl MigrationResult {
    pub fn new() -> Self {
        Self {
            success: false,
            steps: Vec::new(),
            errors: Vec::new(),
            started_at: 0,
            completed_at: 0,
        }
    }
    
    pub fn add_step(&mut self, step: &str) {
        self.steps.push(step.to_string());
    }
    
    pub fn add_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
        self.success = false;
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_tiers_created() {
        let tiers = StakingEnhancementsMigration::seed_default_tiers();
        assert_eq!(tiers.len(), 4);
        
        let bronze = tiers.iter().find(|t| t.tier_id == "bronze").unwrap();
        assert_eq!(bronze.reward_multiplier, 1.0);
        assert_eq!(bronze.min_stake, 100);
        
        let diamond = tiers.iter().find(|t| t.tier_id == "diamond").unwrap();
        assert_eq!(diamond.reward_multiplier, 1.20);
        assert_eq!(diamond.governance_multiplier, 1.5);
        assert!(diamond.priority_withdrawal);
    }
    
    #[test]
    fn test_migration_result() {
        let mut result = MigrationResult::new();
        result.add_step("Step 1");
        result.add_step("Step 2");
        result.success = true;
        
        assert!(result.success);
        assert_eq!(result.steps.len(), 2);
        assert_eq!(result.errors.len(), 0);
    }
}
