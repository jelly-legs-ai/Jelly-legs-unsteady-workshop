// Unified Staking Contract - AeTHer Chain
// Multi-token staking with dynamic reward distribution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported staking token types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StakingToken {
    FLUX,
    ATH,
    AETH,
}

/// Staking tier levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingTier {
    pub tier_id: u8,
    pub name: String,
    pub min_stake: u64,
    pub max_stake: u64,
    pub lock_period_days: u32,
    pub reward_multiplier: f64,
    pub benefits: Vec<String>,
}

/// Individual staking position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    pub position_id: String,
    pub owner: String,
    pub token: StakingToken,
    pub amount: u64,
    pub tier: u8,
    pub start_epoch: u64,
    pub lock_end_epoch: u64,
    pub last_claim_epoch: u64,
    pub accumulated_rewards: u64,
    pub is_active: bool,
    pub early_unstake_penalty: f64,
}

/// Staking pool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub token: StakingToken,
    pub total_staked: u64,
    pub total_rewards_distributed: u64,
    pub current_apr: f64,
    pub last_update_epoch: u64,
    pub participants_count: u64,
}

/// Unstaking request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakingRequest {
    pub request_id: String,
    pub position_id: String,
    pub owner: String,
    pub token: StakingToken,
    pub amount: u64,
    pub request_epoch: u64,
    pub unlock_epoch: u64,
    pub is_instant: bool,
    pub penalty_applied: f64,
}

/// Reward claim record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardClaim {
    pub claim_id: String,
    pub position_id: String,
    pub owner: String,
    pub token: StakingToken,
    pub amount: u64,
    pub claim_epoch: u64,
    pub reward_amount: u64,
}

/// Staking contract main state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedStakingContract {
    pub contract_id: String,
    pub name: String,
    pub version: String,
    pub is_active: bool,
    pub pools: HashMap<String, StakingPool>,
    pub positions: HashMap<String, StakingPosition>,
    pub unstaking_requests: HashMap<String, UnstakingRequest>,
    pub reward_claims: Vec<RewardClaim>,
    pub tiers: Vec<StakingTier>,
    pub total_value_locked: u64,
    pub minimum_stake_amount: u64,
    pub early_unstake_penalty_base: f64,
}

impl UnifiedStakingContract {
    /// Create new unified staking contract
    pub fn new() -> Self {
        let mut pools = HashMap::new();
        
        // Initialize pools for each token
        pools.insert("FLUX".to_string(), StakingPool {
            token: StakingToken::FLUX,
            total_staked: 0,
            total_rewards_distributed: 0,
            current_apr: 12.5,
            last_update_epoch: 0,
            participants_count: 0,
        });
        
        pools.insert("ATH".to_string(), StakingPool {
            token: StakingToken::ATH,
            total_staked: 0,
            total_rewards_distributed: 0,
            current_apr: 15.0,
            last_update_epoch: 0,
            participants_count: 0,
        });
        
        pools.insert("AETH".to_string(), StakingPool {
            token: StakingToken::AETH,
            total_staked: 0,
            total_rewards_distributed: 0,
            current_apr: 10.0,
            last_update_epoch: 0,
            participants_count: 0,
        });
        
        // Define staking tiers
        let tiers = vec![
            StakingTier {
                tier_id: 1,
                name: "Bronze".to_string(),
                min_stake: 100,
                max_stake: 999,
                lock_period_days: 30,
                reward_multiplier: 1.0,
                benefits: vec!["Basic rewards".to_string()],
            },
            StakingTier {
                tier_id: 2,
                name: "Silver".to_string(),
                min_stake: 1000,
                max_stake: 4999,
                lock_period_days: 90,
                reward_multiplier: 1.25,
                benefits: vec!["Enhanced rewards".to_string(), "Priority support".to_string()],
            },
            StakingTier {
                tier_id: 3,
                name: "Gold".to_string(),
                min_stake: 5000,
                max_stake: 24999,
                lock_period_days: 180,
                reward_multiplier: 1.5,
                benefits: vec!["Premium rewards".to_string(), "Governance voting".to_string(), "Exclusive access".to_string()],
            },
            StakingTier {
                tier_id: 4,
                name: "Platinum".to_string(),
                min_stake: 25000,
                max_stake: u64::MAX,
                lock_period_days: 365,
                reward_multiplier: 2.0,
                benefits: vec!["Maximum rewards".to_string(), "Full governance".to_string(), "Validator eligibility".to_string(), "Protocol dividends".to_string()],
            },
        ];
        
        UnifiedStakingContract {
            contract_id: "unified_staking_v1".to_string(),
            name: "AeTHer Unified Staking".to_string(),
            version: "1.0.0".to_string(),
            is_active: true,
            pools,
            positions: HashMap::new(),
            unstaking_requests: HashMap::new(),
            reward_claims: Vec::new(),
            tiers,
            total_value_locked: 0,
            minimum_stake_amount: 100,
            early_unstake_penalty_base: 0.05, // 5% base penalty
        }
    }
    
