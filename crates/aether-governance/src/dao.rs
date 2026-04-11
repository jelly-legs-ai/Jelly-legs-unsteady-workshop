//! DAO governance for Aether blockchain
//!
//! Implements:
//! - Proposal creation and lifecycle
//! - Voting periods with configurable durations
//! - Proposal types: parameter changes, fund allocation, code upgrades
//! - Execution of passed proposals
//! - Quorum and supermajority requirements

use crate::votes::{Vote, VoteChoice, VoteTally, VotingPowerSnapshot, VotingPowerCalculator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Proposal status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    /// Draft, not yet submitted
    Draft,
    /// Pending activation (waiting for voting delay)
    Pending,
    /// Active voting in progress
    Active,
    /// Voting ended, passed
    Passed,
    /// Voting ended, failed
    Failed,
    /// Successfully executed
    Executed,
    /// Cancelled by proposer
    Cancelled,
    /// Expired without execution
    Expired,
    /// Vetoed by security council
    Vetoed,
}

impl Default for ProposalStatus {
    fn default() -> Self {
        ProposalStatus::Draft
    }
}

/// Proposal type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalType {
    /// Change network parameter (e.g., block time, fee parameters)
    ParameterChange {
        parameter: String,
        current_value: String,
        new_value: String,
    },
    /// Fund allocation from treasury
    FundAllocation {
        recipient: [u8; 32],
        amount: u64,
        token_type: String,
        purpose: String,
    },
    /// Protocol upgrade
    ProtocolUpgrade {
        version: String,
        description: String,
        upgrade_hash: [u8; 32],
    },
    /// Add/remove validator
    ValidatorChange {
        validator: [u8; 32],
        action: ValidatorAction,
    },
    /// Treasury signer change
    SignerChange {
        signer: [u8; 32],
        action: SignerAction,
    },
    /// General text proposal (signal)
    TextProposal {
        title: String,
        description: String,
    },
}

/// Validator action type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidatorAction {
    Add,
    Remove,
    Slash { percentage_bps: u64 },
}

/// Signer action type  
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignerAction {
    Add,
    Remove,
}

/// Governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal ID
    pub id: u64,
    /// Proposal title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Proposal type with specific parameters
    pub proposal_type: ProposalType,
    /// Proposer address
    pub proposer: [u8; 32],
    /// Creation timestamp
    pub created_at: u64,
    /// When voting begins
    pub voting_start: u64,
    /// When voting ends
    pub voting_end: u64,
    /// When proposal expires if not executed
    pub execution_deadline: u64,
    /// Current status
    pub status: ProposalStatus,
    /// Vote tally
    pub tally: VoteTally,
    /// Votes cast (voter -> vote)
    pub votes: HashMap<[u8; 32], Vote>,
    /// Snapshot block for voting power
    pub snapshot_block: u64,
}

impl Proposal {
    /// Create a new proposal
    pub fn new(
        id: u64,
        title: String,
        description: String,
        proposal_type: ProposalType,
        proposer: [u8; 32],
        created_at: u64,
        voting_delay: u64,
        voting_duration: u64,
        execution_delay: u64,
        snapshot_block: u64,
    ) -> Self {
        Self {
            id,
            title,
            description,
            proposal_type,
            proposer,
            created_at,
            voting_start: created_at + voting_delay,
            voting_end: created_at + voting_delay + voting_duration,
            execution_deadline: created_at + voting_delay + voting_duration + execution_delay,
            status: ProposalStatus::Pending,
            tally: VoteTally::new(),
            votes: HashMap::new(),
            snapshot_block,
        }
    }

    /// Check if proposal is in voting period
    pub fn is_voting_active(&self, current_time: u64) -> bool {
        current_time >= self.voting_start && current_time < self.voting_end
    }

    /// Check if voting has ended
    pub fn voting_ended(&self, current_time: u64) -> bool {
        current_time >= self.voting_end
    }

    /// Check if proposal can be executed
    pub fn can_execute(&self, current_time: u64) -> bool {
        self.status == ProposalStatus::Passed && current_time < self.execution_deadline
    }

