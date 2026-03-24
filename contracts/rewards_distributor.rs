// Rewards Distributor Contract - AeTHer Chain
// Automated reward distribution across miners, validators, and stakers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reward type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardType {
    MiningReward,
    StakingReward,
    ValidatorReward,
    DelegatorReward,
    GovernanceReward,
    ReferralBonus,
    LoyaltyBonus,
    EpochBonus,
}

/// Reward distribution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retried,
}

/// Individual reward record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRecord {
    pub id: String,
    pub recipient_address: String,
    pub reward_type: RewardType,
    pub amount: u64,
    pub token_type: String, // AETH, FLUX, ATH
    pub epoch: u64,
    pub calculated_at: u64,
    pub distributed_at: Option<u64>,
    pub status: DistributionStatus,
    pub transaction_hash: Option<String>,
    pub retry_count: u64,
    pub failure_reason: Option<String>,
}

/// Reward calculation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardCalculationParams {
    pub base_reward_rate: f64,
    pub device_tier_multiplier: f64,
    pub uptime_multiplier: f64,
    pub reputation_multiplier: f64,
    pub network_difficulty_factor: f64,
    pub epoch_total_budget: u64,
    pub min_reward_threshold: u64,
    pub max_reward_cap: u64,
}

/// Batch distribution job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionBatch {
    pub batch_id: String,
    pub epoch: u64,
    pub total_rewards: u64,
    pub recipient_count: u64,
    pub created_at: u64,
    pub processed_at: Option<u64>,
    pub status: DistributionStatus,
    pub reward_records: Vec<RewardRecord>,
    pub gas_estimate: u64,
    pub actual_gas_used: Option<u64>,
}

/// Reward distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub epoch: u64,
    pub total_distributed: u64,
    pub total_pending: u64,
    pub total_failed: u64,
    pub successful_count: u64,
    pub failed_count: u64,
    pub average_reward: f64,
    pub median_reward: u64,
    pub distribution_time_ms: u64,
}

/// Tier-based reward breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRewardBreakdown {
    pub mobile_total: u64,
    pub mobile_count: u64,
    pub laptop_total: u64,
    pub laptop_count: u64,
    pub desktop_total: u64,
    pub desktop_count: u64,
    pub server_total: u64,
    pub server_count: u64,
    pub average_per_tier: HashMap<String, f64>,
}

/// Rewards distributor contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsDistributor {
    pub pending_rewards: HashMap<String, Vec<RewardRecord>>,
    pub completed_distributions: Vec<DistributionBatch>,
    pub failed_distributions: Vec<RewardRecord>,
    pub calculation_params: RewardCalculationParams,
    pub current_epoch: u64,
    pub total_lifetime_distributed: u64,
    pub total_recipients: u64,
    pub distribution_history: HashMap<u64, DistributionStats>,
    pub auto_retry_enabled: bool,
    pub max_retry_attempts: u64,
    pub batch_size_limit: u64,
}

impl RewardsDistributor {
    /// Create new rewards distributor
    pub fn new() -> Self {
        let calculation_params = RewardCalculationParams {
            base_reward_rate: 0.001,
            device_tier_multiplier: 1.0,
            uptime_multiplier: 1.0,
            reputation_multiplier: 1.0,
            network_difficulty_factor: 1.0,
            epoch_total_budget: 1000000,
            min_reward_threshold: 1,
            max_reward_cap: 10000,
        };

        RewardsDistributor {
            pending_rewards: HashMap::new(),
            completed_distributions: Vec::new(),
            failed_distributions: Vec::new(),
            calculation_params,
            current_epoch: 0,
            total_lifetime_distributed: 0,
            total_recipients: 0,
            distribution_history: HashMap::new(),
            auto_retry_enabled: true,
            max_retry_attempts: 3,
            batch_size_limit: 1000,
        }
    }

