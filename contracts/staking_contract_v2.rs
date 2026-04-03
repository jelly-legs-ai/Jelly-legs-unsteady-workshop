// Staking Contract V2 - Enhanced Multi-Pool Staking System
// AeTHer Chain - Supports multiple staking pools, tiers, and reward distribution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

/// Staking pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub pool_id: String,
    pub name: String,
    pub description: String,
    pub token_symbol: String,
    pub min_stake: u64,
    pub max_stake: u64,
    pub total_staked: u64,
    pub reward_rate: f64,        // APY as decimal (0.15 = 15%)
    pub lock_period_epochs: u64,
    pub early_unstake_penalty: f64, // Penalty rate for early withdrawal
    pub tier_multipliers: HashMap<String, f64>, // Tier name -> bonus multiplier
    pub is_active: bool,
    pub created_at_epoch: u64,
}

/// User's stake in a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakePosition {
    pub position_id: String,
    pub pool_id: String,
    pub owner: String,
    pub amount: u64,
    pub tier: String,
    pub multiplier: f64,
    pub start_epoch: u64,
    pub lock_end_epoch: u64,
    pub last_claim_epoch: u64,
    pub accumulated_rewards: u64,
    pub is_locked: bool,
    pub is_active: bool,
}

/// Reward tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardTier {
    pub tier_name: String,
    pub min_stake: u64,
    pub max_stake: u64,
    pub apy_boost: f64,          // Additional APY bonus
    pub exclusive_pools: bool,   // Access to exclusive pools
    pub governance_weight: u64,  // Voting power multiplier
}

/// Staking pool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPoolState {
    pub pool: StakingPool,
    pub stakers: HashMap<String, StakePosition>,
    pub total_positions: u64,
    pub active_positions: u64,
    pub average_stake_age: f64,
    pub total_rewards_distributed: u64,
    pub epoch_rewards: VecDeque<EpochReward>,
}

/// Epoch reward record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochReward {
    pub epoch: u64,
    pub total_stake: u64,
    pub rewards_issued: u64,
    pub staker_count: u64,
}

/// Stake request for creating new position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeRequest {
    pub pool_id: String,
    pub amount: u64,
    pub auto_compound: bool,
    pub tier_override: Option<String>,
}

/// Unstake request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeRequest {
    pub position_id: String,
    pub amount: Option<u64>, // None = unstake all
    pub force_early: bool,   // Trigger early unstake penalty
}

/// Claim rewards request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRequest {
    pub position_ids: Vec<String>,
    pub reinvest: bool,
}

/// Staking contract main state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingContract {
    pub name: String,
    pub version: String,
    pub owner: String,
    pub pools: HashMap<String, StakingPoolState>,
    pub tiers: Vec<RewardTier>,
    pub global_stats: GlobalStakingStats,
    pub epoch: u64,
    pub emergency_pause: bool,
    pub treasury_address: String,
}

/// Global staking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStakingStats {
    pub total_staked: u64,
    pub total_stakers: u64,
    pub total_rewards_distributed: u64,
    pub average_apy: f64,
    pub pool_count: u64,
    pub largest_pool: String,
}

/// Stake result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeResult {
    pub position_id: String,
    pub pool_id: String,
    pub amount: u64,
    pub tier: String,
    pub effective_multiplier: f64,
    pub lock_end_epoch: u64,
    pub first_rewards_at_epoch: u64,
    pub estimated_apy: f64,
}

/// Unstake result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeResult {
    pub position_id: String,
    pub amount_returned: u64,
    pub rewards_claimed: u64,
    pub penalty_applied: u64,
    pub penalty_destination: String,
    pub final_balance: u64,
}

/// Claim result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimResult {
    pub positions_claimed: u64,
    pub total_rewards: u64,
    pub breakdown: Vec<PositionClaimBreakdown>,
    pub reinvested: bool,
}

/// Per-position claim breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionClaimBreakdown {
    pub position_id: String,
    pub accumulated: u64,
    pub claimed_this_epoch: u64,
    pub new_total: u64,
}

impl StakingContract {
    /// Create new staking contract
    pub fn new(owner: String) -> Self {
        let mut contract = StakingContract {
            name: "AeTHer Chain Staking Contract V2".to_string(),
            version: "2.0.0".to_string(),
            owner,
            pools: HashMap::new(),
            tiers: Vec::new(),
            global_stats: GlobalStakingStats {
                total_staked: 0,
                total_stakers: 0,
                total_rewards_distributed: 0,
                average_apy: 0.0,
                pool_count: 0,
                largest_pool: String::new(),
            },
            epoch: 0,
            emergency_pause: false,
            treasury_address: "treasury".to_string(),
        };
        
        // Initialize default tiers
        contract.init_default_tiers();
        
        // Initialize default pools
        contract.init_default_pools();
        
        contract
    }
    
