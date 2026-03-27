// FLUX Token Contract - AeTHer Chain
// Utility token for AI agent services, transaction fees, and mining rewards

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FLUX token contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxTokenContract {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_supply: u64,
    pub balances: HashMap<String, u64>,
    pub allowances: HashMap<String, HashMap<String, u64>>,
    pub minting_enabled: bool,
    pub mint_cap: u64,
    pub minted_amount: u64,
    pub burn_address: String,
    pub contract_version: String,
}

impl FluxTokenContract {
    /// Create new FLUX token contract
    pub fn new() -> Self {
        let mut balances = HashMap::new();
        // Pre-mine initial supply for distribution
        balances.insert("treasury".to_string(), 500_000_000);
        balances.insert("mining_rewards".to_string(), 300_000_000);
        balances.insert("team_allocation".to_string(), 100_000_000);
        balances.insert("ecosystem_fund".to_string(), 100_000_000);
        
        FluxTokenContract {
            name: "FLUX Token".to_string(),
            symbol: "FLUX".to_string(),
            decimals: 8,
            total_supply: 1_000_000_000, // 1 billion FLUX
            circulating_supply: 0,
            burned_supply: 0,
            balances,
            allowances: HashMap::new(),
            minting_enabled: false,
            mint_cap: 1_500_000_000, // Max 1.5B FLUX ever
            minted_amount: 0,
            burn_address: "0x000000000000000000000000000000000000dead".to_string(),
            contract_version: "1.0.0".to_string(),
        }
    }
    
