//! Mining Rewards Module for AeTHer Chain
//! 
//! Implements Proof-of-Contribution (PoC) consensus for mobile mining:
//! - Device tier-based reward multipliers
//! - Uptime tracking and verification
//! - Network contribution scoring
//! - FLUX reward distribution

use crate::utils::{self, Timestamp};
use std::collections::HashMap;

/// Device tier for mining
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceTier {
    Mobile,      // Smartphones
    Laptop,      // laptops
    Desktop,     // Desktop computers
    Server,      // Dedicated servers
}

impl DeviceTier {
    /// Reward multiplier for this tier
    pub fn reward_multiplier(&self) -> f64 {
        match self {
            DeviceTier::Mobile => 1.0,
            DeviceTier::Laptop => 1.5,
            DeviceTier::Desktop => 2.0,
            DeviceTier::Server => 3.0,
        }
    }
    
    /// Minimum uptime requirement (percentage)
    pub fn min_uptime(&self) -> f64 {
        match self {
            DeviceTier::Mobile => 0.50,   // 50%
            DeviceTier::Laptop => 0.70,   // 70%
            DeviceTier::Desktop => 0.80,  // 80%
            DeviceTier::Server => 0.95,   // 95%
        }
    }
}

/// Mining device state
#[derive(Debug, Clone)]
pub struct MiningDevice {
    /// Owner address
    pub owner: [u8; 32],
    /// Device tier
    pub tier: DeviceTier,
    /// Registration timestamp
    pub registered_at: Timestamp,
    /// Last heartbeat
    pub last_heartbeat: Timestamp,
    /// Total uptime epochs
    pub total_uptime_epochs: u64,
    /// Total mining epochs
    pub total_mining_epochs: u64,
    /// Accumulated FLUX rewards (unclaimed)
    pub pending_rewards: u64,
    /// Total FLUX claimed
    pub total_claimed: u64,
    /// Is currently mining
    pub is_active: bool,
    /// Device reputation score (0-100)
    pub reputation: u8,
}

impl MiningDevice {
    /// Calculate effective uptime percentage
    pub fn uptime_percentage(&self) -> f64 {
        if self.total_mining_epochs == 0 {
            return 0.0;
        }
        self.total_uptime_epochs as f64 / self.total_mining_epochs as f64
    }
    
    /// Check if device meets minimum uptime requirement
    pub fn meets_uptime_requirement(&self) -> bool {
        self.uptime_percentage() >= self.tier.min_uptime()
    }
}

/// Mining configuration
#[derive(Debug, Clone)]
pub struct MiningConfig {
    /// Base reward per epoch (in smallest FLUX units)
    pub base_reward_per_epoch: u64,
    /// Maximum rewards per device per day
    pub max_daily_rewards: u64,
    /// Reputation penalty for missed heartbeats
    pub heartbeat_miss_penalty: u8,
    /// Reputation bonus for consistent uptime
    pub uptime_bonus: u8,
    /// Minimum reputation to earn rewards
    pub min_reputation_for_rewards: u8,
    /// Epoch duration in seconds
    pub epoch_duration_secs: u64,
    /// Heartbeat interval in epochs
    pub heartbeat_interval_epochs: u64,
    /// Network contribution weight for ML inference
    pub ml_inference_weight: f64,
    /// Network contribution weight for storage
    pub storage_weight: f64,
    /// Network contribution weight for light transactions
    pub transaction_weight: f64,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            base_reward_per_epoch: 100_000_000, // 0.1 FLUX (9 decimals)
            max_daily_rewards: 1_200_000_000,   // 1.2 FLUX per day
            heartbeat_miss_penalty: 5,
            uptime_bonus: 2,
            min_reputation_for_rewards: 20,
            epoch_duration_secs: 3600,           // 1 hour
            heartbeat_interval_epochs: 1,         // Every epoch
            ml_inference_weight: 0.4,
            storage_weight: 0.3,
            transaction_weight: 0.3,
        }
    }
}

/// Mining epoch statistics
#[derive(Debug, Clone)]
pub struct EpochStats {
    pub epoch: u64,
    pub total_devices: u64,
    pub active_devices: u64,
    pub total_rewards_distributed: u64,
    pub avg_uptime: f64,
}

/// Mining contract state
#[derive(Debug, Clone)]
pub struct MiningContract {
    pub config: MiningConfig,
    /// Current epoch
    pub current_epoch: u64,
    /// Devices: device_id -> MiningDevice
    devices: HashMap<[u8; 32], MiningDevice>,
    /// Device owners: owner -> Vec<device_id>
    owner_devices: HashMap<[u8; 32], Vec<[u8; 32]>>,
    /// Epoch history
    epoch_history: Vec<EpochStats>,
    /// Total FLUX distributed ever
    pub total_distributed: u64,
}

