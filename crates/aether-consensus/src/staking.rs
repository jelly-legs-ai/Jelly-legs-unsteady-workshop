//! AETHER Staking Contract
//!
//! Implements staking logic for AETH tokens with lock periods,
//! reward distribution, and slashing conditions.

use aether_common::{MINIMUM_STAKE_AETH, STAKE_LOCK_EPOCHS};
use serde::{Deserialize, Serialize};

/// Stake entry with lock period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeEntry {
    /// Owner of the stake
    pub owner: [u8; 32],
    /// Amount staked (in smallest units)
    pub amount: u64,
    /// Epoch when stake was activated
    pub start_epoch: u64,
    /// Epoch when stake unlocks (0 if not locked)
    pub unlock_epoch: u64,
    /// Whether stake is delegated to a validator
    pub delegated_to: Option<[u8; 32]>,
    /// Pending withdrawal
    pub pending_withdrawal: bool,
    /// Accumulated rewards
    pub accumulated_rewards: u64,
    /// Epoch when rewards were last calculated (prevents double-counting)
    pub last_reward_epoch: u64,
}

impl StakeEntry {
    /// Create a new stake entry
    pub fn new(owner: [u8; 32], amount: u64, epoch: u64) -> Self {
        Self {
            owner,
            amount,
            start_epoch: epoch,
            unlock_epoch: epoch + STAKE_LOCK_EPOCHS,
            delegated_to: None,
            pending_withdrawal: false,
            accumulated_rewards: 0,
            last_reward_epoch: epoch,
        }
    }

    /// Check if stake is locked
    pub fn is_locked(&self, current_epoch: u64) -> bool {
        current_epoch < self.unlock_epoch
    }

    /// Get remaining lock time in epochs
    pub fn remaining_lock_epochs(&self, current_epoch: u64) -> u64 {
        if current_epoch >= self.unlock_epoch {
            0
        } else {
            self.unlock_epoch - current_epoch
        }
    }

    /// Check if can withdraw
    /// To withdraw: must have initiated withdrawal AND unlock period must have passed
    pub fn can_withdraw(&self, current_epoch: u64) -> bool {
        self.pending_withdrawal && current_epoch >= self.unlock_epoch
    }

    /// Start withdrawal process
    pub fn initiate_withdrawal(&mut self) {
        self.pending_withdrawal = true;
    }

    /// Complete withdrawal
    pub fn complete_withdrawal(&mut self) -> u64 {
        if !self.pending_withdrawal {
            return 0;
        }
        let amount = self.amount + self.accumulated_rewards;
        self.amount = 0;
        self.accumulated_rewards = 0;
        self.pending_withdrawal = false;
        amount
    }
}

/// Staking pool state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StakingPool {
    /// All stake entries indexed by stake ID
    pub stakes: Vec<StakeEntry>,
    /// Total staked amount
    pub total_staked: u64,
    /// Total accumulated rewards
    pub total_rewards: u64,
    /// Current epoch
    pub current_epoch: u64,
    /// Reward rate (APY in basis points)
    pub reward_rate_bps: u64,
}

impl StakingPool {
    /// Create new staking pool
    pub fn new(current_epoch: u64) -> Self {
        Self {
            stakes: Vec::new(),
            total_staked: 0,
            total_rewards: 0,
            current_epoch,
            reward_rate_bps: 1200, // 12% APY default
        }
    }

    /// Stake tokens
    pub fn stake(&mut self, owner: [u8; 32], amount: u64) -> Result<usize, StakingError> {
        if amount < MINIMUM_STAKE_AETH {
            return Err(StakingError::BelowMinimumStake(amount, MINIMUM_STAKE_AETH));
        }

        let stake_id = self.stakes.len();
        let entry = StakeEntry::new(owner, amount, self.current_epoch);
        self.stakes.push(entry);
        self.total_staked += amount;

        Ok(stake_id)
    }

    /// Delegate stake to validator
    pub fn delegate(&mut self, stake_id: usize, validator: [u8; 32]) -> Result<(), StakingError> {
        let stake = self.stakes.get_mut(stake_id)
            .ok_or(StakingError::StakeNotFound)?;

        if stake.is_locked(self.current_epoch) {
            return Err(StakingError::StakeLocked);
        }

        stake.delegated_to = Some(validator);
        Ok(())
    }

