//! Proof of Work for mobile devices

use serde::{Deserialize, Serialize};

/// Mobile PoW result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobilePowResult {
    pub hash: [u8; 32],
    pub device_tier: MobileTier,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MobileTier {
    Light,
    Standard,
    Performance,
}
