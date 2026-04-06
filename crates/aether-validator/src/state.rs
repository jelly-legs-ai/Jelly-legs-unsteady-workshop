//! Validator state management
//!
//! In-memory state tracking for the running validator.

use crate::genesis::{GenesisBlock, load_genesis_from_file, ValidatorTier, TierConfig};
use crate::keypair::ValidatorIdentity;
use crate::{BlockProduction, EpochInfo, ValidatorInfo, VoteAccountInfo};
use aether_consensus::staking::StakingPool;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tracing::info;

/// Staking position response
#[derive(Debug, Serialize)]
pub struct StakingPositionResponse {
    pub stake_id: usize,
    pub owner: String,
    pub amount: u64,
    pub accumulated_rewards: u64,
    pub total_value: u64,
    pub start_epoch: u64,
    pub unlock_epoch: u64,
    pub is_locked: bool,
    pub remaining_epochs: u64,
    pub delegated_to: Option<String>,
    pub pending_withdrawal: bool,
    pub annual_reward_rate_bps: u64,
}

/// Staking summary for an address
#[derive(Debug, Serialize)]
pub struct StakingSummaryResponse {
    pub address: String,
    pub total_staked: u64,
    pub total_rewards: u64,
    pub active_positions: usize,
    pub positions: Vec<StakingPositionResponse>,
}

