//! Transaction priority scoring

use aether_core::Transaction;

/// Priority scorer
pub struct PriorityScorer;

impl PriorityScorer {
    /// Score transaction priority (0-100)
    pub fn score(tx: &Transaction) -> u8 {
        // Validate priority score is within bounds (0-100)
        // Cap at 100 to prevent malicious transactions from claiming excessive priority
        tx.priority_score.min(100)
    }
}