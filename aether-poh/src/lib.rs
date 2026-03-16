//! AETHER PoH - Proof of History implementation
//!
//! Verifiable delay function for generating historical proofs.

#![warn(missing_docs)]

pub mod generator;
pub mod verifier;

pub use generator::*;
pub use verifier::*;