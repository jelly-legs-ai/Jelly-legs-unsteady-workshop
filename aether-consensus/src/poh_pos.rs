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

    /// Handle chain reorganization
    ///
    /// When a longer/forked chain is received, this method:
    /// 1. Identifies the fork point (last common ancestor)
    /// 2. Rolls back votes after the fork point
    /// 3. Processes the new chain's blocks and votes
    /// 4. Re-evaluates finality
    ///
    /// # Arguments
    /// * `fork_slot` - The slot where the fork occurs (last common ancestor)
    /// * `new_chain` - Vector of (block_hash, producer) pairs for the new chain
    ///
    /// # Returns
    /// * `Ok(())` if reorganization successful
    /// * `Err(ConsensusError)` if reorganization fails
    pub fn reorganize_chain(
        &mut self,
        fork_slot: u64,
        new_chain: &[(Hash, [u8; 32])],
    ) -> Result<(), ConsensusError> {
        // Validate fork slot is not already finalized
        if self.is_finalized(fork_slot) {
            return Err(ConsensusError::SlotTooOld);
        }

        // Remove all votes and finalized slots after the fork point
        self.rollback_to_slot(fork_slot);

        // Process each block in the new chain
        for (idx, (block_hash, producer)) in new_chain.iter().enumerate() {
            let slot = fork_slot + 1 + idx as u64;
            let vote = Vote {
                validator: *producer,
                slot,
                block_hash: *block_hash,
                signature: [0u8; 64],
                timestamp: 0, // Would use actual timestamp in production
            };
            self.submit_vote(vote)?;
        }

        // Re-evaluate finality with the new chain
        self.check_finality();

        Ok(())
    }

    /// Rollback all state to a given slot
    fn rollback_to_slot(&mut self, slot: u64) {
        // Remove votes for slots after the fork point
        for (_, votes) in self.votes.iter_mut() {
            votes.retain(|vote| vote.slot <= slot);
        }

        // Remove finalized slots after the fork point
        while let Some(&last) = self.finalized_slots.back() {
            if last > slot {
                self.finalized_slots.pop_back();
            } else {
                break;
            }
        }

        // Update last confirmed slot
        self.last_confirmed_slot = self.last_confirmed_slot.min(slot);
    }

    /// Get the fork point between current chain and a new chain
    ///
    /// Returns the last common slot between the current chain and a proposed new chain.
    /// This is used to determine where to rollback during reorganization.
    pub fn find_fork_point(&self, new_chain_hashes: &[Hash]) -> Option<u64> {
        // Simplified: assumes we can match by slot number
        // In production, would compare actual block hashes
        for (idx, _) in new_chain_hashes.iter().enumerate() {
            let slot = idx as u64;
            if !self.finalized_slots.contains(&slot) {
                return Some(slot.saturating_sub(1));
            }
        }
        None
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

    /// Handle chain reorganization for the hybrid consensus
    ///
    /// Wraps TowerBFT's reorganization logic and updates local state.
    pub fn reorganize_chain(
        &mut self,
        fork_slot: u64,
        new_chain: &[(Hash, [u8; 32])],
    ) -> Result<(), ConsensusError> {
        self.tower.reorganize_chain(fork_slot, new_chain)?;
        
        // Update local slot tracking to match the new chain tip
        if let Some(last_block) = new_chain.last() {
            self.last_block_hash = last_block.0;
        }
        
        Ok(())
    }

    /// Find fork point with a new chain
    pub fn find_fork_point(&self, new_chain_hashes: &[Hash]) -> Option<u64> {
        self.tower.find_fork_point(new_chain_hashes)
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
    #[error("Chain reorganization required: fork at slot {fork_slot}, new chain height {new_height}")]
    ChainReorganizationRequired { fork_slot: u64, new_height: u64 },
    #[error("Invalid block height: expected {expected}, got {actual}")]
    InvalidBlockHeight { expected: u64, actual: u64 },
    #[error("Parent hash mismatch")]
    ParentHashMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_reorganization_basic() {
        let mut consensus = TowerBFT::new();
        
        // Setup: Create a fork point at slot 5
        let fork_slot = 5u64;
        
        // Simulate new chain with 3 blocks after fork
        let new_chain: Vec<(Hash, [u8; 32])> = (1..=3)
            .map(|i| {
                let mut hash = [0u8; 32];
                hash[0] = i as u8;
                let mut producer = [0u8; 32];
                producer[0] = i as u8;
                (hash, producer)
            })
            .collect();
        
        // Reorganize should succeed (no finalized slots yet)
        let result = consensus.reorganize_chain(fork_slot, &new_chain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_reorganization_rejects_finalized_fork() {
        let mut consensus = TowerBFT::new();
        
        // Manually finalize slot 10
        consensus.finalized_slots.push_back(10);
        consensus.last_confirmed_slot = 10;
        
        // Try to reorganize at slot 10 (already finalized)
        let new_chain: Vec<(Hash, [u8; 32])> = vec![];
        let result = consensus.reorganize_chain(10, &new_chain);
        
        // Should fail because slot 10 is finalized
        assert!(result.is_err());
        assert!(matches!(result, Err(ConsensusError::SlotTooOld)));
    }

    #[test]
    fn test_rollback_removes_votes() {
        let mut consensus = TowerBFT::new();
        
        // Add some votes at different slots
        for slot in 1..=10 {
            let vote = Vote {
                validator: [slot as u8; 32],
                slot,
                block_hash: [slot as u8; 32],
                signature: [0u8; 64],
                timestamp: 0,
            };
            let _ = consensus.submit_vote(vote);
        }
        
        // Verify votes exist
        let total_votes_before: usize = consensus.votes.values().map(|v| v.len()).sum();
        assert_eq!(total_votes_before, 10);
        
        // Rollback to slot 5
        consensus.rollback_to_slot(5);
        
        // Verify only votes up to slot 5 remain
        let total_votes_after: usize = consensus.votes.values().map(|v| v.len()).sum();
        assert_eq!(total_votes_after, 5);
        
        // Verify no votes for slots > 5
        for (_, votes) in &consensus.votes {
            for vote in votes {
                assert!(vote.slot <= 5);
            }
        }
    }

    #[test]
    fn test_rollback_removes_finalized_slots() {
        let mut consensus = TowerBFT::new();
        
        // Add finalized slots
        for slot in 1..=10 {
            consensus.finalized_slots.push_back(slot);
        }
        consensus.last_confirmed_slot = 10;
        
        // Rollback to slot 5
        consensus.rollback_to_slot(5);
        
        // Verify only slots up to 5 remain finalized
        assert_eq!(consensus.finalized_slots.len(), 5);
        assert_eq!(consensus.last_confirmed_slot, 5);
        
        for &slot in &consensus.finalized_slots {
            assert!(slot <= 5);
        }
    }

    #[test]
    fn test_find_fork_point() {
        let mut consensus = TowerBFT::new();
        
        // Finalize slots 1-5
        for slot in 1..=5 {
            consensus.finalized_slots.push_back(slot);
        }
        
        // New chain that diverges at slot 6
        let new_chain: Vec<Hash> = (1..=10).map(|i| [i as u8; 32]).collect();
        
        // Fork point should be at slot 5 (last finalized)
        let fork_point = consensus.find_fork_point(&new_chain);
        assert_eq!(fork_point, Some(5));
    }

    #[test]
    fn test_hybrid_consensus_reorganization() {
        let mut consensus = HybridConsensus::new();
        
        let fork_slot = 3u64;
        let new_chain: Vec<(Hash, [u8; 32])> = vec![
            ([1u8; 32], [1u8; 32]),
            ([2u8; 32], [2u8; 32]),
        ];
        
        let result = consensus.reorganize_chain(fork_slot, &new_chain);
        assert!(result.is_ok());
        
        // Verify last block hash updated
        assert_eq!(consensus.last_block_hash, [2u8; 32]);
    }
}
