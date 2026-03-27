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
    
    // =============================================================================
    // DELEGATION & VALIDATOR HELPERS - Sprint Enhancement
    // =============================================================================
    
    /// Calculate share of network rewards (as percentage) for a given stake amount
    pub fn calculate_network_share(&self, pool_id: &str, amount: u64) -> f64 {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return 0.0,
        };
        
        if pool.total_staked == 0 {
            return 0.0
        }
        
        (amount as f64 / pool.total_staked as f64) * 100.0
    }
    
    /// Calculate validator uptime score based on metrics
    pub fn get_validator_uptime(&self, validator: &str) -> f64 {
        self.validator_metrics.get(validator)
            .map(|m| m.uptime_percent)
            .unwrap_or(0.0)
    }
    
    /// Get validator delegator count
    pub fn get_validator_delegator_count(&self, validator: &str) -> u64 {
        self.validator_metrics.get(validator)
            .map(|m| m.delegator_count)
            .unwrap_or(0)
    }
    
    /// Get validator total delegated amount
    pub fn get_validator_total_delegated(&self, validator: &str) -> u64 {
        self.validator_metrics.get(validator)
            .map(|m| m.total_delegated)
            .unwrap_or(0)
    }
    
    /// Calculate net APY after commission for delegators
    pub fn calculate_net_apy_after_commission(&self, pool_id: &str, validator: &str) -> f64 {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return 0.0,
        };
        
        let gross_apy = pool.reward_rate * 100.0;
        let commission = self.validator_metrics.get(validator)
            .map(|m| m.commission_rate)
            .unwrap_or(0.05);
        
        gross_apy * (1.0 - commission)
    }
    
    // =============================================================================
    // ADVANCED STAKING ANALYTICS - SPRINT ENHANCEMENT
    // =============================================================================
    
    /// Get staking pool distribution across all token types
    pub fn get_pool_distribution(&self) -> PoolDistribution {
        PoolDistribution {
            aeth: self.pools.get("aeth_staking").map(|p| (p.total_staked, p.active_stakers)).unwrap_or((0, 0)),
            flux: self.pools.get("flux_staking").map(|p| (p.total_staked, p.active_stakers)).unwrap_or((0, 0)),
            ath: self.pools.get("ath_staking").map(|p| (p.total_staked, p.active_stakers)).unwrap_or((0, 0)),
            total_staked: self.pools.values().map(|p| p.total_staked).sum(),
            total_stakers: self.pools.values().map(|p| p.active_stakers).sum(),
        }
    }
    
    /// Calculate staking concentration risk (Gini coefficient style)
    pub fn calculate_concentration_index(&self, pool_id: &str) -> f64 {
        let stakes = self.stakes.values().flatten().filter(|s| {
            match pool_id {
                "aeth_staking" => s.token_type == TokenType::AETH,
                "flux_staking" => s.token_type == TokenType::FLUX,
                "ath_staking" => s.token_type == TokenType::ATH,
                _ => false,
            }
        }).collect::<Vec<_>>();
        
        if stakes.is_empty() {
            return 0.0;
        }
        
        let total: u64 = stakes.iter().map(|s| s.amount).sum();
        let mut amounts: Vec<u64> = stakes.iter().map(|s| s.amount).collect();
        amounts.sort();
        
        // Simplified concentration: top 10% share
        let top_10_count = stakes.len().max(1) / 10;
        let top_10_total: u64 = amounts.iter().rev().take(top_10_count).sum();
        
        top_10_total as f64 / total as f64
    }
    
    /// Get average stake size per pool
    pub fn get_average_stake_size(&self, pool_id: &str) -> f64 {
        let stakes = self.stakes.values().flatten().filter(|s| {
            match pool_id {
                "aeth_staking" => s.token_type == TokenType::AETH,
                "flux_staking" => s.token_type == TokenType::FLUX,
                "ath_staking" => s.token_type == TokenType::ATH,
                _ => false,
            }
        }).collect::<Vec<_>>();
        
        if stakes.is_empty() {
            return 0.0;
        }
        
        let total: u64 = stakes.iter().map(|s| s.amount).sum();
        total as f64 / stakes.len() as f64
    }
    
    /// Calculate median stake size
    pub fn get_median_stake_size(&self, pool_id: &str) -> u64 {
        let mut amounts: Vec<u64> = self.stakes.values().flatten().filter(|s| {
            match pool_id {
                "aeth_staking" => s.token_type == TokenType::AETH,
                "flux_staking" => s.token_type == TokenType::FLUX,
                "ath_staking" => s.token_type == TokenType::ATH,
                _ => false,
            }
        }).map(|s| s.amount).collect();
        
        if amounts.is_empty() {
            return 0;
        }
        
        amounts.sort();
        let mid = amounts.len() / 2;
        if amounts.len() % 2 == 0 {
            (amounts[mid - 1] + amounts[mid]) / 2
        } else {
            amounts[mid]
        }
    }
    
    /// Get staking velocity (new stakes per epoch average)
    pub fn get_staking_velocity(&self, pool_id: &str, epochs_window: u64) -> f64 {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return 0.0,
        };
        
        if self.current_epoch < epochs_window {
            return pool.active_stakers as f64 / self.current_epoch as f64;
        }
        
        pool.active_stakers as f64 / epochs_window as f64
    }
    
    /// Calculate compound growth projection for reinvested rewards
    pub fn project_compound_growth(&self, pool_id: &str, initial_stake: u64, epochs: u64, reinvest: bool) -> CompoundProjection {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return CompoundProjection {
                final_amount: initial_stake,
                total_rewards: 0,
                effective_apy: 0.0,
                growth_multiple: 1.0,
            },
        };
        
        let daily_rate = pool.reward_rate / 365.0;
        let mut current = initial_stake;
        let mut total_rewards = 0u64;
        
        for _ in 0..epochs {
            let reward = (current as f64 * daily_rate) as u64;
            total_rewards += reward;
            if reinvest {
                current += reward;
            }
        }
        
        let effective_apy = if initial_stake > 0 {
            ((current as f64 - initial_stake as f64) / initial_stake as f64) * 100.0
        } else {
            0.0
        };
        
        CompoundProjection {
            final_amount: current,
            total_rewards,
            effective_apy,
            growth_multiple: current as f64 / initial_stake.max(1) as f64,
        }
    }
    
    /// Get optimal pool recommendation based on risk profile
    pub fn recommend_pool(&self, risk_tolerance: f64, lockup_preference: &str) -> PoolRecommendation {
        // risk_tolerance: 0.0 (conservative) to 1.0 (aggressive)
        // lockup_preference: "short", "medium", "long"
        
        let pools: Vec<&StakingPool> = self.pools.values().collect();
        
        let recommended = match (risk_tolerance, lockup_preference) {
            (r, "short") if r < 0.5 => pools.iter().find(|p| p.lockup_epochs <= 7 && r < 0.3),
            (r, "short") => pools.iter().find(|p| p.lockup_epochs <= 7),
            (r, "medium") if r < 0.5 => pools.iter().find(|p| p.lockup_epochs <= 14 && r < 0.3),
            (r, "medium") => pools.iter().find(|p| p.lockup_epochs <= 14),
            (r, "long") => pools.iter().find(|p| p.lockup_epochs > 14),
            _ => pools.iter().max_by(|a, b| (a.reward_rate * a.total_staked as f64).partial_cmp(&(b.reward_rate * b.total_staked as f64)).unwrap()),
        };
        
        match recommended {
            Some(pool) => PoolRecommendation {
                pool_id: self.pools.iter().find(|(_, p)| p == pool).map(|(k, _)| k.clone()).unwrap_or_default(),
                reason: format!("APY: {:.2}%, Lockup: {} epochs", pool.reward_rate * 100.0, pool.lockup_epochs),
                expected_apy: pool.reward_rate * 100.0,
                lockup_epochs: pool.lockup_epochs,
                min_stake: pool.min_stake,
            },
            None => PoolRecommendation {
                pool_id: "aeth_staking".to_string(),
                reason: "Default conservative pool".to_string(),
                expected_apy: 15.0,
                lockup_epochs: 7,
                min_stake: 100,
            },
        }
    }
    
    /// Calculate staking efficiency score (0-100)
    pub fn get_staking_efficiency_score(&self, pool_id: &str) -> StakingEfficiency {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return StakingEfficiency {
                score: 0,
                apy_component: 0,
                liquidity_component: 0,
                participation_component: 0,
                status: "No pool found".to_string(),
            },
        };
        
        // APY component (max 40 points)
        let apy_score = (pool.reward_rate * 100.0 / 30.0 * 40.0).min(40.0);
        
        // Liquidity component (max 30 points) - based on total staked
        let liquidity_score = (pool.total_staked as f64 / 10000.0 * 30.0).min(30.0);
        
        // Participation component (max 30 points) - based on active stakers
        let participation_score = (pool.active_stakers as f64 / 100.0 * 30.0).min(30.0);
        
        let total_score = apy_score + liquidity_score + participation_score;
        
        StakingEfficiency {
            score: total_score as u64,
            apy_component: apy_score as u64,
            liquidity_component: liquidity_score as u64,
            participation_component: participation_score as u64,
            status: if total_score >= 80.0 { "Excellent" }
                    else if total_score >= 60.0 { "Good" }
                    else if total_score >= 40.0 { "Fair" }
                    else { "Poor" }.to_string(),
        }
    }
    
    /// Check if a delegator has reached minimum stake for rewards
    pub fn is_minimum_delegation(&self, pool_id: &str, amount: u64) -> bool {
        self.pools.get(pool_id)
            .map(|p| amount >= p.min_stake)
            .unwrap_or(false)
    }
    
    /// Get pool stats summary
    pub fn get_pool_stats(&self, pool_id: &str) -> Option<PoolStats> {
        self.pools.get(pool_id).map(|p| PoolStats {
            name: p.name.clone(),
            total_staked: p.total_staked,
            reward_rate: p.reward_rate,
            apy: p.reward_rate * 100.0,
            min_stake: p.min_stake,
            lockup_epochs: p.lockup_epochs,
            active_stakers: p.active_stakers,
        })
    }
    
    /// Get all delegations for a delegator
    pub fn get_delegator_delegations(&self, delegator: &str) -> Vec<&DelegationInfo> {
        self.delegations.get(delegator)
            .map(|d| d.iter().collect())
            .unwrap_or_default()
    }
    
    /// Get all stakes for an address
    pub fn get_address_stakes(&self, address: &str) -> Vec<&StakeInfo> {
        self.stakes.get(address)
            .map(|s| s.iter().collect())
            .unwrap_or_default()
    }
    
    /// Calculate total staked value across all pools (in token units)
    pub fn total_staked_all_pools(&self) -> u64 {
        self.pools.values().map(|p| p.total_staked).sum()
    }
    
    /// Get network-wide staking metrics
    pub fn get_network_staking_metrics(&self) -> NetworkStakingMetrics {
        let total_staked = self.total_staked_all_pools();
        let mut total_delegated = 0u64;
        let mut total_validators = 0u64;
        let mut total_delegators = 0u64;
        
        for metrics in self.validator_metrics.values() {
            total_delegated += metrics.total_delegated;
            total_delegators += metrics.delegator_count;
            total_validators += 1;
        }
        
        NetworkStakingMetrics {
            total_staked,
            total_delegated,
            total_validators,
            total_delegators,
            average_commission: self.validator_metrics.values()
                .map(|m| m.commission_rate)
                .sum::<f64>() / total_validators.max(1) as f64,
            average_uptime: self.validator_metrics.values()
                .map(|m| m.uptime_percent)
                .sum::<f64>() / total_validators.max(1) as f64,
            total_rewards_distributed: self.total_rewards_distributed,
        }
    }
    
    // =============================================================================
    // MINING REWARD CALCULATION LOGIC - Sprint 3 Enhancement
    // =============================================================================
    
    /// Calculate mining reward based on staked amount and network participation
    pub fn calculate_mining_reward(&self, stake: &StakeInfo, network_participation: f64) -> u64 {
        let pool = match self.get_pool_by_token(&stake.token_type) {
            Some(p) => p,
            None => return 0,
        };
        
        // Base reward rate from pool
        let base_rate = pool.reward_rate;
        
        // Network participation bonus (0.5 to 1.5x)
        let participation_multiplier = if network_participation > 0.8 {
            1.5
        } else if network_participation > 0.5 {
            1.0
        } else {
            0.5
        };
        
        // Stake duration bonus (longer stakes earn more)
        let epochs_staked = self.current_epoch - stake.start_epoch;
        let duration_bonus = if epochs_staked > 30 {
            1.3
        } else if epochs_staked > 14 {
            1.1
        } else {
            1.0
        };
        
        // Calculate reward
        let reward = (stake.amount as f64 * base_rate * participation_multiplier * duration_bonus) as u64;
        
        // Apply minimum reward floor
        reward.max(1)
    }
    
    /// Calculate epoch-based mining rewards for all active stakes
    pub fn calculate_epoch_mining_rewards(&self) -> TotalEpochRewards {
        let mut total_rewards = 0u64;
        let mut rewards_by_token = HashMap::new();
        let mut rewards_by_tier = HashMap::new();
        
        for (address, stakes) in &self.stakes {
            for stake in stakes {
                if !self.is_stake_locked(stake) {
                    let reward = self.calculate_mining_reward(stake, 0.75); // Assume 75% participation
                    total_rewards += reward;
                    
                    *rewards_by_token.entry(stake.token_type.clone()).or_insert(0) += reward;
                    *rewards_by_tier.entry(address.clone()).or_insert(0) += reward;
                }
            }
        }
        
        TotalEpochRewards {
            total: total_rewards,
            by_token: rewards_by_token,
            by_address: rewards_by_tier,
            epoch: self.current_epoch,
        }
    }
    
    /// Distribute mining rewards to eligible stakers
    pub fn distribute_mining_rewards(&mut self, epoch: u64) -> Result<u64, &'static str> {
        let epoch_rewards = self.calculate_epoch_mining_rewards();
        
        for (address, stake_list) in self.stakes.iter_mut() {
            for stake in stake_list.iter_mut() {
                if !self.is_stake_locked(stake) {
                    let reward = self.calculate_mining_reward(stake, 0.75);
                    stake.rewards_claimed += reward;
                    self.total_rewards_distributed += reward;
                }
            }
        }
        
        Ok(epoch_rewards.total)
    }
    
    /// Calculate validator mining reward based on delegated stake and uptime
    pub fn calculate_validator_mining_reward(&self, validator: &str, epoch: u64) -> u64 {
        let metrics = match self.validator_metrics.get(validator) {
            Some(m) => m,
            None => return 0,
        };
        
        // Base reward per delegated token
        let base_rate = 0.0001; // 0.01% per epoch
        
        // Uptime multiplier (0.0 to 1.0)
        let uptime_multiplier = metrics.uptime_percent / 100.0;
        
        // Commission bonus (higher commission = higher reward)
        let commission_bonus = 1.0 + metrics.commission_rate;
        
        // Calculate reward
        let reward = (metrics.total_delegated as f64 * base_rate * uptime_multiplier * commission_bonus) as u64;
        
        reward.max(1)
    }
    
    /// Get mining reward projection for a stake amount
    pub fn project_mining_rewards(&self, pool_id: &str, amount: u64, epochs: u64) -> MiningRewardProjection {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return MiningRewardProjection::default(),
        };
        
        let daily_rate = pool.reward_rate / 365.0;
        let epoch_rate = daily_rate / 24.0; // Assuming 24 epochs per day
        
        let base_reward = (amount as f64 * epoch_rate * epochs as f64) as u64;
        let with_participation_bonus = (base_reward as f64 * 1.25) as u64; // 25% bonus at high participation
        let with_duration_bonus = (base_reward as f64 * 1.3) as u64; // 30% bonus for long stakes
        
        MiningRewardProjection {
            pool_id: pool_id.to_string(),
            stake_amount: amount,
            epochs: epochs,
            base_reward,
            with_participation_bonus,
            with_duration_bonus,
            apy: pool.reward_rate * 100.0,
        }
    }
    
    /// Calculate compound rewards (rewards reinvested)
    pub fn calculate_compound_mining_rewards(&self, stake: &StakeInfo, compounding_epochs: u64) -> u64 {
        let pool = match self.get_pool_by_token(&stake.token_type) {
            Some(p) => p,
            None => return 0,
        };
    
        let epoch_rate = pool.reward_rate / 365.0 / 24.0;
        let initial_amount = stake.amount;
        
        // Compound formula: A = P(1 + r)^n
        let compounded_amount = (initial_amount as f64 * (1.0 + epoch_rate).powf(compounding_epochs as f64)) as u64;
        
        compounded_amount - initial_amount // Return only the rewards portion
    }
    
    /// Get optimal stake amount for target daily reward
    pub fn calculate_stake_for_target_reward(&self, pool_id: &str, target_daily_reward: u64) -> u64 {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return 0,
        };
        
        let daily_rate = pool.reward_rate / 365.0;
        let required_stake = (target_daily_reward as f64 / daily_rate) as u64;
        
        // Ensure meets minimum stake
        required_stake.max(pool.min_stake)
    }
    
    /// Calculate mining reward efficiency (reward per token staked)
    pub fn calculate_reward_efficiency(&self, pool_id: &str) -> f64 {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return 0.0,
        };
        
        if pool.total_staked == 0 {
            return 0.0;
        }
        
        let total_rewards = self.total_rewards_distributed as f64;
        total_rewards / pool.total_staked as f64
    }
    
    /// Get mining reward schedule for a stake
    pub fn get_reward_schedule(&self, stake: &StakeInfo, epochs: u64) -> Vec<RewardScheduleEntry> {
        let mut schedule = Vec::new();
        
        for epoch in 0..epochs {
            let future_epoch = self.current_epoch + epoch;
            let reward = self.calculate_mining_reward(stake, 0.75);
            
            schedule.push(RewardScheduleEntry {
                epoch: future_epoch,
                projected_reward: reward,
                cumulative_reward: stake.rewards_claimed + (reward * epoch),
            });
        }
        
        schedule
    }
    
    // =============================================================================
    // ADVANCED MINING REWARD ENHANCEMENTS - Sprint 22 Backend
    // =============================================================================
    
    /// Calculate dynamic base reward rate based on network conditions
    pub fn calculate_dynamic_base_rate(&self, pool_id: &str, network_load: f64) -> f64 {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return 0.0,
        };
        
        // Network load adjustment: higher load = higher rewards to incentivize participation
        let load_multiplier = if network_load > 0.8 {
            1.5
        } else if network_load > 0.5 {
            1.2
        } else if network_load > 0.3 {
            1.0
        } else {
            0.8
        };
        
        pool.reward_rate * load_multiplier
    }
    
    /// Calculate tier-based mining reward (mobile, laptop, desktop, validator)
    pub fn calculate_tier_bonus(&self, device_tier: &str) -> f64 {
        match device_tier.to_lowercase().as_str() {
            "mobile" => 1.0,      // Base rate
            "laptop" => 1.25,     // 25% bonus
            "desktop" => 1.5,     // 50% bonus
            "validator" => 2.0,   // 100% bonus
            _ => 1.0,
        }
    }
    
    /// Calculate uptime-based reward multiplier
    pub fn calculate_uptime_multiplier(&self, uptime_percent: f64) -> f64 {
        if uptime_percent >= 99.0 {
            1.5  // Perfect uptime bonus
        } else if uptime_percent >= 95.0 {
            1.25 // Excellent uptime
        } else if uptime_percent >= 90.0 {
            1.0  // Standard rate
        } else if uptime_percent >= 80.0 {
            0.75 // Reduced rate
        } else {
            0.5  // Poor uptime penalty
        }
    }
    
    /// Calculate consistency bonus for consecutive epoch participation
    pub fn calculate_consistency_bonus(&self, consecutive_epochs: u64) -> f64 {
        if consecutive_epochs >= 30 {
            1.4  // Month of consistency
        } else if consecutive_epochs >= 14 {
            1.25 // Two weeks
        } else if consecutive_epochs >= 7 {
            1.1  // One week
        } else {
            1.0  // Base rate
        }
    }
    
    /// Calculate total mining reward with all bonuses applied
    pub fn calculate_total_mining_reward(
        &self,
        stake: &StakeInfo,
        network_participation: f64,
        device_tier: &str,
        uptime_percent: f64,
        consecutive_epochs: u64,
    ) -> u64 {
        let pool = match self.get_pool_by_token(&stake.token_type) {
            Some(p) => p,
            None => return 0,
        };
        
        // Base reward
        let base_rate = pool.reward_rate;
        
        // Apply all multipliers
        let participation_mult = if network_participation > 0.8 { 1.5 } else if network_participation > 0.5 { 1.0 } else { 0.5 };
        let tier_mult = self.calculate_tier_bonus(device_tier);
        let uptime_mult = self.calculate_uptime_multiplier(uptime_percent);
        let consistency_mult = self.calculate_consistency_bonus(consecutive_epochs);
        
        // Combined multiplier
        let total_mult = participation_mult * tier_mult * uptime_mult * consistency_mult;
        
        // Calculate final reward
        let reward = (stake.amount as f64 * base_rate * total_mult) as u64;
        
        reward.max(1)
    }
    
    /// Calculate epoch rewards for all miners with detailed breakdown
    pub fn calculate_detailed_epoch_rewards(&self) -> DetailedEpochRewards {
        let mut total_rewards = 0u64;
        let mut by_token = HashMap::new();
        let mut by_tier = HashMap::new();
        let mut by_uptime_range = HashMap::new();
        let mut top_earners = Vec::new();
        
        for (address, stakes) in &self.stakes {
            let mut address_total = 0u64;
            
            for stake in stakes {
                if !self.is_stake_locked(stake) {
                    let reward = self.calculate_total_mining_reward(
                        stake,
                        0.75,
                        "mobile",
                        95.0,
                        14,
                    );
                    total_rewards += reward;
                    address_total += reward;
                    
                    *by_token.entry(stake.token_type.clone()).or_insert(0) += reward;
                    *by_tier.entry("mobile".to_string()).or_insert(0) += reward;
                    
                    let uptime_range = if stake.amount > 10000 {
                        "high"
                    } else if stake.amount > 1000 {
                        "medium"
                    } else {
                        "low"
                    };
                    *by_uptime_range.entry(uptime_range.to_string()).or_insert(0) += reward;
                }
            }
            
            top_earners.push((address.clone(), address_total));
        }
        
        // Sort top earners
        top_earners.sort_by(|a, b| b.1.cmp(&a.1));
        top_earners.truncate(10);
        
        DetailedEpochRewards {
            total: total_rewards,
            by_token,
            by_tier,
            by_uptime_range,
            top_earners,
            epoch: self.current_epoch,
            total_miners: self.stakes.len(),
        }
    }
    
    /// Simulate reward distribution for testing/preview
    pub fn simulate_reward_distribution(&self, epochs: u64) -> RewardSimulation {
        let mut simulated_total = 0u64;
        let mut projected_rewards = HashMap::new();
        
        for (pool_id, pool) in &self.pools {
            let mut pool_total = 0u64;
            
            for (address, stakes) in &self.stakes {
                for stake in stakes {
                    if stake.token_type == pool.token_type && !self.is_stake_locked(stake) {
                        let reward = self.calculate_total_mining_reward(
                            stake,
                            0.75,
                            "mobile",
                            95.0,
                            14,
                        );
                        pool_total += reward * epochs;
                    }
                }
            }
            
            projected_rewards.insert(pool_id.clone(), pool_total);
            simulated_total += pool_total;
        }
        
        RewardSimulation {
            epochs,
            total_projected: simulated_total,
            by_pool: projected_rewards,
            average_apy: self.pools.values().map(|p| p.reward_rate).sum::<f64>() / self.pools.len() as f64,
        }
    }
    
    /// Get reward efficiency metrics for optimization
    pub fn get_reward_efficiency_metrics(&self, pool_id: &str) -> RewardEfficiencyMetrics {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return RewardEfficiencyMetrics::default(),
        };
        
        let total_rewards = self.total_rewards_distributed as f64;
        let total_staked = pool.total_staked as f64;
        
        RewardEfficiencyMetrics {
            reward_per_token: if total_staked > 0.0 { total_rewards / total_staked } else { 0.0 },
            reward_per_staker: if pool.active_stakers > 0 { total_rewards / pool.active_stakers as f64 } else { 0.0 },
            reward_rate: pool.reward_rate,
            effective_apy: pool.reward_rate * 100.0,
            total_distributed: self.total_rewards_distributed,
        }
    }
    
    // =============================================================================
    // SPRINT 22 ENHANCEMENT: Advanced Mining Reward Analytics & Optimization
    // =============================================================================
    
    /// Calculate network health score based on staking distribution
    pub fn calculate_network_health_score(&self) -> NetworkHealthScore {
        let total_staked = self.total_staked_all_pools();
        let pool_count = self.pools.len() as u64;
        let total_stakers = self.pools.values().map(|p| p.active_stakers).sum::<u64>();
        let avg_uptime = self.validator_metrics.values()
            .map(|m| m.uptime_percent)
            .sum::<f64>() / self.validator_metrics.len().max(1) as f64;
        
        // Decentralization score (0-100): based on staker distribution
        let decentralization = if total_stakers > 1000 { 100.0 }
            else if total_stakers > 500 { 80.0 }
            else if total_stakers > 100 { 60.0 }
            else { 40.0 };
        
        // Uptime score (0-100)
        let uptime_score = avg_uptime;
        
        // Diversity score (0-100): based on pool distribution
        let diversity_score = if pool_count >= 3 { 100.0 }
            else if pool_count == 2 { 70.0 }
            else { 40.0 };
        
        // Overall health score
        let overall = (decentralization + uptime_score + diversity_score) / 3.0;
        
        NetworkHealthScore {
            overall_score: overall,
            decentralization_score: decentralization,
            uptime_score,
            diversity_score,
            total_staked,
            total_stakers,
            status: if overall >= 80.0 { "Excellent" }
                   else if overall >= 60.0 { "Good" }
                   else if overall >= 40.0 { "Fair" }
                   else { "Needs Improvement" }.to_string(),
        }
    }
    
    /// Calculate reward acceleration factor for early adopters
    pub fn calculate_early_adopter_bonus(&self, stake: &StakeInfo) -> f64 {
        let epochs_since_start = self.current_epoch - stake.start_epoch;
        
        // Early adopter tiers
        if epochs_since_start <= 30 {
            1.5  // First 30 epochs: 50% bonus
        } else if epochs_since_start <= 90 {
            1.25 // First 90 epochs: 25% bonus
        } else if epochs_since_start <= 180 {
            1.1  // First 180 epochs: 10% bonus
        } else {
            1.0  // Standard rate
        }
    }
    
    /// Calculate community participation bonus (governance voting, referrals, etc.)
    pub fn calculate_community_bonus(&self, address: &str, participation_score: f64) -> f64 {
        if participation_score >= 90.0 {
            1.3  // Highly active community member
        } else if participation_score >= 70.0 {
            1.15 // Active participant
        } else if participation_score >= 50.0 {
            1.05 // Occasional participant
        } else {
            1.0  // Base rate
        }
    }
    
    /// Calculate loyalty multiplier for long-term stakers
    pub fn calculate_loyalty_multiplier(&self, stake: &StakeInfo) -> f64 {
        let epochs_staked = self.current_epoch - stake.start_epoch;
        
        if epochs_staked >= 365 {
            1.5  // Year+ staker: 50% bonus
        } else if epochs_staked >= 180 {
            1.3  // 6+ months: 30% bonus
        } else if epochs_staked >= 90 {
            1.2  // 3+ months: 20% bonus
        } else if epochs_staked >= 30 {
            1.1  // 1+ month: 10% bonus
        } else {
            1.0  // Base rate
        }
    }
    
    /// Calculate risk-adjusted reward rate (higher stakes = slightly lower rate for sustainability)
    pub fn calculate_risk_adjusted_rate(&self, pool_id: &str, stake_amount: u64) -> f64 {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return 0.0,
        };
        
        // Progressive rate adjustment for sustainability
        if stake_amount > 100000 {
            pool.reward_rate * 0.9  // -10% for very large stakes
        } else if stake_amount > 50000 {
            pool.reward_rate * 0.95 // -5% for large stakes
        } else {
            pool.reward_rate // Standard rate
        }
    }
    
    /// Calculate comprehensive mining reward with ALL bonuses
    pub fn calculate_comprehensive_mining_reward(
        &self,
        stake: &StakeInfo,
        network_participation: f64,
        device_tier: &str,
        uptime_percent: f64,
        consecutive_epochs: u64,
        community_score: f64,
    ) -> u64 {
        let pool = match self.get_pool_by_token(&stake.token_type) {
            Some(p) => p,
            None => return 0,
        };
        
        // Base reward
        let base_rate = self.calculate_risk_adjusted_rate(
            match stake.token_type {
                TokenType::AETH => "aeth_staking",
                TokenType::FLUX => "flux_staking",
                TokenType::ATH => "ath_staking",
            },
            stake.amount,
        );
        
        // All multipliers
        let participation_mult = if network_participation > 0.8 { 1.5 } else if network_participation > 0.5 { 1.0 } else { 0.5 };
        let tier_mult = self.calculate_tier_bonus(device_tier);
        let uptime_mult = self.calculate_uptime_multiplier(uptime_percent);
        let consistency_mult = self.calculate_consistency_bonus(consecutive_epochs);
        let early_adopter_mult = self.calculate_early_adopter_bonus(stake);
        let community_mult = self.calculate_community_bonus(&stake.address, community_score);
        let loyalty_mult = self.calculate_loyalty_multiplier(stake);
        
        // Combined multiplier (capped at 3.0x to prevent exploitation)
        let total_mult = (participation_mult * tier_mult * uptime_mult * consistency_mult * 
                         early_adopter_mult * community_mult * loyalty_mult).min(3.0);
        
        // Calculate final reward
        let reward = (stake.amount as f64 * base_rate * total_mult) as u64;
        
        reward.max(1)
    }
    
    /// Get detailed reward breakdown for transparency
    pub fn get_reward_breakdown(
        &self,
        stake: &StakeInfo,
        network_participation: f64,
        device_tier: &str,
        uptime_percent: f64,
        consecutive_epochs: u64,
        community_score: f64,
    ) -> RewardBreakdown {
        let pool = match self.get_pool_by_token(&stake.token_type) {
            Some(p) => p,
            None => return RewardBreakdown::default(),
        };
        
        let base_rate = self.calculate_risk_adjusted_rate(
            match stake.token_type {
                TokenType::AETH => "aeth_staking",
                TokenType::FLUX => "flux_staking",
                TokenType::ATH => "ath_staking",
            },
            stake.amount,
        );
        
        let base_reward = (stake.amount as f64 * base_rate) as u64;
        
        let participation_mult = if network_participation > 0.8 { 1.5 } else if network_participation > 0.5 { 1.0 } else { 0.5 };
        let tier_mult = self.calculate_tier_bonus(device_tier);
        let uptime_mult = self.calculate_uptime_multiplier(uptime_percent);
        let consistency_mult = self.calculate_consistency_bonus(consecutive_epochs);
        let early_adopter_mult = self.calculate_early_adopter_bonus(stake);
        let community_mult = self.calculate_community_bonus(&stake.address, community_score);
        let loyalty_mult = self.calculate_loyalty_multiplier(stake);
        
        let total_mult = (participation_mult * tier_mult * uptime_mult * consistency_mult * 
                         early_adopter_mult * community_mult * loyalty_mult).min(3.0);
        
        let final_reward = (base_reward as f64 * total_mult) as u64;
        
        RewardBreakdown {
            base_reward,
            participation_bonus: (base_reward as f64 * (participation_mult - 1.0)) as u64,
            tier_bonus: (base_reward as f64 * (tier_mult - 1.0)) as u64,
            uptime_bonus: (base_reward as f64 * (uptime_mult - 1.0)) as u64,
            consistency_bonus: (base_reward as f64 * (consistency_mult - 1.0)) as u64,
            early_adopter_bonus: (base_reward as f64 * (early_adopter_mult - 1.0)) as u64,
            community_bonus: (base_reward as f64 * (community_mult - 1.0)) as u64,
            loyalty_bonus: (base_reward as f64 * (loyalty_mult - 1.0)) as u64,
            total_multiplier: total_mult,
            final_reward,
        }
    }
    
    /// Calculate projected APY with all bonuses for a given stake profile
    pub fn calculate_projected_apy_with_bonuses(
        &self,
        pool_id: &str,
        stake_amount: u64,
        device_tier: &str,
        uptime_percent: f64,
        consecutive_epochs: u64,
        community_score: f64,
        epochs: u64,
    ) -> ProjectedAPY {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return ProjectedAPY::default(),
        };
        
        let stake = StakeInfo {
            address: "projection".to_string(),
            token_type: pool.token_type.clone(),
            amount: stake_amount,
            start_epoch: self.current_epoch,
            last_claim_epoch: self.current_epoch,
            rewards_claimed: 0,
            is_locked: false,
            lock_end_epoch: 0,
        };
        
        let epoch_reward = self.calculate_comprehensive_mining_reward(
            &stake,
            0.75,
            device_tier,
            uptime_percent,
            consecutive_epochs,
            community_score,
        );
        
        let annual_reward = epoch_reward * epochs;
        let effective_apy = (annual_reward as f64 / stake_amount as f64) * 100.0;
        
        let base_apy = pool.reward_rate * 100.0;
        let bonus_apy = effective_apy - base_apy;
        
        ProjectedAPY {
            base_apy,
            bonus_apy,
            total_effective_apy: effective_apy,
            projected_annual_reward: annual_reward,
            breakdown: vec![
                ("Base APY".to_string(), base_apy),
                ("Network Participation".to_string(), if 0.75 > 0.5 { (base_apy * 0.5) } else { 0.0 }),
                ("Device Tier".to_string(), (base_apy * (self.calculate_tier_bonus(device_tier) - 1.0))),
                ("Uptime".to_string(), (base_apy * (self.calculate_uptime_multiplier(uptime_percent) - 1.0))),
                ("Consistency".to_string(), (base_apy * (self.calculate_consistency_bonus(consecutive_epochs) - 1.0))),
                ("Community".to_string(), (base_apy * (self.calculate_community_bonus("projection", community_score) - 1.0))),
            ],
        }
    }
    
    /// Get optimal mining configuration for maximum rewards
    pub fn get_optimal_mining_config(&self, stake_amount: u64, pool_id: &str) -> OptimalMiningConfig {
        let pool = match self.pools.get(pool_id) {
            Some(p) => p,
            None => return OptimalMiningConfig::default(),
        };
        
        // Optimal device tier recommendation
        let recommended_tier = if stake_amount > 50000 { "desktop" }
            else if stake_amount > 10000 { "laptop" }
            else { "mobile" };
        
        // Target uptime for optimal rewards
        let target_uptime = 99.0; // Aim for perfect uptime
        
        // Recommended consecutive epochs for consistency bonus
        let target_consecutive = 30; // 1 month for good bonus
        
        // Expected reward multiplier
        let expected_mult = self.calculate_tier_bonus(recommended_tier) * 
                           self.calculate_uptime_multiplier(target_uptime) *
                           self.calculate_consistency_bonus(target_consecutive);
        
        OptimalMiningConfig {
            recommended_tier: recommended_tier.to_string(),
            target_uptime,
            target_consecutive_epochs: target_consecutive,
            expected_multiplier: expected_mult,
            projected_apy_increase: (expected_mult - 1.0) * pool.reward_rate * 100.0,
        }
    }
}
    
    /// Calculate optimal staking strategy for maximum rewards
    pub fn calculate_optimal_staking_strategy(&self, budget: u64, goal_epochs: u64) -> StakingStrategy {
        let mut strategies = Vec::new();
        
        for (pool_id, pool) in &self.pools {
            let stake_amount = budget.min(pool.total_staked.max(pool.min_stake));
            let projected_reward = self.project_mining_rewards(pool_id, stake_amount, goal_epochs);
            
            strategies.push(StakingStrategyOption {
                pool_id: pool_id.clone(),
                token_type: pool.token_type.clone(),
                recommended_stake: stake_amount,
                projected_reward: projected_reward.base_reward,
                apy: pool.reward_rate * 100.0,
                lockup_epochs: pool.lockup_epochs,
            });
        }
        
        // Sort by projected reward
        strategies.sort_by(|a, b| b.projected_reward.cmp(&a.projected_reward));
        
        StakingStrategy {
            budget,
            goal_epochs,
            options: strategies,
            recommended: strategies.first().cloned(),
        }
    }
    
    /// Get personalized reward projection for a user
    pub fn get_personalized_projection(
        &self,
        address: &str,
        device_tier: &str,
        target_epochs: u64,
    ) -> PersonalizedProjection {
        let stakes = self.get_address_stakes(address);
        let mut total_staked = 0u64;
        let mut tokens_by_type = HashMap::new();
        
        for stake in stakes {
            total_staked += stake.amount;
            *tokens_by_type.entry(stake.token_type.clone()).or_insert(0) += stake.amount;
        }
        
        let mut projections = Vec::new();
        
        for (token_type, amount) in &tokens_by_type {
            let pool = self.get_pool_by_token(token_type).unwrap();
            let pool_id = match token_type {
                TokenType::AETH => "aeth_staking",
                TokenType::FLUX => "flux_staking",
                TokenType::ATH => "ath_staking",
            };
            
            let base_reward = self.project_mining_rewards(pool_id, *amount, target_epochs);
            let with_bonuses = self.calculate_total_mining_reward(
                &StakeInfo {
                    address: address.to_string(),
                    token_type: token_type.clone(),
                    amount: *amount,
                    start_epoch: self.current_epoch,
                    last_claim_epoch: self.current_epoch,
                    rewards_claimed: 0,
                    is_locked: false,
                    lock_end_epoch: 0,
                },
                0.75,
                device_tier,
                95.0,
                14,
            ) * target_epochs;
            
            projections.push(TokenProjection {
                token_type: token_type.clone(),
                staked_amount: *amount,
                base_reward: base_reward.base_reward,
                with_bonuses,
                bonus_multiplier: if base_reward.base_reward > 0 { with_bonuses as f64 / base_reward.base_reward as f64 } else { 1.0 },
            });
        }
        
        PersonalizedProjection {
            address: address.to_string(),
            device_tier: device_tier.to_string(),
            total_staked,
            target_epochs,
            projections,
            estimated_total: projections.iter().map(|p| p.with_bonuses).sum(),
        }
    }
    
    // =============================================================================
    // SPRINT 3 BACKEND: Enhanced Mining Reward Calculation with Network Load
    // =============================================================================
    
    /// Calculate mining reward with dynamic network load adjustment
    pub fn calculate_network_adjusted_mining_reward(
        &self,
        stake: &StakeInfo,
        network_load: f64,
        device_tier: &str,
        validator_count: u64,
    ) -> u64 {
        let pool = match self.get_pool_by_token(&stake.token_type) {
            Some(p) => p,
            None => return 0,
        };
        
        // Dynamic base rate based on network load
        let dynamic_rate = self.calculate_dynamic_base_rate(
            match stake.token_type {
                TokenType::AETH => "aeth_staking",
                TokenType::FLUX => "flux_staking",
                TokenType::ATH => "ath_staking",
            },
            network_load,
        );
        
        // Device tier bonus
        let tier_mult = self.calculate_tier_bonus(device_tier);
        
        // Validator distribution bonus (more validators = better decentralization = bonus)
        let validator_mult = if validator_count > 100 {
            1.3
        } else if validator_count > 50 {
            1.15
        } else if validator_count > 20 {
            1.0
        } else {
            0.85
        };
        
        // Calculate reward
        let reward = (stake.amount as f64 * dynamic_rate * tier_mult * validator_mult) as u64;
        
        reward.max(1)
    }
    
    /// Get mining reward analytics dashboard data
    pub fn get_mining_analytics_dashboard(&self) -> MiningAnalyticsDashboard {
        let mut total_rewards_by_tier = HashMap::new();
        let mut total_rewards_by_token = HashMap::new();
        let mut avg_reward_per_staker = HashMap::new();
        let mut top_performers = Vec::new();
        
        for (address, stakes) in &self.stakes {
            let mut addr_total = 0u64;
            let mut addr_tier = "mobile";
            let mut addr_token = TokenType::AETH;
            
            for stake in stakes {
                if !self.is_stake_locked(stake) {
                    let reward = self.calculate_network_adjusted_mining_reward(
                        stake,
                        0.75,
                        "mobile",
                        self.validator_metrics.len() as u64,
                    );
                    addr_total += reward;
                    addr_token = stake.token_type.clone();
                }
            }
            
            *total_rewards_by_tier.entry(addr_tier.to_string()).or_insert(0) += addr_total;
            *total_rewards_by_token.entry(addr_token).or_insert(0) += addr_total;
            avg_reward_per_staker.insert(address.clone(), addr_total);
            
            top_performers.push((address.clone(), addr_total));
        }
        
        top_performers.sort_by(|a, b| b.1.cmp(&a.1));
        top_performers.truncate(10);
        
        MiningAnalyticsDashboard {
            total_rewards_by_tier,
            total_rewards_by_token,
            top_performers,
            avg_reward_per_staker: avg_reward_per_staker.values().sum::<u64>() / avg_reward_per_staker.len().max(1) as u64,
            total_miners: self.stakes.len(),
            total_validators: self.validator_metrics.len(),
            current_epoch: self.current_epoch,
        }
    }
    
    /// Calculate epoch mining rewards with network load factor
    pub fn calculate_epoch_rewards_with_network_load(&self, network_load: f64) -> EpochRewardsWithLoad {
        let mut total_rewards = 0u64;
        let mut by_token = HashMap::new();
        let mut by_tier = HashMap::new();
        let mut load_adjusted_total = 0u64;
        
        for (address, stakes) in &self.stakes {
            for stake in stakes {
                if !self.is_stake_locked(stake) {
                    let base_reward = self.calculate_mining_reward(stake, 0.75);
                    let load_adjusted = self.calculate_network_adjusted_mining_reward(
                        stake,
                        network_load,
                        "mobile",
                        self.validator_metrics.len() as u64,
                    );
                    
                    total_rewards += base_reward;
                    load_adjusted_total += load_adjusted;
                    *by_token.entry(stake.token_type.clone()).or_insert(0) += load_adjusted;
                    *by_tier.entry("mobile".to_string()).or_insert(0) += load_adjusted;
                }
            }
        }
        
        let load_multiplier = if network_load > 0.8 {
            "High (1.5x)"
        } else if network_load > 0.5 {
            "Medium (1.2x)"
        } else if network_load > 0.3 {
            "Normal (1.0x)"
        } else {
            "Low (0.8x)"
        };
        
        EpochRewardsWithLoad {
            base_total: total_rewards,
            load_adjusted_total,
            network_load,
            load_multiplier: load_multiplier.to_string(),
            by_token,
            by_tier,
            epoch: self.current_epoch,
        }
    }
}

