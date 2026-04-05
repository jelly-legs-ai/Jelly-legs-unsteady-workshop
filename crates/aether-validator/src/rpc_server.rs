//! HTTP RPC Server
//!
//! Provides HTTP endpoints for validator state queries and transaction submission.

use crate::block_producer::BlockProducer;
use crate::state::ValidatorState;
use serde::Serialize;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

/// Slot info response
#[derive(Debug, Serialize)]
pub struct SlotResponse {
    pub slot: u64,
    pub block_hash: String,
    pub parent_block_hash: String,
}

/// Block response
#[derive(Debug, Serialize)]
pub struct BlockResponse {
    pub slot: u64,
    pub timestamp: u64,
    pub block_hash: String,
    pub previous_block_hash: String,
    pub poh_seed: String,
    pub transaction_count: usize,
}

/// Genesis response
#[derive(Debug, Serialize)]
pub struct GenesisResponse {
    pub chain_id: String,
    pub genesis_hash: String,
}

/// Health response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
}

/// Validator info response
#[derive(Debug, Serialize)]
pub struct ValidatorInfoResponse {
    pub tier: String,
    pub consensus_weight: f64,
    pub can_produce_blocks: bool,
    pub can_vote: bool,
}

/// Vote accounts response
#[derive(Debug, Serialize)]
pub struct VoteAccountsResponse {
    pub vote_accounts: Vec<crate::rpc_client::VoteAccountInfo>,
    pub total_stake: u64,
}

/// Transaction response
#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub signature: String,
    pub slot: u64,
    pub err: Option<String>,
}

/// Start the HTTP RPC server
pub async fn start_rpc_server(
    addr: &str,
    state: ValidatorState,
    block_producer: Arc<BlockProducer>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let port = listener.local_addr()?.port();
    info!("RPC HTTP server listening on http://0.0.0.0:{}/", port);
    info!("Available endpoints:");
    info!("  GET  /health               - Health check");
    info!("  GET  /v1/slot              - Current slot info");
    info!("  GET  /v1/block?slot=N      - Get block by slot");
    info!("  GET  /v1/genesis           - Genesis configuration");
    info!("  GET  /v1/validators        - Connected validators");
    info!("  GET  /v1/epoch             - Epoch information");
    info!("  GET  /v1/block_production  - Block production stats");
    info!("  GET  /v1/validator/info    - Current validator tier and consensus weight");
    info!("  POST /v1/tx                - Submit a transaction");
    info!("  GET  /v1/tx/<signature>    - Get transaction status");
    info!("  GET  /v1/account/<address> - Get account info");
    info!("  GET  /v1/total_supply      - Get total token supply");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                let state = state.clone();
                let bp = block_producer.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_http_request(socket, state, bp).await {
                        warn!("HTTP request error from {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                warn!("Failed to accept RPC connection: {}", e);
            }
        }
    }
}

