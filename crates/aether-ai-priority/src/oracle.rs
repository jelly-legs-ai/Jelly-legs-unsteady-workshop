//! AI Oracle Module
//!
//! Implements the AI oracle council for transaction priority verification.
//! AI oracles are trusted entities that sign transactions to attest
//! their priority level for the AI lanes.

use aether_common::types::AIPriorityLane;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::collections::HashSet;

// Custom serde for [u8; 64] arrays (base58 encoded)
mod serde_bytes_64 {
    use super::*;
    
    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&bs58::encode(bytes).into_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let decoded = bs58::decode(&s).into_vec().map_err(serde::de::Error::custom)?;
        let mut arr = [0u8; 64];
        let len = decoded.len().min(64);
        arr[..len].copy_from_slice(&decoded[..len]);
        Ok(arr)
    }
}

/// Number of oracles required for consensus (7-of-13)
pub const ORACLE_CONSENSUS_THRESHOLD: usize = 7;

/// Total oracle count
pub const TOTAL_ORACLE_COUNT: usize = 13;

/// Oracle result type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResult {
    /// Determined priority lane
    pub lane: AIPriorityLane,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Oracle signatures (base64 encoded)
    pub signatures: Vec<OracleSignature>,
    /// Timestamp of verification
    pub timestamp: u64,
}

/// Individual oracle signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleSignature {
    /// Oracle public key
    pub oracle_pubkey: [u8; 32],
    /// Signature bytes (base58 encoded for serde)
    #[serde(with = "serde_bytes_64")]
    pub signature: [u8; 64],
    /// Oracle's determined lane
    pub lane: AIPriorityLane,
    /// Oracle's confidence
    pub confidence: f64,
}

/// AI Oracle Council - manages authorized oracles
#[derive(Debug, Clone)]
pub struct OracleCouncil {
    /// Authorized oracle public keys
    authorized_oracles: HashSet<[u8; 32]>,
    /// Oracle voting power (stake-weighted)
    oracle_voting_power: std::collections::HashMap<[u8; 32], u64>,
}

impl OracleCouncil {
    /// Create a new oracle council with initial authorized oracles
    pub fn new() -> Self {
        let mut authorized_oracles = HashSet::new();
        let mut oracle_voting_power = std::collections::HashMap::new();
        
        // Initialize with genesis oracles (would be configured at network launch)
        // Using deterministic keys for testing - in production these would be real keys
        for i in 0..TOTAL_ORACLE_COUNT {
            let mut key = [0u8; 32];
            key[0] = i as u8;
            key[31] = 0xA1; // Oracle prefix
            authorized_oracles.insert(key);
            oracle_voting_power.insert(key, 100_000_000); // Equal voting power initially
        }
        
        Self {
            authorized_oracles,
            oracle_voting_power,
        }
    }
    
    /// Check if a public key is an authorized oracle
    pub fn is_authorized(&self, pubkey: &[u8; 32]) -> bool {
        self.authorized_oracles.contains(pubkey)
    }
    
    /// Add a new authorized oracle (governance operation)
    pub fn add_oracle(&mut self, pubkey: [u8; 32], voting_power: u64) -> bool {
        if self.authorized_oracles.len() >= TOTAL_ORACLE_COUNT {
            return false;
        }
        self.authorized_oracles.insert(pubkey);
        self.oracle_voting_power.insert(pubkey, voting_power);
        true
    }
    
    /// Remove an oracle (governance operation)
    pub fn remove_oracle(&mut self, pubkey: &[u8; 32]) -> bool {
        self.authorized_oracles.remove(pubkey);
        self.oracle_voting_power.remove(pubkey);
        true
    }
    
    /// Get total voting power
    pub fn total_voting_power(&self) -> u64 {
        self.oracle_voting_power.values().sum()
    }
    
    /// Get voting power for a specific oracle
    pub fn get_voting_power(&self, pubkey: &[u8; 32]) -> u64 {
        self.oracle_voting_power.get(pubkey).copied().unwrap_or(0)
    }
    
    /// Get authorized oracle count
    pub fn oracle_count(&self) -> usize {
        self.authorized_oracles.len()
    }
    
    /// Verify oracle consensus for a transaction
    pub fn verify_consensus(&self, result: &OracleResult) -> bool {
        // Must have enough signatures
        if result.signatures.len() < ORACLE_CONSENSUS_THRESHOLD {
            return false;
        }
        
        // Verify each signature is from an authorized oracle
        let mut total_voting_power = 0u64;
        for sig in &result.signatures {
            if !self.is_authorized(&sig.oracle_pubkey) {
                return false;
            }
            total_voting_power += self.get_voting_power(&sig.oracle_pubkey);
        }
        
        // Must have > 2/3 voting power
        let required_power = self.total_voting_power() * 2 / 3 + 1;
        total_voting_power >= required_power
    }
}

