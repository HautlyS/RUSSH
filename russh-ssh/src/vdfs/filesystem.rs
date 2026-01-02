//! Virtual Filesystem Interface
//!
//! Provides a high-level interface for file operations.
//!
//! # Requirements Coverage
//! - Requirement 5.3: Virtual filesystem interface

use super::chunk::{ChunkStore, chunk_data, reassemble_chunks};
use super::metadata::FileMetadata;
use super::sync::{SyncEngine, SyncStatus};
use crate::encryption::hash::hash_data;
use crate::error::VdfsError;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Virtual Distributed File System
///
/// Provides a content-addressed, CRDT-synchronized filesystem.
pub struct VirtualFs {
    /// Chunk storage
    chunks: Arc<ChunkStore>,
    /// Sync engine
    sync: Arc<RwLock<SyncEngine>>,
    /// Mount point (virtual root)
    mount_point: PathBuf,
}

impl VirtualFs {
    /// Create a new virtual filesystem
    pub fn new(node_id: String, mount_point: PathBuf) -> Self {
        Self {
            chunks: Arc::new(ChunkStore::new()),
            sync: Arc::new(RwLock::new(SyncEngine::new(node_id))),
            mount_point,
        }
    }

    /// Create with custom chunk size
    pub fn with_chunk_size(node_id: String, mount_point: PathBuf, chunk_size: usize) -> Self {
        Self {
            chunks: Arc::new(ChunkStore::with_chunk_size(chunk_size)),
            sync: Arc::new(RwLock::new(SyncEngine::new(node_id))),
            mount_point,
        }
    }

    /// Get the mount point
    pub fn mount_point(&self) -> &Path {
        &self.mount_point
    }

    /// Normalize a path relative to mount point
    fn normalize_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.mount_point.join(path)
        }
    }

    /// Write a file
    ///
    /// Chunks the data, stores chunks, and creates metadata.
    pub async fn write(&self, path: &Path, data: &[u8]) -> Result<FileMetadata, VdfsError> {
        let normalized = self.normalize_path(path);
        
        // Chunk the data
        let chunks = chunk_data(data, self.chunks.chunk_size());
        
        // Store chunks and collect IDs
        let mut chunk_ids = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let id = self.chunks.store(chunk).await;
            chunk_ids.push(id);
        }
        
        // Compute content hash
        let content_hash = hash_data(data);
        
        // Create metadata
        let metadata = FileMetadata::new_file(
            normalized.clone(),
            data.len() as u64,
            content_hash,
            chunk_ids,
        );
        
        // Update sync state
        {
            let mut sync = self.sync.write().await;
            sync.create_file(metadata.clone());
        }
        
        Ok(metadata)
    }

    /// Read a file
    ///
    /// Retrieves chunks and reassembles the file.
    pub async fn read(&self, path: &Path) -> Result<Vec<u8>, VdfsError> {
        let normalized = self.normalize_path(path);
        
        // Get metadata
        let metadata = {
            let sync = self.sync.read().await;
            sync.state().get(&normalized).cloned()
        }.ok_or_else(|| VdfsError::NotFound(normalized.clone()))?;
        
        if !metadata.is_file() {
            return Err(VdfsError::NotFound(normalized));
        }
        
        // Retrieve chunks
        let mut chunks = Vec::with_capacity(metadata.chunks.len());
        for chunk_id in &metadata.chunks {
            let chunk = self.chunks.get(chunk_id).await?;
            chunks.push(chunk);
        }
        
        // Reassemble
        let data = reassemble_chunks(&chunks);
        
        // Verify content hash
        let computed_hash = hash_data(&data);
        if let Some(expected_hash) = &metadata.content_hash {
            if computed_hash != *expected_hash {
                return Err(VdfsError::HashMismatch {
                    expected: expected_hash.to_hex(),
                    actual: computed_hash.to_hex(),
                });
            }
        }
        
        Ok(data)
    }

    /// Delete a file
    pub async fn delete(&self, path: &Path) -> Result<(), VdfsError> {
        let normalized = self.normalize_path(path);
        
        // Check if file exists
        let metadata = {
            let sync = self.sync.read().await;
            sync.state().get(&normalized).cloned()
        }.ok_or_else(|| VdfsError::NotFound(normalized.clone()))?;
        
        // Remove chunks (if not referenced elsewhere - simplified: always remove)
        for chunk_id in &metadata.chunks {
            self.chunks.remove(chunk_id).await;
        }
        
        // Update sync state
        {
            let mut sync = self.sync.write().await;
            sync.delete_file(normalized);
        }
        
        Ok(())
    }

    /// Create a directory
    pub async fn mkdir(&self, path: &Path) -> Result<FileMetadata, VdfsError> {
        let normalized = self.normalize_path(path);
        
        let metadata = FileMetadata::new_directory(normalized.clone());
        
        {
            let mut sync = self.sync.write().await;
            sync.create_file(metadata.clone());
        }
        
        Ok(metadata)
    }

    /// List directory contents
    pub async fn list(&self, path: &Path) -> Result<Vec<FileMetadata>, VdfsError> {
        let normalized = self.normalize_path(path);
        
        let sync = self.sync.read().await;
        let files: Vec<FileMetadata> = sync.state()
            .list_files()
            .into_iter()
            .filter(|f| {
                f.path.parent() == Some(&normalized) || 
                (normalized == self.mount_point && f.path.parent().is_none())
            })
            .cloned()
            .collect();
        
        Ok(files)
    }

    /// Get file metadata
    pub async fn stat(&self, path: &Path) -> Result<FileMetadata, VdfsError> {
        let normalized = self.normalize_path(path);
        
        let sync = self.sync.read().await;
        sync.state()
            .get(&normalized)
            .cloned()
            .ok_or_else(|| VdfsError::NotFound(normalized))
    }

    /// Check if a path exists
    pub async fn exists(&self, path: &Path) -> bool {
        let normalized = self.normalize_path(path);
        let sync = self.sync.read().await;
        sync.state().get(&normalized).is_some()
    }

    /// Get sync status for a file
    pub async fn sync_status(&self, path: &Path) -> SyncStatus {
        let normalized = self.normalize_path(path);
        let sync = self.sync.read().await;
        sync.state().get_status(&normalized)
    }

    /// Get the chunk store
    pub fn chunk_store(&self) -> &ChunkStore {
        &self.chunks
    }

    /// Get sync engine for advanced operations
    pub fn sync_engine(&self) -> &Arc<RwLock<SyncEngine>> {
        &self.sync
    }

    /// Get storage statistics
    pub async fn stats(&self) -> FsStats {
        let sync = self.sync.read().await;
        let files = sync.state().list_files();
        
        let file_count = files.iter().filter(|f| f.is_file()).count();
        let dir_count = files.iter().filter(|f| f.is_directory()).count();
        let total_size: u64 = files.iter().map(|f| f.size).sum();
        
        FsStats {
            file_count,
            dir_count,
            total_size,
            chunk_count: self.chunks.len().await,
            chunk_storage_size: self.chunks.total_size().await,
        }
    }
}

