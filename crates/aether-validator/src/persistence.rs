//! State Persistence Module
//!
//! Provides durable storage for validator state, blocks, and accounts.
//! Ensures recovery from restarts without data loss - critical for production testnet.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

/// Snapshot of validator state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSnapshot {
    /// Current slot
    pub current_slot: u64,
    /// Current block hash
    pub block_hash: String,
    /// Blocks produced count
    pub blocks_produced: u64,
    /// Transaction count
    pub transaction_count: u64,
    /// Genesis hash
    pub genesis_hash: String,
    /// Chain ID
    pub chain_id: String,
    /// Peer pubkeys
    pub peers: Vec<String>,
    /// Timestamp of snapshot
    pub timestamp: u64,
}

/// Persisted account state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedAccount {
    pub address: [u8; 32],
    pub lamports: u64,
    pub owner: [u8; 32],
    pub data: Vec<u8>,
    pub rent_epoch: u64,
}

/// Persisted block data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedBlock {
    pub slot: u64,
    pub timestamp: u64,
    pub previous_block_hash: String,
    pub block_hash: String,
    pub transactions: Vec<String>,
    pub poh_seed: String,
    pub state_root: String,
}

/// Persistence manager for validator state
pub struct PersistenceManager {
    /// Base directory for all persistent data
    data_dir: PathBuf,
    /// Snapshot file path
    snapshot_path: PathBuf,
    /// Accounts file path
    accounts_path: PathBuf,
    /// Blocks directory
    blocks_dir: PathBuf,
    /// Write-ahead log for crash recovery
    wal_path: PathBuf,
}

impl PersistenceManager {
    /// Create a new persistence manager with the given ledger path
    pub fn new(ledger_path: &Path) -> anyhow::Result<Self> {
        let data_dir = ledger_path.join("data");
        let blocks_dir = data_dir.join("blocks");
        
        // Create directories if they don't exist
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("Failed to create data dir: {}", data_dir.display()))?;
        fs::create_dir_all(&blocks_dir)
            .with_context(|| format!("Failed to create blocks dir: {}", blocks_dir.display()))?;
        
