//! AI Priority Fee Distribution Module
//!
//! Implements the economic model for AI Priority Lanes:
//! - Critical lane: 10x base fee (100% to treasury)
//! - High lane: 5x base fee (50% treasury, 50% validators)
//! - Standard lane: base fee (100% to validators)
//!
//! The treasury funds: network development, audits, airdrops, validator subsidies
//!
//! AI-vs-AI competition drives fees to the team master wallet, creating a
//! sustainable funding model for the network.

use aether_common::types::AIPriorityLane;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::collections::HashMap;
use std::sync::RwLock;

// Custom serde for [u8; 64] arrays (base58 encoded)
mod serde_bytes_64 {
    use super::*;
    
    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&bs58::encode(bytes).into_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let decoded = bs58::decode(&s).into_vec().map_err(serde::de::Error::custom)?;
        let mut arr = [0u8; 64];
        let len = decoded.len().min(64);
        arr[..len].copy_from_slice(&decoded[..len]);
        Ok(arr)
    }
}

/// Default team treasury address (can be overridden via config)
pub const DEFAULT_TEAM_TREASURY: [u8; 32] = [
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
];

/// Fee distribution percentages (in basis points, 10000 = 100%)
pub mod distribution {
    /// Treasury share of Critical lane fees (100%)
    pub const CRITICAL_TREASURY_BPS: u64 = 10000;
    /// Treasury share of High lane fees (50%)
    pub const HIGH_TREASURY_BPS: u64 = 5000;
    /// Treasury share of Standard lane fees (0%)
    pub const STANDARD_TREASURY_BPS: u64 = 0;
    
    /// Validator share of Critical lane fees (0% - all goes to treasury)
    pub const CRITICAL_VALIDATORS_BPS: u64 = 0;
    /// Validator share of High lane fees (50%)
    pub const HIGH_VALIDATORS_BPS: u64 = 5000;
    /// Validator share of Standard lane fees (100%)
    pub const STANDARD_VALIDATORS_BPS: u64 = 10000;
}

/// Fee distribution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeDistributionConfig {
    /// Team treasury address for AI priority fees
    pub treasury_address: [u8; 32],
    /// Percentage of burned fees (basis points)
    pub burn_percentage_bps: u64,
    /// Minimum fee for Critical lane (lamports)
    pub min_critical_fee: u64,
    /// Minimum fee for High lane (lamports)
    pub min_high_fee: u64,
    /// Base fee per compute unit (lamports)
    pub base_fee_per_cu: u64,
    /// Enable fee burning for deflationary mechanism
    pub enable_burn: bool,
}

impl Default for FeeDistributionConfig {
    fn default() -> Self {
        Self {
            treasury_address: DEFAULT_TEAM_TREASURY,
            burn_percentage_bps: 5000, // 50% burn
            min_critical_fee: 1_000_000, // 0.001 AETH minimum
            min_high_fee: 500_000,
            base_fee_per_cu: 1, // 1 lamport per compute unit
            enable_burn: true,
        }
    }
}

/// Fee receipt for a processed transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeReceipt {
    /// Transaction signature
    #[serde(with = "serde_bytes_64")]
    pub tx_signature: [u8; 64],
    /// Priority lane used
    pub lane: AIPriorityLane,
    /// Total fee paid (lamports)
    pub total_fee: u64,
    /// Amount going to treasury
    pub treasury_amount: u64,
    /// Amount going to validators
    pub validator_amount: u64,
    /// Amount burned
    pub burn_amount: u64,
    /// Slot where fee was processed
    pub slot: u64,
    /// Timestamp
    pub timestamp: u64,
}

/// Per-epoch fee statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpochFeeStats {
    /// Epoch number
    pub epoch: u64,
    /// Total Critical lane fees
    pub critical_fees: u64,
    /// Total High lane fees
    pub high_fees: u64,
    /// Total Standard lane fees
    pub standard_fees: u64,
    /// Total treasury fees collected
    pub treasury_fees: u64,
    /// Total validator fees collected
    pub validator_fees: u64,
    /// Total burned
    pub burned_fees: u64,
    /// Transaction count per lane
    pub critical_tx_count: u64,
    pub high_tx_count: u64,
    pub standard_tx_count: u64,
}

impl EpochFeeStats {
    /// Create new stats for an epoch
    pub fn new(epoch: u64) -> Self {
        Self { epoch, ..Default::default() }
    }
    
