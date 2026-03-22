//! Reward calculation module for AETHER mobile-mining blockchain
//!
//! Handles daily reward calculations based on trust score, device tier,
//! and active mining hours.

use serde::{Deserialize, Serialize};

/// Device tier classification with associated power/capability levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceTier {
    /// Mobile devices (phones, tablets) - lowest tier
    Mobile,
    /// Laptop computers - standard tier
    Laptop,
    /// Desktop computers - higher tier
    Desktop,
    /// Server-grade hardware - highest tier
    Server,
}

impl DeviceTier {
    /// Returns the multiplier associated with this device tier
    pub fn multiplier(&self) -> f64 {
        match self {
            DeviceTier::Mobile => 0.1,
            DeviceTier::Laptop => 1.0,
            DeviceTier::Desktop => 2.5,
            DeviceTier::Server => 10.0,
        }
    }
}

/// Reward calculator for AETHER blockchain mining rewards
///
/// Tracks trust scores and calculates daily rewards based on
/// device tier, activity, and accumulated trust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardCalculator {
    /// Current trust score (0.0 to 1.0)
    trust_score: f64,
    /// Last time the trust score was updated (Unix timestamp)
    last_update: u64,
    /// Decay rate per day when inactive (default: 0.01 = 1%)
    decay_rate: f64,
}

impl Default for RewardCalculator {
    fn default() -> Self {
        Self {
            trust_score: 0.5, // Start with neutral trust
            last_update: 0,
            decay_rate: 0.01,
        }
    }
}

impl RewardCalculator {
    /// Creates a new reward calculator with initial trust score
    pub fn new(initial_trust: f64, last_update: u64) -> Self {
        Self {
            trust_score: initial_trust.clamp(0.0, 1.0),
            last_update,
            decay_rate: 0.01,
        }
    }

    /// Creates a new reward calculator with default values
    pub fn new_default() -> Self {
        Self::default()
    }

    /// Gets the current trust score
    pub fn trust_score(&self) -> f64 {
        self.trust_score
    }

    /// Sets the trust score directly
    pub fn set_trust_score(&mut self, score: f64) {
        self.trust_score = score.clamp(0.0, 1.0);
    }

    /// Gets the last update timestamp
    pub fn last_update(&self) -> u64 {
        self.last_update
    }

    /// Calculates the daily reward based on trust, tier, and activity
    ///
    /// Formula: base_reward × trust_score × tier_multiplier × active_hours
    ///
    /// # Arguments
    /// * `trust_score` - User's trust score (0.0 to 1.0)
    /// * `tier` - Device tier classification
    /// * `active_hours` - Hours of active mining (0.0 to 24.0)
    /// * `base_reward` - Base reward amount in lamports
    ///
    /// # Returns
    /// Calculated daily reward (clamped to base_reward × 24 max)
    pub fn calculate_daily_reward(
        trust_score: f64,
        tier: DeviceTier,
        active_hours: f64,
        base_reward: f64,
    ) -> f64 {
        let trust = trust_score.clamp(0.0, 1.0);
        let hours = active_hours.clamp(0.0, 24.0);
        let multiplier = tier.multiplier();

        let reward = base_reward * trust * multiplier * hours;

        // Cap at theoretical maximum (base_reward × 24)
        reward.min(base_reward * 24.0)
    }

    /// Updates trust score with decay for inactivity
    ///
    /// When a device goes inactive, trust decays by `decay_rate` per day.
    /// Trust cannot go below 0.0.
    ///
    /// # Arguments
    /// * `current_time` - Current Unix timestamp
    /// * `inactive_days` - Number of days inactive (use accumulated days from last_update)
    pub fn update_trust_decay(&mut self, current_time: u64, inactive_days: u64) {
        if inactive_days == 0 {
            self.last_update = current_time;
            return;
        }

        let decay_factor = (1.0 - self.decay_rate).powi(inactive_days as i32);
        self.trust_score = (self.trust_score * decay_factor).max(0.0);
        self.last_update = current_time;
    }

    /// Updates trust score based on positive activity (increases trust)
    ///
    /// Trust increases by a small amount for each day of active mining.
    /// Trust is capped at 1.0.
    ///
    /// # Arguments
    /// * `current_time` - Current Unix timestamp
    /// * `active_days` - Number of days actively mining
    pub fn update_trust_boost(&mut self, current_time: u64, active_days: u64) {
        if active_days == 0 {
            return;
        }

        // Increase trust by ~2% per active day, diminishing as it approaches 1.0
        let boost = 0.02 * (1.0 - self.trust_score) * active_days as f64;
        self.trust_score = (self.trust_score + boost).min(1.0);
        self.last_update = current_time;
    }

