// FLUX/ATH Token Pair Contract - AeTHer Chain
// Liquidity pool pair for FLUX-ATH trading with dynamic pricing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token pair constants
pub const FLUX_SYMBOL: &str = "FLUX";
pub const ATH_SYMBOL: &str = "ATH";
pub const PAIR_NAME: &str = "FLUX/ATH";
pub const FEE_TIER_STANDARD: u32 = 30; // 0.30%
pub const FEE_TIER_STABLE: u32 = 5;    // 0.05%

/// Token amounts for pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAmount {
    pub token: String,
    pub amount: u64,
    pub value_usd: f64,
}

/// Liquidity position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub id: String,
    pub owner: String,
    pub flux_amount: u64,
    pub ath_amount: u64,
    pub lp_tokens: u64,
    pub fee_tier: u32,
    pub created_epoch: u64,
    pub last_claim_epoch: u64,
    pub accumulated_fees: u64,
}

/// Swap quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub input_token: String,
    pub output_token: String,
    pub input_amount: u64,
    pub expected_output: u64,
    pub minimum_output: u64,
    pub price_impact: f64,
    pub route: Vec<String>,
    pub fee: u64,
}

/// Liquidity pool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPairPool {
    pub pair_name: String,
    pub token0: String,
    pub token1: String,
    pub reserve0: u64,
    pub reserve1: u64,
    pub total_lp_supply: u64,
    pub fee_tier: u32,
    pub k_last: u128,  // For AMM constant product
    pub active: bool,
    pub creator: String,
    pub created_epoch: u64,
    // Volume tracking
    pub volume_24h: u64,
    pub volume_7d: u64,
    pub fees_24h: u64,
    pub tx_count: u64,
    // Price tracking
    pub price0_last: u64,
    pub price1_last: u64,
    pub price_change_24h: f64,
}

impl TokenPairPool {
    /// Create new FLUX/ATH pair
    pub fn new(fee_tier: u32, creator: String, epoch: u64) -> Self {
        let (token0, token1) = if FLUX_SYMBOL < ATH_SYMBOL {
            (FLUX_SYMBOL.to_string(), ATH_SYMBOL.to_string())
        } else {
            (ATH_SYMBOL.to_string(), FLUX_SYMBOL.to_string())
        };
        
        TokenPairPool {
            pair_name: PAIR_NAME.to_string(),
            token0,
            token1,
            reserve0: 0,
            reserve1: 0,
            total_lp_supply: 0,
            fee_tier,
            k_last: 0,
            active: true,
            creator,
            created_epoch: epoch,
            volume_24h: 0,
            volume_7d: 0,
            fees_24h: 0,
            tx_count: 0,
            price0_last: 0,
            price1_last: 0,
            price_change_24h: 0.0,
        }
    }

    /// Add liquidity to pool
    pub fn add_liquidity(
        &mut self,
        amount0: u64,
        amount1: u64,
        lp_tokens: u64,
        user: String,
    ) -> Result<LiquidityPosition, String> {
        if !self.active {
            return Err("Pool is not active".to_string());
        }
        if amount0 == 0 || amount1 == 0 {
            return Err("Amounts must be greater than 0".to_string());
        }

        self.reserve0 += amount0;
        self.reserve1 += amount1;
        self.total_lp_supply += lp_tokens;
        self.k_last = (self.reserve0 as u128) * (self.reserve1 as u128);

        Ok(LiquidityPosition {
            id: format!("pos_{}_{}", user, self.tx_count),
            owner: user,
            flux_amount: amount0,
            ath_amount: amount1,
            lp_tokens,
            fee_tier: self.fee_tier,
            created_epoch: self.created_epoch,
            last_claim_epoch: self.created_epoch,
            accumulated_fees: 0,
        })
    }

    /// Calculate swap output using constant product formula
    pub fn get_swap_output(&self, input_amount: u64, input_is_token0: bool) -> (u64, u64) {
        if input_is_token0 {
            // Selling token0 (FLUX if FLUX is token0)
            let reserve_in = self.reserve0;
            let reserve_out = self.reserve1;
            let fee = self.fee_tier as u64;
            
            // With fee: input * 10000 / (10000 - fee)
            let input_with_fee = input_amount * (10000 - fee) / 10000;
            let new_reserve_in = reserve_in + input_with_fee;
            
            // x * y = k
            // (x + dx) * (y - dy) = k
            // y - dy = k / (x + dx)
            // dy = y - k / (x + dx)
            let k = self.k_last as u64;
            if k == 0 || new_reserve_in == 0 {
                return (0, fee);
            }
            let output = reserve_out - (k / new_reserve_in);
            let fee_amount = input_amount - input_with_fee;
            
            (output, fee_amount)
        } else {
            // Selling token1 (ATH if ATH is token1)
            let reserve_in = self.reserve1;
            let reserve_out = self.reserve0;
            let fee = self.fee_tier as u64;
            
            let input_with_fee = input_amount * (10000 - fee) / 10000;
            let new_reserve_in = reserve_in + input_with_fee;
            
            let k = self.k_last as u64;
            if k == 0 || new_reserve_in == 0 {
                return (0, fee);
            }
            let output = reserve_out - (k / new_reserve_in);
            let fee_amount = input_amount - input_with_fee;
            
            (output, fee_amount)
        }
    }