/// Filesystem statistics
#[derive(Debug, Clone)]
pub struct FsStats {
    /// Number of files
    pub file_count: usize,
    /// Number of directories
    pub dir_count: usize,
    /// Total logical size of all files
    pub total_size: u64,
    /// Number of unique chunks
    pub chunk_count: usize,
    /// Total chunk storage size (may be less than total_size due to deduplication)
    pub chunk_storage_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn write_and_read_file() {
        let fs = VirtualFs::new("test-node".to_string(), PathBuf::from("/vfs"));
        
        let data = b"Hello, Virtual Filesystem!";
        let metadata = fs.write(Path::new("test.txt"), data).await.unwrap();
        
        assert_eq!(metadata.size, data.len() as u64);
        assert!(metadata.is_file());
        
        let read_data = fs.read(Path::new("test.txt")).await.unwrap();
        assert_eq!(read_data, data);
    }

    #[tokio::test]
    async fn delete_file() {
        let fs = VirtualFs::new("test-node".to_string(), PathBuf::from("/vfs"));
        
        fs.write(Path::new("to_delete.txt"), b"delete me").await.unwrap();
        assert!(fs.exists(Path::new("to_delete.txt")).await);
        
        fs.delete(Path::new("to_delete.txt")).await.unwrap();
        assert!(!fs.exists(Path::new("to_delete.txt")).await);
    }

    #[tokio::test]
    async fn create_directory() {
        let fs = VirtualFs::new("test-node".to_string(), PathBuf::from("/vfs"));
        
        let metadata = fs.mkdir(Path::new("mydir")).await.unwrap();
        assert!(metadata.is_directory());
        assert!(fs.exists(Path::new("mydir")).await);
    }

    #[tokio::test]
    async fn file_stats() {
        let fs = VirtualFs::new("test-node".to_string(), PathBuf::from("/vfs"));
        
        fs.write(Path::new("file1.txt"), b"content1").await.unwrap();
        fs.write(Path::new("file2.txt"), b"content2").await.unwrap();
        fs.mkdir(Path::new("dir1")).await.unwrap();
        
        let stats = fs.stats().await;
        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.dir_count, 1);
    }
}
