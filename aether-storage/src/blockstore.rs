//! Block store - persistent block storage with slot-based indexing
//!
//! Provides efficient block storage and retrieval with:
//! - Slot-based indexing for fast height lookups
//! - Hash-based indexing for fast block hash lookups
//! - Append-only design for write optimization
//! - Pruning support for old blocks

use aether_core::{Block, BlockHeader, Hash};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Maximum number of blocks to keep in memory before pruning
const DEFAULT_MAX_SLOTS: u64 = 500_000;

/// Block store error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum BlockStoreError {
    #[error("Block not found at slot {0}")]
    SlotNotFound(u64),
    #[error("Block not found with hash {0:?}")]
    HashNotFound(Hash),
    #[error("Slot {0} already has a block")]
    SlotOccupied(u64),
    #[error("Invalid block height: expected {expected}, got {actual}")]
    InvalidHeight { expected: u64, actual: u64 },
    #[error("Block store is full (max slots: {0})")]
    StoreFull(u64),
    #[error("Invalid parent hash at slot {0}")]
    InvalidParent(u64),
}

/// Metadata for a stored block
#[derive(Debug, Clone)]
pub struct BlockMeta {
    /// Block height (slot number)
    pub slot: u64,
    /// Block hash
    pub hash: Hash,
    /// Parent block hash
    pub parent_hash: Hash,
    /// Number of transactions
    pub num_transactions: u64,
    /// Timestamp when block was stored
    pub stored_at: u64,
    /// Block size in bytes (approximate)
    pub size_bytes: u64,
}

/// Block store with slot-based and hash-based indexing
pub struct BlockStore {
    /// Blocks indexed by slot
    blocks: Arc<RwLock<HashMap<u64, Block>>>,
    /// Block metadata indexed by slot
    meta: Arc<RwLock<HashMap<u64, BlockMeta>>>,
    /// Slot-to-hash index
    slot_to_hash: Arc<RwLock<HashMap<u64, Hash>>>,
    /// Hash-to-slot reverse index
    hash_to_slot: Arc<RwLock<HashMap<Hash, u64>>>,
    /// Maximum slots to retain (0 = unlimited)
    max_slots: u64,
    /// First stored slot
    first_slot: Arc<RwLock<Option<u64>>>,
    /// Last stored slot
    last_slot: Arc<RwLock<Option<u64>>>,
    /// Total number of blocks stored
    total_blocks: Arc<RwLock<u64>>,
}

impl BlockStore {
    /// Create a new block store with default capacity
    pub fn new() -> Self {
        Self::with_max_slots(DEFAULT_MAX_SLOTS)
    }