    /// Initialize default reward tiers
    fn init_default_tiers(&mut self) {
        self.tiers = vec![
            RewardTier {
                tier_name: "Bronze".to_string(),
                min_stake: 100,
                max_stake: 9_999,
                apy_boost: 0.0,
                exclusive_pools: false,
                governance_weight: 1,
            },
            RewardTier {
                tier_name: "Silver".to_string(),
                min_stake: 10_000,
                max_stake: 49_999,
                apy_boost: 0.05,  // +5% APY
                exclusive_pools: false,
                governance_weight: 2,
            },
            RewardTier {
                tier_name: "Gold".to_string(),
                min_stake: 50_000,
                max_stake: 99_999,
                apy_boost: 0.10,  // +10% APY
                exclusive_pools: true,
                governance_weight: 5,
            },
            RewardTier {
                tier_name: "Platinum".to_string(),
                min_stake: 100_000,
                max_stake: u64::MAX,
                apy_boost: 0.20,  // +20% APY
                exclusive_pools: true,
                governance_weight: 10,
            },
        ];
    }
    
    /// Initialize default staking pools
    fn init_default_pools(&mut self) {
        // FLUX Staking Pool - Standard
        self.create_pool(StakingPool {
            pool_id: "flux_standard".to_string(),
            name: "FLUX Standard Pool".to_string(),
            description: "Standard FLUX staking with 30-day lock".to_string(),
            token_symbol: "FLUX".to_string(),
            min_stake: 100,
            max_stake: 10_000_000,
            total_staked: 0,
            reward_rate: 0.12, // 12% APY
            lock_period_epochs: 30 * 24, // 30 days in epochs (1 epoch = 1 hour)
            early_unstake_penalty: 0.05,
            tier_multipliers: HashMap::new(),
            is_active: true,
            created_at_epoch: 0,
        });
        
        // FLUX Staking Pool - Flexible
        self.create_pool(StakingPool {
            pool_id: "flux_flexible".to_string(),
            name: "FLUX Flexible Pool".to_string(),
            description: "No lock period, lower APY".to_string(),
            token_symbol: "FLUX".to_string(),
            min_stake: 100,
            max_stake: 5_000_000,
            total_staked: 0,
            reward_rate: 0.05, // 5% APY
            lock_period_epochs: 0,
            early_unstake_penalty: 0.0,
            tier_multipliers: HashMap::new(),
            is_active: true,
            created_at_epoch: 0,
        });
        
        // FLUX Staking Pool - Long Term
        self.create_pool(StakingPool {
            pool_id: "flux_long_term".to_string(),
            name: "FLUX Long Term Pool".to_string(),
            description: "90-day lock for higher rewards".to_string(),
            token_symbol: "FLUX".to_string(),
            min_stake: 1000,
            max_stake: 20_000_000,
            total_staked: 0,
            reward_rate: 0.20, // 20% APY
            lock_period_epochs: 90 * 24,
            early_unstake_penalty: 0.10,
            tier_multipliers: HashMap::new(),
            is_active: true,
            created_at_epoch: 0,
        });
        
        // ATH Staking Pool - Governance
        self.create_pool(StakingPool {
            pool_id: "ath_governance".to_string(),
            name: "ATH Governance Pool".to_string(),
            description: "Stake ATH for governance voting power".to_string(),
            token_symbol: "ATH".to_string(),
            min_stake: 1000,
            max_stake: 5_000_000,
            total_staked: 0,
            reward_rate: 0.08, // 8% APY
            lock_period_epochs: 7 * 24, // 7 days
            early_unstake_penalty: 0.02,
            tier_multipliers: HashMap::new(),
            is_active: true,
            created_at_epoch: 0,
        });
        
        // Update pool count
        self.global_stats.pool_count = self.pools.len() as u64;
    }
    
    /// Create a new staking pool
    pub fn create_pool(&mut self, pool: StakingPool) {
        let pool_state = StakingPoolState {
            pool: pool.clone(),
            stakers: HashMap::new(),
            total_positions: 0,
            active_positions: 0,
            average_stake_age: 0.0,
            total_rewards_distributed: 0,
            epoch_rewards: VecDeque::new(),
        };
        self.pools.insert(pool.pool_id.clone(), pool_state);
    }
    
