//! AETHER Network - P2P networking layer
//!
//! Gossip, block propagation, and peer management.

#![warn(missing_docs)]

pub mod gossip;
pub mod propagation;
pub mod peer;

pub use gossip::*;
pub use propagation::*;
pub use peer::*;