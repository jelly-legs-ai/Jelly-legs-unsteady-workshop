//! Founding Validator Program
//!
//! Early participants who stake get 2x reward multiplier.
//! Founding validators are registered at genesis or shortly after.

use crate::types::{Address, Hash};
use crate::error::AetherError;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Required stake to join as a founding validator (AETH)
pub const FOUNDING_STAKE_AMOUNT: u64 = 10_000;

/// Reward multiplier for founding validators (2.0x)
pub const FOUNDING_REWARD_MULTIPLIER: f64 = 2.0;

/// Maximum time window after genesis to join as founding validator (seconds)
const FOUNDING_WINDOW_SECS: u64 = 30 * 24 * 60 * 60; // 30 days

/// Founding validator record
#[derive(Debug, Clone)]
pub struct FoundingValidator {
    /// Validator wallet address
    pub address: Address,
    /// Staked AETH amount
    pub stake: u64,
    /// Unix timestamp when they joined
    pub join_date: u64,
    /// Reward multiplier (always 2.0 for founding validators)
    pub reward_mult: f64,
}

impl FoundingValidator {
    /// Create a new founding validator record
    pub fn new(address: Address, stake: u64, join_date: u64) -> Self {
        Self {
            address,
            stake,
            join_date,
            reward_mult: FOUNDING_REWARD_MULTIPLIER,
        }
    }

    /// Check if a given address is an eligible founding validator
    pub fn is_eligible(address: &Address, genesis_timestamp: u64) -> bool {
        let now = Self::current_timestamp();
        let window_end = genesis_timestamp.saturating_add(FOUNDING_WINDOW_SECS);
        now <= window_end
    }
}

/// Tracks all founding validators in the network
pub struct FoundingValidatorRegistry {
    /// Map of address -> founding validator record
    validators: HashMap<Address, FoundingValidator>,
    /// Genesis timestamp (network launch time)
    genesis_timestamp: u64,
}

impl FoundingValidatorRegistry {
    /// Create a new registry with a given genesis time
    pub fn new(genesis_timestamp: u64) -> Self {
        Self {
            validators: HashMap::new(),
            genesis_timestamp,
        }
    }

    /// Register a new founding validator if they meet eligibility requirements.
    ///
    /// Requirements:
    /// - Must join within the 30-day founding window
    /// - Must stake exactly FOUNDING_STAKE_AMOUNT (10,000 AETH)
    pub fn register(
        &mut self,
        address: Address,
        stake: u64,
        join_date: Option<u64>,
    ) -> Result<FoundingValidator, AetherError> {
        let join_ts = join_date.unwrap_or_else(Self::current_timestamp);

        // Check eligibility window
        let window_end = self.genesis_timestamp.saturating_add(FOUNDING_WINDOW_SECS);
        if join_ts > window_end {
            return Err(AetherError::FoundingWindowClosed {
                window_end,
                attempted_join: join_ts,
            });
        }

        // Check stake amount
        if stake != FOUNDING_STAKE_AMOUNT {
            return Err(AetherError::IncorrectFoundingStake {
                required: FOUNDING_STAKE_AMOUNT,
                provided: stake,
            });
        }

        // Check not already registered
        if self.validators.contains_key(&address) {
            return Err(AetherError::AlreadyAValidator { address });
        }

        let validator = FoundingValidator::new(address, stake, join_ts);
        self.validators.insert(address, validator.clone());
        Ok(validator)
    }

    /// Calculate the founding reward for a given base reward amount.
    ///
    /// Returns base_reward * FOUNDING_REWARD_MULTIPLIER (2.0x)
    pub fn calculate_founding_reward(&self, base_reward: u64, address: &Address) -> u64 {
        if let Some(validator) = self.validators.get(address) {
            (base_reward as f64 * validator.reward_mult) as u64
        } else {
            base_reward
        }
    }

    /// Claim accumulated founding rewards for a validator.
    /// Returns the total reward including the founding multiplier.
    pub fn claim_founding_rewards(
        &self,
        address: &Address,
        base_rewards_earned: u64,
    ) -> Result<u64, AetherError> {
        let validator = self
            .validators
            .get(address)
            .ok_or(AetherError::NotAValidator { address: *address })?;

        if validator.stake < FOUNDING_STAKE_AMOUNT {
            return Err(AetherError::InsufficientFoundingStake {
                required: FOUNDING_STAKE_AMOUNT,
                actual: validator.stake,
            });
        }

        let total = (base_rewards_earned as f64 * FOUNDING_REWARD_MULTIPLIER) as u64;
        Ok(total)
    }

    /// Check if an address is a registered founding validator
    pub fn is_founding_validator(&self, address: &Address) -> bool {
        self.validators.contains_key(address)
    }

    /// Get the count of registered founding validators
    pub fn founding_count(&self) -> usize {
        self.validators.len()
    }

    /// Get a founding validator by address
    pub fn get(&self, address: &Address) -> Option<&FoundingValidator> {
        self.validators.get(address)
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_founding_validator() {
        let genesis = 100_000_000; // far in the past
        let mut registry = FoundingValidatorRegistry::new(genesis);
        let addr = Address::random();

        let result = registry.register(addr, FOUNDING_STAKE_AMOUNT, None);
        assert!(result.is_ok());
        assert!(registry.is_founding_validator(&addr));
        assert_eq!(registry.founding_count(), 1);
    }

    #[test]
    fn test_wrong_stake_amount() {
        let genesis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut registry = FoundingValidatorRegistry::new(genesis);
        let addr = Address::random();

        let result = registry.register(addr, 5_000, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_founding_reward_multiplier() {
        let genesis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut registry = FoundingValidatorRegistry::new(genesis);
        let addr = Address::random();

        registry.register(addr, FOUNDING_STAKE_AMOUNT, None).unwrap();
        let reward = registry.calculate_founding_reward(100, &addr);
        assert_eq!(reward, 200); // 100 * 2.0
    }
}