    /// Stake tokens in a pool
    pub fn stake(&mut self, request: StakeRequest) -> Result<StakeResult, &'static str> {
        if self.emergency_pause {
            return Err("Staking is paused");
        }
        
        let pool_state = self.pools.get_mut(&request.pool_id)
            .ok_or("Pool not found")?;
        
        if !pool_state.pool.is_active {
            return Err("Pool is not active");
        }
        
        // Validate amount
        if request.amount < pool_state.pool.min_stake {
            return Err("Amount below minimum stake");
        }
        
        if request.amount > pool_state.pool.max_stake {
            return Err("Amount exceeds maximum stake");
        }
        
        // Determine tier
        let tier = self.calculate_tier(request.amount);
        let multiplier = self.get_tier_multiplier(&tier);
        
        // Calculate lock end epoch
        let lock_end = if pool_state.pool.lock_period_epochs > 0 {
            self.epoch + pool_state.pool.lock_period_epochs
        } else {
            self.epoch // No lock for flexible pool
        };
        
        // Create position
        let position_id = format!("pos_{}_{}_{}", request.pool_id, request.amount, self.epoch);
        let position = StakePosition {
            position_id: position_id.clone(),
            pool_id: request.pool_id.clone(),
            owner: "user".to_string(), // Would be passed in real implementation
            amount: request.amount,
            tier: tier.clone(),
            multiplier,
            start_epoch: self.epoch,
            lock_end_epoch: lock_end,
            last_claim_epoch: self.epoch,
            accumulated_rewards: 0,
            is_locked: pool_state.pool.lock_period_epochs > 0,
            is_active: true,
        };
        
        // Update pool state
        pool_state.total_staked += request.amount;
        pool_state.total_positions += 1;
        pool_state.active_positions += 1;
        pool_state.stakers.insert(position_id.clone(), position);
        
        // Update global stats
        self.update_global_stats();
        
        // Calculate estimated APY
        let base_apy = pool_state.pool.reward_rate;
        let effective_apy = base_apy * multiplier;
        
