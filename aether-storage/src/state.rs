//! State management - account state tracking with Merkle roots
//!
//! Provides:
//! - Account state storage with address-based indexing
//! - State root computation via Merkle trees
//! - Snapshot support for fast state sync
//! - Atomic state transitions

use aether_core::{Address, Hash};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Account state
#[derive(Debug, Clone)]
pub struct AccountState {
    /// Account address
    pub address: Address,
    /// Balance in lamports
    pub balance: u64,
    /// Number of transactions sent
    pub nonce: u64,
    /// Whether this account is a validator
    pub is_validator: bool,
    /// Stake amount (0 if not a validator)
    pub stake: u64,
    /// Arbitrary data (smart contract storage, etc.)
    pub data: Vec<u8>,
    /// Last updated slot
    pub last_updated_slot: u64,
}

impl AccountState {
    /// Create a new account with zero balance
    pub fn new(address: Address) -> Self {
        Self {
            address,
            balance: 0,
            nonce: 0,
            is_validator: false,
            stake: 0,
            data: Vec::new(),
            last_updated_slot: 0,
        }
    }

    /// Create account with initial balance
    pub fn with_balance(address: Address, balance: u64) -> Self {
        Self {
            address,
            balance,
            nonce: 0,
            is_validator: false,
            stake: 0,
            data: Vec::new(),
            last_updated_slot: 0,
        }
    }

    /// Hash the account state for Merkle tree
    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(&self.address);
        hasher.update(self.balance.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.update((self.is_validator as u8).to_le_bytes());
        hasher.update(self.stake.to_le_bytes());
        hasher.update(&self.data);
        hasher.update(self.last_updated_slot.to_le_bytes());
        hasher.finalize().into()
    }
}

/// State manager error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum StateError {
    #[error("Account not found: {0:?}")]
    AccountNotFound(Address),
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u64, need: u64 },
    #[error("Invalid nonce: expected {expected}, got {actual}")]
    InvalidNonce { expected: u64, actual: u64 },
    #[error("Account already exists: {0:?}")]
    AccountExists(Address),
    #[error("State root mismatch: expected {expected:?}, got {actual:?}")]
    RootMismatch { expected: Hash, actual: Hash },
}

/// State snapshot for fast sync and rollback
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// Snapshot slot
    pub slot: u64,
    /// State root hash
    pub root: Hash,
    /// All accounts at this slot
    pub accounts: HashMap<Address, AccountState>,
    /// Total supply
    pub total_supply: u64,
}

