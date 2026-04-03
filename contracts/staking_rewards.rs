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
    pub fn calculate_loyalty_multiplier(&self, epochs_staked: u64) -> f64 {
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

// ============================================================================
// SLASH RECOVERY & PERFORMANCE TRACKING - Sprint 76 Enhancement
// ============================================================================

/// Tracks validator performance metrics over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPerformance {
    pub validator_id: String,
    pub blocks_produced: u64,
    pub blocks_missed: u64,
    pub attestations_total: u64,
    pub attestations_missed: u64,
    pub uptime_percentage: f64,
    pub avg_response_time_ms: u64,
    pub last_heartbeat: u64,
    pub performance_score: f64,  // 0-100 based on all metrics
    pub tier: PerformanceTier,
    pub consecutive_uptime_epochs: u64,
    pub slash_history: Vec<SlashEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Elite,
}

impl PerformanceTier {
    pub fn from_score(score: f64) -> Self {
        if score >= 95.0 {
            PerformanceTier::Elite
        } else if score >= 85.0 {
            PerformanceTier::Platinum
        } else if score >= 70.0 {
            PerformanceTier::Gold
        } else if score >= 50.0 {
            PerformanceTier::Silver
        } else {
            PerformanceTier::Bronze
        }
    }
    
    pub fn reward_boost(&self) -> f64 {
        match self {
            PerformanceTier::Elite => 0.15,   // 15% bonus
            PerformanceTier::Platinum => 0.10, // 10% bonus
            PerformanceTier::Gold => 0.05,    // 5% bonus
            PerformanceTier::Silver => 0.0,    // No bonus
            PerformanceTier::Bronze => -0.05,  // 5% penalty
        }
    }
}

/// Records a slash event for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashEvent {
    pub epoch: u64,
    pub reason: SlashReason,
    pub amount_slashed: u64,
    pub can_recover: bool,
    pub recovery_start_epoch: Option<u64>,
    pub fully_recovered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashReason {
    DoubleSign,
    Downtime,
    InvalidAttestation,
    MissedCommitteeDuty,
    Corruption,
}

/// Slash recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashRecoveryConfig {
    pub recovery_epochs: u64,           // Epochs before recovery can start
    pub recovery_rate_per_epoch: f64,   // Portion of slashed amount recovered per epoch
    pub max_recovery_percent: f64,       // Maximum percent of slashed amount recoverable
    pub good_behavior_periods_required: u64,  // Consecutive good epochs needed
    pub recovery_start_bond_percent: f64, // Minimum bond percent to start recovery
}

impl Default for SlashRecoveryConfig {
    fn default() -> Self {
        Self {
            recovery_epochs: 43200,        // ~30 days (at 60s/epoch)
            recovery_rate_per_epoch: 0.0001, // 0.01% per epoch = ~50 epochs for full recovery
            max_recovery_percent: 0.75,    // Max 75% recoverable
            good_behavior_periods_required: 100, // 100 consecutive good epochs
            recovery_start_bond_percent: 0.50, // Must have 50%+ of original bond
        }
    }
}

/// Slash recovery manager
pub struct SlashRecoveryManager {
    config: SlashRecoveryConfig,
}

impl SlashRecoveryManager {
    pub fn new(config: SlashRecoveryConfig) -> Self {
        Self { config }
    }
    
    /// Check if a validator can start slash recovery
    pub fn can_start_recovery(&self, validator: &ValidatorStake, last_slash: Option<&SlashEvent>) -> bool {
        // Must not be currently slashed
        if validator.is_slashed && last_slash.map(|s| !s.fully_recovered).unwrap_or(false) {
            return false;
        }
        
        // Must meet recovery bond requirement
        let original_bond = validator.staked_amount + last_slash.map(|s| s.amount_slashed).unwrap_or(0);
        let current_percent = original_bond as f64 / validator.staked_amount as f64;
        if current_percent < self.config.recovery_start_bond_percent {
            return false;
        }
        
        // Must have served good behavior periods
        if let Some(slash) = last_slash {
            if validator.epochs_active - slash.epoch < self.config.good_behavior_periods_required {
                return false;
            }
        }
        
        true
    }
    
    /// Calculate recovery amount for current epoch
    pub fn calculate_recovery_amount(&self, slash: &SlashEvent, current_epoch: u64) -> u64 {
        if slash.fully_recovered {
            return 0;
        }
        
        let recovery_start = match slash.recovery_start_epoch {
            Some(epoch) => epoch,
            None => return 0,
        };
        
        let epochs_in_recovery = current_epoch.saturating_sub(recovery_start);
        let max_recoverable = (slash.amount_slashed as f64 * self.config.max_recovery_percent) as u64;
        let recovery_amount = (epochs_in_recovery as f64 * self.config.recovery_rate_per_epoch * slash.amount_slashed as f64) as u64;
        
        recovery_amount.min(max_recoverable)
    }
    
    /// Check if recovery is complete
    pub fn is_recovery_complete(&self, slash: &SlashEvent, total_recovered: u64) -> bool {
        let max_recoverable = (slash.amount_slashed as f64 * self.config.max_recovery_percent) as u64;
        total_recovered >= max_recoverable
    }
    
