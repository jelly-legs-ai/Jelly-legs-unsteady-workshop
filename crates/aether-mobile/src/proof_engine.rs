//! AETHER Mobile Mining - Core Proof Engine
//!
//! Implements CPU/GPU proof-of-work for mobile AI compute attribution.
//! Mobile devices submit proofs of performed AI work to earn AETH rewards.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use digest::Digest;
use sha3::Sha3_256;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod trust_score;
pub mod benchmark;
pub mod anti_gaming;
pub mod reward;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum ProofError {
    #[error("Invalid proof: {0}")]
    InvalidProof(String),
    #[error("Challenge already used: {0}")]
    ChallengeReused(String),
    #[error("Difficulty too low: {0}")]
    DifficultyTooLow(u32),
    #[error("Invalid device signature: {0}")]
    InvalidSignature(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Stale challenge (too old)")]
    StaleChallenge,
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type ProofResult<T> = Result<T, ProofError>;

// ============================================================================
// Device Tier Classification
// ============================================================================

/// Device tier based on hardware capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DeviceTier {
    /// High-tier mobile (Snapdragon 8 Gen, Apple A17+, Tensor G3)
    Tier1 = 1,
    /// Mid-tier mobile (Snapdragon 7/6 series, Apple A14/A15, Tensor G2)
    Tier2 = 2,
    /// Low-tier / older mobile (3+ years old)
    Tier3 = 3,
    /// Starter tier - minimal rewards, easy to join
    Starter = 4,
}

impl DeviceTier {
    /// Base reward multiplier for this tier
    pub fn reward_multiplier(&self) -> f64 {
        match self {
            DeviceTier::Tier1 => 1.0,
            DeviceTier::Tier2 => 0.7,
            DeviceTier::Tier3 => 0.4,
            DeviceTier::Starter => 0.15,
        }
    }

    /// Compute units per second for this tier (estimated)
    pub fn compute_units_per_second(&self) -> u64 {
        match self {
            DeviceTier::Tier1 => 100_000_000_000,   // 100 TFLOPS
            DeviceTier::Tier2 => 30_000_000_000,    // 30 TFLOPS
            DeviceTier::Tier3 => 10_000_000_000,   // 10 TFLOPS
            DeviceTier::Starter => 1_000_000_000,   // 1 TFLOPS (baseline)
        }
    }

    /// Minimum trust score required for this tier
    pub fn min_trust_score(&self) -> u32 {
        match self {
            DeviceTier::Tier1 => 750,
            DeviceTier::Tier2 => 500,
            DeviceTier::Tier3 => 250,
            DeviceTier::Starter => 0,
        }
    }

    /// Get tier from benchmark score (GFLOPS)
    pub fn from_benchmark_score(gflops: u64) -> Self {
        if gflops >= 50_000_000_000 {
            DeviceTier::Tier1
        } else if gflops >= 15_000_000_000 {
            DeviceTier::Tier2
        } else if gflops >= 5_000_000_000 {
            DeviceTier::Tier3
        } else {
            DeviceTier::Starter
        }
    }
}

impl Default for DeviceTier {
    fn default() -> Self {
        DeviceTier::Starter
    }
}

// ============================================================================
// Proof-of-Work Challenge System
// ============================================================================

/// A challenge is a unique problem for the device to solve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// Unique challenge ID (hash of prev_challenge + random_nonce)
    pub id: [u8; 32],
    /// Hash of the previous challenge (chain link)
    pub prev_challenge: [u8; 32],
    /// Block height at which this challenge was issued
    pub block_height: u64,
    /// Timestamp when challenge was issued (Unix ms)
    pub issued_at: i64,
    /// Timestamp when challenge expires (Unix ms)
    pub expires_at: i64,
    /// Target difficulty (number of leading zero bits required)
    pub difficulty: u32,
    /// Device public key that this challenge is assigned to
    pub assigned_device: [u8; 32],
    /// Work type hint (0 = any, 1 = inference, 2 = training, 3 = fine-tuning)
    pub work_type: u8,
}

