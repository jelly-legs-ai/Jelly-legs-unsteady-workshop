//! HTTP RPC Server
//!
//! Provides HTTP endpoints for validator state queries.

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

/// Validator info response (tier, consensus weight)
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

/// Transaction response (stub - MVP has no transactions yet)
#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub signature: String,
    pub slot: u64,
    pub err: Option<String>,
}

/// Start the HTTP RPC server
pub async fn start_rpc_server(addr: &str, state: ValidatorState, block_producer: Arc<BlockProducer>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let port = listener.local_addr()?.port();
    info!("RPC HTTP server listening on http://0.0.0.0:{}/", port);
    info!("Available endpoints:");
    info!("  GET /health               - Health check");
    info!("  GET /v1/slot              - Current slot info");
    info!("  GET /v1/block?slot=N      - Get block by slot");
    info!("  GET /v1/genesis           - Genesis configuration");
    info!("  GET /v1/validators        - Connected validators");
    info!("  GET /v1/epoch             - Epoch information");
    info!("  GET /v1/block_production  - Block production stats");
    info!("  GET /v1/validator/info    - Current validator tier and consensus weight");

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

    // Parse request line: "GET /path HTTP/1.1"
    let request_line = lines.first().unwrap_or(&"");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    // Only support GET
    if method != "GET" {
        send_response(&mut socket, 405, "Method Not Allowed", r#"{"error":"Only GET supported"}"#).await?;
        return Ok(());
    }

    // Route the request
    let response = match path {
        "/" | "/health" => {
            let resp = HealthResponse { status: "ok".to_string() };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        "/v1/slot" => {
            let block_hash = block_producer.current_block_hash().await;
            let resp = SlotResponse {
                slot: state.current_slot(),
                block_hash,
                parent_block_hash: state.get_last_block_hash(),
            };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        path if path.starts_with("/v1/block?slot=") => {
            // Parse ?slot=N
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
        "/v1/genesis" => {
            let resp = GenesisResponse {
                chain_id: state.get_chain_id(),
                genesis_hash: state.get_genesis_hash(),
            };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        "/v1/validators" => {
            let validators = state.get_connected_validators();
            (200, serde_json::to_string(&validators).unwrap_or_default())
        }
        "/v1/epoch" => {
            let info = state.epoch_info();
            (200, serde_json::to_string(&info).unwrap_or_default())
        }
        "/v1/block_production" => {
            let bp = state.block_production();
            (200, serde_json::to_string(&bp).unwrap_or_default())
        }
        "/v1/validator/info" => {
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
        path if path.starts_with("/v1/block?slot=") => {
            // Parse ?slot=N
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
        path if path.starts_with("/v1/block/") => {
            // REST style: /v1/block/<slot>
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
        path if path.starts_with("/v1/getTransaction") => {
            // Stub: MVP has no transactions yet
            let sig = path.split("signature=").nth(1).unwrap_or("").split('&').next().unwrap_or("");
            let resp = TransactionResponse {
                signature: sig.to_string(),
                slot: state.current_slot(),
                err: Some("MVP: transactions not yet implemented".to_string()),
            };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        "/v1/voteAccounts" | "/v1/vote_accounts" | "/v1/getVoteAccounts" => {
            let vote_accounts = state.get_vote_accounts();
            let resp = VoteAccountsResponse {
                vote_accounts,
                total_stake: 0, // MVP: stake not tracked per vote account
            };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
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
