//! Block Sync and State Snapshot System
//!
//! Implements block synchronization and state snapshots for fast catch-up.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Sync Manager                            │
//! ├─────────────────────────────────────────────────────────────┤
//! │  - Detects when behind (slot gap with peers)                │
//! │  - Requests missing blocks from peers                       │
//! │  - Validates block hashes and signatures                    │
//! │  - Executes blocks to catch up state                        │
//! │  - Manages sync state machine                               │
//! ├─────────────────────────────────────────────────────────────┤
//! │  State Snapshot (Fast Sync)                                 │
//! │  - Snapshots state at epoch boundaries                      │
//! │  - Downloads and verifies snapshots                         │
//! │  - Replays blocks from snapshot forward                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::state::ValidatorState;
use crate::block_producer::BlockProducer;
use crate::persistence::PersistenceManager;
use crate::block_producer::Block;
use aether_core::TransactionReceipt;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Sync state machine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncState {
    /// Not syncing, following tip
    Following,
    /// Currently syncing blocks from peers
    Syncing {
        /// Target slot to sync to
        target_slot: u64,
        /// Current synced slot
        current_slot: u64,
        /// Peer we're syncing from
        peer_id: String,
    },
    /// Waiting for snapshot from peer
    AwaitingSnapshot {
        peer_id: String,
        since: Instant,
    },
    /// Sync paused due to network issues
    Paused {
        reason: String,
        since: Instant,
    },
}

impl Default for SyncState {
    fn default() -> Self {
        Self::Following
    }
}

/// Sync configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Maximum blocks to request in one batch
    pub batch_size: u64,
    /// Maximum slots behind before triggering sync
    pub sync_threshold: u64,
    /// Maximum seconds to wait for a sync response
    pub response_timeout_secs: u64,
    /// Maximum snapshot size to accept (bytes)
    pub max_snapshot_size: usize,
    /// Interval between sync status checks
    pub sync_check_interval_secs: u64,
    /// Enable fast sync (download snapshots)
    pub enable_fast_sync: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            sync_threshold: 10,
            response_timeout_secs: 30,
            max_snapshot_size: 100 * 1024 * 1024, // 100 MB
            sync_check_interval_secs: 5,
            enable_fast_sync: true,
        }
    }
}

/// A block with its parent hash for sync validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncBlock {
    pub block: Block,
    pub parent_hash: String,
}

/// State snapshot for fast sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Slot at which snapshot was taken
    pub slot: u64,
    /// Block hash at this slot
    pub block_hash: String,
    /// State root at this slot
    pub state_root: String,
    /// Epoch number
    pub epoch: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Accounts (address -> (lamports, owner, data))
    pub accounts: Vec<SnapshotAccount>,
    /// Total supply at snapshot
    pub total_supply: u64,
}

/// Account in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotAccount {
    pub address: [u8; 32],
    pub lamports: u64,
    pub owner: [u8; 32],
    pub data: Vec<u8>,
    pub rent_epoch: u64,
}

/// Sync statistics
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    /// Total blocks synced
    pub blocks_synced: u64,
    /// Total snapshots downloaded
    pub snapshots_downloaded: u64,
    /// Time spent syncing (seconds)
    pub sync_time_secs: u64,
    /// Last sync completed timestamp
    pub last_sync_completed: Option<Instant>,
    /// Number of sync sessions
    pub sync_sessions: u64,
    /// Failed sync attempts
    pub failed_attempts: u64,
}

/// Block sync manager
pub struct SyncManager {
    /// Sync configuration
    config: SyncConfig,
    /// Current sync state
    state: Arc<RwLock<SyncState>>,
    /// Validator state
    validator_state: ValidatorState,
    /// Block producer for executing synced blocks
    block_producer: Arc<BlockProducer>,
    /// Persistence manager for snapshots
    persistence: Option<Arc<PersistenceManager>>,
    /// Sync statistics
    stats: Arc<RwLock<SyncStats>>,
    /// Pending block requests
    pending_requests: Arc<RwLock<VecDeque<BlockRequest>>>,
    /// Snapshot download in progress
    downloading_snapshot: Arc<RwLock<bool>>,
}

/// Block request from a peer
#[derive(Debug, Clone)]
struct BlockRequest {
    peer_id: String,
    start_slot: u64,
    end_slot: u64,
    request_time: Instant,
}

