//! P2P Network Module
//!
//! Validator-to-validator communication using libp2p gossipsub.
//! Implements gossip protocol for block, transaction, and slot propagation.
//! Supports bootstrap node connections, peer discovery, and connection management.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                           P2P Network Stack                              │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  Gossipsub (Block/Transaction/Vote/Slot gossip)                         │
//! │  ├── Topic: aether-blocks (block announcements)                          │
//! │  ├── Topic: aether-tx (transaction propagation)                         │
//! │  ├── Topic: aether-votes (validator votes)                              │
//! │  └── Topic: aether-slots (slot heartbeats)                               │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  Peer Discovery                                                          │
//! │  ├── Kademlia DHT (peer routing)                                         │
//! │  └── mDNS (local network discovery)                                      │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  Transport: TCP + Noise (encryption) + Yamux (muxing)                   │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```

use crate::block_producer::BlockProducer;
use crate::state::ValidatorState;
use aether_core::AetherTransaction;
use libp2p::{
    gossipsub::{self, Event, IdentTopic, MessageAuthenticity, TopicHash},
    identity::Keypair,
    kad::{self, store::MemoryStore},
    mdns, noise,
    swarm::{Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use futures::StreamExt;
use libp2p_swarm_derive::NetworkBehaviour;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

/// Topic for block gossip
pub const BLOCKS_TOPIC: &str = "aether-blocks";

/// Topic for transaction gossip
pub const TX_TOPIC: &str = "aether-tx";

/// Topic for validator votes
pub const VOTES_TOPIC: &str = "aether-votes";

/// Topic for slot heartbeats
pub const SLOT_TOPIC: &str = "aether-slots";

/// Maximum message size for gossipsub (1MB)
pub const MAX_GOSSIPSUB_MSG_SIZE: usize = 1024 * 1024;

/// Network state shared across handlers
pub struct NetworkState {
    pub peer_count: Arc<RwLock<usize>>,
    pub connected_peers: Arc<RwLock<Vec<String>>>,
    pub genesis_hash: Arc<RwLock<String>>,
    pub chain_id: Arc<RwLock<String>>,
    /// Channel to send outbound messages to the network
    pub outbound_tx: Option<mpsc::UnboundedSender<GossipMessage>>,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            peer_count: Arc::new(RwLock::new(0)),
            connected_peers: Arc::new(RwLock::new(Vec::new())),
            genesis_hash: Arc::new(RwLock::new(String::new())),
            chain_id: Arc::new(RwLock::new(String::new())),
            outbound_tx: None,
        }
    }

    /// Initialize with genesis info for handshake verification
    pub fn with_genesis(genesis_hash: &str, chain_id: &str) -> Self {
        Self {
            peer_count: Arc::new(RwLock::new(0)),
            connected_peers: Arc::new(RwLock::new(Vec::new())),
            genesis_hash: Arc::new(RwLock::new(genesis_hash.to_string())),
            chain_id: Arc::new(RwLock::new(chain_id.to_string())),
            outbound_tx: None,
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
            debug!("Peer added: {} (total: {})", peer_id, peers.len());
        }
    }

    pub async fn remove_peer(&self, peer_id: &str) {
        let mut peers = self.connected_peers.write().await;
        peers.retain(|p| p != peer_id);
        *self.peer_count.write().await = peers.len();
        debug!("Peer removed: {} (total: {})", peer_id, peers.len());
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

/// Gossip message types for the Aether network
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GossipMessage {
    /// Slot heartbeat - announces current slot
    #[serde(rename = "slot_update")]
    SlotUpdate {
        slot: u64,
        peer_id: String,
        block_hash: String,
    },
    
    /// Block announcement - new block produced
    #[serde(rename = "block_announce")]
    BlockAnnounce {
        slot: u64,
        block_hash: String,
        prev_hash: String,
        poh_seed: String,
        state_root: String,
        tx_count: usize,
        peer_id: String,
        /// Serialized block data (for full block sync)
        #[serde(skip_serializing_if = "Option::is_none")]
        block_data: Option<String>,
    },
    
    /// Transaction propagation - new transaction submitted
    #[serde(rename = "tx_propagate")]
    TxPropagate {
        /// Serialized transaction
        tx_data: Vec<u8>,
        peer_id: String,
    },
    
    /// Validator vote on a block
    #[serde(rename = "vote")]
    Vote {
        slot: u64,
        block_hash: String,
        validator: String,
        signature: String,
    },
    
    /// Request full block data
    #[serde(rename = "get_block")]
    GetBlock {
        slot: u64,
        requester: String,
    },
    
    /// Block response with full data
    #[serde(rename = "block_response")]
    BlockResponse {
        slot: u64,
        block_json: String,
    },
    
    /// Sync request - request range of blocks
    #[serde(rename = "sync_request")]
    SyncRequest {
        current_slot: u64,
        target_slot: u64,
        requester: String,
    },
    
    /// Ping for connectivity check
    #[serde(rename = "ping")]
    Ping { nonce: u64 },
    
    /// Pong response
    #[serde(rename = "pong")]
    Pong { nonce: u64 },
    
    /// Handshake for peer verification
    #[serde(rename = "handshake")]
    Handshake {
        protocol_version: String,
        genesis_hash: String,
        chain_id: String,
        peer_id: String,
        current_slot: u64,
    },
}

impl GossipMessage {
    /// Serialize message to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
    
    /// Deserialize message from JSON bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

/// Custom network behaviour combining gossipsub, Kademlia DHT, and mDNS
#[derive(NetworkBehaviour)]
struct AetherNetworkBehaviour {
    /// Gossipsub for block/transaction/vote propagation
    gossipsub: gossipsub::Behaviour,
    /// Kademlia DHT for peer discovery
    kademlia: kad::Behaviour<MemoryStore>,
    /// mDNS for local network discovery
    mdns: mdns::tokio::Behaviour,
}

/// P2P node running the full libp2p stack
pub struct P2PNode {
    /// libp2p swarm
    swarm: Swarm<AetherNetworkBehaviour>,
    /// Our peer ID
    peer_id: PeerId,
    /// Network state reference
    network_state: Arc<NetworkState>,
    /// Validator state reference
    validator_state: ValidatorState,
    /// Block producer for processing received blocks
    block_producer: Option<Arc<BlockProducer>>,
    /// Channel for inbound messages (to application)
    #[allow(dead_code)]
    inbound_tx: mpsc::UnboundedSender<GossipMessage>,
    /// Channel for outbound messages (from application)
    #[allow(dead_code)]
    outbound_rx: mpsc::UnboundedReceiver<GossipMessage>,
    /// Block topic hash
    #[allow(dead_code)]
    blocks_topic: TopicHash,
    /// Transaction topic hash
    #[allow(dead_code)]
    tx_topic: TopicHash,
    /// Vote topic hash
    #[allow(dead_code)]
    votes_topic: TopicHash,
    /// Slot topic hash
    #[allow(dead_code)]
    slot_topic: TopicHash,
}

impl P2PNode {
    /// Create a new P2P node
    pub fn new(
        listen_addr: &str,
        _genesis_hash: &str,
        _chain_id: &str,
        validator_state: ValidatorState,
        network_state: Arc<NetworkState>,
    ) -> Result<Self, String> {
        // Generate or load keypair
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        info!("P2P node starting with peer ID: {}", peer_id);
        
        // Create gossipsub config
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Permissive)
            .max_transmit_size(MAX_GOSSIPSUB_MSG_SIZE)
            .build()
            .map_err(|e| format!("Failed to build gossipsub config: {}", e))?;
        
        // Create gossipsub behaviour
        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| format!("Failed to create gossipsub: {}", e))?;
        
        // Create Kademlia DHT
        let store = MemoryStore::new(peer_id);
        let kademlia = kad::Behaviour::new(peer_id, store);
        
        // Create mDNS for local discovery
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id,
        )
        .map_err(|e| format!("Failed to create mDNS: {}", e))?;
        
        // Create combined behaviour
        let behaviour = AetherNetworkBehaviour {
            gossipsub,
            kademlia,
            mdns,
        };
        
        // Build swarm
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| format!("Failed to create TCP transport: {}", e))?
            .with_behaviour(|_| behaviour)
            .map_err(|e| format!("Failed to create swarm behaviour: {}", e))?
            .with_swarm_config(|c: libp2p::swarm::Config| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();
        
        // Parse listen address
        let listen_multiaddr: Multiaddr = listen_addr
            .parse()
            .map_err(|e| format!("Invalid listen address '{}': {}", listen_addr, e))?;
        
        // Listen on the address
        swarm
            .listen_on(listen_multiaddr.clone())
            .map_err(|e| format!("Failed to listen on {}: {}", listen_multiaddr, e))?;
        
        // Get topic hashes for quick lookup (these are just TopicHash, not Result)
        let blocks_topic = IdentTopic::new(BLOCKS_TOPIC).into();
        let tx_topic = IdentTopic::new(TX_TOPIC).into();
        let votes_topic = IdentTopic::new(VOTES_TOPIC).into();
        let slot_topic = IdentTopic::new(SLOT_TOPIC).into();
        
        // Subscribe to topics (subscribe returns Result<bool, SubscriptionError>)
        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(BLOCKS_TOPIC)) {
            warn!("Failed to subscribe to blocks topic: {:?}", e);
        }
        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(TX_TOPIC)) {
            warn!("Failed to subscribe to tx topic: {:?}", e);
        }
        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(VOTES_TOPIC)) {
            warn!("Failed to subscribe to votes topic: {:?}", e);
        }
        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(SLOT_TOPIC)) {
            warn!("Failed to subscribe to slot topic: {:?}", e);
        }
        
        // Create channels for application communication
        let (inbound_tx, _inbound_rx) = mpsc::unbounded_channel();
        let (_outbound_tx, outbound_rx) = mpsc::unbounded_channel();
        
        Ok(Self {
            swarm,
            peer_id,
            network_state,
            validator_state,
            block_producer: None,
            inbound_tx,
            outbound_rx,
            blocks_topic,
            tx_topic,
            votes_topic,
            slot_topic,
        })
    }
    
    /// Set block producer for processing received blocks
    pub fn with_block_producer(mut self, block_producer: Arc<BlockProducer>) -> Self {
        self.block_producer = Some(block_producer);
        self
    }
    
    /// Get our peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }
    
    /// Connect to a bootstrap peer
    pub fn dial_bootstrap(&mut self, bootstrap_addr: &str) -> Result<(), String> {
        let addr: Multiaddr = bootstrap_addr
            .parse()
            .map_err(|e| format!("Invalid bootstrap address '{}': {}", bootstrap_addr, e))?;
        
        self.swarm
            .dial(addr.clone())
            .map_err(|e| format!("Failed to dial bootstrap {}: {}", addr, e))?;
        
        info!("Dialing bootstrap node at {}", bootstrap_addr);
        Ok(())
    }
    
    /// Start the P2P network event loop
    pub async fn run(mut self) {
        info!("P2P network event loop started");
        
        // Periodic slot heartbeat
        let mut heartbeat_interval = interval(Duration::from_secs(5));
        let peer_id_str = self.peer_id.to_string();
        
        loop {
            tokio::select! {
                // Handle swarm events
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {}", address);
                        }
                        
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                            info!("Connected to peer {} ({})", peer_id, endpoint.get_remote_address());
                            self.network_state.add_peer(peer_id.to_string()).await;
                            self.validator_state.add_peer(peer_id.to_string());
                            
                            // Add to Kademlia DHT
                            self.swarm.behaviour_mut().kademlia.add_address(&peer_id, endpoint.get_remote_address().clone());
                        }
                        
                        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                            if let Some(cause) = cause {
                                warn!("Connection closed with peer {}: {}", peer_id, cause);
                            } else {
                                debug!("Connection closed with peer {}", peer_id);
                            }
                            self.network_state.remove_peer(&peer_id.to_string()).await;
                            self.validator_state.remove_peer(&peer_id.to_string());
                        }
                        
                        SwarmEvent::Behaviour(event) => {
                            match event {
                                AetherNetworkBehaviourEvent::Gossipsub(gossipsub_event) => {
                                    self.handle_gossipsub_event(gossipsub_event).await;
                                }
                                
                                AetherNetworkBehaviourEvent::Kademlia(kad_event) => {
                                    self.handle_kademlia_event(kad_event).await;
                                }
                                
                                AetherNetworkBehaviourEvent::Mdns(mdns_event) => {
                                    self.handle_mdns_event(mdns_event).await;
                                }
                            }
                        }
                        
                        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                            if let Some(peer_id) = peer_id {
                                warn!("Failed to connect to peer {}: {}", peer_id, error);
                            } else {
                                debug!("Outgoing connection error: {}", error);
                            }
                        }
                        
                        SwarmEvent::IncomingConnectionError { error, .. } => {
                            debug!("Incoming connection error: {}", error);
                        }
                        
                        _ => {}
                    }
                }
                
                // Handle outbound messages from application
                Some(msg) = self.outbound_rx.recv() => {
                    self.broadcast_message(&msg).await;
                }
                
                // Periodic heartbeat
                _ = heartbeat_interval.tick() => {
                    self.send_slot_heartbeat(&peer_id_str).await;
                }
            }
        }
    }
    
    /// Handle gossipsub events
    async fn handle_gossipsub_event(&mut self, event: gossipsub::Event) {
        match event {
            Event::Message {
                propagation_source: peer_id,
                message_id: _,
                message,
            } => {
                // Decode the message
                match GossipMessage::from_bytes(&message.data) {
                    Ok(msg) => {
                        let topic = message.topic.as_str();
                        debug!("Received {} message from peer {}", topic, peer_id);
                        self.handle_gossip_message(msg, &peer_id.to_string()).await;
                    }
                    Err(e) => {
                        warn!("Failed to decode gossip message from {}: {}", peer_id, e);
                    }
                }
            }
            
            Event::Subscribed { peer_id, topic } => {
                debug!("Peer {} subscribed to topic {}", peer_id, topic);
            }
            
            Event::Unsubscribed { peer_id, topic } => {
                debug!("Peer {} unsubscribed from topic {}", peer_id, topic);
            }
            
            _ => {}
        }
    }
    
    /// Handle Kademlia DHT events
    async fn handle_kademlia_event(&mut self, event: kad::Event) {
        match event {
            kad::Event::RoutingUpdated { peer, is_new_peer, .. } => {
                if is_new_peer {
                    info!("Kademlia: discovered new peer {}", peer);
                }
            }
            
            _ => {}
        }
    }
    
    /// Handle mDNS events for local discovery
    async fn handle_mdns_event(&mut self, event: mdns::Event) {
        match event {
            mdns::Event::Discovered(peers) => {
                for (peer_id, addr) in peers {
                    info!("mDNS discovered peer {} at {}", peer_id, addr);
                    
                    // Add to Kademlia DHT
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                    
                    // Attempt connection
                    if self.swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(BLOCKS_TOPIC)).is_ok() {
                        debug!("Connected to mDNS peer {}", peer_id);
                    }
                }
            }
            
            mdns::Event::Expired(peers) => {
                for (peer_id, _addr) in peers {
                    debug!("mDNS peer expired: {}", peer_id);
                }
            }
        }
    }
    
    /// Handle incoming gossip message
    async fn handle_gossip_message(&mut self, msg: GossipMessage, _peer_id: &str) {
        match msg {
            GossipMessage::SlotUpdate { slot, peer_id: from_peer, block_hash } => {
                debug!("Slot update from {}: slot {} (hash: {})", from_peer, slot, &block_hash[..8.min(block_hash.len())]);
                
                // Sync if peer is ahead
                let our_slot = self.validator_state.current_slot();
                if slot > our_slot {
                    info!("Peer {} at slot {}, we are at {} - syncing", from_peer, slot, our_slot);
                    self.validator_state.sync_slot(slot);
                }
            }
            
            GossipMessage::BlockAnnounce {
                slot,
                block_hash,
                prev_hash,
                poh_seed,
                state_root: _,
                tx_count,
                peer_id: from_peer,
                block_data,
            } => {
                info!("Block announcement from {}: slot {} ({} TXs)", from_peer, slot, tx_count);
                
                // Update our slot if needed
                let our_slot = self.validator_state.current_slot();
                if slot > our_slot {
                    self.validator_state.sync_slot(slot);
                }
                
                // If we have block_producer, we could process the full block
                if let Some(ref _bp) = self.block_producer {
                    if let Some(_data) = block_data {
                        // In full implementation: deserialize and validate block
                        debug!("Received block data for slot {}", slot);
                    }
                }
                
                // Track that we've seen this block
                debug!("Block {} announced by {} (prev: {}, poh: {})", 
                    &block_hash[..8.min(block_hash.len())], from_peer, 
                    &prev_hash[..8.min(prev_hash.len())], poh_seed);
            }
            
            GossipMessage::TxPropagate { tx_data, peer_id: from_peer } => {
                debug!("Transaction propagated from {} ({} bytes)", from_peer, tx_data.len());
                
                // In full implementation: deserialize and add to mempool
                // For now, just log that we received it
                if let Ok(tx) = bincode::deserialize::<AetherTransaction>(&tx_data) {
                    let sig_b58 = bs58::encode(&tx.signature).into_string();
                    debug!("Received transaction {} from {}", 
                        &sig_b58[..8.min(sig_b58.len())], from_peer);
                }
            }
            
            GossipMessage::Vote { slot, block_hash: _, validator, signature } => {
                debug!("Vote for slot {} from validator {} (sig: {})", 
                    slot, validator, &signature[..8.min(signature.len())]);
                
                // In full implementation: verify signature and count vote
            }
            
            GossipMessage::Handshake {
                protocol_version,
                genesis_hash,
                chain_id,
                peer_id: from_peer,
                current_slot,
            } => {
                // Verify chain compatibility
                let our_genesis = self.network_state.genesis_hash.read().await.clone();
                let our_chain = self.network_state.chain_id.read().await.clone();
                
                if genesis_hash != our_genesis || chain_id != our_chain {
                    warn!("Handshake rejected from {} - chain mismatch (our: {}/{}, theirs: {}/{})",
                        from_peer, our_genesis, our_chain, genesis_hash, chain_id);
                    return;
                }
                
                info!("Handshake successful with {} (protocol: {}, slot: {})",
                    from_peer, protocol_version, current_slot);
                
                // Sync if they're ahead
                let our_slot = self.validator_state.current_slot();
                if current_slot > our_slot {
                    info!("Peer {} is ahead at slot {} (we are at {})", from_peer, current_slot, our_slot);
                }
            }
            
            GossipMessage::GetBlock { slot, requester } => {
                debug!("Block request for slot {} from {}", slot, requester);
                
                // In full implementation: look up block and respond
                if let Some(ref bp) = self.block_producer {
                    if let Some(_block) = bp.get_block(slot).await {
                        // Would send BlockResponse back
                        debug!("Would send block {} to {}", slot, requester);
                    }
                }
            }
            
            GossipMessage::BlockResponse { slot, block_json } => {
                debug!("Received block response for slot {} ({} bytes)", slot, block_json.len());
                // In full implementation: parse and process block
            }
            
            GossipMessage::SyncRequest { current_slot, target_slot, requester } => {
                debug!("Sync request from {}: slots {} to {}", requester, current_slot, target_slot);
                // In full implementation: provide blocks in range
            }
            
            GossipMessage::Ping { nonce } => {
                debug!("Ping received with nonce {}", nonce);
                // Would respond with Pong
            }
            
            GossipMessage::Pong { nonce } => {
                debug!("Pong received with nonce {}", nonce);
            }
        }
    }
    
    /// Broadcast a message to all topics
    async fn broadcast_message(&mut self, msg: &GossipMessage) {
        let topic = match msg {
            GossipMessage::BlockAnnounce { .. } | GossipMessage::GetBlock { .. } | GossipMessage::BlockResponse { .. } => {
                IdentTopic::new(BLOCKS_TOPIC)
            }
            GossipMessage::TxPropagate { .. } => IdentTopic::new(TX_TOPIC),
            GossipMessage::Vote { .. } => IdentTopic::new(VOTES_TOPIC),
            GossipMessage::SlotUpdate { .. } | GossipMessage::Handshake { .. } => IdentTopic::new(SLOT_TOPIC),
            _ => IdentTopic::new(SLOT_TOPIC), // Default
        };
        
        match msg.to_bytes() {
            Ok(data) => {
                if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
                    warn!("Failed to publish message: {:?}", e);
                }
            }
            Err(e) => {
                warn!("Failed to serialize message: {}", e);
            }
        }
    }
    
    /// Send slot heartbeat
    async fn send_slot_heartbeat(&mut self, peer_id: &str) {
        let slot = self.validator_state.current_slot();
        let block_hash = self.validator_state.get_last_block_hash();
        
        let msg = GossipMessage::SlotUpdate {
            slot,
            peer_id: peer_id.to_string(),
            block_hash,
        };
        
        self.broadcast_message(&msg).await;
    }
}

