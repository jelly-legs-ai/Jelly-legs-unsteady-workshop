//! Home Validator Node Implementation for AETHER
//!
//! This module implements the home validator node - personal hardware validators
//! that participate in AETHER's PoS consensus and earn AETH tokens.
//!
//! ## Design Philosophy
//! - **Personal Hardware**: Validators run on everyday computing hardware
//! - **No Cloud Required**: Full node operation without AWS/GCP/Azure
//! - **Energy Efficient**: PoS consensus vs energy-intensive PoW

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Minimum hardware specifications for home validators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpecs {
    /// CPU cores (minimum 8 cores recommended)
    pub cpu_cores: u16,
    /// RAM in GB (minimum 16GB recommended)
    pub ram_gb: u16,
    /// Storage in GB (minimum 500GB SSD recommended)
    pub storage_gb: u32,
    /// Network bandwidth in Mbps
    pub bandwidth_mbps: u32,
    /// Whether GPU is available for ZK proof acceleration
    pub has_gpu: bool,
    /// GPU model (if available)
    pub gpu_model: Option<String>,
}

impl Default for HardwareSpecs {
    fn default() -> Self {
        Self {
            cpu_cores: 8,
            ram_gb: 16,
            storage_gb: 500,
            bandwidth_mbps: 100,
            has_gpu: false,
            gpu_model: None,
        }
    }
}

/// Validator tier classification based on hardware
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorTier {
    /// Basic tier - entry level validation
    Basic,
    /// Standard tier - recommended for most users
    Standard,
    /// High Performance tier - for serious validators
    HighPerformance,
}

impl ValidatorTier {
    /// Returns the minimum stake required for this tier
    pub fn min_stake(&self) -> u64 {
        match self {
            ValidatorTier::Basic => 1_000, // 1K AETH
            ValidatorTier::Standard => 5_000, // 5K AETH
            ValidatorTier::HighPerformance => 10_000, // 10K AETH
        }
    }

    /// Returns the reward weight for this tier
    pub fn reward_weight(&self) -> f64 {
        match self {
            ValidatorTier::Basic => 1.0,
            ValidatorTier::Standard => 1.5,
            ValidatorTier::HighPerformance => 2.5,
        }
    }

    /// Determine tier from hardware specs
    pub fn from_hardware(hw: &HardwareSpecs) -> Self {
        if hw.cpu_cores >= 32 && hw.ram_gb >= 64 && hw.has_gpu {
            ValidatorTier::HighPerformance
        } else if hw.cpu_cores >= 16 && hw.ram_gb >= 32 {
            ValidatorTier::Standard
        } else {
            ValidatorTier::Basic
        }
    }
}

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Validator's public key (32 bytes)
    pub identity: [u8; 32],
    /// Vote account public key (for consensus voting)
    pub vote_account: [u8; 32],
    /// Hardware specifications
    pub hardware: HardwareSpecs,
    /// Assigned validator tier
    pub tier: ValidatorTier,
    /// P2P gossip address
    pub gossip_addr: SocketAddr,
    /// TPU (Transaction Processing Unit) address
    pub tpu_addr: SocketAddr,
    /// TVU (Transceiver Vote Unit) address
    pub tvu_addr: SocketAddr,
    /// Whether this is a founding validator
    pub is_founding: bool,
    /// Founding validator ID (if applicable)
    pub founding_id: Option<u64>,
}

impl ValidatorConfig {
    /// Create a new validator config with default hardware
    pub fn new(identity: [u8; 32], vote_account: [u8; 32]) -> Self {
        let hardware = HardwareSpecs::default();
        let tier = ValidatorTier::from_hardware(&hardware);
        
        Self {
            identity,
            vote_account,
            hardware,
            tier,
            gossip_addr: "0.0.0.0:8801".parse().unwrap(),
            tpu_addr: "0.0.0.0:8802".parse().unwrap(),
            tvu_addr: "0.0.0.0:8803".parse().unwrap(),
            is_founding: false,
            founding_id: None,
        }
    }

