// Cross-Chain Bridge Contract for AeTHer Chain
// Enables asset transfers between AeTHer and other major blockchains

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Bridge transaction status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BridgeStatus {
    Pending,
    Confirming,
    Confirmed,
    Executing,
    Completed,
    Failed,
    Refunded,
}

// Bridge transaction priority
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BridgePriority {
    Standard,
    Fast,
    Instant,
}

impl BridgePriority {
    pub fn multiplier(&self) -> f64 {
        match self {
            BridgePriority::Standard => 1.0,
            BridgePriority::Fast => 1.5,
            BridgePriority::Instant => 3.0,
        }
    }
    
    pub fn estimated_time_minutes(&self) -> u32 {
        match self {
            BridgePriority::Standard => 30,
            BridgePriority::Fast => 10,
            BridgePriority::Instant => 2,
        }
    }
}

// Bridge transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub id: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub sender: String,
    pub recipient: String,
    pub token: String,
    pub amount: u64,
    pub fee: u64,
    pub priority: BridgePriority,
    pub status: BridgeStatus,
    pub timestamp: u64,
    pub confirmations: u32,
    pub required_confirmations: u32,
    pub destination_tx: Option<String>,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub refund_address: Option<String>,
    pub metadata: HashMap<String, String>,
}

// Bridge pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgePool {
    pub chain: String,
    pub token: String,
    pub total_liquidity: u64,
    pub available_liquidity: u64,
    pub locked_liquidity: u64,
    pub utilization_rate: f64,
    pub daily_volume: u64,
    pub weekly_volume: u64,
    pub total_volume: u64,
    pub last_update: u64,
    pub liquidity_providers: Vec<String>,
    pub apr: f64,
}

// Liquidity provider share
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityProvider {
    pub address: String,
    pub pool_id: String,
    pub contributed_amount: u64,
    pub share_percent: f64,
    pub rewards_earned: u64,
    pub pending_rewards: u64,
    pub joined_at: u64,
    pub last_claim: u64,
}

// Bridge analytics and stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    pub total_transactions: u64,
    pub total_volume: u64,
    pub total_fees_collected: u64,
    pub avg_transfer_time_minutes: f64,
    pub success_rate: f64,
    pub top_source_chain: String,
    pub top_destination_chain: String,
    pub top_token: String,
}

// Cross-chain bridge state
pub struct CrossChainBridge {
    pub transactions: HashMap<String, BridgeTransaction>,
    pub pools: HashMap<String, BridgePool>,
    pub supported_chains: Vec<String>,
    pub min_transfer_amount: HashMap<String, u64>,
    pub max_transfer_amount: HashMap<String, u64>,
    pub fees: HashMap<String, f64>,
    pub confirmations_required: HashMap<String, u32>,
}

