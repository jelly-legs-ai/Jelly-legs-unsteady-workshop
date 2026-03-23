// Governance Contract - AeTHer Chain
// On-chain governance for AETH token holders

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Proposal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Active,
    Passed,
    Failed,
    Queued,
    Executed,
    Expired,
}

/// Vote choice
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

/// Proposal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    Text,
    ParameterChange,
    TreasurySpend,
    EmergencyShutdown,
    ProtocolUpgrade,
    CommunityFund,
}

/// Proposal details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub author: String,
    pub status: ProposalStatus,
    pub created_epoch: u64,
    pub voting_start_epoch: u64,
    pub voting_end_epoch: u64,
    pub execution_epoch: Option<u64>,
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    pub total_votes: u64,
    pub quorum_required: u64,
    pub execution_data: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Individual vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub choice: VoteChoice,
    pub voting_power: u64,
    pub epoch: u64,
    pub reason: Option<String>,
}

/// Delegation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub delegator: String,
    pub delegate: String,
    pub voting_power: u64,
    pub delegated_epoch: u64,
}

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub quorum_threshold: u64,      // Minimum % of votes required (e.g., 4000 = 40%)
    pub approval_threshold: u64,     // Minimum % of for votes (e.g., 5000 = 50%+1)
    pub voting_period_epochs: u64,   // How long voting lasts
    pub execution_delay_epochs: u64, // Delay after passing before execution
    pub proposal_threshold: u64,     // Minimum AETH to create proposal
    pub max_proposals: u64,          // Maximum active proposals
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            quorum_threshold: 4000,      // 40% quorum
            approval_threshold: 5000,    // 50%+ of votes must be For
            voting_period_epochs: 168,   // ~1 week at 1 epoch = 1 hour
            execution_delay_epochs: 24,  // ~1 day delay after passing
            proposal_threshold: 10000,   // 10,000 AETH to create proposal
            max_proposals: 50,
        }
    }
}

/// Governor contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceContract {
    pub proposals: HashMap<String, Proposal>,
    pub votes: HashMap<String, Vec<Vote>>,
    pub delegations: HashMap<String, Delegation>,
    pub config: GovernanceConfig,
    pub current_epoch: u64,
    pub total_proposals: u64,
    pub executed_proposals: u64,
}

impl GovernanceContract {
    /// Create new governance contract
    pub fn new() -> Self {
        GovernanceContract {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            delegations: HashMap::new(),
            config: GovernanceConfig::default(),
            current_epoch: 0,
            total_proposals: 0,
            executed_proposals: 0,
        }
    }

    /// Create a new proposal
    pub fn create_proposal(
        &mut self,
        proposer: String,
        proposal_type: ProposalType,
        title: String,
        description: String,
        execution_data: Option<String>,
    ) -> Result<Proposal, &'static str> {
        // Check proposal limit
        let active_count = self.proposals.values()
            .filter(|p| p.status == ProposalStatus::Active || p.status == ProposalStatus::Pending)
            .count() as u64;
        
        if active_count >= self.config.max_proposals {
            return Err("Maximum number of active proposals reached");
        }

        let proposal_id = format!("AIP-{}", self.total_proposals + 1);
        
        let proposal = Proposal {
            id: proposal_id.clone(),
            proposal_type,
            title,
            description,
            author: proposer,
            status: ProposalStatus::Pending,
            created_epoch: self.current_epoch,
            voting_start_epoch: self.current_epoch + 1,
            voting_end_epoch: self.current_epoch + 1 + self.config.voting_period_epochs,
            execution_epoch: None,
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            total_votes: 0,
            quorum_required: self.config.quorum_threshold,
            execution_data,
            metadata: HashMap::new(),
        };

        self.proposals.insert(proposal_id.clone(), proposal.clone());
        self.total_proposals += 1;

