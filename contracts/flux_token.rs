// FLUX Token Contract - AeTHer Chain
// Utility token for AI agent services, transaction fees, and mining rewards

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FLUX token contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxTokenContract {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_supply: u64,
    pub balances: HashMap<String, u64>,
    pub allowances: HashMap<String, HashMap<String, u64>>,
    pub minting_enabled: bool,
    pub mint_cap: u64,
    pub minted_amount: u64,
    pub burn_address: String,
    pub contract_version: String,
}

impl FluxTokenContract {
    /// Create new FLUX token contract
    pub fn new() -> Self {
        let mut balances = HashMap::new();
        // Pre-mine initial supply for distribution
        balances.insert("treasury".to_string(), 500_000_000);
        balances.insert("mining_rewards".to_string(), 300_000_000);
        balances.insert("team_allocation".to_string(), 100_000_000);
        balances.insert("ecosystem_fund".to_string(), 100_000_000);
        
        FluxTokenContract {
            name: "FLUX Token".to_string(),
            symbol: "FLUX".to_string(),
            decimals: 8,
            total_supply: 1_000_000_000, // 1 billion FLUX
            circulating_supply: 0,
            burned_supply: 0,
            balances,
            allowances: HashMap::new(),
            minting_enabled: false,
            mint_cap: 1_500_000_000, // Max 1.5B FLUX ever
            minted_amount: 0,
            burn_address: "0x000000000000000000000000000000000000dead".to_string(),
            contract_version: "1.0.0".to_string(),
        }
    }
    
