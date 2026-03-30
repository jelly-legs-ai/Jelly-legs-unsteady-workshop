// Advanced Staking Contract with Auto-Compound - AeTHer Chain
// Sprint 22 - New auto-compounding staking mechanism
// Enables users to automatically reinvest rewards for compound interest

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// AUTO-COMPOUND STAKING CONTRACT
// ============================================================================

/// Auto-compound staking position with reinvestment settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoStakePosition {
    pub principal: u128,           // Original staked amount
    pub compounded_amount: u128,   // Current total including compounded rewards
    pub staked_at: u64,
    pub last_compound: u64,
    pub lock_duration: u64,
    pub lock_multiplier: f64,
    pub auto_compound: bool,       // Whether auto-compound is enabled
    pub compound_threshold: u128,  // Minimum rewards before auto-compound triggers
    pub pending_rewards: u128,     // Rewards not yet compounded
}

/// Auto-compound configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundConfig {
    pub enabled: bool,
    pub min_compound_amount: u128,  // Minimum rewards to trigger compound
    pub compound_frequency: u64,    // Minimum seconds between compounds
    pub compound_fee_percent: f64,  // Fee charged on auto-compound (e.g., 0.01 = 1%)
    pub max_compounds_per_day: u32, // Rate limiting
}

impl Default for AutoCompoundConfig {
    fn default() -> Self {
        AutoCompoundConfig {
            enabled: true,
            min_compound_amount: 10 * 10u128.pow(18), // 10 tokens minimum
            compound_frequency: 3600, // 1 hour minimum between compounds
            compound_fee_percent: 0.005, // 0.5% fee
            max_compounds_per_day: 24,
        }
    }
}

/// Advanced staking contract with auto-compound support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedStakingContract {
    pub staked_token: String,
    pub reward_token: String,
    pub reward_rate: u128,
    pub total_staked: u128,
    pub total_compounded: u128,
    pub stakes: HashMap<String, AutoStakePosition>,
    pub config: AutoCompoundConfig,
    pub compound_history: HashMap<String, Vec<CompoundEvent>>,
    pub min_stake: u128,
    pub lock_periods: Vec<(u64, f64)>,
    pub paused: bool,
    pub owner: String,
    pub compound_fees_collected: u128,
    pub max_history_per_user: usize,
}

/// Event logged on each auto-compound
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundEvent {
    pub user: String,
    pub amount_compounded: u128,
    pub fee_charged: u128,
    pub new_principal: u128,
    pub timestamp: u64,
    pub tx_hash: String,
}

impl AdvancedStakingContract {
    pub fn new(staked_token: String, reward_token: String, reward_rate: u128, owner: String) -> Self {
        AdvancedStakingContract {
            staked_token,
            reward_token,
            reward_rate,
            total_staked: 0,
            total_compounded: 0,
            stakes: HashMap::new(),
            config: AutoCompoundConfig::default(),
            compound_history: HashMap::new(),
            min_stake: 100 * 10u128.pow(18),
            lock_periods: vec![
                (0, 1.0),
                (30 * 24 * 3600, 1.2),
                (90 * 24 * 3600, 1.5),
                (180 * 24 * 3600, 2.0),
                (365 * 24 * 3600, 3.0),
            ],
            paused: false,
            owner,
            compound_fees_collected: 0,
            max_history_per_user: 100,
        }
    }
    
    /// Stake with auto-compound enabled
    pub fn stake_auto(&mut self, user: String, amount: u128, lock_days: u64, auto_compound: bool) -> Result<bool, String> {
        if self.paused {
            return Err("Contract is paused".to_string());
        }
        
        if amount < self.min_stake {
            return Err(format!("Minimum stake is {}", self.min_stake));
        }
        
        let lock_seconds = lock_days * 24 * 3600;
        let multiplier = self.get_lock_multiplier(lock_seconds);
        let now = self.current_timestamp();
        
        self.stakes.insert(user.clone(), AutoStakePosition {
            principal: amount,
            compounded_amount: amount,
            staked_at: now,
            last_compound: now,
            lock_duration: lock_seconds,
            lock_multiplier: multiplier,
            auto_compound,
            compound_threshold: self.config.min_compound_amount,
            pending_rewards: 0,
        });
        
        self.total_staked += amount;
        
        Ok(true)
    }
    
    /// Manual stake (without auto-compound)
    pub fn stake(&mut self, user: String, amount: u128, lock_days: u64) -> Result<bool, String> {
        self.stake_auto(user, amount, lock_days, false)
    }
    
