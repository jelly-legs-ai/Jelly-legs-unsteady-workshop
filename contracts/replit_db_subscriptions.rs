// Replit DB Integration - AeTHer Chain
// Subscription management and recurring billing for agent services

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Subscription tiers for agent services
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionTier {
    Free,
    Basic,
    Professional,
    Enterprise,
    Custom,
}

impl SubscriptionTier {
    pub fn monthly_price_flux(&self) -> u64 {
        match self {
            SubscriptionTier::Free => 0,
            SubscriptionTier::Basic => 500,
            SubscriptionTier::Professional => 2000,
            SubscriptionTier::Enterprise => 10000,
            SubscriptionTier::Custom => 0, // Custom pricing
        }
    }
    
    pub fn rate_limit_per_minute(&self) -> u32 {
        match self {
            SubscriptionTier::Free => 10,
            SubscriptionTier::Basic => 60,
            SubscriptionTier::Professional => 300,
            SubscriptionTier::Enterprise => 1000,
            SubscriptionTier::Custom => 5000,
        }
    }
    
    pub fn max_agents(&self) -> u32 {
        match self {
            SubscriptionTier::Free => 1,
            SubscriptionTier::Basic => 5,
            SubscriptionTier::Professional => 20,
            SubscriptionTier::Enterprise => 100,
            SubscriptionTier::Custom => u32::MAX,
        }
    }
    
    pub fn features(&self) -> Vec<String> {
        match self {
            SubscriptionTier::Free => vec!["Basic API access".to_string()],
            SubscriptionTier::Basic => vec![
                "API access".to_string(),
                "5 agents".to_string(),
                "Email support".to_string(),
                "Basic analytics".to_string(),
            ],
            SubscriptionTier::Professional => vec![
                "Priority API".to_string(),
                "20 agents".to_string(),
                "Priority support".to_string(),
                "Advanced analytics".to_string(),
                "Custom integrations".to_string(),
                "SLA guarantee".to_string(),
            ],
            SubscriptionTier::Enterprise => vec![
                "Dedicated infrastructure".to_string(),
                "100 agents".to_string(),
                "24/7 support".to_string(),
                "Enterprise analytics".to_string(),
                "Custom SLA".to_string(),
                "On-premise deployment".to_string(),
                "White-label option".to_string(),
            ],
            SubscriptionTier::Custom => vec!["Custom features".to_string()],
        }
    }
}

/// Subscription status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Trial,
    Expired,
    Cancelled,
    PastDue,
    Suspended,
}

/// Recurring billing cycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BillingCycle {
    Monthly,
    Quarterly,
    Yearly,
    Lifetime,
}

impl BillingCycle {
    pub fn discount_percent(&self) -> f64 {
        match self {
            BillingCycle::Monthly => 0.0,
            BillingCycle::Quarterly => 10.0, // 10% discount
            BillingCycle::Yearly => 20.0,   // 20% discount
            BillingCycle::Lifetime => 50.0, // 50% discount
        }
    }
    
    pub fn epochs_duration(&self) -> u64 {
        match self {
            BillingCycle::Monthly => 720,    // 30 days * 24 epochs
            BillingCycle::Quarterly => 2160, // 90 days
            BillingCycle::Yearly => 8760,    // 365 days
            BillingCycle::Lifetime => u64::MAX,
        }
    }
}

/// Subscription record stored in Replit DB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub subscription_id: String,
    pub user_address: String,
    pub agent_id: Option<String>,
    pub tier: SubscriptionTier,
    pub status: SubscriptionStatus,
    pub billing_cycle: BillingCycle,
    pub started_at: u64,
    pub current_period_start: u64,
    pub current_period_end: u64,
    pub trial_end: Option<u64>,
    pub cancel_at_period_end: bool,
    pub cancelled_at: Option<u64>,
    pub flux_paid_total: u64,
    pub last_payment_epoch: u64,
    pub next_billing_epoch: u64,
    pub payment_method: String,
    pub auto_renew: bool,
    pub metadata: HashMap<String, String>,
}

/// Usage metrics for billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub user_address: String,
    pub epoch: u64,
    pub api_calls: u64,
    pub agent_tasks: u64,
    pub compute_units: u64,
    pub storage_mb: u64,
    pub bandwidth_mb: u64,
    pub premium_features_used: Vec<String>,
}

/// Replit DB connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplitDbConfig {
    pub database_url: String,
    pub api_key: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub batch_size: u32,
}

