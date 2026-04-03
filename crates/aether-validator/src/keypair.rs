//! Keypair management for validator identity
//!
//! Handles generation, loading, and saving of Ed25519 keypairs
//! for validator identity and vote accounts.

use anyhow::{Context, Result};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

/// Validator identity keypair
#[derive(Clone)]
pub struct ValidatorIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl ValidatorIdentity {
    /// Get the public key as a hex string
    pub fn pubkey(&self) -> String {
        bs58::encode(self.verifying_key.as_bytes()).into_string()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }
}

/// Vote account keypair with validator reference
#[derive(Serialize, Deserialize)]
struct VoteAccountJson {
    pubkey: String,
    validator_pubkey: String,
    commission: u8,
}

/// Generate a new Ed25519 keypair
pub fn generate_keypair() -> ValidatorIdentity {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let signing_key = SigningKey::from_bytes(&bytes);
    let verifying_key = signing_key.verifying_key();
    
    ValidatorIdentity {
        signing_key,
        verifying_key,
    }
}

/// Save identity to a JSON file
pub fn save_identity(path: &std::path::Path, identity: &ValidatorIdentity) -> Result<()> {
    let json = serde_json::json!({
        "pubkey": identity.pubkey(),
        "secret": bs58::encode(identity.signing_key.to_bytes()).into_string(),
    });
    
    std::fs::write(path, serde_json::to_string_pretty(&json)?)
        .context("Failed to write identity file")?;
    
    info!("Saved identity to {}", path.display());
    Ok(())
}

/// Load identity from a JSON file
pub fn load_identity(path: &Path) -> Result<ValidatorIdentity> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read identity file")?;
    
    let json: serde_json::Value = serde_json::from_str(&content)
        .context("Invalid identity file format")?;
    
    let secret = json["secret"]
        .as_str()
        .context("Missing secret in identity file")?;
    
    let bytes: Vec<u8> = bs58::decode(secret)
        .into_vec()
        .context("Invalid base58 in identity file")?;
    
    let signing_key = SigningKey::from_bytes(&bytes[..32].try_into()?);
    let verifying_key = signing_key.verifying_key();
    
    Ok(ValidatorIdentity {
        signing_key,
        verifying_key,
    })
}

/// Load identity or create new one if file doesn't exist
pub fn load_or_create_identity(path: &std::path::Path) -> Result<ValidatorIdentity> {
    if path.exists() {
        load_identity(path)
    } else {
        let identity = generate_keypair();
        save_identity(path, &identity)?;
        Ok(identity)
    }
}

/// Save vote account to a JSON file
pub fn save_vote_account(
    path: &std::path::Path,
    vote_identity: &ValidatorIdentity,
    validator_pubkey: &str,
    commission: u8,
) -> Result<()> {
    let json = serde_json::json!({
        "pubkey": vote_identity.pubkey(),
        "secret": bs58::encode(vote_identity.signing_key.to_bytes()).into_string(),
        "validator_pubkey": validator_pubkey,
        "commission": commission,
        "vote_type": "single",
    });
    
    std::fs::write(path, serde_json::to_string_pretty(&json)?)
        .context("Failed to write vote account file")?;
    
    info!("Saved vote account to {}", path.display());
    Ok(())
}
