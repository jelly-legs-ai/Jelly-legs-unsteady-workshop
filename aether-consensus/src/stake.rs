//! Staking operations for AeTHer Chain
//! Handles stake delegation, reward calculation, and withdrawal

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in seconds since epoch
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Minimum stake required to become a validator
pub const MIN_STAKE_AMOUNT: u64 = 100_000_000; // 100 AETH (with 6 decimals)

/// Base APY for staking (14%)
pub const BASE_APY: f64 = 0.14;

/// Maximum lock period in seconds (1 year)
pub const MAX_LOCK_PERIOD: u64 = 365 * 24 * 60 * 60;

/// Stake tier multipliers based on lock duration
#[derive(Clone, Copy, Debug)]
pub enum StakeTier {
    Flexible,    // No lock - 1.0x multiplier
    Short,       // 7 days - 1.1x multiplier  
    Medium,       // 30 days - 1.25x multiplier
    Long,         // 90 days - 1.5x multiplier
    VeryLong,     // 180 days - 1.75x multiplier
    Locked,       // 365 days - 2.0x multiplier
}

impl StakeTier {
    pub fn multiplier(&self) -> f64 {
        match self {
            StakeTier::Flexible => 1.0,
            StakeTier::Short => 1.1,
            StakeTier::Medium => 1.25,
            StakeTier::Long => 1.5,
            StakeTier::VeryLong => 1.75,
            StakeTier::Locked => 2.0,
        }
    }
    
    pub fn lock_seconds(&self) -> u64 {
        match self {
            StakeTier::Flexible => 0,
            StakeTier::Short => 7 * 24 * 60 * 60,
            StakeTier::Medium => 30 * 24 * 60 * 60,
            StakeTier::Long => 90 * 24 * 60 * 60,
            StakeTier::VeryLong => 180 * 24 * 60 * 60,
            StakeTier::Locked => 365 * 24 * 60 * 60,
        }
    }
    
    pub fn from_lock_seconds(seconds: u64) -> Self {
        if seconds == 0 {
            StakeTier::Flexible
        } else if seconds < 7 * 24 * 60 * 60 {
            StakeTier::Short
        } else if seconds < 30 * 24 * 60 * 60 {
            StakeTier::Medium
        } else if seconds < 90 * 24 * 60 * 60 {
            StakeTier::Long
        } else if seconds < 180 * 24 * 60 * 60 {
            StakeTier::VeryLong
        } else {
            StakeTier::Locked
        }
    }
}

/// Individual stake position
#[derive(Clone, Debug)]
pub struct StakePosition {
    /// Owner address
    pub owner: String,
    /// Amount staked (in smallest unit)
    pub amount: u64,
    /// Start timestamp
    pub start_time: u64,
    /// Lock duration in seconds
    pub lock_period: u64,
    /// Accumulated rewards (before withdrawal)
    pub accumulated_rewards: u64,
    /// Last claim timestamp
    pub last_claim: u64,
}

impl StakePosition {
    /// Create a new stake position
    pub fn new(owner: String, amount: u64, lock_period: u64) -> Self {
        let now = current_timestamp();
        Self {
            owner,
            amount,
            start_time: now,
            lock_period,
            accumulated_rewards: 0,
            last_claim: now, // Set to start_time so rewards accrue from stake inception
        }
    }
    
    /// Create a new stake position with explicit start time (for testing/genesis)
    pub fn new_with_start_time(owner: String, amount: u64, lock_period: u64, start_time: u64) -> Self {
        Self {
            owner,
            amount,
            start_time,
            lock_period,
            accumulated_rewards: 0,
            last_claim: start_time, // Rewards accrue from the actual start time
        }
    }
    
    /// Calculate current tier based on lock period
    pub fn tier(&self) -> StakeTier {
        StakeTier::from_lock_seconds(self.lock_period)
    }
    
    /// Check if stake is still locked
    pub fn is_locked(&self) -> bool {
        let now = current_timestamp();
        now < self.start_time + self.lock_period
    }
    
    /// Calculate current rewards earned
    pub fn calculate_rewards(&self) -> u64 {
        if self.amount == 0 {
            return 0;
        }
        
        let now = current_timestamp();
        
        // Time elapsed since last claim (in years)
        let seconds_since_claim = now.saturating_sub(self.last_claim);
        let years_elapsed = seconds_since_claim as f64 / (365.0 * 24.0 * 60.0 * 60.0);
        
        // Base reward = amount * APY * time * tier multiplier
        let tier_mult = self.tier().multiplier();
        let base_reward = self.amount as f64 * BASE_APY * years_elapsed * tier_mult;
        
        base_reward as u64
    }
    