    /// Stake tokens
    pub fn stake(
        &mut self,
        position_id: String,
        owner: String,
        token: StakingToken,
        amount: u64,
        tier_requested: u8,
    ) -> Result<StakingPosition, &'static str> {
        if !self.is_active {
            return Err("Staking contract is not active");
        }
        
        if amount < self.minimum_stake_amount {
            return Err("Amount below minimum stake");
        }
        
        let token_symbol = match token {
            StakingToken::FLUX => "FLUX",
            StakingToken::ATH => "ATH",
            StakingToken::AETH => "AETH",
        };
        
        let pool = self.pools.get_mut(token_symbol)
            .ok_or("Pool not found")?;
        
        let tier = self.tiers.iter().find(|t| t.tier_id == tier_requested)
            .ok_or("Invalid tier")?;
        
        if amount < tier.min_stake || amount > tier.max_stake {
            return Err("Amount not in tier range");
        }
        
        let position = StakingPosition {
            position_id: position_id.clone(),
            owner: owner.clone(),
            token: token.clone(),
            amount,
            tier: tier_requested,
            start_epoch: 0, // Would be set from blockchain epoch
            lock_end_epoch: 0, // Would be set from blockchain epoch + lock_period
            last_claim_epoch: 0,
            accumulated_rewards: 0,
            is_active: true,
            early_unstake_penalty: self.early_unstake_penalty_base,
        };
        
        // Update pool
        pool.total_staked += amount;
        pool.participants_count += 1;
        
        // Update TVL
        self.total_value_locked += amount;
        
        // Store position
        self.positions.insert(position_id.clone(), position.clone());
        
