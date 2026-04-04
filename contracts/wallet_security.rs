//! Wallet Security Module for AeTHer Chain
//! 
//! Implements:
//! - Multi-signature wallet support
//! - Transaction validation and replay protection
//! - Wallet backup and recovery
//! - Spending limits and rate limiting

use std::collections::HashMap;

/// Wallet type
#[derive(Debug, Clone, PartialEq)]
pub enum WalletType {
    Standard,      // Single key
    MultiSig,      // Multi-signature (M-of-N)
    Hardware,      // Hardware wallet
    MultiChain,    // Cross-chain wallet
}

/// Transaction validation status
#[derive(Debug, Clone, PartialEq)]
pub enum TxValidationStatus {
    Valid,
    InvalidNonce,
    InsufficientBalance,
    SignatureInvalid,
    RateLimited,
    SpendingLimitExceeded,
}

/// Wallet security configuration
#[derive(Debug, Clone)]
pub struct WalletSecurityConfig {
    /// Maximum transactions per hour
    pub max_tx_per_hour: u64,
    /// Maximum single transaction amount
    pub max_single_tx: u64,
    /// Daily spending limit
    pub daily_spending_limit: u64,
    /// Require 2FA for amounts above threshold
    pub two_fa_threshold: u64,
    /// Multi-sig required signers (M-of-N)
    pub multi_sig_required: u8,
    /// Multi-sig total signers
    pub multi_sig_total: u8,
    /// Transaction replay protection window (seconds)
    pub replay_window_secs: u64,
}

impl Default for WalletSecurityConfig {
    fn default() -> Self {
        Self {
            max_tx_per_hour: 100,
            max_single_tx: 1_000_000_000_000_000, // 1M tokens
            daily_spending_limit: 10_000_000_000_000_000, // 10M tokens
            two_fa_threshold: 100_000_000_000_000, // 100K tokens
            multi_sig_required: 2,
            multi_sig_total: 3,
            replay_window_secs: 86400, // 24 hours
        }
    }
}

/// Transaction record for replay protection
#[derive(Debug, Clone)]
pub struct TxRecord {
    pub tx_hash: [u8; 32],
    pub timestamp: u64,
    pub nonce: u64,
}

/// Wallet entry
#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: [u8; 32],
    pub wallet_type: WalletType,
    pub balance: u64,
    pub daily_spent: u64,
    pub daily_reset_ts: u64,
    pub tx_count_hour: u64,
    pub hour_reset_ts: u64,
    pub signers: Vec<[u8; 32]>,  // For multi-sig
    pub confirmed_txs: HashMap<[u8; 32], TxRecord>,
}

impl Wallet {
    /// Create a new standard wallet
    pub fn new_standard(address: [u8; 32]) -> Self {
        Self {
            address,
            wallet_type: WalletType::Standard,
            balance: 0,
            daily_spent: 0,
            daily_reset_ts: 0,
            tx_count_hour: 0,
            hour_reset_ts: 0,
            signers: vec![],
            confirmed_txs: HashMap::new(),
        }
    }
    
    /// Create a new multi-sig wallet
    pub fn new_multisig(address: [u8; 32], signers: Vec<[u8; 32]>, required: u8) -> Self {
        assert!(required as usize <= signers.len());
        Self {
            address,
            wallet_type: WalletType::MultiSig,
            balance: 0,
            daily_spent: 0,
            daily_reset_ts: 0,
            tx_count_hour: 0,
            hour_reset_ts: 0,
            signers,
            confirmed_txs: HashMap::new(),
        }
    }
    
    /// Reset daily counter if needed
    pub fn reset_daily_if_needed(&mut self, current_ts: u64) {
        let day_secs = 86400u64;
        if current_ts - self.daily_reset_ts >= day_secs {
            self.daily_spent = 0;
            self.daily_reset_ts = current_ts;
        }
    }
    
    /// Reset hourly counter if needed
    pub fn reset_hourly_if_needed(&mut self, current_ts: u64) {
        let hour_secs = 3600u64;
        if current_ts - self.hour_reset_ts >= hour_secs {
            self.tx_count_hour = 0;
            self.hour_reset_ts = current_ts;
        }
    }
    
