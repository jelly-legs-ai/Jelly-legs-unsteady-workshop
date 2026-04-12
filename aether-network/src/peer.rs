//! Peer management for P2P networking
//!
//! Provides:
//! - Peer discovery and tracking
//! - Health monitoring and scoring
//! - Stake-weighted peer selection
//! - Connection lifecycle management
//! - Gossip protocol integration

use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use rand::Rng;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

/// Maximum number of peers to maintain connections with
const MAX_PEERS: usize = 200;

/// Maximum number of peers in discovery queue
const MAX_DISCOVERY_QUEUE: usize = 500;

/// How long before a peer is considered stale (seconds)
const STALE_THRESHOLD_SECS: u64 = 120;

/// How long before removing a dead peer (seconds)
const DEAD_THRESHOLD_SECS: u64 = 300;

/// Peer connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    /// Attempting to connect
    Connecting,
    /// Successfully connected and active
    Active,
    /// Temporarily disconnected, will retry
    Disconnected,
    /// Permanently removed
    Banned,
}

/// Peer metadata and statistics
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer socket address
    pub addr: SocketAddr,
    /// Peer public key (for validator identification)
    pub pubkey: Option<[u8; 32]>,
    /// Current connection state
    pub state: PeerState,
    /// Stake weight (0 for non-validators)
    pub stake_weight: u64,
    /// Whether this peer is a validator
    pub is_validator: bool,
    /// Number of successful connections
    pub successful_connections: u32,
    /// Number of failed connection attempts
    pub failed_connections: u32,
    /// Last seen timestamp (Unix seconds)
    pub last_seen: u64,
    /// Last attempted connection
    pub last_attempt: Option<u64>,
    /// Latency in milliseconds (rolling average)
    pub latency_ms: f64,
    /// Bytes received from this peer
    pub bytes_received: u64,
    /// Bytes sent to this peer
    pub bytes_sent: u64,
    /// Blocks received from this peer
    pub blocks_received: u64,
    /// Reputation score (0-100)
    pub reputation: f64,
}

impl PeerInfo {
    /// Create a new peer entry
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            pubkey: None,
            state: PeerState::Connecting,
            stake_weight: 0,
            is_validator: false,
            successful_connections: 0,
            failed_connections: 0,
            last_seen: current_timestamp(),
            last_attempt: None,
            latency_ms: 0.0,
            bytes_received: 0,
            bytes_sent: 0,
            blocks_received: 0,
            reputation: 50.0, // Start at neutral
        }
    }

    /// Create a new validator peer
    pub fn new_validator(addr: SocketAddr, pubkey: [u8; 32], stake: u64) -> Self {
        Self {
            addr,
            pubkey: Some(pubkey),
            state: PeerState::Connecting,
            stake_weight: stake,
            is_validator: true,
            reputation: 70.0, // Validators start with higher trust
            ..Self::new(addr)
        }
    }

    /// Check if peer is currently active
    pub fn is_active(&self) -> bool {
        self.state == PeerState::Active
    }

    /// Check if peer is considered healthy
    pub fn is_healthy(&self) -> bool {
        if !self.is_active() {
            return false;
        }
        let now = current_timestamp();
        let age = now.saturating_sub(self.last_seen);
        age < STALE_THRESHOLD_SECS && self.reputation > 20.0
    }

    /// Get peer score for connection prioritization
    pub fn connection_score(&self) -> f64 {
        let base = match self.state {
            PeerState::Active => 1.0,
            PeerState::Connecting => 0.5,
            PeerState::Disconnected => 0.1,
            PeerState::Banned => 0.0,
        };

        let stake_bonus = if self.stake_weight > 0 {
            (self.stake_weight as f64).ln() * 0.1
        } else {
            0.0
        };

        let reputation_factor = self.reputation / 100.0;
        let latency_factor = if self.latency_ms > 0.0 {
            1.0 / (1.0 + self.latency_ms / 100.0)
        } else {
            1.0
        };

        base * reputation_factor * latency_factor + stake_bonus
    }
}

/// Peer manager for the P2P network
pub struct PeerManager {
    /// Known peers indexed by address
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    /// Peers in discovery queue (not yet connected)
    discovery_queue: Arc<RwLock<VecDeque<SocketAddr>>>,
    /// Set of banned peer addresses
    banned: Arc<RwLock<HashSet<SocketAddr>>>,
    /// Our own stake weight (for stake-weighted selection)
    own_stake: Arc<RwLock<u64>>,
}