    /// Create a new block store with custom max slots
    pub fn with_max_slots(max_slots: u64) -> Self {
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
            meta: Arc::new(RwLock::new(HashMap::new())),
            slot_to_hash: Arc::new(RwLock::new(HashMap::new())),
            hash_to_slot: Arc::new(RwLock::new(HashMap::new())),
            max_slots,
            first_slot: Arc::new(RwLock::new(None)),
            last_slot: Arc::new(RwLock::new(None)),
            total_blocks: Arc::new(RwLock::new(0)),
        }
    }

    /// Insert a block into the store
    pub async fn insert(&self, block: Block) -> Result<(), BlockStoreError> {
        let slot = block.header.height;
        let hash = block.header.poh_hash;
        let num_txs = block.transactions.len() as u64;

        // Check if slot is already occupied
        {
            let blocks = self.blocks.read().await;
            if blocks.contains_key(&slot) {
                return Err(BlockStoreError::SlotOccupied(slot));
            }
        }

        // Validate parent hash continuity (except genesis)
        if slot > 0 {
            let last_slot = self.last_slot.read().await;
            if let Some(&last) = last_slot.as_ref() {
                let blocks = self.blocks.read().await;
                if let Some(parent_block) = blocks.get(&last) {
                    if parent_block.header.poh_hash != block.header.prev_hash {
                        return Err(BlockStoreError::InvalidParent(slot));
                    }
                }
            }
        }

        // Compute approximate block size
        let size_bytes = std::mem::size_of::<BlockHeader>() as u64
            + block.transactions.iter().map(|tx| tx.data.len() as u64 + 128).sum::<u64>();

        // Create metadata
        let meta = BlockMeta {
            slot,
            hash,
            parent_hash: block.header.prev_hash,
            num_transactions: num_txs,
            stored_at: current_timestamp(),
            size_bytes,
        };

        // Insert into all indexes
        {
            let mut blocks = self.blocks.write().await;
            blocks.insert(slot, block);
        }
        {
            let mut meta_map = self.meta.write().await;
            meta_map.insert(slot, meta);
        }
        {
            let mut s2h = self.slot_to_hash.write().await;
            s2h.insert(slot, hash);
        }
        {
            let mut h2s = self.hash_to_slot.write().await;
            h2s.insert(hash, slot);
        }

        // Update slot tracking
        {
            let mut first_slot = self.first_slot.write().await;
            if first_slot.is_none() || slot < first_slot.unwrap() {
                *first_slot = Some(slot);
            }
        }
        {
            let mut last_slot = self.last_slot.write().await;
            if last_slot.is_none() || slot > last_slot.unwrap() {
                *last_slot = Some(slot);
            }
        }
        {
            let mut total = self.total_blocks.write().await;
            *total += 1;
        }

        // Prune old blocks if over limit
        self.prune_old_blocks().await;

        debug!("Stored block at slot {} ({} txs, {} bytes)", slot, num_txs, size_bytes);
        Ok(())
    }

    /// Get a block by slot number
    pub async fn get_by_slot(&self, slot: u64) -> Result<Block, BlockStoreError> {
        let blocks = self.blocks.read().await;
        blocks.get(&slot).cloned().ok_or(BlockStoreError::SlotNotFound(slot))
    }

    /// Get a block by its hash
    pub async fn get_by_hash(&self, hash: &Hash) -> Result<Block, BlockStoreError> {
        let slot = {
            let h2s = self.hash_to_slot.read().await;
            *h2s.get(hash).ok_or(BlockStoreError::HashNotFound(hash.clone()))?
        };

        self.get_by_slot(slot).await
    }

    /// Get block metadata by slot
    pub async fn get_meta(&self, slot: u64) -> Result<BlockMeta, BlockStoreError> {
        let meta = self.meta.read().await;
        meta.get(&slot).cloned().ok_or(BlockStoreError::SlotNotFound(slot))
    }

    /// Get the last (highest) slot
    pub async fn last_slot(&self) -> Option<u64> {
        *self.last_slot.read().await
    }

    /// Get the first (lowest) slot
    pub async fn first_slot(&self) -> Option<u64> {
        *self.first_slot.read().await
    }

    /// Get total number of blocks stored
    pub async fn total_blocks(&self) -> u64 {
        *self.total_blocks.read().await
    }

    /// Check if a slot exists
    pub async fn contains_slot(&self, slot: u64) -> bool {
        self.blocks.read().await.contains_key(&slot)
    }

    /// Get a range of blocks [start, end]
    pub async fn get_range(&self, start: u64, end: u64) -> Vec<Block> {
        let blocks = self.blocks.read().await;
        let mut result = Vec::new();
        for slot in start..=end {
            if let Some(block) = blocks.get(&slot) {
                result.push(block.clone());
            }
        }
        result
    }

    /// Get block headers for a range of slots
    pub async fn get_headers_range(&self, start: u64, end: u64) -> Vec<BlockHeader> {
        let blocks = self.blocks.read().await;
        let mut result = Vec::new();
        for slot in start..=end {
            if let Some(block) = blocks.get(&slot) {
                result.push(block.header.clone());
            }
        }
        result
    }

    /// Prune old blocks when store exceeds max_slots
    async fn prune_old_blocks(&self) {
        if self.max_slots == 0 {
            return; // Unlimited storage
        }

        let total = *self.total_blocks.read().await;
        if total <= self.max_slots {
            return;
        }

        // Calculate how many to prune (prune 10% over limit at a time)
        let prune_count = total - self.max_slots + (self.max_slots / 10).max(1);

        let first_slot = {
            let first = self.first_slot.read().await;
            match *first {
                Some(s) => s,
                None => return,
            }
        };

        let prune_end = first_slot + prune_count - 1;

        info!("Pruning blocks from slot {} to {}", first_slot, prune_end);

        for slot in first_slot..=prune_end {
            // Remove from hash index first
            if let Some(hash) = self.slot_to_hash.read().await.get(&slot).copied() {
                self.hash_to_slot.write().await.remove(&hash);
            }
            self.slot_to_hash.write().await.remove(&slot);
            self.blocks.write().await.remove(&slot);
            self.meta.write().await.remove(&slot);
        }

        // Update first slot
        *self.first_slot.write().await = Some(prune_end + 1);
        *self.total_blocks.write().await -= prune_count;
    }

    /// Get store statistics
    pub async fn stats(&self) -> BlockStoreStats {
        let blocks = self.blocks.read().await;
        let total_txs: u64 = blocks.values().map(|b| b.transactions.len() as u64).sum();
        let total_size: u64 = self.meta.read().await.values().map(|m| m.size_bytes).sum();

        BlockStoreStats {
            total_blocks: *self.total_blocks.read().await,
            first_slot: *self.first_slot.read().await,
            last_slot: *self.last_slot.read().await,
            total_transactions: total_txs,
            total_size_bytes: total_size,
        }
    }
}

