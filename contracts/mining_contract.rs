// Mining Contract - AeTHer Chain
// Enhanced proof-of-availability mining with dynamic difficulty adjustment

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Device tier for mining rewards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DeviceTier {
    Mobile = 1,
    Laptop = 2,
    Desktop = 3,
    Server = 4,
}

impl DeviceTier {
    pub fn multiplier(&self) -> f64 {
        match self {
            DeviceTier::Mobile => 1.0,
            DeviceTier::Laptop => 2.5,
            DeviceTier::Desktop => 4.0,
            DeviceTier::Server => 8.0,
        }
    }

    pub fn min_uptime_hours(&self) -> u64 {
        match self {
            DeviceTier::Mobile => 1,
            DeviceTier::Laptop => 2,
            DeviceTier::Desktop => 4,
            DeviceTier::Server => 6,
        }
    }
}

/// Miner status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MinerStatus {
    Active,
    Offline,
    Slashed,
    PendingActivation,
}

/// Miner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerInfo {
    pub address: String,
    pub device_tier: DeviceTier,
    pub total_mined: u64,
    pub last_claim_epoch: u64,
    pub consecutive_uptime_epochs: u64,
    pub reputation_score: f64,
    pub status: MinerStatus,
    pub registered_at: u64,
    pub last_active_epoch: u64,
    pub penalty_count: u64,
}

/// Network-wide mining statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMiningStats {
    pub total_active_miners: u64,
    pub total_miners_tier_mobile: u64,
    pub total_miners_tier_laptop: u64,
    pub total_miners_tier_desktop: u64,
    pub total_miners_tier_server: u64,
    pub epoch_rewards_distributed: u64,
    pub current_epoch_difficulty: u64,
    pub average_uptime_score: f64,
    pub network_hashrate_equivalent: u64,
}

/// Mining contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningContract {
    pub miners: HashMap<String, MinerInfo>,
    pub network_stats: NetworkMiningStats,
    pub base_reward_per_epoch: u64,
    pub current_epoch: u64,
    pub difficulty_adjustment_interval: u64,
    pub target_epoch_duration_secs: u64,
    pub minimum_rewards_pool: u64,
    pub emergency_difficulty: u64,
}

impl MiningContract {
    /// Create new mining contract
    pub fn new() -> Self {
        MiningContract {
            miners: HashMap::new(),
            network_stats: NetworkMiningStats {
                total_active_miners: 0,
                total_miners_tier_mobile: 0,
                total_miners_tier_laptop: 0,
                total_miners_tier_desktop: 0,
                total_miners_tier_server: 0,
                epoch_rewards_distributed: 0,
                current_epoch_difficulty: 1000,
                average_uptime_score: 0.0,
                network_hashrate_equivalent: 0,
            },
            base_reward_per_epoch: 1000,
            current_epoch: 0,
            difficulty_adjustment_interval: 100,
            target_epoch_duration_secs: 3600, // 1 hour
            minimum_rewards_pool: 100_000,
            emergency_difficulty: 500,
        }
    }

    /// Register a new miner
    pub fn register_miner(&mut self, address: String, device_tier: DeviceTier) -> Result<MinerInfo, String> {
        if self.miners.contains_key(&address) {
            return Err("Miner already registered".to_string());
        }

        let miner = MinerInfo {
            address: address.clone(),
            device_tier,
            total_mined: 0,
            last_claim_epoch: 0,
            consecutive_uptime_epochs: 0,
            reputation_score: 50.0, // Start with neutral reputation
            status: MinerStatus::PendingActivation,
            registered_at: self.current_epoch,
            last_active_epoch: 0,
            penalty_count: 0,
        };

        // Update tier counts
        match device_tier {
            DeviceTier::Mobile => self.network_stats.total_miners_tier_mobile += 1,
            DeviceTier::Laptop => self.network_stats.total_miners_tier_laptop += 1,
            DeviceTier::Desktop => self.network_stats.total_miners_tier_desktop += 1,
            DeviceTier::Server => self.network_stats.total_miners_tier_server += 1,
        }
        self.network_stats.total_active_miners += 1;

        self.miners.insert(address, miner.clone());
        Ok(miner)
    }