/// Mining analytics dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningAnalyticsDashboard {
    pub total_rewards_by_tier: HashMap<String, u64>,
    pub total_rewards_by_token: HashMap<TokenType, u64>,
    pub top_performers: Vec<(String, u64)>,
    pub avg_reward_per_staker: u64,
    pub total_miners: usize,
    pub total_validators: usize,
    pub current_epoch: u64,
}

/// Epoch rewards with network load factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochRewardsWithLoad {
    pub base_total: u64,
    pub load_adjusted_total: u64,
    pub network_load: f64,
    pub load_multiplier: String,
    pub by_token: HashMap<TokenType, u64>,
    pub by_tier: HashMap<String, u64>,
    pub epoch: u64,
}

/// Total epoch rewards summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalEpochRewards {
    pub total: u64,
    pub by_token: HashMap<TokenType, u64>,
    pub by_address: HashMap<String, u64>,
    pub epoch: u64,
}

/// Mining reward projection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MiningRewardProjection {
    pub pool_id: String,
    pub stake_amount: u64,
    pub epochs: u64,
    pub base_reward: u64,
    pub with_participation_bonus: u64,
    pub with_duration_bonus: u64,
    pub apy: f64,
}

/// Reward schedule entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardScheduleEntry {
    pub epoch: u64,
    pub projected_reward: u64,
    pub cumulative_reward: u64,
}

