//! Account State Database
//! 
//! In-memory account state store with genesis initialization.
//! Accounts hold lamports (AETH tokens), program ownership, and arbitrary data.

use aether_core::{Account, Address, GenesisAccount};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sha2::{Digest, Sha256};

/// State database — thread-safe account storage
pub struct StateDB {
    accounts: Arc<RwLock<HashMap<Address, Account>>>,
    nonce: Arc<RwLock<HashMap<Address, u64>>>, // Nonce per account for TX ordering
}

impl StateDB {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            nonce: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize from genesis accounts
    pub fn init_from_genesis(&self, genesis_accounts: Vec<GenesisAccount>) {
        let mut accounts = self.accounts.write().unwrap();
        for acc in genesis_accounts {
            let mut account = Account::new(acc.address, acc.lamports);
            if let Some(data) = acc.data {
                account.data = data;
            }
            accounts.insert(acc.address, account);
        }
    }
    
    /// Get an account
    pub fn get_account(&self, address: &Address) -> Option<Account> {
        let accounts = self.accounts.read().unwrap();
        accounts.get(address).cloned().filter(|a| a.exists)
    }
    
    /// Set or create an account (used by programs)
    pub fn set_account(&self, account: Account) {
        let mut accounts = self.accounts.write().unwrap();
        accounts.insert(account.address, account);
    }
    
    /// Credit lamports to an account
    pub fn credit(&self, address: &Address, amount: u64) -> Result<(), &'static str> {
        let mut accounts = self.accounts.write().unwrap();
        let account = accounts.get_mut(address).ok_or("Account not found")?;
        if !account.exists {
            return Err("Account deleted");
        }
        account.lamports += amount;
        Ok(())
    }
    
    /// Debit lamports from an account
    pub fn debit(&self, address: &Address, amount: u64) -> Result<(), &'static str> {
        let mut accounts = self.accounts.write().unwrap();
        let account = accounts.get_mut(address).ok_or("Account not found")?;
        if !account.exists {
            return Err("Account deleted");
        }
        if account.lamports < amount {
            return Err("Insufficient lamports");
        }
        account.lamports -= amount;
        Ok(())
    }
    
    /// Create a new account (used by system program for new accounts)
    pub fn create_account(&self, address: Address, lamports: u64) -> Result<Account, &'static str> {
        let mut accounts = self.accounts.write().unwrap();
        if accounts.contains_key(&address) {
            return Err("Account already exists");
        }
        let account = Account::new(address, lamports);
        accounts.insert(address, account.clone());
        Ok(account)
    }
    
    /// Delete an account (mark as not exists)
    pub fn delete_account(&self, address: &Address) {
        let mut accounts = self.accounts.write().unwrap();
        if let Some(account) = accounts.get_mut(address) {
            account.exists = false;
        }
    }
    
    /// Get nonce for an account (for replay protection)
    pub fn get_nonce(&self, address: &Address) -> u64 {
        let nonce = self.nonce.read().unwrap();
        *nonce.get(address).unwrap_or(&0)
    }
    
    /// Increment and return new nonce
    pub fn increment_nonce(&self, address: &Address) -> u64 {
        let mut nonce = self.nonce.write().unwrap();
        let new_nonce = *nonce.entry(*address).or_insert(0) + 1;
        *nonce.get_mut(address).unwrap() = new_nonce;
        new_nonce
    }
    
    /// Compute state root — hash of all account hashes
    pub fn compute_state_root(&self) -> [u8; 32] {
        let accounts = self.accounts.read().unwrap();
        let mut hasher = Sha256::new();
        let mut sorted: Vec<_> = accounts.values().filter(|a| a.exists).collect();
        sorted.sort_by_key(|a| a.address);
        for account in sorted {
            hasher.update(account.hash());
        }
        hasher.finalize().into()
    }
    
    /// Get all accounts (for debugging/inspection)
    pub fn get_all_accounts(&self) -> Vec<Account> {
        let accounts = self.accounts.read().unwrap();
        accounts.values().filter(|a| a.exists).cloned().collect()
    }
    
    /// Get total supply of AETH (sum of all lamports)
    pub fn total_supply(&self) -> u64 {
        let accounts = self.accounts.read().unwrap();
        accounts.values().filter(|a| a.exists).map(|a| a.lamports).sum()
    }
}

impl Default for StateDB {
    fn default() -> Self {
        Self::new()
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
