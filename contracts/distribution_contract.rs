// AeTHer Chain - Token Distribution Contract
// Handles vesting schedules, Airdrops, and team allocations

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DistributionContract {
    pub owner: String,
    pub allocations: HashMap<String, Allocation>,
    pub vested_releases: HashMap<String, Vec<VestedRelease>>,
    pub airdrops: HashMap<String, Airdrop>,
    pub total_distributed: u64,
    pub events: Vec<DistributionEvent>,
}

#[derive(Debug, Clone)]
pub struct Allocation {
    pub beneficiary: String,
    pub total_amount: u64,
    pub claimed_amount: u64,
    pub start_time: u64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub vesting_schedule: VestingSchedule,
}

#[derive(Debug, Clone)]
pub enum VestingSchedule {
    Linear,        // Equal portions over time
    Exponential,   // More tokens early
    FrontLoaded,   // Most tokens in first period
    BackLoaded,    // Most tokens in last period
}

#[derive(Debug, Clone)]
pub struct VestedRelease {
    pub release_time: u64,
    pub amount: u64,
    pub claimed: bool,
}

#[derive(Debug, Clone)]
pub struct Airdrop {
    pub id: String,
    pub total_amount: u64,
    pub recipients: Vec<AirdropRecipient>,
    pub start_time: u64,
    pub end_time: u64,
    pub status: AirdropStatus,
}

#[derive(Debug, Clone)]
pub struct AirdropRecipient {
    pub address: String,
    pub amount: u64,
    pub claimed: bool,
}

#[derive(Debug, Clone)]
pub enum AirdropStatus {
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum DistributionEvent {
    AllocationCreated { beneficiary: String, amount: u64 },
    TokensClaimed { beneficiary: String, amount: u64 },
    AirdropCreated { id: String, recipients: u32 },
    AirdropClaimed { id: String, recipient: String, amount: u64 },
    VestingScheduleModified { beneficiary: String },
}

impl DistributionContract {
    pub fn new(owner: String) -> Self {
        Self {
            owner,
            allocations: HashMap::new(),
            vested_releases: HashMap::new(),
            airdrops: HashMap::new(),
            total_distributed: 0,
            events: Vec::new(),
        }
    }

    // Create a new token allocation with vesting
    pub fn create_allocation(
        &mut self,
        beneficiary: String,
        total_amount: u64,
        start_time: u64,
        cliff_duration: u64,
        vesting_duration: u64,
        schedule: VestingSchedule,
    ) -> Result<(), DistributionError> {
        // Validate inputs
        if total_amount == 0 {
            return Err(DistributionError::InvalidAmount);
        }
        if vesting_duration == 0 {
            return Err(DistributionError::InvalidVestingDuration);
        }

        let allocation = Allocation {
            beneficiary: beneficiary.clone(),
            total_amount,
            claimed_amount: 0,
            start_time,
            cliff_duration,
            vesting_duration,
            vesting_schedule: schedule.clone(),
        };

        // Generate vested releases based on schedule
        let releases = self.generate_vested_releases(&allocation);
        self.vested_releases.insert(beneficiary.clone(), releases);

        self.allocations.insert(beneficiary.clone(), allocation);
        
        self.events.push(DistributionEvent::AllocationCreated {
            beneficiary,
            amount: total_amount,
        });

        Ok(())
    }

    // Generate vesting release schedule
    fn generate_vested_releases(&self, allocation: &Allocation) -> Vec<VestedRelease> {
        let mut releases = Vec::new();
        let vesting_start = allocation.start_time + allocation.cliff_duration;
        let total_releases = 12; // Monthly releases
        let release_amount = allocation.total_amount / total_releases as u64;

        for i in 0..total_releases {
            releases.push(VestedRelease {
                release_time: vesting_start + (i as u64 * (allocation.vesting_duration / total_releases as u64)),
                amount: release_amount,
                claimed: false,
            });
        }

        releases
    }

