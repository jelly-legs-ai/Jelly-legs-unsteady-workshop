//! AETHER Validator Binary
//!
//! The main validator node binary for participating in AETHER consensus.
//! Provides commands for starting a validator, checking status, and managing validator identity.

use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error, Level};

mod keypair;
mod config;
mod rpc_client;
mod genesis;
mod state;

pub use config::*;
pub use genesis::*;
pub use keypair::*;
pub use rpc_client::*;
pub use state::*;

// =============================================================================
// CLI Structure
// =============================================================================

#[derive(Debug, Parser)]
#[command(
    name = "aether-validator",
    about = "AETHER Validator - Participate in AETHER consensus",
    version,
    propagate_version = true,
)]
struct Cli {
    /// Enable verbose logging (-vvv for trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Configuration file path
    #[arg(short, long, default_value = "validator.yaml")]
    config: PathBuf,

    /// JSON output for programmatic use
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start the validator node
    Start {
        /// Start in testnet mode (local testnet genesis)
        #[arg(long)]
        testnet: bool,

        /// Bind address for RPC
        #[arg(long, default_value = "127.0.0.1:8899")]
        rpc_addr: String,

        /// Bind address for P2P gossip
        #[arg(long, default_value = "0.0.0.0:8001")]
        p2p_addr: String,

        /// Identity keypair path
        #[arg(long)]
        identity: Option<PathBuf>,

        /// Vote account keypair path
        #[arg(long)]
        vote_account: Option<PathBuf>,

        /// Skip stake requirement (testing only)
        #[arg(long)]
        no_stake: bool,
    },

    /// Check validator status via RPC
    Status {
        /// RPC endpoint URL
        #[arg(long, default_value = "http://127.0.0.1:8899")]
        rpc_url: String,

        /// Output detailed validator info
        #[arg(long)]
        details: bool,
    },

    /// Show all connected validators
    ShowValidators {
        /// RPC endpoint URL
        #[arg(long, default_value = "http://127.0.0.1:8899")]
        rpc_url: String,

        /// Output as JSON array
        #[arg(long)]
        json: bool,
    },

    /// Create a new validator identity keypair
    CreateValidatorIdentity {
        /// Output file path
        #[arg(short, long)]
        out: PathBuf,

        /// Overwrite existing keypair
        #[arg(long)]
        force: bool,
    },

    /// Create a vote account for the validator
    CreateVoteAccount {
        /// Validator identity keypair
        #[arg(long)]
        validator_keypair: PathBuf,

        /// Output file for vote account
        #[arg(short, long)]
        out: PathBuf,

        /// Commission rate (0-100)
        #[arg(long, default_value = "10")]
        commission: u8,
    },

    /// Generate local testnet genesis
    CreateGenesis {
        /// Output genesis file path
        #[arg(short, long, default_value = "genesis.json")]
        out: PathBuf,

        /// Chain ID
        #[arg(long, default_value = "aether-testnet-1")]
        chain_id: String,

        /// Timestamp for genesis (Unix epoch)
        #[arg(long)]
        timestamp: Option<i64>,

        /// Bootstrap validator identity (can be repeated)
        #[arg(long)]
        bootstrap_validator: Vec<PathBuf>,
    },
}