impl MiningContract {
    /// Create new mining contract
    pub fn new(config: MiningConfig) -> Self {
        Self {
            config,
            current_epoch: 0,
            devices: HashMap::new(),
            owner_devices: HashMap::new(),
            epoch_history: Vec::new(),
            total_distributed: 0,
        }
    }
    
    /// Register a new mining device
    pub fn register_device(
        &mut self,
        device_id: [u8; 32],
        owner: [u8; 32],
        tier: DeviceTier,
    ) -> Result<(), &'static str> {
        if self.devices.contains_key(&device_id) {
            return Err("Device already registered");
        }
        
        let now = utils::now();
        let device = MiningDevice {
            owner,
            tier,
            registered_at: now,
            last_heartbeat: now,
            total_uptime_epochs: 0,
            total_mining_epochs: 0,
            pending_rewards: 0,
            total_claimed: 0,
            is_active: true,
            reputation: 50, // Start with neutral reputation
        };
        
        self.devices.insert(device_id, device);
        self.owner_devices
            .entry(owner)
            .or_insert_with(Vec::new)
            .push(device_id);
        
        Ok(())
    }
    
    /// Process device heartbeat
    pub fn heartbeat(
        &mut self,
        device_id: &[u8; 32],
        contribution_score: f64, // 0.0 - 1.0
    ) -> Result<u64, &'static str> {
        let device = self.devices.get_mut(device_id)
            .ok_or("Device not found")?;
        
        let now = utils::now();
        let epochs_since_last = ((now - device.last_heartbeat) / self.config.epoch_duration_secs)
            .max(1);
        
        // Update mining statistics
        device.total_mining_epochs += epochs_since_last;
        device.total_uptime_epochs += epochs_since_last; // Assuming heartbeat = uptime
        device.last_heartbeat = now;
        
        // Calculate and accumulate rewards
        let rewards = self.calculate_epoch_rewards(device, contribution_score);
        device.pending_rewards += rewards;
        
        Ok(rewards)
    }
    
    /// Calculate rewards for an epoch
    fn calculate_epoch_rewards(&self, device: &MiningDevice, contribution_score: f64) -> u64 {
        // Check minimum reputation
        if device.reputation < self.config.min_reputation_for_rewards {
            return 0;
        }
        
        // Base reward
        let mut reward = self.config.base_reward_per_epoch;
        
        // Apply device tier multiplier
        reward = (reward as f64 * device.tier.reward_multiplier()) as u64;
        
        // Apply reputation multiplier (50-100 -> 0.5x-1.0x)
        let reputation_multiplier = 0.5 + (device.reputation as f64 / 200.0);
        reward = (reward as f64 * reputation_multiplier) as u64;
        
        // Apply contribution score
        reward = (reward as f64 * contribution_score.clamp(0.0, 1.0)) as u64;
        
        // Cap at daily max (distributed per epoch, so divide by epochs per day)
        let epochs_per_day = 86400 / self.config.epoch_duration_secs;
        let daily_cap = self.config.max_daily_rewards / epochs_per_day;
        reward.min(daily_cap)
    }
    
    /// Update device reputation
    pub fn update_reputation(
        &mut self,
        device_id: &[u8; 32],
        missed_heartbeats: u64,
    ) -> Result<(), &'static str> {
        let device = self.devices.get_mut(device_id)
            .ok_or("Device not found")?;
        
        // Decrease for missed heartbeats
        let penalty = (missed_heartbeats * self.config.heartbeat_miss_penalty as u64) as u8;
        device.reputation = device.reputation.saturating_sub(penalty);
        
        // Bonus for consistent uptime
        if device.meets_uptime_requirement() {
            device.reputation = (device.reputation + self.config.uptime_bonus).min(100);
        }
        
        Ok(())
    }
    
    /// Claim accumulated rewards
    pub fn claim_rewards(&mut self, device_id: &[u8; 32]) -> Result<u64, &'static str> {
        let device = self.devices.get_mut(device_id)
            .ok_or("Device not found")?;
        
        let claimable = device.pending_rewards;
        if claimable == 0 {
            return Err("No rewards to claim");
        }
        
        device.total_claimed += claimable;
        device.pending_rewards = 0;
        self.total_distributed += claimable;
        
        Ok(claimable)
    }
    
    /// Process epoch transition
    pub fn advance_epoch(&mut self) -> EpochStats {
        self.current_epoch += 1;
        
        let mut total_uptime: f64 = 0.0;
        let mut active_count = 0u64;
        
        for device in self.devices.values_mut() {
            // Penalize devices that missed heartbeats
            let now = utils::now();
            let epochs_missed = ((now - device.last_heartbeat) / self.config.epoch_duration_secs)
                .saturating_sub(1);
            
            if epochs_missed > 0 {
                device.total_uptime_epochs = device.total_uptime_epochs
                    .saturating_sub(epochs_missed.min(device.total_uptime_epochs));
                self.update_reputation(&[0u8; 32], epochs_missed).ok();
            }
            
            if device.is_active {
                active_count += 1;
                total_uptime += device.uptime_percentage();
            }
        }
        
        let avg_uptime = if active_count > 0 {
            total_uptime / active_count as f64
        } else {
            0.0
        };
        
        let stats = EpochStats {
            epoch: self.current_epoch,
            total_devices: self.devices.len() as u64,
            active_devices: active_count,
            total_rewards_distributed: 0, // Calculated during reward distribution
            avg_uptime,
        };
        
        self.epoch_history.push(stats.clone());
        stats
    }
    
    /// Get device info
    pub fn get_device(&self, device_id: &[u8; 32]) -> Option<&MiningDevice> {
        self.devices.get(device_id)
    }
    
    /// Get all devices for an owner
    pub fn get_owner_devices(&self, owner: &[u8; 32]) -> Vec<&MiningDevice> {
        self.owner_devices
            .get(owner)
            .map(|ids| ids.iter().filter_map(|id| self.devices.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Calculate projected daily earnings for a device
    pub fn projected_daily_earnings(&self, device: &MiningDevice) -> u64 {
        let epochs_per_day = 86400 / self.config.epoch_duration_secs;
        let base_earnings = self.calculate_epoch_rewards(device, 1.0);
        base_earnings * epochs_per_day
    }
    
    /// Get mining statistics
    pub fn get_stats(&self) -> MiningStats {
        let total_devices = self.devices.len() as u64;
        let active_devices = self.devices.values().filter(|d| d.is_active).count() as u64;
        let avg_reputation = if total_devices > 0 {
            self.devices.values().map(|d| d.reputation as u64).sum::<u64>() / total_devices
        } else {
            0
        };
        
        MiningStats {
            total_devices,
            active_devices,
            avg_reputation: avg_reputation as u8,
            total_flux_distributed: self.total_distributed,
            current_epoch: self.current_epoch,
            avg_uptime: if active_devices > 0 {
                self.devices.values()
                    .filter(|d| d.is_active)
                    .map(|d| d.uptime_percentage())
                    .sum::<f64>() / active_devices as f64
            } else {
                0.0
            },
        }
    }
}

/// Mining statistics summary
#[derive(Debug, Clone)]
pub struct MiningStats {
    pub total_devices: u64,
    pub active_devices: u64,
    pub avg_reputation: u8,
    pub total_flux_distributed: u64,
    pub current_epoch: u64,
    pub avg_uptime: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_tier_multipliers() {
        assert_eq!(DeviceTier::Mobile.reward_multiplier(), 1.0);
        assert_eq!(DeviceTier::Laptop.reward_multiplier(), 1.5);
        assert_eq!(DeviceTier::Desktop.reward_multiplier(), 2.0);
        assert_eq!(DeviceTier::Server.reward_multiplier(), 3.0);
    }

    #[test]
    fn test_device_registration() {
        let mut mining = MiningContract::new(MiningConfig::default());
        let device_id = [1u8; 32];
        let owner = [2u8; 32];
        
        mining.register_device(device_id, owner, DeviceTier::Mobile).unwrap();
        
        let device = mining.get_device(&device_id).unwrap();
        assert_eq!(device.tier, DeviceTier::Mobile);
        assert_eq!(device.reputation, 50);
    }

    #[test]
    fn test_reward_calculation() {
        let mining = MiningContract::new(MiningConfig::default());
        let device = MiningDevice {
            owner: [1u8; 32],
            tier: DeviceTier::Mobile,
            registered_at: 0,
            last_heartbeat: 0,
            total_uptime_epochs: 100,
            total_mining_epochs: 100,
            pending_rewards: 0,
            total_claimed: 0,
            is_active: true,
            reputation: 100,
        };
        
        let reward = mining.calculate_epoch_rewards(&device, 1.0);
        assert!(reward > 0);
    }

    #[test]
    fn test_uptime_calculation() {
        let mut device = MiningDevice {
            owner: [1u8; 32],
            tier: DeviceTier::Mobile,
            registered_at: 0,
            last_heartbeat: 0,
            total_uptime_epochs: 80,
            total_mining_epochs: 100,
            pending_rewards: 0,
            total_claimed: 0,
            is_active: true,
            reputation: 50,
        };
        
        assert_eq!(device.uptime_percentage(), 0.8);
        assert!(device.meets_uptime_requirement()); // 80% >= 50% required
    }
}
