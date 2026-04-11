//! Treasury management for Aether DAO
//!
//! Implements:
//! - Multi-sig controlled treasury
//! - Fund allocation and tracking
//! - Withdrawal proposals and execution
//! - Epoch-based budgeting
//! - Transparent on-chain accounting

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

// Custom serde for [u8; 64] arrays (base58 encoded)
mod serde_bytes_64 {
    use super::*;
    
    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&bs58::encode(bytes).into_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let decoded = bs58::decode(&s).into_vec().map_err(serde::de::Error::custom)?;
        let mut arr = [0u8; 64];
        let len = decoded.len().min(64);
        arr[..len].copy_from_slice(&decoded[..len]);
        Ok(arr)
    }
    
    pub mod option {
        use super::*;
        
        pub fn serialize<S>(value: &Option<[u8; 64]>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
            match value {
                Some(bytes) => serializer.serialize_some(&bs58::encode(bytes).into_string()),
                None => serializer.serialize_none(),
            }
        }
        
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 64]>, D::Error>
        where D: Deserializer<'de> {
            use serde::de::Error;
            let opt = Option::<String>::deserialize(deserializer)?;
            match opt {
                Some(s) => {
                    let decoded = bs58::decode(&s).into_vec().map_err(D::Error::custom)?;
                    let mut arr = [0u8; 64];
                    let len = decoded.len().min(64);
                    arr[..len].copy_from_slice(&decoded[..len]);
                    Ok(Some(arr))
                }
                None => Ok(None),
            }
        }
    }
}

/// Treasury configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryConfig {
    /// Minimum signers for multi-sig
    pub min_signers: u8,
    /// Total signers count
    pub total_signers: u8,
    /// Time lock duration for withdrawals (seconds)
    pub timelock_duration: u64,
    /// Maximum single withdrawal (lamports)
    pub max_single_withdrawal: u64,
    /// Daily withdrawal limit (lamports)
    pub daily_withdrawal_limit: u64,
}

impl Default for TreasuryConfig {
    fn default() -> Self {
        Self {
            min_signers: 3,
            total_signers: 5,
            timelock_duration: 86400, // 24 hours
            max_single_withdrawal: 100_000_000_000_000, // 100k AETH
            daily_withdrawal_limit: 500_000_000_000_000, // 500k AETH
        }
    }
}

/// Treasury balance tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TreasuryBalances {
    /// ATH (governance token) balance
    pub ath_balance: u64,
    /// FLUX (utility token) balance  
    pub flux_balance: u64,
    /// Total fees collected (lifetime)
    pub total_fees_collected: u64,
    /// Total distributed (lifetime)
    pub total_distributed: u64,
}

impl TreasuryBalances {
    pub fn new() -> Self {
        Self::default()
    }

    /// Credit ATH to treasury
    pub fn credit_ath(&mut self, amount: u64) {
        self.ath_balance = self.ath_balance.saturating_add(amount);
    }

    /// Debit ATH from treasury
    pub fn debit_ath(&mut self, amount: u64) -> Result<(), TreasuryError> {
        if self.ath_balance < amount {
            return Err(TreasuryError::InsufficientBalance);
        }
        self.ath_balance = self.ath_balance.saturating_sub(amount);
        self.total_distributed = self.total_distributed.saturating_add(amount);
        Ok(())
    }

    /// Credit FLUX to treasury
    pub fn credit_flux(&mut self, amount: u64) {
        self.flux_balance = self.flux_balance.saturating_add(amount);
    }

    /// Debit FLUX from treasury
    pub fn debit_flux(&mut self, amount: u64) -> Result<(), TreasuryError> {
        if self.flux_balance < amount {
            return Err(TreasuryError::InsufficientBalance);
        }
        self.flux_balance = self.flux_balance.saturating_sub(amount);
        self.total_distributed = self.total_distributed.saturating_add(amount);
        Ok(())
    }
}

