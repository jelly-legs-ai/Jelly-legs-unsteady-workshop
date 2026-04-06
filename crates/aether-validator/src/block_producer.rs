//! Block Production Module
//!
//! Produces blocks at a fixed interval (400ms per slot) with PoH hashing.
//! Includes transaction execution and mempool management.

use crate::executor::Executor;
use crate::state_db::StateDB;
use crate::state::ValidatorState;
use aether_core::{
    AetherTransaction, Account, Address, TransactionReceipt,
};
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
    pub receipts: Vec<TransactionReceipt>,
    pub poh_seed: String,
    pub state_root: String,
}

/// Block producer that runs as an async task
pub struct BlockProducer {
    state: ValidatorState,
    block_history: Arc<RwLock<VecDeque<Block>>>,
    transaction_pool: Arc<RwLock<VecDeque<AetherTransaction>>>,
    state_db: Arc<StateDB>,
    executor: Arc<Executor>,
}

impl BlockProducer {
    pub fn new(state: ValidatorState, state_db: StateDB) -> Self {
        let executor = Arc::new(Executor::new(state_db.clone()));
        Self {
            state,
            block_history: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_BLOCK_HISTORY))),
            transaction_pool: Arc::new(RwLock::new(VecDeque::new())),
            state_db: Arc::new(state_db),
            executor,
        }
    }

    /// Start the block production loop
    pub async fn run(self: Arc<Self>) {
        let slot_time_ms = self.state.get_slot_time_ms();
        info!("Block producer started (slot time: {}ms from genesis)", slot_time_ms);
        
        let mut slot_timer = interval(Duration::from_millis(slot_time_ms));
        slot_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        let mut current_slot = self.state.current_slot();
        
        loop {
            tokio::select! {
                _ = slot_timer.tick() => {
                    self.produce_block(current_slot).await;
                    current_slot += 1;
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
        } else if slot == 0 {
            // First block (slot 0) - use genesis hash as the parent
            self.state.get_genesis_hash()
        } else {
            // Fallback for early slots if history is empty
            self.state.get_genesis_hash()
        };
        drop(history);

        // Compute PoH seed (proof of history)
        let poh_seed = Self::compute_poh_seed(slot, timestamp, &previous_hash);
        
        // Execute pending transactions
        let receipts = self.execute_pending_transactions(slot).await;
        let state_root = bs58::encode(self.state_db.compute_state_root()).into_string();
        
        // Collect TX signatures for the block
        let tx_signatures: Vec<String> = receipts.iter()
            .map(|r| bs58::encode(&r.signature).into_string())
            .collect();
        
        // Compute block hash including transaction data
        let block_hash = Self::compute_block_hash_with_txs(
            slot, &previous_hash, &poh_seed, &state_root, &receipts,
        );

        // Update receipts with block hash
        let receipts = receipts.into_iter().map(|mut r| {
            r.block_hash = block_hash.clone();
            r
        }).collect();

        // Create block
        let block = Block {
            slot,
            timestamp,
            previous_block_hash: previous_hash,
            block_hash: block_hash.clone(),
            transactions: tx_signatures,
            receipts,
            poh_seed,
            state_root,
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
            "Produced block {} with hash {} ({} transactions)",
            slot,
            &block.block_hash[..16.min(block.block_hash.len())],
            block.transactions.len()
        );
    }

    /// Execute all pending transactions and return receipts
    async fn execute_pending_transactions(&self, slot: u64) -> Vec<TransactionReceipt> {
        let mut receipts = Vec::new();
        
        // Drain up to 100 transactions from the pool
        let txs_to_process = {
            let mut pool = self.transaction_pool.write().await;
            let count = pool.len().min(100);
            let txs: Vec<_> = pool.drain(0..count).collect();
            txs
        };
        
        for tx in txs_to_process {
            let result = self.executor.execute(&tx);
            let receipt = TransactionReceipt {
                signature: tx.signature.clone(),
                slot,
                block_hash: String::new(),
                tx_type: tx.tx_type.clone(),
                signer: tx.signer,
                result,
                timestamp: tx.timestamp,
            };
            receipts.push(receipt);
        }
        
        receipts
    }

    /// Submit a transaction to the mempool
    /// Returns the base58-encoded signature as the transaction ID for later lookup.
    pub async fn submit_transaction(&self, tx: AetherTransaction) -> Result<String, String> {
        let sig = bs58::encode(&tx.signature).into_string();
        
        let mut pool = self.transaction_pool.write().await;
        pool.push_back(tx);
        Ok(sig)
    }

    /// Get a transaction receipt by signature (base58-encoded)
    pub async fn get_receipt(&self, signature: &str) -> Option<TransactionReceipt> {
        let history = self.block_history.read().await;
        for block in history.iter() {
            if let Some(receipt) = block.receipts.iter().find(|r| {
                bs58::encode(&r.signature).into_string() == signature
            }) {
                return Some(receipt.clone());
            }
        }
        None
    }

    /// Get account state
    pub async fn get_account(&self, address: &Address) -> Option<Account> {
        self.state_db.get_account(address).await
    }

    /// Get total supply (async)
    pub async fn total_supply(&self) -> u64 {
        self.state_db.total_supply().await
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
    #[allow(dead_code)]
    fn compute_block_hash(slot: u64, previous_hash: &str, poh_seed: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"aether-block-v1");
        hasher.update(slot.to_le_bytes());
        hasher.update(previous_hash.as_bytes());
        hasher.update(poh_seed.as_bytes());
        let result = hasher.finalize();
        bs58::encode(result).into_string()
    }

    /// Compute block hash including transaction data
    fn compute_block_hash_with_txs(
        slot: u64,
        prev_hash: &str,
        poh_seed: &str,
        state_root: &str,
        receipts: &[TransactionReceipt],
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"aether-block-v2");
        hasher.update(slot.to_le_bytes());
        hasher.update(prev_hash.as_bytes());
        hasher.update(poh_seed.as_bytes());
        hasher.update(state_root.as_bytes());
        for r in receipts {
            hasher.update(&r.signature);
        }
        let result = hasher.finalize();
        bs58::encode(result).into_string()
    }

    /// Get a block by slot number
    pub async fn get_block(&self, slot: u64) -> Option<Block> {
        let history = self.block_history.read().await;
        history.iter().find(|b| b.slot == slot).cloned()
    }

    /// Get current block hash (most recently produced block)
    pub async fn current_block_hash(&self) -> String {
        let history = self.block_history.read().await;
        history.back().map(|b| b.block_hash.clone())
            .unwrap_or_else(|| self.state.get_genesis_hash())
    }
}