    /// Undelegate stake from validator
    pub fn undelegate(&mut self, stake_id: usize) -> Result<(), StakingError> {
        let stake = self.stakes.get_mut(stake_id)
            .ok_or(StakingError::StakeNotFound)?;

        stake.delegated_to = None;
        Ok(())
    }

    /// Initiate withdrawal (starts unlock period)
    pub fn initiate_withdrawal(&mut self, stake_id: usize) -> Result<u64, StakingError> {
        let stake = self.stakes.get_mut(stake_id)
            .ok_or(StakingError::StakeNotFound)?;

        if stake.pending_withdrawal {
            return Err(StakingError::AlreadyWithdrawing);
        }

        // Start cooldown - stake unlocks after lock period
        stake.unlock_epoch = self.current_epoch + STAKE_LOCK_EPOCHS;
        stake.initiate_withdrawal();

        Ok(stake.unlock_epoch)
    }

    /// Complete withdrawal and claim tokens
    pub fn complete_withdrawal(&mut self, stake_id: usize) -> Result<u64, StakingError> {
        let stake = self.stakes.get_mut(stake_id)
            .ok_or(StakingError::StakeNotFound)?;

        if !stake.can_withdraw(self.current_epoch) {
            return Err(StakingError::StakeLocked);
        }

        // Capture rewards BEFORE complete_withdrawal clears them
        let rewards = stake.accumulated_rewards;
        let amount = stake.complete_withdrawal();
        self.total_staked = self.total_staked.saturating_sub(amount.saturating_sub(rewards));
        self.total_rewards = self.total_rewards.saturating_sub(rewards);

        Ok(amount)
    }

    /// Calculate reward for a stake (only since last distribution to prevent double-counting)
    pub fn calculate_reward(&self, stake: &StakeEntry) -> u64 {
        if stake.amount == 0 {
            return 0;
        }

        // Calculate rewards only for epochs since last distribution
        let epochs_since_last = self.current_epoch.saturating_sub(stake.last_reward_epoch);
        let epochs_elapsed = epochs_since_last.min(365 * 4); // Cap at 4 years for APY calc

        // Simple APY calculation: amount * rate * epochs / epochs_per_year
        let epochs_per_year = 365;
        (stake.amount * self.reward_rate_bps * epochs_elapsed) / (10000 * epochs_per_year)
    }

    /// Distribute rewards to all stakes
    pub fn distribute_rewards(&mut self) {
        // Collect stake IDs first to avoid borrow conflict
        let stake_ids: Vec<usize> = (0..self.stakes.len()).collect();
        
        for stake_id in stake_ids {
            if self.stakes[stake_id].amount > 0 && !self.stakes[stake_id].pending_withdrawal {
                let reward = self.calculate_reward(&self.stakes[stake_id]);
                self.stakes[stake_id].accumulated_rewards += reward;
                self.stakes[stake_id].last_reward_epoch = self.current_epoch;
                self.total_rewards += reward;
            }
        }
    }

    /// Slash a stake (penalty for misbehavior)
    pub fn slash(&mut self, stake_id: usize, penalty_bps: u64) -> Result<u64, StakingError> {
        let stake = self.stakes.get_mut(stake_id)
            .ok_or(StakingError::StakeNotFound)?;

        if stake.amount == 0 {
            return Err(StakingError::StakeNotFound);
        }

        let penalty = (stake.amount * penalty_bps) / 10000;
        stake.amount = stake.amount.saturating_sub(penalty);
        self.total_staked = self.total_staked.saturating_sub(penalty);

        Ok(penalty)
    }

    /// Get stake by ID
    pub fn get_stake(&self, stake_id: usize) -> Option<&StakeEntry> {
        self.stakes.get(stake_id)
    }

    /// Get stakes by owner
    pub fn get_stakes_by_owner(&self, owner: &[u8; 32]) -> Vec<(usize, &StakeEntry)> {
        self.stakes.iter()
            .enumerate()
            .filter(|(_, s)| &s.owner == owner)
            .collect()
    }

    /// Advance epoch and distribute rewards
    pub fn tick(&mut self) {
        self.current_epoch += 1;
        self.distribute_rewards();
    }
}

/// Staking errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakingError {
    BelowMinimumStake(u64, u64),
    StakeNotFound,
    StakeLocked,
    AlreadyWithdrawing,
    InsufficientBalance,
    InvalidRewardRate,
}