/// Withdrawal request from treasury
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalRequest {
    /// Unique request ID
    pub id: u64,
    /// Recipient address
    pub recipient: [u8; 32],
    /// Amount to withdraw
    pub amount: u64,
    /// Token type (ATH or FLUX)
    pub token_type: TokenType,
    /// Purpose/reason for withdrawal
    pub purpose: String,
    /// Timestamp when request was created
    pub created_at: u64,
    /// Timestamp when request can be executed (after timelock)
    pub execute_after: u64,
    /// Approvals from signers
    pub approvals: Vec<[u8; 32]>,
    /// Current status
    pub status: WithdrawalStatus,
    /// Transaction hash after execution
    #[serde(with = "serde_bytes_64::option")]
    pub execution_tx: Option<[u8; 64]>,
}

impl WithdrawalRequest {
    /// Create a new withdrawal request
    pub fn new(
        id: u64,
        recipient: [u8; 32],
        amount: u64,
        token_type: TokenType,
        purpose: String,
        created_at: u64,
        timelock_duration: u64,
    ) -> Self {
        Self {
            id,
            recipient,
            amount,
            token_type,
            purpose,
            created_at,
            execute_after: created_at + timelock_duration,
            approvals: Vec::new(),
            status: WithdrawalStatus::Pending,
            execution_tx: None,
        }
    }

    /// Add an approval
    pub fn approve(&mut self, signer: [u8; 32]) -> Result<(), TreasuryError> {
        if self.status != WithdrawalStatus::Pending && self.status != WithdrawalStatus::Approved {
            return Err(TreasuryError::InvalidStatus);
        }
        if self.approvals.contains(&signer) {
            return Err(TreasuryError::AlreadyApproved);
        }
        self.approvals.push(signer);
        Ok(())
    }

    /// Check if request has enough approvals
    pub fn has_approvals(&self, required: u8) -> bool {
        self.approvals.len() >= required as usize
    }

    /// Check if timelock has expired
    pub fn timelock_expired(&self, current_time: u64) -> bool {
        current_time >= self.execute_after
    }

    /// Mark as executed
    pub fn execute(&mut self, tx_hash: [u8; 64]) {
        self.status = WithdrawalStatus::Executed;
        self.execution_tx = Some(tx_hash);
    }

    /// Mark as rejected
    pub fn reject(&mut self) {
        self.status = WithdrawalStatus::Rejected;
    }
}

/// Token type for treasury operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenType {
    ATH,
    FLUX,
}

/// Withdrawal request status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WithdrawalStatus {
    /// Pending approval
    Pending,
    /// Approved by signers, waiting for timelock
    Approved,
    /// Ready to execute
    Ready,
    /// Successfully executed
    Executed,
    /// Rejected by signers
    Rejected,
    /// Cancelled by proposer
    Cancelled,
}

/// Treasury signer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasurySigner {
    /// Signer address
    pub address: [u8; 32],
    /// Whether signer is active
    pub active: bool,
    /// When signer was added
    pub added_at: u64,
}

impl TreasurySigner {
    pub fn new(address: [u8; 32], added_at: u64) -> Self {
        Self {
            address,
            active: true,
            added_at,
        }
    }
}

/// Budget allocation category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    /// Category name
    pub category: String,
    /// Allocated amount for the epoch
    pub allocated: u64,
    /// Spent amount so far
    pub spent: u64,
    /// Remaining amount
    pub remaining: u64,
}

impl BudgetAllocation {
    pub fn new(category: String, allocated: u64) -> Self {
        Self {
            category,
            allocated,
            spent: 0,
            remaining: allocated,
        }
    }

    /// Record spending
    pub fn spend(&mut self, amount: u64) -> Result<(), TreasuryError> {
        if self.remaining < amount {
            return Err(TreasuryError::BudgetExceeded);
        }
        self.spent = self.spent.saturating_add(amount);
        self.remaining = self.remaining.saturating_sub(amount);
        Ok(())
    }
}

/// Aether Treasury
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treasury {
    /// Treasury configuration
    pub config: TreasuryConfig,
    /// Current balances
    pub balances: TreasuryBalances,
    /// Authorized signers (multi-sig)
    pub signers: HashMap<[u8; 32], TreasurySigner>,
    /// Pending withdrawal requests
    pub withdrawals: HashMap<u64, WithdrawalRequest>,
    /// Next withdrawal ID
    pub next_withdrawal_id: u64,
    /// Budget allocations by category
    pub budgets: HashMap<String, BudgetAllocation>,
    /// Daily withdrawal tracking
    pub daily_withdrawn: u64,
    pub daily_reset_time: u64,
    /// Current epoch
    pub current_epoch: u64,
}

