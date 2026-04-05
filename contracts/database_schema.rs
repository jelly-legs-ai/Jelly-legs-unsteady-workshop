// Database Schema - AeTHer Chain
// Replit DB / SQLite schema definitions for chain data persistence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// TABLE DEFINITIONS
// ============================================================================

/// Users table - Core user account data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersTable {
    pub user_id: String,           // Primary key: user_{address}
    pub wallet_address: String,    // Unique, indexed
    pub username: String,          // Unique
    pub email: String,             // Unique, encrypted
    pub password_hash: String,     // Encrypted
    pub created_at: u64,           // Epoch timestamp
    pub last_login: u64,
    pub kyc_status: String,        // pending, submitted, verified, rejected, expired
    pub kyc_tier: u32,             // 0-3
    pub reputation_score: f64,     // 0-100
    pub total_earnings: u64,       // Total FLUX earned
    pub is_suspended: bool,
    pub suspension_reason: Option<String>,
    pub referral_code: String,
    pub referred_by: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Accounts table - Wallet/account balances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsTable {
    pub account_id: String,        // Primary key: acc_{address}
    pub user_id: String,           // Foreign key -> users.user_id
    pub wallet_address: String,    // Unique, indexed
    pub aeth_balance: u64,         // Native token (8 decimals)
    pub flux_balance: u64,         // Utility token (8 decimals)
    pub ath_balance: u64,          // Governance token (8 decimals)
    pub staked_aeth: u64,
    pub staked_flux: u64,
    pub staked_ath: u64,
    pub delegated_amount: u64,
    pub pending_rewards: u64,
    pub total_transactions: u64,
    pub last_activity: u64,
    pub account_type: String,      // user, agent, contract, treasury
}

/// Agents table - Registered AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsTable {
    pub agent_id: String,          // Primary key: agent_{uuid}
    pub owner_user_id: String,     // Foreign key -> users.user_id
    pub agent_name: String,
    pub agent_type: String,        // mining, staking, governance, trading, etc.
    pub capabilities: Vec<String>, // JSON array
    pub status: String,            // active, inactive, suspended, under_review
    pub reputation_score: f64,     // 0-100
    pub total_tasks: u64,
    pub successful_tasks: u64,
    pub total_earnings: u64,
    pub pricing_model: String,     // JSON: {type: "fixed"|"hourly"|"subscription", ...}
    pub availability: f64,         // 0.0-1.0
    pub created_at: u64,
    pub last_active: u64,
    pub kyc_verified: bool,
    pub metadata: HashMap<String, String>,
}

/// Devices table - Mining device registrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicesTable {
    pub device_id: String,         // Primary key: dev_{uuid}
    pub owner_user_id: String,     // Foreign key -> users.user_id
    pub device_name: String,
    pub device_tier: String,       // mobile, laptop, desktop, server
    pub ram_gb: u32,
    pub cpu_cores: u32,
    pub gpu_model: Option<String>,
    pub os_type: String,           // android, ios, windows, macos, linux
    pub app_version: String,
    pub status: String,            // active, inactive, offline, banned
    pub uptime_percentage: f64,
    pub contribution_score: f64,
    pub epochs_mined: u64,
    pub total_rewards: u64,
    pub pending_rewards: u64,
    pub last_heartbeat: u64,
    pub registered_at: u64,
    pub metadata: HashMap<String, String>,
}

/// Stakes table - Staking positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakesTable {
    pub stake_id: String,          // Primary key: stake_{uuid}
    pub user_id: String,           // Foreign key -> users.user_id
    pub pool_id: String,           // Foreign key -> staking_pools.pool_id
    pub token_type: String,        // AETH, FLUX, ATH
    pub amount: u64,
    pub start_epoch: u64,
    pub lock_end_epoch: u64,
    pub is_locked: bool,
    pub auto_compound: bool,
    pub rewards_claimed: u64,
    pub last_claim_epoch: u64,
    pub status: String,            // active, unstaking, claimed, expired
    pub created_at: u64,
}

/// Staking Pools table - Pool configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPoolsTable {
    pub pool_id: String,           // Primary key: pool_{token}
    pub name: String,
    pub token_type: String,        // AETH, FLUX, ATH
    pub reward_rate: f64,          // APY as decimal
    pub min_stake: u64,
    pub max_stake: Option<u64>,
    pub lockup_epochs: u64,
    pub total_staked: u64,
    pub active_stakers: u64,
    pub total_rewards_distributed: u64,
    pub is_active: bool,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
}

