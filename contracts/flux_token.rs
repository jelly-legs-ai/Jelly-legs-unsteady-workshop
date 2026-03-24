// FLUX Token Contract - AeTHer Chain
// Enhanced implementation for FLUX utility token with burning, minting, and transfer tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FLUX Token Configuration
pub const FLUX_TOKEN_NAME: &str = "FLUX";
pub const FLUX_TOKEN_SYMBOL: &str = "FLUX";
pub const FLUX_TOKEN_DECIMALS: u8 = 18;
pub const FLUX_MAX_SUPPLY: u64 = 10_000_000_000_u64; // 10 billion FLUX
pub const FLUX_INITIAL_MINTED: u64 = 1_000_000_000_u64; // 1 billion initially minted

/// FLUX Token State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxToken {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub mining_reward_per_epoch: u64,
    pub last_reward_distribution: u64,
    pub burned_amount: u64,
    pub treasury_balance: u64,
    pub balances: HashMap<String, u64>,
    pub allowances: HashMap<String, HashMap<String, u64>>,
}

/// Transfer event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub epoch: u64,
    pub timestamp: u64,
    pub tx_hash: String,
}

/// Burn event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnEvent {
    pub burner: String,
    pub amount: u64,
    pub epoch: u64,
    pub timestamp: u64,
    pub reason: String,
}

/// Initialize FLUX token
pub fn init_flux_token() -> FluxToken {
    let mut token = FluxToken {
        total_supply: FLUX_INITIAL_MINTED,
        circulating_supply: FLUX_INITIAL_MINTED,
        mining_reward_per_epoch: 1000, // 1000 FLUX per epoch
        last_reward_distribution: 0,
        burned_amount: 0,
        treasury_balance: FLUX_MAX_SUPPLY - FLUX_INITIAL_MINTED,
        balances: HashMap::new(),
        allowances: HashMap::new(),
    };
    
    // Mint initial treasury allocation
    token.balances.insert("treasury".to_string(), token.treasury_balance);
    
    token
}

/// Mint new FLUX tokens (treasury only)
pub fn mint_flux(token: &mut FluxToken, amount: u64, recipient: String) -> Result<u64, String> {
    if token.total_supply + amount > FLUX_MAX_SUPPLY {
        return Err("Minting would exceed max supply".to_string());
    }
    
    if token.treasury_balance < amount {
        return Err("Insufficient treasury balance".to_string());
    }
    
    token.total_supply += amount;
    token.circulating_supply += amount;
    token.treasury_balance -= amount;
    
    *token.balances.entry(recipient.clone()).or_insert(0) += amount;
    
    Ok(amount)
}

/// Burn FLUX tokens
pub fn burn_flux(token: &mut FluxToken, owner: String, amount: u64, reason: String) -> Result<BurnEvent, String> {
    let balance = token.balances.get(&owner).copied().unwrap_or(0);
    
    if balance < amount {
        return Err("Insufficient balance to burn".to_string());
    }
    
    token.balances.insert(owner.clone(), balance - amount);
    token.circulating_supply -= amount;
    token.burned_amount += amount;
    
    let burn_event = BurnEvent {
        burner: owner,
        amount,
        epoch: token.last_reward_distribution,
        timestamp: token.last_reward_distribution * 3600,
        reason,
    };
    
    Ok(burn_event)
}

/// Transfer FLUX tokens
pub fn transfer_flux(token: &mut FluxToken, from: String, to: String, amount: u64) -> Result<TransferEvent, String> {
    let from_balance = token.balances.get(&from).copied().unwrap_or(0);
    
    if from_balance < amount {
        return Err("Insufficient balance".to_string());
    }
    
    token.balances.insert(from.clone(), from_balance - amount);
    *token.balances.entry(to.clone()).or_insert(0) += amount;
    
    let transfer_event = TransferEvent {
        from,
        to,
        amount,
        epoch: token.last_reward_distribution,
        timestamp: token.last_reward_distribution * 3600,
        tx_hash: format!("tx_{}_{}", from, to),
    };
    
    Ok(transfer_event)
}

