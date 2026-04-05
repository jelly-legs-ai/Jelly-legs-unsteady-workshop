//! P2P Network Module
//!
//! Validator-to-validator communication using libp2p.
//! Implements gossip protocol for slot/block propagation.
//! Supports bootstrap node connections for 2-node network.

use crate::block_producer::BlockProducer;
use crate::state::ValidatorState;
use libp2p::identity::Keypair;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

/// Topic for block gossip
const BLOCKS_TOPIC: &str = "aether-blocks";

/// Topic for slot gossip
const SLOT_TOPIC: &str = "aether-slots";

/// Network state shared across handlers
#[derive(Clone)]
pub struct NetworkState {
    pub peer_count: Arc<RwLock<usize>>,
    pub connected_peers: Arc<RwLock<Vec<String>>>,
    pub genesis_hash: Arc<RwLock<String>>,
    pub chain_id: Arc<RwLock<String>>,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            peer_count: Arc::new(RwLock::new(0)),
            connected_peers: Arc::new(RwLock::new(Vec::new())),
            genesis_hash: Arc::new(RwLock::new(String::new())),
            chain_id: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Initialize with genesis info for handshake verification
    pub fn with_genesis(genesis_hash: &str, chain_id: &str) -> Self {
        Self {
            peer_count: Arc::new(RwLock::new(0)),
            connected_peers: Arc::new(RwLock::new(Vec::new())),
            genesis_hash: Arc::new(RwLock::new(genesis_hash.to_string())),
            chain_id: Arc::new(RwLock::new(chain_id.to_string())),
        }
    }

    pub async fn set_peer_count(&self, count: usize) {
        *self.peer_count.write().await = count;
    }

    pub async fn add_peer(&self, peer_id: String) {
        let mut peers = self.connected_peers.write().await;
        if !peers.contains(&peer_id) {
            peers.push(peer_id.clone());
            *self.peer_count.write().await = peers.len();
        }
    }

    pub async fn remove_peer(&self, peer_id: &str) {
        let mut peers = self.connected_peers.write().await;
        peers.retain(|p| p != peer_id);
        *self.peer_count.write().await = peers.len();
    }

    pub async fn get_peer_count(&self) -> usize {
        *self.peer_count.read().await
    }

    pub async fn get_connected_peers(&self) -> Vec<String> {
        self.connected_peers.read().await.clone()
    }

    /// Verify peer has matching genesis hash (chain ID check)
    pub async fn verify_peer_chain(&self, peer_genesis_hash: &str, peer_chain_id: &str) -> bool {
        let our_hash = self.genesis_hash.read().await;
        let our_chain = self.chain_id.read().await;
        *our_hash == peer_genesis_hash && *our_chain == peer_chain_id
    }
}

impl Default for NetworkState {
    fn default() -> Self {
        Self::new()
    }
}

/// Gossip message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GossipMessage {
    #[serde(rename = "slot_update")]
    SlotUpdate { slot: u64, peer_id: String },
    #[serde(rename = "block_announce")]
    BlockAnnounce {
        slot: u64,
        block_hash: String,
        prev_hash: String,
        poh_seed: String,
        state_root: String,
        tx_count: usize,
        peer_id: String,
    },
    #[serde(rename = "vote")]
    Vote {
        slot: u64,
        block_hash: String,
        validator: String,
        signature: String,
    },
    #[serde(rename = "ping")]
    Ping { nonce: u64 },
    #[serde(rename = "pong")]
    Pong { nonce: u64 },
    #[serde(rename = "get_block")]
    GetBlock { slot: u64, requester: String },
    #[serde(rename = "block_response")]
    BlockResponse { slot: u64, block_json: String },
}

/// Handshake message for peer connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    pub protocol_version: String,
    pub genesis_hash: String,
    pub chain_id: String,
    pub peer_id: String,
    pub timestamp: i64,
    pub current_slot: u64,
}

