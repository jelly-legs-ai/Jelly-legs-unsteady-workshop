//! Aether Validator Library
//!
//! This library exposes the validator's internal modules for integration testing
//! and potential reuse. The binary crate (`main.rs`) re-exports these same modules.

pub mod block_producer;
pub mod config;
pub mod executor;
pub mod genesis;
pub mod keypair;
pub mod network;
pub mod persistence;
pub mod rpc_client;
pub mod rpc_server;
pub mod state;
pub mod state_db;
pub mod sync;

// Re-export commonly used items for convenience
pub use block_producer::*;
pub use config::*;
pub use executor::*;
pub use genesis::*;
pub use keypair::*;
pub use network::*;
pub use persistence::*;
pub use rpc_client::*;
pub use rpc_server::*;
pub use state::*;
pub use state_db::*;
pub use sync::*;