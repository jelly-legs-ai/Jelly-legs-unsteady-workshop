// Governance Enhanced - AeTHer Chain
// Advanced governance with veto power, emergency proposals, cross-chain governance

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
    Vetoed,
    Expired,
}

/// Vote type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteType {
    For,
    Against,
    Abstain,
    Veto,  // Council veto on controversial proposals
}

/// Vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub vote_type: VoteType,
    pub voting_power: u64,
    pub timestamp: u64,
    pub reason: Option<String>,
}

/// Proposal type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Standard,        // Regular governance proposal
    Emergency,        // Fast-track emergency proposal
    Council,          // Council-only decision
    ParameterChange,  // Protocol parameter update
    Treasury,         // Treasury spending
    Upgrade,          // Protocol upgrade
    CrossChain,       // Cross-chain governance
}

/// Proposal content/action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalAction {
    pub target: String,
    pub value: u64,
    pub data: Vec<u8>,
    pub description: String,
}

/// Governance proposal enhanced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub status: ProposalStatus,
    pub actions: Vec<ProposalAction>,
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    pub veto_votes: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub execution_epoch: u64,
    pub veto_deadline_epoch: u64,
    pub quorum_required: f64,
    pub total_voting_power: u64,
    pub votes: Vec<Vote>,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
    pub discussion_link: Option<String>,
    pub is_critical: bool,
    pub emergency_dismissed: bool,
}

impl Proposal {
    pub fn get_total_votes(&self) -> u64 {
        self.for_votes + self.against_votes + self.abstain_votes + self.veto_votes
    }
    
    pub fn get_for_percentage(&self) -> f64 {
        let total = self.get_total_votes();
        if total == 0 { return 0.0; }
        (self.for_votes as f64 / total as f64) * 100.0
    }
}

/// Delegate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateInfo {
    pub delegator: String,
    pub delegatee: String,
    pub voting_power: u64,
    pub delegated_at: u64,
    pub auto_delegate: bool,
}

/// Council member info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilMember {
    pub member_id: String,
    pub name: String,
    pub voting_power: u64,
    pub is_active: bool,
    pub joined_epoch: u64,
    pub can_veto: bool,
}

/// Cross-chain governance message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainGovernanceMessage {
    pub message_id: u64,
    pub source_chain: String,
    pub target_chain: String,
    pub proposal_id: u64,
    pub action: CrossChainAction,
    pub vote_result: bool,
    pub validator_set_hash: Vec<u8>,
    pub signatures: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CrossChainAction {
    Vote,
    Execute,
    Cancel,
    UpdateValidatorSet,
}

/// Treasury information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treasury {
    pub balance: u64,
    pub total_inflow: u64,
    pub total_outflow: u64,
    pub proposals_approved: u64,
    pub last_replenish_epoch: u64,
}

impl Treasury {
    pub fn new(initial_balance: u64) -> Self {
        Self {
            balance: initial_balance,
            total_inflow: 0,
            total_outflow: 0,
            proposals_approved: 0,
            last_replenish_epoch: 0,
        }
    }
    
    pub fn replenish(&mut self, amount: u64, current_epoch: u64) {
        self.balance += amount;
        self.total_inflow += amount;
        self.last_replenish_epoch = current_epoch;
    }
    
    pub fn spend(&mut self, amount: u64, recipient: &str) -> Result<(), String> {
        if self.balance < amount {
            return Err("Insufficient treasury balance".to_string());
        }
        self.balance -= amount;
        self.total_outflow += amount;
        println!("Treasury spent {} to {}", amount, recipient);
        Ok(())
    }
}