    /// Transfer tokens
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), &'static str> {
        if from == to {
            return Err("Cannot transfer to self");
        }
        
        let from_balance = self.balances.get(from).copied().unwrap_or(0);
        if from_balance < amount {
            return Err("Insufficient balance");
        }
        
        *self.balances.entry(from.to_string()).or_insert(0) -= amount;
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
        
        Ok(())
    }
    
    /// Approve spender to use tokens
    pub fn approve(&mut self, owner: &str, spender: &str, amount: u64) -> Result<(), &'static str> {
        self.allowances
            .entry(owner.to_string())
            .or_insert_with(HashMap::new)
            .insert(spender.to_string(), amount);
        Ok(())
    }
    
    /// Transfer from approved allowance
    pub fn transfer_from(&mut self, owner: &str, spender: &str, to: &str, amount: u64) -> Result<(), &'static str> {
        let owner_allowances = self.allowances.get_mut(owner)
            .ok_or("No allowances set")?;
        
        let allowance = owner_allowances.get_mut(spender)
            .ok_or("No allowance for spender")?;
        
        if *allowance < amount {
            return Err("Allowance exceeded");
        }
        
        self.transfer(owner, to, amount)?;
        *allowance -= amount;
        
        Ok(())
    }
    
    /// Mint new FLUX tokens (only if enabled)
    pub fn mint(&mut self, to: &str, amount: u64) -> Result<(), &'static str> {
        if !self.minting_enabled {
            return Err("Minting is disabled");
        }
        
        if self.minted_amount + amount > self.mint_cap {
            return Err("Mint cap exceeded");
        }
        
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
        self.total_supply += amount;
        self.minted_amount += amount;
        
        Ok(())
    }
    
    /// Burn FLUX tokens
    pub fn burn(&mut self, from: &str, amount: u64) -> Result<(), &'static str> {
        let balance = self.balances.get(from).copied().unwrap_or(0);
        if balance < amount {
            return Err("Insufficient balance to burn");
        }
        
        *self.balances.get_mut(from).unwrap() -= amount;
        self.burned_supply += amount;
        self.circulating_supply = self.circulating_supply.saturating_sub(amount);
        
        // Transfer to burn address (effectively remove from circulation)
        *self.balances.entry(self.burn_address.clone()).or_insert(0) += amount;
        
        Ok(())
    }
    
    /// Get balance of an address
    pub fn balance_of(&self, address: &str) -> u64 {
        *self.balances.get(address).unwrap_or(&0)
    }
    
    /// Get allowance for spender
    pub fn allowance(&self, owner: &str, spender: &str) -> u64 {
        self.allowances
            .get(owner)
            .and_then(|spenders| spenders.get(spender))
            .copied()
            .unwrap_or(0)
    }
    
    /// Calculate FLUX in circulation (excluding burn address)
    pub fn calculate_circulating_supply(&self) -> u64 {
        let burn_balance = self.balance_of(&self.burn_address);
        self.total_supply - burn_balance
    }
    
    /// Get token stats summary
    pub fn get_token_stats(&self) -> TokenStats {
        TokenStats {
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            total_supply: self.total_supply,
            circulating_supply: self.calculate_circulating_supply(),
            burned_supply: self.burned_supply,
            minted_amount: self.minted_amount,
            mint_cap: self.mint_cap,
            minting_enabled: self.minting_enabled,
            holder_count: self.balances.len(),
            contract_version: self.contract_version.clone(),
        }
    }
    
    /// Enable/disable minting
    pub fn set_minting_enabled(&mut self, enabled: bool) {
        self.minting_enabled = enabled;
    }
    
    /// Distribute mining rewards
    pub fn distribute_mining_rewards(&mut self, miners: &[&str], rewards_per_miner: u64) -> Result<u64, &'static str> {
        let mining_balance = self.balance_of("mining_rewards");
        let total_needed = miners.len() as u64 * rewards_per_miner;
        
        if mining_balance < total_needed {
            return Err("Insufficient mining rewards balance");
        }
        
        for miner in miners {
            self.transfer("mining_rewards", miner, rewards_per_miner)?;
        }
        
        Ok(total_needed)
    }
    
    /// Calculate FLUX per USD at given price
    pub fn flux_per_usd(&self, price_usd: f64) -> f64 {
        if price_usd <= 0.0 {
            return 0.0;
        }
        1.0 / price_usd
    }
    
    /// Calculate USD value of FLUX amount
    pub fn usd_value(&self, amount: u64, price_usd: f64) -> f64 {
        amount as f64 * price_usd
    }
    
    // =============================================================================
    // STAKING REWARDS DISTRIBUTION - Sprint 11 Enhancement
    // =============================================================================
    
    /// Distribute staking rewards to eligible addresses
    pub fn distribute_staking_rewards(&mut self, stakers: &[&str], rewards_per_staker: u64) -> Result<u64, &'static str> {
        let treasury_balance = self.balance_of("treasury");
        let total_needed = stakers.len() as u64 * rewards_per_staker;
        
        if treasury_balance < total_needed {
            return Err("Insufficient treasury balance for rewards");
        }
        
        for staker in stakers {
            self.transfer("treasury", *staker, rewards_per_staker)?;
        }
        
        Ok(total_needed)
    }
    
    /// Calculate staking APY based on total staked and rewards rate
    pub fn calculate_staking_apy(&self, total_staked: u64, annual_rewards: u64) -> f64 {
        if total_staked == 0 {
            return 0.0;
        }
        (annual_rewards as f64 / total_staked as f64) * 100.0
    }
    
    /// Get reward per epoch for staking pool
    pub fn get_epoch_reward_rate(&self, pool_id: &str, total_staked: u64) -> u64 {
        // Base rate: 15% APY / 365 days / 24 epochs
        let base_rate = 0.15 / 365.0 / 24.0;
        (total_staked as f64 * base_rate) as u64
    }
    
    /// Calculate compound staking rewards (rewards reinvested)
    pub fn calculate_compound_staking_rewards(&self, principal: u64, apy: f64, epochs: u64) -> u64 {
        let epoch_rate = apy / 365.0 / 24.0;
        let compounded = (principal as f64 * (1.0 + epoch_rate).powf(epochs as f64)) as u64;
        compounded - principal
    }
    
    /// Get staking rewards tier based on stake amount
    pub fn get_rewards_tier(&self, stake_amount: u64) -> RewardsTier {
        if stake_amount >= 100_000 {
            RewardsTier::Platinum
        } else if stake_amount >= 50_000 {
            RewardsTier::Gold
        } else if stake_amount >= 10_000 {
            RewardsTier::Silver
        } else {
            RewardsTier::Bronze
        }
    }
    
    /// Calculate tier bonus multiplier
    pub fn get_tier_bonus(&self, tier: &RewardsTier) -> f64 {
        match tier {
            RewardsTier::Platinum => 1.5, // 50% bonus
            RewardsTier::Gold => 1.25,    // 25% bonus
            RewardsTier::Silver => 1.1,   // 10% bonus
            RewardsTier::Bronze => 1.0,   // No bonus
        }
    }
    
    /// Lock tokens for staking (transfer to staking contract)
    pub fn lock_for_staking(&mut self, from: &str, amount: u64, lock_epochs: u64) -> Result<StakingLock, &'static str> {
        let balance = self.balance_of(from);
        if balance < amount {
            return Err("Insufficient balance");
        }
        
        // Transfer to staking contract address
        self.transfer(from, "staking_contract", amount)?;
        
        let lock = StakingLock {
            lock_id: format!("lock_{}_{}", from, self.circulating_supply),
            owner: from.to_string(),
            amount,
            locked_at: self.circulating_supply, // Using supply as epoch proxy
            unlock_at: self.circulating_supply + lock_epochs,
            is_active: true,
        };
        
        Ok(lock)
    }
    
    /// Unlock staked tokens after lock period
    pub fn unlock_staking(&mut self, lock_id: &str) -> Result<u64, &'static str> {
        // In production, would query actual staking contract
        // This is a stub for the interface
        Ok(0)
    }
    
    /// Slash staked tokens for misbehavior
    pub fn slash_staking(&mut self, lock_id: &str, slash_percent: f64) -> Result<u64, &'static str> {
        // In production, would interact with slashing contract
        Ok(0)
    }
    
    /// Get total staking rewards distributed
    pub fn total_staking_rewards_distributed(&self) -> u64 {
        let staking_balance = self.balance_of("staking_contract");
        let treasury_balance = self.balance_of("treasury");
        let mining_balance = self.balance_of("mining_rewards");
        
        // Rewards distributed = initial treasury - current treasury
        500_000_000 - treasury_balance - staking_balance - mining_balance
    }
    
    /// Calculate inflation rate based on minting
    pub fn calculate_inflation_rate(&self) -> f64 {
        if self.total_supply == 0 {
            return 0.0;
        }
        (self.minted_amount as f64 / self.total_supply as f64) * 100.0
    }
    
    /// Get token holder distribution stats
    pub fn get_holder_distribution(&self) -> HolderDistribution {
        let mut distribution = HolderDistribution {
            whales: 0,      // > 1M FLUX
            large: 0,       // 100K - 1M
            medium: 0,      // 10K - 100K
            small: 0,       // 1K - 10K
            micro: 0,       // < 1K
            total_holders: self.balances.len(),
        };
        
        for balance in self.balances.values() {
            if *balance >= 1_000_000 {
                distribution.whales += 1;
            } else if *balance >= 100_000 {
                distribution.large += 1;
            } else if *balance >= 10_000 {
                distribution.medium += 1;
            } else if *balance >= 1_000 {
                distribution.small += 1;
            } else {
                distribution.micro += 1;
            }
        }
        
        distribution
    }
    
    /// Check if address is eligible for airdrop
    pub fn is_airdrop_eligible(&self, address: &str, min_balance: u64, active_epochs: u64) -> bool {
        let balance = self.balance_of(address);
        balance >= min_balance
        // In production, would also check active_epochs
    }
    
    /// Distribute airdrop to eligible addresses
    pub fn distribute_airdrop(&mut self, eligible_addresses: &[&str], amount_per_address: u64) -> Result<u64, &'static str> {
        let ecosystem_balance = self.balance_of("ecosystem_fund");
        let total_needed = eligible_addresses.len() as u64 * amount_per_address;
        
        if ecosystem_balance < total_needed {
            return Err("Insufficient ecosystem fund balance");
        }
        
        for address in eligible_addresses {
            self.transfer("ecosystem_fund", *address, amount_per_address)?;
        }
        
        Ok(total_needed)
    }
    
    /// Calculate token velocity (transfers per epoch)
    pub fn calculate_token_velocity(&self, transfers_per_epoch: u64, circulating_supply: u64) -> f64 {
        if circulating_supply == 0 {
            return 0.0;
        }
        transfers_per_epoch as f64 / circulating_supply as f64
    }
    
    /// Get token supply breakdown
    pub fn get_supply_breakdown(&self) -> SupplyBreakdown {
        SupplyBreakdown {
            total_supply: self.total_supply,
            circulating_supply: self.calculate_circulating_supply(),
            burned_supply: self.burned_supply,
            treasury_balance: self.balance_of("treasury"),
            mining_rewards_balance: self.balance_of("mining_rewards"),
            team_balance: self.balance_of("team_allocation"),
            ecosystem_balance: self.balance_of("ecosystem_fund"),
            staking_contract_balance: self.balance_of("staking_contract"),
            locked_supply: self.balance_of("staking_contract"),
            liquid_supply: self.calculate_circulating_supply() - self.balance_of("staking_contract"),
        }
    }
}