    /// Transfer tokens
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), &'static str> {
        if from == to {
            return Err("Cannot transfer to self");
        }
        
        let from_balance = self.balances.get(from).copied().unwrap_or(0);
        if from_balance < amount {
            return Err("Insufficient balance");
        }
        
        *self.balances.entry(from.to_string()).or_insert(0) -= amount;
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
        
        Ok(())
    }
    
    /// Approve spender to use tokens
    pub fn approve(&mut self, owner: &str, spender: &str, amount: u64) -> Result<(), &'static str> {
        self.allowances
            .entry(owner.to_string())
            .or_insert_with(HashMap::new)
            .insert(spender.to_string(), amount);
        Ok(())
    }
    
    /// Transfer from approved allowance
    pub fn transfer_from(&mut self, owner: &str, spender: &str, to: &str, amount: u64) -> Result<(), &'static str> {
        let owner_allowances = self.allowances.get_mut(owner)
            .ok_or("No allowances set")?;
        
        let allowance = owner_allowances.get_mut(spender)
            .ok_or("No allowance for spender")?;
        
        if *allowance < amount {
            return Err("Allowance exceeded");
        }
        
        self.transfer(owner, to, amount)?;
        *allowance -= amount;
        
        Ok(())
    }
    
    /// Mint new FLUX tokens (only if enabled)
    pub fn mint(&mut self, to: &str, amount: u64) -> Result<(), &'static str> {
        if !self.minting_enabled {
            return Err("Minting is disabled");
        }
        
        if self.minted_amount + amount > self.mint_cap {
            return Err("Mint cap exceeded");
        }
        
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
        self.total_supply += amount;
        self.minted_amount += amount;
        
        Ok(())
    }
    
    /// Burn tokens (reduce supply)
    pub fn burn(&mut self, from: &str, amount: u64) -> Result<(), &'static str> {
        let balance = self.balances.get(from).copied().unwrap_or(0);
        if balance < amount {
            return Err("Insufficient balance for burn");
        }
        
        *self.balances.entry(from.to_string()).or_insert(0) -= amount;
        self.burned_supply += amount;
        self.circulating_supply = self.circulating_supply.saturating_sub(amount);
        
        Ok(())
    }
    
    /// Burn from burn address (permanent supply reduction)
    pub fn burn_from_address(&mut self, amount: u64) -> Result<(), &'static str> {
        let balance = self.balances.get(&self.burn_address).copied().unwrap_or(0);
        if balance < amount {
            return Err("Insufficient balance in burn address");
        }
        
        *self.balances.entry(self.burn_address.to_string()).or_insert(0) -= amount;
        self.burned_supply += amount;
        self.total_supply = self.total_supply.saturating_sub(amount);
        
        Ok(())
    }
    
    // =============================================================================
    // SPRINT 2 ENHANCEMENT: Advanced Token Economics & Fee Structures
    // =============================================================================
    
    /// Calculate transaction fee based on amount and tier
    pub fn calculate_transaction_fee(&self, amount: u64, fee_tier: &str) -> u64 {
        let base_rate = match fee_tier {
            "standard" => 0.003,  // 0.3%
            "premium" => 0.001,   // 0.1%
            "vip" => 0.0005,      // 0.05%
            "agent" => 0.0001,    // 0.01% for AI agents
            _ => 0.003,
        };
        
        let fee = (amount as f64 * base_rate) as u64;
        fee.max(1) // Minimum 1 FLUX fee
    }
    
    /// Calculate dynamic fee based on network congestion
    pub fn calculate_dynamic_fee(&self, amount: u64, network_congestion: f64) -> u64 {
        // Base fee rate
        let base_rate = 0.003;
        
        // Congestion multiplier (0.5x at low congestion, 2.0x at high)
        let congestion_mult = if network_congestion > 0.8 {
            2.0
        } else if network_congestion > 0.5 {
            1.5
        } else if network_congestion > 0.3 {
            1.0
        } else {
            0.5
        };
        
        let fee = (amount as f64 * base_rate * congestion_mult) as u64;
        fee.max(1)
    }
    
    /// Calculate fee discount for staking FLUX
    pub fn calculate_staking_discount(&self, staked_amount: u64) -> f64 {
        if staked_amount >= 100000 {
            0.5  // 50% discount for 100K+ staked
        } else if staked_amount >= 50000 {
            0.7  // 30% discount
        } else if staked_amount >= 10000 {
            0.85 // 15% discount
        } else if staked_amount >= 1000 {
            0.95 // 5% discount
        } else {
            1.0  // No discount
        }
    }
    
    /// Calculate final fee with all discounts applied
    pub fn calculate_final_fee(
        &self,
        amount: u64,
        fee_tier: &str,
        network_congestion: f64,
        staked_amount: u64,
    ) -> u64 {
        let base_fee = self.calculate_dynamic_fee(amount, network_congestion);
        let tier_rate = match fee_tier {
            "standard" => 1.0,
            "premium" => 0.33,
            "vip" => 0.17,
            "agent" => 0.033,
            _ => 1.0,
        };
        
        let discounted_fee = (base_fee as f64 * tier_rate) as u64;
        let staking_discount = self.calculate_staking_discount(staked_amount);
        
        let final_fee = (discounted_fee as f64 * staking_discount) as u64;
        final_fee.max(1)
    }
    
    /// Distribute transaction fees to treasury and validators
    pub fn distribute_fees(&mut self, fee_amount: u64, treasury_share: f64) -> FeeDistribution {
        let treasury_amount = (fee_amount as f64 * treasury_share) as u64;
        let validator_amount = fee_amount - treasury_amount;
        
        // Add to treasury balance
        *self.balances.entry("treasury".to_string()).or_insert(0) += treasury_amount;
        
        // Add to mining rewards for validators
        *self.balances.entry("mining_rewards".to_string()).or_insert(0) += validator_amount;
        
        FeeDistribution {
            total_fee: fee_amount,
            treasury_amount,
            validator_amount,
            treasury_share: treasury_share * 100.0,
        }
    }
    
    /// Calculate token velocity (transaction volume / circulating supply)
    pub fn calculate_token_velocity(&self, transaction_volume: u64) -> f64 {
        if self.circulating_supply == 0 {
            return 0.0;
        }
        transaction_volume as f64 / self.circulating_supply as f64
    }
    
    /// Get token holder distribution stats
    pub fn get_holder_distribution(&self) -> HolderDistribution {
        let mut holders: Vec<(&String, &u64)> = self.balances.iter().collect();
        holders.sort_by(|a, b| b.1.cmp(a.1));
        
        let total_holders = holders.len();
        let top_10_total: u64 = holders.iter().take(10).map(|(_, b)| **b).sum();
        let top_50_total: u64 = holders.iter().take(50).map(|(_, b)| **b).sum();
        let top_100_total: u64 = holders.iter().take(100).map(|(_, b)| **b).sum();
        
        HolderDistribution {
            total_holders,
            top_10_concentration: top_10_total as f64 / self.circulating_supply as f64,
            top_50_concentration: top_50_total as f64 / self.circulating_supply as f64,
            top_100_concentration: top_100_total as f64 / self.circulating_supply as f64,
            gini_coefficient: self.calculate_gini_coefficient(&holders),
        }
    }
    
    /// Calculate Gini coefficient for wealth distribution
    fn calculate_gini_coefficient(&self, holders: &[(&String, &u64)]) -> f64 {
        if holders.is_empty() {
            return 0.0;
        }
        
        let n = holders.len() as f64;
        let total: u64 = holders.iter().map(|(_, b)| **b).sum();
        
        if total == 0 {
            return 0.0;
        }
        
        // Simplified Gini: cumulative share analysis
        let mut cumulative = 0.0;
        let mut gini_sum = 0.0;
        
        for (i, (_, balance)) in holders.iter().enumerate() {
            let share = *balance as f64 / total as f64;
            cumulative += share;
            gini_sum += cumulative;
        }
        
        let gini = 1.0 - (2.0 * gini_sum / n - (n + 1.0) / n);
        gini.max(0.0).min(1.0)
    }
    
    /// Calculate market cap equivalent (assuming price feed)
    pub fn calculate_market_cap(&self, price_per_token: f64) -> f64 {
        self.circulating_supply as f64 * price_per_token
    }
    
    /// Calculate fully diluted valuation
    pub fn calculate_fdv(&self, price_per_token: f64) -> f64 {
        self.total_supply as f64 * price_per_token
    }
    
    /// Get inflation rate (minted / total supply)
    pub fn get_inflation_rate(&self) -> f64 {
        if self.total_supply == 0 {
            return 0.0;
        }
        self.minted_amount as f64 / self.total_supply as f64
    }
    
    /// Calculate burn rate (burned / circulating)
    pub fn get_burn_rate(&self) -> f64 {
        if self.circulating_supply == 0 {
            return 0.0;
        }
        self.burned_supply as f64 / self.circulating_supply as f64
    }
    
    /// Get token economics summary
    pub fn get_token_economics_summary(&self) -> TokenEconomicsSummary {
        TokenEconomicsSummary {
            total_supply: self.total_supply,
            circulating_supply: self.circulating_supply,
            burned_supply: self.burned_supply,
            minted_amount: self.minted_amount,
            mint_cap_remaining: self.mint_cap - self.minted_amount,
            inflation_rate: self.get_inflation_rate(),
            burn_rate: self.get_burn_rate(),
            holder_count: self.balances.len(),
        }
    }
    
    /// Check if address qualifies for agent tier pricing
    pub fn is_agent_address(&self, address: &str) -> bool {
        // Simple heuristic: addresses with "agent" or high balance
        address.contains("agent") || self.balances.get(address).unwrap_or(&0) > &50000
    }
    
    /// Airdrop tokens to multiple addresses
    pub fn airdrop(&mut self, recipients: Vec<(&str, u64)>) -> Result<AirdropResult, &'static str> {
        let mut total_amount = 0u64;
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();
        
        for (address, amount) in recipients {
            total_amount += amount;
        }
        
        // Check if treasury has enough
        let treasury_balance = self.balances.get("treasury").copied().unwrap_or(0);
        if treasury_balance < total_amount {
            return Err("Insufficient treasury balance for airdrop");
        }
        
        for (address, amount) in recipients {
            match self.transfer("treasury", address, amount) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push((address.to_string(), e.to_string()));
                }
            }
        }
        
        Ok(AirdropResult {
            total_amount,
            successful,
            failed,
            errors,
        })
    }
    
    /// Vesting schedule check
    pub fn check_vesting_schedule(&self, address: &str, cliff_epochs: u64, total_epochs: u64, current_epoch: u64) -> VestingStatus {
        let balance = self.balances.get(address).copied().unwrap_or(0);
        
        if current_epoch < cliff_epochs {
            return VestingStatus {
                vested_amount: 0,
                locked_amount: balance,
                vested_percentage: 0.0,
                epochs_until_cliff: cliff_epochs - current_epoch,
                status: "Cliff period active".to_string(),
            };
        }
        
        let epochs_since_cliff = current_epoch - cliff_epochs;
        let vested_percentage = (epochs_since_cliff as f64 / (total_epochs - cliff_epochs) as f64).min(1.0);
        let vested_amount = (balance as f64 * vested_percentage) as u64;
        let locked_amount = balance - vested_amount;
        
        VestingStatus {
            vested_amount,
            locked_amount,
            vested_percentage: vested_percentage * 100.0,
            epochs_until_cliff: 0,
            status: if vested_percentage >= 1.0 { "Fully vested" } else { "Vesting in progress" }.to_string(),
        }
    }
    
    /// Calculate liquidity score based on distribution
    pub fn calculate_liquidity_score(&self) -> LiquidityScore {
        let holder_dist = self.get_holder_distribution();
        
        // Decentralization score (inverse of concentration)
        let decentralization = (1.0 - holder_dist.top_10_concentration) * 100.0;
        
        // Holder count score
        let holder_score = if holder_dist.total_holders > 10000 {
            100.0
        } else if holder_dist.total_holders > 1000 {
            80.0
        } else if holder_dist.total_holders > 100 {
            60.0
        } else {
            40.0
        };
        
        // Velocity score
        let velocity_score = if self.circulating_supply > 0 {
            ((self.balances.len() as f64 / 100.0) * 50.0).min(100.0)
        } else {
            0.0
        };
        
        let overall_score = (decentralization + holder_score + velocity_score) / 3.0;
        
        LiquidityScore {
            overall_score,
            decentralization_score: decentralization,
            holder_score,
            velocity_score,
            status: if overall_score >= 80.0 { "Excellent" }
                   else if overall_score >= 60.0 { "Good" }
                   else if overall_score >= 40.0 { "Fair" }
                   else { "Poor" }.to_string(),
        }
    }
    
    /// Get fee tier recommendation based on holding amount
    pub fn recommend_fee_tier(&self, address: &str) -> FeeTierRecommendation {
        let balance = self.balances.get(address).copied().unwrap_or(0);
        
        if balance >= 100000 {
            FeeTierRecommendation {
                tier: "vip".to_string(),
                reason: "High balance holder (>100K FLUX)".to_string(),
                discount_rate: 0.0005,
                estimated_savings_percent: 83.3,
            }
        } else if balance >= 50000 {
            FeeTierRecommendation {
                tier: "premium".to_string(),
                reason: "Medium balance holder (>50K FLUX)".to_string(),
                discount_rate: 0.001,
                estimated_savings_percent: 66.7,
            }
        } else if balance >= 1000 || address.contains("agent") {
            FeeTierRecommendation {
                tier: "agent".to_string(),
                reason: "AI agent or small holder".to_string(),
                discount_rate: 0.0001,
                estimated_savings_percent: 96.7,
            }
        } else {
            FeeTierRecommendation {
                tier: "standard".to_string(),
                reason: "Standard tier (<1K FLUX)".to_string(),
                discount_rate: 0.003,
                estimated_savings_percent: 0.0,
            }
        }
    }
    
    /// Calculate deflationary pressure (burn rate vs mint rate)
    pub fn calculate_deflationary_pressure(&self) -> DeflationaryPressure {
        let burn_ratio = if self.total_supply > 0 {
            self.burned_supply as f64 / self.total_supply as f64
        } else {
            0.0
        };
        
        let mint_ratio = if self.mint_cap > 0 {
            self.minted_amount as f64 / self.mint_cap as f64
        } else {
            0.0
        };
        
        let net_pressure = burn_ratio - mint_ratio;
        
        DeflationaryPressure {
            burn_ratio,
            mint_ratio,
            net_pressure,
            trend: if net_pressure > 0.01 { "Deflationary" }
                   else if net_pressure < -0.01 { "Inflationary" }
                   else { "Neutral" }.to_string(),
        }
    }
    
    /// Get treasury health metrics
    pub fn get_treasury_health(&self) -> TreasuryHealth {
        let treasury_balance = self.balances.get("treasury").copied().unwrap_or(0);
        let mining_balance = self.balances.get("mining_rewards").copied().unwrap_or(0);
        let ecosystem_balance = self.balances.get("ecosystem_fund").copied().unwrap_or(0);
        
        let total_reserved = treasury_balance + mining_balance + ecosystem_balance;
        let treasury_ratio = if total_reserved > 0 {
            treasury_balance as f64 / total_reserved as f64
        } else {
            0.0
        };
        
        TreasuryHealth {
            treasury_balance,
            mining_balance,
            ecosystem_balance,
            total_reserved,
            treasury_ratio,
            runway_score: if treasury_balance > 100_000_000 { 100.0 }
                          else if treasury_balance > 50_000_000 { 80.0 }
                          else if treasury_balance > 10_000_000 { 60.0 }
                          else { 40.0 },
        }
    }
    
    /// Calculate token utility score based on usage patterns
    pub fn calculate_utility_score(&self, tx_count: u64, active_addresses: u64) -> UtilityScore {
        let tx_velocity = if self.circulating_supply > 0 {
            tx_count as f64 / self.circulating_supply as f64
        } else {
            0.0
        };
        
        let adoption_score = if active_addresses > 10000 {
            100.0
        } else if active_addresses > 1000 {
            80.0
        } else if active_addresses > 100 {
            60.0
        } else {
            40.0
        };
        
        let utility_score = (tx_velocity * 1000.0 * 50.0 + adoption_score * 0.5).min(100.0);
        
        UtilityScore {
            overall_score: utility_score,
            tx_velocity,
            adoption_score,
            active_addresses,
            status: if utility_score >= 80.0 { "High Utility" }
                   else if utility_score >= 60.0 { "Medium Utility" }
                   else if utility_score >= 40.0 { "Low Utility" }
                   else { "Minimal Utility" }.to_string(),
        }
    }
    
    // =============================================================================
    // SPRINT 9: Enhanced Token Economics & Cross-Chain Bridge Support
    // =============================================================================
    
    /// Simulate token economics scenario
    pub fn simulate_economics_scenario(&self, scenario: &EconomicsScenario) -> EconomicsSimulation {
        let mut simulated_supply = self.circulating_supply;
        let mut simulated_burned = self.burned_supply;
        let mut simulated_minted = self.minted_amount;
        
        match scenario {
            EconomicsScenario::HighAdoption { tx_growth, user_growth } => {
                // Simulate increased burns from higher tx volume
                let burn_increase = (tx_growth * 0.01 * self.circulating_supply as f64) as u64;
                simulated_burned += burn_increase;
            }
            EconomicsScenario::LowAdoption { tx_decline, user_decline } => {
                // Simulate reduced activity
                let mint_increase = (*user_decline * 0.001 * self.total_supply as f64) as u64;
                simulated_minted += mint_increase;
            }
        }
        
        EconomicsSimulation {
            final_supply: simulated_supply,
            final_burned: simulated_burned,
            final_minted: simulated_minted,
        }
    }
    
    /// Cross-chain bridge lock for wrapped tokens
    pub fn bridge_lock(&mut self, from: &str, amount: u64, target_chain: &str) -> Result<BridgeLockReceipt, &'static str> {
        let balance = self.balance_of(from);
        if balance < amount {
            return Err("Insufficient balance for bridge");
        }
        
        // Lock tokens in bridge contract
        self.transfer(from, "bridge_contract", amount)?;
        
        let receipt = BridgeLockReceipt {
            lock_id: format!("bridge_{}_{}_{}", from, target_chain, self.circulating_supply),
            source_chain: "aether".to_string(),
            target_chain: target_chain.to_string(),
            amount,
            locked_at: self.circulating_supply,
            status: "locked".to_string(),
        };
        
        Ok(receipt)
    }
    
    /// Cross-chain bridge unlock (mint wrapped tokens on target chain)
    pub fn bridge_unlock(&mut self, receipt_id: &str, to: &str) -> Result<u64, &'static str> {
        // In production, would verify cross-chain proof
        // This is a stub for the interface
        Ok(0)
    }
    
    /// Get bridge statistics
    pub fn get_bridge_stats(&self) -> BridgeStats {
        let bridge_balance = self.balance_of("bridge_contract");
        BridgeStats {
            total_locked: bridge_balance,
            total_bridged_out: 0,
            total_bridged_in: 0,
            active_locks: 0,
        }
    }
    
    /// Calculate bridge fee for cross-chain transfer
    pub fn calculate_bridge_fee(&self, amount: u64, target_chain: &str) -> u64 {
        let base_fee = (amount as f64 * 0.005) as u64; // 0.5% base fee
        let chain_multiplier = match target_chain {
            "ethereum" => 1.5,
            "bsc" => 1.2,
            "polygon" => 1.0,
            "solana" => 1.3,
            _ => 1.0,
        };
        (base_fee as f64 * chain_multiplier) as u64
    }
    
    /// Wrapped FLUX mint (for bridged-in tokens)
    pub fn mint_wrapped_flux(&mut self, to: &str, amount: u64, source_chain: &str) -> Result<(), &'static str> {
        // Mint wrapped FLUX on AeTHer Chain
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
        self.total_supply += amount;
        Ok(())
    }
    
    /// Burn wrapped FLUX (for bridging out)
    pub fn burn_wrapped_flux(&mut self, from: &str, amount: u64) -> Result<(), &'static str> {
        self.burn(from, amount)?;
        Ok(())
    }
    
    /// Get liquidity pool info
    pub fn get_liquidity_pool_info(&self, pool_id: &str) -> LiquidityPoolInfo {
        let pool_balance = self.balance_of(&format!("pool_{}", pool_id));
        LiquidityPoolInfo {
            pool_id: pool_id.to_string(),
            total_liquidity: pool_balance,
            apr: 0.12, // 12% base APR
            volume_24h: 0,
            fees_24h: 0,
        }
    }
    
    /// Add liquidity to pool
    pub fn add_liquidity(&mut self, provider: &str, pool_id: &str, amount: u64) -> Result<LiquidityReceipt, &'static str> {
        let balance = self.balance_of(provider);
        if balance < amount {
            return Err("Insufficient balance");
        }
        
        self.transfer(provider, &format!("pool_{}", pool_id), amount)?;
        
        Ok(LiquidityReceipt {
            receipt_id: format!("liq_{}_{}_{}", provider, pool_id, self.circulating_supply),
            provider: provider.to_string(),
            pool_id: pool_id.to_string(),
            amount,
            lp_tokens_issued: amount, // 1:1 for simplicity
        })
    }
    
    /// Remove liquidity from pool
    pub fn remove_liquidity(&mut self, provider: &str, pool_id: &str, lp_amount: u64) -> Result<u64, &'static str> {
        // In production, would calculate share of pool
        Ok(lp_amount)
    }
    
    /// Calculate impermanent loss for liquidity provider
    pub fn calculate_impermanent_loss(&self, initial_price_ratio: f64, current_price_ratio: f64, initial_value: f64) -> f64 {
        let sqrt_ratio = (current_price_ratio / initial_price_ratio).sqrt();
        let il = 2.0 * sqrt_ratio / (1.0 + sqrt_ratio) - 1.0;
        il * initial_value
    }
    
    /// Get token economics dashboard data
    pub fn get_economics_dashboard(&self) -> EconomicsDashboard {
        let holder_dist = self.get_holder_distribution();
        let treasury_health = self.get_treasury_health();
        let liquidity_score = self.calculate_liquidity_score();
        
        EconomicsDashboard {
            supply_metrics: self.get_token_economics_summary(),
            holder_metrics: holder_dist,
            treasury_metrics: treasury_health,
            liquidity_metrics: liquidity_score,
            timestamp: self.circulating_supply, // Proxy for timestamp
        }
    }
    
    /// Emergency pause function (governance only)
    pub fn emergency_pause(&mut self, reason: &str) -> EmergencyPause {
        self.minting_enabled = false;
        EmergencyPause {
            paused_at: self.circulating_supply,
            reason: reason.to_string(),
            transfers_enabled: true,
            burns_enabled: true,
        }
    }
    
    /// Unpause after emergency
    pub fn emergency_unpause(&mut self) {
        self.minting_enabled = true;
    }
    
    /// Get circulating supply excluding locked/bridged amounts
    pub fn get_true_circulating_supply(&self, locked_amount: u64, bridged_amount: u64) -> u64 {
        self.circulating_supply.saturating_sub(locked_amount).saturating_sub(bridged_amount)
    }
    
    /// Calculate fully diluted market cap at given price
    pub fn calculate_fully_diluted_valuation(&self, price_usd: f64) -> f64 {
        self.total_supply as f64 * price_usd
    }
    
    /// Get token holder concentration risk assessment
    pub fn assess_concentration_risk(&self) -> ConcentrationRisk {
        let holder_dist = self.get_holder_distribution();
        let top_10_pct = holder_dist.top_10_concentration * 100.0;
        
        ConcentrationRisk {
            risk_level: if top_10_pct > 50.0 { "High" }
                       else if top_10_pct > 30.0 { "Medium" }
                       else { "Low" }.to_string(),
            top_10_percentage: top_10_pct,
            top_50_percentage: holder_dist.top_50_concentration * 100.0,
            top_100_percentage: holder_dist.top_100_concentration * 100.0,
            recommendation: if top_10_pct > 50.0 {
                "Consider incentives for broader distribution".to_string()
            } else {
                "Distribution within acceptable range".to_string()
            },
        }
    }
}