/// Governance contract state enhanced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceContractEnhanced {
    pub proposals: HashMap<u64, Proposal>,
    pub delegates: HashMap<String, Vec<DelegateInfo>>,
    pub voting_power: HashMap<String, u64>,
    pub proposal_count: u64,
    pub voting_period_epochs: u64,
    pub emergency_voting_period_epochs: u64,
    pub quorum_threshold: f64,
    pub emergency_quorum_threshold: f64,
    pub proposal_threshold: u64,
    pub current_epoch: u64,
    pub total_proposals: u64,
    pub active_proposals: u64,
    pub council: HashMap<String, CouncilMember>,
    pub council_veto_enabled: bool,
    pub treasury: Treasury,
    pub cross_chain_messages: HashMap<u64, CrossChainGovernanceMessage>,
    pub execution_queue: Vec<u64>,
}

impl GovernanceContractEnhanced {
    /// Create new enhanced governance contract
    pub fn new() -> Self {
        GovernanceContractEnhanced {
            proposals: HashMap::new(),
            delegates: HashMap::new(),
            voting_power: HashMap::new(),
            proposal_count: 0,
            voting_period_epochs: 7 * 24,
            emergency_voting_period_epochs: 1 * 24,
            quorum_threshold: 0.60,
            emergency_quorum_threshold: 0.75,
            proposal_threshold: 100,
            current_epoch: 0,
            total_proposals: 0,
            active_proposals: 0,
            council: HashMap::new(),
            council_veto_enabled: true,
            treasury: Treasury::new(10_000_000_000_000),
            cross_chain_messages: HashMap::new(),
            execution_queue: Vec::new(),
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
            match proposal.status {
                ProposalStatus::Pending if self.current_epoch >= proposal.start_epoch => {
                    proposal.status = ProposalStatus::Active;
                    self.active_proposals += 1;
                }
                ProposalStatus::Active if self.current_epoch >= proposal.end_epoch => {
                    proposal.status = if self.has_passed(proposal) {
                        ProposalStatus::Passed
                    } else {
                        ProposalStatus::Failed
                    };
                    self.active_proposals = self.active_proposals.saturating_sub(1);
                    
                    if proposal.status == ProposalStatus::Passed {
                        self.execution_queue.push(proposal.id);
                    }
                }
                ProposalStatus::Passed if proposal.proposal_type == ProposalType::Emergency 
                    && self.current_epoch >= proposal.veto_deadline_epoch => {
                    proposal.status = ProposalStatus::Executed;
                }
                _ => {}
            }
        }
    }

    /// Check if proposal has passed
    fn has_passed(&self, proposal: &Proposal) -> bool {
        // Check veto first
        if proposal.veto_votes > 0 {
            let veto_threshold = proposal.total_voting_power / 10;
            if proposal.veto_votes >= veto_threshold {
                return false;
            }
        }
        
        let total_votes = proposal.get_total_votes();
        let quorum = total_votes as f64 / proposal.total_voting_power as f64;
        let quorum_req = if proposal.proposal_type == ProposalType::Emergency {
            self.emergency_quorum_threshold
        } else {
            self.quorum_threshold
        };
        
        if quorum < quorum_req {
            return false;
        }
        
        proposal.for_votes > proposal.against_votes
    }

    /// Create a new governance proposal
    pub fn create_proposal(
        &mut self,
        proposer: String,
        proposal_type: ProposalType,
        title: String,
        description: String,
        actions: Vec<ProposalAction>,
        tags: Vec<String>,
        is_critical: bool,
    ) -> Result<Proposal, String> {
        let voting_power = self.voting_power.get(&proposer).unwrap_or(&0);
        let threshold = self.get_proposal_threshold(&proposal_type);
        
        if *voting_power < threshold {
            return Err(format!(
                "Insufficient voting power for {} proposal. Required: {}, Has: {}",
                format!("{:?}", proposal_type).to_lowercase(),
                threshold, voting_power
            ));
        }

        self.proposal_count += 1;
        let proposal_id = self.proposal_count;
        let (start_epoch, end_epoch, veto_deadline_epoch) = self.calculate_timing(&proposal_type);
        let quorum = if proposal_type == ProposalType::Emergency {
            self.emergency_quorum_threshold
        } else {
            self.quorum_threshold
        };

        let proposal = Proposal {
            id: proposal_id,
            proposal_type,
            title,
            description,
            proposer,
            status: ProposalStatus::Pending,
            actions,
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            veto_votes: 0,
            start_epoch,
            end_epoch,
            execution_epoch: end_epoch + 1,
            veto_deadline_epoch,
            quorum_required: quorum,
            total_voting_power: *self.voting_power.get(&proposer).unwrap_or(&0),
            votes: Vec::new(),
            metadata: HashMap::new(),
            tags,
            discussion_link: None,
            is_critical,
            emergency_dismissed: false,
        };

        self.total_proposals += 1;
        self.proposals.insert(proposal_id, proposal.clone());
        Ok(proposal)
    }