    // Claim available tokens
    pub fn claim_tokens(&mut self, beneficiary: &str) -> Result<u64, DistributionError> {
        let allocation = self.allocations.get_mut(beneficiary)
            .ok_or(DistributionError::NoAllocation)?;

        let releases = self.vested_releases.get_mut(beneficiary)
            .ok_or(DistributionError::NoVestingSchedule)?;

        let current_time = self.get_current_time();
        let mut claimable_amount = 0u64;

        // Check if cliff has passed
        if current_time < allocation.start_time + allocation.cliff_duration {
            return Err(DistributionError::CliffNotReached);
        }

        // Calculate claimable from releases
        for release in releases.iter_mut() {
            if !release.claimed && current_time >= release.release_time {
                claimable_amount += release.amount;
                release.claimed = true;
            }
        }

        if claimable_amount == 0 {
            return Err(DistributionError::NoClaimableTokens);
        }

        allocation.claimed_amount += claimable_amount;
        self.total_distributed += claimable_amount;

        self.events.push(DistributionEvent::TokensClaimed {
            beneficiary: beneficiary.to_string(),
            amount: claimable_amount,
        });

        Ok(claimable_amount)
    }

    // Create an airdrop
    pub fn create_airdrop(
        &mut self,
        id: String,
        total_amount: u64,
        recipients: Vec<(String, u64)>,
        duration: u64,
    ) -> Result<(), DistributionError> {
        if recipients.is_empty() {
            return Err(DistributionError::InvalidRecipients);
        }

        let total_recipient_amount: u64 = recipients.iter().map(|(_, a)| a).sum();
        if total_recipient_amount > total_amount {
            return Err(DistributionError::InsufficientFunds);
        }

        let current_time = self.get_current_time();
        let airdrop_recipients: Vec<AirdropRecipient> = recipients
            .into_iter()
            .map(|(address, amount)| AirdropRecipient {
                address,
                amount,
                claimed: false,
            })
            .collect();

        let airdrop = Airdrop {
            id: id.clone(),
            total_amount,
            recipients: airdrop_recipients,
            start_time: current_time,
            end_time: current_time + duration,
            status: AirdropStatus::Active,
        };

        self.airdrops.insert(id.clone(), airdrop);

        self.events.push(DistributionEvent::AirdropCreated {
            id,
            recipients: recipients.len() as u32,
        });

        Ok(())
    }

    // Claim airdrop tokens
    pub fn claim_airdrop(&mut self, airdrop_id: &str, recipient: &str) -> Result<u64, DistributionError> {
        let airdrop = self.airdrops.get_mut(airdrop_id)
            .ok_or(DistributionError::AirdropNotFound)?;

        match airdrop.status {
            AirdropStatus::Completed => return Err(DistributionError::AirdropCompleted),
            AirdropStatus::Cancelled => return Err(DistributionError::AirdropCancelled),
            AirdropStatus::Active => {},
        }

        let current_time = self.get_current_time();
        if current_time > airdrop.end_time {
            airdrop.status = AirdropStatus::Completed;
            return Err(DistributionError::AirdropExpired);
        }

        let recipient_data = airdrop.recipients.iter_mut()
            .find(|r| r.address == recipient)
            .ok_or(DistributionError::RecipientNotInAirdrop)?;

        if recipient_data.claimed {
            return Err(DistributionError::AlreadyClaimed);
        }

        recipient_data.claimed = true;
        self.total_distributed += recipient_data.amount;

        self.events.push(DistributionEvent::AirdropClaimed {
            id: airdrop_id.to_string(),
            recipient: recipient.to_string(),
            amount: recipient_data.amount,
        });

        Ok(recipient_data.amount)
    }

    // Get claimable amount for an address
    pub fn get_claimable_amount(&self, beneficiary: &str) -> u64 {
        let allocation = match self.allocations.get(beneficiary) {
            Some(a) => a,
            None => return 0,
        };

        let current_time = self.get_current_time();
        if current_time < allocation.start_time + allocation.cliff_duration {
            return 0;
        }

        let releases = match self.vested_releases.get(beneficiary) {
            Some(r) => r,
            None => return 0,
        };

        releases.iter()
            .filter(|r| !r.claimed && current_time >= r.release_time)
            .map(|r| r.amount)
            .sum()
    }