    /// Add a fee receipt to stats
    pub fn add_receipt(&mut self, receipt: &FeeReceipt) {
        match receipt.lane {
            AIPriorityLane::Critical => {
                self.critical_fees += receipt.total_fee;
                self.critical_tx_count += 1;
            }
            AIPriorityLane::High => {
                self.high_fees += receipt.total_fee;
                self.high_tx_count += 1;
            }
            AIPriorityLane::Standard => {
                self.standard_fees += receipt.total_fee;
                self.standard_tx_count += 1;
            }
        }
        self.treasury_fees += receipt.treasury_amount;
        self.validator_fees += receipt.validator_amount;
        self.burned_fees += receipt.burn_amount;
    }
    
    /// Total fees for the epoch
    pub fn total_fees(&self) -> u64 {
        self.critical_fees + self.high_fees + self.standard_fees
    }
    
    /// Average fee per transaction
    pub fn average_fee(&self) -> u64 {
        let total_tx = self.critical_tx_count + self.high_tx_count + self.standard_tx_count;
        if total_tx == 0 { 0 } else { self.total_fees() / total_tx }
    }
}

/// Validator fee reward tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidatorFeeRewards {
    /// Validator pubkey
    pub validator: [u8; 32],
    /// Total fees earned (lifetime)
    pub lifetime_fees: u64,
    /// Fees earned this epoch
    pub epoch_fees: u64,
    /// Epochs participated
    pub epochs_active: u64,
    /// Last epoch fees were claimed
    pub last_claim_epoch: u64,
}

/// Treasury tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TreasuryState {
    /// Treasury address
    pub address: [u8; 32],
    /// Total fees collected (lifetime)
    pub lifetime_fees: u64,
    /// Fees this epoch
    pub epoch_fees: u64,
    /// Total burned
    pub lifetime_burned: u64,
    /// Epoch fee history (last 100 epochs)
    pub epoch_history: Vec<EpochFeeStats>,
}

/// AI Priority Fee Distributor
/// 
/// Manages fee collection, distribution between treasury and validators,
/// and epoch-based accounting.
pub struct FeeDistributor {
    config: FeeDistributionConfig,
    /// Per-epoch statistics
    epoch_stats: RwLock<HashMap<u64, EpochFeeStats>>,
    /// Current epoch
    current_epoch: RwLock<u64>,
    /// Treasury state
    treasury: RwLock<TreasuryState>,
    /// Validator rewards
    validator_rewards: RwLock<HashMap<[u8; 32], ValidatorFeeRewards>>,
    /// Pending receipts for current slot
    pending_receipts: RwLock<Vec<FeeReceipt>>,
    /// Total fees collected (all time)
    total_collected: RwLock<u64>,
}

impl FeeDistributor {
    /// Create new fee distributor with default config
    pub fn new() -> Self {
        Self::with_config(FeeDistributionConfig::default())
    }
    
    /// Create fee distributor with custom config
    pub fn with_config(config: FeeDistributionConfig) -> Self {
        let treasury_address = config.treasury_address;
        Self {
            config,
            epoch_stats: RwLock::new(HashMap::new()),
            current_epoch: RwLock::new(0),
            treasury: RwLock::new(TreasuryState {
                address: treasury_address,
                ..Default::default()
            }),
            validator_rewards: RwLock::new(HashMap::new()),
            pending_receipts: RwLock::new(Vec::new()),
            total_collected: RwLock::new(0),
        }
    }
    
    /// Set the team treasury address
    pub fn set_treasury_address(&mut self, address: [u8; 32]) {
        let mut treasury = self.treasury.write().unwrap();
        treasury.address = address;
    }
    
    /// Process a transaction fee and calculate distribution
    /// 
    /// Returns a FeeReceipt showing how the fee was split.
    /// This should be called for every transaction before execution.
    pub fn process_fee(
        &self,
        tx_signature: [u8; 64],
        lane: AIPriorityLane,
        compute_units: u64,
        slot: u64,
        timestamp: u64,
    ) -> FeeReceipt {
        // Calculate total fee based on lane and compute units
        let total_fee = self.calculate_total_fee(lane, compute_units);
        
        // Determine distribution
        let (treasury_amount, validator_amount) = self.distribute_fee(total_fee, lane);
        
        // Calculate burn amount (if enabled)
        let burn_amount = if self.config.enable_burn {
            let burnable = treasury_amount + validator_amount;
            burnable * self.config.burn_percentage_bps / 10000
        } else {
            0
        };
        
        // Adjust amounts after burn
        let treasury_after_burn = treasury_amount.saturating_sub(burn_amount / 2);
        let validator_after_burn = validator_amount.saturating_sub(burn_amount / 2);
        
        let receipt = FeeReceipt {
            tx_signature,
            lane,
            total_fee,
            treasury_amount: treasury_after_burn,
            validator_amount: validator_after_burn,
            burn_amount,
            slot,
            timestamp,
        };
        
        // Add to pending receipts
        {
            let mut pending = self.pending_receipts.write().unwrap();
            pending.push(receipt.clone());
        }
        
        receipt
    }
    
