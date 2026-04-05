//! Proof of Stake (PoS) Implementation
//!
//! Implements stake-weighted leader election and validator selection
//! with modifications for AI validator tiers.

use aether_common::{ValidatorTier, MINIMUM_STAKE_AETH, MINIMUM_AI_STAKE_AETH};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Validator stake information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStake {
    /// Validator public key
    pub pubkey: [u8; 32],
    /// Staked amount (lamports)
    pub stake: u64,
    /// Delegated stake from others
    pub delegated_stake: u64,
    /// Validator commission percentage
    pub commission: u8,
    /// Validator tier
    pub tier: ValidatorTier,
    /// Warmup/cooldown epochs
    pub warmup_epochs: u64,
    pub cooldown_epochs: u64,
    /// Whether stake is active
    pub is_active: bool,
    /// Last vote slot
    pub last_vote: u64,
}

impl ValidatorStake {
    /// Create new validator stake
    pub fn new(pubkey: [u8; 32], stake: u64, tier: ValidatorTier) -> Self {
        Self {
            pubkey,
            stake,
            delegated_stake: 0,
            commission: 10, // 10% default
            tier,
            warmup_epochs: 0,
            cooldown_epochs: 0,
            is_active: stake >= MINIMUM_STAKE_AETH,
            last_vote: 0,
        }
    }

    /// Get total stake
    pub fn total_stake(&self) -> u64 {
        self.stake.saturating_add(self.delegated_stake)
    }

    /// Get effective stake (after commission)
    pub fn effective_stake(&self) -> u64 {
        let total = self.total_stake();
        let commission_rate = self.commission as u64;
        total.saturating_sub(total * commission_rate / 100)
    }

    /// Check if validator meets minimum stake for its tier
    pub fn meets_minimum_stake(&self) -> bool {
        let min_stake = match self.tier {
            ValidatorTier::AI => MINIMUM_AI_STAKE_AETH,
            ValidatorTier::Standard | ValidatorTier::Light => MINIMUM_STAKE_AETH,
        };
        self.total_stake() >= min_stake
    }

    /// Can this validator produce blocks?
    pub fn can_produce_blocks(&self) -> bool {
        self.is_active && self.meets_minimum_stake() && self.tier.can_produce_blocks()
    }

    /// Get reward multiplier based on tier
    pub fn reward_multiplier(&self) -> f64 {
        self.tier.reward_multiplier()
    }
}

/// Stake pool for managing all validator stakes
#[derive(Debug, Clone, Default)]
pub struct StakePool {
    /// All validators
    pub validators: Vec<ValidatorStake>,
    /// Total active stake
    pub total_active_stake: u64,
    /// Current epoch
    pub current_epoch: u64,
}

impl StakePool {
    /// Create new stake pool
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update validator stake
    pub fn update_stake(&mut self, validator: ValidatorStake) {
        if let Some(existing) = self.validators.iter_mut()
            .find(|v| v.pubkey == validator.pubkey) {
            *existing = validator;
        } else {
            self.validators.push(validator);
        }
        self.recalculate_total();
    }

    /// Add delegated stake to a validator (called when user delegates to them)
    pub fn add_delegated_stake(&mut self, validator_pubkey: &[u8; 32], amount: u64) {
        if let Some(validator) = self.validators.iter_mut()
            .find(|v| &v.pubkey == validator_pubkey) {
            validator.delegated_stake = validator.delegated_stake.saturating_add(amount);
            self.recalculate_total();
        }
    }

    /// Remove delegated stake from a validator (called when user undelegates)
    pub fn remove_delegated_stake(&mut self, validator_pubkey: &[u8; 32], amount: u64) {
        if let Some(validator) = self.validators.iter_mut()
            .find(|v| &v.pubkey == validator_pubkey) {
            validator.delegated_stake = validator.delegated_stake.saturating_sub(amount);
            self.recalculate_total();
        }
    }

    /// Remove validator
    pub fn remove_validator(&mut self, pubkey: &[u8; 32]) -> Option<ValidatorStake> {
        let index = self.validators.iter().position(|v| &v.pubkey == pubkey)?;
        let validator = self.validators.remove(index);
        self.recalculate_total();
        Some(validator)
    }

    /// Get validator by pubkey
    pub fn get_validator(&self, pubkey: &[u8; 32]) -> Option<&ValidatorStake> {
        self.validators.iter().find(|v| &v.pubkey == pubkey)
    }

