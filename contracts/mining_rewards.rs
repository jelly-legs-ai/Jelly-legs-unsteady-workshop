// Mining Reward Calculation - AeTHer Chain
// Calculates FLUX mining rewards for mobile, laptop, and desktop devices
// Based on proof-of-contribution (PoC) algorithm

use serde::{Deserialize, Serialize};

/// Device tier for mining reward multipliers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceTier {
    Mobile,      // Smartphone (Android/iOS)
    Laptop,      // Laptop/notebook
    Desktop,     // Desktop/workstation
    Server,      // Dedicated server/VPS
}

impl DeviceTier {
    pub fn multiplier(&self) -> f64 {
        match self {
            DeviceTier::Mobile => 1.0,
            DeviceTier::Laptop => 2.5,
            DeviceTier::Desktop => 5.0,
            DeviceTier::Server => 10.0,
        }
    }
    
    pub fn from_ram_gb(ram_gb: u32) -> Self {
        if ram_gb < 4 {
            DeviceTier::Mobile
        } else if ram_gb < 16 {
            DeviceTier::Laptop
        } else if ram_gb < 64 {
            DeviceTier::Desktop
        } else {
            DeviceTier::Server
        }
    }
}

/// Mining reward configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningRewardConfig {
    pub base_reward_per_epoch: u64,  // Base FLUX per epoch (1 FLUX = 10^8)
    pub epoch_duration_seconds: u64, // Epoch length in seconds
    pub uptime_threshold: f64,       // Minimum uptime for rewards (e.g., 0.8 = 80%)
    pub contribution_factor: f64,    // Weight of contribution score
    pub network_difficulty: f64,     // Dynamic difficulty adjustment
    pub inflation_cap: u64,          // Max FLUX mintable per year
    pub total_minted: u64,           // Track total minted FLUX
}

impl Default for MiningRewardConfig {
    fn default() -> Self {
        Self {
            base_reward_per_epoch: 10_000_000, // 0.1 FLUX base per epoch
            epoch_duration_seconds: 3600,      // 1 hour epochs
            uptime_threshold: 0.8,             // 80% minimum uptime
            contribution_factor: 0.5,          // 50% weight on contribution
            network_difficulty: 1.0,           // Start at base difficulty
            inflation_cap: 100_000_000_000_000, // 1 billion FLUX per year (8 decimals)
            total_minted: 0,
        }
    }
}

/// Miner information and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Miner {
    pub miner_id: String,
    pub device_tier: DeviceTier,
    pub ram_gb: u32,
    pub cpu_cores: u32,
    pub uptime_percentage: f64,
    pub contribution_score: f64, // 0.0 to 1.0
    pub epochs_mined: u64,
    pub total_rewards_earned: u64,
    pub pending_rewards: u64,
    pub last_claim_epoch: u64,
    pub is_active: bool,
    pub registered_epoch: u64,
}

impl Miner {
    pub fn new(miner_id: String, ram_gb: u32, cpu_cores: u32) -> Self {
        let device_tier = DeviceTier::from_ram_gb(ram_gb);
        
        Self {
            miner_id,
            device_tier,
            ram_gb,
            cpu_cores,
            uptime_percentage: 100.0,
            contribution_score: 0.5,
            epochs_mined: 0,
            total_rewards_earned: 0,
            pending_rewards: 0,
            last_claim_epoch: 0,
            is_active: true,
            registered_epoch: 0,
        }
    }
    
    pub fn update_contribution_score(&mut self, tasks_completed: u64, tasks_assigned: u64) {
        if tasks_assigned == 0 {
            self.contribution_score = 0.0;
        } else {
            self.contribution_score = tasks_completed as f64 / tasks_assigned as f64;
        }
    }
}

/// Mining reward calculation engine
pub struct MiningCalculator {
    config: MiningRewardConfig,
}

impl MiningCalculator {
    pub fn new(config: MiningRewardConfig) -> Self {
        Self { config }
    }
    
    /// Calculate base reward for a single epoch
    pub fn calculate_epoch_reward(&self, miner: &Miner) -> u64 {
        if !miner.is_active {
            return 0;
        }
        
        // Check uptime threshold
        if miner.uptime_percentage < self.config.uptime_threshold * 100.0 {
            return 0;
        }
        
        // Base reward
        let base = self.config.base_reward_per_epoch as f64;
        
        // Apply device tier multiplier
        let tier_mult = miner.device_tier.multiplier();
        
        // Apply contribution score factor
        let contribution_factor = self.config.contribution_factor;
        let contribution_bonus = 1.0 + (miner.contribution_score * contribution_factor);
        
        // Apply network difficulty
        let difficulty_adjustment = 1.0 / self.config.network_difficulty;
        
        // Calculate final reward
        let reward = base * tier_mult * contribution_bonus * difficulty_adjustment;
        
        reward as u64
    }
    
    /// Calculate rewards for multiple epochs
    pub fn calculate_multi_epoch_reward(&self, miner: &Miner, epochs: u64) -> u64 {
        let epoch_reward = self.calculate_epoch_reward(miner);
        epoch_reward * epochs
    }
    
    /// Claim pending rewards (call after calculation)
    pub fn claim_rewards(&self, miner: &mut Miner, current_epoch: u64) -> u64 {
        let epochs_since_claim = current_epoch - miner.last_claim_epoch;
        let reward = self.calculate_multi_epoch_reward(miner, epochs_since_claim);
        
        miner.pending_rewards += reward;
        miner.total_rewards_earned += reward;
        miner.epochs_mined += epochs_since_claim;
        miner.last_claim_epoch = current_epoch;
        
        self.config.total_minted += reward;
        
        reward
    }
    
