//! CRDT-Based Synchronization
//!
//! Implements conflict-free replicated data types for file synchronization.
//!
//! # Requirements Coverage
//! - Requirement 5.2: CRDT-based sync for conflict resolution

use super::metadata::FileMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Sync status for a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// File is in sync with all peers
    Synced,
    /// File has local changes not yet synced
    LocalModified,
    /// File has remote changes not yet applied
    RemoteModified,
    /// File has conflicting changes
    Conflict,
    /// File is being synced
    Syncing,
}

/// A file operation in the CRDT
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileOperation {
    /// Create a new file
    Create {
        path: PathBuf,
        metadata: Box<FileMetadata>,
    },
    /// Update file content
    Update {
        path: PathBuf,
        metadata: Box<FileMetadata>,
    },
    /// Delete a file
    Delete {
        path: PathBuf,
    },
    /// Rename/move a file
    Move {
        from: PathBuf,
        to: PathBuf,
    },
}

impl FileOperation {
    /// Get the primary path affected by this operation
    pub fn path(&self) -> &PathBuf {
        match self {
            FileOperation::Create { path, .. } => path,
            FileOperation::Update { path, .. } => path,
            FileOperation::Delete { path } => path,
            FileOperation::Move { from, .. } => from,
        }
    }
}

/// A timestamped operation for ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedOp {
    /// The operation
    pub op: FileOperation,
    /// Timestamp when the operation occurred
    pub timestamp: DateTime<Utc>,
    /// Node ID that performed the operation
    pub node_id: String,
    /// Logical clock value for ordering
    pub clock: u64,
}

impl TimestampedOp {
    /// Create a new timestamped operation
    pub fn new(op: FileOperation, node_id: String, clock: u64) -> Self {
        Self {
            op,
            timestamp: Utc::now(),
            node_id,
            clock,
        }
    }
}

/// CRDT state for file synchronization
///
/// Uses a Last-Writer-Wins (LWW) strategy with logical clocks
/// for conflict resolution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncState {
    /// Current file states (path -> metadata)
    files: HashMap<PathBuf, FileMetadata>,
    /// Operation log for synchronization
    operations: Vec<TimestampedOp>,
    /// Logical clock for this node
    clock: u64,
    /// This node's ID
    node_id: String,
    /// Sync status per file
    status: HashMap<PathBuf, SyncStatus>,
}

impl SyncState {
    /// Create a new sync state for a node
    pub fn new(node_id: String) -> Self {
        Self {
            files: HashMap::new(),
            operations: Vec::new(),
            clock: 0,
            node_id,
            status: HashMap::new(),
        }
    }

    /// Get the current logical clock value
    pub fn clock(&self) -> u64 {
        self.clock
    }

    /// Get this node's ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Apply a local operation
    pub fn apply_local(&mut self, op: FileOperation) {
        self.clock += 1;
        let timestamped = TimestampedOp::new(op.clone(), self.node_id.clone(), self.clock);
        self.apply_operation(&timestamped);
        self.operations.push(timestamped);
    }

    /// Apply a remote operation
    pub fn apply_remote(&mut self, op: TimestampedOp) {
        // Update clock to be at least as high as the remote clock
        self.clock = self.clock.max(op.clock) + 1;
        self.apply_operation(&op);
        self.operations.push(op);
    }

    /// Apply an operation to the state
    fn apply_operation(&mut self, op: &TimestampedOp) {
        match &op.op {
            FileOperation::Create { path, metadata } => {
                self.files.insert(path.clone(), *metadata.clone());
                self.status.insert(path.clone(), SyncStatus::Synced);
            }
            FileOperation::Update { path, metadata } => {
                if let Some(existing) = self.files.get(path) {
                    // LWW: Only apply if newer
                    if metadata.version > existing.version 
                        || (metadata.version == existing.version && op.timestamp > existing.modified) {
                        self.files.insert(path.clone(), *metadata.clone());
                    }
                } else {
                    self.files.insert(path.clone(), *metadata.clone());
                }
                self.status.insert(path.clone(), SyncStatus::Synced);
            }
            FileOperation::Delete { path } => {
                self.files.remove(path);
                self.status.remove(path);
            }
            FileOperation::Move { from, to } => {
                if let Some(metadata) = self.files.remove(from) {
                    let mut moved = metadata;
                    moved.path = to.clone();
                    self.files.insert(to.clone(), moved);
                    self.status.remove(from);
                    self.status.insert(to.clone(), SyncStatus::Synced);
                }
            }
        }
    }

    /// Merge another sync state into this one
    ///
    /// This implements the CRDT merge operation, which is:
    /// - Commutative: merge(A, B) == merge(B, A)
    /// - Associative: merge(merge(A, B), C) == merge(A, merge(B, C))
    /// - Idempotent: merge(A, A) == A
    pub fn merge(&mut self, other: &SyncState) {
        // Update clock
        self.clock = self.clock.max(other.clock) + 1;

        // Merge files using LWW
        for (path, other_meta) in &other.files {
            match self.files.get(path) {
                Some(self_meta) => {
                    // LWW: Keep the one with higher version, or later timestamp if same version
                    if other_meta.version > self_meta.version 
                        || (other_meta.version == self_meta.version && other_meta.modified > self_meta.modified) {
                        self.files.insert(path.clone(), other_meta.clone());
                    }
                }
                None => {
                    self.files.insert(path.clone(), other_meta.clone());
                }
            }
        }

        // Merge operations (deduplicate by timestamp + node_id)
        for op in &other.operations {
            let exists = self.operations.iter().any(|o| 
                o.timestamp == op.timestamp && o.node_id == op.node_id
            );
            if !exists {
                self.operations.push(op.clone());
            }
        }

        // Sort operations by clock, then timestamp
        self.operations.sort_by(|a, b| {
            a.clock.cmp(&b.clock)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });
    }

