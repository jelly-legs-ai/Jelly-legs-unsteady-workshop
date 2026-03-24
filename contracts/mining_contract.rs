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

    /// Calculate mining reward with all factors - SPRINT 22 ENHANCED
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

        // === SPRINT 22 ENHANCEMENT: Advanced Reward Factors ===
        
        // Network contribution bonus - miners who contribute during low-participation epochs get bonus
        let participation_rate = self.get_current_participation_rate();
        let contribution_bonus = if participation_rate < 0.6 {
            // Low participation - active miners get 25% bonus
            1.25
        } else if participation_rate < 0.8 {
            // Medium participation - 10% bonus
            1.10
        } else {
            1.0
        };
        reward *= contribution_bonus;

        // Longevity bonus - miners with 100+ consecutive epochs get extra
        if miner.consecutive_uptime_epochs >= 100 {
            reward *= 1.15; // 15% longevity bonus
        } else if miner.consecutive_uptime_epochs >= 50 {
            reward *= 1.08; // 8% bonus
        } else if miner.consecutive_uptime_epochs >= 25 {
            reward *= 1.04; // 4% bonus
        }

        // Epoch timing bonus - rewards vary by epoch to distribute load
        let epoch_factor = match self.current_epoch % 24 {
            0..=5 => 1.1,   // Early hours - higher reward
            6..=12 => 1.0,  // Normal
            13..=18 => 0.95, // Mid-day - slightly lower
            _ => 1.05,      // Evening - slight boost
        };
        reward *= epoch_factor;

        // Apply floor to rewards
        reward.max(1.0) as u64
    }

    /// Get current network participation rate
    fn get_current_participation_rate(&self) -> f64 {
        if self.network_stats.total_active_miners == 0 {
            return 0.0;
        }
        let active = self.miners.values()
            .filter(|m| m.status == MinerStatus::Active)
            .count() as f64;
        active / self.network_stats.total_active_miners as f64
    }

    // =============================================================================
    // SPRINT 22 ENHANCEMENT: Advanced Mining Analytics & Network Optimization
    // =============================================================================
    
    /// Calculate network decentralization index (0-100 scale)
    pub fn calculate_decentralization_index(&self) -> f64 {
        let total_miners = self.network_stats.total_active_miners;
        if total_miners == 0 {
            return 0.0;
        }
        
        // Tier distribution balance score
        let mobile_ratio = self.network_stats.total_miners_tier_mobile as f64 / total_miners as f64;
        let laptop_ratio = self.network_stats.total_miners_tier_laptop as f64 / total_miners as f64;
        let desktop_ratio = self.network_stats.total_miners_tier_desktop as f64 / total_miners as f64;
        let server_ratio = self.network_stats.total_miners_tier_server as f64 / total_miners as f64;
        
        // Ideal distribution: 50% mobile, 25% laptop, 15% desktop, 10% server
        let ideal = vec![0.50, 0.25, 0.15, 0.10];
        let actual = vec![mobile_ratio, laptop_ratio, desktop_ratio, server_ratio];
        
        // Calculate deviation from ideal (lower = better)
        let mut deviation = 0.0;
        for i in 0..4 {
            deviation += (ideal[i] - actual[i]).abs();
        }
        
        // Convert to 0-100 score (perfect distribution = 100)
        let max_deviation = 2.0; // Worst case
        ((1.0 - deviation / max_deviation) * 100.0).max(0.0)
    }
    
    /// Calculate mining reward efficiency (rewards per unit of hashrate)
    pub fn calculate_reward_efficiency(&self) -> f64 {
        if self.network_stats.network_hashrate_equivalent == 0 {
            return 0.0;
        }
        
        self.network_stats.epoch_rewards_distributed as f64 / 
        self.network_stats.network_hashrate_equivalent as f64
    }
    
    /// Get tier distribution breakdown
    pub fn get_tier_distribution(&self) -> TierDistribution {
        let total = self.network_stats.total_active_miners.max(1) as f64;
        
        TierDistribution {
            mobile: (self.network_stats.total_miners_tier_mobile as f64 / total * 100.0).round(),
            laptop: (self.network_stats.total_miners_tier_laptop as f64 / total * 100.0).round(),
            desktop: (self.network_stats.total_miners_tier_desktop as f64 / total * 100.0).round(),
            server: (self.network_stats.total_miners_tier_server as f64 / total * 100.0).round(),
            total_miners: self.network_stats.total_active_miners,
        }
    }
    
    /// Calculate network hashrate growth rate
    pub fn calculate_hashrate_growth(&self, previous_hashrate: u64) -> f64 {
        if previous_hashrate == 0 {
            return 0.0;
        }
        
        let growth = (self.network_stats.network_hashrate_equivalent as i64 - previous_hashrate as i64) as f64;
        (growth / previous_hashrate as f64) * 100.0
    }
    
    /// Get miner activity heat map by tier
    pub fn get_activity_heatmap(&self) -> ActivityHeatmap {
        let mut tier_activity = HashMap::new();
        
        for (_, miner) in &self.miners {
            if miner.status == MinerStatus::Active {
                let tier_name = match miner.device_tier {
                    DeviceTier::Mobile => "mobile",
                    DeviceTier::Laptop => "laptop",
                    DeviceTier::Desktop => "desktop",
                    DeviceTier::Server => "server",
                };
                
                let uptime_bucket = if miner.consecutive_uptime_epochs >= 30 {
                    "high"
                } else if miner.consecutive_uptime_epochs >= 7 {
                    "medium"
                } else {
                    "low"
                };
                
                let key = format!("{}_{}", tier_name, uptime_bucket);
                *tier_activity.entry(key).or_insert(0) += 1;
            }
        }
        
        ActivityHeatmap {
            by_tier_and_uptime: tier_activity,
            total_active: self.network_stats.total_active_miners,
        }
    }
    
    /// Calculate optimal difficulty adjustment for next epoch
    pub fn calculate_optimal_difficulty(&self, target_block_time: f64, actual_block_time: f64) -> u64 {
        let current_difficulty = self.network_stats.current_epoch_difficulty as f64;
        
        // Difficulty adjustment formula (similar to Bitcoin)
        let adjustment_factor = target_block_time / actual_block_time.max(1.0);
        
        // Cap adjustment to prevent extreme swings (max 4x change per adjustment)
        let capped_factor = adjustment_factor.max(0.25).min(4.0);
        
        let new_difficulty = (current_difficulty * capped_factor) as u64;
        
        // Enforce minimum difficulty floor
        new_difficulty.max(self.emergency_difficulty)
    }
    
    /// Get mining profitability estimate per tier
    pub fn get_profitability_by_tier(&self, electricity_cost_per_hour: f64) -> TierProfitability {
        let mut profitability = HashMap::new();
        
        for tier in [DeviceTier::Mobile, DeviceTier::Laptop, DeviceTier::Desktop, DeviceTier::Server] {
            let tier_miners: Vec<&MinerInfo> = self.miners.values()
                .filter(|m| m.device_tier == tier)
                .collect();
            
            if tier_miners.is_empty() {
                continue;
            }
            
            let avg_reward = tier_miners.iter()
                .map(|m| self.calculate_reward(m))
                .sum::<u64>() / tier_miners.len() as u64;
            
            let power_consumption_watts = match tier {
                DeviceTier::Mobile => 5,
                DeviceTier::Laptop => 45,
                DeviceTier::Desktop => 200,
                DeviceTier::Server => 500,
            };
            
            let electricity_cost = (power_consumption_watts as f64 / 1000.0) * electricity_cost_per_hour;
            let revenue_per_hour = avg_reward as f64; // Assuming 1 epoch = 1 hour
            
            profitability.insert(
                format!("{:?}", tier),
                TierProfit {
                    average_reward: avg_reward,
                    electricity_cost,
                    net_profit: revenue_per_hour - electricity_cost,
                    profit_margin: if revenue_per_hour > 0.0 {
                        ((revenue_per_hour - electricity_cost) / revenue_per_hour * 100.0).round()
                    } else {
                        0.0
                    },
                },
            );
        }
        
        TierProfitability { by_tier: profitability }
    }
    
    /// Identify underperforming miners (for optimization recommendations)
    pub fn get_underperforming_miners(&self, threshold_percentile: f64) -> Vec<UnderperformingMiner> {
        let mut miner_rewards: Vec<(String, u64, f64)> = self.miners.iter()
            .filter(|(_, m)| m.status == MinerStatus::Active)
            .map(|(addr, m)| (addr.clone(), self.calculate_reward(m), self.calculate_uptime_score(m)))
            .collect();
        
        miner_rewards.sort_by(|a, b| a.1.cmp(&b.1));
        
        let threshold_index = (miner_rewards.len() as f64 * threshold_percentile / 100.0) as usize;
        
        miner_rewards.iter()
            .take(threshold_index)
            .map(|(addr, reward, uptime)| {
                let miner = self.miners.get(addr).unwrap();
                UnderperformingMiner {
                    address: addr.clone(),
                    current_reward: *reward,
                    uptime_score: *uptime,
                    device_tier: miner.device_tier,
                    improvement_potential: self.calculate_improvement_potential(miner),
                }
            })
            .collect()
    }
    
    /// Calculate improvement potential for a miner
    pub fn calculate_improvement_potential(&self, miner: &MinerInfo) -> ImprovementPotential {
        let current_reward = self.calculate_reward(miner);
        
        // Potential with perfect uptime
        let max_uptime_reward = {
            let mut temp_miner = miner.clone();
            temp_miner.consecutive_uptime_epochs = 24;
            self.calculate_reward(&temp_miner)
        };
        
        // Potential with upgraded tier
        let next_tier_reward = match miner.device_tier {
            DeviceTier::Mobile => {
                let mut temp = miner.clone();
                temp.device_tier = DeviceTier::Laptop;
                self.calculate_reward(&temp)
            },
            DeviceTier::Laptop => {
                let mut temp = miner.clone();
                temp.device_tier = DeviceTier::Desktop;
                self.calculate_reward(&temp)
            },
            DeviceTier::Desktop => {
                let mut temp = miner.clone();
                temp.device_tier = DeviceTier::Server;
                self.calculate_reward(&temp)
            },
            DeviceTier::Server => current_reward, // Already max tier
        };
        
        ImprovementPotential {
            current_reward,
            with_perfect_uptime: max_uptime_reward,
            with_tier_upgrade: next_tier_reward,
            uptime_gap: max_uptime_reward - current_reward,
            tier_gap: next_tier_reward - current_reward,
            recommended_action: if miner.consecutive_uptime_epochs < miner.device_tier.min_uptime_hours() {
                "Improve uptime consistency"
            } else if miner.device_tier != DeviceTier::Server {
                "Consider tier upgrade"
            } else {
                "Maintain current performance"
            }.to_string(),
        }
    }
    
    /// Calculate network sustainability score (0-100)
    pub fn calculate_sustainability_score(&self) -> SustainabilityScore {
        // Reward distribution sustainability
        let rewards_vs_pool = self.network_stats.epoch_rewards_distributed as f64 / 
                             self.minimum_rewards_pool as f64;
        let rewards_score = (rewards_vs_pool * 50.0).min(50.0);
        
        // Miner retention score (based on reputation distribution)
        let high_rep_miners = self.miners.values()
            .filter(|m| m.reputation_score >= 70.0)
            .count();
        let retention_score = if self.miners.is_empty() {
            0.0
        } else {
            (high_rep_miners as f64 / self.miners.len() as f64) * 25.0
        };
        
        // Decentralization score
        let decentralization_score = self.calculate_decentralization_index() * 0.25;
        
        SustainabilityScore {
            overall_score: (rewards_score + retention_score + decentralization_score).round(),
            rewards_sustainability: rewards_score.round(),
            miner_retention: retention_score.round(),
            decentralization: decentralization_score.round(),
            status: if self.network_stats.epoch_rewards_distributed >= self.minimum_rewards_pool {
                "Sustainable"
            } else {
                "At Risk"
            }.to_string(),
        }
    }
    
    /// Get mining trend analysis (epoch-over-epoch changes)
    pub fn get_mining_trends(&self, previous_epoch_stats: &NetworkMiningStats) -> MiningTrends {
        let miner_change = self.network_stats.total_active_miners as i64 - 
                          previous_epoch_stats.total_active_miners as i64;
        let reward_change = self.network_stats.epoch_rewards_distributed as i64 - 
                           previous_epoch_stats.epoch_rewards_distributed as i64;
        let hashrate_change = self.network_stats.network_hashrate_equivalent as i64 - 
                             previous_epoch_stats.network_hashrate_equivalent as i64;
        
        MiningTrends {
            miner_growth: miner_change,
            miner_growth_percent: if previous_epoch_stats.total_active_miners > 0 {
                (miner_change as f64 / previous_epoch_stats.total_active_miners as f64 * 100.0).round()
            } else {
                0.0
            },
            reward_growth: reward_change,
            reward_growth_percent: if previous_epoch_stats.epoch_rewards_distributed > 0 {
                (reward_change as f64 / previous_epoch_stats.epoch_rewards_distributed as f64 * 100.0).round()
            } else {
                0.0
            },
            hashrate_growth: hashrate_change,
            hashrate_growth_percent: if previous_epoch_stats.network_hashrate_equivalent > 0 {
                (hashrate_change as f64 / previous_epoch_stats.network_hashrate_equivalent as f64 * 100.0).round()
            } else {
                0.0
            },
            trend_direction: if miner_change > 0 && reward_change > 0 {
                "Growing"
            } else if miner_change < 0 && reward_change < 0 {
                "Declining"
            } else {
                "Mixed"
            }.to_string(),
        }
    }
    
    /// Calculate fair reward distribution index (Gini coefficient style)
    pub fn calculate_reward_distribution_fairness(&self) -> f64 {
        let mut rewards: Vec<u64> = self.miners.values()
            .filter(|m| m.status == MinerStatus::Active)
            .map(|m| self.calculate_reward(m))
            .collect();
        
        if rewards.is_empty() {
            return 1.0; // Perfect equality when no miners
        }
        
        rewards.sort();
        let total_rewards: u64 = rewards.iter().sum();
        let n = rewards.len();
        
        // Calculate Gini coefficient (0 = perfect equality, 1 = perfect inequality)
        let mut cumulative = 0u64;
        let mut gini_sum = 0.0;
        
        for (i, &reward) in rewards.iter().enumerate() {
            cumulative += reward;
            let cumulative_percent = cumulative as f64 / total_rewards as f64;
            let ideal_percent = (i + 1) as f64 / n as f64;
            gini_sum += (ideal_percent - cumulative_percent).abs();
        }
        
        let gini = gini_sum / n as f64;
        
        // Convert to fairness score (1 - gini, so 1 = perfect fairness)
        (1.0 - gini).max(0.0)
    }
    
    /// Get personalized mining optimization recommendations for a miner
    pub fn get_personalized_recommendations(&self, address: &str) -> MiningRecommendations {
        let miner = match self.miners.get(address) {
            Some(m) => m,
            None => return MiningRecommendations::default(),
        };
        
        let current_reward = self.calculate_reward(miner);
        let uptime_score = self.calculate_uptime_score(miner);
        let improvement = self.calculate_improvement_potential(miner);
        
        let mut recommendations = Vec::new();
        
        // Uptime recommendation
        if uptime_score < 0.9 {
            recommendations.push(Recommendation {
                category: "Uptime".to_string(),
                current_value: uptime_score,
                target_value: 1.0,
                impact: improvement.uptime_gap,
                action: format!("Maintain {}+ hours consecutive uptime", miner.device_tier.min_uptime_hours()),
                priority: if uptime_score < 0.5 { "High" } else { "Medium" }.to_string(),
            });
        }
        
        // Tier upgrade recommendation
        if miner.device_tier != DeviceTier::Server && improvement.tier_gap > improvement.uptime_gap {
            recommendations.push(Recommendation {
                category: "Hardware".to_string(),
                current_value: miner.device_tier.multiplier(),
                target_value: match miner.device_tier {
                    DeviceTier::Mobile => DeviceTier::Laptop.multiplier(),
                    DeviceTier::Laptop => DeviceTier::Desktop.multiplier(),
                    DeviceTier::Desktop => DeviceTier::Server.multiplier(),
                    DeviceTier::Server => miner.device_tier.multiplier(),
                },
                impact: improvement.tier_gap,
                action: "Upgrade to higher tier device for better rewards".to_string(),
                priority: "Medium".to_string(),
            });
        }
        
        // Reputation building recommendation
        if miner.reputation_score < 70.0 {
            recommendations.push(Recommendation {
                category: "Reputation".to_string(),
                current_value: miner.reputation_score,
                target_value: 100.0,
                impact: (current_reward as f64 * 0.5) as u64, // Potential 50% increase
                action: "Maintain consistent uptime and avoid penalties".to_string(),
                priority: if miner.reputation_score < 50.0 { "High" } else { "Medium" }.to_string(),
            });
        }
        
        MiningRecommendations {
            address: address.to_string(),
            current_reward,
            potential_max_reward: improvement.with_perfect_uptime.max(improvement.with_tier_upgrade),
            recommendations,
            overall_improvement_potential: improvement.with_perfect_uptime.max(improvement.with_tier_upgrade) - current_reward,
        }
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
    
    /// Calculate mining profitability for a given tier (reward per kWh unit)
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
        
        // Daily power cost in USD
        let daily_power_cost = (power_watts / 1000.0) * 24.0 * electricity_cost_per_kwh;
        
        // Profit margin (rewards - costs)
        daily_rewards - daily_power_cost
    }
    
    /// Calculate ROI (Return on Investment) for mining hardware
    pub fn calculate_roi(&self, tier: DeviceTier, hardware_cost: f64) -> f64 {
        let monthly_reward = self.calculate_monthly_rewards_for_tier(tier);
        let monthly_roi = (monthly_reward * 0.0001) / hardware_cost * 100.0; // Convert to percentage
        monthly_roi
    }
    
    /// Calculate monthly rewards for a tier (average)
    pub fn calculate_monthly_rewards_for_tier(&self, tier: DeviceTier) -> u64 {
        let base_reward = self.base_reward_per_epoch;
        let multiplier = tier.multiplier() as u64;
        base_reward * multiplier * 24 * 30 // 30 days, 24 epochs/day
    }
    
    /// Get estimated annual revenue for a miner
    pub fn estimate_annual_revenue(&self, miner: &MinerInfo) -> u64 {
        let monthly = self.calculate_monthly_rewards(miner);
        monthly * 12
    }
    
    /// Calculate compound growth of mining rewards (reinvested)
    pub fn calculate_compound_growth(&self, initial_stake: u64, epochs: u64, reinvest_rate: f64) -> u64 {
        let mut total = initial_stake;
        for _ in 0..epochs {
            let reward = total / 100; // 1% per epoch simplified
            let reinvested = (reward as f64 * reinvest_rate) as u64;
            total += reinvested;
        }
        total
    }
    
    /// Get mining reward distribution schedule
    pub fn get_reward_schedule(&self) -> RewardSchedule {
        RewardSchedule {
            epochs_per_day: 24,
            epochs_per_week: 168,
            epochs_per_month: 720,
            epochs_per_year: 8760,
            base_reward_per_epoch: self.base_reward_per_epoch,
            difficulty_adjustment_interval: self.difficulty_adjustment_interval,
        }
    }
    
    /// Calculate penalty for early unstake (slashing)
    pub fn calculate_slashing_penalty(&self, stake_amount: u64, reason: SlashingReason) -> u64 {
        let penalty_rate = match reason {
            SlashingReason::Downtime => 5, // 5% penalty
            SlashingReason::DoubleSigning => 50, // 50% penalty
            SlashingReason::Fraud => 100, // 100% slash
        };
        stake_amount * penalty_rate / 100
    }
    
    /// Get optimal mining tier recommendation based on budget
    pub fn recommend_tier_for_budget(&self, budget_usd: f64) -> DeviceTier {
        if budget_usd < 100.0 {
            DeviceTier::Mobile
        } else if budget_usd < 500.0 {
            DeviceTier::Laptop
        } else if budget_usd < 1500.0 {
            DeviceTier::Desktop
        } else {
            DeviceTier::Server
        }
    }
    
    /// Calculate break-even point for mining investment
    pub fn calculate_break_even_epochs(&self, tier: DeviceTier, hardware_cost: f64, electricity_cost: f64) -> u64 {
        let profit_per_epoch = self.calculate_mining_profitability(tier, electricity_cost) / 24.0;
        if profit_per_epoch <= 0.0 {
            return u64::MAX; // Never breaks even
        }
        (hardware_cost / profit_per_epoch).ceil() as u64
    }
    
    /// Simulate mining rewards over time with difficulty adjustments
    pub fn simulate_mining_rewards(&self, initial_miners: u64, epochs: u64) -> Vec<EpochSimulation> {
        let mut simulations = Vec::new();
        let mut current_difficulty = self.network_stats.current_epoch_difficulty;
        let mut miner_count = initial_miners;
        
        for epoch in 0..epochs {
            let reward_per_miner = self.base_reward_per_epoch * 1000 / current_difficulty.max(1);
            simulations.push(EpochSimulation {
                epoch,
                difficulty: current_difficulty,
                miner_count,
                reward_per_miner,
                total_distributed: reward_per_miner * miner_count,
            });
            
            // Adjust difficulty every N epochs
            if epoch % self.difficulty_adjustment_interval == 0 && epoch > 0 {
                current_difficulty = self.adjust_difficulty_simulation(current_difficulty, miner_count);
                // Simulate miner churn based on profitability
                if reward_per_miner < 100 {
                    miner_count = miner_count * 95 / 100; // 5% leave
                } else if reward_per_miner > 500 {
                    miner_count = miner_count * 105 / 100; // 5% join
                }
            }
        }
        
        simulations
    }
    
    /// Adjust difficulty in simulation context
    fn adjust_difficulty_simulation(&self, current: u64, miners: u64) -> u64 {
        if miners > 1000 {
            (current as f64 * 1.1).min(5000.0) as u64
        } else if miners < 100 {
            (current as f64 * 0.9).max(self.emergency_difficulty) as u64
        } else {
            current
        }
    }
    
    // =============================================================================
    // ADVANCED MINING ANALYTICS - Sprint Enhancement
    // =============================================================================
    
    /// Calculate mining efficiency score (0-100) based on reward/cost ratio
    pub fn calculate_efficiency_score(&self, miner: &MinerInfo) -> f64 {
        let reward = self.calculate_reward(miner) as f64;
        let uptime = self.calculate_uptime_score(miner);
        let reputation = miner.reputation_score;
        
        // Weighted score: 40% reward, 30% uptime, 30% reputation
        let score = (reward * 0.4) + (uptime * 100.0 * 0.3) + (reputation * 0.3);
        score.min(100.0)
    }
    
    /// Get mining power distribution (Gini coefficient for reward inequality)
    pub fn calculate_reward_gini(&self) -> f64 {
        let mut rewards: Vec<u64> = self.miners.values()
            .map(|m| self.calculate_reward(m))
            .collect();
        rewards.sort();
        
        let n = rewards.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }
        
        let mean = rewards.iter().sum::<u64>() as f64 / n;
        let mut cumsum = 0.0;
        let mut weighted_sum = 0.0;
        
        for (i, &reward) in rewards.iter().enumerate() {
            let r = reward as f64;
            cumsum += r;
            weighted_sum += (i as f64 + 1.0) * r;
        }
        
        let gini = (2.0 * weighted_sum) / (n * cumsum) - (n + 1.0) / n;
        gini.max(0.0).min(1.0)
    }
    
    /// Predict next epoch difficulty based on current trends
    pub fn predict_next_difficulty(&self) -> u64 {
        let participation = self.network_stats.total_active_miners as f64 / 
            (self.network_stats.total_miners_tier_mobile + 
             self.network_stats.total_miners_tier_laptop + 
             self.network_stats.total_miners_tier_desktop + 
             self.network_stats.total_miners_tier_server) as f64;
        
        if participation > 0.9 {
            (self.network_stats.current_epoch_difficulty as f64 * 1.05).min(5000.0) as u64
        } else if participation < 0.5 {
            (self.network_stats.current_epoch_difficulty as f64 * 0.9).max(self.emergency_difficulty) as u64
        } else {
            self.network_stats.current_epoch_difficulty
        }
    }
    
    /// Calculate optimal stake amount for maximum ROI
    pub fn calculate_optimal_stake(&self, token_type: TokenType, budget: u64) -> u64 {
        let pool = self.get_staking_pool_for_token(token_type);
        if let Some(p) = pool {
            // Optimal is min stake for testing, or max for serious mining
            if budget < p.min_stake * 10 {
                p.min_stake
            } else {
                budget.min(1_000_000) // Cap at 1M for safety
            }
        } else {
            100
        }
    }
    
    /// Get staking pool for token type
    fn get_staking_pool_for_token(&self, token_type: TokenType) -> Option<&StakingPool> {
        for (_, pool) in &self.pools {
            if pool.token_type == token_type {
                return Some(pool);
            }
        }
        None
    }
    
    /// Calculate mining hashrate per dollar invested
    pub fn calculate_hashrate_per_dollar(&self, tier: DeviceTier, hardware_cost: f64) -> f64 {
        let hashrate = match tier {
            DeviceTier::Mobile => 0.5,
            DeviceTier::Laptop => 2.0,
            DeviceTier::Desktop => 5.0,
            DeviceTier::Server => 20.0,
        };
        hashrate * 1000.0 / hardware_cost.max(1.0)
    }
    
    /// Estimate time to reach mining milestone
    pub fn estimate_time_to_milestone(&self, miner: &MinerInfo, milestone: u64) -> u64 {
        let remaining = milestone - miner.total_mined;
        if remaining == 0 {
            return 0;
        }
        let reward_per_epoch = self.calculate_reward(miner);
        if reward_per_epoch == 0 {
            return u64::MAX;
        }
        remaining / reward_per_epoch
    }
    
    /// Get mining risk assessment (Low/Medium/High)
    pub fn assess_mining_risk(&self, miner: &MinerInfo) -> MiningRisk {
        if miner.status == MinerStatus::Slashed {
            MiningRisk::Critical
        } else if miner.penalty_count >= 2 {
            MiningRisk::High
        } else if miner.reputation_score < 30.0 {
            MiningRisk::Medium
        } else if miner.consecutive_uptime_epochs < 3 {
            MiningRisk::Low
        } else {
            MiningRisk::VeryLow
        }
    }
    
    /// Calculate diversification score for mining portfolio
    pub fn calculate_diversification_score(&self, stakes: &[StakeInfo]) -> f64 {
        if stakes.is_empty() {
            return 0.0;
        }
        
        let mut token_counts: HashMap<TokenType, u64> = HashMap::new();
        for stake in stakes {
            *token_counts.entry(stake.token_type.clone()).or_insert(0) += stake.amount;
        }
        
        let total = stakes.iter().map(|s| s.amount).sum::<u64>() as f64;
        if total == 0.0 {
            return 0.0;
        }
        
        // Shannon diversity index
        let mut diversity = 0.0;
        for (_, amount) in token_counts {
            let p = amount as f64 / total;
            if p > 0.0 {
                diversity -= p * p.ln();
            }
        }
        
        diversity.min(2.0) // Normalize to 0-2 scale
    }
            if epoch % self.difficulty_adjustment_interval == 0 && epoch > 0 {
                current_difficulty = (current_difficulty as f64 * 1.02).min(5000.0) as u64;
                miner_count = (miner_count as f64 * 1.01).min(10000.0) as u64; // Growth simulation
            }
        }
        
        simulations
    }
    
    /// Get mining efficiency score (0-100)
    pub fn calculate_efficiency_score(&self, miner: &MinerInfo) -> u64 {
        let uptime_score = (self.calculate_uptime_score(miner) * 50.0) as u64;
        let reputation_score = (miner.reputation_score / 2.0) as u64;
        uptime_score + reputation_score
    }
}

