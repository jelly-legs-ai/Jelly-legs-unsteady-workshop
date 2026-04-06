//! AetherFlow Consensus Engine
//!
//! Main consensus engine that combines PoH + PoS with AI priority lanes.

use crate::{ConsensusError, ConsensusResult};
use crate::poh::{PoHGenerator, PoHEntry, verify_poh_chain};
use crate::pos::{StakePool, LeaderSchedule, ValidatorStake};
use aether_common::{AIPriorityLane, AITransactionMeta, SLOT_TIME_MS, SignatureBytes};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

/// AetherFlow block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherBlockHeader {
    /// Slot number
    pub slot: u64,
    /// PoH hash at this slot
    pub poh_hash: [u8; 32],
    /// Previous block hash
    pub parent_hash: [u8; 32],
    /// Block producer pubkey
    pub producer: [u8; 32],
    /// Block timestamp
    pub timestamp: u64,
    /// State root after executing this block
    pub state_root: [u8; 32],
    /// Transaction merkle root
    pub tx_root: [u8; 32],
    /// Number of transactions
    pub tx_count: u32,
    /// AI priority metadata
    pub ai_meta: BlockAIMeta,
}

/// AI metadata for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockAIMeta {
    /// Count of transactions per lane
    pub critical_tx_count: u32,
    pub high_tx_count: u32,
    pub standard_tx_count: u32,
    /// Average priority fees
    pub avg_critical_fee: u64,
    pub avg_high_fee: u64,
    pub avg_standard_fee: u64,
}

impl Default for BlockAIMeta {
    fn default() -> Self {
        Self {
            critical_tx_count: 0,
            high_tx_count: 0,
            standard_tx_count: 0,
            avg_critical_fee: 0,
            avg_high_fee: 0,
            avg_standard_fee: 0,
        }
    }
}

/// AetherFlow block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherBlock {
    /// Block header
    pub header: AetherBlockHeader,
    /// PoH entries for this block
    pub poh_entries: Vec<PoHEntry>,
    /// Transactions in priority order
    pub transactions: Vec<AetherTransaction>,
    /// Block signature
    pub signature: SignatureBytes,
}

/// Transaction with AETHER metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherTransaction {
    /// Transaction data
    pub data: Vec<u8>,
    /// AI priority metadata
    pub ai_meta: AITransactionMeta,
    /// Sender signature
    pub signature: SignatureBytes,
    /// Compute units consumed
    pub compute_units_consumed: u64,
}

/// AI Priority Queue for transaction ordering
#[derive(Debug)]
pub struct AIPriorityQueue {
    /// Critical priority transactions
    critical: VecDeque<AetherTransaction>,
    /// High priority transactions
    high: VecDeque<AetherTransaction>,
    /// Standard transactions
    standard: VecDeque<AetherTransaction>,
    /// Max transactions per block
    max_tx_per_block: usize,
    /// Max compute units per block
    max_compute_per_block: u64,
}

impl AIPriorityQueue {
    pub fn new(max_tx_per_block: usize, max_compute_per_block: u64) -> Self {
        Self {
            critical: VecDeque::new(),
            high: VecDeque::new(),
            standard: VecDeque::new(),
            max_tx_per_block,
            max_compute_per_block,
        }
    }

    /// Add transaction to appropriate lane
    pub fn push(&mut self, tx: AetherTransaction) {
        match tx.ai_meta.lane {
            AIPriorityLane::Critical => self.critical.push_back(tx),
            AIPriorityLane::High => self.high.push_back(tx),
            AIPriorityLane::Standard => self.standard.push_back(tx),
        }
    }

    /// Get transactions for next block
    pub fn get_block_transactions(&mut self) -> Vec<AetherTransaction> {
        let mut result = Vec::new();
        let mut total_compute: u64 = 0;

        // Take lanes out to avoid borrow conflicts
        let mut critical_lane = VecDeque::new();
        std::mem::swap(&mut critical_lane, &mut self.critical);
        let mut high_lane = VecDeque::new();
        std::mem::swap(&mut high_lane, &mut self.high);
        let mut standard_lane = VecDeque::new();
        std::mem::swap(&mut standard_lane, &mut self.standard);

        // Critical lane gets 40% of block space
        let critical_limit = self.max_tx_per_block * 4 / 10;
        self.drain_lane(&mut result, &mut total_compute, &mut critical_lane, critical_limit);

        // High lane gets 30% of block space
        let high_limit = self.max_tx_per_block * 3 / 10;
        self.drain_lane(&mut result, &mut total_compute, &mut high_lane, high_limit);

        // Standard lane gets remaining 30%
        let remaining = self.max_tx_per_block - result.len();
        self.drain_lane(&mut result, &mut total_compute, &mut standard_lane, remaining);

        // Put remaining transactions back
        std::mem::swap(&mut self.critical, &mut critical_lane);
        std::mem::swap(&mut self.high, &mut high_lane);
        std::mem::swap(&mut self.standard, &mut standard_lane);

        result
    }