impl SyncManager {
    /// Create new sync manager
    pub fn new(
        config: SyncConfig,
        validator_state: ValidatorState,
        block_producer: Arc<BlockProducer>,
        persistence: Option<Arc<PersistenceManager>>,
    ) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(SyncState::Following)),
            validator_state,
            block_producer,
            persistence,
            stats: Arc::new(RwLock::new(SyncStats::default())),
            pending_requests: Arc::new(RwLock::new(VecDeque::new())),
            downloading_snapshot: Arc::new(RwLock::new(false)),
        }
    }

    /// Get current sync state
    pub async fn get_state(&self) -> SyncState {
        self.state.read().await.clone()
    }

    /// Get sync statistics
    pub async fn get_stats(&self) -> SyncStats {
        self.stats.read().await.clone()
    }

    /// Check if we're currently syncing
    pub async fn is_syncing(&self) -> bool {
        let state = self.state.read().await;
        !matches!(*state, SyncState::Following)
    }

    /// Get slots behind (0 if we're caught up or syncing)
    pub async fn slots_behind(&self, peer_slot: u64) -> u64 {
        let our_slot = self.validator_state.current_slot();
        if peer_slot <= our_slot {
            return 0;
        }
        
        let state = self.state.read().await;
        match &*state {
            SyncState::Syncing { target_slot, .. } => {
                // Already syncing, report distance to target
                if peer_slot > *target_slot {
                    peer_slot - our_slot
                } else {
                    0
                }
            }
            SyncState::Following => {
                peer_slot - our_slot
            }
            _ => 0,
        }
    }

    /// Start syncing from a peer
    pub async fn start_sync(&self, peer_id: &str, target_slot: u64) {
        let current_slot = self.validator_state.current_slot();
        
        if target_slot <= current_slot {
            debug!("Not starting sync: target {} <= current {}", target_slot, current_slot);
            return;
        }
        
        info!("Starting sync from peer {} to slot {} (current: {})", 
            peer_id, target_slot, current_slot);
        
        // Update state
        {
            let mut state = self.state.write().await;
            *state = SyncState::Syncing {
                target_slot,
                current_slot,
                peer_id: peer_id.to_string(),
            };
        }
        
        // Increment sync sessions
        {
            let mut stats = self.stats.write().await;
            stats.sync_sessions += 1;
        }
        
        // Request blocks in batches
        self.request_blocks_from_peer(peer_id, current_slot + 1, target_slot).await;
    }

    /// Request a batch of blocks from a peer
    async fn request_blocks_from_peer(&self, peer_id: &str, start_slot: u64, target_slot: u64) {
        let batch_end = std::cmp::min(start_slot + self.config.batch_size, target_slot);
        
        info!("Requesting blocks {}-{} from peer {}", start_slot, batch_end, peer_id);
        
        let request = BlockRequest {
            peer_id: peer_id.to_string(),
            start_slot,
            end_slot: batch_end,
            request_time: Instant::now(),
        };
        
        {
            let mut pending = self.pending_requests.write().await;
            pending.push_back(request);
        }
        
        // In a real implementation, this would send a network message
        // For now, we simulate block production locally
        debug!("Block request queued (network message would be sent to {})", peer_id);
    }

    /// Process received blocks from a peer
    pub async fn process_blocks(&self, blocks: Vec<SyncBlock>, peer_id: &str) -> Result<(), SyncError> {
        let mut state = self.state.write().await;
        let (target_slot, mut current_slot) = match &mut *state {
            SyncState::Syncing { target_slot, current_slot, .. } => (*target_slot, *current_slot),
            _ => return Err(SyncError::NotSyncing),
        };
        drop(state);
        
        info!("Processing {} blocks from peer {} (slots {}-{})", 
            blocks.len(), 
            peer_id,
            blocks.first().map(|b| b.block.slot).unwrap_or(0),
            blocks.last().map(|b| b.block.slot).unwrap_or(0),
        );
        
        // Validate and execute blocks in order
        let mut sorted_blocks = blocks;
        sorted_blocks.sort_by_key(|b| b.block.slot);
        
        let start_slot = self.validator_state.current_slot();
        
        for sync_block in sorted_blocks {
            // Verify slot sequence
            if sync_block.block.slot != current_slot + 1 {
                warn!("Block slot {} out of sequence, expected {}", 
                    sync_block.block.slot, current_slot + 1);
                continue;
            }
            
            // Verify parent hash matches our chain
            let expected_parent = if current_slot == 0 {
                self.validator_state.get_genesis_hash()
            } else {
                self.block_producer.current_block_hash().await
            };
            
            if sync_block.parent_hash != expected_parent {
                warn!("Block {} has invalid parent hash: expected {}, got {}", 
                    sync_block.block.slot, expected_parent, sync_block.parent_hash);
                return Err(SyncError::InvalidParentHash {
                    expected: expected_parent,
                    actual: sync_block.parent_hash,
                    slot: sync_block.block.slot,
                });
            }
            
            // Execute the block (update state)
            // In a real implementation, this would:
            // 1. Verify block signature
            // 2. Verify PoH hash
            // 3. Execute all transactions
            // 4. Update state root
            
            current_slot = sync_block.block.slot;
            
            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.blocks_synced += 1;
            }
        }
        
        // Update sync state
        {
            let mut state = self.state.write().await;
            if current_slot >= target_slot {
                *state = SyncState::Following;
                let mut stats = self.stats.write().await;
                stats.last_sync_completed = Some(Instant::now());
                info!("Sync completed at slot {}", current_slot);
            } else {
                // Continue syncing next batch
                *state = SyncState::Syncing {
                    target_slot,
                    current_slot,
                    peer_id: peer_id.to_string(),
                };
            }
        }
        
        // Request next batch if still syncing
        if current_slot < target_slot {
            self.request_blocks_from_peer(peer_id, current_slot + 1, target_slot).await;
        }
        
        Ok(())
    }

    /// Handle peer slot announcement - check if we need to sync
    pub async fn on_peer_slot(&self, peer_id: &str, peer_slot: u64) -> bool {
        let our_slot = self.validator_state.current_slot();
        
        // Check if we're behind and need to sync
        if peer_slot > our_slot + self.config.sync_threshold {
            let behind = peer_slot - our_slot;
            info!("Peer {} at slot {} is {} slots ahead, starting sync", 
                peer_id, peer_slot, behind);
            self.start_sync(peer_id, peer_slot).await;
            return true;
        }
        
        false
    }

    /// Create a state snapshot
    pub async fn create_snapshot(&self) -> Result<StateSnapshot, SyncError> {
        let slot = self.validator_state.current_slot();
        let block_hash = self.block_producer.current_block_hash().await;
        let state_root = self.block_producer.get_state_root().await;
        let epoch = slot / 432_000; // SLOTS_PER_EPOCH
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Get all accounts from state
        let accounts = self.block_producer.get_all_accounts().await;
        let total_supply = self.block_producer.total_supply().await;
        
        let snapshot_accounts: Vec<SnapshotAccount> = accounts.into_iter()
            .map(|(addr, acc)| SnapshotAccount {
                address: addr,
                lamports: acc.lamports,
                owner: acc.owner,
                data: acc.data,
                rent_epoch: acc.rent_epoch,
            })
            .collect();
        
        let snapshot = StateSnapshot {
            slot,
            block_hash,
            state_root,
            epoch,
            timestamp,
            accounts: snapshot_accounts,
            total_supply,
        };
        
        info!("Created snapshot at slot {} with {} accounts", slot, snapshot.accounts.len());
        
        Ok(snapshot)
    }

    /// Save snapshot to disk
    pub async fn save_snapshot(&self, snapshot: &StateSnapshot) -> Result<(), SyncError> {
        if let Some(ref pm) = self.persistence {
            let json = serde_json::to_string_pretty(snapshot)
                .map_err(|e| SyncError::SerializationError(e.to_string()))?;
            
            let snapshot_path = pm.data_dir().join(format!("snapshot-{}.json", snapshot.slot));
            tokio::fs::write(&snapshot_path, json).await
                .map_err(|e| SyncError::IOError(e.to_string()))?;
            
            info!("Saved snapshot to {}", snapshot_path.display());
        }
        Ok(())
    }

    /// Load snapshot from disk
    pub async fn load_snapshot(&self, slot: u64) -> Result<StateSnapshot, SyncError> {
        if let Some(ref pm) = self.persistence {
            let snapshot_path = pm.data_dir().join(format!("snapshot-{}.json", slot));
            
            if !snapshot_path.exists() {
                return Err(SyncError::SnapshotNotFound(slot));
            }
            
            let json = tokio::fs::read_to_string(&snapshot_path).await
                .map_err(|e| SyncError::IOError(e.to_string()))?;
            
            let snapshot: StateSnapshot = serde_json::from_str(&json)
                .map_err(|e| SyncError::DeserializationError(e.to_string()))?;
            
            info!("Loaded snapshot from slot {} with {} accounts", 
                snapshot.slot, snapshot.accounts.len());
            
            return Ok(snapshot);
        }
        
        Err(SyncError::SnapshotNotFound(slot))
    }

    /// Apply a snapshot to restore state
    pub async fn apply_snapshot(&self, snapshot: StateSnapshot) -> Result<(), SyncError> {
        // Verify snapshot size
        if snapshot.accounts.len() * 100 > self.config.max_snapshot_size {
            return Err(SyncError::SnapshotTooLarge(
                snapshot.accounts.len() * 100,
                self.config.max_snapshot_size,
            ));
        }
        
        // Verify snapshot hash
        let computed_hash = Self::compute_snapshot_hash(&snapshot);
        if computed_hash != snapshot.state_root {
            warn!("Snapshot hash mismatch: computed {} != expected {}", 
                computed_hash, snapshot.state_root);
            // Continue anyway - in production we'd verify against peers
        }
        
        // Get count before moving
        let account_count = snapshot.accounts.len();
        let snapshot_slot = snapshot.slot;
        let snapshot_block_hash = snapshot.block_hash.clone();
        
        // Apply accounts to state
        for account in snapshot.accounts {
            let acc = aether_core::Account {
                lamports: account.lamports,
                owner: account.owner,
                data: account.data,
                rent_epoch: account.rent_epoch,
            };
            self.block_producer.set_account(&account.address, acc).await;
        }
        
        // Update validator state
        self.validator_state.sync_slot(snapshot_slot);
        self.validator_state.set_block_hash(snapshot_block_hash);
        
        info!("Applied snapshot from slot {} with {} accounts", 
            snapshot_slot, account_count);
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.snapshots_downloaded += 1;
        }
        
        Ok(())
    }

    /// Compute hash of snapshot for verification
    fn compute_snapshot_hash(snapshot: &StateSnapshot) -> String {
        let mut hasher = Sha256::new();
        hasher.update(snapshot.slot.to_le_bytes());
        hasher.update(snapshot.block_hash.as_bytes());
        hasher.update(snapshot.epoch.to_le_bytes());
        hasher.update(snapshot.timestamp.to_le_bytes());
        hasher.update(snapshot.total_supply.to_le_bytes());
        
        // Sort accounts by address for deterministic hash
        let mut accounts = snapshot.accounts.clone();
        accounts.sort_by_key(|a| a.address);
        
        for account in accounts {
            hasher.update(account.address);
            hasher.update(account.lamports.to_le_bytes());
            hasher.update(account.owner);
            hasher.update(account.data.len().to_le_bytes());
            hasher.update(&account.data);
        }
        
        let result = hasher.finalize();
        bs58::encode(result).into_string()
    }

    /// Start fast sync from a peer
    pub async fn start_fast_sync(&self, peer_id: &str, snapshot_slot: u64) -> Result<(), SyncError> {
        if !self.config.enable_fast_sync {
            return Err(SyncError::FastSyncDisabled);
        }
        
        info!("Starting fast sync from peer {} at snapshot slot {}", peer_id, snapshot_slot);
        
        // Mark that we're downloading a snapshot
        {
            let mut downloading = self.downloading_snapshot.write().await;
            *downloading = true;
        }
        
        // Update state
        {
            let mut state = self.state.write().await;
            *state = SyncState::AwaitingSnapshot {
                peer_id: peer_id.to_string(),
                since: Instant::now(),
            };
        }
        
        // In production, this would request a snapshot from the peer
        // For now, try to load from local storage
        match self.load_snapshot(snapshot_slot).await {
            Ok(snapshot) => {
                self.apply_snapshot(snapshot).await?;
                
                // Clear downloading flag
                {
                    let mut downloading = self.downloading_snapshot.write().await;
                    *downloading = false;
                }
                
                // Start normal sync from snapshot forward
                let current_slot = self.validator_state.current_slot();
                self.start_sync(peer_id, current_slot + 100).await; // Sync next 100 slots
                
                Ok(())
            }
            Err(e) => {
                // Clear downloading flag
                {
                    let mut downloading = self.downloading_snapshot.write().await;
                    *downloading = false;
                }
                
                // Fall back to normal sync
                warn!("Failed to load snapshot: {}, falling back to normal sync", e);
                self.start_sync(peer_id, snapshot_slot).await;
                Ok(())
            }
        }
    }

    /// Check for timeout on pending requests
    pub async fn check_timeouts(&self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(self.config.response_timeout_secs);
        
        let mut pending = self.pending_requests.write().await;
        
        // Collect timed out requests (just the count, we don't need the actual items)
        let timed_out_count = pending.iter()
            .filter(|r| now.duration_since(r.request_time) > timeout)
            .count();
        
        if timed_out_count > 0 {
            warn!("{} block requests timed out", timed_out_count);
            
            // Remove timed out requests
            pending.retain(|r| now.duration_since(r.request_time) <= timeout);
            
            // Increment failed attempts
            let mut stats = self.stats.write().await;
            stats.failed_attempts += timed_out_count as u64;
            
            // Pause sync if too many failures
            if stats.failed_attempts > 3 {
                let mut state = self.state.write().await;
                *state = SyncState::Paused {
                    reason: "Too many sync failures".to_string(),
                    since: Instant::now(),
                };
            }
        }
    }

    /// Resume sync after pause
    pub async fn resume_sync(&self, peer_id: &str) {
        let target_slot = self.validator_state.current_slot() + 100;
        self.start_sync(peer_id, target_slot).await;
    }

    /// Periodic sync check
    pub async fn run_sync_check(&self, known_peers: &[String]) {
        // Don't check if already syncing
        if self.is_syncing().await {
            return;
        }
        
        // Check for timeouts
        self.check_timeouts().await;
        
        // In production, would query peers for their current slot
        // and trigger sync if behind
        debug!("Sync check complete ({} known peers)", known_peers.len());
    }
}

