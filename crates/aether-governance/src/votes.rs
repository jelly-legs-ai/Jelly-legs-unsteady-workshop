//! Voting mechanisms for Aether DAO governance
//!
//! Implements:
//! - Quadratic voting for fair representation
//! - Vote delegation
//! - Vote weights based on ATH token holdings
//! - Snapshot voting for gas-efficient governance

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

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

/// Vote choice
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoteChoice {
    Against = 0,
    Abstain = 1,
    For = 2,
}

impl Default for VoteChoice {
    fn default() -> Self {
        VoteChoice::Abstain
    }
}

/// A single vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Voter's public key
    pub voter: [u8; 32],
    /// Vote choice (for/against/abstain)
    pub choice: VoteChoice,
    /// Voting weight (based on ATH holdings at snapshot)
    pub weight: u64,
    /// Timestamp of vote
    pub timestamp: u64,
    /// Optional reason for the vote
    pub reason: Option<String>,
    /// Vote signature
    #[serde(with = "serde_bytes_64")]
    pub signature: [u8; 64],
}

impl Vote {
    /// Create a new vote
    pub fn new(
        voter: [u8; 32],
        choice: VoteChoice,
        weight: u64,
        timestamp: u64,
        signature: [u8; 64],
    ) -> Self {
        Self {
            voter,
            choice,
            weight,
            timestamp,
            reason: None,
            signature,
        }
    }

    /// Create a vote with a reason
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}

/// Vote aggregation for a proposal
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoteTally {
    /// Total votes for
    pub for_votes: u64,
    /// Total votes against
    pub against_votes: u64,
    /// Total abstain votes
    pub abstain_votes: u64,
    /// Total voters who participated
    pub voter_count: u64,
}

impl VoteTally {
    /// Create a new empty tally
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vote to the tally
    pub fn add_vote(&mut self, vote: &Vote) {
        match vote.choice {
            VoteChoice::For => self.for_votes += vote.weight,
            VoteChoice::Against => self.against_votes += vote.weight,
            VoteChoice::Abstain => self.abstain_votes += vote.weight,
        }
        self.voter_count += 1;
    }

    /// Remove a vote from the tally (for vote changes)
    pub fn remove_vote(&mut self, vote: &Vote) {
        match vote.choice {
            VoteChoice::For => self.for_votes = self.for_votes.saturating_sub(vote.weight),
            VoteChoice::Against => self.against_votes = self.against_votes.saturating_sub(vote.weight),
            VoteChoice::Abstain => self.abstain_votes = self.abstain_votes.saturating_sub(vote.weight),
        }
        self.voter_count = self.voter_count.saturating_sub(1);
    }

    /// Get total votes cast
    pub fn total_votes(&self) -> u64 {
        self.for_votes + self.against_votes + self.abstain_votes
    }

    /// Calculate the percentage of "for" votes
    pub fn for_percentage(&self) -> f64 {
        let total = self.total_votes();
        if total == 0 {
            return 0.0;
        }
        (self.for_votes as f64 / total as f64) * 100.0
    }

    /// Check if quorum is reached
    pub fn has_quorum(&self, quorum_threshold: u64) -> bool {
        self.total_votes() >= quorum_threshold
    }

    /// Check if the proposal passes based on majority
    pub fn passes(&self, quorum_threshold: u64, supermajority_bps: u64) -> bool {
        if !self.has_quorum(quorum_threshold) {
            return false;
        }
        
        let total_decisive = self.for_votes + self.against_votes;
        if total_decisive == 0 {
            return false;
        }
        
        // Supermajority threshold (e.g., 6600 = 66%)
        let for_bps = (self.for_votes * 10000) / total_decisive;
        for_bps >= supermajority_bps
    }
}

/// Voting power snapshot at a specific block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingPowerSnapshot {
    /// Block number for this snapshot
    pub block_number: u64,
    /// Timestamp of snapshot
    pub timestamp: u64,
    /// Voting power per address at snapshot
    pub powers: HashMap<[u8; 32], u64>,
    /// Total voting power at snapshot
    pub total_power: u64,
}

impl VotingPowerSnapshot {
    /// Create a new snapshot
    pub fn new(block_number: u64, timestamp: u64) -> Self {
        Self {
            block_number,
            timestamp,
            powers: HashMap::new(),
            total_power: 0,
        }
    }

    /// Set voting power for an address
    pub fn set_power(&mut self, address: [u8; 32], power: u64) {
        // Remove old power if exists
        if let Some(&old) = self.powers.get(&address) {
            self.total_power = self.total_power.saturating_sub(old);
        }
        // Add new power
        self.powers.insert(address, power);
        self.total_power += power;
    }

    /// Get voting power for an address
    pub fn get_power(&self, address: &[u8; 32]) -> u64 {
        self.powers.get(address).copied().unwrap_or(0)
    }
}

/// Vote delegation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// Delegator address
    pub delegator: [u8; 32],
    /// Delegate address
    pub delegate: [u8; 32],
    /// Timestamp when delegation was set
    pub timestamp: u64,
    /// Whether delegation is active
    pub active: bool,
}

impl Delegation {
    /// Create a new delegation
    pub fn new(delegator: [u8; 32], delegate: [u8; 32], timestamp: u64) -> Self {
        Self {
            delegator,
            delegate,
            timestamp,
            active: true,
        }
    }

