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

    /// Get miner statistics
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