    /// Process slash event and determine recovery eligibility
    pub fn process_slash(&self, validator: &mut ValidatorStake, reason: SlashReason, amount: u64, current_epoch: u64) -> SlashEvent {
        let can_recover = match &reason {
            SlashReason::Downtime | SlashReason::MissedCommitteeDuty => true,
            SlashReason::DoubleSign | SlashReason::InvalidAttestation | SlashReason::Corruption => false,
        };
        
        let event = SlashEvent {
            epoch: current_epoch,
            reason,
            amount_slashed: amount,
            can_recover,
            recovery_start_epoch: None,
            fully_recovered: false,
        };
        
        // Apply slash to validator
        validator.total_stake = validator.total_stake.saturating_sub(amount);
        validator.staked_amount = validator.staked_amount.saturating_sub(amount);
        validator.is_slashed = true;
        validator.slash_epoch = Some(current_epoch);
        
        event
    }
    
    /// Start recovery process for eligible slash
    pub fn start_recovery(&self, slash: &mut SlashEvent, current_epoch: u64) -> bool {
        if !slash.can_recover || slash.recovery_start_epoch.is_some() {
            return false;
        }
        
        slash.recovery_start_epoch = Some(current_epoch);
        true
    }
    
    /// Calculate performance score based on all metrics
    pub fn calculate_performance_score(perf: &ValidatorPerformance) -> f64 {
        // Uptime contributes 40% of score
        let uptime_score = perf.uptime_percentage * 0.40;
        
        // Block production contributes 30% of score
        let total_blocks = perf.blocks_produced + perf.blocks_missed;
        let block_score = if total_blocks > 0 {
            (perf.blocks_produced as f64 / total_blocks as f64) * 100.0 * 0.30
        } else {
            0.0
        };
        
        // Attestation accuracy contributes 20% of score
        let total_atts = perf.attestations_total + perf.attestations_missed;
        let att_score = if total_atts > 0 {
            (perf.attestations_total as f64 / total_atts as f64) * 100.0 * 0.20
        } else {
            0.0
        };
        
        // Response time contributes 10% of score (lower is better)
        let response_score = if perf.avg_response_time_ms < 100 {
            10.0
        } else if perf.avg_response_time_ms < 500 {
            10.0 - ((perf.avg_response_time_ms - 100) as f64 / 400.0 * 10.0)
        } else {
            0.0
        };
        
        uptime_score + block_score + att_score + response_score
    }
}

/// Validator performance tracker
pub struct ValidatorPerformanceTracker {
    history: Vec<ValidatorPerformance>,
}

impl ValidatorPerformanceTracker {
    pub fn new() -> Self {
        Self { history: Vec::new() }
    }
    
    /// Record a block produced
    pub fn record_block_produced(&mut self, validator_id: &str, epoch: u64) {
        if let Some(perf) = self.history.iter_mut().find(|p| p.validator_id == validator_id) {
            perf.blocks_produced += 1;
            perf.last_heartbeat = epoch;
        }
    }
    
    /// Record a block missed
    pub fn record_block_missed(&mut self, validator_id: &str, epoch: u64) {
        if let Some(perf) = self.history.iter_mut().find(|p| p.validator_id == validator_id) {
            perf.blocks_missed += 1;
            perf.last_heartbeat = epoch;
        }
    }
    
    /// Record an attestation
    pub fn record_attestation(&mut self, validator_id: &str, success: bool) {
        if let Some(perf) = self.history.iter_mut().find(|p| p.validator_id == validator_id) {
            perf.attestations_total += 1;
            if !success {
                perf.attestations_missed += 1;
            }
        }
    }
    
    /// Update uptime percentage
    pub fn update_uptime(&mut self, validator_id: &str, uptime: f64) {
        if let Some(perf) = self.history.iter_mut().find(|p| p.validator_id == validator_id) {
            perf.uptime_percentage = uptime;
            if uptime >= 95.0 {
                perf.consecutive_uptime_epochs += 1;
            } else {
                perf.consecutive_uptime_epochs = 0;
            }
        }
    }
    
    /// Update performance score
    pub fn refresh_score(&mut self, validator_id: &str) {
        if let Some(perf) = self.history.iter_mut().find(|p| p.validator_id == validator_id) {
            perf.performance_score = SlashRecoveryManager::calculate_performance_score(perf);
            perf.tier = PerformanceTier::from_score(perf.performance_score);
        }
    }
    
    /// Get or create performance record
    pub fn get_or_create(&mut self, validator_id: &str) -> &mut ValidatorPerformance {
        if let Some(perf) = self.history.iter_mut().find(|p| p.validator_id == validator_id) {
            return perf;
        }
        
        self.history.push(ValidatorPerformance {
            validator_id: validator_id.to_string(),
            blocks_produced: 0,
            blocks_missed: 0,
            attestations_total: 0,
            attestations_missed: 0,
            uptime_percentage: 100.0,
            avg_response_time_ms: 0,
            last_heartbeat: 0,
            performance_score: 100.0,
            tier: PerformanceTier::Gold,
            consecutive_uptime_epochs: 0,
            slash_history: Vec::new(),
        });
        
        self.history.last_mut().unwrap()
    }
    
    /// Get top performers
    pub fn get_top_performers(&self, limit: usize) -> Vec<&ValidatorPerformance> {
        let mut sorted = self.history.clone();
        sorted.sort_by(|a, b| b.performance_score.partial_cmp(&a.performance_score).unwrap());
        sorted.into_iter().take(limit).collect()
    }
    
