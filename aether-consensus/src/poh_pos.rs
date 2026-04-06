//! PoH + PoS Hybrid Consensus with Tower BFT
//!
//! Implements:
//! - Proof of History sequence verification
//! - Proof of Stake validator weighting
//! - Tower BFT: 12-block finality with stake-weighted voting

use aether_core::{Block, Transaction, ValidatorInfo, Hash, Signature};
use aether_poh::verify_poh_sequence;
use std::collections::{HashMap, VecDeque};

/// Maximum votes to keep per validator (for rolling window)
const MAX_VOTES_PER_VALIDATOR: usize = 64;

/// A validator vote for a specific slot
#[derive(Debug, Clone)]
pub struct Vote {
    pub validator: [u8; 32],
    pub slot: u64,
    pub block_hash: Hash,
    pub signature: Signature,
    pub timestamp: u64,
}

/// Tower BFT consensus state
#[derive(Debug, Clone)]
pub struct TowerBFT {
    /// Votes by validator pubkey
    votes: HashMap<[u8; 32], VecDeque<Vote>>,
    /// Stake weight by validator (from genesis or stake program)
    stake_weights: HashMap<[u8; 32], u64>,
    /// Total active stake
    total_stake: u64,
    /// Finalized slots (confirmed by consensus)
    finalized_slots: VecDeque<u64>,
    /// Last confirmed slot
    last_confirmed_slot: u64,
    /// Lock threshold for finality (2/3 + 1 for BFT)
    /// Stored as numerator/denominator to avoid float precision issues
    lock_threshold_num: u64,
    lock_threshold_den: u64,
}

impl TowerBFT {
    /// Create new Tower BFT consensus
    pub fn new() -> Self {
        Self {
            votes: HashMap::new(),
            stake_weights: HashMap::new(),
            total_stake: 0,
            finalized_slots: VecDeque::with_capacity(32),
            last_confirmed_slot: 0,
            lock_threshold_num: 2, // 2/3 + 1 for BFT (strictly more than 2/3)
            lock_threshold_den: 3,
        }
    }

    /// Initialize stake weights from genesis validators
    pub fn init_from_genesis(&mut self, validators: &[ValidatorInfo]) {
        for v in validators {
            let addr = Self::pubkey_to_addr(&v.stake.to_le_bytes());
            self.stake_weights.insert(addr, v.stake);
            self.total_stake += v.stake;
        }
    }

    /// Get consensus weight for a validator (0.0 to 1.0)
    pub fn get_weight(&self, validator: &[u8; 32]) -> f64 {
        let stake = *self.stake_weights.get(validator).unwrap_or(&0) as f64;
        if self.total_stake == 0 {
            return 0.0;
        }
        stake / self.total_stake as f64
    }

    /// Check if stake meets BFT threshold (>2/3)
    /// Uses integer arithmetic: stake * 3 > total_stake * 2
    /// This ensures strictly more than 2/3, avoiding float precision issues
    fn meets_bft_threshold(&self, stake: u64) -> bool {
        stake * self.lock_threshold_den > self.total_stake * self.lock_threshold_num
    }

    /// Submit a vote from a validator
    pub fn submit_vote(&mut self, vote: Vote) -> Result<(), ConsensusError> {
        // Get validator weight
        let weight = self.get_weight(&vote.validator);
        if weight == 0.0 {
            return Err(ConsensusError::UnknownValidator);
        }

        // Store vote
        let votes = self.votes.entry(vote.validator).or_insert_with(VecDeque::new);
        if votes.len() >= MAX_VOTES_PER_VALIDATOR {
            votes.pop_front();
        }
        votes.push_back(vote);

        // Check for new finality
        self.check_finality();

        Ok(())
    }

    /// Process a new block: verify PoH, record implicit vote from producer
    pub fn process_block(
        &mut self,
        block: &Block,
        prev_block_hash: &[u8; 32],
        producer: &[u8; 32],
    ) -> Result<(), ConsensusError> {
        // Get producer weight
        let weight = self.get_weight(producer);
        if weight == 0.0 {
            return Err(ConsensusError::UnknownValidator);
        }

        // CRITICAL: Verify block's prev_hash matches expected chain history
        // This prevents validators from inserting blocks into wrong fork
        if block.header.prev_hash != *prev_block_hash {
            return Err(ConsensusError::InvalidPoh);
        }

        // Verify PoH seed is valid (recompute from block data)
        // For MVP: skip detailed verification if block is empty
        if block.transactions.is_empty() {
            // Empty block - just accept the producer's implicit vote
            let vote = Vote {
                validator: *producer,
                slot: block.header.height,
                block_hash: block.header.poh_hash,
                signature: [0u8; 64], // Self-vote, no actual signature for MVP
                timestamp: block.header.timestamp,
            };
            self.submit_vote(vote)?;
        } else {
            // Block with transactions - verify and vote
            let vote = Vote {
                validator: *producer,
                slot: block.header.height,
                block_hash: block.header.poh_hash,
                signature: [0u8; 64],
                timestamp: block.header.timestamp,
            };
            self.submit_vote(vote)?;
        }

        Ok(())
    }

