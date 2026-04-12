//! Feature extraction for ML-based transaction priority
//!
//! Extracts structured features from raw transactions for use
//! in the priority prediction model. Features are normalized
//! to 0.0-1.0 range for consistent model input.

use aether_core::Transaction;

/// Extracted transaction features for ML model input
#[derive(Debug, Clone)]
pub struct TransactionFeatures {
    /// Fee-to-amount ratio (0.0-1.0) - higher fee ratio = more willing to pay
    pub fee_ratio: f64,
    /// Transaction size score (0.0-1.0) - normalized data size
    pub size_score: f64,
    /// Sender reputation score (0.0-1.0) - derived from priority_score
    pub sender_reputation: f64,
    /// Urgency score (0.0-1.0) - based on data presence and type
    pub urgency_score: f64,
    /// Network congestion score (0.0-1.0)
    pub congestion_score: f64,
    /// Time pattern score (0.0-1.0) - time-of-day pattern
    pub time_pattern_score: f64,
}

impl TransactionFeatures {
    /// Extract features from a transaction
    pub fn extract(tx: &Transaction, congestion: f64) -> Self {
        Self {
            fee_ratio: Self::compute_fee_ratio(tx),
            size_score: Self::compute_size_score(tx),
            sender_reputation: Self::compute_reputation(tx),
            urgency_score: Self::compute_urgency(tx),
            congestion_score: congestion.clamp(0.0, 1.0),
            time_pattern_score: Self::compute_time_pattern(tx),
        }
    }

    /// Compute fee ratio: how much the sender is willing to pay relative to amount
    fn compute_fee_ratio(tx: &Transaction) -> f64 {
        // Use priority_score as a proxy for fee willingness
        // Higher priority_score = more willing to pay fees
        let base = tx.priority_score as f64 / 100.0;

        // Scale by amount (smaller amounts with high priority are more impressive)
        if tx.amount == 0 {
            return base * 0.5; // Zero-amount txs get moderate fee score
        }

        // Logarithmic scaling prevents large amounts from dominating
        let amount_factor = (1.0 + (tx.amount as f64).ln()).min(10.0) / 10.0;
        (base * 0.7 + amount_factor * 0.3).clamp(0.0, 1.0)
    }

    /// Compute size score: larger transactions need more processing
    fn compute_size_score(tx: &Transaction) -> f64 {
        // Base size from transaction data
        let data_size = tx.data.len() as f64;

        // Normalize: 0 bytes = 0.0, ~1KB = 0.5, ~10KB+ = 1.0
        if data_size == 0.0 {
            0.1 // Minimal base score for empty data
        } else {
            (data_size / 10000.0).min(1.0)
        }
    }

    /// Compute sender reputation from transaction metadata
    fn compute_reputation(tx: &Transaction) -> f64 {
        // Use priority_score as reputation proxy
        // In production, this would query an on-chain reputation system
        (tx.priority_score as f64 / 100.0).clamp(0.0, 1.0)
    }

    /// Compute urgency score based on transaction characteristics
    fn compute_urgency(tx: &Transaction) -> f64 {
        let mut score = 0.3; // Base urgency

        // Transactions with data (smart contract calls) are typically more urgent
        if !tx.data.is_empty() {
            score += 0.3;
        }

        // High priority score indicates urgency
        score += (tx.priority_score as f64 / 100.0) * 0.4;

        score.clamp(0.0, 1.0)
    }

    /// Compute time pattern score (for production: would use actual timestamps)
    fn compute_time_pattern(tx: &Transaction) -> f64 {
        // Simplified: use priority score as a proxy for time sensitivity
        // In production, would analyze time-of-day patterns
        (tx.priority_score as f64 / 200.0).clamp(0.0, 1.0)
    }

    /// Convert features to a vector for ML model input
    pub fn to_vector(&self) -> Vec<f32> {
        vec![
            self.fee_ratio as f32,
            self.size_score as f32,
            self.sender_reputation as f32,
            self.urgency_score as f32,
            self.congestion_score as f32,
            self.time_pattern_score as f32,
        ]
    }

    /// Get feature names in order
    pub fn feature_names() -> Vec<&'static str> {
        vec![
            "fee_ratio",
            "size_score",
            "sender_reputation",
            "urgency_score",
            "congestion_score",
            "time_pattern_score",
        ]
    }
}

/// Batch feature extraction for multiple transactions
pub fn extract_batch(transactions: &[Transaction], congestion: f64) -> Vec<TransactionFeatures> {
    transactions
        .iter()
        .map(|tx| TransactionFeatures::extract(tx, congestion))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(amount: u64, priority: u8, data: Vec<u8>) -> Transaction {
        Transaction {
            signature: [0u8; 64],
            from: [1u8; 32],
            to: [2u8; 32],
            amount,
            data,
            priority_score: priority,
        }
    }

    #[test]
    fn test_feature_extraction() {
        let tx = make_tx(1_000_000, 80, vec![1, 2, 3]);
        let features = TransactionFeatures::extract(&tx, 0.5);

        assert!(features.fee_ratio > 0.0);
        assert!(features.size_score > 0.0);
        assert!(features.sender_reputation > 0.0);
        assert!(features.urgency_score > 0.0);
        assert!((features.congestion_score - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_feature_ranges() {
        let tx = make_tx(100, 50, vec![]);
        let features = TransactionFeatures::extract(&tx, 0.3);

        assert!(features.fee_ratio >= 0.0 && features.fee_ratio <= 1.0);
        assert!(features.size_score >= 0.0 && features.size_score <= 1.0);
        assert!(features.sender_reputation >= 0.0 && features.sender_reputation <= 1.0);
        assert!(features.urgency_score >= 0.0 && features.urgency_score <= 1.0);
        assert!(features.congestion_score >= 0.0 && features.congestion_score <= 1.0);
    }

    #[test]
    fn test_zero_amount_transaction() {
        let tx = make_tx(0, 50, vec![]);
        let features = TransactionFeatures::extract(&tx, 0.5);

        // Zero amount should still produce valid features
        assert!(features.fee_ratio >= 0.0);
    }

    #[test]
    fn test_to_vector() {
        let tx = make_tx(1000, 75, vec![1, 2, 3]);
        let features = TransactionFeatures::extract(&tx, 0.5);
        let vec = features.to_vector();

        assert_eq!(vec.len(), 6);
        assert_eq!(TransactionFeatures::feature_names().len(), 6);
    }

    #[test]
    fn test_batch_extraction() {
        let txs = vec![
            make_tx(1000, 50, vec![]),
            make_tx(5000, 80, vec![1, 2, 3]),
        ];

        let features = extract_batch(&txs, 0.3);
        assert_eq!(features.len(), 2);
    }

    #[test]
    fn test_higher_priority_higher_urgency() {
        let tx_low = make_tx(1000, 20, vec![]);
        let tx_high = make_tx(1000, 90, vec![]);

        let features_low = TransactionFeatures::extract(&tx_low, 0.5);
        let features_high = TransactionFeatures::extract(&tx_high, 0.5);

        assert!(features_high.urgency_score > features_low.urgency_score);
        assert!(features_high.sender_reputation > features_low.sender_reputation);
    }
}