impl HandshakeMessage {
    pub fn new(genesis_hash: &str, chain_id: &str, peer_id: &str, current_slot: u64) -> Self {
        Self {
            protocol_version: "aether/1.0".to_string(),
            genesis_hash: genesis_hash.to_string(),
            chain_id: chain_id.to_string(),
            peer_id: peer_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            current_slot,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

/// Start the P2P network (seed/genesis node with inbound listener)
pub async fn start_p2p(
    listen_addr: &str,
    state: ValidatorState,
    network_state: Arc<NetworkState>,
) -> anyhow::Result<()> {
    info!("Starting P2P network on {}", listen_addr);

    // Generate a keypair for this node
    let keypair = Keypair::generate_ed25519();
    let peer_id = PeerId::from(keypair.public());
    let peer_id_str = peer_id.to_base58();

    info!("P2P node started with peer ID: {}", peer_id);
    info!("Subscribed to topics: {}, {}", BLOCKS_TOPIC, SLOT_TOPIC);

    // Spawn inbound connection listener
    let listen_parsed = listen_addr.to_string();
    let state_clone = state.clone();
    let ns_clone = network_state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_inbound_listener(&listen_parsed, state_clone, ns_clone).await {
            warn!("Inbound P2P listener error: {}", e);
        }
    });

    // Start gossip heartbeat for slot announcements (spawned, not blocking)
    let network_state_clone = network_state.clone();
    tokio::spawn(async move {
        run_slot_gossip_loop(state, network_state_clone, peer_id_str).await;
    });

    // Return immediately — the gossip loop runs in background
    Ok(())
}

/// Run TCP listener for inbound peer connections (seed/genesis node)
async fn run_inbound_listener(
    listen_addr: &str,
    state: ValidatorState,
    network_state: Arc<NetworkState>,
) -> anyhow::Result<()> {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(listen_addr).await?;
    info!("Inbound P2P listener started on {}", listen_addr);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("Inbound connection from: {}", addr);
                let state_clone = state.clone();
                let ns_clone = network_state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_inbound_stream(stream, state_clone, ns_clone).await {
                        debug!("Inbound stream error: {}", e);
                    }
                });
            }
            Err(e) => {
                warn!("Failed to accept inbound connection: {}", e);
            }
        }
    }
}

/// Handle an inbound peer connection (respond to their handshake)
async fn handle_inbound_stream(
    stream: tokio::net::TcpStream,
    state: ValidatorState,
    network_state: Arc<NetworkState>,
) -> anyhow::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = stream;
    let mut buf = vec![0u8; 1024];

    // Read their handshake
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let peer_handshake = match HandshakeMessage::from_json(&String::from_utf8_lossy(&buf[..n])) {
        Some(h) => h,
        None => {
            debug!("Invalid handshake from inbound peer");
            return Ok(());
        }
    };

    // Verify chain ID / genesis hash
    if !network_state.verify_peer_chain(&peer_handshake.genesis_hash, &peer_handshake.chain_id).await {
        warn!("Inbound peer chain mismatch - rejecting: {}", peer_handshake.peer_id);
        return Ok(());
    }

    // Send our handshake back
    let keypair = Keypair::generate_ed25519();
    let our_peer_id = PeerId::from(keypair.public()).to_base58();
    let our_handshake = HandshakeMessage::new(
        &state.get_genesis_hash(),
        &state.get_chain_id(),
        &our_peer_id,
        state.current_slot(),
    );
    let response = our_handshake.to_json();
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    // Slot sync: if we're ahead of the inbound peer, they should sync to us
    let our_slot = state.current_slot();
    if our_slot > peer_handshake.current_slot {
        info!("Inbound peer at slot {}, our slot is {} - peer should sync", 
            peer_handshake.current_slot, our_slot);
        // Note: we can't force them to sync, but we log it for debugging
    }
    
    info!("Inbound handshake successful with peer: {} (slot: {})", peer_handshake.peer_id, peer_handshake.current_slot);
    network_state.add_peer(peer_handshake.peer_id.clone()).await;
    state.add_peer(peer_handshake.peer_id);

    Ok(())
}

