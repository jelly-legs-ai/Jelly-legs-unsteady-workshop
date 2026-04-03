// Staking Reward Calculation Logic - AeTHer Chain
// Calculates validator and delegator rewards based on stake, uptime, and network performance

use serde::{Deserialize, Serialize};

/// Staking reward configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingRewardConfig {
    pub base_apy: f64,           // Base annual percentage yield (e.g., 0.12 = 12%)
    pub uptime_bonus: f64,       // Bonus for 95%+ uptime (e.g., 0.03 = 3%)
    pub loyalty_multiplier: f64, // Multiplier for long-term stakers
    pub min_stake: u64,          // Minimum stake required (in AETH)
    pub max_stake: u64,          // Maximum stake per validator
    pub reward_distribution_interval: u64, // Epochs between distributions
    pub inflation_rate: f64,     // Annual inflation for rewards
    pub slash_penalty: f64,      // Penalty for misbehavior (e.g., 0.05 = 5%)
}

impl Default for StakingRewardConfig {
    fn default() -> Self {
        Self {
            base_apy: 0.12,              // 12% base APY
            uptime_bonus: 0.03,          // 3% bonus for excellent uptime
            loyalty_multiplier: 1.0,     // No multiplier by default
            min_stake: 100_000_000,      // 100 AETH (8 decimals)
            max_stake: 10_000_000_000,   // 10,000 AETH cap
            reward_distribution_interval: 10, // Every 10 epochs
            inflation_rate: 0.08,        // 8% annual inflation
            slash_penalty: 0.05,         // 5% slash for misbehavior
        }
    }
}

/// Validator stake information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStake {
    pub validator_id: String,
    pub staked_amount: u64,
    pub delegated_amount: u64,
    pub total_stake: u64,
    pub uptime_percentage: f64,
    pub epochs_active: u64,
    pub commission_rate: f64,      // Validator commission (e.g., 0.05 = 5%)
    pub last_reward_claim: u64,    // Epoch of last reward claim
    pub is_slashed: bool,
    pub slash_epoch: Option<u64>,
}

impl ValidatorStake {
    pub fn new(validator_id: String, staked_amount: u64) -> Self {
        Self {
            validator_id,
            staked_amount,
            delegated_amount: 0,
            total_stake: staked_amount,
            uptime_percentage: 100.0,
            epochs_active: 0,
            commission_rate: 0.05,
            last_reward_claim: 0,
            is_slashed: false,
            slash_epoch: None,
        }
    }
}

/// Delegator stake information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatorStake {
    pub delegator_id: String,
    pub validator_id: String,
    pub staked_amount: u64,
    pub epochs_staked: u64,
    pub rewards_earned: u64,
    pub pending_rewards: u64,
    pub is_claimed: bool,
}

impl DelegatorStake {
    pub fn new(delegator_id: String, validator_id: String, staked_amount: u64) -> Self {
        Self {
            delegator_id,
            validator_id,
            staked_amount,
            epochs_staked: 0,
            rewards_earned: 0,
            pending_rewards: 0,
            is_claimed: false,
        }
    }
}

/// Reward calculation engine
pub struct RewardCalculator {
    config: StakingRewardConfig,
}

impl RewardCalculator {
    pub fn new(config: StakingRewardConfig) -> Self {
        Self { config }
    }

    /// Calculate base reward for an epoch
    pub fn calculate_epoch_reward(&self, stake_amount: u64, epochs: u64) -> u64 {
        // Convert APY to per-epoch rate (assuming ~8760 epochs per year)
        let epochs_per_year = 8760;
        let epoch_rate = self.config.base_apy / epochs_per_year as f64;
        
        let base_reward = (stake_amount as f64 * epoch_rate) as u64;
        base_reward
    }

    /// Calculate uptime bonus
    pub fn calculate_uptime_bonus(&self, base_reward: u64, uptime: f64) -> u64 {
        if uptime >= 95.0 {
            (base_reward as f64 * self.config.uptime_bonus) as u64
        } else {
            0
        }
    }

    /// Calculate loyalty multiplier based on staking duration
    pub fn calculate_oyalty_multiplier(&self, epochs_staked: u64) -> f64 {
        // 1.0 for <100 epochs, 1.05 for 100-500, 1.1 for 500-1000, 1.15 for 1000+
        if epochs_staked < 100 {
            1.0
        } else if epochs_staked < 500 {
            1.05
        } else if epochs_staked < 1000 {
            1.1
        } else {
            1.15
        }
    }

    /// Calculate total validator reward
    pub fn calculate_validator_reward(
        &self,
        validator: &ValidatorStake,
        epochs_since_last_claim: u64,
    ) -> u64 {
        if validator.is_slashed {
            return 0;
        }

        let base_reward = self.calculate_epoch_reward(validator.total_stake, epochs_since_last_claim);
        let uptime_bonus = self.calculate_uptime_bonus(base_reward, validator.uptime_percentage);
        let loyalty_mult = self.calculate_loyalty_multiplier(validator.epochs_active);
        
        let total_reward = ((base_reward + uptime_bonus) as f64 * loyalty_mult) as u64;
        
        // Apply commission for validator's share
        let validator_share = (total_reward as f64 * (1.0 - validator.commission_rate)) as u64;
        
        validator_share
    }

    /// Calculate delegator reward (after validator commission)
    pub fn calculate_delegator_reward(
        &self,
        delegator: &DelegatorStake,
        validator: &ValidatorStake,
        epochs_since_last_claim: u64,
    ) -> u64 {
        if validator.is_slashed {
            return 0;
        }

        let base_reward = self.calculate_epoch_reward(delegator.staked_amount, epochs_since_last_claim);
        let uptime_bonus = self.calculate_uptime_bonus(base_reward, validator.uptime_percentage);
        let loyalty_mult = self.calculate_loyalty_multiplier(delegator.epochs_staked);
        
        let gross_reward = ((base_reward + uptime_bonus) as f64 * loyalty_mult) as u64;
        
        // Validator takes commission, delegator gets remainder
        let delegator_share = (gross_reward as f64 * (1.0 - validator.commission_rate)) as u64;
        
        delegator_share
    }

