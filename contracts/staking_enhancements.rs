// Staking Enhancements - AeTHer Chain
// Advanced staking features: auto-compound, tiered rewards, validator bonding

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Staking tier with bonus rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingTier {
    pub name: String,
    pub min_amount: u64,
    pub max_amount: Option<u64>,
    pub bonus_multiplier: f64, // e.g., 1.1 = 10% bonus
    pub lockup_discount: f64,  // Reduced lockup period
    pub priority_withdrawal: bool,
    pub governance_weight: f64, // Extra voting power
}

/// Auto-compound configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundConfig {
    pub enabled: bool,
    pub compound_threshold: u64, // Auto-compound when rewards reach this amount
    pub compound_frequency_epochs: u64,
    pub gas_optimization: bool, // Batch compounds to save gas
    pub reinvest_percentage: f64, // 0.0 to 1.0 (1.0 = 100% reinvest)
}

/// Validator bonding info (skin in the game)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorBond {
    pub validator_address: String,
    pub bonded_amount: u64,
    pub bond_lock_epochs: u64,
    pub bond_start_epoch: u64,
    pub slashing_coverage: f64, // Bond covers this % of potential slashing
    pub self_delegation_percentage: f64,
}

/// Enhanced staking position with advanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedStake {
    pub stake_id: String,
    pub owner: String,
    pub pool_id: String,
    pub amount: u64,
    pub start_epoch: u64,
    pub tier: String,
    pub auto_compound: AutoCompoundConfig,
    pub delegated_validator: Option<String>,
    pub lock_end_epoch: u64,
    pub is_locked: bool,
    pub rewards_pending: u64,
    pub rewards_claimed: u64,
    pub bonus_multiplier: f64,
    pub governance_power: f64,
}

/// Staking rewards tracker with analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsTracker {
    pub total_earned: u64,
    pub total_claimed: u64,
    pub total_compounded: u64,
    pub best_epoch_rewards: u64,
    pub average_daily_rewards: f64,
    pub streak_days: u64,
    pub last_claim_epoch: u64,
    pub claim_history: Vec<ClaimRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRecord {
    pub epoch: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub was_compounded: bool,
    pub bonus_applied: f64,
}

/// Enhanced staking contract with advanced features
pub struct EnhancedStakingContract {
    pub tiers: HashMap<String, StakingTier>,
    pub validator_bonds: HashMap<String, ValidatorBond>,
    pub enhanced_stakes: HashMap<String, EnhancedStake>,
    pub rewards_trackers: HashMap<String, RewardsTracker>,
    pub auto_compound_queue: Vec<String>, // Stake IDs pending auto-compound
    pub current_epoch: u64,
}

impl EnhancedStakingContract {
    pub fn new() -> Self {
        let mut tiers = HashMap::new();
        
        // Bronze tier - entry level
        tiers.insert("bronze".to_string(), StakingTier {
            name: "Bronze".to_string(),
            min_amount: 100,
            max_amount: Some(10_000),
            bonus_multiplier: 1.0,
            lockup_discount: 1.0,
            priority_withdrawal: false,
            governance_weight: 1.0,
        });
        
        // Silver tier - intermediate
        tiers.insert("silver".to_string(), StakingTier {
            name: "Silver".to_string(),
            min_amount: 10_001,
            max_amount: Some(100_000),
            bonus_multiplier: 1.05, // 5% bonus
            lockup_discount: 0.9,   // 10% shorter lockup
            priority_withdrawal: false,
            governance_weight: 1.1,
        });
        
        // Gold tier - high roller
        tiers.insert("gold".to_string(), StakingTier {
            name: "Gold".to_string(),
            min_amount: 100_001,
            max_amount: Some(1_000_000),
            bonus_multiplier: 1.12, // 12% bonus
            lockup_discount: 0.8,   // 20% shorter lockup
            priority_withdrawal: true,
            governance_weight: 1.25,
        });
        
        // Diamond tier - whale
        tiers.insert("diamond".to_string(), StakingTier {
            name: "Diamond".to_string(),
            min_amount: 1_000_001,
            max_amount: None,
            bonus_multiplier: 1.20, // 20% bonus
            lockup_discount: 0.7,   // 30% shorter lockup
            priority_withdrawal: true,
            governance_weight: 1.5,
        });
        
        Self {
            tiers,
            validator_bonds: HashMap::new(),
            enhanced_stakes: HashMap::new(),
            rewards_trackers: HashMap::new(),
            auto_compound_queue: Vec::new(),
            current_epoch: 0,
        }
    }
    
