// Staking Contract - AeTHer Chain
// Stake AETH/FLUX tokens to earn rewards

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stake information for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub address: String,
    pub token_type: TokenType,
    pub amount: u64,
    pub start_epoch: u64,
    pub last_claim_epoch: u64,
    pub rewards_claimed: u64,
    pub is_locked: bool,
    pub lock_end_epoch: u64,
}

/// Token type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    AETH,
    FLUX,
    ATH,
}

/// Staking pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub name: String,
    pub token_type: TokenType,
    pub total_staked: u64,
    pub reward_rate: f64, // APY as decimal
    pub min_stake: u64,
    pub lockup_epochs: u64,
    pub active_stakers: u64,
}

/// Delegation info for stake delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationInfo {
    pub delegator: String,
    pub validator: String,
    pub amount: u64,
    pub start_epoch: u64,
    pub last_claim_epoch: u64,
    pub rewards_claimed: u64,
    pub pool_id: String,
}

/// Validator performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetrics {
    pub validator_address: String,
    pub total_delegated: u64,
    pub delegator_count: u64,
    pub commission_rate: f64,
    pub uptime_percent: f64,
    pub slashing_events: u64,
}

/// Staking contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingContract {
    pub pools: HashMap<String, StakingPool>,
    pub stakes: HashMap<String, Vec<StakeInfo>>,
    pub delegations: HashMap<String, Vec<DelegationInfo>>,
    pub validator_metrics: HashMap<String, ValidatorMetrics>,
    pub current_epoch: u64,
    pub total_rewards_distributed: u64,
}

impl StakingContract {
    /// Create new staking contract
    pub fn new() -> Self {
        let mut pools = HashMap::new();
        
        // AETH staking pool - 15% APY, 7 day lock
        pools.insert("aeth_staking".to_string(), StakingPool {
            name: "AeTHer Staking".to_string(),
            token_type: TokenType::AETH,
            total_staked: 0,
            reward_rate: 0.15,
            min_stake: 100,
            lockup_epochs: 7,
            active_stakers: 0,
        });
        
        // FLUX staking pool - 20% APY, 14 day lock
        pools.insert("flux_staking".to_string(), StakingPool {
            name: "FLUX Staking".to_string(),
            token_type: TokenType::FLUX,
            total_staked: 0,
            reward_rate: 0.20,
            min_stake: 50,
            lockup_epochs: 14,
            active_stakers: 0,
        });
        
        // ATH staking pool - 25% APY, 30 day lock
        pools.insert("ath_staking".to_string(), StakingPool {
            name: "ATH Staking".to_string(),
            token_type: TokenType::ATH,
            total_staked: 0,
            reward_rate: 0.25,
            min_stake: 1000,
            lockup_epochs: 30,
            active_stakers: 0,
        });
        
        StakingContract {
            pools,
            stakes: HashMap::new(),
            delegations: HashMap::new(),
            validator_metrics: HashMap::new(),
            current_epoch: 0,
            total_rewards_distributed: 0,
        }
    }
    
    /// Delegate stake to a validator
    pub fn delegate(&mut self, delegator: &str, validator: &str, pool_id: &str, amount: u64) -> Result<DelegationInfo, &'static str> {
        let pool = self.pools.get_mut(pool_id)
            .ok_or("Pool not found")?;
        
        if amount < pool.min_stake {
            return Err("Amount below minimum delegation");
        }
        
        let delegation = DelegationInfo {
            delegator: delegator.to_string(),
            validator: validator.to_string(),
            amount,
            start_epoch: self.current_epoch,
            last_claim_epoch: self.current_epoch,
            rewards_claimed: 0,
            pool_id: pool_id.to_string(),
        };
        
        pool.total_staked += amount;
        pool.active_stakers += 1;
        
        // Update validator metrics
        let metrics = self.validator_metrics.entry(validator.to_string())
            .or_insert(ValidatorMetrics {
                validator_address: validator.to_string(),
                total_delegated: 0,
                delegator_count: 0,
                commission_rate: 0.05, // 5% default commission
                uptime_percent: 100.0,
                slashing_events: 0,
            });
        metrics.total_delegated += amount;
        metrics.delegator_count += 1;
        
