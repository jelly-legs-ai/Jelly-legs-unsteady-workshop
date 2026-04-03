// Liquidity Pool Contract for FLUX/ATH Trading Pairs
// Part of AeTHer Chain DeFi Suite

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Liquidity position for a user in a specific pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub pool_id: String,
    pub user_address: String,
    pub liquidity_tokens: u64,
    pub flux_amount: u64,
    pub ath_amount: u64,
    pub entry_price_flux: f64,
    pub entry_price_ath: f64,
    pub last_claim_time: u64,
    pub total_fees_earned: u64,
}

/// Pool statistics and reserves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    pub pool_id: String,
    pub reserve_flux: u64,
    pub reserve_ath: u64,
    pub total_liquidity_tokens: u64,
    pub current_price_flux_per_ath: f64,
    pub total_volume_24h: u64,
    pub total_fees_24h: u64,
    pub amp_coefficient: u64, // Amplification coefficient for stableswap
    pub fee_rate: u64,        // Basis points (e.g., 30 = 0.3%)
    pub admin_fee_rate: u64, // Basis points for protocol fee
}

/// Liquidity pool contract
pub struct LiquidityPoolContract {
    pools: Arc<RwLock<HashMap<String, PoolState>>>,
    positions: Arc<RwLock<HashMap<String, LiquidityPosition>>>,
    snapshots: Arc<RwLock<HashMap<String, Vec<PoolSnapshot>>>>,
}

/// Historical pool snapshot for charting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSnapshot {
    pub timestamp: u64,
    pub reserve_flux: u64,
    pub reserve_ath: u64,
    pub total_liquidity_tokens: u64,
    pub price_flux_per_ath: f64,
    pub volume_24h: u64,
}

impl LiquidityPoolContract {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize a new FLUX/ATH liquidity pool
    pub async fn initialize_pool(
        &self,
        pool_id: String,
        initial_flux: u64,
        initial_ath: u64,
        amp_coefficient: u64,
        fee_rate: u64,
        admin_fee_rate: u64,
    ) -> Result<PoolState, String> {
        if fee_rate > 1000 {
            return Err("Fee rate cannot exceed 10%".to_string());
        }
        if admin_fee_rate > 500 {
            return Err("Admin fee cannot exceed 5%".to_string());
        }

        let initial_price = if initial_ath > 0 {
            initial_flux as f64 / initial_ath as f64
        } else {
            1.0
        };

        let pool = PoolState {
            pool_id: pool_id.clone(),
            reserve_flux: initial_flux,
            reserve_ath: initial_ath,
            total_liquidity_tokens: (initial_flux * initial_ath).sqrt() as u64,
            current_price_flux_per_ath: initial_price,
            total_volume_24h: 0,
            total_fees_24h: 0,
            amp_coefficient,
            fee_rate,
            admin_fee_rate,
        };

        let mut pools = self.pools.write().await;
        pools.insert(pool_id.clone(), pool.clone());

        // Create initial snapshot
        let mut snapshots = self.snapshots.write().await;
        snapshots.insert(pool_id, vec![PoolSnapshot {
            timestamp: chrono::Utc::now().timestamp() as u64,
            reserve_flux: initial_flux,
            reserve_ath: initial_ath,
            total_liquidity_tokens: pool.total_liquidity_tokens,
            price_flux_per_ath: initial_price,
            volume_24h: 0,
        }]);

        Ok(pool)
    }