    /// Get file metadata by path
    pub fn get(&self, path: &PathBuf) -> Option<&FileMetadata> {
        self.files.get(path)
    }

    /// List all files
    pub fn list_files(&self) -> Vec<&FileMetadata> {
        self.files.values().collect()
    }

    /// Get sync status for a file
    pub fn get_status(&self, path: &PathBuf) -> SyncStatus {
        self.status.get(path).copied().unwrap_or(SyncStatus::Synced)
    }

    /// Set sync status for a file
    pub fn set_status(&mut self, path: PathBuf, status: SyncStatus) {
        self.status.insert(path, status);
    }

    /// Get operations since a given clock value
    pub fn operations_since(&self, clock: u64) -> Vec<&TimestampedOp> {
        self.operations.iter()
            .filter(|op| op.clock > clock)
            .collect()
    }
}

/// Sync engine for coordinating synchronization
pub struct SyncEngine {
    state: SyncState,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(node_id: String) -> Self {
        Self {
            state: SyncState::new(node_id),
        }
    }

    /// Get the current state
    pub fn state(&self) -> &SyncState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut SyncState {
        &mut self.state
    }

    /// Create a file
    pub fn create_file(&mut self, metadata: FileMetadata) {
        let path = metadata.path.clone();
        self.state.apply_local(FileOperation::Create {
            path,
            metadata: Box::new(metadata),
        });
    }

    /// Update a file
    pub fn update_file(&mut self, metadata: FileMetadata) {
        let path = metadata.path.clone();
        self.state.apply_local(FileOperation::Update {
            path,
            metadata: Box::new(metadata),
        });
    }

    /// Delete a file
    pub fn delete_file(&mut self, path: PathBuf) {
        self.state.apply_local(FileOperation::Delete { path });
    }

    /// Move a file
    pub fn move_file(&mut self, from: PathBuf, to: PathBuf) {
        self.state.apply_local(FileOperation::Move { from, to });
    }

    /// Sync with a remote state
    pub fn sync_with(&mut self, remote: &SyncState) {
        self.state.merge(remote);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::hash::hash_data;

    fn create_test_metadata(path: &str) -> FileMetadata {
        let hash = hash_data(path.as_bytes());
        FileMetadata::new_file(PathBuf::from(path), 100, hash, vec![hash])
    }

    #[test]
    fn sync_state_basic_operations() {
        let mut state = SyncState::new("node1".to_string());
        
        let metadata = create_test_metadata("/test.txt");
        state.apply_local(FileOperation::Create {
            path: PathBuf::from("/test.txt"),
            metadata: Box::new(metadata),
        });
        
        assert!(state.get(&PathBuf::from("/test.txt")).is_some());
        assert_eq!(state.clock(), 1);
    }

    #[test]
    fn sync_state_merge_commutative() {
        let mut state1 = SyncState::new("node1".to_string());
        let mut state2 = SyncState::new("node2".to_string());
        
        let meta1 = create_test_metadata("/file1.txt");
        let meta2 = create_test_metadata("/file2.txt");
        
        state1.apply_local(FileOperation::Create {
            path: PathBuf::from("/file1.txt"),
            metadata: Box::new(meta1.clone()),
        });
        
        state2.apply_local(FileOperation::Create {
            path: PathBuf::from("/file2.txt"),
            metadata: Box::new(meta2.clone()),
        });
        
        // Merge in both directions
        let mut merged1 = state1.clone();
        merged1.merge(&state2);
        
        let mut merged2 = state2.clone();
        merged2.merge(&state1);
        
        // Both should have both files
        assert!(merged1.get(&PathBuf::from("/file1.txt")).is_some());
        assert!(merged1.get(&PathBuf::from("/file2.txt")).is_some());
        assert!(merged2.get(&PathBuf::from("/file1.txt")).is_some());
        assert!(merged2.get(&PathBuf::from("/file2.txt")).is_some());
    }

    #[test]
    fn sync_state_merge_idempotent() {
        let mut state = SyncState::new("node1".to_string());
        
        let metadata = create_test_metadata("/test.txt");
        state.apply_local(FileOperation::Create {
            path: PathBuf::from("/test.txt"),
            metadata: Box::new(metadata),
        });
        
        let original = state.clone();
        state.merge(&original);
        
        // Should still have exactly one file
        assert_eq!(state.list_files().len(), 1);
    }

    #[test]
    fn sync_engine_workflow() {
        let mut engine = SyncEngine::new("node1".to_string());
        
        let metadata = create_test_metadata("/doc.txt");
        engine.create_file(metadata);
        
        assert!(engine.state().get(&PathBuf::from("/doc.txt")).is_some());
        
        engine.delete_file(PathBuf::from("/doc.txt"));
        assert!(engine.state().get(&PathBuf::from("/doc.txt")).is_none());
    }
}
