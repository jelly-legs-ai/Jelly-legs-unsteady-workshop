//! Validator state management
//!
//! In-memory state tracking for the running validator.

use crate::genesis::{GenesisBlock, load_genesis_from_file, ValidatorTier, TierConfig};
use crate::keypair::ValidatorIdentity;
use crate::{BlockProduction, EpochInfo, ValidatorInfo, VoteAccountInfo};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tracing::info;

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
    
    // Validator tier configuration
    tier: RwLock<ValidatorTier>,
    tier_config: RwLock<Option<TierConfig>>,
    
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
    
    // Observer relay tracking (bytes relayed this epoch)
    relay_bytes: AtomicU64,
    
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
                tier: RwLock::new(ValidatorTier::Full),
                tier_config: RwLock::new(Some(TierConfig::default())),
                current_slot: AtomicU64::new(0),
                block_height: AtomicU64::new(0),
                transaction_count: AtomicU64::new(0),
                epoch: AtomicU64::new(0),
                slot_index: AtomicU64::new(0),
                peer_count: AtomicU64::new(0),
                peer_pubkeys: RwLock::new(Vec::new()),
                blocks_produced: AtomicU64::new(0),
                vote_count: AtomicU64::new(0),
                relay_bytes: AtomicU64::new(0),
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
        
        // Extract tier config from genesis
        let (tier, tier_config) = if let Some(ref tc) = genesis.consensus.tier_config {
            (tc.tier, Some(tc.clone()))
        } else {
            (ValidatorTier::Full, Some(TierConfig::default()))
        };
        
        Ok(Self {
            inner: Arc::new(ValidatorStateInner {
                identity: RwLock::new(Some(identity)),
                genesis: RwLock::new(Some(genesis)),
                tier: RwLock::new(tier),
                tier_config: RwLock::new(tier_config),
                current_slot: AtomicU64::new(0),
                block_height: AtomicU64::new(0),
                transaction_count: AtomicU64::new(0),
                epoch: AtomicU64::new(0),
                slot_index: AtomicU64::new(0),
                peer_count: AtomicU64::new(0),
                peer_pubkeys: RwLock::new(Vec::new()),
                blocks_produced: AtomicU64::new(0),
                vote_count: AtomicU64::new(0),
                relay_bytes: AtomicU64::new(0),
                block_hash: RwLock::new(genesis_hash),
                ledger_path,
                testnet,
            }),
        })
    }

    // Genesis accessors - safe versions that handle poisoned locks gracefully
    pub fn has_genesis(&self) -> bool {
        self.inner.genesis.read().map(|g| g.is_some()).unwrap_or(false)
    }

    pub fn get_genesis(&self) -> Option<GenesisBlock> {
        self.inner.genesis.read().ok().and_then(|g| g.clone())
    }

    pub fn get_genesis_hash(&self) -> String {
        // Safely read genesis hash without panicking on poisoned lock
        match self.inner.genesis.read() {
            Ok(genesis_opt) => {
                genesis_opt
                    .as_ref()
                    .map(|g| g.genesis_hash.clone())
                    .unwrap_or_else(|| self.inner.block_hash.read().map(|bh| bh.clone()).unwrap_or_default())
            }
            Err(_) => {
                // Lock poisoned - return current block hash as fallback
                self.inner.block_hash.read().map(|bh| bh.clone()).unwrap_or_default()
            }
        }
    }

    pub fn get_chain_id(&self) -> String {
        // Safely read chain ID without panicking on poisoned lock
        match self.inner.genesis.read() {
            Ok(genesis_opt) => {
                genesis_opt
                    .as_ref()
                    .map(|g| g.chain_id.clone())
                    .unwrap_or_else(|| "aether-testnet-1".to_string())
            }
            Err(_) => "aether-testnet-1".to_string(),
        }
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
        // Safely acquire write lock, return early if poisoned
        let mut peers = match self.inner.peer_pubkeys.write() {
            Ok(lock) => lock,
            Err(_) => return, // Lock poisoned - skip peer addition
        };
        
        if !peers.contains(&pubkey) {
            peers.push(pubkey);
            drop(peers);
            // Safely read peer count, skip update if lock is poisoned
            if let Ok(peer_list) = self.inner.peer_pubkeys.read() {
                self.inner.peer_count.store(peer_list.len() as u64, Ordering::Relaxed);
            }
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
        let epoch = self.epoch_info();
        BlockProduction {
            blocks_produced: self.inner.blocks_produced.load(Ordering::Relaxed),
            entries_produced: self.inner.vote_count.load(Ordering::Relaxed),
            epoch: epoch.epoch,
            slot_index: epoch.slot_index,
            slots_in_epoch: epoch.slots_in_epoch,
            absolute_slot: epoch.absolute_slot,
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
        match self.inner.genesis.read() {
            Ok(genesis_opt) => {
                genesis_opt
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
            Err(_) => Vec::new(),
        }
    }

    pub fn set_current_slot(&self, slot: u64) {
        self.inner.current_slot.store(slot, Ordering::Relaxed);
        self.inner.slot_index.store(slot % 432_000, Ordering::Relaxed);
        self.inner.epoch.store(slot / 432_000, Ordering::Relaxed);
    }

    /// Sync slot to a peer's slot (used when joining network behind)
    pub fn sync_slot(&self, peer_slot: u64) {
        info!("Syncing slot from {} to {}", self.current_slot(), peer_slot);
        self.set_current_slot(peer_slot);
    }

    pub fn set_block_hash(&self, hash: String) {
        if let Ok(mut bh) = self.inner.block_hash.write() {
            *bh = hash;
        }
    }

    pub fn get_last_block_hash(&self) -> String {
        self.inner.block_hash.read().map(|bh| bh.clone()).unwrap_or_default()
    }

    pub fn increment_produced_blocks(&self) {
        self.inner.blocks_produced.fetch_add(1, Ordering::Relaxed);
        self.inner.transaction_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_initial_balances(&self) -> Vec<(String, u64)> {
        // Return genesis initial balances if loaded
        match self.inner.genesis.read() {
            Ok(genesis_opt) => {
                genesis_opt
                    .as_ref()
                    .map(|g| {
                        g.bootstrap_validators
                            .iter()
                            .map(|v| (v.identity_pubkey.clone(), v.stake))
                            .collect()
                    })
                    .unwrap_or_default()
            }
            Err(_) => Vec::new(),
        }
    }

    pub fn get_slot_time_ms(&self) -> u64 {
        self.inner.genesis.read()
            .ok()
            .and_then(|g| g.as_ref().map(|g| g.consensus.slot_time_ms))
            .unwrap_or(400)
    }

    pub fn get_tower_finality(&self) -> u64 {
        self.inner.genesis.read()
            .ok()
            .and_then(|g| g.as_ref().map(|g| g.consensus.tower_finality))
            .unwrap_or(12)
    }

    // ========================================================================
    // Validator Tier Methods
    // ========================================================================

    /// Get the validator's tier
    pub fn tier(&self) -> ValidatorTier {
        self.inner.tier.read().map(|t| *t).unwrap_or(ValidatorTier::Observer)
    }

    /// Get the tier configuration
    pub fn tier_config(&self) -> Option<TierConfig> {
        self.inner.tier_config.read().ok().and_then(|t| t.clone())
    }

    /// Check if this validator is a Full validator
    pub fn is_full(&self) -> bool {
        self.tier() == ValidatorTier::Full
    }

    /// Check if this validator is a Lite validator
    pub fn is_lite(&self) -> bool {
        self.tier() == ValidatorTier::Lite
    }

    /// Check if this validator is an Observer node
    pub fn is_observer(&self) -> bool {
        self.tier() == ValidatorTier::Observer
    }

    /// Get consensus weight for this validator
    pub fn consensus_weight(&self) -> f64 {
        self.tier_config()
            .map(|tc| tc.consensus_weight)
            .unwrap_or(1.0)
    }

    /// Check if this validator can produce blocks
    pub fn can_produce_blocks(&self) -> bool {
        self.tier_config()
            .map(|tc| tc.can_produce_blocks)
            .unwrap_or(true)
    }

    /// Check if this validator can vote on consensus
    pub fn can_vote(&self) -> bool {
        self.tier_config()
            .map(|tc| tc.can_vote)
            .unwrap_or(true)
    }

    /// Set the validator tier (used during initialization)
    pub fn set_tier(&self, tier: ValidatorTier, config: Option<TierConfig>) {
        if let Ok(mut tier_lock) = self.inner.tier.write() {
            *tier_lock = tier;
        }
        if let Ok(mut config_lock) = self.inner.tier_config.write() {
            *config_lock = config;
        }
    }

    // ========================================================================
    // Observer Relay Tracking & Rewards
    // ========================================================================

    /// Get total bytes relayed this epoch
    pub fn relay_bytes(&self) -> u64 {
        self.inner.relay_bytes.load(Ordering::Relaxed)
    }

    /// Add bytes relayed (called when relaying data)
    pub fn add_relay_bytes(&self, bytes: u64) {
        self.inner.relay_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Reset relay counter (called at epoch boundary)
    pub fn reset_relay_bytes(&self) {
        self.inner.relay_bytes.store(0, Ordering::Relaxed);
    }

    /// Calculate FLUX rewards from relay activity for this epoch
    /// Formula: relay_bytes × flux_epoch_relay_rate × node_reputation
    /// Node reputation starts at 0.5, increases by 0.1 per epoch with >95% uptime, caps at 1.0
    pub fn calculate_relay_reward(&self, node_reputation: f64) -> f64 {
        let relay_rate = self
            .tier_config()
            .map(|tc| tc.relay_reward_rate)
            .or_else(|| {
                self.get_genesis()
                    .map(|g| g.rewards.flux_epoch_relay_rate)
            })
            .unwrap_or(0.000001); // Default: 1 FLUX per MB

        let bytes = self.relay_bytes();
        let reward = (bytes as f64) * relay_rate * node_reputation;
        reward
    }

    /// Get relay reward rate from tier config or genesis
    pub fn get_relay_reward_rate(&self) -> f64 {
        self.tier_config()
            .map(|tc| tc.relay_reward_rate)
            .or_else(|| {
                self.get_genesis()
                    .map(|g| g.rewards.flux_epoch_relay_rate)
            })
            .unwrap_or(0.000001)
    }
}
