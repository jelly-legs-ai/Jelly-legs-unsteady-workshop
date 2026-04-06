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
    let mut nonce: u64 = 0;

    loop {
        let mut hasher = Sha256::new();
        hasher.update(target_data);
        hasher.update(&nonce.to_le_bytes());
        let result = hasher.finalize();
        let hash: [u8; 32] = result.try_into().unwrap_or([0u8; 32]);

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

/// Generate a proof of work by finding a nonce that produces a hash
/// meeting the difficulty target.
///
/// Difficulty is specified in bits (0-256). Higher difficulty = more leading zeros.
/// For example, difficulty=16 means the hash must have at least 16 leading zero bits.
///
/// Returns ProofResult with the found hash, nonce, and difficulty, or None if
/// no valid nonce is found within the search limit.
pub fn generate_proof_with_limit(data: &[u8], difficulty: u32, max_nonce: u64) -> Option<ProofResult> {
    // Cap difficulty at 256 for SHA-256
    let difficulty = difficulty.min(256);
    
    for nonce in 0..max_nonce {
        // Hash data + nonce
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(&nonce.to_le_bytes());
        let hash: [u8; 32] = hasher.finalize().into();
        
        // Check if hash meets difficulty
        if verify_proof(&hash, difficulty) {
            return Some(ProofResult {
                hash,
                nonce,
                difficulty,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
    }
    
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_proof_low_difficulty() {
        // Low difficulty should find a proof quickly
        let data = b"test_data";
        let result = generate_proof(data, 8, 100_000);
        
        assert!(result.is_some(), "Should find proof for difficulty 8");
        let proof = result.expect("Failed to generate proof");
        assert!(verify_proof(&proof.hash, proof.difficulty));
        assert!(proof.nonce < 100_000);
    }

    #[test]
    fn test_generate_proof_medium_difficulty() {
        // Medium difficulty should still find a proof
        let data = b"blockchain_test";
        let result = generate_proof(data, 16, 1_000_000);
        
        assert!(result.is_some(), "Should find proof for difficulty 16");
        let proof = result.expect("Failed to generate proof for medium difficulty");
        assert!(verify_proof(&proof.hash, proof.difficulty));
    }

    #[test]
    fn test_generate_proof_no_solution() {
        // Very high difficulty with low max_nonce should fail
        let data = b"impossible_proof";
        let result = generate_proof(data, 256, 100);
        
        assert!(result.is_none(), "Should not find proof for impossible difficulty");
    }

    #[test]
    fn test_generate_verify_roundtrip() {
        // Generate a proof and verify it
        let data = b"roundtrip_test";
        let result = generate_proof(data, 12, 500_000);
        
        assert!(result.is_some());
        let proof = result.expect("Failed to generate proof for roundtrip test");
        
        // Verify the proof is valid
        assert!(verify_proof(&proof.hash, proof.difficulty));
        
        // Verify hash matches what we'd compute
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(&proof.nonce.to_le_bytes());
        let expected_hash: [u8; 32] = hasher.finalize().into();
        assert_eq!(proof.hash, expected_hash);
    }

    #[test]
    fn test_generate_proof_deterministic() {
        // Same input should produce same output
        let data = b"deterministic_test";
        let result1 = generate_proof(data, 10, 500_000);
        let result2 = generate_proof(data, 10, 500_000);
        
        assert!(result1.is_some());
        assert!(result2.is_some());
        
        let proof1 = result1.expect("Failed to generate proof (run 1)");
        let proof2 = result2.expect("Failed to generate proof (run 2)");
        
        assert_eq!(proof1.nonce, proof2.nonce);
        assert_eq!(proof1.hash, proof2.hash);
    }

    #[test]
    fn test_verify_proof_edge_cases() {
        // Test difficulty 0 (any hash is valid)
        let hash = [0xFFu8; 32];
        assert!(verify_proof(&hash, 0));

        // Test difficulty 256 (only all-zeros hash is valid)
        let zeros = [0u8; 32];
        assert!(verify_proof(&zeros, 256));
        
        let non_zero = [1u8; 32];
        assert!(!verify_proof(&non_zero, 256));
    }
}