/// Handle an HTTP request
async fn handle_http_request(
    mut socket: TcpStream,
    state: ValidatorState,
    block_producer: Arc<BlockProducer>,
) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 8192];
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let request_str = String::from_utf8_lossy(&buf[..n]);
    let lines: Vec<&str> = request_str.lines().collect();
    
    if lines.is_empty() {
        return Ok(());
    }

    let request_line = lines.first().unwrap_or(&"");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    // Route the request
    let response: (u16, String) = match (method, path) {
        // Health
        ("GET", "/" | "/health") => {
            let resp = HealthResponse { status: "ok".to_string() };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Slot
        ("GET", "/v1/slot") => {
            let current_slot = state.current_slot();
            // current_block_hash() returns the hash of the most recently produced block,
            // which corresponds to slot (current_slot - 1) since increment_slot()
            // is called after produce_block() increments the counter.
            let block_hash = block_producer.current_block_hash().await;
            // The genesis block (slot 0) is its own parent. For all other slots,
            // look up the actual previous_block_hash from the block at (current_slot - 1),
            // because the block at `current_slot` hasn't been produced yet.
            let parent_block_hash = if current_slot == 0 {
                block_hash.clone()
            } else if let Some(block) = block_producer.get_block(current_slot - 1).await {
                block.previous_block_hash
            } else {
                block_hash.clone()
            };
            let resp = SlotResponse {
                slot: current_slot,
                block_hash,
                parent_block_hash,
            };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Genesis
        ("GET", "/v1/genesis") => {
            let resp = GenesisResponse {
                chain_id: state.get_chain_id(),
                genesis_hash: state.get_genesis_hash(),
            };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Validators
        ("GET", "/v1/validators") => {
            let validators = state.get_connected_validators();
            (200, serde_json::to_string(&validators).unwrap_or_default())
        }
        // Epoch
        ("GET", "/v1/epoch") => {
            let info = state.epoch_info();
            (200, serde_json::to_string(&info).unwrap_or_default())
        }
        // Block production
        ("GET", "/v1/block_production") => {
            let bp = state.block_production();
            (200, serde_json::to_string(&bp).unwrap_or_default())
        }
        // Validator info
        ("GET", "/v1/validator/info") => {
            let tier = state.tier();
            let tier_str = match tier {
                crate::genesis::ValidatorTier::Full => "full",
                crate::genesis::ValidatorTier::Lite => "lite",
                crate::genesis::ValidatorTier::Observer => "observer",
            };
            let resp = ValidatorInfoResponse {
                tier: tier_str.to_string(),
                consensus_weight: state.consensus_weight(),
                can_produce_blocks: state.can_produce_blocks(),
                can_vote: state.can_vote(),
            };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Block by slot query param
        ("GET", path) if path.starts_with("/v1/block?slot=") => {
            if let Ok(slot) = path.split("slot=").nth(1)
                .and_then(|s| s.split('&').next())
                .unwrap_or("")
                .parse::<u64>()
            {
                if let Some(block) = block_producer.get_block(slot).await {
                    let resp = BlockResponse {
                        slot: block.slot,
                        timestamp: block.timestamp,
                        block_hash: block.block_hash,
                        previous_block_hash: block.previous_block_hash,
                        poh_seed: block.poh_seed,
                        transaction_count: block.transactions.len(),
                    };
                    (200, serde_json::to_string(&resp).unwrap_or_default())
                } else {
                    (404, r#"{"error":"Block not found"}"#.to_string())
                }
            } else {
                (400, r#"{"error":"Invalid slot parameter"}"#.to_string())
            }
        }
        // Block by slot path param
        ("GET", path) if path.starts_with("/v1/block/") => {
            let slot_str = path.strip_prefix("/v1/block/").unwrap_or("").split('?').next().unwrap_or("");
            if let Ok(slot) = slot_str.parse::<u64>() {
                if let Some(block) = block_producer.get_block(slot).await {
                    let resp = BlockResponse {
                        slot: block.slot,
                        timestamp: block.timestamp,
                        block_hash: block.block_hash,
                        previous_block_hash: block.previous_block_hash,
                        poh_seed: block.poh_seed,
                        transaction_count: block.transactions.len(),
                    };
                    (200, serde_json::to_string(&resp).unwrap_or_default())
                } else {
                    (404, r#"{"error":"Block not found"}"#.to_string())
                }
            } else {
                (400, r#"{"error":"Invalid slot number"}"#.to_string())
            }
        }
        // Transaction status stub
        ("GET", path) if path.starts_with("/v1/getTransaction") || path.starts_with("/v1/tx/") => {
            let sig = path.split("signature=").nth(1)
                .or_else(|| path.strip_prefix("/v1/tx/"))
                .unwrap_or("")
                .split('?')
                .next()
                .unwrap_or("");
            if let Some(receipt) = block_producer.get_receipt(sig).await {
                let resp = serde_json::json!({
                    "signature": sig,
                    "slot": receipt.slot,
                    "block_hash": receipt.block_hash,
                    "success": receipt.result.success,
                    "error": receipt.result.error,
                    "timestamp": receipt.timestamp
                });
                (200, serde_json::to_string(&resp).unwrap_or_default())
            } else {
                (404, r#"{"error":"Transaction not found"}"#.to_string())
            }
        }
        // Vote accounts
        ("GET", "/v1/voteAccounts" | "/v1/vote_accounts" | "/v1/getVoteAccounts") => {
            let vote_accounts = state.get_vote_accounts();
            let resp = VoteAccountsResponse {
                vote_accounts,
                total_stake: 0,
            };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Total supply
        ("GET", "/v1/total_supply") => {
            let supply = block_producer.total_supply().await;
            let resp = serde_json::json!({"total_supply": supply, "unit": "lamports"});
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Account info
        ("GET", path) if path.starts_with("/v1/account/") => {
            let addr_str = path.strip_prefix("/v1/account/").unwrap_or("").split('?').next().unwrap_or("");
            let addr_bytes = bs58::decode(addr_str).into_vec().unwrap_or_default();
            let mut addr = [0u8; 32];
            addr.copy_from_slice(&addr_bytes[..32.min(addr_bytes.len())]);
            
            if let Some(account) = block_producer.get_account(&addr).await {
                let resp = serde_json::json!({
                    "address": addr_str,
                    "lamports": account.lamports,
                    "owner": bs58::encode(account.owner).into_string(),
                    "data_size": account.data.len(),
                    "rent_epoch": account.rent_epoch
                });
                (200, serde_json::to_string(&resp).unwrap_or_default())
            } else {
                let resp = serde_json::json!({
                    "address": addr_str,
                    "lamports": 0,
                    "owner": "11111111111111111111111111111111",
                    "data_size": 0,
                    "rent_epoch": 0
                });
                (200, serde_json::to_string(&resp).unwrap_or_default())
            }
        }
        // Submit transaction
        ("POST", "/v1/tx" | "/v1/submit") => {
            let body_start = request_str.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
            let body = &request_str[body_start..];
            match serde_json::from_str::<serde_json::Value>(body) {
                Ok(json) => {
                    let tx_type = json.get("tx_type").and_then(|v| v.as_str()).unwrap_or("transfer");
                    let signer = json.get("signer").and_then(|v| v.as_str()).unwrap_or("");
                    let payload = json.get("payload");
                    let sig_str = json.get("signature").and_then(|v| v.as_str()).unwrap_or("");
                    let fee = json.get("fee").and_then(|v| v.as_u64()).unwrap_or(0);
                    
                    let sig_bytes = bs58::decode(sig_str).into_vec().unwrap_or_default();
                    let mut signature = [0u8; 64];
                    signature.copy_from_slice(&sig_bytes[..64.min(sig_bytes.len())]);
                    
                    let signer_bytes = bs58::decode(signer).into_vec().unwrap_or_default();
                    let mut signer_arr = [0u8; 32];
                    signer_arr.copy_from_slice(&signer_bytes[..32.min(signer_bytes.len())]);
                    
                    let tx = aether_core::AetherTransaction {
                        signature,
                        signer: signer_arr,
                        tx_type: match tx_type {
                            "transfer" => aether_core::TransactionType::Transfer,
                            "stake" => aether_core::TransactionType::Stake,
                            "unstake" => aether_core::TransactionType::Unstake,
                            "claim_rewards" => aether_core::TransactionType::ClaimRewards,
                            "create_nft" => aether_core::TransactionType::CreateNFT,
                            "mint_nft" => aether_core::TransactionType::MintNFT,
                            "transfer_nft" => aether_core::TransactionType::TransferNFT,
                            "update_metadata" => aether_core::TransactionType::UpdateMetadata,
                            _ => aether_core::TransactionType::Transfer,
                        },
                        payload: aether_core::TransactionPayload::Transfer {
                            recipient: payload.and_then(|p| p.get("recipient")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            amount: payload.and_then(|p| p.get("amount")).and_then(|v| v.as_u64()).unwrap_or(0),
                            nonce: payload.and_then(|p| p.get("nonce")).and_then(|v| v.as_u64()).unwrap_or(0),
                        },
                        fee,
                        slot: 0,
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    };
                    
                    match block_producer.submit_transaction(tx).await {
                        Ok(sig) => {
                            let resp = serde_json::json!({"signature": sig, "slot": state.current_slot()});
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (500, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Not found
        _ => {
            (404, r#"{"error":"Endpoint not found"}"#.to_string())
        }
    };

    send_response(&mut socket, response.0, "OK", &response.1).await?;
    Ok(())
}

/// Send an HTTP response
async fn send_response(
    socket: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    body: &str,
) -> anyhow::Result<()> {
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
        Content-Type: application/json\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        Access-Control-Allow-Origin: *\r\n\
        \r\n\
        {}",
        status_code,
        status_text,
        body.len(),
        body
    );

    socket.write_all(response.as_bytes()).await?;
    socket.flush().await?;
    Ok(())
}
