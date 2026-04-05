// ATH Token Contract - AeTHer Chain Governance Token
// Enhanced implementation for ATH token with ERC20-style transfers, governance, and staking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ATH Token Configuration
pub const ATH_TOKEN_NAME: &str = "Aether";
pub const ATH_TOKEN_SYMBOL: &str = "ATH";
pub const ATH_TOKEN_DECIMALS: u8 = 18;
pub const ATH_MAX_SUPPLY: u64 = 1_000_000_000_u64; // 1 billion ATH
pub const ATH_INITIAL_MINTED: u64 = 100_000_000_u64; // 100 million initially minted

/// ATH Token State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthToken {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub reserve_pool: u64,
    pub governance_treasury: u64,
    pub balances: HashMap<String, u64>,
    pub allowances: HashMap<String, HashMap<String, u64>>,
    pub voting_powers: HashMap<String, u64>,
}

/// Transfer event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub epoch: u64,
    pub timestamp: u64,
    pub tx_hash: String,
}

/// Governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: u64,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub executed: bool,
    pub cancelled: bool,
}

/// Initialize ATH token
pub fn init_ath_token() -> AthToken {
    let mut token = AthToken {
        total_supply: ATH_INITIAL_MINTED,
        circulating_supply: ATH_INITIAL_MINTED,
        reserve_pool: ATH_MAX_SUPPLY / 5,       // 20% reserved
        governance_treasury: ATH_MAX_SUPPLY / 5, // 20% to treasury
        balances: HashMap::new(),
        allowances: HashMap::new(),
        voting_powers: HashMap::new(),
    };
    
    // Mint initial allocations
    token.balances.insert("reserve".to_string(), token.reserve_pool);
    token.balances.insert("governance".to_string(), token.governance_treasury);
    
    token
}

/// Mint new ATH tokens (governance only)
pub fn mint_ath(token: &mut AthToken, amount: u64, recipient: String) -> Result<u64, String> {
    if token.total_supply + amount > ATH_MAX_SUPPLY {
        return Err("Minting would exceed max supply".to_string());
    }
    
    if token.governance_treasury < amount {
        return Err("Insufficient governance treasury".to_string());
    }
    
    token.total_supply += amount;
    token.circulating_supply += amount;
    token.governance_treasury -= amount;
    
    *token.balances.entry(recipient.clone()).or_insert(0) += amount;
    *token.voting_powers.entry(recipient.clone()).or_insert(0) += amount;
    
    Ok(amount)
}

/// Burn ATH tokens
pub fn burn_ath(token: &mut AthToken, owner: String, amount: u64, reason: String) -> Result<(), String> {
    let balance = token.balances.get(&owner).copied().unwrap_or(0);
    
    if balance < amount {
        return Err("Insufficient balance to burn".to_string());
    }
    
    token.balances.insert(owner.clone(), balance - amount);
    token.circulating_supply -= amount;
    token.voting_powers.insert(owner, token.voting_powers.get(&owner).copied().unwrap_or(0) - amount);
    
    Ok(())
}

/// Transfer ATH tokens
pub fn transfer_ath(token: &mut AthToken, from: String, to: String, amount: u64) -> Result<TransferEvent, String> {
    let from_balance = token.balances.get(&from).copied().unwrap_or(0);
    
    if from_balance < amount {
        return Err("Insufficient balance".to_string());
    }
    
    token.balances.insert(from.clone(), from_balance - amount);
    *token.balances.entry(to.clone()).or_insert(0) += amount;
    
    // Transfer voting power
    let from_power = token.voting_powers.get(&from).copied().unwrap_or(0);
    token.voting_powers.insert(from.clone(), from_power - amount);
    *token.voting_powers.entry(to.clone()).or_insert(0) += amount;
    
    let transfer_event = TransferEvent {
        from,
        to,
        amount,
        epoch: 0,
        timestamp: 0,
        tx_hash: format!("tx_ath_{}_{}", from, to),
    };
    
    Ok(transfer_event)
}

/// Approve spender to transfer tokens on behalf of owner
pub fn approve_ath(token: &mut AthToken, owner: String, spender: String, amount: u64) {
    token.allowances
        .entry(owner.clone())
        .or_insert_with(HashMap::new)
        .insert(spender, amount);
}

/// Transfer from (using allowance)
pub fn transfer_from_ath(token: &mut AthToken, spender: String, from: String, to: String, amount: u64) -> Result<TransferEvent, String> {
    let owner_allowances = token.allowances.get(&from);
    
    match owner_allowances {
        Some(allowances) => {
            let spender_allowance = allowances.get(&spender).copied().unwrap_or(0);
            
            if spender_allowance < amount {
                return Err("Allowance exceeded".to_string());
            }
            
            // Reduce allowance
            token.allowances.get_mut(&from).unwrap().insert(spender, spender_allowance - amount);
            
            // Execute transfer
            transfer_ath(token, from, to, amount)
        }
        None => Err("No allowance set".to_string()),
    }
}