/// Tier counts summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierCounts {
    pub mobile: u64,
    pub laptop: u64,
    pub desktop: u64,
    pub server: u64,
    pub total_active: u64,
}

/// Reward schedule information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardSchedule {
    pub epochs_per_day: u64,
    pub epochs_per_week: u64,
    pub epochs_per_month: u64,
    pub epochs_per_year: u64,
    pub base_reward_per_epoch: u64,
    pub difficulty_adjustment_interval: u64,
}

/// Slashing reasons
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SlashingReason {
    Downtime,
    DoubleSigning,
    Fraud,
}

/// Epoch simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSimulation {
    pub epoch: u64,
    pub difficulty: u64,
    pub miner_count: u64,
    pub reward_per_miner: u64,
    pub total_distributed: u64,
}
        
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
    
    // =============================================================================
    // SPRINT 22 ANALYTICS STRUCTS
    // =============================================================================
    
    /// Tier distribution percentages
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TierDistribution {
        pub mobile: f64,
        pub laptop: f64,
        pub desktop: f64,
        pub server: f64,
        pub total_miners: u64,
    }
    
    /// Activity heatmap by tier and uptime
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ActivityHeatmap {
        pub by_tier_and_uptime: HashMap<String, u64>,
        pub total_active: u64,
    }
    
    /// Tier profitability analysis
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TierProfitability {
        pub by_tier: HashMap<String, TierProfit>,
    }
    
    /// Individual tier profit metrics
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TierProfit {
        pub average_reward: u64,
        pub electricity_cost: f64,
        pub net_profit: f64,
        pub profit_margin: f64,
    }
    
    /// Underperforming miner analysis
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UnderperformingMiner {
        pub address: String,
        pub current_reward: u64,
        pub uptime_score: f64,
        pub device_tier: DeviceTier,
        pub improvement_potential: ImprovementPotential,
    }
    
    /// Improvement potential breakdown
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ImprovementPotential {
        pub current_reward: u64,
        pub with_perfect_uptime: u64,
        pub with_tier_upgrade: u64,
        pub uptime_gap: u64,
        pub tier_gap: u64,
        pub recommended_action: String,
    }
    
    /// Network sustainability score
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SustainabilityScore {
        pub overall_score: f64,
        pub rewards_sustainability: f64,
        pub miner_retention: f64,
        pub decentralization: f64,
        pub status: String,
    }
    
    /// Mining trend analysis
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MiningTrends {
        pub miner_growth: i64,
        pub miner_growth_percent: f64,
        pub reward_growth: i64,
        pub reward_growth_percent: f64,
        pub hashrate_growth: i64,
        pub hashrate_growth_percent: f64,
        pub trend_direction: String,
    }
    
    /// Personalized mining recommendations
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct MiningRecommendations {
        pub address: String,
        pub current_reward: u64,
        pub potential_max_reward: u64,
        pub recommendations: Vec<Recommendation>,
        pub overall_improvement_potential: u64,
    }
    
    /// Individual recommendation item
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Recommendation {
        pub category: String,
        pub current_value: f64,
        pub target_value: f64,
        pub impact: u64,
        pub action: String,
        pub priority: String,
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
    
    // =============================================================================
    // ADVANCED REWARD ANALYTICS - SPRINT ENHANCEMENT
    // =============================================================================
    
    /// Calculate reward distribution across all tiers
    pub fn get_tier_reward_distribution(&self) -> TierRewardDistribution {
        let mut distribution = TierRewardDistribution {
            mobile: (0, 0),
            laptop: (0, 0),
            desktop: (0, 0),
            server: (0, 0),
            total: 0,
        };
        
        for miner in self.miners.values() {
            if miner.status != MinerStatus::Active {
                continue;
            }
            
            let reward = self.calculate_reward(miner);
            let count = match miner.device_tier {
                DeviceTier::Mobile => &mut distribution.mobile.0,
                DeviceTier::Laptop => &mut distribution.laptop.0,
                DeviceTier::Desktop => &mut distribution.desktop.0,
                DeviceTier::Server => &mut distribution.server.0,
            };
            *count += 1;
            
            let total_reward = match miner.device_tier {
                DeviceTier::Mobile => &mut distribution.mobile.1,
                DeviceTier::Laptop => &mut distribution.laptop.1,
                DeviceTier::Desktop => &mut distribution.desktop.1,
                DeviceTier::Server => &mut distribution.server.1,
            };
            *total_reward += reward;
            
            distribution.total += reward;
        }
        
        distribution
    }
    
    /// Get reward percentile rankings (top 10%, 25%, 50%, 75%)
    pub fn get_reward_percentiles(&self) -> RewardPercentiles {
        let mut rewards: Vec<u64> = self.miners
            .values()
            .filter(|m| m.status == MinerStatus::Active)
            .map(|m| self.calculate_reward(m))
            .collect();
        
        rewards.sort();
        let len = rewards.len();
        
        if len == 0 {
            return RewardPercentiles {
                p10: 0,
                p25: 0,
                p50: 0,
                p75: 0,
                p90: 0,
                max: 0,
                min: 0,
                mean: 0.0,
            };
        }
        
        let sum: u64 = rewards.iter().sum();
        RewardPercentiles {
            p10: rewards[len * 10 / 100],
            p25: rewards[len * 25 / 100],
            p50: rewards[len * 50 / 100],
            p75: rewards[len * 75 / 100],
            p90: rewards[len * 90 / 100],
            max: *rewards.last().unwrap(),
            min: *rewards.first().unwrap(),
            mean: sum as f64 / len as f64,
        }
    }
    
    /// Calculate optimal stake amount for maximum ROI
    pub fn calculate_optimal_stake_amount(&self, budget: u64, risk_tolerance: f64) -> StakeRecommendation {
        // Risk tolerance: 0.0 (conservative) to 1.0 (aggressive)
        let recommended_tier = match risk_tolerance {
            r if r < 0.25 => DeviceTier::Mobile,      // Conservative
            r if r < 0.5 => DeviceTier::Laptop,       // Moderate
            r if r < 0.75 => DeviceTier::Desktop,     // Aggressive
            _ => DeviceTier::Server,                  // Very aggressive
        };
        
        let miner = MinerInfo {
            address: "temp".to_string(),
            device_tier: recommended_tier,
            total_mined: 0,
            last_claim_epoch: 0,
            consecutive_uptime_epochs: 50,
            reputation_score: 60.0,
            status: MinerStatus::Active,
            registered_at: 0,
            last_active_epoch: 0,
            penalty_count: 0,
        };
        
        let daily_reward = self.calculate_daily_rewards(&miner);
        let monthly_reward = self.calculate_monthly_rewards(&miner);
        let break_even_days = self.calculate_break_even_epochs(recommended_tier, budget as f64, 0.12) / 24;
        
        StakeRecommendation {
            tier: recommended_tier,
            estimated_daily_reward: daily_reward,
            estimated_monthly_reward: monthly_reward,
            break_even_days,
            roi_12_months: (monthly_reward * 12) as f64 / budget as f64 * 100.0,
        }
    }
    
    /// Get network reward health score (0-100)
    pub fn get_network_reward_health(&self) -> NetworkRewardHealth {
        let active_miners = self.get_active_miners().len();
        let total_rewards = self.network_stats.epoch_rewards_distributed;
        let avg_reputation = self.get_network_average_reputation();
        let participation_rate = self.get_network_utilization();
        
        // Score components
        let activity_score = (active_miners as f64 / 100.0).min(40.0); // Max 40 points
        let distribution_score = if total_rewards > 0 { 30.0 } else { 0.0 }; // 30 points
        let reputation_score = (avg_reputation / 100.0) * 20.0; // Max 20 points
        let participation_score = (participation_rate / 100.0) * 10.0; // Max 10 points
        
        let total_score = activity_score + distribution_score + reputation_score + participation_score;
        
        NetworkRewardHealth {
            score: total_score as u64,
            activity_score: activity_score as u64,
            distribution_score: distribution_score as u64,
            reputation_score: reputation_score as u64,
            participation_score: participation_score as u64,
            status: if total_score >= 80.0 { "Excellent" }
                    else if total_score >= 60.0 { "Good" }
                    else if total_score >= 40.0 { "Fair" }
                    else { "Poor" }.to_string(),
        }
    }
    
    /// Compare mining vs staking returns for same capital
    pub fn compare_mining_staking_returns(&self, capital: u64, epochs: u64) -> ReturnComparison {
        // Mining returns (server tier for max returns)
        let server_miner = MinerInfo {
            address: "temp".to_string(),
            device_tier: DeviceTier::Server,
            total_mined: 0,
            last_claim_epoch: 0,
            consecutive_uptime_epochs: 100,
            reputation_score: 80.0,
            status: MinerStatus::Active,
            registered_at: 0,
            last_active_epoch: 0,
            penalty_count: 0,
        };
        
        let mining_reward = self.calculate_reward(&server_miner) * epochs;
        
        // Staking returns (ATH pool - highest APY)
        let staking_apy = self.pools.get("ath_staking").map(|p| p.reward_rate).unwrap_or(0.25);
        let staking_reward = (capital as f64 * staking_apy * epochs as f64 / 365.0) as u64;
        
        ReturnComparison {
            mining_return: mining_reward,
            staking_return: staking_reward,
            better_option: if mining_reward > staking_reward { "Mining" } else { "Staking" }.to_string(),
            difference: (mining_reward as i64 - staking_reward as i64).abs() as u64,
            mining_roi: mining_reward as f64 / capital as f64 * 100.0,
            staking_roi: staking_reward as f64 / capital as f64 * 100.0,
        }
    }
}

/// Tier reward distribution (miner count, total rewards)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRewardDistribution {
    pub mobile: (u64, u64),
    pub laptop: (u64, u64),
    pub desktop: (u64, u64),
    pub server: (u64, u64),
    pub total: u64,
}

/// Reward percentile statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardPercentiles {
    pub p10: u64,
    pub p25: u64,
    pub p50: u64,
    pub p75: u64,
    pub p90: u64,
    pub max: u64,
    pub min: u64,
    pub mean: f64,
}

/// Stake recommendation based on budget and risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeRecommendation {
    pub tier: DeviceTier,
    pub estimated_daily_reward: u64,
    pub estimated_monthly_reward: u64,
    pub break_even_days: u64,
    pub roi_12_months: f64,
}

/// Network reward health score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRewardHealth {
    pub score: u64,
    pub activity_score: u64,
    pub distribution_score: u64,
    pub reputation_score: u64,
    pub participation_score: u64,
    pub status: String,
}

/// Mining vs staking return comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnComparison {
    pub mining_return: u64,
    pub staking_return: u64,
    pub better_option: String,
    pub difference: u64,
    pub mining_roi: f64,
    pub staking_roi: f64,
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