    /// Revoke the delegation
    pub fn revoke(&mut self) {
        self.active = false;
    }
}

/// Voting power calculator
#[derive(Debug, Clone)]
pub struct VotingPowerCalculator {
    /// Minimum ATH required to vote
    pub min_voting_power: u64,
    /// Maximum voting power per address (for quadratic voting limits)
    pub max_voting_power: u64,
    /// Whether quadratic voting is enabled
    pub quadratic_voting: bool,
}

impl Default for VotingPowerCalculator {
    fn default() -> Self {
        Self {
            min_voting_power: 1_000_000_000, // 1 ATH minimum
            max_voting_power: 1_000_000_000_000, // 1000 ATH max
            quadratic_voting: false,
        }
    }
}

impl VotingPowerCalculator {
    /// Create a new calculator
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable quadratic voting
    pub fn with_quadratic(mut self) -> Self {
        self.quadratic_voting = true;
        self
    }

    /// Calculate voting power from token balance
    /// 
    /// For quadratic voting: power = sqrt(balance)
    /// For linear voting: power = balance
    pub fn calculate(&self, balance: u64) -> u64 {
        if balance < self.min_voting_power {
            return 0;
        }

        let power = if self.quadratic_voting {
            // Quadratic: sqrt(balance) * scale factor
            let sqrt = (balance as f64).sqrt() as u64;
            sqrt
        } else {
            // Linear: 1:1 with balance
            balance
        };

        power.min(self.max_voting_power)
    }

    /// Check if an address has voting power
    pub fn can_vote(&self, balance: u64) -> bool {
        balance >= self.min_voting_power
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_creation() {
        let vote = Vote::new(
            [1u8; 32],
            VoteChoice::For,
            1000,
            1000000,
            [0u8; 64],
        );
        
        assert_eq!(vote.choice, VoteChoice::For);
        assert_eq!(vote.weight, 1000);
    }

    #[test]
    fn test_vote_tally_basic() {
        let mut tally = VoteTally::new();
        
        tally.add_vote(&Vote::new([1u8; 32], VoteChoice::For, 1000, 0, [0u8; 64]));
        tally.add_vote(&Vote::new([2u8; 32], VoteChoice::For, 500, 0, [0u8; 64]));
        tally.add_vote(&Vote::new([3u8; 32], VoteChoice::Against, 800, 0, [0u8; 64]));
        
        assert_eq!(tally.for_votes, 1500);
        assert_eq!(tally.against_votes, 800);
        assert_eq!(tally.voter_count, 3);
        assert_eq!(tally.total_votes(), 2300);
    }

    #[test]
    fn test_vote_tally_quorum() {
        let mut tally = VoteTally::new();
        tally.add_vote(&Vote::new([1u8; 32], VoteChoice::For, 1000, 0, [0u8; 64]));
        
        // Quorum threshold 500
        assert!(tally.has_quorum(500));
        assert!(!tally.has_quorum(2000));
    }

    #[test]
    fn test_vote_tally_passes() {
        let mut tally = VoteTally::new();
        
        // 66% for votes (supermajority 6600 bps = 66%)
        tally.add_vote(&Vote::new([1u8; 32], VoteChoice::For, 660, 0, [0u8; 64]));
        tally.add_vote(&Vote::new([2u8; 32], VoteChoice::Against, 340, 0, [0u8; 64]));
        
        // Quorum = 500, Supermajority = 66%
        assert!(tally.passes(500, 6600));
        
        // Add more against votes
        tally.add_vote(&Vote::new([3u8; 32], VoteChoice::Against, 500, 0, [0u8; 64]));
        
        // Now fails (less than 66% for)
        assert!(!tally.passes(500, 6600));
    }

    #[test]
    fn test_voting_power_snapshot() {
        let mut snapshot = VotingPowerSnapshot::new(100, 1234567890);
        
        snapshot.set_power([1u8; 32], 1000);
        snapshot.set_power([2u8; 32], 500);
        
        assert_eq!(snapshot.get_power(&[1u8; 32]), 1000);
        assert_eq!(snapshot.get_power(&[2u8; 32]), 500);
        assert_eq!(snapshot.total_power, 1500);
    }

    #[test]
    fn test_voting_power_calculator_linear() {
        let calc = VotingPowerCalculator::new();
        
        assert_eq!(calc.calculate(2_000_000_000), 2_000_000_000);
        assert_eq!(calc.calculate(500_000_000), 0); // Below minimum
    }

    #[test]
    fn test_voting_power_calculator_quadratic() {
        let calc = VotingPowerCalculator::default().with_quadratic();
        
        // sqrt(1_000_000_000) ≈ 31622
        let power = calc.calculate(1_000_000_000);
        assert!(power > 30000 && power < 32000);
    }

    #[test]
    fn test_delegation() {
        let mut delegation = Delegation::new([1u8; 32], [2u8; 32], 1000);
        assert!(delegation.active);
        
        delegation.revoke();
        assert!(!delegation.active);
    }

    #[test]
    fn test_vote_with_reason() {
        let vote = Vote::new([1u8; 32], VoteChoice::For, 1000, 0, [0u8; 64])
            .with_reason("This proposal benefits the network".to_string());
        
        assert_eq!(vote.reason, Some("This proposal benefits the network".to_string()));
    }
}