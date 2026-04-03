//! Staking Contract for AeTHer Chain
//! 
//! Handles:
//! - Validator stake delegation
//! - Stake warmup/cooldown periods
//! - Reward distribution
//! - Slashing logic

use crate::utils::{self, Timestamp};
use core::cmp::min;

/// Stake status
#[derive(Debug, Clone, PartialEq)]
pub enum StakeStatus {
    Warmup,
    Active,
    Cooldown,
    Unlocked,
}

/// A stake entry
#[derive(Debug, Clone)]
pub struct StakeEntry {
    pub owner: [u8; 32],
    pub validator: [u8; 32],
    pub amount: u64,
    pub status: StakeStatus,
    pub start_epoch: u64,
    pub warmup_end: u64,
    pub cooldown_end: u64,
    pub last_reward_epoch: u64,
    pub accumulator: u64, // accumulated but not claimed
}

/// Staking contract configuration
#[derive(Debug, Clone)]
pub struct StakingConfig {
    /// Minimum stake amount
    pub min_stake: u64,
    /// Minimum delegation amount
    pub min_delegation: u64,
    /// Warmup epochs
    pub warmup_epochs: u64,
    /// Cooldown epochs
    pub cooldown_epochs: u64,
    /// Base reward rate (annual, in basis points)
    pub base_reward_bps: u64,
    /// Slash rate for downtime (basis points)
    pub downtime_slash_bps: u64,
    /// Slash rate for double sign (basis points)
    pub double_sign_slash_bps: u64,
    /// Maximum lock epochs for multiplier
    pub max_lock_epochs: u64,
}

impl Default for StakingConfig {
    fn default() -> Self {
        Self {
            min_stake: 10_000_000_000_000, // 10,000 ATH
            min_delegation: 100_000_000, // 100 ATH
            warmup_epochs: 2,
            cooldown_epochs: 4,
            base_reward_bps: 1250, // 12.5% APY
            downtime_slash_bps: 50, // 0.5%
            double_sign_slash_bps: 100, // 1%
            max_lock_epochs: 365,
        }
    }
}

/// Staking contract state
#[derive(Debug, Clone)]
pub struct StakingContract {
    pub config: StakingConfig,
    /// Current epoch
    pub current_epoch: u64,
    /// Total staked amount
    pub total_staked: u64,
    /// Validator total stakes: validator -> amount
    validator_stakes: std::collections::HashMap<[u8; 32], u64>,
    /// Stake entries: owner -> Vec<StakeEntry>
    stakes: std::collections::HashMap<[u8; 32], Vec<StakeEntry>>,
    /// Pending unstakes: owner -> Vec<(amount, cooldown_end)>
    pending_unstakes: std::collections::HashMap<[u8; 32], Vec<(u64, u64)>>,
}

impl StakingContract {
    /// Create new staking contract
    pub fn new(config: StakingConfig) -> Self {
        Self {
            config,
            current_epoch: 0,
            total_staked: 0,
            validator_stakes: std::collections::HashMap::new(),
            stakes: std::collections::HashMap::new(),
            pending_unstakes: std::collections::HashMap::new(),
        }
    }

    /// Stake tokens
    pub fn stake(
        &mut self,
        owner: &[u8; 32],
        validator: &[u8; 32],
        amount: u64,
    ) -> Result<(), &'static str> {
        if amount < self.config.min_stake {
            return Err("Below minimum stake amount");
        }

        let stake_entry = StakeEntry {
            owner: *owner,
            validator: *validator,
            amount,
            status: StakeStatus::Warmup,
            start_epoch: self.current_epoch,
            warmup_end: self.current_epoch + self.config.warmup_epochs,
            cooldown_end: 0,
            last_reward_epoch: self.current_epoch,
            accumulator: 0,
        };

        self.stakes
            .entry(*owner)
            .or_insert_with(Vec::new)
            .push(stake_entry);

        *self.validator_stakes.entry(*validator).or_insert(0) += amount;
        self.total_staked += amount;