impl Treasury {
    /// Create a new treasury
    pub fn new(config: TreasuryConfig) -> Self {
        Self {
            config,
            balances: TreasuryBalances::new(),
            signers: HashMap::new(),
            withdrawals: HashMap::new(),
            next_withdrawal_id: 1,
            budgets: HashMap::new(),
            daily_withdrawn: 0,
            daily_reset_time: 0,
            current_epoch: 0,
        }
    }

    /// Initialize with default config
    pub fn with_default_config() -> Self {
        Self::new(TreasuryConfig::default())
    }

    /// Add a signer
    pub fn add_signer(&mut self, address: [u8; 32], timestamp: u64) -> Result<(), TreasuryError> {
        if self.signers.len() >= self.config.total_signers as usize {
            return Err(TreasuryError::TooManySigners);
        }
        self.signers.insert(address, TreasurySigner::new(address, timestamp));
        Ok(())
    }

    /// Remove a signer
    pub fn remove_signer(&mut self, address: [u8; 32]) -> Result<(), TreasuryError> {
        if self.signers.len() <= self.config.min_signers as usize {
            return Err(TreasuryError::TooFewSigners);
        }
        self.signers.remove(&address);
        Ok(())
    }

    /// Check if address is an active signer
    pub fn is_signer(&self, address: &[u8; 32]) -> bool {
        self.signers.get(address).map(|s| s.active).unwrap_or(false)
    }

    /// Create a withdrawal request
    pub fn create_withdrawal(
        &mut self,
        recipient: [u8; 32],
        amount: u64,
        token_type: TokenType,
        purpose: String,
        timestamp: u64,
    ) -> Result<u64, TreasuryError> {
        // Check amount limits
        if amount > self.config.max_single_withdrawal {
            return Err(TreasuryError::AmountTooLarge);
        }

        // Check daily limit
        if timestamp - self.daily_reset_time > 86400 {
            // Reset daily counter
            self.daily_withdrawn = 0;
            self.daily_reset_time = timestamp;
        }
        if self.daily_withdrawn + amount > self.config.daily_withdrawal_limit {
            return Err(TreasuryError::DailyLimitExceeded);
        }

        // Check balance
        match token_type {
            TokenType::ATH if self.balances.ath_balance < amount => {
                return Err(TreasuryError::InsufficientBalance);
            }
            TokenType::FLUX if self.balances.flux_balance < amount => {
                return Err(TreasuryError::InsufficientBalance);
            }
            _ => {}
        }

        let request = WithdrawalRequest::new(
            self.next_withdrawal_id,
            recipient,
            amount,
            token_type,
            purpose,
            timestamp,
            self.config.timelock_duration,
        );

        let id = self.next_withdrawal_id;
        self.next_withdrawal_id += 1;
        self.withdrawals.insert(id, request);

        Ok(id)
    }

    /// Approve a withdrawal request
    pub fn approve_withdrawal(
        &mut self,
        withdrawal_id: u64,
        signer: [u8; 32],
    ) -> Result<(), TreasuryError> {
        if !self.is_signer(&signer) {
            return Err(TreasuryError::Unauthorized);
        }

        let request = self.withdrawals.get_mut(&withdrawal_id)
            .ok_or(TreasuryError::WithdrawalNotFound)?;

        request.approve(signer)?;

        // Check if enough approvals
        if request.has_approvals(self.config.min_signers) {
            request.status = WithdrawalStatus::Approved;
        }

        Ok(())
    }

    /// Execute a withdrawal after timelock
    pub fn execute_withdrawal(
        &mut self,
        withdrawal_id: u64,
        current_time: u64,
        tx_hash: [u8; 64],
    ) -> Result<(), TreasuryError> {
        let request = self.withdrawals.get_mut(&withdrawal_id)
            .ok_or(TreasuryError::WithdrawalNotFound)?;

        // Check approvals
        if !request.has_approvals(self.config.min_signers) {
            return Err(TreasuryError::InsufficientApprovals);
        }

        // Check timelock
        if !request.timelock_expired(current_time) {
            return Err(TreasuryError::TimelockNotExpired);
        }

        // Check daily limit again
        if current_time - self.daily_reset_time > 86400 {
            self.daily_withdrawn = 0;
            self.daily_reset_time = current_time;
        }
        if self.daily_withdrawn + request.amount > self.config.daily_withdrawal_limit {
            return Err(TreasuryError::DailyLimitExceeded);
        }

        // Deduct from balance
        match request.token_type {
            TokenType::ATH => self.balances.debit_ath(request.amount)?,
            TokenType::FLUX => self.balances.debit_flux(request.amount)?,
        }

        // Update daily tracking
        self.daily_withdrawn += request.amount;

        // Mark as executed
        request.execute(tx_hash);

        Ok(())
    }

