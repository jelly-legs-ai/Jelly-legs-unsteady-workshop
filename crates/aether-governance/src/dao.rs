//! DAO governance

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: [u8; 32],
    pub title: String,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    Active,
    Passed,
    Failed,
}