        Ok(())
    }

    /// Begin unstake process
    pub fn begin_unstake(&mut self, owner: &[u8; 32], amount: u64) -> Result<(), &'static str> {
        let stakes = self.stakes.get_mut(owner).ok_or("No stakes found")?;
        
        // Find active stake to remove
        let mut remaining = amount;
        for stake in stakes.iter_mut() {
            if stake.status == StakeStatus::Active && stake.amount <= remaining {
                remaining -= stake.amount;
                stake.status = StakeStatus::Unlocked;
                stake.cooldown_end = self.current_epoch + self.config.cooldown_epochs;
                
                // Move to pending unstakes
                self.pending_unstakes
                    .entry(*owner)
                    .or_insert_with(Vec::new)
                    .push((stake.amount, stake.cooldown_end));
                
                // Update totals
                self.total_staked -= stake.amount;
                *self.validator_stakes.entry(stake.validator).or_insert(0) -= stake.amount;
            }
        }

        if remaining > 0 {
            return Err("Insufficient active stake");
        }

        Ok(())
    }

    /// Claim unstaked tokens (after cooldown)
    pub fn claim_unstake(&mut self, owner: &[u8; 32]) -> Result<u64, &'static str> {
        let pending = self.pending_unstakes.get_mut(owner).ok_or("No pending unstakes")?;
        let now = self.current_epoch;
        
        let mut claimable = 0u64;
        pending.retain(|(amount, cooldown_end)| {
            if *cooldown_end <= now {
                claimable += *amount;
                false
            } else {
                true
            }
        });

        Ok(claimable)
    }

    /// Calculate current reward for a stake
    pub fn calculate_reward(&self, stake: &StakeEntry) -> u64 {
        if stake.status != StakeStatus::Active {
            return 0;
        }

        let epochs = self.current_epoch.saturating_sub(stake.last_reward_epoch);
        if epochs == 0 {
            return 0;
        }

        // Base reward: amount * rate * epochs
        let annual_rate = self.config.base_reward_bps as f64 / 10000.0;
        let epoch_rate = annual_rate / (365.25 * 24.0); // Approximate epochs per year
        let epochs_f = epochs as f64;
        
        let reward = stake.amount as f64 * epoch_rate * epochs_f;
        reward as u64
    }

    /// Distribute rewards for an epoch
    pub fn distribute_rewards(&mut self, validator: &[u8; 32]) -> Result<u64, &'static str> {
        let total_validator_stake = self.validator_stakes.get(validator).copied().unwrap_or(0);
        if total_validator_stake == 0 {
            return Ok(0);
        }

        let epochs = self.current_epoch.saturating_sub(0); // Simplified
        if epochs == 0 {
            return Ok(0);
        }

        // Calculate and distribute rewards proportionally
        let reward_per_stake_unit = (self.config.base_reward_bps as f64 * epochs as f64 / 10000.0) 
            * 1_000_000_000.0; // Convert to nano-ATH

        let mut total_distributed = 0u64;
        for stakes in self.stakes.values_mut() {
            for stake in stakes.iter_mut() {
                if stake.validator == *validator && stake.status == StakeStatus::Active {
                    let reward = (stake.amount as f64 * reward_per_stake_unit / total_validator_stake as f64) as u64;
                    stake.accumulator += reward;
                    total_distributed += reward;
                }
            }
        }

        Ok(total_distributed)
    }

    /// Slash a validator's stake
    pub fn slash(&mut self, validator: &[u8; 32], reason: &str) -> Result<u64, &'static str> {
        let slash_bps = match reason {
            "downtime" => self.config.downtime_slash_bps,
            "double_sign" => self.config.double_sign_slash_bps,
            _ => return Err("Unknown slash reason"),
        };

        let total_validator_stake = self.validator_stakes.get(validator).copied().unwrap_or(0);
        let slash_amount = (total_validator_stake as f64 * slash_bps as f64 / 10000.0) as u64;

        // Apply slash proportionally to all stakers
        for stakes in self.stakes.values_mut() {
            for stake in stakes.iter_mut() {
                if stake.validator == *validator && stake.status == StakeStatus::Active {
                    let stake_slash = (stake.amount as f64 * slash_bps as f64 / 10000.0) as u64;
                    stake.amount = stake.amount.saturating_sub(stake_slash);
                }
            }
        }

        *self.validator_stakes.entry(*validator).or_insert(0) -= slash_amount;
        self.total_staked = self.total_staked.saturating_sub(slash_amount);

        Ok(slash_amount)
    }

    /// Advance epoch
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
        
        // Activate stakes that finished warmup
        for stakes in self.stakes.values_mut() {
            for stake in stakes.iter_mut() {
                if stake.status == StakeStatus::Warmup && stake.warmup_end <= self.current_epoch {
                    stake.status = StakeStatus::Active;
                }
            }
        }
    }

    /// Get total staked for an address
    pub fn total_staked_for(&self, owner: &[u8; 32]) -> u64 {
        self.stakes
            .get(owner)
            .map(|stakes| {
                stakes.iter()
                    .filter(|s| s.status == StakeStatus::Active || s.status == StakeStatus::Warmup)
                    .map(|s| s.amount)
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Get pending unstake amount
    pub fn pending_unstake_for(&self, owner: &[u8; 32]) -> u64 {
        self.pending_unstakes
            .get(owner)
            .map(|unstakes| unstakes.iter().map(|(a, _)| a).sum())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake_and_unstake() {
        let mut staking = StakingContract::new(StakingConfig::default());
        let owner = [1u8; 32];
        let validator = [2u8; 32];
        
        staking.stake(&owner, &validator, 1000_000_000_000).unwrap();
        assert_eq!(staking.total_staked, 1000_000_000_000);
        
        // Advance past warmup
        staking.advance_epoch();
        staking.advance_epoch();
        
        // Begin unstake
        staking.begin_unstake(&owner, 500_000_000_000).unwrap();
        assert_eq!(staking.total_staked, 500_000_000_000);
    }

    #[test]
    fn test_reward_accumulation() {
        let mut staking = StakingContract::new(StakingConfig::default());
        let owner = [1u8; 32];
        let validator = [2u8; 32];
        
        // Stake 1000 ATH
        staking.stake(&owner, &validator, 1000_000_000_000).unwrap();
        
        // Advance through warmup
        staking.advance_epoch();
        staking.advance_epoch();
        
        // Check stake is active
        let total = staking.total_staked_for(&owner);
        assert_eq!(total, 1000_000_000_000);
        
        // Distribute some rewards
        let distributed = staking.distribute_rewards(&validator).unwrap();
        assert!(distributed > 0, "Should distribute rewards");
    }

    #[test]
    fn test_slashing() {
        let mut staking = StakingContract::new(StakingConfig::default());
        let validator = [2u8; 32];
        let owner1 = [1u8; 32];
        let owner2 = [3u8; 32];
        
        // Two stakers
        staking.stake(&owner1, &validator, 1000_000_000_000).unwrap();
        staking.stake(&owner2, &validator, 1000_000_000_000).unwrap();
        
        // Advance through warmup
        staking.advance_epoch();
        staking.advance_epoch();
        
        let initial_total = staking.total_staked;
        
        // Slash for downtime
        staking.slash(&validator, "downtime").unwrap();
        
        // Total should be reduced
        assert!(staking.total_staked < initial_total);
    }

    #[test]
    fn test_multi_delegation() {
        let mut staking = StakingContract::new(StakingConfig::default());
        let owner = [1u8; 32];
        let validator1 = [2u8; 32];
        let validator2 = [3u8; 32];
        
        // Stake to two validators
        staking.stake(&owner, &validator1, 500_000_000_000).unwrap();
        staking.stake(&owner, &validator2, 500_000_000_000).unwrap();
        
        let total = staking.total_staked_for(&owner);
        assert_eq!(total, 1_000_000_000_000);
    }

    #[test]
    fn test_pending_unstake_claim() {
        let mut staking = StakingContract::new(StakingConfig::default());
        let owner = [1u8; 32];
        let validator = [2u8; 32];
        
        staking.stake(&owner, &validator, 1000_000_000_000).unwrap();
        
        // Advance through warmup
        staking.advance_epoch();
        staking.advance_epoch();
        
        // Begin unstake
        staking.begin_unstake(&owner, 1000_000_000_000).unwrap();
        
        // Can't claim yet (cooldown)
        let claimable = staking.claim_unstake(&owner).unwrap();
        assert_eq!(claimable, 0);
        
        // Advance through cooldown
        for _ in 0..4 {
            staking.advance_epoch();
        }
        
        // Now can claim
        let claimable = staking.claim_unstake(&owner).unwrap();
        assert_eq!(claimable, 1000_000_000_000);
    }

    #[test]
    fn test_min_stake_validation() {
        let mut staking = StakingContract::new(StakingConfig::default());
        let owner = [1u8; 32];
        let validator = [2u8; 32];
        
        // Below minimum
        let result = staking.stake(&owner, &validator, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_double_sign_slash_harsher() {
        let mut staking = StakingContract::new(StakingConfig::default());
        let validator = [2u8; 32];
        let owner = [1u8; 32];
        
        staking.stake(&owner, &validator, 10_000_000_000_000).unwrap();
        
        // Advance through warmup
        staking.advance_epoch();
        staking.advance_epoch();
        
        let initial = staking.total_staked;
        
        // Downtime slash (0.5%)
        let downtime_slash = staking.slash(&validator, "downtime").unwrap();
        
        // Double sign slash (1%) - should be more
        staking.advance_epoch();
        staking.advance_epoch();
        let double_sign_slash = staking.slash(&validator, "double_sign").unwrap();
        
        // Double sign should be ~2x downtime slash
        assert!(double_sign_slash > downtime_slash);
        assert!(staking.total_staked < initial);
    }
}
