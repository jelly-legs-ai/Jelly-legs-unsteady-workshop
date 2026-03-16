//! PoH Generator

/// PoH generator
pub struct PohGenerator {
    /// Current hash
    pub hash: [u8; 32],
    /// Number of hashes generated
    pub num_hashes: u64,
}

impl PohGenerator {
    /// Create new generator
    pub fn new() -> Self {
        Self {
            hash: [0; 32],
            num_hashes: 0,
        }
    }
    
    /// Generate next hash
    pub fn tick(&mut self) {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.hash);
        self.hash = hasher.finalize().into();
        self.num_hashes += 1;
    }
}