/// Subscription persistence layer for Replit DB
pub struct ReplitDbSubscriptions {
    config: ReplitDbConfig,
}

impl ReplitDbSubscriptions {
    /// Create new Replit DB subscription manager
    pub fn new(config: ReplitDbConfig) -> Self {
        Self { config }
    }
    
    /// Store subscription in Replit DB
    pub async fn store_subscription(&self, subscription: &Subscription) -> Result<String, DbError> {
        // Key format: subscription:{subscription_id}
        let key = format!("subscription:{}", subscription.subscription_id);
        // In production: use Replit DB HTTP API
        // POST to database_url with key/value
        Ok(key)
    }
    
    /// Retrieve subscription from Replit DB
    pub async fn get_subscription(&self, subscription_id: &str) -> Result<Subscription, DbError> {
        let key = format!("subscription:{}", subscription_id);
        // In production: GET from Replit DB
        // Return deserialized Subscription
        Err(DbError::NotFound)
    }
    
    /// Get all subscriptions for a user
    pub async fn get_user_subscriptions(&self, user_address: &str) -> Result<Vec<Subscription>, DbError> {
        // Query prefix: subscription:* where user_address matches
        // Return filtered list
        Ok(vec![])
    }
    
    /// Update subscription status
    pub async fn update_status(&self, subscription_id: &str, status: SubscriptionStatus) -> Result<(), DbError> {
        // Get existing, update status, store back
        Ok(())
    }
    
    /// Store usage metrics for billing period
    pub async fn store_usage(&self, usage: &UsageMetrics) -> Result<String, DbError> {
        let key = format!("usage:{}:{}", usage.user_address, usage.epoch);
        Ok(key)
    }
    
    /// Get aggregated usage for billing period
    pub async fn get_period_usage(&self, user_address: &str, start_epoch: u64, end_epoch: u64) -> Result<AggregatedUsage, DbError> {
        // Query all usage:*:epoch keys in range
        // Sum api_calls, agent_tasks, compute_units, storage_mb
        Ok(AggregatedUsage {
            total_api_calls: 0,
            total_agent_tasks: 0,
            total_compute_units: 0,
            total_storage_mb: 0,
            epochs_counted: 0,
        })
    }
    
    /// Check if subscription is active and valid
    pub async fn validate_subscription(&self, subscription_id: &str) -> Result<ValidationResult, DbError> {
        let subscription = self.get_subscription(subscription_id).await?;
        
        let is_active = subscription.status == SubscriptionStatus::Active;
        let is_not_expired = subscription.current_period_end > Self::current_epoch();
        let is_within_limits = true; // Check usage vs tier limits
        
        Ok(ValidationResult {
            is_valid: is_active && is_not_expired && is_within_limits,
            subscription,
            remaining_quota: 1000,
            reset_epoch: 0,
        })
    }
    
    /// Process recurring billing for subscription
    pub async fn process_billing(&self, subscription_id: &str) -> Result<BillingResult, DbError> {
        // Get subscription
        // Calculate amount due (apply discount for billing cycle)
        // Charge user's payment method
        // Update current_period_start/end
        // Update next_billing_epoch
        // Record transaction
        Ok(BillingResult {
            success: true,
            amount_charged: 0,
            new_period_end: 0,
            transaction_id: String::new(),
        })
    }
    
    /// Cancel subscription at period end
    pub async fn cancel_subscription(&self, subscription_id: &str, immediate: bool) -> Result<(), DbError> {
        // If immediate: set status = Cancelled, end current period
        // If not immediate: set cancel_at_period_end = true
        Ok(())
    }
    
    /// Get subscriptions due for renewal
    pub async fn get_renewals_due(&self, epoch: u64) -> Result<Vec<Subscription>, DbError> {
        // Query all subscriptions where next_billing_epoch <= epoch
        // Filter out those with cancel_at_period_end = true
        Ok(vec![])
    }
    
    /// Batch process renewals (gas-optimized)
    pub async fn batch_process_renewals(&self, subscription_ids: &[String]) -> Result<BatchBillingResult, DbError> {
        let mut results = Vec::new();
        let mut total_charged = 0u64;
        let mut successes = 0u32;
        let mut failures = 0u32;
        
        for id in subscription_ids {
            match self.process_billing(id).await {
                Ok(result) => {
                    if result.success {
                        successes += 1;
                        total_charged += result.amount_charged;
                    } else {
                        failures += 1;
                    }
                    results.push(result);
                }
                Err(_) => failures += 1,
            }
        }
        
        Ok(BatchBillingResult {
            total_processed: subscription_ids.len() as u32,
            successes,
            failures,
            total_amount_charged: total_charged,
            results,
        })
    }
    