    /// Calculate uptime score for a miner (0.0 to 1.0)
    pub fn calculate_uptime_score(&self, miner: &MinerInfo) -> f64 {
        let tier = miner.device_tier;
        let min_uptime = tier.min_uptime_hours();
        let actual_uptime = self.get_actual_uptime(miner);
        
        if actual_uptime >= min_uptime {
            // Full uptime or better
            1.0
        } else if actual_uptime == 0 {
            // Complete downtime
            0.0
        } else {
            // Partial uptime
            actual_uptime as f64 / min_uptime as f64
        }
    }

    /// Get actual uptime hours (simplified - would be calculated from actual epoch data)
    fn get_actual_uptime(&self, miner: &MinerInfo) -> u64 {
        // In production, this would check actual epoch participation data
        // For now, simplified calculation based on consecutive epochs
        miner.consecutive_uptime_epochs.min(24)
    }

    /// Calculate mining reward with all factors
    pub fn calculate_reward(&self, miner: &MinerInfo) -> u64 {
        // Skip if miner is slashed
        if miner.status == MinerStatus::Slashed {
            return 0;
        }

        // Base reward
        let mut reward = self.base_reward_per_epoch as f64;

        // Tier multiplier
        reward *= miner.device_tier.multiplier();

        // Uptime score (0.0 to 1.0)
        let uptime_score = self.calculate_uptime_score(miner);
        reward *= uptime_score;

        // Reputation bonus (0.5x to 2.0x based on reputation 0-100)
        let reputation_factor = 0.5 + (miner.reputation_score / 100.0);
        reward *= reputation_factor;

        // Network difficulty factor
        let difficulty_factor = 1000.0 / self.network_stats.current_epoch_difficulty.max(1) as f64;
        reward *= difficulty_factor;

        // Apply floor to rewards
        reward.max(1.0) as u64
    }

    /// Record epoch participation for a miner
    pub fn record_participation(&mut self, address: &str, participated: bool) -> Result<(), String> {
        let miner = self.miners.get_mut(address)
            .ok_or("Miner not found")?;

        if participated {
            miner.consecutive_uptime_epochs += 1;
            miner.last_active_epoch = self.current_epoch;
            
            // Increase reputation for good participation
            miner.reputation_score = (miner.reputation_score + 0.1).min(100.0);
            
            if miner.status == MinerStatus::PendingActivation || miner.status == MinerStatus::Offline {
                miner.status = MinerStatus::Active;
            }
        } else {
            miner.consecutive_uptime_epochs = 0;
            miner.status = MinerStatus::Offline;
            
            // Decrease reputation for missed epochs
            miner.reputation_score = (miner.reputation_score - 1.0).max(0.0);
            
            // Track penalties
            if miner.reputation_score < 20.0 {
                miner.penalty_count += 1;
            }
            
            // Slash if too many penalties
            if miner.penalty_count >= 3 {
                miner.status = MinerStatus::Slashed;
            }
        }

        Ok(())
    }

    /// Adjust network difficulty based on participation
    pub fn adjust_difficulty(&mut self) {
        let participation_rate = if self.network_stats.total_active_miners > 0 {
            let active = self.miners.values()
                .filter(|m| m.status == MinerStatus::Active)
                .count() as f64;
            active / self.network_stats.total_active_miners as f64
        } else {
            0.0
        };

        // Increase difficulty if participation is high (rewards are too generous)
        // Decrease difficulty if participation is low (rewards too scarce)
        let current_difficulty = self.network_stats.current_epoch_difficulty;
        
        let new_difficulty = if participation_rate > 0.9 {
            // High participation - increase difficulty slightly
            (current_difficulty as f64 * 1.05).min(5000.0) as u64
        } else if participation_rate < 0.5 {
            // Low participation - decrease difficulty
            (current_difficulty as f64 * 0.9).max(self.emergency_difficulty) as u64
        } else {
            // Stable - gradual increase
            (current_difficulty as f64 * 1.01).min(5000.0) as u64
        };

        self.network_stats.current_epoch_difficulty = new_difficulty;
    }

    /// Claim mining rewards
    pub fn claim_rewards(&mut self, address: &str) -> Result<u64, String> {
        let miner = self.miners.get_mut(address)
            .ok_or("Miner not found")?;

        if miner.status == MinerStatus::Slashed {
            return Err("Miner has been slashed".to_string());
        }

        // Calculate unclaimed rewards
        let epochs_since_claim = self.current_epoch - miner.last_claim_epoch;
        let mut total_reward = 0u64;

        for _ in 0..epochs_since_claim {
            total_reward += self.calculate_reward(miner);
        }

        // Update miner state
        miner.total_mined += total_reward;
        miner.last_claim_epoch = self.current_epoch;
        self.network_stats.epoch_rewards_distributed += total_reward;

        Ok(total_reward)
    }