/// Calculate staking rewards based on amount and duration
pub fn calculate_staking_reward(
    staked_amount: u64,
    lock_days: u64,
    network_tier: NetworkTier,
) -> u64 {
    let base_apy = match network_tier {
        NetworkTier::Bronze => 0.05,   // 5% APY
        NetworkTier::Silver => 0.08,   // 8% APY
        NetworkTier::Gold => 0.12,      // 12% APY
        NetworkTier::Platinum => 0.15,  // 15% APY
    };
    
    // Duration bonus: extra APY for longer locks
    let duration_multiplier = (lock_days as f64 / 365.0).min(2.0);
    
    (staked_amount as f64 * base_apy * duration_multiplier) as u64
}

/// Dual-token staking reward calculation (returns both AETH and FLUX)
/// AETH rewards are for staking, FLUX rewards are for participation
pub struct DualTokenReward {
    pub aeth_reward: u64,
    pub flux_reward: u64,
    pub bonus_flux: u64,
}

pub fn calculate_dual_token_reward(
    staked_amount: u64,
    lock_days: u64,
    network_tier: NetworkTier,
    network_participation: f64,
) -> DualTokenReward {
    let base_reward = calculate_staking_reward(staked_amount, lock_days, network_tier);
    
    let aeth_portion = match network_tier {
        NetworkTier::Bronze => 0.70,
        NetworkTier::Silver => 0.72,
        NetworkTier::Gold => 0.75,
        NetworkTier::Platinum => 0.80,
    };
    
    let flux_base = (staked_amount as f64 * 0.03 * (lock_days as f64 / 365.0)) as u64;
    let participation_bonus = (flux_base as f64 * network_participation) as u64;
    
    let tier_bonus = match network_tier {
        NetworkTier::Bronze => 0.0,
        NetworkTier::Silver => 0.05,
        NetworkTier::Gold => 0.10,
        NetworkTier::Platinum => 0.20,
    };
    
    DualTokenReward {
        aeth_reward: (base_reward as f64 * aeth_portion) as u64,
        flux_reward: flux_base + participation_bonus,
        bonus_flux: (flux_base as f64 * tier_bonus) as u64,
    }
}

/// Dual-token staking reward calculation (returns both AETH and FLUX)
/// AETH rewards are for staking, FLUX rewards are for participation
pub struct DualTokenReward {
    pub aeth_reward: u64,
    pub flux_reward: u64,
    pub bonus_flux: u64,  // Extra FLUX for early participation
}

pub fn calculate_dual_token_reward(
    staked_amount: u64,
    lock_days: u64,
    network_tier: NetworkTier,
    network_participation: f64, // 0.0 - 1.0 based on mining/validation activity
) -> DualTokenReward {
    let base_reward = calculate_staking_reward(staked_amount, lock_days, network_tier);
    
    // AETH reward (70% of base, adjusted by tier)
    let aeth_portion = match network_tier {
        NetworkTier::Bronze => 0.70,
        NetworkTier::Silver => 0.72,
        NetworkTier::Gold => 0.75,
        NetworkTier::Platinum => 0.80,
    };
    
    // FLUX reward (30% of base, plus participation bonus)
    let flux_base = (staked_amount as f64 * 0.03 * (lock_days as f64 / 365.0)) as u64;
    let participation_bonus = (flux_base as f64 * network_participation) as u64;
    
    // Bonus FLUX for higher tiers
    let tier_bonus = match network_tier {
        NetworkTier::Bronze => 0.0,
        NetworkTier::Silver => 0.05,
        NetworkTier::Gold => 0.10,
        NetworkTier::Platinum => 0.20,
    };
    
    DualTokenReward {
        aeth_reward: (base_reward as f64 * aeth_portion) as u64,
        flux_reward: flux_base + participation_bonus,
        bonus_flux: (flux_base as f64 * tier_bonus) as u64,
    }
}

/// Network tier for staking calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
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

/// Create governance proposal
pub fn create_proposal(
    token: &mut AthToken,
    proposer: String,
    title: String,
    description: String,
    voting_period_epochs: u64,
) -> Result<GovernanceProposal, String> {
    let voting_power = token.voting_powers.get(&proposer).copied().unwrap_or(0);
    
    if voting_power < 1000 {
        return Err("Insufficient voting power to create proposal (min 1000 ATH)".to_string());
    }
    
    let proposal_id = token.balances.len() as u64; // Use counter as ID
    
    let proposal = GovernanceProposal {
        id: proposal_id,
        proposer,
        title,
        description,
        votes_for: 0,
        votes_against: 0,
        votes_abstain: 0,
        start_epoch: 0,
        end_epoch: voting_period_epochs,
        executed: false,
        cancelled: false,
    };
    
    Ok(proposal)
}

