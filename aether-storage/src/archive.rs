//! Archival storage - cold storage and snapshot management for AETHER
//!
//! Provides:
//! - Block archival to cold storage
//! - State snapshots for fast sync
//! - Genesis state management
//! - Data availability guarantees

use aether_core::{Block, Hash};
use crate::state::{StateManager, AccountState};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Archive error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum ArchiveError {
    #[error("Snapshot not found at slot {0}")]
    SnapshotNotFound(u64),
    #[error("Archive already exists at slot {0}")]
    ArchiveExists(u64),
    #[error("Insufficient data for archive at slot {0}")]
    InsufficientData(u64),
}

/// Archived block data
#[derive(Debug, Clone)]
pub struct ArchivedBlock {
    /// Slot number
    pub slot: u64,
    /// Compressed block data hash
    pub hash: Hash,
    /// Timestamp of archival
    pub archived_at: u64,
    /// Number of transactions in block
    pub transaction_count: u64,
}

/// State archive at a specific slot
#[derive(Debug, Clone)]
pub struct StateArchive {
    /// Slot number
    pub slot: u64,
    /// State root hash
    pub state_root: Hash,
    /// Total supply at this slot
    pub total_supply: u64,
    /// Number of accounts
    pub account_count: usize,
    /// Timestamp of archival
    pub archived_at: u64,
}

/// Archive manager for cold storage and snapshots
pub struct Archive {
    /// Archived blocks (cold storage index)
    block_archives: Arc<RwLock<HashMap<u64, ArchivedBlock>>>,
    /// State archives
    state_archives: Arc<RwLock<HashMap<u64, StateArchive>>>,
    /// Genesis state
    genesis_accounts: Arc<RwLock<HashMap<Hash, AccountState>>>,
    /// Archive interval (archive every N slots)
    archive_interval: u64,
    /// Last archived slot
    last_archived_slot: Arc<RwLock<Option<u64>>>,
}

impl Archive {
    /// Create a new archive manager
    pub fn new() -> Self {
        Self::with_interval(100) // Default: archive every 100 slots
    }

