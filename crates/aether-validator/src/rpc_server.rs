//! HTTP RPC Server
//!
//! Provides HTTP endpoints for validator state queries and transaction submission.
//! Includes AI Priority Lane endpoints for fee economics and lane configuration.

use crate::block_producer::BlockProducer;
use crate::state::ValidatorState;
use aether_ai_priority::fee_distribution::FeeDistributionConfig;
use bs58;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Rate limiter: tracks requests per IP within a time window
struct RateLimiter {
    /// Maps IP -> (request_count, window_start)
    clients: HashMap<String, (u32, Instant)>,
    /// Maximum requests per IP per window
    max_requests: u32,
    /// Time window duration
    window: Duration,
}

impl RateLimiter {
    fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            clients: HashMap::new(),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Check if a request from this IP is allowed. Returns false if rate-limited.
    fn check_and_record(&mut self, ip: &str) -> bool {
        let now = Instant::now();
        let entry = self.clients.entry(ip.to_string()).or_insert((0, now));

        // Reset window if expired
        if now.duration_since(entry.1) > self.window {
            *entry = (1, now);
            return true;
        }

        // Check limit
        if entry.0 >= self.max_requests {
            return false;
        }

        entry.0 += 1;
        true
    }

    /// Clean up expired entries to prevent memory growth
    fn cleanup(&mut self) {
        let now = Instant::now();
        self.clients.retain(|_, (count, start)| {
            now.duration_since(*start) <= self.window && *count > 0
        });
    }
}

/// Sanitize a URL path to prevent path traversal attacks
fn sanitize_path(path: &str) -> Option<&str> {
    // Reject paths with null bytes, path traversal, or suspicious sequences
    if path.contains('\0') || path.contains("..") || path.contains("//") {
        return None;
    }
    // Only allow printable ASCII characters
    if !path.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
        return None;
    }
    Some(path)
}

/// Validate and decode a bs58 address. Returns None if invalid or too long.
#[allow(dead_code)]
fn decode_address(addr: &str) -> Option<[u8; 32]> {
    // Limit address length to prevent DoS
    if addr.len() > 64 {
        return None;
    }
    let decoded = bs58::decode(addr).into_vec().ok()?;
    if decoded.len() < 32 {
        let mut arr = [0u8; 32];
        arr[..decoded.len()].copy_from_slice(&decoded);
        Some(arr)
    } else {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&decoded[..32]);
        Some(arr)
    }
}

/// Validate that a string field is within length limits and contains safe characters
#[allow(dead_code)]
fn validate_string_field(s: &str, max_len: usize, field_name: &str) -> Result<(), String> {
    if s.len() > max_len {
        Err(format!("{} exceeds maximum length of {} characters", field_name, max_len))
    } else if s.contains('\0') {
        Err(format!("{} contains null bytes", field_name))
    } else {
        Ok(())
    }
}

/// Maximum length for JSON body fields
#[allow(dead_code)]
const MAX_TITLE_LEN: usize = 256;
#[allow(dead_code)]
const MAX_DESCRIPTION_LEN: usize = 4096;
#[allow(dead_code)]
const MAX_PURPOSE_LEN: usize = 512;

/// Slot info response
#[derive(Debug, Serialize)]
pub struct SlotResponse {
    pub slot: u64,
    pub block_hash: String,
    pub parent_block_hash: String,
    /// Is validator healthy and synchronized
    pub healthy: bool,
    /// Error message if unhealthy (null if healthy)
    pub error: Option<String>,
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

/// Stake request
#[derive(Debug, serde::Deserialize)]
pub struct StakeRequest {
    pub owner: String,
    pub amount: u64,
    #[serde(default)]
    pub delegate_to: Option<String>,
}

/// Unstake request
#[derive(Debug, serde::Deserialize)]
pub struct UnstakeRequest {
    pub owner: String,
    pub stake_id: usize,
}

/// Claim request
#[derive(Debug, serde::Deserialize)]
pub struct ClaimRequest {
    pub owner: String,
    pub stake_id: usize,
}

/// Start the HTTP RPC server
pub async fn start_rpc_server(
    addr: &str,
    state: ValidatorState,
    block_producer: Arc<BlockProducer>,
    shutdown: crate::shutdown::ShutdownSignal,
) -> anyhow::Result<()> {
    // Rate limiter: 100 requests per 10 seconds per IP
    let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(100, 10)));
    // Cleanup interval for expired rate limit entries
    let cleanup_counter = Arc::new(std::sync::atomic::AtomicU64::new(0));

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
    info!("  === AI Priority Lane Endpoints ===");
    info!("  GET  /v1/ai_priority/lanes       - Lane config + current fee economics");
    info!("  GET  /v1/ai_priority/economics    - Fee economics summary");
    info!("  GET  /v1/ai_priority/epoch        - Current epoch fee stats");
    info!("  GET  /v1/ai_priority/epoch/<N>    - Fee stats for epoch N");
    info!("  GET  /v1/ai_priority/treasury    - Treasury state + history");
    info!("  GET  /v1/ai_priority/rewards/<addr> - Validator fee rewards");
    info!("  POST /v1/ai_priority/submit       - Submit transaction with lane field");
    info!("  === Staking Endpoints ===");
    info!("  GET  /v1/staking/pool                    - Staking pool info");
    info!("  GET  /v1/staking/positions/<address>    - Staking positions for address");
    info!("  GET  /v1/staking/summary/<address>      - Staking summary for address");
    info!("  GET  /v1/staking/validator/<address>    - Validator staking info");
    info!("  POST /v1/staking/stake                  - Create a new stake");
    info!("  POST /v1/staking/unstake                - Initiate unstake");
    info!("  POST /v1/staking/withdraw               - Complete withdrawal");
    info!("  POST /v1/staking/claim                  - Claim staking rewards");
    info!("  POST /v1/staking/delegate               - Delegate stake to validator");
    info!("  === Governance Endpoints ===");
    info!("  GET  /v1/governance/config              - Governance configuration");
    info!("  GET  /v1/governance/stats               - Governance statistics");
    info!("  GET  /v1/governance/council             - Security council members");
    info!("  GET  /v1/governance/proposals            - Active proposals");
    info!("  GET  /v1/governance/proposals/active     - Active proposals (alias)");
    info!("  GET  /v1/governance/proposals/status/:s  - Proposals by status");
    info!("  GET  /v1/governance/proposal/:id         - Get proposal details");
    info!("  POST /v1/governance/proposal             - Create a new proposal");
    info!("  POST /v1/governance/vote                 - Cast a vote on a proposal");
    info!("  POST /v1/governance/execute              - Execute a passed proposal");
    info!("  POST /v1/governance/veto                 - Veto a proposal (council)");
    info!("  POST /v1/governance/cancel               - Cancel a proposal (proposer)");
    info!("  === Treasury Endpoints ===");
    info!("  GET  /v1/treasury/summary               - Treasury balances & stats");
    info!("  GET  /v1/treasury/budget                 - Budget allocations");
    info!("  GET  /v1/treasury/withdrawals/pending     - Pending withdrawals");
    info!("  GET  /v1/treasury/withdrawal/:id         - Get withdrawal details");
    info!("  POST /v1/treasury/withdraw               - Create withdrawal request");
    info!("  POST /v1/treasury/approve                - Approve withdrawal (signer)");
    info!("  POST /v1/treasury/execute                 - Execute withdrawal after timelock");
    info!("  POST /v1/treasury/add_signer             - Add treasury signer");

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((socket, addr)) => {
                        let state = state.clone();
                        let bp = block_producer.clone();
                        let rl = rate_limiter.clone();
                        let cc = cleanup_counter.clone();
                        tokio::spawn(async move {
                    // Rate limit check
                    {
                        let mut limiter = rl.lock().await;
                        let ip = addr.ip().to_string();
                        if !limiter.check_and_record(&ip) {
                            warn!("Rate limit exceeded for {}", ip);
                            let response = "HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":\"Rate limit exceeded\"}";
                            let _ = socket.writable().await;
                            // Best-effort write; if it fails the client will see a disconnect
                            use tokio::io::AsyncWriteExt;
                            let mut stream = socket;
                            let _ = stream.write_all(response.as_bytes()).await;
                            return;
                        }
                        // Periodic cleanup every 1000 requests
                        if cc.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 1000 == 0 {
                            limiter.cleanup();
                        }
                    }
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
            _ = shutdown.recv() => {
                info!("RPC server shutting down gracefully");
                break;
            }
        }
    }

    info!("RPC server stopped");
    Ok(())
}