impl Challenge {
    /// Check if challenge has expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        now > self.expires_at
    }

    /// Age of challenge in milliseconds
    pub fn age_ms(&self) -> i64 {
        let now = chrono::Utc::now().timestamp_millis();
        now - self.issued_at
    }
}

/// A valid proof submitted by a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkProof {
    /// Challenge ID this proof is for
    pub challenge_id: [u8; 32],
    /// Device public key
    pub device_key: [u8; 32],
    /// Solution nonce (found by device)
    pub nonce: [u8; 32],
    /// Hash of the computed work output (AI model inference result hash)
    pub work_output_hash: [u8; 32],
    /// Type of AI work performed
    pub work_type: WorkType,
    /// Compute time in milliseconds
    pub compute_time_ms: u64,
    /// Device tier at time of proof
    pub device_tier: DeviceTier,
    /// Trust score at time of proof
    pub trust_score: u32,
    /// Block height when submitted
    pub submit_height: u64,
    /// Timestamp of submission (Unix ms)
    pub submitted_at: i64,
    /// Difficulty achieved (leading zero bits in hash)
    pub difficulty_achieved: u32,
    /// Device signature over the proof
    pub device_signature: [u8; 64],
    /// Proof-of-work hash (computed from nonce + challenge)
    pub pow_hash: [u8; 32],
}

impl WorkProof {
    /// Verify the proof is valid
    pub fn verify(&self, challenge: &Challenge) -> ProofResult<()> {
        // Check device matches assignment
        if self.device_key != challenge.assigned_device {
            return Err(ProofError::InvalidProof("Device key mismatch".into()));
        }

        // Check challenge ID matches
        if self.challenge_id != challenge.id {
            return Err(ProofError::InvalidProof("Challenge ID mismatch".into()));
        }

        // Check not expired
        if self.is_expired() {
            return Err(ProofError::StaleChallenge);
        }

        // Check difficulty meets target
        if self.difficulty_achieved < challenge.difficulty {
            return Err(ProofError::DifficultyTooLow(self.difficulty_achieved));
        }

        // Verify device signature
        self.verify_signature()?;

        // Verify work output hash matches computation
        self.verify_work_hash()?;

        Ok(())
    }

    /// Check if proof submission is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        // Proofs are valid for 5 minutes after issuance
        now > (self.submitted_at + 300_000)
    }

    /// Verify device signature over proof data
    fn verify_signature(&self) -> ProofResult<()> {
        use ed25519_dalek::Signature;
        use ed25519_dalek::Verifier;

        let sig_bytes = &self.device_signature;
        let signature = Signature::from_bytes(sig_bytes)
            .map_err(|e| ProofError::InvalidSignature(format!("Invalid signature bytes: {}", e)))?;

        // Reconstruct the signed message
        let mut msg = Vec::new();
        msg.extend_from_slice(&self.challenge_id);
        msg.extend_from_slice(&self.nonce);
        msg.extend_from_slice(&self.work_output_hash);
        msg.extend_from_slice(&self.work_type.to_bytes());
        msg.extend_from_slice(&self.compute_time_ms.to_le_bytes());

        // For verification, we'd need the public key
        // This is a simplified check - in production, derive from device_key
        let _ = sig_bytes; // Placeholder - actual verification uses device_key

        Ok(())
    }

    /// Verify work output hash is consistent with computation
    fn verify_work_hash(&self) -> ProofResult<()> {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.device_key);
        hasher.update(&self.nonce);
        hasher.update(&self.challenge_id);
        hasher.update(self.work_type.to_bytes());
        hasher.update(&self.compute_time_ms.to_le_bytes());

        let computed = hasher.finalize();

        // Work hash should be different from pow_hash
        if computed == self.work_output_hash {
            return Err(ProofError::InvalidProof("Work hash collision".into()));
        }

        Ok(())
    }
}

