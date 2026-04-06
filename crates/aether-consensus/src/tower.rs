//! Tower Consensus Implementation
//!
//! Implements a fork choice rule based on "tower" of confirmed blocks,
//! similar to Solana's Tower BFT consensus mechanism.

use crate::{ConsensusError, ConsensusResult};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Vote in the tower
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TowerVote {
    /// Slot being voted on
    pub slot: u64,
    /// Confirmation count (how many descendants confirmed)
    pub confirmation_count: u32,
    /// Timestamp of vote
    pub timestamp: u64,
}

/// Validator vote state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidatorVoteState {
    /// Tower of votes (oldest to newest)
    pub votes: Vec<TowerVote>,
    /// Root slot (last confirmed block)
    pub root_slot: u64,
    /// Whether validator has voted on current fork
    pub has_voted: bool,
    /// Set of slots this validator has already voted on (prevents double voting)
    pub voted_slots: std::collections::HashSet<u64>,
}

/// Tower consensus state
#[derive(Debug)]
pub struct TowerConsensus {
    /// Validator pubkey -> vote state
    validator_votes: HashMap<[u8; 32], ValidatorVoteState>,
    /// Slot -> set of validators who voted
    slot_votes: HashMap<u64, HashSet<[u8; 32]>>,
    /// Confirmation depth for finality (32 votes = ~12.8s)
    pub confirmation_depth: u32,
    /// Maximum votes to keep in tower (must be > confirmation_depth to allow votes to mature)
    max_tower_size: usize,
    /// Stake weight per slot
    slot_stake: HashMap<u64, u64>,
}

impl TowerConsensus {
    pub fn new() -> Self {
        Self {
            validator_votes: HashMap::new(),
            slot_votes: HashMap::new(),
            confirmation_depth: 32,
            max_tower_size: 64, // Keep twice the confirmation depth to allow votes to mature
            slot_stake: HashMap::new(),
        }
    }

    /// Process a vote from a validator
    pub fn process_vote(
        &mut self,
        validator: [u8; 32],
        slot: u64,
        stake: u64,
    ) -> ConsensusResult<()> {
        let vote_state = self.validator_votes.entry(validator).or_default();

        // CRITICAL: Prevent double voting - reject if already voted on this slot
        if vote_state.voted_slots.contains(&slot) {
            return Err(ConsensusError::TowerError(
                format!("Validator {:?} already voted on slot {} - double voting detected", validator, slot)
            ));
        }

        // Check if vote is on top of current tower
        if let Some(last_vote) = vote_state.votes.last() {
            if slot <= last_vote.slot {
                return Err(ConsensusError::TowerError(
                    format!("Vote slot {} must be greater than last vote {}", slot, last_vote.slot)
                ));
            }
        }

        // Pop expired votes when tower exceeds max size (but keep voted_slots for double-vote detection)
        while vote_state.votes.len() >= self.max_tower_size {
            let _ = vote_state.votes.remove(0);
            // Don't remove from voted_slots - we need to remember all votes to prevent double voting
        }

        // Add new vote
        let vote = TowerVote {
            slot,
            confirmation_count: 0,
            timestamp: current_timestamp(),
        };
        vote_state.votes.push(vote);
        vote_state.voted_slots.insert(slot);
        vote_state.has_voted = true;

        // Record vote
        self.slot_votes.entry(slot).or_default().insert(validator);
        *self.slot_stake.entry(slot).or_insert(0) += stake;

        // Update confirmation counts
        self.update_confirmations();

        Ok(())
    }

    /// Update confirmation counts for all votes
    fn update_confirmations(&mut self) {
        for (_validator, state) in &mut self.validator_votes {
            for i in 0..state.votes.len() {
                let slot = state.votes[i].slot;
                
                // Count how many votes in tower are descendants of this slot
                let confirmations = state.votes[i + 1..].iter()
                    .filter(|v| v.slot > slot)
                    .count() as u32;
                
                state.votes[i].confirmation_count = confirmations;
            }

            // Update root if confirmed
            for vote in &state.votes {
                if vote.confirmation_count >= self.confirmation_depth {
                    state.root_slot = state.root_slot.max(vote.slot);
                }
            }
        }
    }

    /// Check if a slot is confirmed (has > 2/3 stake weight in votes)
    pub fn is_slot_confirmed(
        &self,
        slot: u64,
        total_stake: u64,
    ) -> bool {
        let slot_stake = self.slot_stake.get(&slot).copied().unwrap_or(0);
        
        // Need strictly more than 2/3 for confirmation (Byzantine fault tolerance)
        // Formula: threshold = floor(2/3 * total_stake) + 1
        // This ensures slot_stake must be > 2/3, not just >= truncated 2/3
        // Example: total_stake=100 → threshold=67, need 68 (>66.67)
        //          total_stake=3   → threshold=2, need 3 (>2)
        let threshold = (total_stake * 2) / 3 + 1;
        slot_stake >= threshold
    }

