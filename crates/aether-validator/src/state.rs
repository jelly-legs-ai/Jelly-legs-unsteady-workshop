//! Validator state management
//!
//! In-memory state tracking for the running validator.

use crate::genesis::{GenesisBlock, load_genesis_from_file};
use crate::keypair::ValidatorIdentity;
use crate::{BlockProduction, EpochInfo, ValidatorInfo, VoteAccountInfo};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Thread-safe validator state shared across all async tasks
#[derive(Clone)]
pub struct ValidatorState {
    inner: Arc<ValidatorStateInner>,
}

struct ValidatorStateInner {
    // Identity
    #[allow(dead_code)]
    identity: RwLock<Option<ValidatorIdentity>>,
    
    // Genesis
    genesis: RwLock<Option<GenesisBlock>>,
    
    // Chain state
    current_slot: AtomicU64,
    block_height: AtomicU64,
    transaction_count: AtomicU64,
    
    // Epoch state
    epoch: AtomicU64,
    slot_index: AtomicU64,
    
    // Network state
    peer_count: AtomicU64,
    peer_pubkeys: RwLock<Vec<String>>,
    
    // Block production
    blocks_produced: AtomicU64,
    vote_count: AtomicU64,
    block_hash: RwLock<String>,
    
    // Ledger
    #[allow(dead_code)]
    ledger_path: PathBuf,
    
    // Testnet mode
    #[allow(dead_code)]
    testnet: bool,
}

