//! Fork Choice Rule Implementation
//!
//! Implements the fork choice rule for selecting the best chain
//! based on stake weight and recency.

use crate::{ConsensusError, ConsensusResult};
use std::collections::{HashMap, HashSet};

/// Block information for fork choice
#[derive(Debug, Clone)]
pub struct BlockInfo {
    /// Slot number
    pub slot: u64,
    /// Parent slot
    pub parent_slot: u64,
    /// Block hash
    pub hash: [u8; 32],
    /// Total stake voting for this block
    pub stake_weight: u64,
    /// Blockhash of parent
    pub parent_hash: [u8; 32],
    /// Depth in chain
    pub depth: u64,
}

/// Fork choice tree
#[derive(Debug)]
pub struct ForkChoice {
    /// All blocks by hash
    blocks: HashMap<[u8; 32], BlockInfo>,
    /// Children mapping (parent -> children)
    children: HashMap<[u8; 32], Vec<[u8; 32]>>,
    /// Root block hash
    root: [u8; 32],
    /// Current best block hash
    best_block: [u8; 32],
}

impl ForkChoice {
    /// Create new fork choice tree
    pub fn new(root_hash: [u8; 32]) -> Self {
        let root_block = BlockInfo {
            slot: 0,
            parent_slot: 0,
            hash: root_hash,
            stake_weight: u64::MAX, // Root has max weight
            parent_hash: root_hash,
            depth: 0,
        };

        let mut blocks = HashMap::new();
        blocks.insert(root_hash, root_block);

        Self {
            blocks,
            children: HashMap::new(),
            root: root_hash,
            best_block: root_hash,
        }
    }

    /// Add a block to the fork choice tree
    pub fn add_block(
        &mut self,
        hash: [u8; 32],
        parent_hash: [u8; 32],
        slot: u64,
        stake_weight: u64,
    ) -> ConsensusResult<()> {
        // Parent must exist
        let parent = self.blocks.get(&parent_hash)
            .ok_or_else(|| ConsensusError::ForkChoiceViolation(
                format!("Parent block {:?} not found", parent_hash)
            ))?;

        // Slot must be > parent slot
        if slot <= parent.slot {
            return Err(ConsensusError::ForkChoiceViolation(
                format!("Block slot {} must be > parent slot {}", slot, parent.slot)
            ));
        }

        let block = BlockInfo {
            slot,
            parent_slot: parent.slot,
            hash,
            stake_weight,
            parent_hash,
            depth: parent.depth + 1,
        };

        self.blocks.insert(hash, block);
        self.children.entry(parent_hash).or_default().push(hash);

        // Update best block if needed
        self.update_best_block();

        Ok(())
    }

    /// Update stake weight for a block
    pub fn update_stake(
        &mut self,
        hash: [u8; 32],
        additional_stake: u64,
    ) {
        if let Some(block) = self.blocks.get_mut(&hash) {
            block.stake_weight += additional_stake;
        }
        self.update_best_block();
    }

    /// Update best block based on fork choice rule
    fn update_best_block(&mut self,
    ) {
        let best = self.find_best_block_recursive(self.root);
        self.best_block = best;
    }

    /// Recursively find best block
    fn find_best_block_recursive(
        &self,
        current: [u8; 32],
    ) -> [u8; 32] {
        let current_block = match self.blocks.get(&current) {
            Some(b) => b,
            None => return current,
        };

        let children = match self.children.get(&current) {
            Some(c) if !c.is_empty() => c,
            _ => return current,
        };

        // Find child with highest stake weight, breaking ties by slot (most recent)
        let best_child = children.iter()
            .filter_map(|h| self.blocks.get(h))
            .max_by(|a, b| {
                a.stake_weight.cmp(&b.stake_weight)
                    .then_with(|| a.slot.cmp(&b.slot))
            });

        match best_child {
            Some(child) if child.stake_weight > current_block.stake_weight => {
                self.find_best_block_recursive(child.hash)
            }
            _ => current,
        }
    }

    /// Get current best block
    pub fn get_best_block(&self) -> Option<&BlockInfo> {
        self.blocks.get(&self.best_block)
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &[u8; 32]) -> Option<&BlockInfo> {
        self.blocks.get(hash)
    }

    /// Get chain from root to block (inclusive of root, inclusive of block)
    pub fn get_chain(&self, block_hash: [u8; 32]) -> Vec<&BlockInfo> {
        let mut chain = Vec::new();
        let mut current = block_hash;

        // Walk backwards from block to root, including both endpoints
        while let Some(block) = self.blocks.get(&current) {
            chain.push(block);
            if current == self.root {
                // Reached genesis root — include it and stop
                break;
            }
            current = block.parent_hash;
        }

        chain.reverse();
        chain
    }