    // =============================================================================
    // REWARD CALCULATION HELPERS - Sprint Enhancement
    // =============================================================================
    
    /// Calculate daily mining rewards based on current tier and uptime
    pub fn calculate_daily_rewards(&self, miner: &MinerInfo) -> u64 {
        let epoch_reward = self.calculate_reward(miner);
        epoch_reward * 24 // Assuming 24 epochs per day
    }
    
    /// Calculate weekly mining rewards
    pub fn calculate_weekly_rewards(&self, miner: &MinerInfo) -> u64 {
        self.calculate_daily_rewards(miner) * 7
    }
    
    /// Calculate monthly mining rewards
    pub fn calculate_monthly_rewards(&self, miner: &MinerInfo) -> u64 {
        self.calculate_daily_rewards(miner) * 30
    }
    
    /// Get network hashrate equivalent (aggregate of all miners)
    pub fn get_network_hashrate(&self) -> u64 {
        let mut total_hashrate = 0u64;
        for miner in self.miners.values() {
            if miner.status == MinerStatus::Active {
                let tier_hashrate = match miner.device_tier {
                    DeviceTier::Mobile => 0.5,
                    DeviceTier::Laptop => 2.0,
                    DeviceTier::Desktop => 5.0,
                    DeviceTier::Server => 20.0,
                };
                total_hashrate += (tier_hashrate * 1000) as u64; // Convert to kh/s equivalent
            }
        }
        total_hashrate
    }
    
    /// Get active miner count by tier
    pub fn get_miners_by_tier(&self) -> TierCounts {
        TierCounts {
            mobile: self.network_stats.total_miners_tier_mobile,
            laptop: self.network_stats.total_miners_tier_laptop,
            desktop: self.network_stats.total_miners_tier_desktop,
            server: self.network_stats.total_miners_tier_server,
            total_active: self.network_stats.total_active_miners,
        }
    }
    
    /// Calculate mining profitability for a given tier (reward per能耗 unit)
    pub fn calculate_mining_profitability(&self, tier: DeviceTier, electricity_cost_per_kwh: f64) -> f64 {
        let base_reward = self.base_reward_per_epoch as f64;
        let tier_multiplier = tier.multiplier();
        
        // Estimated power consumption per tier (watts)
        let power_watts = match tier {
            DeviceTier::Mobile => 5.0,
            DeviceTier::Laptop => 50.0,
            DeviceTier::Desktop => 300.0,
            DeviceTier::Server => 500.0,
        };
        
        // Daily reward in USD equivalent (assuming token price)
        let daily_rewards = base_reward * tier_multiplier * 24 * 0.0001; // rough token value
        
        // Daily electricity cost
        let daily_power_cost = (power_watts / 1000.0) * 24.0 * electricity_cost_per_kwh;
        
        // Net profit (rewards - cost)
        daily_rewards - daily_power_cost
    }
    
    // =============================================================================
    // MINING POOL SYSTEM - NEW SPRINT ADDITION
    // =============================================================================
    
    /// Mining pool information for pooled mining rewards
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MiningPool {
        pub pool_id: String,
        pub name: String,
        pub owner: String,
        pub total_hashrate: u64,
        pub total_miners: u64,
        pub pool_fee_percent: f64,
        pub total_rewards_distributed: u64,
        pub created_at: u64,
        pub is_active: bool,
    }
    
    /// Pool miner participation record
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PoolMiner {
        pub miner_address: String,
        pub pool_id: String,
        pub joined_at: u64,
        pub hashrate_contributed: u64,
        pub rewards_earned: u64,
        pub last_claim_epoch: u64,
    }
    
    /// Add mining pools to contract state
    pub mining_pools: HashMap<String, MiningPool>,
    pub pool_miners: HashMap<String, Vec<PoolMiner>>,
    
    /// Create a new mining pool
    pub fn create_pool(&mut self, owner: String, name: String, pool_fee: f64) -> Result<MiningPool, String> {
        if pool_fee < 0.0 || pool_fee > 10.0 {
            return Err("Pool fee must be between 0% and 10%".to_string());
        }
        
        let pool_id = format!("pool_{}_{}", owner, self.current_epoch);
        let pool = MiningPool {
            pool_id: pool_id.clone(),
            name,
            owner: owner.clone(),
            total_hashrate: 0,
            total_miners: 0,
            pool_fee_percent: pool_fee,
            total_rewards_distributed: 0,
            created_at: self.current_epoch,
            is_active: true,
        };
        
        self.mining_pools.insert(pool_id.clone(), pool.clone());
        self.pool_miners.insert(pool_id, Vec::new());
        
        Ok(pool)
    }
    
