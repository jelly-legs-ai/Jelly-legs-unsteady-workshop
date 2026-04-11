//! Transaction priority scoring with AI-powered lane assignment
//!
//! Implements multi-lane transaction routing:
//! - Priority Lane: High-value, time-sensitive transactions (AI-scored)
//! - Standard Lane: Regular transactions
//! - Low-Priority Lane: Low-fee, bulk transactions
//!
//! AI Priority Lane uses a scoring model that considers:
//! - Transaction fee/amount ratio
//! - Account history (reputation)
//! - Network congestion
//! - Time urgency signals

use aether_core::Transaction;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Maximum transactions per lane queue
const MAX_LANE_SIZE: usize = 10_000;

/// Transaction priority lane
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityLane {
    /// AI Priority Lane - fastest processing for high-value txs
    AiPriority = 0,
    /// High priority - time-sensitive transactions
    High = 1,
    /// Standard priority - regular transactions
    Standard = 2,
    /// Low priority - bulk/batch transactions
    Low = 3,
}

impl PriorityLane {
    /// Get the lane name
    pub fn name(&self) -> &'static str {
        match self {
            PriorityLane::AiPriority => "ai_priority",
            PriorityLane::High => "high",
            PriorityLane::Standard => "standard",
            PriorityLane::Low => "low",
        }
    }

    /// Get all lanes in priority order
    pub fn all() -> &'static [PriorityLane] {
        &[
            PriorityLane::AiPriority,
            PriorityLane::High,
            PriorityLane::Standard,
            PriorityLane::Low,
        ]
    }
}

/// Scored transaction with priority metadata
#[derive(Debug, Clone)]
pub struct ScoredTransaction {
    /// The original transaction
    pub transaction: Transaction,
    /// Computed priority score (0-10000, higher = more important)
    pub score: u32,
    /// Assigned lane
    pub lane: PriorityLane,
    /// Timestamp when scored
    pub scored_at: u64,
    /// Estimated fee (in lamports)
    pub estimated_fee: u64,
}

/// Priority scorer configuration
#[derive(Debug, Clone)]
pub struct ScorerConfig {
    /// Minimum score for AI Priority lane
    pub ai_priority_threshold: u32,
    /// Minimum score for High priority lane
    pub high_priority_threshold: u32,
    /// Minimum score for Standard lane
    pub standard_priority_threshold: u32,
    /// Weight for fee amount in scoring
    pub fee_weight: f64,
    /// Weight for account reputation in scoring
    pub reputation_weight: f64,
    /// Weight for urgency signal in scoring
    pub urgency_weight: f64,
    /// Weight for network congestion adjustment
    pub congestion_weight: f64,
}

impl Default for ScorerConfig {
    fn default() -> Self {
        Self {
            ai_priority_threshold: 7500,
            high_priority_threshold: 5000,
            standard_priority_threshold: 2500,
            fee_weight: 0.4,
            reputation_weight: 0.25,
            urgency_weight: 0.2,
            congestion_weight: 0.15,
        }
    }
}

/// AI Priority Lane manager
pub struct PriorityLaneManager {
    /// Transaction queues per lane
    lanes: Arc<RwLock<Vec<VecDeque<ScoredTransaction>>>>,
    /// Scorer configuration
    config: ScorerConfig,
    /// Current network congestion level (0.0 - 1.0)
    congestion_level: Arc<RwLock<f64>>,
    /// Total transactions processed
    total_processed: Arc<RwLock<u64>>,
    /// Total transactions dropped (queue full)
    total_dropped: Arc<RwLock<u64>>,
}

