//! AETHER Core Types

use serde::{Deserialize, Serialize};
use borsh::{BorshSerialize, BorshDeserialize};

/// AI Priority Lane for transaction ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[repr(u8)]
pub enum AIPriorityLane {
    /// Critical lane - AI governance decisions, emergency ops
    Critical = 0,
    /// High lane - AI agent transactions, MEV protection
    High = 1,
    /// Standard lane - Regular user transactions
    Standard = 2,
}

impl AIPriorityLane {
    /// Get the multiplier for this lane
    pub fn multiplier(&self) -> u64 {
        match self {
            AIPriorityLane::Critical => 10,
            AIPriorityLane::High => 5,
            AIPriorityLane::Standard => 1,
        }
    }

    /// Get the minimum priority fee for this lane
    pub fn min_priority_fee(&self) -> u64 {
        match self {
            AIPriorityLane::Critical => 1_000_000, // 0.001 AETH
            AIPriorityLane::High => 500_000,       // 0.0005 AETH
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
    /// AI Validator - Full AI capabilities
    AI = 0,
    /// Standard Validator - Block production, validation
    Standard = 1,
    /// Light Validator - Vote verification only
    Light = 2,
}

impl ValidatorTier {
    /// Get the reward multiplier for this tier
    pub fn reward_multiplier(&self) -> f64 {
        match self {
            ValidatorTier::AI => 1.25,      // 25% bonus
            ValidatorTier::Standard => 1.0,
            ValidatorTier::Light => 0.5,    // 50% of base
        }
    }

    /// Check if validator can produce blocks
    pub fn can_produce_blocks(&self) -> bool {
        matches!(self, ValidatorTier::AI | ValidatorTier::Standard)
    }

    /// Check if validator has AI capabilities
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
    /// Priority lane for this transaction
    pub lane: AIPriorityLane,
    /// AI oracle signature (if validated by AI)
    pub ai_signature: Option<[u8; 64]>, // Ed25519 signature
    /// Compute units requested
    pub compute_units: u64,
    /// Priority fee paid
    pub priority_fee: u64,
}

impl Default for AITransactionMeta {
    fn default() -> Self {
        Self {
            lane: AIPriorityLane::Standard,
            ai_signature: None,
            compute_units: 200_000, // Default compute limit
            priority_fee: 0,
        }
    }
}

/// Proposal types for governance
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ProposalType {
    /// Protocol upgrade proposal
    ProtocolUpgrade { bytecode_hash: [u8; 32] },
    /// Parameter change proposal
    ParameterChange { param: String, value: u64 },
    /// Treasury spend proposal
    TreasurySpend { amount: u64, recipient: [u8; 32] },
    /// Emergency action proposal
    EmergencyAction { instruction: Vec<u8> },
    /// AI model update proposal
    AIModelUpdate { model_hash: [u8; 32] },
}

/// Vote decision for governance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[repr(u8)]
pub enum VoteDecision {
    Against = 0,
    For = 1,
    Abstain = 2,
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
    /// Individual AI oracle votes
    pub votes: Vec<AIVote>,
    /// Consensus score (0.0 - 1.0)
    pub consensus_score: f64,
    /// AI confidence metric
    pub confidence: f64,
}

/// Individual AI vote
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AIVote {
    /// Oracle public key
    pub oracle: [u8; 32],
    /// Decision
    pub decision: VoteDecision,
    /// Oracle weight
    pub weight: f64,
    /// Confidence in decision
    pub confidence: f64,
    /// Signature over vote
    pub signature: [u8; 64],
}

/// Tokenomics configuration
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TokenomicsConfig {
    /// Total supply
    pub total_supply: u64,
    /// Current circulating supply
    pub circulating_supply: u64,
    /// Inflation rate (basis points, e.g., 450 = 4.5%)
    pub inflation_rate_bps: u64,
    /// Burn rate (basis points of fees)
    pub burn_rate_bps: u64,
}

impl Default for TokenomicsConfig {
    fn default() -> Self {
        Self {
            total_supply: crate::TOTAL_SUPPLY_AETH,
            circulating_supply: 0,
            inflation_rate_bps: 450, // 4.5% initial
            burn_rate_bps: 5000,     // 50% of fees
        }
    }
}
