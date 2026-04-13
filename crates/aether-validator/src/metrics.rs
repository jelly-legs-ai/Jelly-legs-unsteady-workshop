//! Prometheus Metrics Module
//!
//! Exposes validator metrics in Prometheus exposition format at `/metrics`.
//! Critical for production testnet — enables monitoring, alerting, and debugging.
//!
//! # Metrics Exposed
//!
//! - `aether_slot_current`           — Current slot (gauge)
//! - `aether_blocks_produced_total`  — Total blocks produced (counter)
//! - `aether_tx_processed_total`     — Total transactions processed (counter)
//! - `aether_tx_per_lane_total`      — Transactions per priority lane (counter by lane)
//! - `aether_tx_pool_pending`        — Pending transactions in mempool (gauge by lane)
//! - `aether_peer_count`             — Connected peer count (gauge)
//! - `aether_epoch_current`          — Current epoch (gauge)
//! - `aether_fee_total`              — Fees collected by category (counter)
//! - `aether_block_time_ms`          — Time to produce last block (gauge)
//! - `aether_state_root`             — Current state root hash info (gauge = 1, label = hash prefix)
//! - `aether_validator_tier`         — Validator tier (gauge = 1, label = tier)
//! - `aether_health_status`          — Health probe result (gauge, 1=healthy/0=unhealthy)
//! - `aether_syncing`               — Whether the node is currently syncing (gauge, 0 or 1)

use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Thread-safe metrics registry. Update from any task, scrape from the RPC handler.
pub struct Metrics {
    // Counters
    blocks_produced: AtomicU64,
    tx_processed: AtomicU64,
    tx_critical: AtomicU64,
    tx_high: AtomicU64,
    tx_standard: AtomicU64,
    fees_treasury: AtomicU64,
    fees_validator: AtomicU64,
    fees_burned: AtomicU64,

    // Gauges
    slot_current: AtomicU64,
    epoch_current: AtomicU64,
    peer_count: AtomicU64,
    pool_critical: AtomicU64,
    pool_high: AtomicU64,
    pool_standard: AtomicU64,
    block_time_ms: AtomicU64,
    health_status: AtomicU64, // 1 = healthy, 0 = unhealthy
    syncing: AtomicBool,
    validator_tier_full: AtomicBool,
    validator_tier_lite: AtomicBool,
    validator_tier_observer: AtomicBool,

    // Block timing
    last_block_time: Arc<tokio::sync::RwLock<Option<Instant>>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            blocks_produced: AtomicU64::new(0),
            tx_processed: AtomicU64::new(0),
            tx_critical: AtomicU64::new(0),
            tx_high: AtomicU64::new(0),
            tx_standard: AtomicU64::new(0),
            fees_treasury: AtomicU64::new(0),
            fees_validator: AtomicU64::new(0),
            fees_burned: AtomicU64::new(0),
            slot_current: AtomicU64::new(0),
            epoch_current: AtomicU64::new(0),
            peer_count: AtomicU64::new(0),
            pool_critical: AtomicU64::new(0),
            pool_high: AtomicU64::new(0),
            pool_standard: AtomicU64::new(0),
            block_time_ms: AtomicU64::new(0),
            health_status: AtomicU64::new(1),
            syncing: AtomicBool::new(false),
            validator_tier_full: AtomicBool::new(true),
            validator_tier_lite: AtomicBool::new(false),
            validator_tier_observer: AtomicBool::new(false),
            last_block_time: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    // --- Counter updates ---