impl PriorityLaneManager {
    /// Create a new priority lane manager with default config
    pub fn new() -> Self {
        Self::with_config(ScorerConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: ScorerConfig) -> Self {
        let lanes = PriorityLane::all()
            .iter()
            .map(|_| VecDeque::with_capacity(MAX_LANE_SIZE))
            .collect();
        Self {
            lanes: Arc::new(RwLock::new(lanes)),
            config,
            congestion_level: Arc::new(RwLock::new(0.0)),
            total_processed: Arc::new(RwLock::new(0)),
            total_dropped: Arc::new(RwLock::new(0)),
        }
    }

    /// Score a transaction and assign it to a lane
    pub fn score_transaction(&self, tx: &Transaction) -> ScoredTransaction {
        let congestion = *self.congestion_level.blocking_read();

        // Compute raw score components
        let fee_score = self.compute_fee_score(tx);
        let reputation_score = self.compute_reputation_score(tx);
        let urgency_score = self.compute_urgency_score(tx);
        let congestion_adj = self.compute_congestion_adjustment(congestion);

        // Weighted combination
        let raw_score = (fee_score as f64 * self.config.fee_weight)
            + (reputation_score as f64 * self.config.reputation_weight)
            + (urgency_score as f64 * self.config.urgency_weight)
            + (congestion_adj as f64 * self.config.congestion_weight);

        // Scale to 0-10000 range and apply transaction's own priority_score
        let base_priority = (tx.priority_score as f64 / 100.0) * 10.0; // 0-10 from tx
        let final_score = ((raw_score + base_priority) * 100.0).min(10000.0) as u32;

        // Assign to lane
        let lane = self.assign_lane(final_score);

        ScoredTransaction {
            transaction: tx.clone(),
            score: final_score,
            lane,
            scored_at: current_timestamp(),
            estimated_fee: self.estimate_fee(tx, congestion),
        }
    }

    /// Submit a transaction to the appropriate lane
    pub async fn submit(&self, tx: &Transaction) -> Result<ScoredTransaction, LaneError> {
        let scored = self.score_transaction(tx);
        let lane_idx = scored.lane as usize;

        let mut lanes = self.lanes.write().await;
        if lanes[lane_idx].len() >= MAX_LANE_SIZE {
            *self.total_dropped.write().await += 1;
            return Err(LaneError::LaneFull(scored.lane));
        }

        // Insert in priority order (highest score first)
        let pos = lanes[lane_idx]
            .iter()
            .position(|existing| existing.score <= scored.score)
            .unwrap_or(lanes[lane_idx].len());

        lanes[lane_idx].insert(pos, scored.clone());

        debug!("Transaction submitted to {} lane (score: {})", scored.lane.name(), scored.score);
        Ok(scored)
    }

    /// Pop the highest-priority transaction across all lanes
    pub async fn pop_next(&self) -> Option<ScoredTransaction> {
        let mut lanes = self.lanes.write().await;

        // Check lanes in priority order
        for lane_idx in 0..PriorityLane::all().len() {
            if let Some(tx) = lanes[lane_idx].pop_front() {
                *self.total_processed.write().await += 1;
                return Some(tx);
            }
        }
        None
    }

    /// Pop up to N transactions from all lanes (in priority order)
    pub async fn pop_batch(&self, max_count: usize) -> Vec<ScoredTransaction> {
        let mut result = Vec::with_capacity(max_count);
        let mut lanes = self.lanes.write().await;

        while result.len() < max_count {
            let mut found = false;
            for lane_idx in 0..PriorityLane::all().len() {
                if result.len() >= max_count {
                    break;
                }
                if let Some(tx) = lanes[lane_idx].pop_front() {
                    result.push(tx);
                    found = true;
                }
            }
            if !found {
                break;
            }
        }

        *self.total_processed.write().await += result.len() as u64;
        result
    }

    /// Get the number of transactions in a specific lane
    pub async fn lane_size(&self, lane: PriorityLane) -> usize {
        self.lanes.read().await[lane as usize].len()
    }

    /// Get total queue size across all lanes
    pub async fn total_queue_size(&self) -> usize {
        self.lanes.read().await.iter().map(|q| q.len()).sum()
    }

    /// Update network congestion level (0.0 - 1.0)
    pub async fn update_congestion(&self, level: f64) {
        *self.congestion_level.write().await = level.clamp(0.0, 1.0);
    }

    /// Get current congestion level
    pub async fn congestion_level(&self) -> f64 {
        *self.congestion_level.read().await
    }

    /// Get lane manager statistics
    pub async fn stats(&self) -> LaneStats {
        let lanes = self.lanes.read().await;
        let ai_priority_len = lanes[0].len();
        let high_len = lanes[1].len();
        let standard_len = lanes[2].len();
        let low_len = lanes[3].len();

        LaneStats {
            ai_priority_queue_size: ai_priority_len,
            high_priority_queue_size: high_len,
            standard_queue_size: standard_len,
            low_priority_queue_size: low_len,
            total_queue_size: ai_priority_len + high_len + standard_len + low_len,
            total_processed: *self.total_processed.read().await,
            total_dropped: *self.total_dropped.read().await,
            congestion_level: *self.congestion_level.read().await,
        }
    }

    // --- Private scoring methods ---

    /// Compute fee-based score (0-10000)
    fn compute_fee_score(&self, tx: &Transaction) -> f64 {
        // Higher amount = higher priority, but with diminishing returns
        // Logarithmic scaling to prevent whale dominance
        if tx.amount == 0 {
            return 1000.0; // Minimum fee score for zero-amount txs (governance, etc.)
        }
        // Score based on amount: 1000 + log2(amount) * 100, capped at 10000
        ((1000.0 + (tx.amount as f64).log2() * 100.0)).min(10000.0)
    }

    /// Compute reputation score (0-10000)
    fn compute_reputation_score(&self, tx: &Transaction) -> f64 {
        // Use priority_score from transaction as a proxy for account reputation
        // This allows wallets/clients to signal priority
        (tx.priority_score as f64 / 100.0) * 100.0
    }

    /// Compute urgency score (0-10000)
    fn compute_urgency_score(&self, tx: &Transaction) -> f64 {
        // Transactions with data (smart contract calls) are often more urgent
        if !tx.data.is_empty() {
            5000.0
        } else {
            3000.0
        }
    }

    /// Compute congestion adjustment (0-10000)
    fn compute_congestion_adjustment(&self, congestion: f64) -> f64 {
        // During high congestion, prioritize higher-fee transactions more
        // During low congestion, be more egalitarian
        (1.0 - congestion) * 5000.0
    }

    /// Assign a score to the appropriate lane
    fn assign_lane(&self, score: u32) -> PriorityLane {
        if score >= self.config.ai_priority_threshold {
            PriorityLane::AiPriority
        } else if score >= self.config.high_priority_threshold {
            PriorityLane::High
        } else if score >= self.config.standard_priority_threshold {
            PriorityLane::Standard
        } else {
            PriorityLane::Low
        }
    }

    /// Estimate transaction fee based on amount and congestion
    fn estimate_fee(&self, tx: &Transaction, congestion: f64) -> u64 {
        let base_fee = 5000; // 5000 lamports base fee
        let amount_fee = (tx.amount as f64 * 0.001) as u64; // 0.1% of amount
        let congestion_multiplier = 1.0 + congestion * 2.0; // Up to 3x during high congestion

        ((base_fee + amount_fee) as f64 * congestion_multiplier) as u64
    }
}

impl Default for PriorityLaneManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Lane errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum LaneError {
    #[error("Lane {0:?} is full")]
    LaneFull(PriorityLane),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
}

/// Lane statistics
#[derive(Debug, Clone)]
pub struct LaneStats {
    pub ai_priority_queue_size: usize,
    pub high_priority_queue_size: usize,
    pub standard_queue_size: usize,
    pub low_priority_queue_size: usize,
    pub total_queue_size: usize,
    pub total_processed: u64,
    pub total_dropped: u64,
    pub congestion_level: f64,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::{Address, Signature};

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