impl Default for BlockStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Block store statistics
#[derive(Debug, Clone)]
pub struct BlockStoreStats {
    pub total_blocks: u64,
    pub first_slot: Option<u64>,
    pub last_slot: Option<u64>,
    pub total_transactions: u64,
    pub total_size_bytes: u64,
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

    fn make_block(height: u64, prev_hash: Hash, poh_hash: Hash) -> Block {
        Block {
            header: BlockHeader {
                height,
                prev_hash,
                timestamp: height * 400,
                poh_hash,
                state_root: Hash::ZERO,
            },
            transactions: vec![],
        }
    }

    fn hash_seq(start: u8) -> Hash {
        let mut h = [0u8; 32];
        h[0] = start;
        Hash::new(h)
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let store = BlockStore::new();
        let block = make_block(0, Hash::ZERO, hash_seq(1));
        store.insert(block).await.unwrap();

        let retrieved = store.get_by_slot(0).await.unwrap();
        assert_eq!(retrieved.header.height, 0);
    }

    #[tokio::test]
    async fn test_slot_occupied() {
        let store = BlockStore::new();
        let block = make_block(0, Hash::ZERO, hash_seq(1));
        store.insert(block).await.unwrap();

        let dup = make_block(0, Hash::ZERO, hash_seq(2));
        assert!(matches!(
            store.insert(dup).await,
            Err(BlockStoreError::SlotOccupied(0))
        ));
    }

    #[tokio::test]
    async fn test_get_by_hash() {
        let store = BlockStore::new();
        let hash = hash_seq(42);
        let block = make_block(0, Hash::ZERO, hash);
        store.insert(block).await.unwrap();

        let retrieved = store.get_by_hash(&hash).await.unwrap();
        assert_eq!(retrieved.header.height, 0);
    }

    #[tokio::test]
    async fn test_range_query() {
        let store = BlockStore::new();
        for i in 0..5u64 {
            let prev = if i == 0 { Hash::ZERO } else { hash_seq(i as u8) };
            let block = make_block(i, prev, hash_seq((i + 1) as u8));
            store.insert(block).await.unwrap();
        }

        let range = store.get_range(1, 3).await;
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].header.height, 1);
        assert_eq!(range[2].header.height, 3);
    }

    #[tokio::test]
    async fn test_pruning() {
        let store = BlockStore::with_max_slots(3);
        for i in 0..10u64 {
            let prev = if i == 0 { Hash::ZERO } else { hash_seq(i as u8) };
            let block = make_block(i, prev, hash_seq((i + 1) as u8));
            store.insert(block).await.unwrap();
        }

        // After pruning, first slots should be gone
        assert!(!store.contains_slot(0).await);
        // Later slots should still exist
        assert!(store.contains_slot(9).await);
    }

    #[tokio::test]
    async fn test_stats() {
        let store = BlockStore::new();
        let block = make_block(0, Hash::ZERO, hash_seq(1));
        store.insert(block).await.unwrap();

        let stats = store.stats().await;
        assert_eq!(stats.total_blocks, 1);
        assert_eq!(stats.first_slot, Some(0));
        assert_eq!(stats.last_slot, Some(0));
    }
}