//! Treasury management

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treasury {
    pub aeth_balance: u64,
    pub flux_balance: u64,
}
