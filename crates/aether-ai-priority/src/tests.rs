//! Tests for AI Priority Module

use super::*;
use aether_common::types::AIPriorityLane;

#[test]
fn test_priority_lane_ordering() {
    // Verify lane ordering: Critical(0) < High(1) < Standard(2)
    assert!((AIPriorityLane::Critical as u8) < (AIPriorityLane::High as u8));
    assert!((AIPriorityLane::High as u8) < (AIPriorityLane::Standard as u8));
}

#[test]
fn test_ai_priority_error_display() {
    let err = AIPriorityError::InvalidSignature("test".to_string());
    assert!(err.to_string().contains("Invalid AI signature"));
    
    let err = AIPriorityError::LaneFull(AIPriorityLane::Critical);
    assert!(err.to_string().contains("Priority lane full"));
}
