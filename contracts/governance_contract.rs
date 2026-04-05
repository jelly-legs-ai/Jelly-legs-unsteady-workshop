// Governance Contract - AeTHer Chain
// Quadratic voting-based DAO governance for protocol upgrades

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Proposal status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Active,
    Passed,
    Failed,
    Executed,
    Queued,
    Expired,
}

/// Proposal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Protocol parameter changes
    ParameterChange {
        parameter: String,
        new_value: String,
    },
    /// Treasury allocation
    TreasuryAllocation {
        recipient: String,
        amount: u64,
        token: TokenType,
    },
    /// Smart contract upgrade
    ContractUpgrade {
        contract_id: String,
        new_code_hash: String,
    },
    /// Community fund distribution
    CommunityFund {
        description: String,
        distributions: Vec<Distribution>,
    },
    /// Emergency security action
    EmergencySecurity {
        action: String,
        target: String,
    },
}

/// Token type for treasury operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TokenType {
    AETH,
    FLUX,
    ATH,
}

/// Distribution for community fund proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Distribution {
    pub recipient: String,
    pub amount: u64,
    pub percentage: f64,
}

/// Vote choice
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

/// Individual vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub choice: VoteChoice,
    pub voting_power: u64,
    pub quadratic_power: f64,
    pub timestamp: u64,
    pub reason: Option<String>,
}

/// Proposal structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub status: ProposalStatus,
    pub author: String,
    pub created_at: u64,
    pub voting_start: u64,
    pub voting_end: u64,
    pub execution_delay: u64,
    pub execution_time: Option<u64>,
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    pub total_voters: u64,
    pub quorum: u64,
    pub vote_counts: HashMap<VoteChoice, u64>,
    pub voters: Vec<Vote>,
    pub execution_data: Option<String>,
}

/// Delegated vote tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub delegator: String,
    pub delegate: String,
    pub voting_power: u64,
    pub locked_until: u64,
}

/// Governor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorSettings {
    pub voting_period: u64,        // in epochs
    pub quorum_threshold: u64,      // minimum votes required (raw)
    pub quorum_percentage: f64,    // percentage of total supply
    pub proposal_threshold: u64,   // minimum tokens to create proposal
    pub execution_delay: u64,      // delay before execution (epochs)
    pub veto_enabled: bool,
    pub veto_threshold: u64,
    pub quadratic_slope: f64,     // slope for quadratic voting calculation
    pub quadratic_constant: f64,   // constant for quadratic voting
}

impl Default for GovernorSettings {
    fn default() -> Self {
        GovernorSettings {
            voting_period: 168,           // ~7 days (168 epochs at 1 hour each)
            quorum_threshold: 100_000_000, // 100M tokens
            quorum_percentage: 4.0,        // 4% of total supply
            proposal_threshold: 10_000_000, // 10M tokens to create
            execution_delay: 48,            // 48 epochs (~2 days)
            veto_enabled: true,
            veto_threshold: 50_000_000,    // 50M tokens
            quadratic_slope: 1.0,
            quadratic_constant: 1.0,
        }
    }
}

/// Governance contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceContract {
    pub proposals: HashMap<String, Proposal>,
    pub delegates: HashMap<String, Vec<Delegation>>,
    pub delegators: HashMap<String, String>,  // delegator -> delegate
    pub settings: GovernorSettings,
    pub total_proposals: u64,
    pub proposal_counter: u64,
    pub treasury_balance: HashMap<TokenType, u64>,
    pub emergency_actions: Vec<EmergencyAction>,
    pub veto_power_used: u64,
}

/// Emergency action record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAction {
    pub id: String,
    pub action_type: String,
    pub target: String,
    pub executed_by: String,
    pub timestamp: u64,
    pub reason: String,
}

impl GovernanceContract {
    /// Create new governance contract
    pub fn new() -> Self {
        GovernanceContract {
            proposals: HashMap::new(),
            delegates: HashMap::new(),
            delegators: HashMap::new(),
            settings: GovernorSettings::default(),
            total_proposals: 0,
            proposal_counter: 0,
            treasury_balance: HashMap::new(),
            emergency_actions: Vec::new(),
            veto_power_used: 0,
        }
    }

