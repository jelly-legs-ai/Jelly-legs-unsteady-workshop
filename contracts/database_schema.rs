// Database Schema - AeTHer Chain
// PostgreSQL schema definitions for on-chain data indexing and querying

use serde::{Deserialize, Serialize};

/// Database schema version
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Account types in the AeTHer Chain ecosystem
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountType {
    User,
    Validator,
    Agent,
    Contract,
    Treasury,
    Unknown,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::User => "user",
            AccountType::Validator => "validator",
            AccountType::Agent => "agent",
            AccountType::Contract => "contract",
            AccountType::Treasury => "treasury",
            AccountType::Unknown => "unknown",
        }
    }
}

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: String,
    pub address: String,
    pub account_type: AccountType,
    pub created_at: i64,
    pub aeth_balance: i64,
    pub flux_balance: i64,
    pub staked_aeth: i64,
    pub pending_rewards: i64,
    pub is_verified: bool,
    pub kyc_level: u8,
}

/// Validator node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub id: String,
    pub address: String,
    pub name: String,
    pub ip_address: String,
    pub port: u16,
    pub staked_amount: i64,
    pub delegations_received: i64,
    pub commission_rate: f64,
    pub uptime_percent: f64,
    pub total_blocks_produced: u64,
    pub total_blocks_missed: u64,
    pub slashing_events: u32,
    pub last_active_epoch: u64,
    pub status: ValidatorStatus,
    pub location: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Jailed,
    Unjailing,
}

/// Staking position record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    pub id: String,
    pub owner_address: String,
    pub validator_id: Option<String>,
    pub pool_id: String,
    pub token_type: String,
    pub amount: i64,
    pub start_epoch: u64,
    pub end_epoch: Option<u64>,
    pub lock_end_epoch: Option<u64>,
    pub is_delegated: bool,
    pub rewards_claimed: i64,
    pub pending_rewards: i64,
    pub tier: Option<String>,
}

/// FLUX mining reward record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningReward {
    pub id: String,
    pub miner_address: String,
    pub device_id: String,
    pub epoch: u64,
    pub work_type: String,
    pub work_contribution: i64,
    pub reward_amount: i64,
    pub reward_epoch: u64,
    pub is_claimed: bool,
    pub claimed_at: Option<i64>,
}

/// AI Agent registration and KYC record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: String,
    pub agent_address: String,
    pub owner_address: String,
    pub name: String,
    pub symbol: String,
    pub category: String,
    pub capabilities: Vec<String>,
    pub kyc_status: KYCStatus,
    pub kyc_issuer: Option<String>,
    pub kyc_timestamp: Option<i64>,
    pub reputation_score: f64,
    pub total_tasks_executed: u64,
    pub total_flux_earned: i64,
    pub stake_bonded: i64,
    pub verified_claims: Vec<String>,
    pub metadata_url: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KYCStatus {
    Unverified,
    Pending,
    Verified,
    Revoked,
    Expired,
}

/// Governance proposal record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: String,
    pub proposal_id: u64,
    pub title: String,
    pub description: String,
    pub author_address: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub created_at: i64,
    pub voting_start_epoch: u64,
    pub voting_end_epoch: u64,
    pub total_yes_votes: i64,
    pub total_no_votes: i64,
    pub total_abstain_votes: i64,
    pub quorum_required: i64,
    pub execution_payload: Option<String>,
    pub execution_result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    ParameterChange,
    TreasurySpend,
    ProtocolUpgrade,
    Emergency,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Failed,
    Executed,
    Cancelled,
}

/// Vote record for governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRecord {
    pub id: String,
    pub proposal_id: u64,
    pub voter_address: String,
    pub vote: VoteChoice,
    pub voting_power: i64,
    pub quadratic_weight: f64,
    pub timestamp: i64,
    pub epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

/// Cross-chain bridge transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub id: String,
    pub transaction_hash: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub token_type: String,
    pub amount: i64,
    pub sender_address: String,
    pub recipient_address: String,
    pub status: BridgeStatus,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub bridge_fee: i64,
    pub minting_tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BridgeStatus {
    Pending,
    Bridging,
    Completed,
    Failed,
    Refunded,
}