/// Validators table - Validator nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorsTable {
    pub validator_id: String,      // Primary key: val_{address}
    pub wallet_address: String,    // Unique
    pub operator_user_id: Option<String>, // Foreign key -> users.user_id
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub commission_rate: f64,      // 0.0-1.0
    pub total_delegated: u64,
    pub delegator_count: u64,
    pub uptime_percent: f64,
    pub slashing_events: u64,
    pub total_rewards_earned: u64,
    pub status: String,            // active, inactive, jailed, tombstoned
    pub jailed_until: Option<u64>,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
}

/// Delegations table - Stake delegations to validators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationsTable {
    pub delegation_id: String,     // Primary key: del_{uuid}
    pub delegator_user_id: String, // Foreign key -> users.user_id
    pub validator_id: String,      // Foreign key -> validators.validator_id
    pub amount: u64,
    pub start_epoch: u64,
    pub last_claim_epoch: u64,
    pub rewards_claimed: u64,
    pub is_claimable: bool,
    pub status: String,            // active, unbonding, claimed
    pub unbonding_epoch: Option<u64>,
    pub created_at: u64,
}

/// Transactions table - All chain transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsTable {
    pub tx_hash: String,           // Primary key
    pub block_number: u64,
    pub epoch: u64,
    pub from_address: String,
    pub to_address: String,
    pub tx_type: String,           // transfer, stake, unstake, claim, delegate, etc.
    pub token_type: String,        // AETH, FLUX, ATH
    pub amount: u64,
    pub fee: u64,
    pub status: String,            // pending, confirmed, failed
    pub timestamp: u64,
    pub gas_used: u64,
    pub memo: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Blocks table - Chain blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocksTable {
    pub block_number: u64,         // Primary key
    pub block_hash: String,        // Unique
    pub parent_hash: String,
    pub epoch: u64,
    pub proposer_validator: String,
    pub transaction_count: u32,
    pub total_fees: u64,
    pub block_size: u32,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub timestamp: u64,
    pub state_root: String,
    pub metadata: HashMap<String, String>,
}

/// Epochs table - Epoch statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochsTable {
    pub epoch: u64,                // Primary key
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub block_count: u32,
    pub transaction_count: u64,
    pub total_rewards_distributed: u64,
    pub active_validators: u32,
    pub active_miners: u32,
    pub total_staked: u64,
    pub network_uptime: f64,
    pub status: String,            // active, completed, archived
}

/// Subscriptions table - User subscriptions (Replit DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionsTable {
    pub subscription_id: String,   // Primary key: sub_{uuid}
    pub user_id: String,           // Foreign key -> users.user_id
    pub agent_id: Option<String>,  // Foreign key -> agents.agent_id
    pub tier: String,              // free, basic, professional, enterprise
    pub status: String,            // active, trial, expired, cancelled, past_due
    pub billing_cycle: String,     // monthly, quarterly, yearly, lifetime
    pub started_at: u64,
    pub current_period_start: u64,
    pub current_period_end: u64,
    pub trial_end: Option<u64>,
    pub cancel_at_period_end: bool,
    pub flux_paid_total: u64,
    pub next_billing_epoch: u64,
    pub auto_renew: bool,
    pub metadata: HashMap<String, String>,
}

/// Invoices table - Billing invoices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoicesTable {
    pub invoice_id: String,        // Primary key: inv_{uuid}
    pub subscription_id: String,   // Foreign key -> subscriptions.subscription_id
    pub user_id: String,           // Foreign key -> users.user_id
    pub amount_flux: u64,
    pub status: String,            // draft, open, paid, uncollectible, void
    pub created_at: u64,
    pub due_at: u64,
    pub paid_at: Option<u64>,
    pub billing_period_start: u64,
    pub billing_period_end: u64,
    pub line_items: Vec<String>,   // JSON array
    pub metadata: HashMap<String, String>,
}