    /// Get mutable validator
    pub fn get_validator_mut(&mut self, pubkey: &[u8; 32]) -> Option<&mut ValidatorStake> {
        self.validators.iter_mut().find(|v| &v.pubkey == pubkey)
    }

    /// Get active validators
    pub fn active_validators(&self) -> Vec<&ValidatorStake> {
        self.validators.iter()
            .filter(|v| v.is_active && v.meets_minimum_stake())
            .collect()
    }

    /// Get validators that can produce blocks
    pub fn block_producers(&self) -> Vec<&ValidatorStake> {
        self.validators.iter()
            .filter(|v| v.can_produce_blocks())
            .collect()
    }

    /// Get AI validators
    pub fn ai_validators(&self) -> Vec<&ValidatorStake> {
        self.validators.iter()
            .filter(|v| matches!(v.tier, ValidatorTier::AI) && v.is_active)
            .collect()
    }

    /// Recalculate total active stake
    fn recalculate_total(&mut self) {
        self.total_active_stake = self.validators.iter()
            .filter(|v| v.is_active && v.meets_minimum_stake())
            .map(|v| v.total_stake())
            .sum();
    }

    /// Get stake weight for validator (0.0 - 1.0)
    pub fn get_stake_weight(&self, pubkey: &[u8; 32]) -> f64 {
        let validator = match self.get_validator(pubkey) {
            Some(v) => v,
            None => return 0.0,
        };

        if self.total_active_stake == 0 {
            return 0.0;
        }

        validator.total_stake() as f64 / self.total_active_stake as f64
    }
}

/// Leader election using deterministic randomness
pub struct LeaderSchedule {
    /// Slot -> Leader pubkey mapping
    pub leaders: Vec<[u8; 32]>,
    /// Epoch this schedule applies to
    pub epoch: u64,
}

impl LeaderSchedule {
    /// Generate leader schedule for an epoch
    pub fn generate(
        stake_pool: &StakePool,
        epoch: u64,
        num_slots: u64,
    ) -> Self {
        let block_producers = stake_pool.block_producers();
        
        if block_producers.is_empty() {
            return Self {
                leaders: vec![[0u8; 32]; num_slots as usize],
                epoch,
            };
        }

        // Create weighted list based on stake
        let mut weighted_producers: Vec<(Vec<u8>, u64)> = block_producers.iter()
            .map(|v| (v.pubkey.to_vec(),
                // Use effective stake for weighting
                v.effective_stake()
            ))
            .collect();

        // Sort by stake (highest first)
        weighted_producers.sort_by_key(|(_, stake)| std::cmp::Reverse(*stake));

        let total_stake: u64 = weighted_producers.iter().map(|(_, s)| *s).sum();
        
        let mut leaders = Vec::with_capacity(num_slots as usize);
        let mut rng_seed = hash_seed(epoch, &stake_pool);

        for slot in 0..num_slots {
            // Use stake-weighted selection
            let leader = select_weighted_leader(
                &weighted_producers,
                total_stake,
                &mut rng_seed,
            );
            leaders.push(leader);
        }

        Self { leaders, epoch }
    }

    /// Get leader for a slot
    pub fn get_leader(&self, slot: u64) -> Option<&[u8; 32]> {
        self.leaders.get(slot as usize)
    }
}

/// Hash seed for leader election
fn hash_seed(epoch: u64, stake_pool: &StakePool) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(&epoch.to_le_bytes());
    hasher.update(&stake_pool.total_active_stake.to_le_bytes());
    
    for validator in &stake_pool.validators {
        hasher.update(&validator.pubkey);
        hasher.update(&validator.total_stake().to_le_bytes());
    }
    
    hasher.finalize().into()
}

/// Simple deterministic RNG
fn next_random(seed: &mut [u8; 32]) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(&mut *seed);
    let hash: [u8; 32] = hasher.finalize().into();
    *seed = hash;
    
    u64::from_le_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7],
    ])
}

/// Select leader using stake-weighted randomness
fn select_weighted_leader(
    weighted_producers: &[(Vec<u8>, u64)],
    total_stake: u64,
    seed: &mut [u8; 32],
) -> [u8; 32] {
    let random_value = next_random(seed);
    let threshold = (random_value as u128 * total_stake as u128 / u64::MAX as u128) as u64;
    
    let mut cumulative = 0u64;
    for (pubkey, stake) in weighted_producers {
        cumulative += stake;
        if cumulative >= threshold {
            let mut result = [0u8; 32];
            result.copy_from_slice(&pubkey[..32.min(pubkey.len())]);
            return result;
        }
    }
    
    // Fallback to last producer
    let last = weighted_producers.last().unwrap();
    let mut result = [0u8; 32];
    result.copy_from_slice(&last.0[..32.min(last.0.len())]);
    result
}

