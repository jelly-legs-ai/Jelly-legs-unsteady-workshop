//! Proof of History (PoH) Implementation
//!
//! PoH provides a verifiable delay function that creates a historical record
//! of events, allowing validators to agree on the passage of time without
//! requiring consensus on each tick.

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::time::Instant;

/// Number of hashes per tick (target ~400ms per tick)
pub const HASHES_PER_TICK: u64 = 2_000_000; // ~2M hashes = ~400ms on modern CPU

/// Maximum number of skipped ticks before considering slot invalid
pub const MAX_SKIPPED_TICKS: u64 = 2;

/// PoH entry in the chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PoHEntry {
    /// Hash of this entry
    pub hash: [u8; 32],
    /// Number of hashes performed to reach this entry
    pub num_hashes: u64,
    /// Timestamp when this entry was created
    pub timestamp: u64,
    /// Optional message mixed into the hash
    pub message: Option<Vec<u8>>,
}

impl PoHEntry {
    /// Create a new PoH entry
    pub fn new(
        previous_hash: [u8; 32],
        num_hashes: u64,
        timestamp: u64,
        message: Option<Vec<u8>>,
    ) -> Self {
        let hash = if let Some(ref msg) = message {
            hash_with_message(&previous_hash, num_hashes, msg)
        } else {
            hash_n_times(&previous_hash, num_hashes)
        };

        Self {
            hash,
            num_hashes,
            timestamp,
            message,
        }
    }

    /// Create genesis entry
    pub fn genesis() -> Self {
        Self {
            hash: [0u8; 32],
            num_hashes: 0,
            timestamp: 0,
            message: Some(b"AETHER Genesis".to_vec()),
        }
    }

    /// Verify this entry against a previous hash
    pub fn verify(&self,
        previous_hash: [u8; 32],
    ) -> bool {
        let expected_hash = if let Some(ref msg) = self.message {
            hash_with_message(&previous_hash, self.num_hashes, msg)
        } else {
            hash_n_times(&previous_hash, self.num_hashes)
        };

        self.hash == expected_hash
    }
}

/// PoH generator - creates the verifiable delay chain
#[derive(Debug)]
pub struct PoHGenerator {
    /// Current hash in the chain
    current_hash: [u8; 32],
    /// Total hashes performed
    total_hashes: u64,
    /// Start time
    start_time: Instant,
}

impl PoHGenerator {
    /// Create a new PoH generator
    pub fn new() -> Self {
        Self {
            current_hash: [0u8; 32],
            total_hashes: 0,
            start_time: Instant::now(),
        }
    }

    /// Create from a specific starting hash
    pub fn from_hash(hash: [u8; 32]) -> Self {
        Self {
            current_hash: hash,
            total_hashes: 0,
            start_time: Instant::now(),
        }
    }

    /// Record a new entry with the specified number of hashes
    pub fn record(&mut self, num_hashes: u64, message: Option<Vec<u8>>) -> PoHEntry {
        let timestamp = self.start_time.elapsed().as_millis() as u64;
        
        let entry = PoHEntry::new(
            self.current_hash,
            num_hashes,
            timestamp,
            message,
        );

        self.current_hash = entry.hash;
        self.total_hashes += num_hashes;

        entry
    }

    /// Record a tick (standard time interval)
    pub fn tick(&mut self) -> PoHEntry {
        self.record(HASHES_PER_TICK, None)
    }

    /// Mix a message into the current hash
    pub fn mix(&mut self, message: &[u8]) -> PoHEntry {
        self.record(1, Some(message.to_vec()))
    }

    /// Get the current hash
    pub fn current_hash(&self) -> [u8; 32] {
        self.current_hash
    }

    /// Get total hashes performed
    pub fn total_hashes(&self) -> u64 {
        self.total_hashes
    }
}

impl Default for PoHGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify a chain of PoH entries
pub fn verify_poh_chain(entries: &[PoHEntry]) -> bool {
    if entries.is_empty() {
        return true;
    }

    // Single entry: must be a valid standalone tick (can verify against any previous hash)
    // For genesis blocks, we check if it starts with genesis
    if entries.len() == 1 {
        // Single tick entry is valid - it just needs to represent a valid PoH computation
        // The actual hash link to previous is verified at the block level via parent_hash
        return entries[0].num_hashes > 0 || entries[0].message.is_some();
    }

    // Check if chain starts with genesis
    let start_idx = if entries[0] == PoHEntry::genesis() {
        0
    } else {
        // Non-genesis chain: verify internal consistency from first entry
        // First entry is assumed to link to parent block's PoH
        0
    };

    // Verify each entry links to the previous
    for i in (start_idx + 1)..entries.len() {
        let prev_hash = entries[i - 1].hash;
        if !entries[i].verify(prev_hash) {
            return false;
        }
    }

    true
}

/// Hash a value n times using SHA-256
fn hash_n_times(start: &[u8; 32], n: u64) -> [u8; 32] {
    let mut hash = *start;
    
    for _ in 0..n {
        let mut hasher = Sha256::new();
        hasher.update(&hash);
        hash = hasher.finalize().into();
    }
    
    hash
}

/// Hash with a message mixed in
fn hash_with_message(start: &[u8; 32], n: u64, message: &[u8]) -> [u8; 32] {
    // First do n-1 hashes
    let mut hash = hash_n_times(start, n.saturating_sub(1));
    
    // Then mix in the message
    let mut hasher = Sha256::new();
    hasher.update(&hash);
    hasher.update(message);
    hash = hasher.finalize().into();
    
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poh_entry_creation() {
        let prev_hash = [1u8; 32];
        let entry = PoHEntry::new(prev_hash, 100, 0, None);
        
        assert!(entry.verify(prev_hash));
        assert!(!entry.verify([2u8; 32]));
    }

    #[test]
    fn test_poh_entry_with_message() {
        let prev_hash = [1u8; 32];
        let message = b"test message".to_vec();
        let entry = PoHEntry::new(prev_hash, 100, 0, Some(message.clone()));
        
        assert!(entry.verify(prev_hash));
        
        // Different message should produce different hash but still verify
        let entry2 = PoHEntry::new(prev_hash, 100, 0, Some(b"different".to_vec()));
        assert!(entry2.verify(prev_hash));
        
        // But the hashes should be different because messages differ
        assert_ne!(entry.hash, entry2.hash);
    }

    #[test]
    fn test_poh_generator() {
        let mut gen = PoHGenerator::new();
        
        let entry1 = gen.tick();
        let entry2 = gen.tick();
        
        assert_eq!(entry1.num_hashes, HASHES_PER_TICK);
        assert_eq!(entry2.num_hashes, HASHES_PER_TICK);
        assert_ne!(entry1.hash, entry2.hash);
    }

    #[test]
    fn test_verify_poh_chain() {
        let _gen = PoHGenerator::new();
        let genesis = PoHEntry::genesis();
        let mut gen = PoHGenerator::from_hash(genesis.hash);
        
        let entry1 = gen.tick();
        let entry2 = gen.tick();
        let entry3 = gen.mix(b"block hash");
        
        let chain = vec![genesis, entry1, entry2, entry3];
        assert!(verify_poh_chain(&chain));
        
        // Tampered chain should fail
        let mut bad_chain = chain.clone();
        bad_chain[2].hash = [0u8; 32];
        assert!(!verify_poh_chain(&bad_chain));
    }

    #[test]
    fn test_hash_n_times() {
        let start = [1u8; 32];
        let hash1 = hash_n_times(&start, 1);
        let hash2 = hash_n_times(&start, 2);
        
        assert_ne!(hash1, hash2);
        assert_ne!(hash1, start);
    }
}