/// Economics scenario for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomicsScenario {
    HighAdoption { tx_growth: f64, user_growth: f64 },
    LowAdoption { tx_decline: f64, user_decline: f64 },
}

/// Economics simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicsSimulation {
    pub final_supply: u64,
    pub final_burned: u64,
    pub final_minted: u64,
}

/// Bridge lock receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeLockReceipt {
    pub lock_id: String,
    pub source_chain: String,
    pub target_chain: String,
    pub amount: u64,
    pub locked_at: u64,
    pub status: String,
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    pub total_locked: u64,
    pub total_bridged_out: u64,
    pub total_bridged_in: u64,
    pub active_locks: u64,
}

/// Liquidity pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPoolInfo {
    pub pool_id: String,
    pub total_liquidity: u64,
    pub apr: f64,
    pub volume_24h: u64,
    pub fees_24h: u64,
}

/// Liquidity receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityReceipt {
    pub receipt_id: String,
    pub provider: String,
    pub pool_id: String,
    pub amount: u64,
    pub lp_tokens_issued: u64,
}

/// Fee distribution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeDistribution {
    pub total_fee: u64,
    pub treasury_amount: u64,
    pub validator_amount: u64,
    pub treasury_share: f64,
}

/// Token economics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEconomicsSummary {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_supply: u64,
    pub minted_amount: u64,
    pub mint_cap_remaining: u64,
    pub inflation_rate: f64,
    pub burn_rate: f64,
    pub holder_count: usize,
}

