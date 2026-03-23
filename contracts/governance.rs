// Governance Contract - AeTHer Chain
// On-chain governance with proposals, voting, and execution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Proposal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Active,
    Passed,
    Failed,
    Executed,
    Cancelled,
}

/// Vote type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteType {
    For,
    Against,
    Abstain,
}

/// Vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub vote_type: VoteType,
    pub voting_power: u64,
    pub timestamp: u64,
}

/// Proposal content/action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalAction {
    pub target: String,
    pub value: u64,
    pub data: Vec<u8>,
    pub description: String,
}

/// Governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub status: ProposalStatus,
    pub actions: Vec<ProposalAction>,
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub execution_epoch: u64,
    pub quorum_required: f64,
    pub total_voting_power: u64,
    pub votes: Vec<Vote>,
    pub metadata: HashMap<String, String>,
}

/// Delegate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateInfo {
    pub delegator: String,
    pub delegatee: String,
    pub voting_power: u64,
    pub delegated_at: u64,
}

/// Governance contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceContract {
    pub proposals: HashMap<u64, Proposal>,
    pub delegates: HashMap<String, Vec<DelegateInfo>>,
    pub voting_power: HashMap<String, u64>,
    pub proposal_count: u64,
    pub voting_period_epochs: u64,
    pub quorum_threshold: f64,
    pub proposal_threshold: u64,
    pub current_epoch: u64,
    pub total_proposals: u64,
    pub active_proposals: u64,
}

impl GovernanceContract {
    /// Create new governance contract
    pub fn new() -> Self {
        GovernanceContract {
            proposals: HashMap::new(),
            delegates: HashMap::new(),
            voting_power: HashMap::new(),
            proposal_count: 0,
            voting_period_epochs: 7 * 24, // 7 days in epochs (1 epoch = 1 hour)
            quorum_threshold: 0.60, // 60% quorum required
            proposal_threshold: 100, // Minimum 100 AETH to propose
            current_epoch: 0,
            total_proposals: 0,
            active_proposals: 0,
        }
    }