/// Network statistics aggregated by epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochStats {
    pub epoch: u64,
    pub total_transactions: u64,
    pub total_fees_collected: i64,
    pub total_rewards_distributed: i64,
    pub active_validators: u32,
    pub total_stake: i64,
    pub total_delegations: i64,
    pub avg_block_time_ms: u32,
    pub total_slashing_events: u32,
    pub mining_rewards_issued: i64,
    pub timestamp: i64,
}

/// Device registration for mobile mining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningDevice {
    pub id: String,
    pub device_address: String,
    pub owner_address: String,
    pub device_type: DeviceType,
    pub device_tier: DeviceTier,
    pub operating_system: String,
    pub uptime_last_7_days: f64,
    pub total_work_contributions: i64,
    pub total_rewards_earned: i64,
    pub is_registered: bool,
    pub registration_epoch: u64,
    pub last_active_epoch: u64,
    pub country_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Mobile,
    Laptop,
    Desktop,
    Server,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceTier {
    Tier1,  // High-end devices
    Tier2,  // Mid-range devices  
    Tier3,  // Entry-level devices
}

impl DeviceTier {
    pub fn reward_multiplier(&self) -> f64 {
        match self {
            DeviceTier::Tier1 => 1.5,
            DeviceTier::Tier2 => 1.0,
            DeviceTier::Tier3 => 0.5,
        }
    }
}

/// SQL Schema Definitions

pub const CREATE_TABLES_SQL: &str = r#"
-- AeTHer Chain Database Schema v1.0.0

-- User accounts table
CREATE TABLE IF NOT EXISTS user_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address VARCHAR(64) UNIQUE NOT NULL,
    account_type VARCHAR(20) NOT NULL DEFAULT 'user',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    aeth_balance BIGINT DEFAULT 0,
    flux_balance BIGINT DEFAULT 0,
    staked_aeth BIGINT DEFAULT 0,
    pending_rewards BIGINT DEFAULT 0,
    is_verified BOOLEAN DEFAULT FALSE,
    kyc_level SMALLINT DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    CONSTRAINT valid_account_type CHECK (account_type IN ('user', 'validator', 'agent', 'contract', 'treasury'))
);

CREATE INDEX idx_user_address ON user_accounts(address);
CREATE INDEX idx_user_type ON user_accounts(account_type);

-- Validators table
CREATE TABLE IF NOT EXISTS validators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address VARCHAR(64) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    ip_address VARCHAR(45),
    port SMALLINT DEFAULT 8080,
    staked_amount BIGINT NOT NULL DEFAULT 0,
    delegations_received BIGINT NOT NULL DEFAULT 0,
    commission_rate DECIMAL(5,4) DEFAULT 0.1000,
    uptime_percent DECIMAL(5,2) DEFAULT 0.00,
    total_blocks_produced BIGINT DEFAULT 0,
    total_blocks_missed BIGINT DEFAULT 0,
    slashing_events SMALLINT DEFAULT 0,
    last_active_epoch BIGINT DEFAULT 0,
    status VARCHAR(20) DEFAULT 'inactive',
    location VARCHAR(100),
    version VARCHAR(20),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT valid_status CHECK (status IN ('active', 'inactive', 'jailed', 'unjailing'))
);

CREATE INDEX idx_validator_address ON validators(address);
CREATE INDEX idx_validator_status ON validators(status);

-- Staking positions table
CREATE TABLE IF NOT EXISTS staking_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_address VARCHAR(64) NOT NULL,
    validator_id UUID REFERENCES validators(id),
    pool_id VARCHAR(50) NOT NULL,
    token_type VARCHAR(10) NOT NULL,
    amount BIGINT NOT NULL,
    start_epoch BIGINT NOT NULL,
    end_epoch BIGINT,
    lock_end_epoch BIGINT,
    is_delegated BOOLEAN DEFAULT FALSE,
    rewards_claimed BIGINT DEFAULT 0,
    pending_rewards BIGINT DEFAULT 0,
    tier VARCHAR(20),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_stake_owner ON staking_positions(owner_address);
CREATE INDEX idx_stake_validator ON staking_positions(validator_id);
CREATE INDEX idx_stake_pool ON staking_positions(pool_id);

-- Mining rewards table
CREATE TABLE IF NOT EXISTS mining_rewards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    miner_address VARCHAR(64) NOT NULL,
    device_id VARCHAR(100) NOT NULL,
    epoch BIGINT NOT NULL,
    work_type VARCHAR(50) NOT NULL,
    work_contribution BIGINT NOT NULL,
    reward_amount BIGINT NOT NULL,
    reward_epoch BIGINT NOT NULL,
    is_claimed BOOLEAN DEFAULT FALSE,
    claimed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_mining_epoch ON mining_rewards(epoch);