/// Vesting status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingStatus {
    pub vested_amount: u64,
    pub locked_amount: u64,
    pub vested_percentage: f64,
    pub epochs_until_cliff: u64,
    pub status: String,
}

/// Liquidity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityScore {
    pub overall_score: f64,
    pub decentralization_score: f64,
    pub holder_score: f64,
    pub velocity_score: f64,
    pub status: String,
}

/// Fee tier recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeTierRecommendation {
    pub tier: String,
    pub reason: String,
    pub discount_rate: f64,
    pub estimated_savings_percent: f64,
}

/// Deflationary pressure metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeflationaryPressure {
    pub burn_ratio: f64,
    pub mint_ratio: f64,
    pub net_pressure: f64,
    pub trend: String,
}

/// Treasury health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryHealth {
    pub treasury_balance: u64,
    pub mining_balance: u64,
    pub ecosystem_balance: u64,
    pub total_reserved: u64,
    pub treasury_ratio: f64,
    pub runway_score: f64,
}

/// Token utility score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityScore {
    pub overall_score: f64,
    pub tx_velocity: f64,
    pub adoption_score: f64,
    pub active_addresses: u64,
    pub status: String,
}

/// Airdrop result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirdropResult {
    pub total_amount: u64,
    pub successful: u64,
    pub failed: u64,
    pub errors: Vec<(String, String)>,
}

