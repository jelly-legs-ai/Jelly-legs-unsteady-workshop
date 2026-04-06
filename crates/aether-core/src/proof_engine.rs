//! AETHER Proof Engine
//!
//! Proof generation and verification for AetherFlow consensus.

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// Proof of Work result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResult {
    pub hash: [u8; 32],
    pub nonce: u64,
    pub difficulty: u32,
    pub timestamp: u64,
}

/// Generate a proof meeting the given difficulty target
///
/// Uses SHA-256 hashing with incremental nonce search.
/// Returns a ProofResult with the winning hash, nonce, and difficulty.
/// The target data is typically a block header or message to be proven.
pub fn generate_proof(target_data: &[u8], difficulty: u32) -> ProofResult {
    let difficulty = difficulty.min(256);
    let mut hasher = Sha256::new();
    let mut nonce: u64 = 0;

    loop {
        hasher = Sha256::new();
        hasher.update(target_data);
        hasher.update(&nonce.to_le_bytes());
        let result = hasher.finalize();
        let hash: [u8; 32] = result.try_into().unwrap();

        if verify_proof(&hash, difficulty) {
            return ProofResult {
                hash,
                nonce,
                difficulty,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            };
        }
        nonce += 1;
    }
}

/// Verify a proof meets difficulty target
///
/// Difficulty is specified in bits. For example, difficulty=16 means
/// the hash must have at least 16 leading zero bits (i.e., the integer
/// value of the hash must be less than 2^(256-16)).
pub fn verify_proof(hash: &[u8; 32], difficulty: u32) -> bool {
    // Difficulty can't exceed 256 for SHA-256
    let difficulty = difficulty.min(256);
    
    // Number of full zero bytes before the first non-zero byte
    let full_zero_bytes = (difficulty / 8) as usize;
    // Number of leading zero bits in the first non-zero byte
    let remaining_bits = (difficulty % 8) as usize;

    // Check all full zero bytes
    if hash[..full_zero_bytes].iter().any(|&b| b != 0) {
        return false;
    }

    // If difficulty is byte-aligned, we're done
    if remaining_bits == 0 {
        return true;
    }

    // Check the next byte has enough leading zeros
    let first_byte = hash.get(full_zero_bytes).copied().unwrap_or(0);
    let required_mask = 0xFFu8 << (8 - remaining_bits);
    
    (first_byte & required_mask) == 0
}
