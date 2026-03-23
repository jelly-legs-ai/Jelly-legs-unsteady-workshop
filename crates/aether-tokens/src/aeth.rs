//! AETH Token Contract (Governance Token)
//! 
//! AETH is the governance token used for:
//! - Validator staking
//! - DAO governance voting
//! - Network security (slashing collateral)

use serde::{Deserialize, Serialize};
use super::{VALIDATOR_MIN_STAKE, STAKING_APY_BASIS_POINTS};

/// AETH token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AethToken {
    /// Total supply
    pub total_supply: u64,
    
    /// Circulating supply
    pub circulating_supply: u64,
    
    /// Total staked amount
    pub total_staked: u64,
    
    /// Staking reward pool
    pub staking_pool: u64,
    
    /// Whether transfers are paused (for governance)
    pub paused: bool,
}

impl Default for AethToken {
    fn default() -> Self {
        Self {
            total_supply: 0,
            circulating_supply: 0,
            total_staked: 0,
            staking_pool: 0,
            paused: false,
        }
    }
}

impl AethToken {
    /// Create new AETH token instance
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Initialize with initial supply
    pub fn initialize(&mut self, initial_supply: u64) -> Result<(), super::error::TokenError> {
        if initial_supply > super::AETH_MAX_SUPPLY {
            return Err(super::error::TokenError::ExceedsMaxSupply);
        }
        
        self.total_supply = initial_supply;
        self.circulating_supply = initial_supply;
        self.staking_pool = initial_supply / 10; // 10% reserved for staking rewards
        Ok(())
    }
    
    /// Stake tokens for validation
    pub fn stake(
        &mut self, 
        account: &mut StakingAccount, 
        amount: u64
    ) -> Result<(), super::error::TokenError> {
        if amount < VALIDATOR_MIN_STAKE {
            return Err(super::error::TokenError::BelowMinStake);
        }
        
        if account.balance < amount {
            return Err(super::error::TokenError::InsufficientBalance);
        }
        
        account.balance = account.balance
            .checked_sub(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        account.staked_amount = account.staked_amount
            .checked_add(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        account.last_stake_epoch = account.current_epoch;
        
        self.total_staked = self.total_staked
            .checked_add(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        Ok(())
    }
    
    /// Unstake tokens
    pub fn unstake(
        &mut self,
        account: &mut StakingAccount,
        amount: u64
    ) -> Result<(), super::error::TokenError> {
        if amount > account.staked_amount {
            return Err(super::error::TokenError::InsufficientBalance);
        }
        
        // Check lock period (can't unstake within same epoch)
        if account.current_epoch == account.last_stake_epoch {
            return Err(super::error::TokenError::Unauthorized);
        }
        
        account.staked_amount = account.staked_amount
            .checked_sub(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        account.balance = account.balance
            .checked_add(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        self.total_staked = self.total_staked
            .checked_sub(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        Ok(())
    }
    
    /// Calculate staking rewards
    pub fn calculate_rewards(staked_amount: u64, epochs: u64) -> u64 {
        // APY in basis points, convert to per-epoch rate
        let epochs_per_year: u64 = 365 * 24; // Hourly epochs
        
        let yearly_reward = staked_amount
            .checked_mul(STAKING_APY_BASIS_POINTS as u64)
            .unwrap_or(staked_amount)
            .checked_div(10000)
            .unwrap_or(0);
            
        yearly_reward
            .checked_mul(epochs)
            .unwrap_or(yearly_reward)
            .checked_div(epochs_per_year)
            .unwrap_or(0)
    }
    
    /// Claim staking rewards
    pub fn claim_rewards(&mut self, account: &mut StakingAccount) -> Result<u64, super::error::TokenError> {
        let epochs_since_last_claim = account.current_epoch
            .saturating_sub(account.last_claim_epoch);
            
        if epochs_since_last_claim == 0 {
            return Ok(0);
        }
        
        let rewards = Self::calculate_rewards(account.staked_amount, epochs_since_last_claim);
        
        if rewards > self.staking_pool {
            return Err(super::error::TokenError::InsufficientRewards);
        }
        
        account.pending_rewards = account.pending_rewards
            .checked_add(rewards)
            .ok_or(super::error::TokenError::Overflow)?;
            
        account.last_claim_epoch = account.current_epoch;
        
        self.staking_pool = self.staking_pool
            .checked_sub(rewards)
            .ok_or(super::error::TokenError::Overflow)?;
            
        Ok(rewards)
    }
    
    /// Slash stake (for validator misbehavior)
    pub fn slash(
        &mut self,
        account: &mut StakingAccount,
        penalty_percent: u8
    ) -> Result<u64, super::error::TokenError> {
        let slash_amount = account.staked_amount
            .checked_mul(penalty_percent as u64)
            .unwrap_or(0)
            .checked_div(100)
            .unwrap_or(0);
            
        if slash_amount > 0 {
            account.staked_amount = account.staked_amount
                .checked_sub(slash_amount)
                .unwrap_or(0);
                
            self.total_staked = self.total_staked
                .checked_sub(slash_amount)
                .unwrap_or(self.total_staked);
        }
        
        Ok(slash_amount)
    }
}

/// Staking account state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StakingAccount {
    /// Token balance (unstaked)
    pub balance: u64,
    
    /// Staked amount
    pub staked_amount: u64,
    
    /// Pending rewards to claim
    pub pending_rewards: u64,
    
    /// Last stake epoch
    pub last_stake_epoch: u64,
    
    /// Last claim epoch
    pub last_claim_epoch: u64,
    
    /// Current epoch (for calculations)
    pub current_epoch: u64,
    
    /// Validator ID (if registered)
    pub validator_id: Option<[u8; 32]>,
    
    /// Whether validator is active
    pub is_active: bool,
}
