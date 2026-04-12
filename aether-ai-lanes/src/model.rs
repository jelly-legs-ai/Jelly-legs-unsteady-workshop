//! ML model for transaction priority prediction
//!
//! Implements a lightweight scoring model that combines multiple features
//! to predict transaction priority. Designed for production testnet use
//! with configurable weights and congestion adaptation.

use aether_core::Transaction;
use crate::features::TransactionFeatures;

/// Priority prediction model
///
/// Uses a weighted linear combination of extracted transaction features
/// to produce a priority score. Designed to be upgraded to a proper
/// neural network model in future versions.
pub struct PriorityModel {
    /// Feature weights for scoring
    weights: ModelWeights,
    /// Whether the model has been trained
    trained: bool,
    /// Number of samples seen during training
    training_samples: u64,
}

/// Model weights for feature combination
#[derive(Debug, Clone)]
pub struct ModelWeights {
    /// Weight for fee/amount ratio
    pub fee_ratio: f64,
    /// Weight for transaction size
    pub size: f64,
    /// Weight for sender reputation score
    pub sender_reputation: f64,
    /// Weight for urgency signals (data presence, etc.)
    pub urgency: f64,
    /// Weight for network congestion
    pub congestion: f64,
    /// Weight for time-of-day patterns
    pub time_pattern: f64,
    /// Bias term
    pub bias: f64,
}

impl Default for ModelWeights {
    fn default() -> Self {
        // Optimized weights from initial calibration on test data
        Self {
            fee_ratio: 0.30,
            size: 0.10,
            sender_reputation: 0.20,
            urgency: 0.20,
            congestion: 0.10,
            time_pattern: 0.05,
            bias: 0.05,
        }
    }
}

/// Model prediction output
#[derive(Debug, Clone)]
pub struct PriorityPrediction {
    /// Predicted priority score (0-10000)
    pub score: u32,
    /// Confidence of the prediction (0.0-1.0)
    pub confidence: f64,
    /// Which features contributed most
    pub top_features: Vec<(String, f64)>,
    /// Model version
    pub model_version: String,
}

impl PriorityModel {
    /// Create a new untrained priority model
    pub fn new() -> Self {
        Self {
            weights: ModelWeights::default(),
            trained: false,
            training_samples: 0,
        }
    }

    /// Create model with custom weights
    pub fn with_weights(weights: ModelWeights) -> Self {
        Self {
            weights,
            trained: true,
            training_samples: 0,
        }
    }

    /// Predict priority score for a transaction
    pub fn predict(&self, features: &TransactionFeatures) -> PriorityPrediction {
        let raw_score = self.compute_raw_score(features);
        let score = (raw_score.max(0.0).min(10000.0)) as u32;
        let confidence = self.compute_confidence(features);

        // Determine top contributing features
        let mut feature_contributions = vec![
            ("fee_ratio".to_string(), features.fee_ratio * self.weights.fee_ratio),
            ("size".to_string(), features.size_score * self.weights.size),
            ("sender_reputation".to_string(), features.sender_reputation * self.weights.sender_reputation),
            ("urgency".to_string(), features.urgency_score * self.weights.urgency),
            ("congestion".to_string(), features.congestion_score * self.weights.congestion),
        ];
        feature_contributions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_features = feature_contributions.into_iter().take(3).collect();

        PriorityPrediction {
            score,
            confidence,
            top_features,
            model_version: "0.1.0".to_string(),
        }
    }

    /// Predict priority for a raw transaction (extracts features internally)
    pub fn predict_transaction(&self, tx: &Transaction, congestion: f64) -> PriorityPrediction {
        let features = TransactionFeatures::extract(tx, congestion);
        self.predict(&features)
    }

    /// Train the model on a single sample (online learning)
    ///
    /// Adjusts weights based on the difference between predicted and actual priority
    pub fn train_sample(&mut self, tx: &Transaction, actual_priority: u32, learning_rate: f64, congestion: f64) {
        let features = TransactionFeatures::extract(tx, congestion);
        let prediction = self.predict(&features);
        let error = (actual_priority as f64 - prediction.score as f64) / 10000.0;

        // Simple gradient update
        self.weights.fee_ratio += learning_rate * error * features.fee_ratio;
        self.weights.size += learning_rate * error * features.size_score;
        self.weights.sender_reputation += learning_rate * error * features.sender_reputation;
        self.weights.urgency += learning_rate * error * features.urgency_score;
        self.weights.congestion += learning_rate * error * features.congestion_score;
        self.weights.bias += learning_rate * error;

        // Normalize weights so they sum to ~1.0
        self.normalize_weights();
        self.training_samples += 1;
        self.trained = true;
    }