    /// Create with custom hardware specs
    pub fn with_hardware(mut self, hardware: HardwareSpecs) -> Self {
        self.hardware = hardware;
        self.tier = ValidatorTier::from_hardware(&self.hardware);
        self
    }

    /// Check if hardware meets minimum requirements
    pub fn meets_minimum_requirements(&self) -> bool {
        self.hardware.cpu_cores >= 4
            && self.hardware.ram_gb >= 8
            && self.hardware.storage_gb >= 256
            && self.hardware.bandwidth_mbps >= 50
    }
}

/// Validator state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorState {
    /// Current epoch
    pub epoch: u64,
    /// Total stake amount
    pub stake: u64,
    /// Effective stake (adjusted by tier weight)
    pub effective_stake: f64,
    /// Last vote slot
    pub last_vote_slot: u64,
    /// Total votes cast
    pub total_votes: u64,
    /// Whether validator is active (producing blocks)
    pub is_active: bool,
    /// Credits earned (for rewards)
    pub credits: u64,
    /// Consecutive blocks produced (uptime metric)
    pub consecutive_blocks: u64,
    /// Validator uptime percentage (0.0 - 100.0)
    pub uptime_percent: f64,
}

impl Default for ValidatorState {
    fn default() -> Self {
        Self {
            epoch: 0,
            stake: 0,
            effective_stake: 0.0,
            last_vote_slot: 0,
            total_votes: 0,
            is_active: false,
            credits: 0,
            consecutive_blocks: 0,
            uptime_percent: 0.0,
        }
    }
}

/// P2P peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer public key
    pub identity: [u8; 32],
    /// Peer gossip address
    pub gossip_addr: SocketAddr,
    /// Whether peer is a validator
    pub is_validator: bool,
    /// Peer stake amount
    pub stake: u64,
    /// Connection state
    pub connected: bool,
    /// Last heartbeat timestamp
    pub last_heartbeat: u64,
}

/// Home validator node
pub struct ValidatorNode {
    /// Validator configuration
    config: ValidatorConfig,
    /// Current state
    state: ValidatorState,
    /// Connected peers
    peers: Arc<RwLock<Vec<PeerInfo>>>,
    /// Whether the node is running
    running: Arc<RwLock<bool>>,
}

impl ValidatorNode {
    /// Create a new validator node
    pub fn new(config: ValidatorConfig) -> Self {
        let effective_stake = config.tier.reward_weight() * config.stake as f64;
        
        Self {
            config,
            state: ValidatorState {
                effective_stake,
                ..Default::default()
            },
            peers: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Get validator configuration
    pub fn config(&self) -> &ValidatorConfig {
        &self.config
    }

    /// Get current state
    pub fn state(&self) -> &ValidatorState {
        &self.state
    }

    /// Update stake amount
    pub fn update_stake(&mut self, new_stake: u64) {
        self.state.stake = new_stake;
        self.state.effective_stake = self.config.tier.reward_weight() * new_stake as f64;
    }

    /// Add a peer to the peer list
    pub async fn add_peer(&self, peer: PeerInfo) {
        let mut peers = self.peers.write().await;
        if !peers.iter().any(|p| p.identity == peer.identity) {
            peers.push(peer);
        }
    }

    /// Remove a peer from the peer list
    pub async fn remove_peer(&self, identity: &[u8; 32]) {
        let mut peers = self.peers.write().await;
        peers.retain(|p| &p.identity != identity);
    }

    /// Get all connected peers
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().await.clone()
    }

    /// Check if node is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Start the validator node
    pub async fn start(&self) -> Result<(), ValidatorError> {
        if !self.config.meets_minimum_requirements() {
            return Err(ValidatorError::InsufficientHardware);
        }

        *self.running.write().await = true;
        self.state.is_active = true;
        
        // Initialize P2P networking
        self.initialize_p2p().await?;
        
        // Start consensus participation
        self.start_consensus().await?;
        
        Ok(())
    }

    /// Stop the validator node
    pub async fn stop(&self) {
        *self.running.write().await = false;
        self.state.is_active = false;
    }

    /// Initialize P2P networking
    async fn initialize_p2p(&self) -> Result<(), ValidatorError> {
        // In a full implementation, this would:
        // 1. Start TCP listener on gossip_addr
        // 2. Initialize Kademlia DHT for peer discovery
        // 3. Connect to seed nodes / bootstrap peers
        // 4. Start gossip protocol
        // 5. Exchange peer lists with neighbors
        
        Ok(())
    }

    /// Start consensus participation
    async fn start_consensus(&self) -> Result<(), ValidatorError> {
        // In a full implementation, this would:
        // 1. Connect to gossip network
        // 2. Download latest block height from peers
        // 3. Start Tower BFT voting
        // 4. Participate in leader election
        // 5. Produce blocks when elected as leader
        
        Ok(())
    }

    /// Vote on a block (part of consensus)
    pub async fn vote(&mut self, slot: u64, hash: [u8; 32]) -> Result<(), ValidatorError> {
        self.state.last_vote_slot = slot;
        self.state.total_votes += 1;
        Ok(())
    }

    /// Record a block produced by this validator
    pub async fn record_block(&mut self) {
        self.state.consecutive_blocks += 1;
        self.state.credits += 1;
    }

    /// Update uptime calculation
    pub fn update_uptime(&mut self, total_slots: u64) {
        if total_slots > 0 {
            self.state.uptime_percent = (self.state.consecutive_blocks as f64 / total_slots as f64) * 100.0;
        }
    }
}

/// Validator-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ValidatorError {
    #[error("Insufficient hardware for validation")]
    InsufficientHardware,
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Consensus error: {0}")]
    Consensus(String),
    