    /// Calculate quadratic voting power
    /// Uses sqrt(votes) to prevent governance capture by large holders
    pub fn calculate_quadratic_vote(&self, token_amount: u64) -> f64 {
        let tokens = token_amount as f64;
        let raw_power = (self.settings.quadratic_constant + 
                        tokens * self.settings.quadratic_slope).sqrt();
        
        // Cap at initial voting power to prevent overflow
        raw_power.min(token_amount as f64)
    }

    /// Create a new governance proposal
    pub fn create_proposal(
        &mut self,
        author: &str,
        proposal_type: ProposalType,
        title: String,
        description: String,
        voting_power: u64,
    ) -> Result<Proposal, &'static str> {
        // Check proposal threshold
        if voting_power < self.settings.proposal_threshold {
            return Err("Insufficient voting power to create proposal");
        }

        self.proposal_counter += 1;
        let proposal_id = format!("AIP-{:04}", self.proposal_counter);

        let current_time = 0; // Would be actual chain time

        let proposal = Proposal {
            id: proposal_id.clone(),
            proposal_type,
            title,
            description,
            status: ProposalStatus::Pending,
            author: author.to_string(),
            created_at: current_time,
            voting_start: current_time,
            voting_end: current_time + self.settings.voting_period,
            execution_delay: self.settings.execution_delay,
            execution_time: None,
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            total_voters: 0,
            quorum: self.settings.quorum_threshold,
            vote_counts: HashMap::new(),
            voters: Vec::new(),
            execution_data: None,
        };

        self.proposals.insert(proposal_id.clone(), proposal.clone());
        self.total_proposals += 1;

