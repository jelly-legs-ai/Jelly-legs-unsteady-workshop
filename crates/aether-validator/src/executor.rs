//! Transaction Executor
//!
//! Executes Aether transactions against the state database.

use crate::state_db::StateDB;
use aether_core::{AetherTransaction, Address, ExecutionResult, TransactionPayload};
use ed25519_dalek::{Signature, Verifier};
use sha2::{Digest, Sha256};

pub struct Executor {
    state_db: StateDB,
}

impl Executor {
    pub fn new(state_db: StateDB) -> Self {
        Self { state_db }
    }

    pub fn execute(&self, tx: &AetherTransaction) -> ExecutionResult {
        if let Err(e) = self.verify_signature(tx) {
            return ExecutionResult::failure(format!("Invalid signature: {}", e));
        }
        
        match &tx.payload {
            TransactionPayload::Transfer { recipient, amount, nonce: _ } => {
                let recipient = decode_address(recipient);
                self.execute_transfer(&tx.signer, &recipient, *amount)
            }
            TransactionPayload::Stake { validator, amount } => {
                let validator = decode_address(validator);
                self.execute_stake(&tx.signer, &validator, *amount)
            }
            TransactionPayload::Unstake { stake_account, amount } => {
                let stake_account = decode_address(stake_account);
                self.execute_unstake(&tx.signer, &stake_account, *amount)
            }
            TransactionPayload::ClaimRewards { stake_account } => {
                let stake_account = decode_address(stake_account);
                self.execute_claim_rewards(&tx.signer, &stake_account)
            }
            TransactionPayload::CreateNFT { .. } => ExecutionResult::success(),
            TransactionPayload::MintNFT { .. } => ExecutionResult::success(),
            TransactionPayload::TransferNFT { .. } => ExecutionResult::success(),
            TransactionPayload::UpdateMetadata { .. } => ExecutionResult::success(),
        }
    }

    /// Verify Ed25519 signature of a transaction.
    /// The message signed is a SHA-256 digest of the canonical transaction data
    /// (everything except the signature field itself).
    fn verify_signature(&self, tx: &AetherTransaction) -> Result<(), String> {
        // Reconstruct the message that was signed: a digest of the transaction
        // fields (signer, tx_type, payload, fee, slot, timestamp) in a stable order.
        let message = self.transaction_message(tx);
        
        // Parse the public key from the signer field.
        let public_key = ed25519_dalek::PublicKey::from_bytes(&tx.signer)
            .map_err(|e| format!("Invalid signer public key: {}", e))?;
        
        // Parse the signature. The signature must be a valid 64-byte Ed25519 signature.
        let signature = Signature::from_bytes(&tx.signature)
            .map_err(|e| format!("Invalid signature encoding: {}", e))?;
        
        // Ed25519 verification: the message must have been signed by the private
        // key corresponding to the provided public key.
        public_key.verify(&message, &signature)
            .map_err(|_| "Ed25519 signature verification failed".to_string())
    }

    /// Build a canonical 32-byte message digest for a transaction.
    /// This must match what the client signed. We digest all significant fields
    /// in a stable (sorted) order so the same transaction always produces the
    /// same digest.
    fn transaction_message(&self, tx: &AetherTransaction) -> [u8; 32] {
        let mut hasher = Sha256::new();
        // Include all non-signature fields so the digest commits to the full tx.
        hasher.update(&tx.signer);
        hasher.update(tx_type_name(&tx.tx_type).as_bytes());
        hasher.update(payload_bytes(&tx.payload));
        hasher.update(tx.fee.to_le_bytes());
        hasher.update(tx.slot.to_le_bytes());
        hasher.update(tx.timestamp.to_le_bytes());
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result[..32]);
        out
    }

    fn execute_transfer(&self, from: &[u8; 32], to: &[u8; 32], amount: u64) -> ExecutionResult {
        self.state_db
            .transfer_sync(from, to, amount)
            .map(|_| ExecutionResult::success())
            .unwrap_or_else(|e| ExecutionResult::failure(e))
    }

    fn execute_stake(&self, signer: &[u8; 32], validator: &[u8; 32], amount: u64) -> ExecutionResult {
        // Debit lamports from signer first
        if let Err(e) = self.state_db.debit_sync(signer, amount) {
            return ExecutionResult::failure(e);
        }
        // Credit to a stake account derived from signer + validator
        let stake_key = Self::derive_stake_account(signer, validator);
        if let Err(e) = self.state_db.credit_sync(&stake_key, amount) {
            // Rollback: credit back to signer
            let _ = self.state_db.credit_sync(signer, amount);
            return ExecutionResult::failure(e);
        }
        ExecutionResult::success()
    }

    fn derive_stake_account(signer: &[u8; 32], validator: &[u8; 32]) -> Address {
        let mut hasher = Sha256::new();
        hasher.update(b"stake-account");
        hasher.update(signer);
        hasher.update(validator);
        let result = hasher.finalize();
        let mut addr = [0u8; 32];
        addr.copy_from_slice(&result[..32]);
        addr
    }

    fn execute_unstake(&self, signer: &[u8; 32], stake_account: &[u8; 32], amount: u64) -> ExecutionResult {
        // Debit lamports from stake account first
        if let Err(e) = self.state_db.debit_sync(stake_account, amount) {
            return ExecutionResult::failure(e);
        }
        // Credit back to the signer's main account
        if let Err(e) = self.state_db.credit_sync(signer, amount) {
            // Rollback: credit back to stake account
            let _ = self.state_db.credit_sync(stake_account, amount);
            return ExecutionResult::failure(e);
        }
        ExecutionResult::success()
    }

    fn execute_claim_rewards(&self, _signer: &[u8; 32], stake_account: &[u8; 32]) -> ExecutionResult {
        match self.state_db.get_account_sync(stake_account) {
            Some(_) => ExecutionResult::success(),
            None => ExecutionResult::failure("Stake account not found"),
        }
    }
}

