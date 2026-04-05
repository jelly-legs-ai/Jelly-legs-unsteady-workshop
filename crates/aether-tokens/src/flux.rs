//! FLUX Token - Utility Token for AeTHer Chain
//! 
//! FLUX is the utility token used for:
//! - AI agent service payments
//! - Transaction fees
//! - Mobile mining rewards
//! - DeFi operations

use crate::utils::{self, Timestamp};

/// FLUX Token configuration
#[derive(Debug, Clone)]
pub struct FluxConfig {
    /// Maximum total supply: 10 billion * 10^9 (9 decimals)
    pub max_supply: u64,
    /// Annual inflation rate (in basis points, e.g., 500 = 5%)
    pub inflation_bps: u64,
    /// Initial circulating supply for testing
    pub initial_supply: u64,
    /// Treasury address for fee collection
    pub treasury_address: [u8; 32],
    /// Mining rewards address
    pub mining_rewards_address: [u8; 32],
}

impl Default for FluxConfig {
    fn default() -> Self {
        Self {
            max_supply: 10_000_000_000 * 10u64.pow(9), // 10 billion
            inflation_bps: 500, // 5% annual inflation
            initial_supply: 0,
            treasury_address: [0u8; 32],
            mining_rewards_address: [1u8; 32],
        }
    }
}

/// FLUX Token state
#[derive(Debug, Clone)]
pub struct FluxToken {
    pub config: FluxConfig,
    /// Current total supply (circulating)
    pub total_supply: u64,
    /// balances[address] = amount
    balances: std::collections::HashMap<[u8; 32], u64>,
    /// Allowances for delegated transfers
    allowances: std::collections::HashMap<([u8; 32], [u8; 32]), u64>,
    /// Last inflation mint timestamp
    last_inflation_mint: Timestamp,
}

impl FluxToken {
    /// Create a new FLUX token instance
    pub fn new(config: FluxConfig) -> Self {
        let initial_supply = config.initial_supply;
        Self {
            config,
            total_supply: initial_supply,
            balances: std::collections::HashMap::new(),
            allowances: std::collections::HashMap::new(),
            last_inflation_mint: utils::now(),
        }
    }

    /// Initialize with genesis balances
    pub fn with_genesis(&mut self, allocations: Vec<([u8; 32], u64)>) {
        for (address, amount) in allocations {
            *self.balances.entry(address).or_insert(0) += amount;
        }
        self.total_supply = self.config.initial_supply;
    }

    /// Get balance of an address
    pub fn balance_of(&self, address: &[u8; 32]) -> u64 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    /// Transfer FLUX tokens
    pub fn transfer(&mut self, from: &[u8; 32], to: &[u8; 32], amount: u64) -> Result<(), &'static str> {
        if amount == 0 {
            return Err("Transfer amount must be positive");
        }
        
        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err("Insufficient balance");
        }
        
        // Deduct from sender
        *self.balances.entry(*from).or_insert(0) -= amount;
        
        // Add to recipient
        *self.balances.entry(*to).or_insert(0) += amount;
        
        Ok(())
    }

    /// Approve spender to use tokens
    pub fn approve(&mut self, owner: &[u8; 32], spender: &[u8; 32], amount: u64) -> Result<(), &'static str> {
        self.allowances.insert((*owner, *spender), amount);
        Ok(())
    }

    /// Get allowance
    pub fn allowance(&self, owner: &[u8; 32], spender: &[u8; 32]) -> u64 {
        self.allowances.get(&(*owner, *spender)).copied().unwrap_or(0)
    }

    /// Transfer from (requires approval)
    pub fn transfer_from(
        &mut self,
        spender: &[u8; 32],
        from: &[u8; 32],
        to: &[u8; 32],
        amount: u64,
    ) -> Result<(), &'static str> {
        let allowed = self.allowance(from, spender);
        if allowed < amount {
            return Err("Insufficient allowance");
        }
        
        // Deduct from allowance
        self.allowances.insert((*from, *spender), allowed - amount);
        
        // Perform transfer
        self.transfer(from, to, amount)
    }

    /// Mint new FLUX (for rewards/inflation)
    pub fn mint(&mut self, to: &[u8; 32], amount: u64) -> Result<(), &'static str> {
        let new_total = self.total_supply.saturating_add(amount);
        if new_total > self.config.max_supply {
            return Err("Would exceed max supply");
        }
        
        self.total_supply = new_total;
        *self.balances.entry(*to).or_insert(0) += amount;
        
        Ok(())
    }

    /// Burn tokens (reduces supply)
    pub fn burn(&mut self, from: &[u8; 32], amount: u64) -> Result<(), &'static str> {
        let balance = self.balance_of(from);
        if balance < amount {
            return Err("Insufficient balance to burn");
        }
        
        self.total_supply = self.total_supply.saturating_sub(amount);
        *self.balances.entry(*from).or_insert(0) -= amount;
        
        Ok(())
    }

    /// Calculate annual inflation mint
    pub fn process_inflation(&mut self) -> Result<u64, &'static str> {
        let now = utils::now();
        let elapsed_years = (now - self.last_inflation_mint) as f64 / (365.25 * 24.0 * 3600.0);
        
        if elapsed_years < 1.0 / 365.25 {
            // Less than 1 day, no inflation
            return Ok(0);
        }
        
        // Calculate inflation amount
        let inflation_amount = (self.total_supply as f64 * self.config.inflation_bps as f64 / 10000.0 * elapsed_years) as u64;
        
        if inflation_amount > 0 {
            let reward_addr = self.config.mining_rewards_address;
            self.mint(&reward_addr, inflation_amount)?;
            self.last_inflation_mint = now;
        }
        
        Ok(inflation_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flux_transfer() {
        let mut flux = FluxToken::new(FluxConfig::default());
        let addr1 = [1u8; 32];
        let addr2 = [2u8; 32];
        
        flux.mint(&addr1, 1000).unwrap();
        flux.transfer(&addr1, &addr2, 500).unwrap();
        
        assert_eq!(flux.balance_of(&addr1), 500);
        assert_eq!(flux.balance_of(&addr2), 500);
    }

    #[test]
    fn test_flux_insufficient_balance() {
        let mut flux = FluxToken::new(FluxConfig::default());
        let addr1 = [1u8; 32];
        let addr2 = [2u8; 32];
        
        flux.mint(&addr1, 100).unwrap();
        let result = flux.transfer(&addr1, &addr2, 200);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_flux_approval_and_transfer_from() {
        let mut flux = FluxToken::new(FluxConfig::default());
        let owner = [1u8; 32];
        let spender = [2u8; 32];
        let recipient = [3u8; 32];
        
        flux.mint(&owner, 1000).unwrap();
        flux.approve(&owner, &spender, 500).unwrap();
        flux.transfer_from(&spender, &owner, &recipient, 300).unwrap();
        
        assert_eq!(flux.balance_of(&recipient), 300);
        assert_eq!(flux.allowance(&owner, &spender), 200);
    }
}
