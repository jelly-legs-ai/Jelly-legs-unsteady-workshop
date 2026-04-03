//! Validator state management
//!
//! In-memory state tracking for the running validator.

use crate::keypair::ValidatorIdentity;
use crate::{BlockProduction, EpochInfo, ValidatorInfo, VoteAccountInfo};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Thread-safe validator state shared across all async tasks
#[derive(Clone)]
pub struct ValidatorState {
    inner: Arc<ValidatorStateInner>,
}

struct ValidatorStateInner {
    // Identity
    #[allow(dead_code)]
    identity: RwLock<Option<ValidatorIdentity>>,
    
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
    
    // Ledger
    #[allow(dead_code)]
    ledger_path: PathBuf,
    
    // Testnet mode
    #[allow(dead_code)]
    testnet: bool,
}

impl ValidatorState {
    pub fn new(identity: ValidatorIdentity, testnet: bool, ledger_path: PathBuf) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Arc::new(ValidatorStateInner {
                identity: RwLock::new(Some(identity)),
                current_slot: AtomicU64::new(0),
                block_height: AtomicU64::new(0),
                transaction_count: AtomicU64::new(0),
                epoch: AtomicU64::new(0),
                slot_index: AtomicU64::new(0),
                peer_count: AtomicU64::new(0),
                peer_pubkeys: RwLock::new(Vec::new()),
                blocks_produced: AtomicU64::new(0),
                vote_count: AtomicU64::new(0),
                ledger_path,
                testnet,
            }),
        })
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
        let mut peers = self.inner.peer_pubkeys.blocking_write();
        if !peers.contains(&pubkey) {
            peers.push(pubkey);
            self.inner.peer_count.store(peers.len() as u64, Ordering::Relaxed);
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
        // Return empty for MVP
        Vec::new()
    }
}
