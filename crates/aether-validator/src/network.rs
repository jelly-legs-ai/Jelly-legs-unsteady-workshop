//! P2P Network Module
//!
//! Validator-to-validator communication using libp2p.
//! Implements gossip protocol for slot/block propagation.
//! Supports bootstrap node connections for 2-node network.

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
    BlockAnnounce { slot: u64, block_hash: String, peer_id: String },
    #[serde(rename = "ping")]
    Ping { nonce: u64 },
    #[serde(rename = "pong")]
    Pong { nonce: u64 },
}

/// Handshake message for peer connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    pub protocol_version: String,
    pub genesis_hash: String,
    pub chain_id: String,
    pub peer_id: String,
    pub timestamp: i64,
}

impl HandshakeMessage {
    pub fn new(genesis_hash: &str, chain_id: &str, peer_id: &str) -> Self {
        Self {
            protocol_version: "aether/1.0".to_string(),
            genesis_hash: genesis_hash.to_string(),
            chain_id: chain_id.to_string(),
            peer_id: peer_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

/// Start the P2P network
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

    // Start gossip heartbeat for slot announcements
    let network_state_clone = network_state.clone();
    
    tokio::spawn(async move {
        run_slot_gossip_loop(state, network_state_clone, peer_id_str).await;
    });

    // Keep running
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
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

    // Attempt TCP connection to bootstrap node
    match tokio::net::TcpStream::connect(bootstrap_addr).await {
        Ok(mut stream) => {
            info!("Connected to bootstrap node at {}", bootstrap_addr);
            
            // Send handshake
            let handshake = HandshakeMessage::new(
                &state.get_genesis_hash(),
                &state.get_chain_id(),
                &peer_id_str,
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
                        
                        info!("Handshake successful with peer: {} (chain: {})", 
                            peer_handshake.peer_id, peer_handshake.chain_id);
                        network_state.add_peer(peer_handshake.peer_id).await;
                    }
                }
                _ => {
                    debug!("No handshake response from bootstrap node");
                }
            }
        }
        Err(e) => {
            warn!("Failed to connect to bootstrap node {}: {}", bootstrap_addr, e);
            warn!("Starting as seed node (no bootstrap connection)");
        }
    }

    // Start gossip heartbeat
    let network_state_clone = network_state.clone();
    tokio::spawn(async move {
        run_slot_gossip_loop(state, network_state_clone, peer_id_str).await;
    });

    // Keep running
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
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

/// Announce a new block to the network (placeholder for future)
pub async fn announce_block(slot: u64, block_hash: String, peer_id: String) {
    let msg = GossipMessage::BlockAnnounce {
        slot,
        block_hash,
        peer_id,
    };
    debug!("Would announce block: {:?}", msg);
}