    /// Check if proposal has expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.execution_deadline && self.status != ProposalStatus::Executed
    }

    /// Cast a vote on this proposal
    pub fn vote(&mut self, vote: Vote) -> Result<(), GovernanceError> {
        // Check if already voted
        if self.votes.contains_key(&vote.voter) {
            return Err(GovernanceError::AlreadyVoted);
        }

        // Add vote to tally
        self.tally.add_vote(&vote);
        self.votes.insert(vote.voter, vote);

        Ok(())
    }

    /// Change an existing vote
    pub fn change_vote(&mut self, new_vote: Vote) -> Result<(), GovernanceError> {
        // Remove old vote
        if let Some(old_vote) = self.votes.remove(&new_vote.voter) {
            self.tally.remove_vote(&old_vote);
        }

        // Add new vote
        self.tally.add_vote(&new_vote);
        self.votes.insert(new_vote.voter, new_vote);

        Ok(())
    }

    /// Update status based on voting results
    pub fn finalize(&mut self, quorum: u64, supermajority_bps: u64) {
        if self.tally.passes(quorum, supermajority_bps) {
            self.status = ProposalStatus::Passed;
        } else {
            self.status = ProposalStatus::Failed;
        }
    }

    /// Cancel the proposal
    pub fn cancel(&mut self, proposer: &[u8; 32]) -> Result<(), GovernanceError> {
        if self.proposer != *proposer {
            return Err(GovernanceError::NotProposer);
        }
        if self.status == ProposalStatus::Executed {
            return Err(GovernanceError::AlreadyExecuted);
        }
        self.status = ProposalStatus::Cancelled;
        Ok(())
    }

    /// Veto the proposal
    pub fn veto(&mut self) {
        self.status = ProposalStatus::Vetoed;
    }

    /// Mark as executed
    pub fn execute(&mut self) {
        self.status = ProposalStatus::Executed;
    }
}

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Delay before voting starts (seconds)
    pub voting_delay: u64,
    /// Duration of voting period (seconds)
    pub voting_duration: u64,
    /// Time after voting ends before execution expires (seconds)
    pub execution_delay: u64,
    /// Quorum threshold (total voting power required)
    pub quorum_threshold: u64,
    /// Supermajority threshold (basis points, e.g., 6600 = 66%)
    pub supermajority_bps: u64,
    /// Minimum ATH required to create proposal
    pub min_proposal_deposit: u64,
    /// Maximum active proposals
    pub max_active_proposals: usize,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            voting_delay: 86400,          // 1 day
            voting_duration: 259200,       // 3 days
            execution_delay: 172800,       // 2 days
            quorum_threshold: 10_000_000_000, // 10 ATH worth of votes
            supermajority_bps: 6600,       // 66%
            min_proposal_deposit: 100_000_000_000, // 100 ATH
            max_active_proposals: 20,
        }
    }
}

/// Aether DAO Governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherDAO {
    /// Governance configuration
    pub config: GovernanceConfig,
    /// All proposals
    pub proposals: HashMap<u64, Proposal>,
    /// Next proposal ID
    pub next_proposal_id: u64,
    /// Proposal deposits (proposer -> amount)
    pub deposits: HashMap<[u8; 32], u64>,
    /// Voting power snapshots (block -> snapshot)
    pub snapshots: HashMap<u64, VotingPowerSnapshot>,
    /// Voting power calculator
    #[serde(skip)]
    pub power_calculator: VotingPowerCalculator,
    /// Security council members (can veto)
    pub security_council: Vec<[u8; 32]>,
    /// Current block number
    pub current_block: u64,
    /// Current timestamp
    pub current_time: u64,
}

impl AetherDAO {
    /// Create a new DAO
    pub fn new(config: GovernanceConfig) -> Self {
        Self {
            config,
            proposals: HashMap::new(),
            next_proposal_id: 1,
            deposits: HashMap::new(),
            snapshots: HashMap::new(),
            power_calculator: VotingPowerCalculator::new(),
            security_council: Vec::new(),
            current_block: 0,
            current_time: 0,
        }
    }

    /// Create with default config
    pub fn with_default_config() -> Self {
        Self::new(GovernanceConfig::default())
    }

    /// Add security council member
    pub fn add_council_member(&mut self, member: [u8; 32]) {
        if !self.security_council.contains(&member) {
            self.security_council.push(member);
        }
    }

    /// Remove security council member
    pub fn remove_council_member(&mut self, member: &[u8; 32]) {
        self.security_council.retain(|m| m != member);
    }

    /// Check if address is in security council
    pub fn is_council_member(&self, address: &[u8; 32]) -> bool {
        self.security_council.contains(address)
    }

    /// Create a voting power snapshot at current block
    pub fn create_snapshot(&mut self, balances: Vec<([u8; 32], u64)>) -> u64 {
        let block = self.current_block;
        let mut snapshot = VotingPowerSnapshot::new(block, self.current_time);
        
        for (address, balance) in balances {
            let power = self.power_calculator.calculate(balance);
            if power > 0 {
                snapshot.set_power(address, power);
            }
        }
        
        self.snapshots.insert(block, snapshot);
        block
    }