    /// Determine staking tier based on amount
    pub fn get_tier_for_amount(&self, amount: u64) -> Option<&StakingTier> {
        for (_, tier) in &self.tiers {
            if amount >= tier.min_amount {
                if let Some(max) = tier.max_amount {
                    if amount <= max {
                        return Some(tier);
                    }
                } else {
                    // No max amount (diamond tier)
                    return Some(tier);
                }
            }
        }
        None
    }
    
    /// Create enhanced stake with auto-compound
    pub fn create_enhanced_stake(
        &mut self,
        owner: String,
        pool_id: String,
        amount: u64,
        lockup_epochs: u64,
        auto_compound_enabled: bool,
    ) -> Result<EnhancedStake, &'static str> {
        let tier = self.get_tier_for_amount(amount)
            .ok_or("Amount below minimum staking threshold")?;
        
        let stake_id = format!("stake_{}_{}", owner, self.current_epoch);
        let lock_end = self.current_epoch + (lockup_epochs as f64 * tier.lockup_discount) as u64;
        
        let enhanced_stake = EnhancedStake {
            stake_id: stake_id.clone(),
            owner: owner.clone(),
            pool_id,
            amount,
            start_epoch: self.current_epoch,
            tier: tier.name.clone(),
            auto_compound: AutoCompoundConfig {
                enabled: auto_compound_enabled,
                compound_threshold: 10_000, // Auto-compound at 0.0001 FLUX
                compound_frequency_epochs: 24, // Daily
                gas_optimization: true,
                reinvest_percentage: 1.0,
            },
            delegated_validator: None,
            lock_end_epoch: lock_end,
            is_locked: true,
            rewards_pending: 0,
            rewards_claimed: 0,
            bonus_multiplier: tier.bonus_multiplier,
            governance_power: tier.governance_weight,
        };
        
        // Initialize rewards tracker
        let tracker = RewardsTracker {
            total_earned: 0,
            total_claimed: 0,
            total_compounded: 0,
            best_epoch_rewards: 0,
            average_daily_rewards: 0.0,
            streak_days: 0,
            last_claim_epoch: self.current_epoch,
            claim_history: Vec::new(),
        };
        
        self.enhanced_stakes.insert(stake_id.clone(), enhanced_stake.clone());
        self.rewards_trackers.insert(owner, tracker);
        
