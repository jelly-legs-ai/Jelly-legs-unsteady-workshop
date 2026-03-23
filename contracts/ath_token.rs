// ATH Token Contract - AeTHer Chain Governance Token
// Combined ATH/AETH token with validator staking and governance

use serde::{Deserialize, Serialize};

/// ATH Token Configuration
pub const ATH_TOKEN_NAME: &str = "Aether";
pub const ATH_TOKEN_SYMBOL: &str = "ATH";
pub const ATH_TOKEN_DECIMALS: u8 = 18;
pub const ATH_MAX_SUPPLY: u64 = 1_000_000_000_u64; // 1 billion ATH
pub const MIN_STAKE_AMOUNT: u64 = 100_u64; // Minimum 100 ATH to stake

/// Network tier for staking calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

impl NetworkTier {
    pub fn base_apy(&self) -> f64 {
        match self {
            NetworkTier::Bronze => 0.05,   // 5% APY
            NetworkTier::Silver => 0.08,   // 8% APY
            NetworkTier::Gold => 0.12,     // 12% APY
            NetworkTier::Platinum => 0.15, // 15% APY
        }
    }
}

/// ATH Token State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthToken {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub staked_amount: u64,
    pub reserve_pool: u64,
    pub governance_treasury: u64,
    pub validator_count: u64,
    pub governance_participants: u64,
}

/// Initialize ATH token
pub fn init_ath_token() -> AthToken {
    AthToken {
        total_supply: ATH_MAX_SUPPLY,
        circulating_supply: ATH_MAX_SUPPLY - (ATH_MAX_SUPPLY / 5) * 2, // 60% circulating
        staked_amount: 0,
        reserve_pool: ATH_MAX_SUPPLY / 5,       // 20% reserved
        governance_treasury: ATH_MAX_SUPPLY / 5, // 20% to treasury
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
    pub tier: NetworkTier,
    pub is_active: bool,
    pub activation_epoch: u64,
}

/// Create a new validator stake
pub fn create_validator(
    address: String,
    stake_amount: u64,
    commission_rate: f64,
    tier: NetworkTier,
) -> Result<ValidatorStake, &'static str> {
    if stake_amount < MIN_STAKE_AMOUNT {
        return Err("Stake amount below minimum required");
    }
    if commission_rate > 1.0 || commission_rate < 0.0 {
        return Err("Invalid commission rate");
    }
    
    Ok(ValidatorStake {
        address,
        staked_amount,
        delegated_amount: 0,
        commission_rate,
        uptime_score: 1.0,
        tier,
        is_active: true,
        activation_epoch: 0,
    })
}

/// Calculate staking rewards based on amount, duration and tier
pub fn calculate_staking_reward(
    staked_amount: u64,
    lock_days: u64,
    tier: NetworkTier,
) -> u64 {
    let base_apy = tier.base_apy();
    
    // Duration bonus: extra APY for longer locks
    let duration_multiplier = (lock_days as f64 / 365.0).min(2.0);
    
    (staked_amount as f64 * base_apy * duration_multiplier) as u64
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

/// Staking position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    pub owner: String,
    pub amount: u64,
    pub start_epoch: u64,
    pub lock_end_epoch: u64,
    pub tier: NetworkTier,
    pub pending_rewards: u64,
}

/// Create a new staking position
pub fn create_staking_position(
    owner: String,
    amount: u64,
    start_epoch: u64,
    lock_days: u64,
    tier: NetworkTier,
) -> StakingPosition {
    StakingPosition {
        owner,
        amount,
        start_epoch,
        lock_end_epoch: start_epoch + (lock_days * 24), // epochs are hourly
        tier,
        pending_rewards: 0,
    }
}

/// Check if a position can be unlocked
pub fn can_unlock(position: &StakingPosition, current_epoch: u64) -> bool {
    current_epoch >= position.lock_end_epoch
}

/// Calculate pending rewards for a staking position
pub fn calculate_pending_rewards(position: &StakingPosition, current_epoch: u64) -> u64 {
    if current_epoch < position.start_epoch {
        return 0;
    }
    
    let epochs_staked = current_epoch - position.start_epoch;
    calculate_staking_reward(position.amount, epochs_staked / 24, position.tier)
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
    fn test_ath_token_init() {
        let token = init_ath_token();
        assert_eq!(token.total_supply, ATH_MAX_SUPPLY);
        assert_eq!(token.reserve_pool, ATH_MAX_SUPPLY / 5);
        assert_eq!(token.validator_count, 0);
    }

    #[test]
    fn test_create_validator_success() {
        let validator = create_validator(
            "aether_validator_1".to_string(),
            500,
            0.1,
            NetworkTier::Gold,
        );
        assert!(validator.is_ok());
        let v = validator.unwrap();
        assert_eq!(v.staked_amount, 500);
        assert_eq!(v.tier, NetworkTier::Gold);
    }

    #[test]
    fn test_create_validator_below_min() {
        let validator = create_validator(
            "aether_validator_2".to_string(),
            50, // Below 100 minimum
            0.1,
            NetworkTier::Bronze,
        );
        assert!(validator.is_err());
    }

    #[test]
    fn test_bronze_staking_reward() {
        let reward = calculate_staking_reward(1000, 365, NetworkTier::Bronze);
        assert_eq!(reward, 50); // 5% of 1000
    }

    #[test]
    fn test_platinum_staking_reward() {
        let reward = calculate_staking_reward(1000, 365, NetworkTier::Platinum);
        assert_eq!(reward, 150); // 15% of 1000
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
            NetworkTier::Silver,
        ).unwrap();
        
        let slashed = slash_validator(&mut validator, 0.05); // 5% slash
        assert_eq!(slashed, 50);
        assert_eq!(validator.staked_amount, 950);
    }

    #[test]
    fn test_staking_position_creation() {
        let position = create_staking_position(
            "0x1234".to_string(),
            1000,
            100,
            30,
            NetworkTier::Silver,
        );
        assert_eq!(position.amount, 1000);
        assert!(!can_unlock(&position, 101));
    }
}