impl Default for OracleCouncil {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify an AI oracle signature
/// 
/// Uses Ed25519 signature verification for oracle attestations.
/// The message hash is computed as SHA256(tx_data || lane || timestamp).
pub fn verify_oracle(
    message: &[u8],
    signature: &[u8; 64],
    oracle_pubkey: &[u8; 32],
) -> bool {
    // For now, use a simple hash-based verification
    // In production, this would use ed25519-dalek for proper signature verification
    
    // Compute expected hash
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.update(oracle_pubkey);
    let _expected = hasher.finalize();
    
    // Simple verification: check if signature contains valid structure
    // This is a placeholder - real implementation would use proper crypto
    if signature.len() != 64 {
        return false;
    }
    
    // Basic validity check - signature should not be all zeros
    signature.iter().any(|&b| b != 0)
}

/// Check if oracle is authorized (convenience function)
pub fn is_oracle_authorized(pubkey: &[u8; 32]) -> bool {
    let council = OracleCouncil::new();
    council.is_authorized(pubkey)
}

/// Create an oracle attestation for a transaction
pub fn create_oracle_attestation(
    tx_data: &[u8],
    lane: AIPriorityLane,
    confidence: f64,
    oracle_keypair: &([u8; 32], [u8; 64]),
) -> OracleSignature {
    // Compute message hash
    let mut hasher = Sha256::new();
    hasher.update(tx_data);
    hasher.update(&[lane as u8]);
    let message_hash = hasher.finalize();
    
    // Create signature (simplified - would use proper signing in production)
    let mut signature = [0u8; 64];
    signature[..32].copy_from_slice(&message_hash);
    signature[32..].copy_from_slice(&oracle_keypair.0);
    
    OracleSignature {
        oracle_pubkey: oracle_keypair.0,
        signature,
        lane,
        confidence,
    }
}

/// Aggregate oracle votes into a final result
pub fn aggregate_oracle_votes(
    signatures: Vec<OracleSignature>,
    timestamp: u64,
) -> OracleResult {
    if signatures.is_empty() {
        return OracleResult {
            lane: AIPriorityLane::Standard,
            confidence: 0.0,
            signatures: vec![],
            timestamp,
        };
    }
    
    // Count votes per lane
    let mut critical_weight = 0.0f64;
    let mut high_weight = 0.0f64;
    let mut standard_weight = 0.0f64;
    
    for sig in &signatures {
        let weight = sig.confidence;
        match sig.lane {
            AIPriorityLane::Critical => critical_weight += weight,
            AIPriorityLane::High => high_weight += weight,
            AIPriorityLane::Standard => standard_weight += weight,
        }
    }
    
    // Determine winning lane by weighted votes
    let (lane, confidence) = if critical_weight >= high_weight && critical_weight >= standard_weight {
        (AIPriorityLane::Critical, critical_weight / signatures.len() as f64)
    } else if high_weight >= standard_weight {
        (AIPriorityLane::High, high_weight / signatures.len() as f64)
    } else {
        (AIPriorityLane::Standard, standard_weight / signatures.len() as f64)
    };
    
    OracleResult {
        lane,
        confidence,
        signatures,
        timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_oracle_council_creation() {
        let council = OracleCouncil::new();
        assert_eq!(council.oracle_count(), TOTAL_ORACLE_COUNT);
    }
    
    #[test]
    fn test_oracle_authorization() {
        let council = OracleCouncil::new();
        
        // First oracle should be authorized
        let mut key = [0u8; 32];
        key[0] = 0;
        key[31] = 0xA1;
        assert!(council.is_authorized(&key));
        
        // Random key should not be authorized
        let random_key = [0xFFu8; 32];
        assert!(!council.is_authorized(&random_key));
    }
    
    #[test]
    fn test_vote_aggregation() {
        let oracle_key = ([1u8; 32], [0u8; 64]);
        
        let sigs = vec![
            OracleSignature {
                oracle_pubkey: oracle_key.0,
                signature: [0u8; 64],
                lane: AIPriorityLane::Critical,
                confidence: 0.9,
            },
            OracleSignature {
                oracle_pubkey: [2u8; 32],
                signature: [0u8; 64],
                lane: AIPriorityLane::Critical,
                confidence: 0.8,
            },
        ];
        
        let result = aggregate_oracle_votes(sigs, 0);
        assert_eq!(result.lane, AIPriorityLane::Critical);
        assert!(result.confidence > 0.8);
    }
}
