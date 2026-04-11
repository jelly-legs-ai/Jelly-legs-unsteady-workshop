//! Block Production Module
//!
//! Produces blocks at a fixed interval (400ms per slot) with PoH hashing.
//! Includes transaction execution and mempool management.
//! Persists blocks to disk for crash recovery.
//!
//! AI Priority Lanes:
//! - Critical: AI governance, emergency operations (10x base fee, 100% to treasury)
//! - High: AI agent transactions, MEV protection (5x base fee, 50% treasury/50% validators)
//! - Standard: Regular user transactions (base fee, 100% to validators)
//!
//! Transaction Ordering (AI-vs-AI Competition):
//! Transactions are ordered by priority lane (Critical > High > Standard) and within each
//! lane by fee (highest fee first). This enables AI operators to compete for block inclusion
//! by offering higher fees, with the proceeds funding network development.

use crate::executor::Executor;
use crate::state_db::StateDB;
use crate::state::ValidatorState;
use crate::persistence::{PersistenceManager, PersistedBlock, PersistedAccount};
use aether_core::{
    AetherTransaction, Account, Address, TransactionReceipt, TransactionPayload,
};
use aether_ai_priority::fee_distribution::FeeDistributor;
use aether_common::types::AIPriorityLane;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

// ============================================================================
// Transaction Priority Queue for AI-vs-AI Competition
// ============================================================================

/// Priority-ordered transaction for the mempool.
/// Ordered by (lane_priority, fee) descending - highest priority and highest fee first.
#[derive(Debug, Clone)]
struct PrioritizedTransaction {
    /// The actual transaction
    tx: AetherTransaction,
    /// Derived priority lane from fee
    lane: AIPriorityLane,
    /// Compute units estimate
    compute_units: u64,
    /// Arrival sequence (for FIFO fairness within same fee)
    sequence: u64,
}

impl PrioritizedTransaction {
    fn new(tx: AetherTransaction, lane: AIPriorityLane, compute_units: u64, sequence: u64) -> Self {
        Self { tx, lane, compute_units, sequence }
    }
    
    /// Priority ordering key: (lane_priority, fee, -sequence)
    /// Higher lane priority first, then higher fee, then earlier arrival (lower sequence wins)
    fn priority_key(&self) -> (u8, u64, std::cmp::Reverse<u64>) {
        let lane_priority = match self.lane {
            AIPriorityLane::Critical => 2,  // Highest
            AIPriorityLane::High => 1,
            AIPriorityLane::Standard => 0,  // Lowest
        };
        (lane_priority, self.tx.fee, std::cmp::Reverse(self.sequence))
    }
}

/// Transaction pool with priority-based ordering for AI competition.
/// 
/// Maintains separate internal queues for each lane to enable:
/// 1. Fair capacity allocation (40% Critical, 30% High, 30% Standard)
/// 2. Fee-based ordering within each lane (higher fee = earlier execution)
/// 3. FIFO fairness for same-fee transactions
struct TransactionPool {
    /// All transactions awaiting execution
    transactions: Vec<PrioritizedTransaction>,
    /// Sequence counter for arrival ordering
    sequence: u64,
    /// Maximum transactions per block
    max_tx_per_block: usize,
    /// Maximum compute units per block
    max_compute_per_block: u64,
}

impl TransactionPool {
    fn new(max_tx_per_block: usize, max_compute_per_block: u64) -> Self {
        Self {
            transactions: Vec::new(),
            sequence: 0,
            max_tx_per_block,
            max_compute_per_block,
        }
    }
    
    /// Add a transaction to the pool
    fn push(&mut self, tx: AetherTransaction, compute_units: u64) {
        let lane = Self::derive_priority_lane(tx.fee);
        let prioritized = PrioritizedTransaction::new(tx, lane, compute_units, self.sequence);
        self.sequence += 1;
        self.transactions.push(prioritized);
    }
    
    /// Derive priority lane from fee amount
    fn derive_priority_lane(fee: u64) -> AIPriorityLane {
        if fee >= 1_000_000 {
            AIPriorityLane::Critical
        } else if fee >= 500_000 {
            AIPriorityLane::High
        } else {
            AIPriorityLane::Standard
        }
    }
    