        Ok(proposal)
    }

    /// Cast a vote on a proposal
    pub fn cast_vote(
        &mut self,
        proposal_id: &str,
        voter: &str,
        choice: VoteChoice,
        voting_power: u64,
        reason: Option<String>,
    ) -> Result<(), &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        // Check if proposal is active
        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not accepting votes");
        }

        // Check if voter already voted
        if proposal.voters.iter().any(|v| v.voter == voter) {
            return Err("Already voted on this proposal");
        }

        // Calculate quadratic voting power
        let quadratic_power = self.calculate_quadratic_vote(voting_power);

        let vote = Vote {
            voter: voter.to_string(),
            choice: choice.clone(),
            voting_power,
            quadratic_power,
            timestamp: 0, // Would be actual timestamp
            reason,
        };

        // Update vote counts
        *proposal.vote_counts.entry(choice.clone()).or_insert(0) += 1;

        // Update weighted votes
        match choice {
            VoteChoice::For => proposal.for_votes += voting_power,
            VoteChoice::Against => proposal.against_votes += voting_power,
            VoteChoice::Abstain => proposal.abstain_votes += voting_power,
        }

        proposal.total_voters += 1;
        proposal.voters.push(vote);

        Ok(())
    }

    /// Finalize a proposal after voting ends
    pub fn finalize_proposal(&mut self, proposal_id: &str) -> Result<ProposalStatus, &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not in active state");
        }

        let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;

        // Check quorum
        if total_votes < proposal.quorum {
            proposal.status = ProposalStatus::Failed;
            return Ok(ProposalStatus::Failed);
        }

        // Check if passed (simple majority for now, could add quadratic weighting)
        if proposal.for_votes > proposal.against_votes {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Failed;
        }

        Ok(proposal.status.clone())
    }

    /// Queue a passed proposal for execution
    pub fn queue_proposal(&mut self, proposal_id: &str) -> Result<(), &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal must be passed to queue");
        }

        proposal.status = ProposalStatus::Queued;
        proposal.execution_time = Some(proposal.voting_end + proposal.execution_delay);

        Ok(())
    }

    /// Execute a queued proposal
    pub fn execute_proposal(&mut self, proposal_id: &str) -> Result<(), &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Queued {
            return Err("Proposal must be queued to execute");
        }

        // Check execution time
        if let Some(exec_time) = proposal.execution_time {
            let current_time = 0; // Would be actual time
            if current_time < exec_time {
                return Err("Execution delay not elapsed");
            }
        }

        proposal.status = ProposalStatus::Executed;

        // Execute the proposal action based on type
        match &proposal.proposal_type {
            ProposalType::TreasuryAllocation { recipient, amount, token } => {
                self.execute_treasury_transfer(recipient, *amount, token.clone())?;
            }
            ProposalType::ParameterChange { parameter, new_value } => {
                self.execute_parameter_change(parameter, new_value)?;
            }
            _ => {
                // Other proposal types would have custom execution logic
            }
        }

        Ok(())
    }

    /// Execute treasury transfer
    fn execute_treasury_transfer(&mut self, recipient: &str, amount: u64, token: TokenType) -> Result<(), &'static str> {
        let balance = self.treasury_balance.get(&token).copied().unwrap_or(0);
        
        if balance < amount {
            return Err("Insufficient treasury balance");
        }

        *self.treasury_balance.get_mut(&token).unwrap() -= amount;
        
        // In real implementation, would transfer to recipient
        Ok(())
    }

    /// Execute parameter change
    fn execute_parameter_change(&mut self, parameter: &str, new_value: &str) -> Result<(), &'static str> {
        // Update governance settings based on parameter
        match parameter {
            "voting_period" => {
                if let Ok(v) = new_value.parse::<u64>() {
                    self.settings.voting_period = v;
                }
            }
            "quorum_percentage" => {
                if let Ok(v) = new_value.parse::<f64>() {
                    self.settings.quorum_percentage = v;
                }
            }
            _ => return Err("Unknown parameter"),
        }
        Ok(())
    }

    /// Delegate voting power to another address
    pub fn delegate(&mut self, delegator: &str, delegate: &str, voting_power: u64) -> Result<(), &'static str> {
        if delegator == delegate {
            return Err("Cannot delegate to self");
        }

        let delegation = Delegation {
            delegator: delegator.to_string(),
            delegate: delegate.to_string(),
            voting_power,
            locked_until: 0, // Would be actual lock expiry
        };

        // Update delegate's delegations
        let delegations = self.delegates.entry(delegate.to_string()).or_insert_with(Vec::new);
        delegations.push(delegation);

        // Update delegator's delegate
        self.delegators.insert(delegator.to_string(), delegate.to_string());

        Ok(())
    }

    /// Revoke delegation
    pub fn revoke_delegation(&mut self, delegator: &str) -> Result<(), &'static str> {
        if let Some(delegate) = self.delegators.remove(delegator) {
            if let Some(delegations) = self.delegates.get_mut(&delegate) {
                delegations.retain(|d| d.delegator != delegator);
            }
        }
        Ok(())
    }

    /// Emergency veto (for security council)
    pub fn emergency_veto(&mut self, proposal_id: &str, reason: &str) -> Result<(), &'static str> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if !self.settings.veto_enabled {
            return Err("Veto is not enabled");
        }

        if proposal.status == ProposalStatus::Executed {
            return Err("Cannot veto executed proposal");
        }

        proposal.status = ProposalStatus::Failed;

        let action = EmergencyAction {
            id: format!("EMERGENCY-{:}", proposal_id),
            action_type: "VETO".to_string(),
            target: proposal_id.to_string(),
            executed_by: "SECURITY_COUNCIL".to_string(),
            timestamp: 0,
            reason: reason.to_string(),
        };

        self.emergency_actions.push(action);
        self.veto_power_used += 1;

        Ok(())
    }

    /// Get active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    /// Get voter's vote on a proposal
    pub fn get_voter_vote(&self, proposal_id: &str, voter: &str) -> Option<&Vote> {
        self.proposals
            .get(proposal_id)
            .and_then(|p| p.voters.iter().find(|v| v.voter == voter))
    }

    /// Calculate total delegated voting power for an address
    pub fn get_delegated_power(&self, address: &str) -> u64 {
        self.delegates
            .get(address)
            .map(|d| d.iter().map(|del| del.voting_power).sum())
            .unwrap_or(0)
    }

    /// Get proposal results summary
    pub fn get_proposal_results(&self, proposal_id: &str) -> Option<ProposalResults> {
        self.proposals.get(proposal_id).map(|p| {
            let total = p.for_votes + p.against_votes + p.abstain_votes;
            ProposalResults {
                proposal_id: p.id.clone(),
                title: p.title.clone(),
                status: p.status.clone(),
                for_votes: p.for_votes,
                against_votes: p.against_votes,
                abstain_votes: p.abstain_votes,
                total_votes: total,
                for_percentage: if total > 0 { (p.for_votes as f64 / total as f64) * 100.0 } else { 0.0 },
                against_percentage: if total > 0 { (p.against_votes as f64 / total as f64) * 100.0 } else { 0.0 },
                quorum_reached: total >= p.quorum,
                total_voters: p.total_voters,
            }
        })
    }
}