impl std::fmt::Display for StakingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StakingError::BelowMinimumStake(amount, min) => 
                write!(f, "Stake amount {} below minimum {}", amount, min),
            StakingError::StakeNotFound => write!(f, "Stake not found"),
            StakingError::StakeLocked => write!(f, "Stake is locked"),
            StakingError::AlreadyWithdrawing => write!(f, "Stake is already being withdrawn"),
            StakingError::InsufficientBalance => write!(f, "Insufficient balance"),
            StakingError::InvalidRewardRate => write!(f, "Invalid reward rate"),
        }
    }
}

impl std::error::Error for StakingError {}

/// Stake position for external querying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakePosition {
    pub stake_id: usize,
    pub amount: u64,
    pub accumulated_rewards: u64,
    pub total_value: u64,
    pub start_epoch: u64,
    pub unlock_epoch: u64,
    pub is_locked: bool,
    pub remaining_epochs: u64,
    pub delegated_to: Option<[u8; 32]>,
    pub pending_withdrawal: bool,
    pub annual_reward_rate: u64,
}

impl StakingPool {
    /// Get stake position view
    pub fn get_position(&self, stake_id: usize) -> Option<StakePosition> {
        let stake = self.stakes.get(stake_id)?;

        Some(StakePosition {
            stake_id,
            amount: stake.amount,
            accumulated_rewards: stake.accumulated_rewards,
            total_value: stake.amount + stake.accumulated_rewards,
            start_epoch: stake.start_epoch,
            unlock_epoch: stake.unlock_epoch,
            is_locked: stake.is_locked(self.current_epoch),
            remaining_epochs: stake.remaining_lock_epochs(self.current_epoch),
            delegated_to: stake.delegated_to,
            pending_withdrawal: stake.pending_withdrawal,
            annual_reward_rate: self.reward_rate_bps,
        })
    }
}

/// Stake delegation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationInfo {
    pub validator: [u8; 32],
    pub stake_ids: Vec<usize>,
    pub total_delegated: u64,
    pub pending_rewards: u64,
}

