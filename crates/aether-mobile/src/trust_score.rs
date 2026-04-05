//! AETHER Mobile Mining - Trust Score System
//!
//! Implements a reputation system for mobile mining participants.
//! Trust scores determine reward multipliers and eligibility for higher tiers.

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::proof_engine::{DeviceTier, WorkProof};

/// Trust score errors
#[derive(Error, Debug)]
pub enum TrustError {
    #[error("Insufficient trust score: {0} < required {1}")]
    InsufficientScore(u32, u32),
    #[error("No history available for device")]
    NoHistory,
    #[error("Device banned: {0}")]
    Banned(String),
}

/// Trust score state for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustState {
    /// Device public key
    pub device_key: [u8; 32],
    /// Current trust score (0-1000)
    pub score: u32,
    /// Lifetime proofs submitted
    pub lifetime_proofs: u64,
    /// Successful proofs (accepted by network)
    pub successful_proofs: u64,
    /// Failed proofs (rejected)
    pub failed_proofs: u64,
    /// Consecutive successful proofs
    pub consecutive_success: u32,
    /// Consecutive failed proofs
    pub consecutive_failure: u32,
    /// Last proof timestamp
    pub last_proof_at: Option<i64>,
    /// Account creation time
    pub created_at: i64,
    /// Last score update time
    pub updated_at: i64,
    /// Whether device is banned
    pub banned: bool,
    /// Ban reason if banned
    pub ban_reason: Option<String>,
    /// Tier upgrade eligibility
    pub tier_upgrade_eligible: bool,
    /// Number of tier upgrades achieved
    pub tier_upgrades: u8,
}

impl TrustState {
    /// Create a new trust state for a device
    pub fn new(device_key: [u8; 32]) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            device_key,
            score: 500, // Start at neutral
            lifetime_proofs: 0,
            successful_proofs: 0,
            failed_proofs: 0,
            consecutive_success: 0,
            consecutive_failure: 0,
            last_proof_at: None,
            created_at: now,
            updated_at: now,
            banned: false,
            ban_reason: None,
            tier_upgrade_eligible: false,
            tier_upgrades: 0,
        }
    }

    /// Calculate current reward multiplier based on trust score
    pub fn reward_multiplier(&self) -> f64 {
        if self.banned {
            return 0.0;
        }

        let base = self.score as f64 / 1000.0;

        // Apply consecutive success bonus (up to +20%)
        let streak_bonus = (self.consecutive_success.min(20) as f64) * 0.01;

        // Apply consecutive failure penalty (up to -30%)
        let streak_penalty = (self.consecutive_failure.min(30) as f64) * 0.01;

        (base + streak_bonus - streak_penalty).clamp(0.1, 1.5)
    }

    /// Get the effective device tier based on trust
    pub fn effective_tier(&self, hardware_tier: DeviceTier) -> DeviceTier {
        // Downgrade if trust is low
        if self.score < 250 {
            DeviceTier::Starter
        } else if self.score < 500 && hardware_tier != DeviceTier::Starter {
            DeviceTier::Tier3
        } else {
            hardware_tier
        }
    }

    /// Check if device can participate in mining
    pub fn can_mine(&self) -> bool {
        !self.banned && self.score >= 100
    }
}

/// A single proof event for history tracking
#[derive(Debug, Clone)]
struct ProofEvent {
    timestamp: i64,
    success: bool,
    compute_time_ms: u64,
    expected_time_ms: u64,
    difficulty_achieved: u32,
    difficulty_required: u32,
}

/// Trust score manager
pub struct TrustScoreManager {
    /// Per-device trust states
    states: RwLock<std::collections::HashMap<[u8; 32], Arc<TrustState>>>,
    /// Per-device proof history (ring buffer)
    history: RwLock<std::collections::HashMap<[u8; 32], VecDeque<ProofEvent>>>,
    /// Configuration
    config: TrustConfig,
}