/// Proposal results for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalResults {
    pub proposal_id: String,
    pub title: String,
    pub status: ProposalStatus,
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    pub total_votes: u64,
    pub for_percentage: f64,
    pub against_percentage: f64,
    pub quorum_reached: bool,
    pub total_voters: u64,
}

// =============================================================================
// SPRINT 11: Advanced Governance Mechanisms & Treasury Management
// =============================================================================

/// Timelock mechanism for delayed execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timelock {
    pub timelock_id: String,
    pub proposal_id: String,
    pub executor: String,
    pub delay_epochs: u64,
    pub ready_at_epoch: u64,
    pub executed: bool,
    pub execution_epoch: Option<u64>,
    pub cancelled: bool,
}

impl Timelock {
    pub fn new(proposal_id: &str, executor: &str, delay_epochs: u64, current_epoch: u64) -> Self {
        Self {
            timelock_id: format!("timelock_{}_{}", proposal_id, current_epoch),
            proposal_id: proposal_id.to_string(),
            executor: executor.to_string(),
            delay_epochs,
            ready_at_epoch: current_epoch + delay_epochs,
            executed: false,
            execution_epoch: None,
            cancelled: false,
        }
    }
    
    pub fn can_execute(&self, current_epoch: u64) -> bool {
        !self.executed && !self.cancelled && current_epoch >= self.ready_at_epoch
    }
    
    pub fn execute(&mut self, current_epoch: u64) -> Result<(), &'static str> {
        if !self.can_execute(current_epoch) {
            return Err("Timelock not ready");
        }
        self.executed = true;
        self.execution_epoch = Some(current_epoch);
        Ok(())
    }
    
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
}

/// Multi-sig wallet for governance treasury
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigWallet {
    pub wallet_id: String,
    pub name: String,
    pub signers: Vec<String>,
    pub threshold: u64,  // Required signatures for execution
    pub pending_transactions: Vec<MultiSigTransaction>,
    pub executed_transactions: Vec<MultiSigTransaction>,
    pub treasury_balance: HashMap<TokenType, u64>,
}

impl MultiSigWallet {
    pub fn new(wallet_id: &str, name: &str, signers: Vec<String>, threshold: u64) -> Self {
        Self {
            wallet_id: wallet_id.to_string(),
            name: name.to_string(),
            signers,
            threshold,
            pending_transactions: Vec::new(),
            executed_transactions: Vec::new(),
            treasury_balance: HashMap::new(),
        }
    }
    
    pub fn submit_transaction(&mut self, submitter: &str, recipient: &str, amount: u64, token: TokenType) -> Result<u64, &'static str> {
        if !self.signers.contains(&submitter.to_string()) {
            return Err("Not a signer");
        }
        
        let tx_id = self.pending_transactions.len() as u64;
        let tx = MultiSigTransaction {
            tx_id,
            submitter: submitter.to_string(),
            recipient: recipient.to_string(),
            amount,
            token,
            signatures: vec![submitter.to_string()],
            executed: false,
            submitted_at: 0,
        };
        
