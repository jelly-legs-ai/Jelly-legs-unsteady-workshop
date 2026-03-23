// Aether Chain - Staking Contract
// Handles staking, rewards, and lock periods for ATH token

use crate::error::AetherError;
use crate::types::{Account, StakeInfo, StakeState};

// Minimum stake amount (in smallest units - 10^9 = 1 ATH)
pub const MIN_STAKE_AMOUNT: u64 = 100_000_000_000; // 100 ATH
// Lock period in slots (assuming ~400ms per slot, 7 days ≈ 1,814,400 slots)
pub const LOCK_PERIOD_SLOTS: u64 = 1_814_400;
// APY for staking rewards (expressed as basis points - 1250 = 12.5%)
pub const BASE_APY_BPS: u64 = 1250;
// Reward distribution interval in slots
pub const REWARD_INTERVAL_SLOTS: u64 = 100;

/// Initializes a new staking account for a user
pub fn initialize_stake(account: &mut Account, owner: [u8; 32]) -> Result<(), AetherError> {
    if account.data.is_some() {
        return Err(AetherError::AccountAlreadyInitialized);
    }
    
    let stake_info = StakeInfo {
        owner,
        amount: 0,
        start_slot: 0,
        last_claim_slot: 0,
        state: StakeState::Inactive,
        reward_debt: 0,
    };
    
    account.data = Some(stake_info);
    Ok(())
}

/// Stakes tokens for a user
pub fn stake(
    account: &mut Account,
    amount: u64,
    current_slot: u64,
) -> Result<(), AetherError> {
    if amount < MIN_STAKE_AMOUNT {
        return Err(AetherError::InsufficientStakeAmount);
    }
    
    let stake_info = account.data.as_mut().ok_or(AetherError::AccountNotInitialized)?;
    
    // Update stake amount
    stake_info.amount = stake_info.amount.saturating_add(amount);
    stake_info.start_slot = current_slot;
    stake_info.last_claim_slot = current_slot;
    stake_info.state = StakeState::Active;
    
    Ok(())
}

/// Claims accumulated staking rewards
pub fn claim_rewards(
    account: &mut Account,
    current_slot: u64,
) -> Result<u64, AetherError> {
    let stake_info = account.data.as_mut().ok_or(AetherError::AccountNotInitialized)?;
    
    if stake_info.state != StakeState::Active || stake_info.amount == 0 {
        return Err(AetherError::NoActiveStake);
    }
    
    // Calculate rewards based on time elapsed and APY
    let slots_elapsed = current_slot.saturating_sub(stake_info.last_claim_slot);
    let rewards = calculate_rewards(stake_info.amount, slots_elapsed)?;
    
    stake_info.last_claim_slot = current_slot;
    stake_info.reward_debt = stake_info.reward_debt.saturating_add(rewards);
    
    Ok(rewards)
}

/// Calculates pending rewards without claiming
pub fn get_pending_rewards(stake_info: &StakeInfo, current_slot: u64) -> Result<u64, AetherError> {
    if stake_info.state != StakeState::Active || stake_info.amount == 0 {
        return Ok(0);
    }
    
    let slots_elapsed = current_slot.saturating_sub(stake_info.last_claim_slot);
    calculate_rewards(stake_info.amount, slots_elapsed)
}

/// Internal reward calculation
fn calculate_rewards(amount: u64, slots_elapsed: u64) -> Result<u64, AetherError> {
    if slots_elapsed == 0 || amount == 0 {
        return Ok(0);
    }
    
    // APY calculation: rewards = amount * (apy / 10000) * (slots_elapsed / slots_per_year)
    // slots_per_year ≈ 788,400 (assuming 400ms slot time)
    let slots_per_year: u64 = 788_400;
    
    // Use 128-bit integer arithmetic to prevent overflow
    let amount128: u128 = amount as u128;
    let apy128: u128 = BASE_APY_BPS as u128;
    let elapsed128: u128 = slots_elapsed as u128;
    let year128: u128 = slots_per_year as u128;
    let basis128: u128 = 10000;
    
    let rewards128 = amount128 * apy128 * elapsed128 / (basis128 * year128);
    
    Ok(rewards128 as u64)
}

/// Unlocks stake after lock period (requires claiming rewards first)
pub fn unlock(
    account: &mut Account,
    current_slot: u64,
) -> Result<u64, AetherError> {
    let stake_info = account.data.as_mut().ok_or(AetherError::AccountNotInitialized)?;
    
    if stake_info.state != StakeState::Active {
        return Err(AetherError::NoActiveStake);
    }
    
    // Check lock period
    let slots_elapsed = current_slot.saturating_sub(stake_info.start_slot);
    if slots_elapsed < LOCK_PERIOD_SLOTS {
        return Err(AetherError::StakeLocked);
    }
    
    let unlocked_amount = stake_info.amount;
    
    // Reset stake info but preserve rewards
    stake_info.amount = 0;
    stake_info.state = StakeState::Inactive;
    
    Ok(unlocked_amount)
}

/// Emergency unstake without rewards (incurs penalty)
pub fn emergency_unstake(
    account: &mut Account,
    current_slot: u64,
) -> Result<u64, AetherError> {
    let stake_info = account.data.as_mut().ok_or(AetherError::AccountNotInitialized)?;
    
    if stake_info.state != StakeState::Active {
        return Err(AetherError::NoActiveStake);
    }
    
    // Claim any pending rewards
    let _ = claim_rewards(account, current_slot)?;
    
    // Apply 10% penalty for emergency unstake
    let penalty = stake_info.amount / 10;
    let unlocked_amount = stake_info.amount - penalty;
    
    // Reset stake info
    stake_info.amount = 0;
    stake_info.state = StakeState::Inactive;
    
    Ok(unlocked_amount)
}

/// Gets current stake info
pub fn get_stake_info(account: &Account) -> Result<StakeInfo, AetherError> {
    account.data.as_ref().ok_or(AetherError::AccountNotInitialized).cloned()
}