/// Trust score configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustConfig {
    /// Starting score for new devices
    pub initial_score: u32,
    /// Score for successful proof
    pub success_reward: i32,
    /// Score penalty for failed proof
    pub failure_penalty: i32,
    /// Score bonus for consecutive successes (per proof)
    pub streak_bonus: i32,
    /// Score penalty for consecutive failures (per proof)
    pub streak_penalty: i32,
    /// Maximum consecutive failures before soft ban
    pub max_consecutive_failures: u32,
    /// Minimum score to avoid hard ban
    pub min_score: u32,
    /// Score needed for tier upgrade eligibility
    pub tier_upgrade_threshold: u32,
    /// Number of successful proofs needed for tier upgrade
    pub proofs_for_tier_upgrade: u32,
    /// History window for score calculation (ms)
    pub history_window_ms: i64,
    /// Penalty for submitting too fast (bot detection)
    pub fast_submit_penalty: i32,
    /// Minimum compute time ratio (actual/expected)
    pub min_compute_ratio: f64,
    /// Maximum compute time ratio
    pub max_compute_ratio: f64,
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            initial_score: 500,
            success_reward: 5,
            failure_penalty: 25,
            streak_bonus: 1,
            streak_penalty: 3,
            max_consecutive_failures: 5,
            min_score: 50,
            tier_upgrade_threshold: 750,
            proofs_for_tier_upgrade: 1000,
            history_window_ms: 86400 * 1000, // 24 hours
            fast_submit_penalty: 50,
            min_compute_ratio: 0.1,  // Must take at least 10% of expected time
            max_compute_ratio: 10.0, // Can't be more than 10x expected (too slow)
        }
    }
}