/// Maximum request body size (1 MB)
const MAX_REQUEST_BODY_SIZE: usize = 1024 * 1024;

/// Handle an HTTP request
async fn handle_http_request(
    mut socket: TcpStream,
    state: ValidatorState,
    block_producer: Arc<BlockProducer>,
) -> anyhow::Result<()> {
    // Read headers first (up to 16KB for headers)
    let mut headers_buf = vec![0u8; 16384];
    let mut total_read = 0usize;
    
    // Read until we find \r\n\r\n (end of headers)
    let header_end = loop {
        if total_read >= headers_buf.len() {
            // Headers too large
            send_response(&mut socket, 431, "Request Header Fields Too Large", r#"{"error":"Headers too large"}"#).await?;
            return Ok(());
        }
        
        let n = socket.read(&mut headers_buf[total_read..]).await?;
        if n == 0 {
            return Ok(()); // Client disconnected
        }
        total_read += n;
        
        let header_str = String::from_utf8_lossy(&headers_buf[..total_read]);
        if let Some(pos) = header_str.find("\r\n\r\n") {
            break pos;
        }
        
        // Safety: don't read more than reasonable
        if total_read > 16384 {
            send_response(&mut socket, 431, "Request Header Fields Too Large", r#"{"error":"Headers too large"}"#).await?;
            return Ok(());
        }
    };

    let request_str = String::from_utf8_lossy(&headers_buf[..total_read]);
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

    // Sanitize path to prevent traversal attacks
    let path = match sanitize_path(path) {
        Some(p) => p,
        None => {
            send_response(&mut socket, 400, "Bad Request", r#"{"error":"Invalid path"}"#).await?;
            return Ok(());
        }
    };

    // Handle CORS preflight (OPTIONS) requests
    if method == "OPTIONS" {
        send_cors_preflight(&mut socket).await?;
        return Ok(());
    }

    // Extract Content-Length and read body if present
    let body_start = header_end + 4; // Skip \r\n\r\n
    let content_length: usize = extract_header(&lines, "content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    
    // Security: reject oversized bodies
    if content_length > MAX_REQUEST_BODY_SIZE {
        send_response(&mut socket, 413, "Payload Too Large", r#"{"error":"Request body too large"}"#).await?;
        return Ok(());
    }

    // Collect the body (may already be in headers_buf or need more reading)
    let body = if content_length > 0 {
        let already_read = total_read.saturating_sub(body_start);
        let remaining = content_length.saturating_sub(already_read);
        
        let mut body_data = headers_buf[body_start..total_read].to_vec();
        if remaining > 0 {
            let mut extra = vec![0u8; remaining];
            let mut extra_read = 0usize;
            while extra_read < remaining {
                let n = socket.read(&mut extra[extra_read..]).await?;
                if n == 0 { break; }
                extra_read += n;
            }
            body_data.extend_from_slice(&extra[..extra_read]);
        }
        String::from_utf8_lossy(&body_data).to_string()
    } else {
        // For POST without Content-Length or GET, try to use whatever's after headers
        if body_start < total_read {
            String::from_utf8_lossy(&headers_buf[body_start..total_read]).to_string()
        } else {
            String::new()
        }
    };

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
            // Get the block hash for the current slot (most recently produced block)
            let block_hash = block_producer.current_block_hash().await;
            
            // Get parent block hash correctly using checked arithmetic to avoid underflow:
            // - For slot 0: use genesis hash from state
            // - For slot > 0: use the block hash from the previous slot
            let parent_block_hash = if let Some(prev_slot) = current_slot.checked_sub(1) {
                match block_producer.get_block(prev_slot).await {
                    Some(block) => block.block_hash,
                    None => state.get_genesis_hash(),
                }
            } else {
                // current_slot is 0, use genesis hash
                state.get_genesis_hash()
            };
            
            // Check validator health:
            // - Healthy if no blocks produced yet (fresh start)
            // - Healthy if blocks produced and block_hash has advanced from genesis
            // - Unhealthy if blocks produced but block_hash is still genesis (sync issue)
            let blocks_produced = state.blocks_produced();
            let genesis_hash = state.get_genesis_hash();
            let (healthy, error) = if blocks_produced == 0 {
                // No blocks produced yet - normal for fresh start, even if hash is empty/genesis
                (true, None)
            } else if block_hash.is_empty() {
                // Blocks produced but hash is empty = definite sync issue
                (false, Some("Validator not producing blocks - blocks_produced > 0 but block hash is empty".to_string()))
            } else if genesis_hash.is_empty() {
                // Genesis hash is empty/zero-initialized, can't compare - assume healthy if we have a hash
                (true, None)
            } else if block_hash == genesis_hash {
                // Blocks produced but hash hasn't advanced from non-empty genesis = sync issue
                (false, Some("Validator not producing blocks - blocks_produced > 0 but block hash unchanged from genesis".to_string()))
            } else {
                // Normal operation - blocks produced and hash has advanced
                (true, None)
            };
            
            let resp = SlotResponse {
                slot: current_slot,
                block_hash,
                parent_block_hash,
                healthy,
                error,
            };
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Block height (alias for slot, returns current slot as block height)
        ("GET", "/v1/blockheight" | "/v1/block_height" | "/v1/height") => {
            let current_slot = state.current_slot();
            let resp = serde_json::json!({
                "blockHeight": current_slot,
                "slot": current_slot
            });
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
        // ========== AI Priority Lane Endpoints ==========
        // Get lane configuration and current fee economics
        ("GET", "/v1/ai_priority/lanes" | "/v1/ai_priority/config") => {
            let fd = block_producer.fee_distributor();
            let stats = fd.current_epoch_stats();
            let treasury = fd.treasury_state();
            let cfg = FeeDistributionConfig::default();
            let resp = serde_json::json!({
                "lanes": {
                    "critical": {
                        "multiplier": 10,
                        "min_fee": cfg.min_critical_fee,
                        "treasury_share_bps": 10000,
                        "validator_share_bps": 0,
                        "description": "AI governance, emergency operations (10x base fee, 100% to treasury)"
                    },
                    "high": {
                        "multiplier": 5,
                        "min_fee": cfg.min_high_fee,
                        "treasury_share_bps": 5000,
                        "validator_share_bps": 5000,
                        "description": "AI agent transactions, MEV protection (5x base fee, 50/50 split)"
                    },
                    "standard": {
                        "multiplier": 1,
                        "min_fee": cfg.base_fee_per_cu,
                        "treasury_share_bps": 0,
                        "validator_share_bps": 10000,
                        "description": "Regular user transactions (base fee, 100% to validators)"
                    }
                },
                "current_epoch": fd.current_epoch(),
                "epoch_stats": {
                    "critical_fees": stats.critical_fees,
                    "critical_tx_count": stats.critical_tx_count,
                    "high_fees": stats.high_fees,
                    "high_tx_count": stats.high_tx_count,
                    "standard_fees": stats.standard_fees,
                    "standard_tx_count": stats.standard_tx_count,
                    "total_treasury_fees": stats.treasury_fees,
                    "total_validator_fees": stats.validator_fees,
                    "total_burned": stats.burned_fees
                },
                "treasury": {
                    "address": bs58::encode(treasury.address).into_string(),
                    "lifetime_fees": treasury.lifetime_fees,
                    "epoch_fees": treasury.epoch_fees,
                    "lifetime_burned": treasury.lifetime_burned
                }
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get fee economics summary
        ("GET", "/v1/ai_priority/economics") => {
            let fd = block_producer.fee_distributor();
            let summary = fd.fee_economics_summary();
            (200, serde_json::to_string(&summary).unwrap_or_default())
        }
        // Get current epoch fee statistics
        ("GET", "/v1/ai_priority/epoch_stats" | "/v1/ai_priority/epoch") => {
            let fd = block_producer.fee_distributor();
            let stats = fd.current_epoch_stats();
            (200, serde_json::to_string(&stats).unwrap_or_default())
        }
        // Get fee stats for a specific epoch
        ("GET", path) if path.starts_with("/v1/ai_priority/epoch_stats/") || path.starts_with("/v1/ai_priority/epoch/") => {
            let epoch_str = path.split('/').last().unwrap_or("");
            if let Ok(epoch) = epoch_str.parse::<u64>() {
                let fd = block_producer.fee_distributor();
                if let Some(stats) = fd.get_epoch_stats(epoch) {
                    (200, serde_json::to_string(&stats).unwrap_or_default())
                } else {
                    (404, serde_json::json!({"error": "Epoch not found", "epoch": epoch}).to_string())
                }
            } else {
                (400, r#"{"error":"Invalid epoch number"}"#.to_string())
            }
        }
        // Get treasury state
        ("GET", "/v1/ai_priority/treasury") => {
            let fd = block_producer.fee_distributor();
            let treasury = fd.treasury_state();
            let resp = serde_json::json!({
                "address": bs58::encode(treasury.address).into_string(),
                "lifetime_fees": treasury.lifetime_fees,
                "epoch_fees": treasury.epoch_fees,
                "lifetime_burned": treasury.lifetime_burned,
                "recent_epochs": treasury.epoch_history.iter().map(|e| {
                    serde_json::json!({
                        "epoch": e.epoch,
                        "treasury_fees": e.treasury_fees,
                        "validator_fees": e.validator_fees,
                        "burned": e.burned_fees,
                        "critical_tx": e.critical_tx_count,
                        "high_tx": e.high_tx_count,
                        "standard_tx": e.standard_tx_count
                    })
                }).collect::<Vec<_>>()
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get validator fee rewards by pubkey
        ("GET", path) if path.starts_with("/v1/ai_priority/validator_rewards/") 
                      || path.starts_with("/v1/ai_priority/rewards/") => {
            let addr_str = path.split('/').last().unwrap_or("");
            let addr_bytes = bs58::decode(addr_str).into_vec().unwrap_or_default();
            let mut addr = [0u8; 32];
            addr.copy_from_slice(&addr_bytes[..32.min(addr_bytes.len())]);
            
            let fd = block_producer.fee_distributor();
            if let Some(rewards) = fd.get_validator_rewards(&addr) {
                (200, serde_json::to_string(&rewards).unwrap_or_default())
            } else {
                (404, serde_json::json!({
                    "error": "Validator not found or has no fee rewards",
                    "validator": addr_str
                }).to_string())
            }
        }
        // Submit transaction with AI priority lane (accepts lane in body)
        ("POST", "/v1/ai_priority/submit") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let tx_type = json.get("tx_type").and_then(|v| v.as_str()).unwrap_or("transfer");
                    let signer = json.get("signer").and_then(|v| v.as_str()).unwrap_or("");
                    let payload = json.get("payload");
                    let sig_str = json.get("signature").and_then(|v| v.as_str()).unwrap_or("");
                    let fee = json.get("fee").and_then(|v| v.as_u64()).unwrap_or(0);
                    // AI Priority Lane: critical, high, or standard (defaults to standard)
                    let lane_str = json.get("lane").and_then(|v| v.as_str()).unwrap_or("standard");
                    let lane = match lane_str {
                        "critical" => "critical",
                        "high" => "high",
                        _ => "standard",
                    };
                    
                    // Minimum fee enforcement based on lane
                    let min_fee = match lane {
                        "critical" => 1_000_000u64,
                        "high" => 500_000u64,
                        _ => 5_000u64,
                    };
                    let final_fee = fee.max(min_fee);
                    
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
                        fee: final_fee,
                        slot: 0,
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
                    };
                    
                    match block_producer.submit_transaction(tx).await {
                        Ok(sig) => {
                            let resp = serde_json::json!({
                                "signature": sig,
                                "slot": state.current_slot(),
                                "lane": lane,
                                "fee": final_fee,
                                "min_fee_for_lane": min_fee,
                                "message": format!("Transaction submitted via {} lane", lane)
                            });
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
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
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
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
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
        // =================================================================
        // STAKING ENDPOINTS
        // =================================================================
        // Get staking positions for an address
        ("GET", path) if path.starts_with("/v1/staking/positions/") => {
            let addr_str = path.strip_prefix("/v1/staking/positions/").unwrap_or("").split('?').next().unwrap_or("");
            let positions = state.get_staking_positions(addr_str);
            let resp = serde_json::json!({
                "address": addr_str,
                "positions": positions,
                "count": positions.len()
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get staking summary for an address
        ("GET", path) if path.starts_with("/v1/staking/summary/") => {
            let addr_str = path.strip_prefix("/v1/staking/summary/").unwrap_or("").split('?').next().unwrap_or("");
            let summary = state.get_staking_summary(addr_str);
            (200, serde_json::to_string(&summary).unwrap_or_default())
        }
        // Create a new stake
        ("POST", "/v1/staking/stake") => {
            // body already parsed above
            match serde_json::from_str::<StakeRequest>(&body) {
                Ok(req) => {
                    // Decode owner address
                    let owner_bytes = bs58::decode(&req.owner).into_vec().unwrap_or_default();
                    let mut owner = [0u8; 32];
                    owner.copy_from_slice(&owner_bytes[..32.min(owner_bytes.len())]);
                    
                    // Get optional delegate_to
                    let delegate_to = req.delegate_to.as_ref().and_then(|v| {
                        let bytes = bs58::decode(v).into_vec().ok()?;
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&bytes[..32.min(bytes.len())]);
                        Some(arr)
                    });
                    
                    match state.create_stake(owner, req.amount, delegate_to) {
                        Ok(stake_id) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "stake_id": stake_id,
                                "owner": req.owner,
                                "amount": req.amount,
                                "message": "Stake created successfully"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Initiate unstake
        ("POST", "/v1/staking/unstake") => {
            // body already parsed above
            match serde_json::from_str::<UnstakeRequest>(&body) {
                Ok(req) => {
                    let owner_bytes = bs58::decode(&req.owner).into_vec().unwrap_or_default();
                    let mut owner = [0u8; 32];
                    owner.copy_from_slice(&owner_bytes[..32.min(owner_bytes.len())]);
                    
                    match state.initiate_unstake(owner, req.stake_id) {
                        Ok(unlock_epoch) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "stake_id": req.stake_id,
                                "unlock_epoch": unlock_epoch,
                                "message": "Unstake initiated. Funds will be available after unlock period."
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Complete withdrawal after lock period
        ("POST", "/v1/staking/withdraw") => {
            // body already parsed above
            match serde_json::from_str::<UnstakeRequest>(&body) {
                Ok(req) => {
                    let owner_bytes = bs58::decode(&req.owner).into_vec().unwrap_or_default();
                    let mut owner = [0u8; 32];
                    owner.copy_from_slice(&owner_bytes[..32.min(owner_bytes.len())]);
                    
                    match state.complete_withdrawal(owner, req.stake_id) {
                        Ok(amount) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "stake_id": req.stake_id,
                                "amount_withdrawn": amount,
                                "message": "Withdrawal completed successfully"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Claim staking rewards
        ("POST", "/v1/staking/claim") => {
            // body already parsed above
            match serde_json::from_str::<ClaimRequest>(&body) {
                Ok(req) => {
                    let owner_bytes = bs58::decode(&req.owner).into_vec().unwrap_or_default();
                    let mut owner = [0u8; 32];
                    owner.copy_from_slice(&owner_bytes[..32.min(owner_bytes.len())]);
                    
                    match state.claim_rewards(owner, req.stake_id) {
                        Ok(rewards) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "stake_id": req.stake_id,
                                "rewards_claimed": rewards,
                                "message": "Rewards claimed successfully"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Get staking pool info
        ("GET", "/v1/staking/pool") => {
            let pool_info = state.get_staking_pool_info();
            (200, serde_json::to_string(&pool_info).unwrap_or_default())
        }
        // Get validator staking info
        ("GET", path) if path.starts_with("/v1/staking/validator/") => {
            let addr_str = path.strip_prefix("/v1/staking/validator/").unwrap_or("").split('?').next().unwrap_or("");
            let info = state.get_validator_staking_info(addr_str);
            (200, serde_json::to_string(&info).unwrap_or_default())
        }
        // Delegate stake to validator
        ("POST", "/v1/staking/delegate") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let owner_str = json.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                    let stake_id = json.get("stake_id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let validator_str = json.get("validator").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let owner_bytes = bs58::decode(owner_str).into_vec().unwrap_or_default();
                    let mut owner = [0u8; 32];
                    owner.copy_from_slice(&owner_bytes[..32.min(owner_bytes.len())]);
                    
                    let validator_bytes = bs58::decode(validator_str).into_vec().unwrap_or_default();
                    let mut validator = [0u8; 32];
                    validator.copy_from_slice(&validator_bytes[..32.min(validator_bytes.len())]);
                    
                    match state.delegate_stake(owner, stake_id, validator) {
                        Ok(()) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "stake_id": stake_id,
                                "validator": validator_str,
                                "message": "Stake delegated successfully"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // =================================================================
        // GOVERNANCE ENDPOINTS
        // =================================================================
        // Get governance config
        ("GET", "/v1/governance/config") => {
            let config = state.governance_config();
            let resp = serde_json::json!({
                "voting_delay": config.voting_delay,
                "voting_duration": config.voting_duration,
                "execution_delay": config.execution_delay,
                "quorum_threshold": config.quorum_threshold,
                "supermajority_bps": config.supermajority_bps,
                "min_proposal_deposit": config.min_proposal_deposit,
                "max_active_proposals": config.max_active_proposals,
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get governance stats
        ("GET", "/v1/governance/stats") => {
            let stats = state.governance_stats();
            let resp = serde_json::json!({
                "total": stats.total,
                "draft": stats.draft,
                "pending": stats.pending,
                "active": stats.active,
                "passed": stats.passed,
                "failed": stats.failed,
                "executed": stats.executed,
                "cancelled": stats.cancelled,
                "expired": stats.expired,
                "vetoed": stats.vetoed,
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get security council members
        ("GET", "/v1/governance/council") => {
            let council = state.governance_council();
            let resp = serde_json::json!({
                "members": council,
                "count": council.len(),
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get all active proposals
        ("GET", "/v1/governance/proposals" | "/v1/governance/proposals/active") => {
            let proposals = state.get_active_governance_proposals();
            let proposal_list: Vec<serde_json::Value> = proposals.iter().map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "title": p.title,
                    "status": match p.status {
                        aether_governance::ProposalStatus::Draft => "draft",
                        aether_governance::ProposalStatus::Pending => "pending",
                        aether_governance::ProposalStatus::Active => "active",
                        aether_governance::ProposalStatus::Passed => "passed",
                        aether_governance::ProposalStatus::Failed => "failed",
                        aether_governance::ProposalStatus::Executed => "executed",
                        aether_governance::ProposalStatus::Cancelled => "cancelled",
                        aether_governance::ProposalStatus::Expired => "expired",
                        aether_governance::ProposalStatus::Vetoed => "vetoed",
                    },
                    "proposer": bs58::encode(p.proposer).into_string(),
                    "created_at": p.created_at,
                    "voting_start": p.voting_start,
                    "voting_end": p.voting_end,
                    "execution_deadline": p.execution_deadline,
                    "tally": {
                        "for_votes": p.tally.for_votes,
                        "against_votes": p.tally.against_votes,
                        "abstain_votes": p.tally.abstain_votes,
                        "voter_count": p.tally.voter_count,
                    },
                })
            }).collect();
            let resp = serde_json::json!({
                "proposals": proposal_list,
                "count": proposal_list.len(),
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get proposals by status
        ("GET", path) if path.starts_with("/v1/governance/proposals/status/") => {
            let status_str = path.strip_prefix("/v1/governance/proposals/status/").unwrap_or("").split('?').next().unwrap_or("");
            let status = match status_str {
                "draft" => Some(aether_governance::ProposalStatus::Draft),
                "pending" => Some(aether_governance::ProposalStatus::Pending),
                "active" => Some(aether_governance::ProposalStatus::Active),
                "passed" => Some(aether_governance::ProposalStatus::Passed),
                "failed" => Some(aether_governance::ProposalStatus::Failed),
                "executed" => Some(aether_governance::ProposalStatus::Executed),
                "cancelled" => Some(aether_governance::ProposalStatus::Cancelled),
                "expired" => Some(aether_governance::ProposalStatus::Expired),
                "vetoed" => Some(aether_governance::ProposalStatus::Vetoed),
                _ => None,
            };
            let status = match status {
                Some(s) => s,
                None => {
                    let resp = serde_json::json!({"error": "Invalid status. Use: draft, pending, active, passed, failed, executed, cancelled, expired, vetoed"});
                    send_response(&mut socket, 400, "Bad Request", &serde_json::to_string(&resp).unwrap_or_default()).await?;
                    return Ok(());
                }
            };
            let proposals = state.get_governance_proposals_by_status(status);
            let proposal_list: Vec<serde_json::Value> = proposals.iter().map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "title": p.title,
                    "status": status_str,
                    "proposer": bs58::encode(p.proposer).into_string(),
                    "created_at": p.created_at,
                    "voting_start": p.voting_start,
                    "voting_end": p.voting_end,
                })
            }).collect();
            let resp = serde_json::json!({
                "proposals": proposal_list,
                "count": proposal_list.len(),
                "status": status_str,
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get a specific proposal
        ("GET", path) if path.starts_with("/v1/governance/proposal/") => {
            let id_str = path.strip_prefix("/v1/governance/proposal/").unwrap_or("").split('?').next().unwrap_or("");
            match id_str.parse::<u64>() {
                Ok(proposal_id) => {
                    match state.get_governance_proposal(proposal_id) {
                        Some(proposal) => {
                            let status_str = match proposal.status {
                                aether_governance::ProposalStatus::Draft => "draft",
                                aether_governance::ProposalStatus::Pending => "pending",
                                aether_governance::ProposalStatus::Active => "active",
                                aether_governance::ProposalStatus::Passed => "passed",
                                aether_governance::ProposalStatus::Failed => "failed",
                                aether_governance::ProposalStatus::Executed => "executed",
                                aether_governance::ProposalStatus::Cancelled => "cancelled",
                                aether_governance::ProposalStatus::Expired => "expired",
                                aether_governance::ProposalStatus::Vetoed => "vetoed",
                            };
                            let vote_list: Vec<serde_json::Value> = proposal.votes.iter().map(|(voter, vote)| {
                                serde_json::json!({
                                    "voter": bs58::encode(voter).into_string(),
                                    "choice": match vote.choice {
                                        aether_governance::VoteChoice::For => "for",
                                        aether_governance::VoteChoice::Against => "against",
                                        aether_governance::VoteChoice::Abstain => "abstain",
                                    },
                                    "weight": vote.weight,
                                    "timestamp": vote.timestamp,
                                })
                            }).collect();
                            let resp = serde_json::json!({
                                "id": proposal.id,
                                "title": proposal.title,
                                "description": proposal.description,
                                "status": status_str,
                                "proposal_type": proposal.proposal_type,
                                "proposer": bs58::encode(proposal.proposer).into_string(),
                                "created_at": proposal.created_at,
                                "voting_start": proposal.voting_start,
                                "voting_end": proposal.voting_end,
                                "execution_deadline": proposal.execution_deadline,
                                "snapshot_block": proposal.snapshot_block,
                                "tally": {
                                    "for_votes": proposal.tally.for_votes,
                                    "against_votes": proposal.tally.against_votes,
                                    "abstain_votes": proposal.tally.abstain_votes,
                                    "voter_count": proposal.tally.voter_count,
                                },
                                "votes": vote_list,
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        None => {
                            let resp = serde_json::json!({"error": "Proposal not found", "proposal_id": proposal_id});
                            (404, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(_) => {
                    let resp = serde_json::json!({"error": "Invalid proposal ID"});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Create a new governance proposal
        ("POST", "/v1/governance/proposal") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let title = json.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled").to_string();
                    let description = json.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let proposer_str = json.get("proposer").and_then(|v| v.as_str()).unwrap_or("");
                    let deposit = json.get("deposit").and_then(|v| v.as_u64()).unwrap_or(0);
                    
                    let proposer_bytes = bs58::decode(proposer_str).into_vec().unwrap_or_default();
                    let mut proposer = [0u8; 32];
                    proposer.copy_from_slice(&proposer_bytes[..32.min(proposer_bytes.len())]);
                    
                    // Parse proposal type from JSON
                    let proposal_type_str = json.get("proposal_type").and_then(|v| v.as_str()).unwrap_or("text");
                    let proposal_type_obj = json.get("proposal_type_details");
                    
                    let proposal_type = match proposal_type_str {
                        "parameter_change" => {
                            let details = proposal_type_obj.unwrap_or(&serde_json::Value::Null);
                            aether_governance::ProposalType::ParameterChange {
                                parameter: details.get("parameter").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                current_value: details.get("current_value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                new_value: details.get("new_value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            }
                        }
                        "fund_allocation" => {
                            let details = proposal_type_obj.unwrap_or(&serde_json::Value::Null);
                            let recipient_str = details.get("recipient").and_then(|v| v.as_str()).unwrap_or("");
                            let recipient_bytes = bs58::decode(recipient_str).into_vec().unwrap_or_default();
                            let mut recipient = [0u8; 32];
                            recipient.copy_from_slice(&recipient_bytes[..32.min(recipient_bytes.len())]);
                            aether_governance::ProposalType::FundAllocation {
                                recipient,
                                amount: details.get("amount").and_then(|v| v.as_u64()).unwrap_or(0),
                                token_type: details.get("token_type").and_then(|v| v.as_str()).unwrap_or("ATH").to_string(),
                                purpose: details.get("purpose").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            }
                        }
                        "protocol_upgrade" => {
                            let details = proposal_type_obj.unwrap_or(&serde_json::Value::Null);
                            let hash_str = details.get("upgrade_hash").and_then(|v| v.as_str()).unwrap_or("");
                            let hash_bytes = bs58::decode(hash_str).into_vec().unwrap_or_default();
                            let mut upgrade_hash = [0u8; 32];
                            upgrade_hash.copy_from_slice(&hash_bytes[..32.min(hash_bytes.len())]);
                            aether_governance::ProposalType::ProtocolUpgrade {
                                version: details.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                description: details.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                upgrade_hash,
                            }
                        }
                        "validator_change" => {
                            let details = proposal_type_obj.unwrap_or(&serde_json::Value::Null);
                            let validator_str = details.get("validator").and_then(|v| v.as_str()).unwrap_or("");
                            let validator_bytes = bs58::decode(validator_str).into_vec().unwrap_or_default();
                            let mut validator = [0u8; 32];
                            validator.copy_from_slice(&validator_bytes[..32.min(validator_bytes.len())]);
                            let action_str = details.get("action").and_then(|v| v.as_str()).unwrap_or("add");
                            let action = match action_str {
                                "remove" => aether_governance::ValidatorAction::Remove,
                                "slash" => aether_governance::ValidatorAction::Slash {
                                    percentage_bps: details.get("percentage_bps").and_then(|v| v.as_u64()).unwrap_or(0),
                                },
                                _ => aether_governance::ValidatorAction::Add,
                            };
                            aether_governance::ProposalType::ValidatorChange { validator, action }
                        }
                        _ => {
                            // Default to text proposal
                            aether_governance::ProposalType::TextProposal {
                                title: title.clone(),
                                description: description.clone(),
                            }
                        }
                    };
                    
                    match state.create_governance_proposal(title, description, proposal_type, proposer, deposit) {
                        Ok(proposal_id) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "proposal_id": proposal_id,
                                "proposer": proposer_str,
                                "deposit": deposit,
                                "message": "Proposal created successfully"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Cast a vote on a proposal
        ("POST", "/v1/governance/vote") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let proposal_id = json.get("proposal_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let voter_str = json.get("voter").and_then(|v| v.as_str()).unwrap_or("");
                    let choice_str = json.get("choice").and_then(|v| v.as_str()).unwrap_or("abstain");
                    let sig_str = json.get("signature").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let voter_bytes = bs58::decode(voter_str).into_vec().unwrap_or_default();
                    let mut voter = [0u8; 32];
                    voter.copy_from_slice(&voter_bytes[..32.min(voter_bytes.len())]);
                    
                    let sig_bytes = bs58::decode(sig_str).into_vec().unwrap_or_default();
                    let mut signature = [0u8; 64];
                    signature.copy_from_slice(&sig_bytes[..64.min(sig_bytes.len())]);
                    
                    let choice = match choice_str {
                        "for" | "yes" => aether_governance::VoteChoice::For,
                        "against" | "no" => aether_governance::VoteChoice::Against,
                        _ => aether_governance::VoteChoice::Abstain,
                    };
                    
                    match state.governance_vote(proposal_id, voter, choice, signature) {
                        Ok(()) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "proposal_id": proposal_id,
                                "voter": voter_str,
                                "choice": choice_str,
                                "message": "Vote cast successfully"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Execute a passed proposal
        ("POST", "/v1/governance/execute") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let proposal_id = json.get("proposal_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    
                    match state.execute_governance_proposal(proposal_id) {
                        Ok(proposal_type) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "proposal_id": proposal_id,
                                "executed_type": proposal_type,
                                "message": "Proposal executed successfully"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Veto a proposal (security council only)
        ("POST", "/v1/governance/veto") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let proposal_id = json.get("proposal_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let vetoer_str = json.get("vetoer").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let vetoer_bytes = bs58::decode(vetoer_str).into_vec().unwrap_or_default();
                    let mut vetoer = [0u8; 32];
                    vetoer.copy_from_slice(&vetoer_bytes[..32.min(vetoer_bytes.len())]);
                    
                    match state.veto_governance_proposal(proposal_id, vetoer) {
                        Ok(()) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "proposal_id": proposal_id,
                                "message": "Proposal vetoed by security council"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Cancel a proposal (proposer only)
        ("POST", "/v1/governance/cancel") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let proposal_id = json.get("proposal_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let canceller_str = json.get("canceller").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let canceller_bytes = bs58::decode(canceller_str).into_vec().unwrap_or_default();
                    let mut canceller = [0u8; 32];
                    canceller.copy_from_slice(&canceller_bytes[..32.min(canceller_bytes.len())]);
                    
                    match state.cancel_governance_proposal(proposal_id, canceller) {
                        Ok(()) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "proposal_id": proposal_id,
                                "message": "Proposal cancelled"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // =================================================================
        // TREASURY ENDPOINTS
        // =================================================================
        // Get treasury summary
        ("GET", "/v1/treasury/summary") => {
            let summary = state.treasury_summary();
            let resp = serde_json::json!({
                "ath_balance": summary.ath_balance,
                "flux_balance": summary.flux_balance,
                "total_fees_collected": summary.total_fees_collected,
                "total_distributed": summary.total_distributed,
                "pending_withdrawals": summary.pending_withdrawals,
                "signer_count": summary.signer_count,
                "current_epoch": summary.current_epoch,
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get treasury budget status
        ("GET", "/v1/treasury/budget") => {
            let budgets = state.treasury_budget_status();
            let budget_list: Vec<serde_json::Value> = budgets.iter().map(|(name, allocated, spent, remaining)| {
                serde_json::json!({
                    "category": name,
                    "allocated": allocated,
                    "spent": spent,
                    "remaining": remaining,
                })
            }).collect();
            let resp = serde_json::json!({
                "budgets": budget_list,
                "count": budget_list.len(),
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get pending treasury withdrawals
        ("GET", "/v1/treasury/withdrawals/pending") => {
            let withdrawals = state.treasury_pending_withdrawals();
            let withdrawal_list: Vec<serde_json::Value> = withdrawals.iter().map(|w| {
                serde_json::json!({
                    "id": w.id,
                    "recipient": bs58::encode(w.recipient).into_string(),
                    "amount": w.amount,
                    "token_type": match w.token_type {
                        aether_governance::TokenType::ATH => "ATH",
                        aether_governance::TokenType::FLUX => "FLUX",
                    },
                    "purpose": w.purpose,
                    "created_at": w.created_at,
                    "execute_after": w.execute_after,
                    "approvals": w.approvals.iter().map(|a| bs58::encode(a).into_string()).collect::<Vec<_>>(),
                    "status": match w.status {
                        aether_governance::WithdrawalStatus::Pending => "pending",
                        aether_governance::WithdrawalStatus::Approved => "approved",
                        aether_governance::WithdrawalStatus::Ready => "ready",
                        aether_governance::WithdrawalStatus::Executed => "executed",
                        aether_governance::WithdrawalStatus::Rejected => "rejected",
                        aether_governance::WithdrawalStatus::Cancelled => "cancelled",
                    },
                })
            }).collect();
            let resp = serde_json::json!({
                "withdrawals": withdrawal_list,
                "count": withdrawal_list.len(),
            });
            (200, serde_json::to_string(&resp).unwrap_or_default())
        }
        // Get a specific treasury withdrawal
        ("GET", path) if path.starts_with("/v1/treasury/withdrawal/") => {
            let id_str = path.strip_prefix("/v1/treasury/withdrawal/").unwrap_or("").split('?').next().unwrap_or("");
            match id_str.parse::<u64>() {
                Ok(withdrawal_id) => {
                    match state.treasury_get_withdrawal(withdrawal_id) {
                        Some(w) => {
                            let resp = serde_json::json!({
                                "id": w.id,
                                "recipient": bs58::encode(w.recipient).into_string(),
                                "amount": w.amount,
                                "token_type": match w.token_type {
                                    aether_governance::TokenType::ATH => "ATH",
                                    aether_governance::TokenType::FLUX => "FLUX",
                                },
                                "purpose": w.purpose,
                                "created_at": w.created_at,
                                "execute_after": w.execute_after,
                                "approvals": w.approvals.iter().map(|a| bs58::encode(a).into_string()).collect::<Vec<_>>(),
                                "status": match w.status {
                                    aether_governance::WithdrawalStatus::Pending => "pending",
                                    aether_governance::WithdrawalStatus::Approved => "approved",
                                    aether_governance::WithdrawalStatus::Ready => "executed",
                                    aether_governance::WithdrawalStatus::Rejected => "rejected",
                                    aether_governance::WithdrawalStatus::Cancelled => "cancelled",
                                    aether_governance::WithdrawalStatus::Executed => "executed",
                                },
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        None => {
                            let resp = serde_json::json!({"error": "Withdrawal not found", "withdrawal_id": withdrawal_id});
                            (404, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(_) => {
                    let resp = serde_json::json!({"error": "Invalid withdrawal ID"});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Create a treasury withdrawal request
        ("POST", "/v1/treasury/withdraw") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let recipient_str = json.get("recipient").and_then(|v| v.as_str()).unwrap_or("");
                    let amount = json.get("amount").and_then(|v| v.as_u64()).unwrap_or(0);
                    let token_type_str = json.get("token_type").and_then(|v| v.as_str()).unwrap_or("ATH");
                    let purpose = json.get("purpose").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let timestamp = json.get("timestamp").and_then(|v| v.as_u64())
                        .unwrap_or_else(|| std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
                    
                    let recipient_bytes = bs58::decode(recipient_str).into_vec().unwrap_or_default();
                    let mut recipient = [0u8; 32];
                    recipient.copy_from_slice(&recipient_bytes[..32.min(recipient_bytes.len())]);
                    
                    let token_type = match token_type_str {
                        "FLUX" => aether_governance::TokenType::FLUX,
                        _ => aether_governance::TokenType::ATH,
                    };
                    
                    match state.treasury_create_withdrawal(recipient, amount, token_type, purpose, timestamp) {
                        Ok(withdrawal_id) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "withdrawal_id": withdrawal_id,
                                "recipient": recipient_str,
                                "amount": amount,
                                "token_type": token_type_str,
                                "message": "Withdrawal request created. Requires multi-sig approval and timelock."
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Approve a treasury withdrawal
        ("POST", "/v1/treasury/approve") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let withdrawal_id = json.get("withdrawal_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let signer_str = json.get("signer").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let signer_bytes = bs58::decode(signer_str).into_vec().unwrap_or_default();
                    let mut signer = [0u8; 32];
                    signer.copy_from_slice(&signer_bytes[..32.min(signer_bytes.len())]);
                    
                    match state.treasury_approve_withdrawal(withdrawal_id, signer) {
                        Ok(()) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "withdrawal_id": withdrawal_id,
                                "message": "Withdrawal approved"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Execute a treasury withdrawal (after timelock)
        ("POST", "/v1/treasury/execute") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let withdrawal_id = json.get("withdrawal_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let tx_hash_str = json.get("tx_hash").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let tx_hash_bytes = bs58::decode(tx_hash_str).into_vec().unwrap_or_default();
                    let mut tx_hash = [0u8; 64];
                    tx_hash.copy_from_slice(&tx_hash_bytes[..64.min(tx_hash_bytes.len())]);
                    
                    let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                    
                    match state.treasury_execute_withdrawal(withdrawal_id, current_time, tx_hash) {
                        Ok(()) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "withdrawal_id": withdrawal_id,
                                "message": "Withdrawal executed successfully"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({"error": format!("Parse error: {}", e)});
                    (400, serde_json::to_string(&resp).unwrap_or_default())
                }
            }
        }
        // Add treasury signer
        ("POST", "/v1/treasury/add_signer") => {
            // body already parsed above
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let signer_str = json.get("signer").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let signer_bytes = bs58::decode(signer_str).into_vec().unwrap_or_default();
                    let mut signer = [0u8; 32];
                    signer.copy_from_slice(&signer_bytes[..32.min(signer_bytes.len())]);
                    
                    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                    
                    match state.treasury_add_signer(signer, timestamp) {
                        Ok(()) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "signer": signer_str,
                                "message": "Treasury signer added"
                            });
                            (200, serde_json::to_string(&resp).unwrap_or_default())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({"error": e});
                            (400, serde_json::to_string(&resp).unwrap_or_default())
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

/// Send a CORS preflight response for OPTIONS requests
async fn send_cors_preflight(socket: &mut TcpStream) -> anyhow::Result<()> {
    let response = format!(
        "HTTP/1.1 204 No Content\r\n\
        Access-Control-Allow-Origin: *\r\n\
        Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
        Access-Control-Allow-Headers: Content-Type, Authorization, X-Request-ID\r\n\
        Access-Control-Max-Age: 86400\r\n\
        Content-Length: 0\r\n\
        Connection: close\r\n\
        \r\n"
    );
    socket.write_all(response.as_bytes()).await?;
    socket.flush().await?;
    Ok(())
}

/// Extract a header value from parsed HTTP request lines (case-insensitive)
fn extract_header<'a>(lines: &'a [&str], header_name: &str) -> Option<&'a str> {
    let name_lower = header_name.to_lowercase();
    for line in lines.iter().skip(1) {
        // Lines after the request line are headers in "Name: Value" format
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_lowercase();
            if key == name_lower {
                return Some(line[colon_pos + 1..].trim());
            }
        }
        // Empty line signals end of headers
        if line.is_empty() {
            break;
        }
    }
    None
}
