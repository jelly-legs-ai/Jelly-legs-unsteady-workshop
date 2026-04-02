// Enhanced Staking Contract - AeTHer Chain
// Dual-token (FLUX/ATH) staking with flexible lock periods, boosted rewards, and auto-compounding

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced staking contract with dual-token support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedStakingContract {
    pub name: String,
    pub version: String,
    pub supported_tokens: Vec<StakingToken>,
    pub staking_pools: HashMap<String, StakingPool>,
    pub user_positions: HashMap<String, Vec<StakingPosition>>,
    pub reward_pool: HashMap<String, u64>,
    pub total_staked: HashMap<String, u64>,
    pub total_rewards_distributed: u64,
    pub contract_state: StakingState,
    pub admin_address: String,
    pub fee_recipient: String,
    pub platform_fee_percent: f64,
}

/// Supported staking token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingToken {
    pub token_id: String,
    pub token_name: String,
    pub token_symbol: String,
    pub contract_address: String,
    pub decimals: u8,
    is_active: bool,
    min_stake: u64,
    max_stake: u64,
}

/// Staking pool with lock period options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub pool_id: String,
    pub token_id: String,
    pub pool_name: String,
    pub total_staked: u64,
    pub total_stakers: u64,
    pub base_apy: f64,
    pub lock_periods: Vec<LockPeriod>,
    pub reward_multiplier: f64,
    pub is_active: bool,
    pub created_at: u64,
    pub last_reward_update: u64,
}

/// Lock period option with multiplier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockPeriod {
    pub period_id: String,
    pub name: String,
    pub duration_days: u64,
    pub duration_epochs: u64,
    pub reward_multiplier: f64,
    pub min_stake: u64,
    pub early_withdrawal_penalty: f64,
    pub is_active: bool,
}

/// User staking position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    pub position_id: String,
    pub user_address: String,
    pub pool_id: String,
    pub token_id: String,
    pub staked_amount: u64,
    pub lock_period_id: String,
    pub staked_at: u64,
    pub unlocks_at: u64,
    pub pending_rewards: u64,
    pub claimed_rewards: u64,
    pub last_reward_calc: u64,
    pub is_locked: bool,
    pub is_active: bool,
    pub auto_compound: bool,
}

/// Reward calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardCalculation {
    pub position_id: String,
    pub principal: u64,
    pub base_rewards: u64,
    pub multiplier_bonus: u64,
    pub loyalty_bonus: u64,
    pub total_rewards: u64,
    pub apy_effective: f64,
    pub days_staked: u64,
}

/// Staking state enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StakingState {
    Active,
    Paused,
    EmergencyStop,
    Maintenance,
}

/// Staking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStats {
    pub total_value_locked: u64,
    pub total_stakers: u64,
    pub total_positions: u64,
    pub total_rewards_distributed: u64,
    pub average_stake_size: u64,
    pub average_lock_duration: f64,
    pub pool_breakdown: Vec<PoolBreakdown>,
    pub token_breakdown: Vec<TokenBreakdown>,
}

/// Pool breakdown for stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolBreakdown {
    pub pool_id: String,
    pub pool_name: String,
    pub total_staked: u64,
    pub staker_count: u64,
    pub apy: f64,
}

/// Token breakdown for stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBreakdown {
    pub token_id: String,
    pub token_symbol: String,
    pub total_staked: u64,
    pub staker_count: u64,
    pub percentage_of_tvl: f64,
}

/// User staking summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStakingSummary {
    pub user_address: String,
    pub total_staked: u64,
    pub total_pending_rewards: u64,
    pub total_claimed_rewards: u64,
    pub active_positions: u64,
    pub positions: Vec<PositionSummary>,
    pub estimated_apy: f64,
}

/// Position summary for user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSummary {
    pub position_id: String,
    pub pool_name: String,
    pub token_symbol: String,
    pub staked_amount: u64,
    pub pending_rewards: u64,
    pub days_remaining: u64,
    pub apy: f64,
    pub is_locked: bool,
}