    /// Processes trust update based on time elapsed since last update
    ///
    /// Automatically calculates days elapsed and applies appropriate
    /// trust changes (decay for inactive, boost for active).
    ///
    /// # Arguments
    /// * `current_time` - Current Unix timestamp
    /// * `was_active` - Whether the device was active during the elapsed time
    pub fn process_trust_update(&mut self, current_time: u64, was_active: bool) {
        const SECONDS_PER_DAY: u64 = 86400;

        if self.last_update == 0 {
            self.last_update = current_time;
            return;
        }

        let elapsed = current_time.saturating_sub(self.last_update);
        let days_elapsed = elapsed / SECONDS_PER_DAY;

        if days_elapsed == 0 {
            return;
        }

        if was_active {
            self.update_trust_boost(current_time, days_elapsed);
        } else {
            self.update_trust_decay(current_time, days_elapsed);
        }
    }

    /// Resets trust score to default value
    pub fn reset_trust(&mut self) {
        self.trust_score = 0.5;
    }
}

/// Constants for device tier multipliers
pub mod constants {
    /// Mobile device reward multiplier
    pub const MOBILE_MULTIPLIER: f64 = 0.1;
    /// Laptop device reward multiplier
    pub const LAPTOP_MULTIPLIER: f64 = 1.0;
    /// Desktop device reward multiplier
    pub const DESKTOP_MULTIPLIER: f64 = 2.5;
    /// Server device reward multiplier
    pub const SERVER_MULTIPLIER: f64 = 10.0;

    /// Default base reward in lamports (1 AETHER = 1,000,000,000 lamports)
    pub const DEFAULT_BASE_REWARD: f64 = 100_000_000.0; // 0.1 AETHER

    /// Trust score decay rate per day (1% per day)
    pub const DEFAULT_DECAY_RATE: f64 = 0.01;

    /// Maximum trust score
    pub const MAX_TRUST_SCORE: f64 = 1.0;
    /// Minimum trust score
    pub const MIN_TRUST_SCORE: f64 = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_tier_multipliers() {
        assert_eq!(DeviceTier::Mobile.multiplier(), 0.1);
        assert_eq!(DeviceTier::Laptop.multiplier(), 1.0);
        assert_eq!(DeviceTier::Desktop.multiplier(), 2.5);
        assert_eq!(DeviceTier::Server.multiplier(), 10.0);
    }

    #[test]
    fn test_calculate_daily_reward_basic() {
        let base = 100.0;

        // Mobile with full trust and 1 hour
        let mobile = RewardCalculator::calculate_daily_reward(1.0, DeviceTier::Mobile, 1.0, base);
        assert_eq!(mobile, 10.0); // 100 * 1.0 * 0.1 * 1

        // Laptop with full trust and 1 hour
        let laptop = RewardCalculator::calculate_daily_reward(1.0, DeviceTier::Laptop, 1.0, base);
        assert_eq!(laptop, 100.0); // 100 * 1.0 * 1.0 * 1

        // Desktop with full trust and 1 hour
        let desktop = RewardCalculator::calculate_daily_reward(1.0, DeviceTier::Desktop, 1.0, base);
        assert_eq!(laptop, 100.0);
        assert_eq!(desktop, 250.0); // 100 * 1.0 * 2.5 * 1

        // Server with full trust and 1 hour
        let server = RewardCalculator::calculate_daily_reward(1.0, DeviceTier::Server, 1.0, base);
        assert_eq!(server, 1000.0); // 100 * 1.0 * 10.0 * 1
    }

    #[test]
    fn test_calculate_daily_reward_with_trust() {
        let base = 100.0;
        let tier = DeviceTier::Laptop;
        let hours = 10.0;

        // 50% trust
        let half = RewardCalculator::calculate_daily_reward(0.5, tier, hours, base);
        assert_eq!(half, 500.0); // 100 * 0.5 * 1.0 * 10

        // 25% trust
        let quarter = RewardCalculator::calculate_daily_reward(0.25, tier, hours, base);
        assert_eq!(quarter, 250.0); // 100 * 0.25 * 1.0 * 10
    }

