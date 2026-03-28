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
    // Sprint 22 Enhancements
    pub streak_bonus_multiplier: f64,  // Bonus for consecutive epochs (max 2.0x)
    pub network_bonus_pool: u64,       // Bonus pool for top contributors
    pub peak_hours_multiplier: f64,    // Bonus for mining during peak demand
    pub early_adopter_bonus: f64,      // Bonus for early network participants
    pub geo_diversity_bonus: f64,      // Bonus for underrepresented regions
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
            // Sprint 22 Enhancements
            streak_bonus_multiplier: 1.5,      // Max 1.5x bonus for streaks
            network_bonus_pool: 1_000_000_000, // 10 FLUX bonus pool per epoch
            peak_hours_multiplier: 1.2,        // 20% bonus during peak hours
            early_adopter_bonus: 2.0,          // 2x for first 10K miners (epoch < 8760)
            geo_diversity_bonus: 1.3,          // 30% bonus for underrepresented regions
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
    // Sprint 22 Enhancements
    pub consecutive_epochs: u64,      // Streak counter for bonus
    pub last_active_epoch: u64,       // Track last epoch for streak calculation
    pub region_code: String,          // Geographic region for diversity bonus
    pub peak_hours_mined: u64,        // Epochs mined during peak hours
    pub total_tasks_verified: u64,    // Total verification tasks completed
    pub reputation_score: f64,        // Long-term reputation (0.0-1.0)
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
            // Sprint 22 Enhancements
            consecutive_epochs: 0,
            last_active_epoch: 0,
            region_code: String::from("UNKNOWN"),
            peak_hours_mined: 0,
            total_tasks_verified: 0,
            reputation_score: 0.5,
        }
    }
    
    pub fn new_with_region(miner_id: String, ram_gb: u32, cpu_cores: u32, region: String) -> Self {
        let mut miner = Self::new(miner_id, ram_gb, cpu_cores);
        miner.region_code = region;
        miner
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
        
        // Calculate base reward before bonuses
        let mut reward = base * tier_mult * contribution_bonus * difficulty_adjustment;
        
        // Sprint 22: Apply streak bonus (consecutive epochs)
        let streak_bonus = if miner.consecutive_epochs >= 24 {
            // Max bonus after 24 consecutive epochs
            self.config.streak_bonus_multiplier
        } else if miner.consecutive_epochs >= 12 {
            1.25
        } else if miner.consecutive_epochs >= 6 {
            1.1
        } else {
            1.0
        };
        reward *= streak_bonus.min(self.config.streak_bonus_multiplier);
        
        // Sprint 22: Apply peak hours bonus
        // Peak hours: 09:00-12:00 and 19:00-23:00 UTC
        let current_hour = (miner.last_active_epoch % 24) as u32;
        let is_peak_hour = (current_hour >= 9 && current_hour < 12) || (current_hour >= 19 && current_hour < 23);
        if is_peak_hour {
            reward *= self.config.peak_hours_multiplier;
        }
        
        // Sprint 22: Apply early adopter bonus (first year of network)
        if miner.registered_epoch < 8760 {
            reward *= self.config.early_adopter_bonus;
        }
        
        // Sprint 22: Apply geo diversity bonus for underrepresented regions
        let underrepresented_regions = ["AF", "SA", "SEA", "OC"];
        if underrepresented_regions.contains(&miner.region_code.as_str()) {
            reward *= self.config.geo_diversity_bonus;
        }
        
        // Sprint 22: Apply reputation bonus for long-term miners
        let reputation_bonus = 1.0 + (miner.reputation_score * 0.2);
        reward *= reputation_bonus;
        
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
    
    /// Update miner streak and activity tracking (call each epoch)
    pub fn update_miner_activity(&self, miner: &mut Miner, current_epoch: u64) {
        if current_epoch == miner.last_active_epoch + 1 {
            // Consecutive epoch - increment streak
            miner.consecutive_epochs += 1;
        } else if current_epoch > miner.last_active_epoch + 1 {
            // Gap in mining - reset streak
            miner.consecutive_epochs = 1;
        }
        miner.last_active_epoch = current_epoch;
        
        // Update reputation based on long-term behavior
        if miner.consecutive_epochs >= 48 {
            // Bonus reputation for 2+ day streaks
            miner.reputation_score = (miner.reputation_score + 0.01).min(1.0);
        } else if miner.consecutive_epochs == 0 {
            // Penalty for breaking streak
            miner.reputation_score = (miner.reputation_score - 0.02).max(0.0);
        }
    }
    
    /// Record task verification for contribution tracking
    pub fn record_task_verification(&self, miner: &mut Miner, tasks_verified: u64) {
        miner.total_tasks_verified += tasks_verified;
        // Update contribution score based on verification history
        let expected_tasks = miner.epochs_mined * 10; // Expect ~10 tasks per epoch
        if expected_tasks > 0 {
            miner.contribution_score = (miner.total_tasks_verified as f64 / expected_tasks as f64).min(1.0);
        }
    }
    
    /// Distribute network bonus pool to top contributors
    pub fn distribute_bonus_pool(&self, miners: &mut [Miner], current_epoch: u64) {
        // Sort by contribution score and reputation
        miners.sort_by(|a, b| {
            let score_a = a.contribution_score * 0.6 + a.reputation_score * 0.4;
            let score_b = b.contribution_score * 0.6 + b.reputation_score * 0.4;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Top 10% get bonus from pool
        let top_count = (miners.len() as f64 * 0.1).max(1.0) as usize;
        let bonus_per_miner = self.config.network_bonus_pool / top_count as u64;
        
        for miner in miners.iter_mut().take(top_count) {
            miner.pending_rewards += bonus_per_miner;
        }
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
    
    /// Batch calculate rewards for multiple miners (optimized for bulk operations)
    pub fn batch_calculate_rewards(&self, miners: &[&Miner]) -> Vec<(String, u64)> {
        miners.iter()
            .map(|miner| (miner.miner_id.clone(), self.calculate_epoch_reward(miner)))
            .collect()
    }
    
    /// Calculate reward projection with trend analysis
    pub fn project_rewards_with_trend(&self, miner: &Miner, epochs: u64) -> RewardProjection {
        let base_reward = self.calculate_multi_epoch_reward(miner, epochs);
        
        // Calculate trend based on contribution score trajectory
        let trend_factor = if miner.contribution_score > 0.8 {
            1.15 // Optimistic: 15% bonus for high performers
        } else if miner.contribution_score > 0.5 {
            1.0 // Neutral
        } else {
            0.85 // Pessimistic: 15% reduction for low performers
        };
        
        let projected = (base_reward as f64 * trend_factor) as u64;
        let confidence = match miner.uptime_percentage {
            x if x >= 95.0 => 0.95,
            x if x >= 85.0 => 0.85,
            x if x >= 75.0 => 0.70,
            _ => 0.50,
        };
        
        RewardProjection {
            miner_id: miner.miner_id.clone(),
            epochs,
            base_projection: base_reward,
            trend_adjusted_projection: projected,
            confidence,
            trend: if trend_factor > 1.0 { "positive" } else if trend_factor < 1.0 { "negative" } else { "stable" }.to_string(),
        }
    }
    
    /// Optimize miner configuration for maximum rewards
    pub fn suggest_optimizations(&self, miner: &Miner) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();
        
        // Check uptime optimization
        if miner.uptime_percentage < self.config.uptime_threshold * 100.0 {
            suggestions.push(OptimizationSuggestion {
                category: "uptime".to_string(),
                current_value: miner.uptime_percentage,
                target_value: self.config.uptime_threshold * 100.0,
                impact: "Enable mining rewards (currently earning 0)".to_string(),
                priority: 1,
            });
        } else if miner.uptime_percentage < 95.0 {
            let potential_gain = (95.0 - miner.uptime_percentage) * 0.05;
            suggestions.push(OptimizationSuggestion {
                category: "uptime".to_string(),
                current_value: miner.uptime_percentage,
                target_value: 95.0,
                impact: format!("Potential {:.1}% reward increase", potential_gain),
                priority: 2,
            });
        }
        
        // Check contribution score optimization
        if miner.contribution_score < 0.8 {
            let potential_gain = (0.8 - miner.contribution_score) * self.config.contribution_factor * 100.0;
            suggestions.push(OptimizationSuggestion {
                category: "contribution".to_string(),
                current_value: miner.contribution_score * 100.0,
                target_value: 80.0,
                impact: format!("Potential {:.1}% reward increase", potential_gain),
                priority: 2,
            });
        }
        
        // Check device tier upgrade path
        let next_tier = match miner.device_tier {
            DeviceTier::Mobile => Some((DeviceTier::Laptop, 2.5, "Upgrade to laptop tier (8+ GB RAM)")),
            DeviceTier::Laptop => Some((DeviceTier::Desktop, 5.0, "Upgrade to desktop tier (16+ GB RAM)")),
            DeviceTier::Desktop => Some((DeviceTier::Server, 10.0, "Upgrade to server tier (64+ GB RAM)")),
            DeviceTier::Server => None,
        };
        
        if let Some((tier, multiplier, desc)) = next_tier {
            let current_mult = miner.device_tier.multiplier();
            let upgrade_gain = ((multiplier - current_mult) / current_mult) * 100.0;
            suggestions.push(OptimizationSuggestion {
                category: "hardware".to_string(),
                current_value: current_mult,
                target_value: multiplier,
                impact: format!("{:.0}% reward increase - {}", upgrade_gain, desc),
                priority: 3,
            });
        }
        
        // Sort by priority
        suggestions.sort_by_key(|s| s.priority);
        suggestions
    }
    
    /// Calculate efficiency score for a miner (0-100 scale)
    pub fn calculate_efficiency_score(&self, miner: &Miner) -> EfficiencyScore {
        let uptime_score = (miner.uptime_percentage / 100.0 * 40.0).min(40.0);
        let contribution_score = (miner.contribution_score * 40.0).min(40.0);
        let tier_score = match miner.device_tier {
            DeviceTier::Mobile => 5.0,
            DeviceTier::Laptop => 10.0,
            DeviceTier::Desktop => 15.0,
            DeviceTier::Server => 20.0,
        };
        
        let total = uptime_score + contribution_score + tier_score;
        let grade = match total {
            x if x >= 90.0 => 'A',
            x if x >= 80.0 => 'B',
            x if x >= 70.0 => 'C',
            x if x >= 60.0 => 'D',
            _ => 'F',
        };
        
        EfficiencyScore {
            score: total.round(),
            grade,
            uptime_component: uptime_score,
            contribution_component: contribution_score,
            hardware_component: tier_score,
        }
    }
}

/// Reward projection with trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardProjection {
    pub miner_id: String,
    pub epochs: u64,
    pub base_projection: u64,
    pub trend_adjusted_projection: u64,
    pub confidence: f64,
    pub trend: String,
}

/// Optimization suggestion for miners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub current_value: f64,
    pub target_value: f64,
    pub impact: String,
    pub priority: u32,
}