impl EnhancedStakingContract {
    /// Create new enhanced staking contract
    pub fn new() -> Self {
        let mut supported_tokens = Vec::new();
        let mut staking_pools = HashMap::new();
        let mut reward_pool = HashMap::new();
        let mut total_staked = HashMap::new();
        
        // Supported tokens
        supported_tokens.push(StakingToken {
            token_id: "flux".to_string(),
            token_name: "FLUX Token".to_string(),
            token_symbol: "FLUX".to_string(),
            contract_address: "0xflux...".to_string(),
            decimals: 8,
            is_active: true,
            min_stake: 100,
            max_stake: 10_000_000,
        });
        
        supported_tokens.push(StakingToken {
            token_id: "ath".to_string(),
            token_name: "AeTHer Governance Token".to_string(),
            token_symbol: "ATH".to_string(),
            contract_address: "0xath...".to_string(),
            decimals: 8,
            is_active: true,
            min_stake: 50,
            max_stake: 5_000_000,
        });
        
        // FLUX staking pools with multiple lock periods
        let flux_pool = StakingPool {
            pool_id: "flux_flexible".to_string(),
            token_id: "flux".to_string(),
            pool_name: "FLUX Flexible Staking".to_string(),
            total_staked: 0,
            total_stakers: 0,
            base_apy: 0.12, // 12% base APY
            lock_periods: vec![
                LockPeriod {
                    period_id: "flux_7d".to_string(),
                    name: "7 Days".to_string(),
                    duration_days: 7,
                    duration_epochs: 7,
                    reward_multiplier: 1.0,
                    min_stake: 100,
                    early_withdrawal_penalty: 0.5, // 50% penalty
                    is_active: true,
                },
                LockPeriod {
                    period_id: "flux_30d".to_string(),
                    name: "30 Days".to_string(),
                    duration_days: 30,
                    duration_epochs: 30,
                    reward_multiplier: 1.25,
                    min_stake: 500,
                    early_withdrawal_penalty: 0.3,
                    is_active: true,
                },
                LockPeriod {
                    period_id: "flux_90d".to_string(),
                    name: "90 Days".to_string(),
                    duration_days: 90,
                    duration_epochs: 90,
                    reward_multiplier: 1.5,
                    min_stake: 1000,
                    early_withdrawal_penalty: 0.1,
                    is_active: true,
                },
                LockPeriod {
                    period_id: "flux_180d".to_string(),
                    name: "180 Days".to_string(),
                    duration_days: 180,
                    duration_epochs: 180,
                    reward_multiplier: 2.0,
                    min_stake: 5000,
                    early_withdrawal_penalty: 0.05,
                    is_active: true,
                },
                LockPeriod {
                    period_id: "flux_365d".to_string(),
                    name: "365 Days".to_string(),
                    duration_days: 365,
                    duration_epochs: 365,
                    reward_multiplier: 3.0,
                    min_stake: 10000,
                    early_withdrawal_penalty: 0.0,
                    is_active: true,
                },
            ],
            reward_multiplier: 1.0,
            is_active: true,
            created_at: 0,
            last_reward_update: 0,
        };
        staking_pools.insert("flux_flexible".to_string(), flux_pool);
        
        // ATH staking pools
        let ath_pool = StakingPool {
            pool_id: "ath_governance".to_string(),
            token_id: "ath".to_string(),
            pool_name: "ATH Governance Staking".to_string(),
            total_staked: 0,
            total_stakers: 0,
            base_apy: 0.18, // 18% base APY (higher for governance)
            lock_periods: vec![
                LockPeriod {
                    period_id: "ath_30d".to_string(),
                    name: "30 Days".to_string(),
                    duration_days: 30,
                    duration_epochs: 30,
                    reward_multiplier: 1.0,
                    min_stake: 50,
                    early_withdrawal_penalty: 0.5,
                    is_active: true,
                },
                LockPeriod {
                    period_id: "ath_90d".to_string(),
                    name: "90 Days".to_string(),
                    duration_days: 90,
                    duration_epochs: 90,
                    reward_multiplier: 1.5,
                    min_stake: 250,
                    early_withdrawal_penalty: 0.2,
                    is_active: true,
                },
                LockPeriod {
                    period_id: "ath_180d".to_string(),
                    name: "180 Days".to_string(),
                    duration_days: 180,
                    duration_epochs: 180,
                    reward_multiplier: 2.25,
                    min_stake: 1000,
                    early_withdrawal_penalty: 0.1,
                    is_active: true,
                },
                LockPeriod {
                    period_id: "ath_365d".to_string(),
                    name: "365 Days".to_string(),
                    duration_days: 365,
                    duration_epochs: 365,
                    reward_multiplier: 3.5,
                    min_stake: 5000,
                    early_withdrawal_penalty: 0.0,
                    is_active: true,
                },
            ],
            reward_multiplier: 1.0,
            is_active: true,
            created_at: 0,
            last_reward_update: 0,
        };
        staking_pools.insert("ath_governance".to_string(), ath_pool);
        
        // Initialize reward pools
        reward_pool.insert("flux".to_string(), 10_000_000); // 10M FLUX rewards
        reward_pool.insert("ath".to_string(), 5_000_000);   // 5M ATH rewards
        
        // Initialize total staked
        total_staked.insert("flux".to_string(), 0);
        total_staked.insert("ath".to_string(), 0);
        
        EnhancedStakingContract {
            name: "AeTHer Enhanced Staking".to_string(),
            version: "2.0.0".to_string(),
            supported_tokens,
            staking_pools,
            user_positions: HashMap::new(),
            reward_pool,
            total_staked,
            total_rewards_distributed: 0,
            contract_state: StakingState::Active,
            admin_address: "admin".to_string(),
            fee_recipient: "treasury".to_string(),
            platform_fee_percent: 0.02, // 2% platform fee on rewards
        }
    }
    
