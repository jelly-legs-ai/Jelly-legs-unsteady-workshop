//! State Database
//!
//! In-memory account state with persistence support.

use aether_core::{Account, Address, GenesisAccount};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sha2::{Digest, Sha256};

pub struct StateDB {
    accounts: Arc<RwLock<HashMap<Address, Account>>>,
    nonce: Arc<RwLock<HashMap<Address, u64>>>,
}

impl StateDB {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            nonce: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn init_from_genesis(&self, genesis_accounts: Vec<GenesisAccount>) {
        // Safely acquire write lock, return early if poisoned
        let mut accounts = match self.accounts.write() {
            Ok(lock) => lock,
            Err(_) => return, // Lock poisoned - skip initialization
        };
        for acc in genesis_accounts {
            let account = Account {
                lamports: acc.lamports,
                owner: [0u8; 32],
                data: acc.data.unwrap_or_default(),
                rent_epoch: 0,
            };
            accounts.insert(acc.address, account);
        }
    }
    
    /// Get account state (async-safe: uses try_read so it won't panic if called from within async context)
    pub async fn get_account(&self, address: &Address) -> Option<Account> {
        self.accounts.read().ok().and_then(|accounts| accounts.get(address).cloned())
    }
    
    /// Get account state (sync version)
    pub fn get_account_sync(&self, address: &Address) -> Option<Account> {
        self.accounts.read().ok().and_then(|accounts| accounts.get(address).cloned())
    }
    
    pub async fn set_account(&self, address: &Address, account: Account) {
        if let Ok(mut accounts) = self.accounts.write() {
            accounts.insert(*address, account);
        }
    }
    
    pub fn set_account_sync(&self, address: &Address, account: Account) {
        if let Ok(mut accounts) = self.accounts.write() {
            accounts.insert(*address, account);
        }
    }
    
    pub async fn credit(&self, address: &Address, amount: u64) -> Result<(), String> {
        let mut accounts = self.accounts.write().map_err(|_| "Lock poisoned")?;
        let account = accounts.get_mut(address).ok_or("Account not found")?;
        account.lamports += amount;
        Ok(())
    }
    
    pub async fn debit(&self, address: &Address, amount: u64) -> Result<(), String> {
        let mut accounts = self.accounts.write().map_err(|_| "Lock poisoned")?;
        let account = accounts.get_mut(address).ok_or("Account not found")?;
        if account.lamports < amount {
            return Err(format!("Insufficient lamports: have {}, need {}", account.lamports, amount));
        }
        account.lamports -= amount;
        Ok(())
    }
    
    pub async fn transfer(&self, from: &Address, to: &Address, amount: u64) -> Result<(), String> {
        self.debit(from, amount).await?;
        self.credit(to, amount).await?;
        Ok(())
    }
    
    pub fn transfer_sync(&self, from: &Address, to: &Address, amount: u64) -> Result<(), String> {
        self.debit_sync(from, amount)?;
        self.credit_sync(to, amount)?;
        Ok(())
    }
    
    pub fn credit_sync(&self, address: &Address, amount: u64) -> Result<(), String> {
        let mut accounts = self.accounts.write().map_err(|_| "Lock poisoned")?;
        let account = accounts.get_mut(address).ok_or("Account not found")?;
        account.lamports += amount;
        Ok(())
    }
    
    pub fn debit_sync(&self, address: &Address, amount: u64) -> Result<(), String> {
        let mut accounts = self.accounts.write().map_err(|_| "Lock poisoned")?;
        let account = accounts.get_mut(address).ok_or("Account not found")?;
        if account.lamports < amount {
            return Err(format!("Insufficient lamports: have {}, need {}", account.lamports, amount));
        }
        account.lamports -= amount;
        Ok(())
    }
    
    pub async fn get_nonce(&self, address: &Address) -> u64 {
        self.nonce.read().ok().and_then(|n| n.get(address).copied()).unwrap_or(0)
    }
    
    #[allow(dead_code)]
    pub async fn increment_nonce(&self, address: &Address) -> u64 {
        // Safely acquire write lock, return 0 if poisoned
        let mut nonce = match self.nonce.write() {
            Ok(lock) => lock,
            Err(_) => return 0, // Lock poisoned
        };
        let new_nonce = *nonce.entry(*address).or_insert(0) + 1;
        if let Some(entry) = nonce.get_mut(address) {
            *entry = new_nonce;
        }
        new_nonce
    }
    
    /// Get total supply (async-safe)
    pub async fn total_supply(&self) -> u64 {
        self.accounts.read().ok().map(|a| a.values().map(|acc| acc.lamports).sum()).unwrap_or(0)
    }
    
    /// Get total supply (sync)
    pub fn total_supply_sync(&self) -> u64 {
        self.accounts.read().ok().map(|a| a.values().map(|acc| acc.lamports).sum()).unwrap_or(0)
    }
    
    #[allow(dead_code)]
    pub fn account_count(&self) -> usize {
        self.accounts.read().map(|a| a.len()).unwrap_or(0)
    }
    
    /// Get all accounts (for persistence)
    pub fn get_all_accounts_sync(&self) -> Vec<(Address, Account)> {
        match self.accounts.read() {
            Ok(accounts) => accounts.iter().map(|(k, v)| (*k, v.clone())).collect(),
            Err(_) => Vec::new(),
        }
    }
    
    pub fn compute_state_root(&self) -> [u8; 32] {
        let accounts = match self.accounts.read() {
            Ok(a) => a,
            Err(_) => return [0u8; 32],
        };
        let mut hasher = Sha256::new();
        let mut addresses: Vec<_> = accounts.keys().collect();
        addresses.sort();
        for addr in addresses {
            if let Some(account) = accounts.get(addr) {
                hasher.update(addr);
                hasher.update(account.lamports.to_le_bytes());
                hasher.update(&account.owner);
            }
        }
        let result = hasher.finalize();
        let mut root = [0u8; 32];
        root.copy_from_slice(&result[..32]);
        root
    }
}

impl Clone for StateDB {
    fn clone(&self) -> Self {
        Self {
            accounts: self.accounts.clone(),
            nonce: self.nonce.clone(),
        }
    }
}

impl Default for StateDB {
    fn default() -> Self {
        Self::new()
    }
}