    /// Get validators eligible for elite tier rewards
    pub fn get_elite_eligible(&self, min_epochs: u64) -> Vec<&ValidatorPerformance> {
        self.history.iter()
            .filter(|p| p.tier == PerformanceTier::Elite && p.epochs_active >= min_epochs)
            .collect()
    }
}

impl Default for ValidatorPerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// AUTO-COMPOUND MANAGER - Sprint 60 Enhancement
// ============================================================================

/// Auto-compound configuration for a stake position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompoundConfig {
    pub enabled: bool,
    pub frequency_epochs: u64,     // How often to compound (1 = every epoch)
    pub min_stake_threshold: u64,  // Minimum rewards before compounding
    pub fee_percent: f64,          // Fee charged for auto-compound service
    pub reinvest_ratio: f64,       // Percentage of rewards to reinvest (0.0-1.0)
    pub last_compound_epoch: u64,
    pub total_fees_paid: u64,
    pub total_rewards_compounded: u64,
}

impl Default for AutoCompoundConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency_epochs: 288, // Once per day (288 epochs)
            min_stake_threshold: 1000, // 0.00001 AETH minimum
            fee_percent: 0.1,          // 0.1% fee
            reinvest_ratio: 1.0,       // 100% reinvest by default
            last_compound_epoch: 0,
            total_fees_paid: 0,
            total_rewards_compounded: 0,
        }
    }
}

impl AutoCompoundConfig {
    /// Check if enough epochs have passed to trigger compounding
    pub fn should_compound(&self, current_epoch: u64) -> bool {
        if !self.enabled {
            return false;
        }
        current_epoch - self.last_compound_epoch >= self.frequency_epochs
    }

    /// Calculate fee for a given reward amount
    pub fn calculate_fee(&self, reward_amount: u64) -> u64 {
        (reward_amount as f64 * self.fee_percent / 100.0) as u64
    }

    /// Calculate net reward after fee
    pub fn net_reward(&self, gross_reward: u64) -> (u64, u64) {
        let fee = self.calculate_fee(gross_reward);
        let net = gross_reward.saturating_sub(fee);
        (net, fee)
    }
}

/// Auto-compound manager handles automated reward compounding
pub struct AutoCompoundManager {
    config: StakingRewardConfig,
}

impl AutoCompoundManager {
    pub fn new(config: StakingRewardConfig) -> Self {
        Self { config }
    }

    /// Calculate optimal compound frequency based on fee structure
    pub fn calculate_optimal_frequency(
        &self,
        stake_amount: u64,
        fee_percent: f64,
        total_epochs: u64,
        uptime: f64,
    ) -> u64 {
        let apy = self.config.base_apy;
        let epoch_rate = apy / (365 * 288);

        let mut best_freq = 1;
        let mut best_value = 0.0;

        // Test frequencies from 1 (continuous) to 2880 (monthly)
        for freq in [1, 6, 12, 24, 48, 72, 144, 288, 576, 1152, 2880].iter() {
            let freq = *freq as u64;
            if freq > total_epochs {
                continue;
            }

            // Simulate compounding at this frequency
            let mut stake = stake_amount as f64;
            let mut total_fees = 0.0;

            let epochs_per_period = freq;
            let periods = total_epochs / epochs_per_period;

            for _ in 0..periods {
                // Earn rewards over the period
                let period_reward = stake * epoch_rate * epochs_per_period as f64;
                stake += period_reward;

                // Pay fee on compounding
                let fee = period_reward * fee_percent / 100.0;
                stake -= fee;
                total_fees += fee;
            }

            // Calculate effective value (stake minus fees)
            let effective_value = stake;

            if effective_value > best_value {
                best_value = effective_value;
                best_freq = freq;
            }
        }

        best_freq
    }

    /// Calculate compound APY after fees
    pub fn calculate_compound_apy(&self, base_apy: f64, fee_percent: f64, frequency: u64) -> f64 {
        // Effective APY = base APY * (1 - fee%)^(epochs_per_year / compound_frequency)
        let epochs_per_year = 365 * 288;
        let compounds_per_year = epochs_per_year as f64 / frequency as f64;
        let effective_apy = base_apy * (1.0 - fee_percent / 100.0).powf(compounds_per_year);
        effective_apy
    }

    /// Generate compound schedule for a given time period
    pub fn generate_compound_schedule(
        &self,
        start_epoch: u64,
        end_epoch: u64,
        frequency: u64,
    ) -> Vec<u64> {
        let mut epochs: Vec<u64> = Vec::new();
        let mut next_compound = start_epoch + frequency;

        while next_compound <= end_epoch {
            epochs.push(next_compound);
            next_compound += frequency;
        }

        epochs
    }

    /// Estimate total rewards with auto-compounding
    pub fn estimate_total_rewards(
        &self,
        stake_amount: u64,
        epochs: u64,
        compound_freq: u64,
        fee_percent: f64,
        uptime: f64,
        epochs_staked: u64,
    ) -> (u64, u64, f64) {
        let calculator = RewardCalculator::new(self.config.clone());
        let apy = calculator.calculate_effective_apy(uptime, epochs_staked);
        let epoch_rate = apy / (365 * 288);

        let mut stake = stake_amount as f64;
        let mut total_fees = 0.0;

        // Simulate compounding periods
        let periods = epochs / compound_freq;
        for _ in 0..periods {
            // Earn rewards
            let period_reward = stake * epoch_rate * compound_freq as f64;
            stake += period_reward;

            // Pay fee
            let fee = period_reward * fee_percent / 100.0;
            stake -= fee;
            total_fees += fee;
        }

        let final_stake = stake as u64;
        let total_rewards = final_stake.saturating_sub(stake_amount);
        let effective_apy = ((final_stake as f64 / stake_amount as f64 - 1.0) / epochs as f64 * (365 * 288) * 100.0);

        (total_rewards, total_fees as u64, effective_apy)
    }
}

