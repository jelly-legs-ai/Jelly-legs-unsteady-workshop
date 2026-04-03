// Token Trading Pair Module - AeTHer Chain
// FLUX/ATH liquidity pool and trading pair structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trading pair configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPair {
    pub pair_id: String,
    pub base_token: String,    // e.g., "FLUX"
    pub quote_token: String,  // e.g., "ATH"
    pub pool_address: String,
    pub is_active: bool,
    pub min_trade_amount: u64,
    pub max_trade_amount: u64,
    pub fee_percentage: f64,
    pub created_at_epoch: u64,
}

/// Liquidity position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub position_id: String,
    pub pair_id: String,
    pub owner: String,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub lp_tokens: u64,
    pub pool_share: f64,
    pub created_at_epoch: u64,
    pub last_claim_epoch: u64,
    pub accumulated_fees_base: u64,
    pub accumulated_fees_quote: u64,
}

/// Trade request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRequest {
    pub pair_id: String,
    pub trader: String,
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub is_buy: bool,  // true = buy base (sell quote), false = sell base (buy quote)
    pub slippage_tolerance: f64,
}

/// Trade execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub success: bool,
    pub pair_id: String,
    pub amount_in: u64,
    pub amount_out: u64,
    pub execution_price: f64,
    pub spot_price: f64,
    pub fee_paid: u64,
    pub pool_address: String,
    pub transaction_hash: Option<String>,
}

/// Liquidity pool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub pair_id: String,
    pub reserve_base: u64,
    pub reserve_quote: u64,
    pub total_lp_tokens: u64,
    pub annual_fee_revenue_base: u64,
    pub annual_fee_revenue_quote: u64,
    pub last_sync_epoch: u64,
    pub amp_coefficient: f64,  // For stableswap pools
}

/// Price oracle data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceOracle {
    pub pair_id: String,
    pub current_price: f64,
    pub price_24h_ago: f64,
    pub price_change_24h: f64,
    pub volume_24h: u64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub last_update_epoch: u64,
}

impl TradingPair {
    /// Calculate spot price for a pair
    pub fn spot_price(&self, reserve_base: u64, reserve_quote: u64, is_buy: bool) -> f64 {
        if reserve_base == 0 || reserve_quote == 0 {
            return 0.0;
        }
        if is_buy {
            reserve_quote as f64 / reserve_base as f64
        } else {
            reserve_base as f64 / reserve_quote as f64
        }
    }
    
    /// Validate trade amount against pair limits
    pub fn validate_trade_amount(&self, amount: u64) -> Result<(), String> {
        if amount < self.min_trade_amount {
            return Err(format!("Amount {} below minimum {}", amount, self.min_trade_amount));
        }
        if amount > self.max_trade_amount {
            return Err(format!("Amount {} above maximum {}", amount, self.max_trade_amount));
        }
        Ok(())
    }
}

/// AMM utilities for constant product market maker
pub mod amm {
    use super::*;
    
    /// Calculate output amount using constant product formula
    /// x * y = k
    /// Output = (input * reserve_out) / (reserve_in + input)
    pub fn calculate_output(
        reserve_in: u64,
        reserve_out: u64,
        amount_in: u64,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> u64 {
        if reserve_in == 0 || reserve_out == 0 {
            return 0;
        }
        
        // Apply fee
        let amount_in_with_fee = amount_in * fee_numerator / fee_denominator;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * fee_denominator / fee_numerator + amount_in_with_fee;
        
        numerator / denominator
    }
    
    /// Calculate input amount required for desired output
    /// Using inverse of constant product formula
    pub fn calculate_input_for_output(
        reserve_in: u64,
        reserve_out: u64,
        amount_out: u64,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> u64 {
        if reserve_in == 0 || reserve_out == 0 || amount_out >= reserve_out {
            return 0;
        }
        
        let numerator = reserve_in * amount_out * fee_denominator;
        let denominator = (reserve_out - amount_out) * fee_numerator;
        
        numerator / denominator + 1
    }
    
    /// Calculate liquidity provider tokens for new position
    pub fn calculate_lp_tokens(
        total_lp: u64,
        reserve_base: u64,
        reserve_quote: u64,
        amount_base: u64,
        amount_quote: u64,
    ) -> u64 {
        if total_lp == 0 {
            // Initial liquidity - sqrt(a * b)
            let product = (amount_base as u128 * amount_quote as u128) as u64;
            return sqrt(product);
        }
        
        let share_base = amount_base * total_lp / reserve_base;
        let share_quote = amount_quote * total_lp / reserve_quote;
        
        std::cmp::min(share_base, share_quote)
    }
    
    /// Square root approximation using Newton-Raphson
    pub fn sqrt(n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        
        let mut x = n;
        let mut y = (x + 1) / 2;
        
        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }
        
        x
    }
    
    /// Calculate pool share percentage
    pub fn calculate_pool_share(
        user_lp: u64,
        total_lp: u64,
    ) -> f64 {
        if total_lp == 0 {
            return 0.0;
        }
        (user_lp as f64 / total_lp as f64) * 100.0
    }
}

/// Calculate price impact for a trade
pub fn calculate_price_impact(
    reserve_in: u64,
    reserve_out: u64,
    amount_in: u64,
    is_buy: bool,
) -> f64 {
    let spot_before = if is_buy {
        reserve_out as f64 / reserve_in as f64
    } else {
        reserve_in as f64 / reserve_out as f64
    };
    
    let amount_out = amm::calculate_output(
        reserve_in,
        reserve_out,
        amount_in,
        997,  // 0.3% fee
        1000,
    );
    
    let spot_after = if is_buy {
        (reserve_out - amount_out) as f64 / (reserve_in + amount_in) as f64
    } else {
        (reserve_in + amount_in) as f64 / (reserve_out - amount_out) as f64
    };
    
    ((spot_before - spot_after) / spot_before * 100.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spot_price_calculation() {
        let pair = TradingPair {
            pair_id: "FLUX_ATH".to_string(),
            base_token: "FLUX".to_string(),
            quote_token: "ATH".to_string(),
            pool_address: "pool1".to_string(),
            is_active: true,
            min_trade_amount: 1000,
            max_trade_amount: 10_000_000_000,
            fee_percentage: 0.3,
            created_at_epoch: 0,
        };
        
        // Buy: 100 FLUX cost
        let price = pair.spot_price(1_000_000_000, 50_000_000, true);
        assert_eq!(price, 50.0); // 50 ATH per FLUX
        
        // Sell: 100 FLUX gets
        let price = pair.spot_price(1_000_000_000, 50_000_000, false);
        assert_eq!(price, 50.0);
    }
    
    #[test]
    fn test_amm_output_calculation() {
        let output = amm::calculate_output(1_000_000, 50_000_000, 10_000, 997, 1000);
        assert!(output > 0);
        assert!(output < 50_000_000);
    }
    
    #[test]
    fn test_lp_token_calculation() {
        let lp = amm::calculate_lp_tokens(1_000_000, 1_000_000_000, 50_000_000_000, 100_000_000, 5_000_000_000);
        assert!(lp > 0);
    }
    
    #[test]
    fn test_price_impact() {
        let impact = calculate_price_impact(1_000_000_000, 50_000_000_000, 100_000_000, true);
        assert!(impact > 0.0);
        assert!(impact < 100.0);
    }
}