/// Calculate rewards for an epoch
pub fn calculate_epoch_rewards(
    epoch: u64,
    total_stake: u64,
    base_emission: u64,
) -> u64 {
    // Simple inflation schedule
    let year = epoch / 182; // ~1 year (365 days / 2 day epochs)
    
    let inflation_rate = match year {
        0 => 450,  // 4.5% year 1
        1 => 400,  // 4.0% year 2
        2 => 350,  // 3.5% year 3
        3 => 300,  // 3.0% year 4
        4 => 250,  // 2.5% year 5
        5..=9 => 150, // 1.5% years 6-10
        _ => 0,     // 0% after year 10
    };

    // Base emission * inflation rate (in basis points)
    base_emission * inflation_rate / 10_000
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validator(pubkey: [u8; 32], stake: u64, tier: ValidatorTier) -> ValidatorStake {
        ValidatorStake::new(pubkey, stake, tier)
    }

    #[test]
    fn test_validator_stake() {
        let validator = create_test_validator([1u8; 32], 20_000_000_000_000, ValidatorTier::Standard);
        
        assert_eq!(validator.total_stake(), 20_000_000_000_000);
        assert!(validator.meets_minimum_stake());
        assert!(validator.can_produce_blocks());
        assert!(!validator.tier.has_ai_capabilities());
    }

    #[test]
    fn test_ai_validator_minimum() {
        let validator = create_test_validator([1u8; 32], 20_000_000_000_000, ValidatorTier::AI);
        
        // Should fail - needs 100k AETH minimum
        assert!(!validator.meets_minimum_stake());
        assert!(!validator.can_produce_blocks());

        let ai_validator = create_test_validator([2u8; 32], 100_000_000_000_000, ValidatorTier::AI);
        assert!(ai_validator.meets_minimum_stake());
        assert!(ai_validator.can_produce_blocks());
        assert!(ai_validator.tier.has_ai_capabilities());
    }

    #[test]
    fn test_stake_pool() {
        let mut pool = StakePool::new();
        
        let v1 = create_test_validator([1u8; 32], 20_000_000_000_000, ValidatorTier::Standard);
        let v2 = create_test_validator([2u8; 32], 30_000_000_000_000, ValidatorTier::Standard);
        
        pool.update_stake(v1);
        pool.update_stake(v2);
        
        assert_eq!(pool.validators.len(), 2);
        assert_eq!(pool.total_active_stake, 50_000_000_000_000);
        
        let producers = pool.block_producers();
        assert_eq!(producers.len(), 2);
    }

    #[test]
    fn test_leader_schedule() {
        let mut pool = StakePool::new();
        
        // Create validators with different stakes
        let v1 = create_test_validator([1u8; 32], 90_000_000_000_000, ValidatorTier::Standard);
        let v2 = create_test_validator([2u8; 32], 10_000_000_000_000, ValidatorTier::Standard);
        
        pool.update_stake(v1);
        pool.update_stake(v2);
        
        let schedule = LeaderSchedule::generate(&pool, 0, 100);
        
        // Both validators should be in the schedule
        let leaders: std::collections::HashSet<_> = schedule.leaders.iter().collect();
        assert_eq!(leaders.len(), 2);
        assert!(leaders.contains(&[1u8; 32]));
        assert!(leaders.contains(&[2u8; 32]));
    }

    #[test]
    fn test_epoch_rewards() {
        // Year 1 should be 4.5%
        let rewards = calculate_epoch_rewards(0, 1_000_000_000_000_000_000, 1_000_000_000_000);
        assert_eq!(rewards, 45_000_000_000); // 4.5% of base

        // Year 5 should be 2.5%
        let rewards_y5 = calculate_epoch_rewards(4 * 182, 1_000_000_000_000_000_000, 1_000_000_000_000);
        assert_eq!(rewards_y5, 25_000_000_000);

        // After year 10 should be 0
        let rewards_y11 = calculate_epoch_rewards(10 * 182, 1_000_000_000_000_000_000, 1_000_000_000_000);
        assert_eq!(rewards_y11, 0);
    }
}