impl ValidatorState {
    /// Create new state WITHOUT genesis (self-generated internal state)
    pub fn new(identity: ValidatorIdentity, testnet: bool, ledger_path: PathBuf) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Arc::new(ValidatorStateInner {
                identity: RwLock::new(Some(identity)),
                genesis: RwLock::new(None),
                current_slot: AtomicU64::new(0),
                block_height: AtomicU64::new(0),
                transaction_count: AtomicU64::new(0),
                epoch: AtomicU64::new(0),
                slot_index: AtomicU64::new(0),
                peer_count: AtomicU64::new(0),
                peer_pubkeys: RwLock::new(Vec::new()),
                blocks_produced: AtomicU64::new(0),
                vote_count: AtomicU64::new(0),
                block_hash: RwLock::new("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
                ledger_path,
                testnet,
            }),
        })
    }

    /// Create new state WITH genesis file loaded
    pub fn with_genesis(
        identity: ValidatorIdentity,
        testnet: bool,
        ledger_path: PathBuf,
        genesis_path: &Path,
    ) -> anyhow::Result<Self> {
        let genesis = load_genesis_from_file(genesis_path)?;
        let genesis_hash = genesis.genesis_hash.clone();
        
        Ok(Self {
            inner: Arc::new(ValidatorStateInner {
                identity: RwLock::new(Some(identity)),
                genesis: RwLock::new(Some(genesis)),
                current_slot: AtomicU64::new(0),
                block_height: AtomicU64::new(0),
                transaction_count: AtomicU64::new(0),
                epoch: AtomicU64::new(0),
                slot_index: AtomicU64::new(0),
                peer_count: AtomicU64::new(0),
                peer_pubkeys: RwLock::new(Vec::new()),
                blocks_produced: AtomicU64::new(0),
                vote_count: AtomicU64::new(0),
                block_hash: RwLock::new(genesis_hash),
                ledger_path,
                testnet,
            }),
        })
    }

    // Genesis accessors
    pub fn has_genesis(&self) -> bool {
        self.inner.genesis.read().unwrap().is_some()
    }

    pub fn get_genesis(&self) -> Option<GenesisBlock> {
        self.inner.genesis.read().unwrap().clone()
    }

    pub fn get_genesis_hash(&self) -> String {
        self.inner.genesis.read()
            .unwrap()
            .as_ref()
            .map(|g| g.genesis_hash.clone())
            .unwrap_or_else(|| crate::genesis::generate_genesis_hash())
    }

    pub fn get_chain_id(&self) -> String {
        self.inner.genesis.read()
            .unwrap()
            .as_ref()
            .map(|g| g.chain_id.clone())
            .unwrap_or_else(|| "aether-testnet-1".to_string())
    }

    // Slot and block accessors
    pub fn current_slot(&self) -> u64 {
        self.inner.current_slot.load(Ordering::Relaxed)
    }

    pub fn block_height(&self) -> u64 {
        self.inner.block_height.load(Ordering::Relaxed)
    }

    pub fn transaction_count(&self) -> u64 {
        self.inner.transaction_count.load(Ordering::Relaxed)
    }

    pub fn vote_count(&self) -> u64 {
        self.inner.vote_count.load(Ordering::Relaxed)
    }

    pub fn peer_count(&self) -> u64 {
        self.inner.peer_count.load(Ordering::Relaxed)
    }

    pub fn blocks_produced(&self) -> u64 {
        self.inner.blocks_produced.load(Ordering::Relaxed)
    }

    pub fn increment_slot(&self) {
        let slot = self.inner.current_slot.fetch_add(1, Ordering::Relaxed) + 1;
        self.inner.block_height.store(slot, Ordering::Relaxed);
        self.inner.slot_index.store(slot % 432_000, Ordering::Relaxed);
        self.inner.epoch.store(slot / 432_000, Ordering::Relaxed);
        self.inner.transaction_count.fetch_add(1, Ordering::Relaxed);
        
        if slot.is_multiple_of(32) {
            self.inner.blocks_produced.fetch_add(1, Ordering::Relaxed);
            self.inner.vote_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn update_peer_count(&self, count: u64) {
        self.inner.peer_count.store(count, Ordering::Relaxed);
    }

    pub fn add_peer(&self, pubkey: String) {
        let mut peers = self.inner.peer_pubkeys.write().unwrap();
        if !peers.contains(&pubkey) {
            peers.push(pubkey);
            drop(peers);
            self.inner.peer_count.store(self.inner.peer_pubkeys.read().unwrap().len() as u64, Ordering::Relaxed);
        }
    }

    pub fn epoch_info(&self) -> EpochInfo {
        let slot = self.current_slot();
        EpochInfo {
            epoch: slot / 432_000,
            slot_index: slot % 432_000,
            slots_in_epoch: 432_000,
            absolute_slot: slot,
            transaction_count: self.transaction_count(),
        }
    }

    pub fn block_production(&self) -> BlockProduction {
        BlockProduction {
            blocks_produced: self.inner.blocks_produced.load(Ordering::Relaxed),
            entries_produced: self.inner.vote_count.load(Ordering::Relaxed),
        }
    }

    pub fn get_connected_validators(&self) -> Vec<ValidatorInfo> {
        // Return self as the only known validator (MVP)
        vec![ValidatorInfo {
            identity_pubkey: "LocalValidator11111111111111111111111111".to_string(),
            activated_stake: 10_000_000,
            commission: 10,
            active: true,
        }]
    }

    pub fn get_vote_accounts(&self) -> Vec<VoteAccountInfo> {
        // Return bootstrap validators from genesis as vote accounts
        self.inner.genesis.read()
            .unwrap()
            .as_ref()
            .map(|g| {
                g.bootstrap_validators
                    .iter()
                    .map(|v| VoteAccountInfo {
                        pubkey: v.identity_pubkey.clone(),
                        validator_pubkey: v.identity_pubkey.clone(),
                        commission: v.commission,
                        active: true,
                    })
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    pub fn set_current_slot(&self, slot: u64) {
        self.inner.current_slot.store(slot, Ordering::Relaxed);
        self.inner.slot_index.store(slot % 432_000, Ordering::Relaxed);
        self.inner.epoch.store(slot / 432_000, Ordering::Relaxed);
    }

    pub fn set_block_hash(&self, hash: String) {
        let mut bh = self.inner.block_hash.write().unwrap();
        *bh = hash;
    }

    pub fn get_last_block_hash(&self) -> String {
        self.inner.block_hash.read().unwrap().clone()
    }

    pub fn increment_produced_blocks(&self) {
        self.inner.blocks_produced.fetch_add(1, Ordering::Relaxed);
        self.inner.transaction_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_initial_balances(&self) -> Vec<(String, u64)> {
        // Return genesis initial balances if loaded
        self.inner.genesis.read()
            .unwrap()
            .as_ref()
            .map(|g| {
                g.bootstrap_validators
                    .iter()
                    .map(|v| (v.identity_pubkey.clone(), v.stake))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_slot_time_ms(&self) -> u64 {
        self.inner.genesis.read()
            .unwrap()
            .as_ref()
            .map(|g| g.consensus.slot_time_ms)
            .unwrap_or(400)
    }

    pub fn get_tower_finality(&self) -> u64 {
        self.inner.genesis.read()
            .unwrap()
            .as_ref()
            .map(|g| g.consensus.tower_finality)
            .unwrap_or(12)
    }
}