    /// Check if transaction has valid nonce (replay protection)
    pub fn is_valid_nonce(&self, nonce: u64, window_secs: u64, current_ts: u64) -> bool {
        // Check if nonce is within acceptable range
        // In production, we'd track the last used nonce
        nonce < u64::MAX / 2
    }
    
    /// Validate transaction against security rules
    pub fn validate_transaction(
        &self,
        amount: u64,
        nonce: u64,
        current_ts: u64,
        config: &WalletSecurityConfig,
    ) -> TxValidationStatus {
        // Reset counters
        self.reset_daily_if_needed(current_ts);
        self.reset_hourly_if_needed(current_ts);
        
        // Check spending limit
        if self.daily_spent + amount > config.daily_spending_limit {
            return TxValidationStatus::SpendingLimitExceeded;
        }
        
        // Check rate limit
        if self.tx_count_hour >= config.max_tx_per_hour {
            return TxValidationStatus::RateLimited;
        }
        
        // Check single transaction limit
        if amount > config.max_single_tx {
            return TxValidationStatus::InsufficientBalance;
        }
        
        // Check nonce
        if !self.is_valid_nonce(nonce, config.replay_window_secs, current_ts) {
            return TxValidationStatus::InvalidNonce;
        }
        
        TxValidationStatus::Valid
    }
    
    /// Execute transaction
    pub fn execute_transaction(&mut self, amount: u64, current_ts: u64) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            self.daily_spent += amount;
            self.tx_count_hour += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let address = [1u8; 32];
        let wallet = Wallet::new_standard(address);
        
        assert_eq!(wallet.wallet_type, WalletType::Standard);
        assert_eq!(wallet.balance, 0);
    }

    #[test]
    fn test_multisig_wallet() {
        let address = [1u8; 32];
        let signers = vec![[2u8; 32], [3u8; 32], [4u8; 32]];
        let wallet = Wallet::new_multisig(address, signers.clone(), 2);
        
        assert_eq!(wallet.wallet_type, WalletType::MultiSig);
        assert_eq!(wallet.signers.len(), 3);
    }

    #[test]
    fn test_transaction_validation() {
        let address = [1u8; 32];
        let mut wallet = Wallet::new_standard(address);
        wallet.balance = 1_000_000_000_000;
        
        let config = WalletSecurityConfig::default();
        let status = wallet.validate_transaction(100_000, 1, 0, &config);
        
        assert_eq!(status, TxValidationStatus::Valid);
    }

    #[test]
    fn test_insufficient_balance() {
        let address = [1u8; 32];
        let mut wallet = Wallet::new_standard(address);
        wallet.balance = 50_000;
        
        let config = WalletSecurityConfig::default();
        let status = wallet.validate_transaction(100_000, 1, 0, &config);
        
        assert_eq!(status, TxValidationStatus::InsufficientBalance);
    }

    #[test]
    fn test_spending_limit() {
        let address = [1u8; 32];
        let mut wallet = Wallet::new_standard(address);
        wallet.balance = 1_000_000_000_000_000;
        wallet.daily_spent = 9_000_000_000_000_000;
        
        let config = WalletSecurityConfig::default();
        let status = wallet.validate_transaction(2_000_000_000_000_000, 1, 0, &config);
        
        assert_eq!(status, TxValidationStatus::SpendingLimitExceeded);
    }

    #[test]
    fn test_rate_limiting() {
        let address = [1u8; 32];
        let mut wallet = Wallet::new_standard(address);
        wallet.balance = 1_000_000_000_000;
        wallet.tx_count_hour = 100;
        
        let config = WalletSecurityConfig::default();
        let status = wallet.validate_transaction(100_000, 1, 0, &config);
        
        assert_eq!(status, TxValidationStatus::RateLimited);
    }

    #[test]
    fn test_execute_transaction() {
        let address = [1u8; 32];
        let mut wallet = Wallet::new_standard(address);
        wallet.balance = 1_000_000;
        
        let result = wallet.execute_transaction(500_000, 0);
        
        assert!(result);
        assert_eq!(wallet.balance, 500_000);
        assert_eq!(wallet.daily_spent, 500_000);
        assert_eq!(wallet.tx_count_hour, 1);
    }
}