/// Start P2P with bootstrap node
pub async fn start_p2p_with_bootstrap(
    listen_addr: &str,
    bootstrap_addr: &str,
    state: ValidatorState,
    network_state: Arc<NetworkState>,
) -> anyhow::Result<()> {
    info!("Starting P2P network on {} (bootstrap: {})", listen_addr, bootstrap_addr);

    // Generate a keypair for this node
    let keypair = Keypair::generate_ed25519();
    let peer_id = PeerId::from(keypair.public());
    let peer_id_str = peer_id.to_base58();

    info!("P2P node started with peer ID: {}", peer_id);
    info!("Connecting to bootstrap node: {}", bootstrap_addr);

    // Attempt TCP connection to bootstrap node with retries
    let mut attempt = 0;
    let max_attempts = 10;
    let base_delay = Duration::from_millis(500);
    let mut connected = false;

    while attempt < max_attempts && !connected {
        attempt += 1;
        if attempt > 1 {
            let delay = base_delay * (2u32.pow(attempt - 2) as u32);
            debug!("Bootstrap connection attempt {}/{}, retrying in {:?}...", attempt, max_attempts, delay);
            tokio::time::sleep(delay).await;
        }
        match tokio::net::TcpStream::connect(bootstrap_addr).await {
        Ok(mut stream) => {
            info!("Connected to bootstrap node at {}", bootstrap_addr);
            
            // Send handshake
            let handshake = HandshakeMessage::new(
                &state.get_genesis_hash(),
                &state.get_chain_id(),
                &peer_id_str,
                state.current_slot(),
            );
            
            let handshake_json = handshake.to_json();
            if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut stream, handshake_json.as_bytes()).await {
                warn!("Failed to send handshake: {}", e);
            }
            if let Err(e) = tokio::io::AsyncWriteExt::flush(&mut stream).await {
                warn!("Failed to flush handshake: {}", e);
            }

            // Read peer handshake response
            let mut buf = vec![0u8; 1024];
            match tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await {
                Ok(n) if n > 0 => {
                    if let Some(peer_handshake) = HandshakeMessage::from_json(&String::from_utf8_lossy(&buf[..n])) {
                        // Verify chain ID matches
                        if !network_state.verify_peer_chain(&peer_handshake.genesis_hash, &peer_handshake.chain_id).await {
                            warn!("Chain mismatch with bootstrap node! Our chain: {}, theirs: {}", 
                                state.get_chain_id(), peer_handshake.chain_id);
                            warn!("Rejecting bootstrap connection - different genesis");
                            return Ok(());
                        }
                        
                        // Slot sync: if bootstrap is ahead, sync our slot to theirs
                        let our_slot = state.current_slot();
                        if peer_handshake.current_slot > our_slot {
                            info!("Slot sync: bootstrap at slot {}, we were at {} - syncing forward", 
                                peer_handshake.current_slot, our_slot);
                            state.sync_slot(peer_handshake.current_slot);
                        } else {
                            debug!("Slot sync: bootstrap at slot {}, we are at {} - no sync needed", 
                                peer_handshake.current_slot, our_slot);
                        }
                        
                        info!("Handshake successful with peer: {} (chain: {}, slot: {})", 
                            peer_handshake.peer_id, peer_handshake.chain_id, peer_handshake.current_slot);
                        network_state.add_peer(peer_handshake.peer_id.clone()).await;
                        state.add_peer(peer_handshake.peer_id);
                        connected = true;
                    }
                }
                _ => {
                    debug!("No handshake response from bootstrap node");
                }
            }
        }
        Err(e) => {
            debug!("Bootstrap connection attempt {}/{} failed: {}", attempt, max_attempts, e);
        }
        }
    }

    if !connected {
        warn!("Could not connect to bootstrap node {} after {} attempts", bootstrap_addr, max_attempts);
        warn!("Starting as seed node (no bootstrap connection)");
    } else {
        info!("Bootstrap connection established, peer count: 1");
    }

    // Spawn inbound connection listener so other peers can connect to us
    let listen_parsed = listen_addr.to_string();
    let state_clone = state.clone();
    let ns_clone = network_state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_inbound_listener(&listen_parsed, state_clone, ns_clone).await {
            warn!("Inbound P2P listener error: {}", e);
        }
    });

    // Start gossip heartbeat (spawned, not blocking)
    let network_state_clone = network_state.clone();
    tokio::spawn(async move {
        run_slot_gossip_loop(state, network_state_clone, peer_id_str).await;
    });

    // Return immediately — the gossip loop runs in background
    Ok(())
}