    /// Clean up expired subscriptions (cron job)
    pub async fn cleanup_expired(&self) -> Result<u32, DbError> {
        // Find all subscriptions where current_period_end < current_epoch
        // And status != Cancelled
        // Set status = Expired
        // Return count of expired subscriptions
        Ok(0)
    }
    
    fn current_epoch() -> u64 {
        // In production: get from chain state
        48291
    }
}

/// Aggregated usage for billing period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedUsage {
    pub total_api_calls: u64,
    pub total_agent_tasks: u64,
    pub total_compute_units: u64,
    pub total_storage_mb: u64,
    pub epochs_counted: u64,
}

/// Subscription validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub subscription: Subscription,
    pub remaining_quota: u64,
    pub reset_epoch: u64,
}

/// Single billing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingResult {
    pub success: bool,
    pub amount_charged: u64,
    pub new_period_end: u64,
    pub transaction_id: String,
    pub error_message: Option<String>,
}

/// Batch billing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchBillingResult {
    pub total_processed: u32,
    pub successes: u32,
    pub failures: u32,
    pub total_amount_charged: u64,
    pub results: Vec<BillingResult>,
}

/// Database operation errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl DbError {
    pub fn not_found() -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: "Resource not found in database".to_string(),
            retryable: false,
        }
    }
    
    pub fn connection_failed() -> Self {
        Self {
            code: "CONNECTION_FAILED".to_string(),
            message: "Failed to connect to Replit DB".to_string(),
            retryable: true,
        }
    }
    
    pub fn timeout() -> Self {
        Self {
            code: "TIMEOUT".to_string(),
            message: "Database operation timed out".to_string(),
            retryable: true,
        }
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}
    pub bandwidth_mb: u64,
    pub overage_charges: u64,
}

/// Invoice record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice_id: String,
    pub subscription_id: String,
    pub user_address: String,
    pub amount_flux: u64,
    pub status: InvoiceStatus,
    pub created_at: u64,
    pub due_at: u64,
    pub paid_at: Option<u64>,
    pub billing_period_start: u64,
    pub billing_period_end: u64,
    pub line_items: Vec<LineItem>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Uncollectible,
    Void,
}

/// Line item for invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub description: String,
    pub amount_flux: u64,
    pub quantity: u64,
    pub unit_price: u64,
    pub prorated: bool,
}

/// Payment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub payment_id: String,
    pub invoice_id: String,
    pub subscription_id: String,
    pub user_address: String,
    pub amount_flux: u64,
    pub status: PaymentStatus,
    pub transaction_hash: String,
    pub created_at: u64,
    pub confirmed_at: Option<u64>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
}

/// Replit DB subscription manager
pub struct SubscriptionManager {
    pub subscriptions: HashMap<String, Subscription>,
    pub invoices: HashMap<String, Invoice>,
    pub payments: HashMap<String, Payment>,
    pub usage_metrics: HashMap<String, UsageMetrics>,
    pub current_epoch: u64,
    pub db_prefix: String,
}

impl SubscriptionManager {
    /// Create new subscription manager
    pub fn new(db_prefix: &str) -> Self {
        SubscriptionManager {
            subscriptions: HashMap::new(),
            invoices: HashMap::new(),
            payments: HashMap::new(),
            usage_metrics: HashMap::new(),
            current_epoch: 0,
            db_prefix: db_prefix.to_string(),
        }
    }
    
    /// Create new subscription
    pub fn create_subscription(&mut self, user_address: &str, tier: SubscriptionTier, billing_cycle: BillingCycle) -> Result<Subscription, String> {
        let subscription_id = format!("sub_{}_{}", user_address, self.current_epoch);
        
        let trial_end = if tier == SubscriptionTier::Professional || tier == SubscriptionTier::Enterprise {
            Some(self.current_epoch + 336) // 14 day trial
        } else {
            None
        };
        
        let period_end = self.current_epoch + billing_cycle.epochs_duration();
        
        let subscription = Subscription {
            subscription_id: subscription_id.clone(),
            user_address: user_address.to_string(),
            agent_id: None,
            tier: tier.clone(),
            status: if trial_end.is_some() { SubscriptionStatus::Trial } else { SubscriptionStatus::Active },
            billing_cycle: billing_cycle.clone(),
            started_at: self.current_epoch,
            current_period_start: self.current_epoch,
            current_period_end: period_end,
            trial_end,
            cancel_at_period_end: false,
            cancelled_at: None,
            flux_paid_total: 0,
            last_payment_epoch: 0,
            next_billing_epoch: period_end,
            payment_method: "FLUX".to_string(),
            auto_renew: true,
            metadata: HashMap::new(),
        };
        
        self.subscriptions.insert(subscription_id.clone(), subscription.clone());
        
        // Store in Replit DB (simulated)
        self.db_put(&format!("subscriptions/{}", subscription_id), &subscription);
        
        Ok(subscription)
    }
    