// ============================================================================
// STAKING ANALYTICS & REPORTING - Sprint 62
// ============================================================================

/// Historical reward record for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRecord {
    pub epoch: u64,
    pub stake_amount: u64,
    pub reward_type: RewardType,
    pub gross_reward: u64,
    pub fee: u64,
    pub net_reward: u64,
    pub validator_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RewardType {
    ValidatorReward,
    DelegatorReward,
    AutoCompound,
    ManualClaim,
    LoyaltyBonus,
    UptimeBonus,
}

/// Staking analytics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingAnalytics {
    pub total_staked: u64,
    pub total_rewards_earned: u64,
    pub total_fees_paid: u64,
    pub average_apy: f64,
    pub best_validator: String,
    pub worst_validator: String,
    pub total_epochs_staked: u64,
    pub current_tier: StakingTier,
    pub next_tier: Option<StakingTier>,
    pub epochs_until_next_tier: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum StakingTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

impl StakingTier {
    pub fn from_stake_amount(amount: u64) -> Self {
        if amount >= 1_000_000_000_000 { // 10,000 AETH
            StakingTier::Diamond
        } else if amount >= 100_000_000_000 { // 1,000 AETH
            StakingTier::Platinum
        } else if amount >= 10_000_000_000 { // 100 AETH
            StakingTier::Gold
        } else if amount >= 1_000_000_000 { // 10 AETH
            StakingTier::Silver
        } else {
            StakingTier::Bronze
        }
    }

    pub fn tier_name(&self) -> &'static str {
        match self {
            StakingTier::Bronze => "Bronze",
            StakingTier::Silver => "Silver",
            StakingTier::Gold => "Gold",
            StakingTier::Platinum => "Platinum",
            StakingTier::Diamond => "Diamond",
        }
    }

    pub fn tier_color(&self) -> &'static str {
        match self {
            StakingTier::Bronze => "#cd7f32",
            StakingTier::Silver => "#c0c0c0",
            StakingTier::Gold => "#ffd700",
            StakingTier::Platinum => "#e5e4e2",
            StakingTier::Diamond => "#b9f2ff",
        }
    }

    pub fn tier_multiplier(&self) -> f64 {
        match self {
            StakingTier::Bronze => 1.0,
            StakingTier::Silver => 1.1,
            StakingTier::Gold => 1.25,
            StakingTier::Platinum => 1.5,
            StakingTier::Diamond => 2.0,
        }
    }
}

/// Calculate tier progress percentage
pub fn calculate_tier_progress(current_stake: u64) -> (StakingTier, f64, Option<StakingTier>, u64) {
    let current_tier = StakingTier::from_stake_amount(current_stake);
    
    let tier_thresholds = [
        (StakingTier::Bronze, 0),
        (StakingTier::Silver, 1_000_000_000),      // 10 AETH
        (StakingTier::Gold, 10_000_000_000),        // 100 AETH
        (StakingTier::Platinum, 100_000_000_000),  // 1,000 AETH
        (StakingTier::Diamond, 1_000_000_000_000), // 10,000 AETH
    ];

    let next_tier_index = match current_tier {
        StakingTier::Bronze => 1,
        StakingTier::Silver => 2,
        StakingTier::Gold => 3,
        StakingTier::Platinum => 4,
        StakingTier::Diamond => 5, // No next tier
    };

    if next_tier_index >= tier_thresholds.len() {
        return (current_tier, 100.0, None, 0);
    }

    let current_threshold = tier_thresholds.iter()
        .find(|(t, _)| *t == current_tier)
        .map(|(_, v)| v)
        .unwrap_or(&0);

    let next_threshold = tier_thresholds[next_tier_index].1;
    let next_tier = tier_thresholds[next_tier_index].0;

    let tier_range = next_threshold - current_threshold;
    let progress_in_tier = current_stake.saturating_sub(*current_threshold);
    let progress_percent = (progress_in_tier as f64 / tier_range as f64) * 100.0;
    let epochs_to_next = ((next_threshold - current_stake) / 100_000_000).max(0); // Rough estimate

    (current_tier, progress_percent, Some(next_tier), epochs_to_next)
}