    /// Join a mining pool
    pub fn join_pool(&mut self, miner_address: String, pool_id: String) -> Result<PoolMiner, String> {
        let pool = self.mining_pools.get_mut(&pool_id)
            .ok_or("Pool not found")?;
        
        if !pool.is_active {
            return Err("Pool is not active".to_string());
        }
        
        let miner = self.miners.get(&miner_address)
            .ok_or("Miner not found")?;
        
        if miner.status != MinerStatus::Active {
            return Err("Miner must be active to join pool".to_string());
        }
        
        // Check if already in a pool
        let pools = self.pool_miners.get_mut(&pool_id).unwrap();
        if pools.iter().any(|pm| pm.miner_address == miner_address) {
            return Err("Miner already in pool".to_string());
        }
        
        let pool_miner = PoolMiner {
            miner_address: miner_address.clone(),
            pool_id: pool_id.clone(),
            joined_at: self.current_epoch,
            hashrate_contributed: self.get_miner_hashrate(&miner_address),
            rewards_earned: 0,
            last_claim_epoch: self.current_epoch,
        };
        
        pools.push(pool_miner.clone());
        pool.total_miners += 1;
        pool.total_hashrate += pool_miner.hashrate_contributed;
        
        Ok(pool_miner)
    }
    
    /// Get miner's hashrate based on tier
    fn get_miner_hashrate(&self, miner_address: &str) -> u64 {
        if let Some(miner) = self.miners.get(miner_address) {
            match miner.device_tier {
                DeviceTier::Mobile => 500,
                DeviceTier::Laptop => 2000,
                DeviceTier::Desktop => 5000,
                DeviceTier::Server => 20000,
            }
        } else {
            0
        }
    }
    
    /// Leave a mining pool
    pub fn leave_pool(&mut self, miner_address: String, pool_id: String) -> Result<(), String> {
        let pool = self.mining_pools.get_mut(&pool_id)
            .ok_or("Pool not found")?;
        
        let miners = self.pool_miners.get_mut(&pool_id).unwrap();
        let idx = miners.iter().position(|pm| pm.miner_address == miner_address)
            .ok_or("Miner not in pool")?;
        
        let miner = miners.remove(idx);
        pool.total_miners -= 1;
        pool.total_hashrate = pool.total_hashrate.saturating_sub(miner.hashrate_contributed);
        
        Ok(())
    }
    
    /// Claim pool mining rewards (distributed proportionally by hashrate)
    pub fn claim_pool_rewards(&mut self, miner_address: &str, pool_id: &str) -> Result<u64, String> {
        let pool = self.mining_pools.get_mut(pool_id)
            .ok_or("Pool not found")?;
        
        let miners = self.pool_miners.get_mut(pool_id).unwrap();
        let miner_record = miners.iter_mut().find(|pm| pm.miner_address == miner_address)
            .ok_or("Miner not in pool")?;
        
        // Calculate pool rewards for epochs since last claim
        let epochs = self.current_epoch - miner_record.last_claim_epoch;
        let pool_reward_per_epoch = self.base_reward_per_epoch * pool.total_miners;
        
        // Miner's share based on hashrate contribution
        let share = if pool.total_hashrate > 0 {
            miner_record.hashrate_contributed as f64 / pool.total_hashrate as f64
        } else {
            0.0
        };
        
        // Apply pool fee
        let gross_reward = (pool_reward_per_epoch * epochs as u64) as f64 * share;
        let pool_fee = gross_reward * (pool.pool_fee_percent / 100.0);
        let net_reward = (gross_reward - pool_fee) as u64;
        
        miner_record.rewards_earned += net_reward;
        miner_record.last_claim_epoch = self.current_epoch;
        pool.total_rewards_distributed += net_reward;
        
        Ok(net_reward)
    }
    
