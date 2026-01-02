//! Content-Addressed Chunk Storage
//!
//! Stores file data as BLAKE3-addressed chunks for deduplication
//! and efficient synchronization.
//!
//! # Requirements Coverage
//! - Requirement 5.1: Content-addressed storage using BLAKE3
//! - Requirement 5.4: Deterministic chunking

use crate::encryption::hash::{ContentHash, hash_data};
use crate::error::VdfsError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default chunk size (64 KB)
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Chunk identifier (BLAKE3 hash)
pub type ChunkId = ContentHash;

/// A content-addressed chunk of data
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The chunk's content hash (also serves as its ID)
    pub id: ChunkId,
    /// The raw data
    pub data: Vec<u8>,
}

impl Chunk {
    /// Create a new chunk from data
    ///
    /// The chunk ID is computed as the BLAKE3 hash of the data.
    pub fn new(data: Vec<u8>) -> Self {
        let id = hash_data(&data);
        Self { id, data }
    }

    /// Verify the chunk's integrity
    ///
    /// Returns true if the data matches the stored hash.
    pub fn verify(&self) -> bool {
        let computed = hash_data(&self.data);
        computed == self.id
    }

    /// Get the chunk size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// In-memory chunk store
///
/// Provides content-addressed storage for chunks with deduplication.
#[derive(Debug, Default)]
pub struct ChunkStore {
    chunks: Arc<RwLock<HashMap<ChunkId, Chunk>>>,
    chunk_size: usize,
}

impl ChunkStore {
    /// Create a new chunk store with default chunk size
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }

    /// Create a chunk store with custom chunk size
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
            chunk_size,
        }
    }

    /// Get the configured chunk size
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Store a chunk
    ///
    /// Returns the chunk ID. If a chunk with the same content already exists,
    /// it won't be duplicated (content-addressed deduplication).
    pub async fn store(&self, chunk: Chunk) -> ChunkId {
        let id = chunk.id;
        let mut chunks = self.chunks.write().await;
        chunks.entry(id).or_insert(chunk);
        id
    }

    /// Store raw data as a chunk
    ///
    /// Creates a chunk from the data and stores it.
    pub async fn store_data(&self, data: Vec<u8>) -> ChunkId {
        let chunk = Chunk::new(data);
        self.store(chunk).await
    }

    /// Retrieve a chunk by ID
    pub async fn get(&self, id: &ChunkId) -> Result<Chunk, VdfsError> {
        let chunks = self.chunks.read().await;
        chunks.get(id)
            .cloned()
            .ok_or_else(|| VdfsError::ChunkNotFound(id.to_hex()))
    }

    /// Check if a chunk exists
    pub async fn contains(&self, id: &ChunkId) -> bool {
        let chunks = self.chunks.read().await;
        chunks.contains_key(id)
    }

    /// Remove a chunk
    pub async fn remove(&self, id: &ChunkId) -> Option<Chunk> {
        let mut chunks = self.chunks.write().await;
        chunks.remove(id)
    }

    /// Get the number of stored chunks
    pub async fn len(&self) -> usize {
        let chunks = self.chunks.read().await;
        chunks.len()
    }

    /// Check if the store is empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// List all chunk IDs
    pub async fn list_ids(&self) -> Vec<ChunkId> {
        let chunks = self.chunks.read().await;
        chunks.keys().cloned().collect()
    }

    /// Get total storage size in bytes
    pub async fn total_size(&self) -> usize {
        let chunks = self.chunks.read().await;
        chunks.values().map(|c| c.size()).sum()
    }
    
    /// Garbage collect unreferenced chunks
    /// 
    /// Removes all chunks that are not in the `referenced_ids` set.
    /// Returns the number of chunks removed and bytes freed.
    /// 
    /// # Memory Safety
    /// This method should be called periodically to prevent unbounded memory growth.
    pub async fn garbage_collect(&self, referenced_ids: &std::collections::HashSet<ChunkId>) -> (usize, usize) {
        let mut chunks = self.chunks.write().await;
        let mut removed_count = 0;
        let mut freed_bytes = 0;
        
        chunks.retain(|id, chunk| {
            if referenced_ids.contains(id) {
                true
            } else {
                removed_count += 1;
                freed_bytes += chunk.size();
                false
            }
        });
        
        (removed_count, freed_bytes)
    }
    
    /// Clear all chunks from the store
    /// 
    /// Returns the number of chunks removed and bytes freed.
    pub async fn clear(&self) -> (usize, usize) {
        let mut chunks = self.chunks.write().await;
        let count = chunks.len();
        let bytes: usize = chunks.values().map(|c| c.size()).sum();
        chunks.clear();
        (count, bytes)
    }
}