    /// Calculate mining reward based on multiple factors
    pub fn calculate_mining_reward(
        &self,
        base_amount: u64,
        device_tier: u8,
        uptime_score: f64,
        reputation_score: f64,
        network_factor: f64,
    ) -> u64 {
        let tier_multiplier = match device_tier {
            1 => 1.0,   // Mobile
            2 => 2.5,   // Laptop
            3 => 4.0,   // Desktop
            4 => 8.0,   // Server
            _ => 1.0,
        };

        let calculated = base_amount as f64
            * self.calculation_params.base_reward_rate
            * tier_multiplier
            * uptime_score
            * reputation_score
            * network_factor;

        let reward = calculated as u64;

        // Apply caps and thresholds
        if reward < self.calculation_params.min_reward_threshold {
            0
        } else if reward > self.calculation_params.max_reward_cap {
            self.calculation_params.max_reward_cap
        } else {
            reward
        }
    }

    /// Calculate staking reward with APY
    pub fn calculate_staking_reward(
        &self,
        staked_amount: u64,
        apy: f64,
        epochs_staked: u64,
    ) -> u64 {
        let epoch_rate = apy / 365.0; // Daily rate
        let reward = (staked_amount as f64) * epoch_rate * (epochs_staked as f64);
        reward as u64
    }

    /// Calculate validator reward with commission
    pub fn calculate_validator_reward(
        &self,
        total_rewards: u64,
        commission_rate: f64,
        uptime_percent: f64,
    ) -> u64 {
        let base_reward = (total_rewards as f64) * commission_rate;
        let adjusted_reward = base_reward * (uptime_percent / 100.0);
        adjusted_reward as u64
    }

    /// Queue reward for distribution
    pub fn queue_reward(
        &mut self,
        recipient: String,
        reward_type: RewardType,
        amount: u64,
        token_type: String,
    ) -> String {
        let record_id = format!("reward_{}_{}", self.current_epoch, uuid::simple());
        let record = RewardRecord {
            id: record_id.clone(),
            recipient_address: recipient,
            reward_type,
            amount,
            token_type,
            epoch: self.current_epoch,
            calculated_at: self.get_timestamp(),
            distributed_at: None,
            status: DistributionStatus::Pending,
            transaction_hash: None,
            retry_count: 0,
            failure_reason: None,
        };

        self.pending_rewards
            .entry(record.recipient_address.clone())
            .or_insert_with(Vec::new)
            .push(record);

        record_id
    }

    /// Process batch distribution
    pub fn process_batch(&mut self, epoch: u64) -> DistributionBatch {
        let batch_id = format!("batch_{}_{}", epoch, self.get_timestamp());
        let mut batch_rewards = Vec::new();
        let mut total_amount = 0u64;

        // Collect all pending rewards for this epoch
        for (address, rewards) in &mut self.pending_rewards {
            for reward in rewards.iter_mut() {
                if reward.epoch == epoch && reward.status == DistributionStatus::Pending {
                    reward.status = DistributionStatus::Processing;
                    batch_rewards.push(reward.clone());
                    total_amount += reward.amount;
                }
            }
        }

        let batch = DistributionBatch {
            batch_id: batch_id.clone(),
            epoch,
            total_rewards: total_amount,
            recipient_count: batch_rewards.len() as u64,
            created_at: self.get_timestamp(),
            processed_at: None,
            status: DistributionStatus::Processing,
            reward_records: batch_rewards,
            gas_estimate: self.estimate_gas(batch_rewards.len()),
            actual_gas_used: None,
        };

        batch
    }

    /// Mark batch as completed
    pub fn complete_batch(&mut self, batch_id: &str, tx_hash: String) {
        for batch in &mut self.completed_distributions {
            if &batch.batch_id == batch_id {
                batch.status = DistributionStatus::Completed;
                batch.processed_at = Some(self.get_timestamp());
                for record in &mut batch.reward_records {
                    record.status = DistributionStatus::Completed;
                    record.distributed_at = Some(self.get_timestamp());
                    record.transaction_hash = Some(tx_hash.clone());
                }
                self.total_lifetime_distributed += batch.total_rewards;
                break;
            }
        }
    }