        Ok(StakeResult {
            position_id,
            pool_id: request.pool_id,
            amount: request.amount,
            tier,
            effective_multiplier: multiplier,
            lock_end_epoch: lock_end,
            first_rewards_at_epoch: self.epoch + 1,
            estimated_apy: effective_apy * 100.0,
        })
    }
    
    /// Request unstake from a position
    pub fn unstake(&mut self, request: UnstakeRequest) -> Result<UnstakeResult, &'static str> {
        if self.emergency_pause {
            return Err("Unstaking is paused");
        }
        
        // Find position
        let mut position_opt: Option<StakePosition> = None;
        let mut pool_id: Option<String> = None;
        
        for (pid, pool_state) in &mut self.pools {
            if let Some(pos) = pool_state.stakers.get_mut(&request.position_id) {
                position_opt = Some(pos.clone());
                pool_id = Some(pid.clone());
                break;
            }
        }
        
        let mut position = position_opt.ok_or("Position not found")?;
        let pid = pool_id.unwrap();
        
        // Check if can unstake
        let is_early = self.epoch < position.lock_end_epoch;
        let amount_to_unstake = request.amount.unwrap_or(position.amount);
        
        if amount_to_unstake > position.amount {
            return Err("Amount exceeds staked amount");
        }
        
        // Calculate penalty if early
        let penalty = if is_early && request.force_early {
            let penalty_amount = (amount_to_unstake as f64 * self.pools.get(&pid).unwrap().pool.early_unstake_penalty) as u64;
            penalty_amount
        } else if is_early {
            return Err("Position is still locked");
        } else {
            0
        };
        
        // Claim accumulated rewards
        let rewards = self.calculate_position_rewards(&position, &pid);
        position.accumulated_rewards += rewards;
        
        // Calculate final amounts
        let amount_returned = amount_to_unstake - penalty;
        let final_balance = position.amount - amount_to_unstake;
        
        // Update position
        if final_balance == 0 {
            position.is_active = false;
        } else {
            position.amount = final_balance;
        }
        
        // Update pool state
        let pool_state = self.pools.get_mut(&pid).unwrap();
        pool_state.total_staked -= amount_to_unstake;
        pool_state.active_positions -= 1;
        
        // Update global stats
        self.update_global_stats();
        
        Ok(UnstakeResult {
            position_id: request.position_id,
            amount_returned,
            rewards_claimed: rewards,
            penalty_applied: penalty,
            penalty_destination: self.treasury_address.clone(),
            final_balance,
        })
    }
    
    /// Calculate rewards for a position
    pub fn calculate_position_rewards(&self, position: &StakePosition, pool_id: &str) -> u64 {
        let pool = self.pools.get(pool_id).unwrap();
        
        if !position.is_active {
            return 0;
        }
        
        let epochs_elapsed = self.epoch - position.last_claim_epoch;
        if epochs_elapsed == 0 {
            return 0;
        }
        
        // Base reward rate per epoch
        let epochs_per_year = 365.0 * 24.0;
        let annual_rate = pool.pool.reward_rate;
        let epoch_rate = annual_rate / epochs_per_year;
        
        // Apply tier multiplier
        let effective_rate = epoch_rate * position.multiplier;
        
        // Calculate reward
        let reward = (position.amount as f64 * effective_rate * epochs_elapsed as f64) as u64;
        reward
    }
    
    /// Claim rewards from positions
    pub fn claim_rewards(&mut self, request: ClaimRequest) -> Result<ClaimResult, &'static str> {
        if self.emergency_pause {
            return Err("Claiming is paused");
        }
        
        let mut total_rewards = 0u64;
        let mut breakdown = Vec::new();
        let mut positions_claimed = 0u64;
        
        for (pool_id, pool_state) in &mut self.pools {
            for pos in pool_state.stakers.values_mut() {
                if request.position_ids.contains(&pos.position_id) && pos.is_active {
                    let accumulated = self.calculate_position_rewards(pos, pool_id);
                    pos.accumulated_rewards += accumulated;
                    pos.last_claim_epoch = self.epoch;
                    
                    breakdown.push(PositionClaimBreakdown {
                        position_id: pos.position_id.clone(),
                        accumulated: pos.accumulated_rewards,
                        claimed_this_epoch: accumulated,
                        new_total: pos.accumulated_rewards,
                    });
                    
                    total_rewards += accumulated;
                    positions_claimed += 1;
                }
            }
        }
        
        // Update global stats
        self.global_stats.total_rewards_distributed += total_rewards;
        
        Ok(ClaimResult {
            positions_claimed,
            total_rewards,
            breakdown,
            reinvest: request.reinvest,
        })
    }
    
    /// Calculate which tier an amount qualifies for
    pub fn calculate_tier(&self, amount: u64) -> String {
        for tier in self.tiers.iter().rev() {
            if amount >= tier.min_stake {
                return tier.tier_name.clone();
            }
        }
        "Bronze".to_string()
    }
    
    /// Get multiplier for a tier
    pub fn get_tier_multiplier(&self, tier_name: &str) -> f64 {
        for tier in &self.tiers {
            if tier.tier_name == tier_name {
                return 1.0 + tier.apy_boost;
            }
        }
        1.0
    }
    
    /// Update global staking statistics
    pub fn update_global_stats(&mut self) {
        let mut total: u64 = 0;
        let mut largest: u64 = 0;
        let mut largest_pool = String::new();
        
        for (pool_id, pool_state) in &self.pools {
            total += pool_state.total_staked;
            if pool_state.total_staked > largest {
                largest = pool_state.total_staked;
                largest_pool = pool_id.clone();
            }
        }
        
        self.global_stats.total_staked = total;
        self.global_stats.largest_pool = largest_pool;
        
        // Count unique stakers
        let mut unique_stakers: std::collections::HashSet<String> = std::collections::HashSet::new();
        for pool_state in self.pools.values() {
            for pos in pool_state.stakers.values() {
                if pos.is_active {
                    unique_stakers.insert(pos.owner.clone());
                }
            }
        }
        self.global_stats.total_stakers = unique_stakers.len() as u64;
        
        // Calculate average APY
        if !self.pools.is_empty() {
            let total_apy: f64 = self.pools.values().map(|p| p.pool.reward_rate).sum();
            self.global_stats.average_apy = total_apy / self.pools.len() as f64;
        }
    }
    
    /// Advance epoch and process epoch rewards
    pub fn advance_epoch(&mut self) {
        self.epoch += 1;
        
        // Record epoch rewards for each pool
        for (pool_id, pool_state) in &mut self.pools {
            let epoch_reward = EpochReward {
                epoch: self.epoch,
                total_stake: pool_state.total_staked,
                rewards_issued: 0,
                staker_count: pool_state.active_positions as u64,
            };
            
            // Keep only last 365 epochs
            if pool_state.epoch_rewards.len() >= 365 {
                pool_state.epoch_rewards.pop_front();
            }
            pool_state.epoch_rewards.push_back(epoch_reward);
        }
    }
    
    /// Get pool statistics
    pub fn get_pool_stats(&self, pool_id: &str) -> Option<PoolStats> {
        let pool_state = self.pools.get(pool_id)?;
        
        Some(PoolStats {
            pool_id: pool_id.to_string(),
            name: pool_state.pool.name.clone(),
            total_staked: pool_state.total_staked,
            active_positions: pool_state.active_positions,
            reward_rate: pool_state.pool.reward_rate * 100.0,
            min_stake: pool_state.pool.min_stake,
            max_stake: pool_state.pool.max_stake,
            lock_period_epochs: pool_state.pool.lock_period_epochs,
            is_active: pool_state.pool.is_active,
            total_rewards_distributed: pool_state.total_rewards_distributed,
        })
    }
    
    /// Get user's stake positions
    pub fn get_user_positions(&self, owner: &str) -> Vec<UserPositionSummary> {
        let mut positions = Vec::new();
        
        for (pool_id, pool_state) in &self.pools {
            for pos in pool_state.stakers.values() {
                if pos.owner == owner && pos.is_active {
                    positions.push(UserPositionSummary {
                        position_id: pos.position_id.clone(),
                        pool_id: pool_id.clone(),
                        pool_name: pool_state.pool.name.clone(),
                        amount: pos.amount,
                        tier: pos.tier.clone(),
                        current_rewards: self.calculate_position_rewards(pos, pool_id) + pos.accumulated_rewards,
                        start_epoch: pos.start_epoch,
                        lock_end_epoch: pos.lock_end_epoch,
                        is_locked: self.epoch < pos.lock_end_epoch,
                    });
                }
            }
        }
        
        positions
    }
    
    /// Emergency pause all staking operations
    pub fn emergency_pause_staking(&mut self, reason: &str) {
        self.emergency_pause = true;
        // In production, would emit event
    }
    
    /// Resume staking after emergency
    pub fn resume_staking(&mut self) {
        self.emergency_pause = false;
    }
    
    /// Get staking dashboard data
    pub fn get_dashboard(&self) -> StakingDashboard {
        StakingDashboard {
            global_stats: self.global_stats.clone(),
            pools: self.pools.keys().cloned().collect(),
            current_epoch: self.epoch,
            is_paused: self.emergency_pause,
            total_value_staked: self.global_stats.total_staked,
            average_apy: self.global_stats.average_apy * 100.0,
        }
    }
}

