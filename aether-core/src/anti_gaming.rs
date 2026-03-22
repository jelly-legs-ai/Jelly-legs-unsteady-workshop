//! Anti-Gaming Module for Project AETHER
//! Detects and penalizes cheating behaviors in the mobile-mining blockchain

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Tracks device fingerprints and IP addresses for multiaccount detection
#[derive(Debug, Clone)]
pub struct DeviceRecord {
    pub device_id: String,
    pub ip_address: String,
    pub first_seen: u64,
    pub last_seen: u64,
}

/// AntiGaming module - detects various cheating strategies
#[derive(Debug, Clone)]
pub struct AntiGaming {
    /// Known device records keyed by device_id
    devices: HashMap<String, DeviceRecord>,
    /// IP to device_id mapping for multiaccount detection
    ip_to_devices: HashMap<String, Vec<String>>,
    /// Minimum stake required to participate
    min_stake: u64,
    /// Slash percentage for violations (0-100)
    slash_percentage: u8,
}

impl Default for AntiGaming {
    fn default() -> Self {
        Self::new(1000, 50)
    }
}

impl AntiGaming {
    /// Create a new AntiGaming detector
    pub fn new(min_stake: u64, slash_percentage: u8) -> Self {
        Self {
            devices: HashMap::new(),
            ip_to_devices: HashMap::new(),
            min_stake,
            slash_percentage,
        }
    }

    /// Register a device for tracking
    pub fn register_device(&mut self, device_id: String, ip_address: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let record = DeviceRecord {
            device_id: device_id.clone(),
            ip_address: ip_address.clone(),
            first_seen: now,
            last_seen: now,
        };

        // Update existing or insert new
        self.devices.insert(device_id.clone(), record);

        // Update IP mapping
        self.ip_to_devices
            .entry(ip_address)
            .or_insert_with(Vec::new)
            .push(device_id);
    }