/// Approve spender to transfer tokens on behalf of owner
pub fn approve_flux(token: &mut FluxToken, owner: String, spender: String, amount: u64) {
    token.allowances
        .entry(owner.clone())
        .or_insert_with(HashMap::new)
        .insert(spender, amount);
}

/// Transfer from (using allowance)
pub fn transfer_from_flux(token: &mut FluxToken, spender: String, from: String, to: String, amount: u64) -> Result<TransferEvent, String> {
    let owner_allowances = token.allowances.get(&from);
    
    match owner_allowances {
        Some(allowances) => {
            let spender_allowance = allowances.get(&spender).copied().unwrap_or(0);
            
            if spender_allowance < amount {
                return Err("Allowance exceeded".to_string());
            }
            
            // Reduce allowance
            token.allowances.get_mut(&from).unwrap().insert(spender, spender_allowance - amount);
            
            // Execute transfer
            transfer_flux(token, from, to, amount)
        }
        None => Err("No allowance set".to_string()),
    }
}

/// Calculate mining reward based on epoch and device tier
pub fn calculate_mining_reward(
    epoch: u64,
    device_tier: DeviceTier,
    uptime_hours: u64,
    network_participation: f64,
) -> u64 {
    let base_reward = 1000_u64;
    let tier_multiplier = match device_tier {
        DeviceTier::Mobile => 1.0,
        DeviceTier::Laptop => 1.5,
        DeviceTier::Desktop => 2.0,
        DeviceTier::Server => 3.0,
    };
    let uptime_factor = (uptime_hours as f64 / 24.0).min(1.0);
    let participation_factor = network_participation.min(1.0);
    
    (base_reward as f64 * tier_multiplier * uptime_factor * participation_factor) as u64
}

/// Device tier for mining rewards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DeviceTier {
    Mobile,
    Laptop,
    Desktop,
    Server,
}

/// Reward distribution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    pub epoch: u64,
    pub recipient: String,
    pub amount: u64,
    pub device_tier: DeviceTier,
    pub timestamp: u64,
}

/// Distribute mining rewards for an epoch
pub fn distribute_epoch_rewards(
    token: &mut FluxToken,
    epoch: u64,
    recipients: Vec<(String, DeviceTier, u64, f64)>, // (address, tier, uptime, participation)
) -> Vec<RewardDistribution> {
    let mut distributions = Vec::new();
    
    for (recipient, tier, uptime, participation) in recipients {
        let reward = calculate_mining_reward(epoch, tier, uptime, participation);
        
        if reward > 0 {
            token.circulating_supply += reward;
            *token.balances.entry(recipient.clone()).or_insert(0) += reward;
            
            distributions.push(RewardDistribution {
                epoch,
                recipient,
                amount: reward,
                device_tier: tier,
                timestamp: epoch * 3600, // epoch duration in seconds
            });
        }
    }
    
    token.last_reward_distribution = epoch;
    distributions
}

/// Get token holder count
pub fn get_holder_count(token: &FluxToken) -> u64 {
    token.balances.iter().filter(|(_, balance)| **balance > 0).count() as u64
}

/// Get top token holders
pub fn get_top_holders(token: &FluxToken, limit: usize) -> Vec<(String, u64)> {
    let mut holders: Vec<_> = token.balances.iter()
        .filter(|(_, balance)| **balance > 0)
        .collect();
    
    holders.sort_by(|a, b| b.1.cmp(a.1));
    holders.truncate(limit);
    
    holders.into_iter().map(|(addr, bal)| (addr.clone(), *bal)).collect()
}

/// Get token metrics
pub fn get_flux_metrics(token: &FluxToken) -> FluxMetrics {
    FluxMetrics {
        total_supply: token.total_supply,
        circulating_supply: token.circulating_supply,
        burned_amount: token.burned_amount,
        treasury_balance: token.treasury_balance,
        holder_count: get_holder_count(token),
        top_10_holders: get_top_holders(token, 10),
    }
}

