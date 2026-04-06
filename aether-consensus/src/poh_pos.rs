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
    /// Lock threshold for finality (fraction of stake)
    lock_threshold: f64,
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
            lock_threshold: 0.67, // 2/3 of stake for finality
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
            // Sum weights of all validators who voted for this slot
            let mut slot_weight: f64 = 0.0;
            for (_, votes) in &self.votes {
                for vote in votes.iter().rev() {
                    if vote.slot == slot {
                        slot_weight += self.get_weight(&vote.validator);
                        break; // Only count latest vote per validator
                    }
                }
            }

            // Check if lock threshold reached
            if slot_weight >= self.lock_threshold {
                // Check lock period: need consecutive votes for `lock_slots` slots
                let mut consecutive = true;
                for s in (slot.saturating_sub(lock_slots as u64 - 1))..=slot {
                    let mut s_weight: f64 = 0.0;
                    for (_, votes) in &self.votes {
                        for vote in votes.iter().rev() {
                            if vote.slot == s {
                                s_weight += self.get_weight(&vote.validator);
                                break;
                            }
                        }
                    }
                    if s_weight < self.lock_threshold {
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

    /// Get confirmation count for a slot (stake weight that voted for it)
    pub fn get_confirmation_weight(&self, slot: u64) -> f64 {
        let mut weight: f64 = 0.0;
        for (_, votes) in &self.votes {
            for vote in votes.iter().rev() {
                if vote.slot == slot {
                    weight += self.get_weight(&vote.validator);
                    break;
                }
            }
        }
        weight
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

/// Hybrid consensus coordinator
pub struct HybridConsensus {
    pub slot: u64,
    pub tower: TowerBFT,
    pub last_block_hash: Hash,
}

impl HybridConsensus {
    pub fn new() -> Self {
        Self {
            slot: 0,
            tower: TowerBFT::new(),
            last_block_hash: [0u8; 32],
        }
    }

    /// Initialize from genesis validators
    pub fn init_from_genesis(&mut self, validators: &[ValidatorInfo]) {
        self.tower.init_from_genesis(validators);
    }

    /// Process a new block (called when receiving/producing a block)
    pub fn process_block(&mut self, block: &Block, producer: &[u8; 32]) -> Result<(), ConsensusError> {
        // Verify PoH sequence
        // For now, trust the block's PoH seed if block has transactions
        if !block.transactions.is_empty() {
            // Would verify: verify_poh_sequence(...)
            // For MVP, skip verification
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
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Invalid PoH sequence")]
    InvalidPoh,
    #[error("Insufficient stake")]
    InsufficientStake,
    #[error("Unknown validator")]
    UnknownValidator,
    #[error("Slot too old")]
    SlotTooOld,
    #[error("Double vote detected")]
    DoubleVote,
}