/// Tasks table - Agent task marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksTable {
    pub task_id: String,           // Primary key: task_{uuid}
    pub creator_user_id: String,   // Foreign key -> users.user_id
    pub assigned_agent_id: Option<String>, // Foreign key -> agents.agent_id
    pub task_type: String,
    pub description: String,
    pub required_capabilities: Vec<String>,
    pub budget: u64,
    pub actual_cost: u64,
    pub status: String,            // open, in_progress, under_review, completed, failed, cancelled
    pub priority: String,          // low, normal, high, critical
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub deadline_epoch: u64,
    pub quality_score: Option<f64>,
    pub metadata: HashMap<String, String>,
}

/// Governance Proposals table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalsTable {
    pub proposal_id: String,       // Primary key: prop_{uuid}
    pub proposer_user_id: String,  // Foreign key -> users.user_id
    pub title: String,
    pub description: String,
    pub proposal_type: String,     // parameter_change, treasury_spend, upgrade, etc.
    pub status: String,            // draft, active, passed, rejected, executed, expired
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    pub quorum_required: u64,
    pub voting_start_epoch: u64,
    pub voting_end_epoch: u64,
    pub execution_epoch: Option<u64>,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
}

/// Votes table - Governance votes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotesTable {
    pub vote_id: String,           // Primary key: vote_{uuid}
    pub proposal_id: String,       // Foreign key -> proposals.proposal_id
    pub voter_user_id: String,     // Foreign key -> users.user_id
    pub vote_weight: u64,          // Based on stake
    pub vote: String,              // for, against, abstain
    pub voted_at: u64,
    pub metadata: HashMap<String, String>,
}