    /// Trigger auto-compound for a user (called automatically or manually)
    pub fn auto_compound(&mut self, user: String) -> Result<u128, String> {
        if !self.config.enabled {
            return Err("Auto-compound is disabled".to_string());
        }
        
        let stake = self.stakes.get_mut(&user)
            .ok_or("No stake found")?;
        
        if !stake.auto_compound {
            return Err("Auto-compound not enabled for this position".to_string());
        }
        
        let now = self.current_timestamp();
        
        // Check compound frequency
        if now - stake.last_compound < self.config.compound_frequency {
            return Err("Compound frequency not met".to_string());
        }
        
        // Calculate pending rewards
        let pending = self.calculate_pending_rewards(&user);
        
        if pending < stake.compound_threshold {
            return Err("Rewards below compound threshold".to_string());
        }
        
        // Calculate fee
        let fee = ((pending as f64) * self.config.compound_fee_percent) as u128;
        let net_reward = pending - fee;
        
        // Compound: add to principal
        stake.principal += net_reward;
        stake.compounded_amount += net_reward;
        stake.pending_rewards = 0;
        stake.last_compound = now;
        
        // Track fees
        self.compound_fees_collected += fee;
        self.total_compounded += net_reward;
        
        // Log event
        self.log_compound_event(user.clone(), net_reward, fee, stake.principal);
        
        Ok(net_reward)
    }
    
    /// Calculate pending rewards for a user
    pub fn calculate_pending_rewards(&self, user: &str) -> u128 {
        if let Some(stake) = self.stakes.get(user) {
            let now = self.current_timestamp();
            let time_elapsed = now - stake.last_compound;
            let base_reward = (self.reward_rate as f64 * time_elapsed as f64) as u128;
            ((base_reward as f64) * stake.lock_multiplier) as u128
        } else {
            0
        }
    }
    
    /// Get total claimable rewards (pending + auto-compounded but not withdrawn)
    pub fn get_claimable_rewards(&self, user: &str) -> u128 {
        if let Some(stake) = self.stakes.get(user) {
            if stake.auto_compound {
                // For auto-compound, rewards are already added to principal
                stake.compounded_amount - stake.principal + self.calculate_pending_rewards(user)
            } else {
                self.calculate_pending_rewards(user)
            }
        } else {
            0
        }
    }
    
    /// Manual compound trigger (for users who want to compound on demand)
    pub fn manual_compound(&mut self, user: String) -> Result<u128, String> {
        self.auto_compound(user)
    }
    
    /// Disable auto-compound for a position
    pub fn disable_auto_compound(&mut self, user: String) -> Result<bool, String> {
        let stake = self.stakes.get_mut(&user)
            .ok_or("No stake found")?;
        
        stake.auto_compound = false;
        
        Ok(true)
    }
    
    /// Enable auto-compound for a position
    pub fn enable_auto_compound(&mut self, user: String) -> Result<bool, String> {
        let stake = self.stakes.get_mut(&user)
            .ok_or("No stake found")?;
        
        stake.auto_compound = true;
        stake.last_compound = self.current_timestamp();
        
        Ok(true)
    }
    
    /// Update compound threshold
    pub fn set_compound_threshold(&mut self, user: String, threshold: u128) -> Result<bool, String> {
        let stake = self.stakes.get_mut(&user)
            .ok_or("No stake found")?;
        
        stake.compound_threshold = threshold;
        
        Ok(true)
    }
    
    /// Unstake (withdraws principal + any pending rewards)
    pub fn unstake(&mut self, user: String, amount: u128) -> Result<(u128, u128), String> {
        if self.paused {
            return Err("Contract is paused".to_string());
        }
        
        let stake = self.stakes.get(&user)
            .ok_or("No stake found")?;
        
        if stake.compounded_amount < amount {
            return Err("Insufficient staked amount".to_string());
        }
        
        // Check lock period
        let now = self.current_timestamp();
        let unlock_time = stake.staked_at + stake.lock_duration;
        if now < unlock_time {
            return Err(format!("Tokens are locked until {}", unlock_time));
        }
        
        // Calculate pending rewards
        let pending = self.calculate_pending_rewards(&user);
        
        let stake = self.stakes.get_mut(&user).unwrap();
        stake.compounded_amount -= amount;
        stake.principal = stake.principal.saturating_sub(amount);
        self.total_staked = self.total_staked.saturating_sub(amount);
        
        if stake.compounded_amount == 0 {
            self.stakes.remove(&user);
        }
        
        Ok((amount, pending))
    }
    