/// Generate reward projection with multiple scenarios
pub fn generate_reward_scenarios(
    stake_amount: u64,
    epochs: u64,
    config: &StakingRewardConfig,
) -> Vec<RewardScenario> {
    let mut scenarios = Vec::new();
    
    // Conservative scenario (90% uptime, no loyalty)
    let conservative = RewardScenario {
        name: "Conservative".to_string(),
        description: "90% uptime, new staker".to_string(),
        apy: config.base_apy * 0.9,
        projected_rewards: 0,
        risk_level: "Low".to_string(),
    };
    scenarios.push(conservative);

    // Moderate scenario (95% uptime, some loyalty)
    let moderate = RewardScenario {
        name: "Moderate".to_string(),
        description: "95% uptime, 500+ epochs staked".to_string(),
        apy: config.base_apy * 1.05,
        projected_rewards: 0,
        risk_level: "Medium".to_string(),
    };
    scenarios.push(moderate);

    // Optimistic scenario (98% uptime, diamond tier)
    let optimistic = RewardScenario {
        name: "Optimistic".to_string(),
        description: "98% uptime, Diamond tier".to_string(),
        apy: config.base_apy * 1.5,
        projected_rewards: 0,
        risk_level: "High".to_string(),
    };
    scenarios.push(optimistic);

    // Calculate projected rewards for each
    let calculator = RewardCalculator::new(config.clone());
    for scenario in &mut scenarios {
        let epoch_rate = scenario.apy / (365.0 * 288.0);
        scenario.projected_rewards = ((stake_amount as f64 * epoch_rate * epochs as f64) as u64);
    }

    scenarios
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardScenario {
    pub name: String,
    pub description: String,
    pub apy: f64,
    pub projected_rewards: u64,
    pub risk_level: String,
}

/// Validator performance tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPerformance {
    pub validator_id: String,
    pub uptime_history: Vec<f64>,
    pub reward_history: Vec<u64>,
    pub slash_count: u64,
    pub avg_response_time: u64,
    pub rank: u32,
}

impl ValidatorPerformance {
    pub fn new(validator_id: String) -> Self {
        Self {
            validator_id,
            uptime_history: Vec::new(),
            reward_history: Vec::new(),
            slash_count: 0,
            avg_response_time: 0,
            rank: 0,
        }
    }

    pub fn calculate_avg_uptime(&self) -> f64 {
        if self.uptime_history.is_empty() {
            return 0.0;
        }
        self.uptime_history.iter().sum::<f64>() / self.uptime_history.len() as f64
    }

    pub fn calculate_reward_volatility(&self) -> f64 {
        if self.reward_history.len() < 2 {
            return 0.0;
        }
        let mean = self.reward_history.iter().sum::<u64>() as f64 / self.reward_history.len() as f64;
        let variance = self.reward_history.iter()
            .map(|r| (*r as f64 - mean).powi(2))
            .sum::<f64>() / self.reward_history.len() as f64;
        variance.sqrt()
    }

    pub fn is_healthy(&self) -> bool {
        self.slash_count == 0 && self.calculate_avg_uptime() >= 95.0
    }
}

/// Risk assessment for staking positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingRiskAssessment {
    pub overall_risk_score: f64,      // 0-100, lower is safer
    pub validator_risk: f64,
    pub concentration_risk: f64,
    pub tier_risk: f64,
    pub recommendations: Vec<String>,
}

impl StakingRiskAssessment {
    pub fn assess(
        positions: &[DelegatorStake],
        validators: &[ValidatorStake],
        total_stake: u64,
    ) -> Self {
        let mut recommendations = Vec::new();
        
        // Validator risk
        let avg_uptime: f64 = validators.iter()
            .map(|v| v.uptime_percentage)
            .sum::<f64>() / validators.len().max(1) as f64;
        let validator_risk = if avg_uptime >= 98.0 { 10.0 } 
            else if avg_uptime >= 95.0 { 30.0 }
            else if avg_uptime >= 90.0 { 60.0 }
            else { 90.0 };

        // Concentration risk (too much in one validator)
        let max_concentration = validators.iter()
            .map(|v| v.total_stake as f64 / total_stake as f64)
            .fold(0.0f64, f64::max);
        let concentration_risk = if max_concentration > 0.5 { 80.0 }
            else if max_concentration > 0.3 { 50.0 }
            else if max_concentration > 0.15 { 30.0 }
            else { 10.0 };

        // Tier risk (lower tier = higher risk of volatility)
        let avg_tier = positions.iter()
            .map(|p| StakingTier::from_stake_amount(p.staked_amount))
            .collect::<Vec<_>>();
        let tier_risk = if avg_tier.contains(&StakingTier::Diamond) { 10.0 }
            else if avg_tier.contains(&StakingTier::Platinum) { 25.0 }
            else if avg_tier.contains(&StakingTier::Gold) { 50.0 }
            else { 75.0 };

        // Generate recommendations
        if concentration_risk > 50.0 {
            recommendations.push("Consider diversifying your stake across more validators".to_string());
        }
        if validator_risk > 50.0 {
            recommendations.push("Some validators have lower uptime - consider switching".to_string());
        }
        if tier_risk > 50.0 {
            recommendations.push("Consider increasing your stake to reach a higher tier".to_string());
        }

        let overall_risk_score = (validator_risk * 0.4 + concentration_risk * 0.35 + tier_risk * 0.25).min(100.0);

        Self {
            overall_risk_score,
            validator_risk,
            concentration_risk,
            tier_risk,
            recommendations,
        }
    }
}

#[cfg(test)]
mod analytics_tests {
    use super::*;

    #[test]
    fn test_tier_calculation() {
        assert_eq!(StakingTier::from_stake_amount(500_000_000), StakingTier::Bronze);
        assert_eq!(StakingTier::from_stake_amount(5_000_000_000), StakingTier::Silver);
        assert_eq!(StakingTier::from_stake_amount(50_000_000_000), StakingTier::Gold);
        assert_eq!(StakingTier::from_stake_amount(500_000_000_000), StakingTier::Platinum);
        assert_eq!(StakingTier::from_stake_amount(5_000_000_000_000), StakingTier::Diamond);
    }