impl PeerManager {
    /// Create a new peer manager
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            discovery_queue: Arc::new(RwLock::new(VecDeque::new())),
            banned: Arc::new(RwLock::new(HashSet::new())),
            own_stake: Arc::new(RwLock::new(0)),
        }
    }

    /// Add a peer to the network
    pub async fn add_peer(&self, addr: SocketAddr) -> Result<(), PeerError> {
        // Check ban list
        if self.banned.read().await.contains(&addr) {
            return Err(PeerError::PeerBanned(addr));
        }

        let mut peers = self.peers.write().await;
        if peers.len() >= MAX_PEERS {
            return Err(PeerError::MaxPeersReached);
        }

        if !peers.contains_key(&addr) {
            peers.insert(addr, PeerInfo::new(addr));
            debug!("Added peer: {}", addr);
        }
        Ok(())
    }

    /// Add a validator peer
    pub async fn add_validator_peer(
        &self,
        addr: SocketAddr,
        pubkey: [u8; 32],
        stake: u64,
    ) -> Result<(), PeerError> {
        if self.banned.read().await.contains(&addr) {
            return Err(PeerError::PeerBanned(addr));
        }

        let mut peers = self.peers.write().await;
        if peers.len() >= MAX_PEERS {
            return Err(PeerError::MaxPeersReached);
        }

        peers.insert(addr, PeerInfo::new_validator(addr, pubkey, stake));
        info!("Added validator peer: {} (stake: {})", addr, stake);
        Ok(())
    }

    /// Remove a peer from the network
    pub async fn remove_peer(&self, addr: &SocketAddr) -> Option<PeerInfo> {
        let mut peers = self.peers.write().await;
        peers.remove(addr)
    }

    /// Mark a peer as connected and active
    pub async fn mark_connected(&self, addr: &SocketAddr) -> Result<(), PeerError> {
        let mut peers = self.peers.write().await;
        let peer = peers.get_mut(addr).ok_or(PeerError::PeerNotFound(*addr))?;

        peer.state = PeerState::Active;
        peer.last_seen = current_timestamp();
        peer.successful_connections += 1;

        // Increase reputation on successful connection
        peer.reputation = (peer.reputation + 1.0).min(100.0);

        debug!("Peer {} connected", addr);
        Ok(())
    }

    /// Mark a peer as disconnected
    pub async fn mark_disconnected(&self, addr: &SocketAddr) -> Result<(), PeerError> {
        let mut peers = self.peers.write().await;
        let peer = peers.get_mut(addr).ok_or(PeerError::PeerNotFound(*addr))?;

        peer.state = PeerState::Disconnected;
        peer.failed_connections += 1;

        // Decrease reputation on disconnection
        peer.reputation = (peer.reputation - 5.0).max(0.0);

        debug!("Peer {} disconnected", addr);
        Ok(())
    }

    /// Update peer latency measurement
    pub async fn update_latency(&self, addr: &SocketAddr, latency_ms: f64) -> Result<(), PeerError> {
        let mut peers = self.peers.write().await;
        let peer = peers.get_mut(addr).ok_or(PeerError::PeerNotFound(*addr))?;

        // Exponential moving average for latency
        if peer.latency_ms == 0.0 {
            peer.latency_ms = latency_ms;
        } else {
            peer.latency_ms = peer.latency_ms * 0.7 + latency_ms * 0.3;
        }
        Ok(())
    }

    /// Record bytes transferred
    pub async fn record_bytes(&self, addr: &SocketAddr, received: u64, sent: u64) {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(addr) {
            peer.bytes_received += received;
            peer.bytes_sent += sent;
        }
    }

    /// Record a block received from a peer
    pub async fn record_block_received(&self, addr: &SocketAddr) {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(addr) {
            peer.blocks_received += 1;
            peer.reputation = (peer.reputation + 2.0).min(100.0);
        }
    }

    /// Ban a peer
    pub async fn ban_peer(&self, addr: SocketAddr) {
        self.remove_peer(&addr).await;
        self.banned.write().await.insert(addr);
        warn!("Banned peer: {}", addr);
    }

    /// Add an address to the discovery queue
    pub async fn discover_peer(&self, addr: SocketAddr) {
        // Don't discover banned or already-known peers
        if self.banned.read().await.contains(&addr) {
            return;
        }
        if self.peers.read().await.contains_key(&addr) {
            return;
        }

        let mut queue = self.discovery_queue.write().await;
        if queue.len() < MAX_DISCOVERY_QUEUE && !queue.contains(&addr) {
            queue.push_back(addr);
        }
    }

    /// Get the next peer to attempt connection from discovery queue
    pub async fn next_discovery(&self) -> Option<SocketAddr> {
        self.discovery_queue.write().await.pop_front()
    }

    /// Get all active peers
    pub async fn get_active_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().await
            .values()
            .filter(|p| p.is_active())
            .cloned()
            .collect()
    }

    /// Get all validator peers
    pub async fn get_validator_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().await
            .values()
            .filter(|p| p.is_validator && p.is_active())
            .cloned()
            .collect()
    }

    /// Select a stake-weighted random peer for block propagation
    ///
    /// Selects peers proportionally to their stake weight, falling back
    /// to reputation-weighted selection for non-validators.
    pub async fn select_stake_weighted_peer(&self) -> Option<PeerInfo> {
        let peers = self.get_active_peers().await;
        if peers.is_empty() {
            return None;
        }

        let total_weight: f64 = peers.iter()
            .map(|p| p.connection_score())
            .sum();

        if total_weight == 0.0 {
            return peers.first().cloned();
        }

        // Weighted random selection
        let mut rng = rand::thread_rng();
        let target = rand::Rng::gen_range(&mut rng, 0.0..total_weight);
        let mut cumulative = 0.0;

        for peer in &peers {
            cumulative += peer.connection_score();
            if cumulative >= target {
                return Some(peer.clone());
            }
        }

        peers.last().cloned()
    }

    /// Get stake-weighted peers for Turbine-style block propagation
    ///
    /// Returns up to `count` peers, ordered by stake weight
    pub async fn get_propagation_peers(&self, count: usize) -> Vec<PeerInfo> {
        let mut peers = self.get_active_peers().await;
        peers.sort_by(|a, b| b.connection_score().partial_cmp(&a.connection_score()).unwrap_or(std::cmp::Ordering::Equal));
        peers.truncate(count);
        peers
    }

    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Get active peer count
    pub async fn active_peer_count(&self) -> usize {
        self.peers.read().await.values().filter(|p| p.is_active()).count()
    }

    /// Get validator peer count
    pub async fn validator_count(&self) -> usize {
        self.peers.read().await.values().filter(|p| p.is_validator && p.is_active()).count()
    }

    /// Prune stale and dead peers
    pub async fn prune_stale_peers(&self) -> usize {
        let now = current_timestamp();
        let mut peers = self.peers.write().await;
        let before = peers.len();

        peers.retain(|_, peer| {
            if peer.state == PeerState::Banned {
                return false;
            }

            let age = now.saturating_sub(peer.last_seen);
            match peer.state {
                PeerState::Active => age < DEAD_THRESHOLD_SECS * 2,
                PeerState::Disconnected => age < DEAD_THRESHOLD_SECS,
                PeerState::Connecting => age < STALE_THRESHOLD_SECS * 2,
                PeerState::Banned => false,
            }
        });

        let removed = before - peers.len();
        if removed > 0 {
            info!("Pruned {} stale peers", removed);
        }
        removed
    }

    /// Set our own stake weight
    pub async fn set_own_stake(&self, stake: u64) {
        *self.own_stake.write().await = stake;
    }

    /// Get our own stake weight
    pub async fn own_stake(&self) -> u64 {
        *self.own_stake.read().await
    }

    /// Get peer manager statistics
    pub async fn stats(&self) -> PeerManagerStats {
        let peers = self.peers.read().await;
        let active = peers.values().filter(|p| p.is_active()).count();
        let validators = peers.values().filter(|p| p.is_validator && p.is_active()).count();
        let avg_latency = peers.values()
            .filter(|p| p.is_active() && p.latency_ms > 0.0)
            .map(|p| p.latency_ms)
            .sum::<f64>()
            / (active as f64).max(1.0);
        let total_bytes_recv: u64 = peers.values().map(|p| p.bytes_received).sum();
        let total_bytes_sent: u64 = peers.values().map(|p| p.bytes_sent).sum();

        PeerManagerStats {
            total_peers: peers.len(),
            active_peers: active,
            validator_peers: validators,
            banned_peers: self.banned.read().await.len(),
            discovery_queue_size: self.discovery_queue.read().await.len(),
            avg_latency_ms: avg_latency,
            total_bytes_received: total_bytes_recv,
            total_bytes_sent: total_bytes_sent,
        }
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Peer manager statistics
#[derive(Debug, Clone)]
pub struct PeerManagerStats {
    pub total_peers: usize,
    pub active_peers: usize,
    pub validator_peers: usize,
    pub banned_peers: usize,
    pub discovery_queue_size: usize,
    pub avg_latency_ms: f64,
    pub total_bytes_received: u64,
    pub total_bytes_sent: u64,
}

/// Peer errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum PeerError {
    #[error("Peer not found: {0}")]
    PeerNotFound(SocketAddr),
    #[error("Peer is banned: {0}")]
    PeerBanned(SocketAddr),
    #[error("Maximum number of peers reached")]
    MaxPeersReached,
    #[error("Connection failed: {0}")]
    ConnectionFailed(SocketAddr),
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(port: u16) -> SocketAddr {
        format!("127.0.0.1:{}", port).parse().unwrap()
    }

    #[tokio::test]
    async fn test_add_and_connect_peer() {
        let pm = PeerManager::new();
        let peer_addr = addr(8000);

        pm.add_peer(peer_addr).await.unwrap();
        assert_eq!(pm.peer_count().await, 1);

        pm.mark_connected(&peer_addr).await.unwrap();
        assert_eq!(pm.active_peer_count().await, 1);
    }

    #[tokio::test]
    async fn test_add_validator_peer() {
        let pm = PeerManager::new();
        let peer_addr = addr(8001);
        let pubkey = [1u8; 32];

        pm.add_validator_peer(peer_addr, pubkey, 50000).await.unwrap();
        assert_eq!(pm.validator_count().await, 1);

        // Need to mark connected first
        pm.mark_connected(&peer_addr).await.unwrap();
        let validators = pm.get_validator_peers().await;
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].stake_weight, 50000);
    }

    #[tokio::test]
    async fn test_ban_peer() {
        let pm = PeerManager::new();
        let peer_addr = addr(8002);

        pm.add_peer(peer_addr).await.unwrap();
        pm.ban_peer(peer_addr).await;

        assert_eq!(pm.peer_count().await, 0);
        assert!(pm.banned.read().await.contains(&peer_addr));

        // Can't add banned peer again
        assert!(pm.add_peer(peer_addr).await.is_err());
    }

    #[tokio::test]
    async fn test_latency_tracking() {
        let pm = PeerManager::new();
        let peer_addr = addr(8003);

        pm.add_peer(peer_addr).await.unwrap();
        pm.mark_connected(&peer_addr).await.unwrap();

        pm.update_latency(&peer_addr, 50.0).await.unwrap();
        pm.update_latency(&peer_addr, 70.0).await.unwrap();

        let peers = pm.get_active_peers().await;
        // EMA: 50*0.7 + 70*0.3 = 35 + 21 = 56
        assert!(peers[0].latency_ms > 40.0 && peers[0].latency_ms < 70.0);
    }

    #[tokio::test]
    async fn test_discovery_queue() {
        let pm = PeerManager::new();
        let peer_addr = addr(8004);

        pm.discover_peer(peer_addr).await;
        assert_eq!(pm.discovery_queue.read().await.len(), 1);

        let next = pm.next_discovery().await;
        assert_eq!(next, Some(peer_addr));
        assert_eq!(pm.discovery_queue.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_peer_health() {
        let pm = PeerManager::new();
        let peer_addr = addr(8005);

        pm.add_peer(peer_addr).await.unwrap();
        pm.mark_connected(&peer_addr).await.unwrap();

        let peers = pm.get_active_peers().await;
        assert!(peers[0].is_healthy());
    }

    #[tokio::test]
    async fn test_stats() {
        let pm = PeerManager::new();

        for i in 0..5 {
            pm.add_peer(addr(9000 + i)).await.unwrap();
            pm.mark_connected(&addr(9000 + i)).await.unwrap();
        }

        let stats = pm.stats().await;
        assert_eq!(stats.total_peers, 5);
        assert_eq!(stats.active_peers, 5);
    }

    #[tokio::test]
    async fn test_max_peers() {
        let pm = PeerManager::new();
        for i in 0..MAX_PEERS {
            pm.add_peer(addr(10000 + i as u16)).await.unwrap();
        }
        // Should fail when max is reached
        let result = pm.add_peer(addr(20000)).await;
        assert!(result.is_err());
    }
}