    /// Get voting power for address at snapshot
    pub fn get_voting_power(&self, address: &[u8; 32], snapshot_block: u64) -> u64 {
        self.snapshots
            .get(&snapshot_block)
            .map(|s| s.get_power(address))
            .unwrap_or(0)
    }

    /// Create a new proposal
    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        proposal_type: ProposalType,
        proposer: [u8; 32],
        deposit: u64,
        snapshot_block: u64,
    ) -> Result<u64, GovernanceError> {
        // Check deposit
        if deposit < self.config.min_proposal_deposit {
            return Err(GovernanceError::InsufficientDeposit);
        }

        // Check active proposal limit
        let active_count = self.proposals.values()
            .filter(|p| p.status == ProposalStatus::Pending || p.status == ProposalStatus::Active)
            .count();
        if active_count >= self.config.max_active_proposals {
            return Err(GovernanceError::TooManyActiveProposals);
        }

        let proposal = Proposal::new(
            self.next_proposal_id,
            title,
            description,
            proposal_type,
            proposer,
            self.current_time,
            self.config.voting_delay,
            self.config.voting_duration,
            self.config.execution_delay,
            snapshot_block,
        );

        let id = self.next_proposal_id;
        self.next_proposal_id += 1;
        
        // Store deposit
        self.deposits.insert(proposer, deposit);
        
        // Store proposal
        self.proposals.insert(id, proposal);

        Ok(id)
    }

    /// Activate pending proposals (start voting)
    pub fn activate_proposals(&mut self) {
        for proposal in self.proposals.values_mut() {
            if proposal.status == ProposalStatus::Pending 
                && proposal.voting_start <= self.current_time 
            {
                proposal.status = ProposalStatus::Active;
            }
        }
    }

    /// Finalize proposals where voting has ended
    pub fn finalize_proposals(&mut self) {
        let quorum = self.config.quorum_threshold;
        let supermajority = self.config.supermajority_bps;
        
        for proposal in self.proposals.values_mut() {
            if proposal.status == ProposalStatus::Active && proposal.voting_ended(self.current_time) {
                proposal.finalize(quorum, supermajority);
            }
        }
    }

    /// Cast a vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: u64,
        voter: [u8; 32],
        choice: VoteChoice,
        signature: [u8; 64],
    ) -> Result<(), GovernanceError> {
        // First, get the snapshot block from the proposal
        let snapshot_block = {
            let proposal = self.proposals.get(&proposal_id)
                .ok_or(GovernanceError::ProposalNotFound)?;
            
            // Check status
            if proposal.status != ProposalStatus::Active {
                return Err(GovernanceError::VotingNotActive);
            }
            
            // Check voting period
            if !proposal.is_voting_active(self.current_time) {
                return Err(GovernanceError::NotInVotingPeriod);
            }
            
            proposal.snapshot_block
        };
        
        // Get voting power from snapshot (immutable borrow)
        let power = self.get_voting_power(&voter, snapshot_block);
        if power == 0 {
            return Err(GovernanceError::NoVotingPower);
        }

        // Create vote
        let vote = Vote::new(voter, choice, power, self.current_time, signature);
        
        // Now get mutable reference to proposal and vote
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        proposal.vote(vote)
    }

    /// Execute a passed proposal
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<ProposalType, GovernanceError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        // Check status
        if proposal.status != ProposalStatus::Passed {
            return Err(GovernanceError::ProposalNotPassed);
        }

        // Check execution deadline
        if proposal.is_expired(self.current_time) {
            proposal.status = ProposalStatus::Expired;
            return Err(GovernanceError::ProposalExpired);
        }

        // Mark as executed
        proposal.execute();

        // Return proposal type for execution by caller
        Ok(proposal.proposal_type.clone())
    }

    /// Veto a proposal (security council only)
    pub fn veto_proposal(
        &mut self,
        proposal_id: u64,
        vetoer: &[u8; 32],
    ) -> Result<(), GovernanceError> {
        if !self.is_council_member(vetoer) {
            return Err(GovernanceError::NotCouncilMember);
        }

        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Passed && proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::CannotVeto);
        }

        proposal.veto();
        Ok(())
    }

    /// Cancel a proposal (proposer only)
    pub fn cancel_proposal(
        &mut self,
        proposal_id: u64,
        canceller: &[u8; 32],
    ) -> Result<(), GovernanceError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        proposal.cancel(canceller)
    }

    /// Update time and process proposals
    pub fn tick(&mut self, block: u64, time: u64) {
        self.current_block = block;
        self.current_time = time;
        self.activate_proposals();
        self.finalize_proposals();
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, id: u64) -> Option<&Proposal> {
        self.proposals.get(&id)
    }

    /// Get all active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.status == ProposalStatus::Active || p.status == ProposalStatus::Pending)
            .collect()
    }

    /// Get proposals by status
    pub fn get_proposals_by_status(&self, status: ProposalStatus) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.status == status)
            .collect()
    }

    /// Get proposal count
    pub fn proposal_count(&self) -> usize {
        self.proposals.len()
    }

    /// Get governance stats
    pub fn stats(&self) -> GovernanceStats {
        let mut stats = GovernanceStats::default();
        
        for proposal in self.proposals.values() {
            match proposal.status {
                ProposalStatus::Pending => stats.pending += 1,
                ProposalStatus::Active => stats.active += 1,
                ProposalStatus::Passed => stats.passed += 1,
                ProposalStatus::Failed => stats.failed += 1,
                ProposalStatus::Executed => stats.executed += 1,
                ProposalStatus::Cancelled => stats.cancelled += 1,
                ProposalStatus::Expired => stats.expired += 1,
                ProposalStatus::Vetoed => stats.vetoed += 1,
                ProposalStatus::Draft => stats.draft += 1,
            }
        }
        
        stats.total = self.proposals.len() as u64;
        stats
    }
}

