//! FLUX Token Contract
//! 
//! FLUX is the utility token used for:
//! - AI agent service payments
//! - Mobile mining rewards
//! - Network transaction fees
//! - DeFi integration

use serde::{Deserialize, Serialize};

/// FLUX token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxToken {
    /// Total supply in smallest units (with decimals)
    pub total_supply: u64,
    
    /// Circulating supply
    pub circulating_supply: u64,
    
    /// Reserve for mining rewards
    pub mining_reserve: u64,
    
    /// Reserve for ecosystem growth
    pub ecosystem_reserve: u64,
    
    /// Whether transfers are paused
    pub paused: bool,
}

impl Default for FluxToken {
    fn default() -> Self {
        Self {
            total_supply: 0,
            circulating_supply: 0,
            mining_reserve: 0,
            ecosystem_reserve: 0,
            paused: false,
        }
    }
}

impl FluxToken {
    /// Create a new FLUX token instance
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Initialize the token with initial supply
    pub fn initialize(&mut self, initial_supply: u64) -> Result<(), super::error::TokenError> {
        if initial_supply > super::FLUX_MAX_SUPPLY {
            return Err(super::error::TokenError::ExceedsMaxSupply);
        }
        
        self.total_supply = initial_supply;
        self.circulating_supply = initial_supply;
        self.mining_reserve = initial_supply / 10; // 10% reserved for mining
        self.ecosystem_reserve = initial_supply / 20; // 5% for ecosystem
        Ok(())
    }
    
    /// Mint new tokens (for mining rewards)
    pub fn mint(&mut self, amount: u64) -> Result<(), super::error::TokenError> {
        let new_total = self.total_supply
            .checked_add(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        if new_total > super::FLUX_MAX_SUPPLY {
            return Err(super::error::TokenError::ExceedsMaxSupply);
        }
        
        self.total_supply = new_total;
        self.circulating_supply = self.circulating_supply
            .checked_add(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        Ok(())
    }
    
    /// Burn tokens
    pub fn burn(&mut self, amount: u64) -> Result<(), super::error::TokenError> {
        if amount > self.circulating_supply {
            return Err(super::error::TokenError::InsufficientBalance);
        }
        
        self.circulating_supply = self.circulating_supply
            .checked_sub(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        self.total_supply = self.total_supply
            .checked_sub(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        Ok(())
    }
    
    /// Transfer tokens
    pub fn transfer(
        &mut self, 
        from: &mut AccountState, 
        to: &mut AccountState, 
        amount: u64
    ) -> Result<(), super::error::TokenError> {
        if self.paused {
            return Err(super::error::TokenError::TokenFrozen);
        }
        
        from.balance = from.balance
            .checked_sub(amount)
            .ok_or(super::error::TokenError::InsufficientBalance)?;
            
        to.balance = to.balance
            .checked_add(amount)
            .ok_or(super::error::TokenError::Overflow)?;
            
        Ok(())
    }
    
    /// Calculate mining reward for a given uptime
    pub fn calculate_mining_reward(uptime_hours: u64, device_tier: u8) -> u64 {
        let base_reward = super::MINING_REWARD_PER_EPOCH;
        let tier_multiplier = match device_tier {
            1 => 100, // Mobile
            2 => 150, // Laptop  
            3 => 200, // Desktop
            _ => 100,
        };
        
        // Reward scales with uptime (capped at 24 hours)
        let uptime_factor = uptime_hours.min(24);
        
        base_reward
            .checked_mul(tier_multiplier as u64)
            .unwrap_or(base_reward)
            .checked_mul(uptime_factor)
            .unwrap_or(base_reward)
            .checked_div(100)
            .unwrap_or(base_reward)
    }
}

/// Account state for token holders
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountState {
    /// Token balance
    pub balance: u64,
    
    /// Pending mining rewards
    pub pending_rewards: u64,
    
    /// Last claim epoch
    pub last_claim_epoch: u64,
    
    /// Whether account is frozen (KYC required)
    pub frozen: bool,
}