        Ok(position)
    }
    
    /// Claim rewards for a position
    pub fn claim_rewards(
        &mut self,
        position_id: &str,
        owner: &str,
    ) -> Result<u64, &'static str> {
        let position = self.positions.get_mut(position_id)
            .ok_or("Position not found")?;
        
        if position.owner != owner {
            return Err("Not position owner");
        }
        
        if !position.is_active {
            return Err("Position not active");
        }
        
        let tier = self.tiers.iter().find(|t| t.tier_id == position.tier)
            .ok_or("Tier not found")?;
        
        let token_symbol = match position.token {
            StakingToken::FLUX => "FLUX",
            StakingToken::ATH => "ATH",
            StakingToken::AETH => "AETH",
        };
        
        let pool = self.pools.get_mut(token_symbol)
            .ok_or("Pool not found")?;
        
        // Calculate pending rewards
        let reward_amount = position.accumulated_rewards;
        
        if reward_amount > 0 {
            // Record the claim
            let claim = RewardClaim {
                claim_id: format!("claim_{}_{}", position_id, pool.last_update_epoch),
                position_id: position_id.to_string(),
                owner: owner.to_string(),
                token: position.token.clone(),
                amount: position.amount,
                claim_epoch: pool.last_update_epoch,
                reward_amount,
            };
            
            self.reward_claims.push(claim);
            
            // Update pool
            pool.total_rewards_distributed += reward_amount;
            
            // Reset accumulated rewards
            position.accumulated_rewards = 0;
            position.last_claim_epoch = pool.last_update_epoch;
        }
        
        Ok(reward_amount)
    }
    
    /// Request unstaking
    pub fn request_unstake(
        &mut self,
        request_id: String,
        position_id: String,
        owner: String,
        amount: u64,
        instant: bool,
    ) -> Result<UnstakingRequest, &'static str> {
        let position = self.positions.get_mut(&position_id)
            .ok_or("Position not found")?;
        
        if position.owner != owner {
            return Err("Not position owner");
        }
        
        if amount > position.amount {
            return Err("Amount exceeds staked amount");
        }
        
        let token_symbol = match position.token {
            StakingToken::FLUX => "FLUX",
            StakingToken::ATH => "ATH",
            StakingToken::AETH => "AETH",
        };
        
        let pool = self.pools.get_mut(token_symbol)
            .ok_or("Pool not found")?;
        
        let penalty = if instant {
            self.early_unstake_penalty_base * amount as f64
        } else {
            0.0
        };
        
        let request = UnstakingRequest {
            request_id: request_id.clone(),
            position_id: position_id.clone(),
            owner: owner.clone(),
            token: position.token.clone(),
            amount,
            request_epoch: pool.last_update_epoch,
            unlock_epoch: if instant { pool.last_update_epoch } else { pool.last_update_epoch + 7 }, // 7 epoch delay for non-instant
            is_instant: instant,
            penalty_applied: penalty,
        };
        
        // Update position
        position.amount -= amount;
        
        // Update pool
        pool.total_staked -= amount;
        
        // Update TVL
        self.total_value_locked -= amount;
        
        // If fully unstaked, deactivate position
        if position.amount == 0 {
            position.is_active = false;
            pool.participants_count = pool.participants_count.saturating_sub(1);
        }
        
        self.unstaking_requests.insert(request_id, request.clone());
        
        Ok(request)
    }
    
    /// Get staking statistics
    pub fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("total_value_locked".to_string(), serde_json::json!(self.total_value_locked));
        stats.insert("total_positions".to_string(), serde_json::json!(self.positions.len()));
        stats.insert("active_positions".to_string(), serde_json::json!(self.positions.values().filter(|p| p.is_active).count()));
        stats.insert("pending_unstakes".to_string(), serde_json::json!(self.unstaking_requests.len()));
        stats.insert("total_rewards_claimed".to_string(), serde_json::json!(self.reward_claims.iter().map(|c| c.reward_amount).sum::<u64>()));
        
        let pool_stats: HashMap<String, serde_json::Value> = self.pools.iter()
            .map(|(symbol, pool)| {
                (symbol.clone(), serde_json::json!({
                    "total_staked": pool.total_staked,
                    "apr": pool.current_apr,
                    "participants": pool.participants_count,
                    "rewards_distributed": pool.total_rewards_distributed,
                }))
            })
            .collect();
        
        stats.insert("pools".to_string(), serde_json::json!(pool_stats));
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stake_and_claim() {
        let mut contract = UnifiedStakingContract::new();
        
        // Stake FLUX
        let result = contract.stake(
            "pos_1".to_string(),
            "user_1".to_string(),
            StakingToken::FLUX,
            5000,
            3, // Gold tier
        );
        
        assert!(result.is_ok());
        let position = result.unwrap();
        assert_eq!(position.amount, 5000);
        assert_eq!(position.tier, 3);
        
        // Check pool updated
        let pool = contract.pools.get("FLUX").unwrap();
        assert_eq!(pool.total_staked, 5000);
        assert_eq!(pool.participants_count, 1);
    }
    
    #[test]
    fn test_unstake_instant_penalty() {
        let mut contract = UnifiedStakingContract::new();
        
        // Stake first
        contract.stake(
            "pos_1".to_string(),
            "user_1".to_string(),
            StakingToken::FLUX,
            1000,
            2,
        ).unwrap();
        
        // Request instant unstake
        let result = contract.request_unstake(
            "unstake_1".to_string(),
            "pos_1".to_string(),
            "user_1".to_string(),
            500,
            true, // instant
        );
        
        assert!(result.is_ok());
        let request = result.unwrap();
        assert!(request.is_instant);
        assert!(request.penalty_applied > 0.0);
    }
}
