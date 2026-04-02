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
    pub const USERS: &str = "/users";
    pub const KYC: &str = "/kyc";
    pub const REWARDS: &str = "/rewards";
    pub const ANALYTICS: &str = "/analytics";
    pub const EXPORT: &str = "/export";
}

// ============================================================================
// SPRINT 4 ENHANCEMENT: User & Agent Management API Routes
// ============================================================================

/// User management endpoints
pub mod users {
    pub const LIST: &str = "/users";
    pub const GET: &str = "/users/{user_id}";
    pub const CREATE: &str = "/users";
    pub const UPDATE: &str = "/users/{user_id}";
    pub const DELETE: &str = "/users/{user_id}";
    pub const LINKED_ACCOUNTS: &str = "/users/{user_id}/accounts";
    pub const LINK_AGENT: &str = "/users/{user_id}/agents/link";
    pub const UNLINK_AGENT: &str = "/users/{user_id}/agents/unlink";
    pub const PREFERENCES: &str = "/users/{user_id}/preferences";
    pub const NOTIFICATIONS: &str = "/users/{user_id}/notifications";
    pub const ACTIVITY_LOG: &str = "/users/{user_id}/activity";
    pub const EXPORT_DATA: &str = "/users/{user_id}/export";
    pub const VERIFY: &str = "/users/{user_id}/verify";
    pub const SUSPEND: &str = "/users/{user_id}/suspend";
    pub const REINSTATE: &str = "/users/{user_id}/reinstate";
    pub const SEARCH: &str = "/users/search";
    pub const BATCH_CREATE: &str = "/users/batch";
    pub const BATCH_UPDATE: &str = "/users/batch/update";
    pub const BULK_DELETE: &str = "/users/batch/delete";
    pub const IMPORT: &str = "/users/import";
    pub const ROLE_ASSIGN: &str = "/users/{user_id}/role";
    pub const PERMISSIONS: &str = "/users/{user_id}/permissions";
    pub const SESSIONS: &str = "/users/{user_id}/sessions";
    pub const DEVICES: &str = "/users/{user_id}/devices";
    pub const SECURITY: &str = "/users/{user_id}/security";
    pub const TWO_FA: &str = "/users/{user_id}/2fa";
    pub const API_KEYS: &str = "/users/{user_id}/api-keys";
    pub const RATE_LIMIT: &str = "/users/{user_id}/rate-limit";
    pub const QUOTA: &str = "/users/{user_id}/quota";
    pub const SUBSCRIPTION: &str = "/users/{user_id}/subscription";
    pub const BILLING: &str = "/users/{user_id}/billing";
    pub const INVOICES: &str = "/users/{user_id}/invoices";
    pub const PAYMENT_METHODS: &str = "/users/{user_id}/payment-methods";
}

/// KYC verification endpoints
pub mod kyc {
    pub const SUBMIT: &str = "/kyc/submit";
    pub const STATUS: &str = "/kyc/status";
    pub const DOCUMENTS: &str = "/kyc/documents";
    pub const UPLOAD: &str = "/kyc/documents/upload";
    pub const VERIFY: &str = "/kyc/verify";
    pub const REJECT: &str = "/kyc/reject";
    pub const APPEAL: &str = "/kyc/appeal";
    pub const TIER_CHECK: &str = "/kyc/tier";
    pub const EXPIRY: &str = "/kyc/expiry";
    pub const RENEW: &str = "/kyc/renew";
}

/// Rewards management endpoints
pub mod rewards {
    pub const CLAIM: &str = "/rewards/claim";
    pub const HISTORY: &str = "/rewards/history";
    pub const PENDING: &str = "/rewards/pending";
    pub const PROJECTED: &str = "/rewards/projected";
    pub const COMPOUND: &str = "/rewards/compound";
    pub const WITHDRAW: &str = "/rewards/withdraw";
    pub const REDELEGATE: &str = "/rewards/redelegate";
    pub const SCHEDULE: &str = "/rewards/schedule";
    pub const TAX_REPORT: &str = "/rewards/tax-report";
    pub const MINING_REWARDS: &str = "/rewards/mining";
    pub const STAKING_REWARDS: &str = "/rewards/staking";
    pub const VALIDATOR_REWARDS: &str = "/rewards/validator";
    pub const OPTIMIZE: &str = "/rewards/optimize";
    pub const BREAKDOWN: &str = "/rewards/breakdown";
    pub const SIMULATE: &str = "/rewards/simulate";
    pub const AIRDROP: &str = "/rewards/airdrop";
    pub const REFERRAL: &str = "/rewards/referral";
}