/// Token metrics struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxMetrics {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_amount: u64,
    pub treasury_balance: u64,
    pub holder_count: u64,
    pub top_10_holders: Vec<(String, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flux_token_init() {
        let token = init_flux_token();
        assert_eq!(token.total_supply, FLUX_INITIAL_MINTED);
        assert_eq!(token.treasury_balance, FLUX_MAX_SUPPLY - FLUX_INITIAL_MINTED);
    }

    #[test]
    fn test_flux_mint() {
        let mut token = init_flux_token();
        let minted = mint_flux(&mut token, 100_000, "user1".to_string()).unwrap();
        assert_eq!(minted, 100_000);
        assert_eq!(*token.balances.get("user1").unwrap(), 100_000);
    }

    #[test]
    fn test_flux_burn() {
        let mut token = init_flux_token();
        mint_flux(&mut token, 100_000, "user1".to_string()).unwrap();
        
        let burn = burn_flux(&mut token, "user1".to_string(), 50_000, "test burn".to_string()).unwrap();
        assert_eq!(burn.amount, 50_000);
        assert_eq!(token.burned_amount, 50_000);
    }

    #[test]
    fn test_flux_transfer() {
        let mut token = init_flux_token();
        mint_flux(&mut token, 100_000, "user1".to_string()).unwrap();
        
        let transfer = transfer_flux(&mut token, "user1".to_string(), "user2".to_string(), 25_000).unwrap();
        assert_eq!(transfer.amount, 25_000);
        assert_eq!(*token.balances.get("user2").unwrap(), 25_000);
    }

    #[test]
    fn test_flux_approve_and_transfer_from() {
        let mut token = init_flux_token();
        mint_flux(&mut token, 100_000, "user1".to_string()).unwrap();
        
        approve_flux(&mut token, "user1".to_string(), "spender".to_string(), 50_000);
        
        let transfer = transfer_from_flux(&mut token, "spender".to_string(), "user1".to_string(), "user2".to_string(), 30_000).unwrap();
        assert_eq!(transfer.amount, 30_000);
    }

    #[test]
    fn test_holder_count() {
        let mut token = init_flux_token();
        mint_flux(&mut token, 100_000, "user1".to_string()).unwrap();
        mint_flux(&mut token, 50_000, "user2".to_string()).unwrap();
        
        assert_eq!(get_holder_count(&token), 3); // treasury + user1 + user2
    }
}irculating_supply += reward;
        
        distributions.push(RewardDistribution {
            epoch,
            recipient,
            amount: reward,
            device_tier: tier,
            timestamp: epoch * 3600, // epoch duration in seconds
        });
    }
    
    token.last_reward_distribution = epoch;
    distributions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flux_token_init() {
        let token = init_flux_token();
        assert_eq!(token.total_supply, FLUX_MAX_SUPPLY);
        assert_eq!(token.circulating_supply, 0);
    }

    #[test]
    fn test_mobile_mining_reward() {
        let reward = calculate_mining_reward(1, DeviceTier::Mobile, 24, 1.0);
        assert_eq!(reward, 1000);
    }

    #[test]
    fn test_laptop_mining_reward() {
        let reward = calculate_mining_reward(1, DeviceTier::Laptop, 24, 1.0);
        assert_eq!(reward, 1500);
    }

    #[test]
    fn test_desktop_mining_reward() {
        let reward = calculate_mining_reward(1, DeviceTier::Desktop, 24, 1.0);
        assert_eq!(reward, 2000);
    }

    #[test]
    fn test_partial_uptime_reward() {
        let reward = calculate_mining_reward(1, DeviceTier::Mobile, 12, 1.0);
        assert_eq!(reward, 500); // 50% uptime = 50% reward
    }
}