    /// Finalize fees for a block - distribute to validators
    /// 
    /// Call this after a block is produced to finalize fee distribution.
    /// Returns the total validator fees for this block.
    pub fn finalize_block(&self, block_producer: &[u8; 32]) -> u64 {
        let receipts: Vec<FeeReceipt> = {
            let mut pending = self.pending_receipts.write().unwrap();
            pending.drain(..).collect()
        };
        
        // Calculate validator's share for this block
        let total_validator_fees: u64 = receipts.iter()
            .map(|r| r.validator_amount)
            .sum();
        
        // Credit validator
        {
            let mut rewards = self.validator_rewards.write().unwrap();
            let entry = rewards.entry(*block_producer).or_insert_with(|| ValidatorFeeRewards {
                validator: *block_producer,
                ..Default::default()
            });
            entry.epoch_fees += total_validator_fees;
            entry.lifetime_fees += total_validator_fees;
        }
        
        // Credit treasury
        let total_treasury_fees: u64 = receipts.iter()
            .map(|r| r.treasury_amount)
            .sum();
        let total_burned: u64 = receipts.iter()
            .map(|r| r.burn_amount)
            .sum();
        
        {
            let mut treasury = self.treasury.write().unwrap();
            treasury.epoch_fees += total_treasury_fees;
            treasury.lifetime_fees += total_treasury_fees;
            treasury.lifetime_burned += total_burned;
        }
        
        // Update epoch stats
        {
            let epoch = *self.current_epoch.read().unwrap();
            let mut stats = self.epoch_stats.write().unwrap();
            let epoch_stats = stats.entry(epoch).or_insert_with(|| EpochFeeStats::new(epoch));
            for receipt in &receipts {
                epoch_stats.add_receipt(receipt);
            }
        }
        
        // Update total collected
        {
            let mut total = self.total_collected.write().unwrap();
            *total += receipts.iter().map(|r| r.total_fee).sum::<u64>();
        }
        
        total_validator_fees
    }
    
    /// Advance to next epoch - finalize epoch statistics
    /// 
    /// Call this at epoch boundary. Returns epoch stats.
    pub fn advance_epoch(&self) -> EpochFeeStats {
        let old_epoch = {
            let mut epoch = self.current_epoch.write().unwrap();
            let old = *epoch;
            *epoch += 1;
            old
        };
        
        // Finalize epoch stats
        let stats = {
            let stats_map = self.epoch_stats.read().unwrap();
            stats_map.get(&old_epoch).cloned().unwrap_or_else(|| EpochFeeStats::new(old_epoch))
        };
        
        // Reset epoch fees for validators
        {
            let mut rewards = self.validator_rewards.write().unwrap();
            for reward in rewards.values_mut() {
                reward.epoch_fees = 0;
                reward.epochs_active += 1;
            }
        }
        
        // Reset treasury epoch fees
        {
            let mut treasury = self.treasury.write().unwrap();
            treasury.epoch_history.push(stats.clone());
            // Keep only last 100 epochs
            if treasury.epoch_history.len() > 100 {
                treasury.epoch_history.remove(0);
            }
            treasury.epoch_fees = 0;
        }
        
        stats
    }
    
    /// Calculate total fee for a transaction
    fn calculate_total_fee(&self, lane: AIPriorityLane, compute_units: u64) -> u64 {
        let base_fee = compute_units * self.config.base_fee_per_cu;
        let (minimum_fee, multiplier) = match lane {
            AIPriorityLane::Critical => (self.config.min_critical_fee, 10u64),
            AIPriorityLane::High => (self.config.min_high_fee, 5u64),
            AIPriorityLane::Standard => (0, 1u64),
        };
        
        let calculated_fee = base_fee * multiplier;
        calculated_fee.max(minimum_fee)
    }
    
