// Database Migration: Staking Enhancements
// Migrates staking data from basic to enhanced format with tiered rewards

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// MIGRATION SCHEMA
// ============================================================================

/// Migration record for tracking applied migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub id: String,
    pub name: String,
    pub applied_at: u64,
    pub epoch: u64,
    pub status: MigrationStatus,
    pub records_migrated: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MigrationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

/// V1 -> V2 migration: Add tiered rewards to stakes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingMigrationV2 {
    // Old format
    pub stake_id: String,
    pub user_address: String,
    pub pool_id: String,
    pub amount: u64,
    pub start_epoch: u64,
    pub lock_end_epoch: u64,
    
    // New fields (added in migration)
    pub tier: String,
    pub tier_multiplier: f64,
    pub lockup_discount: f64,
    pub priority_withdrawal: bool,
    pub governance_weight: f64,
}

/// V2 -> V3 migration: Add auto-compound configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingMigrationV3 {
    // From V2
    pub stake_id: String,
    pub user_address: String,
    pub pool_id: String,
    pub amount: u64,
    pub start_epoch: u64,
    pub lock_end_epoch: u64,
    pub tier: String,
    pub tier_multiplier: f64,
    
    // New auto-compound fields
    pub auto_compound_enabled: bool,
    pub compound_threshold: u64,
    pub compound_frequency_epochs: u64,
    pub reinvest_percentage: f64,
    pub last_compound_epoch: u64,
    pub total_compounds: u64,
    pub total_compounded_amount: u64,
}

/// Migration state manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationManager {
    pub migrations: HashMap<String, MigrationRecord>,
    pub current_epoch: u64,
    pub is_locked: bool,
}

impl MigrationManager {
    /// Create new migration manager
    pub fn new() -> Self {
        MigrationManager {
            migrations: HashMap::new(),
            current_epoch: 0,
            is_locked: false,
        }
    }
    
    /// Register a new migration
    pub fn register_migration(&mut self, id: &str, name: &str, epoch: u64) -> Result<(), &'static str> {
        if self.is_locked {
            return Err("Migration manager is locked");
        }
        
        if self.migrations.contains_key(id) {
            return Err("Migration already registered");
        }
        
        self.migrations.insert(id.to_string(), MigrationRecord {
            id: id.to_string(),
            name: name.to_string(),
            applied_at: 0,
            epoch,
            status: MigrationStatus::Pending,
            records_migrated: 0,
            errors: Vec::new(),
        });
        
        Ok(())
    }
    
    /// Start a migration
    pub fn start_migration(&mut self, id: &str) -> Result<(), &'static str> {
        let migration = self.migrations.get_mut(id)
            .ok_or("Migration not found")?;
        
        if migration.status != MigrationStatus::Pending {
            return Err("Migration already started or completed");
        }
        
        migration.status = MigrationStatus::Running;
        migration.applied_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(())
    }
    
    /// Complete a migration
    pub fn complete_migration(&mut self, id: &str, records_migrated: u64) -> Result<(), &'static str> {
        let migration = self.migrations.get_mut(id)
            .ok_or("Migration not found")?;
        
        if migration.status != MigrationStatus::Running {
            return Err("Migration not running");
        }
        
        migration.status = MigrationStatus::Completed;
        migration.records_migrated = records_migrated;
        
        Ok(())
    }
    
    /// Fail a migration
    pub fn fail_migration(&mut self, id: &str, error: &str) -> Result<(), &'static str> {
        let migration = self.migrations.get_mut(id)
            .ok_or("Migration not found")?;
        
        migration.status = MigrationStatus::Failed;
        migration.errors.push(error.to_string());
        
        Ok(())
    }
    
    /// Rollback a migration
    pub fn rollback_migration(&mut self, id: &str) -> Result<(), &'static str> {
        let migration = self.migrations.get_mut(id)
            .ok_or("Migration not found")?;
        
        if migration.status != MigrationStatus::Failed {
            return Err("Can only rollback failed migrations");
        }
        
        migration.status = MigrationStatus::RolledBack;
        
        Ok(())
    }
    
    /// Lock migration manager (no more migrations can be registered)
    pub fn lock(&mut self) {
        self.is_locked = true;
    }
}

// ============================================================================
// TIER CALCULATION HELPERS
// ============================================================================