/// Start the P2P network (seed/genesis node with inbound listener)
pub async fn start_p2p(
    listen_addr: &str,
    state: ValidatorState,
    network_state: Arc<NetworkState>,
) -> anyhow::Result<()> {
    info!("Starting P2P network on {} (seed/genesis mode)", listen_addr);
    
    // Get genesis info
    let genesis_hash = state.get_genesis_hash();
    let chain_id = state.get_chain_id();
    
    // Create P2P node
    let node = P2PNode::new(
        listen_addr,
        &genesis_hash,
        &chain_id,
        state.clone(),
        network_state.clone(),
    )
    .map_err(|e| anyhow::anyhow!("Failed to create P2P node: {}", e))?;
    
    let peer_id = node.peer_id();
    info!("P2P node started with peer ID: {}", peer_id);
    info!("Subscribed to topics: {}, {}, {}, {}", BLOCKS_TOPIC, TX_TOPIC, VOTES_TOPIC, SLOT_TOPIC);
    
    // Run the event loop
    node.run().await;
    
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
    
    // Get genesis info
    let genesis_hash = state.get_genesis_hash();
    let chain_id = state.get_chain_id();
    
    // Create P2P node
    let mut node = P2PNode::new(
        listen_addr,
        &genesis_hash,
        &chain_id,
        state.clone(),
        network_state.clone(),
    )
    .map_err(|e| anyhow::anyhow!("Failed to create P2P node: {}", e))?;
    
    let peer_id = node.peer_id();
    info!("P2P node started with peer ID: {}", peer_id);
    info!("Subscribed to topics: {}, {}, {}, {}", BLOCKS_TOPIC, TX_TOPIC, VOTES_TOPIC, SLOT_TOPIC);
    
    // Connect to bootstrap node
    if let Err(e) = node.dial_bootstrap(bootstrap_addr) {
        warn!("Failed to dial bootstrap node: {}", e);
        info!("Continuing in seed mode (no bootstrap connection)");
    }
    
    // Run the event loop
    node.run().await;
    
    Ok(())
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
        block_data: None, // Announce only, not full block
    };
    
    // If we have an outbound channel, send the message
    if let Some(tx) = &network_state.outbound_tx {
        if tx.send(msg).is_err() {
            warn!("Failed to send block announcement to network task");
        }
    }
    
    let peers = network_state.get_connected_peers().await;
    debug!("Announced block {} to {} peers", slot, peers.len());
}

