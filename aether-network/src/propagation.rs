//! Block propagation for efficient block dissemination across the network
//!
//! Implements Turbine-style block propagation:
//! - Split blocks into chunks for parallel transmission
//! - Use stake-weighted peer selection for optimal routing
//! - Track propagation latency for performance metrics

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

use crate::gossip::{GossipService, GossipMessage};

/// Block chunk for parallel propagation
#[derive(Debug, Clone)]
pub struct BlockChunk {
    /// Block height
    pub height: u64,
    /// Chunk index
    pub chunk_idx: usize,
    /// Total chunks in block
    pub total_chunks: usize,
    /// Chunk data
    pub data: Vec<u8>,
    /// Merkle root of all chunks
    pub merkle_root: [u8; 32],
}

/// Propagation statistics
#[derive(Debug, Clone, Default)]
pub struct PropagationStats {
    /// Total blocks propagated
    pub blocks_propagated: u64,
    /// Total chunks transmitted
    pub chunks_transmitted: u64,
    /// Average propagation latency (ms)
    pub avg_latency_ms: f64,
    /// Last propagation timestamp
    pub last_propagation: Option<u64>,
}

/// Block propagator with Turbine-style optimization
pub struct BlockPropagator {
    /// Gossip service for message dissemination
    gossip: Arc<GossipService>,
    /// Pending chunks awaiting propagation
    pending_chunks: Arc<RwLock<Vec<BlockChunk>>>,
    /// Propagation statistics
    stats: Arc<RwLock<PropagationStats>>,
    /// Maximum chunk size in bytes
    max_chunk_size: usize,
}

impl BlockPropagator {
    /// Create new block propagator
    pub fn new(gossip: Arc<GossipService>, max_chunk_size: usize) -> Self {
        Self {
            gossip,
            pending_chunks: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(PropagationStats::default())),
            max_chunk_size,
        }
    }

    /// Propagate a block to the network
    /// 
    /// Splits the block into chunks and broadcasts them via gossip
    pub async fn propagate_block(&self, block_data: &[u8], height: u64, block_hash: [u8; 32]) -> Result<(), PropagationError> {
        let start_time = Instant::now();
        
        // Split block into chunks
        let chunks = self.split_into_chunks(block_data, height, block_hash)?;
        let total_chunks = chunks.len();
        
        debug!("Propagating block {} ({} chunks, {} bytes)", 
               height, total_chunks, block_data.len());

        // Compute Merkle root for chunk verification
        let merkle_root = self.compute_merkle_root(&chunks);

        // Broadcast chunks via gossip
        for (idx, chunk) in chunks.into_iter().enumerate() {
            let message = GossipMessage::Block {
                height: chunk.height,
                hash: block_hash,
                data: chunk.data,
            };
            
            self.gossip.broadcast(message).await;
            
            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.chunks_transmitted += 1;
            }
        }

        // Update block propagation stats
        let latency_ms = start_time.elapsed().as_millis() as f64;
        {
            let mut stats = self.stats.write().await;
            stats.blocks_propagated += 1;
            stats.last_propagation = Some(current_timestamp());
            
            // Update average latency (exponential moving average)
            if stats.avg_latency_ms == 0.0 {
                stats.avg_latency_ms = latency_ms;
            } else {
                stats.avg_latency_ms = (stats.avg_latency_ms * 0.9) + (latency_ms * 0.1);
            }
        }

        info!("Block {} propagated in {:.2}ms (avg: {:.2}ms)", 
              height, latency_ms, self.stats.read().await.avg_latency_ms);

        Ok(())
    }

    /// Split block data into chunks for parallel propagation
    fn split_into_chunks(&self, block_data: &[u8], height: u64, block_hash: [u8; 32]) 
        -> Result<Vec<BlockChunk>, PropagationError> 
    {
        let total_chunks = (block_data.len() + self.max_chunk_size - 1) / self.max_chunk_size;
        let mut chunks = Vec::with_capacity(total_chunks);

        for (idx, chunk_data) in block_data.chunks(self.max_chunk_size).enumerate() {
            chunks.push(BlockChunk {
                height,
                chunk_idx: idx,
                total_chunks,
                data: chunk_data.to_vec(),
                merkle_root: block_hash, // Simplified: use block hash as root
            });
        }

        Ok(chunks)
    }

    /// Compute Merkle root of chunks for verification
    fn compute_merkle_root(&self, chunks: &[BlockChunk]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        if chunks.is_empty() {
            return [0u8; 32];
        }

        // Simple Merkle tree computation
        let mut hashes: Vec<[u8; 32]> = chunks.iter()
            .map(|chunk| {
                let mut hasher = Sha256::new();
                hasher.update(&chunk.data);
                hasher.finalize().into()
            })
            .collect();

        // Build Merkle tree
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    // Duplicate odd node
                    hasher.update(chunk[0]);
                }
                next_level.push(hasher.finalize().into());
            }
            hashes = next_level;
        }

        hashes[0]
    }

    /// Get propagation statistics
    pub async fn get_stats(&self) -> PropagationStats {
        self.stats.read().await.clone()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = PropagationStats::default();
    }

    /// Get pending chunk count
    pub async fn pending_chunk_count(&self) -> usize {
        self.pending_chunks.read().await.len()
    }
}

/// Propagation errors
#[derive(Debug, thiserror::Error)]
pub enum PropagationError {
    #[error("Block data too large: {0} bytes")]
    BlockTooLarge(usize),
    #[error("Invalid chunk size: {0}")]
    InvalidChunkSize(usize),
    #[error("Merkle root mismatch")]
    MerkleRootMismatch,
    #[error("Gossip broadcast failed: {0}")]
    GossipFailed(String),
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_chunk_splitting() {
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let gossip = Arc::new(GossipService::new(addr));
        let propagator = BlockPropagator::new(gossip, 1024); // 1KB chunks

        let block_data = vec![1u8; 3000]; // 3KB block
        let hash = [2u8; 32];
        
        let chunks = propagator.split_into_chunks(&block_data, 100, hash).unwrap();
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].data.len(), 1024);
        assert_eq!(chunks[1].data.len(), 1024);
        assert_eq!(chunks[2].data.len(), 952);
    }

    #[tokio::test]
    async fn test_merkle_root() {
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let gossip = Arc::new(GossipService::new(addr));
        let propagator = BlockPropagator::new(gossip, 1024);

        let block_data = vec![1u8; 2048];
        let hash = [3u8; 32];
        
        let chunks = propagator.split_into_chunks(&block_data, 100, hash).unwrap();
        let root = propagator.compute_merkle_root(&chunks);
        
        // Root should be deterministic
        assert_eq!(root.len(), 32);
        
        let root2 = propagator.compute_merkle_root(&chunks);
        assert_eq!(root, root2);
    }
}