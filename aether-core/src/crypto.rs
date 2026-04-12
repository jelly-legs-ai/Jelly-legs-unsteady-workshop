//! Cryptographic utilities for AETHER

use sha2::{Sha256, Digest};

/// Hash data using SHA-256
pub fn hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash two hashes together
pub fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(a);
    hasher.update(b);
    hasher.finalize().into()
}

/// Verify Ed25519 signature
pub fn verify_signature(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), CryptoError> {
    use ed25519_dalek::{VerifyingKey, Signature as EdSignature, Verifier};
    
    let verifying_key = VerifyingKey::from_bytes(public_key)
        .map_err(|_| CryptoError::InvalidPublicKey)?;
    let sig = EdSignature::from_bytes(signature);
    
    verifying_key.verify(message, &sig)
        .map_err(|_| CryptoError::InvalidSignature)
}

/// Crypto errors
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid signature")]
    InvalidSignature,
}