    /// Get transactions for the next block, ordered by priority.
    /// 
    /// Allocation strategy:
    /// - Critical lane: up to 40% of block (AI governance, emergencies)
    /// - High lane: up to 30% of block (AI agents, MEV protection)
    /// - Standard lane: remaining capacity (regular users)
    /// 
    /// Within each lane, transactions are ordered by fee (highest first) 
    /// enabling AI operators to compete for inclusion.
    fn drain_block_transactions(&mut self) -> Vec<AetherTransaction> {
        // Sort all transactions by priority (highest first)
        self.transactions.sort_by(|a, b| {
            // Compare priority keys in reverse (higher priority first)
            b.priority_key().cmp(&a.priority_key())
        });
        
        let mut result = Vec::new();
        let mut total_compute: u64 = 0;
        
        // Track counts per lane for capacity allocation
        let critical_limit = self.max_tx_per_block * 4 / 10;  // 40%
        let high_limit = self.max_tx_per_block * 3 / 10;      // 30%
        let standard_limit = self.max_tx_per_block * 3 / 10;   // 30%
        
        let mut critical_count = 0usize;
        let mut high_count = 0usize;
        let mut standard_count = 0usize;
        
        // Drain transactions in priority order
        let mut remaining: Vec<PrioritizedTransaction> = Vec::new();
        
        for pt in self.transactions.drain(..) {
            // Check compute unit limit
            if total_compute + pt.compute_units > self.max_compute_per_block {
                remaining.push(pt);
                continue;
            }
            
            // Check lane capacity
            let within_lane_capacity = match pt.lane {
                AIPriorityLane::Critical if critical_count < critical_limit => true,
                AIPriorityLane::High if high_count < high_limit => true,
                AIPriorityLane::Standard if standard_count < standard_limit => true,
                // If lane is full but we have remaining capacity, allow overflow
                _ if result.len() < self.max_tx_per_block => true,
                _ => false,
            };
            
            if !within_lane_capacity {
                remaining.push(pt);
                continue;
            }
            
            // Accept transaction
            total_compute += pt.compute_units;
            result.push(pt.tx);
            
            match pt.lane {
                AIPriorityLane::Critical => critical_count += 1,
                AIPriorityLane::High => high_count += 1,
                AIPriorityLane::Standard => standard_count += 1,
            }
        }
        
        // Put remaining transactions back for next block
        self.transactions = remaining;
        
        result
    }
    
    /// Get pool statistics
    fn stats(&self) -> PoolStats {
        let mut critical = 0usize;
        let mut high = 0usize;
        let mut standard = 0usize;
        let mut total_fees = 0u64;
        
        for pt in &self.transactions {
            match pt.lane {
                AIPriorityLane::Critical => critical += 1,
                AIPriorityLane::High => high += 1,
                AIPriorityLane::Standard => standard += 1,
            }
            total_fees += pt.tx.fee;
        }
        
        PoolStats {
            critical_pending: critical,
            high_pending: high,
            standard_pending: standard,
            total_pending: self.transactions.len(),
            total_fees_pending: total_fees,
        }
    }
}

/// Pool statistics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    pub critical_pending: usize,
    pub high_pending: usize,
    pub standard_pending: usize,
    #[allow(dead_code)]
    pub total_pending: usize,
    pub total_fees_pending: u64,
}

/// Slot duration in milliseconds
pub const SLOT_TIME_MS: u64 = 400;

/// Maximum blocks to keep in rolling history
pub const MAX_BLOCK_HISTORY: usize = 1000;

/// Maximum compute units per block (must match consensus spec in aether-consensus)
pub const MAX_COMPUTE_UNITS_PER_BLOCK: u64 = 48_000_000;

/// Maximum transactions per block
pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 10_000;

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
    /// Priority-ordered transaction pool for AI-vs-AI competition
    transaction_pool: Arc<RwLock<TransactionPool>>,
    state_db: Arc<StateDB>,
    executor: Arc<Executor>,
    persistence: Option<Arc<PersistenceManager>>,
    _ledger_path: PathBuf,
    /// AI Priority Fee Distributor — integrated into block production
    fee_distributor: Arc<FeeDistributor>,
}

