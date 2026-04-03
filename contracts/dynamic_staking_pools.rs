// Dynamic Staking Pool Manager - AeTHer Chain
// Manages staking pools with dynamic APY adjustment based on network utilization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network utilization metrics for APY adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUtilization {
    pub active_validators: u64,
    pub total_stake: u64,
    pub target_stake: u64,
    pub utilization_ratio: f64,
    pub demand_factor: f64,
}

/// Dynamic staking pool with market-responsive APY
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicStakingPool {
    pub pool_id: String,
    pub name: String,
    pub base_apy: f64,
    pub min_stake: u64,
    pub max_stake: u64,
    pub current_apr: f64,
    pub dynamic_adjustment_enabled: bool,
    pub adjustment_range: f64,      // Max +/- adjustment (e.g., 0.05 = 5%)
    pub target_utilization: f64,     // Target stake utilization (0.0-1.0)
    pub active_stakers: u64,
    pub total_staked: u64,
    pub lockup_epochs: u64,
}

/// Pool performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolPerformance {
    pub pool_id: String,
    pub period: String,
    pub avg_apr: f64,
    pub total_rewards_distributed: u64,
    pub active_stakers: u64,
    pub stake_growth_percent: f64,
}

/// Dynamic APY adjustment engine
pub struct DynamicAPYEngine {
    config: APYConfig,
    pool_histories: HashMap<String, Vec<PoolPerformance>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APYConfig {
    pub min_apr: f64,              // Minimum APR floor
    pub max_apr: f64,              // Maximum APR ceiling
    pub adjustment_interval: u64, // Epochs between adjustments
    pub smoothing_factor: f64,     // How quickly APR responds (0.0-1.0)
    pub demand_weight: f64,        // Weight of demand in adjustment
    pub utilization_weight: f64,   // Weight of utilization in adjustment
}

impl Default for APYConfig {
    fn default() -> Self {
        Self {
            min_apr: 0.05,         // 5% minimum
            max_apr: 0.25,         // 25% maximum
            adjustment_interval: 100,
            smoothing_factor: 0.1, // Slow adjustment
            demand_weight: 0.6,
            utilization_weight: 0.4,
        }
    }
}

impl DynamicAPYEngine {
    pub fn new(config: APYConfig) -> Self {
        Self {
            config,
            pool_histories: HashMap::new(),
        }
    }

    /// Calculate demand factor based on staking activity
    pub fn calculate_demand_factor(
        &self,
        current_stake: u64,
        target_stake: u64,
        pending_unstakes: u64,
        new_stake_requests: u64,
    ) -> f64 {
        // High demand = high APR
        let stake_ratio = if target_stake > 0 {
            (current_stake as f64 / target_stake as f64).min(2.0)
        } else {
            1.0
        };
        
        // Request pressure
        let request_ratio = if pending_unstakes > 0 {
            new_stake_requests as f64 / pending_unstakes as f64
        } else {
            new_stake_requests as f64 * 2.0 // High default if no unstakes
        }.min(3.0);
        
        // Combined demand factor
        (stake_ratio * 0.7 + request_ratio * 0.3).min(2.0)
    }

    /// Calculate utilization factor
    pub fn calculate_utilization_factor(
        &self,
        current_stake: u64,
        max_stake: u64,
        target_utilization: f64,
    ) -> f64 {
        if max_stake == 0 {
            return 1.0;
        }
        
        let actual_utilization = current_stake as f64 / max_stake as f64;
        
        // If under target, increase APR to attract stakers
        // If over target, decrease APR to discourage over-staking
        if actual_utilization < target_utilization {
            // Boost APR to attract more stake
            1.0 + (target_utilization - actual_utilization) * 0.5
        } else {
            // Reduce APR to discourage excess stake
            1.0 - (actual_utilization - target_utilization) * 0.3
        }.max(0.5).min(1.5)
    }

    /// Calculate new APR based on market conditions
    pub fn calculate_adjusted_apr(
        &self,
        pool: &DynamicStakingPool,
        utilization: &NetworkUtilization,
        pending_unstakes: u64,
        new_stake_requests: u64,
    ) -> f64 {
        // Get demand factor
        let demand_factor = self.calculate_demand_factor(
            pool.total_staked,
            utilization.target_stake,
            pending_unstakes,
            new_stake_requests,
        );
        
        // Get utilization factor
        let util_factor = self.calculate_utilization_factor(
            pool.total_staked,
            pool.max_stake,
            pool.target_utilization,
        );
        
        // Weighted adjustment
        let combined_factor = demand_factor * self.config.demand_weight 
            + util_factor * self.config.utilization_weight;
        
        // Calculate new APR
        let adjustment = (combined_factor - 1.0) * pool.adjustment_range;
        let new_apr = pool.base_apy + adjustment;
        
        // Clamp to configured bounds (but respect pool-specific floor/ceiling)
        let pool_min = pool.base_apy - pool.adjustment_range;
        let pool_max = pool.base_apy + pool.adjustment_range;
        
        new_apr.max(pool_min).min(pool_max)
    }

