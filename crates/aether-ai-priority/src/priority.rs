//! AI Priority Module
//!
//! Handles transaction priority classification and fee calculation
//! for the 3-tier AI priority system.

use aether_common::types::AIPriorityLane;

/// Minimum fees per lane (in lamports)
const MIN_CRITICAL_FEE: u64 = 1_000_000;   // 0.001 AETH
const MIN_HIGH_FEE: u64 = 500_000;          // 0.0005 AETH
const MIN_STANDARD_FEE: u64 = 5_000;        // Base fee

/// Lane capacity percentages (of block)
const CRITICAL_LANE_PCT: u64 = 40;  // 40% of block
const HIGH_LANE_PCT: u64 = 30;      // 30% of block
const STANDARD_LANE_PCT: u64 = 30;  // 30% of block

/// Classify a transaction into a priority lane based on fee and content
/// 
/// This is a fee-based classification. Content-based classification
/// is handled by the classifier module.
pub fn classify_by_fee(priority_fee: u64) -> AIPriorityLane {
    if priority_fee >= MIN_CRITICAL_FEE {
        AIPriorityLane::Critical
    } else if priority_fee >= MIN_HIGH_FEE {
        AIPriorityLane::High
    } else {
        AIPriorityLane::Standard
    }
}

/// Classify a transaction by analyzing its content
/// 
/// Uses heuristics to determine appropriate priority lane.
/// For accurate classification, use the classifier module.
pub fn classify_transaction(data: &[u8]) -> AIPriorityLane {
    // Quick content analysis
    let data_str = String::from_utf8_lossy(data);
    let lower = data_str.to_lowercase();
    
    // Check for critical keywords
    if lower.contains("emergency") || lower.contains("governance") {
        return AIPriorityLane::Critical;
    }
    
    // Check for AI/MEV keywords
    if lower.contains("ai_agent") || lower.contains("mev") 
        || lower.contains("arbitrage") || lower.contains("liquidation") {
        return AIPriorityLane::High;
    }
    
    AIPriorityLane::Standard
}

/// Calculate minimum priority fee for a lane
pub fn min_priority_fee(lane: AIPriorityLane) -> u64 {
    match lane {
        AIPriorityLane::Critical => MIN_CRITICAL_FEE,
        AIPriorityLane::High => MIN_HIGH_FEE,
        AIPriorityLane::Standard => MIN_STANDARD_FEE,
    }
}

/// Calculate priority fee for a lane based on transaction size
/// 
/// The fee scales with size to prevent spam with large transactions
/// while ensuring fair pricing for legitimate use.
pub fn calculate_priority_fee(lane: AIPriorityLane, size_bytes: usize) -> u64 {
    let base_fee = min_priority_fee(lane);
    
    // Add size-based component (1 lamport per byte for standard, scaled for others)
    let size_component = match lane {
        AIPriorityLane::Critical => (size_bytes as u64) * 100,   // 100 lamports/byte
        AIPriorityLane::High => (size_bytes as u64) * 10,       // 10 lamports/byte
        AIPriorityLane::Standard => (size_bytes as u64),        // 1 lamport/byte
    };
    
    base_fee + size_component
}

/// Calculate dynamic fee based on lane congestion
/// 
/// During high demand, fees increase to prioritize valuable transactions.
pub fn calculate_dynamic_fee(
    lane: AIPriorityLane,
    base_fee: u64,
    lane_utilization: f64,  // 0.0 to 1.0
) -> u64 {
    let congestion_multiplier = 1.0 + (lane_utilization * 10.0); // Up to 11x
    
    (base_fee as f64 * congestion_multiplier) as u64
}

/// Get lane capacity for a block
/// 
/// Returns (critical_capacity, high_capacity, standard_capacity)
pub fn get_lane_capacities(block_tx_capacity: usize) -> (usize, usize, usize) {
    let critical = block_tx_capacity * CRITICAL_LANE_PCT as usize / 100;
    let high = block_tx_capacity * HIGH_LANE_PCT as usize / 100;
    let standard = block_tx_capacity * STANDARD_LANE_PCT as usize / 100;
    
    (critical, high, standard)
}

/// Lane statistics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct LaneStats {
    pub lane: AIPriorityLane,
    pub pending_count: usize,
    pub capacity: usize,
    pub utilization: f64,
    pub avg_fee: u64,
    pub avg_compute_units: u64,
}

impl LaneStats {
    pub fn new(lane: AIPriorityLane, capacity: usize) -> Self {
        Self {
            lane,
            pending_count: 0,
            capacity,
            utilization: 0.0,
            avg_fee: 0,
            avg_compute_units: 0,
        }
    }
    
    pub fn update(&mut self, pending: usize, total_fees: u64, total_compute: u64) {
        self.pending_count = pending;
        self.utilization = if self.capacity > 0 {
            pending as f64 / self.capacity as f64
        } else {
            0.0
        };
        self.avg_fee = if pending > 0 { total_fees / pending as u64 } else { 0 };
        self.avg_compute_units = if pending > 0 { total_compute / pending as u64 } else { 0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_classify_by_fee() {
        assert_eq!(classify_by_fee(2_000_000), AIPriorityLane::Critical);
        assert_eq!(classify_by_fee(1_000_000), AIPriorityLane::Critical);
        assert_eq!(classify_by_fee(750_000), AIPriorityLane::High);
        assert_eq!(classify_by_fee(500_000), AIPriorityLane::High);
        assert_eq!(classify_by_fee(100_000), AIPriorityLane::Standard);
        assert_eq!(classify_by_fee(0), AIPriorityLane::Standard);
    }
    
    #[test]
    fn test_min_priority_fee() {
        assert_eq!(min_priority_fee(AIPriorityLane::Critical), 1_000_000);
        assert_eq!(min_priority_fee(AIPriorityLane::High), 500_000);
        assert_eq!(min_priority_fee(AIPriorityLane::Standard), 5_000);
    }
    
    #[test]
    fn test_calculate_priority_fee() {
        // Standard: 5000 base + size
        assert_eq!(calculate_priority_fee(AIPriorityLane::Standard, 100), 5_100);
        
        // High: 500000 base + 10 * size
        assert_eq!(calculate_priority_fee(AIPriorityLane::High, 100), 501_000);
        
        // Critical: 1000000 base + 100 * size
        assert_eq!(calculate_priority_fee(AIPriorityLane::Critical, 100), 1_010_000);
    }
    
    #[test]
    fn test_dynamic_fee() {
        let base = 100_000u64;
        
        // No congestion: 1x multiplier
        assert_eq!(calculate_dynamic_fee(AIPriorityLane::High, base, 0.0), 100_000);
        
        // 50% congestion: 6x multiplier
        assert_eq!(calculate_dynamic_fee(AIPriorityLane::High, base, 0.5), 600_000);
        
        // 100% congestion: 11x multiplier
        assert_eq!(calculate_dynamic_fee(AIPriorityLane::High, base, 1.0), 1_100_000);
    }
    
    #[test]
    fn test_lane_capacities() {
        let (critical, high, standard) = get_lane_capacities(10_000);
        assert_eq!(critical, 4_000);
        assert_eq!(high, 3_000);
        assert_eq!(standard, 3_000);
        assert_eq!(critical + high + standard, 10_000);
    }
}