/// Calculate tier based on stake amount
pub fn calculate_tier(stake_amount: u64) -> (&'static str, f64, f64, bool, f64) {
    if stake_amount >= 1_000_000 {
        // Diamond: 1M+
        ("diamond", 1.20, 0.30, true, 1.5)
    } else if stake_amount >= 100_000 {
        // Gold: 100K - 1M
        ("gold", 1.12, 0.20, true, 1.25)
    } else if stake_amount >= 10_000 {
        // Silver: 10K - 100K
        ("silver", 1.05, 0.10, false, 1.1)
    } else {
        // Bronze: 100 - 10K
        ("bronze", 1.0, 0.0, false, 1.0)
    }
}

/// Migrate a V1 stake to V2 format
pub fn migrate_stake_v1_to_v2(stake: &StakingMigrationV2) -> StakingMigrationV3 {
    let (tier, tier_multiplier, lockup_discount, priority_withdrawal, governance_weight) 
        = calculate_tier(stake.amount);
    
    StakingMigrationV3 {
        stake_id: stake.stake_id.clone(),
        user_address: stake.user_address.clone(),
        pool_id: stake.pool_id.clone(),
        amount: stake.amount,
        start_epoch: stake.start_epoch,
        lock_end_epoch: stake.lock_end_epoch,
        tier: tier.to_string(),
        tier_multiplier,
        auto_compound_enabled: false,
        compound_threshold: 100, // Default 100 tokens
        compound_frequency_epochs: 24, // Default daily
        reinvest_percentage: 0.8, // 80% default
        last_compound_epoch: 0,
        total_compounds: 0,
        total_compounded_amount: 0,
    }
}

// ============================================================================
// VALIDATOR BOND MIGRATION
// ============================================================================

/// Validator bond record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorBondMigration {
    pub validator_address: String,
    pub bonded_amount: u64,
    pub bond_lock_epochs: u64,
    pub bond_start_epoch: u64,
    pub slashing_coverage: f64,
    pub self_delegation_percentage: f64,
    pub migrated: bool,
}

/// Default slashing coverage based on tier
pub fn default_slashing_coverage(tier: &str) -> f64 {
    match tier {
        "diamond" => 0.75, // 75% coverage
        "gold" => 0.60,   // 60% coverage
        "silver" => 0.50,  // 50% coverage
        _ => 0.25,         // 25% coverage for bronze
    }
}

/// Default self-delegation percentage based on tier
pub fn default_self_delegation_pct(tier: &str) -> f64 {
    match tier {
        "diamond" => 0.25, // 25% self-delegation required
        "gold" => 0.20,    // 20%
        "silver" => 0.15,  // 15%
        _ => 0.10,         // 10% for bronze
    }
}

// ============================================================================
// MIGRATION RUNNER
// ============================================================================

/// Migration runner for executing migrations in order
pub struct MigrationRunner {
    pub manager: MigrationManager,
    pub migrations_to_run: Vec<String>,
}

impl MigrationRunner {
    /// Create new migration runner
    pub fn new() -> Self {
        let mut manager = MigrationManager::new();
        
        // Register all known migrations
        let _ = manager.register_migration("v2_tiered_rewards", "Add tiered rewards", 48000);
        let _ = manager.register_migration("v3_auto_compound", "Add auto-compound config", 48500);
        let _ = manager.register_migration("v4_validator_bonds", "Add validator bonding", 49000);
        
        MigrationRunner {
            manager,
            migrations_to_run: vec![
                "v2_tiered_rewards".to_string(),
                "v3_auto_compound".to_string(),
                "v4_validator_bonds".to_string(),
            ],
        }
    }
    
    /// Run all pending migrations
    pub fn run_pending(&mut self) -> Result<Vec<String>, &'static str> {
        let mut completed = Vec::new();
        
        for migration_id in &self.migrations_to_run {
            let status = self.manager.migrations.get(migration_id)
                .map(|m| m.status.clone())
                .unwrap_or(MigrationStatus::Pending);
            
            if status == MigrationStatus::Pending {
                self.manager.start_migration(migration_id)?;
                // In real implementation, would run migration here
                self.manager.complete_migration(migration_id, 0)?;
                completed.push(migration_id.clone());
            }
        }
        
        Ok(completed)
    }
    
    /// Get migration status
    pub fn get_status(&self) -> Vec<(&String, &MigrationRecord)> {
        self.manager.migrations.iter().collect()
    }
}

// ============================================================================
// POOL CONFIGURATIONS
// ============================================================================

/// Pool configuration for staking pools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub pool_id: String,
    pub name: String,
    pub description: String,
    pub reward_token: String,
    pub min_stake: u64,
    pub max_stake: Option<u64>,
    pub early_unstake_penalty: f64,
    pub tier_boost_enabled: bool,
    pub auto_compound_enabled: bool,
    pub governance_enabled: bool,
    pub max_validators: u32,
    pub current_validators: u32,
    pub total_staked: u64,
    pub annual_percentage_yield: f64,
    pub epoch_reward: u64,
    pub last_reward_epoch: u64,
    pub is_active: bool,
    pub is_locked: bool,
}

