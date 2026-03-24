// Token Economics Contract - AeTHer Chain
// Dual-token economy (AETH + FLUX) with vesting, inflation, and treasury management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    AETH,  // Governance token
    FLUX,  // Utility token
    ATH,   // Alternative governance token
}

/// Vesting schedule type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VestingType {
    Linear,           // Linear release over time
    Cliff,            // All at once after cliff period
    Graded,           // Increasing release rate
    Milestone,        // Based on achievement milestones
    Hybrid,           // Combination of above
}

/// Vesting schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    pub schedule_id: String,
    pub beneficiary: String,
    pub token_type: TokenType,
    pub total_amount: u64,
    pub released_amount: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub cliff_time: Option<u64>,
    pub vesting_type: VestingType,
    pub milestones: Vec<Milestone>,
    pub graded_rates: Vec<GradedRate>,
    pub created_at: u64,
    pub cancelled: bool,
    pub cancellation_reason: Option<String>,
}

/// Milestone for milestone-based vesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub milestone_id: String,
    pub description: String,
    pub release_amount: u64,
    pub release_percent: f64,
    pub achieved: bool,
    pub achieved_at: Option<u64>,
    pub verifier: String,
}

/// Graded vesting rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradedRate {
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub release_rate: f64, // tokens per epoch
    pub cumulative_percent: f64,
}

/// Token supply information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSupply {
    pub token_type: TokenType,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub locked_supply: u64,
    pub staked_supply: u64,
    pub treasury_supply: u64,
    pub burned_supply: u64,
    pub inflation_rate: f64,
    pub emission_rate: u64, // tokens per epoch
    pub max_supply: Option<u64>,
}

/// Token distribution breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDistribution {
    pub token_type: TokenType,
    pub public_sale: u64,
    pub public_sale_percent: f64,
    pub team_allocation: u64,
    pub team_percent: f64,
    pub ecosystem_reserve: u64,
    pub ecosystem_percent: f64,
    pub community_rewards: u64,
    pub community_percent: f64,
    pub treasury: u64,
    pub treasury_percent: f64,
    pub staking_rewards: u64,
    pub staking_percent: f64,
    pub liquidity: u64,
    pub liquidity_percent: f64,
}

/// Treasury allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryAllocation {
    pub allocation_id: String,
    pub category: String,
    pub amount: u64,
    pub token_type: TokenType,
    pub purpose: String,
    pub approved_by: String,
    pub approved_at: u64,
    pub disbursed_amount: u64,
    pub remaining_amount: u64,
    pub status: TreasuryStatus,
    pub recipients: Vec<String>,
    pub expiry_epoch: Option<u64>,
}

/// Treasury status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreasuryStatus {
    Proposed,
    Approved,
    PartiallyDisbursed,
    FullyDisbursed,
    Cancelled,
    Expired,
}

/// Inflation schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflationSchedule {
    pub epoch: u64,
    pub base_inflation_rate: f64,
    pub adjusted_inflation_rate: f64,
    pub tokens_minted: u64,
    pub tokens_burned: u64,
    pub net_inflation: u64,
    pub staking_apy: f64,
    pub mining_rewards: u64,
    pub validator_rewards: u64,
    pub treasury_allocation: u64,
}

/// Token burn record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRecord {
    pub burn_id: String,
    pub token_type: TokenType,
    pub amount: u64,
    pub burned_by: String,
    burn_reason: BurnReason,
    transaction_hash: String,
    burned_at: u64,
    epoch: u64,
}

/// Burn reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BurnReason {
    TransactionFee,
    Slashing,
    ProposalDeposit,
    Voluntary,
    ProtocolBurn,
    SupplyAdjustment,
}

/// Token holder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolder {
    pub address: String,
    pub token_type: TokenType,
    pub balance: u64,
    pub staked_balance: u64,
    pub locked_balance: u64,
    pub voting_power: u64,
    pub delegation_count: u64,
    pub first_held_at: u64,
    pub last_transaction_at: u64,
    pub holder_type: HolderType,
}

/// Holder type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolderType {
    Retail,           // < 1000 tokens
    Small,            // 1000 - 10000 tokens
    Medium,           // 10000 - 100000 tokens
    Large,            // 100000 - 1000000 tokens
    Whale,            // > 1000000 tokens
    Contract,         // Smart contract
    Exchange,         // CEX/DEX
    Treasury,         // Protocol treasury
}

