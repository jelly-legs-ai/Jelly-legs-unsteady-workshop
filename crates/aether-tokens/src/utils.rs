//! Utilities for the aether-tokens crate

pub type Timestamp = u64;

/// Get current timestamp (placeholder)
pub fn now() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
