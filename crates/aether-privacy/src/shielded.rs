//! Shielded transaction types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldedTx {
    pub proof: super::zk::ZKProof,
    pub ciphertext: Vec<u8>,
}