/// Token transfer record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    pub transfer_id: String,
    pub token_type: TokenType,
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub transaction_hash: String,
    pub block_number: u64,
    pub epoch: u64,
    pub timestamp: u64,
    pub transfer_type: TransferType,
    pub memo: Option<String>,
    pub fee_paid: u64,
}

/// Transfer type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferType {
    Standard,
    Staking,
    Unstaking,
    Reward,
    VestingRelease,
    Governance,
    Bridge,
    Burn,
    Mint,
}

/// Price oracle data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceOracle {
    pub token_type: TokenType,
    pub price_usd: f64,
    pub price_eth: f64,
    pub price_btc: f64,
    pub market_cap: u64,
    pub volume_24h: u64,
    pub price_change_24h: f64,
    pub last_updated: u64,
    pub oracle_source: String,
    pub confidence_score: f64,
}

/// Token economics contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEconomics {
    pub token_supplies: HashMap<TokenType, TokenSupply>,
    pub token_distributions: HashMap<TokenType, TokenDistribution>,
    pub vesting_schedules: HashMap<String, VestingSchedule>,
    pub treasury_allocations: HashMap<String, TreasuryAllocation>,
    pub inflation_history: HashMap<u64, InflationSchedule>,
    pub burn_records: HashMap<String, BurnRecord>,
    pub token_holders: HashMap<String, HashMap<TokenType, TokenHolder>>,
    pub transfer_history: HashMap<String, Vec<TransferRecord>>,
    pub price_oracles: HashMap<TokenType, PriceOracle>,
    pub current_epoch: u64,
    pub total_vesting_schedules: u64,
    pub total_treasury_allocations: u64,
    pub total_burned_lifetime: HashMap<TokenType, u64>,
    pub emission_schedule_active: bool,
    pub max_inflation_cap: f64,
    pub treasury_multisig_threshold: u64,
}

impl TokenEconomics {
    /// Create new token economics contract
    pub fn new() -> Self {
        let mut token_supplies = HashMap::new();
        
        // AETH supply - Governance token
        token_supplies.insert(TokenType::AETH, TokenSupply {
            token_type: TokenType::AETH,
            total_supply: 100_000_000,
            circulating_supply: 10_000_000,
            locked_supply: 45_000_000,
            staked_supply: 15_000_000,
            treasury_supply: 20_000_000,
            burned_supply: 0,
            inflation_rate: 0.15,
            emission_rate: 100_000,
            max_supply: Some(100_000_000),
        });
        
        // FLUX supply - Utility token
        token_supplies.insert(TokenType::FLUX, TokenSupply {
            token_type: TokenType::FLUX,
            total_supply: 500_000_000,
            circulating_supply: 50_000_000,
            locked_supply: 200_000_000,
            staked_supply: 50_000_000,
            treasury_supply: 150_000_000,
            burned_supply: 5_000_000,
            inflation_rate: 0.08,
            emission_rate: 500_000,
            max_supply: Some(500_000_000),
        });
        
        // ATH supply - Alternative governance
        token_supplies.insert(TokenType::ATH, TokenSupply {
            token_type: TokenType::ATH,
            total_supply: 50_000_000,
            circulating_supply: 5_000_000,
            locked_supply: 25_000_000,
            staked_supply: 10_000_000,
            treasury_supply: 8_000_000,
            burned_supply: 0,
            inflation_rate: 0.10,
            emission_rate: 50_000,
            max_supply: Some(50_000_000),
        });

        let mut token_distributions = HashMap::new();
        
        token_distributions.insert(TokenType::AETH, TokenDistribution {
            token_type: TokenType::AETH,
            public_sale: 10_000_000,
            public_sale_percent: 10.0,
            team_allocation: 15_000_000,
            team_percent: 15.0,
            ecosystem_reserve: 20_000_000,
            ecosystem_percent: 20.0,
            community_rewards: 25_000_000,
            community_percent: 25.0,
            treasury: 20_000_000,
            treasury_percent: 20.0,
            staking_rewards: 8_000_000,
            staking_percent: 8.0,
            liquidity: 2_000_000,
            liquidity_percent: 2.0,
        });
        
        token_distributions.insert(TokenType::FLUX, TokenDistribution {
            token_type: TokenType::FLUX,
            public_sale: 50_000_000,
            public_sale_percent: 10.0,
            team_allocation: 75_000_000,
            team_percent: 15.0,
            ecosystem_reserve: 100_000_000,
            ecosystem_percent: 20.0,
            community_rewards: 150_000_000,
            community_percent: 30.0,
            treasury: 75_000_000,
            treasury_percent: 15.0,
            staking_rewards: 40_000_000,
            staking_percent: 8.0,
            liquidity: 10_000_000,
            liquidity_percent: 2.0,
        });

        let mut total_burned_lifetime = HashMap::new();
        total_burned_lifetime.insert(TokenType::AETH, 0);
        total_burned_lifetime.insert(TokenType::FLUX, 5_000_000);
        total_burned_lifetime.insert(TokenType::ATH, 0);

        TokenEconomics {
            token_supplies,
            token_distributions,
            vesting_schedules: HashMap::new(),
            treasury_allocations: HashMap::new(),
            inflation_history: HashMap::new(),
            burn_records: HashMap::new(),
            token_holders: HashMap::new(),
            transfer_history: HashMap::new(),
            price_oracles: HashMap::new(),
            current_epoch: 0,
            total_vesting_schedules: 0,
            total_treasury_allocations: 0,
            total_burned_lifetime,
            emission_schedule_active: true,
            max_inflation_cap: 0.20,
            treasury_multisig_threshold: 3,
        }
    }