    /// Create archive manager with custom interval
    pub fn with_interval(archive_interval: u64) -> Self {
        Self {
            block_archives: Arc::new(RwLock::new(HashMap::new())),
            state_archives: Arc::new(RwLock::new(HashMap::new())),
            genesis_accounts: Arc::new(RwLock::new(HashMap::new())),
            archive_interval,
            last_archived_slot: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize genesis accounts
    pub async fn init_genesis(&self, accounts: Vec<AccountState>) {
        let mut genesis = self.genesis_accounts.write().await;
        for account in accounts {
            genesis.insert(account.hash(), account);
        }
        info!("Initialized genesis with {} accounts", genesis.len());
    }

    /// Archive a block
    pub async fn archive_block(&self, block: &Block) -> Result<(), ArchiveError> {
        let slot = block.header.height;

        let archived = ArchivedBlock {
            slot,
            hash: block.header.poh_hash.clone(),
            archived_at: current_timestamp(),
            transaction_count: block.transactions.len() as u64,
        };

        let mut archives = self.block_archives.write().await;
        archives.insert(slot, archived);

        *self.last_archived_slot.write().await = Some(slot);
        debug!("Archived block at slot {}", slot);
        Ok(())
    }

    /// Create a state archive from current state
    pub async fn archive_state(&self, state_manager: &StateManager, slot: u64) -> Result<StateArchive, ArchiveError> {
        let stats = state_manager.stats().await;

        let archive = StateArchive {
            slot,
            state_root: stats.state_root,
            total_supply: stats.total_supply,
            account_count: stats.account_count,
            archived_at: current_timestamp(),
        };

        let mut archives = self.state_archives.write().await;
        archives.insert(slot, archive.clone());

        info!("Archived state at slot {} ({} accounts)", slot, stats.account_count);
        Ok(archive)
    }

    /// Get archived block info
    pub async fn get_archived_block(&self, slot: u64) -> Option<ArchivedBlock> {
        self.block_archives.read().await.get(&slot).cloned()
    }

    /// Get state archive at slot
    pub async fn get_state_archive(&self, slot: u64) -> Option<StateArchive> {
        self.state_archives.read().await.get(&slot).cloned()
    }

    /// Get the closest state archive at or before a given slot
    pub async fn get_closest_state_archive(&self, slot: u64) -> Option<StateArchive> {
        let archives = self.state_archives.read().await;
        archives
            .iter()
            .filter(|(s, _)| **s <= slot)
            .max_by_key(|(s, _)| *s)
            .map(|(_, v)| v.clone())
    }

    /// Check if a slot should be archived based on interval
    pub fn should_archive(&self, slot: u64) -> bool {
        slot % self.archive_interval == 0
    }

    /// Get total archived blocks count
    pub async fn archived_block_count(&self) -> usize {
        self.block_archives.read().await.len()
    }

    /// Get total state archives count
    pub async fn state_archive_count(&self) -> usize {
        self.state_archives.read().await.len()
    }

    /// Get last archived slot
    pub async fn last_archived_slot(&self) -> Option<u64> {
        *self.last_archived_slot.read().await
    }

    /// Prune state archives older than a given slot
    pub async fn prune_state_archives(&self, before_slot: u64) -> usize {
        let mut archives = self.state_archives.write().await;
        let before_count = archives.len();
        archives.retain(|slot, _| *slot >= before_slot);
        before_count - archives.len()
    }

    /// Get archive statistics
    pub async fn stats(&self) -> ArchiveStats {
        ArchiveStats {
            archived_blocks: self.block_archives.read().await.len(),
            state_archives: self.state_archives.read().await.len(),
            genesis_accounts: self.genesis_accounts.read().await.len(),
            last_archived_slot: *self.last_archived_slot.read().await,
            archive_interval: self.archive_interval,
        }
    }
}

impl Default for Archive {
    fn default() -> Self {
        Self::new()
    }
}

/// Archive statistics
#[derive(Debug, Clone)]
pub struct ArchiveStats {
    pub archived_blocks: usize,
    pub state_archives: usize,
    pub genesis_accounts: usize,
    pub last_archived_slot: Option<u64>,
    pub archive_interval: u64,
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
    use aether_core::{Block, BlockHeader, Hash};

    fn make_block(height: u64) -> Block {
        Block {
            header: BlockHeader {
                height,
                prev_hash: Hash::ZERO,
                timestamp: height * 400,
                poh_hash: Hash::new([height as u8; 32]),
                state_root: Hash::ZERO,
            },
            transactions: vec![],
        }
    }

    #[tokio::test]
    async fn test_archive_block() {
        let archive = Archive::new();
        let block = make_block(100);

        archive.archive_block(&block).await.unwrap();
        let archived = archive.get_archived_block(100).await.unwrap();
        assert_eq!(archived.slot, 100);
    }

    #[tokio::test]
    async fn test_archive_state() {
        let archive = Archive::new();
        let state = StateManager::new();

        let result = archive.archive_state(&state, 100).await.unwrap();
        assert_eq!(result.slot, 100);
        assert_eq!(result.account_count, 0);
    }

    #[tokio::test]
    async fn test_should_archive() {
        let archive = Archive::with_interval(100);

        assert!(archive.should_archive(0));
        assert!(archive.should_archive(100));
        assert!(archive.should_archive(200));
        assert!(!archive.should_archive(50));
        assert!(!archive.should_archive(150));
    }

    #[tokio::test]
    async fn test_closest_state_archive() {
        let archive = Archive::new();
        let state = StateManager::new();

        archive.archive_state(&state, 100).await.unwrap();
        archive.archive_state(&state, 200).await.unwrap();

        let closest = archive.get_closest_state_archive(150).await.unwrap();
        assert_eq!(closest.slot, 100);

        let closest = archive.get_closest_state_archive(250).await.unwrap();
        assert_eq!(closest.slot, 200);
    }
}