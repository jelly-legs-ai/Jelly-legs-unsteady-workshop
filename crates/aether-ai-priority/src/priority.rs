//! AI Priority Module

use aether_common::types::AIPriorityLane;

/// Classify a transaction into a priority lane
pub fn classify_transaction(_data: &[u8]) -> AIPriorityLane {
    AIPriorityLane::Standard
}

/// Calculate priority fee for a lane
pub fn calculate_priority_fee(lane: AIPriorityLane, _size_bytes: usize) -> u64 {
    match lane {
        AIPriorityLane::Critical => 1_000_000,
        AIPriorityLane::High => 500_000,
        AIPriorityLane::Standard => 0,
    }
}