        self.pending_transactions.push(tx);
        Ok(tx_id)
    }
    
    pub fn sign_transaction(&mut self, tx_id: u64, signer: &str) -> Result<(), &'static str> {
        if !self.signers.contains(&signer.to_string()) {
            return Err("Not a signer");
        }
        
        let tx = self.pending_transactions.get_mut(tx_id as usize)
            .ok_or("Transaction not found")?;
        
        if tx.signatures.contains(&signer.to_string()) {
            return Err("Already signed");
        }
        
        tx.signatures.push(signer.to_string());
        Ok(())
    }
    
    pub fn execute_transaction(&mut self, tx_id: u64) -> Result<(), &'static str> {
        let tx = self.pending_transactions.get_mut(tx_id as usize)
            .ok_or("Transaction not found")?;
        
        if tx.signatures.len() < self.threshold as usize {
            return Err("Insufficient signatures");
        }
        
        if tx.executed {
            return Err("Already executed");
        }
        
        let balance = self.treasury_balance.get(&tx.token).copied().unwrap_or(0);
        if balance < tx.amount {
            return Err("Insufficient treasury balance");
        }
        
        *self.treasury_balance.get_mut(&tx.token).unwrap() -= tx.amount;
        tx.executed = true;
        
        let executed_tx = self.pending_transactions.remove(tx_id as usize);
        self.executed_transactions.push(executed_tx);
        
        Ok(())
    }
}

/// Multi-sig transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigTransaction {
    pub tx_id: u64,
    pub submitter: String,
    pub recipient: String,
    pub amount: u64,
    pub token: TokenType,
    pub signatures: Vec<String>,
    pub executed: bool,
    pub submitted_at: u64,
}

/// Proposal category for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProposalCategory {
    ProtocolUpgrade,
    TreasuryManagement,
    ParameterChange,
    CommunityGrant,
    SecurityPatch,
    GovernanceChange,
    Partnership,
    Other,
}

/// Proposal tags for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalTags {
    pub category: ProposalCategory,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub risk_level: RiskLevel,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk assessment levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Voting power snapshot at specific epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingPowerSnapshot {
    pub address: String,
    pub epoch: u64,
    pub own_power: u64,
    pub delegated_power: u64,
    pub total_power: u64,
    pub delegations_count: u64,
}

/// Conviction voting (time-weighted voting power)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvictionVote {
    pub voter: String,
    pub proposal_id: String,
    pub choice: VoteChoice,
    pub locked_tokens: u64,
    pub lock_start_epoch: u64,
    pub lock_end_epoch: u64,
    pub conviction_score: f64,
}

impl ConvictionVote {
    /// Calculate conviction score based on lock duration
    pub fn calculate_conviction(&self, current_epoch: u64) -> f64 {
        let epochs_locked = current_epoch.saturating_sub(self.lock_start_epoch);
        // Conviction formula: tokens * sqrt(epochs_locked)
        self.locked_tokens as f64 * (epochs_locked as f64).sqrt() / 100.0
    }
}

/// Governance treasury with budget allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceTreasury {
    pub total_balance: HashMap<TokenType, u64>,
    pub allocated_budgets: HashMap<String, Budget>,
    pub spending_history: Vec<TreasurySpend>,
    pub budget_cycles: Vec<BudgetCycle>,
}

impl GovernanceTreasury {
    pub fn new() -> Self {
        Self {
            total_balance: HashMap::new(),
            allocated_budgets: HashMap::new(),
            spending_history: Vec::new(),
            budget_cycles: Vec::new(),
        }
    }
    
    pub fn create_budget(&mut self, name: &str, amount: u64, token: TokenType, epochs: u64) -> Result<(), &'static str> {
        let balance = self.total_balance.get(&token).copied().unwrap_or(0);
        if balance < amount {
            return Err("Insufficient treasury balance");
        }
        
        let budget = Budget {
            name: name.to_string(),
            amount,
            token,
            remaining: amount,
            start_epoch: 0,
            end_epoch: 0,
            spent: 0,
        };
        
