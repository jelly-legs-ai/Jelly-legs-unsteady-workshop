// ATH Token Contract - AeTHer Chain
// Governance token for validator staking and protocol governance

use serde::{Deserialize, Serialize};

/// AETH Token Configuration
pub const AETH_TOKEN_NAME: &str = "AeTHer";
pub const AETH_TOKEN_SYMBOL: &str = "AETH";
pub const AETH_TOKEN_DECIMALS: u8 = 18;
pub const AETH_MAX_SUPPLY: u64 = 1_000_000_000_u64; // 1 billion AETH
pub const MIN_STAKE_AMOUNT: u64 = 100_u64; // Minimum 100 AETH to stake
pub const VALIDATOR_APY: f64 = 0.15; // 15% APY for validators

/// AETH Token State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AethToken {
    pub total_supply: u64,
    pub staked_amount: u64,
    pub validator_count: u64,
    pub governance_participants: u64,
}

/// Initialize AETH token
pub fn init_aeth_token() -> AethToken {
    AethToken {
        total_supply: AETH_MAX_SUPPLY,
        staked_amount: 0,
        validator_count: 0,
        governance_participants: 0,
    }
}

/// Validator stake information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStake {
    pub address: String,
    pub staked_amount: u64,
    pub delegated_amount: u64,
    pub commission_rate: f64,
    pub uptime_score: f64,
    pub is_active: bool,
    pub activation_epoch: u64,
}

/// Create a new validator stake
pub fn create_validator(
    address: String,
    stake_amount: u64,
    commission_rate: f64,
) -> Result<ValidatorStake, &'static str> {
    if stake_amount < MIN_STAKE_AMOUNT {
        return Err("Stake amount below minimum required");
    }
    if commission_rate > 1.0 || commission_rate < 0.0 {
        return Err("Invalid commission rate");
    }
    
    Ok(ValidatorStake {
        address,
        staked_amount: stake_amount,
        delegated_amount: 0,
        commission_rate,
        uptime_score: 1.0,
        is_active: true,
        activation_epoch: 0,
    })
}

/// Calculate staking rewards (APY based)
pub fn calculate_staking_reward(
    principal: u64,
    apy: f64,
    epochs_staked: u64,
    epochs_per_year: u64,
) -> u64 {
    let periods = epochs_staked as f64 / epochs_per_year as f64;
    let reward = principal as f64 * apy * periods;
    reward as u64
}

/// Delegate stake to a validator
pub fn delegate_stake(
    validator: &mut ValidatorStake,
    _delegator: String,
    amount: u64,
) -> Result<(), &'static str> {
    if amount < 1 {
        return Err("Invalid delegation amount");
    }
    validator.delegated_amount += amount;
    Ok(())
}

/// Slash a validator for misbehavior
pub fn slash_validator(
    validator: &mut ValidatorStake,
    slash_percentage: f64,
) -> u64 {
    let slash_amount = (validator.staked_amount as f64 * slash_percentage) as u64;
    validator.staked_amount -= slash_amount;
    validator.uptime_score -= slash_percentage * 0.1;
    if validator.uptime_score < 0.0 {
        validator.uptime_score = 0.0;
    }
    slash_amount
}

/// Governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub author: String,
    pub vote_deadline: u64,
    pub for_votes: u64,
    pub against_votes: u64,
    pub status: ProposalStatus,
    pub total_voting_power: u64,
}

/// Proposal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    Active,
    Passed,
    Failed,
    Executed,
}

/// Cast a vote on a governance proposal (quadratic voting)
pub fn cast_vote(
    proposal: &mut GovernanceProposal,
    _voter: String,
    voting_power: u64,
    support: bool,
) -> Result<(), &'static str> {
    let votes = calculate_quadratic_votes(voting_power);
    
    if support {
        proposal.for_votes += votes;
    } else {
        proposal.against_votes += votes;
    }
    
    proposal.total_voting_power += votes;
    
    // Check if proposal should pass
    let total = proposal.for_votes + proposal.against_votes;
    if total > 0 {
        let approval_ratio = proposal.for_votes as f64 / total as f64;
        if approval_ratio > 0.6 && proposal.total_voting_power > 1000 {
            proposal.status = ProposalStatus::Passed;
        }
    }
    
    Ok(())
}

/// Calculate quadratic voting power
pub fn calculate_quadratic_votes(stake: u64) -> u64 {
    let sqrt = (stake as f64).sqrt();
    sqrt as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aeth_token_init() {
        let token = init_aeth_token();
        assert_eq!(token.total_supply, AETH_MAX_SUPPLY);
        assert_eq!(token.staked_amount, 0);
    }

    #[test]
    fn test_create_validator_success() {
        let validator = create_validator(
            "aether_validator_1".to_string(),
            500,
            0.1,
        );
        assert!(validator.is_ok());
        let v = validator.unwrap();
        assert_eq!(v.staked_amount, 500);
    }

    #[test]
    fn test_create_validator_below_min() {
        let validator = create_validator(
            "aether_validator_2".to_string(),
            50, // Below 100 minimum
            0.1,
        );
        assert!(validator.is_err());
    }

    #[test]
    fn test_quadratic_voting() {
        let votes = calculate_quadratic_votes(10000);
        assert_eq!(votes, 100); // sqrt(10000) = 100
    }

    #[test]
    fn test_slash_validator() {
        let mut validator = create_validator(
            "aether_validator_3".to_string(),
            1000,
            0.1,
        ).unwrap();
        
        let slashed = slash_validator(&mut validator, 0.05); // 5% slash
        assert_eq!(slashed, 50);
        assert_eq!(validator.staked_amount, 950);
    }
}