/// Broadcast a transaction to all connected peers
pub async fn broadcast_transaction(
    tx: &AetherTransaction,
    peer_id: &str,
    network_state: &NetworkState,
) {
    // Serialize the transaction
    let tx_data = match bincode::serialize(tx) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to serialize transaction for broadcast: {}", e);
            return;
        }
    };
    
    let msg = GossipMessage::TxPropagate {
        tx_data,
        peer_id: peer_id.to_string(),
    };
    
    // If we have an outbound channel, send the message
    if let Some(tx_ch) = &network_state.outbound_tx {
        if tx_ch.send(msg).is_err() {
            warn!("Failed to send transaction to network task");
        }
    }
    
    let peers = network_state.get_connected_peers().await;
    debug!("Broadcasted transaction to {} peers", peers.len());
}

/// Broadcast a validator vote to the network
pub async fn broadcast_vote(
    slot: u64,
    block_hash: &str,
    validator_pubkey: &str,
    signature: &[u8],
    _peer_id: &str,
    network_state: &NetworkState,
) {
    let msg = GossipMessage::Vote {
        slot,
        block_hash: block_hash.to_string(),
        validator: validator_pubkey.to_string(),
        signature: bs58::encode(signature).into_string(),
    };
    
    // If we have an outbound channel, send the message
    if let Some(tx) = &network_state.outbound_tx {
        if tx.send(msg).is_err() {
            warn!("Failed to send vote to network task");
        }
    }
    
    let peers = network_state.get_connected_peers().await;
    debug!("Broadcasted vote for slot {} to {} peers", slot, peers.len());
}

