//! Gossip protocol for peer-to-peer message dissemination
//!
//! Implements a simple push-pull gossip protocol for broadcasting
//! blocks, transactions, and votes across the validator network.

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

/// Message types that can be gossiped
#[derive(Debug, Clone)]
pub enum GossipMessage {
    /// New block announcement
    Block {
        height: u64,
        hash: [u8; 32],
        data: Vec<u8>,
    },
    /// Transaction propagation
    Transaction {
        signature: [u8; 64],
        data: Vec<u8>,
    },
    /// Validator vote
    Vote {
        validator: [u8; 32],
        slot: u64,
        data: Vec<u8>,
    },
    /// Peer discovery/heartbeat
    Ping {
        from: SocketAddr,
        timestamp: u64,
    },
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer socket address
    pub addr: SocketAddr,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Peer stake weight (for prioritization)
    pub stake_weight: u64,
    /// Whether peer is a validator
    pub is_validator: bool,
}

/// Gossip service state
pub struct GossipService {
    /// Connected peers
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    /// Seen message hashes (for deduplication)
    seen_messages: Arc<RwLock<HashSet<[u8; 32]>>>,
    /// Message buffer for pull requests
    message_buffer: Arc<RwLock<Vec<GossipMessage>>>,
    /// Local socket address
    local_addr: SocketAddr,
}

impl GossipService {
    /// Create new gossip service
    pub fn new(local_addr: SocketAddr) -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            seen_messages: Arc::new(RwLock::new(HashSet::new())),
            message_buffer: Arc::new(RwLock::new(Vec::new())),
            local_addr,
        }
    }

    /// Add or update a peer
    pub async fn add_peer(&self, addr: SocketAddr, stake_weight: u64, is_validator: bool) {
        let mut peers = self.peers.write().await;
        peers.insert(addr, PeerInfo {
            addr,
            last_seen: current_timestamp(),
            stake_weight,
            is_validator,
        });
        debug!("Added peer: {} (validator: {}, stake: {})", addr, is_validator, stake_weight);
    }

    /// Remove a peer
    pub async fn remove_peer(&self, addr: &SocketAddr) {
        let mut peers = self.peers.write().await;
        if peers.remove(addr).is_some() {
            debug!("Removed peer: {}", addr);
        }
    }

    /// Broadcast a message to all peers
    pub async fn broadcast(&self, message: GossipMessage) {
        // Compute message hash for deduplication
        let msg_hash = hash_gossip_message(&message);
        
        // Check if we've seen this message before
        {
            let mut seen = self.seen_messages.write().await;
            if seen.contains(&msg_hash) {
                debug!("Skipping duplicate message: {:?}", msg_hash);
                return;
            }
            seen.insert(msg_hash);
        }

        // Add to message buffer for pull requests
        {
            let mut buffer = self.message_buffer.write().await;
            buffer.push(message.clone());
            // Keep buffer size manageable
            if buffer.len() > 1000 {
                buffer.remove(0);
            }
        }

        // Broadcast to all peers (in real impl, would use UDP/TCP)
        let peers = self.peers.read().await;
        let peer_count = peers.len();
        
        info!("Broadcasting message to {} peers", peer_count);
        
        // TODO: Implement actual network send via libp2p or UDP
        // For now, just log the broadcast
        for (addr, _peer) in peers.iter() {
            debug!("Would send to peer: {}", addr);
        }
    }

    /// Request messages from peers (pull-based)
    pub async fn request_messages(&self, since_slot: u64) -> Vec<GossipMessage> {
        // In real impl, would send pull requests to peers
        // For now, return buffered messages
        let buffer = self.message_buffer.read().await;
        buffer.iter()
            .filter(|msg| match msg {
                GossipMessage::Block { height, .. } => *height >= since_slot,
                GossipMessage::Vote { slot, .. } => *slot >= since_slot,
                _ => true,
            })
            .cloned()
            .collect()
    }

    /// Get active peer count
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Get validator peer count
    pub async fn validator_count(&self) -> usize {
        self.peers.read().await
            .values()
            .filter(|p| p.is_validator)
            .count()
    }

    /// Prune old seen messages to prevent memory growth
    pub async fn prune_seen_messages(&self, max_age_seconds: u64) {
        // In real impl, would track timestamps with seen messages
        // For now, just clear if buffer gets too large
        let mut seen = self.seen_messages.write().await;
        if seen.len() > 10000 {
            // Clear oldest 50%
            let to_remove: Vec<_> = seen.iter().take(seen.len() / 2).copied().collect();
            for hash in to_remove {
                seen.remove(&hash);
            }
            debug!("Pruned old seen messages");
        }
    }
}

/// Compute hash of a gossip message for deduplication
fn hash_gossip_message(msg: &GossipMessage) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    match msg {
        GossipMessage::Block { height, hash, .. } => {
            hasher.update(b"block");
            hasher.update(height.to_le_bytes());
            hasher.update(hash);
        }
        GossipMessage::Transaction { signature, .. } => {
            hasher.update(b"tx");
            hasher.update(signature);
        }
        GossipMessage::Vote { validator, slot, .. } => {
            hasher.update(b"vote");
            hasher.update(validator);
            hasher.update(slot.to_le_bytes());
        }
        GossipMessage::Ping { from, timestamp } => {
            hasher.update(b"ping");
            hasher.update(from.to_string().as_bytes());
            hasher.update(timestamp.to_le_bytes());
        }
    }
    hasher.finalize().into()
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gossip_broadcast() {
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let gossip = GossipService::new(addr);
        
        let msg = GossipMessage::Block {
            height: 100,
            hash: [1u8; 32],
            data: vec![1, 2, 3],
        };
        
        gossip.broadcast(msg).await;
        
        // Message should be in buffer
        let buffer = gossip.message_buffer.read().await;
        assert_eq!(buffer.len(), 1);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let gossip = GossipService::new(addr);
        
        let peer_addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        gossip.add_peer(peer_addr, 1000, true).await;
        
        assert_eq!(gossip.peer_count().await, 1);
        assert_eq!(gossip.validator_count().await, 1);
        
        gossip.remove_peer(&peer_addr).await;
        assert_eq!(gossip.peer_count().await, 0);
    }
}