//! Hybrid Proof-of-Work / Proof-of-Stake Consensus
//!
//! Allows mobile/work nodes to participate via PoW while
//! home validators secure the network via PoS.

use crate::types::{Block, Address, Hash};
use crate::error::AetherError;

/// PoW reward per submitted proof (AETH)
pub const POW_REWARD: u64 = 1;

/// PoS validator reward per validated block (AETH)
pub const POS_VALIDATOR_REWARD: u64 = 10;

/// Hybrid consensus engine combining PoW and PoS mechanisms
pub struct HybridConsensus {
    /// Minimum PoW difficulty (log2)
    pub pow_difficulty: u32,
    /// PoS validator stake threshold (AETH)
    pub pos_stake_threshold: u64,
    /// Registered PoS validators (address -> stake amount)
    validators: std::collections::HashMap<Address, u64>,
    /// Pending PoW submissions awaiting inclusion
    pending_pow: Vec<PowSubmission>,
}

/// A pending PoW proof submitted by a mobile/work node
#[derive(Debug, Clone)]
pub struct PowSubmission {
    pub node_address: Address,
    pub block_hash: Hash,
    pub proof: Vec<u8>,
    pub timestamp: u64,
}

impl HybridConsensus {
    /// Create a new hybrid consensus engine
    pub fn new(pow_difficulty: u32, pos_stake_threshold: u64) -> Self {
        Self {
            pow_difficulty,
            pos_stake_threshold,
            validators: std::collections::HashMap::new(),
            pending_pow: Vec::new(),
        }
    }

    /// Submit a PoW proof from a mobile or work node.
    ///
    /// Lightweight nodes can participate in consensus without
    /// running a full validator. Returns the reward if valid.
    pub fn submit_pow_proof(
        &mut self,
        node_address: Address,
        block_hash: Hash,
        proof: Vec<u8>,
        timestamp: u64,
    ) -> Result<u64, AetherError> {
        // Verify the PoW proof meets difficulty target
        let proof_hash = Self::hash_proof(&node_address, &block_hash, &proof, timestamp);
        
        if !Self::check_difficulty(&proof_hash, self.pow_difficulty) {
            return Err(AetherError::InvalidProof {
                reason: format!("Proof does not meet difficulty {}", self.pow_difficulty),
            });
        }

        // Store the valid submission for later inclusion
        let submission = PowSubmission {
            node_address,
            block_hash,
            proof,
            timestamp,
        };

        self.pending_pow.push(submission);

        // Reward the node with POW_REWARD AETH
        Ok(POW_REWARD)
    }

    /// Validate a block as a home validator using PoS.
    ///
    /// Validators stake AETH and earn POS_VALIDATOR_REWARD
    /// for each block they validate and sign.
    pub fn validate_block(
        &self,
        validator_address: &Address,
        block: &Block,
        signature: &[u8],
    ) -> Result<u64, AetherError> {
        // Check the validator is registered and has sufficient stake
        let stake = self
            .validators
            .get(validator_address)
            .ok_or(AetherError::NotAValidator {
                address: *validator_address,
            })?;

        if *stake < self.pos_stake_threshold {
            return Err(AetherError::InsufficientStake {
                required: self.pos_stake_threshold,
                actual: *stake,
            });
        }

        // Verify the validator's signature on the block
        let block_data = Self::serialize_block(block);
        if !Self::verify_signature(validator_address, &block_data, signature) {
            return Err(AetherError::InvalidSignature {
                signer: *validator_address,
            });
        }

        // Block is valid — reward the validator
        Ok(POS_VALIDATOR_REWARD)
    }

    /// Register a new PoS validator
    pub fn register_validator(&mut self, address: Address, stake: u64) -> Result<(), AetherError> {
        if stake < self.pos_stake_threshold {
            return Err(AetherError::InsufficientStake {
                required: self.pos_stake_threshold,
                actual: stake,
            });
        }
        self.validators.insert(address, stake);
        Ok(())
    }

    /// Get the current number of registered PoS validators
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }

    /// Check if an address is a registered validator
    pub fn is_validator(&self, address: &Address) -> bool {
        self.validators.contains_key(address)
    }

    // --- Internal helpers ---

    fn hash_proof(node: &Address, block: &Hash, proof: &[u8], ts: u64) -> Hash {
        use crate::crypto::fast_hash;
        let mut data = Vec::with_capacity(32 + 32 + proof.len() + 8);
        data.extend_from_slice(node);
        data.extend_from_slice(block);
        data.extend_from_slice(proof);
        data.extend_from_slice(&ts.to_le_bytes());
        fast_hash(&data)
    }

    fn check_difficulty(hash: &Hash, difficulty: u32) -> bool {
        // Leading bytes must be zero; difficulty is log2(leading_zero_bytes)
        let leading_zeros = hash.iter().take_while(|&&b| b == 0).count();
        let required_zeros = (difficulty / 8) as usize;
        leading_zeros >= required_zeros
    }

    fn serialize_block(block: &Block) -> Vec<u8> {
        // Minimal serialization for signing
        let mut data = Vec::new();
        data.extend_from_slice(&block.height.to_le_bytes());
        data.extend_from_slice(&block.prev_hash);
        data.extend_from_slice(&block.merkle_root);
        data.extend_from_slice(&block.timestamp.to_le_bytes());
        data
    }

    fn verify_signature(address: &Address, data: &[u8], signature: &[u8]) -> bool {
        crate::crypto::verify_signature(address, data, signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_validate() {
        let mut consensus = HybridConsensus::new(16, 1000);
        let validator = Address::random();
        
        consensus.register_validator(validator, 5000).unwrap();
        assert!(consensus.is_validator(&validator));
        assert_eq!(consensus.validator_count(), 1);
    }

    #[test]
    fn test_insufficient_stake() {
        let mut consensus = HybridConsensus::new(16, 1000);
        let validator = Address::random();
        
        let result = consensus.register_validator(validator, 500);
        assert!(result.is_err());
    }
}
