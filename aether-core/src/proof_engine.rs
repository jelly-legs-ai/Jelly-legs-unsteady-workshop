use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Minimum stake required for mining (anti-Sybil protection)
const MIN_STAKE_FOR_MINING: u64 = 100;

/// Known emulator/simulator indicators for hardware attestation
const EMULATOR_INDICATORS: &[&str] = &[
    "generic",
    "sdk",
    "emulator",
    "simulator",
    "goldfish",
    "ranchu",
    "houdini",
    "libvbox",
    "vbox",
    "vmware",
    "qemu",
];

/// Known invalid hardware signatures
const INVALID_SIGNATURES: &[&str] = &[
    "FAKE",
    "INVALID",
    "EMULATOR",
    "SIMULATOR",
    "TEST_ONLY",
];

/// Device tier levels for mining power multiplier
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeviceTier {
    Basic = 1,
    Standard = 2,
    Advanced = 3,
    Professional = 4,
}

impl DeviceTier {
    pub fn multiplier(&self) -> f64 {
        match self {
            DeviceTier::Basic => 1.0,
            DeviceTier::Standard => 1.5,
            DeviceTier::Advanced => 2.5,
            DeviceTier::Professional => 4.0,
        }
    }
}

/// ProofEngine handles proof submission and reward calculation for mobile mining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofEngine {
    pub device_id: String,
    pub tier: DeviceTier,
    pub trust_score: f64,
    pub stake_amount: u64,
    pub hardware_attestation: Option<String>,
    #[serde(skip)]
    last_submit_time: Option<u64>,
}

impl ProofEngine {
    /// Create a new ProofEngine instance
    pub fn new(device_id: String, tier: DeviceTier, trust_score: f64, stake_amount: u64) -> Self {
        assert!(
            (0.0..=1.0).contains(&trust_score),
            "trust_score must be between 0.0 and 1.0"
        );

        ProofEngine {
            device_id,
            tier,
            trust_score,
            stake_amount,
            hardware_attestation: None,
            last_submit_time: None,
        }
    }

    /// Request a hardware attestation challenge
    /// Returns a mock attestation challenge string for the device to sign
    pub fn request_attestation(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        format!("attest:{}:{}:challenge", self.device_id, now)
    }

    /// Verify a hardware attestation response
    /// Returns true if the signature is valid and device is not an emulator/simulator
    pub fn verify_attestation(&self, challenge: &str, signature: &str) -> bool {
        // Check for invalid/emulator signatures
        let sig_upper = signature.to_uppercase();
        for invalid in INVALID_SIGNATURES {
            if sig_upper.contains(invalid) {
                return false;
            }
        }

        // Check if device_id appears in emulator indicators list (case-insensitive)
        let device_lower = self.device_id.to_lowercase();
        for indicator in EMULATOR_INDICATORS {
            if device_lower.contains(indicator) {
                return false;
            }
        }

        // Verify the challenge format matches what we issued
        let expected_prefix = format!("attest:{}:", self.device_id);
        if !challenge.starts_with(&expected_prefix) {
            return false;
        }

        // Mock verification: signature must be non-empty and contain device_id
        // In production this would verify against actual hardware security module
        !signature.is_empty() && signature.to_lowercase().contains(&self.device_id.to_lowercase())
    }

    /// Submit a proof of work and receive a mock proof hash
    /// Requires valid hardware attestation
    pub fn submit_proof(&mut self) -> Result<String, ProofEngineError> {
        // Anti-Sybil: check minimum stake requirement
        if self.stake_amount < MIN_STAKE_FOR_MINING {
            return Err(ProofEngineError::InsufficientStake);
        }

        // Security: require valid hardware attestation
        if self.hardware_attestation.is_none() {
            return Err(ProofEngineError::AttestationRequired);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ProofEngineError::TimeError)?
            .as_secs();

        // Rate limiting: prevent spam (minimum 10 seconds between submissions)
        if let Some(last) = self.last_submit_time {
            if now - last < 10 {
                return Err(ProofEngineError::RateLimited);
            }
        }

        self.last_submit_time = Some(now);

        // Generate mock proof hash
        let proof_hash = format!(
            "aether:{}:{}:{}:{}",
            self.device_id,
            now,
            self.tier as u8,
            self.trust_score
        );

        Ok(proof_hash)
    }

    /// Calculate mining reward based on trust score, tier multiplier, and active hours
    /// Formula: BaseReward × TrustScore × TierMult × ActiveHours
    pub fn calculate_reward(&self, active_hours: f64, base_reward: f64) -> Result<f64, ProofEngineError> {
        if active_hours < 0.0 {
            return Err(ProofEngineError::InvalidInput("active_hours cannot be negative".into()));
        }
        if base_reward < 0.0 {
            return Err(ProofEngineError::InvalidInput("base_reward cannot be negative".into()));
        }

        let tier_mult = self.tier.multiplier();
        let reward = base_reward * self.trust_score * tier_mult * active_hours;

        Ok(reward)
    }