impl CrossChainBridge {
    pub fn new() -> Self {
        let mut bridge = CrossChainBridge {
            transactions: HashMap::new(),
            pools: HashMap::new(),
            supported_chains: vec![
                "Solana".to_string(),
                "Ethereum".to_string(),
                "Polygon".to_string(),
                "Arbitrum".to_string(),
                "Optimism".to_string(),
            ],
            min_transfer_amount: HashMap::new(),
            max_transfer_amount: HashMap::new(),
            fees: HashMap::new(),
            confirmations_required: HashMap::new(),
        };
        
        // Initialize defaults
        bridge.min_transfer_amount.insert("Solana".to_string(), 10);
        bridge.min_transfer_amount.insert("Ethereum".to_string(), 50);
        bridge.min_transfer_amount.insert("Polygon".to_string(), 5);
        bridge.min_transfer_amount.insert("Arbitrum".to_string(), 10);
        bridge.min_transfer_amount.insert("Optimism".to_string(), 10);
        
        bridge.max_transfer_amount.insert("Solana".to_string(), 1_000_000);
        bridge.max_transfer_amount.insert("Ethereum".to_string(), 10_000_000);
        bridge.max_transfer_amount.insert("Polygon".to_string(), 5_000_000);
        bridge.max_transfer_amount.insert("Arbitrum".to_string(), 5_000_000);
        bridge.max_transfer_amount.insert("Optimism".to_string(), 5_000_000);
        
        // Bridge fees (in basis points, e.g., 50 = 0.5%)
        bridge.fees.insert("Solana".to_string(), 0.003); // 0.3%
        bridge.fees.insert("Ethereum".to_string(), 0.005); // 0.5%
        bridge.fees.insert("Polygon".to_string(), 0.001); // 0.1%
        bridge.fees.insert("Arbitrum".to_string(), 0.002); // 0.2%
        bridge.fees.insert("Optimism".to_string(), 0.002); // 0.2%
        
        // Required confirmations for each chain
        bridge.confirmations_required.insert("Solana".to_string(), 31); // ~1 block
        bridge.confirmations_required.insert("Ethereum".to_string(), 12); // ~2 min
        bridge.confirmations_required.insert("Polygon".to_string(), 128); // ~7 min
        bridge.confirmations_required.insert("Arbitrum".to_string(), 1); // ~1 block
        bridge.confirmations_required.insert("Optimism".to_string(), 1); // ~1 block
        
        // Initialize bridge pools
        let tokens = vec!["AETH", "FLUX"];
        let chains = vec!["Solana", "Ethereum", "Polygon", "Arbitrum", "Optimism"];
        
        for chain in chains {
            for token in &tokens {
                let pool_id = format!("{}_{}", chain, token);
                bridge.pools.insert(pool_id.clone(), BridgePool {
                    chain: chain.to_string(),
                    token: token.to_string(),
                    total_liquidity: 0,
                    available_liquidity: 0,
                    locked_liquidity: 0,
                    utilization_rate: 0.0,
                    daily_volume: 0,
                    last_update: 0,
                });
            }
        }
        
        bridge
    }

    // Initiate a new bridge transfer
    pub fn initiate_transfer(
        &mut self,
        source_chain: String,
        destination_chain: String,
        sender: String,
        recipient: String,
        token: String,
        amount: u64,
    ) -> Result<BridgeTransaction, String> {
        // Validate chains
        if !self.supported_chains.contains(&source_chain) {
            return Err(format!("Source chain {} not supported", source_chain));
        }
        if !self.supported_chains.contains(&destination_chain) {
            return Err(format!("Destination chain {} not supported", destination_chain));
        }
        
        if source_chain == destination_chain {
            return Err("Source and destination chains must be different".to_string());
        }
        
        // Validate amount
        let min_amount = self.min_transfer_amount.get(&destination_chain)
            .ok_or("Unknown destination chain")?;
        let max_amount = self.max_transfer_amount.get(&destination_chain)
            .ok_or("Unknown destination chain")?;
        
        if amount < *min_amount {
            return Err(format!("Amount {} below minimum {}", amount, min_amount));
        }
        if amount > *max_amount {
            return Err(format!("Amount {} exceeds maximum {}", amount, max_amount));
        }
        
        // Calculate fee
        let fee_rate = self.fees.get(&destination_chain)
            .ok_or("Unknown destination chain")?;
        let fee = (amount as f64 * fee_rate) as u64;
        let net_amount = amount - fee;
        
        // Generate transaction ID
        let tx_id = format!("bridge_{}_{}_{}", source_chain, destination_chain, 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());
        
        let tx = BridgeTransaction {
            id: tx_id.clone(),
            source_chain,
            destination_chain,
            sender,
            recipient,
            token,
            amount: net_amount,
            fee,
            status: BridgeStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            confirmations: 0,
            destination_tx: None,
        };
        
        self.transactions.insert(tx_id.clone(), tx.clone());
        
        // Lock liquidity in pool
        let pool_id = format!("{}_{}", tx.destination_chain, tx.token);
        if let Some(pool) = self.pools.get_mut(&pool_id) {
            pool.locked_liquidity += net_amount;
            pool.available_liquidity = pool.total_liquidity - pool.locked_liquidity;
            pool.utilization_rate = pool.locked_liquidity as f64 / pool.total_liquidity as f64;
        }
        
        Ok(tx)
    }

    // Confirm a bridge transaction (called by relayers)
    pub fn confirm_transaction(&mut self, tx_id: &str, conf_count: u32) -> Result<(), String> {
        let tx = self.transactions.get_mut(tx_id)
            .ok_or("Transaction not found")?;
        
        let required = self.confirmations_required.get(&tx.destination_chain)
            .ok_or("Unknown destination chain")?;
        
        tx.confirmations = conf_count;
        
        if conf_count >= *required && tx.status == BridgeStatus::Pending {
            tx.status = BridgeStatus::Confirmed;
        }
        
        Ok(())
    }