/// Efficiency score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyScore {
    pub score: f64,
    pub grade: char,
    pub uptime_component: f64,
    pub contribution_component: f64,
    pub hardware_component: f64,
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
    
    #[test]
    fn test_streak_bonus() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut miner_no_streak = Miner::new("no_streak".to_string(), 8, 4);
        miner_no_streak.consecutive_epochs = 0;
        miner_no_streak.last_active_epoch = 100;
        
        let mut miner_long_streak = Miner::new("long_streak".to_string(), 8, 4);
        miner_long_streak.consecutive_epochs = 24;
        miner_long_streak.last_active_epoch = 100;
        
        let reward_no_streak = calculator.calculate_epoch_reward(&miner_no_streak);
        let reward_long_streak = calculator.calculate_epoch_reward(&miner_long_streak);
        
        // Long streak should earn more
        assert!(reward_long_streak > reward_no_streak);
        println!("No streak: {}, 24-epoch streak: {}", reward_no_streak, reward_long_streak);
    }
    
    #[test]
    fn test_early_adopter_bonus() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut early_miner = Miner::new("early".to_string(), 8, 4);
        early_miner.registered_epoch = 100; // Early adopter
        
        let mut late_miner = Miner::new("late".to_string(), 8, 4);
        late_miner.registered_epoch = 10000; // After first year
        
        let early_reward = calculator.calculate_epoch_reward(&early_miner);
        let late_reward = calculator.calculate_epoch_reward(&late_miner);
        
        // Early adopter should earn 2x
        assert_eq!(early_reward, late_reward * 2);
    }
    
    #[test]
    fn test_geo_diversity_bonus() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut us_miner = Miner::new_with_region("us".to_string(), 8, 4, "US".to_string());
        let mut africa_miner = Miner::new_with_region("africa".to_string(), 8, 4, "AF".to_string());
        
        let us_reward = calculator.calculate_epoch_reward(&us_miner);
        let africa_reward = calculator.calculate_epoch_reward(&africa_miner);
        
        // Underrepresented region should get 30% bonus
        assert!(africa_reward > us_reward);
    }
    
    #[test]
    fn test_miner_activity_tracking() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut miner = Miner::new("active".to_string(), 8, 4);
        
        // Simulate 10 consecutive epochs
        for epoch in 1..=10 {
            calculator.update_miner_activity(&mut miner, epoch);
        }
        
        assert_eq!(miner.consecutive_epochs, 10);
        assert_eq!(miner.last_active_epoch, 10);
        
        // Break the streak
        calculator.update_miner_activity(&mut miner, 15);
        assert_eq!(miner.consecutive_epochs, 1); // Reset to 1
    }
    
    #[test]
    fn test_reputation_growth() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut miner = Miner::new("reputable".to_string(), 8, 4);
        let initial_rep = miner.reputation_score;
        
        // 48 consecutive epochs should increase reputation
        for epoch in 1..=48 {
            calculator.update_miner_activity(&mut miner, epoch);
        }
        
        assert!(miner.reputation_score > initial_rep);
    }
}