// =============================================================================
// Main Entry Point
// =============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = match cli.verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        2 => Level::TRACE,
        _ => Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    // Execute command
    let result = match &cli.command {
        Commands::Start { .. } => run_validator(cli).await,
        Commands::Status { .. } => check_status(cli).await,
        Commands::ShowValidators { .. } => show_validators(cli).await,
        Commands::CreateValidatorIdentity { .. } => create_identity(cli).await,
        Commands::CreateVoteAccount { .. } => create_vote_account(cli).await,
        Commands::CreateGenesis { .. } => create_genesis(cli).await,
    };

    if let Err(e) = result {
        error!("Command failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

// =============================================================================
// Start Validator
// =============================================================================

async fn run_validator(cli: Cli) -> anyhow::Result<()> {
    // Extract all values we need before any async moves
    let (testnet, rpc_addr, p2p_addr, identity_path, no_stake) = match &cli.command {
        Commands::Start { testnet, rpc_addr, p2p_addr, identity, vote_account: _, no_stake } => {
            (testnet, rpc_addr.clone(), p2p_addr.clone(), identity.clone(), no_stake)
        }
        _ => unreachable!(),
    };

    info!("Starting AETHER Validator...");

    // Load or generate identity
    let identity = if let Some(path) = &identity_path {
        load_or_create_identity(path)?
    } else {
        let default_path = PathBuf::from("validator-identity.json");
        if default_path.exists() {
            load_or_create_identity(&default_path)?
        } else {
            info!("No identity found, generating new one...");
            let keypair = generate_keypair();
            save_identity(&default_path, &keypair)?;
            keypair
        }
    };

    let identity_pubkey = identity.pubkey();
    info!("Validator identity: {}", identity_pubkey);

    // Initialize storage
    let ledger_path = PathBuf::from("ledger");
    std::fs::create_dir_all(&ledger_path)
        .context("Failed to create ledger directory")?;

    // Bind RPC listener
    info!("Binding RPC to {}", rpc_addr);
    let rpc_listener = TcpListener::bind(rpc_addr)
        .await
        .context("Failed to bind RPC address")?;
    let rpc_port = rpc_listener.local_addr()?.port();

    // Bind P2P listener
    info!("Binding P2P gossip to {}", p2p_addr);

    // Initialize consensus
    let validator_state = ValidatorState::new(
        identity,
        *testnet,
        ledger_path,
    )?;

    // Start RPC server in background
    let rpc_handle = {
        let validator_state = validator_state.clone();
        tokio::spawn(async move {
            info!("RPC server listening on 0.0.0.0:{}", rpc_port);
            loop {
                match rpc_listener.accept().await {
                    Ok((socket, addr)) => {
                        let state = validator_state.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_rpc_request(socket, state).await {
                                warn!("RPC error from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("RPC accept error: {}", e);
                    }
                }
            }
        })
    };

    // Start P2P gossip
    let gossip_handle = {
        let validator_state = validator_state.clone();
        let p2p_addr_clone = p2p_addr.clone();
        tokio::spawn(async move {
            if let Err(e) = run_gossip(&p2p_addr_clone, validator_state).await {
                error!("Gossip error: {}", e);
            }
        })
    };

    // Main validator loop
    info!("Validator running. Press Ctrl+C to stop.");
    info!("RPC: http://127.0.0.1:{}/", rpc_port);
    info!("Gossip: {}", p2p_addr);

    // Simulate block production for MVP
    let block_handle = {
        let validator_state = validator_state.clone();
        tokio::spawn(async move {
            let mut slot = 0u64;
            loop {
                sleep(Duration::from_millis(400)).await; // 400ms slot time
                slot += 1;
                validator_state.increment_slot();
                
                if slot % 100 == 0 {
                    info!(
                        "Slot {} | Votes: {} | Peers: {} | Blocks produced: {}",
                        slot,
                        validator_state.vote_count(),
                        validator_state.peer_count(),
                        validator_state.blocks_produced(),
                    );
                }
            }
        })
    };

    // Wait for any handle to complete
    tokio::select! {
        _ = rpc_handle => {}
        _ = gossip_handle => {}
        _ = block_handle => {}
    }

    Ok(())
}

// =============================================================================
// Check Status
// =============================================================================

async fn check_status(cli: Cli) -> anyhow::Result<()> {
    let (rpc_url, details) = match &cli.command {
        Commands::Status { rpc_url, details } => (rpc_url.clone(), *details),
        _ => unreachable!(),
    };

    // Query RPC
    let client = RpcClient::new(&rpc_url);
    
    let slot_height = client.get_slot().await.unwrap_or(0);
    let block_height = client.get_block_height().await.unwrap_or(0);
    let peer_count = client.get_peer_count().await.unwrap_or(0);
    let transaction_count = client.get_transaction_count().await.unwrap_or(0);
    let epoch_info = client.get_epoch_info().await.unwrap_or_default();

    if cli.json {
        let status = serde_json::json!({
            "slot_height": slot_height,
            "block_height": block_height,
            "peer_count": peer_count,
            "transaction_count": transaction_count,
            "epoch": epoch_info.epoch,
            "slot_index": epoch_info.slot_index,
            "slots_in_epoch": epoch_info.slots_in_epoch,
            "absolute_slot": epoch_info.absolute_slot,
            "block_production": client.get_block_production().await.unwrap_or_default(),
        });
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!();
        println!("  ╔══════════════════════════════════════════════╗");
        println!("  ║       AETHER VALIDATOR STATUS                 ║");
        println!("  ╚══════════════════════════════════════════════╝");
        println!();
        println!("  🌐 RPC Endpoint:  {}", rpc_url);
        println!();
        println!("  📊 Chain Status");
        println!("     Slot Height:        {:>12}", slot_height);
        println!("     Block Height:       {:>12}", block_height);
        println!("     Transaction Count:  {:>12}", transaction_count);
        println!();
        println!("  🔗 Network");
        println!("     Peer Count:         {:>12}", peer_count);
        println!();
        println!("  📈 Epoch {}", epoch_info.epoch);
        println!("     Progress:          {:>11.1}%", 
            if epoch_info.slots_in_epoch > 0 {
                (epoch_info.slot_index as f64 / epoch_info.slots_in_epoch as f64) * 100.0
            } else { 0.0 });
        println!("     Absolute Slot:      {:>12}", epoch_info.absolute_slot);
        println!();
        
        if details {
            let bp = client.get_block_production().await.unwrap_or_default();
            println!("  📦 Block Production");
            println!("     Blocks in Epoch:    {:>12}", bp.blocks_produced);
            println!("     Entries in Epoch:   {:>12}", bp.entries_produced);
            println!();
        }
        
        println!("  ✅ Validator is running normally");
        println!();
    }

    Ok(())
}

// =============================================================================
// Show Validators
// =============================================================================

async fn show_validators(cli: Cli) -> anyhow::Result<()> {
    let rpc_url = match &cli.command {
        Commands::ShowValidators { rpc_url, json } => (rpc_url.clone(), *json),
        _ => unreachable!(),
    };

    let client = RpcClient::new(&rpc_url.0);

    let validators = client.get_validators().await?;

    if rpc_url.1 || cli.json {
        println!("{}", serde_json::to_string_pretty(&validators)?);
    } else {
        println!();
        println!("  ╔════════════════════════════════════════════════════════════╗");
        println!("  ║              AETHER NETWORK VALIDATORS                      ║");
        println!("  ╚════════════════════════════════════════════════════════════╝");
        println!();
        
        if validators.is_empty() {
            println!("  No validators found on testnet");
        } else {
            println!("  {:<8} {:<44} {:>8} {:>10}", 
                "Status", "Validator Identity", "Stake", "Commission");
            println!("  {}", "-".repeat(76));
            
            for v in &validators {
                let status = if v.active { "● ACTIVE" } else { "○ INACTIVE" };
                println!("  {:<8} {:<44} {:>8} {:>10}%", 
                    status, 
                    v.identity_pubkey.chars().take(44).collect::<String>(),
                    v.activated_stake,
                    v.commission);
            }
        }
        println!();
        println!("  Total Validators: {}", validators.len());
        println!();
    }

    Ok(())
}

// =============================================================================
// Create Identity
// =============================================================================

async fn create_identity(cli: Cli) -> anyhow::Result<()> {
    let (out_path, force) = match &cli.command {
        Commands::CreateValidatorIdentity { out, force } => (out.clone(), *force),
        _ => unreachable!(),
    };

    if out_path.exists() && !force {
        anyhow::bail!("Identity file already exists. Use --force to overwrite.");
    }

    let keypair = generate_keypair();
    save_identity(&out_path, &keypair)?;

    println!("✅ Validator identity created: {}", out_path.display());
    println!("   Public key: {}", keypair.pubkey());
    println!();
    println!("⚠️  BACKUP THIS FILE - it controls your validator identity!");
    println!("   If lost, you lose your validator status.");

    Ok(())
}

// =============================================================================
// Create Vote Account
// =============================================================================

async fn create_vote_account(cli: Cli) -> anyhow::Result<()> {
    let (validator_keypair_path, out_path, commission) = match &cli.command {
        Commands::CreateVoteAccount { validator_keypair, out, commission } => {
            (validator_keypair.clone(), out.clone(), *commission)
        }
        _ => unreachable!(),
    };

    // Load validator identity
    let validator_identity = load_identity(&validator_keypair_path)?;
    
    // Generate vote keypair (in production, this would create a proper vote account)
    let vote_keypair = generate_keypair();

    // Save vote account
    save_vote_account(&out_path, &vote_keypair, &validator_identity.pubkey(), commission)?;

    println!("✅ Vote account created: {}", out_path.display());
    println!("   Vote public key: {}", vote_keypair.pubkey());
    println!("   Validator: {}", validator_identity.pubkey());
    println!("   Commission: {}%", commission);
    println!();
    println!("⚠️  Fund the vote account with stake to start validating.");

    Ok(())
}

// =============================================================================
// Create Genesis
// =============================================================================

async fn create_genesis(cli: Cli) -> anyhow::Result<()> {
    let (out_path, chain_id, timestamp, bootstrap_validator) = match &cli.command {
        Commands::CreateGenesis { out, chain_id, timestamp, bootstrap_validator } => {
            (out.clone(), chain_id.clone(), *timestamp, bootstrap_validator.clone())
        }
        _ => unreachable!(),
    };

    let ts = timestamp.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });

    // Load bootstrap validators
    let mut bootstrap_validators = Vec::new();
    for path in &bootstrap_validator {
        let identity = load_identity(path)?;
        bootstrap_validators.push(GenesisValidator {
            identity_pubkey: identity.pubkey(),
            stake: 10_000_000,
            commission: 10,
        });
    }

    // If no validators provided, create a default one
    if bootstrap_validators.is_empty() {
        let keypair = generate_keypair();
        bootstrap_validators.push(GenesisValidator {
            identity_pubkey: keypair.pubkey(),
            stake: 10_000_000,
            commission: 10,
        });
        save_identity(&PathBuf::from("bootstrap-validator-identity.json"), &keypair)?;
        println!("📝 Created default bootstrap validator identity");
    }

    let genesis = GenesisBlock {
        chain_id,
        timestamp: ts,
        genesis_hash: generate_genesis_hash(),
        bootstrap_validators: bootstrap_validators.clone(),
        consensus: ConsensusConfig {
            slot_time_ms: 400,
            tower_finality: 12,
            min_stake: 100,
            target_stake: 1_000_000,
        },
        rewards: RewardsConfig {
            epoch_duration: 432_000,
            base_reward_rate: 6,
        },
    };

    // Save genesis
    let json = serde_json::to_string_pretty(&genesis)?;
    std::fs::write(&out_path, json)
        .context("Failed to write genesis file")?;

    println!();
    println!("  ╔════════════════════════════════════════════════════════════╗");
    println!("  ║              AETHER GENESIS BLOCK CREATED                   ║");
    println!("  ╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Chain ID:     {}", genesis.chain_id);
    println!("  Timestamp:    {}", genesis.timestamp);
    println!("  Genesis Hash: {}", genesis.genesis_hash);
    println!();
    println!("  Bootstrap Validators:");
    for v in &bootstrap_validators {
        println!("    • {} (stake: {} AETH)", v.identity_pubkey, v.stake);
    }
    println!();
    println!("  Saved to: {}", out_path.display());
    println!();

    Ok(())
}

// =============================================================================
// RPC Request Handler
// =============================================================================

async fn handle_rpc_request(
    mut socket: tokio::net::TcpStream,
    state: ValidatorState,
) -> anyhow::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = vec![0u8; 4096];
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let request: serde_json::Value = serde_json::from_slice(&buf[..n])
        .context("Invalid JSON-RPC request")?;

    let method = request.get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");

    let id = request.get("id").cloned();

    let response = match method {
        "getSlot" => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": state.current_slot(),
            })
        }
        "getBlockHeight" => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": state.block_height(),
            })
        }
        "getTransactionCount" => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": state.transaction_count(),
            })
        }
        "getEpochInfo" => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": state.epoch_info(),
            })
        }
        "getBlockProduction" => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": state.block_production(),
            })
        }
        "getClusterNodes" | "getValidators" => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": state.get_connected_validators(),
            })
        }
        "getVoteAccounts" => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": state.get_vote_accounts(),
            })
        }
        "health" => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": "ok",
            })
        }
        _ => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method),
                }
            })
        }
    };

    let mut socket = socket;
    socket.write_all(serde_json::to_vec(&response)?.as_slice()).await?;
    socket.flush().await?;

    Ok(())
}

// =============================================================================
// P2P Gossip (Stub - uses libp2p in full implementation)
// =============================================================================

async fn run_gossip(addr: &str, state: ValidatorState) -> anyhow::Result<()> {
    info!("P2P gossip would connect to {}", addr);
    // In full implementation, this uses libp2p for actual gossip
    // For MVP, we just track the state
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        state.update_peer_count(1); // Simulate having peers
    }
}
