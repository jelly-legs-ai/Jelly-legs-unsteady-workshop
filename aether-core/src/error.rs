//! Error types for AETHER

/// Result type alias
pub type Result<T> = std::result::Result<T, AetherError>;

/// Main error type for AETHER
#[derive(Debug, thiserror::Error)]
pub enum AetherError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Crypto error: {0}")]
    Crypto(#[from] crate::crypto::CryptoError),
    
    #[error("Invalid block")]
    InvalidBlock,
    
    #[error("Invalid transaction")]
    InvalidTransaction,
    
    #[error("Consensus error: {0}")]
    Consensus(String),
    
    #[error("Network error: {0}")]
    Network(String),
}