    // Complete a bridge transaction
    pub fn complete_transaction(&mut self, tx_id: &str, destination_tx: String) -> Result<(), String> {
        let tx = self.transactions.get_mut(tx_id)
            .ok_or("Transaction not found")?;
        
        if tx.status != BridgeStatus::Confirmed {
            return Err(format!("Transaction not in confirmed state: {:?}", tx.status));
        }
        
        tx.status = BridgeStatus::Completed;
        tx.destination_tx = Some(destination_tx);
        
        // Update pool
        let pool_id = format!("{}_{}", tx.destination_chain, tx.token);
        if let Some(pool) = self.pools.get_mut(&pool_id) {
            pool.locked_liquidity -= tx.amount;
            pool.available_liquidity = pool.total_liquidity - pool.locked_liquidity;
            pool.utilization_rate = if pool.total_liquidity > 0 {
                pool.locked_liquidity as f64 / pool.total_liquidity as f64
            } else {
                0.0
            };
            pool.daily_volume += tx.amount;
        }
        
        Ok(())
    }

    // Get bridge statistics
    pub fn get_bridge_stats(&self) -> BridgeStats {
        let total_txs = self.transactions.len();
        let pending_txs = self.transactions.values()
            .filter(|tx| tx.status == BridgeStatus::Pending || tx.status == BridgeStatus::Confirmed)
            .count();
        let completed_txs = self.transactions.values()
            .filter(|tx| tx.status == BridgeStatus::Completed)
            .count();
        
        let total_volume: u64 = self.transactions.values()
            .filter(|tx| tx.status == BridgeStatus::Completed)
            .map(|tx| tx.amount)
            .sum();
        
        let mut chain_volumes: HashMap<String, u64> = HashMap::new();
        for tx in self.transactions.values() {
            if tx.status == BridgeStatus::Completed {
                *chain_volumes.entry(tx.destination_chain.clone()).or_insert(0) += tx.amount;
            }
        }
        
        BridgeStats {
            total_transactions: total_txs,
            pending_transactions: pending_txs,
            completed_transactions: completed_txs,
            total_volume_bridged: total_volume,
            chain_volumes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BridgeStats {
    pub total_transactions: usize,
    pub pending_transactions: usize,
    pub completed_transactions: usize,
    pub total_volume_bridged: u64,
    pub chain_volumes: HashMap<String, u64>,
}

impl Default for CrossChainBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_transfer() {
        let mut bridge = CrossChainBridge::new();
        
        let result = bridge.initiate_transfer(
            "Solana".to_string(),
            "Ethereum".to_string(),
            "sender123".to_string(),
            "recipient456".to_string(),
            "FLUX".to_string(),
            1000,
        );
        
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.status, BridgeStatus::Pending);
        assert_eq!(tx.amount, 995); // 1000 - 0.5% fee
    }

    #[test]
    fn test_invalid_chain() {
        let mut bridge = CrossChainBridge::new();
        
        let result = bridge.initiate_transfer(
            "Bitcoin".to_string(),
            "Ethereum".to_string(),
            "sender123".to_string(),
            "recipient456".to_string(),
            "FLUX".to_string(),
            1000,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_bridge_completion() {
        let mut bridge = CrossChainBridge::new();
        
        let tx = bridge.initiate_transfer(
            "Solana".to_string(),
            "Ethereum".to_string(),
            "sender123".to_string(),
            "recipient456".to_string(),
            "FLUX".to_string(),
            1000,
        ).unwrap();
        
        // Confirm transaction
        bridge.confirm_transaction(&tx.id, 12).unwrap();
        
        // Complete transaction
        bridge.complete_transaction(&tx.id, "0x123abc".to_string()).unwrap();
        
        let completed_tx = bridge.transactions.get(&tx.id).unwrap();
        assert_eq!(completed_tx.status, BridgeStatus::Completed);
        assert_eq!(completed_tx.destination_tx, Some("0x123abc".to_string()));
    }
}