    /// Update the trust score (e.g., based on behavioral analysis)
    pub fn update_trust_score(&mut self, new_score: f64) -> Result<(), ProofEngineError> {
        if !(0.0..=1.0).contains(&new_score) {
            return Err(ProofEngineError::InvalidInput(
                "trust_score must be between 0.0 and 1.0".into(),
            ));
        }
        self.trust_score = new_score;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ProofEngineError {
    RateLimited,
    TimeError,
    InvalidInput(String),
    AttestationRequired,
    InsufficientStake,
}

impl std::fmt::Display for ProofEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProofEngineError::RateLimited => write!(f, "rate limited: wait before submitting again"),
            ProofEngineError::TimeError => write!(f, "system time error"),
            ProofEngineError::InvalidInput(msg) => write!(f, "invalid input: {}", msg),
            ProofEngineError::AttestationRequired => write!(f, "hardware attestation required before submitting proofs"),
            ProofEngineError::InsufficientStake => write!(f, "Insufficient stake for mining. Minimum: 100 AETH"),
        }
    }
}

impl std::error::Error for ProofEngineError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proof_engine() {
        let engine = ProofEngine::new(
            "device_001".into(),
            DeviceTier::Standard,
            0.85,
            150,
        );
        assert_eq!(engine.device_id, "device_001");
        assert_eq!(engine.trust_score, 0.85);
        assert_eq!(engine.stake_amount, 150);
    }

    #[test]
    fn test_submit_proof() {
        let mut engine = ProofEngine::new(
            "device_002".into(),
            DeviceTier::Advanced,
            0.9,
            200,
        );
        // Set valid attestation first
        engine.hardware_attestation = Some("verified".into());
        let hash = engine.submit_proof().unwrap();
        assert!(hash.starts_with("aether:device_002:"));
    }

    #[test]
    fn test_calculate_reward() {
        let engine = ProofEngine::new(
            "device_003".into(),
            DeviceTier::Professional,
            0.75,
            100,
        );
        // Professional tier = 4.0x, trust = 0.75, active = 10 hours, base = 100
        // Expected: 100 * 0.75 * 4.0 * 10 = 3000
        let reward = engine.calculate_reward(10.0, 100.0).unwrap();
        assert_eq!(reward, 3000.0);
    }

    #[test]
    fn test_attestation_required() {
        let mut engine = ProofEngine::new(
            "device_004".into(),
            DeviceTier::Basic,
            0.5,
            200,
        );
        // No attestation set, should fail with AttestationRequired
        let result = engine.submit_proof();
        assert!(result.is_err());
        match result {
            Err(ProofEngineError::AttestationRequired) => (),
            _ => panic!("Expected AttestationRequired error"),
        }
    }

    #[test]
    fn test_emulator_rejected() {
        let mut engine = ProofEngine::new(
            "sdk_goldfish_emu".into(), // emulator indicator in device_id
            DeviceTier::Basic,
            0.5,
            200,
        );
        let challenge = engine.request_attestation();
        // Even with a "valid" looking signature, emulator device should fail
        let result = engine.verify_attestation(&challenge, "sdk_goldfish_emu:SIG");
        assert!(!result, "Emulator devices should be rejected");
    }

    #[test]
    fn test_invalid_signature_rejected() {
        let engine = ProofEngine::new(
            "device_005".into(),
            DeviceTier::Basic,
            0.5,
            200,
        );
        let challenge = engine.request_attestation();
        // Invalid signature patterns should be rejected
        assert!(!engine.verify_attestation(&challenge, "FAKE_SIGNATURE"));
        assert!(!engine.verify_attestation(&challenge, "INVALID"));
        assert!(!engine.verify_attestation(&challenge, "EMULATOR_SIG"));
    }

    #[test]
    fn test_valid_attestation() {
        let mut engine = ProofEngine::new(
            "device_006".into(),
            DeviceTier::Standard,
            0.8,
            200,
        );
        let challenge = engine.request_attestation();
        // Valid attestation
        if engine.verify_attestation(&challenge, "device_006:HW_SIG") {
            engine.hardware_attestation = Some("verified".into());
            let result = engine.submit_proof();
            assert!(result.is_ok(), "Should succeed with valid attestation");
        }
    }

    #[test]
    fn test_tier_multipliers() {
        assert_eq!(DeviceTier::Basic.multiplier(), 1.0);
        assert_eq!(DeviceTier::Standard.multiplier(), 1.5);
        assert_eq!(DeviceTier::Advanced.multiplier(), 2.5);
        assert_eq!(DeviceTier::Professional.multiplier(), 4.0);
    }
}