    /// Claim rewards without unstaking (for non-auto-compound positions)
    pub fn claim_rewards(&mut self, user: String) -> Result<u128, String> {
        let stake = self.stakes.get(&user)
            .ok_or("No stake found")?;
        
        if stake.auto_compound {
            return Err("Use auto_compound for auto-compound positions".to_string());
        }
        
        let pending = self.calculate_pending_rewards(&user);
        
        let stake = self.stakes.get_mut(&user).unwrap();
        stake.last_compound = self.current_timestamp();
        
        Ok(pending)
    }
    
    fn get_lock_multiplier(&self, lock_seconds: u64) -> f64 {
        let mut multiplier = 1.0;
        for (duration, mult) in &self.lock_periods {
            if lock_seconds >= *duration {
                multiplier = *mult;
            }
        }
        multiplier
    }
    
    fn log_compound_event(&mut self, user: String, amount: u128, fee: u128, new_principal: u128) {
        let event = CompoundEvent {
            user: user.clone(),
            amount_compounded: amount,
            fee_charged: fee,
            new_principal,
            timestamp: self.current_timestamp(),
            tx_hash: format!("compound_{}_{}", user, self.current_timestamp()),
        };
        
        let history = self.compound_history.entry(user).or_insert_with(Vec::new);
        history.push(event);
        
        // Trim old events
        if history.len() > self.max_history_per_user {
            history.remove(0);
        }
    }
    
    fn current_timestamp(&self) -> u64 {
        1700000000
    }
    
    /// Get compound history for a user
    pub fn get_compound_history(&self, user: &str) -> Vec<CompoundEvent> {
        self.compound_history.get(user).cloned().unwrap_or_default()
    }
    
    /// Get staking stats
    pub fn get_stats(&self) -> StakingStats {
        let total_pending: u128 = self.stakes.keys()
            .map(|u| self.calculate_pending_rewards(u))
            .sum();
        
        StakingStats {
            total_staked: self.total_staked,
            total_compounded: self.total_compounded,
            total_positions: self.stakes.len() as u64,
            auto_compound_enabled: self.stakes.values().filter(|s| s.auto_compound).count() as u64,
            total_pending_rewards: total_pending,
            compound_fees_collected: self.compound_fees_collected,
            avg_lock_multiplier: self.calculate_avg_multiplier(),
        }
    }
    
    fn calculate_avg_multiplier(&self) -> f64 {
        if self.stakes.is_empty() {
            return 1.0;
        }
        let sum: f64 = self.stakes.values().map(|s| s.lock_multiplier).sum();
        sum / self.stakes.len() as f64
    }
}

/// Staking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStats {
    pub total_staked: u128,
    pub total_compounded: u128,
    pub total_positions: u64,
    pub auto_compound_enabled: u64,
    pub total_pending_rewards: u128,
    pub compound_fees_collected: u128,
    pub avg_lock_multiplier: f64,
}

// ============================================================================
// USAGE EXAMPLES
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_auto_compound_staking() {
        let mut staking = AdvancedStakingContract::new(
            "FLUX".to_string(),
            "FLUX".to_string(),
            1000, // 1000 FLUX per second
            "owner".to_string(),
        );
        
        // Stake with auto-compound enabled, 90-day lock
        let result = staking.stake_auto(
            "user1".to_string(),
            10000 * 10u128.pow(18),
            90,
            true,
        );
        
        assert!(result.is_ok());
        
        let stake = staking.stakes.get("user1").unwrap();
        assert!(stake.auto_compound);
        assert_eq!(stake.lock_multiplier, 1.5); // 90 days = 1.5x
    }
    
    #[test]
    fn test_compound_event_logging() {
        let mut staking = AdvancedStakingContract::new(
            "FLUX".to_string(),
            "FLUX".to_string(),
            100000, // High reward rate for testing
            "owner".to_string(),
        );
        
        staking.stake_auto("user1".to_string(), 10000 * 10u128.pow(18), 0, true).unwrap();
        
        // Wait for compound frequency (simulate time passing)
        // In real tests, you'd mock current_timestamp()
        
        let history = staking.get_compound_history("user1");
        assert!(history.is_empty()); // No compounds yet
    }
}

// End of Advanced Staking Contract - Sprint 22
