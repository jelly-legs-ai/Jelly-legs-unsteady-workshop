// API Routes - AeTHer Chain Backend
// User and Agent Management API endpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::contracts::staking_contract::StakingContract;
use crate::contracts::mining_contract::MiningContract;

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub address: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: u64,
    pub last_active: u64,
    pub reputation_score: f64,
    pub kyc_verified: bool,
    pub total_agents: u64,
    pub total_staked: u64,
    pub total_mined: u64,
}

/// Agent registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistrationRequest {
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub avatar: Option<String>,
    pub owner_address: String,
    pub stake_amount: u64,
}

/// Agent registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistrationResponse {
    pub success: bool,
    pub agent_id: String,
    pub transaction_hash: Option<String>,
    pub message: String,
}

/// User dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDashboard {
    pub profile: UserProfile,
    pub agents: Vec<AgentSummary>,
    pub staking_positions: Vec<StakingPosition>,
    pub mining_rewards: u64,
    pub total_earnings: u64,
    pub pending_actions: Vec<PendingAction>,
}

/// Agent summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    pub earnings_24h: u64,
    pub uptime_percent: f64,
    pub last_active: u64,
}

/// Staking position summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    pub pool_name: String,
    pub amount_staked: u64,
    pub rewards_earned: u64,
    pub apy: f64,
    pub lock_end_epoch: Option<u64>,
}

/// Pending action for user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    pub action_type: String,
    pub description: String,
    pub priority: String,
    pub created_at: u64,
    pub deadline: Option<u64>,
}

/// API Router for user and agent management
pub struct ApiRouter {
    pub users: HashMap<String, UserProfile>,
    pub pending_agents: Vec<AgentRegistrationRequest>,
    pub staking_contract: StakingContract,
    pub mining_contract: MiningContract,
}

impl ApiRouter {
    /// Create new API router
    pub fn new() -> Self {
        ApiRouter {
            users: HashMap::new(),
            pending_agents: Vec::new(),
            staking_contract: StakingContract::new(),
            mining_contract: MiningContract::new(),
        }
    }

    /// Register a new user
    pub fn register_user(&mut self, address: String, username: String, email: Option<String>) -> Result<UserProfile, String> {
        if self.users.contains_key(&address) {
            return Err("User already registered".to_string());
        }

        let profile = UserProfile {
            address: address.clone(),
            username,
            email,
            created_at: self.mining_contract.current_epoch,
            last_active: self.mining_contract.current_epoch,
            reputation_score: 50.0,
            kyc_verified: false,
            total_agents: 0,
            total_staked: 0,
            total_mined: 0,
        };

        self.users.insert(address, profile.clone());
        Ok(profile)
    }

    /// Get user dashboard
    pub fn get_user_dashboard(&self, address: &str) -> Result<UserDashboard, String> {
        let profile = self.users.get(address).ok_or("User not found")?;
        
        let agents = self.get_user_agents(address);
        let staking_positions = self.get_user_staking_positions(address);
        let mining_rewards = self.get_user_mining_rewards(address);
        
        Ok(UserDashboard {
            profile: profile.clone(),
            agents,
            staking_positions,
            mining_rewards,
            total_earnings: mining_rewards + staking_positions.iter().map(|s| s.rewards_earned).sum::<u64>(),
            pending_actions: self.get_pending_actions(address),
        })
    }

    /// Get user's registered agents
    fn get_user_agents(&self, address: &str) -> Vec<AgentSummary> {
        // In production, this would query the agent registry
        vec![
            AgentSummary {
                id: "agent_001".to_string(),
                name: "Code Auditor".to_string(),
                status: "verified".to_string(),
                earnings_24h: 150,
                uptime_percent: 98.5,
                last_active: self.mining_contract.current_epoch,
            },
            AgentSummary {
                id: "agent_002".to_string(),
                name: "Data Analyst".to_string(),
                status: "pending".to_string(),
                earnings_24h: 75,
                uptime_percent: 95.2,
                last_active: self.mining_contract.current_epoch - 2,
            },
        ]
    }