CREATE INDEX idx_mining_miner ON mining_rewards(miner_address);
CREATE INDEX idx_mining_device ON mining_rewards(device_id);

-- Agent registry table
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_address VARCHAR(64) UNIQUE NOT NULL,
    owner_address VARCHAR(64) NOT NULL,
    name VARCHAR(100) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    category VARCHAR(50) NOT NULL,
    capabilities TEXT[] DEFAULT '{}',
    kyc_status VARCHAR(20) DEFAULT 'unverified',
    kyc_issuer VARCHAR(100),
    kyc_timestamp TIMESTAMP WITH TIME ZONE,
    reputation_score DECIMAL(3,2) DEFAULT 0.00,
    total_tasks_executed BIGINT DEFAULT 0,
    total_flux_earned BIGINT DEFAULT 0,
    stake_bonded BIGINT DEFAULT 0,
    verified_claims TEXT[] DEFAULT '{}',
    metadata_url VARCHAR(255),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT valid_kyc CHECK (kyc_status IN ('unverified', 'pending', 'verified', 'revoked', 'expired'))
);

CREATE INDEX idx_agent_address ON agents(agent_address);
CREATE INDEX idx_agent_owner ON agents(owner_address);
CREATE INDEX idx_agent_kyc ON agents(kyc_status);

-- Agent-User linking table (for user/agent management)
CREATE TABLE IF NOT EXISTS agent_user_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    user_address VARCHAR(64) NOT NULL,
    link_type VARCHAR(20) DEFAULT 'owner',
    permissions TEXT[] DEFAULT '{}',
    linked_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    unlinked_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT TRUE,
    metadata JSONB DEFAULT '{}',
    CONSTRAINT valid_link_type CHECK (link_type IN ('owner', 'operator', 'viewer', 'service'))
);

CREATE INDEX idx_link_agent ON agent_user_links(agent_id);
CREATE INDEX idx_link_user ON agent_user_links(user_address);

-- Agent configuration table
CREATE TABLE IF NOT EXISTS agent_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    config_key VARCHAR(100) NOT NULL,
    config_value JSONB NOT NULL,
    version INTEGER DEFAULT 1,
    updated_by VARCHAR(64),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT unique_agent_config UNIQUE(agent_id, config_key)
);

CREATE INDEX idx_config_agent ON agent_configs(agent_id);

-- Agent lifecycle events table
CREATE TABLE IF NOT EXISTS agent_lifecycle_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    event_type VARCHAR(30) NOT NULL,
    event_data JSONB,
    triggered_by VARCHAR(64),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT valid_event_type CHECK (event_type IN ('created', 'paused', 'resumed', 'restarted', 'scaled', 'deployed', 'deleted', 'error', 'health_check'))
);

CREATE INDEX idx_lifecycle_agent ON agent_lifecycle_events(agent_id);
CREATE INDEX idx_lifecycle_type ON agent_lifecycle_events(event_type);

-- Agent health metrics table
CREATE TABLE IF NOT EXISTS agent_health_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    epoch BIGINT NOT NULL,
    cpu_usage_percent DECIMAL(5,2),
    memory_usage_mb BIGINT,
    disk_usage_mb BIGINT,
    network_latency_ms INTEGER,
    error_count INTEGER DEFAULT 0,
    task_queue_size INTEGER DEFAULT 0,
    response_time_avg_ms INTEGER,
    uptime_percent DECIMAL(5,2),
    status VARCHAR(20) DEFAULT 'unknown',
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT valid_health_status CHECK (status IN ('healthy', 'degraded', 'unhealthy', 'unknown'))
);

CREATE INDEX idx_health_agent ON agent_health_metrics(agent_id);
CREATE INDEX idx_health_epoch ON agent_health_metrics(epoch);

-- Agent templates table
CREATE TABLE IF NOT EXISTS agent_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_name VARCHAR(100) NOT NULL,
    template_version VARCHAR(20) NOT NULL,
    category VARCHAR(50) NOT NULL,
    description TEXT,
    author_address VARCHAR(64) NOT NULL,
    base_config JSONB NOT NULL,
    required_capabilities TEXT[] DEFAULT '{}',
    deployment_script_url VARCHAR(255),
    is_public BOOLEAN DEFAULT FALSE,
    price_flux BIGINT DEFAULT 0,
    total_deployments BIGINT DEFAULT 0,
    average_rating DECIMAL(3,2) DEFAULT 0.00,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_template_name ON agent_templates(template_name);
