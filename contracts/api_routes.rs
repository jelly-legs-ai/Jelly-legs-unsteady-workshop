// API Routes - AeTHer Chain
// RESTful API endpoint definitions for chain data access

use serde::{Deserialize, Serialize};

/// API version
pub const API_VERSION: &str = "v1";

/// Base API routes
pub mod routes {
    pub const ACCOUNTS: &str = "/accounts";
    pub const VALIDATORS: &str = "/validators";
    pub const STAKING: &str = "/staking";
    pub const MINING: &str = "/mining";
    pub const AGENTS: &str = "/agents";
    pub const GOVERNANCE: &str = "/governance";
    pub const BRIDGE: &str = "/bridge";
    pub const TRANSACTIONS: &str = "/transactions";
    pub const EPOCHS: &str = "/epochs";
    pub const DEVICES: &str = "/devices";
    pub const TOKENS: &str = "/tokens";
    pub const TREASURY: &str = "/treasury";
    pub const CONTRACTS: &str = "/contracts";
    pub const NETWORK: &str = "/network";
    pub const HEALTH: &str = "/health";
}

/// Account endpoints
pub mod accounts {
    pub const LIST: &str = "/accounts";
    pub const GET: &str = "/accounts/{address}";
    pub const BALANCE: &str = "/accounts/{address}/balance";
    pub const TRANSACTIONS: &str = "/accounts/{address}/transactions";
    pub const STAKES: &str = "/accounts/{address}/stakes";
    pub const DELEGATIONS: &str = "/accounts/{address}/delegations";
    pub const REWARDS: &str = "/accounts/{address}/rewards";
}

/// Validator endpoints
pub mod validators {
    pub const LIST: &str = "/validators";
    pub const GET: &str = "/validators/{address}";
    pub const ACTIVE: &str = "/validators/active";
    pub const INACTIVE: &str = "/validators/inactive";
    pub const SLASHED: &str = "/validators/slashed";
    pub const DELEGATIONS: &str = "/validators/{address}/delegations";
    pub const REWARDS: &str = "/validators/{address}/rewards";
    pub const PERFORMANCE: &str = "/validators/{address}/performance";
    pub const COMMISSION: &str = "/validators/{address}/commission";
}

/// Staking endpoints
pub mod staking {
    pub const POSITIONS: &str = "/staking/positions";
    pub const GET: &str = "/staking/positions/{id}";
    pub const CREATE: &str = "/staking/positions";
    pub const UNSTAKE: &str = "/staking/positions/{id}/unstake";
    pub const CLAIM: &str = "/staking/positions/{id}/claim";
    pub const DELEGATE: &str = "/staking/delegate";
    pub const UNDELEGATE: &str = "/staking/undelegate";
    pub const POOLS: &str = "/staking/pools";
    pub const POOL_STATS: &str = "/staking/pools/{pool_id}/stats";
    pub const REWARDS_CALC: &str = "/staking/rewards/calculate";
    pub const APY: &str = "/staking/apy";
}

/// Mining endpoints
pub mod mining {
    pub const REWARDS: &str = "/mining/rewards";
    pub const CLAIM: &str = "/mining/rewards/claim";
    pub const DEVICE_REGISTER: &str = "/mining/devices/register";
    pub const DEVICE_STATUS: &str = "/mining/devices/{device_id}/status";
    pub const DEVICE_LIST: &str = "/mining/devices";
    pub const WORK_SUBMIT: &str = "/mining/work/submit";
    pub const REWARDS_CALC: &str = "/mining/rewards/calculate";
    pub const LEADERBOARD: &str = "/mining/leaderboard";
    pub const STATS: &str = "/mining/stats";
}

/// Agent endpoints
pub mod agents {
    pub const LIST: &str = "/agents";
    pub const GET: &str = "/agents/{agent_id}";
    pub const REGISTER: &str = "/agents/register";
    pub const KYC_STATUS: &str = "/agents/{agent_id}/kyc";
    pub const KYC_VERIFY: &str = "/agents/{agent_id}/kyc/verify";
    pub const REPUTATION: &str = "/agents/{agent_id}/reputation";
    pub const TASKS: &str = "/agents/{agent_id}/tasks";
    pub const CAPABILITIES: &str = "/agents/{agent_id}/capabilities";
    pub const SLASH: &str = "/agents/{agent_id}/slash";
    pub const MARKETPLACE: &str = "/agents/marketplace";
    pub const SEARCH: &str = "/agents/search";
}