    /// Upgrade subscription tier
    pub fn upgrade_subscription(&mut self, subscription_id: &str, new_tier: SubscriptionTier) -> Result<(), String> {
        let subscription = self.subscriptions.get_mut(subscription_id)
            .ok_or("Subscription not found")?;
        
        let old_tier = subscription.tier.clone();
        let old_price = old_tier.monthly_price_flux();
        let new_price = new_tier.monthly_price_flux();
        
        if new_price <= old_price {
            return Err("New tier must be higher than current tier");
        }
        
        // Prorate the upgrade
        let epochs_remaining = subscription.current_period_end - self.current_epoch;
        let total_epochs = subscription.billing_cycle.epochs_duration();
        let prorated_amount = ((new_price - old_price) as f64 * epochs_remaining as f64 / total_epochs as f64) as u64;
        
        subscription.tier = new_tier;
        subscription.metadata.insert("upgraded_from".to_string(), format!("{:?}", old_tier));
        subscription.metadata.insert("upgrade_epoch".to_string(), self.current_epoch.to_string());
        subscription.metadata.insert("prorated_amount".to_string(), prorated_amount.to_string());
        
        // Create prorated invoice
        self.create_prorated_invoice(subscription_id, prorated_amount, "Tier upgrade");
        
        Ok(())
    }
    
    /// Cancel subscription
    pub fn cancel_subscription(&mut self, subscription_id: &str, immediate: bool) -> Result<(), String> {
        let subscription = self.subscriptions.get_mut(subscription_id)
            .ok_or("Subscription not found")?;
        
        if immediate {
            subscription.status = SubscriptionStatus::Cancelled;
            subscription.cancelled_at = Some(self.current_epoch);
            subscription.cancel_at_period_end = false;
        } else {
            subscription.cancel_at_period_end = true;
            subscription.cancelled_at = Some(self.current_epoch);
        }
        
        Ok(())
    }
    
    /// Reactivate cancelled subscription
    pub fn reactivate_subscription(&mut self, subscription_id: &str) -> Result<(), String> {
        let subscription = self.subscriptions.get_mut(subscription_id)
            .ok_or("Subscription not found")?;
        
        if subscription.status != SubscriptionStatus::Cancelled {
            return Err("Subscription is not cancelled");
        }
        
        subscription.cancel_at_period_end = false;
        subscription.cancelled_at = None;
        subscription.status = SubscriptionStatus::Active;
        
        Ok(())
    }
    
    /// Process recurring billing
    pub fn process_recurring_billing(&mut self) -> Result<u64, String> {
        let mut total_collected = 0u64;
        
        for (subscription_id, subscription) in &mut self.subscriptions.iter_mut() {
            if subscription.status == SubscriptionStatus::Active || subscription.status == SubscriptionStatus::PastDue {
                if self.current_epoch >= subscription.next_billing_epoch {
                    let amount = subscription.tier.monthly_price_flux();
                    
                    // Apply billing cycle discount
                    let discount = subscription.billing_cycle.discount_percent();
                    let discounted_amount = (amount as f64 * (1.0 - discount / 100.0)) as u64;
                    
                    // Create invoice
                    let invoice = self.create_invoice(
                        subscription_id,
                        discounted_amount,
                        "Subscription renewal",
                    );
                    
                    // Simulate payment (in production, would trigger on-chain payment)
                    let payment = self.process_payment(&invoice.invoice_id, subscription.user_address.clone(), discounted_amount);
                    
                    if payment.status == PaymentStatus::Completed {
                        subscription.flux_paid_total += discounted_amount;
                        subscription.last_payment_epoch = self.current_epoch;
                        subscription.next_billing_epoch = self.current_epoch + subscription.billing_cycle.epochs_duration();
                        subscription.current_period_start = self.current_epoch;
                        subscription.current_period_end = self.current_epoch + subscription.billing_cycle.epochs_duration();
                        subscription.status = SubscriptionStatus::Active;
                        total_collected += discounted_amount;
                    } else {
                        subscription.status = SubscriptionStatus::PastDue;
                    }
                }
            }
        }
        
        Ok(total_collected)
    }
    
