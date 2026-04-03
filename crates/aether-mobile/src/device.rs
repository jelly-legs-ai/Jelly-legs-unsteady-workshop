//! Device capability detection

/// Device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub tier: super::pow::MobileTier,
    pub cores: usize,
    pub memory_gb: usize,
}