impl BlockProducer {
    pub fn new(state: ValidatorState, state_db: StateDB) -> Self {
        let executor = Arc::new(Executor::new(state_db.clone()));
        Self {
            state,
            block_history: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_BLOCK_HISTORY))),
            transaction_pool: Arc::new(RwLock::new(TransactionPool::new(
                MAX_TRANSACTIONS_PER_BLOCK,
                MAX_COMPUTE_UNITS_PER_BLOCK,
            ))),
            state_db: Arc::new(state_db),
            executor,
            persistence: None,
            _ledger_path: PathBuf::from("ledger"),
            fee_distributor: Arc::new(FeeDistributor::new()),
        }
    }
    
    /// Create block producer with persistence enabled
    pub fn with_persistence(state: ValidatorState, state_db: StateDB, ledger_path: PathBuf) -> anyhow::Result<Self> {
        let executor = Arc::new(Executor::new(state_db.clone()));
        let persistence = Arc::new(PersistenceManager::new(&ledger_path)?);
        
        // Try to restore previous state
        if let Some(snapshot) = persistence.load_snapshot()? {
            info!("Restoring validator state from snapshot: slot={}, blocks={}", 
                snapshot.current_slot, snapshot.blocks_produced);
            state.set_current_slot(snapshot.current_slot);
            state.set_block_hash(snapshot.block_hash.clone());
            // Restore blocks produced count
            for _ in 0..snapshot.blocks_produced {
                state.increment_produced_blocks();
            }
            
            // Restore peers
            for peer in snapshot.peers {
                state.add_peer(peer);
            }
        }
        
        // Restore accounts
        let accounts = persistence.load_accounts()?;
        if !accounts.is_empty() {
            info!("Restoring {} accounts from disk", accounts.len());
            for acc in accounts {
                let addr = acc.address;
                let account = Account {
                    lamports: acc.lamports,
                    owner: acc.owner,
                    data: acc.data,
                    rent_epoch: acc.rent_epoch,
                };
                state_db.set_account_sync(&addr, account);
            }
        }
        
        // Restore recent blocks into history
        if let Some(latest_slot) = persistence.get_latest_slot()? {
            let start = latest_slot.saturating_sub(MAX_BLOCK_HISTORY as u64);
            let blocks = persistence.load_blocks_range(start, latest_slot + 1)?;
            if !blocks.is_empty() {
                info!("Restoring {} blocks from disk (slots {}-{})", 
                    blocks.len(), start, latest_slot);
                let mut history = VecDeque::with_capacity(MAX_BLOCK_HISTORY);
                for b in blocks {
                    history.push_back(Block {
                        slot: b.slot,
                        timestamp: b.timestamp,
                        previous_block_hash: b.previous_block_hash,
                        block_hash: b.block_hash,
                        transactions: b.transactions,
                        receipts: vec![], // Receipts not restored (can be recomputed if needed)
                        poh_seed: b.poh_seed,
                        state_root: b.state_root,
                    });
                }
                // Block history will be set via Arc in run()
            }
        }
        
        Ok(Self {
            state,
            block_history: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_BLOCK_HISTORY))),
            transaction_pool: Arc::new(RwLock::new(TransactionPool::new(
                MAX_TRANSACTIONS_PER_BLOCK,
                MAX_COMPUTE_UNITS_PER_BLOCK,
            ))),
            state_db: Arc::new(state_db),
            executor,
            persistence: Some(persistence),
            _ledger_path: ledger_path,
            fee_distributor: Arc::new(FeeDistributor::new()),
        })
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
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

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
        
        // Execute pending transactions with error handling
        let receipts = match self.execute_pending_transactions(slot).await {
            r if r.is_empty() && slot > 0 => {
                debug!("Block {} produced with no transactions", slot);
                r
            }
            r => r
        };
        
        // Compute state root with error handling
        let state_root = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.state_db.compute_state_root()
        })) {
            Ok(root) => bs58::encode(&root).into_string(),
            Err(_) => {
                tracing::error!("Failed to compute state root for block {}", slot);
                bs58::encode(&[0u8; 32]).into_string() // Fallback to zero hash
            }
        };
        
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
        
        // Check for epoch transition and advance fee distributor epoch
        let slots_per_epoch = self.state.get_slots_per_epoch();
        let prev_slot = slot.saturating_sub(1);
        let prev_epoch = prev_slot / slots_per_epoch;
        let current_epoch = slot / slots_per_epoch;
        
        if current_epoch > prev_epoch {
            // Epoch transition - finalize epoch stats in fee distributor
            let _epoch_stats = self.fee_distributor.advance_epoch();
            // Also advance staking pool epoch (distributes rewards to all stakes)
            self.state.advance_staking_epoch();
            info!("Epoch transition: {} -> {} at slot {}", prev_epoch, current_epoch, slot);
        }

        // Store in rolling history
        {
            let mut history = self.block_history.write().await;
            history.push_back(block.clone());
            while history.len() > MAX_BLOCK_HISTORY {
                history.pop_front();
            }
        }
        
        // Persist block and state to disk (every 10 blocks to reduce I/O)
        if slot % 10 == 0 {
            self.persist_state().await;
        }

        debug!(
            "Produced block {} with hash {} ({} transactions)",
            slot,
            &block.block_hash[..16.min(block.block_hash.len())],
            block.transactions.len()
        );
    }
    
    /// Persist current state to disk (called periodically)
    async fn persist_state(&self) {
        if let Some(ref pm) = self.persistence {
            // Save validator snapshot
            let snapshot = crate::persistence::ValidatorSnapshot {
                current_slot: self.state.current_slot(),
                block_hash: self.state.get_last_block_hash(),
                blocks_produced: self.state.blocks_produced(),
                transaction_count: self.state.transaction_count(),
                genesis_hash: self.state.get_genesis_hash(),
                chain_id: self.state.get_chain_id(),
                peers: Vec::new(), // Peers tracked separately in NetworkState
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            
            if let Err(e) = pm.save_snapshot(&snapshot) {
                warn!("Failed to persist validator snapshot: {}", e);
            }
            
            // Save accounts
            let accounts = self.state_db.get_all_accounts_sync();
            let persisted: Vec<PersistedAccount> = accounts.into_iter()
                .map(|(addr, acc)| PersistedAccount {
                    address: addr,
                    lamports: acc.lamports,
                    owner: acc.owner,
                    data: acc.data,
                    rent_epoch: acc.rent_epoch,
                })
                .collect();
            
            if let Err(e) = pm.save_accounts(&persisted) {
                warn!("Failed to persist accounts: {}", e);
            }
            
            // Save latest block
            let history = self.block_history.read().await;
            if let Some(latest_block) = history.back() {
                let persisted_block = PersistedBlock {
                    slot: latest_block.slot,
                    timestamp: latest_block.timestamp,
                    previous_block_hash: latest_block.previous_block_hash.clone(),
                    block_hash: latest_block.block_hash.clone(),
                    transactions: latest_block.transactions.clone(),
                    poh_seed: latest_block.poh_seed.clone(),
                    state_root: latest_block.state_root.clone(),
                };
                
                if let Err(e) = pm.save_block(&persisted_block) {
                    warn!("Failed to persist block: {}", e);
                }
            }
            
            debug!("Persisted state at slot {}", snapshot.current_slot);
        }
    }

    /// Execute all pending transactions and return receipts.
    /// 
    /// Transactions are ordered by priority:
    /// 1. Lane priority: Critical > High > Standard
    /// 2. Within lane: Higher fee first (AI operators compete on fees)
    /// 3. Same fee: FIFO (earlier arrival wins)
    /// 
    /// Capacity allocation: 40% Critical, 30% High, 30% Standard
    async fn execute_pending_transactions(&self, slot: u64) -> Vec<TransactionReceipt> {
        // Drain transactions in priority order from the pool
        let txs_to_process = {
            let mut pool = self.transaction_pool.write().await;
            pool.drain_block_transactions()
        };
        
        // Get timestamp for fee receipts
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        let mut receipts = Vec::new();
        let mut total_compute_units: u64 = 0;
        
        // Execute transactions in priority order
        for tx in txs_to_process {
            let compute_units = self.estimate_compute_units(&tx);
            
            // Check compute unit limit
            if total_compute_units.saturating_add(compute_units) > MAX_COMPUTE_UNITS_PER_BLOCK {
                // Skip this transaction - would exceed compute limit
                debug!("Skipping transaction due to compute limit: {} units used", total_compute_units);
                continue;
            }
            
            total_compute_units += compute_units;
            
            let exec_result = self.executor.execute(&tx);
            
            if exec_result.success {
                // Derive AI Priority Lane from the transaction's fee amount
                let lane = self.derive_priority_lane(tx.fee);
                
                // Process fee through the AI Priority Fee Distributor
                let _fee_receipt = self.fee_distributor.process_fee(
                    tx.signature,
                    lane,
                    compute_units,
                    slot,
                    timestamp,
                );
                
                let receipt = TransactionReceipt {
                    signature: tx.signature,
                    slot,
                    block_hash: String::new(),
                    tx_type: tx.tx_type,
                    signer: tx.signer,
                    result: exec_result,
                    timestamp: tx.timestamp,
                };
                receipts.push(receipt);
            } else {
                // Log failed transaction but continue processing others
                debug!("Transaction {} failed: {:?}", 
                    bs58::encode(&tx.signature).into_string(), 
                    exec_result.error);
            }
        }
        
        // Finalize block: distribute validator fees to the block producer
        if let Some(identity_bytes) = self.state.identity_pubkey_bytes() {
            let _validator_fees = self.fee_distributor.finalize_block(&identity_bytes);
        }
        
        debug!(
            "Executed {} transactions using {} compute units (limit: {})",
            receipts.len(),
            total_compute_units,
            MAX_COMPUTE_UNITS_PER_BLOCK
        );
        
        receipts
    }
    
    /// Derive AI Priority Lane from transaction fee.
    /// 
    /// Fee thresholds:
    /// - Critical: >= 1_000_000 lamports (10x base fee, 100% to treasury)
    /// - High:     >= 500_000 lamports (5x base fee, 50% treasury / 50% validators)
    /// - Standard: < 500_000 lamports (base fee, 100% to validators)
    fn derive_priority_lane(&self, fee: u64) -> AIPriorityLane {
        if fee >= 1_000_000 {
            AIPriorityLane::Critical
        } else if fee >= 500_000 {
            AIPriorityLane::High
        } else {
            AIPriorityLane::Standard
        }
    }

    /// Estimate compute units for a transaction based on its type
    /// These estimates should match the actual execution costs in executor.rs
    fn estimate_compute_units(&self, tx: &AetherTransaction) -> u64 {
        // Base cost for signature verification and transaction processing
        const BASE_COST: u64 = 150_000;
        
        // Additional cost based on transaction type
        let type_cost = match &tx.payload {
            TransactionPayload::Transfer { .. } => 50_000,
            TransactionPayload::Stake { .. } => 200_000,
            TransactionPayload::Unstake { .. } => 200_000,
            TransactionPayload::ClaimRewards { .. } => 150_000,
            TransactionPayload::CreateNFT { .. } => 300_000,
            TransactionPayload::MintNFT { .. } => 250_000,
            TransactionPayload::TransferNFT { .. } => 180_000,
            TransactionPayload::UpdateMetadata { .. } => 120_000,
        };
        
        BASE_COST + type_cost
    }

    /// Submit a transaction to the mempool.
    /// Returns the base58-encoded signature as the transaction ID for later lookup.
    /// 
    /// Transactions are prioritized by:
    /// 1. Lane (Critical > High > Standard) - derived from fee amount
    /// 2. Fee (higher fee = earlier inclusion)
    /// 3. Arrival time (FIFO fairness for same-fee transactions)
    pub async fn submit_transaction(&self, tx: AetherTransaction) -> Result<String, String> {
        let sig = bs58::encode(&tx.signature).into_string();
        let compute_units = self.estimate_compute_units(&tx);
        
        let mut pool = self.transaction_pool.write().await;
        pool.push(tx, compute_units);
        
        debug!("Submitted transaction {} with fee {} (lane: {:?})", 
            sig, pool.transactions.last().map(|t| t.tx.fee).unwrap_or(0),
            pool.transactions.last().map(|t| t.lane).unwrap_or(AIPriorityLane::Standard));
        
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

    /// Get current state root (for snapshots)
    pub fn get_state_root_sync(&self) -> String {
        let accounts = self.state_db.get_all_accounts_sync();
        let mut hasher = Sha256::new();
        let mut addrs: Vec<_> = accounts.iter().map(|(addr, _)| addr).collect();
        addrs.sort();
        for addr in addrs {
            hasher.update(addr);
            // Find account for this address in the vector
            if let Some((_, acc)) = accounts.iter().find(|(a, _)| a == addr) {
                hasher.update(acc.lamports.to_le_bytes());
                hasher.update(&acc.owner);
            }
        }
        let result = hasher.finalize();
        bs58::encode(result).into_string()
    }

    /// Get all accounts for snapshot creation
    pub fn get_all_accounts_sync(&self) -> Vec<([u8; 32], Account)> {
        self.state_db.get_all_accounts_sync()
    }

    /// Set account for snapshot restore
    pub fn set_account_sync(&self, address: &[u8; 32], account: Account) {
        self.state_db.set_account_sync(address, account);
    }
    
    /// Get a reference to the AI Priority Fee Distributor.
    /// Used by the RPC server to expose fee economics and lane stats.
    pub fn fee_distributor(&self) -> Arc<FeeDistributor> {
        self.fee_distributor.clone()
    }
    
    /// Get transaction pool statistics.
    /// Returns counts of pending transactions per lane and total fees.
    pub async fn pool_stats(&self) -> (usize, usize, usize, u64) {
        let pool = self.transaction_pool.read().await;
        let stats = pool.stats();
        (stats.critical_pending, stats.high_pending, stats.standard_pending, stats.total_fees_pending)
    }
    
    /// Get detailed pool statistics
    pub async fn get_pool_stats(&self) -> PoolStats {
        let pool = self.transaction_pool.read().await;
        pool.stats()
    }
}