    /// Stake tokens in a pool
    pub fn stake(
        &mut self,
        user_address: &str,
        pool_id: &str,
        lock_period_id: &str,
        amount: u64,
        auto_compound: bool,
    ) -> Result<String, &'static str> {
        if self.contract_state != StakingState::Active {
            return Err("Staking is not active");
        }
        
        let pool = self.staking_pools.get_mut(pool_id)
            .ok_or("Pool not found")?;
        
        if !pool.is_active {
            return Err("Pool is not active");
        }
        
        let lock_period = pool.lock_periods.iter()
            .find(|lp| lp.period_id == lock_period_id && lp.is_active)
            .ok_or("Invalid lock period")?;
        
        if amount < lock_period.min_stake {
            return Err(&format!("Minimum stake is {} tokens", lock_period.min_stake));
        }
        
        // Check token balance (in production, would verify actual balance)
        let token = self.supported_tokens.iter()
            .find(|t| t.token_id == pool.token_id)
            .ok_or("Token not found")?;
        
        if amount > token.max_stake {
            return Err(&format!("Maximum stake is {} tokens", token.max_stake));
        }
        
        // Create staking position
        let position_id = format!("pos_{}_{}", user_address, self.total_staked.get(&pool.token_id).unwrap_or(&0));
        let current_epoch = self.total_staked.get(&pool.token_id).unwrap_or(&0);
        
        let position = StakingPosition {
            position_id: position_id.clone(),
            user_address: user_address.to_string(),
            pool_id: pool_id.to_string(),
            token_id: pool.token_id.clone(),
            staked_amount: amount,
            lock_period_id: lock_period_id.to_string(),
            staked_at: current_epoch,
            unlocks_at: current_epoch + lock_period.duration_epochs,
            pending_rewards: 0,
            claimed_rewards: 0,
            last_reward_calc: current_epoch,
            is_locked: true,
            is_active: true,
            auto_compound,
        };
        
        // Update pool stats
        pool.total_staked += amount;
        pool.total_stakers += 1;
        
        // Update global stats
        *self.total_staked.entry(pool.token_id.clone()).or_insert(0) += amount;
        
        // Add to user positions
        self.user_positions
            .entry(user_address.to_string())
            .or_insert_with(Vec::new)
            .push(position);
        