/// State manager - tracks all account states
pub struct StateManager {
    /// Account states indexed by address
    accounts: Arc<RwLock<HashMap<Address, AccountState>>>,
    /// Current state root
    state_root: Arc<RwLock<Hash>>,
    /// Last computed root slot
    root_slot: Arc<RwLock<u64>>,
    /// Total supply tracking
    total_supply: Arc<RwLock<u64>>,
    /// Snapshots for rollback support
    snapshots: Arc<RwLock<Vec<StateSnapshot>>>,
    /// Maximum snapshots to keep
    max_snapshots: usize,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            state_root: Arc::new(RwLock::new([0u8; 32])),
            root_slot: Arc::new(RwLock::new(0)),
            total_supply: Arc::new(RwLock::new(0)),
            snapshots: Arc::new(RwLock::new(Vec::new())),
            max_snapshots: 10,
        }
    }

    /// Create a new account
    pub async fn create_account(&self, address: Address, initial_balance: u64) -> Result<(), StateError> {
        let mut accounts = self.accounts.write().await;
        if accounts.contains_key(&address) {
            return Err(StateError::AccountExists(address));
        }

        let account = AccountState::with_balance(address, initial_balance);
        accounts.insert(address, account);

        *self.total_supply.write().await += initial_balance;
        drop(accounts);

        // Invalidate root
        *self.root_slot.write().await = 0;

        Ok(())
    }

    /// Get account state (read-only)
    pub async fn get_account(&self, address: &Address) -> Result<AccountState, StateError> {
        let accounts = self.accounts.read().await;
        accounts.get(address).cloned().ok_or(StateError::AccountNotFound(*address))
    }

    /// Get account balance
    pub async fn get_balance(&self, address: &Address) -> u64 {
        let accounts = self.accounts.read().await;
        accounts.get(address).map(|a| a.balance).unwrap_or(0)
    }

    /// Transfer tokens between accounts
    pub async fn transfer(
        &self,
        from: &Address,
        to: &Address,
        amount: u64,
        nonce: u64,
    ) -> Result<Hash, StateError> {
        // Validate nonce
        {
            let accounts = self.accounts.read().await;
            let sender = accounts.get(from).ok_or(StateError::AccountNotFound(*from))?;
            if sender.nonce != nonce {
                return Err(StateError::InvalidNonce { expected: sender.nonce, actual: nonce });
            }
            if sender.balance < amount {
                return Err(StateError::InsufficientBalance { have: sender.balance, need: amount });
            }
        }

        let mut accounts = self.accounts.write().await;

        // Debit sender
        let sender = accounts.get_mut(from).unwrap();
        sender.balance -= amount;
        sender.nonce += 1;

        // Credit receiver (create account if needed)
        let receiver = accounts.entry(*to).or_insert_with(|| AccountState::new(*to));
        receiver.balance += amount;

        drop(accounts);

        // Invalidate cached root
        *self.root_slot.write().await = 0;

        // Compute transaction hash
        let mut hasher = Sha256::new();
        hasher.update(from);
        hasher.update(to);
        hasher.update(amount.to_le_bytes());
        hasher.update(nonce.to_le_bytes());
        Ok(hasher.finalize().into())
    }

    /// Set validator status for an account
    pub async fn set_validator(&self, address: &Address, stake: u64) -> Result<(), StateError> {
        let mut accounts = self.accounts.write().await;
        let account = accounts.get_mut(address).ok_or(StateError::AccountNotFound(*address))?;
        account.is_validator = true;
        account.stake = stake;
        drop(accounts);

        *self.root_slot.write().await = 0;
        Ok(())
    }

    /// Update account data (smart contract storage)
    pub async fn update_account_data(&self, address: &Address, data: Vec<u8>) -> Result<(), StateError> {
        let mut accounts = self.accounts.write().await;
        let account = accounts.get_mut(address).ok_or(StateError::AccountNotFound(*address))?;
        account.data = data;
        drop(accounts);

        *self.root_slot.write().await = 0;
        Ok(())
    }

    /// Compute the state root (Merkle root of all accounts)
    ///
    /// Uses a simplified Merkle tree: sort account hashes, pair them up,
    /// and hash pairs until we reach a single root.
    pub async fn compute_state_root(&self) -> Hash {
        let accounts = self.accounts.read().await;

        if accounts.is_empty() {
            return [0u8; 32];
        }

        // Collect and sort account hashes
        let mut hashes: Vec<Hash> = accounts.values().map(|a| a.hash()).collect();
        hashes.sort_by(|a, b| a.cmp(b));

        // Build Merkle tree
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]); // Duplicate odd node
                }
                next_level.push(hasher.finalize().into());
            }
            hashes = next_level;
        }

        hashes[0]
    }

    /// Get the state root, recomputing if stale
    pub async fn state_root(&self) -> Hash {
        let current_slot = *self.root_slot.read().await;
        if current_slot == 0 {
            let root = self.compute_state_root().await;
            *self.state_root.write().await = root;
            root
        } else {
            *self.state_root.read().await
        }
    }

    /// Mark state root as valid for a given slot
    pub async fn commit_slot(&self, slot: u64) -> Hash {
        let root = self.compute_state_root().await;
        *self.state_root.write().await = root;
        *self.root_slot.write().await = slot;
        root
    }

    /// Take a snapshot of current state
    pub async fn snapshot(&self, slot: u64) -> StateSnapshot {
        let accounts = self.accounts.read().await;
        let root = self.state_root.read().await;

        StateSnapshot {
            slot,
            root: *root,
            accounts: accounts.clone(),
            total_supply: *self.total_supply.read().await,
        }
    }

    /// Save a snapshot for rollback support
    pub async fn save_snapshot(&self, slot: u64) {
        let snapshot = self.snapshot(slot).await;
        let mut snapshots = self.snapshots.write().await;
        snapshots.push(snapshot);

        // Keep only the last N snapshots
        while snapshots.len() > self.max_snapshots {
            snapshots.remove(0);
        }
    }

    /// Roll back to a previous snapshot
    pub async fn rollback(&self, slot: u64) -> Result<(), StateError> {
        let snapshots = self.snapshots.read().await;
        let snapshot = snapshots.iter()
            .find(|s| s.slot == slot)
            .ok_or_else(|| StateError::AccountNotFound([0u8; 32]))? // Reuse error type
            .clone();
        drop(snapshots);

        let mut accounts = self.accounts.write().await;
        accounts.clear();
        for (addr, state) in snapshot.accounts {
            accounts.insert(addr, state);
        }
        *self.total_supply.write().await = snapshot.total_supply;
        *self.state_root.write().await = snapshot.root;
        *self.root_slot.write().await = snapshot.slot;

        info!("Rolled back state to slot {}", slot);
        Ok(())
    }

    /// Get total supply
    pub async fn total_supply(&self) -> u64 {
        *self.total_supply.read().await
    }

    /// Get total number of accounts
    pub async fn account_count(&self) -> usize {
        self.accounts.read().await.len()
    }

    /// Get validator accounts
    pub async fn get_validators(&self) -> Vec<AccountState> {
        let accounts = self.accounts.read().await;
        accounts.values().filter(|a| a.is_validator).cloned().collect()
    }

    /// Get state statistics
    pub async fn stats(&self) -> StateStats {
        let accounts = self.accounts.read().await;
        let total_balance: u64 = accounts.values().map(|a| a.balance).sum();
        let validator_count = accounts.values().filter(|a| a.is_validator).count();
        let total_stake: u64 = accounts.values().map(|a| a.stake).sum();

        StateStats {
            account_count: accounts.len(),
            validator_count,
            total_balance,
            total_stake,
            total_supply: *self.total_supply.read().await,
            state_root: *self.state_root.read().await,
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// State statistics
#[derive(Debug, Clone)]
pub struct StateStats {
    pub account_count: usize,
    pub validator_count: usize,
    pub total_balance: u64,
    pub total_stake: u64,
    pub total_supply: u64,
    pub state_root: Hash,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_account() {
        let state = StateManager::new();
        let addr = [1u8; 32];
        state.create_account(addr, 1000).await.unwrap();

        let account = state.get_account(&addr).await.unwrap();
        assert_eq!(account.balance, 1000);
    }

    #[tokio::test]
    async fn test_duplicate_account() {
        let state = StateManager::new();
        let addr = [1u8; 32];
        state.create_account(addr, 1000).await.unwrap();

        let result = state.create_account(addr, 500).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transfer() {
        let state = StateManager::new();
        let from = [1u8; 32];
        let to = [2u8; 32];

        state.create_account(from, 1000).await.unwrap();
        state.create_account(to, 0).await.unwrap();

        state.transfer(&from, &to, 300, 0).await.unwrap();

        assert_eq!(state.get_balance(&from).await, 700);
        assert_eq!(state.get_balance(&to).await, 300);
    }

    #[tokio::test]
    async fn test_transfer_insufficient_balance() {
        let state = StateManager::new();
        let from = [1u8; 32];
        let to = [2u8; 32];

        state.create_account(from, 100).await.unwrap();
        state.create_account(to, 0).await.unwrap();

        let result = state.transfer(&from, &to, 200, 0).await;
        assert!(matches!(result, Err(StateError::InsufficientBalance { .. })));
    }

    #[tokio::test]
    async fn test_transfer_invalid_nonce() {
        let state = StateManager::new();
        let from = [1u8; 32];
        let to = [2u8; 32];

        state.create_account(from, 1000).await.unwrap();
        state.create_account(to, 0).await.unwrap();

        let result = state.transfer(&from, &to, 100, 5).await;
        assert!(matches!(result, Err(StateError::InvalidNonce { .. })));
    }

    #[tokio::test]
    async fn test_state_root() {
        let state = StateManager::new();
        let addr = [1u8; 32];

        // Empty state root
        let root1 = state.state_root().await;

        state.create_account(addr, 1000).await.unwrap();
        let root2 = state.state_root().await;

        // Roots should differ after state change
        assert_ne!(root1, root2);
    }

    #[tokio::test]
    async fn test_snapshot_and_rollback() {
        let state = StateManager::new();
        let addr = [1u8; 32];

        state.create_account(addr, 1000).await.unwrap();
        state.commit_slot(1).await;
        state.save_snapshot(1).await;

        // Transfer changes state
        let to = [2u8; 32];
        state.create_account(to, 0).await.unwrap();
        state.transfer(&addr, &to, 500, 0).await.unwrap();
        state.commit_slot(2).await;

        assert_eq!(state.get_balance(&addr).await, 500);

        // Rollback to slot 1
        state.rollback(1).await.unwrap();
        assert_eq!(state.get_balance(&addr).await, 1000);
    }

    #[tokio::test]
    async fn test_validator() {
        let state = StateManager::new();
        let addr = [1u8; 32];

        state.create_account(addr, 1_000_000).await.unwrap();
        state.set_validator(&addr, 500_000).await.unwrap();

        let account = state.get_account(&addr).await.unwrap();
        assert!(account.is_validator);
        assert_eq!(account.stake, 500_000);
    }
}