    pub fn inc_blocks_produced(&self) {
        self.blocks_produced.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_tx_processed(&self) {
        self.tx_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_tx_by_lane(&self, lane: &str) {
        match lane {
            "critical" => { self.tx_critical.fetch_add(1, Ordering::Relaxed); }
            "high" => { self.tx_high.fetch_add(1, Ordering::Relaxed); }
            _ => { self.tx_standard.fetch_add(1, Ordering::Relaxed); }
        }
    }

    pub fn add_fees_treasury(&self, amount: u64) {
        self.fees_treasury.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn add_fees_validator(&self, amount: u64) {
        self.fees_validator.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn add_fees_burned(&self, amount: u64) {
        self.fees_burned.fetch_add(amount, Ordering::Relaxed);
    }

    // --- Gauge updates ---

    pub fn set_slot(&self, slot: u64) {
        self.slot_current.store(slot, Ordering::Relaxed);
    }

    pub fn set_epoch(&self, epoch: u64) {
        self.epoch_current.store(epoch, Ordering::Relaxed);
    }

    pub fn set_peer_count(&self, count: u64) {
        self.peer_count.store(count, Ordering::Relaxed);
    }

    pub fn set_pool_pending(&self, critical: usize, high: usize, standard: usize) {
        self.pool_critical.store(critical as u64, Ordering::Relaxed);
        self.pool_high.store(high as u64, Ordering::Relaxed);
        self.pool_standard.store(standard as u64, Ordering::Relaxed);
    }

    pub fn set_block_time_ms(&self, ms: u64) {
        self.block_time_ms.store(ms, Ordering::Relaxed);
    }

    pub fn set_health_status(&self, healthy: bool) {
        self.health_status.store(if healthy { 1 } else { 0 }, Ordering::Relaxed);
    }

    pub fn set_syncing(&self, syncing: bool) {
        self.syncing.store(syncing, Ordering::Relaxed);
    }

    pub fn set_validator_tier(&self, tier: &str) {
        self.validator_tier_full.store(tier == "full", Ordering::Relaxed);
        self.validator_tier_lite.store(tier == "lite", Ordering::Relaxed);
        self.validator_tier_observer.store(tier == "observer", Ordering::Relaxed);
    }

    /// Reset all tier flags (used internally before setting a new tier)
    pub fn reset_tier(&self) {
        self.validator_tier_full.store(false, Ordering::Relaxed);
        self.validator_tier_lite.store(false, Ordering::Relaxed);
        self.validator_tier_observer.store(false, Ordering::Relaxed);
    }

    pub async fn record_block_produced(&self) {
        let now = Instant::now();
        let mut last = self.last_block_time.write().await;
        if let Some(prev) = *last {
            let elapsed = now.duration_since(prev).as_millis() as u64;
            self.set_block_time_ms(elapsed);
        }
        *last = Some(now);
    }

    // --- Prometheus exposition format ---

    /// Render all metrics in Prometheus text exposition format.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(2048);

        // Helper to write a metric line
        let metric = |name: &str, r#type: &str, help: &str, value: u64| {
            format!(
                "# HELP {name} {help}\n# TYPE {name} {type}\n{name} {value}\n",
                name = name,
                type = r#type,
                help = help,
                value = value,
            )
        };

        let metric_label = |name: &str, r#type: &str, help: &str, labels: &str, value: u64| {
            format!(
                "# HELP {name} {help}\n# TYPE {name} {type}\n{name}{{{labels}}} {value}\n",
                name = name,
                type = r#type,
                help = help,
                labels = labels,
                value = value,
            )
        };

        // Gauges
        out.push_str(&metric("aether_slot_current", "gauge", "Current slot", self.slot_current.load(Ordering::Relaxed)));
        out.push_str(&metric("aether_epoch_current", "gauge", "Current epoch", self.epoch_current.load(Ordering::Relaxed)));
        out.push_str(&metric("aether_peer_count", "gauge", "Connected peer count", self.peer_count.load(Ordering::Relaxed)));
        out.push_str(&metric("aether_block_time_ms", "gauge", "Time to produce last block in ms", self.block_time_ms.load(Ordering::Relaxed)));
        out.push_str(&metric("aether_health_status", "gauge", "Health probe result (1=healthy 0=unhealthy)", self.health_status.load(Ordering::Relaxed)));
        out.push_str(&metric("aether_syncing", "gauge", "Whether the node is currently syncing (0 or 1)", self.syncing.load(Ordering::Relaxed) as u64));

        // Validator tier as label
        out.push_str(&metric_label("aether_validator_tier", "gauge", "Validator tier (1=active tier)", "tier=\"full\"", self.validator_tier_full.load(Ordering::Relaxed) as u64));
        out.push_str(&metric_label("aether_validator_tier", "gauge", "Validator tier (1=active tier)", "tier=\"lite\"", self.validator_tier_lite.load(Ordering::Relaxed) as u64));
        out.push_str(&metric_label("aether_validator_tier", "gauge", "Validator tier (1=active tier)", "tier=\"observer\"", self.validator_tier_observer.load(Ordering::Relaxed) as u64));

        // Counters
        out.push_str(&metric("aether_blocks_produced_total", "counter", "Total blocks produced", self.blocks_produced.load(Ordering::Relaxed)));
        out.push_str(&metric("aether_tx_processed_total", "counter", "Total transactions processed", self.tx_processed.load(Ordering::Relaxed)));

        // TX per lane
        out.push_str(&metric_label("aether_tx_per_lane_total", "counter", "Transactions processed per priority lane", "lane=\"critical\"", self.tx_critical.load(Ordering::Relaxed)));
        out.push_str(&metric_label("aether_tx_per_lane_total", "counter", "Transactions processed per priority lane", "lane=\"high\"", self.tx_high.load(Ordering::Relaxed)));
        out.push_str(&metric_label("aether_tx_per_lane_total", "counter", "Transactions processed per priority lane", "lane=\"standard\"", self.tx_standard.load(Ordering::Relaxed)));

        // Mempool depth
        out.push_str(&metric_label("aether_tx_pool_pending", "gauge", "Pending transactions in mempool", "lane=\"critical\"", self.pool_critical.load(Ordering::Relaxed)));
        out.push_str(&metric_label("aether_tx_pool_pending", "gauge", "Pending transactions in mempool", "lane=\"high\"", self.pool_high.load(Ordering::Relaxed)));
        out.push_str(&metric_label("aether_tx_pool_pending", "gauge", "Pending transactions in mempool", "lane=\"standard\"", self.pool_standard.load(Ordering::Relaxed)));

        // Fees
        out.push_str(&metric_label("aether_fee_total", "counter", "Fees collected by destination", "destination=\"treasury\"", self.fees_treasury.load(Ordering::Relaxed)));
        out.push_str(&metric_label("aether_fee_total", "counter", "Fees collected by destination", "destination=\"validator\"", self.fees_validator.load(Ordering::Relaxed)));
        out.push_str(&metric_label("aether_fee_total", "counter", "Fees collected by destination", "destination=\"burned\"", self.fees_burned.load(Ordering::Relaxed)));

        out
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Metrics {
    fn clone(&self) -> Self {
        Self {
            blocks_produced: AtomicU64::new(self.blocks_produced.load(Ordering::Relaxed)),
            tx_processed: AtomicU64::new(self.tx_processed.load(Ordering::Relaxed)),
            tx_critical: AtomicU64::new(self.tx_critical.load(Ordering::Relaxed)),
            tx_high: AtomicU64::new(self.tx_high.load(Ordering::Relaxed)),
            tx_standard: AtomicU64::new(self.tx_standard.load(Ordering::Relaxed)),
            fees_treasury: AtomicU64::new(self.fees_treasury.load(Ordering::Relaxed)),
            fees_validator: AtomicU64::new(self.fees_validator.load(Ordering::Relaxed)),
            fees_burned: AtomicU64::new(self.fees_burned.load(Ordering::Relaxed)),
            slot_current: AtomicU64::new(self.slot_current.load(Ordering::Relaxed)),
            epoch_current: AtomicU64::new(self.epoch_current.load(Ordering::Relaxed)),
            peer_count: AtomicU64::new(self.peer_count.load(Ordering::Relaxed)),
            pool_critical: AtomicU64::new(self.pool_critical.load(Ordering::Relaxed)),
            pool_high: AtomicU64::new(self.pool_high.load(Ordering::Relaxed)),
            pool_standard: AtomicU64::new(self.pool_standard.load(Ordering::Relaxed)),
            block_time_ms: AtomicU64::new(self.block_time_ms.load(Ordering::Relaxed)),
            health_status: AtomicU64::new(self.health_status.load(Ordering::Relaxed)),
            syncing: AtomicBool::new(self.syncing.load(Ordering::Relaxed)),
            validator_tier_full: AtomicBool::new(self.validator_tier_full.load(Ordering::Relaxed)),
            validator_tier_lite: AtomicBool::new(self.validator_tier_lite.load(Ordering::Relaxed)),
            validator_tier_observer: AtomicBool::new(self.validator_tier_observer.load(Ordering::Relaxed)),
            last_block_time: self.last_block_time.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_prometheus_format() {
        let m = Metrics::new();
        m.set_slot(42);
        m.set_epoch(0);
        m.set_peer_count(3);
        m.inc_blocks_produced();
        m.inc_tx_by_lane("critical");
        m.inc_tx_by_lane("high");
        m.inc_tx_by_lane("standard");
        m.add_fees_treasury(1000);

        let output = m.render();

        // Verify Prometheus format
        assert!(output.contains("# HELP aether_slot_current"));
        assert!(output.contains("# TYPE aether_slot_current gauge"));
        assert!(output.contains("aether_slot_current 42"));
        assert!(output.contains("aether_peer_count 3"));
        assert!(output.contains("aether_blocks_produced_total 1"));
        assert!(output.contains("lane=\"critical\""));
        assert!(output.contains("lane=\"high\""));
        assert!(output.contains("lane=\"standard\""));
        assert!(output.contains("destination=\"treasury\""));
    }

    #[test]
    fn test_validator_tier_metrics() {
        let m = Metrics::new();
        m.set_validator_tier("lite");

        let output = m.render();
        assert!(output.contains("tier=\"full\"} 0"));
        assert!(output.contains("tier=\"lite\"} 1"));
        assert!(output.contains("tier=\"observer\"} 0"));
    }

    #[test]
    fn test_syncing_metric() {
        let m = Metrics::new();
        assert_eq!(m.syncing.load(Ordering::Relaxed), false);
        m.set_syncing(true);
        assert_eq!(m.syncing.load(Ordering::Relaxed), true);

        let output = m.render();
        assert!(output.contains("aether_syncing 1"));
    }

    #[tokio::test]
    async fn test_block_time_tracking() {
        let m = Metrics::new();
        m.record_block_produced().await;
        // First block has no previous reference, so block_time_ms stays 0
        assert_eq!(m.block_time_ms.load(Ordering::Relaxed), 0);

        // Small delay then second block
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        m.record_block_produced().await;
        // Should have recorded ~50ms
        let bt = m.block_time_ms.load(Ordering::Relaxed);
        assert!(bt >= 30 && bt <= 200, "block time {}ms out of expected range", bt);
    }

    #[test]
    fn test_clone_preserves_values() {
        let m = Metrics::new();
        m.set_slot(100);
        m.inc_blocks_produced();

        let m2 = m.clone();
        assert_eq!(m2.slot_current.load(Ordering::Relaxed), 100);
        assert_eq!(m2.blocks_produced.load(Ordering::Relaxed), 1);
    }
}