    /// Execute swap
    pub fn swap(
        &mut self,
        input_token: &str,
        input_amount: u64,
        min_output: u64,
    ) -> Result<SwapQuote, String> {
        if !self.active {
            return Err("Pool is not active".to_string());
        }
        if input_amount == 0 {
            return Err("Input amount must be greater than 0".to_string());
        }

        let input_is_token0 = (input_token == self.token0) || 
            (input_token == FLUX_SYMBOL && self.token0 == FLUX_SYMBOL) ||
            (input_token == ATH_SYMBOL && self.token0 == ATH_SYMBOL);
        
        let (output_amount, fee) = self.get_swap_output(input_amount, input_is_token0);
        
        if output_amount < min_output {
            return Err("Slippage tolerance exceeded".to_string());
        }

        // Update reserves
        if input_is_token0 {
            self.reserve0 += input_amount;
            self.reserve1 -= output_amount;
        } else {
            self.reserve1 += input_amount;
            self.reserve0 -= output_amount;
        }

        // Update volume
        self.tx_count += 1;
        self.volume_24h += input_amount;
        self.fees_24h += fee;

        Ok(SwapQuote {
            input_token: input_token.to_string(),
            output_token: if input_is_token0 { 
                self.token1.clone() 
            } else { 
                self.token0.clone() 
            },
            input_amount,
            expected_output: output_amount,
            minimum_output: min_output,
            price_impact: (input_amount as f64) / (self.reserve0.max(1) as f64) * 100.0,
            route: vec![self.token0.clone(), self.token1.clone()],
            fee,
        })
    }

    /// Remove liquidity
    pub fn remove_liquidity(
        &mut self,
        lp_tokens: u64,
    ) -> Result<(u64, u64), String> {
        if !self.active {
            return Err("Pool is not active".to_string());
        }
        if lp_tokens == 0 || lp_tokens > self.total_lp_supply {
            return Err("Invalid LP token amount".to_string());
        }

        // Calculate proportional withdrawal
        let share = lp_tokens as f64 / self.total_lp_supply as f64;
        let amount0 = (self.reserve0 as f64 * share) as u64;
        let amount1 = (self.reserve1 as f64 * share) as u64;

        self.reserve0 -= amount0;
        self.reserve1 -= amount1;
        self.total_lp_supply -= lp_tokens;
        self.k_last = (self.reserve0 as u128) * (self.reserve1 as u128);

        Ok((amount0, amount1))
    }

    /// Get pool stats
    pub fn get_stats(&self) -> PoolStats {
        let tvl0 = self.reserve0;
        let tvl1 = self.reserve1;
        
        PoolStats {
            pair: self.pair_name.clone(),
            reserve0: self.reserve0,
            reserve1: self.reserve1,
            total_lp: self.total_lp_supply,
            tvl_usd: Self::estimate_tvl_usd(tvl0, tvl1),
            volume_24h: self.volume_24h,
            volume_7d: self.volume_7d,
            fees_24h: self.fees_24h,
            tx_count: self.tx_count,
            fee_tier: self.fee_tier,
            price0: self.price0_last,
            price1: self.price1_last,
            price_change_24h: self.price_change_24h,
            apr: Self::estimate_apr(self.volume_24h, self.fees_24h, self.total_lp_supply),
        }
    }

    fn estimate_tvl_usd(reserve0: u64, reserve1: u64) -> f64 {
        // Rough USD estimation
        let flux_price = 0.84;  // FLUX price in USD
        let ath_price = 12.45;  // ATH price in USD
        (reserve0 as f64 * flux_price) + (reserve1 as f64 * ath_price)
    }

    fn estimate_apr(volume_24h: u64, fees_24h: u64, total_lp: u64) -> f64 {
        if total_lp == 0 {
            return 0.0;
        }
        // Annualize fees and calculate APY
        let daily_fees = fees_24h as f64 * 0.01; // Convert to actual tokens
        let yearly_fees = daily_fees * 365.0;
        let lp_value = total_lp as f64 * 1.0; // Rough LP token value
        (yearly_fees / lp_value) * 100.0
    }
}

/// Pool statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub pair: String,
    pub reserve0: u64,
    pub reserve1: u64,
    pub total_lp: u64,
    pub tvl_usd: f64,
    pub volume_24h: u64,
    pub volume_7d: u64,
    pub fees_24h: u64,
    pub tx_count: u64,
    pub fee_tier: u32,
    pub price0: u64,
    pub price1: u64,
    pub price_change_24h: f64,
    pub apr: f64,
}

/// Initialize FLUX/ATH pair pool
pub fn init_flux_ath_pair(fee_tier: u32, creator: String, epoch: u64) -> TokenPairPool {
    TokenPairPool::new(fee_tier, creator, epoch)
}