    fn drain_lane(
        &mut self,
        result: &mut Vec<AetherTransaction>,
        total_compute: &mut u64,
        lane: &mut VecDeque<AetherTransaction>,
        limit: usize,
    ) {
        while result.len() < limit && !lane.is_empty() {
            let tx = lane.pop_front().unwrap();
            
            // Check compute limit
            if *total_compute + tx.ai_meta.compute_units <= self.max_compute_per_block {
                *total_compute += tx.ai_meta.compute_units;
                result.push(tx);
            } else {
                // Put back if we're over compute limit
                lane.push_front(tx);
                break;
            }
        }
    }

    /// Get queue stats
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            critical_pending: self.critical.len(),
            high_pending: self.high.len(),
            standard_pending: self.standard.len(),
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Copy)]
pub struct QueueStats {
    pub critical_pending: usize,
    pub high_pending: usize,
    pub standard_pending: usize,
}

/// AetherFlow consensus engine
pub struct AetherFlow {
    /// PoH generator
    poh: PoHGenerator,
    /// Stake pool
    stake_pool: StakePool,
    /// Current leader schedule
    leader_schedule: Option<LeaderSchedule>,
    /// Transaction queue
    tx_queue: Arc<RwLock<AIPriorityQueue>>,
    /// Current slot
    current_slot: u64,
    /// Block height
    block_height: u64,
    /// Block history (slot -> block)
    blocks: BTreeMap<u64, AetherBlock>,
    /// Current epoch
    current_epoch: u64,
}

impl AetherFlow {
    /// Create new AetherFlow consensus engine
    pub fn new() -> Self {
        Self {
            poh: PoHGenerator::new(),
            stake_pool: StakePool::new(),
            leader_schedule: None,
            tx_queue: Arc::new(RwLock::new(AIPriorityQueue::new(10_000, 48_000_000))),
            current_slot: 0,
            block_height: 0,
            blocks: BTreeMap::new(),
            current_epoch: 0,
        }
    }

    /// Initialize with genesis block
    pub fn initialize_genesis(&mut self) -> ConsensusResult<AetherBlock> {
        let genesis_entry = PoHEntry::genesis();
        let mut poh = PoHGenerator::from_hash(genesis_entry.hash);
        let poh_entry = poh.tick();

        let header = AetherBlockHeader {
            slot: 0,
            poh_hash: poh_entry.hash,
            parent_hash: [0u8; 32],
            producer: [0u8; 32],
            timestamp: 0,
            state_root: [0u8; 32],
            tx_root: [0u8; 32],
            tx_count: 0,
            ai_meta: BlockAIMeta::default(),
        };

        let block = AetherBlock {
            header,
            poh_entries: vec![genesis_entry, poh_entry],
            transactions: vec![],
            signature: SignatureBytes([0u8; 64]),
        };

        self.blocks.insert(0, block.clone());
        self.poh = poh;
        
        Ok(block)
    }

    /// Add validator to stake pool
    pub fn add_validator(&mut self, validator: ValidatorStake) {
        self.stake_pool.update_stake(validator);
        self.update_leader_schedule();
    }

    /// Update leader schedule for current epoch
    fn update_leader_schedule(&mut self) {
        if self.stake_pool.block_producers().is_empty() {
            return;
        }

        let schedule = LeaderSchedule::generate(
            &self.stake_pool,
            self.current_epoch,
            crate::constants::SLOTS_PER_EPOCH,
        );
        self.leader_schedule = Some(schedule);
    }

    /// Get leader for current slot
    pub fn get_slot_leader(&self, slot: u64) -> Option<[u8; 32]> {
        self.leader_schedule.as_ref()
            .and_then(|s| s.get_leader(slot))
            .copied()
    }

    /// Submit transaction to queue
    pub fn submit_transaction(&self, tx: AetherTransaction) -> ConsensusResult<()> {
        let mut queue = self.tx_queue.write().map_err(|_| {
            ConsensusError::AIPriorityError("Queue lock poisoned".to_string())
        })?;
        queue.push(tx);
        Ok(())
    }