    /// Set budget allocation for a category
    pub fn set_budget(&mut self, category: String, amount: u64) {
        self.budgets.insert(category.clone(), BudgetAllocation::new(category, amount));
    }

    /// Spend from a budget category
    pub fn spend_budget(&mut self, category: &str, amount: u64) -> Result<(), TreasuryError> {
        let budget = self.budgets.get_mut(category)
            .ok_or(TreasuryError::BudgetNotFound)?;
        budget.spend(amount)
    }

    /// Get budget status for all categories
    pub fn get_budget_status(&self) -> Vec<(String, u64, u64, u64)> {
        self.budgets.iter()
            .map(|(name, budget)| (name.clone(), budget.allocated, budget.spent, budget.remaining))
            .collect()
    }

    /// Advance epoch - reset budgets
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
        // Reset budgets for new epoch
        for budget in self.budgets.values_mut() {
            budget.spent = 0;
            budget.remaining = budget.allocated;
        }
    }

    /// Get withdrawal by ID
    pub fn get_withdrawal(&self, id: u64) -> Option<&WithdrawalRequest> {
        self.withdrawals.get(&id)
    }

    /// Get all pending withdrawals
    pub fn get_pending_withdrawals(&self) -> Vec<&WithdrawalRequest> {
        self.withdrawals.values()
            .filter(|w| w.status == WithdrawalStatus::Pending || w.status == WithdrawalStatus::Approved)
            .collect()
    }

    /// Get treasury summary
    pub fn summary(&self) -> TreasurySummary {
        TreasurySummary {
            ath_balance: self.balances.ath_balance,
            flux_balance: self.balances.flux_balance,
            total_fees_collected: self.balances.total_fees_collected,
            total_distributed: self.balances.total_distributed,
            pending_withdrawals: self.withdrawals.len() as u64,
            signer_count: self.signers.len() as u8,
            current_epoch: self.current_epoch,
        }
    }
}

/// Treasury summary for quick viewing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasurySummary {
    pub ath_balance: u64,
    pub flux_balance: u64,
    pub total_fees_collected: u64,
    pub total_distributed: u64,
    pub pending_withdrawals: u64,
    pub signer_count: u8,
    pub current_epoch: u64,
}

/// Treasury errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreasuryError {
    InsufficientBalance,
    Unauthorized,
    AlreadyApproved,
    InvalidStatus,
    WithdrawalNotFound,
    InsufficientApprovals,
    TimelockNotExpired,
    AmountTooLarge,
    DailyLimitExceeded,
    TooManySigners,
    TooFewSigners,
    BudgetExceeded,
    BudgetNotFound,
}

impl std::fmt::Display for TreasuryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreasuryError::InsufficientBalance => write!(f, "Insufficient balance"),
            TreasuryError::Unauthorized => write!(f, "Unauthorized signer"),
            TreasuryError::AlreadyApproved => write!(f, "Already approved"),
            TreasuryError::InvalidStatus => write!(f, "Invalid withdrawal status"),
            TreasuryError::WithdrawalNotFound => write!(f, "Withdrawal not found"),
            TreasuryError::InsufficientApprovals => write!(f, "Insufficient approvals"),
            TreasuryError::TimelockNotExpired => write!(f, "Timelock not expired"),
            TreasuryError::AmountTooLarge => write!(f, "Amount exceeds maximum"),
            TreasuryError::DailyLimitExceeded => write!(f, "Daily withdrawal limit exceeded"),
            TreasuryError::TooManySigners => write!(f, "Maximum signers reached"),
            TreasuryError::TooFewSigners => write!(f, "Minimum signers required"),
            TreasuryError::BudgetExceeded => write!(f, "Budget exceeded"),
            TreasuryError::BudgetNotFound => write!(f, "Budget category not found"),
        }
    }
}

