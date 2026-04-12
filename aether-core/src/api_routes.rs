//! API Routes - AeTHer Chain Backend
//!
//! User and agent management API endpoints.
//! This module provides data structures and routing for the
//! validator RPC API and user dashboard.

use serde::{Deserialize, Serialize};

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Account address
    pub address: String,
    /// Username
    pub username: String,
    /// Email (optional)
    pub email: Option<String>,
    /// Account creation timestamp
    pub created_at: u64,
    /// Last active timestamp
    pub last_active: u64,
    /// Reputation score (0-100)
    pub reputation_score: f64,
    /// KYC verification status
    pub kyc_verified: bool,
    /// Total agents registered
    pub total_agents: u64,
    /// Total tokens staked
    pub total_staked: u64,
    /// Total tokens mined
    pub total_mined: u64,
}

/// Agent registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistrationRequest {
    /// Agent name
    pub name: String,
    /// Agent description
    pub description: String,
    /// Agent capabilities
    pub capabilities: Vec<String>,
    /// Avatar URL or emoji
    pub avatar: Option<String>,
    /// Owner address
    pub owner_address: String,
    /// Stake amount
    pub stake_amount: u64,
}

/// Agent registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistrationResponse {
    /// Whether registration was successful
    pub success: bool,
    /// Agent ID
    pub agent_id: String,
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Status message
    pub message: String,
}

/// Validator status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStatus {
    /// Validator pubkey
    pub pubkey: String,
    /// Whether validator is active
    pub active: bool,
    /// Current slot
    pub slot: u64,
    /// Last confirmed slot
    pub last_confirmed_slot: u64,
    /// Stake amount
    pub stake: u64,
    /// Commission rate (0-10000)
    pub commission: u16,
    /// Number of peer connections
    pub peer_count: usize,
    /// Whether validator is healthy
    pub healthy: bool,
    /// Error message if unhealthy
    pub error: Option<String>,
}

/// Network status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Current slot
    pub slot: u64,
    /// Slot height
    pub block_height: u64,
    /// Total validators
    pub validator_count: usize,
    /// Active validators
    pub active_validator_count: usize,
    /// Total staked amount
    pub total_stake: u64,
    /// Network congestion level (0.0-1.0)
    pub congestion: f64,
    /// Average latency in ms
    pub avg_latency_ms: f64,
}

/// RPC API method enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcMethod {
    /// Get validator status
    GetValidatorStatus,
    /// Get network status
    GetNetworkStatus,
    /// Get block by slot
    GetBlock { slot: u64 },
    /// Get block by hash
    GetBlockByHash { hash: String },
    /// Get account info
    GetAccountInfo { address: String },
    /// Get transaction by signature
    GetTransaction { signature: String },
    /// Submit transaction
    SubmitTransaction { data: String },
    /// Get slot info
    GetSlotInfo,
    /// Get epoch info
    GetEpochInfo,
    /// Get supply info
    GetSupply,
}

/// RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: u64,
    /// Method
    pub method: RpcMethod,
}

/// RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse<T: Serialize> {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: u64,
    /// Result (if successful)
    pub result: Option<T>,
    /// Error (if failed)
    pub error: Option<RpcError>,
}

/// RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
}

/// User dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDashboard {
    /// User profile
    pub profile: UserProfile,
    /// Registered agents
    pub agents: Vec<AgentSummary>,
    /// Staking positions
    pub staking_positions: Vec<StakingPositionInfo>,
    /// Mining rewards
    pub mining_rewards: u64,
    /// Total earnings
    pub total_earnings: u64,
    /// Pending actions
    pub pending_actions: Vec<PendingAction>,
}

/// Agent summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    /// Agent ID
    pub id: String,
    /// Agent name
    pub name: String,
    /// Agent status
    pub status: String,
    /// 24h earnings
    pub earnings_24h: u64,
    /// Uptime percentage
    pub uptime_percent: f64,
    /// Last active timestamp
    pub last_active: u64,
}

/// Staking position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPositionInfo {
    /// Pool name
    pub pool_name: String,
    /// Amount staked
    pub amount_staked: u64,
    /// Rewards earned
    pub rewards_earned: u64,
    /// APY
    pub apy: f64,
    /// Lock end epoch
    pub lock_end_epoch: Option<u64>,
}

/// Pending action for user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    /// Action type
    pub action_type: String,
    /// Description
    pub description: String,
    /// Priority
    pub priority: String,
    /// Created timestamp
    pub created_at: u64,
    /// Deadline timestamp
    pub deadline: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_serialization() {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: RpcMethod::GetSlotInfo,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("GetSlotInfo"));
    }

    #[test]
    fn test_validator_status() {
        let status = ValidatorStatus {
            pubkey: "0x1234".to_string(),
            active: true,
            slot: 100,
            last_confirmed_slot: 95,
            stake: 500_000,
            commission: 500,
            peer_count: 10,
            healthy: true,
            error: None,
        };

        assert!(status.healthy);
        assert_eq!(status.slot, 100);
    }

    #[test]
    fn test_network_status() {
        let status = NetworkStatus {
            slot: 1000,
            block_height: 1000,
            validator_count: 5,
            active_validator_count: 4,
            total_stake: 10_000_000,
            congestion: 0.3,
            avg_latency_ms: 45.0,
        };

        assert_eq!(status.slot, 1000);
        assert!(status.congestion < 1.0);
    }
}