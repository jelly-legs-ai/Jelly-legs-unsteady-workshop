//! Transaction priority scoring

use aether_core::Transaction;

/// Priority scorer
pub struct PriorityScorer;

impl PriorityScorer {
    /// Score transaction priority (0-100)
    pub fn score(tx: &Transaction) -> u8 {
        // TODO: ML-based scoring
        tx.priority_score
    }
}