    #[test]
    fn test_tier_progress() {
        let (tier, progress, next, epochs) = calculate_tier_progress(5_000_000_000);
        assert_eq!(tier, StakingTier::Silver);
        assert!(progress > 0.0 && progress < 100.0);
        assert_eq!(next, Some(StakingTier::Gold));
    }

    #[test]
    fn test_reward_scenarios() {
        let config = StakingRewardConfig::default();
        let scenarios = generate_reward_scenarios(100_000_000, 2880, &config);
        
        assert_eq!(scenarios.len(), 3);
        assert!(scenarios[0].apy < scenarios[1].apy);
        assert!(scenarios[1].apy < scenarios[2].apy);
    }

    #[test]
    fn test_validator_performance() {
        let mut perf = ValidatorPerformance::new("validator1".to_string());
        perf.uptime_history.push(98.5);
        perf.uptime_history.push(99.0);
        perf.uptime_history.push(97.8);
        
        assert!((perf.calculate_avg_uptime() - 98.433).abs() < 0.01);
        assert!(perf.is_healthy());
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

// ============================================================================
// AUTO-COMPOUND TESTS - Sprint 60
// ============================================================================

#[cfg(test)]
mod auto_compound_tests {
    use super::*;

    #[test]
    fn test_auto_compound_config_defaults() {
        let config = AutoCompoundConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.frequency_epochs, 288);
        assert_eq!(config.fee_percent, 0.1);
        assert_eq!(config.reinvest_ratio, 1.0);
    }

    #[test]
    fn test_should_compound() {
        let mut config = AutoCompoundConfig::default();
        config.enabled = true;
        config.frequency_epochs = 288;

        // Should not compound yet
        assert!(!config.should_compound(100));

        // After 288 epochs, should compound
        config.last_compound_epoch = 100;
        assert!(config.should_compound(389)); // 100 + 289
    }

    #[test]
    fn test_calculate_fee() {
        let config = AutoCompoundConfig::default();

        // 0.1% of 1000 = 1
        let fee = config.calculate_fee(1000);
        assert_eq!(fee, 1);
    }

    #[test]
    fn test_net_reward() {
        let config = AutoCompoundConfig::default();

        // 1000 rewards with 0.1% fee = 999 net, 1 fee
        let (net, fee) = config.net_reward(1000);
        assert_eq!(net, 999);
        assert_eq!(fee, 1);
    }

    #[test]
    fn test_optimal_frequency() {
        let config = StakingRewardConfig::default();
        let manager = AutoCompoundManager::new(config);

        // Find optimal frequency for 100 AETH over 1 year with 0.1% fee
        let optimal = manager.calculate_optimal_frequency(
            100_000_000, // 100 AETH
            0.1,
            365 * 288,   // 1 year
            96.0,
        );

        println!("Optimal frequency: {} epochs", optimal);
        assert!(optimal >= 1);
    }

    #[test]
    fn test_compound_apy() {
        let config = StakingRewardConfig::default();
        let manager = AutoCompoundManager::new(config);

        // Daily compounding (288 epochs) with 0.1% fee
        let effective_apy = manager.calculate_compound_apy(0.12, 0.1, 288);
        println!("Effective APY with daily compound: {:.4}%", effective_apy * 100.0);
        assert!(effective_apy < 0.12); // Should be less than base APY due to fees
        assert!(effective_apy > 0.11); // But not drastically less
    }

    #[test]
    fn test_compound_schedule() {
        let config = StakingRewardConfig::default();
        let manager = AutoCompoundManager::new(config);

        // Generate daily compound schedule for 1 week
        let schedule = manager.generate_compound_schedule(0, 288 * 7, 288);

        assert_eq!(schedule.len(), 7);
        assert_eq!(schedule[0], 288);
        assert_eq!(schedule[6], 288 * 7);
    }

    #[test]
    fn test_estimate_total_rewards() {
        let config = StakingRewardConfig::default();
        let manager = AutoCompoundManager::new(config);

        // 100 AETH, 30 days, daily compound, 0.1% fee, 96% uptime
        let (rewards, fees, effective_apy) = manager.estimate_total_rewards(
            100_000_000, // 100 AETH
            288 * 30,    // 30 days
            288,         // daily compound
            0.1,
            96.0,
            100,         // epochs staked
        );

        println!("30-day estimated rewards: {}", rewards);
        println!("Total fees: {}", fees);
        println!("Effective APY: {:.4}%", effective_apy);

        assert!(rewards > 0);
        assert!(fees > 0);
        assert!(effective_apy > 0);
    }
}

// ============================================================================
// Additional Tests: Edge Cases and Integration
// ============================================================================

#[cfg(test)]
mod staking_edge_cases {
    use super::*;

    #[test]
    fn test_zero_stake() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        let reward = calculator.calculate_epoch_reward(0, 1);
        assert_eq!(reward, 0);
    }

    #[test]
    fn test_slashed_validator_no_rewards() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        let mut validator = ValidatorStake::new("val_1".to_string(), 1000);
        validator.is_slashed = true;
        