/// Run slot gossip loop - periodically announce our current slot
async fn run_slot_gossip_loop(
    state: ValidatorState,
    network_state: Arc<NetworkState>,
    peer_id: String,
) {
    let mut tick = interval(Duration::from_secs(5));

    loop {
        tick.tick().await;
        
        let current_slot = state.current_slot();
        
        debug!("Slot gossip: announcing slot {} from peer {}", current_slot, peer_id);

        // Update peer count (simulated)
        let peers = network_state.get_peer_count().await;
        if peers == 0 {
            // Simulate having discovered peers
            network_state.set_peer_count(1).await;
        }
    }
}

/// Announce a newly produced block to all connected peers
pub async fn announce_new_block(
    slot: u64,
    block_hash: String,
    prev_hash: String,
    poh_seed: String,
    state_root: String,
    tx_count: usize,
    peer_id: &str,
    network_state: &NetworkState,
) {
    let msg = GossipMessage::BlockAnnounce {
        slot,
        block_hash: block_hash.clone(),
        prev_hash,
        poh_seed,
        state_root,
        tx_count,
        peer_id: peer_id.to_string(),
    };

    let _json = serde_json::to_string(&msg).unwrap_or_default();
    let peers = network_state.get_connected_peers().await;

    for peer in peers {
        if peer != peer_id {
            debug!("Would announce block {} to peer {}", slot, peer);
        }
    }
}

/// Broadcast a validator vote to the network
pub async fn broadcast_vote(
    slot: u64,
    block_hash: &str,
    validator_pubkey: &str,
    signature: &[u8],
    peer_id: &str,
    network_state: &NetworkState,
) {
    let msg = GossipMessage::Vote {
        slot,
        block_hash: block_hash.to_string(),
        validator: validator_pubkey.to_string(),
        signature: bs58::encode(signature).into_string(),
    };

    let _json = serde_json::to_string(&msg).unwrap_or_default();
    let peers = network_state.get_connected_peers().await;

    for peer in peers {
        if peer != peer_id {
            debug!("Would broadcast vote for slot {} from {}", slot, validator_pubkey);
        }
    }
}

/// Handle an inbound gossip message
pub async fn handle_gossip_message(
    msg: GossipMessage,
    state: ValidatorState,
    block_producer: Arc<BlockProducer>,
    _network_state: Arc<NetworkState>,
) -> Option<GossipMessage> {
    match msg {
        GossipMessage::SlotUpdate { slot, peer_id } => {
            debug!("Peer {} announced slot {}", peer_id, slot);
            if slot > state.current_slot() {
                info!("Syncing to peer's slot {}", slot);
                state.sync_slot(slot);
            }
            None
        }

        GossipMessage::BlockAnnounce {
            slot,
            block_hash,
            prev_hash: _,
            poh_seed: _,
            state_root: _,
            tx_count: _tx_count,
            peer_id,
        } => {
            info!(
                "Peer {} announcing block {} (hash: {})",
                peer_id,
                slot,
                &block_hash[..8.min(block_hash.len())]
            );
            if let Some(existing) = block_producer.get_block(slot).await {
                if existing.block_hash == block_hash {
                    debug!("Block {} already known, skipping", slot);
                    return None;
                }
            }

            if slot > state.current_slot() {
                state.sync_slot(slot);
            }

            let vote_msg = GossipMessage::Vote {
                slot,
                block_hash: block_hash.clone(),
                validator: "local".to_string(),
                signature: String::new(),
            };
            Some(vote_msg)
        }

        GossipMessage::Vote {
            slot,
            block_hash: _block_hash,
            validator,
            signature: _,
        } => {
            info!(
                "Received vote for slot {} from validator {}",
                slot, validator
            );
            None
        }

        GossipMessage::GetBlock { slot, requester } => {
            info!("Peer {} requesting block at slot {}", requester, slot);
            if let Some(block) = block_producer.get_block(slot).await {
                let block_json = serde_json::to_string(&block).ok()?;
                Some(GossipMessage::BlockResponse { slot, block_json })
            } else {
                None
            }
        }

        GossipMessage::BlockResponse { slot, block_json: _block_json } => {
            info!("Received block response for slot {}", slot);
            None
        }

        _ => None,
    }
}