    /// Update accumulated rewards
    pub fn accrue_rewards(&mut self) {
        let rewards = self.calculate_rewards();
        self.accumulated_rewards += rewards;
        self.last_claim = current_timestamp();
    }
    
    /// Get total value (stake + unclaimed rewards)
    pub fn total_value(&self) -> u64 {
        self.amount + self.accumulated_rewards
    }
}

/// Stake pool managing all positions
pub struct StakePool {
    /// Total staked amount
    pub total_stake: u64,
    /// Total accumulated rewards
    pub total_rewards: u64,
    /// Stake positions by owner address
    pub positions: HashMap<String, Vec<StakePosition>>,
    /// Unstake requests pending
    pub unstake_requests: HashMap<String, UnstakeRequest>,
}

/// Pending unstake request
#[derive(Clone, Debug)]
pub struct UnstakeRequest {
    /// Owner address
    pub owner: String,
    /// Amount to unstake
    pub amount: u64,
    /// Request timestamp
    pub request_time: u64,
    /// Unlock timestamp (after lock period)
    pub unlock_time: u64,
    /// Position index being unstaked
    pub position_idx: usize,
}

impl StakePool {
    /// Create new stake pool
    pub fn new() -> Self {
        Self {
            total_stake: 0,
            total_rewards: 0,
            positions: HashMap::new(),
            unstake_requests: HashMap::new(),
        }
    }
    
    /// Stake tokens
    pub fn stake(&mut self, owner: String, amount: u64, lock_period: u64) -> Result<usize, &'static str> {
        if amount < MIN_STAKE_AMOUNT {
            return Err("Below minimum stake amount");
        }
        
        if lock_period > MAX_LOCK_PERIOD {
            return Err("Lock period exceeds maximum");
        }
        
        let position = StakePosition::new(owner.clone(), amount, lock_period);
        let positions = self.positions.entry(owner.clone()).or_insert_with(Vec::new);
        let idx = positions.len();
        positions.push(position);
        self.total_stake += amount;
        
        Ok(idx)
    }
    
    /// Claim rewards for an address
    pub fn claim_rewards(&mut self, owner: &str) -> Result<u64, &'static str> {
        let positions = self.positions.get_mut(owner).ok_or("No stake found")?;
        
        let mut total_rewards: u64 = 0;
        for pos in positions.iter_mut() {
            pos.accrue_rewards();
            total_rewards = total_rewards.saturating_add(pos.accumulated_rewards);
            pos.accumulated_rewards = 0;
        }
        
        self.total_rewards = self.total_rewards.saturating_add(total_rewards);
        Ok(total_rewards)
    }
    
    /// Request unstake (after lock period)
    pub fn request_unstake(&mut self, owner: &str, position_idx: usize, amount: u64) -> Result<(), &'static str> {
        let positions = self.positions.get_mut(owner).ok_or("No stake found")?;
        let pos = positions.get_mut(position_idx).ok_or("Invalid position index")?;
        
        if pos.is_locked() {
            return Err("Stake is still locked");
        }
        
        if amount > pos.amount {
            return Err("Amount exceeds staked balance");
        }
        
        let now = current_timestamp();
        
        // Create unstake request with no additional lock (already served original lock)
        let request = UnstakeRequest {
            owner: owner.to_string(),
            amount,
            request_time: now,
            unlock_time: now, // Immediate unlock since lock period was served
            position_idx,
        };
        
        self.unstake_requests.insert(format!("{}_{}", owner, position_idx), request);
        Ok(())
    }
    
    /// Execute unstake and withdraw
    pub fn execute_unstake(&mut self, owner: &str, position_idx: usize) -> Result<u64, &'static str> {
        let key = format!("{}_{}", owner, position_idx);
        let request = self.unstake_requests.remove(&key).ok_or("No unstake request found")?;
        
        let now = current_timestamp();
        
        if now < request.unlock_time {
            return Err("Unstake not yet unlocked");
        }
        
        let positions = self.positions.get_mut(owner).ok_or("No stake found")?;
        if position_idx >= positions.len() {
            return Err("Invalid position index");
        }
        
        let pos = &mut positions[position_idx];
        let amount = request.amount;
        let rewards = pos.accumulated_rewards;
        
        // Remove position or reduce amount
        if pos.amount <= amount {
            positions.remove(position_idx);
        } else {
            pos.amount -= amount;
        }
        
        self.total_stake -= amount;
        
        Ok(amount + rewards)
    }
    
    /// Get stake info for an address
    pub fn get_stake_info(&self, owner: &str) -> Option<StakeInfo> {
        let positions = self.positions.get(owner)?;
        
        let mut total_staked: u64 = 0;
        let mut total_rewards: u64 = 0;
        let mut positions_data: Vec<StakeInfoPosition> = Vec::new();
        
        for (idx, pos) in positions.iter().enumerate() {
            total_staked += pos.amount;
            total_rewards += pos.accumulated_rewards;
            positions_data.push(StakeInfoPosition {
                index: idx,
                amount: pos.amount,
                tier: format!("{:?}", pos.tier()),
                lock_remaining: if pos.is_locked() {
                    Some(pos.start_time + pos.lock_period - current_timestamp())
                } else {
                    None
                },
                accumulated_rewards: pos.accumulated_rewards,
                total_value: pos.total_value(),
            });
        }
        
        Some(StakeInfo {
            total_staked,
            total_rewards,
            positions: positions_data,
        })
    }
}

