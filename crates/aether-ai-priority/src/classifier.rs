//! AI Transaction Classifier Module

use aether_common::types::AIPriorityLane;

/// Classifier result
pub struct Classification {
    pub lane: AIPriorityLane,
    pub score: f64,
    pub reasons: Vec<String>,
}

/// Classify by transaction content
pub fn classify(_tx_data: &[u8]) -> Classification {
    Classification {
        lane: AIPriorityLane::Standard,
        score: 0.5,
        reasons: vec![],
    }
}