    /// Create vesting schedule
    pub fn create_vesting_schedule(
        &mut self,
        beneficiary: String,
        token_type: TokenType,
        total_amount: u64,
        start_time: u64,
        end_time: u64,
        vesting_type: VestingType,
        cliff_time: Option<u64>,
    ) -> String {
        let schedule_id = format!("vest_{}_{}", token_type.as_str(), self.total_vesting_schedules);
        
        let schedule = VestingSchedule {
            schedule_id: schedule_id.clone(),
            beneficiary,
            token_type,
            total_amount,
            released_amount: 0,
            start_time,
            end_time,
            cliff_time,
            vesting_type,
            milestones: Vec::new(),
            graded_rates: Vec::new(),
            created_at: self.get_timestamp(),
            cancelled: false,
            cancellation_reason: None,
        };

        self.vesting_schedules.insert(schedule_id.clone(), schedule);
        self.total_vesting_schedules += 1;
        schedule_id
    }

    /// Add milestone to vesting schedule
    pub fn add_milestone(
        &mut self,
        schedule_id: &str,
        description: String,
        release_amount: u64,
        verifier: String,
    ) -> Option<String> {
        if let Some(schedule) = self.vesting_schedules.get_mut(schedule_id) {
            let milestone_id = format!("ms_{}_{}", schedule_id, schedule.milestones.len());
            
            let milestone = Milestone {
                milestone_id: milestone_id.clone(),
                description,
                release_amount,
                release_percent: (release_amount as f64) / (schedule.total_amount as f64),
                achieved: false,
                achieved_at: None,
                verifier,
            };
            
            schedule.milestones.push(milestone);
            Some(milestone_id)
        } else {
            None
        }
    }

    /// Achieve milestone
    pub fn achieve_milestone(&mut self, schedule_id: &str, milestone_id: &str) -> bool {
        if let Some(schedule) = self.vesting_schedules.get_mut(schedule_id) {
            if let Some(milestone) = schedule.milestones.iter_mut().find(|m| &m.milestone_id == milestone_id) {
                if !milestone.achieved {
                    milestone.achieved = true;
                    milestone.achieved_at = Some(self.get_timestamp());
                    schedule.released_amount += milestone.release_amount;
                    return true;
                }
            }
        }
        false
    }

    /// Calculate vested amount
    pub fn calculate_vested_amount(&self, schedule_id: &str) -> u64 {
        if let Some(schedule) = self.vesting_schedules.get(schedule_id) {
            if schedule.cancelled {
                return 0;
            }
            
            let now = self.get_timestamp();
            
            if now < schedule.start_time {
                return 0;
            }
            
            if now >= schedule.end_time {
                return schedule.total_amount;
            }
            
            match schedule.vesting_type {
                VestingType::Linear => {
                    let total_duration = schedule.end_time - schedule.start_time;
                    let elapsed = now - schedule.start_time;
                    ((elapsed as f64) / (total_duration as f64) * (schedule.total_amount as f64)) as u64
                },
                VestingType::Cliff => {
                    if let Some(cliff) = schedule.cliff_time {
                        if now >= cliff {
                            schedule.total_amount
                        } else {
                            0
                        }
                    } else {
                        schedule.total_amount
                    }
                },
                _ => schedule.released_amount,
            }
        } else {
            0
        }
    }