impl PoolConfig {
    /// Create new pool configuration
    pub fn new(
        pool_id: &str,
        name: &str,
        description: &str,
        reward_token: &str,
    ) -> Self {
        PoolConfig {
            pool_id: pool_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            reward_token: reward_token.to_string(),
            min_stake: 100,
            max_stake: None,
            early_unstake_penalty: 0.05,
            tier_boost_enabled: true,
            auto_compound_enabled: true,
            governance_enabled: true,
            max_validators: 100,
            current_validators: 0,
            total_staked: 0,
            annual_percentage_yield: 12.5,
            epoch_reward: 0,
            last_reward_epoch: 0,
            is_active: true,
            is_locked: false,
        }
    }

    /// Get effective APY including tier boosts
    pub fn get_effective_apy(&self, tier_multiplier: f64, lockup_discount: f64) -> f64 {
        let base_apy = self.annual_percentage_yield;
        let tier_boost = if self.tier_boost_enabled {
            (tier_multiplier - 1.0) * 100.0
        } else {
            0.0
        };
        let lockup_bonus = lockup_discount * 100.0;
        base_apy + tier_boost + lockup_bonus
    }

    /// Calculate unstake penalty based on lockup
    pub fn calculate_unstake_penalty(&self, amount: u64, epochs_locked: u64, current_epoch: u64) -> u64 {
        if current_epoch >= epochs_locked {
            return 0; // No penalty if lockup expired
        }
        let epochs_remaining = epochs_locked - current_epoch;
        let penalty_rate = self.early_unstake_penalty * (epochs_remaining as f64 / 100.0).min(1.0);
        (amount as f64 * penalty_rate) as u64
    }
}

/// Default pool configurations
pub fn get_default_pools() -> Vec<PoolConfig> {
    vec![
        {
            let mut pool = PoolConfig::new("aeth", "AeTHer Staking", "Stake AETH to become a validator and earn rewards", "AETH");
            pool.min_stake = 100_000;
            pool.max_stake = Some(10_000_000);
            pool.annual_percentage_yield = 12.5;
            pool.max_validators = 500;
            pool
        },
        {
            let mut pool = PoolConfig::new("flux", "FLUX Mining Pool", "Stake FLUX to support network operations", "FLUX");
            pool.min_stake = 1_000;
            pool.max_stake = Some(1_000_000);
            pool.annual_percentage_yield = 8.5;
            pool.max_validators = 1000;
            pool
        },
        {
            let mut pool = PoolConfig::new("lp", "LP Staking", "Stake liquidity provider tokens", "FLUX");
            pool.min_stake = 500;
            pool.max_stake = Some(500_000);
            pool.annual_percentage_yield = 15.0;
            pool.max_validators = 200;
            pool.early_unstake_penalty = 0.08;
            pool
        },
        {
            let mut pool = PoolConfig::new("delegation", "Delegation Pool", "Delegate to validators without running a node", "AETH");
            pool.min_stake = 100;
            pool.max_stake = Some(100_000);
            pool.annual_percentage_yield = 6.0;
            pool.max_validators = 10000;
            pool
        },
    ]
}

// ============================================================================
// SLASH REDISTRIBUTION
// ============================================================================

/// Slash event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashEvent {
    pub validator_address: String,
    pub slash_epoch: u64,
    pub slash_amount: u64,
    pub slash_reason: SlashReason,
    pub redistribution_batch: u64,
    pub redistributed: bool,
    pub redistribution_recipients: Vec<RedistributionRecipient>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlashReason {
    DoubleSign,
    Downtime,
    InvalidBlock,
    MissedVotes,
    Corruption,
}

/// Redistribution recipient record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedistributionRecipient {
    pub recipient_address: String,
    pub stake_amount: u64,
    pub redistribution_amount: u64,
    pub percentage_of_slashed: f64,
}