    /// Get user's staking positions
    fn get_user_staking_positions(&self, address: &str) -> Vec<StakingPosition> {
        let mut positions = Vec::new();
        
        if let Some(stakes) = self.staking_contract.stakes.get(address) {
            for stake in stakes {
                if let Some(pool) = self.staking_contract.pools.get(&format!("{}_staking", stake.token_type.to_string().to_lowercase())) {
                    positions.push(StakingPosition {
                        pool_name: pool.name.clone(),
                        amount_staked: stake.amount,
                        rewards_earned: stake.rewards_claimed,
                        apy: pool.reward_rate,
                        lock_end_epoch: if stake.is_locked { Some(stake.lock_end_epoch) } else { None },
                    });
                }
            }
        }
        
        positions
    }

    /// Get user's mining rewards
    fn get_user_mining_rewards(&self, address: &str) -> u64 {
        if let Some(miner) = self.mining_contract.miners.get(address) {
            miner.total_mined
        } else {
            0
        }
    }

    /// Get pending actions for user
    fn get_pending_actions(&self, address: &str) -> Vec<PendingAction> {
        let mut actions = Vec::new();
        
        // Check for pending agent approvals
        for agent in &self.pending_agents {
            if &agent.owner_address == address {
                actions.push(PendingAction {
                    action_type: "agent_approval".to_string(),
                    description: format!("Agent '{}' awaiting verification", agent.name),
                    priority: "medium".to_string(),
                    created_at: self.mining_contract.current_epoch,
                    deadline: Some(self.mining_contract.current_epoch + 10),
                });
            }
        }
        
        // Check for staking rewards to claim
        if let Some(stakes) = self.staking_contract.stakes.get(address) {
            for stake in stakes {
                if stake.rewards_claimed > 0 && stake.last_claim_epoch < self.staking_contract.current_epoch - 1 {
                    actions.push(PendingAction {
                        action_type: "claim_rewards".to_string(),
                        description: format!("Claim {} rewards from staking", stake.token_type.to_string()),
                        priority: "high".to_string(),
                        created_at: self.staking_contract.current_epoch,
                        deadline: Some(self.staking_contract.current_epoch + 5),
                    });
                }
            }
        }
        
        actions
    }

    /// Submit agent registration
    pub fn submit_agent_registration(&mut self, request: AgentRegistrationRequest) -> AgentRegistrationResponse {
        let agent_id = format!("agent_{}", self.pending_agents.len() + 1);
        
        self.pending_agents.push(request.clone());
        
        AgentRegistrationResponse {
            success: true,
            agent_id,
            transaction_hash: Some(format!("0x{}", (0..64).map(|_| (b'0'..=b'9').choose(&mut rand::thread_rng()).unwrap()).collect::<String>())),
            message: "Agent registration submitted for review".to_string(),
        }
    }

    /// Approve agent registration (admin function)
    pub fn approve_agent(&mut self, agent_id: &str) -> Result<(), String> {
        let idx = self.pending_agents.iter().position(|a| format!("agent_{}", self.pending_agents.iter().position(|x| x == a).unwrap() + 1) == agent_id);
        
        if let Some(index) = idx {
            self.pending_agents.remove(index);
            Ok(())
        } else {
            Err("Agent not found".to_string())
        }
    }

    /// Get all pending agent registrations
    pub fn get_pending_agents(&self) -> &Vec<AgentRegistrationRequest> {
        &self.pending_agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_registration() {
        let mut router = ApiRouter::new();
        let profile = router.register_user("0x1234".to_string(), "testuser".to_string(), Some("test@example.com".to_string()));
        assert!(profile.is_ok());
        assert_eq!(profile.unwrap().username, "testuser");
    }

    #[test]
    fn test_agent_registration() {
        let mut router = ApiRouter::new();
        let request = AgentRegistrationRequest {
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            capabilities: vec!["coding".to_string(), "testing".to_string()],
            avatar: Some("🤖".to_string()),
            owner_address: "0x1234".to_string(),
            stake_amount: 1000,
        };
        
        let response = router.submit_agent_registration(request);
        assert!(response.success);
        assert_eq!(response.agent_id, "agent_1");
    }
}