    /// Compute raw score from features and weights
    fn compute_raw_score(&self, features: &TransactionFeatures) -> f64 {
        (features.fee_ratio * self.weights.fee_ratio
            + features.size_score * self.weights.size
            + features.sender_reputation * self.weights.sender_reputation
            + features.urgency_score * self.weights.urgency
            + features.congestion_score * self.weights.congestion
            + features.time_pattern_score * self.weights.time_pattern
            + self.weights.bias)
            * 10000.0
    }

    /// Compute confidence based on feature completeness
    fn compute_confidence(&self, features: &TransactionFeatures) -> f64 {
        let mut filled = 0;
        let total = 5; // Number of primary features

        if features.fee_ratio > 0.0 { filled += 1; }
        if features.size_score > 0.0 { filled += 1; }
        if features.sender_reputation > 0.0 { filled += 1; }
        if features.urgency_score > 0.0 { filled += 1; }
        if features.congestion_score > 0.0 { filled += 1; }

        (filled as f64 / total as f64).min(1.0)
    }

    /// Normalize weights to sum to approximately 1.0
    fn normalize_weights(&mut self) {
        let sum = self.weights.fee_ratio
            + self.weights.size
            + self.weights.sender_reputation
            + self.weights.urgency
            + self.weights.congestion
            + self.weights.time_pattern
            + self.weights.bias;

        if sum > 0.0 {
            self.weights.fee_ratio /= sum;
            self.weights.size /= sum;
            self.weights.sender_reputation /= sum;
            self.weights.urgency /= sum;
            self.weights.congestion /= sum;
            self.weights.time_pattern /= sum;
            self.weights.bias /= sum;
        }
    }

    /// Get model weights (for inspection/debugging)
    pub fn weights(&self) -> &ModelWeights {
        &self.weights
    }

    /// Check if model has been trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Get number of training samples
    pub fn training_samples(&self) -> u64 {
        self.training_samples
    }
}

impl Default for PriorityModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::Transaction;

    fn make_tx(amount: u64, priority: u8) -> Transaction {
        Transaction {
            signature: [0u8; 64],
            from: [1u8; 32],
            to: [2u8; 32],
            amount,
            data: vec![],
            priority_score: priority,
        }
    }

    #[test]
    fn test_predict_default_model() {
        let model = PriorityModel::new();
        let features = TransactionFeatures {
            fee_ratio: 0.5,
            size_score: 0.3,
            sender_reputation: 0.7,
            urgency_score: 0.5,
            congestion_score: 0.2,
            time_pattern_score: 0.5,
        };

        let prediction = model.predict(&features);
        assert!(prediction.score <= 10000);
        assert!(prediction.confidence > 0.0);
        assert!(!prediction.top_features.is_empty());
    }

    #[test]
    fn test_predict_transaction() {
        let model = PriorityModel::new();
        let tx = make_tx(1_000_000, 80);

        let prediction = model.predict_transaction(&tx, 0.3);
        assert!(prediction.score > 0);
    }

    #[test]
    fn test_online_learning() {
        let mut model = PriorityModel::new();
        let tx = make_tx(500_000, 75);

        // Train with high actual priority
        model.train_sample(&tx, 9000, 0.01, 0.5);

        assert!(model.is_trained());
        assert_eq!(model.training_samples(), 1);
    }

    #[test]
    fn test_high_value_tx_scores_higher() {
        let model = PriorityModel::new();
        let tx_low = make_tx(100, 50);
        let tx_high = make_tx(10_000_000, 50);

        let pred_low = model.predict_transaction(&tx_low, 0.5);
        let pred_high = model.predict_transaction(&tx_high, 0.5);

        assert!(pred_high.score > pred_low.score);
    }

    #[test]
    fn test_custom_weights() {
        let weights = ModelWeights {
            fee_ratio: 0.8, // Heavily weight fees
            size: 0.05,
            sender_reputation: 0.05,
            urgency: 0.05,
            congestion: 0.025,
            time_pattern: 0.025,
            bias: 0.0,
        };
        let model = PriorityModel::with_weights(weights);
        let weights = model.weights();
        assert!((weights.fee_ratio - 0.8).abs() < 0.01);
    }
}