    /// Get pool statistics
    pub fn get_pool_stats(&self, pool_id: &str) -> Option<PoolStats> {
        if let Some(pool) = self.mining_pools.get(pool_id) {
            let miners = self.pool_miners.get(pool_id).unwrap_or(&Vec::new());
            Some(PoolStats {
                pool_id: pool.pool_id.clone(),
                name: pool.name.clone(),
                total_miners: pool.total_miners,
                total_hashrate: pool.total_hashrate,
                avg_hashrate_per_miner: if pool.total_miners > 0 {
                    pool.total_hashrate / pool.total_miners
                } else {
                    0
                },
                pool_fee_percent: pool.pool_fee_percent,
                total_rewards_distributed: pool.total_rewards_distributed,
                is_active: pool.is_active,
            })
        } else {
            None
        }
    }
    
    /// Pool statistics struct
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PoolStats {
        pub pool_id: String,
        pub name: String,
        pub total_miners: u64,
        pub total_hashrate: u64,
        pub avg_hashrate_per_miner: u64,
        pub pool_fee_percent: f64,
        pub total_rewards_distributed: u64,
        pub is_active: bool,
    }
    
    /// Tier counts helper struct
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TierCounts {
        pub mobile: u64,
        pub laptop: u64,
        pub desktop: u64,
        pub server: u64,
        pub total_active: u64,
    }
}
        
        // Daily electricity cost
        let daily_power_cost = (power_watts / 1000.0) * 24 * electricity_cost_per_kwh;
        
        if daily_power_cost > 0.0 {
            daily_rewards / daily_power_cost
        } else {
            daily_rewards
        }
    }
    
    /// Estimate time to recover device cost based on tier
    pub fn estimate_roi_days(&self, tier: DeviceTier, device_cost_usd: f64, token_price_usd: f64) -> u64 {
        let miner = MinerInfo {
            address: "temp".to_string(),
            device_tier: tier,
            total_mined: 0,
            last_claim_epoch: 0,
            consecutive_uptime_epochs: 100,
            reputation_score: 75.0,
            status: MinerStatus::Active,
            registered_at: 0,
            last_active_epoch: 0,
            penalty_count: 0,
        };
        
        let daily_rewards = self.calculate_daily_rewards(&miner) as f64 * token_price_usd;
        if daily_rewards > 0.0 {
            (device_cost_usd / daily_rewards) as u64
        } else {
            u64::MAX
        }
    }
    
    /// Get all miners eligible for rewards (active status)
    pub fn get_active_miners(&self) -> Vec<&MinerInfo> {
        self.miners
            .values()
            .filter(|m| m.status == MinerStatus::Active)
            .collect()
    }
    
    /// Get network average reputation score
    pub fn get_network_average_reputation(&self) -> f64 {
        if self.miners.is_empty() {
            return 50.0; // Default neutral reputation
        }
        let sum: f64 = self.miners.values().map(|m| m.reputation_score).sum();
        sum / self.miners.len() as f64
    }
    
    /// Get estimated next difficulty adjustment
    pub fn estimate_difficulty_change(&self) -> f64 {
        let active_count = self.get_active_miners().len() as f64;
        let total_count = self.network_stats.total_active_miners as f64;
        
        if total_count == 0.0 {
            return 0.0;
        }
        
        let participation_rate = active_count / total_count;
        
        if participation_rate > 0.9 {
            5.0 // 5% increase
        } else if participation_rate < 0.5 {
            -10.0 // 10% decrease
        } else {
            1.0 // 1% gradual increase
        }
    }
    
    /// Get miner leaderboard (top 10 by total mined)
    pub fn get_miner_leaderboard(&self, limit: usize) -> Vec<MinerLeaderboardEntry> {
        let mut miners: Vec<_> = self.miners.values().collect();
        miners.sort_by(|a, b| b.total_mined.cmp(&a.total_mined));
        
        miners.into_iter()
            .take(limit)
            .enumerate()
            .map(|(i, m)| MinerLeaderboardEntry {
                rank: i + 1,
                address: m.address.clone(),
                total_mined: m.total_mined,
                tier: m.device_tier,
                uptime_score: self.calculate_uptime_score(m),
            })
            .collect()
    }
    
    /// Check if a miner is eligible for bonus rewards
    pub fn is_bonus_eligible(&self, miner: &MinerInfo) -> bool {
        miner.status == MinerStatus::Active 
            && miner.reputation_score >= 80.0 
            && miner.consecutive_uptime_epochs >= 100
    }
    
    /// Get bonus multiplier for eligible miners
    pub fn get_bonus_multiplier(&self, miner: &MinerInfo) -> f64 {
        if self.is_bonus_eligible(miner) {
            1.25 // 25% bonus
        } else {
            1.0
        }
    }
    
    /// Get miner by address (immutable reference)
    pub fn get_miner(&self, address: &str) -> Option<&MinerInfo> {
        self.miners.get(address)
    }
    
    /// Get miner by address (mutable reference)
    pub fn get_miner_mut(&mut self, address: &str) -> Option<&mut MinerInfo> {
        self.miners.get_mut(address)
    }
    
    /// Count total registered miners
    pub fn total_registered_miners(&self) -> u64 {
        self.miners.len() as u64
    }
    
    /// Get network utilization percentage
    pub fn get_network_utilization(&self) -> f64 {
        if self.network_stats.total_active_miners == 0 {
            return 0.0;
        }
        
        let active = self.get_active_miners().len() as f64;
        let total = self.network_stats.total_active_miners as f64;
        (active / total) * 100.0
    }
    
    /// Calculate penalty factor based on reputation
    pub fn calculate_penalty_factor(&self, miner: &MinerInfo) -> f64 {
        if miner.reputation_score >= 75.0 {
            1.0 // No penalty
        } else if miner.reputation_score >= 50.0 {
            0.75 // 25% reduction
        } else if miner.reputation_score >= 25.0 {
            0.5 // 50% reduction
        } else {
            0.25 // 75% reduction
        }
    }
    
    /// Get projected rewards for a miner
    pub fn get_miner_stats(&self, address: &str) -> Option<MinerStats> {
        self.miners.get(address).map(|m| {
            let current_reward = self.calculate_reward(m);
            MinerStats {
                address: m.address.clone(),
                device_tier: m.device_tier,
                tier_multiplier: m.device_tier.multiplier(),
                total_mined: m.total_mined,
                current_epoch_reward: current_reward,
                uptime_score: self.calculate_uptime_score(m),
                reputation_score: m.reputation_score,
                status: m.status.clone(),
                consecutive_uptime_epochs: m.consecutive_uptime_epochs,
                penalty_count: m.penalty_count,
            }
        })
    }
}

