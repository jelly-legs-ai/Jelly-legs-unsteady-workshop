//! AI Transaction Classifier Module
//!
//! Classifies transactions into priority lanes based on content analysis.
//! AI operators pay premium gas fees for Critical/High priority execution.

use aether_common::types::AIPriorityLane;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// Classification threshold constants
/// One critical marker (0.2) should be enough to classify as Critical
pub const CRITICAL_CONFIDENCE_THRESHOLD: f64 = 0.15;
pub const HIGH_CONFIDENCE_THRESHOLD: f64 = 0.25;

/// Transaction type markers that indicate priority
const CRITICAL_MARKERS: &[&[u8]] = &[
    b"emergency",
    b"critical",
    b"governance",
    b"emergency_shutdown",
    b"protocol_upgrade",
    b"slash_validator",
    b"emergency_withdraw",
    b"pause_contract",
    b"circuit_breaker",
];

const HIGH_MARKERS: &[&[u8]] = &[
    b"ai_agent",
    b"mev",
    b"arbitrage",
    b"liquidation",
    b"flash_loan",
    b"smart_contract",
    b"defi",
    b"swap",
    b"trade",
    b"bridge",
];

/// Classifier result
#[derive(Debug, Clone)]
pub struct Classification {
    /// Determined priority lane
    pub lane: AIPriorityLane,
    /// Confidence score (0.0 - 1.0)
    pub score: f64,
    /// Reasons for classification
    pub reasons: Vec<String>,
    /// Compute units estimated for this transaction
    pub estimated_compute_units: u64,
    /// Recommended priority fee
    pub recommended_fee: u64,
}

impl Classification {
    /// Create a new classification
    pub fn new(lane: AIPriorityLane, score: f64, reasons: Vec<String>) -> Self {
        let compute_units = Self::estimate_compute(lane);
        let recommended_fee = Self::calculate_fee(lane, compute_units);
        
        Self {
            lane,
            score,
            reasons,
            estimated_compute_units: compute_units,
            recommended_fee,
        }
    }
    
    /// Estimate compute units based on lane
    fn estimate_compute(lane: AIPriorityLane) -> u64 {
        match lane {
            AIPriorityLane::Critical => 1_000_000, // Complex governance/emergency ops
            AIPriorityLane::High => 400_000,       // AI agent operations
            AIPriorityLane::Standard => 200_000,   // Standard transactions
        }
    }
    
    /// Calculate priority fee
    fn calculate_fee(lane: AIPriorityLane, compute_units: u64) -> u64 {
        let base_price = match lane {
            AIPriorityLane::Critical => 10_000,
            AIPriorityLane::High => 1_000,
            AIPriorityLane::Standard => 100,
        };
        compute_units * base_price
    }
}

/// Classify a transaction by analyzing its content
/// 
/// This function examines transaction data for markers that indicate
/// the appropriate priority lane. AI transactions are identified by
/// specific patterns and signatures.
pub fn classify(tx_data: &[u8]) -> Classification {
    let (lane, score, reasons) = analyze_transaction(tx_data);
    Classification::new(lane, score, reasons)
}

/// Internal analysis function
fn analyze_transaction(data: &[u8]) -> (AIPriorityLane, f64, Vec<String>) {
    let mut reasons = Vec::new();
    let mut critical_score = 0.0;
    let mut high_score = 0.0;
    
    // Check for critical markers
    for marker in CRITICAL_MARKERS {
        if contains_marker(data, marker) {
            critical_score += 0.2;
            reasons.push(format!("contains critical marker: {}", 
                String::from_utf8_lossy(marker)));
        }
    }
    
    // Check for high priority markers
    for marker in HIGH_MARKERS {
        if contains_marker(data, marker) {
            high_score += 0.15;
            reasons.push(format!("contains high priority marker: {}", 
                String::from_utf8_lossy(marker)));
        }
    }
    
    // Check for AI signature pattern (would be verified separately by oracle)
    if has_ai_signature_pattern(data) {
        high_score += 0.3;
        reasons.push("contains AI signature pattern".to_string());
    }
    
    // Check transaction size - larger transactions may be complex AI operations
    if data.len() > 1000 {
        high_score += 0.1;
        reasons.push("large transaction size suggests complex operation".to_string());
    }
    
    // Check for compute-intensive patterns (function selectors)
    if has_compute_intensive_selector(data) {
        high_score += 0.15;
        reasons.push("compute-intensive function selector detected".to_string());
    }
    
    // Determine final lane
    let (lane, score) = if critical_score >= CRITICAL_CONFIDENCE_THRESHOLD {
        (AIPriorityLane::Critical, critical_score.min(1.0))
    } else if critical_score > 0.3 || high_score >= HIGH_CONFIDENCE_THRESHOLD {
        // If there's any critical signal or strong high signal, use high
        (AIPriorityLane::High, high_score.max(critical_score).min(1.0))
    } else if high_score > 0.2 {
        (AIPriorityLane::High, high_score.min(1.0))
    } else {
        (AIPriorityLane::Standard, 1.0 - high_score - critical_score)
    };
    
    if reasons.is_empty() {
        reasons.push("standard transaction - no priority markers detected".to_string());
    }
    
    (lane, score, reasons)
}

