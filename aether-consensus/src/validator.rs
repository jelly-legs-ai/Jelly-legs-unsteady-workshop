//! Validator management

use aether_core::Address;

/// Validator registry
pub struct ValidatorRegistry {
    validators: Vec<Validator>,
}

/// Validator node info
#[derive(Clone, Debug)]
pub struct Validator {
    /// Validator address
    pub address: Address,
    /// Is active
    pub active: bool,
}

impl ValidatorRegistry {
    /// Create a new empty validator registry
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add a new validator to the registry
    pub fn add_validator(&mut self, address: Address) -> bool {
        // Check if validator already exists
        if self.validators.iter().any(|v| v.address == address) {
            return false;
        }
        
        self.validators.push(Validator {
            address,
            active: true,
        });
        true
    }

    /// Remove a validator from the registry
    pub fn remove_validator(&mut self, address: &Address) -> bool {
        let initial_len = self.validators.len();
        self.validators.retain(|v| v.address != *address);
        self.validators.len() != initial_len
    }

    /// Activate a validator
    pub fn activate_validator(&mut self, address: &Address) -> bool {
        if let Some(validator) = self.validators.iter_mut().find(|v| v.address == *address) {
            validator.active = true;
            return true;
        }
        false
    }

    /// Deactivate a validator
    pub fn deactivate_validator(&mut self, address: &Address) -> bool {
        if let Some(validator) = self.validators.iter_mut().find(|v| v.address == *address) {
            validator.active = false;
            return true;
        }
        false
    }

    /// Check if a validator is active
    pub fn is_active(&self, address: &Address) -> bool {
        self.validators
            .iter()
            .any(|v| v.address == *address && v.active)
    }

    /// Get all active validators
    pub fn get_active_validators(&self) -> Vec<&Validator> {
        self.validators.iter().filter(|v| v.active).collect()
    }

    /// Get validator count
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }

    /// Get active validator count
    pub fn active_validator_count(&self) -> usize {
        self.validators.iter().filter(|v| v.active).count()
    }
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}