/// Governance statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GovernanceStats {
    pub total: u64,
    pub draft: u64,
    pub pending: u64,
    pub active: u64,
    pub passed: u64,
    pub failed: u64,
    pub executed: u64,
    pub cancelled: u64,
    pub expired: u64,
    pub vetoed: u64,
}

/// Governance errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernanceError {
    ProposalNotFound,
    ProposalNotPassed,
    ProposalExpired,
    AlreadyExecuted,
    AlreadyVoted,
    VotingNotActive,
    NotInVotingPeriod,
    NoVotingPower,
    InsufficientDeposit,
    TooManyActiveProposals,
    NotProposer,
    NotCouncilMember,
    CannotVeto,
}

impl std::fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GovernanceError::ProposalNotFound => write!(f, "Proposal not found"),
            GovernanceError::ProposalNotPassed => write!(f, "Proposal not passed"),
            GovernanceError::ProposalExpired => write!(f, "Proposal expired"),
            GovernanceError::AlreadyExecuted => write!(f, "Proposal already executed"),
            GovernanceError::AlreadyVoted => write!(f, "Already voted"),
            GovernanceError::VotingNotActive => write!(f, "Voting not active"),
            GovernanceError::NotInVotingPeriod => write!(f, "Not in voting period"),
            GovernanceError::NoVotingPower => write!(f, "No voting power"),
            GovernanceError::InsufficientDeposit => write!(f, "Insufficient deposit"),
            GovernanceError::TooManyActiveProposals => write!(f, "Too many active proposals"),
            GovernanceError::NotProposer => write!(f, "Not the proposer"),
            GovernanceError::NotCouncilMember => write!(f, "Not a council member"),
            GovernanceError::CannotVeto => write!(f, "Cannot veto this proposal"),
        }
    }
}