        Ok(position_id)
    }
    
    /// Unstake tokens (with early withdrawal penalty if applicable)
    pub fn unstake(&mut self, user_address: &str, position_id: &str) -> Result<UnstakeResult, &'static str> {
        let positions = self.user_positions.get_mut(user_address)
            .ok_or("User has no positions")?;
        
        let position_idx = positions.iter()
            .position(|p| p.position_id == position_id)
            .ok_or("Position not found")?;
        
        let position = &positions[position_idx];
        
        if !position.is_active {
            return Err("Position is already withdrawn");
        }
        
        let pool = self.staking_pools.get(&position.pool_id)
            .ok_or("Pool not found")?;
        
        let lock_period = pool.lock_periods.iter()
            .find(|lp| lp.period_id == position.lock_period_id)
            .ok_or("Lock period not found")?;
        
        let current_epoch = self.total_staked.get(&position.token_id).unwrap_or(&0);
        let is_unlocked = current_epoch >= &position.unlocks_at;
        
        // Calculate final rewards
        let rewards = self.calculate_rewards(user_address, position_id)?;
        
        // Apply early withdrawal penalty if needed
        let (final_amount, penalty_amount) = if !is_unlocked && lock_period.early_withdrawal_penalty > 0.0 {
            let penalty = (position.staked_amount as f64 * lock_period.early_withdrawal_penalty) as u64;
            (position.staked_amount - penalty, penalty)
        } else {
            (position.staked_amount, 0)
        };
        
        let total_payout = final_amount + rewards.total_rewards;
        
        // Update pool stats
        let pool = self.staking_pools.get_mut(&position.pool_id).unwrap();
        pool.total_staked -= position.staked_amount;
        pool.total_stakers -= 1;
        
        // Update global stats
        *self.total_staked.entry(position.token_id.clone()).or_insert(0) -= position.staked_amount;
        
        // Mark position as inactive
        positions[position_idx].is_active = false;
        positions[position_idx].is_locked = false;
        
        // Update total rewards distributed
        self.total_rewards_distributed += rewards.total_rewards;
        
        // Deduct from reward pool
        *self.reward_pool.entry(position.token_id.clone()).or_insert(0) -= rewards.total_rewards;
        
        Ok(UnstakeResult {
            position_id: position_id.to_string(),
            staked_amount: position.staked_amount,
            rewards_earned: rewards.total_rewards,
            penalty_applied: penalty_amount,
            final_payout: total_payout,
            token_id: position.token_id.clone(),
        })
    }
    
    /// Calculate rewards for a position
    pub fn calculate_rewards(&self, user_address: &str, position_id: &str) -> Result<RewardCalculation, &'static str> {
        let positions = self.user_positions.get(user_address)
            .ok_or("User has no positions")?;
        
        let position = positions.iter()
            .find(|p| p.position_id == position_id)
            .ok_or("Position not found")?;
        
        let pool = self.staking_pools.get(&position.pool_id)
            .ok_or("Pool not found")?;
        
        let lock_period = pool.lock_periods.iter()
            .find(|lp| lp.period_id == position.lock_period_id)
            .ok_or("Lock period not found")?;
        
        let current_epoch = self.total_staked.get(&position.token_id).unwrap_or(&0);
        let epochs_staked = current_epoch - position.last_reward_calc;
        
        if epochs_staked == 0 {
            return Ok(RewardCalculation {
                position_id: position_id.to_string(),
                principal: position.staked_amount,
                base_rewards: 0,
                multiplier_bonus: 0,
                loyalty_bonus: 0,
                total_rewards: 0,
                apy_effective: pool.base_apy,
                days_staked: 0,
            });
        }
        
        // Base rewards calculation
        let daily_rate = pool.base_apy / 365.0;
        let base_rewards = (position.staked_amount as f64 * daily_rate * epochs_staked as f64) as u64;
        
        // Lock period multiplier bonus
        let multiplier_bonus = (base_rewards as f64 * (lock_period.reward_multiplier - 1.0)) as u64;
        
        // Loyalty bonus (additional rewards for long-term stakers)
        let total_epochs_staked = current_epoch - position.staked_at;
        let loyalty_multiplier = if total_epochs_staked > 365 {
            1.2
        } else if total_epochs_staked > 180 {
            1.1
        } else if total_epochs_staked > 90 {
            1.05
        } else {
            1.0
        };
        let loyalty_bonus = ((base_rewards + multiplier_bonus) as f64 * (loyalty_multiplier - 1.0)) as u64;
        
        let total_rewards = base_rewards + multiplier_bonus + loyalty_bonus;
        
        // Calculate effective APY
        let apy_effective = if position.staked_amount > 0 {
            (total_rewards as f64 / position.staked_amount as f64) * (365.0 / epochs_staked as f64)
        } else {
            0.0
        };
        
        Ok(RewardCalculation {
            position_id: position_id.to_string(),
            principal: position.staked_amount,
            base_rewards,
            multiplier_bonus,
            loyalty_bonus,
            total_rewards,
            apy_effective,
            days_staked: epochs_staked,
        })
    }
    
    /// Claim accumulated rewards
    pub fn claim_rewards(&mut self, user_address: &str, position_id: &str) -> Result<u64, &'static str> {
        let positions = self.user_positions.get_mut(user_address)
            .ok_or("User has no positions")?;
        
        let position = positions.iter_mut()
            .find(|p| p.position_id == position_id)
            .ok_or("Position not found")?;
        
        if !position.is_active {
            return Err("Position is not active");
        }
        
        let rewards = self.calculate_rewards(user_address, position_id)?;
        
        if rewards.total_rewards == 0 {
            return Ok(0);
        }
        
        // Check reward pool has sufficient balance
        let pool_balance = self.reward_pool.get(&position.token_id).copied().unwrap_or(0);
        if pool_balance < rewards.total_rewards {
            return Err("Insufficient reward pool balance");
        }
        
        // Apply platform fee
        let platform_fee = (rewards.total_rewards as f64 * self.platform_fee_percent) as u64;
        let net_rewards = rewards.total_rewards - platform_fee;
        
        // Update position
        position.claimed_rewards += net_rewards;
        position.pending_rewards = 0;
        position.last_reward_calc = self.total_staked.get(&position.token_id).unwrap_or(&0);
        
        // Auto-compound if enabled
        if position.auto_compound {
            position.staked_amount += net_rewards;
            
            // Update pool and global stats
            let pool = self.staking_pools.get_mut(&position.pool_id).unwrap();
            pool.total_staked += net_rewards;
            *self.total_staked.entry(position.token_id.clone()).or_insert(0) += net_rewards;
        }
        
        // Update reward pool
        *self.reward_pool.entry(position.token_id.clone()).or_insert(0) -= rewards.total_rewards;
        
        // Update fee recipient
        *self.reward_pool.entry("platform_fees".to_string()).or_insert(0) += platform_fee;
        
        // Update total distributed
        self.total_rewards_distributed += net_rewards;
        
        Ok(net_rewards)
    }
    
    /// Get user staking summary
    pub fn get_user_summary(&self, user_address: &str) -> UserStakingSummary {
        let positions = self.user_positions.get(user_address);
        
        if positions.is_none() || positions.as_ref().unwrap().is_empty() {
            return UserStakingSummary {
                user_address: user_address.to_string(),
                total_staked: 0,
                total_pending_rewards: 0,
                total_claimed_rewards: 0,
                active_positions: 0,
                positions: Vec::new(),
                estimated_apy: 0.0,
            };
        }
        
        let positions = positions.unwrap();
        let mut total_staked = 0u64;
        let mut total_pending = 0u64;
        let mut total_claimed = 0u64;
        let mut active_count = 0u64;
        let mut position_summaries = Vec::new();
        let mut total_apy_weighted = 0.0;
        
        for position in positions {
            if !position.is_active {
                continue;
            }
            
            total_staked += position.staked_amount;
            total_claimed += position.claimed_rewards;
            active_count += 1;
            
            // Calculate pending rewards
            if let Ok(rewards) = self.calculate_rewards(user_address, &position.position_id) {
                total_pending += rewards.total_rewards;
                total_apy_weighted += rewards.apy_effective * position.staked_amount as f64;
            }
            
            let pool = self.staking_pools.get(&position.pool_id);
            let current_epoch = self.total_staked.get(&position.token_id).unwrap_or(&0);
            let days_remaining = if position.unlocks_at > *current_epoch {
                (position.unlocks_at - current_epoch)
            } else {
                0
            };
            
            position_summaries.push(PositionSummary {
                position_id: position.position_id.clone(),
                pool_name: pool.map(|p| p.pool_name.clone()).unwrap_or_default(),
                token_symbol: self.supported_tokens.iter()
                    .find(|t| t.token_id == position.token_id)
                    .map(|t| t.token_symbol.clone())
                    .unwrap_or_default(),
                staked_amount: position.staked_amount,
                pending_rewards: total_pending,
                days_remaining,
                apy: pool.map(|p| p.base_apy).unwrap_or(0.0),
                is_locked: position.is_locked && current_epoch < &position.unlocks_at,
            });
        }
        
        let estimated_apy = if total_staked > 0 {
            total_apy_weighted / total_staked as f64
        } else {
            0.0
        };
        
        UserStakingSummary {
            user_address: user_address.to_string(),
            total_staked,
            total_pending_rewards: total_pending,
            total_claimed_rewards: total_claimed,
            active_positions: active_count,
            positions: position_summaries,
            estimated_apy,
        }
    }
    
    /// Get staking statistics
    pub fn get_staking_stats(&self) -> StakingStats {
        let total_value_locked: u64 = self.total_staked.values().sum();
        let total_positions: u64 = self.user_positions.values()
            .map(|p| p.len() as u64)
            .sum();
        let total_stakers = self.user_positions.len() as u64;
        
        let pool_breakdown: Vec<PoolBreakdown> = self.staking_pools.values()
            .map(|p| PoolBreakdown {
                pool_id: p.pool_id.clone(),
                pool_name: p.pool_name.clone(),
                total_staked: p.total_staked,
                staker_count: p.total_stakers,
                apy: p.base_apy,
            })
            .collect();
        
        let token_breakdown: Vec<TokenBreakdown> = self.supported_tokens.iter()
            .filter(|t| t.is_active)
            .map(|t| {
                let staked = self.total_staked.get(&t.token_id).copied().unwrap_or(0);
                let percentage = if total_value_locked > 0 {
                    staked as f64 / total_value_locked as f64
                } else {
                    0.0
                };
                
                let staker_count = self.user_positions.values()
                    .flat_map(|p| p.iter())
                    .filter(|pos| pos.token_id == t.token_id && pos.is_active)
                    .count() as u64;
                
                TokenBreakdown {
                    token_id: t.token_id.clone(),
                    token_symbol: t.token_symbol.clone(),
                    total_staked: staked,
                    staker_count,
                    percentage_of_tvl: percentage,
                }
            })
            .collect();
        
        let average_stake_size = if total_positions > 0 {
            total_value_locked / total_positions
        } else {
            0
        };
        
        let average_lock_duration = 90.0; // Simplified - would calculate from actual positions
        
        StakingStats {
            total_value_locked,
            total_stakers,
            total_positions,
            total_rewards_distributed: self.total_rewards_distributed,
            average_stake_size,
            average_lock_duration,
            pool_breakdown,
            token_breakdown,
        }
    }
    
    /// Add rewards to reward pool (admin function)
    pub fn add_rewards(&mut self, token_id: &str, amount: u64) -> Result<(), &'static str> {
        if self.contract_state != StakingState::Active {
            return Err("Staking is not active");
        }
        
        *self.reward_pool.entry(token_id.to_string()).or_insert(0) += amount;
        Ok(())
    }
    
    /// Pause staking
    pub fn pause_staking(&mut self) {
        self.contract_state = StakingState::Paused;
    }
    
    /// Resume staking
    pub fn resume_staking(&mut self) {
        self.contract_state = StakingState::Active;
    }
    
    /// Emergency stop
    pub fn emergency_stop(&mut self) {
        self.contract_state = StakingState::EmergencyStop;
    }
    
    /// Get available pools for a token
    pub fn get_pools_for_token(&self, token_id: &str) -> Vec<&StakingPool> {
        self.staking_pools.values()
            .filter(|p| p.token_id == token_id && p.is_active)
            .collect()
    }
    
    /// Get lock periods for a pool
    pub fn get_lock_periods(&self, pool_id: &str) -> Option<&Vec<LockPeriod>> {
        self.staking_pools.get(pool_id).map(|p| &p.lock_periods)
    }
    
    /// Calculate optimal lock period for desired APY
    pub fn recommend_lock_period(&self, pool_id: &str, desired_apy: f64) -> Option<&LockPeriod> {
        let pool = self.staking_pools.get(pool_id)?;
        
        let effective_apy_target = desired_apy / pool.base_apy;
        
        pool.lock_periods.iter()
            .filter(|lp| lp.is_active && lp.reward_multiplier >= effective_apy_target)
            .min_by(|a, b| a.reward_multiplier.partial_cmp(&b.reward_multiplier).unwrap())
    }
}

/// Unstake result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeResult {
    pub position_id: String,
    pub staked_amount: u64,
    pub rewards_earned: u64,
    pub penalty_applied: u64,
    pub final_payout: u64,
    pub token_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_staking_creation() {
        let staking = EnhancedStakingContract::new();
        assert_eq!(staking.staking_pools.len(), 2);
        assert_eq!(staking.contract_state, StakingState::Active);
    }
    
    #[test]
    fn test_stake_tokens() {
        let mut staking = EnhancedStakingContract::new();
        let position_id = staking.stake("user1", "flux_flexible", "flux_30d", 1000, false).unwrap();
        assert!(position_id.starts_with("pos_user1_"));
    }
    
    #[test]
    fn test_reward_calculation() {
        let mut staking = EnhancedStakingContract::new();
        staking.stake("user1", "flux_flexible", "flux_30d", 1000, false).unwrap();
        
        // Simulate time passing
        *staking.total_staked.get_mut("flux").unwrap() += 10;
        
        let rewards = staking.calculate_rewards("user1", "pos_user1_0").unwrap();
        assert!(rewards.total_rewards > 0);
    }
}
