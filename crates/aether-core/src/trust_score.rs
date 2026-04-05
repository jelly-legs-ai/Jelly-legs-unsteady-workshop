//! AETHER Trust Score
//!
//! Reputation and trust scoring for validators and nodes.

use serde::{Deserialize, Serialize};

/// Trust score for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    pub node_id: [u8; 32],
    pub uptime_score: f64,
    pub reliability_score: f64,
    pub behavior_score: f64,
    pub total_score: f64,
}

impl TrustScore {
    /// Calculate combined trust score
    pub fn calculate_total(&mut self) {
        self.total_score = (self.uptime_score + self.reliability_score + self.behavior_score) / 3.0;
    }
}
