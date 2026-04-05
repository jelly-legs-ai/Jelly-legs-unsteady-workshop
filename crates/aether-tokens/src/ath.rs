//! ATH Token - Governance Token for AeTHer Chain
//!
//! ATH is the governance token used for:
//! - Validator staking
//! - Governance voting
//! - Agent KYC bonding
//! - Security collateral

/// ATH Token configuration
#[derive(Debug, Clone)]
pub struct AthConfig {
    /// Maximum total supply: 1 billion * 10^9 (9 decimals)
    pub max_supply: u64,
    /// Foundation reserve (locked)
    pub foundation_reserve: u64,
    /// Ecosystem development fund
    pub ecosystem_fund: u64,
    /// Treasury address
    pub treasury_address: [u8; 32],
}

impl Default for AthConfig {
    fn default() -> Self {
        Self {
            max_supply: 1_000_000_000 * 10u64.pow(9), // 1 billion
            foundation_reserve: 200_000_000 * 10u64.pow(9), // 20%
            ecosystem_fund: 100_000_000 * 10u64.pow(9), // 10%
            treasury_address: [0u8; 32],
        }
    }
}

/// ATH Token state
#[derive(Debug, Clone)]
pub struct AthToken {
    pub config: AthConfig,
    /// Current total supply
    pub total_supply: u64,
    /// Circulating supply (total - reserved)
    pub circulating_supply: u64,
    /// balances[address] = amount
    balances: std::collections::HashMap<[u8; 32], u64>,
    /// Locked balances (vesting)
    locked: std::collections::HashMap<([u8; 32], [u8; 32]), u64>, // (beneficiary, locker) -> amount
    /// Allowances
    allowances: std::collections::HashMap<([u8; 32], [u8; 32]), u64>,
}

impl AthToken {
    /// Create a new ATH token instance
    pub fn new(config: AthConfig) -> Self {
        Self {
            config,
            total_supply: 0,
            circulating_supply: 0,
            balances: std::collections::HashMap::new(),
            locked: std::collections::HashMap::new(),
            allowances: std::collections::HashMap::new(),
        }
    }

    /// Initialize with genesis distribution
    pub fn with_genesis(&mut self, allocations: Vec<([u8; 32], u64)>) {
        for (address, amount) in allocations {
            *self.balances.entry(address).or_insert(0) += amount;
            self.circulating_supply += amount;
        }
        self.total_supply = self.config.foundation_reserve 
            + self.config.ecosystem_fund 
            + self.circulating_supply;
    }

    /// Get balance of an address
    pub fn balance_of(&self, address: &[u8; 32]) -> u64 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    /// Get spendable balance (total - locked)
    pub fn spendable_balance_of(&self, address: &[u8; 32]) -> u64 {
        let total = self.balance_of(address);
        let locked: u64 = self.locked
            .iter()
            .filter(|((addr, _), _)| addr == address)
            .map(|(_, amount)| *amount)
            .sum();
        total.saturating_sub(locked)
    }

    /// Transfer ATH tokens
    pub fn transfer(&mut self, from: &[u8; 32], to: &[u8; 32], amount: u64) -> Result<(), &'static str> {
        if amount == 0 {
            return Err("Transfer amount must be positive");
        }
        
        let spendable = self.spendable_balance_of(from);
        if spendable < amount {
            return Err("Insufficient spendable balance (locked tokens)");
        }
        
        // Deduct from sender
        *self.balances.entry(*from).or_insert(0) -= amount;
        
        // Add to recipient
        *self.balances.entry(*to).or_insert(0) += amount;
        
        Ok(())
    }

    /// Lock tokens for vesting
    pub fn lock(
        &mut self, 
        beneficiary: &[u8; 32], 
        locker: &[u8; 32], 
        amount: u64,
    ) -> Result<(), &'static str> {
        let balance = self.balance_of(beneficiary);
        let current_locked: u64 = self.locked
            .iter()
            .filter(|((addr, _), _)| addr == beneficiary)
            .map(|(_, amt)| *amt)
            .sum();
            
        if balance - current_locked < amount {
            return Err("Insufficient unlocked balance for locking");
        }
        
        self.locked.insert((*beneficiary, *locker), amount);
        Ok(())
    }

    /// Unlock tokens
    pub fn unlock(&mut self, beneficiary: &[u8; 32], locker: &[u8; 32]) -> Result<u64, &'static str> {
        let locked = self.locked.get(&(*beneficiary, *locker)).copied().unwrap_or(0);
        if locked > 0 {
            self.locked.remove(&(*beneficiary, *locker));
        }
        Ok(locked)
    }

    /// Approve spender
    pub fn approve(&mut self, owner: &[u8; 32], spender: &[u8; 32], amount: u64) -> Result<(), &'static str> {
        self.allowances.insert((*owner, *spender), amount);
        Ok(())
    }

    /// Get allowance
    pub fn allowance(&self, owner: &[u8; 32], spender: &[u8; 32]) -> u64 {
        self.allowances.get(&(*owner, *spender)).copied().unwrap_or(0)
    }

    /// Transfer from with allowance check
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
        
        self.allowances.insert((*from, *spender), allowed - amount);
        self.transfer(from, to, amount)
    }

    /// Burn tokens (reduces supply)
    pub fn burn(&mut self, from: &[u8; 32], amount: u64) -> Result<(), &'static str> {
        let spendable = self.spendable_balance_of(from);
        if spendable < amount {
            return Err("Insufficient spendable balance to burn");
        }
        
        self.total_supply = self.total_supply.saturating_sub(amount);
        self.circulating_supply = self.circulating_supply.saturating_sub(amount);
        *self.balances.entry(*from).or_insert(0) -= amount;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ath_transfer() {
        let mut ath = AthToken::new(AthConfig::default());
        let addr1 = [1u8; 32];
        let addr2 = [2u8; 32];
        
        *ath.balances.entry(addr1).or_insert(0) = 1000;
        ath.circulating_supply = 1000;
        
        ath.transfer(&addr1, &addr2, 500).unwrap();
        
        assert_eq!(ath.balance_of(&addr1), 500);
        assert_eq!(ath.balance_of(&addr2), 500);
    }

    #[test]
    fn test_ath_locking() {
        let mut ath = AthToken::new(AthConfig::default());
        let beneficiary = [1u8; 32];
        let locker = [2u8; 32];
        
        *ath.balances.entry(beneficiary).or_insert(0) = 1000;
        ath.circulating_supply = 1000;
        
        ath.lock(&beneficiary, &locker, 600).unwrap();
        
        // Should only be able to transfer unlocked amount
        assert_eq!(ath.spendable_balance_of(&beneficiary), 400);
    }
}
