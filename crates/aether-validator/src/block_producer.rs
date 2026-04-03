//! Block Production Module
//!
//! Produces blocks at a fixed interval (400ms per slot) with PoH hashing.

use crate::state::ValidatorState;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info};

/// Slot duration in milliseconds
pub const SLOT_TIME_MS: u64 = 400;

/// Maximum blocks to keep in rolling history
pub const MAX_BLOCK_HISTORY: usize = 1000;

/// A produced block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub slot: u64,
    pub timestamp: u64,
    pub previous_block_hash: String,
    pub block_hash: String,
    pub transactions: Vec<String>,
    pub poh_seed: String,
}

/// Block producer that runs as an async task
pub struct BlockProducer {
    state: ValidatorState,
    block_history: Arc<RwLock<VecDeque<Block>>>,
}

impl BlockProducer {
    pub fn new(state: ValidatorState) -> Self {
        Self {
            state,
            block_history: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_BLOCK_HISTORY))),
        }
    }

    /// Start the block production loop
    pub async fn run(self: Arc<Self>) {
        info!("Block producer started (slot time: {}ms)", SLOT_TIME_MS);
        
        let mut slot_timer = interval(Duration::from_millis(SLOT_TIME_MS));
        slot_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        let mut current_slot = self.state.current_slot();
        
        loop {
            tokio::select! {
                _ = slot_timer.tick() => {
                    current_slot += 1;
                    self.produce_block(current_slot).await;
                }
            }
        }
    }

    /// Produce a single block
    async fn produce_block(&self, slot: u64) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Get previous block hash
        let history = self.block_history.read().await;
        let previous_hash = if let Some(last_block) = history.back() {
            last_block.block_hash.clone()
        } else {
            // Genesis block - use zero hash
            "0000000000000000000000000000000000000000000000000000000000000000".to_string()
        };
        drop(history);

        // Compute PoH seed (proof of history)
        let poh_seed = Self::compute_poh_seed(slot, timestamp, &previous_hash);
        
        // Compute block hash (includes PoH)
        let block_hash = Self::compute_block_hash(slot, &previous_hash, &poh_seed);

        // Create block
        let block = Block {
            slot,
            timestamp,
            previous_block_hash: previous_hash,
            block_hash: block_hash.clone(),
            transactions: Vec::new(), // MVP: no transactions yet
            poh_seed,
        };

        // Update state
        self.state.set_current_slot(slot);
        self.state.set_block_hash(block_hash.clone());
        self.state.increment_produced_blocks();

        // Store in rolling history
        {
            let mut history = self.block_history.write().await;
            history.push_back(block.clone());
            while history.len() > MAX_BLOCK_HISTORY {
                history.pop_front();
            }
        }

        debug!(
            "Produced block {} with hash {}",
            slot,
            &block.block_hash[..16]
        );
    }

    /// Compute PoH seed using SHA-256
    fn compute_poh_seed(slot: u64, timestamp: u64, previous_hash: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"aether-poh-v1");
        hasher.update(slot.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        hasher.update(previous_hash.as_bytes());
        let result = hasher.finalize();
        bs58::encode(result).into_string()
    }

    /// Compute block hash from slot, previous hash, and PoH seed
    fn compute_block_hash(slot: u64, previous_hash: &str, poh_seed: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"aether-block-v1");
        hasher.update(slot.to_le_bytes());
        hasher.update(previous_hash.as_bytes());
        hasher.update(poh_seed.as_bytes());
        let result = hasher.finalize();
        bs58::encode(result).into_string()
    }

    /// Get a block by slot number
    pub async fn get_block(&self, slot: u64) -> Option<Block> {
        let history = self.block_history.read().await;
        history.iter().find(|b| b.slot == slot).cloned()
    }

    /// Get current block hash
    pub async fn current_block_hash(&self) -> String {
        let history = self.block_history.read().await;
        history.back().map(|b| b.block_hash.clone())
            .unwrap_or_else(|| "0000000000000000000000000000000000000000000000000000000000000000".to_string())
    }
}