    /// Record usage metrics
    pub fn record_usage(&mut self, user_address: &str, metrics: UsageMetrics) {
        self.usage_metrics.insert(format!("{}-{}", user_address, metrics.epoch), metrics);
        self.db_put(&format!("usage/{}/{}", user_address, metrics.epoch), &metrics);
    }
    
    /// Get usage for billing period
    pub fn get_period_usage(&self, user_address: &str, start_epoch: u64, end_epoch: u64) -> UsageMetrics {
        let mut total = UsageMetrics {
            user_address: user_address.to_string(),
            epoch: 0,
            api_calls: 0,
            agent_tasks: 0,
            compute_units: 0,
            storage_mb: 0,
            bandwidth_mb: 0,
            overage_charges: 0,
        };
        
        for (key, metrics) in &self.usage_metrics {
            if key.starts_with(user_address) {
                if metrics.epoch >= start_epoch && metrics.epoch <= end_epoch {
                    total.api_calls += metrics.api_calls;
                    total.agent_tasks += metrics.agent_tasks;
                    total.compute_units += metrics.compute_units;
                    total.storage_mb = total.storage_mb.max(metrics.storage_mb);
                    total.bandwidth_mb += metrics.bandwidth_mb;
                }
            }
        }
        
        total
    }
    
    /// Calculate overage charges
    pub fn calculate_overage(&self, subscription: &Subscription, usage: &UsageMetrics) -> u64 {
        let rate_limit = subscription.tier.rate_limit_per_minute() * 60 * 24; // Daily limit
        let overage = if usage.api_calls > rate_limit {
            usage.api_calls - rate_limit
        } else {
            0
        };
        
        // Charge 0.001 FLUX per overage API call
        overage / 1000
    }
    
    /// Check if subscription is active
    pub fn is_active(&self, subscription_id: &str) -> bool {
        if let Some(sub) = self.subscriptions.get(subscription_id) {
            if sub.status == SubscriptionStatus::Active {
                return self.current_epoch < sub.current_period_end;
            }
            if sub.status == SubscriptionStatus::Trial {
                if let Some(trial_end) = sub.trial_end {
                    return self.current_epoch < trial_end;
                }
            }
        }
        false
    }
    
    /// Get subscription by user address
    pub fn get_user_subscription(&self, user_address: &str) -> Option<&Subscription> {
        self.subscriptions.values().find(|s| s.user_address == user_address)
    }
    
    /// Get all active subscriptions
    pub fn get_active_subscriptions(&self) -> Vec<&Subscription> {
        self.subscriptions.values()
            .filter(|s| s.status == SubscriptionStatus::Active || s.status == SubscriptionStatus::Trial)
            .collect()
    }
    
    /// Get MRR (Monthly Recurring Revenue) in FLUX
    pub fn get_mrr(&self) -> u64 {
        self.subscriptions.values()
            .filter(|s| s.status == SubscriptionStatus::Active)
            .map(|s| s.tier.monthly_price_flux())
            .sum()
    }
    
    /// Get ARR (Annual Recurring Revenue) in FLUX
    pub fn get_arr(&self) -> u64 {
        self.get_mrr() * 12
    }
    
    /// Simulate Replit DB put
    fn db_put(&self, key: &str, value: &impl Serialize) {
        // In production: await db.set(key, JSON.stringify(value))
        println!("Replit DB: SET {} = {:?}", key, value);
    }
    
    /// Simulate Replit DB get
    #[allow(dead_code)]
    fn db_get(&self, key: &str) -> Option<String> {
        // In production: await db.get(key)
        println!("Replit DB: GET {}", key);
        None
    }
    
    /// Create invoice
    fn create_invoice(&mut self, subscription_id: &str, amount: u64, description: &str) -> Invoice {
        let invoice_id = format!("inv_{}_{}", subscription_id, self.current_epoch);
        let subscription = &self.subscriptions[subscription_id];
        
        let invoice = Invoice {
            invoice_id: invoice_id.clone(),
            subscription_id: subscription_id.to_string(),
            user_address: subscription.user_address.clone(),
            amount_flux: amount,
            status: InvoiceStatus::Open,
            created_at: self.current_epoch,
            due_at: self.current_epoch + 720, // 30 days
            paid_at: None,
            billing_period_start: subscription.current_period_start,
            billing_period_end: subscription.current_period_end,
            line_items: vec![LineItem {
                description: description.to_string(),
                amount_flux: amount,
                quantity: 1,
                unit_price: amount,
                prorated: false,
            }],
            metadata: HashMap::new(),
        };
        
        self.invoices.insert(invoice_id.clone(), invoice.clone());
        self.db_put(&format!("invoices/{}", invoice_id), &invoice);
        
        invoice
    }
    