    /// Adjust network difficulty based on total hashrate/miners
    pub fn adjust_difficulty(&mut self, total_miners: u64, target_miners: u64) {
        if total_miners > target_miners {
            // Increase difficulty to reduce rewards
            self.config.network_difficulty *= 1.05;
        } else if total_miners < target_miners / 2 {
            // Decrease difficulty to increase rewards
            self.config.network_difficulty /= 1.05;
        }
        
        // Clamp difficulty between 0.5 and 2.0
        self.config.network_difficulty = self.config.network_difficulty.max(0.5).min(2.0);
    }
    
    /// Check if inflation cap is reached
    pub fn check_inflation_cap(&self) -> bool {
        self.config.total_minted >= self.config.inflation_cap
    }
    
    /// Get estimated daily rewards for a miner
    pub fn estimate_daily_rewards(&self, miner: &Miner) -> u64 {
        // 24 epochs per day (1 hour each)
        self.calculate_multi_epoch_reward(miner, 24)
    }
    
    /// Get estimated monthly rewards for a miner
    pub fn estimate_monthly_rewards(&self, miner: &Miner) -> u64 {
        // ~720 epochs per month
        self.calculate_multi_epoch_reward(miner, 720)
    }
}

/// Mining pool for combined mining operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningPool {
    pub pool_id: String,
    pub pool_name: String,
    pub owner_id: String,
    pub members: Vec<String>,
    pub total_hashrate: f64,
    pub pool_fee: f64,           // Pool fee percentage (e.g., 0.02 = 2%)
    pub total_rewards: u64,
    pub last_distribution_epoch: u64,
}

impl MiningPool {
    pub fn new(pool_id: String, pool_name: String, owner_id: String) -> Self {
        Self {
            pool_id,
            pool_name,
            owner_id,
            members: vec![owner_id.clone()],
            total_hashrate: 0.0,
            pool_fee: 0.02,
            total_rewards: 0,
            last_distribution_epoch: 0,
        }
    }
    
    pub fn add_member(&mut self, miner_id: String) {
        if !self.members.contains(&miner_id) {
            self.members.push(miner_id);
        }
    }
    
    pub fn distribute_rewards(&mut self, total_reward: u64, current_epoch: u64) {
        let fee = (total_reward as f64 * self.pool_fee) as u64;
        let distributable = total_reward - fee;
        
        // Simple equal distribution (can be enhanced with hashrate weighting)
        let per_member = distributable / self.members.len() as u64;
        
        self.total_rewards += distributable;
        self.last_distribution_epoch = current_epoch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_tier_multiplier() {
        assert_eq!(DeviceTier::Mobile.multiplier(), 1.0);
        assert_eq!(DeviceTier::Laptop.multiplier(), 2.5);
        assert_eq!(DeviceTier::Desktop.multiplier(), 5.0);
        assert_eq!(DeviceTier::Server.multiplier(), 10.0);
    }

    #[test]
    fn test_device_tier_from_ram() {
        assert_eq!(DeviceTier::from_ram_gb(2), DeviceTier::Mobile);
        assert_eq!(DeviceTier::from_ram_gb(8), DeviceTier::Laptop);
        assert_eq!(DeviceTier::from_ram_gb(32), DeviceTier::Desktop);
        assert_eq!(DeviceTier::from_ram_gb(128), DeviceTier::Server);
    }

    #[test]
    fn test_mining_reward_calculation() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut mobile_miner = Miner::new("mobile_001".to_string(), 4, 4);
        let mut desktop_miner = Miner::new("desktop_001".to_string(), 32, 8);
        
        // Set contribution scores
        mobile_miner.contribution_score = 0.9;
        desktop_miner.contribution_score = 0.9;
        
        let mobile_reward = calculator.calculate_epoch_reward(&mobile_miner);
        let desktop_reward = calculator.calculate_epoch_reward(&desktop_miner);
        
        println!("Mobile miner reward: {} FLUX", mobile_reward);
        println!("Desktop miner reward: {} FLUX", desktop_reward);
        
        // Desktop should earn ~5x mobile (tier multiplier)
        assert!(desktop_reward > mobile_reward * 4);
        assert!(desktop_reward < mobile_reward * 6);
    }

    #[test]
    fn test_uptime_threshold() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut miner = Miner::new("test_miner".to_string(), 8, 4);
        miner.uptime_percentage = 75.0; // Below 80% threshold
        
        let reward = calculator.calculate_epoch_reward(&miner);
        assert_eq!(reward, 0); // No reward below threshold
        
        miner.uptime_percentage = 85.0; // Above threshold
        let reward = calculator.calculate_epoch_reward(&miner);
        assert!(reward > 0);
    }

    #[test]
    fn test_daily_reward_estimation() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut miner = Miner::new("test_miner".to_string(), 16, 8);
        miner.contribution_score = 1.0;
        
        let daily = calculator.estimate_daily_rewards(&miner);
        let monthly = calculator.estimate_monthly_rewards(&miner);
        
        println!("Daily estimate: {} FLUX", daily);
        println!("Monthly estimate: {} FLUX", monthly);
        
        assert!(monthly > daily * 28);
        assert!(monthly < daily * 32);
    }
}