/// Cast vote on governance proposal
pub fn cast_vote(
    proposal: &mut GovernanceProposal,
    voter: String,
    vote: VoteChoice,
    voting_power: u64,
) -> Result<(), String> {
    if proposal.start_epoch == 0 {
        return Err("Proposal not yet active".to_string());
    }
    
    if proposal.end_epoch == 0 {
        return Err("Proposal voting period not set".to_string());
    }
    
    match vote {
        VoteChoice::For => proposal.votes_for += voting_power,
        VoteChoice::Against => proposal.votes_against += voting_power,
        VoteChoice::Abstain => proposal.votes_abstain += voting_power,
    }
    
    Ok(())
}

/// Vote choice enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

/// Get token holder count
pub fn get_ath_holder_count(token: &AthToken) -> u64 {
    token.balances.iter().filter(|(_, balance)| **balance > 0).count() as u64
}

/// Get top token holders
pub fn get_ath_top_holders(token: &AthToken, limit: usize) -> Vec<(String, u64)> {
    let mut holders: Vec<_> = token.balances.iter()
        .filter(|(_, balance)| **balance > 0)
        .collect();
    
    holders.sort_by(|a, b| b.1.cmp(a.1));
    holders.truncate(limit);
    
    holders.into_iter().map(|(addr, bal)| (addr.clone(), *bal)).collect()
}

/// Get token metrics
pub fn get_ath_metrics(token: &AthToken) -> AthMetrics {
    AthMetrics {
        total_supply: token.total_supply,
        circulating_supply: token.circulating_supply,
        reserve_pool: token.reserve_pool,
        governance_treasury: token.governance_treasury,
        holder_count: get_ath_holder_count(token),
        top_10_holders: get_ath_top_holders(token, 10),
        total_voting_power: token.voting_powers.values().sum(),
    }
}

/// Token metrics struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthMetrics {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub reserve_pool: u64,
    pub governance_treasury: u64,
    pub holder_count: u64,
    pub top_10_holders: Vec<(String, u64)>,
    pub total_voting_power: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ath_token_init() {
        let token = init_ath_token();
        assert_eq!(token.total_supply, ATH_INITIAL_MINTED);
        assert_eq!(token.reserve_pool, ATH_MAX_SUPPLY / 5);
    }

    #[test]
    fn test_ath_mint() {
        let mut token = init_ath_token();
        let minted = mint_ath(&mut token, 10_000, "user1".to_string()).unwrap();
        assert_eq!(minted, 10_000);
        assert_eq!(*token.balances.get("user1").unwrap(), 10_000);
        assert_eq!(*token.voting_powers.get("user1").unwrap(), 10_000);
    }

    #[test]
    fn test_ath_burn() {
        let mut token = init_ath_token();
        mint_ath(&mut token, 10_000, "user1".to_string()).unwrap();
        
        burn_ath(&mut token, "user1".to_string(), 5_000, "test burn".to_string()).unwrap();
        assert_eq!(*token.balances.get("user1").unwrap(), 5_000);
    }

    #[test]
    fn test_ath_transfer() {
        let mut token = init_ath_token();
        mint_ath(&mut token, 10_000, "user1".to_string()).unwrap();
        
        let transfer = transfer_ath(&mut token, "user1".to_string(), "user2".to_string(), 3_000).unwrap();
        assert_eq!(transfer.amount, 3_000);
        assert_eq!(*token.balances.get("user2").unwrap(), 3_000);
    }

    #[test]
    fn test_ath_approve_and_transfer_from() {
        let mut token = init_ath_token();
        mint_ath(&mut token, 10_000, "user1".to_string()).unwrap();
        
        approve_ath(&mut token, "user1".to_string(), "spender".to_string(), 5_000);
        
        let transfer = transfer_from_ath(&mut token, "spender".to_string(), "user1".to_string(), "user2".to_string(), 2_000).unwrap();
        assert_eq!(transfer.amount, 2_000);
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

    #[test]
    fn test_governance_proposal() {
        let mut token = init_ath_token();
        mint_ath(&mut token, 5_000, "proposer".to_string()).unwrap();
        
        let proposal = create_proposal(
            &mut token,
            "proposer".to_string(),
            "Test Proposal".to_string(),
            "Testing governance".to_string(),
            100,
        ).unwrap();
        
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.votes_for, 0);
    }

    #[test]
    fn test_cast_vote() {
        let mut proposal = GovernanceProposal {
            id: 1,
            proposer: "proposer".to_string(),
            title: "Test".to_string(),
            description: "Test desc".to_string(),
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            start_epoch: 1,
            end_epoch: 100,
            executed: false,
            cancelled: false,
        };
        
        cast_vote(&mut proposal, "voter".to_string(), VoteChoice::For, 1000).unwrap();
        assert_eq!(proposal.votes_for, 1000);
    }
}ath_token();
        assert_eq!(token.total_supply, ATH_MAX_SUPPLY);
        assert_eq!(token.reserve_pool, ATH_MAX_SUPPLY / 5);
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