    /// Create prorated invoice for upgrades
    fn create_prorated_invoice(&mut self, subscription_id: &str, amount: u64, description: &str) -> Invoice {
        let invoice_id = format!("inv_prorate_{}_{}", subscription_id, self.current_epoch);
        let subscription = &self.subscriptions[subscription_id];
        
        let invoice = Invoice {
            invoice_id: invoice_id.clone(),
            subscription_id: subscription_id.to_string(),
            user_address: subscription.user_address.clone(),
            amount_flux: amount,
            status: InvoiceStatus::Open,
            created_at: self.current_epoch,
            due_at: self.current_epoch + 720,
            paid_at: None,
            billing_period_start: subscription.current_period_start,
            billing_period_end: subscription.current_period_end,
            line_items: vec![LineItem {
                description: description.to_string(),
                amount_flux: amount,
                quantity: 1,
                unit_price: amount,
                prorated: true,
            }],
            metadata: HashMap::new(),
        };
        
        self.invoices.insert(invoice_id.clone(), invoice.clone());
        self.db_put(&format!("invoices/{}", invoice_id), &invoice);
        
        invoice
    }
    
    /// Process payment
    fn process_payment(&mut self, invoice_id: &str, user_address: String, amount: u64) -> Payment {
        let payment_id = format!("pay_{}_{}", invoice_id, self.current_epoch);
        let invoice = &self.invoices[invoice_id];
        
        let payment = Payment {
            payment_id: payment_id.clone(),
            invoice_id: invoice_id.to_string(),
            subscription_id: invoice.subscription_id.clone(),
            user_address,
            amount_flux: amount,
            status: PaymentStatus::Completed, // Simulated success
            transaction_hash: format!("0x{}", payment_id),
            created_at: self.current_epoch,
            confirmed_at: Some(self.current_epoch),
            failure_reason: None,
        };
        
        self.payments.insert(payment_id.clone(), payment.clone());
        self.db_put(&format!("payments/{}", payment_id), &payment);
        
        // Update invoice status
        if let Some(inv) = self.invoices.get_mut(invoice_id) {
            inv.status = InvoiceStatus::Paid;
            inv.paid_at = Some(self.current_epoch);
        }
        
        payment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subscription_tier_pricing() {
        assert_eq!(SubscriptionTier::Free.monthly_price_flux(), 0);
        assert_eq!(SubscriptionTier::Basic.monthly_price_flux(), 500);
        assert_eq!(SubscriptionTier::Professional.monthly_price_flux(), 2000);
        assert_eq!(SubscriptionTier::Enterprise.monthly_price_flux(), 10000);
    }
    
    #[test]
    fn test_billing_cycle_discount() {
        assert_eq!(BillingCycle::Monthly.discount_percent(), 0.0);
        assert_eq!(BillingCycle::Yearly.discount_percent(), 20.0);
    }
    
    #[test]
    fn test_create_subscription() {
        let mut manager = SubscriptionManager::new("test");
        let sub = manager.create_subscription("user1", SubscriptionTier::Basic, BillingCycle::Monthly).unwrap();
        assert_eq!(sub.tier, SubscriptionTier::Basic);
        assert_eq!(sub.status, SubscriptionStatus::Active);
    }
    
    #[test]
    fn test_trial_subscription() {
        let mut manager = SubscriptionManager::new("test");
        let sub = manager.create_subscription("user1", SubscriptionTier::Enterprise, BillingCycle::Monthly).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Trial);
        assert!(sub.trial_end.is_some());
    }
    
    #[test]
    fn test_mrr_calculation() {
        let mut manager = SubscriptionManager::new("test");
        manager.create_subscription("user1", SubscriptionTier::Basic, BillingCycle::Monthly).unwrap();
        manager.create_subscription("user2", SubscriptionTier::Professional, BillingCycle::Monthly).unwrap();
        assert_eq!(manager.get_mrr(), 2500); // 500 + 2000
    }
}