/// Pool statistics for API responses
pub struct PoolStats {
    pub name: String,
    pub total_staked: u64,
    pub reward_rate: f64,
    pub apy: f64,
    pub min_stake: u64,
    pub lockup_epochs: u64,
    pub active_stakers: u64,
}

/// Network-wide staking metrics
pub struct NetworkStakingMetrics {
    pub total_staked: u64,
    pub total_delegated: u64,
    pub total_validators: u64,
    pub total_delegators: u64,
    pub average_commission: f64,
    pub average_uptime: f64,
    pub total_rewards_distributed: u64,
}

/// Detailed epoch rewards with breakdowns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedEpochRewards {
    pub total: u64,
    pub by_token: HashMap<TokenType, u64>,
    pub by_tier: HashMap<String, u64>,
    pub by_uptime_range: HashMap<String, u64>,
    pub top_earners: Vec<(String, u64)>,
    pub epoch: u64,
    pub total_miners: usize,
}

/// Reward simulation for testing/preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardSimulation {
    pub epochs: u64,
    pub total_projected: u64,
    pub by_pool: HashMap<String, u64>,
    pub average_apy: f64,
}

/// Pool distribution across token types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolDistribution {
    pub aeth: (u64, u64),      // (total_staked, active_stakers)
    pub flux: (u64, u64),
    pub ath: (u64, u64),
    pub total_staked: u64,
    pub total_stakers: u64,
}