        self.allocated_budgets.insert(name.to_string(), budget);
        Ok(())
    }
    
    pub fn spend(&mut self, budget_name: &str, recipient: &str, amount: u64, reason: &str) -> Result<(), &'static str> {
        let budget = self.allocated_budgets.get_mut(budget_name)
            .ok_or("Budget not found")?;
        
        if budget.remaining < amount {
            return Err("Insufficient budget remaining");
        }
        
        budget.remaining -= amount;
        budget.spent += amount;
        
        self.spending_history.push(TreasurySpend {
            budget_name: budget_name.to_string(),
            recipient: recipient.to_string(),
            amount,
            token: budget.token.clone(),
            reason: reason.to_string(),
            epoch: 0,
        });
        
        Ok(())
    }
    
    pub fn get_budget_utilization(&self, budget_name: &str) -> Option<BudgetUtilization> {
        self.allocated_budgets.get(budget_name).map(|b| {
            BudgetUtilization {
                name: b.name.clone(),
                total: b.amount,
                spent: b.spent,
                remaining: b.remaining,
                utilization_percent: (b.spent as f64 / b.amount as f64) * 100.0,
            }
        })
    }
}

/// Budget allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub name: String,
    pub amount: u64,
    pub token: TokenType,
    pub remaining: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub spent: u64,
}

/// Budget cycle (quarterly, annually, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCycle {
    pub cycle_id: String,
    pub name: String,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub total_budget: HashMap<TokenType, u64>,
    pub actual_spending: HashMap<TokenType, u64>,
}

/// Treasury spend record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasurySpend {
    pub budget_name: String,
    pub recipient: String,
    pub amount: u64,
    pub token: TokenType,
    pub reason: String,
    pub epoch: u64,
}

/// Budget utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUtilization {
    pub name: String,
    pub total: u64,
    pub spent: u64,
    pub remaining: u64,
    pub utilization_percent: f64,
}

/// Proposal discussion comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalComment {
    pub comment_id: String,
    pub proposal_id: String,
    pub author: String,
    pub content: String,
    pub timestamp: u64,
    pub upvotes: u64,
    pub downvotes: u64,
    pub parent_comment_id: Option<String>,
    pub edited: bool,
}

/// Governance analytics dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAnalytics {
    pub total_proposals: u64,
    pub active_proposals: u64,
    pub passed_proposals: u64,
    pub failed_proposals: u64,
    pub total_voters: u64,
    pub average_participation: f64,
    pub total_delegations: u64,
    pub treasury_balance: HashMap<TokenType, u64>,
    pub top_delegates: Vec<(String, u64)>,
    pub proposal_categories: HashMap<ProposalCategory, u64>,
}

/// Voter participation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterMetrics {
    pub address: String,
    pub proposals_voted: u64,
    pub proposals_created: u64,
    pub delegation_count: u64,
    pub voting_power: u64,
    pub conviction_score: f64,
    pub governance_rank: u64,
}

/// Governance notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceNotification {
    pub user: String,
    pub notify_on_new_proposal: bool,
    pub notify_on_voting_end: bool,
    pub notify_on_execution: bool,
    pub notify_on_delegation: bool,
    pub channels: Vec<String>,  // email, discord, telegram, etc.
}