/// Governance endpoints
pub mod governance {
    pub const PROPOSALS: &str = "/governance/proposals";
    pub const GET: &str = "/governance/proposals/{id}";
    pub const CREATE: &str = "/governance/proposals";
    pub const VOTE: &str = "/governance/proposals/{id}/vote";
    pub const ACTIVE: &str = "/governance/proposals/active";
    pub const PASSED: &str = "/governance/proposals/passed";
    pub const REJECTED: &str = "/governance/proposals/rejected";
    pub const EXECUTE: &str = "/governance/proposals/{id}/execute";
    pub const DELEGATE_VOTE: &str = "/governance/delegate";
    pub const VOTING_POWER: &str = "/governance/voting-power/{address}";
}

/// Bridge endpoints
pub mod bridge {
    pub const TRANSACTIONS: &str = "/bridge/transactions";
    pub const GET: &str = "/bridge/transactions/{tx_hash}";
    pub const INITIATE: &str = "/bridge/initiate";
    pub const STATUS: &str = "/bridge/transactions/{tx_hash}/status";
    pub const CHAINS: &str = "/bridge/chains";
    pub const RATES: &str = "/bridge/rates";
    pub const HISTORY: &str = "/bridge/history/{address}";
}

/// Transaction endpoints
pub mod transactions {
    pub const LIST: &str = "/transactions";
    pub const GET: &str = "/transactions/{hash}";
    pub const PENDING: &str = "/transactions/pending";
    pub const CONFIRMED: &str = "/transactions/confirmed";
    pub const FAILED: &str = "/transactions/failed";
    pub const BY_BLOCK: &str = "/transactions/block/{block_number}";
    pub const BY_EPOCH: &str = "/transactions/epoch/{epoch}";
    pub const BY_ADDRESS: &str = "/transactions/address/{address}";
}

/// Epoch endpoints
pub mod epochs {
    pub const CURRENT: &str = "/epochs/current";
    pub const GET: &str = "/epochs/{epoch}";
    pub const LIST: &str = "/epochs";
    pub const STATS: &str = "/epochs/{epoch}/stats";
    pub const VALIDATORS: &str = "/epochs/{epoch}/validators";
    pub const REWARDS: &str = "/epochs/{epoch}/rewards";
    pub const TRANSACTIONS: &str = "/epochs/{epoch}/transactions";
}

/// Device endpoints
pub mod devices {
    pub const LIST: &str = "/devices";
    pub const GET: &str = "/devices/{device_id}";
    pub const REGISTER: &str = "/devices/register";
    pub const STATUS: &str = "/devices/{device_id}/status";
    pub const WORK_HISTORY: &str = "/devices/{device_id}/work";
    pub const REWARDS: &str = "/devices/{device_id}/rewards";
    pub const BY_OWNER: &str = "/devices/owner/{address}";
    pub const BY_TIER: &str = "/devices/tier/{tier}";
}

/// Token endpoints
pub mod tokens {
    pub const AETH: &str = "/tokens/aeth";
    pub const FLUX: &str = "/tokens/flux";
    pub const ATH: &str = "/tokens/ath";
    pub const SUPPLY: &str = "/tokens/{symbol}/supply";
    pub const PRICE: &str = "/tokens/{symbol}/price";
    pub const HOLDERS: &str = "/tokens/{symbol}/holders";
    pub const TRANSFERS: &str = "/tokens/{symbol}/transfers";
    pub const BURNS: &str = "/tokens/{symbol}/burns";
    pub const MINTS: &str = "/tokens/{symbol}/mints";
}

/// Treasury endpoints
pub mod treasury {
    pub const BALANCE: &str = "/treasury/balance";
    pub const ALLOCATIONS: &str = "/treasury/allocations";
    pub const PROPOSALS: &str = "/treasury/proposals";
    pub const SPEND_HISTORY: &str = "/treasury/spend-history";
    pub const MULTISIG: &str = "/treasury/multisig";
}

/// Contract endpoints
pub mod contracts {
    pub const LIST: &str = "/contracts";
    pub const GET: &str = "/contracts/{address}";
    pub const VERIFY: &str = "/contracts/{address}/verify";
    pub const SOURCE: &str = "/contracts/{address}/source";
    pub const EVENTS: &str = "/contracts/{address}/events";
    pub const READ: &str = "/contracts/{address}/read";
    pub const SIMULATE: &str = "/contracts/{address}/simulate";
}

/// Network endpoints
pub mod network {
    pub const STATS: &str = "/network/stats";
    pub const HEALTH: &str = "/network/health";
    pub const TPS: &str = "/network/tps";
    pub const FINALITY: &str = "/network/finality";
    pub const PEERS: &str = "/network/peers";
    pub const SYNC: &str = "/network/sync";
    pub const CONFIG: &str = "/network/config";
    pub const CHAIN_ID: &str = "/network/chain-id";
}

/// Health check endpoints
pub mod health {
    pub const STATUS: &str = "/health";
    pub const DB: &str = "/health/db";
    pub const RPC: &str = "/health/rpc";
    pub const VALIDATORS: &str = "/health/validators";
    pub const BRIDGE: &str = "/health/bridge";
    pub const AGENTS: &str = "/health/agents";
}