/// Compound growth projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundProjection {
    pub final_amount: u64,
    pub total_rewards: u64,
    pub effective_apy: f64,
    pub growth_multiple: f64,
}

/// Pool recommendation based on risk profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolRecommendation {
    pub pool_id: String,
    pub reason: String,
    pub expected_apy: f64,
    pub lockup_epochs: u64,
    pub min_stake: u64,
}

/// Staking efficiency score components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingEfficiency {
    pub score: u64,
    pub apy_component: u64,
    pub liquidity_component: u64,
    pub participation_component: u64,
    pub status: String,
}

/// Reward efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RewardEfficiencyMetrics {
    pub reward_per_token: f64,
    pub reward_per_staker: f64,
    pub reward_rate: f64,
    pub effective_apy: f64,
    pub total_distributed: u64,
}

/// Staking strategy option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStrategyOption {
    pub pool_id: String,
    pub token_type: TokenType,
    pub recommended_stake: u64,
    pub projected_reward: u64,
    pub apy: f64,
    pub lockup_epochs: u64,
}

/// Optimal staking strategy recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStrategy {
    pub budget: u64,
    pub goal_epochs: u64,
    pub options: Vec<StakingStrategyOption>,
    pub recommended: Option<StakingStrategyOption>,
}

