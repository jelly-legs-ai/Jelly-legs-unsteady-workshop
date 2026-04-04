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
    pub halving_interval_epochs: u64, // Epochs between reward halvings
    pub current_halving: u64,        // Current halving epoch count
    pub bonus_multipliers: BonusMultipliers, // Special bonus conditions
}

impl MiningRewardConfig {
    /// Calculate network growth bonus based on miner count change
    pub fn network_growth_bonus(&self, previous_miner_count: u64, current_miner_count: u64) -> f64 {
        // When network grows, early miners earn more (network effect bonus)
        if current_miner_count > previous_miner_count && previous_miner_count > 0 {
            let growth_rate = (current_miner_count as f64 / previous_miner_count as f64) - 1.0;
            // Cap at 25% bonus for 50%+ growth
            (growth_rate * 0.5).min(0.25)
        } else {
            0.0
        }
    }
}

/// Bonus multipliers for special mining conditions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BonusMultipliers {
    pub early_adopter_bonus: f64,    // Bonus for early miners (e.g., 1.5x)
    pub loyalty_bonus: f64,          // Bonus for long-term miners
    pub network_growth_bonus: f64,   // Bonus when network grows
    pub stake_multiplier: f64,       // Bonus for staking while mining
    pub consecutive_epochs_bonus: f64, // Bonus for consecutive mining
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
            halving_interval_epochs: 43800,    // ~5 years at 1hr epochs (Bitcoin-style halving)
            current_halving: 0,
            bonus_multipliers: BonusMultipliers {
                early_adopter_bonus: 1.5,      // 1.5x for early miners
                loyalty_bonus: 0.0,            // Calculated dynamically
                network_growth_bonus: 0.0,     // Reserved for future use
                stake_multiplier: 1.2,         // 1.2x if staking while mining
                consecutive_epochs_bonus: 0.1, // 10% bonus for 99%+ uptime
            },
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
        
        // Apply halving reduction
        let halving_factor = self.calculate_halving_factor(miner.registered_epoch);
        
        // Calculate bonus multipliers
        let bonus_mult = self.calculate_bonus_multipliers(miner);
        
        // Calculate final reward
        let reward = base * tier_mult * contribution_bonus * difficulty_adjustment * halving_factor * bonus_mult;
        
        reward as u64
    }
    
    /// Calculate halving factor based on epoch count
    pub fn calculate_halving_factor(&self, registered_epoch: u64) -> f64 {
        let epochs_since_start = registered_epoch;
        let halvings = epochs_since_start / self.config.halving_interval_epochs;
        
        // Each halving reduces reward by 50%
        0.5_f64.powi(halvings as i32)
    }
    
    /// Calculate combined bonus multipliers for a miner
    pub fn calculate_bonus_multipliers(&self, miner: &Miner) -> f64 {
        let mut multiplier = 1.0;
        
        // Early adopter bonus
        multiplier += self.config.bonus_multipliers.early_adopter_bonus - 1.0;
        
        // Loyalty bonus (increases with epochs mined)
        let loyalty_factor = (miner.epochs_mined as f64 / 1000.0).min(0.5);
        multiplier += loyalty_factor;
        
        // Consecutive epochs bonus
        if miner.uptime_percentage >= 99.0 {
            multiplier += self.config.bonus_multipliers.consecutive_epochs_bonus;
        }
        
        multiplier.min(3.0) // Cap at 3x total bonus
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
    
    /// Calculate compound mining bonus - extra rewards when mining + staking FLUX
    /// Enhanced with exponential scaling for large FLUX stakers
    pub fn calculate_compound_mining_bonus(&self, miner: &Miner, staked_flux: u64) -> f64 {
        if staked_flux == 0 {
            return 1.0;
        }
        
        // Base stake multiplier from config
        let base_mult = self.config.bonus_multipliers.stake_multiplier;
        
        // Enhanced stake scaling with exponential growth for large stakers
        // Tier 1: 0-10K FLUX (linear) -> 1.0x to 1.1x
        // Tier 2: 10K-100K FLUX (linear) -> 1.1x to 1.2x  
        // Tier 3: 100K-1M FLUX (exponential) -> 1.2x to 1.5x
        // Tier 4: 1M+ FLUX (VIP exponential) -> 1.5x to 2.0x
        let stake_scale = if staked_flux < 10_000 {
            1.0 + (staked_flux as f64 / 100_000.0)
        } else if staked_flux < 100_000 {
            1.1 + ((staked_flux - 10_000) as f64 / 900_000.0)
        } else if staked_flux < 1_000_000 {
            // Exponential growth: 1.2x base + logarithmic increase
            let excess = (staked_flux as f64 / 100_000.0).ln() / 10.0;
            1.2 + excess.min(0.3)
        } else {
            // VIP tier: 1.5x base + smaller logarithmic increase
            let excess = (staked_flux as f64 / 1_000_000.0).ln() / 5.0;
            1.5 + excess.min(0.5)
        };
        
        // Combined with uptime bonus
        let uptime_mult = if miner.uptime_percentage >= 99.0 {
            1.15 // 15% extra for excellent uptime
        } else if miner.uptime_percentage >= 95.0 {
            1.08
        } else if miner.uptime_percentage >= 90.0 {
            1.03
        } else {
            1.0
        };
        
        // Cap total multiplier at 3.0x
        (base_mult * stake_scale * uptime_mult).min(3.0)
    }
    
    /// Auto-reinvest mining rewards into staking
    pub fn calculate_reinvest_amount(&self, miner: &Miner, current_epoch: u64, min_threshold: u64) -> u64 {
        let pending = miner.pending_rewards;
        
        if pending < min_threshold {
            return 0;
        }
        
        // Auto-compound 80% of rewards, keep 20% for claiming
        let reinvest = (pending * 80) / 100;
        reinvest
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

/// Mining + Staking Combined Position for compound rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStakePosition {
    pub miner_id: String,
    pub staked_flux: u64,
    pub stake_start_epoch: u64,
    pub mining_rewards_accrued: u64,
    pub auto_compound_enabled: bool,
    pub last_reinvest_epoch: u64,
    pub reinvest_count: u64,
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

/// Dashboard summary for mining network overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningDashboardSummary {
    pub total_miners: u64,
    pub active_miners: u64,
    pub total_rewards_distributed: u64,
    pub avg_contribution_score: f64,
    pub network_hashrate_equivalent: f64,
    pub tier_breakdown: TierBreakdown,
    pub recent_rewards: Vec<MinerRewardSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierBreakdown {
    pub mobile: u64,
    pub laptop: u64,
    pub desktop: u64,
    pub server: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerRewardSnapshot {
    pub miner_id: String,
    pub reward: u64,
    pub epoch: u64,
    pub timestamp: u64,
}

impl MiningCalculator {
    /// Generate dashboard summary from a list of miners
    pub fn generate_dashboard_summary(&self, miners: &[Miner]) -> MiningDashboardSummary {
        let active_miners = miners.iter().filter(|m| m.is_active).count() as u64;
        
        let total_rewards = miners.iter().map(|m| m.total_rewards_earned).sum::<u64>();
        
        let avg_score = if miners.is_empty() {
            0.0
        } else {
            miners.iter().map(|m| m.contribution_score).sum::<f64>() / miners.len() as f64
        };
        
        // Calculate network hashrate equivalent (TH/s simplified)
        let network_hashrate: f64 = miners.iter()
            .filter(|m| m.is_active)
            .map(|m| m.device_tier.multiplier() * m.uptime_percentage / 100.0)
            .sum();
        
        let mut tier_breakdown = TierBreakdown {
            mobile: 0,
            laptop: 0,
            desktop: 0,
            server: 0,
        };
        
        for miner in miners {
            match miner.device_tier {
                DeviceTier::Mobile => tier_breakdown.mobile += 1,
                DeviceTier::Laptop => tier_breakdown.laptop += 1,
                DeviceTier::Desktop => tier_breakdown.desktop += 1,
                DeviceTier::Server => tier_breakdown.server += 1,
            }
        }
        
        // Recent rewards (last 10)
        let mut recent_rewards: Vec<MinerRewardSnapshot> = miners
            .iter()
            .filter(|m| m.total_rewards_earned > 0)
            .take(10)
            .map(|m| MinerRewardSnapshot {
                miner_id: m.miner_id.clone(),
                reward: m.pending_rewards,
                epoch: m.last_claim_epoch,
                timestamp: 0,
            })
            .collect();
        
        recent_rewards.sort_by(|a, b| b.epoch.cmp(&a.epoch));
        
        MiningDashboardSummary {
            total_miners: miners.len() as u64,
            active_miners,
            total_rewards_distributed: total_rewards,
            avg_contribution_score: avg_score,
            network_hashrate_equivalent: network_hashrate,
            tier_breakdown,
            recent_rewards,
        }
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
    fn test_batch_reward_calculation() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut miners = vec![
            Miner::new("miner_001".to_string(), 4, 4),   // Mobile
            Miner::new("miner_002".to_string(), 16, 8), // Desktop
            Miner::new("miner_003".to_string(), 64, 16), // Server
        ];
        
        // Set high contribution scores
        for miner in &mut miners {
            miner.contribution_score = 0.95;
            miner.uptime_percentage = 98.0;
        }
        
        let miner_refs: Vec<&Miner> = miners.iter().collect();
        let rewards = calculator.batch_calculate_rewards(&miner_refs);
        
        assert_eq!(rewards.len(), 3);
        
        // Verify server tier earns more than mobile
        let mobile_reward = rewards.iter().find(|(id, _)| id == "miner_001").unwrap().1;
        let server_reward = rewards.iter().find(|(id, _)| id == "miner_003").unwrap().1;
        
        println!("Mobile reward: {} FLUX", mobile_reward);
        println!("Server reward: {} FLUX", server_reward);
        
        assert!(server_reward > mobile_reward * 8);
    }
    
    #[test]
    fn test_dashboard_summary() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut miners = vec![
            Miner::new("miner_001".to_string(), 4, 4),   // Mobile
            Miner::new("miner_002".to_string(), 16, 8), // Laptop
            Miner::new("miner_003".to_string(), 32, 8), // Desktop
            Miner::new("miner_004".to_string(), 128, 32), // Server
        ];
        
        for miner in &mut miners {
            miner.contribution_score = 0.85;
            miner.uptime_percentage = 95.0;
            miner.total_rewards_earned = 1_000_000_000;
        }
        miners[0].is_active = true;
        miners[1].is_active = true;
        miners[2].is_active = false;
        miners[3].is_active = true;
        
        let summary = calculator.generate_dashboard_summary(&miners);
        
        assert_eq!(summary.total_miners, 4);
        assert_eq!(summary.active_miners, 3);
        assert_eq!(summary.tier_breakdown.mobile, 1);
        assert_eq!(summary.tier_breakdown.laptop, 1);
        assert_eq!(summary.tier_breakdown.desktop, 1);
        assert_eq!(summary.tier_breakdown.server, 1);
        
        println!("Dashboard Summary: {:?}", summary);
    }
    
    #[test]
    fn test_compound_mining_bonus_enhanced() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        let mut miner = Miner::new("test_miner".to_string(), 16, 8);
        miner.contribution_score = 0.9;
        miner.uptime_percentage = 99.0;
        
        // Test different stake tiers
        let bonus_1k = calculator.calculate_compound_mining_bonus(&miner, 1_000);
        let bonus_10k = calculator.calculate_compound_mining_bonus(&miner, 10_000);
        let bonus_100k = calculator.calculate_compound_mining_bonus(&miner, 100_000);
        let bonus_500k = calculator.calculate_compound_mining_bonus(&miner, 500_000);
        let bonus_1m = calculator.calculate_compound_mining_bonus(&miner, 1_000_000);
        let bonus_5m = calculator.calculate_compound_mining_bonus(&miner, 5_000_000);
        
        println!("Bonus at 1K FLUX: {:.3f}x", bonus_1k);
        println!("Bonus at 10K FLUX: {:.3f}x", bonus_10k);
        println!("Bonus at 100K FLUX: {:.3f}x", bonus_100k);
        println!("Bonus at 500K FLUX: {:.3f}x", bonus_500k);
        println!("Bonus at 1M FLUX: {:.3f}x", bonus_1m);
        println!("Bonus at 5M FLUX: {:.3f}x", bonus_5m);
        
        // Verify scaling (each tier should be higher than previous)
        assert!(bonus_10k > bonus_1k);
        assert!(bonus_100k > bonus_10k);
        assert!(bonus_500k > bonus_100k);
        assert!(bonus_1m > bonus_500k);
        assert!(bonus_5m > bonus_1m);
        
        // Verify VIP tier at 1M+ is substantial
        assert!(bonus_1m >= 1.5);
        assert!(bonus_5m >= 1.8);
        
        // Verify cap at 3.0x
        let extreme_bonus = calculator.calculate_compound_mining_bonus(&miner, 100_000_000);
        assert!(extreme_bonus <= 3.0);
    }
    
    #[test]
    fn test_compound_mining_bonus_uptime_tiers() {
        let config = MiningRewardConfig::default();
        let calculator = MiningCalculator::new(config);
        
        // Test different uptime tiers
        let mut miner = Miner::new("test_miner".to_string(), 16, 8);
        miner.contribution_score = 0.9;
        
        miner.uptime_percentage = 91.0;
        let bonus_normal = calculator.calculate_compound_mining_bonus(&miner, 100_000);
        
        miner.uptime_percentage = 95.0;
        let bonus_good = calculator.calculate_compound_mining_bonus(&miner, 100_000);
        
        miner.uptime_percentage = 99.0;
        let bonus_excellent = calculator.calculate_compound_mining_bonus(&miner, 100_000);
        
        println!("Bonus at 91% uptime: {:.3f}x", bonus_normal);
        println!("Bonus at 95% uptime: {:.3f}x", bonus_good);
        println!("Bonus at 99% uptime: {:.3f}x", bonus_excellent);
        
        // Each higher uptime tier should give better bonus
        assert!(bonus_good > bonus_normal);
        assert!(bonus_excellent > bonus_good);
    }
}
