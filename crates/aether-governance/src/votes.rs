//! Voting mechanisms

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: [u8; 32],
    pub vote: i8, // -1 against, 0 abstain, 1 for
    pub weight: u64,
}