impl std::error::Error for GovernanceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dao_creation() {
        let dao = AetherDAO::with_default_config();
        assert_eq!(dao.config.voting_delay, 86400);
        assert_eq!(dao.proposals.len(), 0);
    }

    #[test]
    fn test_proposal_creation() {
        let mut dao = AetherDAO::with_default_config();
        dao.current_time = 1000;
        dao.current_block = 100;
        
        // Create snapshot
        let snapshot_block = dao.create_snapshot(vec![
            ([1u8; 32], 10_000_000_000), // Proposer has voting power
        ]);
        
        let proposal_id = dao.create_proposal(
            "Test Proposal".to_string(),
            "A test proposal".to_string(),
            ProposalType::TextProposal {
                title: "Test".to_string(),
                description: "Test description".to_string(),
            },
            [1u8; 32],
            100_000_000_000,
            snapshot_block,
        ).unwrap();
        
        assert_eq!(proposal_id, 1);
        let proposal = dao.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Pending);
    }

    #[test]
    fn test_proposal_voting_flow() {
        let mut dao = AetherDAO::with_default_config();
        dao.current_time = 1000;
        dao.current_block = 100;
        
        // Create snapshot with voters
        let snapshot_block = dao.create_snapshot(vec![
            ([1u8; 32], 10_000_000_000_000), // 10k ATH
            ([2u8; 32], 5_000_000_000_000),  // 5k ATH
            ([3u8; 32], 5_000_000_000_000),  // 5k ATH
        ]);
        
        let proposal_id = dao.create_proposal(
            "Test".to_string(),
            "Test".to_string(),
            ProposalType::TextProposal {
                title: "Test".to_string(),
                description: "Test".to_string(),
            },
            [1u8; 32],
            100_000_000_000,
            snapshot_block,
        ).unwrap();
        
        // Advance past voting delay
        dao.tick(101, 1000 + 86400);
        
        // Check status is now active
        assert_eq!(dao.get_proposal(proposal_id).unwrap().status, ProposalStatus::Active);
        
        // Cast votes
        dao.vote(proposal_id, [1u8; 32], VoteChoice::For, [0u8; 64]).unwrap();
        dao.vote(proposal_id, [2u8; 32], VoteChoice::For, [0u8; 64]).unwrap();
        dao.vote(proposal_id, [3u8; 32], VoteChoice::Against, [0u8; 64]).unwrap();
        
        // Advance past voting duration
        dao.tick(102, 1000 + 86400 + 259200 + 1);
        
        // Should be passed (15k for vs 5k against, 75% for)
        assert_eq!(dao.get_proposal(proposal_id).unwrap().status, ProposalStatus::Passed);
    }

    #[test]
    fn test_proposal_insufficient_deposit() {
        let mut dao = AetherDAO::with_default_config();
        dao.current_time = 1000;
        
        let snapshot_block = dao.create_snapshot(vec![([1u8; 32], 1_000_000_000)]);
        
        let result = dao.create_proposal(
            "Test".to_string(),
            "Test".to_string(),
            ProposalType::TextProposal {
                title: "Test".to_string(),
                description: "Test".to_string(),
            },
            [1u8; 32],
            1_000_000_000, // Too low
            snapshot_block,
        );
        
        assert_eq!(result, Err(GovernanceError::InsufficientDeposit));
    }

    #[test]
    fn test_proposal_veto() {
        let mut dao = AetherDAO::with_default_config();
        dao.add_council_member([99u8; 32]);
        dao.current_time = 1000;
        dao.current_block = 100;
        
        let snapshot_block = dao.create_snapshot(vec![([1u8; 32], 10_000_000_000_000)]);
        
        let proposal_id = dao.create_proposal(
            "Test".to_string(),
            "Test".to_string(),
            ProposalType::TextProposal {
                title: "Test".to_string(),
                description: "Test".to_string(),
            },
            [1u8; 32],
            100_000_000_000,
            snapshot_block,
        ).unwrap();
        
        // Activate the proposal first (it's in Pending status)
        dao.tick(101, 1000 + 86400);
        
        // Now it should be Active and can be vetoed
        dao.veto_proposal(proposal_id, &[99u8; 32]).unwrap();
        
        assert_eq!(dao.get_proposal(proposal_id).unwrap().status, ProposalStatus::Vetoed);
    }

    #[test]
    fn test_proposal_cancellation() {
        let mut dao = AetherDAO::with_default_config();
        dao.current_time = 1000;
        dao.current_block = 100;
        
        let snapshot_block = dao.create_snapshot(vec![([1u8; 32], 10_000_000_000_000)]);
        
        let proposal_id = dao.create_proposal(
            "Test".to_string(),
            "Test".to_string(),
            ProposalType::TextProposal {
                title: "Test".to_string(),
                description: "Test".to_string(),
            },
            [1u8; 32],
            100_000_000_000,
            snapshot_block,
        ).unwrap();
        
        // Cancel by proposer
        dao.cancel_proposal(proposal_id, &[1u8; 32]).unwrap();
        
        assert_eq!(dao.get_proposal(proposal_id).unwrap().status, ProposalStatus::Cancelled);
    }

    #[test]
    fn test_governance_stats() {
        let mut dao = AetherDAO::with_default_config();
        dao.current_time = 1000;
        dao.current_block = 100;
        
        let snapshot_block = dao.create_snapshot(vec![([1u8; 32], 10_000_000_000_000)]);
        
        // Create multiple proposals
        for i in 0..3 {
            dao.create_proposal(
                format!("Test {}", i),
                "Test".to_string(),
                ProposalType::TextProposal {
                    title: "Test".to_string(),
                    description: "Test".to_string(),
                },
                [1u8; 32],
                100_000_000_000,
                snapshot_block,
            ).unwrap();
        }
        
        let stats = dao.stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.pending, 3);
    }
}