/// Token-specific projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenProjection {
    pub token_type: TokenType,
    pub staked_amount: u64,
    pub base_reward: u64,
    pub with_bonuses: u64,
    pub bonus_multiplier: f64,
}

/// Personalized reward projection for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedProjection {
    pub address: String,
    pub device_tier: String,
    pub total_staked: u64,
    pub target_epochs: u64,
    pub projections: Vec<TokenProjection>,
    pub estimated_total: u64,
}

/// Network health score metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthScore {
    pub overall_score: f64,
    pub decentralization_score: f64,
    pub uptime_score: f64,
    pub diversity_score: f64,
    pub total_staked: u64,
    pub total_stakers: u64,
    pub status: String,
}

/// Detailed reward breakdown for transparency
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RewardBreakdown {
    pub base_reward: u64,
    pub participation_bonus: u64,
    pub tier_bonus: u64,
    pub uptime_bonus: u64,
    pub consistency_bonus: u64,
    pub early_adopter_bonus: u64,
    pub community_bonus: u64,
    pub loyalty_bonus: u64,
    pub total_multiplier: f64,
    pub final_reward: u64,
}

/// Projected APY with all bonuses
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectedAPY {
    pub base_apy: f64,
    pub bonus_apy: f64,
    pub total_effective_apy: f64,
    pub projected_annual_reward: u64,
    pub breakdown: Vec<(String, f64)>,
}