/// Pool statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub pool_id: String,
    pub name: String,
    pub total_staked: u64,
    pub active_positions: u64,
    pub reward_rate: f64,
    pub min_stake: u64,
    pub max_stake: u64,
    pub lock_period_epochs: u64,
    pub is_active: bool,
    pub total_rewards_distributed: u64,
}

/// User position summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPositionSummary {
    pub position_id: String,
    pub pool_id: String,
    pub pool_name: String,
    pub amount: u64,
    pub tier: String,
    pub current_rewards: u64,
    pub start_epoch: u64,
    pub lock_end_epoch: u64,
    pub is_locked: bool,
}

/// Staking dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingDashboard {
    pub global_stats: GlobalStakingStats,
    pub pools: Vec<String>,
    pub current_epoch: u64,
    pub is_paused: bool,
    pub total_value_staked: u64,
    pub average_apy: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_staking_contract_creation() {
        let contract = StakingContract::new("owner".to_string());
        assert_eq!(contract.pools.len(), 4);
        assert_eq!(contract.tiers.len(), 4);
    }
    
    #[test]
    fn test_tier_calculation() {
        let contract = StakingContract::new("owner".to_string());
        assert_eq!(contract.calculate_tier(500), "Bronze");
        assert_eq!(contract.calculate_tier(10_000), "Silver");
        assert_eq!(contract.calculate_tier(50_000), "Gold");
        assert_eq!(contract.calculate_tier(100_000), "Platinum");
    }
    
    #[test]
    fn test_stake_creation() {
        let mut contract = StakingContract::new("owner".to_string());
        let result = contract.stake(StakeRequest {
            pool_id: "flux_standard".to_string(),
            amount: 5000,
            auto_compound: false,
            tier_override: None,
        });
        assert!(result.is_ok());
    }
}