CREATE INDEX idx_template_category ON agent_templates(category);
CREATE INDEX idx_template_author ON agent_templates(author_address);

-- Agent marketplace listings table
CREATE TABLE IF NOT EXISTS agent_marketplace_listings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    seller_address VARCHAR(64) NOT NULL,
    listing_type VARCHAR(20) DEFAULT 'sale',
    price_flux BIGINT NOT NULL,
    currency VARCHAR(10) DEFAULT 'FLUX',
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    sold_at TIMESTAMP WITH TIME ZONE,
    buyer_address VARCHAR(64),
    CONSTRAINT valid_listing_type CHECK (listing_type IN ('sale', 'rental', 'subscription', 'freemium'))
);

CREATE INDEX idx_listing_agent ON agent_marketplace_listings(agent_id);
CREATE INDEX idx_listing_seller ON agent_marketplace_listings(seller_address);
CREATE INDEX idx_listing_active ON agent_marketplace_listings(is_active);

-- Agent task execution logs table (enhanced)
CREATE TABLE IF NOT EXISTS agent_task_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    task_id VARCHAR(100) NOT NULL,
    task_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    input_data JSONB,
    output_data JSONB,
    flux_earned BIGINT DEFAULT 0,
    execution_time_ms INTEGER,
    error_message TEXT,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT valid_task_status CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled'))
);

CREATE INDEX idx_task_agent ON agent_task_logs(agent_id);
CREATE INDEX idx_task_status ON agent_task_logs(status);
CREATE INDEX idx_task_type ON agent_task_logs(task_type);

-- Governance proposals table
CREATE TABLE IF NOT EXISTS proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id BIGSERIAL UNIQUE,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    author_address VARCHAR(64) NOT NULL,
    proposal_type VARCHAR(30) NOT NULL,
    status VARCHAR(20) DEFAULT 'draft',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    voting_start_epoch BIGINT,
    voting_end_epoch BIGINT,
    total_yes_votes BIGINT DEFAULT 0,
    total_no_votes BIGINT DEFAULT 0,
    total_abstain_votes BIGINT DEFAULT 0,
    quorum_required BIGINT DEFAULT 0,
    execution_payload JSONB,
    execution_result TEXT,
    CONSTRAINT valid_proposal_type CHECK (proposal_type IN ('parameter_change', 'treasury_spend', 'protocol_upgrade', 'emergency', 'general')),
    CONSTRAINT valid_proposal_status CHECK (status IN ('draft', 'active', 'passed', 'failed', 'executed', 'cancelled'))
);

CREATE INDEX idx_proposal_id ON proposals(proposal_id);
CREATE INDEX idx_proposal_status ON proposals(status);
CREATE INDEX idx_proposal_author ON proposals(author_address);

-- Vote records table
CREATE TABLE IF NOT EXISTS votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id BIGINT REFERENCES proposals(proposal_id),
    voter_address VARCHAR(64) NOT NULL,
    vote VARCHAR(10) NOT NULL,
    voting_power BIGINT NOT NULL,
    quadratic_weight DECIMAL(10,4) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    epoch BIGINT NOT NULL,
    CONSTRAINT valid_vote CHECK (vote IN ('yes', 'no', 'abstain'))
);

CREATE INDEX idx_vote_proposal ON votes(proposal_id);
CREATE INDEX idx_vote_voter ON votes(voter_address);

-- Bridge transactions table
CREATE TABLE IF NOT EXISTS bridge_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_hash VARCHAR(128) UNIQUE NOT NULL,
    source_chain VARCHAR(30) NOT NULL,
    destination_chain VARCHAR(30) NOT NULL,
    token_type VARCHAR(20) NOT NULL,
    amount BIGINT NOT NULL,
    sender_address VARCHAR(64) NOT NULL,
    recipient_address VARCHAR(64) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    bridge_fee BIGINT DEFAULT 0,
    minting_tx_hash VARCHAR(128),
    CONSTRAINT valid_bridge_status CHECK (status IN ('pending', 'bridging', 'completed', 'failed', 'refunded'))
);