    /// Distribute fee between treasury and validators based on lane
    fn distribute_fee(&self, fee: u64, lane: AIPriorityLane) -> (u64, u64) {
        let (treasury_bps, validator_bps) = match lane {
            AIPriorityLane::Critical => (
                distribution::CRITICAL_TREASURY_BPS,
                distribution::CRITICAL_VALIDATORS_BPS,
            ),
            AIPriorityLane::High => (
                distribution::HIGH_TREASURY_BPS,
                distribution::HIGH_VALIDATORS_BPS,
            ),
            AIPriorityLane::Standard => (
                distribution::STANDARD_TREASURY_BPS,
                distribution::STANDARD_VALIDATORS_BPS,
            ),
        };
        
        let treasury_amount = fee * treasury_bps / 10000;
        let validator_amount = fee * validator_bps / 10000;
        
        (treasury_amount, validator_amount)
    }
    
    /// Get current epoch
    pub fn current_epoch(&self) -> u64 {
        *self.current_epoch.read().unwrap()
    }
    
    /// Get total fees collected (all time)
    pub fn total_fees_collected(&self) -> u64 {
        *self.total_collected.read().unwrap()
    }
    
    /// Get treasury state
    pub fn treasury_state(&self) -> TreasuryState {
        self.treasury.read().unwrap().clone()
    }
    
    /// Get validator rewards
    pub fn get_validator_rewards(&self, validator: &[u8; 32]) -> Option<ValidatorFeeRewards> {
        let rewards = self.validator_rewards.read().unwrap();
        rewards.get(validator).cloned()
    }
    
    /// Get epoch statistics
    pub fn get_epoch_stats(&self, epoch: u64) -> Option<EpochFeeStats> {
        let stats = self.epoch_stats.read().unwrap();
        stats.get(&epoch).cloned()
    }
    
    /// Get current epoch statistics
    pub fn current_epoch_stats(&self) -> EpochFeeStats {
        let epoch = self.current_epoch.read().unwrap();
        let stats = self.epoch_stats.read().unwrap();
        stats.get(&epoch).cloned().unwrap_or_else(|| EpochFeeStats::new(*epoch))
    }
    
    /// Claim validator rewards (resets epoch_fees)
    /// Returns the amount claimed
    pub fn claim_validator_rewards(&self, validator: &[u8; 32]) -> u64 {
        let mut rewards = self.validator_rewards.write().unwrap();
        if let Some(entry) = rewards.get_mut(validator) {
            let amount = entry.epoch_fees;
            entry.epoch_fees = 0;
            entry.last_claim_epoch = *self.current_epoch.read().unwrap();
            amount
        } else {
            0
        }
    }
    
    /// Get fee economics summary for display
    pub fn fee_economics_summary(&self) -> FeeEconomicsSummary {
        let treasury = self.treasury.read().unwrap();
        let total = self.total_collected.read().unwrap();
        let stats = self.epoch_stats.read().unwrap();
        
        let recent_epochs: Vec<EpochFeeStats> = stats.values()
            .cloned()
            .collect();
        
        FeeEconomicsSummary {
            total_fees_collected: *total,
            treasury_lifetime: treasury.lifetime_fees,
            treasury_burned: treasury.lifetime_burned,
            current_epoch: *self.current_epoch.read().unwrap(),
            epoch_count: stats.len() as u64,
            recent_epoch_stats: recent_epochs.into_iter().take(10).collect(),
        }
    }
}

impl Default for FeeDistributor {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of fee economics for RPC queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEconomicsSummary {
    pub total_fees_collected: u64,
    pub treasury_lifetime: u64,
    pub treasury_burned: u64,
    pub current_epoch: u64,
    pub epoch_count: u64,
    pub recent_epoch_stats: Vec<EpochFeeStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fee_calculation_critical_lane() {
        let distributor = FeeDistributor::new();
        let receipt = distributor.process_fee(
            [1u8; 64],
            AIPriorityLane::Critical,
            200_000, // compute units
            1,
            1000,
        );
        
        // Critical lane should have high minimum
        assert!(receipt.total_fee >= 1_000_000);
        
        // 100% goes to treasury for Critical lane
        assert_eq!(receipt.validator_amount, 0);
        assert!(receipt.treasury_amount > 0);
    }
    
    #[test]
    fn test_fee_calculation_high_lane() {
        let distributor = FeeDistributor::new();
        let receipt = distributor.process_fee(
            [1u8; 64],
            AIPriorityLane::High,
            200_000,
            1,
            1000,
        );
        
        // High lane has 5x multiplier
        assert!(receipt.total_fee >= 500_000);
        
        // 50% treasury, 50% validators
        assert!(receipt.treasury_amount > 0);
        assert!(receipt.validator_amount > 0);
    }
    