    #[error("Staking error: {0}")]
    Staking(String),
    
    #[error("Node is not running")]
    NotRunning,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// P2P message types for validator communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    /// Gossip message for peer discovery
    Gossip {
        sender: [u8; 32],
        epoch: u64,
        block_hash: Option<[u8; 32]>,
    },
    /// Vote message for consensus
    Vote {
        validator: [u8; 32],
        slot: u64,
        hash: [u8; 32],
    },
    /// Block announcement
    BlockAnnounce {
        producer: [u8; 32],
        slot: u64,
        hash: [u8; 32],
    },
    /// Request for peer list
    PeerRequest {
        requester: [u8; 32],
    },
    /// Peer list response
    PeerResponse {
        peers: Vec<PeerInfo>,
    },
}

impl P2PMessage {
    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Deserialize message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}

/// Start a home validator node with the given configuration
pub async fn start_validator(config: ValidatorConfig) -> Result<ValidatorNode, ValidatorError> {
    let validator = ValidatorNode::new(config);
    validator.start().await?;
    Ok(validator)
}

/// Create a basic tier validator with minimum requirements
pub fn create_basic_validator(identity: [u8; 32], vote_account: [u8; 32]) -> ValidatorConfig {
    ValidatorConfig::new(identity, vote_account)
}

/// Create a standard tier validator
pub fn create_standard_validator(
    identity: [u8; 32],
    vote_account: [u8; 32],
    cpu_cores: u16,
    ram_gb: u16,
) -> ValidatorConfig {
    let hardware = HardwareSpecs {
        cpu_cores,
        ram_gb,
        storage_gb: 1000,
        bandwidth_mbps: 500,
        has_gpu: false,
        gpu_model: None,
    };
    ValidatorConfig::new(identity, vote_account).with_hardware(hardware)
}

