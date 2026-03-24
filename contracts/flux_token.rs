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
