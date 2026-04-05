//! State Database
//!
//! In-memory account state with persistence support.

use aether_core::{Account, Address, GenesisAccount};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
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
        let mut accounts = self.accounts.blocking_write();
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
    
    pub fn get_account(&self, address: &Address) -> Option<Account> {
        let accounts = self.accounts.blocking_read();
        accounts.get(address).cloned()
    }
    
    pub fn set_account(&self, address: &Address, account: Account) {
        let mut accounts = self.accounts.blocking_write();
        accounts.insert(*address, account);
    }
    
    pub fn credit(&self, address: &Address, amount: u64) -> Result<(), String> {
        let mut accounts = self.accounts.blocking_write();
        let account = accounts.get_mut(address).ok_or("Account not found")?;
        account.lamports += amount;
        Ok(())
    }
    
    pub fn debit(&self, address: &Address, amount: u64) -> Result<(), String> {
        let mut accounts = self.accounts.blocking_write();
        let account = accounts.get_mut(address).ok_or("Account not found")?;
        if account.lamports < amount {
            return Err(format!("Insufficient lamports: have {}, need {}", account.lamports, amount));
        }
        account.lamports -= amount;
        Ok(())
    }
    
    pub fn transfer(&self, from: &Address, to: &Address, amount: u64) -> Result<(), String> {
        self.debit(from, amount)?;
        self.credit(to, amount)?;
        Ok(())
    }
    
    pub fn get_nonce(&self, address: &Address) -> u64 {
        let nonce = self.nonce.blocking_read();
        *nonce.get(address).unwrap_or(&0)
    }
    
    #[allow(dead_code)]
    pub fn increment_nonce(&self, address: &Address) -> u64 {
        let mut nonce = self.nonce.blocking_write();
        let new_nonce = *nonce.entry(*address).or_insert(0) + 1;
        *nonce.get_mut(address).unwrap() = new_nonce;
        new_nonce
    }
    
    pub fn total_supply(&self) -> u64 {
        let accounts = self.accounts.blocking_read();
        accounts.values().map(|a| a.lamports).sum()
    }
    
    #[allow(dead_code)]
    pub fn account_count(&self) -> usize {
        let accounts = self.accounts.blocking_read();
        accounts.len()
    }
    
    pub fn compute_state_root(&self) -> [u8; 32] {
        let accounts = self.accounts.blocking_read();
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