    /// Get confirmation depth for a slot
    pub fn get_confirmation_depth(&self,
        slot: u64,
    ) -> u32 {
        self.slot_votes.get(&slot)
            .map(|validators| {
                validators.iter()
                    .filter_map(|v| self.validator_votes.get(v))
                    .filter_map(|state| {
                        state.votes.iter()
                            .find(|vote| vote.slot == slot)
                            .map(|vote| vote.confirmation_count)
                    })
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    /// Get root slot (latest confirmed block)
    pub fn get_root_slot(&self) -> u64 {
        self.validator_votes.values()
            .map(|state| state.root_slot)
            .max()
            .unwrap_or(0)
    }

    /// Get stake weight for a slot
    pub fn get_slot_stake(&self, slot: u64) -> u64 {
        self.slot_stake.get(&slot).copied().unwrap_or(0)
    }

    /// Check if a fork is valid (descends from root)
    pub fn is_valid_fork(&self,
        proposed_slot: u64,
        ancestor_slots: &[u64],
    ) -> bool {
        let root = self.get_root_slot();
        
        // Proposed slot must be > root
        if proposed_slot <= root {
            return false;
        }

        // Must have root in ancestors
        ancestor_slots.contains(&root)
    }

    /// Get validators who voted for a slot
    pub fn get_voters(&self, slot: u64) -> Vec<&[u8; 32]> {
        self.slot_votes.get(&slot)
            .map(|set: &HashSet<[u8; 32]>| set.iter().collect())
            .unwrap_or_default()
    }
}

impl Default for TowerConsensus {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tower_voting() {
        let mut tower = TowerConsensus::new();
        let validator = [1u8; 32];
        let stake = 1000;

        // Vote on slots 1-5
        for slot in 1..=5 {
            tower.process_vote(validator, slot, stake).unwrap();
        }

        let state = tower.validator_votes.get(&validator).unwrap();
        assert_eq!(state.votes.len(), 5);
        assert_eq!(state.votes.last().unwrap().slot, 5);
    }

    #[test]
    fn test_slot_confirmation() {
        let mut tower = TowerConsensus::new();
        let total_stake = 3000;

        // 3 validators with 1000 stake each
        let v1 = [1u8; 32];
        let v2 = [2u8; 32];
        let v3 = [3u8; 32];

        // All vote for slot 1
        tower.process_vote(v1, 1, 1000).unwrap();
        tower.process_vote(v2, 1, 1000).unwrap();
        
        // Should not be confirmed yet (only 2/3)
        assert!(!tower.is_slot_confirmed(1, total_stake));

        // Third validator votes
        tower.process_vote(v3, 1, 1000).unwrap();
        
        // Now confirmed
        assert!(tower.is_slot_confirmed(1, total_stake));
    }

    #[test]
    fn test_fork_validity() {
        let mut tower = TowerConsensus::new();
        
        // Use 3 validators to achieve 2/3 threshold
        let v1 = [1u8; 32];
        let v2 = [2u8; 32];
        let v3 = [3u8; 32];
        let stake = 1000;

        // All validators vote on slot 1
        tower.process_vote(v1, 1, stake).unwrap();
        tower.process_vote(v2, 1, stake).unwrap();
        tower.process_vote(v3, 1, stake).unwrap();

        // Build confirmations - all validators vote on slots 2-33 (32 confirmations needed)
        for slot in 2..=33 {
            tower.process_vote(v1, slot, 0).unwrap();
            tower.process_vote(v2, slot, 0).unwrap();
            tower.process_vote(v3, slot, 0).unwrap();
        }

        // Root should be set after 32 confirmations
        let root = tower.get_root_slot();
        assert!(root >= 1);

        // Valid fork descends from root
        assert!(tower.is_valid_fork(root + 10, &[root, root + 5, root + 10]));
        
        // Invalid fork doesn't include root
        assert!(!tower.is_valid_fork(100, &[50, 75, 100]));
    }

    #[test]
    fn test_double_vote_prevention() {
        let mut tower = TowerConsensus::new();
        let validator = [1u8; 32];
        let stake = 1000;

        // First vote should succeed
        let result1 = tower.process_vote(validator, 5, stake);
        assert!(result1.is_ok());

        // Second vote on same slot should fail (double voting)
        let result2 = tower.process_vote(validator, 5, stake);
        assert!(result2.is_err());
        let err_msg = result2.unwrap_err().to_string();
        assert!(err_msg.contains("already voted"));
        assert!(err_msg.contains("double voting"));

        // Vote on different slot should succeed
        let result3 = tower.process_vote(validator, 6, stake);
        assert!(result3.is_ok());
    }

    #[test]
    fn test_double_vote_different_validators() {
        let mut tower = TowerConsensus::new();
        let v1 = [1u8; 32];
        let v2 = [2u8; 32];
        let stake = 1000;

        // Both validators can vote on same slot (this is normal)
        assert!(tower.process_vote(v1, 5, stake).is_ok());
        assert!(tower.process_vote(v2, 5, stake).is_ok());

        // But each validator can only vote once per slot
        assert!(tower.process_vote(v1, 5, stake).is_err());
        assert!(tower.process_vote(v2, 5, stake).is_err());
    }
}