/// Create a high-performance validator with GPU
pub fn create_high_perf_validator(
    identity: [u8; 32],
    vote_account: [u8; 32],
    gpu_model: String,
) -> ValidatorConfig {
    let hardware = HardwareSpecs {
        cpu_cores: 32,
        ram_gb: 64,
        storage_gb: 2000,
        bandwidth_mbps: 1000,
        has_gpu: true,
        gpu_model: Some(gpu_model),
    };
    ValidatorConfig::new(identity, vote_account).with_hardware(hardware)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_specs_default() {
        let hw = HardwareSpecs::default();
        assert_eq!(hw.cpu_cores, 8);
        assert_eq!(hw.ram_gb, 16);
        assert_eq!(hw.storage_gb, 500);
        assert!(!hw.has_gpu);
    }

    #[test]
    fn test_validator_tier_from_hardware() {
        // Basic tier
        let basic = HardwareSpecs {
            cpu_cores: 4,
            ram_gb: 8,
            storage_gb: 256,
            bandwidth_mbps: 50,
            has_gpu: false,
            gpu_model: None,
        };
        assert_eq!(ValidatorTier::from_hardware(&basic), ValidatorTier::Basic);

        // Standard tier
        let standard = HardwareSpecs {
            cpu_cores: 16,
            ram_gb: 32,
            storage_gb: 1000,
            bandwidth_mbps: 500,
            has_gpu: false,
            gpu_model: None,
        };
        assert_eq!(ValidatorTier::from_hardware(&standard), ValidatorTier::Standard);

        // High performance tier
        let high_perf = HardwareSpecs {
            cpu_cores: 32,
            ram_gb: 64,
            storage_gb: 2000,
            bandwidth_mbps: 1000,
            has_gpu: true,
            gpu_model: Some("RTX 4090".to_string()),
        };
        assert_eq!(ValidatorTier::from_hardware(&high_perf), ValidatorTier::HighPerformance);
    }

    #[test]
    fn test_validator_tier_stake_requirements() {
        assert_eq!(ValidatorTier::Basic.min_stake(), 1_000);
        assert_eq!(ValidatorTier::Standard.min_stake(), 5_000);
        assert_eq!(ValidatorTier::HighPerformance.min_stake(), 10_000);
    }

    #[test]
    fn test_validator_tier_reward_weights() {
        assert_eq!(ValidatorTier::Basic.reward_weight(), 1.0);
        assert_eq!(ValidatorTier::Standard.reward_weight(), 1.5);
        assert_eq!(ValidatorTier::HighPerformance.reward_weight(), 2.5);
    }

    #[test]
    fn test_validator_config_meets_minimum() {
        let identity = [0u8; 32];
        let vote_account = [1u8; 32];
        
        let config = ValidatorConfig::new(identity, vote_account);
        assert!(config.meets_minimum_requirements());
        
        // Below minimum
        let weak_hw = HardwareSpecs {
            cpu_cores: 2,
            ram_gb: 4,
            storage_gb: 128,
            bandwidth_mbps: 10,
            has_gpu: false,
            gpu_model: None,
        };
        let weak_config = ValidatorConfig::new(identity, vote_account).with_hardware(weak_hw);
        assert!(!weak_config.meets_minimum_requirements());
    }

    #[test]
    fn test_validator_node_creation() {
        let identity = [0u8; 32];
        let vote_account = [1u8; 32];
        let config = ValidatorConfig::new(identity, vote_account);
        
        let validator = ValidatorNode::new(config.clone());
        
        assert_eq!(validator.state().stake, 0);
        assert!(!validator.state().is_active);
    }

    #[test]
    fn test_update_stake() {
        let identity = [0u8; 32];
        let vote_account = [1u8; 32];
        let config = ValidatorConfig::new(identity, vote_account);
        
        let mut validator = ValidatorNode::new(config);
        
        // Update stake for standard tier (1.5x weight)
        validator.update_stake(5000);
        
        assert_eq!(validator.state().stake, 5000);
        assert_eq!(validator.state().effective_stake, 5000.0 * 1.5);
    }

    #[test]
    fn test_p2p_message_serialization() {
        let msg = P2PMessage::Gossip {
            sender: [5u8; 32],
            epoch: 100,
            block_hash: Some([10u8; 32]),
        };
        
        let bytes = msg.to_bytes();
        let decoded = P2PMessage::from_bytes(&bytes).unwrap();
        
        match decoded {
            P2PMessage::Gossip { sender, epoch, .. } => {
                assert_eq!(sender, [5u8; 32]);
                assert_eq!(epoch, 100);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