/// Extended governance contract with advanced features
impl GovernanceContract {
    /// Create timelock for proposal execution
    pub fn create_timelock(&mut self, proposal_id: &str, executor: &str, delay_epochs: u64) -> Result<Timelock, &'static str> {
        let proposal = self.proposals.get(proposal_id)
            .ok_or("Proposal not found")?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal must be passed to create timelock");
        }
        
        let timelock = Timelock::new(proposal_id, executor, delay_epochs, self.proposal_counter);
        Ok(timelock)
    }
    
    /// Execute timelocked proposal
    pub fn execute_timelocked(&mut self, timelock_id: &str, current_epoch: u64) -> Result<(), &'static str> {
        // In production, would look up timelock from storage
        // This is a stub for the interface
        Ok(())
    }
    
    /// Create multi-sig treasury wallet
    pub fn create_multisig_wallet(&mut self, name: &str, signers: Vec<String>, threshold: u64) -> Result<MultiSigWallet, &'static str> {
        if threshold > signers.len() as u64 {
            return Err("Threshold cannot exceed signer count");
        }
        
        let wallet = MultiSigWallet::new(
            &format!("multisig_{}_{}", name, self.total_proposals),
            name,
            signers,
            threshold,
        );
        
        Ok(wallet)
    }
    
    /// Get proposal by category
    pub fn get_proposals_by_category(&self, category: &ProposalCategory) -> Vec<&Proposal> {
        // In production, would have category index
        self.proposals.values().collect()
    }
    
    /// Create voting power snapshot
    pub fn create_snapshot(&self, address: &str, epoch: u64) -> VotingPowerSnapshot {
        let own_power = 0u64; // Would query token balance
        let delegated_power = self.get_delegated_power(address);
        
        VotingPowerSnapshot {
            address: address.to_string(),
            epoch,
            own_power,
            delegated_power,
            total_power: own_power + delegated_power,
            delegations_count: self.delegates.get(address).map(|d| d.len() as u64).unwrap_or(0),
        }
    }
    
    /// Submit conviction vote (time-locked voting)
    pub fn submit_conviction_vote(
        &mut self,
        proposal_id: &str,
        voter: &str,
        choice: VoteChoice,
        locked_tokens: u64,
        lock_epochs: u64,
    ) -> Result<ConvictionVote, &'static str> {
        let proposal = self.proposals.get(proposal_id)
            .ok_or("Proposal not found")?;
        
        let vote = ConvictionVote {
            voter: voter.to_string(),
            proposal_id: proposal_id.to_string(),
            choice,
            locked_tokens,
            lock_start_epoch: self.proposal_counter,
            lock_end_epoch: self.proposal_counter + lock_epochs,
            conviction_score: 0.0,
        };
        
        Ok(vote)
    }
    
    /// Add comment to proposal
    pub fn add_proposal_comment(
        &mut self,
        proposal_id: &str,
        author: &str,
        content: &str,
        parent_comment_id: Option<String>,
    ) -> Result<ProposalComment, &'static str> {
        let comment = ProposalComment {
            comment_id: format!("comment_{}_{}", proposal_id, self.total_proposals),
            proposal_id: proposal_id.to_string(),
            author: author.to_string(),
            content: content.to_string(),
            timestamp: 0,
            upvotes: 0,
            downvotes: 0,
            parent_comment_id,
            edited: false,
        };
        
        Ok(comment)
    }
    
    /// Get governance analytics dashboard
    pub fn get_analytics(&self) -> GovernanceAnalytics {
        let mut active = 0u64;
        let mut passed = 0u64;
        let mut failed = 0u64;
        let mut total_voters = 0u64;
        
        for proposal in self.proposals.values() {
            match proposal.status {
                ProposalStatus::Active => active += 1,
                ProposalStatus::Passed | ProposalStatus::Executed => passed += 1,
                ProposalStatus::Failed => failed += 1,
                _ => {}
            }
            total_voters += proposal.total_voters;
        }
        
        GovernanceAnalytics {
            total_proposals: self.total_proposals,
            active_proposals: active,
            passed_proposals: passed,
            failed_proposals: failed,
            total_voters,
            average_participation: if self.total_proposals > 0 {
                total_voters as f64 / self.total_proposals as f64
            } else {
                0.0
            },
            total_delegations: self.delegators.len() as u64,
            treasury_balance: self.treasury_balance.clone(),
            top_delegates: Vec::new(),
            proposal_categories: HashMap::new(),
        }
    }
    
    /// Get voter metrics
    pub fn get_voter_metrics(&self, address: &str) -> VoterMetrics {
        let mut proposals_voted = 0u64;
        let mut proposals_created = 0u64;
        
        for proposal in self.proposals.values() {
            if proposal.author == address {
                proposals_created += 1;
            }
            if proposal.voters.iter().any(|v| v.voter == address) {
                proposals_voted += 1;
            }
        }
        
        VoterMetrics {
            address: address.to_string(),
            proposals_voted,
            proposals_created,
            delegation_count: self.delegates.get(address).map(|d| d.len() as u64).unwrap_or(0),
            voting_power: self.get_delegated_power(address),
            conviction_score: 0.0,
            governance_rank: 0,
        }
    }
    
    /// Calculate governance health score
    pub fn calculate_governance_health(&self) -> GovernanceHealthScore {
        let total_proposals = self.total_proposals;
        let active_count = self.get_active_proposals().len() as u64;
        let total_voters: u64 = self.proposals.values().map(|p| p.total_voters).sum();
        let avg_participation = if total_proposals > 0 {
            total_voters as f64 / total_proposals as f64
        } else {
            0.0
        };
        
        // Participation score (0-40)
        let participation_score = (avg_participation / 100.0 * 40.0).min(40.0);
        
        // Activity score (0-30)
        let activity_score = (active_count as f64 / 10.0 * 30.0).min(30.0);
        
        // Decentralization score (0-30)
        let unique_voters = self.proposals.values()
            .flat_map(|p| p.voters.iter().map(|v| v.voter.clone()))
            .collect::<std::collections::HashSet<_>>()
            .len();
        let decentralization_score = (unique_voters as f64 / 100.0 * 30.0).min(30.0);
        
        let overall = participation_score + activity_score + decentralization_score;
        
        GovernanceHealthScore {
            overall_score: overall,
            participation_score,
            activity_score,
            decentralization_score,
            status: if overall >= 80.0 { "Excellent" }
                   else if overall >= 60.0 { "Good" }
                   else if overall >= 40.0 { "Fair" }
                   else { "Needs Improvement" }.to_string(),
        }
    }
}

