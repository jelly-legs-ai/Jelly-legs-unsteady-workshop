// Staking Contract - AeTHer Chain
// implements staking, delegation, and reward distribution for AETH token

use serde::{Deserialize, Serialize};

/// Stake entry for a user/validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeEntry {
    pub address: String,
    pub amount: u64,
    pub start_epoch: u64,
    pub last_claim_epoch: u64,
    pub is_validator: bool,
    pub delegated_from: Vec<String>, // addresses that delegated to this stake
}

/// Unstaking request - tokens locked until withdrawal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakingRequest {
    pub address: String,
    pub amount: u64,
    pub request_epoch: u64,
    pub unlock_epoch: u64, // unlock after 7 days (~7 epochs if 1 epoch = 1 day)
    pub claimed: bool,
}

/// Staking pool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub total_staked: u64,
    pub total_delegated: u64,
    pub active_validators: u64,
    pub current_epoch: u64,
    pub epoch_reward: u64,
}

/// Initialize staking pool
pub fn init_staking_pool(epoch_reward: u64) -> StakingPool {
    StakingPool {
        total_staked: 0,
        total_delegated: 0,
        active_validators: 0,
        current_epoch: 0,
        epoch_reward,
    }
}

/// Constants
pub const EPOCHS_PER_DAY: u64 = 1;
pub const UNSTAKE_LOCK_EPOCHS: u64 = 7; // 7 epochs = 7 days
pub const MIN_DELEGATION: u64 = 10_u64; // Minimum 10 AETH to delegate
pub const MAX_VALIDATORS: u64 = 100;

/// Stake AETH tokens
pub fn stake(
    pool: &mut StakingPool,
    entry: &mut StakeEntry,
    amount: u64,
    current_epoch: u64,
) -> Result<(), &'static str> {
    if amount < 100 {
        return Err("Minimum stake is 100 AETH");
    }
    
    entry.address = entry.address.clone();
    entry.amount += amount;
    entry.start_epoch = current_epoch;
    pool.total_staked += amount;
    
    Ok(())
}

/// Request unstaking - tokens locked for 7 epochs
pub fn request_unstake(
    pool: &mut StakingPool,
    entry: &mut StakeEntry,
    unstake_requests: &mut Vec<UnstakingRequest>,
    amount: u64,
    current_epoch: u64,
) -> Result<UnstakingRequest, &'static str> {
    if amount > entry.amount {
        return Err("Cannot unstake more than staked amount");
    }
    if entry.is_validator && pool.active_validators <= 1 {
        return Err("Cannot unstake - would leave no active validators");
    }
    
    entry.amount -= amount;
    pool.total_staked -= amount;
    
    let request = UnstakingRequest {
        address: entry.address.clone(),
        amount,
        request_epoch: current_epoch,
        unlock_epoch: current_epoch + UNSTAKE_LOCK_EPOCHS,
        claimed: false,
    };
    
    unstake_requests.push(request.clone());
    
    if entry.is_validator && entry.amount < 100 {
        entry.is_validator = false;
        pool.active_validators -= 1;
    }
    
    Ok(request)
}

/// Claim unstaked tokens after lock period
pub fn claim_unstake(
    unstake_requests: &mut [UnstakingRequest],
    address: &str,
    current_epoch: u64,
) -> Result<u64, &'static str> {
    let mut total_claimable = 0u64;
    
    for request in unstake_requests.iter_mut() {
        if request.address == address && !request.claimed && request.unlock_epoch <= current_epoch {
            total_claimable += request.amount;
            request.claimed = true;
        }
    }
    
    if total_claimable == 0 {
        return Err("No claimable unstakes available");
    }
    
    Ok(total_claimable)
}

/// Delegate to a validator
pub fn delegate(
    pool: &mut StakingPool,
    validator_entry: &mut StakeEntry,
    delegator_entry: &mut StakeEntry,
    amount: u64,
    current_epoch: u64,
) -> Result<(), &'static str> {
    if !validator_entry.is_validator {
        return Err("Can only delegate to active validators");
    }
    if amount < MIN_DELEGATION {
        return Err("Minimum delegation is 10 AETH");
    }
    if delegator_entry.amount < amount {
        return Err("Insufficient balance to delegate");
    }
    
    delegator_entry.amount -= amount;
    delegator_entry.delegated_from.push(delegator_entry.address.clone());
    
    validator_entry.delegated_amount += amount;
    pool.total_delegated += amount;
    
    Ok(())
}