/// API Response wrappers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: i64,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub limit: u32,
    pub total: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Query parameters for list endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListQueryParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub filter: Option<serde_json::Value>,
    pub search: Option<String>,
}

/// Rate limiting config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_limit: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_limit: 10,
        }
    }
}

/// API endpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMeta {
    pub path: String,
    pub method: String,
    pub description: String,
    pub auth_required: bool,
    pub rate_limit: RateLimitConfig,
    pub cache_ttl_secs: Option<u32>,
    pub tags: Vec<String>,
}

/// Build all endpoint metadata
pub fn get_all_endpoints() -> Vec<EndpointMeta> {
    vec![
        // Accounts
        EndpointMeta {
            path: accounts::GET.to_string(),
            method: "GET".to_string(),
            description: "Get account details by address".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["accounts".to_string()],
        },
        EndpointMeta {
            path: accounts::BALANCE.to_string(),
            method: "GET".to_string(),
            description: "Get account token balances".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(30),
            tags: vec!["accounts".to_string(), "balances".to_string()],
        },
        // Validators
        EndpointMeta {
            path: validators::LIST.to_string(),
            method: "GET".to_string(),
            description: "List all validators".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["validators".to_string()],
        },
        EndpointMeta {
            path: validators::ACTIVE.to_string(),
            method: "GET".to_string(),
            description: "List active validators".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(30),
            tags: vec!["validators".to_string()],
        },
        // Staking
        EndpointMeta {
            path: staking::POOLS.to_string(),
            method: "GET".to_string(),
            description: "List staking pools".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["staking".to_string()],
        },
        EndpointMeta {
            path: staking::REWARDS_CALC.to_string(),
            method: "POST".to_string(),
            description: "Calculate staking rewards".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig { requests_per_minute: 10, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["staking".to_string(), "rewards".to_string()],
        },
        // Mining
        EndpointMeta {
            path: mining::REWARDS.to_string(),
            method: "GET".to_string(),
            description: "Get mining rewards".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(30),
            tags: vec!["mining".to_string()],
        },
        EndpointMeta {
            path: mining::CLAIM.to_string(),
            method: "POST".to_string(),
            description: "Claim mining rewards".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig { requests_per_minute: 5, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["mining".to_string(), "rewards".to_string()],
        },
        // Agents
        EndpointMeta {
            path: agents::LIST.to_string(),
            method: "GET".to_string(),
            description: "List registered agents".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["agents".to_string()],
        },
        EndpointMeta {
            path: agents::REGISTER.to_string(),
            method: "POST".to_string(),
            description: "Register new agent".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig { requests_per_minute: 5, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["agents".to_string()],
        },
        // Governance
        EndpointMeta {
            path: governance::PROPOSALS.to_string(),
            method: "GET".to_string(),
            description: "List governance proposals".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(30),
            tags: vec!["governance".to_string()],
        },
        EndpointMeta {
            path: governance::VOTE.to_string(),
            method: "POST".to_string(),
            description: "Submit vote on proposal".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig { requests_per_minute: 10, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["governance".to_string(), "voting".to_string()],
        },
        // Bridge
        EndpointMeta {
            path: bridge::INITIATE.to_string(),
            method: "POST".to_string(),
            description: "Initiate cross-chain bridge".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig { requests_per_minute: 5, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["bridge".to_string()],
        },
        // Tokens
        EndpointMeta {
            path: tokens::AETH.to_string(),
            method: "GET".to_string(),
            description: "Get AETH token info".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["tokens".to_string()],
        },
        EndpointMeta {
            path: tokens::FLUX.to_string(),
            method: "GET".to_string(),
            description: "Get FLUX token info".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["tokens".to_string()],
        },
        // Network
        EndpointMeta {
            path: network::STATS.to_string(),
            method: "GET".to_string(),
            description: "Get network statistics".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(10),
            tags: vec!["network".to_string()],
        },
        EndpointMeta {
            path: health::STATUS.to_string(),
            method: "GET".to_string(),
            description: "Health check endpoint".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig { requests_per_minute: 300, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["health".to_string()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routes_defined() {
        assert!(!routes::ACCOUNTS.is_empty());
        assert!(!routes::VALIDATORS.is_empty());
        assert!(!routes::STAKING.is_empty());
    }

    #[test]
    fn test_endpoint_meta() {
        let endpoints = get_all_endpoints();
        assert!(endpoints.len() > 10);
        assert!(endpoints.iter().any(|e| e.tags.contains(&"accounts".to_string())));
        assert!(endpoints.iter().any(|e| e.tags.contains(&"validators".to_string())));
    }

    #[test]
    fn test_rate_limit_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 60);
        assert_eq!(config.requests_per_hour, 1000);
    }
}
