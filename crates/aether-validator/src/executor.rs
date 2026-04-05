//! Transaction Executor
//!
//! Executes Aether transactions against the state database.

use crate::state_db::StateDB;
use aether_core::{AetherTransaction, ExecutionResult, TransactionPayload};

pub struct Executor {
    state_db: StateDB,
}

impl Executor {
    pub fn new(state_db: StateDB) -> Self {
        Self { state_db }
    }

    pub fn execute(&self, tx: &AetherTransaction) -> ExecutionResult {
        if !self.verify_signature(tx) {
            return ExecutionResult::failure("Invalid signature");
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

    fn verify_signature(&self, _tx: &AetherTransaction) -> bool {
        // TODO: In production, use ed25519_dalek to verify signature
        true
    }

    fn execute_transfer(&self, from: &[u8; 32], to: &[u8; 32], amount: u64) -> ExecutionResult {
        self.state_db
            .transfer(from, to, amount)
            .map(|_| ExecutionResult::success())
            .unwrap_or_else(|e| ExecutionResult::failure(e))
    }

    fn execute_stake(&self, signer: &[u8; 32], _validator: &[u8; 32], amount: u64) -> ExecutionResult {
        match self.state_db.get_account(signer) {
            Some(account) => {
                if account.lamports < amount {
                    ExecutionResult::failure("Insufficient lamports for staking")
                } else {
                    ExecutionResult::success()
                }
            }
            None => ExecutionResult::failure("Account not found"),
        }
    }

    fn execute_unstake(&self, _signer: &[u8; 32], stake_account: &[u8; 32], amount: u64) -> ExecutionResult {
        match self.state_db.get_account(stake_account) {
            Some(account) => {
                if account.lamports < amount {
                    ExecutionResult::failure("Insufficient staked lamports")
                } else {
                    ExecutionResult::success()
                }
            }
            None => ExecutionResult::failure("Stake account not found"),
        }
    }

    fn execute_claim_rewards(&self, _signer: &[u8; 32], stake_account: &[u8; 32]) -> ExecutionResult {
        match self.state_db.get_account(stake_account) {
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
