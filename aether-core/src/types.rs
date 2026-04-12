//! Core types for AETHER blockchain
//!
//! Defines fundamental types (Hash, Signature, Address) with proper serde
//! serialization support. These are newtypes wrapping fixed-size byte arrays,
//! providing hex-based serialization and all common traits needed downstream.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;

// ---------------------------------------------------------------------------
// Hash (32 bytes)
// ---------------------------------------------------------------------------

/// A 32-byte hash (e.g. block hash, PoH hash, state root).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// The zero hash (all bytes = 0).
    pub const ZERO: Hash = Hash([0u8; 32]);

    /// Create a Hash from raw bytes.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Return the inner bytes as a slice.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Convert from hex string.
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl Copy for Hash {}

impl Default for Hash {
    fn default() -> Self {
        Self::ZERO
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8; 32]> for Hash {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<Hash> for [u8; 32] {
    fn from(h: Hash) -> [u8; 32] {
        h.0
    }
}

impl From<sha2::digest::Output<sha2::Sha256>> for Hash {
    fn from(output: sha2::digest::Output<sha2::Sha256>) -> Self {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&output);
        Self(bytes)
    }
}

impl Ord for Hash {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Hash {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for Hash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Hash::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Signature (64 bytes)
// ---------------------------------------------------------------------------

/// A 64-byte Ed25519 signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

impl Signature {
    /// The zero signature (all bytes = 0).
    pub const ZERO: Signature = Signature([0u8; 64]);

    /// Create a Signature from raw bytes.
    pub fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Return the inner bytes.
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Convert from hex string.
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0u8; 64];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl Copy for Signature {}

impl Default for Signature {
    fn default() -> Self {
        Self::ZERO
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 64]> for Signature {
    fn from(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
}

impl Serialize for Signature {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Signature::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Address (32 bytes)
// ---------------------------------------------------------------------------

/// A 32-byte address (public key).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address(pub [u8; 32]);

impl Address {
    /// The zero address (all bytes = 0).
    pub const ZERO: Address = Address([0u8; 32]);

    /// Create an Address from raw bytes.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Return the inner bytes as a slice.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Convert from hex string.
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl Copy for Address {}

impl Default for Address {
    fn default() -> Self {
        Self::ZERO
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8; 32]> for Address {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for Address {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<Address> for [u8; 32] {
    fn from(a: Address) -> [u8; 32] {
        a.0
    }
}

impl Ord for Address {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Address {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for Address {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Address::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Core domain types
// ---------------------------------------------------------------------------

/// AETHER block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block height
    pub height: u64,
    /// Previous block hash
    pub prev_hash: Hash,
    /// Timestamp
    pub timestamp: u64,
    /// PoH hash
    pub poh_hash: Hash,
    /// State root
    pub state_root: Hash,
}

/// AETHER transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction signature
    pub signature: Signature,
    /// Sender address
    pub from: Address,
    /// Recipient address
    pub to: Address,
    /// Amount in lamports
    pub amount: u64,
    /// Transaction data
    pub data: Vec<u8>,
    /// AI priority score (0-100)
    pub priority_score: u8,
}

/// Block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// Transactions in block
    pub transactions: Vec<Transaction>,
}

/// Validator info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator address
    pub address: Address,
    /// Stake amount
    pub stake: u64,
    /// Commission rate (0-10000, representing 0-100%)
    pub commission: u16,
}

/// Genesis account for initial state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAccount {
    /// Account address
    pub address: [u8; 32],
    /// Initial lamports balance
    pub lamports: u64,
    /// Optional account data
    pub data: Option<Vec<u8>>,
}

/// Canonical transaction type alias.
pub type AetherTransaction = Transaction;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_hex_roundtrip() {
        let h = Hash::new([0xab; 32]);
        let json = serde_json::to_string(&h).unwrap();
        let h2: Hash = serde_json::from_str(&json).unwrap();
        assert_eq!(h, h2);
    }

    #[test]
    fn signature_hex_roundtrip() {
        let sig = Signature::new([0xcd; 64]);
        let json = serde_json::to_string(&sig).unwrap();
        let sig2: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(sig, sig2);
    }

    #[test]
    fn address_hex_roundtrip() {
        let addr = Address::new([0xef; 32]);
        let json = serde_json::to_string(&addr).unwrap();
        let addr2: Address = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, addr2);
    }

    #[test]
    fn block_serialization_roundtrip() {
        let block = Block {
            header: BlockHeader {
                height: 42,
                prev_hash: Hash::new([1u8; 32]),
                timestamp: 1700000000,
                poh_hash: Hash::new([2u8; 32]),
                state_root: Hash::new([3u8; 32]),
            },
            transactions: vec![Transaction {
                signature: Signature::new([4u8; 64]),
                from: Address::new([5u8; 32]),
                to: Address::new([6u8; 32]),
                amount: 1000,
                data: vec![7, 8, 9],
                priority_score: 50,
            }],
        };
        let json = serde_json::to_string(&block).unwrap();
        let block2: Block = serde_json::from_str(&json).unwrap();
        assert_eq!(block.header.height, block2.header.height);
        assert_eq!(block.transactions.len(), block2.transactions.len());
        assert_eq!(block.transactions[0].amount, block2.transactions[0].amount);
    }

    #[test]
    fn hash_copy_and_ordering() {
        let a = Hash::new([1u8; 32]);
        let b = a; // Copy
        assert_eq!(a, b);
        let c = Hash::new([2u8; 32]);
        assert!(a < c);
    }

    #[test]
    fn hash_from_sha256() {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"test");
        let result: Hash = hasher.finalize().into();
        assert_ne!(result, Hash::ZERO);
    }
}