/// Staking lock record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingLock {
    pub lock_id: String,
    pub owner: String,
    pub amount: u64,
    pub locked_at: u64,
    pub unlock_at: u64,
    pub is_active: bool,
}

/// Rewards tier based on stake amount
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RewardsTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

/// Token holder distribution stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolderDistribution {
    pub whales: u64,
    pub large: u64,
    pub medium: u64,
    pub small: u64,
    pub micro: u64,
    pub total_holders: usize,
}

/// Token supply breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyBreakdown {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_supply: u64,
    pub treasury_balance: u64,
    pub mining_rewards_balance: u64,
    pub team_balance: u64,
    pub ecosystem_balance: u64,
    pub staking_contract_balance: u64,
    pub locked_supply: u64,
    pub liquid_supply: u64,
}

/// Token statistics for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub name: String,
    pub symbol: String,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_supply: u64,
    pub minted_amount: u64,
    pub mint_cap: u64,
    pub minting_enabled: bool,
    pub holder_count: usize,
    pub contract_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_creation() {
        let contract = FluxTokenContract::new();
        assert_eq!(contract.total_supply, 1_000_000_000);
        assert_eq!(contract.symbol, "FLUX");
        assert_eq!(contract.decimals, 8);
    }
    
    #[test]
    fn test_transfer() {
        let mut contract = FluxTokenContract::new();
        contract.transfer("treasury", "user1", 1000).unwrap();
        assert_eq!(contract.balance_of("user1"), 1000);
        assert!(contract.balance_of("treasury") < 500_000_000);
    }
    
    #[test]
    fn test_burn() {
        let mut contract = FluxTokenContract::new();
        contract.transfer("treasury", "user1", 1000).unwrap();
        contract.burn("user1", 500).unwrap();
        assert_eq!(contract.balance_of("user1"), 500);
        assert_eq!(contract.burned_supply, 500);
    }
}