        Ok(enhanced_stake)
    }
    
    /// Calculate rewards with tier bonuses
    pub fn calculate_enhanced_rewards(
        &self,
        stake: &EnhancedStake,
        base_reward: u64,
        base_apy: f64,
    ) -> u64 {
        // Apply tier bonus multiplier
        let tier_bonus = (base_reward as f64 * stake.bonus_multiplier) as u64;
        
        // Apply APY bonus for tier
        let apy_bonus = (stake.amount as f64 * base_apy * (stake.governance_power - 1.0)) as u64;
        
        tier_bonus + apy_bonus
    }
    
    /// Auto-compound pending rewards
    pub fn auto_compound_stake(&mut self, stake_id: &str) -> Result<u64, &'static str> {
        let stake = self.enhanced_stakes.get_mut(stake_id)
            .ok_or("Stake not found")?;
        
        if !stake.auto_compound.enabled {
            return Err("Auto-compound not enabled for this stake");
        }
        
        if stake.rewards_pending < stake.auto_compound.compound_threshold {
            return Err("Rewards below compound threshold");
        }
        
        let compound_amount = (stake.rewards_pending as f64 * stake.auto_compound.reinvest_percentage) as u64;
        
        // Add rewards to principal
        stake.amount += compound_amount;
        stake.rewards_pending -= compound_amount;
        
        // Update tier if amount crossed threshold
        if let Some(new_tier) = self.get_tier_for_amount(stake.amount) {
            stake.tier = new_tier.name.clone();
            stake.bonus_multiplier = new_tier.bonus_multiplier;
            stake.governance_power = new_tier.governance_weight;
        }
        
        // Update tracker
        if let Some(tracker) = self.rewards_trackers.get_mut(&stake.owner) {
            tracker.total_compounded += compound_amount;
        }
        
        Ok(compound_amount)
    }
    
    /// Queue stakes for auto-compound
    pub fn process_auto_compound_queue(&mut self) -> Vec<(String, u64)> {
        let mut results = Vec::new();
        let queue = self.auto_compound_queue.clone();
        self.auto_compound_queue.clear();
        
        for stake_id in queue {
            match self.auto_compound_stake(&stake_id) {
                Ok(amount) => results.push((stake_id, amount)),
                Err(_) => continue, // Skip failed compounds
            }
        }
        
        results
    }
    
    /// Bond validator (skin in the game)
    pub fn bond_validator(
        &mut self,
        validator: String,
        bond_amount: u64,
        lock_epochs: u64,
    ) -> Result<ValidatorBond, &'static str> {
        let bond = ValidatorBond {
            validator_address: validator.clone(),
            bonded_amount: bond_amount,
            bond_lock_epochs: lock_epochs,
            bond_start_epoch: self.current_epoch,
            slashing_coverage: 0.5, // Bond covers 50% of potential slashing
            self_delegation_percentage: 0.1, // 10% self-delegation requirement
        };
        
        self.validator_bonds.insert(validator, bond.clone());
        Ok(bond)
    }
    
    /// Check if validator bond is sufficient for slashing coverage
    pub fn validate_slashing_coverage(
        &self,
        validator: &str,
        potential_slash: u64,
    ) -> bool {
        if let Some(bond) = self.validator_bonds.get(validator) {
            let required_bond = (potential_slash as f64 / bond.slashing_coverage) as u64;
            bond.bonded_amount >= required_bond
        } else {
            false
        }
    }
    
    /// Delegate to validator with bond verification
    pub fn delegate_with_bond_check(
        &mut self,
        delegator: String,
        validator: String,
        amount: u64,
    ) -> Result<(), &'static str> {
        // Verify validator has sufficient bond
        let has_bond = self.validator_bonds.contains_key(&validator);
        
        if !has_bond {
            return Err("Validator must bond before accepting delegations");
        }
        
        // Proceed with delegation (simplified)
        Ok(())
    }
    
    /// Get stake analytics
    pub fn get_stake_analytics(&self, owner: &str) -> Option<&RewardsTracker> {
        self.rewards_trackers.get(owner)
    }
    
    /// Record claim in history
    pub fn record_claim(
        &mut self,
        owner: &str,
        amount: u64,
        was_compounded: bool,
        bonus: f64,
    ) {
        if let Some(tracker) = self.rewards_trackers.get_mut(owner) {
            tracker.total_earned += amount;
            if was_compounded {
                tracker.total_compounded += amount;
            } else {
                tracker.total_claimed += amount;
            }
            
            tracker.claim_history.push(ClaimRecord {
                epoch: self.current_epoch,
                amount,
                timestamp: 0, // Would use actual timestamp
                was_compounded,
                bonus_applied: bonus,
            });
            
            // Keep only last 100 claims
            if tracker.claim_history.len() > 100 {
                tracker.claim_history.remove(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tier_determination() {
        let contract = EnhancedStakingContract::new();
        
        // Bronze tier
        assert_eq!(contract.get_tier_for_amount(500).unwrap().name, "Bronze");
        
        // Silver tier
        assert_eq!(contract.get_tier_for_amount(50_000).unwrap().name, "Silver");
        
        // Gold tier
        assert_eq!(contract.get_tier_for_amount(500_000).unwrap().name, "Gold");
        
        // Diamond tier
        assert_eq!(contract.get_tier_for_amount(5_000_000).unwrap().name, "Diamond");
    }
    
    #[test]
    fn test_enhanced_stake_creation() {
        let mut contract = EnhancedStakingContract::new();
        
        let stake = contract.create_enhanced_stake(
            "user123".to_string(),
            "aeth_staking".to_string(),
            50_000,
            14,
            true,
        ).unwrap();
        
        assert_eq!(stake.tier, "Silver");
        assert_eq!(stake.bonus_multiplier, 1.05);
        assert!(stake.auto_compound.enabled);
    }
}