        let key = delegator.to_string();
        let delegations = self.delegations.entry(key).or_insert_with(Vec::new);
        delegations.push(delegation.clone());
        
        Ok(delegation)
    }
    
    /// Calculate delegation rewards
    pub fn calculate_delegation_rewards(&self, delegation: &DelegationInfo) -> u64 {
        let pool = self.pools.get(&delegation.pool_id).unwrap();
        let epochs_staked = self.current_epoch - delegation.last_claim_epoch;
        let reward_per_epoch = (delegation.amount as f64 * pool.reward_rate) / 365.0;
        
        // Apply validator commission
        let metrics = self.validator_metrics.get(&delegation.validator);
        let commission = metrics.map(|m| m.commission_rate).unwrap_or(0.05);
        let net_reward = reward_per_epoch * (1.0 - commission);
        
        (net_reward * epochs_staked as f64) as u64
    }
    
    /// Claim delegation rewards
    pub fn claim_delegation_rewards(&mut self, delegator: &str, pool_id: &str) -> Result<u64, &'static str> {
        let delegations = self.delegations.get_mut(delegator)
            .ok_or("No delegations found")?;
        
        for delegation in delegations.iter_mut() {
            if delegation.pool_id == pool_id {
                let rewards = self.calculate_delegation_rewards(delegation);
                delegation.last_claim_epoch = self.current_epoch;
                delegation.rewards_claimed += rewards;
                self.total_rewards_distributed += rewards;
                return Ok(rewards);
            }
        }
        
        Err("No active delegation found")
    }
    
    /// Redelegate rewards (compound)
    pub fn redelegate_rewards(&mut self, delegator: &str, pool_id: &str) -> Result<u64, &'static str> {
        let rewards = self.claim_delegation_rewards(delegator, pool_id)?;
        
        let delegations = self.delegations.get_mut(delegator)
            .ok_or("No delegations found")?;
        
        for delegation in delegations.iter_mut() {
            if delegation.pool_id == pool_id {
                delegation.amount += rewards;
                break;
            }
        }
        
        Ok(rewards)
    }
    
    /// Undelegate stake
    pub fn undelegate(&mut self, delegator: &str, pool_id: &str, amount: u64) -> Result<u64, &'static str> {
        let delegations = self.delegations.get_mut(delegator)
            .ok_or("No delegations found")?;
        
        for delegation in delegations.iter_mut() {
            if delegation.pool_id == pool_id && delegation.amount >= amount {
                let rewards = self.calculate_delegation_rewards(delegation);
                delegation.amount -= amount;
                delegation.last_claim_epoch = self.current_epoch;
                delegation.rewards_claimed += rewards;
                
                if let Some(pool) = self.pools.get_mut(pool_id) {
                    pool.total_staked -= amount;
                    pool.active_stakers = pool.active_stakers.saturating_sub(1);
                }
                
                if let Some(metrics) = self.validator_metrics.get_mut(&delegation.validator) {
                    metrics.total_delegated = metrics.total_delegated.saturating_sub(amount);
                    metrics.delegator_count = metrics.delegator_count.saturating_sub(1);
                }
                
                self.total_rewards_distributed += rewards;
                return Ok(amount + rewards);
            }
        }
        
        Err("Insufficient delegation or pool not found")
    }
    
    /// Stake tokens
    pub fn stake(&mut self, address: &str, pool_id: &str, amount: u64) -> Result<StakeInfo, &'static str> {
        let pool = self.pools.get_mut(pool_id)
            .ok_or("Pool not found")?;
        
        if amount < pool.min_stake {
            return Err("Amount below minimum stake");
        }
        
        let stake_info = StakeInfo {
            address: address.to_string(),
            token_type: pool.token_type.clone(),
            amount,
            start_epoch: self.current_epoch,
            last_claim_epoch: self.current_epoch,
            rewards_claimed: 0,
            is_locked: true,
            lock_end_epoch: self.current_epoch + pool.lockup_epochs,
        };
        
        pool.total_staked += amount;
        pool.active_stakers += 1;
        
        let key = address.to_string();
        let stakes = self.stakes.entry(key).or_insert_with(Vec::new);
        stakes.push(stake_info.clone());
        
        Ok(stake_info)
    }
    
    /// Calculate pending rewards for a stake
    pub fn calculate_rewards(&self, stake: &StakeInfo) -> u64 {
        let pool_id = match stake.token_type {
            TokenType::AETH => "aeth_staking",
            TokenType::FLUX => "flux_staking",
            TokenType::ATH => "ath_staking",
        };
        
        let pool = self.pools.get(pool_id).unwrap();
        let epochs_staked = self.current_epoch - stake.last_claim_epoch;
        let reward_per_epoch = (stake.amount as f64 * pool.reward_rate) / 365.0;
        
        (reward_per_epoch * epochs_staked as f64) as u64
    }
    
    /// Claim rewards without unstaking
    pub fn claim_rewards(&mut self, address: &str, pool_id: &str) -> Result<u64, &'static str> {
        let stakes = self.stakes.get_mut(address)
            .ok_or("No stakes found")?;
        
        let pool = self.pools.get(pool_id)
            .ok_or("Pool not found")?;
        
        for stake in stakes.iter_mut() {
            if stake.token_type == pool.token_type && !stake.is_locked {
                let rewards = self.calculate_rewards(stake);
                stake.last_claim_epoch = self.current_epoch;
                stake.rewards_claimed += rewards;
                self.total_rewards_distributed += rewards;
                return Ok(rewards);
            }
        }
        
        Err("No active stake found")
    }
    
    /// Unstake tokens
    pub fn unstake(&mut self, address: &str, pool_id: &str, amount: u64) -> Result<u64, &'static str> {
        let stakes = self.stakes.get_mut(address)
            .ok_or("No stakes found")?;
        
        let pool = self.pools.get_mut(pool_id)
            .ok_or("Pool not found")?;
        
        for stake in stakes.iter_mut() {
            if stake.token_type == pool.token_type && stake.amount >= amount {
                if self.current_epoch < stake.lock_end_epoch {
                    return Err("Tokens still locked");
                }
                
                let rewards = self.calculate_rewards(stake);
                stake.amount -= amount;
                stake.last_claim_epoch = self.current_epoch;
                stake.rewards_claimed += rewards;
                
                pool.total_staked -= amount;
                pool.active_stakers = pool.active_stakers.saturating_sub(1);
                self.total_rewards_distributed += rewards;
                
                return Ok(amount + rewards);
            }
        }
        
        Err("Insufficient stake")
    }
    
    /// Advance epoch
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
    }
    
    // =============================================================================
    // REWARD CALCULATION HELPERS - Sprint Enhancement
    // =============================================================================
    
    /// Calculate APY for a given pool
    pub fn calculate_pool_apy(&self, pool_id: &str) -> f64 {
        self.pools.get(pool_id)
            .map(|p| p.reward_rate * 100.0)
            .unwrap_or(0.0)
    }
    
    /// Calculate daily reward rate (APY / 365)
    pub fn daily_reward_rate(&self, pool_id: &str) -> f64 {
        self.pools.get(pool_id)
            .map(|p| p.reward_rate / 365.0)
            .unwrap_or(0.0)
    }
    
    /// Calculate hourly reward rate (APY / 365 / 24)
    pub fn hourly_reward_rate(&self, pool_id: &str) -> f64 {
        self.pools.get(pool_id)
            .map(|p| p.reward_rate / 365.0 / 24.0)
            .unwrap_or(0.0)
    }
    
    /// Calculate projected rewards for a given stake amount and duration
    pub fn project_rewards(&self, pool_id: &str, amount: u64, days: u64) -> u64 {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return 0,
        };
        
        let daily_rate = pool.reward_rate / 365.0;
        let yearly_rewards = amount as f64 * daily_rate * days as f64;
        yearly_rewards as u64
    }
    
    /// Calculate total staking value in USD (assuming price feed)
    pub fn calculate_total_staked_value(&self, aeth_price: f64, flux_price: f64, ath_price: f64) -> f64 {
        let mut total = 0.0;
        
        for (pool_id, pool) in &self.pools {
            let token_price = match pool.token_type {
                TokenType::AETH => aeth_price,
                TokenType::FLUX => flux_price,
                TokenType::ATH => ath_price,
            };
            total += pool.total_staked as f64 * token_price;
        }
        
        total
    }
    
    /// Get staking APR (Annual Percentage Rate) adjusted for compounding
    pub fn calculate_compounded_apr(&self, pool_id: &str, compounding_frequency: u64) -> f64 {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return 0.0,
        };
        
        let n = compounding_frequency as f64; // Number of compounding periods per year
        let r = pool.reward_rate; // Annual rate
        
        // APY = (1 + r/n)^n - 1
        let apy = (1.0 + r / n).powf(n) - 1.0;
        apy * 100.0 // Return as percentage
    }
    
    /// Calculate lockup period end date (in epochs)
    pub fn get_lockup_end_epoch(&self, pool_id: &str) -> u64 {
        self.pools.get(pool_id)
            .map(|p| self.current_epoch + p.lockup_epochs)
            .unwrap_or(0)
    }
    
    /// Check if a stake is still locked
    pub fn is_stake_locked(&self, stake: &StakeInfo) -> bool {
        if !stake.is_locked {
            return false;
        }
        self.current_epoch < stake.lock_end_epoch
    }
    
    /// Get unlockable stake amount
    pub fn get_unlockable_amount(&self, stakes: &[StakeInfo]) -> u64 {
        stakes.iter()
            .filter(|s| !self.is_stake_locked(s))
            .map(|s| s.amount)
            .sum()
    }
    
    /// Calculate validator uptime score
    pub fn calculate_uptime_score(&self, validator: &str) -> f64 {
        self.validator_metrics.get(validator)
            .map(|m| m.uptime_percent / 100.0)
            .unwrap_or(0.0)
    }
    
    /// Calculate effective reward rate after validator commission
    pub fn effective_reward_rate(&self, pool_id: &str, validator: &str) -> f64 {
        let pool_rate = self.daily_reward_rate(pool_id);
        let metrics = match self.validator_metrics.get(validator) {
            Some(m) => m,
            None => return pool_rate,
        };
        
        pool_rate * (1.0 - metrics.commission_rate)
    }
    
    /// Get all pools info
    pub fn get_all_pools(&self) -> Vec<(&String, &StakingPool)> {
        self.pools.iter().collect()
    }
    
    /// Get pool by token type
    pub fn get_pool_by_token(&self, token_type: &TokenType) -> Option<&StakingPool> {
        self.pools.values()
            .find(|p| &p.token_type == token_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_staking_contract() {
        let mut contract = StakingContract::new();
        assert_eq!(contract.current_epoch, 0);
        
        let stake = contract.stake("user1", "aeth_staking", 1000).unwrap();
        assert_eq!(stake.amount, 1000);
        
        contract.advance_epoch();
        let rewards = contract.calculate_rewards(&stake);
        assert!(rewards > 0);
    }
    
    #[test]
    fn test_reward_calculation_helpers() {
        let contract = StakingContract::new();
        
        // Test APY calculation
        let apy = contract.calculate_pool_apy("aeth_staking");
        assert_eq!(apy, 15.0); // 15% APY
        
        // Test daily rate
        let daily_rate = contract.daily_reward_rate("aeth_staking");
        assert!((daily_rate - 0.000411).abs() < 0.0001); // ~0.0411% daily
        
        // Test projected rewards
        let projected = contract.project_rewards("aeth_staking", 10000, 30);
        assert!(projected > 0);
        
        // Test compounded APR
        let compounded = contract.calculate_compounded_apr("aeth_staking", 365);
        assert!(compounded > 15.0); // Should be slightly higher than 15% with daily compounding
    }
    
    #[test]
    fn test_stake_lockup() {
        let mut contract = StakingContract::new();
        let stake = contract.stake("user1", "aeth_staking", 1000).unwrap();
        
        // Initial lockup should be active
        assert!(contract.is_stake_locked(&stake));
        
        // Advance epochs beyond lockup
        for _ in 0..10 {
            contract.advance_epoch();
        }
        
        // After 10 epochs, still locked (7 day lock = 7 epochs)
        let updated_stake = contract.stakes.get("user1").unwrap().first().unwrap();
        assert!(!contract.is_stake_locked(updated_stake));
    }
}