/// Notifications table - User notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsTable {
    pub notification_id: String,   // Primary key: notif_{uuid}
    pub user_id: String,           // Foreign key -> users.user_id
    pub title: String,
    pub message: String,
    pub notification_type: String, // reward, stake, governance, system, etc.
    pub priority: String,          // low, normal, high, urgent
    pub is_read: bool,
    pub created_at: u64,
    pub read_at: Option<u64>,
    pub action_url: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Activity Logs table - Audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogsTable {
    pub log_id: String,            // Primary key: log_{uuid}
    pub user_id: String,           // Foreign key -> users.user_id
    pub action: String,            // login, stake, claim, vote, etc.
    pub resource_type: String,     // user, stake, proposal, etc.
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String,            // success, failure, pending
    pub error_message: Option<String>,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// INDEX DEFINITIONS
// ============================================================================

/// Database index configurations
pub struct IndexConfig {
    pub table: &'static str,
    pub index_name: &'static str,
    pub columns: Vec<&'static str>,
    pub unique: bool,
}

pub fn get_all_indexes() -> Vec<IndexConfig> {
    vec![
        // Users indexes
        IndexConfig {
            table: "users",
            index_name: "idx_users_wallet",
            columns: vec!["wallet_address"],
            unique: true,
        },
        IndexConfig {
            table: "users",
            index_name: "idx_users_username",
            columns: vec!["username"],
            unique: true,
        },
        IndexConfig {
            table: "users",
            index_name: "idx_users_kyc",
            columns: vec!["kyc_status"],
            unique: false,
        },
        // Accounts indexes
        IndexConfig {
            table: "accounts",
            index_name: "idx_accounts_wallet",
            columns: vec!["wallet_address"],
            unique: true,
        },
        IndexConfig {
            table: "accounts",
            index_name: "idx_accounts_user",
            columns: vec!["user_id"],
            unique: false,
        },
        // Agents indexes
        IndexConfig {
            table: "agents",
            index_name: "idx_agents_owner",
            columns: vec!["owner_user_id"],
            unique: false,
        },
        IndexConfig {
            table: "agents",
            index_name: "idx_agents_status",
            columns: vec!["status"],
            unique: false,
        },
        // Stakes indexes
        IndexConfig {
            table: "stakes",
            index_name: "idx_stakes_user",
            columns: vec!["user_id"],
            unique: false,
        },
        IndexConfig {
            table: "stakes",
            index_name: "idx_stakes_pool",
            columns: vec!["pool_id"],
            unique: false,
        },
        IndexConfig {
            table: "stakes",
            index_name: "idx_stakes_status",
            columns: vec!["status"],
            unique: false,
        },
        // Transactions indexes
        IndexConfig {
            table: "transactions",
            index_name: "idx_tx_from",
            columns: vec!["from_address"],
            unique: false,
        },
        IndexConfig {
            table: "transactions",
            index_name: "idx_tx_to",
            columns: vec!["to_address"],
            unique: false,
        },
        IndexConfig {
            table: "transactions",
            index_name: "idx_tx_epoch",
            columns: vec!["epoch"],
            unique: false,
        },
        // Subscriptions indexes
        IndexConfig {
            table: "subscriptions",
            index_name: "idx_subs_user",
            columns: vec!["user_id"],
            unique: false,
        },
        IndexConfig {
            table: "subscriptions",
            index_name: "idx_subs_status",
            columns: vec!["status"],
            unique: false,
        },
    ]
}

// ============================================================================
// DATABASE SCHEMA MANAGER
// ============================================================================

/// Schema version for migrations
pub const SCHEMA_VERSION: u32 = 1;

/// Database schema manager
pub struct DatabaseSchema {
    pub version: u32,
    pub tables: Vec<&'static str>,
}

impl DatabaseSchema {
    pub fn new() -> Self {
        DatabaseSchema {
            version: SCHEMA_VERSION,
            tables: vec![
                "users",
                "accounts",
                "agents",
                "devices",
                "stakes",
                "staking_pools",
                "validators",
                "delegations",
                "transactions",
                "blocks",
                "epochs",
                "subscriptions",
                "invoices",
                "tasks",
                "proposals",
                "votes",
                "notifications",
                "activity_logs",
            ],
        }
    }
    
    /// Get CREATE TABLE statement for a table
    pub fn get_create_table(&self, table_name: &str) -> &'static str {
        match table_name {
            "users" => r#"
                CREATE TABLE IF NOT EXISTS users (
                    user_id TEXT PRIMARY KEY,
                    wallet_address TEXT UNIQUE NOT NULL,
                    username TEXT UNIQUE NOT NULL,
                    email TEXT UNIQUE NOT NULL,
                    password_hash TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    last_login INTEGER NOT NULL,
                    kyc_status TEXT NOT NULL,
                    kyc_tier INTEGER NOT NULL,
                    reputation_score REAL NOT NULL,
                    total_earnings INTEGER NOT NULL,
                    is_suspended BOOLEAN NOT NULL,
                    suspension_reason TEXT,
                    referral_code TEXT NOT NULL,
                    referred_by TEXT,
                    metadata TEXT
                )
            "#,
            "accounts" => r#"
                CREATE TABLE IF NOT EXISTS accounts (
                    account_id TEXT PRIMARY KEY,
                    user_id TEXT NOT NULL,
                    wallet_address TEXT UNIQUE NOT NULL,
                    aeth_balance INTEGER NOT NULL,
                    flux_balance INTEGER NOT NULL,
                    ath_balance INTEGER NOT NULL,
                    staked_aeth INTEGER NOT NULL,
                    staked_flux INTEGER NOT NULL,
                    staked_ath INTEGER NOT NULL,
                    delegated_amount INTEGER NOT NULL,
                    pending_rewards INTEGER NOT NULL,
                    total_transactions INTEGER NOT NULL,
                    last_activity INTEGER NOT NULL,
                    account_type TEXT NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(user_id)
                )
            "#,
            "stakes" => r#"
                CREATE TABLE IF NOT EXISTS stakes (
                    stake_id TEXT PRIMARY KEY,
                    user_id TEXT NOT NULL,
                    pool_id TEXT NOT NULL,
                    token_type TEXT NOT NULL,
                    amount INTEGER NOT NULL,
                    start_epoch INTEGER NOT NULL,
                    lock_end_epoch INTEGER NOT NULL,
                    is_locked BOOLEAN NOT NULL,
                    auto_compound BOOLEAN NOT NULL,
                    rewards_claimed INTEGER NOT NULL,
                    last_claim_epoch INTEGER NOT NULL,
                    status TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(user_id),
                    FOREIGN KEY (pool_id) REFERENCES staking_pools(pool_id)
                )
            "#,
            "staking_pools" => r#"
                CREATE TABLE IF NOT EXISTS staking_pools (
                    pool_id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    token_type TEXT NOT NULL,
                    reward_rate REAL NOT NULL,
                    min_stake INTEGER NOT NULL,
                    max_stake INTEGER,
                    lockup_epochs INTEGER NOT NULL,
                    total_staked INTEGER NOT NULL,
                    active_stakers INTEGER NOT NULL,
                    total_rewards_distributed INTEGER NOT NULL,
                    is_active BOOLEAN NOT NULL,
                    created_at INTEGER NOT NULL
                )
            "#,
            "subscriptions" => r#"
                CREATE TABLE IF NOT EXISTS subscriptions (
                    subscription_id TEXT PRIMARY KEY,
                    user_id TEXT NOT NULL,
                    agent_id TEXT,
                    tier TEXT NOT NULL,
                    status TEXT NOT NULL,
                    billing_cycle TEXT NOT NULL,
                    started_at INTEGER NOT NULL,
                    current_period_start INTEGER NOT NULL,
                    current_period_end INTEGER NOT NULL,
                    trial_end INTEGER,
                    cancel_at_period_end BOOLEAN NOT NULL,
                    flux_paid_total INTEGER NOT NULL,
                    next_billing_epoch INTEGER NOT NULL,
                    auto_renew BOOLEAN NOT NULL,
                    metadata TEXT,
                    FOREIGN KEY (user_id) REFERENCES users(user_id),
                    FOREIGN KEY (agent_id) REFERENCES agents(agent_id)
                )
            "#,
            _ => "",
        }
    }
    
    /// Get all CREATE TABLE statements
    pub fn get_all_create_tables(&self) -> Vec<(&'static str, &'static str)> {
        self.tables.iter()
            .map(|t| (*t, self.get_create_table(t)))
            .filter(|(_, sql)| !sql.is_empty())
            .collect()
    }
    
    /// Get migration script for version upgrade
    pub fn get_migration(&self, from_version: u32, to_version: u32) -> Option<&'static str> {
        if from_version >= to_version {
            return None;
        }
        
        match (from_version, to_version) {
            (0, 1) => Some(r#"
                -- Migration v0 -> v1: Initial schema
                -- All tables created fresh
            "#),
            _ => None,
        }
    }
}