/// Split data into chunks using fixed-size chunking
///
/// # Requirements Coverage
/// - Requirement 5.4: Deterministic chunking
pub fn chunk_data(data: &[u8], chunk_size: usize) -> Vec<Chunk> {
    data.chunks(chunk_size)
        .map(|slice| Chunk::new(slice.to_vec()))
        .collect()
}

/// Reassemble chunks into original data
pub fn reassemble_chunks(chunks: &[Chunk]) -> Vec<u8> {
    chunks.iter()
        .flat_map(|c| c.data.iter().cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_creation_and_verification() {
        let data = b"Hello, World!".to_vec();
        let chunk = Chunk::new(data.clone());
        
        assert!(chunk.verify());
        assert_eq!(chunk.data, data);
        assert_eq!(chunk.size(), data.len());
    }

    #[test]
    fn chunk_deterministic_id() {
        let data = b"Same data".to_vec();
        let chunk1 = Chunk::new(data.clone());
        let chunk2 = Chunk::new(data);
        
        assert_eq!(chunk1.id, chunk2.id);
    }

    #[tokio::test]
    async fn chunk_store_basic_operations() {
        let store = ChunkStore::new();
        
        let data = b"Test data".to_vec();
        let id = store.store_data(data.clone()).await;
        
        assert!(store.contains(&id).await);
        
        let retrieved = store.get(&id).await.unwrap();
        assert_eq!(retrieved.data, data);
    }

    #[tokio::test]
    async fn chunk_store_deduplication() {
        let store = ChunkStore::new();
        
        let data = b"Duplicate data".to_vec();
        let id1 = store.store_data(data.clone()).await;
        let id2 = store.store_data(data).await;
        
        assert_eq!(id1, id2);
        assert_eq!(store.len().await, 1);
    }

    #[test]
    fn chunking_and_reassembly() {
        let data = b"This is some test data that will be chunked".to_vec();
        let chunks = chunk_data(&data, 10);
        
        assert_eq!(chunks.len(), 5); // 44 bytes / 10 = 5 chunks
        
        let reassembled = reassemble_chunks(&chunks);
        assert_eq!(reassembled, data);
    }
    
    #[tokio::test]
    async fn chunk_store_garbage_collection() {
        let store = ChunkStore::new();
        
        // Store some chunks
        let id1 = store.store_data(b"chunk 1".to_vec()).await;
        let id2 = store.store_data(b"chunk 2".to_vec()).await;
        let id3 = store.store_data(b"chunk 3".to_vec()).await;
        
        assert_eq!(store.len().await, 3);
        
        // Only keep id1 and id3
        let mut referenced = std::collections::HashSet::new();
        referenced.insert(id1.clone());
        referenced.insert(id3.clone());
        
        let (removed, freed) = store.garbage_collect(&referenced).await;
        
        assert_eq!(removed, 1);
        assert!(freed > 0);
        assert_eq!(store.len().await, 2);
        assert!(store.contains(&id1).await);
        assert!(!store.contains(&id2).await);
        assert!(store.contains(&id3).await);
    }
    
    #[tokio::test]
    async fn chunk_store_clear() {
        let store = ChunkStore::new();
        
        store.store_data(b"chunk 1".to_vec()).await;
        store.store_data(b"chunk 2".to_vec()).await;
        
        assert_eq!(store.len().await, 2);
        
        let (count, bytes) = store.clear().await;
        
        assert_eq!(count, 2);
        assert!(bytes > 0);
        assert!(store.is_empty().await);
    }
}