/// Emergency pause state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyPause {
    pub paused_at: u64,
    pub reason: String,
    pub transfers_enabled: bool,
    pub burns_enabled: bool,
}

/// Concentration risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationRisk {
    pub risk_level: String,
    pub top_10_percentage: f64,
    pub top_50_percentage: f64,
    pub top_100_percentage: f64,
    pub recommendation: String,
}

/// Economics dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicsDashboard {
    pub supply_metrics: TokenEconomicsSummary,
    pub holder_metrics: HolderDistribution,
    pub treasury_metrics: TreasuryHealth,
    pub liquidity_metrics: LiquidityScore,
    pub timestamp: u64,
}

        if self.minted_amount + amount > self.mint_cap {
            return Err("Mint cap exceeded");
        }
        
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
        self.total_supply += amount;
        self.minted_amount += amount;
        
        Ok(())
    }
    
    /// Burn FLUX tokens
    pub fn burn(&mut self, from: &str, amount: u64) -> Result<(), &'static str> {
        let balance = self.balances.get(from).copied().unwrap_or(0);
        if balance < amount {
            return Err("Insufficient balance to burn");
        }
        
        *self.balances.get_mut(from).unwrap() -= amount;
        self.burned_supply += amount;
        self.circulating_supply = self.circulating_supply.saturating_sub(amount);
        
        // Transfer to burn address (effectively remove from circulation)
        *self.balances.entry(self.burn_address.clone()).or_insert(0) += amount;
        
        Ok(())
    }
    
    /// Get balance of an address
    pub fn balance_of(&self, address: &str) -> u64 {
        *self.balances.get(address).unwrap_or(&0)
    }
    
    /// Get allowance for spender
    pub fn allowance(&self, owner: &str, spender: &str) -> u64 {
        self.allowances
            .get(owner)
            .and_then(|spenders| spenders.get(spender))
            .copied()
            .unwrap_or(0)
    }
    
    /// Calculate FLUX in circulation (excluding burn address)
    pub fn calculate_circulating_supply(&self) -> u64 {
        let burn_balance = self.balance_of(&self.burn_address);
        self.total_supply - burn_balance
    }
    
    /// Get token stats summary
    pub fn get_token_stats(&self) -> TokenStats {
        TokenStats {
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            total_supply: self.total_supply,
            circulating_supply: self.calculate_circulating_supply(),
            burned_supply: self.burned_supply,
            minted_amount: self.minted_amount,
            mint_cap: self.mint_cap,
            minting_enabled: self.minting_enabled,
            holder_count: self.balances.len(),
            contract_version: self.contract_version.clone(),
        }
    }
    
    /// Enable/disable minting
    pub fn set_minting_enabled(&mut self, enabled: bool) {
        self.minting_enabled = enabled;
    }
    
    /// Distribute mining rewards
    pub fn distribute_mining_rewards(&mut self, miners: &[&str], rewards_per_miner: u64) -> Result<u64, &'static str> {
        let mining_balance = self.balance_of("mining_rewards");
        let total_needed = miners.len() as u64 * rewards_per_miner;
        
        if mining_balance < total_needed {
            return Err("Insufficient mining rewards balance");
        }
        
        for miner in miners {
            self.transfer("mining_rewards", miner, rewards_per_miner)?;
        }
        
        Ok(total_needed)
    }
    
    /// Calculate FLUX per USD at given price
    pub fn flux_per_usd(&self, price_usd: f64) -> f64 {
        if price_usd <= 0.0 {
            return 0.0;
        }
        1.0 / price_usd
    }
    
    /// Calculate USD value of FLUX amount
    pub fn usd_value(&self, amount: u64, price_usd: f64) -> f64 {
        amount as f64 * price_usd
    }
    
    // =============================================================================
    // STAKING REWARDS DISTRIBUTION - Sprint 11 Enhancement
    // =============================================================================
    
    /// Distribute staking rewards to eligible addresses
    pub fn distribute_staking_rewards(&mut self, stakers: &[&str], rewards_per_staker: u64) -> Result<u64, &'static str> {
        let treasury_balance = self.balance_of("treasury");
        let total_needed = stakers.len() as u64 * rewards_per_staker;
        
        if treasury_balance < total_needed {
            return Err("Insufficient treasury balance for rewards");
        }
        
        for staker in stakers {
            self.transfer("treasury", *staker, rewards_per_staker)?;
        }
        
        Ok(total_needed)
    }
    
    /// Calculate staking APY based on total staked and rewards rate
    pub fn calculate_staking_apy(&self, total_staked: u64, annual_rewards: u64) -> f64 {
        if total_staked == 0 {
            return 0.0;
        }
        (annual_rewards as f64 / total_staked as f64) * 100.0
    }
    
    /// Get reward per epoch for staking pool
    pub fn get_epoch_reward_rate(&self, pool_id: &str, total_staked: u64) -> u64 {
        // Base rate: 15% APY / 365 days / 24 epochs
        let base_rate = 0.15 / 365.0 / 24.0;
        (total_staked as f64 * base_rate) as u64
    }
    
    /// Calculate compound staking rewards (rewards reinvested)
    pub fn calculate_compound_staking_rewards(&self, principal: u64, apy: f64, epochs: u64) -> u64 {
        let epoch_rate = apy / 365.0 / 24.0;
        let compounded = (principal as f64 * (1.0 + epoch_rate).powf(epochs as f64)) as u64;
        compounded - principal
    }
    
    /// Get staking rewards tier based on stake amount
    pub fn get_rewards_tier(&self, stake_amount: u64) -> RewardsTier {
        if stake_amount >= 100_000 {
            RewardsTier::Platinum
        } else if stake_amount >= 50_000 {
            RewardsTier::Gold
        } else if stake_amount >= 10_000 {
            RewardsTier::Silver
        } else {
            RewardsTier::Bronze
        }
    }
    
    /// Calculate tier bonus multiplier
    pub fn get_tier_bonus(&self, tier: &RewardsTier) -> f64 {
        match tier {
            RewardsTier::Platinum => 1.5, // 50% bonus
            RewardsTier::Gold => 1.25,    // 25% bonus
            RewardsTier::Silver => 1.1,   // 10% bonus
            RewardsTier::Bronze => 1.0,   // No bonus
        }
    }
    
    /// Lock tokens for staking (transfer to staking contract)
    pub fn lock_for_staking(&mut self, from: &str, amount: u64, lock_epochs: u64) -> Result<StakingLock, &'static str> {
        let balance = self.balance_of(from);
        if balance < amount {
            return Err("Insufficient balance");
        }
        
        // Transfer to staking contract address
        self.transfer(from, "staking_contract", amount)?;
        
        let lock = StakingLock {
            lock_id: format!("lock_{}_{}", from, self.circulating_supply),
            owner: from.to_string(),
            amount,
            locked_at: self.circulating_supply, // Using supply as epoch proxy
            unlock_at: self.circulating_supply + lock_epochs,
            is_active: true,
        };
        
        Ok(lock)
    }
    
    /// Unlock staked tokens after lock period
    pub fn unlock_staking(&mut self, lock_id: &str) -> Result<u64, &'static str> {
        // In production, would query actual staking contract
        // This is a stub for the interface
        Ok(0)
    }
    
    /// Slash staked tokens for misbehavior
    pub fn slash_staking(&mut self, lock_id: &str, slash_percent: f64) -> Result<u64, &'static str> {
        // In production, would interact with slashing contract
        Ok(0)
    }
    
    /// Get total staking rewards distributed
    pub fn total_staking_rewards_distributed(&self) -> u64 {
        let staking_balance = self.balance_of("staking_contract");
        let treasury_balance = self.balance_of("treasury");
        let mining_balance = self.balance_of("mining_rewards");
        
        // Rewards distributed = initial treasury - current treasury
        500_000_000 - treasury_balance - staking_balance - mining_balance
    }
    
    /// Calculate inflation rate based on minting
    pub fn calculate_inflation_rate(&self) -> f64 {
        if self.total_supply == 0 {
            return 0.0;
        }
        (self.minted_amount as f64 / self.total_supply as f64) * 100.0
    }
    
    /// Get token holder distribution stats
    pub fn get_holder_distribution(&self) -> HolderDistribution {
        let mut distribution = HolderDistribution {
            whales: 0,      // > 1M FLUX
            large: 0,       // 100K - 1M
            medium: 0,      // 10K - 100K
            small: 0,       // 1K - 10K
            micro: 0,       // < 1K
            total_holders: self.balances.len(),
        };
        
        for balance in self.balances.values() {
            if *balance >= 1_000_000 {
                distribution.whales += 1;
            } else if *balance >= 100_000 {
                distribution.large += 1;
            } else if *balance >= 10_000 {
                distribution.medium += 1;
            } else if *balance >= 1_000 {
                distribution.small += 1;
            } else {
                distribution.micro += 1;
            }
        }
        
        distribution
    }
    
    /// Check if address is eligible for airdrop
    pub fn is_airdrop_eligible(&self, address: &str, min_balance: u64, active_epochs: u64) -> bool {
        let balance = self.balance_of(address);
        balance >= min_balance
        // In production, would also check active_epochs
    }
    
    /// Distribute airdrop to eligible addresses
    pub fn distribute_airdrop(&mut self, eligible_addresses: &[&str], amount_per_address: u64) -> Result<u64, &'static str> {
        let ecosystem_balance = self.balance_of("ecosystem_fund");
        let total_needed = eligible_addresses.len() as u64 * amount_per_address;
        
        if ecosystem_balance < total_needed {
            return Err("Insufficient ecosystem fund balance");
        }
        
        for address in eligible_addresses {
            self.transfer("ecosystem_fund", *address, amount_per_address)?;
        }
        
        Ok(total_needed)
    }
    
    /// Calculate token velocity (transfers per epoch)
    pub fn calculate_token_velocity(&self, transfers_per_epoch: u64, circulating_supply: u64) -> f64 {
        if circulating_supply == 0 {
            return 0.0;
        }
        transfers_per_epoch as f64 / circulating_supply as f64
    }
    
    /// Get token supply breakdown
    pub fn get_supply_breakdown(&self) -> SupplyBreakdown {
        SupplyBreakdown {
            total_supply: self.total_supply,
            circulating_supply: self.calculate_circulating_supply(),
            burned_supply: self.burned_supply,
            treasury_balance: self.balance_of("treasury"),
            mining_rewards_balance: self.balance_of("mining_rewards"),
            team_balance: self.balance_of("team_allocation"),
            ecosystem_balance: self.balance_of("ecosystem_fund"),
            staking_contract_balance: self.balance_of("staking_contract"),
            locked_supply: self.balance_of("staking_contract"),
            liquid_supply: self.calculate_circulating_supply() - self.balance_of("staking_contract"),
        }
    }
}