    /// Produce a new block
    pub fn produce_block(
        &mut self,
        producer: [u8; 32],
    ) -> ConsensusResult<AetherBlock> {
        // Verify we're the leader
        let expected_leader = self.get_slot_leader(self.current_slot)
            .ok_or_else(|| ConsensusError::InvalidBlockProducer {
                expected: [0u8; 32],
                actual: producer,
            })?;

        if expected_leader != producer {
            return Err(ConsensusError::InvalidBlockProducer {
                expected: expected_leader,
                actual: producer,
            });
        }

        // Get transactions from queue
        let transactions = {
            let mut queue = self.tx_queue.write().map_err(|_| {
                ConsensusError::AIPriorityError("Queue lock poisoned".to_string())
            })?;
            queue.get_block_transactions()
        };

        // Calculate AI metadata
        let ai_meta = self.calculate_block_ai_meta(&transactions);

        // Generate PoH entries
        let poh_entry = self.poh.tick();
        let poh_hash = poh_entry.hash;

        // Calculate transaction root
        let tx_root = self.calculate_tx_root(&transactions);

        // Create parent hash
        let parent_hash = self.blocks.get(&self.current_slot.saturating_sub(1))
            .map(|b| b.header.poh_hash)
            .unwrap_or([0u8; 32]);

        // Create block header
        let header = AetherBlockHeader {
            slot: self.current_slot,
            poh_hash,
            parent_hash,
            producer,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            state_root: [0u8; 32], // Would calculate from execution
            tx_root,
            tx_count: transactions.len() as u32,
            ai_meta,
        };

        // Sign block (simplified - would use actual keypair)
        let signature = SignatureBytes([0u8; 64]);

        let block = AetherBlock {
            header,
            poh_entries: vec![poh_entry],
            transactions,
            signature,
        };

        // Store block
        self.blocks.insert(self.current_slot, block.clone());
        self.current_slot += 1;
        self.block_height += 1;

        // Check for epoch transition
        if self.current_slot % crate::constants::SLOTS_PER_EPOCH == 0 {
            self.current_epoch += 1;
            self.update_leader_schedule();
        }

        Ok(block)
    }

    /// Calculate AI metadata for block
    fn calculate_block_ai_meta(&self, transactions: &[AetherTransaction]) -> BlockAIMeta {
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut standard_count = 0;
        let mut critical_fees = 0u64;
        let mut high_fees = 0u64;
        let mut standard_fees = 0u64;

        for tx in transactions {
            match tx.ai_meta.lane {
                AIPriorityLane::Critical => {
                    critical_count += 1;
                    critical_fees += tx.ai_meta.priority_fee;
                }
                AIPriorityLane::High => {
                    high_count += 1;
                    high_fees += tx.ai_meta.priority_fee;
                }
                AIPriorityLane::Standard => {
                    standard_count += 1;
                    standard_fees += tx.ai_meta.priority_fee;
                }
            }
        }

        BlockAIMeta {
            critical_tx_count: critical_count,
            high_tx_count: high_count,
            standard_tx_count: standard_count,
            avg_critical_fee: if critical_count > 0 { critical_fees / critical_count as u64 } else { 0 },
            avg_high_fee: if high_count > 0 { high_fees / high_count as u64 } else { 0 },
            avg_standard_fee: if standard_count > 0 { standard_fees / standard_count as u64 } else { 0 },
        }
    }

    /// Calculate transaction merkle root
    fn calculate_tx_root(&self, transactions: &[AetherTransaction]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        if transactions.is_empty() {
            return [0u8; 32];
        }

        let leaves: Vec<[u8; 32]> = transactions.iter()
            .map(|tx| {
                let mut hasher = Sha256::new();
                hasher.update(&tx.data);
                hasher.finalize().into()
            })
            .collect();

        aether_common::crypto::calculate_merkle_root(&leaves)
    }

    /// Verify a block
    pub fn verify_block(&self, block: &AetherBlock) -> ConsensusResult<bool> {
        // Verify PoH chain
        if !verify_poh_chain(&block.poh_entries) {
            return Ok(false);
        }

        // Verify producer is valid leader
        if let Some(leader) = self.get_slot_leader(block.header.slot) {
            if leader != block.header.producer {
                return Ok(false);
            }
        }

        // Verify transactions are properly ordered by priority
        // Lane priority: Critical(0) > High(1) > Standard(2)
        // Once we move to a lower priority lane, we cannot go back to higher
        let mut last_lane: Option<AIPriorityLane> = None;
        for tx in &block.transactions {
            // Reject if a higher-priority lane appears after a lower-priority one
            // i.e., lane number decreased (moved up in priority)
            if let Some(prev_lane) = last_lane {
                if (tx.ai_meta.lane as u8) < (prev_lane as u8) {
                    return Ok(false);
                }
            }
            last_lane = Some(tx.ai_meta.lane);
        }

        Ok(true)
    }