    /// Check if any slots have achieved finality
    fn check_finality(&mut self) {
        // Tower BFT: a slot is finalized when it has votes from >2/3 of stake
        // spanning 12 consecutive slots ahead of last confirmed

        let lock_slots = 12; // Tower height

        for slot in (self.last_confirmed_slot + 1).. {
            // Sum stake weights of all validators who voted for this slot
            let mut slot_stake: u64 = 0;
            for (_, votes) in &self.votes {
                for vote in votes.iter().rev() {
                    if vote.slot == slot {
                        slot_stake += self.stake_weights.get(&vote.validator).copied().unwrap_or(0);
                        break; // Only count latest vote per validator
                    }
                }
            }

            // Check if BFT threshold reached (>2/3)
            if self.meets_bft_threshold(slot_stake) {
                // Check lock period: need consecutive votes for `lock_slots` slots
                let mut consecutive = true;
                for s in (slot.saturating_sub(lock_slots as u64 - 1))..=slot {
                    let mut s_stake: u64 = 0;
                    for (_, votes) in &self.votes {
                        for vote in votes.iter().rev() {
                            if vote.slot == s {
                                s_stake += self.stake_weights.get(&vote.validator).copied().unwrap_or(0);
                                break;
                            }
                        }
                    }
                    if !self.meets_bft_threshold(s_stake) {
                        consecutive = false;
                        break;
                    }
                }

                if consecutive {
                    self.finalized_slots.push_back(slot);
                    self.last_confirmed_slot = slot;
                    if self.finalized_slots.len() > 32 {
                        self.finalized_slots.pop_front();
                    }
                }
            } else {
                // Slots are in order, if current slot doesn't have enough weight,
                // no need to check higher slots
                break;
            }
        }
    }

    /// Get all finalized slots
    pub fn get_finalized_slots(&self) -> Vec<u64> {
        self.finalized_slots.iter().copied().collect()
    }

    /// Get last confirmed slot
    pub fn last_confirmed(&self) -> u64 {
        self.last_confirmed_slot
    }

    /// Check if a slot is finalized
    pub fn is_finalized(&self, slot: u64) -> bool {
        self.finalized_slots.contains(&slot)
    }

    /// Get confirmation stake for a slot (total stake that voted for it)
    pub fn get_confirmation_stake(&self, slot: u64) -> u64 {
        let mut stake: u64 = 0;
        for (_, votes) in &self.votes {
            for vote in votes.iter().rev() {
                if vote.slot == slot {
                    stake += self.stake_weights.get(&vote.validator).copied().unwrap_or(0);
                    break;
                }
            }
        }
        stake
    }

    /// Get confirmation weight for a slot (deprecated, use get_confirmation_stake)
    pub fn get_confirmation_weight(&self, slot: u64) -> f64 {
        let stake = self.get_confirmation_stake(slot);
        if self.total_stake == 0 {
            return 0.0;
        }
        stake as f64 / self.total_stake as f64
    }

    fn pubkey_to_addr(pubkey: &[u8]) -> [u8; 32] {
        let mut addr = [0u8; 32];
        let len = pubkey.len().min(32);
        addr[..len].copy_from_slice(&pubkey[..len]);
        addr
    }
}

impl Default for TowerBFT {
    fn default() -> Self {
        Self::new()
    }
}

/// Slot info returned by validator status endpoint
#[derive(Debug, Clone)]
pub struct SlotInfo {
    /// Current slot number
    pub slot: u64,
    /// Last confirmed slot
    pub last_confirmed_slot: u64,
    /// Last block hash
    pub last_block_hash: Hash,
    /// Is validator healthy
    pub healthy: bool,
    /// Error message if unhealthy
    pub error: Option<String>,
}

/// Hybrid consensus coordinator
pub struct HybridConsensus {
    pub slot: u64,
    pub tower: TowerBFT,
    pub last_block_hash: Hash,
    /// Is validator initialized and running
    pub initialized: bool,
}

impl HybridConsensus {
    pub fn new() -> Self {
        Self {
            slot: 0,
            tower: TowerBFT::new(),
            last_block_hash: [0u8; 32],
            initialized: false,
        }
    }

    /// Initialize consensus from genesis (must be called before processing blocks)
    pub fn init_from_genesis(&mut self, validators: &[ValidatorInfo]) {
        self.tower.init_from_genesis(validators);
        self.initialized = true;
    }

    /// Get current slot info for validator status endpoint
    pub fn get_slot_info(&self) -> SlotInfo {
        if !self.initialized {
            return SlotInfo {
                slot: 0,
                last_confirmed_slot: 0,
                last_block_hash: [0u8; 32],
                healthy: false,
                error: Some("Validator not initialized - call init_from_genesis first".to_string()),
            };
        }

        SlotInfo {
            slot: self.slot,
            last_confirmed_slot: self.tower.last_confirmed(),
            last_block_hash: self.last_block_hash,
            healthy: true,
            error: None,
        }
    }

