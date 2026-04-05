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
///
/// This function verifies that applying `ticks` SHA-256 operations to
/// `prev_block_hash` produces a result that matches `poh_seed`. This ensures
/// the prover actually performed the required sequential work between blocks.
///
/// # Security Critical
/// This is a consensus-critical function. An incorrect implementation could
/// allow validators to submit blocks with invalid PoH sequences, breaking
/// the proof-of-history guarantee.
pub fn verify_poh_between_blocks(
    prev_block_hash: &[u8; 32],
    poh_seed: &[u8; 32],
    ticks: u64,
) -> bool {
    // Guard against zero ticks - seed must equal prev hash
    if ticks == 0 {
        return *prev_block_hash == *poh_seed;
    }

    // Bound iteration to prevent DoS from malformed tick counts
    const MAX_VERIFY_TICKS: u64 = 500_000_000;
    if ticks > MAX_VERIFY_TICKS {
        return false;
    }

    // Actually verify the PoH chain by recomputing it
    let mut current = *prev_block_hash;
    for _ in 0..ticks {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&current);
        current = hasher.finalize().into();
    }

    // The computed hash must match the provided PoH seed
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

    #[test]
    fn test_verify_poh_between_blocks_zero_ticks() {
        // Zero ticks: prev hash must equal seed
        let h = [42u8; 32];
        assert!(verify_poh_between_blocks(&h, &h, 0));
        let different = [0u8; 32];
        assert!(!verify_poh_between_blocks(&h, &different, 0));
    }

    #[test]
    fn test_verify_poh_between_blocks_single_tick() {
        let start = [0u8; 32];
        let expected = Sha256::digest(&start).into();
        assert!(verify_poh_between_blocks(&start, &expected, 1));

        let wrong = [255u8; 32];
        assert!(!verify_poh_between_blocks(&start, &wrong, 1));
    }

    #[test]
    fn test_verify_poh_between_blocks_multiple_ticks() {
        let mut current = [1u8; 32];
        for _ in 0..10 {
            current = Sha256::digest(&current).into();
        }
        // current is now the result of 10 hashes from [1u8; 32]
        assert!(verify_poh_between_blocks(&[1u8; 32], &current, 10));
        assert!(!verify_poh_between_blocks(&[1u8; 32], &current, 9));
        assert!(!verify_poh_between_blocks(&[1u8; 32], &current, 11));
    }

    #[test]
    fn test_verify_poh_between_blocks_over_max() {
        let h = [0u8; 32];
        assert!(!verify_poh_between_blocks(&h, &h, 500_000_001));
    }
}