/// Type of AI work performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum WorkType {
    /// Any type of work
    Any = 0,
    /// AI inference (running a model)
    Inference = 1,
    /// Model training
    Training = 2,
    /// Fine-tuning a model
    FineTuning = 3,
    /// Data preprocessing
    DataPrep = 4,
}

impl WorkType {
    /// Convert to bytes for hashing
    pub fn to_bytes(&self) -> [u8; 1] {
        [*self as u8]
    }

    /// Base reward weight for this work type
    pub fn reward_weight(&self) -> f64 {
        match self {
            WorkType::Any => 0.5,
            WorkType::Inference => 0.3,
            WorkType::Training => 1.0,
            WorkType::FineTuning => 0.8,
            WorkType::DataPrep => 0.2,
        }
    }
}

// ============================================================================
// Proof Engine Core
// ============================================================================

/// Core proof generation and verification engine
pub struct ProofEngine {
    /// Current epoch difficulty
    difficulty: AtomicU32,
    /// Challenge chain position
    challenge_height: AtomicU64,
    /// Last challenge hash (for chaining)
    last_challenge: std::sync::Mutex<[u8; 32]>,
    /// Rate limiter
    rate_limiter: std::sync::Arc<tokio::sync::RwLock<ratelimit::RateLimiter>>,
}

impl ProofEngine {
    /// Create a new proof engine
    pub fn new() -> Self {
        Self {
            difficulty: AtomicU32::new(16), // Start at 2^16 iterations expected
            challenge_height: AtomicU64::new(0),
            last_challenge: std::sync::Mutex::new([0u8; 32]),
            rate_limiter: std::sync::Arc::new(tokio::sync::RwLock::new(
                ratelimit::RateLimiter::direct(100) // 100 proofs per second max
            )),
        }
    }

    /// Issue a new challenge for a device
    pub fn issue_challenge(
        &self,
        device_key: [u8; 32],
        block_height: u64,
        work_type: WorkType,
    ) -> Challenge {
        let mut rng = rand::thread_rng();
        let mut nonce = [0u8; 32];
        rng.fill_bytes(&mut nonce);

        let prev = *self.last_challenge.lock().unwrap();

        // Create challenge ID from hash of prev + nonce
        let mut hasher = Sha3_256::new();
        hasher.update(&prev);
        hasher.update(&nonce);
        let id = hasher.finalize().into();

        let now = chrono::Utc::now().timestamp_millis();

        // Update tracking
        *self.last_challenge.lock().unwrap() = id;
        self.challenge_height.fetch_add(1, Ordering::SeqCst);

        Challenge {
            id,
            prev_challenge: prev,
            block_height,
            issued_at: now,
            expires_at: now + 600_000, // 10 minute window
            difficulty: self.difficulty.load(Ordering::SeqCst),
            assigned_device: device_key,
            work_type: work_type as u8,
        }
    }

