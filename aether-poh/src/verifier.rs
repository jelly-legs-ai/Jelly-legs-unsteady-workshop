//! PoH Verifier
//!
//! Verifies PoH sequences efficiently using batch verification.

use sha2::{Sha256, Digest};

/// Maximum number of hashes to verify in a single sequence
/// Prevents DoS attacks where an attacker requests verification
/// of an extremely large sequence to exhaust CPU resources
/// Set to ~10 seconds of work at 2M hashes/sec = 20M hashes
pub const MAX_VERIFY_HASHES: u64 = 20_000_000;

/// Verify a PoH sequence from start to end
/// Given start_hash, number of hashes, and expected final hash,
/// verify the sequence is correct by recomputing
/// 
/// Returns false if num_hashes exceeds MAX_VERIFY_HASHES (DoS protection)
pub fn verify_poh_sequence(
    start_hash: &[u8; 32],
    num_hashes: u64,
    expected_final: &[u8; 32],
) -> bool {
    // DoS protection: reject sequences that are too large to verify
    if num_hashes > MAX_VERIFY_HASHES {
        return false;
    }

    let mut current = *start_hash;

    // For large sequences, use batch hashing
    // For MVP, we verify by recomputing (could be slow for very large n)
    // In production, use incremental verification or skip verification
    // for slots we don't care about
    for _ in 0..num_hashes {
        let mut hasher = Sha256::new();
        hasher.update(&current);
        current = hasher.finalize().into();
    }

    current == *expected_final
}

/// Verify PoH ticks between two blocks
/// 
/// Returns false if ticks exceeds MAX_VERIFY_HASHES (DoS protection)
pub fn verify_poh_between_blocks(
    prev_block_hash: &[u8; 32],
    poh_seed: &[u8; 32],
    ticks: u64,
) -> bool {
    // DoS protection: reject sequences that are too large to verify
    if ticks > MAX_VERIFY_HASHES {
        return false;
    }

    // The PoH seed is derived from slot, timestamp, and prev_hash
    // We verify that applying `ticks` SHA-256 operations to prev_block_hash
    // produces the poh_seed (or at least one valid hash iteration)
    // For MVP: verify at least one hash iteration to ensure chain continuity
    if ticks == 0 {
        // No ticks - poh_seed should match prev_block_hash exactly
        return poh_seed == prev_block_hash;
    }
    
    // Verify by recomputing the hash chain
    let mut current = *prev_block_hash;
    for _ in 0..ticks {
        let mut hasher = Sha256::new();
        hasher.update(&current);
        current = hasher.finalize().into();
    }
    
    current == *poh_seed
}

/// Compute the number of PoH ticks between two timestamps
pub fn compute_ticks(start_time: u64, end_time: u64, ticks_per_second: u64) -> u64 {
    let elapsed_ms = end_time.saturating_sub(start_time);
    (elapsed_ms / 1000) * ticks_per_second
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_poh_sequence_valid() {
        let start = [1u8; 32];
        // Compute a valid hash
        let mut hasher = Sha256::new();
        hasher.update(&start);
        let expected: [u8; 32] = hasher.finalize().into();
        
        assert!(verify_poh_sequence(&start, 1, &expected));
    }

    #[test]
    fn test_verify_poh_sequence_dos_protection() {
        let start = [1u8; 32];
        let fake_hash = [2u8; 32];
        
        // Should reject sequences that exceed MAX_VERIFY_HASHES
        assert!(!verify_poh_sequence(&start, MAX_VERIFY_HASHES + 1, &fake_hash));
        assert!(!verify_poh_sequence(&start, u64::MAX, &fake_hash));
    }

    #[test]
    fn test_verify_poh_between_blocks_dos_protection() {
        let prev = [1u8; 32];
        let seed = [2u8; 32];
        
        // Should reject excessive tick counts
        assert!(!verify_poh_between_blocks(&prev, &seed, MAX_VERIFY_HASHES + 1));
        assert!(!verify_poh_between_blocks(&prev, &seed, u64::MAX));
    }

    #[test]
    fn test_verify_poh_between_blocks_zero_ticks() {
        let hash = [42u8; 32];
        
        // Zero ticks means seed must match prev exactly
        assert!(verify_poh_between_blocks(&hash, &hash, 0));
        
        let different = [99u8; 32];
        assert!(!verify_poh_between_blocks(&hash, &different, 0));
    }
}