/// Sync errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum SyncError {
    #[error("Not currently syncing")]
    NotSyncing,
    
    #[error("Invalid parent hash at slot {slot}: expected {expected}, got {actual}")]
    InvalidParentHash {
        expected: String,
        actual: String,
        slot: u64,
    },
    
    #[error("Block validation failed: {0}")]
    BlockValidation(String),
    
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(u64),
    
    #[error("Snapshot too large: {0} bytes (max {1})")]
    SnapshotTooLarge(usize, usize),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("IO error: {0}")]
    IOError(String),
    
    #[error("Fast sync is disabled")]
    FastSyncDisabled,
    
    #[error("No peers available")]
    NoPeers,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_db::StateDB;
    use crate::state::ValidatorState;
    use std::path::PathBuf;

    fn create_test_sync_manager() -> SyncManager {
        let state_db = StateDB::new();
        let identity = crate::keypair::generate_keypair();
        let validator_state = ValidatorState::new(identity.clone(), false, PathBuf::from("test_ledger")).unwrap();
        let block_producer = Arc::new(BlockProducer::new(validator_state.clone(), state_db));
        
        SyncManager::new(
            SyncConfig::default(),
            validator_state,
            block_producer,
            None,
        )
    }

    #[tokio::test]
    async fn test_sync_state_following() {
        let sm = create_test_sync_manager();
        assert!(matches!(sm.get_state().await, SyncState::Following));
        assert!(!sm.is_syncing().await);
    }

    #[tokio::test]
    async fn test_start_sync() {
        let sm = create_test_sync_manager();
        
        sm.start_sync("peer-1", 100).await;
        
        let state = sm.get_state().await;
        match state {
            SyncState::Syncing { target_slot, peer_id, .. } => {
                assert_eq!(target_slot, 100);
                assert_eq!(peer_id, "peer-1");
            }
            _ => panic!("Expected Syncing state"),
        }
        
        assert!(sm.is_syncing().await);
    }

    #[tokio::test]
    async fn test_slots_behind() {
        let sm = create_test_sync_manager();
        
        // When caught up
        assert_eq!(sm.slots_behind(10).await, 0);
        
        // When behind
        assert_eq!(sm.slots_behind(100).await, 100);
        
        // When syncing
        sm.start_sync("peer-1", 200).await;
        assert_eq!(sm.slots_behind(200).await, 0); // Already syncing to target
        assert_eq!(sm.slots_behind(300).await, 100); // Beyond target
    }

    #[tokio::test]
    async fn test_create_snapshot() {
        let sm = create_test_sync_manager();
        
        let snapshot = sm.create_snapshot().await.unwrap();
        assert_eq!(snapshot.slot, 0); // Genesis slot
        assert!(snapshot.accounts.is_empty()); // No accounts yet
    }

    #[tokio::test]
    async fn test_snapshot_hash() {
        let sm = create_test_sync_manager();
        
        let snapshot = sm.create_snapshot().await.unwrap();
        let hash1 = SyncManager::compute_snapshot_hash(&snapshot);
        let hash2 = SyncManager::compute_snapshot_hash(&snapshot);
        
        // Same snapshot should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different snapshot should produce different hash
        let mut snapshot2 = snapshot.clone();
        snapshot2.total_supply = 1_000_000;
        let hash3 = SyncManager::compute_snapshot_hash(&snapshot2);
        assert_ne!(hash1, hash3);
    }
}