    /// Update last_seen timestamp for a device
    pub fn heartbeat(&mut self, device_id: &str) {
        if let Some(record) = self.devices.get_mut(device_id) {
            record.last_seen = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }

    /// Detect emulator indicators
    /// Common emulator signs: suspicious CPU info, generic device names, 
    /// known emulator MAC addresses, etc.
    pub fn detect_emulator(&self, device_fingerprint: &DeviceFingerprint) -> bool {
        // Check for known emulator CPU identifiers
        if device_fingerprint.cpu_info.contains("Goldfish")
            || device_fingerprint.cpu_info.contains("Android SDK")
            || device_fingerprint.cpu_info.contains("Emulator")
            || device_fingerprint.cpu_info.contains("qemu")
        {
            return true;
        }

        // Check for generic/wrong manufacturer flags
        if device_fingerprint.manufacturer == "unknown"
            || device_fingerprint.manufacturer.is_empty()
        {
            return true;
        }

        // Check for suspicious screen resolution patterns (emulators often use standard ones)
        let suspicious_resolutions = ["600x1024", "720x1280", "1080x1920"];
        if suspicious_resolutions.contains(&device_fingerprint.resolution.as_str())
            && device_fingerprint.is_screen_square
        {
            // Square-ish resolutions are common in emulators
            return true;
        }

        // Check for known emulator MAC prefixes
        let emulator_mac_prefixes = ["00:00:00", "02:00:00", "00:15:5B", "A4:78:00"];
        for prefix in emulator_mac_prefixes {
            if device_fingerprint.mac_address.starts_with(prefix) {
                return true;
            }
        }

        false
    }

    /// Detect multiaccounting - same IP with multiple devices
    pub fn detect_multiaccount(&self, ip_address: &str, current_device_id: &str) -> DetectionResult {
        if let Some(devices) = self.ip_to_devices.get(ip_address) {
            let count = devices.len();
            
            // If this IP has been used by multiple different devices
            if count > 1 && !devices.contains(&current_device_id.to_string()) {
                return DetectionResult::new(
                    true,
                    format!(
                        "Multiaccount detected: IP {} has {} different devices",
                        ip_address, count
                    ),
                    50, // 50% slash for multiaccounting
                );
            }
        }

        DetectionResult::new(false, "No multiaccount detected".to_string(), 0)
    }

    /// Detect fake uptime - node claims more uptime than physically possible
    /// Uptime cannot exceed time since first registration
    pub fn detect_fake_uptime(&self, device_id: &str, claimed_uptime_secs: u64) -> DetectionResult {
        if let Some(record) = self.devices.get(device_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let actual_uptime = now.saturating_sub(record.first_seen);

            if claimed_uptime_secs > actual_uptime {
                let excess = claimed_uptime_secs - actual_uptime;
                return DetectionResult::new(
                    true,
                    format!(
                        "Fake uptime detected: claimed {}s but only {}s possible (excess: {}s)",
                        claimed_uptime_secs, actual_uptime, excess
                    ),
                    75, // 75% slash for fake uptime (more severe)
                );
            }
        }

        DetectionResult::new(false, "No fake uptime detected".to_string(), 0)
    }

    /// Calculate stake penalty based on violation severity
    pub fn slash_stake(&self, current_stake: u64, slash_percentage: u8) -> u64 {
        if slash_percentage == 0 || current_stake == 0 {
            return current_stake;
        }

        let penalty = (current_stake as u128 * slash_percentage as u128 / 100) as u64;
        current_stake.saturating_sub(penalty)
    }

    /// Get all devices associated with an IP
    pub fn get_devices_by_ip(&self, ip_address: &str) -> Vec<&DeviceRecord> {
        self.ip_to_devices
            .get(ip_address)
            .map(|devices| {
                devices
                    .iter()
                    .filter_map(|id| self.devices.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Device fingerprint for emulator detection
#[derive(Debug, Clone)]
pub struct DeviceFingerprint {
    pub device_id: String,
    pub cpu_info: String,
    pub manufacturer: String,
    pub model: String,
    pub resolution: String,
    pub is_screen_square: bool,
    pub mac_address: String,
}

impl Default for DeviceFingerprint {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            cpu_info: "Unknown".to_string(),
            manufacturer: "Unknown".to_string(),
            model: "Unknown".to_string(),
            resolution: "0x0".to_string(),
            is_screen_square: false,
            mac_address: "00:00:00:00:00:00".to_string(),
        }
    }
}

/// Result of a cheating detection check
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub is_cheating: bool,
    pub reason: String,
    pub slash_percentage: u8,
}

impl DetectionResult {
    pub fn new(is_cheating: bool, reason: String, slash_percentage: u8) -> Self {
        Self {
            is_cheating,
            reason,
            slash_percentage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_emulator_cpu_info() {
        let anti = AntiGaming::default();
        
        // Test emulator CPU detection
        let fp = DeviceFingerprint {
            device_id: "device1".to_string(),
            cpu_info: "Goldfish ARM ARMv7 Processor".to_string(),
            manufacturer: "Samsung".to_string(),
            resolution: "1440x2560".to_string(),
            is_screen_square: false,
            mac_address: "A4:78:00:12:34:56".to_string(),
        };
        assert!(anti.detect_emulator(&fp));
    }

    #[test]
    fn test_detect_emulator_mac() {
        let anti = AntiGaming::default();
        
        let fp = DeviceFingerprint {
            device_id: "device2".to_string(),
            cpu_info: "Qualcomm Snapdragon".to_string(),
            manufacturer: "Samsung".to_string(),
            resolution: "1440x2560".to_string(),
            is_screen_square: false,
            mac_address: "00:00:00:12:34:56".to_string(), // Emulator MAC prefix
        };
        assert!(anti.detect_emulator(&fp));
    }

    #[test]
    fn test_detect_emulator_clean() {
        let anti = AntiGaming::default();
        
        let fp = DeviceFingerprint {
            device_id: "device3".to_string(),
            cpu_info: "Qualcomm Snapdragon 888".to_string(),
            manufacturer: "Samsung".to_string(),
            model: "Galaxy S21".to_string(),
            resolution: "1440x3200".to_string(),
            is_screen_square: false,
            mac_address: "A4:5E:60:C1:2D:3E".to_string(),
        };
        assert!(!anti.detect_emulator(&fp));
    }

    #[test]
    fn test_detect_multiaccount() {
        let mut anti = AntiGaming::new(1000, 50);
        
        // Register multiple devices on same IP
        anti.register_device("device1".to_string(), "192.168.1.100".to_string());
        anti.register_device("device2".to_string(), "192.168.1.100".to_string());
        anti.register_device("device3".to_string(), "192.168.1.100".to_string());
        
        // Check from a new device on same IP
        let result = anti.detect_multiaccount("192.168.1.100", "device4");
        assert!(result.is_cheating);
        assert!(result.reason.contains("Multiaccount"));
        assert_eq!(result.slash_percentage, 50);
    }

    #[test]
    fn test_detect_multiaccount_single_device() {
        let mut anti = AntiGaming::new(1000, 50);
        
        // Register single device on IP
        anti.register_device("device1".to_string(), "192.168.1.100".to_string());
        
        // Same device checking in - should not be flagged
        let result = anti.detect_multiaccount("192.168.1.100", "device1");
        assert!(!result.is_cheating);
    }

    #[test]
    fn test_detect_fake_uptime() {
        let mut anti = AntiGaming::new(1000, 50);
        
        // Register device
        anti.register_device("device1".to_string(), "192.168.1.1".to_string());
        
        // Simulate claimed uptime of 1 year (impossible for newly registered device)
        let result = anti.detect_fake_uptime("device1", 365 * 24 * 60 * 60);
        assert!(result.is_cheating);
        assert!(result.reason.contains("Fake uptime"));
        assert_eq!(result.slash_percentage, 75);
    }

    #[test]
    fn test_detect_fake_uptime_legitimate() {
        let mut anti = AntiGaming::new(1000, 50);
        
        // Register device (this will set first_seen to now)
        anti.register_device("device1".to_string(), "192.168.1.1".to_string());
        
        // Claim 0 seconds - perfectly valid
        let result = anti.detect_fake_uptime("device1", 0);
        assert!(!result.is_cheating);
    }

    #[test]
    fn test_slash_stake() {
        let anti = AntiGaming::new(1000, 50);
        
        // 50% slash on 1000 stake = 500
        assert_eq!(anti.slash_stake(1000, 50), 500);
        
        // 25% slash on 1000 stake = 250
        assert_eq!(anti.slash_stake(1000, 25), 250);
        
        // 100% slash = 0
        assert_eq!(anti.slash_stake(1000, 100), 0);
        
        // 0% slash = full stake
        assert_eq!(anti.slash_stake(1000, 0), 1000);
    }

    #[test]
    fn test_slash_stake_does_not_go_negative() {
        let anti = AntiGaming::new(100, 75);
        
        // Small stake, high slash - should not underflow
        assert_eq!(anti.slash_stake(10, 75), 2); // 10 - (10 * 75 / 100) = 2
    }

    #[test]
    fn test_register_and_heartbeat() {
        let mut anti = AntiGaming::new(1000, 50);
        
        anti.register_device("device1".to_string(), "10.0.0.1".to_string());
        
        assert!(anti.devices.contains_key("device1"));
        assert!(anti.ip_to_devices.contains_key("10.0.0.1"));
        
        // Heartbeat should update last_seen without error
        anti.heartbeat("device1");
        
        // Unknown device should not panic
        anti.heartbeat("unknown_device");
    }

    #[test]
    fn test_get_devices_by_ip() {
        let mut anti = AntiGaming::new(1000, 50);
        
        anti.register_device("device1".to_string(), "192.168.1.1".to_string());
        anti.register_device("device2".to_string(), "192.168.1.1".to_string());
        anti.register_device("device3".to_string(), "192.168.1.2".to_string());
        
        let devices_on_ip1 = anti.get_devices_by_ip("192.168.1.1");
        assert_eq!(devices_on_ip1.len(), 2);
        
        let devices_on_ip2 = anti.get_devices_by_ip("192.168.1.2");
        assert_eq!(devices_on_ip2.len(), 1);
        
        let devices_on_unknown = anti.get_devices_by_ip("192.168.1.99");
        assert_eq!(devices_on_unknown.len(), 0);
    }

    #[test]
    fn test_detection_result() {
        let result = DetectionResult::new(true, "Test violation".to_string(), 25);
        
        assert!(result.is_cheating);
        assert_eq!(result.reason, "Test violation");
        assert_eq!(result.slash_percentage, 25);
    }
}
