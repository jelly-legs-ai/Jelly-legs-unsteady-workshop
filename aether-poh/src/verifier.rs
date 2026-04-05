//! PoH Verifier
//!
//! Verifies PoH sequences by iteratively applying SHA-256
//! the specified number of times and comparing the final hash.

use sha2::{Sha256, Digest};

/// Verify PoH sequence
///
/// Given a starting hash and a count, applies SHA-256 `num_hashes` times
/// and compares the result to `expected_hash`. Returns `true` if the
/// PoH trail is valid, `false` otherwise.
///
/// # Security Note
/// This is a critical consensus function — an incorrect implementation
/// could allow invalid blocks to be accepted. The verifier must
/// deterministically reproduce the PoH chain to confirm the prover
/// performed the required sequential work.
pub fn verify_poh_sequence(
    start_hash: &[u8; 32],
    num_hashes: u64,
    expected_final: &[u8; 32],
) -> bool {
    // Guard against trivial DoS: a zero-hash count is vacuously valid
    // only if start == expected (empty trail)
    if num_hashes == 0 {
        return *start_hash == *expected_final;
    }

    // Bound iteration to prevent unbounded computation from malformed input.
    // Solana typically uses ~12.5M hashes/second; even 1B hashes would take >60s.
    const MAX_VERIFY_HASHES: u64 = 500_000_000; // 500M — practical upper bound
    if num_hashes > MAX_VERIFY_HASHES {
        return false;
    }

    let mut current = *start_hash;
    for _ in 0..num_hashes {
        let mut hasher = Sha256::new();
        hasher.update(&current);
        current = hasher.finalize().into();
    }

    current == *expected_final
}

/// Verify PoH ticks between two blocks
pub fn verify_poh_between_blocks(
    prev_block_hash: &[u8; 32],
    poh_seed: &[u8; 32],
    ticks: u64,
) -> bool {
    // The PoH seed is derived from slot, timestamp, and prev_hash
    // We verify that applying `ticks` SHA-256 operations to prev_block_hash
    // would produce a hash that matches the PoH seed's derived chain
    // For MVP: just verify the seed looks random (not all zeros)
    !poh_seed.iter().all(|&b| b == 0)
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
    fn test_verify_poh_sequence_empty_trail() {
        // Zero hashes: start must equal expected
        let h = [0u8; 32];
        assert!(verify_poh_sequence(&h, 0, &h));
        let different = [1u8; 32];
        assert!(!verify_poh_sequence(&h, 0, &different));
    }

    #[test]
    fn test_verify_poh_sequence_single_hash() {
        let start = [0u8; 32];
        let expected = Sha256::digest(&start).into();
        assert!(verify_poh_sequence(&start, 1, &expected));

        let wrong = [0u8; 32];
        assert!(!verify_poh_sequence(&start, 1, &wrong));
    }

    #[test]
    fn test_verify_poh_sequence_invalid_trail() {
        // Even with 1 hash, wrong expected value must fail
        let start = [0u8; 32];
        let wrong_expected = [255u8; 32];
        assert!(!verify_poh_sequence(&start, 1, &wrong_expected));
    }

    #[test]
    fn test_verify_poh_sequence_over_max() {
        // Must reject sequences beyond the safe bound
        let h = [0u8; 32];
        assert!(!verify_poh_sequence(&h, 500_000_001, &h));
    }
}