    fn get_proposal_threshold(&self, proposal_type: &ProposalType) -> u64 {
        match proposal_type {
            ProposalType::Standard => self.proposal_threshold,
            ProposalType::Emergency => self.proposal_threshold * 5,
            ProposalType::Council => self.proposal_threshold * 10,
            ProposalType::ParameterChange => self.proposal_threshold / 2,
            ProposalType::Treasury => self.proposal_threshold * 3,
            ProposalType::Upgrade => self.proposal_threshold * 10,
            ProposalType::CrossChain => self.proposal_threshold * 5,
        }
    }

    fn calculate_timing(&self, proposal_type: &ProposalType) -> (u64, u64, u64) {
        let start_epoch = self.current_epoch + 1;
        let voting_period = match proposal_type {
            ProposalType::Emergency => self.emergency_voting_period_epochs,
            _ => self.voting_period_epochs,
        };
        let end_epoch = start_epoch + voting_period;
        let veto_deadline = end_epoch + (self.council_veto_enabled as u64 * 24);
        (start_epoch, end_epoch, veto_deadline)
    }

    /// Cast a vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: u64,
        voter: String,
        vote_type: VoteType,
        voting_power: u64,
        reason: Option<String>,
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
            reason,
        };

        match vote_type {
            VoteType::For => proposal.for_votes += voting_power,
            VoteType::Against => proposal.against_votes += voting_power,
            VoteType::Abstain => proposal.abstain_votes += voting_power,
            VoteType::Veto => {
                if !self.is_council_member(&voter) {
                    return Err("Only council members can veto".to_string());
                }
                proposal.veto_votes += voting_power;
            }
        }

        proposal.votes.push(vote);
        Ok(())
    }

    fn is_council_member(&self, member_id: &str) -> bool {
        self.council.get(member_id)
            .map(|m| m.is_active && m.can_veto)
            .unwrap_or(false)
    }

    /// Council veto on a proposal
    pub fn council_veto(
        &mut self,
        proposal_id: u64,
        council_member: String,
        reason: String,
    ) -> Result<(), String> {
        if !self.council_veto_enabled {
            return Err("Council veto is disabled".to_string());
        }
        
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or("Proposal not found")?;
            
        if proposal.proposal_type == ProposalType::Council {
            return Err("Cannot veto council proposals".to_string());
        }
        
        let member = self.council.get_mut(&council_member)
            .ok_or("Council member not found")?;
            
        if !member.can_veto {
            return Err("This council member cannot veto".to_string());
        }
        
        let vote = Vote {
            voter: council_member,
            vote_type: VoteType::Veto,
            voting_power: member.voting_power,
            timestamp: self.current_epoch,
            reason: Some(reason),
        };
        
        proposal.veto_votes += member.voting_power;
        proposal.votes.push(vote);
        
        Ok(())
    }

    /// Add council member
    pub fn add_council_member(&mut self, member: CouncilMember) -> Result<(), String> {
        if self.council.contains_key(&member.member_id) {
            return Err("Council member already exists".to_string());
        }
        self.council.insert(member.member_id.clone(), member);
        Ok(())
    }

    /// Remove council member
    pub fn remove_council_member(&mut self, member_id: &str) -> Result<(), String> {
        let member = self.council.get_mut(member_id)
            .ok_or("Council member not found")?;
        member.is_active = false;
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

        // Handle treasury proposals
        if proposal.proposal_type == ProposalType::Treasury {
            self.execute_treasury_actions(proposal)?;
        }

        proposal.status = ProposalStatus::Executed;
        self.execution_queue.retain(|&id| id != proposal_id);
        Ok(proposal.actions.clone())
    }

    fn execute_treasury_actions(&mut self, proposal: &mut Proposal) -> Result<(), String> {
        for action in &proposal.actions {
            self.treasury.spend(action.value, &action.target)?;
        }
        self.treasury.proposals_approved += 1;
        Ok(())
    }

    /// Delegate voting power with auto-delegate option
    pub fn delegate(
        &mut self, 
        delegator: String, 
        delegatee: String, 
        amount: u64,
        auto_delegate: bool,
    ) -> Result<(), String> {
        if amount == 0 {
            return Err("Amount must be greater than 0".to_string());
        }

        let delegator_power = self.voting_power.get(&delegator).unwrap_or(&0);
        if *delegator_power < amount {
            return Err("Insufficient voting power to delegate".to_string());
        }

        *self.voting_power.entry(delegator.clone()).or_insert(0) -= amount;
        *self.voting_power.entry(delegatee.clone()).or_insert(0) += amount;

        let delegation = DelegateInfo {
            delegator,
            delegatee,
            voting_power: amount,
            delegated_at: self.current_epoch,
            auto_delegate,
        };

        self.delegates
            .entry(delegation.delegator.clone())
            .or_insert_with(Vec::new)
            .push(delegation);

        Ok(())
    }

    /// Revoke delegation
    pub fn revoke_delegation(&mut self, delegator: String, delegatee: &str, amount: u64) -> Result<(), String> {
        let delegations = self.delegates.get_mut(&delegator)
            .ok_or("No delegations found")?;
        
        let total_delegated: u64 = delegations.iter()
            .filter(|d| d.delegatee == delegatee)
            .map(|d| d.voting_power)
            .sum();
            
        if total_delegated < amount {
            return Err("Insufficient delegated amount".to_string());
        }
        
        // Reduce delegatee's voting power
        *self.voting_power.entry(delegatee.to_string()).or_insert(0) -= amount;
        *self.voting_power.entry(delegator.clone()).or_insert(0) += amount;
        
        // Update delegation records
        let mut remaining = amount;
        for d in delegations.iter_mut().rev() {
            if d.delegatee == delegatee && remaining > 0 {
                let reduce = remaining.min(d.voting_power);
                d.voting_power -= reduce;
                remaining -= reduce;
            }
        }
        
        delegations.retain(|d| d.voting_power > 0);
        Ok(())
    }

    /// Send cross-chain governance message
    pub fn send_cross_chain_message(
        &mut self,
        target_chain: String,
        proposal_id: u64,
        action: CrossChainAction,
    ) -> Result<CrossChainGovernanceMessage, String> {
        let proposal = self.proposals.get(proposal_id)
            .ok_or("Proposal not found")?;
            
        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal must be passed to send cross-chain message".to_string());
        }
        
        let message_id = self.cross_chain_messages.len() as u64 + 1;
        
        let message = CrossChainGovernanceMessage {
            message_id,
            source_chain: "AetherChain".to_string(),
            target_chain,
            proposal_id,
            action,
            vote_result: true,
            validator_set_hash: vec![],
            signatures: vec![],
        };
        
        self.cross_chain_messages.insert(message_id, message.clone());
        Ok(message)
    }

    /// Get active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    /// Get proposals by type
    pub fn get_proposals_by_type(&self, proposal_type: &ProposalType) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| &p.proposal_type == proposal_type)
            .collect()
    }

    /// Get proposals by tag
    pub fn get_proposals_by_tag(&self, tag: &str) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get voter stats
    pub fn get_voter_stats(&self, voter: &str) -> VoterStats {
        let proposals_voted: Vec<&Proposal> = self.proposals
            .values()
            .filter(|p| p.votes.iter().any(|v| &v.voter == voter))
            .collect();
            
        let for_votes = proposals_voted.iter()
            .filter(|p| p.votes.iter().any(|v| &v.voter == voter && v.vote_type == VoteType::For))
            .count();
            
        let against_votes = proposals_voted.iter()
            .filter(|p| p.votes.iter().any(|v| &v.voter == voter && v.vote_type == VoteType::Against))
            .count();
            
        let consensus_rate = if !proposals_voted.is_empty() {
            proposals_voted.iter()
                .filter(|p| p.status == ProposalStatus::Passed && p.for_votes > p.against_votes)
                .count() as f64 / proposals_voted.len() as f64
        } else {
            0.0
        };
        
        VoterStats {
            total_votes: proposals_voted.len(),
            for_votes,
            against_votes,
            consensus_rate,
            voting_power: *self.voting_power.get(voter).unwrap_or(&0),
        }
    }

    /// Calculate current quorum
    pub fn calculate_quorum(&self, proposal_id: u64) -> Option<f64> {
        self.proposals.get(&proposal_id).map(|p| {
            let total_votes = p.get_total_votes();
            total_votes as f64 / p.total_voting_power as f64
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterStats {
    pub total_votes: usize,
    pub for_votes: usize,
    pub against_votes: usize,
    pub consensus_rate: f64,
    pub voting_power: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_standard_proposal() {
        let mut gov = GovernanceContractEnhanced::new();
        gov.voting_power.insert("alice".to_string(), 500);
        
        let result = gov.create_proposal(
            "alice".to_string(),
            ProposalType::Standard,
            "Test Proposal".to_string(),
            "Description".to_string(),
            vec![],
            vec!["test".to_string()],
            false,
        );
        
        assert!(result.is_ok());
        let proposal = result.unwrap();
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.status, ProposalStatus::Pending);
    }

    #[test]
    fn test_council_veto() {
        let mut gov = GovernanceContractEnhanced::new();
        gov.voting_power.insert("alice".to_string(), 500);
        gov.voting_power.insert("council1".to_string(), 1000);
        
        gov.add_council_member(CouncilMember {
            member_id: "council1".to_string(),
            name: "Council One".to_string(),
            voting_power: 1000,
            is_active: true,
            joined_epoch: 0,
            can_veto: true,
        }).unwrap();
        
        let proposal = gov.create_proposal(
            "alice".to_string(),
            ProposalType::Standard,
            "Test".to_string(),
            "Test".to_string(),
            vec![],
            vec![],
            false,
        ).unwrap();
        
        gov.set_epoch(proposal.start_epoch);
        
        // Council vetoes
        let result = gov.council_veto(proposal.id, "council1".to_string(), "Too risky".to_string());
        assert!(result.is_ok());
        
        // Check veto votes
        let p = gov.get_proposal(proposal.id).unwrap();
        assert!(p.veto_votes > 0);
    }

    #[test]
    fn test_treasury_spending() {
        let mut gov = GovernanceContractEnhanced::new();
        gov.voting_power.insert("alice".to_string(), 1000);
        
        let proposal = gov.create_proposal(
            "alice".to_string(),
            ProposalType::Treasury,
            "Fund Development".to_string(),
            "Fund core development".to_string(),
            vec![
                ProposalAction {
                    target: "dev_team".to_string(),
                    value: 100_000_000,
                    data: vec![],
                    description: "Q1 Development Funding".to_string(),
                }
            ],
            vec!["treasury".to_string()],
            false,
        ).unwrap();
        
        gov.set_epoch(proposal.start_epoch);
        gov.vote(proposal.id, "alice".to_string(), VoteType::For, 1000, None).unwrap();
        gov.set_epoch(proposal.end_epoch + 1);
        
        let result = gov.execute_proposal(proposal.id);
        assert!(result.is_ok());
    }
}