impl Default for DatabaseSchema {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// REPLIT DB KEY PATTERNS
// ============================================================================

/// Replit DB key pattern helpers
pub mod db_keys {
    pub fn user(user_id: &str) -> String {
        format!("users/{}", user_id)
    }
    
    pub fn account(wallet_address: &str) -> String {
        format!("accounts/{}", wallet_address)
    }
    
    pub fn agent(agent_id: &str) -> String {
        format!("agents/{}", agent_id)
    }
    
    pub fn device(device_id: &str) -> String {
        format!("devices/{}", device_id)
    }
    
    pub fn stake(stake_id: &str) -> String {
        format!("stakes/{}", stake_id)
    }
    
    pub fn subscription(subscription_id: &str) -> String {
        format!("subscriptions/{}", subscription_id)
    }
    
    pub fn invoice(invoice_id: &str) -> String {
        format!("invoices/{}", invoice_id)
    }
    
    pub fn task(task_id: &str) -> String {
        format!("tasks/{}", task_id)
    }
    
    pub fn proposal(proposal_id: &str) -> String {
        format!("proposals/{}", proposal_id)
    }
    
    pub fn notification(user_id: &str, notification_id: &str) -> String {
        format!("users/{}/notifications/{}", user_id, notification_id)
    }
    
    pub fn activity_log(user_id: &str, log_id: &str) -> String {
        format!("users/{}/activity/{}", user_id, log_id)
    }
    
    pub fn usage(user_id: &str, epoch: u64) -> String {
        format!("usage/{}/{}", user_id, epoch)
    }
    
    pub fn mining_rewards(device_id: &str, epoch: u64) -> String {
        format!("mining/{}/{}/rewards", device_id, epoch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_version() {
        let schema = DatabaseSchema::new();
        assert_eq!(schema.version, SCHEMA_VERSION);
    }
    
    #[test]
    fn test_tables_defined() {
        let schema = DatabaseSchema::new();
        assert!(schema.tables.len() >= 10);
        assert!(schema.tables.contains(&"users"));
        assert!(schema.tables.contains(&"stakes"));
        assert!(schema.tables.contains(&"subscriptions"));
    }
    
    #[test]
    fn test_db_keys() {
        assert_eq!(db_keys::user("user123"), "users/user123");
        assert_eq!(db_keys::subscription("sub456"), "subscriptions/sub456");
        assert_eq!(db_keys::usage("user1", 100), "usage/user1/100");
    }
}