/// Projected rewards for a miner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedRewards {
    pub address: String,
    pub per_epoch: u64,
    pub daily: u64,
    pub weekly: u64,
    pub monthly: u64,
    pub tier: DeviceTier,
    pub uptime_score: f64,
}

/// Miner statistics for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStats {
    pub address: String,
    pub device_tier: DeviceTier,
    pub tier_multiplier: f64,
    pub total_mined: u64,
    pub current_epoch_reward: u64,
    pub uptime_score: f64,
    pub reputation_score: f64,
    pub status: MinerStatus,
    pub consecutive_uptime_epochs: u64,
    pub penalty_count: u64,
}

/// Tier counts for network stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierCounts {
    pub mobile: u64,
    pub laptop: u64,
    pub desktop: u64,
    pub server: u64,
    pub total_active: u64,
}

/// Miner leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerLeaderboardEntry {
    pub rank: usize,
    pub address: String,
    pub total_mined: u64,
    pub tier: DeviceTier,
    pub uptime_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_tier_multiplier() {
        assert_eq!(DeviceTier::Mobile.multiplier(), 1.0);
        assert_eq!(DeviceTier::Laptop.multiplier(), 2.5);
        assert_eq!(DeviceTier::Desktop.multiplier(), 4.0);
        assert_eq!(DeviceTier::Server.multiplier(), 8.0);
    }

    #[test]
    fn test_register_miner() {
        let mut contract = MiningContract::new();
        let result = contract.register_miner("miner1".to_string(), DeviceTier::Mobile);
        assert!(result.is_ok());
        assert_eq!(contract.network_stats.total_active_miners, 1);
    }

    #[test]
    fn test_calculate_reward() {
        let mut contract = MiningContract::new();
        contract.register_miner("miner1".to_string(), DeviceTier::Desktop).unwrap();
        
        let miner = contract.miners.get("miner1").unwrap();
        let reward = contract.calculate_reward(miner);
        
        // Desktop has 4x multiplier
        assert_eq!(reward, contract.base_reward_per_epoch * 4);
    }

    #[test]
    fn test_reputation_penalty() {
        let mut contract = MiningContract::new();
        contract.register_miner("miner1".to_string(), DeviceTier::Mobile).unwrap();
        
        // Record no participation multiple times
        for _ in 0..5 {
            contract.record_participation("miner1", false).unwrap();
        }
        
        let miner = contract.miners.get("miner1").unwrap();
        assert!(miner.reputation_score < 50.0);
    }
}
