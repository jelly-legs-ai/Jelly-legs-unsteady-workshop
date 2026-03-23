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

/// Staking contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingContract {
    pub pools: HashMap<String, StakingPool>,
    pub stakes: HashMap<String, Vec<StakeInfo>>,
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
            current_epoch: 0,
            total_rewards_distributed: 0,
        }
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
}