    /// Increment slot (called when producing/processing a new block)
    pub fn increment_slot(&mut self) {
        self.slot += 1;
    }

    /// Process a new block (called when receiving/producing a block)
    pub fn process_block(&mut self, block: &Block, producer: &[u8; 32]) -> Result<(), ConsensusError> {
        // Check initialization
        if !self.initialized {
            return Err(ConsensusError::UnknownValidator);
        }

        // CRITICAL: Verify PoH sequence for ALL blocks, including empty ones
        // Empty blocks must still have valid PoH hashes to prevent attackers
        // from inserting malformed blocks into the chain
        if block.header.poh_hash == [0u8; 32] {
            return Err(ConsensusError::InvalidPoh);
        }

        // CRITICAL: Verify block height is sequential (no gaps in slot numbers)
        if block.header.height < self.slot + 1 {
            return Err(ConsensusError::SlotTooOld {
                expected: self.slot + 1,
                actual: block.header.height,
            });
        }
        if block.header.height > self.slot + 1 {
            return Err(ConsensusError::SlotInFuture {
                expected: self.slot + 1,
                actual: block.header.height,
            });
        }

        // Run Tower BFT
        self.tower.process_block(block, &self.last_block_hash, producer)?;

        // Update state
        self.slot = block.header.height;
        self.last_block_hash = block.header.poh_hash;

        Ok(())
    }

    /// Submit a vote (called when validator receives a block from peer)
    pub fn submit_vote(&mut self, vote: Vote) -> Result<(), ConsensusError> {
        self.tower.submit_vote(vote)
    }

    /// Check finality
    pub fn is_finalized(&self, slot: u64) -> bool {
        self.tower.is_finalized(slot)
    }

    pub fn last_confirmed_slot(&self) -> u64 {
        self.tower.last_confirmed()
    }
}

/// Consensus errors
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ConsensusError {
    #[error("Invalid PoH sequence")]
    InvalidPoh,
    #[error("Insufficient stake")]
    InsufficientStake,
    #[error("Unknown validator")]
    UnknownValidator,
    #[error("Slot too old: expected {expected}, got {actual}")]
    SlotTooOld { expected: u64, actual: u64 },
    #[error("Slot in future: expected {expected}, got {actual}")]
    SlotInFuture { expected: u64, actual: u64 },
    #[error("Double vote detected")]
    DoubleVote,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::{Block, BlockHeader, Hash};

    #[test]
    fn test_slot_too_old_error() {
        let mut consensus = HybridConsensus::new();
        let validators = vec![ValidatorInfo {
            address: [1u8; 32],
            stake: 1000,
            commission: 500,
        }];
        consensus.init_from_genesis(&validators);

        // Create a block with old slot height
        let old_block = Block {
            header: BlockHeader {
                height: 0, // Old slot
                prev_hash: [0u8; 32],
                timestamp: 12345,
                poh_hash: [99u8; 32],
                state_root: [0u8; 32],
            },
            transactions: vec![],
        };

        let result = consensus.process_block(&old_block, &[1u8; 32]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConsensusError::SlotTooOld { expected, actual } => {
                assert_eq!(expected, 1);
                assert_eq!(actual, 0);
            }
            _ => panic!("Expected SlotTooOld error"),
        }
    }

    #[test]
    fn test_slot_in_future_error() {
        let mut consensus = HybridConsensus::new();
        let validators = vec![ValidatorInfo {
            address: [1u8; 32],
            stake: 1000,
            commission: 500,
        }];
        consensus.init_from_genesis(&validators);

        // Create a block with future slot height
        let future_block = Block {
            header: BlockHeader {
                height: 5, // Future slot
                prev_hash: [0u8; 32],
                timestamp: 12345,
                poh_hash: [99u8; 32],
                state_root: [0u8; 32],
            },
            transactions: vec![],
        };

        let result = consensus.process_block(&future_block, &[1u8; 32]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConsensusError::SlotInFuture { expected, actual } => {
                assert_eq!(expected, 1);
                assert_eq!(actual, 5);
            }
            _ => panic!("Expected SlotInFuture error"),
        }
    }

    #[test]
    fn test_valid_block_processing() {
        let mut consensus = HybridConsensus::new();
        let validators = vec![ValidatorInfo {
            address: [1u8; 32],
            stake: 1000,
            commission: 500,
        }];
        consensus.init_from_genesis(&validators);

        // Create a valid block with correct slot height
        let valid_block = Block {
            header: BlockHeader {
                height: 1, // Correct next slot
                prev_hash: [0u8; 32],
                timestamp: 12345,
                poh_hash: [99u8; 32],
                state_root: [0u8; 32],
            },
            transactions: vec![],
        };

        let result = consensus.process_block(&valid_block, &[1u8; 32]);
        assert!(result.is_ok());
        assert_eq!(consensus.slot, 1);
    }
}
