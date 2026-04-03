//! AETHER Core Types

use serde::{Deserialize, Serialize};
use borsh::{BorshSerialize, BorshDeserialize};

/// Wrapper for 64-byte signatures with manual serde support
/// (serde derive doesn't support arrays > 32 bytes)
#[derive(Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct SignatureBytes(pub [u8; 64]);

impl SignatureBytes {
    pub fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
    pub fn as_slice(&self) -> &[u8; 64] {
        &self.0
    }
}

impl std::fmt::Debug for SignatureBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SignatureBytes({:.8}...)", hex::encode(&self.0[..8]))
    }
}

impl Serialize for SignatureBytes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for SignatureBytes {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex_str = <String as serde::Deserialize>::deserialize(deserializer)?;
        let bytes = hex::decode(hex_str)
            .map_err(|e| serde::de::Error::custom(format!("hex decode error: {}", e)))?;
        let bytes: [u8; 64] = bytes.try_into().map_err(|_| {
            serde::de::Error::custom("expected exactly 64 bytes")
        })?;
        Ok(SignatureBytes(bytes))
    }
}

/// AI Priority Lane for transaction ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[repr(u8)]
pub enum AIPriorityLane {
    Critical = 0,
    High = 1,
    Standard = 2,
}

impl AIPriorityLane {
    pub fn multiplier(&self) -> u64 {
        match self {
            AIPriorityLane::Critical => 10,
            AIPriorityLane::High => 5,
            AIPriorityLane::Standard => 1,
        }
    }
    pub fn min_priority_fee(&self) -> u64 {
        match self {
            AIPriorityLane::Critical => 1_000_000,
            AIPriorityLane::High => 500_000,
            AIPriorityLane::Standard => 0,
        }
    }
}

impl Default for AIPriorityLane {
    fn default() -> Self {
        AIPriorityLane::Standard
    }
}

/// Validator tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[repr(u8)]
pub enum ValidatorTier {
    AI = 0,
    Standard = 1,
    Light = 2,
}

impl ValidatorTier {
    pub fn reward_multiplier(&self) -> f64 {
        match self {
            ValidatorTier::AI => 1.25,
            ValidatorTier::Standard => 1.0,
            ValidatorTier::Light => 0.5,
        }
    }
    pub fn can_produce_blocks(&self) -> bool {
        matches!(self, ValidatorTier::AI | ValidatorTier::Standard)
    }
    pub fn has_ai_capabilities(&self) -> bool {
        matches!(self, ValidatorTier::AI)
    }
}

impl Default for ValidatorTier {
    fn default() -> Self {
        ValidatorTier::Standard
    }
}

/// Transaction metadata with AI priority
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AITransactionMeta {
    pub lane: AIPriorityLane,
    #[serde(skip)]
    pub ai_signature: Option<SignatureBytes>,
    pub compute_units: u64,
    pub priority_fee: u64,
}

impl Default for AITransactionMeta {
    fn default() -> Self {
        Self {
            lane: AIPriorityLane::Standard,
            ai_signature: None,
            compute_units: 200_000,
            priority_fee: 0,
        }
    }
}

/// Proposal types for governance
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ProposalType {
    ProtocolUpgrade { bytecode_hash: [u8; 32] },
    ParameterChange { param: String, value: u64 },
    TreasurySpend { amount: u64, recipient: [u8; 32] },
    EmergencyAction { instruction: Vec<u8> },
    AIModelUpdate { model_hash: [u8; 32] },
}

/// Vote decision for governance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[repr(i8)]
pub enum VoteDecision {
    Against = -1,
    For = 1,
    Abstain = 0,
}

/// Proposal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[repr(u8)]
pub enum ProposalStatus {
    Pending = 0,
    Active = 1,
    Succeeded = 2,
    Defeated = 3,
    Executed = 4,
    Cancelled = 5,
}

/// AI vote data structure
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AIVoteData {
    pub votes: Vec<AIVote>,
    pub consensus_score: f64,
    pub confidence: f64,
}

/// Individual AI vote
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AIVote {
    pub oracle: [u8; 32],
    pub decision: VoteDecision,
    pub weight: f64,
    pub confidence: f64,
    pub signature: SignatureBytes,
}

/// Tokenomics configuration
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TokenomicsConfig {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub inflation_rate_bps: u64,
    pub burn_rate_bps: u64,
}

impl Default for TokenomicsConfig {
    fn default() -> Self {
        Self {
            total_supply: crate::TOTAL_SUPPLY_AETH,
            circulating_supply: 0,
            inflation_rate_bps: 450,
            burn_rate_bps: 5000,
        }
    }
}