/// Calculate pending staking rewards
pub fn calculate_pending_rewards(
    entry: &StakeEntry,
    pool: &StakingPool,
    current_epoch: u64,
) -> u64 {
    if entry.amount == 0 {
        return 0;
    }
    
    let epochs_since_start = current_epoch.saturating_sub(entry.last_claim_epoch);
    if epochs_since_start == 0 {
        return 0;
    }
    
    let stake_ratio = entry.amount as f64 / pool.total_staked as f64;
    let delegator_ratio = if entry.is_validator {
        // validators get more
        1.5
    } else {
        1.0
    };
    
    let base_reward = pool.epoch_reward as f64 * stake_ratio * delegator_ratio;
    let epochs_reward = base_reward * epochs_since_start as f64;
    
    epochs_reward as u64
}

/// Claim staking rewards
pub fn claim_rewards(
    entry: &mut StakeEntry,
    pool: &mut StakingPool,
    current_epoch: u64,
) -> u64 {
    let pending = calculate_pending_rewards(entry, pool, current_epoch);
    
    if pending > 0 {
        entry.amount += pending;
        entry.last_claim_epoch = current_epoch;
        pool.total_staked += pending;
    }
    
    pending
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake() {
        let mut pool = init_staking_pool(10000);
        let mut entry = StakeEntry {
            address: "test_addr".to_string(),
            amount: 0,
            start_epoch: 0,
            last_claim_epoch: 0,
            is_validator: false,
            delegated_from: vec![],
        };
        
        let result = stake(&mut pool, &mut entry, 500, 1);
        assert!(result.is_ok());
        assert_eq!(entry.amount, 500);
        assert_eq!(pool.total_staked, 500);
    }

    #[test]
    fn test_unstake_request() {
        let mut pool = init_staking_pool(10000);
        let mut entry = StakeEntry {
            address: "test_addr".to_string(),
            amount: 500,
            start_epoch: 0,
            last_claim_epoch: 0,
            is_validator: false,
            delegated_from: vec![],
        };
        let mut unstake_requests = vec![];
        
        let result = request_unstake(&mut pool, &mut entry, &mut unstake_requests, 200, 5);
        assert!(result.is_ok());
        assert_eq!(entry.amount, 300);
        assert_eq!(pool.total_staked, 300);
        assert_eq!(unstake_requests.len(), 1);
        assert_eq!(unstake_requests[0].unlock_epoch, 12); // 5 + 7
    }

    #[test]
    fn test_claim_unstake() {
        let mut requests = vec![
            UnstakingRequest {
                address: "test_addr".to_string(),
                amount: 100,
                request_epoch: 5,
                unlock_epoch: 10,
                claimed: false,
            },
        ];
        
        // Try to claim before unlock
        let result = claim_unstake(&mut requests, "test_addr", 9);
        assert!(result.is_err());
        
        // Claim after unlock
        let result = claim_unstake(&mut requests, "test_addr", 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
        assert!(requests[0].claimed);
    }

    #[test]
    fn test_delegate() {
        let mut pool = init_staking_pool(10000);
        let mut validator = StakeEntry {
            address: "validator_1".to_string(),
            amount: 1000,
            start_epoch: 0,
            last_claim_epoch: 0,
            is_validator: true,
            delegated_from: vec![],
        };
        let mut delegator = StakeEntry {
            address: "delegator_1".to_string(),
            amount: 500,
            start_epoch: 0,
            last_claim_epoch: 0,
            is_validator: false,
            delegated_from: vec![],
        };
        
        let result = delegate(&mut pool, &mut validator, &mut delegator, 100, 1);
        assert!(result.is_ok());
        assert_eq!(delegator.amount, 400);
        assert_eq!(pool.total_delegated, 100);
    }
}
