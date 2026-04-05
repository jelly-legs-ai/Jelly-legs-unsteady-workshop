//! Transaction Execution Engine
//! 
//! Executes signed transactions against the state DB.
//! Produces execution results and state changes.

use crate::state_db::StateDB;
use aether_core::*;
use sha2::{Digest, Sha256};

/// Transaction executor
pub struct Executor {
    state_db: StateDB,
}

impl Executor {
    pub fn new(state_db: StateDB) -> Self {
        Self { state_db }
    }
    
    /// Execute a signed AetherTransaction
    pub fn execute(&self, tx: &AetherTransaction) -> ExecutionResult {
        // Basic validation - signature verification stub
        if !self.verify_signature(tx) {
            return ExecutionResult::failure("Invalid signature");
        }
        
        // Check nonce for replay protection
        let expected_nonce = self.state_db.get_nonce(&tx.signer);
        if let TransactionPayload::Transfer { nonce, .. } = &tx.payload {
            if nonce < &expected_nonce {
                return ExecutionResult::failure("Nonce too low: replay protection");
            }
        }
        
        // Execute by type
        let result = match &tx.payload {
            TransactionPayload::Transfer { recipient, amount, nonce: _ } => {
                let recipient_addr = decode_bs58_address(recipient);
                self.execute_transfer(tx, &recipient_addr, *amount)
            }
            TransactionPayload::Stake { validator, amount, tier } => {
                let validator_addr = decode_bs58_address(validator);
                self.execute_stake(tx, &validator_addr, *amount, tier)
            }
            TransactionPayload::Unstake { position_index, amount } => {
                self.execute_unstake(tx, *position_index, *amount)
            }
            TransactionPayload::ClaimRewards { position_index } => {
                self.execute_claim_rewards(tx, *position_index)
            }
            TransactionPayload::CreateNFT { name, metadata_uri, supply } => {
                self.execute_create_nft(tx, name, metadata_uri, *supply)
            }
            TransactionPayload::MintNFT { nft_id, amount } => {
                let nft_id_arr = decode_bs58_address(nft_id);
                self.execute_mint_nft(tx, &nft_id_arr, *amount)
            }
            TransactionPayload::TransferNFT { nft_id, recipient } => {
                let nft_id_arr = decode_bs58_address(nft_id);
                let recipient_addr = decode_bs58_address(recipient);
                self.execute_transfer_nft(tx, &nft_id_arr, &recipient_addr)
            }
            TransactionPayload::UpdateMetadata { nft_id, metadata_uri } => {
                let nft_id_arr = decode_bs58_address(nft_id);
                self.execute_update_metadata(tx, &nft_id_arr, metadata_uri)
            }
            TransactionPayload::Delegate { validator, amount } => {
                let validator_addr = decode_bs58_address(validator);
                self.execute_delegate(tx, &validator_addr, *amount)
            }
            TransactionPayload::Vote { slot, block_hash } => {
                let block_hash_arr = decode_bs58_address(block_hash);
                self.execute_vote(tx, *slot, &block_hash_arr)
            }
        };
        
        // If successful, increment nonce
        if result.success {
            self.state_db.increment_nonce(&tx.signer);
            // Deduct fee
            if tx.fee > 0 {
                let _ = self.state_db.debit(&tx.signer, tx.fee);
            }
        }
        
        result
    }
    
    fn verify_signature(&self, tx: &AetherTransaction) -> bool {
        // MVP: Stub signature verification
        true
    }
    
    fn execute_transfer(&self, tx: &AetherTransaction, recipient: &Address, amount: u64) -> ExecutionResult {
        let mut changes = Vec::new();
        
        let sender_old = self.state_db.get_account(&tx.signer).map(|a| a.lamports).unwrap_or(0);
        let recipient_old = self.state_db.get_account(recipient).map(|a| a.lamports).unwrap_or(0);
        
        if let Err(e) = self.state_db.debit(&tx.signer, amount) {
            return ExecutionResult::failure_with(e.to_string(), 100);
        }
        
        let recipient_account = self.state_db.get_account(recipient);
        if recipient_account.is_none() {
            let _ = self.state_db.create_account(*recipient, 0);
        }
        if let Err(e) = self.state_db.credit(recipient, amount) {
            return ExecutionResult::failure_with(format!("Credit failed: {}", e), 100);
        }
        
        changes.push(StateChange::lamports(tx.signer, sender_old, sender_old - amount));
        changes.push(StateChange::lamports(*recipient, recipient_old, recipient_old + amount));
        
        ExecutionResult::success_with(changes, 100)
    }
    