    /// Create treasury allocation
    pub fn create_treasury_allocation(
        &mut self,
        category: String,
        amount: u64,
        token_type: TokenType,
        purpose: String,
        approved_by: String,
        recipients: Vec<String>,
    ) -> String {
        let allocation_id = format!("treasury_{}_{}", category, self.total_treasury_allocations);
        
        let allocation = TreasuryAllocation {
            allocation_id: allocation_id.clone(),
            category,
            amount,
            token_type,
            purpose,
            approved_by,
            approved_at: self.get_timestamp(),
            disbursed_amount: 0,
            remaining_amount: amount,
            status: TreasuryStatus::Approved,
            recipients,
            expiry_epoch: Some(self.current_epoch + 50),
        };

        self.treasury_allocations.insert(allocation_id.clone(), allocation);
        self.total_treasury_allocations += 1;
        allocation_id
    }

    /// Disburse treasury allocation
    pub fn disburse_allocation(&mut self, allocation_id: &str, amount: u64, recipient: &str) -> bool {
        if let Some(allocation) = self.treasury_allocations.get_mut(allocation_id) {
            if allocation.remaining_amount >= amount && allocation.status == TreasuryStatus::Approved {
                allocation.disbursed_amount += amount;
                allocation.remaining_amount -= amount;
                
                if allocation.remaining_amount == 0 {
                    allocation.status = TreasuryStatus::FullyDisbursed;
                } else {
                    allocation.status = TreasuryStatus::PartiallyDisbursed;
                }
                
                return true;
            }
        }
        false
    }

    /// Record token burn
    pub fn record_burn(
        &mut self,
        token_type: TokenType,
        amount: u64,
        burned_by: String,
        burn_reason: BurnReason,
        transaction_hash: String,
    ) -> String {
        let burn_id = format!("burn_{}_{}", token_type.as_str(), self.get_timestamp());
        
        let record = BurnRecord {
            burn_id: burn_id.clone(),
            token_type,
            amount,
            burned_by,
            burn_reason,
            transaction_hash,
            burned_at: self.get_timestamp(),
            epoch: self.current_epoch,
        };

        self.burn_records.insert(burn_id.clone(), record);
        *self.total_burned_lifetime.entry(token_type).or_insert(0) += amount;
        
        // Update supply
        if let Some(supply) = self.token_supplies.get_mut(&token_type) {
            supply.burned_supply += amount;
            supply.total_supply -= amount;
        }
        
        burn_id
    }

    /// Record token transfer
    pub fn record_transfer(
        &mut self,
        token_type: TokenType,
        from: String,
        to: String,
        amount: u64,
        transaction_hash: String,
        transfer_type: TransferType,
    ) -> String {
        let transfer_id = format!("tx_{}_{}", token_type.as_str(), self.get_timestamp());
        
        let record = TransferRecord {
            transfer_id: transfer_id.clone(),
            token_type,
            from_address: from,
            to_address: to,
            amount,
            transaction_hash,
            block_number: self.current_epoch * 100,
            epoch: self.current_epoch,
            timestamp: self.get_timestamp(),
            transfer_type,
            memo: None,
            fee_paid: amount / 1000,
        };

        self.transfer_history
            .entry(record.to_address.clone())
            .or_insert_with(Vec::new)
            .push(record);
        
        transfer_id
    }

    /// Update price oracle
    pub fn update_price_oracle(
        &mut self,
        token_type: TokenType,
        price_usd: f64,
        market_cap: u64,
        volume_24h: u64,
        oracle_source: String,
    ) {
        let oracle = PriceOracle {
            token_type,
            price_usd,
            price_eth: price_usd / 3000.0,
            price_btc: price_usd / 50000.0,
            market_cap,
            volume_24h,
            price_change_24h: 0.0,
            last_updated: self.get_timestamp(),
            oracle_source,
            confidence_score: 0.95,
        };
        
        self.price_oracles.insert(token_type, oracle);
    }

    /// Get token supply info
    pub fn get_token_supply(&self, token_type: TokenType) -> Option<TokenSupply> {
        self.token_supplies.get(&token_type).cloned()
    }

    /// Get token distribution
    pub fn get_token_distribution(&self, token_type: TokenType) -> Option<TokenDistribution> {
        self.token_distributions.get(&token_type).cloned()
    }

    /// Get vesting schedule
    pub fn get_vesting_schedule(&self, schedule_id: &str) -> Option<VestingSchedule> {
        self.vesting_schedules.get(schedule_id).cloned()
    }