impl Clone for Executor {
    fn clone(&self) -> Self {
        Self { state_db: self.state_db.clone() }
    }
}

fn decode_address(s: &str) -> [u8; 32] {
    let decoded = bs58::decode(s).into_vec().unwrap_or_default();
    let mut addr = [0u8; 32];
    addr[..decoded.len().min(32)].copy_from_slice(&decoded[..decoded.len().min(32)]);
    addr
}

/// Serialize a TransactionPayload to canonical bytes for hashing.
fn tx_type_name(tx_type: &aether_core::TransactionType) -> &'static str {
    match tx_type {
        aether_core::TransactionType::Transfer => "transfer",
        aether_core::TransactionType::Stake => "stake",
        aether_core::TransactionType::Unstake => "unstake",
        aether_core::TransactionType::ClaimRewards => "claim_rewards",
        aether_core::TransactionType::CreateNFT => "create_nft",
        aether_core::TransactionType::MintNFT => "mint_nft",
        aether_core::TransactionType::TransferNFT => "transfer_nft",
        aether_core::TransactionType::UpdateMetadata => "update_metadata",
    }
}
    match payload {
        TransactionPayload::Transfer { recipient, amount, nonce } => {
            let mut b = Vec::new();
            b.extend_from_slice(b"transfer");
            b.extend_from_slice(recipient.as_bytes());
            b.extend_from_slice(&amount.to_le_bytes());
            b.extend_from_slice(&nonce.to_le_bytes());
            b
        }
        TransactionPayload::Stake { validator, amount } => {
            let mut b = Vec::new();
            b.extend_from_slice(b"stake");
            b.extend_from_slice(validator.as_bytes());
            b.extend_from_slice(&amount.to_le_bytes());
            b
        }
        TransactionPayload::Unstake { stake_account, amount } => {
            let mut b = Vec::new();
            b.extend_from_slice(b"unstake");
            b.extend_from_slice(stake_account.as_bytes());
            b.extend_from_slice(&amount.to_le_bytes());
            b
        }
        TransactionPayload::ClaimRewards { stake_account } => {
            let mut b = Vec::new();
            b.extend_from_slice(b"claim_rewards");
            b.extend_from_slice(stake_account.as_bytes());
            b
        }
        TransactionPayload::CreateNFT { metadata_url, royalties } => {
            let mut b = Vec::new();
            b.extend_from_slice(b"create_nft");
            b.extend_from_slice(metadata_url.as_bytes());
            b.extend_from_slice(&royalties.to_le_bytes());
            b
        }
        TransactionPayload::MintNFT { nft_id, amount } => {
            let mut b = Vec::new();
            b.extend_from_slice(b"mint_nft");
            b.extend_from_slice(nft_id.as_bytes());
            b.extend_from_slice(&amount.to_le_bytes());
            b
        }
        TransactionPayload::TransferNFT { nft_id, recipient } => {
            let mut b = Vec::new();
            b.extend_from_slice(b"transfer_nft");
            b.extend_from_slice(nft_id.as_bytes());
            b.extend_from_slice(recipient.as_bytes());
            b
        }
        TransactionPayload::UpdateMetadata { nft_id, metadata_url } => {
            let mut b = Vec::new();
            b.extend_from_slice(b"update_metadata");
            b.extend_from_slice(nft_id.as_bytes());
            b.extend_from_slice(metadata_url.as_bytes());
            b
        }
    }
}