    #[test]
    fn test_calculate_daily_reward_hours_variance() {
        let base = 100.0;
        let trust = 1.0;
        let tier = DeviceTier::Laptop;

        // 0 hours = 0 reward
        assert_eq!(RewardCalculator::calculate_daily_reward(trust, tier, 0.0, base), 0.0);

        // 12 hours
        assert_eq!(RewardCalculator::calculate_daily_reward(trust, tier, 12.0, base), 1200.0);

        // 24 hours (max)
        assert_eq!(RewardCalculator::calculate_daily_reward(trust, tier, 24.0, base), 2400.0);
    }

    #[test]
    fn test_calculate_daily_reward_max_cap() {
        let base = 100.0;
        let trust = 1.0;
        let tier = DeviceTier::Server; // 10x multiplier

        // Even with 24 hours, capped at base * 24
        let capped = RewardCalculator::calculate_daily_reward(trust, tier, 24.0, base);
        assert_eq!(capped, 2400.0); // 100 * 24 = 2400 (capped, not 100*1*10*24=24000)
    }

    #[test]
    fn test_trust_score_bounds() {
        let mut calc = RewardCalculator::new_default();
        assert_eq!(calc.trust_score(), 0.5);

        // Trust cannot exceed 1.0
        calc.set_trust_score(1.5);
        assert_eq!(calc.trust_score(), 1.0);

        // Trust cannot go below 0.0
        calc.set_trust_score(-0.5);
        assert_eq!(calc.trust_score(), 0.0);
    }

    #[test]
    fn test_trust_decay() {
        let mut calc = RewardCalculator::new(0.5, 0);
        let current_time = 86400; // 1 day in seconds

        // Decay for 1 day at 1% per day
        calc.update_trust_decay(current_time, 1);
        assert!((calc.trust_score() - 0.495).abs() < 0.001); // 0.5 * 0.99

        // Reset and decay for 10 days
        calc.set_trust_score(0.5);
        calc.update_trust_decay(current_time, 10);
        assert!((calc.trust_score() - 0.5 * 0.99_f64.powi(10)).abs() < 0.001);
    }

    #[test]
    fn test_trust_boost() {
        let mut calc = RewardCalculator::new(0.5, 0);

        // Boost for 1 day
        calc.update_trust_boost(86400, 1);
        assert!(calc.trust_score() > 0.5);

        // 30 days of active mining should approach max trust
        calc.set_trust_score(0.8);
        calc.update_trust_boost(86400 * 30, 30);
        assert!(calc.trust_score() > 0.8);
        assert!(calc.trust_score() <= 1.0);
    }

    #[test]
    fn test_process_trust_update_active() {
        let mut calc = RewardCalculator::new(0.5, 100000);
        let new_time = 100000 + 86400 * 5; // 5 days later, was active

        calc.process_trust_update(new_time, true);
        assert!(calc.trust_score() > 0.5);
        assert_eq!(calc.last_update(), new_time);
    }

    #[test]
    fn test_process_trust_update_inactive() {
        let mut calc = RewardCalculator::new(0.5, 100000);
        let new_time = 100000 + 86400 * 5; // 5 days later, was inactive

        let old_score = calc.trust_score();
        calc.process_trust_update(new_time, false);
        assert!(calc.trust_score() < old_score);
        assert_eq!(calc.last_update(), new_time);
    }

    #[test]
    fn test_reward_calculator_default() {
        let calc = RewardCalculator::default();
        assert_eq!(calc.trust_score, 0.5);
        assert_eq!(calc.last_update, 0);
        assert_eq!(calc.decay_rate, 0.01);
    }

    #[test]
    fn test_reward_calculator_new() {
        let calc = RewardCalculator::new(0.75, 12345);
        assert_eq!(calc.trust_score(), 0.75);
        assert_eq!(calc.last_update(), 12345);
    }

    #[test]
    fn test_constants_match_tier() {
        assert_eq!(DeviceTier::Mobile.multiplier(), constants::MOBILE_MULTIPLIER);
        assert_eq!(DeviceTier::Laptop.multiplier(), constants::LAPTOP_MULTIPLIER);
        assert_eq!(DeviceTier::Desktop.multiplier(), constants::DESKTOP_MULTIPLIER);
        assert_eq!(DeviceTier::Server.multiplier(), constants::SERVER_MULTIPLIER);
    }
}