/// Optimal mining configuration recommendation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimalMiningConfig {
    pub recommended_tier: String,
    pub target_uptime: f64,
    pub target_consecutive_epochs: u64,
    pub expected_multiplier: f64,
    pub projected_apy_increase: f64,
}

// =============================================================================
// SPRINT 10: Cross-Chain Staking & Liquid Staking Derivatives
// =============================================================================

/// Cross-chain staking position for bridged assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainStake {
    pub stake_id: String,
    pub owner: String,
    pub source_chain: String,      // e.g., "ethereum", "bsc", "polygon"
    pub destination_chain: String,  // e.g., "aether"
    pub amount: u64,
    pub wrapped_amount: u64,       // Amount of wrapped tokens received
    pub start_epoch: u64,
    pub lock_end_epoch: u64,
    pub bridge_fee_paid: u64,
    pub status: CrossChainStakeStatus,
}

/// Cross-chain stake status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CrossChainStakeStatus {
    PendingBridge,
    Bridging,
    Active,
    Unbonding,
    Completed,
    Failed,
}

/// Liquid staking derivative token (stAETH, stFLUX, stATH)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidStakingToken {
    pub token_symbol: String,      // e.g., "stAETH", "stFLUX"
    pub underlying_token: TokenType,
    pub total_supply: u64,
    pub total_staked: u64,
    pub exchange_rate: f64,         // 1 stToken = X underlying (increases with rewards)
    pub initial_exchange_rate: f64,
    pub last_reward_epoch: u64,
    pub accumulated_rewards: u64,
    pub fee_rate: f64,              // Protocol fee on rewards (e.g., 0.1 = 10%)
}