/// Analytics & reporting endpoints
pub mod analytics {
    pub const DASHBOARD: &str = "/analytics/dashboard";
    pub const PORTFOLIO: &str = "/analytics/portfolio";
    pub const PERFORMANCE: &str = "/analytics/performance";
    pub const STAKING_ANALYTICS: &str = "/analytics/staking";
    pub const MINING_ANALYTICS: &str = "/analytics/mining";
    pub const NETWORK_STATS: &str = "/analytics/network";
    pub const TREND: &str = "/analytics/trend";
    pub const COMPARE: &str = "/analytics/compare";
    pub const EXPORT_CSV: &str = "/analytics/export/csv";
    pub const EXPORT_JSON: &str = "/analytics/export/json";
    pub const REALTIME: &str = "/analytics/realtime";
    pub const HISTORICAL: &str = "/analytics/historical";
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
    // User/Agent Management
    pub const LINK_USER: &str = "/agents/{agent_id}/link-user";
    pub const UNLINK_USER: &str = "/agents/{agent_id}/unlink-user";
    pub const OWNER: &str = "/agents/{agent_id}/owner";
    pub const CONFIG: &str = "/agents/{agent_id}/config";
    pub const UPDATE_CONFIG: &str = "/agents/{agent_id}/config/update";
    // Agent Lifecycle
    pub const PAUSE: &str = "/agents/{agent_id}/pause";
    pub const RESUME: &str = "/agents/{agent_id}/resume";
    pub const RESTART: &str = "/agents/{agent_id}/restart";
    pub const DELETE: &str = "/agents/{agent_id}/delete";
    pub const SCALE: &str = "/agents/{agent_id}/scale";
    pub const DEPLOY: &str = "/agents/deploy";
    pub const BATCH_CREATE: &str = "/agents/batch";
    // Agent Templates & Marketplace
    pub const TEMPLATES: &str = "/agents/templates";
    pub const CREATE_TEMPLATE: &str = "/agents/templates/create";
    pub const LISTINGS: &str = "/agents/marketplace/listings";
    pub const PURCHASE: &str = "/agents/marketplace/purchase";
    // Agent Health & Monitoring
    pub const HEALTH: &str = "/agents/{agent_id}/health";
    pub const METRICS: &str = "/agents/{agent_id}/metrics";
    pub const LOGS: &str = "/agents/{agent_id}/logs";
    pub const STATUS: &str = "/agents/{agent_id}/status";
    pub const LANE: &str = "/agents/{agent_id}/lane";
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

/// Treasury endpoints (Sprint 25)
pub mod treasury {
    pub const BALANCE: &str = "/treasury/balance";
    pub const SUMMARY: &str = "/treasury/summary";
    pub const DEPOSIT: &str = "/treasury/deposit";
    pub const WITHDRAW: &str = "/treasury/withdraw";
    pub const PROPOSALS: &str = "/treasury/proposals";
    pub const PROPOSAL_GET: &str = "/treasury/proposals/{id}";
    pub const PROPOSAL_CREATE: &str = "/treasury/proposals/create";
    pub const PROPOSAL_APPROVE: &str = "/treasury/proposals/{id}/approve";
    pub const PROPOSAL_EXECUTE: &str = "/treasury/proposals/{id}/execute";
    pub const ALLOCATIONS: &str = "/treasury/allocations";
    pub const ALLOCATION_GET: &str = "/treasury/allocations/{id}";
    pub const GUARDIANS: &str = "/treasury/guardians";
    pub const GUARDIAN_ADD: &str = "/treasury/guardians/add";
    pub const GUARDIAN_REMOVE: &str = "/treasury/guardians/remove";
    pub const SPENDING_LIMIT: &str = "/treasury/spending-limit";
    pub const SPENDING_HISTORY: &str = "/treasury/spending/history";
    pub const DAILY_STATS: &str = "/treasury/daily-stats";
}

/// Token endpoints
pub mod tokens {
    pub const LIST: &str = "/tokens";
    pub const AETH: &str = "/tokens/aeth";
    pub const FLUX: &str = "/tokens/flux";
    pub const ATH: &str = "/tokens/ath";
    pub const AETH_BALANCE: &str = "/tokens/aeth/balance/{address}";
    pub const FLUX_BALANCE: &str = "/tokens/flux/balance/{address}";
    pub const ATH_BALANCE: &str = "/tokens/ath/balance/{address}";
    pub const AETH_TRANSFER: &str = "/tokens/aeth/transfer";
    pub const FLUX_TRANSFER: &str = "/tokens/flux/transfer";
    pub const ATH_TRANSFER: &str = "/tokens/ath/transfer";
    pub const AETH_SUPPLY: &str = "/tokens/aeth/supply";
    pub const FLUX_SUPPLY: &str = "/tokens/flux/supply";
    pub const ATH_SUPPLY: &str = "/tokens/ath/supply";
    pub const BURN: &str = "/tokens/burn";
    pub const BURN_HISTORY: &str = "/tokens/burn/history";
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

/// User management request/response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisterRequest {
    pub username: String,
    pub email: String,
    pub wallet_address: String,
    pub password_hash: String,
    pub referral_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginRequest {
    pub identifier: String, // username or email
    pub password_hash: String,
    pub device_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub wallet_address: String,
    pub created_at: i64,
    pub last_login: i64,
    pub kyc_status: KycStatus,
    pub reputation_score: f64,
    pub total_earnings: u64,
    pub active_stakes: u64,
    pub mining_devices: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KycStatus {
    Pending,
    Submitted,
    Verified,
    Rejected,
    Expired,
}

/// Agent management request/response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegisterRequest {
    pub agent_name: String,
    pub agent_type: String,
    pub owner_address: String,
    pub capabilities: Vec<String>,
    pub pricing_model: PricingModel,
    pub availability: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub agent_id: String,
    pub agent_name: String,
    pub agent_type: String,
    pub owner_address: String,
    pub capabilities: Vec<String>,
    pub pricing_model: PricingModel,
    pub reputation_score: f64,
    pub total_tasks_completed: u64,
    pub average_rating: f64,
    pub availability: f64,
    pub status: AgentStatus,
    pub created_at: i64,
    pub last_active: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Active,
    Inactive,
    Suspended,
    UnderReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    Fixed { price_per_task: u64 },
    Hourly { rate_per_hour: u64 },
    Subscription { monthly_fee: u64 },
    Performance { base_fee: u64, performance_bonus_percent: f64 },
}

/// Task management structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task_type: String,
    pub description: String,
    pub required_capabilities: Vec<String>,
    pub budget: u64,
    pub deadline_epoch: u64,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub task_type: String,
    pub description: String,
    pub status: TaskStatus,
    pub assigned_agent_id: Option<String>,
    pub budget: u64,
    pub actual_cost: u64,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub deadline_epoch: u64,
    pub priority: TaskPriority,
    pub quality_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Open,
    InProgress,
    UnderReview,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
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
        // Agents - NEW ENDPOINTS
        EndpointMeta {
            path: agents::LIST.to_string(),
            method: "GET".to_string(),
            description: "List all registered agents".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["agents".to_string()],
        },
        EndpointMeta {
            path: agents::GET.to_string(),
            method: "GET".to_string(),
            description: "Get agent details by ID".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["agents".to_string()],
        },
        EndpointMeta {
            path: agents::REGISTER.to_string(),
            method: "POST".to_string(),
            description: "Register a new AI agent".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig { requests_per_minute: 5, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["agents".to_string(), "registration".to_string()],
        },
        EndpointMeta {
            path: agents::SEARCH.to_string(),
            method: "GET".to_string(),
            description: "Search agents by capabilities".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(120),
            tags: vec!["agents".to_string(), "search".to_string()],
        },
        EndpointMeta {
            path: agents::MARKETPLACE.to_string(),
            method: "GET".to_string(),
            description: "Browse agent marketplace".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["agents".to_string(), "marketplace".to_string()],
        },
        // User management - NEW ENDPOINTS
        EndpointMeta {
            path: "/users/register".to_string(),
            method: "POST".to_string(),
            description: "Register new user account".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig { requests_per_minute: 3, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["users".to_string(), "auth".to_string()],
        },
        EndpointMeta {
            path: "/users/login".to_string(),
            method: "POST".to_string(),
            description: "Authenticate user login".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig { requests_per_minute: 10, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["users".to_string(), "auth".to_string()],
        },
        EndpointMeta {
            path: "/users/{user_id}".to_string(),
            method: "GET".to_string(),
            description: "Get user profile".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["users".to_string()],
        },
        EndpointMeta {
            path: "/users/{user_id}/kyc".to_string(),
            method: "GET".to_string(),
            description: "Get user KYC status".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(30),
            tags: vec!["users".to_string(), "kyc".to_string()],
        },
        // Task management - NEW ENDPOINTS
        EndpointMeta {
            path: "/tasks/create".to_string(),
            method: "POST".to_string(),
            description: "Create new task for agents".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig { requests_per_minute: 20, ..Default::default() },
            cache_ttl_secs: None,
            tags: vec!["tasks".to_string()],
        },
        EndpointMeta {
            path: "/tasks/{task_id}".to_string(),
            method: "GET".to_string(),
            description: "Get task details".to_string(),
            auth_required: true,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(30),
            tags: vec!["tasks".to_string()],
        },
        EndpointMeta {
            path: "/tasks/active".to_string(),
            method: "GET".to_string(),
            description: "List active tasks".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_secs: Some(60),
            tags: vec!["tasks".to_string()],
        },
        // Health
        EndpointMeta {
            path: health::STATUS.to_string(),
            method: "GET".to_string(),
            description: "API health check".to_string(),
            auth_required: false,
            rate_limit: RateLimitConfig { requests_per_minute: 100, ..Default::default() },
            cache_ttl_secs: Some(10),
            tags: vec!["health".to_string()],
        },
    ]
}

/// API Router - Request dispatcher
pub struct ApiRouter {
    pub endpoints: Vec<EndpointMeta>,
    pub rate_limits: std::collections::HashMap<String, u32>,
}

impl ApiRouter {
    pub fn new() -> Self {
        ApiRouter {
            endpoints: get_all_endpoints(),
            rate_limits: std::collections::HashMap::new(),
        }
    }
    
    /// Register a new user
    pub fn register_user(&mut self, request: UserRegisterRequest) -> Result<UserResponse, ApiError> {
        // Validate input
        if request.username.len() < 3 || request.username.len() > 32 {
            return Err(ApiError {
                code: "INVALID_USERNAME".to_string(),
                message: "Username must be 3-32 characters".to_string(),
                details: None,
            });
        }
        
        if !request.email.contains('@') {
            return Err(ApiError {
                code: "INVALID_EMAIL".to_string(),
                message: "Invalid email format".to_string(),
                details: None,
            });
        }
        
        // Generate user ID
        let user_id = format!("user_{}", uuid::simple().as_str());
        
        Ok(UserResponse {
            user_id,
            username: request.username,
            email: request.email,
            wallet_address: request.wallet_address,
            created_at: chrono::Utc::now().timestamp(),
            last_login: 0,
            kyc_status: KycStatus::Pending,
            reputation_score: 50.0,
            total_earnings: 0,
            active_stakes: 0,
            mining_devices: 0,
        })
    }
    
    /// Authenticate user login
    pub fn login_user(&mut self, request: UserLoginRequest) -> Result<LoginResponse, ApiError> {
        // In production, this would verify credentials against database
        let session_token = format!("sess_{}", uuid::simple().as_str());
        let expires_at = chrono::Utc::now().timestamp() + 86400; // 24 hours
        
        Ok(LoginResponse {
            session_token,
            expires_at,
            user_id: request.identifier,
        })
    }
    
    /// Register a new AI agent
    pub fn register_agent(&mut self, request: AgentRegisterRequest) -> Result<AgentResponse, ApiError> {
        // Validate agent name
        if request.agent_name.len() < 3 || request.agent_name.len() > 64 {
            return Err(ApiError {
                code: "INVALID_AGENT_NAME".to_string(),
                message: "Agent name must be 3-64 characters".to_string(),
                details: None,
            });
        }
        
        // Validate capabilities
        if request.capabilities.is_empty() {
            return Err(ApiError {
                code: "NO_CAPABILITIES".to_string(),
                message: "Agent must have at least one capability".to_string(),
                details: None,
            });
        }
        
        // Generate agent ID
        let agent_id = format!("agent_{}", uuid::simple().as_str());
        
        Ok(AgentResponse {
            agent_id,
            agent_name: request.agent_name,
            agent_type: request.agent_type,
            owner_address: request.owner_address,
            capabilities: request.capabilities,
            pricing_model: request.pricing_model,
            reputation_score: 50.0,
            total_tasks_completed: 0,
            average_rating: 0.0,
            availability: request.availability,
            status: AgentStatus::Active,
            created_at: chrono::Utc::now().timestamp(),
            last_active: 0,
        })
    }
    
    /// Create a new task
    pub fn create_task(&mut self, request: TaskRequest) -> Result<TaskResponse, ApiError> {
        // Validate budget
        if request.budget == 0 {
            return Err(ApiError {
                code: "INVALID_BUDGET".to_string(),
                message: "Budget must be greater than 0".to_string(),
                details: None,
            });
        }
        
        // Validate deadline
        let current_epoch = 0; // In production, fetch from chain
        if request.deadline_epoch <= current_epoch {
            return Err(ApiError {
                code: "INVALID_DEADLINE".to_string(),
                message: "Deadline must be in the future".to_string(),
                details: None,
            });
        }
        
        // Generate task ID
        let task_id = format!("task_{}", uuid::simple().as_str());
        
        Ok(TaskResponse {
            task_id,
            task_type: request.task_type,
            description: request.description,
            status: TaskStatus::Open,
            assigned_agent_id: None,
            budget: request.budget,
            actual_cost: 0,
            created_at: chrono::Utc::now().timestamp(),
            started_at: None,
            completed_at: None,
            deadline_epoch: request.deadline_epoch,
            priority: request.priority,
            quality_score: None,
        })
    }
    
    /// Search agents by capabilities
    pub fn search_agents(&self, capabilities: Vec<String>) -> Vec<AgentResponse> {
        // In production, this would query the agent database
        vec![]
    }
    
    /// Get rate limit for endpoint
    pub fn get_rate_limit(&self, path: &str) -> RateLimitConfig {
        self.endpoints.iter()
            .find(|e| e.path == path)
            .map(|e| e.rate_limit.clone())
            .unwrap_or_default()
    }
    
    /// Check if request is rate limited
    pub fn is_rate_limited(&self, client_ip: &str, path: &str) -> bool {
        let key = format!("{}:{}", client_ip, path);
        let count = self.rate_limits.get(&key).copied().unwrap_or(0);
        let limit = self.get_rate_limit(path).requests_per_minute;
        count >= limit
    }
    
    /// Record request for rate limiting
    pub fn record_request(&mut self, client_ip: &str, path: &str) {
        let key = format!("{}:{}", client_ip, path);
        *self.rate_limits.entry(key).or_insert(0) += 1;
    }
}

/// Login response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub session_token: String,
    pub expires_at: i64,
    pub user_id: String,
}

/// JWT claims for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // subject (user/agent ID)
    pub iat: i64,    // issued at
    pub exp: i64,    // expiration
    pub role: String, // user, agent, admin
    pub permissions: Vec<String>,
}
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