        Ok(proposal)
    }

    /// Activate a pending proposal (moves to active voting)
    pub fn activate_proposal(&mut self, proposal_id: &str) -> Result<(), &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Pending {
            return Err("Proposal is not in pending state");
        }

        if self.current_epoch < proposal.voting_start_epoch {
            return Err("Voting period has not started yet");
        }

        proposal.status = ProposalStatus::Active;
        Ok(())
    }

    /// Cast a vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: &str,
        voter: String,
        choice: VoteChoice,
        voting_power: u64,
        reason: Option<String>,
    ) -> Result<(), &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        // Check proposal is active
        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not in active voting state");
        }

        // Check voting window
        if self.current_epoch < proposal.voting_start_epoch {
            return Err("Voting has not started");
        }
        if self.current_epoch >= proposal.voting_end_epoch {
            return Err("Voting period has ended");
        }

        // Record vote
        let vote = Vote {
            voter: voter.clone(),
            choice: choice.clone(),
            voting_power,
            epoch: self.current_epoch,
            reason,
        };

        // Update proposal vote counts
        match choice {
            VoteChoice::For => {
                proposal.for_votes += voting_power;
            },
            VoteChoice::Against => {
                proposal.against_votes += voting_power;
            },
            VoteChoice::Abstain => {
                proposal.abstain_votes += voting_power;
            },
        }
        proposal.total_votes += voting_power;

        // Store vote record
        let key = proposal_id.to_string();
        let votes = self.votes.entry(key).or_insert_with(Vec::new);
        
        // Remove existing vote if any
        votes.retain(|v| v.voter != voter);
        votes.push(vote);

        Ok(())
    }

    /// Finalize a proposal after voting ends
    pub fn finalize_proposal(&mut self, proposal_id: &str) -> Result<ProposalStatus, &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not active");
        }

        if self.current_epoch < proposal.voting_end_epoch {
            return Err("Voting period has not ended");
        }

        // Calculate percentages
        let total = proposal.total_votes.max(1);
        let for_percent = (proposal.for_votes * 10000) / total;
        let against_percent = (proposal.against_votes * 10000) / total;
        let quorum_percent = (proposal.total_votes * 10000) / (proposal.quorum_required.max(1));

        // Determine outcome
        let passed = for_percent >= self.config.approval_threshold 
            && quorum_percent >= 10000;

        proposal.status = if passed {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Failed
        };

        Ok(proposal.status.clone())
    }

    /// Queue a passed proposal for execution
    pub fn queue_proposal(&mut self, proposal_id: &str) -> Result<(), &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal has not passed");
        }

        proposal.status = ProposalStatus::Queued;
        proposal.execution_epoch = Some(
            self.current_epoch + self.config.execution_delay_epochs
        );

        Ok(())
    }

    /// Execute a queued proposal
    pub fn execute_proposal(&mut self, proposal_id: &str) -> Result<(), &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Queued {
            return Err("Proposal is not queued");
        }

        if let Some(exec_epoch) = proposal.execution_epoch {
            if self.current_epoch < exec_epoch {
                return Err("Execution delay has not elapsed");
            }
        }

        proposal.status = ProposalStatus::Executed;
        self.executed_proposals += 1;

        Ok(())
    }

    /// Delegate voting power to another address
    pub fn delegate(&mut self, delegator: String, delegate: String, voting_power: u64) -> Result<(), &'static str> {
        if delegator == delegate {
            return Err("Cannot delegate to yourself");
        }

        let delegation = Delegation {
            delegator: delegator.clone(),
            delegate: delegate.clone(),
            voting_power,
            delegated_epoch: self.current_epoch,
        };

        self.delegations.insert(delegator, delegation);
        Ok(())
    }

    /// Undelegate voting power
    pub fn undelegate(&mut self, delegator: &str) -> Result<(), &'static str> {
        self.delegations.remove(delegator)
            .ok_or("No active delegation found")?;
        Ok(())
    }

    /// Get voting power for an address (own + delegated)
    pub fn get_voting_power(&self, address: &str, own_stake: u64) -> u64 {
        let delegated: u64 = self.delegations.values()
            .filter(|d| d.delegate == address)
            .map(|d| d.voting_power)
            .sum();
        
        own_stake + delegated
    }

    /// Advance epoch
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
        
        // Update proposal statuses
        for proposal in self.proposals.values_mut() {
            match proposal.status {
                ProposalStatus::Pending if self.current_epoch >= proposal.voting_start_epoch => {
                    proposal.status = ProposalStatus::Active;
                },
                ProposalStatus::Active if self.current_epoch >= proposal.voting_end_epoch => {
                    // Auto-finalize
                    let total = proposal.total_votes.max(1);
                    let for_percent = (proposal.for_votes * 10000) / total;
                    let quorum_percent = (proposal.total_votes * 10000) / (proposal.quorum_required.max(1));
                    
                    proposal.status = if for_percent >= self.config.approval_threshold 
                        && quorum_percent >= 10000 {
                        ProposalStatus::Passed
                    } else {
                        ProposalStatus::Failed
                    };
                },
                ProposalStatus::Passed if proposal.execution_epoch.is_some() 
                    && self.current_epoch >= proposal.execution_epoch.unwrap() => {
                    proposal.status = ProposalStatus::Executed;
                    self.executed_proposals += 1;
                },
                _ => {}
            }
        }
    }

    /// Get proposal results summary
    pub fn get_proposal_summary(&self, proposal_id: &str) -> Option<ProposalSummary> {
        self.proposals.get(proposal_id).map(|p| {
            let total = p.total_votes.max(1);
            ProposalSummary {
                id: p.id.clone(),
                title: p.title.clone(),
                status: p.status.clone(),
                for_percent: (p.for_votes * 100) / total,
                against_percent: (p.against_votes * 100) / total,
                abstain_percent: (p.abstain_votes * 100) / total,
                total_votes: p.total_votes,
                quorum_required: p.quorum_required,
            }
        })
    }
}

/// Summary view of a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalSummary {
    pub id: String,
    pub title: String,
    pub status: ProposalStatus,
    pub for_percent: u64,
    pub against_percent: u64,
    pub abstain_percent: u64,
    pub total_votes: u64,
    pub quorum_required: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let mut gov = GovernanceContract::new();
        let proposal = gov.create_proposal(
            "0x123".to_string(),
            ProposalType::Text,
            "Test Proposal".to_string(),
            "Description".to_string(),
            None,
        ).unwrap();
        
        assert_eq!(proposal.id, "AIP-1");
        assert_eq!(proposal.status, ProposalStatus::Pending);
    }

    #[test]
    fn test_voting() {
        let mut gov = GovernanceContract::new();
        
        gov.create_proposal(
            "0x123".to_string(),
            ProposalType::TreasurySpend,
            "Treasury Spend".to_string(),
            "Spend 1000 AETH".to_string(),
            Some("transfer:0x456:1000".to_string()),
        ).unwrap();
        
        gov.advance_epoch();
        gov.activate_proposal("AIP-1").unwrap();
        
        gov.vote("AIP-1", "voter1".to_string(), VoteChoice::For, 1000, None).unwrap();
        gov.vote("AIP-1", "voter2".to_string(), VoteChoice::Against, 500, None).unwrap();
        
        let summary = gov.get_proposal_summary("AIP-1").unwrap();
        assert_eq!(summary.for_percent, 67);
        assert_eq!(summary.against_percent, 33);
    }

    #[test]
    fn test_delegation() {
        let mut gov = GovernanceContract::new();
        gov.delegate(" delegator".to_string(), "delegate".to_string(), 500).unwrap();
        
        let power = gov.get_voting_power("delegate", 1000);
        assert_eq!(power, 1500);
    }
}