impl LiquidStakingToken {
    pub fn new(symbol: &str, token: TokenType, initial_rate: f64) -> Self {
        Self {
            token_symbol: symbol.to_string(),
            underlying_token: token,
            total_supply: 0,
            total_staked: 0,
            exchange_rate: initial_rate,
            initial_exchange_rate: initial_rate,
            last_reward_epoch: 0,
            accumulated_rewards: 0,
            fee_rate: 0.1,
        }
    }
    
    /// Calculate amount of underlying tokens received for staking
    pub fn stake(&mut self, amount: u64) -> u64 {
        let mint_amount = (amount as f64 / self.exchange_rate) as u64;
        self.total_supply += mint_amount;
        self.total_staked += amount;
        mint_amount
    }
    
    /// Calculate amount of underlying tokens received for unstaking
    pub fn unstake(&mut self, stoken_amount: u64) -> Result<u64, &'static str> {
        if stoken_amount > self.total_supply {
            return Err("Insufficient liquid staking token supply");
        }
        
        let underlying_amount = (stoken_amount as f64 * self.exchange_rate) as u64;
        self.total_supply -= stoken_amount;
        self.total_staked = self.total_staked.saturating_sub(underlying_amount);
        Ok(underlying_amount)
    }
    
    /// Update exchange rate based on accumulated rewards
    pub fn update_exchange_rate(&mut self, new_rewards: u64) {
        self.accumulated_rewards += new_rewards;
        let fee = (new_rewards as f64 * self.fee_rate) as u64;
        let net_rewards = new_rewards - fee;
        
        if self.total_staked > 0 {
            let new_rate = (self.total_staked + net_rewards) as f64 / self.total_supply.max(1) as f64;
            self.exchange_rate = new_rate.max(self.initial_exchange_rate);
        }
    }
    
    /// Get APY for liquid staking token
    pub fn get_apy(&self, epochs_per_year: u64) -> f64 {
        if self.total_staked == 0 || self.last_reward_epoch == 0 {
            return 0.0;
        }
        
        let reward_rate = self.accumulated_rewards as f64 / self.total_staked as f64;
        reward_rate * epochs_per_year as f64 * 100.0
    }
}

