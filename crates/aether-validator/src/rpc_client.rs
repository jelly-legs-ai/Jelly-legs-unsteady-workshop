//! Simple RPC client for validator status queries
//!
//! Provides a minimal JSON-RPC client to query local validator state.

use serde::{Deserialize, Serialize};

/// Epoch information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpochInfo {
    pub epoch: u64,
    pub slot_index: u64,
    pub slots_in_epoch: u64,
    pub absolute_slot: u64,
    pub transaction_count: u64,
}

/// Block production statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockProduction {
    pub blocks_produced: u64,
    pub entries_produced: u64,
}

/// Validator info from getValidators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub identity_pubkey: String,
    pub activated_stake: u64,
    pub commission: u8,
    pub active: bool,
}

/// Vote account info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteAccountInfo {
    pub pubkey: String,
    pub validator_pubkey: String,
    pub commission: u8,
    pub active: bool,
}

/// Minimal RPC client for local queries
pub struct RpcClient {
    url: String,
}

impl RpcClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    /// Query current slot
    pub async fn get_slot(&self) -> anyhow::Result<u64> {
        self.call("getSlot", serde_json::Value::Null).await
    }

    /// Query current block height
    pub async fn get_block_height(&self) -> anyhow::Result<u64> {
        self.call("getBlockHeight", serde_json::Value::Null).await
    }

    /// Query transaction count
    pub async fn get_transaction_count(&self) -> anyhow::Result<u64> {
        self.call("getTransactionCount", serde_json::Value::Null).await
    }

    /// Query epoch info
    pub async fn get_epoch_info(&self) -> anyhow::Result<EpochInfo> {
        self.call("getEpochInfo", serde_json::Value::Null).await
    }

    /// Query block production
    pub async fn get_block_production(&self) -> anyhow::Result<BlockProduction> {
        self.call("getBlockProduction", serde_json::Value::Null).await
    }

    /// Query peer count
    pub async fn get_peer_count(&self) -> anyhow::Result<usize> {
        self.call("getPeerCount", serde_json::Value::Null).await
    }

    /// Query connected validators
    pub async fn get_validators(&self) -> anyhow::Result<Vec<ValidatorInfo>> {
        self.call("getValidators", serde_json::Value::Null).await
    }

    /// Query vote accounts
    pub async fn get_vote_accounts(&self) -> anyhow::Result<Vec<VoteAccountInfo>> {
        self.call("getVoteAccounts", serde_json::Value::Null).await
    }

    /// Generic JSON-RPC call
    async fn call<T: for<'de> Deserialize<'de> + Default>(&self, method: &str, params: serde_json::Value) -> anyhow::Result<T> {
        let client = reqwest::Client::new();
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let response = client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("RPC request failed: {}", e))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse RPC response: {}", e))?;

        if let Some(error) = body.get("error") {
            anyhow::bail!("RPC error: {}", error);
        }

        body.get("result")
            .cloned()
            .map(|r| serde_json::from_value(r).unwrap_or_default())
            .ok_or_else(|| anyhow::anyhow!("Missing result in RPC response"))
    }
}

/// Base58 encoding/decoding using the bs58 crate
pub mod base58 {
    /// Encode bytes to base58 string
    pub fn encode(data: &[u8]) -> String {
        bs58::encode(data).into_string()
    }

    /// Decode base58 string to bytes
    pub fn decode(data: &str) -> Vec<u8> {
        bs58::decode(data).into_vec().unwrap_or_default()
    }
}
