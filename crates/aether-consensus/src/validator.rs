//! Validator Node Implementation
//!
//! Core validator logic for running an AETHER validator node.

use crate::{ConsensusError, ConsensusResult};
use crate::aetherflow::AetherFlow;
use crate::pos::{StakePool, ValidatorStake};
use crate::tower::TowerConsensus;
use aether_common::{ValidatorTier, MINIMUM_STAKE_AETH};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Validator node state
pub struct ValidatorNode {
    /// Validator identity
    pub identity: ValidatorIdentity,
    /// Consensus engine
    consensus: Arc<RwLock<AetherFlow>>,
    /// Vote state
    vote_state: TowerConsensus,
    /// Last slot processed
    last_slot: u64,
    /// Is running
    running: bool,
    /// Slot timer
    slot_timer: Instant,
}

/// Validator identity
#[derive(Debug, Clone)]
pub struct ValidatorIdentity {
    /// Public key
    pub pubkey: [u8; 32],
    /// Private key (simplified - would be proper keypair)
    pub secret: [u8; 64],
    /// Validator tier
    pub tier: ValidatorTier,
    /// Commission percentage
    pub commission: u8,
}

impl ValidatorNode {
    /// Create new validator node
    pub fn new(
        identity: ValidatorIdentity,
        consensus: Arc<RwLock<AetherFlow>>,
    ) -> Self {
        Self {
            identity,
            consensus,
            vote_state: TowerConsensus::new(),
            last_slot: 0,
            running: false,
            slot_timer: Instant::now(),
        }
    }

    /// Start the validator
    pub fn start(&mut self) {
        self.running = true;
        self.slot_timer = Instant::now();
    }

    /// Stop the validator
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if validator is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Process a slot
    pub fn process_slot(&mut self) -> ConsensusResult<Option<SlotOutcome>> {
        if !self.running {
            return Ok(None);
        }

        let consensus = self.consensus.read().map_err(|_| {
            ConsensusError::TowerError("Consensus lock poisoned".to_string())
        })?;

        let current_slot = consensus.current_slot();
        drop(consensus);

        if current_slot <= self.last_slot {
            return Ok(None);
        }

        self.last_slot = current_slot;

        // Check if we're the leader
        let outcome = if self.is_leader(current_slot) {
            // Produce block
            self.produce_block(current_slot)?
        } else {
            // Vote on block
            self.vote_on_block(current_slot)?
        };

        Ok(Some(outcome))
    }

    /// Check if this validator is the leader for a slot
    fn is_leader(&self,
        slot: u64,
    ) -> bool {
        let consensus = match self.consensus.read() {
            Ok(c) => c,
            Err(_) => return false,
        };

        consensus.get_slot_leader(slot)
            .map(|leader| leader == self.identity.pubkey)
            .unwrap_or(false)
    }

    /// Produce a block as leader
    fn produce_block(
        &self,
        _slot: u64,
    ) -> ConsensusResult<SlotOutcome> {
        let mut consensus = self.consensus.write().map_err(|_| {
            ConsensusError::TowerError("Consensus lock poisoned".to_string())
        })?;

        let block = consensus.produce_block(self.identity.pubkey)?;

        Ok(SlotOutcome::ProducedBlock(block))
    }

    /// Vote on a block
    fn vote_on_block(
        &mut self,
        slot: u64,
    ) -> ConsensusResult<SlotOutcome> {
        // Get our stake weight
        let stake = self.get_stake_weight();

        // Submit vote to tower consensus
        self.vote_state.process_vote(
            self.identity.pubkey,
            slot,
            stake,
        )?;

        Ok(SlotOutcome::Voted { slot, stake })
    }

    /// Get validator's stake weight
    fn get_stake_weight(&self) -> u64 {
        let consensus = match self.consensus.read() {
            Ok(c) => c,
            Err(_) => return 0,
        };

        consensus.stake_pool().get_stake_weight(&self.identity.pubkey) as u64
    }

    /// Get next slot time
    pub fn next_slot_time(&self) -> Duration {
        let elapsed = self.slot_timer.elapsed().as_millis() as u64;
        let slot_duration = 400u64; // 400ms
        let current_slot = elapsed / slot_duration;
        let next_slot_start = (current_slot + 1) * slot_duration;
        
        Duration::from_millis(next_slot_start - elapsed)
    }

    /// Update stake in pool
    pub fn update_stake(&self, stake: u64) {
        let validator = ValidatorStake::new(
            self.identity.pubkey,
            stake,
            self.identity.tier,
        );

        if let Ok(mut consensus) = self.consensus.write() {
            consensus.add_validator(validator);
        }
    }
}

/// Outcome of processing a slot
#[derive(Debug)]
pub enum SlotOutcome {
    /// Produced a block
    ProducedBlock(crate::aetherflow::AetherBlock),
    /// Voted on a block
    Voted { slot: u64, stake: u64 },
    /// Skipped (no leader or already voted)
    Skipped { reason: String },
}

/// Validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Stake amount
    pub stake: u64,
    /// Validator tier
    pub tier: ValidatorTier,
    /// Commission percentage
    pub commission: u8,
    /// Enable AI processing (Tier 1 only)
    pub enable_ai: bool,
    /// Enable zk-STARK processing (Tier 1 only)
    pub enable_zk: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            stake: MINIMUM_STAKE_AETH,
            tier: ValidatorTier::Standard,
            commission: 10,
            enable_ai: false,
            enable_zk: false,
        }
    }
}

/// Validator metrics
#[derive(Debug, Clone, Default)]
pub struct ValidatorMetrics {
    /// Total blocks produced
    pub blocks_produced: u64,
    /// Total votes submitted
    pub votes_submitted: u64,
    /// Total rewards earned
    pub rewards_earned: u64,
    /// Uptime percentage
    pub uptime_percent: f64,
    /// Average block time
    pub avg_block_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_identity(tier: ValidatorTier) -> ValidatorIdentity {
        ValidatorIdentity {
            pubkey: [1u8; 32],
            secret: [0u8; 64],
            tier,
            commission: 10,
        }
    }

    #[test]
    fn test_validator_creation() {
        let consensus = Arc::new(RwLock::new(AetherFlow::new()));
        let identity = create_test_identity(ValidatorTier::Standard);
        
        let validator = ValidatorNode::new(identity, consensus);
        
        assert_eq!(validator.last_slot, 0);
        assert!(!validator.running);
    }

    #[test]
    fn test_validator_start_stop() {
        let consensus = Arc::new(RwLock::new(AetherFlow::new()));
        let identity = create_test_identity(ValidatorTier::Standard);
        
        let mut validator = ValidatorNode::new(identity, consensus);
        
        validator.start();
        assert!(validator.running);
        
        validator.stop();
        assert!(!validator.running);
    }

    #[test]
    fn test_config_defaults() {
        let config = ValidatorConfig::default();
        
        assert_eq!(config.stake, MINIMUM_STAKE_AETH);
        assert_eq!(config.tier, ValidatorTier::Standard);
        assert_eq!(config.commission, 10);
        assert!(!config.enable_ai);
        assert!(!config.enable_zk);
    }
}