    /// Get current price
    pub fn get_price(&self, token_type: TokenType) -> Option<f64> {
        self.price_oracles.get(&token_type).map(|o| o.price_usd)
    }

    /// Get inflation for epoch
    pub fn get_inflation(&self, epoch: u64) -> Option<InflationSchedule> {
        self.inflation_history.get(&epoch).cloned()
    }

    /// Calculate inflation for current epoch
    pub fn calculate_epoch_inflation(&mut self) -> InflationSchedule {
        let mut tokens_minted = 0u64;
        let mut tokens_burned = 0u64;
        
        for (token_type, supply) in &self.token_supplies {
            let minted = (supply.total_supply as f64 * supply.inflation_rate) as u64;
            tokens_minted += minted;
            
            // Record in history
            let inflation = InflationSchedule {
                epoch: self.current_epoch,
                base_inflation_rate: supply.inflation_rate,
                adjusted_inflation_rate: supply.inflation_rate,
                tokens_minted: minted,
                tokens_burned: 0,
                net_inflation: minted,
                staking_apy: 0.15,
                mining_rewards: minted / 2,
                validator_rewards: minted / 4,
                treasury_allocation: minted / 4,
            };
            
            self.inflation_history.insert(self.current_epoch, inflation);
        }
        
        InflationSchedule {
            epoch: self.current_epoch,
            base_inflation_rate: 0.12,
            adjusted_inflation_rate: 0.12,
            tokens_minted,
            tokens_burned: tokens_burned,
            net_inflation: tokens_minted - tokens_burned,
            staking_apy: 0.15,
            mining_rewards: tokens_minted / 2,
            validator_rewards: tokens_minted / 4,
            treasury_allocation: tokens_minted / 4,
        }
    }

    /// Get timestamp (placeholder)
    fn get_timestamp(&self) -> u64 {
        self.current_epoch * 1000
    }
}

impl TokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::AETH => "aeth",
            TokenType::FLUX => "flux",
            TokenType::ATH => "ath",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vesting_schedule_creation() {
        let mut economics = TokenEconomics::new();
        
        let schedule_id = economics.create_vesting_schedule(
            "team_member_1".to_string(),
            TokenType::AETH,
            1_000_000,
            1000,
            10000,
            VestingType::Linear,
            Some(2000),
        );
        
        assert!(!schedule_id.is_empty());
        assert_eq!(economics.total_vesting_schedules, 1);
    }

    #[test]
    fn test_milestone_vesting() {
        let mut economics = TokenEconomics::new();
        
        let schedule_id = economics.create_vesting_schedule(
            "advisor_1".to_string(),
            TokenType::FLUX,
            500_000,
            1000,
            20000,
            VestingType::Milestone,
            None,
        );
        
        let milestone_id = economics.add_milestone(
            &schedule_id,
            "Q1 Deliverable".to_string(),
            125_000,
            "governance".to_string(),
        );
        
        assert!(milestone_id.is_some());
        
        let achieved = economics.achieve_milestone(&schedule_id, &milestone_id.unwrap());
        assert!(achieved);
        
        let vested = economics.calculate_vested_amount(&schedule_id);
        assert_eq!(vested, 125_000);
    }

    #[test]
    fn test_treasury_allocation() {
        let mut economics = TokenEconomics::new();
        
        let allocation_id = economics.create_treasury_allocation(
            "Development".to_string(),
            5_000_000,
            TokenType::AETH,
            "Q2 Development Grants".to_string(),
            "multisig".to_string(),
            vec!["dev1".to_string(), "dev2".to_string()],
        );
        
        assert!(!allocation_id.is_empty());
        
        let disbursed = economics.disburse_allocation(&allocation_id, 2_000_000, "dev1");
        assert!(disbursed);
    }

    #[test]
    fn test_token_burn() {
        let mut economics = TokenEconomics::new();
        
        let initial_supply = economics.get_token_supply(TokenType::FLUX).unwrap().total_supply;
        
        let burn_id = economics.record_burn(
            TokenType::FLUX,
            100_000,
            "user123".to_string(),
            BurnReason::TransactionFee,
            "tx_hash_123".to_string(),
        );
        
        assert!(!burn_id.is_empty());
        
        let new_supply = economics.get_token_supply(TokenType::FLUX).unwrap().total_supply;
        assert_eq!(new_supply, initial_supply - 100_000);
    }
}