impl std::error::Error for TreasuryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treasury_creation() {
        let treasury = Treasury::with_default_config();
        assert_eq!(treasury.config.min_signers, 3);
        assert_eq!(treasury.balances.ath_balance, 0);
    }

    #[test]
    fn test_add_signer() {
        let mut treasury = Treasury::with_default_config();
        treasury.add_signer([1u8; 32], 1000).unwrap();
        
        assert!(treasury.is_signer(&[1u8; 32]));
        assert!(!treasury.is_signer(&[2u8; 32]));
    }

    #[test]
    fn test_withdrawal_flow() {
        let mut treasury = Treasury::with_default_config();
        
        // Add signers
        for i in 1..=3 {
            treasury.add_signer([i; 32], 1000).unwrap();
        }
        
        // Credit balance
        treasury.balances.credit_ath(1_000_000);
        
        // Create withdrawal
        let request_id = treasury.create_withdrawal(
            [9u8; 32],
            100_000,
            TokenType::ATH,
            "Test withdrawal".to_string(),
            2000,
        ).unwrap();
        
        // Approve by 3 signers
        for i in 1u8..=3 {
            treasury.approve_withdrawal(request_id, [i; 32]).unwrap();
        }
        
        // Check status
        let request = treasury.get_withdrawal(request_id).unwrap();
        assert_eq!(request.status, WithdrawalStatus::Approved);
        
        // Execute after timelock (86400 seconds)
        treasury.execute_withdrawal(request_id, 2000 + 86400, [0u8; 64]).unwrap();
        
        // Check balance
        assert_eq!(treasury.balances.ath_balance, 900_000);
    }

    #[test]
    fn test_daily_limit() {
        let config = TreasuryConfig {
            daily_withdrawal_limit: 1000,
            min_signers: 1,
            timelock_duration: 0, // No timelock for test
            ..Default::default()
        };
        let mut treasury = Treasury::new(config);
        
        treasury.add_signer([1u8; 32], 1000).unwrap();
        treasury.balances.credit_ath(1_000_000);
        
        // First withdrawal within limit
        let id1 = treasury.create_withdrawal(
            [9u8; 32], 600, TokenType::ATH, "First".to_string(), 1000
        ).unwrap();
        
        // Approve and execute first withdrawal
        treasury.approve_withdrawal(id1, [1u8; 32]).unwrap();
        treasury.execute_withdrawal(id1, 1000, [0u8; 64]).unwrap();
        
        // Now daily_withdrawn is 600, second withdrawal of 500 (total 1100) should fail
        let result = treasury.create_withdrawal(
            [9u8; 32], 500, TokenType::ATH, "Second".to_string(), 1000
        );
        assert_eq!(result, Err(TreasuryError::DailyLimitExceeded));
    }

    #[test]
    fn test_budget_tracking() {
        let mut treasury = Treasury::with_default_config();
        
        treasury.set_budget("development".to_string(), 1000);
        treasury.spend_budget("development", 300).unwrap();
        
        let status = treasury.get_budget_status();
        assert_eq!(status[0], ("development".to_string(), 1000, 300, 700));
    }

    #[test]
    fn test_timelock() {
        let mut treasury = Treasury::with_default_config();
        treasury.config.timelock_duration = 100;
        
        for i in 1..=3 {
            treasury.add_signer([i; 32], 1000).unwrap();
        }
        treasury.balances.credit_ath(1_000_000);
        
        let request_id = treasury.create_withdrawal(
            [9u8; 32], 1000, TokenType::ATH, "Test".to_string(), 1000
        ).unwrap();
        
        for i in 1u8..=3 {
            treasury.approve_withdrawal(request_id, [i; 32]).unwrap();
        }
        
        // Try to execute before timelock
        let result = treasury.execute_withdrawal(request_id, 1050, [0u8; 64]);
        assert_eq!(result, Err(TreasuryError::TimelockNotExpired));
        
        // Execute after timelock
        treasury.execute_withdrawal(request_id, 1101, [0u8; 64]).unwrap();
    }
}