    fn execute_stake(&self, tx: &AetherTransaction, validator: &Address, amount: u64, tier: &str) -> ExecutionResult {
        let min_stake = match tier {
            "full" => 10_000 * 100_000_000,
            "lite" => 1_000 * 100_000_000,
            _ => 0,
        };
        
        if amount < min_stake {
            return ExecutionResult::failure_with(
                format!("Below minimum stake for {} tier", tier),
                100
            );
        }
        
        if let Err(e) = self.state_db.debit(&tx.signer, amount) {
            return ExecutionResult::failure_with(e.to_string(), 100);
        }
        
        ExecutionResult::success_with(vec![], 200)
    }
    
    fn execute_unstake(&self, tx: &AetherTransaction, _position_index: usize, amount: u64) -> ExecutionResult {
        let old_balance = self.state_db.get_account(&tx.signer).map(|a| a.lamports).unwrap_or(0);
        let _ = self.state_db.credit(&tx.signer, amount);
        
        ExecutionResult::success_with(vec![StateChange::lamports(tx.signer, old_balance, old_balance + amount)], 200)
    }
    
    fn execute_claim_rewards(&self, tx: &AetherTransaction, _position_index: usize) -> ExecutionResult {
        ExecutionResult::success_with(vec![], 150)
    }
    
    fn execute_create_nft(&self, tx: &AetherTransaction, name: &str, metadata_uri: &str, supply: u64) -> ExecutionResult {
        let mut hasher = Sha256::new();
        hasher.update(&tx.signature);
        hasher.update(name.as_bytes());
        let nft_id: [u8; 32] = hasher.finalize().into();
        
        let mut nft_data = Vec::new();
        nft_data.extend_from_slice(&tx.signer);
        nft_data.extend_from_slice(name.len().to_le_bytes());
        nft_data.extend_from_slice(name.as_bytes());
        nft_data.extend_from_slice(metadata_uri.len().to_le_bytes());
        nft_data.extend_from_slice(metadata_uri.as_bytes());
        nft_data.extend_from_slice(&supply.to_le_bytes());
        
        let mut nft_account = Account::new(nft_id, 0);
        nft_account.data = nft_data;
        nft_account.owner = tx.signer;
        
        self.state_db.set_account(nft_account);
        
        ExecutionResult::success_with(vec![], 300)
    }
    
    fn execute_mint_nft(&self, tx: &AetherTransaction, nft_id: &[u8; 32], _amount: u64) -> ExecutionResult {
        let nft_account = self.state_db.get_account(nft_id);
        if nft_account.is_none() {
            return ExecutionResult::failure_with("NFT not found".to_string(), 100);
        }
        ExecutionResult::success_with(vec![], 200)
    }
    
    fn execute_transfer_nft(&self, tx: &AetherTransaction, nft_id: &[u8; 32], _recipient: &Address) -> ExecutionResult {
        let nft_account = self.state_db.get_account(nft_id);
        if nft_account.is_none() {
            return ExecutionResult::failure_with("NFT not found".to_string(), 100);
        }
        ExecutionResult::success_with(vec![], 200)
    }
    
    fn execute_update_metadata(&self, tx: &AetherTransaction, nft_id: &[u8; 32], _metadata_uri: &str) -> ExecutionResult {
        let nft_account = self.state_db.get_account(nft_id);
        if nft_account.is_none() {
            return ExecutionResult::failure_with("NFT not found".to_string(), 100);
        }
        ExecutionResult::success_with(vec![], 150)
    }
    
    fn execute_delegate(&self, tx: &AetherTransaction, validator: &Address, amount: u64) -> ExecutionResult {
        if let Err(e) = self.state_db.debit(&tx.signer, amount) {
            return ExecutionResult::failure_with(e.to_string(), 150);
        }
        ExecutionResult::success_with(vec![], 150)
    }
    
    fn execute_vote(&self, tx: &AetherTransaction, slot: u64, _block_hash: &[u8; 32]) -> ExecutionResult {
        ExecutionResult::success_with(vec![], 100)
    }
}

impl Clone for Executor {
    fn clone(&self) -> Self {
        Self {
            state_db: self.state_db.clone(),
        }
    }
}

/// Decode a base58-encoded address string to a 32-byte array
fn decode_bs58_address(encoded: &str) -> Address {
    let bytes = bs58::decode(encoded).into_vec().unwrap_or_default();
    let mut addr = [0u8; 32];
    addr.copy_from_slice(&bytes[..32.min(bytes.len())]);
    addr
}