    fn make_tx_with_data(amount: u64, priority: u8, data: Vec<u8>) -> Transaction {
        Transaction {
            signature: [0u8; 64],
            from: [1u8; 32],
            to: [2u8; 32],
            amount,
            data,
            priority_score: priority,
        }
    }

    #[tokio::test]
    async fn test_lane_assignment() {
        let manager = PriorityLaneManager::new();

        // High priority transaction
        let tx_high = make_tx(10_000_000, 100);
        let scored = manager.score_transaction(&tx_high);
        assert!(matches!(scored.lane, PriorityLane::AiPriority | PriorityLane::High));

        // Low priority transaction
        let tx_low = make_tx(100, 10);
        let scored = manager.score_transaction(&tx_low);
        assert!(matches!(scored.lane, PriorityLane::Standard | PriorityLane::Low));
    }

    #[tokio::test]
    async fn test_submit_and_pop() {
        let manager = PriorityLaneManager::new();

        let tx1 = make_tx(1000, 50);
        let tx2 = make_tx(10000, 90);

        manager.submit(&tx2).await.unwrap();
        manager.submit(&tx1).await.unwrap();

        // Should pop highest priority first
        let next = manager.pop_next().await.unwrap();
        assert!(next.score >= manager.score_transaction(&tx1).score);
    }

    #[tokio::test]
    async fn test_batch_pop() {
        let manager = PriorityLaneManager::new();

        for i in 0..10 {
            let tx = make_tx((i + 1) * 1000, 50);
            manager.submit(&tx).await.unwrap();
        }

        let batch = manager.pop_batch(5).await;
        assert_eq!(batch.len(), 5);
    }

    #[tokio::test]
    async fn test_congestion_update() {
        let manager = PriorityLaneManager::new();

        manager.update_congestion(0.5).await;
        assert!((manager.congestion_level().await - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_lane_full() {
        let config = ScorerConfig {
            ai_priority_threshold: 0, // Everything goes to AI lane
            ..Default::default()
        };
        let manager = PriorityLaneManager::with_config(config);

        // Fill up the AI lane
        for _ in 0..MAX_LANE_SIZE {
            let tx = make_tx(1_000_000, 100);
            manager.submit(&tx).await.unwrap();
        }

        // Next submission should fail
        let tx = make_tx(1_000_000, 100);
        let result = manager.submit(&tx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = PriorityLaneManager::new();
        let tx = make_tx(1000, 50);
        manager.submit(&tx).await.unwrap();

        let stats = manager.stats().await;
        assert!(stats.total_queue_size >= 1);
    }
}