    /// Add liquidity to a pool
    pub async fn add_liquidity(
        &self,
        pool_id: String,
        user_address: String,
        flux_amount: u64,
        ath_amount: u64,
    ) -> Result<(u64, u64, u64), String> {
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(&pool_id)
            .ok_or("Pool not found")?;

        // Calculate minimum amounts to maintain price
        let min_flux = (flux_amount * 99) / 100;
        let min_ath = (ath_amount * 99) / 100;

        // Calculate liquidity tokens to mint
        let total_liquidity = pool.total_liquidity_tokens;
        let liquidity_tokens = if total_liquidity == 0 {
            (flux_amount * ath_amount).sqrt() as u64
        } else {
            let flux_per_token = pool.reserve_flux as f64 / total_liquidity as f64;
            let ath_per_token = pool.reserve_ath as f64 / total_liquidity as f64;
            ((flux_amount as f64 / flux_per_token) + (ath_amount as f64 / ath_per_token)) as u64 / 2
        };

        // Update reserves
        pool.reserve_flux += flux_amount;
        pool.reserve_ath += ath_amount;
        pool.total_liquidity_tokens += liquidity_tokens;

        // Update user position
        let mut positions = self.positions.write().await;
        let pos_key = format!("{}:{}", pool_id, user_address);
        
        let position = positions.entry(pos_key.clone()).or_insert(LiquidityPosition {
            pool_id: pool_id.clone(),
            user_address: user_address.clone(),
            liquidity_tokens: 0,
            flux_amount: 0,
            ath_amount: 0,
            entry_price_flux: 0.0,
            entry_price_ath: 0.0,
            last_claim_time: chrono::Utc::now().timestamp() as u64,
            total_fees_earned: 0,
        });

        position.liquidity_tokens += liquidity_tokens;
        position.flux_amount += flux_amount;
        position.ath_amount += ath_amount;
        position.entry_price_flux = pool.reserve_flux as f64 / pool.reserve_ath as f64;
        position.entry_price_ath = pool.reserve_ath as f64 / pool.reserve_flux as f64;

        Ok((liquidity_tokens, min_flux, min_ath))
    }

    /// Remove liquidity from a pool
    pub async fn remove_liquidity(
        &self,
        pool_id: String,
        user_address: String,
        liquidity_tokens: u64,
    ) -> Result<(u64, u64), String> {
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(&pool_id)
            .ok_or("Pool not found")?;

        let mut positions = self.positions.write().await;
        let pos_key = format!("{}:{}", pool_id, user_address);
        let position = positions.get_mut(&pos_key)
            .ok_or("Position not found")?;

        if position.liquidity_tokens < liquidity_tokens {
            return Err("Insufficient liquidity tokens".to_string());
        }

        // Calculate proportional amounts
        let ratio = liquidity_tokens as f64 / pool.total_liquidity_tokens as f64;
        let flux_out = (pool.reserve_flux as f64 * ratio) as u64;
        let ath_out = (pool.reserve_ath as f64 * ratio) as u64;

        // Update pool
        pool.reserve_flux -= flux_out;
        pool.reserve_ath -= ath_out;
        pool.total_liquidity_tokens -= liquidity_tokens;

        // Update position
        position.liquidity_tokens -= liquidity_tokens;
        position.flux_amount -= flux_out;
        position.ath_amount -= ath_out;

        Ok((flux_out, ath_out))
    }

    /// Execute a swap (flux to ath or ath to flux)
    pub async fn swap(
        &self,
        pool_id: String,
        user_address: String,
        token_in: String, // "flux" or "ath"
        amount_in: u64,
    ) -> Result<u64, String> {
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(&pool_id)
            .ok_or("Pool not found")?;

        let (reserve_in, reserve_out) = if token_in == "flux" {
            (pool.reserve_flux, pool.reserve_ath)
        } else {
            (pool.reserve_ath, pool.reserve_flux)
        };

        // Calculate amount out with fee
        let fee = (amount_in * pool.fee_rate as u64) / 10000;
        let amount_in_after_fee = amount_in - fee;
        
        // Constant product formula: x * y = k
        let amount_out = (amount_in_after_fee * reserve_out) / (reserve_in + amount_in_after_fee);

        // Update reserves
        if token_in == "flux" {
            pool.reserve_flux += amount_in;
            pool.reserve_ath -= amount_out;
        } else {
            pool.reserve_ath += amount_in;
            pool.reserve_flux -= amount_out;
        }

        // Update pool volume
        pool.total_volume_24h += amount_in;
        pool.total_fees_24h += fee;

        // Update price
        pool.current_price_flux_per_ath = pool.reserve_flux as f64 / pool.reserve_ath as f64;

        Ok(amount_out)
    }