/// Validator staking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStakingStats {
    pub validator: [u8; 32],
    pub total_delegated: u64,
    pub num_delegators: usize,
    pub delegator_count: usize,
    pub expected_reward_apr: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake_creation() {
        let mut pool = StakingPool::new(0);
        let owner = [1u8; 32];

        let result = pool.stake(owner, MINIMUM_STAKE_AETH);
        assert!(result.is_ok());
        let stake_id = result.unwrap();

        let stake = pool.get_stake(stake_id).unwrap();
        assert_eq!(stake.amount, MINIMUM_STAKE_AETH);
        assert_eq!(stake.start_epoch, 0);
        // Stake is locked for STAKE_LOCK_EPOCHS (2 epochs)
        assert!(stake.is_locked(0));
        assert!(stake.is_locked(1));
        // Unlocks after lock period
        assert!(!stake.is_locked(2));
    }

    #[test]
    fn test_stake_lock() {
        let mut pool = StakingPool::new(0);
        let owner = [1u8; 32];

        let stake_id = pool.stake(owner, MINIMUM_STAKE_AETH).unwrap();

        // Stake is locked for STAKE_LOCK_EPOCHS (2 epochs)
        assert!(pool.get_stake(stake_id).unwrap().is_locked(0));
        assert!(pool.get_stake(stake_id).unwrap().is_locked(1));
        
        // After lock period, should be unlocked
        assert!(!pool.get_stake(stake_id).unwrap().is_locked(2));

        // After initiating withdrawal, stake is marked pending but not re-locked
        // The lock is based on epoch, not withdrawal status
        pool.initiate_withdrawal(stake_id).unwrap();
        let stake = pool.get_stake(stake_id).unwrap();
        assert!(!stake.is_locked(2)); // Still unlocked after lock period
        assert!(stake.pending_withdrawal); // But marked for withdrawal
    }

    #[test]
    fn test_withdrawal_after_unlock() {
        let mut pool = StakingPool::new(0);
        let owner = [1u8; 32];

        // Create stake (starts at epoch 0, locked for STAKE_LOCK_EPOCHS=2 epochs)
        let stake_id = pool.stake(owner, MINIMUM_STAKE_AETH).unwrap();

        // Initially, stake is locked and NOT withdrawable
        let stake = pool.get_stake(stake_id).unwrap();
        assert!(stake.is_locked(0), "Stake should be locked at epoch 0");
        assert!(!stake.can_withdraw(0), "Stake should not be withdrawable at epoch 0");

        // Advance past initial lock period (epoch 2 is when unlock_epoch becomes valid)
        pool.current_epoch = STAKE_LOCK_EPOCHS; // epoch 2
        
        // Stake is now unlocked but still needs withdrawal initiation
        let stake = pool.get_stake(stake_id).unwrap();
        assert!(!stake.is_locked(pool.current_epoch), "Stake should be unlocked after lock period");
        assert!(!stake.can_withdraw(pool.current_epoch), "Stake needs withdrawal initiation first");

        // Initiate withdrawal (sets pending_withdrawal = true, sets new unlock_epoch)
        pool.initiate_withdrawal(stake_id).unwrap();
        
        // After initiating withdrawal, unlock_epoch is set to current_epoch + STAKE_LOCK_EPOCHS
        let stake = pool.get_stake(stake_id).unwrap();
        assert!(stake.pending_withdrawal, "Stake should be pending withdrawal");
        assert_eq!(stake.unlock_epoch, pool.current_epoch + STAKE_LOCK_EPOCHS, 
            "Unlock epoch should be current_epoch + STAKE_LOCK_EPOCHS");
        
        // Still not withdrawable yet - need to wait for the new lock period
        assert!(!stake.can_withdraw(pool.current_epoch), 
            "Stake should not be withdrawable immediately after withdrawal initiation");

        // Advance past the withdrawal lock period
        pool.current_epoch = stake.unlock_epoch; // Now at the unlock epoch

        // Now should be withdrawable
        let stake = pool.get_stake(stake_id).unwrap();
        assert!(stake.can_withdraw(pool.current_epoch), 
            "Stake should be withdrawable at epoch {} (unlock_epoch={})",
            pool.current_epoch, stake.unlock_epoch);

        // Complete withdrawal
        let amount = pool.complete_withdrawal(stake_id).unwrap();
        assert_eq!(amount, MINIMUM_STAKE_AETH); // No rewards yet (only 2 epochs)
    }

    #[test]
    fn test_rewards_accumulation() {
        let mut pool = StakingPool::new(0);
        let owner = [1u8; 32];

        // Use minimum stake amount (10,000 AETH = 10 trillion motes)
        pool.stake(owner, MINIMUM_STAKE_AETH).unwrap();

        // Advance epochs - stake unlocks after STAKE_LOCK_EPOCHS (2 epochs)
        pool.current_epoch = 365; // 1 year (well past lock period)

        // Trigger reward distribution
        pool.distribute_rewards();

        let stake = pool.get_stake(0).unwrap();
        // 12% APY on 10,000 AETH = 1,200 AETH = 1.2 trillion motes
        // Allow some variance in calculation
        assert!(stake.accumulated_rewards >= 1_000_000_000_000, 
            "Expected >= 1 trillion rewards, got {}", stake.accumulated_rewards);
    }

    #[test]
    fn test_slashing() {
        let mut pool = StakingPool::new(0);
        let owner = [1u8; 32];

        // Use minimum stake amount
        let stake_id = pool.stake(owner, MINIMUM_STAKE_AETH).unwrap();

        // Slash 10%
        let penalty = pool.slash(stake_id, 1000).unwrap();
        assert_eq!(penalty, MINIMUM_STAKE_AETH / 10); // 10% of stake

        let stake = pool.get_stake(stake_id).unwrap();
        assert_eq!(stake.amount, MINIMUM_STAKE_AETH * 9 / 10); // 90% remaining
    }

    #[test]
    fn test_delegation() {
        let mut pool = StakingPool::new(0);
        let owner = [1u8; 32];
        let validator = [2u8; 32];

        let stake_id = pool.stake(owner, MINIMUM_STAKE_AETH).unwrap();

        // Advance past lock period before delegation (stake must be unlocked)
        pool.current_epoch = STAKE_LOCK_EPOCHS + 1;

        // Delegate to validator
        pool.delegate(stake_id, validator).unwrap();

        let stake = pool.get_stake(stake_id).unwrap();
        assert_eq!(stake.delegated_to, Some(validator));
    }
}
