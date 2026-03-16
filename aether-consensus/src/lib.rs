//! AETHER Consensus - PoH + PoS consensus implementation
//!
//! This crate implements the hybrid Proof of History + Proof of Stake
//! consensus mechanism for AETHER.

#![warn(missing_docs)]

pub mod poh_pos;
pub mod validator;
pub mod stake;

pub use poh_pos::*;
pub use validator::*;
pub use stake::*;