        Ok(Self {
            snapshot_path: data_dir.join("validator-snapshot.json"),
            accounts_path: data_dir.join("accounts.json"),
            blocks_dir,
            wal_path: data_dir.join("wal.json"),
            data_dir,
        })
    }
    
    /// Save validator state snapshot
    pub fn save_snapshot(&self, snapshot: &ValidatorSnapshot) -> anyhow::Result<()> {
        // Write to WAL first for crash safety
        self.write_wal("snapshot", &serde_json::to_string(snapshot)?)?;
        
        // Write atomically using temp file
        let temp_path = format!("{}.tmp", self.snapshot_path.display());
        let json = serde_json::to_string_pretty(snapshot)?;
        
        let mut file = fs::File::create(&temp_path)
            .with_context(|| "Failed to create temp snapshot file")?;
        file.write_all(json.as_bytes())
            .with_context(|| "Failed to write snapshot")?;
        file.sync_all()
            .with_context(|| "Failed to sync snapshot")?;
        
        // Atomic rename
        fs::rename(&temp_path, &self.snapshot_path)
            .with_context(|| "Failed to rename snapshot file")?;
        
        // Clear WAL after successful write
        self.clear_wal()?;
        
        debug!("Saved validator snapshot at slot {}", snapshot.current_slot);
        Ok(())
    }
    
    /// Load validator state snapshot
    pub fn load_snapshot(&self) -> anyhow::Result<Option<ValidatorSnapshot>> {
        // Check for WAL recovery first
        if let Some(wal_data) = self.recover_wal()? {
            warn!("Recovered from WAL after crash");
            if wal_data.starts_with("snapshot:") {
                let json = wal_data.trim_start_matches("snapshot:");
                if let Ok(snapshot) = serde_json::from_str(json) {
                    return Ok(Some(snapshot));
                }
            }
        }
        
        if !self.snapshot_path.exists() {
            debug!("No snapshot file found");
            return Ok(None);
        }
        
        let mut file = fs::File::open(&self.snapshot_path)
            .with_context(|| "Failed to open snapshot file")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| "Failed to read snapshot")?;
        
        let snapshot: ValidatorSnapshot = serde_json::from_str(&contents)
            .with_context(|| "Failed to parse snapshot JSON")?;
        
        info!("Loaded validator snapshot: slot={}, blocks_produced={}", 
            snapshot.current_slot, snapshot.blocks_produced);
        
        Ok(Some(snapshot))
    }
    
    /// Save accounts to disk
    pub fn save_accounts(&self, accounts: &[PersistedAccount]) -> anyhow::Result<()> {
        // Write to WAL first
        self.write_wal("accounts", &serde_json::to_string(accounts)?)?;
        
        let temp_path = format!("{}.tmp", self.accounts_path.display());
        let json = serde_json::to_string_pretty(accounts)?;
        
        let mut file = fs::File::create(&temp_path)
            .with_context(|| "Failed to create temp accounts file")?;
        file.write_all(json.as_bytes())
            .with_context(|| "Failed to write accounts")?;
        file.sync_all()
            .with_context(|| "Failed to sync accounts")?;
        
        fs::rename(&temp_path, &self.accounts_path)
            .with_context(|| "Failed to rename accounts file")?;
        
        self.clear_wal()?;
        
        debug!("Saved {} accounts", accounts.len());
        Ok(())
    }
    
    /// Load accounts from disk
    pub fn load_accounts(&self) -> anyhow::Result<Vec<PersistedAccount>> {
        // Check WAL recovery
        if let Some(wal_data) = self.recover_wal()? {
            if wal_data.starts_with("accounts:") {
                let json = wal_data.trim_start_matches("accounts:");
                if let Ok(accounts) = serde_json::from_str(json) {
                    return Ok(accounts);
                }
            }
        }
        
        if !self.accounts_path.exists() {
            debug!("No accounts file found");
            return Ok(Vec::new());
        }
        
        let mut file = fs::File::open(&self.accounts_path)
            .with_context(|| "Failed to open accounts file")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| "Failed to read accounts")?;
        
        let accounts: Vec<PersistedAccount> = serde_json::from_str(&contents)
            .with_context(|| "Failed to parse accounts JSON")?;
        
        info!("Loaded {} accounts from disk", accounts.len());
        Ok(accounts)
    }
    
    /// Save a block to disk
    pub fn save_block(&self, block: &PersistedBlock) -> anyhow::Result<()> {
        // Store blocks in chunks of 1000 slots per file
        let chunk = block.slot / 1000;
        let chunk_path = self.blocks_dir.join(format!("blocks-{:06}.json", chunk));
        
        // Load existing chunk or create new
        let mut blocks: Vec<PersistedBlock> = if chunk_path.exists() {
            let mut file = fs::File::open(&chunk_path)
                .with_context(|| "Failed to open blocks chunk")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .with_context(|| "Failed to read blocks chunk")?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        // Check if block already exists
        if !blocks.iter().any(|b| b.slot == block.slot) {
            blocks.push(block.clone());
            blocks.sort_by_key(|b| b.slot);
            
            // Write atomically
            let temp_path = format!("{}.tmp", chunk_path.display());
            let json = serde_json::to_string(&blocks)?;
            let mut file = fs::File::create(&temp_path)
                .with_context(|| "Failed to create temp blocks file")?;
            file.write_all(json.as_bytes())
                .with_context(|| "Failed to write blocks")?;
            file.sync_all()?;
            
            fs::rename(&temp_path, &chunk_path)?;
        }
        
        debug!("Saved block at slot {}", block.slot);
        Ok(())
    }
    
    /// Load a block by slot
    pub fn load_block(&self, slot: u64) -> anyhow::Result<Option<PersistedBlock>> {
        let chunk = slot / 1000;
        let chunk_path = self.blocks_dir.join(format!("blocks-{:06}.json", chunk));
        
        if !chunk_path.exists() {
            return Ok(None);
        }
        
        let mut file = fs::File::open(&chunk_path)
            .with_context(|| "Failed to open blocks chunk")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| "Failed to read blocks chunk")?;
        
        let blocks: Vec<PersistedBlock> = serde_json::from_str(&contents)
            .with_context(|| "Failed to parse blocks JSON")?;
        
        Ok(blocks.into_iter().find(|b| b.slot == slot))
    }
    
    /// Load blocks in range [start, end)
    pub fn load_blocks_range(&self, start: u64, end: u64) -> anyhow::Result<Vec<PersistedBlock>> {
        let mut all_blocks = Vec::new();
        
        let start_chunk = start / 1000;
        let end_chunk = end / 1000;
        
        for chunk in start_chunk..=end_chunk {
            let chunk_path = self.blocks_dir.join(format!("blocks-{:06}.json", chunk));
            
            if !chunk_path.exists() {
                continue;
            }
            
            let mut file = fs::File::open(&chunk_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            let blocks: Vec<PersistedBlock> = serde_json::from_str(&contents)
                .unwrap_or_default();
            
            all_blocks.extend(
                blocks.into_iter()
                    .filter(|b| b.slot >= start && b.slot < end)
            );
        }
        
        all_blocks.sort_by_key(|b| b.slot);
        Ok(all_blocks)
    }
    
    /// Get the highest slot we have persisted
    pub fn get_latest_slot(&self) -> anyhow::Result<Option<u64>> {
        let mut max_slot = None;
        
        for entry in fs::read_dir(&self.blocks_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            
            if name_str.starts_with("blocks-") && name_str.ends_with(".json") {
                // Try to read the last block in this chunk
                let mut file = fs::File::open(entry.path())?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                
                if let Ok(blocks) = serde_json::from_str::<Vec<PersistedBlock>>(&contents) {
                    if let Some(last) = blocks.last() {
                        max_slot = Some(max_slot.unwrap_or(0).max(last.slot));
                    }
                }
            }
        }
        
        Ok(max_slot)
    }
    
    /// Write to WAL for crash recovery
    fn write_wal(&self, operation: &str, data: &str) -> anyhow::Result<()> {
        let wal_entry = format!("{}:{}", operation, data);
        let mut file = fs::File::create(&self.wal_path)?;
        file.write_all(wal_entry.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
    
    /// Clear WAL after successful operation
    fn clear_wal(&self) -> anyhow::Result<()> {
        if self.wal_path.exists() {
            fs::remove_file(&self.wal_path)?;
        }
        Ok(())
    }
    
    /// Recover from WAL if exists
    fn recover_wal(&self) -> anyhow::Result<Option<String>> {
        if !self.wal_path.exists() {
            return Ok(None);
        }
        
        let mut file = fs::File::open(&self.wal_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        Ok(Some(contents))
    }
    
    /// Compute integrity hash of persisted data
    pub fn compute_integrity_hash(&self) -> anyhow::Result<String> {
        let mut hasher = Sha256::new();
        
        // Hash snapshot
        if let Some(snapshot) = self.load_snapshot()? {
            hasher.update(serde_json::to_string(&snapshot)?);
        }
        
        // Hash accounts
        let accounts = self.load_accounts()?;
        hasher.update(serde_json::to_string(&accounts)?);
        
        // Hash latest blocks
        if let Some(latest) = self.get_latest_slot()? {
            let start = latest.saturating_sub(100);
            let blocks = self.load_blocks_range(start, latest)?;
            hasher.update(serde_json::to_string(&blocks)?);
        }
        
        let result = hasher.finalize();
        Ok(bs58::encode(result).into_string())
    }
    
    /// Get data directory path
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
    
    /// Check if persisted state exists
    pub fn has_persisted_state(&self) -> bool {
        self.snapshot_path.exists()
    }
}

impl Clone for PersistenceManager {
    fn clone(&self) -> Self {
        Self {
            data_dir: self.data_dir.clone(),
            snapshot_path: self.snapshot_path.clone(),
            accounts_path: self.accounts_path.clone(),
            blocks_dir: self.blocks_dir.clone(),
            wal_path: self.wal_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_snapshot_persistence() {
        let dir = TempDir::new().unwrap();
        let pm = PersistenceManager::new(dir.path()).unwrap();
        
        let snapshot = ValidatorSnapshot {
            current_slot: 12345,
            block_hash: "test-hash".to_string(),
            blocks_produced: 100,
            transaction_count: 500,
            genesis_hash: "genesis-hash".to_string(),
            chain_id: "test-chain".to_string(),
            peers: vec!["peer1".to_string()],
            timestamp: 1234567890,
        };
        
        pm.save_snapshot(&snapshot).unwrap();
        let loaded = pm.load_snapshot().unwrap().unwrap();
        
        assert_eq!(loaded.current_slot, 12345);
        assert_eq!(loaded.block_hash, "test-hash");
        assert_eq!(loaded.blocks_produced, 100);
    }
    
    #[test]
    fn test_accounts_persistence() {
        let dir = TempDir::new().unwrap();
        let pm = PersistenceManager::new(dir.path()).unwrap();
        
        let accounts = vec![
            PersistedAccount {
                address: [1u8; 32],
                lamports: 1000,
                owner: [0u8; 32],
                data: vec![],
                rent_epoch: 0,
            },
            PersistedAccount {
                address: [2u8; 32],
                lamports: 2000,
                owner: [0u8; 32],
                data: vec![1, 2, 3],
                rent_epoch: 1,
            },
        ];
        
        pm.save_accounts(&accounts).unwrap();
        let loaded = pm.load_accounts().unwrap();
        
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].lamports, 1000);
        assert_eq!(loaded[1].data, vec![1, 2, 3]);
    }
    
    #[test]
    fn test_block_persistence() {
        let dir = TempDir::new().unwrap();
        let pm = PersistenceManager::new(dir.path()).unwrap();
        
        let block = PersistedBlock {
            slot: 500,
            timestamp: 1234567890,
            previous_block_hash: "prev".to_string(),
            block_hash: "curr".to_string(),
            transactions: vec!["tx1".to_string()],
            poh_seed: "seed".to_string(),
            state_root: "root".to_string(),
        };
        
        pm.save_block(&block).unwrap();
        let loaded = pm.load_block(500).unwrap().unwrap();
        
        assert_eq!(loaded.slot, 500);
        assert_eq!(loaded.block_hash, "curr");
        assert_eq!(loaded.transactions.len(), 1);
    }
}