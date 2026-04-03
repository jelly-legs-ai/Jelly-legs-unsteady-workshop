//! AETHER Proof Engine
//!
//! Proof generation and verification for AetherFlow consensus.

use serde::{Deserialize, Serialize};

/// Proof of Work result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResult {
    pub hash: [u8; 32],
    pub nonce: u64,
    pub difficulty: u32,
}

/// Verify a proof meets difficulty target
pub fn verify_proof(hash: &[u8; 32], difficulty: u32) -> bool {
    // Count leading zero bytes
    let leading_zeros = hash.iter().take_while(|&&b| b == 0).count();
    (leading_zeros as u32) >= difficulty / 8
}