    /// Get pool state
    pub async fn get_pool(&self, pool_id: String) -> Option<PoolState> {
        let pools = self.pools.read().await;
        pools.get(&pool_id).cloned()
    }

    /// Get user position
    pub async fn get_position(&self, pool_id: String, user_address: String) -> Option<LiquidityPosition> {
        let positions = self.positions.read().await;
        positions.get(&format!("{}:{}", pool_id, user_address)).cloned()
    }

    /// Get historical snapshots for charting
    pub async fn get_pool_history(&self, pool_id: String, limit: usize) -> Vec<PoolSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.get(&pool_id)
            .map(|s| s.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default()
    }

    /// Calculate impermanent loss for a position
    pub async fn calculate_impermanent_loss(
        &self,
        pool_id: String,
        user_address: String,
    ) -> Result<f64, String> {
        let pools = self.pools.read().await;
        let positions = self.positions.read().await;
        
        let pool = pools.get(&pool_id).ok_or("Pool not found")?;
        let pos_key = format!("{}:{}", pool_id, user_address);
        let position = positions.get(&pos_key).ok_or("Position not found")?;

        // IL = 2 * sqrt(k) / (k + 1) - 1 where k = price_ratio
        let current_price = pool.current_price_flux_per_ath;
        let entry_price = position.entry_price_flux;
        let price_ratio = current_price / entry_price;
        
        let il = 2.0 * (price_ratio.sqrt()) / (price_ratio + 1.0) - 1.0;
        
        Ok(il * 100.0) // Return as percentage
    }

    /// Compound accumulated fees
    pub async fn claim_fees(&self, pool_id: String, user_address: String) -> Result<u64, String> {
        let mut positions = self.positions.write().await;
        let pos_key = format!("{}:{}", pool_id, user_address);
        let position = positions.get_mut(&pos_key)
            .ok_or("Position not found")?;

        let unclaimed_fees = position.total_fees_earned;
        position.last_claim_time = chrono::Utc::now().timestamp() as u64;
        
        Ok(unclaimed_fees)
    }
}

impl Default for LiquidityPoolContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_pool() {
        let contract = LiquidityPoolContract::new();
        let result = contract.initialize_pool(
            "FLUX-ATH-01".to_string(),
            1_000_000,
            1_000_000,
            100,
            30,
            10,
        ).await;
        
        assert!(result.is_ok());
        let pool = result.unwrap();
        assert_eq!(pool.reserve_flux, 1_000_000);
        assert_eq!(pool.reserve_ath, 1_000_000);
    }

    #[tokio::test]
    async fn test_add_liquidity() {
        let contract = LiquidityPoolContract::new();
        contract.initialize_pool(
            "FLUX-ATH-01".to_string(),
            1_000_000,
            1_000_000,
            100,
            30,
            10,
        ).await.unwrap();

        let result = contract.add_liquidity(
            "FLUX-ATH-01".to_string(),
            "user123".to_string(),
            100_000,
            100_000,
        ).await;

        assert!(result.is_ok());
        let (tokens, _, _) = result.unwrap();
        assert!(tokens > 0);
    }

    #[tokio::test]
    async fn test_swap() {
        let contract = LiquidityPoolContract::new();
        contract.initialize_pool(
            "FLUX-ATH-01".to_string(),
            1_000_000,
            1_000_000,
            100,
            30,
            10,
        ).await.unwrap();

        let result = contract.swap(
            "FLUX-ATH-01".to_string(),
            "user123".to_string(),
            "flux".to_string(),
            10_000,
        ).await;

        assert!(result.is_ok());
        let amount_out = result.unwrap();
        assert!(amount_out > 0);
    }
}