    /// Prune blocks before finalized slot
    pub fn prune_before(
        &mut self,
        finalized_slot: u64,
    ) {
        // Find blocks to remove
        let to_remove: Vec<[u8; 32]> = self.blocks.values()
            .filter(|b| b.slot < finalized_slot)
            .filter(|b| b.hash != self.root)
            .map(|b| b.hash)
            .collect();

        for hash in &to_remove {
            self.blocks.remove(hash);
            self.children.remove(hash);
            
            // Remove from parent's children list
            for children in self.children.values_mut() {
                children.retain(|h| h != hash);
            }
        }
    }

    /// Get all tips (blocks with no children)
    pub fn get_tips(&self) -> Vec<&BlockInfo> {
        self.blocks.values()
            .filter(|b| !self.children.contains_key(&b.hash) || 
                       self.children[&b.hash].is_empty())
            .collect()
    }

    /// Get fork depth of a block
    pub fn get_fork_depth(&self, block_hash: [u8; 32]) -> u64 {
        let mut depth = 0;
        let mut current = block_hash;

        while let Some(block) = self.blocks.get(&current) {
            if current == self.root {
                break;
            }
            
            // Count siblings
            if let Some(siblings) = self.children.get(&block.parent_hash) {
                if siblings.len() > 1 {
                    depth += 1;
                }
            }
            
            current = block.parent_hash;
        }

        depth
    }
}

impl Default for ForkChoice {
    fn default() -> Self {
        Self::new([0u8; 32])
    }
}

/// Compare two chains using LMD GHOST (Latest Message Driven Greediest Heaviest Observed SubTree)
pub fn lmd_ghost_compare(
    a: &[BlockInfo],
    b: &[BlockInfo],
) -> std::cmp::Ordering {
    // Higher stake weight wins
    let stake_cmp = a.last().map(|b| b.stake_weight)
        .cmp(&b.last().map(|b| b.stake_weight));
    
    if stake_cmp != std::cmp::Ordering::Equal {
        return stake_cmp;
    }
    
    // Higher slot wins (more recent)
    let slot_cmp = a.last().map(|b| b.slot)
        .cmp(&b.last().map(|b| b.slot));
    
    if slot_cmp != std::cmp::Ordering::Equal {
        return slot_cmp;
    }
    
    // Higher depth wins (longer chain)
    a.len().cmp(&b.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_hash(n: u8) -> [u8; 32] {
        let mut hash = [0u8; 32];
        hash[0] = n;
        hash
    }

    #[test]
    fn test_fork_choice_basic() {
        let root = create_hash(0);
        let mut fc = ForkChoice::new(root);

        // Add blocks
        let b1 = create_hash(1);
        fc.add_block(b1, root, 1, 100).unwrap();

        let b2 = create_hash(2);
        fc.add_block(b2, b1, 2, 150).unwrap();

        let best = fc.get_best_block().unwrap();
        assert_eq!(best.hash, b2);
        assert_eq!(best.depth, 2);
    }

    #[test]
    fn test_fork_choice_heavier_fork() {
        let root = create_hash(0);
        let mut fc = ForkChoice::new(root);

        // Create two competing forks
        let b1 = create_hash(1);
        fc.add_block(b1, root, 1, 100).unwrap();

        let b1_alt = create_hash(10);
        fc.add_block(b1_alt, root, 1, 200).unwrap();

        let b2 = create_hash(2);
        fc.add_block(b2, b1, 2, 100).unwrap();

        let b2_alt = create_hash(20);
        fc.add_block(b2_alt, b1_alt, 2, 200).unwrap();

        // Should pick heavier fork
        let best = fc.get_best_block().unwrap();
        assert_eq!(best.hash, b2_alt);
    }

    #[test]
    fn test_get_chain() {
        let root = create_hash(0);
        let mut fc = ForkChoice::new(root);

        let b1 = create_hash(1);
        fc.add_block(b1, root, 1, 100).unwrap();

        let b2 = create_hash(2);
        fc.add_block(b2, b1, 2, 150).unwrap();

        let b3 = create_hash(3);
        fc.add_block(b3, b2, 3, 200).unwrap();

        let chain = fc.get_chain(b3);
        assert_eq!(chain.len(), 4); // root + 3 blocks
        assert_eq!(chain[0].hash, root);
        assert_eq!(chain[3].hash, b3);
    }

    #[test]
    fn test_prune() {
        let root = create_hash(0);
        let mut fc = ForkChoice::new(root);

        // Add blocks at slots 1-5
        let mut prev = root;
        for i in 1..=5 {
            let hash = create_hash(i as u8);
            fc.add_block(hash, prev, i, 100 * i as u64).unwrap();
            prev = hash;
        }

        // Prune blocks before slot 3
        fc.prune_before(3);

        // Only blocks at slot 3+ should remain (plus root)
        assert!(fc.get_block(&root).is_some());
        assert!(fc.get_block(&create_hash(1)).is_none());
        assert!(fc.get_block(&create_hash(2)).is_none());
        assert!(fc.get_block(&create_hash(3)).is_some());
    }
}
