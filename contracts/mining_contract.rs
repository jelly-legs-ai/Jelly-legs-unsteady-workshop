// Mining Contract - AeTHer Chain
// FLUX token mining through device participation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Device tier for mining
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceTier {
    Mobile,
    Laptop,
    Desktop,
    Server,
}

impl DeviceTier {
    /// Get hashrate multiplier for this tier
    pub fn hashrate_multiplier(&self) -> f64 {
        match self {
            DeviceTier::Mobile => 0.5,
            DeviceTier::Laptop => 2.0,
            DeviceTier::Desktop => 5.0,
            DeviceTier::Server => 20.0,
        }
    }
    
    /// Get earnings multiplier
    pub fn earnings_multiplier(&self) -> f64 {
        match self {
            DeviceTier::Mobile => 0.1,
            DeviceTier::Laptop => 1.0,
            DeviceTier::Desktop => 2.5,
            DeviceTier::Server => 10.0,
        }
    }
}

/// Mining device registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningDevice {
    pub device_id: String,
    pub owner: String,
    pub tier: DeviceTier,
    pub hashrate: u64,          // Current hashrate (MH/s)
    pub uptime_hours: u64,
    pub total_mined: u64,        // Total FLUX mined
    pub registered_epoch: u64,
    pub last_claim_epoch: u64,
    pub is_active: bool,
}

/// Mining pool info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningPool {
    pub name: String,
    pub total_hashrate: u64,
    pub active_miners: u64,
    pub epoch_reward: u64,
    pub difficulty: u64,
}

/// Mining contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningContract {
    pub devices: HashMap<String, MiningDevice>,
    pub pools: HashMap<String, MiningPool>,
    pub current_epoch: u64,
    pub total_flux_mined: u64,
    pub base_reward_per_epoch: u64,
}

impl MiningContract {
    /// Create new mining contract
    pub fn new() -> Self {
        let mut pools = HashMap::new();
        
        // Default FLUX mining pool
        pools.insert("flux_main".to_string(), MiningPool {
            name: "FLUX Main Pool".to_string(),
            total_hashrate: 0,
            active_miners: 0,
            epoch_reward: 1000,
            difficulty: 1000000,
        });
        
        MiningContract {
            devices: HashMap::new(),
            pools,
            current_epoch: 0,
            total_flux_mined: 0,
            base_reward_per_epoch: 1000,
        }
    }
    
    /// Register a new mining device
    pub fn register_device(
        &mut self,
        device_id: String,
        owner: String,
        tier: DeviceTier,
    ) -> Result<MiningDevice, &'static str> {
        if self.devices.contains_key(&device_id) {
            return Err("Device already registered");
        }
        
        let base_hashrate: u64 = match tier {
            DeviceTier::Mobile => 5,
            DeviceTier::Laptop => 20,
            DeviceTier::Desktop => 100,
            DeviceTier::Server => 500,
        };
        
        let device = MiningDevice {
            device_id: device_id.clone(),
            owner,
            tier: tier.clone(),
            hashrate: base_hashrate,
            uptime_hours: 0,
            total_mined: 0,
            registered_epoch: self.current_epoch,
            last_claim_epoch: self.current_epoch,
            is_active: true,
        };
        
        self.devices.insert(device_id.clone(), device.clone());
        
        // Update pool stats
        if let Some(pool) = self.pools.get_mut("flux_main") {
            pool.active_miners += 1;
            pool.total_hashrate += base_hashrate;
        }
        
        Ok(device)
    }
    
    /// Calculate mining reward for a device
    pub fn calculate_reward(&self, device: &MiningDevice) -> u64 {
        if !device.is_active {
            return 0;
        }
        
        // Base reward * tier multiplier * uptime factor
        let tier_mult = device.tier.earnings_multiplier();
        let uptime_factor = (device.uptime_hours as f64 / 24.0).min(1.0).max(0.1);
        
        let pool = self.pools.get("flux_main").unwrap();
        let network_share = device.hashrate as f64 / pool.total_hashrate as f64.max(1.0);
        
        let reward = (self.base_reward_per_epoch as f64 * tier_mult * uptime_factor * network_share) as u64;
        reward.max(1) // Minimum 1 FLUX
    }
    
    /// Update device uptime
    pub fn update_uptime(&mut self, device_id: &str, hours: u64) -> Result<(), &'static str> {
        let device = self.devices.get_mut(device_id)
            .ok_or("Device not found")?;
        device.uptime_hours = device.uptime_hours.saturating_add(hours).min(24);
        Ok(())
    }
    
    /// Claim mining rewards
    pub fn claim_rewards(&mut self, device_id: &str) -> Result<u64, &'static str> {
        let device = self.devices.get_mut(device_id)
            .ok_or("Device not found")?;
        
        if !device.is_active {
            return Err("Device is not active");
        }
        
        let rewards = self.calculate_reward(device);
        
        if rewards == 0 {
            return Err("No rewards to claim");
        }
        
        device.total_mined += rewards;
        device.last_claim_epoch = self.current_epoch;
        self.total_flux_mined += rewards;
        
        Ok(rewards)
    }
    
    /// Deregister a device
    pub fn deregister_device(&mut self, device_id: &str) -> Result<u64, &'static str> {
        let device = self.devices.remove(device_id)
            .ok_or("Device not found")?;
        
        // Update pool
        if let Some(pool) = self.pools.get_mut("flux_main") {
            pool.active_miners = pool.active_miners.saturating_sub(1);
            pool.total_hashrate = pool.total_hashrate.saturating_sub(device.hashrate);
        }
        
        Ok(device.total_mined)
    }
    
    /// Advance epoch and distribute rewards
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
    }
    
    /// Get network stats
    pub fn get_network_stats(&self) -> MiningNetworkStats {
        let pool = self.pools.get("flux_main").unwrap();
        MiningNetworkStats {
            total_hashrate: pool.total_hashrate,
            active_miners: pool.active_miners,
            epoch: self.current_epoch,
            difficulty: pool.difficulty,
            total_flux_mined: self.total_flux_mined,
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningNetworkStats {
    pub total_hashrate: u64,
    pub active_miners: u64,
    pub epoch: u64,
    pub difficulty: u64,
    pub total_flux_mined: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_device() {
        let mut contract = MiningContract::new();
        let device = contract.register_device(
            "device_001".to_string(),
            "owner_1".to_string(),
            DeviceTier::Laptop,
        ).unwrap();
        
        assert_eq!(device.tier, DeviceTier::Laptop);
        assert_eq!(device.hashrate, 20);
    }

    #[test]
    fn test_claim_rewards() {
        let mut contract = MiningContract::new();
        contract.register_device(
            "device_002".to_string(),
            "owner_2".to_string(),
            DeviceTier::Desktop,
        ).unwrap();
        
        contract.update_uptime("device_002", 24).unwrap();
        let rewards = contract.claim_rewards("device_002").unwrap();
        
        assert!(rewards > 0);
    }

    #[test]
    fn test_deregister_device() {
        let mut contract = MiningContract::new();
        contract.register_device(
            "device_003".to_string(),
            "owner_3".to_string(),
            DeviceTier::Server,
        ).unwrap();
        
        let total = contract.deregister_device("device_003").unwrap();
        assert!(contract.devices.get("device_003").is_none());
    }
}