/// Slash redistribution pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashRedistributionPool {
    pub pool_id: String,
    pub total_slashed_pending: u64,
    pub total_redistributed: u64,
    pub redistribution_batch: u64,
    pub last_redistribution_epoch: u64,
    pub redistribution_schedule: RedistributionSchedule,
    pub affected_stakers: Vec<AffectedStaker>,
    pub redistribution_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedistributionSchedule {
    Immediate,
    EpochBatch { epochs_per_batch: u64 },
    Vesting { vesting_epochs: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedStaker {
    pub staker_address: String,
    pub original_stake: u64,
    pub slash_percentage: f64,
    pub pending_redistribution: u64,
    pub received_redistribution: u64,
}

impl SlashRedistributionPool {
    /// Create new redistribution pool
    pub fn new(pool_id: &str) -> Self {
        SlashRedistributionPool {
            pool_id: pool_id.to_string(),
            total_slashed_pending: 0,
            total_redistributed: 0,
            redistribution_batch: 0,
            last_redistribution_epoch: 0,
            redistribution_schedule: RedistributionSchedule::EpochBatch { epochs_per_batch: 24 },
            affected_stakers: Vec::new(),
            redistribution_percentage: 0.25, // 25% of slashed gets redistributed
        }
    }

    /// Add slashed amount to redistribution pool
    pub fn add_slashed(&mut self, slash_amount: u64, reason: &SlashReason) -> u64 {
        let redistributable = (slash_amount as f64 * self.redistribution_percentage) as u64;
        self.total_slashed_pending += redistributable;
        redistributable
    }

    /// Calculate redistribution for a staker
    pub fn calculate_redistribution(&self, staker: &AffectedStaker, total_active_stake: u64) -> u64 {
        if total_active_stake == 0 {
            return 0;
        }
        let staker_share = staker.stake_amount as f64 / total_active_stake as f64;
        (self.total_slashed_pending as f64 * staker_share) as u64
    }

    /// Process redistribution batch
    pub fn process_batch(&mut self, current_epoch: u64, total_active_stake: u64) -> Vec<RedistributionRecipient> {
        // Check if we should process based on schedule
        let should_process = match &self.redistribution_schedule {
            RedistributionSchedule::Immediate => true,
            RedistributionSchedule::EpochBatch { epochs_per_batch } => {
                current_epoch - self.last_redistribution_epoch >= *epochs_per_batch
            }
            RedistributionSchedule::Vesting { vesting_epochs } => {
                current_epoch - self.last_redistribution_epoch >= *vesting_epochs
            }
        };

        if !should_process || self.total_slashed_pending == 0 {
            return Vec::new();
        }

        let mut recipients = Vec::new();

        for staker in &mut self.affected_stakers {
            let redistribution = self.calculate_redistribution(staker, total_active_stake);
            if redistribution > 0 {
                recipients.push(RedistributionRecipient {
                    recipient_address: staker.staker_address.clone(),
                    stake_amount: staker.stake_amount,
                    redistribution_amount: redistribution,
                    percentage_of_slashed: redistribution as f64 / self.total_slashed_pending as f64,
                });
                staker.received_redistribution += redistribution;
                staker.pending_redistribution = staker.pending_redistribution.saturating_sub(redistribution);
            }
        }

        self.total_redistributed += recipients.iter().map(|r| r.redistribution_amount).sum::<u64>();
        self.total_slashed_pending = 0;
        self.last_redistribution_epoch = current_epoch;
        self.redistribution_batch += 1;

        recipients
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_tier() {
        assert_eq!(calculate_tier(500).0, "bronze");
        assert_eq!(calculate_tier(10_000).0, "silver");
        assert_eq!(calculate_tier(100_000).0, "gold");
        assert_eq!(calculate_tier(1_000_000).0, "diamond");
        
        assert_eq!(calculate_tier(5_000_000).0, "diamond");
        assert_eq!(calculate_tier(50_000).0, "silver");
    }

    #[test]
    fn test_tier_multipliers() {
        assert_eq!(calculate_tier(500).1, 1.0);
        assert_eq!(calculate_tier(50_000).1, 1.05);
        assert_eq!(calculate_tier(500_000).1, 1.12);
        assert_eq!(calculate_tier(5_000_000).1, 1.20);
    }

    #[test]
    fn test_migration_manager() {
        let mut manager = MigrationManager::new();
        
        assert!(manager.register_migration("test_1", "Test Migration", 100).is_ok());
        assert!(manager.register_migration("test_1", "Test Migration", 100).is_err()); // Duplicate
        
        assert!(manager.start_migration("test_1").is_ok());
        assert!(manager.complete_migration("test_1", 100).is_ok());
        
        let migration = manager.migrations.get("test_1").unwrap();
        assert_eq!(migration.status, MigrationStatus::Completed);
        assert_eq!(migration.records_migrated, 100);
    }

    #[test]
    fn test_stake_migration_v1_to_v3() {
        let v2_stake = StakingMigrationV2 {
            stake_id: "stake_001".to_string(),
            user_address: "0x1234".to_string(),
            pool_id: "aeth".to_string(),
            amount: 50_000,
            start_epoch: 48000,
            lock_end_epoch: 48500,
            tier: "silver".to_string(),
            tier_multiplier: 1.05,
            lockup_discount: 0.10,
            priority_withdrawal: false,
            governance_weight: 1.1,
        };
        
        let v3_stake = migrate_stake_v1_to_v2(&v2_stake);
        
        assert_eq!(v3_stake.tier, "silver");
        assert_eq!(v3_stake.tier_multiplier, 1.05);
        assert!(!v3_stake.auto_compound_enabled);
        assert_eq!(v3_stake.compound_threshold, 100);
        assert_eq!(v3_stake.reinvest_percentage, 0.8);
    }

    #[test]
    fn test_default_slashing_coverage() {
        assert_eq!(default_slashing_coverage("bronze"), 0.25);
        assert_eq!(default_slashing_coverage("silver"), 0.50);
        assert_eq!(default_slashing_coverage("gold"), 0.60);
        assert_eq!(default_slashing_coverage("diamond"), 0.75);
    }

    #[test]
    fn test_default_self_delegation() {
        assert_eq!(default_self_delegation_pct("bronze"), 0.10);
        assert_eq!(default_self_delegation_pct("silver"), 0.15);
        assert_eq!(default_self_delegation_pct("gold"), 0.20);
        assert_eq!(default_self_delegation_pct("diamond"), 0.25);
    }

    #[test]
    fn test_pool_config_creation() {
        let pool = PoolConfig::new("test", "Test Pool", "A test pool", "TEST");
        assert_eq!(pool.pool_id, "test");
        assert_eq!(pool.min_stake, 100);
        assert_eq!(pool.annual_percentage_yield, 12.5);
        assert!(pool.is_active);
    }

    #[test]
    fn test_pool_effective_apy() {
        let mut pool = PoolConfig::new("test", "Test", "Test", "TEST");
        pool.annual_percentage_yield = 10.0;
        pool.tier_boost_enabled = true;

        // Tier multiplier of 1.2 = 20% boost
        let apy = pool.get_effective_apy(1.2, 0.1);
        assert_eq!(apy, 10.0 + 20.0 + 10.0); // base + tier + lockup
    }

    #[test]
    fn test_early_unstake_penalty() {
        let pool = PoolConfig::new("test", "Test", "Test", "TEST");
        pool.early_unstake_penalty = 0.05;

        // 50 epochs remaining, penalty should be 2.5%
        let penalty = pool.calculate_unstake_penalty(10000, 50000, 49950);
        assert!(penalty > 0);

        // No penalty after lockup expires
        let no_penalty = pool.calculate_unstake_penalty(10000, 50000, 50000);
        assert_eq!(no_penalty, 0);
    }

    #[test]
    fn test_default_pools() {
        let pools = get_default_pools();
        assert_eq!(pools.len(), 4);
        assert_eq!(pools[0].pool_id, "aeth");
        assert_eq!(pools[1].pool_id, "flux");
        assert_eq!(pools[2].pool_id, "lp");
        assert_eq!(pools[3].pool_id, "delegation");
    }

    #[test]
    fn test_slash_redistribution_pool() {
        let mut pool = SlashRedistributionPool::new("test_pool");
        
        // Add slashed amount
        let redistributable = pool.add_slashed(1000, &SlashReason::DoubleSign);
        assert_eq!(redistributable, 250); // 25% of 1000
        
        assert_eq!(pool.total_slashed_pending, 250);
    }

    #[test]
    fn test_redistribution_calculation() {
        let mut pool = SlashRedistributionPool::new("test_pool");
        pool.total_slashed_pending = 1000;
        
        let staker = AffectedStaker {
            staker_address: "0x1234".to_string(),
            original_stake: 5000,
            slash_percentage: 0.0,
            pending_redistribution: 0,
            received_redistribution: 0,
        };
        
        // Staker has 50% of total stake
        let redistribution = pool.calculate_redistribution(&staker, 10000);
        assert_eq!(redistribution, 500);
    }

    #[test]
    fn test_batch_redistribution() {
        let mut pool = SlashRedistributionPool::new("test_pool");
        pool.total_slashed_pending = 1000;
        
        pool.affected_stakers.push(AffectedStaker {
            staker_address: "0x1234".to_string(),
            original_stake: 5000,
            slash_percentage: 0.0,
            pending_redistribution: 500,
            received_redistribution: 0,
        });
        
        // Process batch
        let recipients = pool.process_batch(100, 10000);
        
        assert!(!recipients.is_empty());
        assert_eq!(pool.total_redistributed, 500);
        assert_eq!(pool.total_slashed_pending, 0);
    }
}