    /// Set current epoch
    pub fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
        self.update_proposal_statuses();
    }

    /// Update proposal statuses based on current epoch
    fn update_proposal_statuses(&mut self) {
        for proposal in self.proposals.values_mut() {
            if proposal.status == ProposalStatus::Pending && self.current_epoch >= proposal.start_epoch {
                proposal.status = ProposalStatus::Active;
                self.active_proposals += 1;
            }
            if proposal.status == ProposalStatus::Active && self.current_epoch >= proposal.end_epoch {
                proposal.status = if self.has_passed(proposal) {
                    ProposalStatus::Passed
                } else {
                    ProposalStatus::Failed
                };
                self.active_proposals = self.active_proposals.saturating_sub(1);
            }
        }
    }

    /// Check if proposal has passed
    fn has_passed(&self, proposal: &Proposal) -> bool {
        let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        let quorum = total_votes as f64 / proposal.total_voting_power as f64;
        
        // Check quorum
        if quorum < proposal.quorum_required {
            return false;
        }
        
        // Check majority
        proposal.for_votes > proposal.against_votes
    }

    /// Create a new governance proposal
    pub fn create_proposal(
        &mut self,
        proposer: String,
        title: String,
        description: String,
        actions: Vec<ProposalAction>,
    ) -> Result<Proposal, String> {
        // Check proposal threshold
        let voting_power = self.voting_power.get(&proposer).unwrap_or(&0);
        if *voting_power < self.proposal_threshold {
            return Err(format!(
                "Insufficient voting power. Required: {}, Has: {}",
                self.proposal_threshold, voting_power
            ));
        }

        self.proposal_count += 1;
        let proposal_id = self.proposal_count;
        let start_epoch = self.current_epoch + 1;
        let end_epoch = start_epoch + self.voting_period_epochs;

        let proposal = Proposal {
            id: proposal_id,
            title,
            description,
            proposer,
            status: ProposalStatus::Pending,
            actions,
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            start_epoch,
            end_epoch,
            execution_epoch: end_epoch + 1,
            quorum_required: self.quorum_threshold,
            total_voting_power: *self.voting_power.get(&proposer).unwrap_or(&0),
            votes: Vec::new(),
            metadata: HashMap::new(),
        };

        self.total_proposals += 1;
        self.proposals.insert(proposal_id, proposal.clone());
        Ok(proposal)
    }

    /// Cast a vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: u64,
        voter: String,
        vote_type: VoteType,
        voting_power: u64,
    ) -> Result<(), String> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not active".to_string());
        }

        if self.current_epoch >= proposal.end_epoch {
            return Err("Voting period has ended".to_string());
        }

        // Check if already voted
        if proposal.votes.iter().any(|v| v.voter == voter) {
            return Err("Already voted on this proposal".to_string());
        }

        let vote = Vote {
            voter: voter.clone(),
            vote_type: vote_type.clone(),
            voting_power,
            timestamp: self.current_epoch,
        };

        match vote_type {
            VoteType::For => proposal.for_votes += voting_power,
            VoteType::Against => proposal.against_votes += voting_power,
            VoteType::Abstain => proposal.abstain_votes += voting_power,
        }

        proposal.votes.push(vote);
        Ok(())
    }

    /// Execute a passed proposal
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<Vec<ProposalAction>, String> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal has not passed".to_string());
        }

        if self.current_epoch < proposal.execution_epoch {
            return Err("Execution epoch not reached".to_string());
        }

        proposal.status = ProposalStatus::Executed;
        Ok(proposal.actions.clone())
    }

    /// Cancel a proposal
    pub fn cancel_proposal(&mut self, proposal_id: u64, caller: String) -> Result<(), String> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.proposer != caller {
            return Err("Only proposer can cancel".to_string());
        }

        if proposal.status == ProposalStatus::Executed {
            return Err("Cannot cancel executed proposal".to_string());
        }

        if proposal.status == ProposalStatus::Active {
            self.active_proposals = self.active_proposals.saturating_sub(1);
        }

        proposal.status = ProposalStatus::Cancelled;
        Ok(())
    }

    /// Delegate voting power
    pub fn delegate(&mut self, delegator: String, delegatee: String, amount: u64) -> Result<(), String> {
        if amount == 0 {
            return Err("Amount must be greater than 0".to_string());
        }

        let delegator_power = self.voting_power.get(&delegator).unwrap_or(&0);
        if *delegator_power < amount {
            return Err("Insufficient voting power to delegate".to_string());
        }

        // Reduce delegator's voting power
        *self.voting_power.entry(delegator.clone()).or_insert(0) -= amount;
        
        // Add to delegatee's voting power
        *self.voting_power.entry(delegatee.clone()).or_insert(0) += amount;

        // Record delegation
        let delegation = DelegateInfo {
            delegator,
            delegatee,
            voting_power: amount,
            delegated_at: self.current_epoch,
        };

        self.delegates
            .entry(delegation.delegator.clone())
            .or_insert_with(Vec::new)
            .push(delegation);

        Ok(())
    }

    /// Get active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&Proposal> {
        self.proposals.get(&proposal_id)
    }

    /// Get voter's vote on a proposal
    pub fn get_voter_vote(&self, proposal_id: u64, voter: &str) -> Option<&Vote> {
        self.proposals
            .get(&proposal_id)
            .and_then(|p| p.votes.iter().find(|v| v.voter == voter))
    }

    /// Calculate current quorum
    pub fn calculate_quorum(&self, proposal_id: u64) -> Option<f64> {
        self.proposals.get(&proposal_id).map(|p| {
            let total_votes = p.for_votes + p.against_votes + p.abstain_votes;
            total_votes as f64 / p.total_voting_power as f64
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let mut gov = GovernanceContract::new();
        gov.voting_power.insert("alice".to_string(), 500);
        
        let result = gov.create_proposal(
            "alice".to_string(),
            "Test Proposal".to_string(),
            "Description".to_string(),
            vec![],
        );
        
        assert!(result.is_ok());
        let proposal = result.unwrap();
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.status, ProposalStatus::Pending);
    }

    #[test]
    fn test_vote_on_proposal() {
        let mut gov = GovernanceContract::new();
        gov.voting_power.insert("alice".to_string(), 500);
        gov.voting_power.insert("bob".to_string(), 300);
        
        let proposal = gov.create_proposal(
            "alice".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            vec![],
        ).unwrap();
        
        gov.set_epoch(proposal.start_epoch);
        
        let result = gov.vote(proposal.id, "bob".to_string(), VoteType::For, 300);
        assert!(result.is_ok());
        
        let p = gov.get_proposal(proposal.id).unwrap();
        assert_eq!(p.for_votes, 300);
    }
}