/// Handle an inbound gossip message (for compatibility with existing code)
#[allow(dead_code)]
pub async fn handle_gossip_message(
    msg: GossipMessage,
    state: ValidatorState,
    block_producer: Arc<BlockProducer>,
    _network_state: Arc<NetworkState>,
) -> Option<GossipMessage> {
    match msg {
        GossipMessage::SlotUpdate { slot, peer_id, block_hash } => {
            debug!("Peer {} announced slot {} (hash: {})", peer_id, slot, &block_hash[..8.min(block_hash.len())]);
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
            tx_count,
            peer_id,
            block_data,
        } => {
            info!(
                "Peer {} announcing block {} (hash: {}, {} TXs)",
                peer_id, slot, &block_hash[..8.min(block_hash.len())], tx_count
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

            // If we have block data, we could process it
            if block_data.is_some() {
                debug!("Received full block data for slot {}", slot);
            }

            None
        }

        GossipMessage::Vote {
            slot,
            block_hash: _,
            validator,
            signature,
        } => {
            info!(
                "Received vote for slot {} from validator {} (sig: {})",
                slot, validator, &signature[..8.min(signature.len())]
            );
            None
        }
        
        GossipMessage::TxPropagate { tx_data, peer_id } => {
            debug!("Received transaction from {} ({} bytes)", peer_id, tx_data.len());
            // In full implementation: add to mempool
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

        GossipMessage::BlockResponse { slot, block_json } => {
            info!("Received block response for slot {} ({} bytes)", slot, block_json.len());
            None
        }
        
        GossipMessage::Handshake { protocol_version, genesis_hash, chain_id, peer_id, current_slot } => {
            debug!("Handshake from {} (protocol: {}, chain: {}, slot: {})",
                peer_id, protocol_version, chain_id, current_slot);
            
            // Verify genesis hash matches
            let our_hash = state.get_genesis_hash();
            let our_chain = state.get_chain_id();
            if genesis_hash != our_hash || chain_id != our_chain {
                warn!("Chain mismatch with peer {}: our {}/{}, theirs {}/{}",
                    peer_id, our_hash, our_chain, genesis_hash, chain_id);
                return None;
            }
            
            // Sync if they're ahead
            if current_slot > state.current_slot() {
                info!("Peer {} ahead at slot {}, syncing", peer_id, current_slot);
                state.sync_slot(current_slot);
            }
            
            None
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gossip_message_serialization() {
        let msg = GossipMessage::SlotUpdate {
            slot: 12345,
            peer_id: "peer-123".to_string(),
            block_hash: "hash-abc".to_string(),
        };
        
        let bytes = msg.to_bytes().unwrap();
        let decoded = GossipMessage::from_bytes(&bytes).unwrap();
        
        match decoded {
            GossipMessage::SlotUpdate { slot, peer_id, block_hash } => {
                assert_eq!(slot, 12345);
                assert_eq!(peer_id, "peer-123");
                assert_eq!(block_hash, "hash-abc");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_block_announce_serialization() {
        let msg = GossipMessage::BlockAnnounce {
            slot: 100,
            block_hash: "block-hash-123".to_string(),
            prev_hash: "prev-hash-456".to_string(),
            poh_seed: "poh-seed-789".to_string(),
            state_root: "state-root-abc".to_string(),
            tx_count: 42,
            peer_id: "validator-1".to_string(),
            block_data: Some("{\"slot\":100}".to_string()),
        };
        
        let bytes = msg.to_bytes().unwrap();
        let decoded = GossipMessage::from_bytes(&bytes).unwrap();
        
        match decoded {
            GossipMessage::BlockAnnounce { slot, tx_count, block_data, .. } => {
                assert_eq!(slot, 100);
                assert_eq!(tx_count, 42);
                assert!(block_data.is_some());
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_network_state() {
        let state = NetworkState::with_genesis("genesis-hash-123", "testnet-1");
        
        assert_eq!(*state.genesis_hash.blocking_read(), "genesis-hash-123");
        assert_eq!(*state.chain_id.blocking_read(), "testnet-1");
        // Peer count starts at 0
        assert_eq!(*state.peer_count.blocking_read(), 0);
    }
}