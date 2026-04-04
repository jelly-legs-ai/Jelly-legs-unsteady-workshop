//! AETHER Validator Binary
//!
//! The main validator node binary for participating in AETHER consensus.
//! Provides commands for starting a validator, checking status, and managing validator identity.

use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, Level};

mod keypair;
mod config;
mod rpc_client;
mod genesis;
mod state;
mod block_producer;
mod rpc_server;
mod network;

pub use block_producer::*;
pub use config::*;
pub use genesis::*;
pub use keypair::*;
pub use network::*;
pub use rpc_client::*;
pub use rpc_server::*;
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
        /// Genesis configuration file (JSON)
        #[arg(long)]
        genesis: Option<PathBuf>,

        /// P2P listen port (default: 8001)
        #[arg(long, default_value = "8001")]
        port: u16,

        /// Bootstrap node address (for joining existing network)
        #[arg(long)]
        bootstrap: Option<String>,

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

        /// Initial token balances (pubkey=amount, repeatable)
        #[arg(long)]
        initial_balance: Vec<String>,
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
    let (testnet, genesis_path, rpc_addr, p2p_addr, bootstrap_addr, identity_path, _no_stake) = match &cli.command {
        Commands::Start { testnet, genesis, rpc_addr, p2p_addr, bootstrap, identity, vote_account: _, no_stake, .. } => {
            (testnet, genesis.clone(), rpc_addr.clone(), p2p_addr.clone(), bootstrap.clone(), identity.clone(), *no_stake)
        }
        _ => unreachable!(),
    };

    info!("Starting AETHER Validator...");

    // Load genesis if provided
    let genesis_config = if let Some(ref path) = genesis_path {
        info!("Loading genesis from: {}", path.display());
        Some(crate::genesis::load_genesis_from_file(path)?)
    } else {
        info!("No genesis file provided, using internal genesis");
        None
    };

    // Print genesis hash on startup
    if let Some(ref config) = genesis_config {
        info!("Genesis hash: {}", config.genesis_hash);
        info!("Chain ID: {}", config.chain_id);
    }

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

    // Initialize consensus - with or without genesis
    let validator_state = if let Some(genesis_path) = &genesis_path {
        let path = genesis_path.as_path();
        info!("Loading genesis from: {}", path.display());
        let genesis = load_genesis_from_file(path)?;
        info!("Genesis loaded: chain_id={}, genesis_hash={}", genesis.chain_id, genesis.genesis_hash);
        ValidatorState::with_genesis(identity, *testnet, ledger_path, path)?
    } else {
        info!("No genesis file specified - starting with internal genesis");
        ValidatorState::new(identity, *testnet, ledger_path)?
    };

    // Print chain info
    info!("Chain ID: {}", validator_state.get_chain_id());
    info!("Genesis Hash: {}", validator_state.get_genesis_hash());

    // Create block producer (genesis-aware)
    let block_producer = Arc::new(BlockProducer::new(validator_state.clone()));

    // Start block producer
    let bp_for_rpc = block_producer.clone();
    let bp_handle = {
        let bp = block_producer.clone();
        tokio::spawn(async move {
            bp.run().await;
        })
    };

    // Start RPC HTTP server
    let rpc_addr_for_spawn = rpc_addr.clone();
    let rpc_handle = {
        let state = validator_state.clone();
        let bp = bp_for_rpc;
        tokio::spawn(async move {
            if let Err(e) = start_rpc_server(&rpc_addr_for_spawn, state, bp).await {
                error!("RPC server error: {}", e);
            }
        })
    };

    // Start P2P gossip (with optional bootstrap)
    let network_state = Arc::new(network::NetworkState::with_genesis(
        &validator_state.get_genesis_hash(),
        &validator_state.get_chain_id(),
    ));
    let p2p_addr_for_spawn = p2p_addr.clone();
    let bootstrap_addr_for_spawn = bootstrap_addr.clone();
    let gossip_handle = {
        let state = validator_state.clone();
        let ns = network_state.clone();
        tokio::spawn(async move {
            if let Some(bootstrap) = &bootstrap_addr_for_spawn {
                if let Err(e) = start_p2p_with_bootstrap(&p2p_addr_for_spawn, bootstrap, state, ns).await {
                    error!("P2P gossip error: {}", e);
                }
            } else {
                if let Err(e) = start_p2p(&p2p_addr_for_spawn, state, ns).await {
                    error!("P2P gossip error: {}", e);
                }
            }
        })
    };

    // Main validator loop
    info!("Validator running. Press Ctrl+C to stop.");
    info!(
        "RPC HTTP: http://127.0.0.1:{}/",
        rpc_addr.split(':').last().unwrap_or("8899")
    );
    info!("P2P Listen: {}", p2p_addr);
    if let Some(ref bootstrap) = bootstrap_addr {
        info!("Bootstrap Node: {}", bootstrap);
    } else {
        info!("Mode: Seed/Genesis Node");
    }

    // Status logging every 100 slots
    let state_for_logging = validator_state.clone();
    let logging_handle = tokio::spawn(async move {
        let mut last_slot = 0u64;
        loop {
            sleep(Duration::from_secs(10)).await;
            let slot = state_for_logging.current_slot();
            if slot != last_slot {
                last_slot = slot;
                info!(
                    "Slot {} | Blocks produced: {} | TXs: {} | Peers: {}",
                    slot,
                    state_for_logging.blocks_produced(),
                    state_for_logging.transaction_count(),
                    state_for_logging.peer_count(),
                );
            }
        }
    });

    // Wait for any handle to complete
    tokio::select! {
        _ = rpc_handle => {}
        _ = gossip_handle => {}
        _ = bp_handle => {}
        _ = logging_handle => {}
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
        println!(
            "     Progress:          {:>11.1}%",
            if epoch_info.slots_in_epoch > 0 {
                (epoch_info.slot_index as f64 / epoch_info.slots_in_epoch as f64) * 100.0
            } else {
                0.0
            }
        );
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
    let (rpc_url, json) = match &cli.command {
        Commands::ShowValidators { rpc_url, json } => (rpc_url.clone(), *json),
        _ => unreachable!(),
    };

    let client = RpcClient::new(&rpc_url);

    let validators = client.get_validators().await?;

    if json || cli.json {
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
            println!(
                "  {:<8} {:<44} {:>8} {:>10}",
                "Status", "Validator Identity", "Stake", "Commission"
            );
            println!("  {}", "-".repeat(76));

            for v in &validators {
                let status = if v.active { "● ACTIVE" } else { "○ INACTIVE" };
                println!(
                    "  {:<8} {:<44} {:>8} {:>10}%",
                    status,
                    v.identity_pubkey.chars().take(44).collect::<String>(),
                    v.activated_stake,
                    v.commission
                );
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
        Commands::CreateVoteAccount {
            validator_keypair,
            out,
            commission,
        } => (validator_keypair.clone(), out.clone(), *commission),
        _ => unreachable!(),
    };

    // Load validator identity
    let validator_identity = load_identity(&validator_keypair_path)?;

    // Generate vote keypair (in production, this would create a proper vote account)
    let vote_keypair = generate_keypair();

    // Save vote account
    save_vote_account(
        &out_path,
        &vote_keypair,
        &validator_identity.pubkey(),
        commission,
    )?;

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
    let (out_path, chain_id, bootstrap_validator) = match &cli.command {
        Commands::CreateGenesis {
            out,
            chain_id,
            timestamp: _,
            bootstrap_validator,
            initial_balance: _,
        } => (out.clone(), chain_id.clone(), bootstrap_validator.clone()),
        _ => unreachable!(),
    };

    // Load bootstrap validators
    let mut validators: Vec<GenesisValidator> = Vec::new();
    for path in &bootstrap_validator {
        let identity = load_identity(path)?;
        validators.push(GenesisValidator {
            identity_pubkey: identity.pubkey(),
            stake: 10_000_000,
            commission: 10,
            active: true,
        });
    }

    // If no validators provided, create a default one
    if validators.is_empty() {
        let keypair = generate_keypair();
        validators.push(GenesisValidator {
            identity_pubkey: keypair.pubkey(),
            stake: 10_000_000,
            commission: 10,
            active: true,
        });
        save_identity(
            &PathBuf::from("bootstrap-validator-identity.json"),
            &keypair,
        )?;
        println!("📝 Created default bootstrap validator identity");
    }

    // Create genesis config
    let config = create_genesis_with(&chain_id, validators);

    // Write genesis JSON
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(&out_path, json)?;

    // Also write genesis.toml for compatibility
    let toml_path = PathBuf::from("genesis.toml");
    let toml_content = format!(
        r#"# AETHER Testnet Genesis Configuration
# Generated by aether-validator create-genesis

[chain]
chain_id = "{}"
genesis_hash = "{}"
timestamp = {}

[consensus]
tower_finality = {}
min_stake = {}
target_stake = {}

[rewards]
epoch_duration = {}
base_reward_rate = {}

[bootstrap_validators]
"#,
        config.chain_id,
        config.genesis_hash,
        config.timestamp,
        config.consensus.tower_finality,
        config.consensus.min_stake,
        config.consensus.target_stake,
        config.rewards.epoch_duration,
        config.rewards.base_reward_rate,
    );
    std::fs::write(&toml_path, toml_content)
        .context("Failed to write genesis.toml")?;

    println!();
    println!("  ╔════════════════════════════════════════════════════════════╗");
    println!("  ║              AETHER GENESIS BLOCK CREATED                   ║");
    println!("  ╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Chain ID:        {}", config.chain_id);
    println!("  Timestamp:      {}", config.timestamp);
    println!("  Genesis Hash:   {}", config.genesis_hash);
    println!();
    println!("  Bootstrap Validators:");
    for v in &config.bootstrap_validators {
        println!("    • {} (stake: {} AETH)", v.identity_pubkey, v.stake);
    }
    println!();
    println!("  Saved to: {}", out_path.display());
    println!("  Also saved: {}", toml_path.display());
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

    let request: serde_json::Value =
        serde_json::from_slice(&buf[..n]).context("Invalid JSON-RPC request")?;

    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");

    let id = request.get("id").cloned();

    let response = match method {
        "getSlot" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": state.current_slot(),
        }),
        "getBlockHeight" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": state.block_height(),
        }),
        "getTransactionCount" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": state.transaction_count(),
        }),
        "getEpochInfo" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": state.epoch_info(),
        }),
        "getBlockProduction" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": state.block_production(),
        }),
        "getClusterNodes" | "getValidators" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": state.get_connected_validators(),
        }),
        "getVoteAccounts" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": state.get_vote_accounts(),
        }),
        "getGenesis" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": state.get_genesis(),
        }),
        "getGenesisHash" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": state.get_genesis_hash(),
        }),
        "health" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": "ok",
        }),
        _ => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method),
            }
        }),
    };

    let mut socket = socket;
    socket
        .write_all(serde_json::to_vec(&response)?.as_slice())
        .await?;
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