//! AETHER Cryptographic Utilities

use sha2::{Sha256, Digest};
use ed25519_dalek::{Signature, VerifyingKey};

/// Hash data using SHA-256
pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash data using BLAKE3
pub fn hash_blake3(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Verify Ed25519 signature
pub fn verify_ed25519(
    message: &[u8],
    signature: &[u8; 64],
    public_key: &[u8; 32],
) -> Result<bool, Box<dyn std::error::Error>> {
    let pubkey = VerifyingKey::from_bytes(public_key.into())?;
    let sig = Signature::from_bytes(signature);
    Ok(pubkey.verify(message, &sig).is_ok())
}

/// Calculate Merkle root from leaves
pub fn calculate_merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    if leaves.len() == 1 {
        return leaves[0];
    }

    let mut current_level: Vec<[u8; 32]> = leaves.to_vec();
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(2) {
            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(&chunk[0]);
            
            if chunk.len() == 2 {
                combined[32..].copy_from_slice(&chunk[1]);
            } else {
                combined[32..].copy_from_slice(&chunk[0]);
            }
            
            next_level.push(hash_sha256(&combined));
        }
        
        current_level = next_level;
    }
    
    current_level[0]
}

/// Compute nullifier for privacy transactions
pub fn compute_nullifier(spending_key: &[u8; 32], commitment_index: u64) -> [u8; 32] {
    let mut data = [0u8; 40];
    data[..32].copy_from_slice(spending_key);
    data[32..].copy_from_slice(&commitment_index.to_le_bytes());
    hash_sha256(&data)
}

/// Generate commitment for shielded transaction
pub fn generate_commitment(
    amount: u64,
    randomness: &[u8; 32],
    recipient: &[u8; 32],
) -> [u8; 32] {
    let mut data = [0u8; 72];
    data[..8].copy_from_slice(&amount.to_le_bytes());
    data[8..40].copy_from_slice(randomness);
    data[40..].copy_from_slice(recipient);
    hash_sha256(&data)
}