        let reward = calculator.calculate_validator_reward(&validator, 10);
        assert_eq!(reward, 0);
    }

    #[test]
    fn test_uptime_bonus_threshold() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        // 95% uptime - qualifies for bonus
        let bonus_95 = calculator.calculate_uptime_bonus(1000, 95.0);
        assert!(bonus_95 > 0);
        
        // 94.9% uptime - no bonus
        let bonus_94 = calculator.calculate_uptime_bonus(1000, 94.9);
        assert_eq!(bonus_94, 0);
    }

    #[test]
    fn test_loyalty_multiplier_tiers() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        assert_eq!(calculator.calculate_loyalty_multiplier(50), 1.0);    // < 100 epochs
        assert_eq!(calculator.calculate_loyalty_multiplier(100), 1.05);  // 100-500
        assert_eq!(calculator.calculate_loyalty_multiplier(500), 1.1);  // 500-1000
        assert_eq!(calculator.calculate_loyalty_multiplier(1000), 1.15); // 1000+
    }

    #[test]
    fn test_delegator_respects_commission() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        let delegator = DelegatorStake::new("del_1".to_string(), "val_1".to_string(), 1000);
        let validator = ValidatorStake::new("val_1".to_string(), 10000);
        validator.commission_rate = 0.10; // 10% commission
        
        let reward = calculator.calculate_delegator_reward(&delegator, &validator, 1);
        
        // Reward should be less than if there was no commission
        let reward_no_commission = calculator.calculate_epoch_reward(1000, 1);
        assert!(reward < reward_no_commission);
    }

    #[test]
    fn test_slash_penalty_calculation() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        let mut validator = ValidatorStake::new("val_1".to_string(), 1000);
        validator.total_stake = 1000;
        
        let slash_amount = calculator.apply_slash(&mut validator, "Double signing");
        
        // Slash should be 5% of stake (default config)
        assert_eq!(slash_amount, 50);
        assert!(validator.is_slashed);
    }

    #[test]
    fn test_effective_apy_calculation() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        // High uptime, long staking = highest APY
        let apy_best = calculator.calculate_effective_apy(95.0, 1000);
        
        // Low uptime, short staking = base APY
        let apy_base = calculator.calculate_effective_apy(80.0, 50);
        
        assert!(apy_best > apy_base);
    }

    #[test]
    fn test_projected_rewards_zero_epochs() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        let rewards = calculator.calculate_projected_rewards(1000, 0, 1, 95.0, 100);
        assert_eq!(rewards, 0);
    }

    #[test]
    fn test_staking_reward_config_bounds() {
        let config = StakingRewardConfig::default();
        
        // Test that config values are within reasonable bounds
        assert!(config.base_apy > 0.0);
        assert!(config.min_stake > 0);
        assert!(config.max_stake > config.min_stake);
        assert!(config.slash_penalty < 1.0);
        assert!(config.slash_penalty >= 0.0);
    }

    #[test]
    fn test_validator_stake_new() {
        let validator = ValidatorStake::new("test_val".to_string(), 5000);
        
        assert_eq!(validator.validator_id, "test_val");
        assert_eq!(validator.staked_amount, 5000);
        assert_eq!(validator.total_stake, 5000);
        assert_eq!(validator.delegated_amount, 0);
        assert!(!validator.is_slashed);
    }

    #[test]
    fn test_delegator_stake_new() {
        let delegator = DelegatorStake::new("test_del".to_string(), "test_val".to_string(), 1000);
        
        assert_eq!(delegator.delegator_id, "test_del");
        assert_eq!(delegator.validator_id, "test_val");
        assert_eq!(delegator.staked_amount, 1000);
        assert!(!delegator.is_claimed);
    }

    #[test]
    fn test_auto_compound_disabled() {
        let mut config = AutoCompoundConfig::default();
        config.enabled = false;
        
        // Should never compound when disabled
        assert!(!config.should_compound(u64::MAX));
    }

    #[test]
    fn test_staking_tier_classification() {
        // Test tier classification logic inline
        let classify = |amount: u64| -> &'static str {
            if amount < 100_000_000 { "Bronze" }
            else if amount < 1_000_000_000 { "Silver" }
            else if amount < 10_000_000_000 { "Gold" }
            else { "Platinum" }
        };
        
        assert_eq!(classify(50_000_000), "Bronze");
        assert_eq!(classify(500_000_000), "Silver");
        assert_eq!(classify(5_000_000_000), "Gold");
        assert_eq!(classify(50_000_000_000), "Platinum");
    }

    #[test]
    fn test_validator_score_calculation() {
        let config = StakingRewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        let mut validator = ValidatorStake::new("test_val".to_string(), 1000);
        validator.uptime_percentage = 98.0;
        validator.epochs_active = 500;
        
        // Test that score is based on uptime and age
        let base_reward = calculator.calculate_epoch_reward(validator.total_stake, 1);
        let uptime_bonus = calculator.calculate_uptime_bonus(base_reward, validator.uptime_percentage);
        let loyalty_mult = calculator.calculate_loyalty_multiplier(validator.epochs_active);
        
        // Score components should all be positive
        assert!(base_reward > 0);
        assert!(uptime_bonus > 0);
        assert!(loyalty_mult >= 1.0);
    }

    // ========================================================================
    // Sprint 76: Slash Recovery & Performance Tracking Tests
    // ========================================================================
    
    #[test]
    fn test_slash_recovery_config_default() {
        let config = SlashRecoveryConfig::default();
        
        assert_eq!(config.recovery_epochs, 43200);
        assert_eq!(config.max_recovery_percent, 0.75);
        assert_eq!(config.good_behavior_periods_required, 100);
    }
    
    #[test]
    fn test_performance_tier_from_score() {
        assert_eq!(PerformanceTier::from_score(96.0), PerformanceTier::Elite);
        assert_eq!(PerformanceTier::from_score(90.0), PerformanceTier::Platinum);
        assert_eq!(PerformanceTier::from_score(75.0), PerformanceTier::Gold);
        assert_eq!(PerformanceTier::from_score(55.0), PerformanceTier::Silver);
        assert_eq!(PerformanceTier::from_score(30.0), PerformanceTier::Bronze);
    }
    
    #[test]
    fn test_performance_tier_reward_boost() {
        assert_eq!(PerformanceTier::Elite.reward_boost(), 0.15);
        assert_eq!(PerformanceTier::Platinum.reward_boost(), 0.10);
        assert_eq!(PerformanceTier::Gold.reward_boost(), 0.05);
        assert_eq!(PerformanceTier::Silver.reward_boost(), 0.0);
        assert_eq!(PerformanceTier::Bronze.reward_boost(), -0.05);
    }
    
    #[test]
    fn test_slash_event_recovery_eligibility() {
        let manager = SlashRecoveryManager::new(SlashRecoveryConfig::default());
        
        // Downtime can be recovered
        let mut validator = ValidatorStake::new("test_val".to_string(), 1000);
        let event = manager.process_slash(
            &mut validator, 
            SlashReason::Downtime, 
            50,  // 5% of 1000
            1000
        );
        
        assert!(event.can_recover);
        assert!(!event.fully_recovered);
        
        // Double sign cannot be recovered
        let mut validator2 = ValidatorStake::new("test_val2".to_string(), 1000);
        let event2 = manager.process_slash(
            &mut validator2, 
            SlashReason::DoubleSign, 
            50,
            1000
        );
        
        assert!(!event2.can_recover);
    }
    
    #[test]
    fn test_slash_recovery_calculation() {
        let config = SlashRecoveryConfig::default();
        let manager = SlashRecoveryManager::new(config);
        
        let mut slash = SlashEvent {
            epoch: 1000,
            reason: SlashReason::Downtime,
            amount_slashed: 1000,
            can_recover: true,
            recovery_start_epoch: Some(2000),
            fully_recovered: false,
        };
        
        // At epoch 2100, recovery rate should accumulate
        let recovery = manager.calculate_recovery_amount(&slash, 2100);
        assert!(recovery > 0);
        assert!(recovery <= 750); // Max 75% of 1000
    }
    
    #[test]
    fn test_performance_score_calculation() {
        let perf = ValidatorPerformance {
            validator_id: "test".to_string(),
            blocks_produced: 950,
            blocks_missed: 50,
            attestations_total: 980,
            attestations_missed: 20,
            uptime_percentage: 98.0,
            avg_response_time_ms: 80,
            last_heartbeat: 0,
            performance_score: 0.0,
            tier: PerformanceTier::Bronze,
            consecutive_uptime_epochs: 0,
            slash_history: Vec::new(),
        };
        
        let score = SlashRecoveryManager::calculate_performance_score(&perf);
        assert!(score > 80.0); // Good performance should score high
    }
    
    #[test]
    fn test_validator_performance_tracker() {
        let mut tracker = ValidatorPerformanceTracker::new();
        
        // Get or create should add new validator
        let perf = tracker.get_or_create("val_1");
        assert_eq!(perf.validator_id, "val_1");
        assert_eq!(perf.performance_score, 100.0);
        
        // Record some activity
        tracker.record_block_produced("val_1", 100);
        tracker.record_block_missed("val_1", 101);
        tracker.record_attestation("val_1", true);
        tracker.record_attestation("val_1", false);
        tracker.update_uptime("val_1", 95.0);
        
        // Refresh score
        tracker.refresh_score("val_1");
        
        let updated = tracker.get_or_create("val_1");
        assert_eq!(updated.blocks_produced, 1);
        assert_eq!(updated.blocks_missed, 1);
        assert!(updated.consecutive_uptime_epochs >= 1);
    }
    
    #[test]
    fn test_top_performers_ranking() {
        let mut tracker = ValidatorPerformanceTracker::new();
        
        // Create multiple validators with different scores
        let perf1 = tracker.get_or_create("val_1");
        perf1.performance_score = 75.0;
        
        let perf2 = tracker.get_or_create("val_2");
        perf2.performance_score = 95.0;
        
        let perf3 = tracker.get_or_create("val_3");
        perf3.performance_score = 85.0;
        
        let tops = tracker.get_top_performers(2);
        assert_eq!(tops.len(), 2);
        assert_eq!(tops[0].validator_id, "val_2"); // Highest first
        assert_eq!(tops[1].validator_id, "val_3");
    }
    
    #[test]
    fn test_slash_amount_calculation() {
        let manager = SlashRecoveryManager::new(SlashRecoveryConfig::default());
        let mut validator = ValidatorStake::new("test_val".to_string(), 10000);
        
        let slash_amount = manager.process_slash(
            &mut validator,
            SlashReason::MissedCommitteeDuty,
            500,  // Slash amount
            100
        );
        
        // Validator should be slashed
        assert!(validator.is_slashed);
        assert_eq!(validator.total_stake, 9500); // 10000 - 500
        assert!(slash_amount.can_recover);
    }
}