    #[test]
    fn test_fee_calculation_standard_lane() {
        let distributor = FeeDistributor::new();
        let receipt = distributor.process_fee(
            [1u8; 64],
            AIPriorityLane::Standard,
            200_000,
            1,
            1000,
        );
        
        // Standard lane: 100% to validators, 0% to treasury
        assert!(receipt.validator_amount > 0);
        assert_eq!(receipt.treasury_amount, 0);
    }
    
    #[test]
    fn test_block_finalization() {
        let distributor = FeeDistributor::new();
        let validator = [2u8; 32];
        
        // Process a few fees
        distributor.process_fee([1u8; 64], AIPriorityLane::Standard, 200_000, 1, 1000);
        distributor.process_fee([2u8; 64], AIPriorityLane::High, 200_000, 1, 1000);
        distributor.process_fee([3u8; 64], AIPriorityLane::Critical, 200_000, 1, 1000);
        
        // Finalize block
        let validator_fees = distributor.finalize_block(&validator);
        
        // Validator should have received fees from Standard + High lanes
        assert!(validator_fees > 0);
        
        // Check validator rewards
        let rewards = distributor.get_validator_rewards(&validator).unwrap();
        assert_eq!(rewards.epoch_fees, validator_fees);
    }
    
    #[test]
    fn test_epoch_advancement() {
        let distributor = FeeDistributor::new();
        let validator = [2u8; 32];
        
        // Process and finalize in epoch 0
        distributor.process_fee([1u8; 64], AIPriorityLane::Critical, 200_000, 1, 1000);
        distributor.finalize_block(&validator);
        
        // Advance epoch
        let stats = distributor.advance_epoch();
        
        assert_eq!(stats.epoch, 0);
        assert!(stats.critical_fees > 0);
        assert_eq!(distributor.current_epoch(), 1);
    }
    
    #[test]
    fn test_validator_claim_rewards() {
        let distributor = FeeDistributor::new();
        let validator = [2u8; 32];
        
        distributor.process_fee([1u8; 64], AIPriorityLane::Standard, 200_000, 1, 1000);
        distributor.finalize_block(&validator);
        
        let rewards = distributor.get_validator_rewards(&validator).unwrap();
        let initial_fees = rewards.epoch_fees;
        
        // Claim rewards
        let claimed = distributor.claim_validator_rewards(&validator);
        assert_eq!(claimed, initial_fees);
        
        // Epoch fees should be reset after claim
        let rewards_after = distributor.get_validator_rewards(&validator).unwrap();
        assert_eq!(rewards_after.epoch_fees, 0);
    }
    
    #[test]
    fn test_treasury_accumulation() {
        let distributor = FeeDistributor::new();
        let validator = [2u8; 32];
        
        // Critical lane: 100% to treasury
        distributor.process_fee([1u8; 64], AIPriorityLane::Critical, 200_000, 1, 1000);
        distributor.finalize_block(&validator);
        
        let treasury = distributor.treasury_state();
        assert!(treasury.epoch_fees > 0);
    }
    
    #[test]
    fn test_fee_distribution_percentages() {
        let distributor = FeeDistributor::new();
        
        // Test Critical: 100% treasury, 0% validators
        let (t, v) = distributor.distribute_fee(1_000_000, AIPriorityLane::Critical);
        assert_eq!(t, 1_000_000);
        assert_eq!(v, 0);
        
        // Test High: 50% treasury, 50% validators
        let (t, v) = distributor.distribute_fee(1_000_000, AIPriorityLane::High);
        assert_eq!(t, 500_000);
        assert_eq!(v, 500_000);
        
        // Test Standard: 0% treasury, 100% validators
        let (t, v) = distributor.distribute_fee(1_000_000, AIPriorityLane::Standard);
        assert_eq!(t, 0);
        assert_eq!(v, 1_000_000);
    }
    
    #[test]
    fn test_multiple_validators() {
        let distributor = FeeDistributor::new();
        let v1 = [1u8; 32];
        let v2 = [2u8; 32];
        
        // V1 produces a block
        distributor.process_fee([1u8; 64], AIPriorityLane::High, 200_000, 1, 1000);
        let _fees1 = distributor.finalize_block(&v1);
        
        // V2 produces a block
        distributor.process_fee([2u8; 64], AIPriorityLane::Standard, 200_000, 2, 1000);
        let _fees2 = distributor.finalize_block(&v2);
        
        // Both should have rewards
        let r1 = distributor.get_validator_rewards(&v1).unwrap();
        let r2 = distributor.get_validator_rewards(&v2).unwrap();
        
        assert!(r1.lifetime_fees > 0);
        assert!(r2.lifetime_fees > 0);
    }
}