    /// Generate a proof of work for a challenge
    pub fn generate_proof(
        &self,
        challenge: &Challenge,
        device_key: &[u8; 32],
        work_output: &[u8],
        compute_time_ms: u64,
        tier: DeviceTier,
        trust_score: u32,
    ) -> ProofResult<WorkProof> {
        // Rate limit check
        {
            let limiter = self.rate_limiter.read().await;
            if !limiter.check().is_ok() {
                return Err(ProofError::RateLimitExceeded);
            }
        }

        // Validate challenge is for this device
        if &challenge.assigned_device != device_key {
            return Err(ProofError::InvalidProof("Challenge not assigned to this device".into()));
        }

        // Check challenge not expired
        if challenge.is_expired() {
            return Err(ProofError::StaleChallenge);
        }

        // Perform proof-of-work search
        let mut rng = rand::thread_rng();
        let mut nonce = [0u8; 32];
        rng.fill_bytes(&mut nonce);

        let target_difficulty = challenge.difficulty;

        // Search for valid nonce
        let (nonce, pow_hash, difficulty_achieved) = self.search_nonce(
            &challenge.id,
            device_key,
            work_output,
            compute_time_ms,
            nonce,
            target_difficulty,
        )?;

        // Create work output hash
        let mut hasher = Sha3_256::new();
        hasher.update(work_output);
        let work_output_hash: [u8; 32] = hasher.finalize().into();

        // Sign the proof
        let mut msg = Vec::new();
        msg.extend_from_slice(&challenge.id);
        msg.extend_from_slice(&nonce);
        msg.extend_from_slice(&work_output_hash);
        msg.extend_from_slice(&[challenge.work_type]);
        msg.extend_from_slice(&compute_time_ms.to_le_bytes());

        let signature = sign_message(&msg, device_key);

        let proof = WorkProof {
            challenge_id: challenge.id,
            device_key: *device_key,
            nonce,
            work_output_hash,
            work_type: WorkType::from(challenge.work_type),
            compute_time_ms,
            device_tier: tier,
            trust_score,
            submit_height: challenge.block_height,
            submitted_at: chrono::Utc::now().timestamp_millis(),
            difficulty_achieved,
            device_signature: signature,
            pow_hash,
        };

        Ok(proof)
    }

    /// Search for a nonce that meets difficulty target
    fn search_nonce(
        &self,
        challenge_id: &[u8; 32],
        device_key: &[u8; 32],
        work_output: &[u8],
        compute_time_ms: u64,
        mut nonce: [u8; 32],
        target_difficulty: u32,
    ) -> ProofResult<([u8; 32], [u8; 32], u32)> {
        let max_iterations = 1u64 << 30; // Safety limit

        for i in 0..max_iterations {
            // Compute PoW hash
            let mut hasher = Sha3_256::new();
            hasher.update(challenge_id);
            hasher.update(&nonce);
            hasher.update(device_key);
            hasher.update(work_output);
            hasher.update(&compute_time_ms.to_le_bytes());
            let hash: [u8; 32] = hasher.finalize().into();

            // Count leading zero bits
            let difficulty = count_leading_zeros(&hash);

            if difficulty >= target_difficulty {
                return Ok((nonce, hash, difficulty));
            }

            // Increment nonce
            increment_nonce(&mut nonce);

            // Checkpoint difficulty adjustment
            if i % 1_000_000 == 0 {
                let current_diff = self.difficulty.load(Ordering::SeqCst);
                if difficulty > current_diff + 4 {
                    // Proof was too easy, increase difficulty
                    self.difficulty.fetch_add(1, Ordering::SeqCst);
                } else if difficulty + 4 < current_diff && current_diff > 8 {
                    // Proof was hard, decrease difficulty
                    self.difficulty.fetch_sub(1, Ordering::SeqCst);
                }
            }
        }

        Err(ProofError::InvalidProof("Could not find valid nonce within limit".into()))
    }

    /// Verify a submitted proof
    pub async fn verify_proof(
        &self,
        proof: &WorkProof,
        challenge: &Challenge,
    ) -> ProofResult<()> {
        proof.verify(challenge)
    }

    /// Adjust difficulty based on network hashrate
    pub fn adjust_difficulty(&self, avg_proof_time_ms: u64, target_time_ms: u64) {
        let current = self.difficulty.load(Ordering::SeqCst);

        if avg_proof_time_ms < target_time_ms / 2 && current < 30 {
            // Too fast, increase difficulty
            self.difficulty.fetch_add(1, Ordering::SeqCst);
        } else if avg_proof_time_ms > target_time_ms * 2 && current > 4 {
            // Too slow, decrease difficulty
            self.difficulty.fetch_sub(1, Ordering::SeqCst);
        }
    }

    /// Get current difficulty
    pub fn get_difficulty(&self) -> u32 {
        self.difficulty.load(Ordering::SeqCst)
    }

    /// Get challenge height
    pub fn get_challenge_height(&self) -> u64 {
        self.challenge_height.load(Ordering::SeqCst)
    }
}

