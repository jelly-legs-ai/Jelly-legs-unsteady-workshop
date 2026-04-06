//! AI Priority Fees Module
//!
//! Implements the fee structure for AI priority lanes.
//! AI operators pay premium fees for priority execution.
//! Fees are burned to create deflationary pressure and fund network development.

use aether_common::types::AIPriorityLane;
use aether_common::constants::{
    BASE_TRANSACTION_FEE,
    AI_VERIFICATION_FEE,
    CRITICAL_LANE_MULTIPLIER,
    HIGH_LANE_MULTIPLIER,
    TRANSACTION_FEE_BURN_PERCENT,
    PRIORITY_FEE_BURN_PERCENT,
};

/// Fee calculator for AI priority transactions
#[derive(Debug, Clone)]
pub struct FeeCalculator {
    /// Base fee per compute unit
    base_compute_price: u64,
    /// Verification fee for AI signatures
    ai_verification_fee: u64,
}

impl Default for FeeCalculator {
    fn default() -> Self {
        Self {
            base_compute_price: 100, // 100 lamports per compute unit
            ai_verification_fee: AI_VERIFICATION_FEE,
        }
    }
}

impl FeeCalculator {
    /// Create a new fee calculator
    pub fn new(base_compute_price: u64, ai_verification_fee: u64) -> Self {
        Self {
            base_compute_price,
            ai_verification_fee,
        }
    }
    
    /// Calculate compute unit price for a lane
    /// 
    /// Higher priority lanes have exponentially higher prices
    /// to incentivize efficient use of block space.
    pub fn compute_unit_price(&self, lane: AIPriorityLane) -> u64 {
        match lane {
            AIPriorityLane::Critical => self.base_compute_price * CRITICAL_LANE_MULTIPLIER,
            AIPriorityLane::High => self.base_compute_price * HIGH_LANE_MULTIPLIER,
            AIPriorityLane::Standard => self.base_compute_price,
        }
    }
    
    /// Calculate total fee for a transaction
    /// 
    /// fee = base_fee + (compute_units * lane_price) + ai_verification_fee
    pub fn calculate_fee(&self, lane: AIPriorityLane, compute_units: u64) -> u64 {
        let base = BASE_TRANSACTION_FEE;
        let lane_price = self.compute_unit_price(lane);
        let compute_fee = compute_units.saturating_mul(lane_price);
        
        let total = base.saturating_add(compute_fee);
        
        // Add AI verification fee for AI lanes
        match lane {
            AIPriorityLane::Critical | AIPriorityLane::High => {
                total.saturating_add(self.ai_verification_fee)
            }
            AIPriorityLane::Standard => total,
        }
    }
    
    /// Calculate how much of the fee is burned
    /// 
    /// Priority fees are 100% burned, regular fees are 50% burned
    /// For Standard lane, only the base fee exists (50% burned)
    /// For AI lanes, priority fees include compute + verification (100% burned)
    pub fn calculate_burn_amount(&self, lane: AIPriorityLane, total_fee: u64) -> u64 {
        let base_fee = BASE_TRANSACTION_FEE;
        
        // Base fee burn is always 50%
        let base_burn = base_fee * TRANSACTION_FEE_BURN_PERCENT as u64 / 100;
        
        // Priority fee burn depends on lane
        match lane {
            AIPriorityLane::Standard => {
                // Standard lane: only base fee portion is burned at 50%
                // The compute fee for standard is just regular gas, not priority
                base_burn
            }
            AIPriorityLane::Critical | AIPriorityLane::High => {
                // AI lanes: priority fees (compute + verification) are 100% burned
                let priority_fee = total_fee.saturating_sub(base_fee);
                let priority_burn = priority_fee * PRIORITY_FEE_BURN_PERCENT as u64 / 100;
                base_burn.saturating_add(priority_burn)
            }
        }
    }
    
    /// Calculate the network treasury portion
    /// 
    /// Non-burned fees go to treasury for development/airdrops
    pub fn calculate_treasury_amount(&self, total_fee: u64, burn_amount: u64) -> u64 {
        total_fee.saturating_sub(burn_amount)
    }
    
    /// Estimate fee for a transaction based on estimated compute units
    pub fn estimate_fee(&self, lane: AIPriorityLane, estimated_compute_units: u64) -> FeeEstimate {
        let total_fee = self.calculate_fee(lane, estimated_compute_units);
        let burn_amount = self.calculate_burn_amount(lane, total_fee);
        let treasury_amount = self.calculate_treasury_amount(total_fee, burn_amount);
        
        FeeEstimate {
            lane,
            compute_units: estimated_compute_units,
            compute_unit_price: self.compute_unit_price(lane),
            base_fee: BASE_TRANSACTION_FEE,
            priority_fee: total_fee.saturating_sub(BASE_TRANSACTION_FEE),
            ai_verification_fee: if lane != AIPriorityLane::Standard {
                Some(self.ai_verification_fee)
            } else {
                None
            },
            total_fee,
            burn_amount,
            treasury_amount,
        }
    }
}

