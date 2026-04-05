//! AI Priority Fees Module

use aether_common::types::AIPriorityLane;

/// Calculate compute unit price for a lane
pub fn compute_unit_price(lane: AIPriorityLane) -> u64 {
    match lane {
        AIPriorityLane::Critical => 10_000,
        AIPriorityLane::High => 1_000,
        AIPriorityLane::Standard => 0,
    }
}

/// Calculate total fee for a transaction
pub fn calculate_fee(lane: AIPriorityLane, compute_units: u64) -> u64 {
    compute_units * compute_unit_price(lane)
}