/// Validator staking info
#[derive(Debug, Serialize)]
pub struct ValidatorStakingInfo {
    pub identity: String,
    pub total_delegated: u64,
    pub delegator_count: usize,
    pub commission_bps: u64,
    pub active: bool,
}

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
    
    // Staking pool
    staking_pool: RwLock<StakingPool>,
    
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
                staking_pool: RwLock::new(StakingPool::new(0)),
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
                staking_pool: RwLock::new(StakingPool::new(0)),
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
        // Use genesis epoch duration for slot index and epoch calculation
        let slots_per_epoch = self.get_genesis()
            .map(|g| g.rewards.epoch_duration)
            .unwrap_or(432_000);
        self.inner.slot_index.store(slot % slots_per_epoch, Ordering::Relaxed);
        self.inner.epoch.store(slot / slots_per_epoch, Ordering::Relaxed);
        self.inner.transaction_count.fetch_add(1, Ordering::Relaxed);
        
        // Note: blocks_produced is now tracked by block_producer.increment_produced_blocks()
        // to avoid double-counting. Vote count is incremented per-block there as well.
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
        // Read epoch duration from genesis config, default to 432_000 if not available
        let slots_in_epoch = self.get_genesis()
            .map(|g| g.rewards.epoch_duration)
            .unwrap_or(432_000);
        EpochInfo {
            epoch: slot / slots_in_epoch,
            slot_index: slot % slots_in_epoch,
            slots_in_epoch,
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

    /// Get slots per epoch from genesis config
    pub fn get_slots_per_epoch(&self) -> u64 {
        self.get_genesis()
            .map(|g| g.rewards.epoch_duration)
            .unwrap_or(432_000)
    }

    pub fn get_connected_validators(&self) -> Vec<ValidatorInfo> {
        // Return genesis bootstrap validators + any connected peer validators
        let mut validators = Vec::new();

        // Include genesis bootstrap validators with real stake data
        if let Some(genesis) = self.get_genesis() {
            for bv in &genesis.bootstrap_validators {
                validators.push(ValidatorInfo {
                    identity_pubkey: bv.identity_pubkey.clone(),
                    activated_stake: bv.stake,
                    commission: bv.commission,
                    active: bv.active,
                });
            }
        }

        // If no genesis validators, include self from identity
        if validators.is_empty() {
            let identity_str = self.inner.identity.read()
                .ok()
                .and_then(|id| id.as_ref().map(|i| i.pubkey().to_string()))
                .unwrap_or_else(|| "Unknown".to_string());
            validators.push(ValidatorInfo {
                identity_pubkey: identity_str,
                activated_stake: 0,
                commission: 10,
                active: true,
            });
        }

        // Include connected peer validators (deduplicated)
        if let Ok(peers) = self.inner.peer_pubkeys.read() {
            for peer in peers.iter() {
                if !validators.iter().any(|v| &v.identity_pubkey == peer) {
                    validators.push(ValidatorInfo {
                        identity_pubkey: peer.clone(),
                        activated_stake: 0,
                        commission: 10,
                        active: true,
                    });
                }
            }
        }

        validators
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
    
    /// Get the validator identity public key as bytes, if available.
    pub fn identity_pubkey_bytes(&self) -> Option<[u8; 32]> {
        let identity = self.inner.identity.read().ok()?;
        let identity = identity.as_ref()?;
        let pubkey_str = identity.pubkey();
        let decoded = bs58::decode(&pubkey_str).into_vec().ok()?;
        if decoded.len() < 32 {
            return None;
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&decoded[..32]);
        Some(key)
    }
    
    /// Get the validator identity public key as a bs58 string, if available.
    pub fn identity_pubkey(&self) -> Option<String> {
        let identity = self.inner.identity.read().ok()?;
        let identity = identity.as_ref()?;
        Some(identity.pubkey())
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

    // ========================================================================
    // Staking Operations
    // ========================================================================

    /// Create a new stake
    pub fn create_stake(&self, owner: [u8; 32], amount: u64, delegate_to: Option<[u8; 32]>) -> Result<usize, String> {
        let mut pool = self.inner.staking_pool.write().map_err(|_| "Lock poisoned".to_string())?;
        let stake_id = pool.stake(owner, amount).map_err(|e| e.to_string())?;
        
        // If delegation target provided, delegate immediately
        if let Some(validator) = delegate_to {
            pool.delegate(stake_id, validator).map_err(|e| e.to_string())?;
        }
        
        Ok(stake_id)
    }

    /// Initiate unstake for a stake position
    pub fn initiate_unstake(&self, owner: [u8; 32], stake_id: usize) -> Result<u64, String> {
        let mut pool = self.inner.staking_pool.write().map_err(|_| "Lock poisoned".to_string())?;
        
        // Verify ownership
        let stake = pool.get_stake(stake_id).ok_or("Stake not found")?;
        if stake.owner != owner {
            return Err("Not stake owner".to_string());
        }
        
        let unlock_epoch = pool.initiate_withdrawal(stake_id).map_err(|e| e.to_string())?;
        Ok(unlock_epoch)
    }

    /// Complete withdrawal after lock period
    pub fn complete_withdrawal(&self, owner: [u8; 32], stake_id: usize) -> Result<u64, String> {
        let mut pool = self.inner.staking_pool.write().map_err(|_| "Lock poisoned".to_string())?;
        
        // Verify ownership
        let stake = pool.get_stake(stake_id).ok_or("Stake not found")?;
        if stake.owner != owner {
            return Err("Not stake owner".to_string());
        }
        
        pool.complete_withdrawal(stake_id).map_err(|e| e.to_string())
    }

    /// Claim accumulated rewards
    pub fn claim_rewards(&self, owner: [u8; 32], stake_id: usize) -> Result<u64, String> {
        let mut pool = self.inner.staking_pool.write().map_err(|_| "Lock poisoned".to_string())?;
        
        // Verify ownership
        let stake = pool.get_stake(stake_id).ok_or("Stake not found")?;
        if stake.owner != owner {
            return Err("Not stake owner".to_string());
        }
        
        let rewards = stake.accumulated_rewards;
        pool.stakes.get_mut(stake_id).map(|s| s.accumulated_rewards = 0);
        Ok(rewards)
    }

    /// Delegate stake to a validator
    pub fn delegate_stake(&self, owner: [u8; 32], stake_id: usize, validator: [u8; 32]) -> Result<(), String> {
        let mut pool = self.inner.staking_pool.write().map_err(|_| "Lock poisoned".to_string())?;
        
        // Verify ownership
        let stake = pool.get_stake(stake_id).ok_or("Stake not found")?;
        if stake.owner != owner {
            return Err("Not stake owner".to_string());
        }
        
        pool.delegate(stake_id, validator).map_err(|e| e.to_string())
    }

    /// Get staking positions for an address (as JSON-compatible values)
    pub fn get_staking_positions(&self, owner: &str) -> Vec<serde_json::Value> {
        let owner_bytes = match bs58::decode(owner).into_vec() {
            Ok(bytes) => {
                let mut arr = [0u8; 32];
                if bytes.len() >= 32 {
                    arr.copy_from_slice(&bytes[..32]);
                } else {
                    arr[..bytes.len()].copy_from_slice(&bytes);
                }
                arr
            }
            Err(_) => return Vec::new(),
        };

        let pool = match self.inner.staking_pool.read() {
            Ok(pool) => pool,
            Err(_) => return Vec::new(),
        };

        let positions = pool.get_stakes_by_owner(&owner_bytes);
        positions.into_iter().map(|(id, stake)| {
            serde_json::json!({
                "stake_id": id,
                "owner": bs58::encode(stake.owner).into_string(),
                "amount": stake.amount,
                "accumulated_rewards": stake.accumulated_rewards,
                "start_epoch": stake.start_epoch,
                "unlock_epoch": stake.unlock_epoch,
                "is_locked": stake.is_locked(pool.current_epoch),
                "delegated_to": stake.delegated_to.map(|v| bs58::encode(v).into_string()),
                "pending_withdrawal": stake.pending_withdrawal
            })
        }).collect()
    }

    /// Get staking summary for an address
    pub fn get_staking_summary(&self, owner: &str) -> StakingSummaryResponse {
        let owner_bytes = match bs58::decode(owner).into_vec() {
            Ok(bytes) => {
                let mut arr = [0u8; 32];
                if bytes.len() >= 32 {
                    arr.copy_from_slice(&bytes[..32]);
                } else {
                    arr[..bytes.len()].copy_from_slice(&bytes);
                }
                arr
            }
            Err(_) => {
                return StakingSummaryResponse {
                    address: owner.to_string(),
                    total_staked: 0,
                    total_rewards: 0,
                    active_positions: 0,
                    positions: Vec::new(),
                };
            }
        };

        let pool = match self.inner.staking_pool.read() {
            Ok(pool) => pool,
            Err(_) => {
                return StakingSummaryResponse {
                    address: owner.to_string(),
                    total_staked: 0,
                    total_rewards: 0,
                    active_positions: 0,
                    positions: Vec::new(),
                };
            }
        };

        let positions = pool.get_stakes_by_owner(&owner_bytes);
        let total_staked: u64 = positions.iter().map(|(_, s)| s.amount).sum();
        let total_rewards: u64 = positions.iter().map(|(_, s)| s.accumulated_rewards).sum();
        let active_count = positions.iter().filter(|(_, s)| !s.pending_withdrawal).count();

        let position_responses: Vec<StakingPositionResponse> = positions.into_iter().map(|(id, stake)| {
            StakingPositionResponse {
                stake_id: id,
                owner: bs58::encode(stake.owner).into_string(),
                amount: stake.amount,
                accumulated_rewards: stake.accumulated_rewards,
                total_value: stake.amount + stake.accumulated_rewards,
                start_epoch: stake.start_epoch,
                unlock_epoch: stake.unlock_epoch,
                is_locked: stake.is_locked(pool.current_epoch),
                remaining_epochs: stake.remaining_lock_epochs(pool.current_epoch),
                delegated_to: stake.delegated_to.map(|v| bs58::encode(v).into_string()),
                pending_withdrawal: stake.pending_withdrawal,
                annual_reward_rate_bps: pool.reward_rate_bps,
            }
        }).collect();

        StakingSummaryResponse {
            address: owner.to_string(),
            total_staked,
            total_rewards,
            active_positions: active_count,
            positions: position_responses,
        }
    }

    /// Get staking pool info
    pub fn get_staking_pool_info(&self) -> serde_json::Value {
        let pool = match self.inner.staking_pool.read() {
            Ok(pool) => pool,
            Err(_) => {
                return serde_json::json!({
                    "error": "Failed to access staking pool"
                });
            }
        };

        serde_json::json!({
            "current_epoch": pool.current_epoch,
            "total_staked": pool.total_staked,
            "total_rewards": pool.total_rewards,
            "reward_rate_bps": pool.reward_rate_bps,
            "stake_count": pool.stakes.len()
        })
    }

    /// Get validator staking info
    pub fn get_validator_staking_info(&self, validator: &str) -> ValidatorStakingInfo {
        let validator_bytes = match bs58::decode(validator).into_vec() {
            Ok(bytes) => {
                let mut arr = [0u8; 32];
                if bytes.len() >= 32 {
                    arr.copy_from_slice(&bytes[..32]);
                } else {
                    arr[..bytes.len()].copy_from_slice(&bytes);
                }
                arr
            }
            Err(_) => {
                return ValidatorStakingInfo {
                    identity: validator.to_string(),
                    total_delegated: 0,
                    delegator_count: 0,
                    commission_bps: 0,
                    active: false,
                };
            }
        };

        let pool = match self.inner.staking_pool.read() {
            Ok(pool) => pool,
            Err(_) => {
                return ValidatorStakingInfo {
                    identity: validator.to_string(),
                    total_delegated: 0,
                    delegator_count: 0,
                    commission_bps: 0,
                    active: false,
                };
            }
        };

        let delegated_stakes: Vec<_> = pool.stakes.iter()
            .filter(|s| s.delegated_to == Some(validator_bytes))
            .collect();

        let total_delegated: u64 = delegated_stakes.iter().map(|s| s.amount).sum();
        let delegator_count = delegated_stakes.len();

        ValidatorStakingInfo {
            identity: validator.to_string(),
            total_delegated,
            delegator_count,
            commission_bps: 0, // Would come from validator registry
            active: true,
        }
    }
}