/// Stake information for an address
#[derive(Clone, Debug)]
pub struct StakeInfo {
    pub total_staked: u64,
    pub total_rewards: u64,
    pub positions: Vec<StakeInfoPosition>,
}

/// Individual position info
#[derive(Clone, Debug)]
pub struct StakeInfoPosition {
    pub index: usize,
    pub amount: u64,
    pub tier: String,
    pub lock_remaining: Option<u64>,
    pub accumulated_rewards: u64,
    pub total_value: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake_position_creation() {
        let pos = StakePosition::new("test_owner".to_string(), 1_000_000, 0);
        assert_eq!(pos.owner, "test_owner");
        assert_eq!(pos.amount, 1_000_000);
        assert_eq!(pos.lock_period, 0);
        assert_eq!(pos.accumulated_rewards, 0);
        // last_claim should equal start_time so rewards accrue from inception
        assert_eq!(pos.last_claim, pos.start_time);
    }

    #[test]
    fn test_stake_position_with_custom_start_time() {
        let custom_start = 1_000_000_000;
        let pos = StakePosition::new_with_start_time(
            "test_owner".to_string(),
            1_000_000,
            0,
            custom_start,
        );
        assert_eq!(pos.start_time, custom_start);
        assert_eq!(pos.last_claim, custom_start);
    }

    #[test]
    fn test_rewards_accrue_from_start_time() {
        // Create a position with a start time 1 year in the past
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let one_year_ago = now - (365 * 24 * 60 * 60);
        
        let mut pos = StakePosition::new_with_start_time(
            "test_owner".to_string(),
            1_000_000_000, // 1000 AETH
            0, // Flexible
            one_year_ago,
        );
        
        // After 1 year with 14% APY and 1.0x multiplier, should earn ~140M rewards
        pos.accrue_rewards();
        
        // Rewards should be approximately 14% of stake (140M)
        // Allow some variance due to timing
        assert!(pos.accumulated_rewards > 100_000_000);
        assert!(pos.accumulated_rewards < 200_000_000);
    }

    #[test]
    fn test_stake_tier_multipliers() {
        assert_eq!(StakeTier::Flexible.multiplier(), 1.0);
        assert_eq!(StakeTier::Short.multiplier(), 1.1);
        assert_eq!(StakeTier::Medium.multiplier(), 1.25);
        assert_eq!(StakeTier::Long.multiplier(), 1.5);
        assert_eq!(StakeTier::VeryLong.multiplier(), 1.75);
        assert_eq!(StakeTier::Locked.multiplier(), 2.0);
    }

    #[test]
    fn test_stake_pool_operations() {
        let mut pool = StakePool::new();
        
        // Stake tokens
        let idx = pool.stake("owner1".to_string(), 1_000_000_000, 0);
        assert!(idx.is_ok());
        assert_eq!(idx.unwrap(), 0);
        assert_eq!(pool.total_stake, 1_000_000_000);
        
        // Get stake info
        let info = pool.get_stake_info("owner1");
        assert!(info.is_some());
        assert_eq!(info.unwrap().total_staked, 1_000_000_000);
    }
}