    /// Apply slash penalty for misbehavior
    pub fn apply_slash(&self, validator: &mut ValidatorStake, reason: &str) -> u64 {
        validator.is_slashed = true;
        validator.slash_epoch = Some(validator.epochs_active);
        
        let slash_amount = (validator.total_stake as f64 * self.config.slash_penalty) as u64;
        validator.total_stake -= slash_amount;
        validator.staked_amount -= slash_amount;
        
        slash_amount
    }

    /// Calculate APY based on current parameters
    pub fn calculate_effective_apy(&self, uptime: f64, epochs_staked: u64) -> f64 {
        let base = self.config.base_apy;
        let uptime_bonus = if uptime >= 95.0 { self.config.uptime_bonus } else { 0.0 };
        let loyalty_mult = self.calculate_loyalty_multiplier(epochs_staked);
        
        (base + uptime_bonus) * loyalty_mult
    }

    /// Calculate projected rewards over a number of epochs with compounding
    pub fn calculate_projected_rewards(
        &self,
        stake_amount: u64,
        epochs: u64,
        compound_frequency: u64,
        uptime: f64,
        epochs_staked: u64,
    ) -> u64 {
        if stake_amount == 0 || epochs == 0 {
            return 0;
        }

        let mut current_stake = stake_amount as f64;
        let apy = self.calculate_effective_apy(uptime, epochs_staked);
        let epoch_rate = apy / (365 * 288); // ~288 epochs per day

        for epoch in 0..epochs {
            // Calculate reward for this epoch
            let epoch_reward = current_stake * epoch_rate;
            current_stake += epoch_reward;

            // Compound at specified frequency
            if compound_frequency > 0 && (epoch + 1) % compound_frequency == 0 {
                // Rewards already added to stake, nothing to do
            }
        }

        (current_stake - stake_amount as f64) as u64
    }

    /// Find optimal compound frequency to maximize rewards
    pub fn find_optimal_compound_frequency(
        &self,
        stake_amount: u64,
        total_epochs: u64,
        min_frequency: u64,
        max_frequency: u64,
        uptime: f64,
        epochs_staked: u64,
    ) -> (u64, u64) {
        let mut best_frequency = max_frequency;
        let mut best_rewards = 0;

        for freq in (min_frequency..=max_frequency).step_by(1) {
            let rewards = self.calculate_projected_rewards(
                stake_amount,
                total_epochs,
                freq,
                uptime,
                epochs_staked,
            );
            if rewards > best_rewards {
                best_rewards = rewards;
                best_frequency = freq;
            }
        }

        (best_frequency, best_rewards)
    }

    /// Calculate break-even epoch for auto-compound vs manual claim
    pub fn calculate_auto_compound_breakeven(
        &self,
        stake_amount: u64,
        claim_frequency: u64,
        compound_fee_percent: f64,
        uptime: f64,
        epochs_staked: u64,
    ) -> u64 {
        // With auto-compound, rewards compound continuously (frequency = 1)
        // vs manual claiming at claim_frequency
        // The fee is paid on each compound

        let apy = self.calculate_effective_apy(uptime, epochs_staked);
        let epoch_rate = apy / (365 * 288);

        // For manual: claim and don't reinvest until claim_frequency
        let mut manual_stake = stake_amount as f64;
        let mut auto_stake = stake_amount as f64;

        let mut epoch = 0;
        while manual_stake < auto_stake * (1.0 - compound_fee_percent / 100.0) && epoch < 365 * 288 {
            let manual_reward = if epoch > 0 && epoch % claim_frequency == 0 {
                manual_stake * epoch_rate
            } else {
                0.0
            };

            let auto_reward = auto_stake * epoch_rate * (1.0 - compound_fee_percent / 100.0);

            manual_stake += manual_reward;
            auto_stake += auto_reward;
            epoch += 1;
        }

        epoch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_reward_calculation() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        // 100 AETH stake, 1 epoch
        let reward = calculator.calculate_epoch_reward(100_000_000, 1);
        assert!(reward > 0);
        println!("Base reward for 100 AETH: {}", reward);
    }

    #[test]
    fn test_uptime_bonus() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        let base_reward = 1000;
        
        // 96% uptime should get bonus
        let bonus_96 = calculator.calculate_uptime_bonus(base_reward, 96.0);
        assert!(bonus_96 > 0);
        
        // 94% uptime should get no bonus
        let bonus_94 = calculator.calculate_uptime_bonus(base_reward, 94.0);
        assert_eq!(bonus_94, 0);
    }

    #[test]
    fn test_loyalty_multiplier() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        assert_eq!(calculator.calculate_loyalty_multiplier(50), 1.0);
        assert_eq!(calculator.calculate_loyalty_multiplier(200), 1.05);
        assert_eq!(calculator.calculate_loyalty_multiplier(700), 1.1);
        assert_eq!(calculator.calculate_loyalty_multiplier(1500), 1.15);
    }

    #[test]
    fn test_effective_apy() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        // Max APY with 95%+ uptime and 1000+ epochs
        let max_apy = calculator.calculate_effective_apy(96.0, 1500);
        assert!((max_apy - 0.165).abs() < 0.001); // ~16.5% APY
        
        // Base APY with low uptime and new staker
        let base_apy = calculator.calculate_effective_apy(90.0, 50);
        assert!((base_apy - 0.12).abs() < 0.001); // 12% base
    }
}