    /// Handle distribution failure
    pub fn handle_failure(&mut self, record_id: &str, reason: String) {
        for (address, rewards) in &mut self.pending_rewards {
            for reward in rewards.iter_mut() {
                if &reward.id == record_id {
                    reward.status = DistributionStatus::Failed;
                    reward.failure_reason = Some(reason.clone());
                    reward.retry_count += 1;

                    if self.auto_retry_enabled && reward.retry_count < self.max_retry_attempts {
                        reward.status = DistributionStatus::Retried;
                    } else {
                        self.failed_distributions.push(reward.clone());
                    }
                    break;
                }
            }
        }
    }

    /// Get distribution stats for epoch
    pub fn get_epoch_stats(&self, epoch: u64) -> DistributionStats {
        let mut total_distributed = 0u64;
        let mut total_pending = 0u64;
        let mut total_failed = 0u64;
        let mut successful_count = 0u64;
        let mut failed_count = 0u64;
        let mut rewards = Vec::new();

        for batch in &self.completed_distributions {
            if batch.epoch == epoch {
                total_distributed += batch.total_rewards;
                successful_count += batch.recipient_count;
                for record in &batch.reward_records {
                    rewards.push(record.amount);
                }
            }
        }

        for (_, address_rewards) in &self.pending_rewards {
            for reward in address_rewards {
                if reward.epoch == epoch {
                    if reward.status == DistributionStatus::Failed {
                        total_failed += reward.amount;
                        failed_count += 1;
                    } else {
                        total_pending += reward.amount;
                    }
                }
            }
        }

        let avg = if !rewards.is_empty() {
            rewards.iter().sum::<u64>() as f64 / rewards.len() as f64
        } else {
            0.0
        };

        rewards.sort();
        let median = if rewards.is_empty() {
            0
        } else {
            rewards[rewards.len() / 2]
        };

        DistributionStats {
            epoch,
            total_distributed,
            total_pending,
            total_failed,
            successful_count,
            failed_count,
            average_reward: avg,
            median_reward: median,
            distribution_time_ms: 0,
        }
    }

    /// Get tier-based reward breakdown
    pub fn get_tier_breakdown(&self, epoch: u64) -> TierRewardBreakdown {
        let mut breakdown = TierRewardBreakdown {
            mobile_total: 0,
            mobile_count: 0,
            laptop_total: 0,
            laptop_count: 0,
            desktop_total: 0,
            desktop_count: 0,
            server_total: 0,
            server_count: 0,
            average_per_tier: HashMap::new(),
        };

        // This would integrate with mining contract to get tier data
        // For now, placeholder structure
        breakdown
    }

    /// Estimate gas for batch
    fn estimate_gas(&self, recipient_count: usize) -> u64 {
        // Base gas + per-recipient cost
        let base_gas = 21000u64;
        let per_recipient = 5000u64;
        base_gas + (recipient_count as u64 * per_recipient)
    }

    /// Get current timestamp (placeholder)
    fn get_timestamp(&self) -> u64 {
        // In production, use actual timestamp
        self.current_epoch * 1000
    }
}

/// UUID simple generator (placeholder for production)
mod uuid {
    pub fn simple() -> String {
        format!("{}", rand::random::<u64>())
    }
}

mod rand {
    pub fn random<T>() -> T {
        // Placeholder - use actual random crate in production
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_reward_calculation() {
        let distributor = RewardsDistributor::new();
        
        // Mobile miner with good uptime and reputation
        let reward = distributor.calculate_mining_reward(
            1000,
            1,  // Mobile
            0.95, // 95% uptime
            0.90, // Good reputation
            1.0,  // Normal difficulty
        );
        
        assert!(reward > 0);
        assert!(reward < 1000);
    }

    #[test]
    fn test_staking_reward_calculation() {
        let distributor = RewardsDistributor::new();
        
        let reward = distributor.calculate_staking_reward(
            10000,
            0.15, // 15% APY
            7,    // 7 epochs
        );
        
        assert!(reward > 0);
    }

    #[test]
    fn test_reward_queue() {
        let mut distributor = RewardsDistributor::new();
        
        let record_id = distributor.queue_reward(
            "addr123".to_string(),
            RewardType::MiningReward,
            500,
            "FLUX".to_string(),
        );
        
        assert!(!record_id.is_empty());
        assert!(distributor.pending_rewards.contains_key("addr123"));
    }
}