/// Governance health score metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceHealthScore {
    pub overall_score: f64,
    pub participation_score: f64,
    pub activity_score: f64,
    pub decentralization_score: f64,
    pub status: String,
}

/// Governance notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user: String,
    pub email_enabled: bool,
    pub discord_enabled: bool,
    pub telegram_enabled: bool,
    pub min_proposal_value: u64,
    pub categories_subscribed: Vec<ProposalCategory>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_voting() {
        let contract = GovernanceContract::new();
        
        // 100 tokens should give sqrt(100) = 10 voting power
        let power = contract.calculate_quadratic_vote(100);
        assert!((power - 10.0).abs() < 0.1);
        
        // 10000 tokens should give sqrt(10000) = 100 voting power
        let power = contract.calculate_quadratic_vote(10000);
        assert!((power - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_create_proposal() {
        let mut contract = GovernanceContract::new();
        
        let proposal = contract.create_proposal(
            "user1",
            ProposalType::ParameterChange {
                parameter: "voting_period".to_string(),
                new_value: "200".to_string(),
            },
            "Increase Voting Period".to_string(),
            "Increase voting period to improve governance quality".to_string(),
            15_000_000, // Above threshold
        );
        
        assert!(proposal.is_ok());
        let proposal = proposal.unwrap();
        assert_eq!(proposal.status, ProposalStatus::Pending);
        assert!(proposal.id.starts_with("AIP-"));
    }

    #[test]
    fn test_voting() {
        let mut contract = GovernanceContract::new();
        
        // Create proposal
        let proposal = contract.create_proposal(
            "user1",
            ProposalType::TreasuryAllocation {
                recipient: "treasury".to_string(),
                amount: 1000,
                token: TokenType::AETH,
            },
            "Treasury Allocation".to_string(),
            "Allocate funds to development".to_string(),
            15_000_000,
        ).unwrap();
        
        // Activate voting
        let proposal = contract.proposals.get_mut(&proposal.id).unwrap();
        proposal.status = ProposalStatus::Active;
        
        // Cast votes
        assert!(contract.cast_vote(&proposal.id, "voter1", VoteChoice::For, 1000000, None).is_ok());
        assert!(contract.cast_vote(&proposal.id, "voter2", VoteChoice::Against, 500000, None).is_ok());
        
        let results = contract.get_proposal_results(&proposal.id).unwrap();
        assert_eq!(results.for_votes, 1000000);
        assert_eq!(results.against_votes, 500000);
    }

    #[test]
    fn test_delegation() {
        let mut contract = GovernanceContract::new();
        
        assert!(contract.delegate("user1", "user2", 1000).is_ok());
        assert!(contract.revoke_delegation("user1").is_ok());
        
        let delegated = contract.get_delegated_power("user2");
        assert_eq!(delegated, 0); // After revocation
    }
}