    /// Smooth APR transition to avoid sudden jumps
    pub fn smooth_apr_transition(&self, current_apr: f64, target_apr: f64) -> f64 {
        let diff = target_apr - current_apr;
        current_apr + diff * self.config.smoothing_factor
    }

    /// Record pool performance for historical tracking
    pub fn record_performance(&mut self, pool_id: String, performance: PoolPerformance) {
        self.pool_histories
            .entry(pool_id)
            .or_insert_with(Vec::new)
            .push(performance);
        
        // Keep only last 100 records
        let history = self.pool_histories.get_mut(&pool_id);
        if let Some(h) = history {
            if h.len() > 100 {
                h.remove(0);
            }
        }
    }

    /// Get average APR over a period
    pub fn get_average_apr(&self, pool_id: &str, periods: usize) -> f64 {
        self.pool_histories
            .get(pool_id)
            .map(|h| {
                let recent: Vec<_> = h.iter().rev().take(periods).collect();
                if recent.is_empty() {
                    0.0
                } else {
                    recent.iter().map(|p| p.avg_apr).sum::<f64>() / recent.len() as f64
                }
            })
            .unwrap_or(0.0)
    }
}

/// Stake tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeTier {
    pub name: String,
    pub min_amount: u64,
    pub max_amount: u64,
    pub bonus_multiplier: f64,
    pub perks: Vec<String>,
}

impl StakeTier {
    pub fn get_tiers() -> Vec<StakeTier> {
        vec![
            StakeTier {
                name: "Bronze".to_string(),
                min_amount: 100_000_000,          // 100 AETH
                max_amount: 999_999_999,
                bonus_multiplier: 1.0,
                perks: vec!["Basic rewards".to_string()],
            },
            StakeTier {
                name: "Silver".to_string(),
                min_amount: 1_000_000_000,        // 1,000 AETH
                max_amount: 9_999_999_999,
                bonus_multiplier: 1.15,           // 15% bonus
                perks: vec!["15% bonus rewards".to_string(), "Priority support".to_string()],
            },
            StakeTier {
                name: "Gold".to_string(),
                min_amount: 10_000_000_000,       // 10,000 AETH
                max_amount: 99_999_999_999,
                bonus_multiplier: 1.30,           // 30% bonus
                perks: vec!["30% bonus rewards".to_string(), "Governance voting".to_string(), "Early access features".to_string()],
            },
            StakeTier {
                name: "Platinum".to_string(),
                min_amount: 100_000_000_000,      // 100,000 AETH
                max_amount: u64::MAX,
                bonus_multiplier: 1.50,           // 50% bonus
                perks: vec!["50% bonus rewards".to_string(), "Full governance".to_string(), 
                    "Validator candidate".to_string(), "Exclusive events".to_string()],
            },
        ]
    }

    pub fn get_tier_for_amount(amount: u64) -> StakeTier {
        Self::get_tiers()
            .into_iter()
            .rev()
            .find(|t| amount >= t.min_amount)
            .unwrap_or_else(|| Self::get_tiers().first().unwrap().clone())
    }
}

/// Multi-pool staking manager
pub struct MultiPoolStakingManager {
    pub pools: HashMap<String, DynamicStakingPool>,
    pub apy_engine: DynamicAPYEngine,
}

impl MultiPoolStakingManager {
    pub fn new() -> Self {
        let mut manager = Self {
            pools: HashMap::new(),
            apy_engine: DynamicAPYEngine::new(APYConfig::default()),
        };
        
        // Initialize default pools
        manager.initialize_default_pools();
        manager
    }

    fn initialize_default_pools(&mut self) {
        // AETH Staking Pool
        self.pools.insert("aeth_staking".to_string(), DynamicStakingPool {
            pool_id: "aeth_staking".to_string(),
            name: "AETH Staking Pool".to_string(),
            base_apy: 0.12,
            min_stake: 100_000_000,        // 100 AETH
            max_stake: 1_000_000_000_000,  // 1M AETH
            current_apr: 0.12,
            dynamic_adjustment_enabled: true,
            adjustment_range: 0.03,        // +/- 3%
            target_utilization: 0.7,
            active_stakers: 0,
            total_staked: 0,
            lockup_epochs: 7,
        });

        // FLUX Rewards Pool
        self.pools.insert("flux_staking".to_string(), DynamicStakingPool {
            pool_id: "flux_staking".to_string(),
            name: "FLUX Utility Pool".to_string(),
            base_apy: 0.18,
            min_stake: 1_000_000_000,      // 1,000 FLUX
            max_stake: 100_000_000_000_000,
            current_apr: 0.18,
            dynamic_adjustment_enabled: true,
            adjustment_range: 0.05,
            target_utilization: 0.6,
            active_stakers: 0,
            total_staked: 0,
            lockup_epochs: 14,
        });

        // Validator Boost Pool
        self.pools.insert("validator_boost".to_string(), DynamicStakingPool {
            pool_id: "validator_boost".to_string(),
            name: "Validator Boost Pool".to_string(),
            base_apy: 0.15,
            min_stake: 10_000_000_000,     // 10,000 AETH
            max_stake: 500_000_000_000,    // 500K AETH
            current_apr: 0.15,
            dynamic_adjustment_enabled: true,
            adjustment_range: 0.04,
            target_utilization: 0.5,
            active_stakers: 0,
            total_staked: 0,
            lockup_epochs: 30,
        });
    }