    // Get vesting info for an address
    pub fn get_vesting_info(&self, beneficiary: &str) -> Option<VestingInfo> {
        let allocation = self.allocations.get(beneficiary)?;
        let releases = self.vested_releases.get(beneficiary)?;

        let claimed = releases.iter()
            .filter(|r| r.claimed)
            .map(|r| r.amount)
            .sum();

        let locked = allocation.total_amount - claimed - self.get_claimable_amount(beneficiary);

        Some(VestingInfo {
            total_allocated: allocation.total_amount,
            claimed,
            claimable: self.get_claimable_amount(beneficiary),
            locked,
            vesting_start: allocation.start_time + allocation.cliff_duration,
            vesting_end: allocation.start_time + allocation.cliff_duration + allocation.vesting_duration,
        })
    }

    fn get_current_time(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[derive(Debug, Clone)]
pub struct VestingInfo {
    pub total_allocated: u64,
    pub claimed: u64,
    pub claimable: u64,
    pub locked: u64,
    pub vesting_start: u64,
    pub vesting_end: u64,
}

#[derive(Debug, Clone)]
pub enum DistributionError {
    InvalidAmount,
    InvalidVestingDuration,
    NoAllocation,
    NoVestingSchedule,
    CliffNotReached,
    NoClaimableTokens,
    InvalidRecipients,
    InsufficientFunds,
    AirdropNotFound,
    AirdropCompleted,
    AirdropCancelled,
    AirdropExpired,
    RecipientNotInAirdrop,
    AlreadyClaimed,
}

impl std::fmt::Display for DistributionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributionError::InvalidAmount => write!(f, "Invalid amount"),
            DistributionError::InvalidVestingDuration => write!(f, "Invalid vesting duration"),
            DistributionError::NoAllocation => write!(f, "No allocation found"),
            DistributionError::NoVestingSchedule => write!(f, "No vesting schedule"),
            DistributionError::CliffNotReached => write!(f, "Cliff period not reached"),
            DistributionError::NoClaimableTokens => write!(f, "No claimable tokens"),
            DistributionError::InvalidRecipients => write!(f, "Invalid recipients"),
            DistributionError::InsufficientFunds => write!(f, "Insufficient funds"),
            DistributionError::AirdropNotFound => write!(f, "Airdrop not found"),
            DistributionError::AirdropCompleted => write!(f, "Airdrop already completed"),
            DistributionError::AirdropCancelled => write!(f, "Airdrop cancelled"),
            DistributionError::AirdropExpired => write!(f, "Airdrop expired"),
            DistributionError::RecipientNotInAirdrop => write!(f, "Recipient not in airdrop"),
            DistributionError::AlreadyClaimed => write!(f, "Already claimed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_allocation() {
        let mut contract = DistributionContract::new("owner".to_string());
        
        let result = contract.create_allocation(
            "beneficiary1".to_string(),
            1000000,
            1000,
            100,  // cliff
            1000, // vesting duration
            VestingSchedule::Linear,
        );
        
        assert!(result.is_ok());
        assert_eq!(contract.allocations.len(), 1);
    }

    #[test]
    fn test_claim_after_cliff() {
        let mut contract = DistributionContract::new("owner".to_string());
        
        contract.create_allocation(
            "user1".to_string(),
            12000,
            0,
            5,
            10,
            VestingSchedule::Linear,
        ).unwrap();

        // Try to claim before cliff - should fail
        assert!(matches!(
            contract.claim_tokens("user1"),
            Err(DistributionError::CliffNotReached)
        ));
    }

    #[test]
    fn test_airdrop_creation() {
        let mut contract = DistributionContract::new("owner".to_string());
        
        let recipients = vec![
            ("addr1".to_string(), 1000),
            ("addr2".to_string(), 2000),
            ("addr3".to_string(), 1500),
        ];
        
        let result = contract.create_airdrop(
            "airdrop1".to_string(),
            5000,
            recipients,
            86400, // 24 hours
        );
        
        assert!(result.is_ok());
        assert_eq!(contract.airdrops.len(), 1);
    }

    #[test]
    fn test_multiple_vesting_schedules() {
        let schedules = vec![
            VestingSchedule::Linear,
            VestingSchedule::Exponential,
            VestingSchedule::FrontLoaded,
            VestingSchedule::BackLoaded,
        ];
        
        for schedule in schedules {
            let mut contract = DistributionContract::new("owner".to_string());
            let result = contract.create_allocation(
                "user".to_string(),
                10000,
                0,
                0,
                12,
                schedule,
            );
            assert!(result.is_ok());
        }
    }
}