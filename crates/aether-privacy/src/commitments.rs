//! Commitment schemes for privacy

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub root: [u8; 32],
    pub proof_path: Vec<[u8; 32]>,
}