impl TrustScoreManager {
    /// Create a new trust score manager
    pub fn new() -> Self {
        Self {
            states: RwLock::new(std::collections::HashMap::new()),
            history: RwLock::new(std::collections::HashMap::new()),
            config: TrustConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: TrustConfig) -> Self {
        Self {
            states: RwLock::new(std::collections::HashMap::new()),
            history: RwLock::new(std::collections::HashMap::new()),
            config,
        }
    }

    /// Get or create trust state for a device
    pub fn get_or_create_state(&self, device_key: [u8; 32]) -> Arc<TrustState> {
        // Try to get existing state
        {
            let states = self.states.read().unwrap();
            if let Some(state) = states.get(&device_key) {
                return Arc::clone(state);
            }
        }

        // Create new state
        let state = Arc::new(TrustState::new(device_key));

        let mut states = self.states.write().unwrap();
        states.insert(device_key, Arc::clone(&state));

        // Initialize history
        let mut history = self.history.write().unwrap();
        history.insert(device_key, VecDeque::with_capacity(1000));

        Arc::clone(&state)
    }

    /// Record a successful proof
    pub fn record_success(
        &self,
        device_key: [u8; 32],
        proof: &WorkProof,
        expected_compute_time_ms: u64,
        difficulty_required: u32,
    ) -> Arc<TrustState> {
        let state = self.get_or_create_state(device_key);

        // Update state
        {
            let mut states = self.states.write().unwrap();
            if let Some(s) = states.get_mut(&device_key) {
                // Clone current state data
                let mut s = (**s).clone();

                s.lifetime_proofs += 1;
                s.successful_proofs += 1;
                s.consecutive_success += 1;
                s.consecutive_failure = 0;
                s.last_proof_at = Some(proof.submitted_at);
                s.updated_at = Utc::now().timestamp_millis();

                // Check for tier upgrade eligibility
                if s.successful_proofs >= self.config.proofs_for_tier_upgrade as u64
                    && s.score >= self.config.tier_upgrade_threshold
                    && !s.tier_upgrade_eligible
                {
                    s.tier_upgrade_eligible = true;
                }

                // Apply score reward
                let mut score_delta = self.config.success_reward as i32;
                score_delta += self.config.streak_bonus as i32 * s.consecutive_success as i32;
                s.score = (s.score as i32 + score_delta).clamp(0, 1000) as u32;

                // Replace in hashmap
                let new_state = Arc::new(s);
                *states.get_mut(&device_key).unwrap() = new_state;
            }
        }

        // Record event in history
        {
            let mut history = self.history.write().unwrap();
            if let Some(h) = history.get_mut(&device_key) {
                h.push_back(ProofEvent {
                    timestamp: proof.submitted_at,
                    success: true,
                    compute_time_ms: proof.compute_time_ms,
                    expected_time_ms: expected_compute_time_ms,
                    difficulty_achieved: proof.difficulty_achieved,
                    difficulty_required,
                });

                // Trim old events
                let cutoff = Utc::now() - Duration::milliseconds(self.config.history_window_ms);
                let cutoff_ts = cutoff.timestamp_millis();
                while h.front().map(|e| e.timestamp < cutoff_ts).unwrap_or(false) {
                    h.pop_front();
                }
            }
        }

        self.get_or_create_state(device_key)
    }

    /// Record a failed proof
    pub fn record_failure(
        &self,
        device_key: [u8; 32],
        submitted_at: i64,
        compute_time_ms: u64,
        expected_compute_time_ms: u64,
        reason: &str,
    ) -> Arc<TrustState> {
        let state = self.get_or_create_state(device_key);

        // Update state
        {
            let mut states = self.states.write().unwrap();
            if let Some(s) = states.get_mut(&device_key) {
                let mut s = (**s).clone();

                s.lifetime_proofs += 1;
                s.failed_proofs += 1;
                s.consecutive_failure += 1;
                s.consecutive_success = 0;
                s.last_proof_at = Some(submitted_at);
                s.updated_at = Utc::now().timestamp_millis();

                // Apply score penalty
                let mut score_delta = -(self.config.failure_penalty as i32);
                score_delta -= self.config.streak_penalty as i32 * s.consecutive_failure as i32;
                s.score = (s.score as i32 + score_delta).clamp(0, 1000) as u32;

                // Check for ban
                if s.consecutive_failure >= self.config.max_consecutive_failures
                    || s.score < self.config.min_score
                {
                    s.banned = true;
                    s.ban_reason = Some(reason.to_string());
                }

                let new_state = Arc::new(s);
                *states.get_mut(&device_key).unwrap() = new_state;
            }
        }

        // Record event
        {
            let mut history = self.history.write().unwrap();
            if let Some(h) = history.get_mut(&device_key) {
                h.push_back(ProofEvent {
                    timestamp: submitted_at,
                    success: false,
                    compute_time_ms,
                    expected_time_ms: expected_compute_time_ms,
                    difficulty_achieved: 0,
                    difficulty_required: 0,
                });
            }
        }

        self.get_or_create_state(device_key)
    }

    /// Check for suspicious patterns (bot detection)
    pub fn check_suspicious(&self, device_key: [u8; 32]) -> Option<SuspiciousFlags> {
        let history = self.history.read().unwrap();
        let h = history.get(&device_key)?;

        if h.is_empty() {
            return None;
        }

        let mut flags = SuspiciousFlags::default();

        // Check for too-fast submissions
        let recent: Vec<_> = h.iter().rev().take(10).collect();
        if recent.len() >= 2 {
            let mut total_time = 0i64;
            for window in recent.windows(2) {
                total_time += window[1].timestamp - window[0].timestamp;
            }
            let avg_gap = total_time as f64 / (recent.len() - 1) as f64;
            if avg_gap < 1000.0 {
                // Less than 1 second between proofs
                flags.too_fast_submission = true;
            }
        }

        // Check for suspiciously consistent compute times
        if h.len() >= 5 {
            let compute_times: Vec<u64> = h.iter().rev().take(20).map(|e| e.compute_time_ms).collect();
            let variance = compute_variance(&compute_times);
            if variance < 0.01 {
                // Nearly identical every time - likely bot
                flags.suspiciously_consistent = true;
            }
        }

        // Check for impossible compute ratios
        for event in h.iter().rev().take(10) {
            if event.success && event.expected_time_ms > 0 {
                let ratio = event.compute_time_ms as f64 / event.expected_time_ms as f64;
                if ratio < self.config.min_compute_ratio {
                    flags.too_fast_compute = true;
                }
                if ratio > self.config.max_compute_ratio {
                    flags.too_slow_compute = true;
                }
            }
        }

        // Check for low difficulty proofs
        for event in h.iter().rev().take(10) {
            if event.success && event.difficulty_achieved < event.difficulty_required {
                flags.low_difficulty = true;
            }
        }

        if flags.any() {
            Some(flags)
        } else {
            None
        }
    }

    /// Get historical performance stats
    pub fn get_stats(&self, device_key: [u8; 32]) -> Option<TrustStats> {
        let state = self.states.read().unwrap().get(&device_key)?;
        let history = self.history.read().unwrap().get(&device_key)?;

        let cutoff = Utc::now() - Duration::milliseconds(self.config.history_window_ms);
        let cutoff_ts = cutoff.timestamp_millis();

        let recent: Vec<_> = history.iter().filter(|e| e.timestamp >= cutoff_ts).collect();

        let success_rate = if !recent.is_empty() {
            recent.iter().filter(|e| e.success).count() as f64 / recent.len() as f64
        } else {
            0.0
        };

        let avg_compute_time = if !recent.is_empty() {
            recent.iter().map(|e| e.compute_time_ms as f64).sum::<f64>() / recent.len() as f64
        } else {
            0.0
        };

        Some(TrustStats {
            score: state.score,
            lifetime_proofs: state.lifetime_proofs,
            successful_proofs: state.successful_proofs,
            failed_proofs: state.failed_proofs,
            success_rate_24h: success_rate,
            avg_compute_time_ms: avg_compute_time as u64,
            consecutive_success: state.consecutive_success,
            tier_upgrade_eligible: state.tier_upgrade_eligible,
            banned: state.banned,
        })
    }
}

impl Default for TrustScoreManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Suspicious activity flags
#[derive(Debug, Clone, Default)]
pub struct SuspiciousFlags {
    pub too_fast_submission: bool,
    pub suspiciously_consistent: bool,
    pub too_fast_compute: bool,
    pub too_slow_compute: bool,
    pub low_difficulty: bool,
}

impl SuspiciousFlags {
    pub fn any(&self) -> bool {
        self.too_fast_submission
            || self.suspiciously_consistent
            || self.too_fast_compute
            || self.too_slow_compute
            || self.low_difficulty
    }
}

/// Trust statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustStats {
    pub score: u32,
    pub lifetime_proofs: u64,
    pub successful_proofs: u64,
    pub failed_proofs: u64,
    pub success_rate_24h: f64,
    pub avg_compute_time_ms: u64,
    pub consecutive_success: u32,
    pub tier_upgrade_eligible: bool,
    pub banned: bool,
}

/// Calculate variance of compute times
fn compute_variance(values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<u64>() as f64 / values.len() as f64;
    let variance = values
        .iter()
        .map(|v| {
            let diff = *v as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / values.len() as f64;

    // Normalized variance
    if mean > 0.0 {
        variance / (mean * mean)
    } else {
        0.0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_state_new() {
        let key = [0u8; 32];
        let state = TrustState::new(key);
        assert_eq!(state.score, 500);
        assert!(!state.banned);
    }

    #[test]
    fn test_reward_multiplier() {
        let key = [0u8; 32];
        let state = TrustState::new(key);
        let multiplier = state.reward_multiplier();
        assert!(multiplier >= 0.1 && multiplier <= 1.5);
    }

    #[test]
    fn test_compute_variance() {
        let values = vec![100, 100, 100, 100];
        let var = compute_variance(&values);
        assert!(var < 0.01);

        let values2 = vec![50, 100, 150, 200];
        let var2 = compute_variance(&values2);
        assert!(var2 > 0.1);
    }
}