    /// Adjust APR for a pool based on current conditions
    pub fn adjust_pool_apr(
        &mut self,
        pool_id: &str,
        utilization: &NetworkUtilization,
        pending_unstakes: u64,
        new_stake_requests: u64,
    ) -> Option<f64> {
        let pool = self.pools.get_mut(pool_id)?;
        
        if !pool.dynamic_adjustment_enabled {
            return Some(pool.current_apr);
        }
        
        let target_apr = self.apy_engine.calculate_adjusted_apr(
            pool,
            utilization,
            pending_unstakes,
            new_stake_requests,
        );
        
        // Apply smoothing
        let new_apr = self.apy_engine.smooth_apr_transition(pool.current_apr, target_apr);
        pool.current_apr = new_apr;
        
        Some(new_apr)
    }

    /// Get optimal pool for a given stake amount
    pub fn get_optimal_pool(&self, amount: u64) -> Option<String> {
        let tier = StakeTier::get_tier_for_amount(amount);
        
        // Find pools that meet minimum requirements
        let eligible_pools: Vec<_> = self.pools
            .values()
            .filter(|p| amount >= p.min_stake)
            .collect();
        
        // Return pool with highest current APR
        eligible_pools
            .into_iter()
            .max_by(|a, b| a.current_apr.partial_cmp(&b.current_apr).unwrap())
            .map(|p| p.pool_id.clone())
    }

    /// Calculate projected rewards across multiple pools
    pub fn calculate_multi_pool_projection(
        &self,
        amounts: &HashMap<String, u64>,
        epochs: u64,
        compound_frequency: u64,
    ) -> HashMap<String, u64> {
        let mut projections = HashMap::new();
        
        for (pool_id, amount) in amounts {
            if let Some(pool) = self.pools.get(pool_id) {
                let apy = pool.current_apr;
                let epoch_rate = apy / 8760.0; // Approximate
                
                let mut current = *amount as f64;
                for epoch in 0..epochs {
                    let reward = current * epoch_rate;
                    current += reward;
                    
                    if compound_frequency > 0 && (epoch + 1) % compound_frequency == 0 {
                        // Compound
                    }
                }
                
                projections.insert(pool_id.clone(), (current - *amount as f64) as u64);
            }
        }
        
        projections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demand_factor_calculation() {
        let engine = DynamicAPYEngine::new(APYConfig::default());
        
        let demand = engine.calculate_demand_factor(
            500_000_000_000, // 500K staked
            1_000_000_000_000, // 1M target
            10,
            20,
        );
        
        assert!(demand > 0.0);
        println!("Demand factor: {}", demand);
    }

    #[test]
    fn test_utilization_factor() {
        let engine = DynamicAPYEngine::new(APYConfig::default());
        
        // Under target utilization - should boost
        let factor_low = engine.calculate_utilization_factor(
            300_000_000_000, // 300K
            1_000_000_000_000, // 1M max
            0.7,
        );
        assert!(factor_low > 1.0);
        
        // Over target utilization - should reduce
        let factor_high = engine.calculate_utilization_factor(
            900_000_000_000,
            1_000_000_000_000,
            0.5,
        );
        assert!(factor_high < 1.0);
    }

    #[test]
    fn test_tier_system() {
        let tiers = StakeTier::get_tiers();
        assert_eq!(tiers.len(), 4);
        
        let bronze = StakeTier::get_tier_for_amount(500_000_000);
        assert_eq!(bronze.name, "Bronze");
        
        let platinum = StakeTier::get_tier_for_amount(200_000_000_000);
        assert_eq!(platinum.name, "Platinum");
    }

    #[test]
    fn test_multi_pool_manager() {
        let mut manager = MultiPoolStakingManager::new();
        
        let utilization = NetworkUtilization {
            active_validators: 50,
            total_stake: 500_000_000_000,
            target_stake: 1_000_000_000_000,
            utilization_ratio: 0.5,
            demand_factor: 1.0,
        };
        
        let new_apr = manager.adjust_pool_apr(
            "aeth_staking",
            &utilization,
            10,
            25,
        );
        
        assert!(new_apr.is_some());
        println!("Adjusted APR: {}", new_apr.unwrap());
    }
}