CREATE INDEX idx_bridge_tx_hash ON bridge_transactions(transaction_hash);
CREATE INDEX idx_bridge_status ON bridge_transactions(status);
CREATE INDEX idx_bridge_sender ON bridge_transactions(sender_address);

-- Epoch statistics table
CREATE TABLE IF NOT EXISTS epoch_stats (
    epoch BIGINT PRIMARY KEY,
    total_transactions BIGINT DEFAULT 0,
    total_fees_collected BIGINT DEFAULT 0,
    total_rewards_distributed BIGINT DEFAULT 0,
    active_validators SMALLINT DEFAULT 0,
    total_stake BIGINT DEFAULT 0,
    total_delegations BIGINT DEFAULT 0,
    avg_block_time_ms SMALLINT DEFAULT 0,
    total_slashing_events SMALLINT DEFAULT 0,
    mining_rewards_issued BIGINT DEFAULT 0,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Mining devices table
CREATE TABLE IF NOT EXISTS mining_devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_address VARCHAR(100) UNIQUE NOT NULL,
    owner_address VARCHAR(64) NOT NULL,
    device_type VARCHAR(20) NOT NULL,
    device_tier VARCHAR(10) NOT NULL,
    operating_system VARCHAR(30) NOT NULL,
    uptime_last_7_days DECIMAL(5,2) DEFAULT 0.00,
    total_work_contributions BIGINT DEFAULT 0,
    total_rewards_earned BIGINT DEFAULT 0,
    is_registered BOOLEAN DEFAULT FALSE,
    registration_epoch BIGINT,
    last_active_epoch BIGINT DEFAULT 0,
    country_code VARCHAR(3),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT valid_device_type CHECK (device_type IN ('mobile', 'laptop', 'desktop', 'server')),
    CONSTRAINT valid_device_tier CHECK (device_tier IN ('tier1', 'tier2', 'tier3'))
);

CREATE INDEX idx_device_address ON mining_devices(device_address);
CREATE INDEX idx_device_owner ON mining_devices(owner_address);
CREATE INDEX idx_device_status ON mining_devices(is_registered);

-- Transaction index table for fast lookups
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_hash VARCHAR(128) UNIQUE NOT NULL,
    block_number BIGINT NOT NULL,
    epoch BIGINT NOT NULL,
    sender_address VARCHAR(64) NOT NULL,
    recipient_address VARCHAR(64),
    token_type VARCHAR(10),
    amount BIGINT,
    fee BIGINT,
    transaction_type VARCHAR(30) NOT NULL,
    status VARCHAR(20) DEFAULT 'confirmed',
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT valid_tx_type CHECK (transaction_type IN ('transfer', 'stake', 'unstake', 'claim', 'delegate', 'undelegate', 'register', 'kyc', 'vote', 'bridge', 'contract')),
    CONSTRAINT valid_tx_status CHECK (status IN ('pending', 'confirmed', 'failed'))
);

CREATE INDEX idx_tx_hash ON transactions(transaction_hash);
CREATE INDEX idx_tx_block ON transactions(block_number);
CREATE INDEX idx_tx_sender ON transactions(sender_address);
CREATE INDEX idx_tx_recipient ON transactions(recipient_address);
CREATE INDEX idx_tx_epoch ON transactions(epoch);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_account_creation() {
        let user = UserAccount {
            id: "test-1".to_string(),
            address: "0x1234".to_string(),
            account_type: AccountType::User,
            created_at: 1234567890,
            aeth_balance: 1000,
            flux_balance: 5000,
            staked_aeth: 500,
            pending_rewards: 25,
            is_verified: true,
            kyc_level: 1,
        };
        assert_eq!(user.aeth_balance, 1000);
    }

    #[test]
    fn test_validator_status() {
        assert_eq!(ValidatorStatus::Active.as_str(), "active");
        assert_eq!(ValidatorStatus::Jailed.as_str(), "jailed");
    }

    #[test]
    fn test_device_tier_multiplier() {
        assert_eq!(DeviceTier::Tier1.reward_multiplier(), 1.5);
        assert_eq!(DeviceTier::Tier2.reward_multiplier(), 1.0);
        assert_eq!(DeviceTier::Tier3.reward_multiplier(), 0.5);
    }

    #[test]
    fn test_kyc_status() {
        assert_eq!(KYCStatus::Verified.as_str(), "verified");
        assert_eq!(KYCStatus::Pending.as_str(), "pending");
    }
}