    /// Get block by slot
    pub fn get_block(&self, slot: u64) -> Option<&AetherBlock> {
        self.blocks.get(&slot)
    }

    /// Get current height
    pub fn block_height(&self) -> u64 {
        self.block_height
    }

    /// Get current slot
    pub fn current_slot(&self) -> u64 {
        self.current_slot
    }

    /// Get stake pool
    pub fn stake_pool(&self) -> &StakePool {
        &self.stake_pool
    }

    /// Get leader schedule
    pub fn get_leader_schedule(&self) -> Option<&LeaderSchedule> {
        self.leader_schedule.as_ref()
    }

    /// Get current epoch
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Get queue stats
    pub fn queue_stats(&self) -> ConsensusResult<QueueStats> {
        let queue = self.tx_queue.read().map_err(|_| {
            ConsensusError::AIPriorityError("Queue lock poisoned".to_string())
        })?;
        Ok(queue.stats())
    }
}

impl Default for AetherFlow {
    fn default() -> Self {
        Self::new()
    }
}

/// Constants module
pub mod constants {
    use super::*;
    
    /// Slots per epoch
    pub const SLOTS_PER_EPOCH: u64 = 432_000;
    
    /// Target slot time
    pub const TARGET_SLOT_TIME_MS: u64 = SLOT_TIME_MS;
    
    /// Max transactions per block
    pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 10_000;
    
    /// Max compute units per block
    pub const MAX_COMPUTE_UNITS_PER_BLOCK: u64 = 48_000_000;
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_common::types::*;

    fn create_test_transaction(lane: AIPriorityLane, fee: u64) -> AetherTransaction {
        AetherTransaction {
            data: vec![1, 2, 3],
            ai_meta: AITransactionMeta {
                lane,
                ai_signature: None,
                compute_units: 200_000,
                priority_fee: fee,
            },
            signature: SignatureBytes([0u8; 64]),
            compute_units_consumed: 200_000,
        }
    }

    #[test]
    fn test_ai_priority_queue_ordering() {
        let mut queue = AIPriorityQueue::new(100, 48_000_000);

        // Add transactions in reverse priority order
        queue.push(create_test_transaction(AIPriorityLane::Standard, 100));
        queue.push(create_test_transaction(AIPriorityLane::High, 500));
        queue.push(create_test_transaction(AIPriorityLane::Critical, 1000));
        queue.push(create_test_transaction(AIPriorityLane::Standard, 200));
        queue.push(create_test_transaction(AIPriorityLane::Critical, 2000));

        // Get block transactions - should be ordered by priority
        let block_txs = queue.get_block_transactions();

        // Critical should come first
        assert_eq!(block_txs[0].ai_meta.lane, AIPriorityLane::Critical);
        assert_eq!(block_txs[1].ai_meta.lane, AIPriorityLane::Critical);
        
        // Then High
        assert_eq!(block_txs[2].ai_meta.lane, AIPriorityLane::High);
        
        // Then Standard
        assert_eq!(block_txs[3].ai_meta.lane, AIPriorityLane::Standard);
        assert_eq!(block_txs[4].ai_meta.lane, AIPriorityLane::Standard);
    }

    #[test]
    fn test_aetherflow_genesis() {
        let mut flow = AetherFlow::new();
        let genesis = flow.initialize_genesis().unwrap();

        assert_eq!(genesis.header.slot, 0);
        assert_eq!(genesis.header.tx_count, 0);
        assert!(genesis.transactions.is_empty());
    }

    #[test]
    fn test_block_ai_meta_calculation() {
        let mut flow = AetherFlow::new();
        let _ = flow.initialize_genesis();

        let txs = vec![
            create_test_transaction(AIPriorityLane::Critical, 1000),
            create_test_transaction(AIPriorityLane::Critical, 2000),
            create_test_transaction(AIPriorityLane::High, 500),
            create_test_transaction(AIPriorityLane::Standard, 100),
            create_test_transaction(AIPriorityLane::Standard, 200),
        ];

        let meta = flow.calculate_block_ai_meta(&txs);

        assert_eq!(meta.critical_tx_count, 2);
        assert_eq!(meta.high_tx_count, 1);
        assert_eq!(meta.standard_tx_count, 2);
        assert_eq!(meta.avg_critical_fee, 1500); // (1000 + 2000) / 2
        assert_eq!(meta.avg_high_fee, 500);
        assert_eq!(meta.avg_standard_fee, 150); // (100 + 200) / 2
    }
}