/// Fee estimate breakdown
#[derive(Debug, Clone)]
pub struct FeeEstimate {
    /// Priority lane
    pub lane: AIPriorityLane,
    /// Estimated compute units
    pub compute_units: u64,
    /// Price per compute unit
    pub compute_unit_price: u64,
    /// Base transaction fee
    pub base_fee: u64,
    /// Priority fee (compute + AI verification)
    pub priority_fee: u64,
    /// AI verification fee (if applicable)
    pub ai_verification_fee: Option<u64>,
    /// Total fee
    pub total_fee: u64,
    /// Amount to burn
    pub burn_amount: u64,
    /// Amount to treasury
    pub treasury_amount: u64,
}

impl std::fmt::Display for FeeEstimate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Fee Estimate ({:?} Lane):", self.lane)?;
        writeln!(f, "  Compute Units: {}", self.compute_units)?;
        writeln!(f, "  Compute Price: {} lamports/unit", self.compute_unit_price)?;
        writeln!(f, "  Base Fee: {} lamports", self.base_fee)?;
        writeln!(f, "  Priority Fee: {} lamports", self.priority_fee)?;
        if let Some(ai_fee) = self.ai_verification_fee {
            writeln!(f, "  AI Verification Fee: {} lamports", ai_fee)?;
        }
        writeln!(f, "  Total: {} lamports ({:.6} AETH)", 
            self.total_fee, 
            self.total_fee as f64 / 1_000_000_000.0)?;
        writeln!(f, "  Burned: {} lamports", self.burn_amount)?;
        writeln!(f, "  To Treasury: {} lamports", self.treasury_amount)
    }
}

/// Calculate compute unit price for a lane (standalone function)
pub fn compute_unit_price(lane: AIPriorityLane) -> u64 {
    FeeCalculator::default().compute_unit_price(lane)
}

/// Calculate total fee for a transaction (standalone function)
pub fn calculate_fee(lane: AIPriorityLane, compute_units: u64) -> u64 {
    FeeCalculator::default().calculate_fee(lane, compute_units)
}

/// Global fee calculator instance
static FEE_CALCULATOR: std::sync::OnceLock<FeeCalculator> = std::sync::OnceLock::new();

/// Get the global fee calculator
pub fn get_fee_calculator() -> &'static FeeCalculator {
    FEE_CALCULATOR.get_or_init(FeeCalculator::default)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compute_unit_price() {
        let calc = FeeCalculator::default();
        
        // Critical: 100 * 10 = 1000
        assert_eq!(calc.compute_unit_price(AIPriorityLane::Critical), 1000);
        
        // High: 100 * 5 = 500
        assert_eq!(calc.compute_unit_price(AIPriorityLane::High), 500);
        
        // Standard: 100
        assert_eq!(calc.compute_unit_price(AIPriorityLane::Standard), 100);
    }
    
    #[test]
    fn test_fee_calculation() {
        let calc = FeeCalculator::default();
        
        // Standard: 5000 base + (200000 * 100) = 5000 + 20000000 = 20005000
        let standard_fee = calc.calculate_fee(AIPriorityLane::Standard, 200_000);
        assert_eq!(standard_fee, 20_005_000);
        
        // High: 5000 base + (200000 * 500) + 1000000 AI = 5000 + 100000000 + 1000000 = 101005000
        let high_fee = calc.calculate_fee(AIPriorityLane::High, 200_000);
        assert_eq!(high_fee, 101_005_000);
        
        // Critical: 5000 base + (200000 * 1000) + 1000000 AI = 5000 + 200000000 + 1000000 = 201005000
        let critical_fee = calc.calculate_fee(AIPriorityLane::Critical, 200_000);
        assert_eq!(critical_fee, 201_005_000);
    }
    
    #[test]
    fn test_burn_calculation() {
        let calc = FeeCalculator::default();
        
        // Standard: 50% of base fee burned
        let standard_fee = calc.calculate_fee(AIPriorityLane::Standard, 200_000);
        let standard_burn = calc.calculate_burn_amount(AIPriorityLane::Standard, standard_fee);
        // Base fee burn: 5000 * 50% = 2500
        // Priority fee burn: 0 * 100% = 0
        assert_eq!(standard_burn, 2500);
        
        // High: 50% of base + 100% of priority burned
        let high_fee = calc.calculate_fee(AIPriorityLane::High, 200_000);
        let high_burn = calc.calculate_burn_amount(AIPriorityLane::High, high_fee);
        // Base fee burn: 5000 * 50% = 2500
        // Priority fee burn: (high_fee - 5000) * 100% = (high_fee - 5000)
        // Total burn = 2500 + (high_fee - 5000) = high_fee - 2500
        assert_eq!(high_burn, high_fee - 2500);
    }
    
    #[test]
    fn test_fee_estimate() {
        let calc = FeeCalculator::default();
        let estimate = calc.estimate_fee(AIPriorityLane::Critical, 500_000);
        
        assert_eq!(estimate.lane, AIPriorityLane::Critical);
        assert_eq!(estimate.compute_units, 500_000);
        assert!(estimate.ai_verification_fee.is_some());
        assert!(estimate.burn_amount > 0);
        assert!(estimate.treasury_amount > 0);
    }
    
    #[test]
    fn test_standalone_functions() {
        assert!(compute_unit_price(AIPriorityLane::Critical) > compute_unit_price(AIPriorityLane::Standard));
        assert!(calculate_fee(AIPriorityLane::High, 100_000) > calculate_fee(AIPriorityLane::Standard, 100_000));
    }
}
