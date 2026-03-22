//! AETHER Core - Core types and primitives
//!
//! This crate provides the fundamental types, primitives, and utilities
//! used throughout the AETHER blockchain.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod types;
pub mod crypto;
pub mod error;
pub mod reward;
pub mod anti_gaming;

pub use types::*;
pub use crypto::*;
pub use error::*;
pub use reward::*;
pub use anti_gaming::*;