//! Fork Choice Rule Implementation
//!
//! Implements the fork choice rule for selecting the best chain
//! based on stake weight and recency.

use crate::{ConsensusError, ConsensusResult};
use std::collections::HashMap;

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
        // Block hash must be unique
        if self.blocks.contains_key(&hash) {
            return Err(ConsensusError::ForkChoiceViolation(
                format!("Block {:?} already exists", hash)
            ));
        }

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

    /// Update stake weight for a block and propagate to all ancestors.
    /// In LMD GHOST, a vote for a block implicitly votes for all ancestors,
    /// so stake must propagate up the chain to the root.
    /// 
    /// Returns the number of blocks whose stake was updated, or an error if the block doesn't exist.
    pub fn update_stake(
        &mut self,
        hash: [u8; 32],
        additional_stake: u64,
    ) -> ConsensusResult<usize> {
        // Verify block exists first
        if !self.blocks.contains_key(&hash) {
            return Err(ConsensusError::ForkChoiceViolation(
                format!("Block {:?} not found for stake update", hash)
            ));
        }

        // First, collect the ancestor chain INCLUDING the target block to avoid borrow checker issues
        let mut ancestors = Vec::new();
        let mut current = hash;
        
        // FIX: Push the target block FIRST before traversing to parent
        // This ensures the voted block itself receives the stake (core LMD GHOST rule)
        ancestors.push(current);
        
        loop {
            // Don't include root in stake propagation (it has u64::MAX)
            if current == self.root {
                break;
            }
            
            if let Some(block) = self.blocks.get(&current) {
                current = block.parent_hash;
                // Only add non-root ancestors
                if current != self.root {
                    ancestors.push(current);
                }
            } else {
                break;
            }
        }
        
        // Now update stake for the target block AND all ancestors (excluding root)
        let mut updated_count = 0;
        for ancestor_hash in ancestors {
            if let Some(block) = self.blocks.get_mut(&ancestor_hash) {
                block.stake_weight = block.stake_weight.saturating_add(additional_stake);
                updated_count += 1;
            }
        }
        
        self.update_best_block();
        Ok(updated_count)
    }

    /// Update best block based on LMD GHOST fork choice rule
    fn update_best_block(&mut self) {
        let best = self.find_best_block_recursive(self.root);
        self.best_block = best;
    }

    /// Get the cumulative stake weight of the best block's subtree.
    pub fn get_best_weight(&self) -> u64 {
        self.compute_subtree_weight(self.best_block)
    }

    /// Recursively find best block using LMD GHOST (cumulative fork weight)
    fn find_best_block_recursive(
        &self,
        current: [u8; 32],
    ) -> [u8; 32] {
        // Verify block exists before proceeding
        if !self.blocks.contains_key(&current) {
            return current;
        }

        let children = match self.children.get(&current) {
            Some(c) if !c.is_empty() => c,
            _ => return current,
        };

        // Compute cumulative stake weight for each child subtree (LMD GHOST rule)
        // A child inherits all stake from its ancestors, so we sum the full fork weight
        let best_child = children.iter()
            .filter_map(|h| self.blocks.get(h))
            .map(|child| {
                let cumulative_weight = self.compute_subtree_weight(child.hash);
                (child, cumulative_weight)
            })
            .max_by(|(_, weight_a), (_, weight_b)| weight_a.cmp(weight_b))
            .map(|(child, _)| child);

        // Always descend into the heaviest child subtree if one exists
        // This ensures we follow the chain with maximum cumulative stake
        match best_child {
            Some(child) => self.find_best_block_recursive(child.hash),
            _ => current,
        }
    }

    /// Compute total cumulative stake weight for a subtree rooted at block hash.
    /// This sums the block's own weight plus all descendants, giving the total
    /// fork weight for LMD GHOST fork choice.
    fn compute_subtree_weight(&self, block_hash: [u8; 32]) -> u64 {
        let mut stack = vec![block_hash];
        let mut total = 0u64;

        while let Some(current) = stack.pop() {
            if let Some(block) = self.blocks.get(&current) {
                total += block.stake_weight;
                if let Some(children) = self.children.get(&current) {
                    for child_hash in children {
                        stack.push(*child_hash);
                    }
                }
            }
        }

        total
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

    #[test]
    fn test_lmd_ghost_cumulative_fork_weight() {
        // The core LMD GHOST invariant: fork choice must use cumulative fork weight,
        // not individual block weight. A chain of many low-stake blocks must beat
        // a single high-stake block on the competing fork.
        let root = create_hash(0);
        let mut fc = ForkChoice::new(root);

        // Fork A: single block with high individual stake (300)
        let fork_a_head = create_hash(1);
        fc.add_block(fork_a_head, root, 1, 300).unwrap();

        // Fork B: 3 blocks with lower individual stakes (total = 250)
        let fork_b_1 = create_hash(10);
        fc.add_block(fork_b_1, root, 1, 100).unwrap();

        let fork_b_2 = create_hash(11);
        fc.add_block(fork_b_2, fork_b_1, 2, 100).unwrap();

        let fork_b_3 = create_hash(12);
        fc.add_block(fork_b_3, fork_b_2, 3, 50).unwrap();

        // Cumulative weight Fork B = 250, Fork A head = 300
        // But Fork A is a single block with no descendants beyond root
        // Fork B has more total descendants
        // With correct LMD GHOST: Fork A weight = 300 (just that block)
        //                          Fork B weight = 100 + 100 + 50 = 250
        // Both children of root: root has implicit children tracked
        // compute_subtree_weight(root) = INF (root), children checked separately
        // The best block should be the one whose subtree has higher total weight
        let best = fc.get_best_block().unwrap();
        // Fork A head has 300, Fork B 3rd block has 250 cumulative
        assert_eq!(best.hash, fork_a_head);

        // Now add more stake to Fork B to flip the decision
        // Add another descendant to Fork B with stake 60
        let fork_b_4 = create_hash(13);
        fc.add_block(fork_b_4, fork_b_3, 4, 60).unwrap();
        // Fork B total = 310 > 300

        let best = fc.get_best_block().unwrap();
        assert_eq!(best.hash, fork_b_4);
    }

    #[test]
    fn test_stake_propagation_ancestors() {
        // LMD GHOST requires that stake updates propagate to all ancestors
        let root = create_hash(0);
        let mut fc = ForkChoice::new(root);

        // Build chain: root -> b1 -> b2 -> b3
        let b1 = create_hash(1);
        fc.add_block(b1, root, 1, 100).unwrap();

        let b2 = create_hash(2);
        fc.add_block(b2, b1, 2, 100).unwrap();

        let b3 = create_hash(3);
        fc.add_block(b3, b2, 3, 100).unwrap();

        // Initial weights: each block has 100 (root has u64::MAX)
        assert_eq!(fc.get_block(&b1).unwrap().stake_weight, 100);
        assert_eq!(fc.get_block(&b2).unwrap().stake_weight, 100);
        assert_eq!(fc.get_block(&b3).unwrap().stake_weight, 100);

        // Update stake on b3 - should propagate to b2, b1 (3 blocks total)
        let updated = fc.update_stake(b3, 50).unwrap();
        assert_eq!(updated, 3, "Should update 3 blocks (b1, b2, b3)");

        // All ancestors should now have +50 stake (root excluded due to u64::MAX)
        assert_eq!(fc.get_block(&b1).unwrap().stake_weight, 150);
        assert_eq!(fc.get_block(&b2).unwrap().stake_weight, 150);
        assert_eq!(fc.get_block(&b3).unwrap().stake_weight, 150);
    }

    #[test]
    fn test_update_stake_nonexistent_block() {
        // Verify that updating stake on non-existent block returns error
        let root = create_hash(0);
        let mut fc = ForkChoice::new(root);

        let fake_hash = create_hash(99);
        let result = fc.update_stake(fake_hash, 100);
        
        assert!(result.is_err(), "Should return error for non-existent block");
        
        match result {
            Err(ConsensusError::ForkChoiceViolation(msg)) => {
                assert!(msg.contains("not found"), "Error message should mention block not found");
            }
            _ => panic!("Should return ForkChoiceViolation error"),
        }
    }

    #[test]
    fn test_add_duplicate_block() {
        // Verify that adding a block with duplicate hash returns error
        let root = create_hash(0);
        let mut fc = ForkChoice::new(root);

        let b1 = create_hash(1);
        fc.add_block(b1, root, 1, 100).unwrap();

        // Try to add same block again
        let result = fc.add_block(b1, root, 2, 200);
        
        assert!(result.is_err(), "Should return error for duplicate block");
        
        match result {
            Err(ConsensusError::ForkChoiceViolation(msg)) => {
                assert!(msg.contains("already exists"), "Error message should mention already exists");
            }
            _ => panic!("Should return ForkChoiceViolation error"),
        }
    }
}