/// Check if data contains a specific marker
fn contains_marker(data: &[u8], marker: &[u8]) -> bool {
    if marker.len() > data.len() {
        return false;
    }
    
    // Case-insensitive substring search
    let marker_lower: Vec<u8> = marker.iter().map(|&b| b.to_ascii_lowercase()).collect();
    let data_lower: Vec<u8> = data.iter().map(|&b| b.to_ascii_lowercase()).collect();
    
    data_lower.windows(marker.len())
        .any(|window| window == marker_lower.as_slice())
}

/// Check for AI signature pattern in transaction
/// AI signatures typically follow a specific format
fn has_ai_signature_pattern(data: &[u8]) -> bool {
    // Look for AI-specific prefixes or patterns
    // In production, this would verify actual cryptographic signatures
    let ai_patterns: &[&[u8]] = &[
        b"\xA1\x01", // AI signature prefix
        b"AI_SIG:",
        b"ai_signature",
        b"agent_id",
        b"model_hash",
    ];
    
    ai_patterns.iter().any(|pattern| contains_marker(data, pattern))
}

/// Check for compute-intensive function selectors
fn has_compute_intensive_selector(data: &[u8]) -> bool {
    // Common compute-intensive function selectors (first 4 bytes of keccak256)
    // These would be identified from actual contract ABIs
    let compute_selectors: &[&[u8]] = &[
        &[0xa9, 0x05, 0x9c, 0xbb], // transfer (ERC20)
        &[0x23, 0xb8, 0x72, 0xdd], // transferFrom (ERC20)
        &[0x09, 0x5e, 0xa7, 0xb3], // approve (ERC20)
        &[0x18, 0xcb, 0xaf, 0xe5], // swapExactTokensForTokens (DEX)
        &[0x7f, 0xf3, 0x6a, 0xb5], // flashLoan
        &[0xc2, 0x8e, 0x5b, 0x4f], // liquidate (lending)
    ];
    
    if data.len() < 4 {
        return false;
    }
    
    compute_selectors.iter().any(|selector| data.starts_with(selector))
}

/// Calculate transaction hash for classification caching
pub fn tx_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.update(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_be_bytes()
    );
    hasher.finalize().into()
}

/// Batch classify multiple transactions
pub fn classify_batch(transactions: &[&[u8]]) -> Vec<Classification> {
    transactions.iter().map(|tx| classify(tx)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_classify_standard() {
        let tx = b"simple transfer to alice";
        let result = classify(tx);
        assert_eq!(result.lane, AIPriorityLane::Standard);
    }
    
    #[test]
    fn test_classify_critical() {
        let tx = b"emergency_shutdown protocol";
        let result = classify(tx);
        assert_eq!(result.lane, AIPriorityLane::Critical);
        assert!(!result.reasons.is_empty());
    }
    
    #[test]
    fn test_classify_high_ai_agent() {
        let tx = b"ai_agent execute trade";
        let result = classify(tx);
        assert_eq!(result.lane, AIPriorityLane::High);
        assert!(result.estimated_compute_units > 0);
    }
    
    #[test]
    fn test_classify_high_mev() {
        let tx = b"mev arbitrage opportunity";
        let result = classify(tx);
        assert_eq!(result.lane, AIPriorityLane::High);
    }
    
    #[test]
    fn test_compute_units_by_lane() {
        let critical = Classification::new(AIPriorityLane::Critical, 1.0, vec![]);
        let high = Classification::new(AIPriorityLane::High, 1.0, vec![]);
        let standard = Classification::new(AIPriorityLane::Standard, 1.0, vec![]);
        
        assert!(critical.estimated_compute_units > high.estimated_compute_units);
        assert!(high.estimated_compute_units > standard.estimated_compute_units);
    }
    
    #[test]
    fn test_batch_classify() {
        let txs: Vec<&[u8]> = vec![
            b"simple transfer",
            b"emergency protocol",
            b"ai_agent swap",
        ];
        
        let results = classify_batch(&txs);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].lane, AIPriorityLane::Standard);
        assert_eq!(results[1].lane, AIPriorityLane::Critical);
        assert_eq!(results[2].lane, AIPriorityLane::High);
    }
}