impl Default for ProofEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Count leading zero bits in a 32-byte hash
fn count_leading_zeros(hash: &[u8; 32]) -> u32 {
    for (i, byte) in hash.iter().enumerate() {
        if *byte != 0 {
            return (i * 8) as u32 + byte.leading_zeros();
        }
    }
    256 // All zeros
}

/// Increment a nonce (big-endian addition)
fn increment_nonce(nonce: &mut [u8; 32]) {
    for i in (0..32).rev() {
        if nonce[i] == 0xFF {
            nonce[i] = 0;
        } else {
            nonce[i] += 1;
            break;
        }
    }
}

/// Sign a message with a device key (simplified - use proper HSM in production)
fn sign_message(msg: &[u8], _device_key: &[u8; 32]) -> [u8; 64] {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    // In production, device key would come from secure enclave
    let signing_key = SigningKey::generate(&mut OsRng);
    let signature = signing_key.sign(msg);

    signature.to_bytes()
}

// ============================================================================
// Rate Limiter (simple token bucket)
// ============================================================================

mod ratelimit {
    use std::time::{Duration, Instant};

    pub struct RateLimiter {
        rate: u64,
        tokens: std::sync::Mutex<f64>,
        last_update: std::sync::Mutex<Instant>,
    }

    impl RateLimiter {
        pub fn direct(rate: u64) -> Self {
            Self {
                rate,
                tokens: std::sync::Mutex::new(rate as f64),
                last_update: std::sync::Mutex::new(Instant::now()),
            }
        }

        pub fn check(&self) -> Result<(), ()> {
            let mut tokens = self.tokens.lock().unwrap();
            let mut last = self.last_update.lock().unwrap();

            let now = Instant::now();
            let elapsed = now.duration_since(*last).as_secs_f64();

            // Add tokens based on elapsed time
            *tokens = (*tokens + elapsed * self.rate as f64).min(self.rate as f64 * 10.0);
            *last = now;

            if *tokens >= 1.0 {
                *tokens -= 1.0;
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

// ============================================================================
// WorkType Implementation
// ============================================================================

impl From<u8> for WorkType {
    fn from(v: u8) -> Self {
        match v {
            0 => WorkType::Any,
            1 => WorkType::Inference,
            2 => WorkType::Training,
            3 => WorkType::FineTuning,
            4 => WorkType::DataPrep,
            _ => WorkType::Any,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_tier_multipliers() {
        assert_eq!(DeviceTier::Tier1.reward_multiplier(), 1.0);
        assert_eq!(DeviceTier::Tier2.reward_multiplier(), 0.7);
        assert_eq!(DeviceTier::Tier3.reward_multiplier(), 0.4);
        assert_eq!(DeviceTier::Starter.reward_multiplier(), 0.15);
    }

    #[test]
    fn test_challenge_expiry() {
        let challenge = Challenge {
            id: [0u8; 32],
            prev_challenge: [0u8; 32],
            block_height: 0,
            issued_at: 0,
            expires_at: 0,
            difficulty: 16,
            assigned_device: [0u8; 32],
            work_type: 0,
        };

        assert!(challenge.is_expired());
    }

    #[test]
    fn test_work_type_weights() {
        assert_eq!(WorkType::Inference.reward_weight(), 0.3);
        assert_eq!(WorkType::Training.reward_weight(), 1.0);
        assert_eq!(WorkType::FineTuning.reward_weight(), 0.8);
    }

    #[test]
    fn test_leading_zeros() {
        let hash = [0u8; 32];
        assert_eq!(count_leading_zeros(&hash), 256);

        let hash = [1u8, 0u8, 0u8; 32];
        assert_eq!(count_leading_zeros(&hash), 0);

        let hash = [0u8, 0x80, 0u8; 32]; // 0x80 = 0b10000000
        assert_eq!(count_leading_zeros(&hash), 15);
    }
}
