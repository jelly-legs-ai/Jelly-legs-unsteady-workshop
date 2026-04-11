//! AETHER Governance Module
//!
//! Complete DAO governance system including:
//! - Proposal creation and lifecycle management
//! - Quadratic voting for fair representation
//! - Multi-sig treasury with timelock
//! - Security council for emergency veto
//! - Voting power snapshots and delegation
//!
//! # Example
//!
//! ```ignore
//! use aether_governance::{AetherDAO, Treasury, GovernanceConfig, ProposalType};
//!
//! // Create DAO with default config
//! let mut dao = AetherDAO::with_default_config();
//!
//! // Add security council members
//! dao.add_council_member([1u8; 32]);
//!
//! // Create a voting power snapshot
//! let snapshot = dao.create_snapshot(vec![
//!     ([1u8; 32], 10_000_000_000_000), // 10k ATH
//! ]);
//!
//! // Create a proposal
//! let proposal_id = dao.create_proposal(
//!     "Upgrade Protocol".to_string(),
//!     "Upgrade to v2.0".to_string(),
//!     ProposalType::ProtocolUpgrade {
//!         version: "2.0".to_string(),
//!         description: "Major upgrade".to_string(),
//!         upgrade_hash: [0u8; 32],
//!     },
//!     [1u8; 32],
//!     100_000_000_000,
//!     snapshot,
//! ).unwrap();
//! ```

pub mod dao;
pub mod treasury;
pub mod votes;

// Re-export all public types
pub use dao::{
    AetherDAO,
    GovernanceConfig,
    GovernanceError,
    GovernanceStats,
    Proposal,
    ProposalStatus,
    ProposalType,
    SignerAction,
    ValidatorAction,
};

pub use treasury::{
    BudgetAllocation,
    Treasury,
    TreasuryBalances,
    TreasuryConfig,
    TreasuryError,
    TreasurySigner,
    TreasurySummary,
    TokenType,
    WithdrawalRequest,
    WithdrawalStatus,
};

pub use votes::{
    Delegation,
    Vote,
    VoteChoice,
    VoteTally,
    VotingPowerCalculator,
    VotingPowerSnapshot,
};