/// Liquid staking position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidStakingPosition {
    pub owner: String,
    pub stoken_symbol: String,
    pub stoken_amount: u64,
    pub underlying_value: u64,
    pub staked_epoch: u64,
    pub claimable_rewards: u64,
}

/// Delegation voucher for transferable staking position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationVoucher {
    pub voucher_id: String,
    pub owner: String,
    pub validator: String,
    pub staked_amount: u64,
    pub maturity_epoch: u64,
    pub transferable: bool,
    pub current_owner: String,
    pub transfer_history: Vec<(String, String, u64)>, // (from, to, epoch)
}

/// Staking bond for institutional stakers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingBond {
    pub bond_id: String,
    pub issuer: String,
    pub principal: u64,
    pub term_epochs: u64,
    pub coupon_rate: f64,          // Periodic reward rate
    pub start_epoch: u64,
    pub maturity_epoch: u64,
    pub coupons_paid: u64,
    pub is_redeemable: bool,
    pub collateral_token: TokenType,
}

impl StakingBond {
    pub fn new(issuer: &str, principal: u64, term_epochs: u64, coupon_rate: f64, collateral: TokenType) -> Self {
        Self {
            bond_id: format!("bond_{}_{}", issuer, principal),
            issuer: issuer.to_string(),
            principal,
            term_epochs,
            coupon_rate,
            start_epoch: 0,
            maturity_epoch: 0,
            coupons_paid: 0,
            is_redeemable: false,
            collateral_token: collateral,
        }
    }
    
    pub fn activate(&mut self, current_epoch: u64) {
        self.start_epoch = current_epoch;
        self.maturity_epoch = current_epoch + self.term_epochs;
        self.is_redeemable = false;
    }
    
    pub fn pay_coupon(&mut self, amount: u64) {
        self.coupons_paid += amount;
    }
    
    pub fn redeem(&mut self) -> Result<u64, &'static str> {
        if !self.is_redeemable {
            return Err("Bond not yet redeemable");
        }
        Ok(self.principal)
    }
}

/// Staking derivatives pool for trading staked positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingDerivativesPool {
    pub pool_id: String,
    pub name: String,
    pub underlying_token: TokenType,
    pub total_pool_value: u64,
    pub share_count: u64,
    pub positions: Vec<DerivativePosition>,
    pub fee_rate: f64,
    pub performance_fee: f64,
}

/// Derivative position in staking pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivativePosition {
    pub owner: String,
    pub shares: u64,
    pub entry_value: u64,
    pub entry_epoch: u64,
    pub claimable_rewards: u64,
}

/// Staking options contract (call/put on staking rewards)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingOption {
    pub option_id: String,
    pub holder: String,
    pub option_type: OptionType,
    pub underlying_pool: String,
    pub strike_reward_rate: f64,
    pub premium_paid: u64,
    pub expiration_epoch: u64,
    pub exercised: bool,
    pub notional_value: u64,
}

/// Option type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptionType {
    Call,   // Right to receive higher rewards
    Put,    // Right to protect against lower rewards
}

/// Staking insurance policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingInsurance {
    pub policy_id: String,
    pub insured: String,
    pub covered_amount: u64,
    pub coverage_type: InsuranceCoverageType,
    pub premium_per_epoch: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub claims_made: u64,
    pub is_active: bool,
}

/// Insurance coverage types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InsuranceCoverageType {
    SlashingProtection,      // Cover validator slashing losses
    SmartContractRisk,       // Cover contract exploit losses
    StableYield,             // Guarantee minimum APY
    PrincipalProtection,     // Protect principal amount
}

/// Auto-compound configuration for staking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundConfig {
    pub enabled: bool,
    pub threshold_amount: u64,      // Auto-compound when rewards reach this amount
    pub compound_frequency_epochs: u64,
    pub max_compound_per_epoch: u64,
    pub gas_optimization: bool,      // Batch compounds to save gas
}

impl Default for AutoCompoundConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_amount: 100,
            compound_frequency_epochs: 1,
            max_compound_per_epoch: 10,
            gas_optimization: true,
        }
    }
}

/// Auto-compound performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoCompoundMetrics {
    pub total_compounds: u64,
    pub total_reinvested: u64,
    pub compound_interest_earned: u64,
    pub avg_compound_interval_epochs: f64,
    pub effective_apy_boost: f64,     // APY increase from compounding
    pub gas_saved: u64,
}

/// Staking reward optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingOptimization {
    pub current_strategy: String,
    pub suggested_strategy: String,
    pub current_projected_reward: u64,
    pub optimized_projected_reward: u64,
    pub improvement_percent: f64,
    pub action_required: Vec<String>,
    pub risk_level: String,  // "low", "medium", "high"
}

/// Compound event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundEvent {
    pub epoch: u64,
    pub amount_compounded: u64,
    pub new_principal: u64,
    pub rewards_added: u64,
    pub gas_cost: u64,
    pub effective_rate: f64,
}

/// Staking tier benefits comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierComparison {
    pub tier: String,
    pub min_stake: u64,
    pub base_apy: f64,
    pub bonus_apy: f64,
    pub total_apy: f64,
    pub lockup_epochs: u64,
    pub early_withdrawal_penalty: f64,
    pub special_benefits: Vec<String>,
}

/// Stake distribution analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeDistribution {
    pub total_staked: u64,
    pub staker_count: u64,
    pub avg_stake_per_user: f64,
    pub median_stake: u64,
    pub top_10_percent_stake: u64,
    pub gini_coefficient: f64,  // Measure of stake concentration (0 = equal, 1 = unequal)
    pub distribution_health: String,  // "healthy", "concentrated", "decentralized"
}

/// Validator recommendation for delegators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorRecommendation {
    pub validator_address: String,
    pub commission_rate: f64,
    pub uptime_percent: f64,
    pub total_delegated: u64,
    pub delegator_count: u64,
    pub historical_performance: f64,  // 0-100 score
    pub risk_score: f64,              // 0-100, lower is better
    pub recommended_stake: u64,
    pub reason: String,
}

/// Staking goal tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingGoal {
    pub goal_name: String,
    pub target_amount: u64,
    pub current_amount: u64,
    pub target_epoch: u64,
    pub current_epoch: u64,
    pub progress_percent: f64,
    pub on_track: bool,
    pub recommended_action: Option<String>,
}

/// APY comparison across pools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APYComparison {
    pub pool_name: String,
    pub base_apy: f64,
    pub with_auto_compound_apy: f64,
    pub with_bonuses_apy: f64,
    pub effective_apy: f64,
    pub risk_adjusted_apy: f64,
    pub rank: u32,
}

/// Stake withdrawal planner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalPlan {
    pub total_staked: u64,
    pub requested_withdrawal: u64,
    pub lock_end_epoch: u64,
    pub current_epoch: u64,
    pub epochs_remaining: u64,
    pub early_withdrawal_penalty: u64,
    pub net_received: u64,
    pub recommended_wait_epochs: u64,
    pub potential_gain_from_waiting: u64,
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
