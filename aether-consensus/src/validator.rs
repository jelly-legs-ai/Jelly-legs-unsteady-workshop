//! Validator management

use aether_core::Address;

/// Validator registry
pub struct ValidatorRegistry {
    validators: Vec<Validator>,
}

/// Validator node info
pub struct Validator {
    /// Validator address
    pub address: Address,
    /// Is active
    pub active: bool,
}