/// Staking lock record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingLock {
    pub lock_id: String,
    pub owner: String,
    pub amount: u64,
    pub locked_at: u64,
    pub unlock_at: u64,
    pub is_active: bool,
}

/// Rewards tier based on stake amount
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RewardsTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

/// Token holder distribution stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolderDistribution {
    pub whales: u64,
    pub large: u64,
    pub medium: u64,
    pub small: u64,
    pub micro: u64,
    pub total_holders: usize,
}

/// Token supply breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyBreakdown {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_supply: u64,
    pub treasury_balance: u64,
    pub mining_rewards_balance: u64,
    pub team_balance: u64,
    pub ecosystem_balance: u64,
    pub staking_contract_balance: u64,
    pub locked_supply: u64,
    pub liquid_supply: u64,
}

/// Token statistics for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub name: String,
    pub symbol: String,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_supply: u64,
    pub minted_amount: u64,
    pub mint_cap: u64,
    pub minting_enabled: bool,
    pub holder_count: usize,
    pub contract_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_creation() {
        let contract = FluxTokenContract::new();
        assert_eq!(contract.total_supply, 1_000_000_000);
        assert_eq!(contract.symbol, "FLUX");
        assert_eq!(contract.decimals, 8);
    }
    
    #[test]
    fn test_transfer() {
        let mut contract = FluxTokenContract::new();
        contract.transfer("treasury", "user1", 1000).unwrap();
        assert_eq!(contract.balance_of("user1"), 1000);
        assert!(contract.balance_of("treasury") < 500_000_000);
    }
    
    #[test]
    fn test_burn() {
        let mut contract = FluxTokenContract::new();
        contract.transfer("treasury", "user1", 1000).unwrap();
        contract.burn("user1", 500).unwrap();
        assert_eq!(contract.balance_of("user1"), 500);
        assert_eq!(contract.burned_supply, 500);
    }
}
