//! File Metadata
//!
//! Stores metadata about files in the virtual filesystem.
//!
//! # Requirements Coverage
//! - Requirement 5.5: File metadata serialization

use crate::encryption::hash::ContentHash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File type in the virtual filesystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Symbolic link
    Symlink,
}

/// File permissions (Unix-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    /// Owner can read
    pub owner_read: bool,
    /// Owner can write
    pub owner_write: bool,
    /// Owner can execute
    pub owner_execute: bool,
    /// Group can read
    pub group_read: bool,
    /// Group can write
    pub group_write: bool,
    /// Group can execute
    pub group_execute: bool,
    /// Others can read
    pub other_read: bool,
    /// Others can write
    pub other_write: bool,
    /// Others can execute
    pub other_execute: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        // Default: rw-r--r-- (644)
        Self {
            owner_read: true,
            owner_write: true,
            owner_execute: false,
            group_read: true,
            group_write: false,
            group_execute: false,
            other_read: true,
            other_write: false,
            other_execute: false,
        }
    }
}

impl Permissions {
    /// Create permissions from Unix mode bits
    pub fn from_mode(mode: u32) -> Self {
        Self {
            owner_read: mode & 0o400 != 0,
            owner_write: mode & 0o200 != 0,
            owner_execute: mode & 0o100 != 0,
            group_read: mode & 0o040 != 0,
            group_write: mode & 0o020 != 0,
            group_execute: mode & 0o010 != 0,
            other_read: mode & 0o004 != 0,
            other_write: mode & 0o002 != 0,
            other_execute: mode & 0o001 != 0,
        }
    }

    /// Convert to Unix mode bits
    pub fn to_mode(&self) -> u32 {
        let mut mode = 0u32;
        if self.owner_read {
            mode |= 0o400;
        }
        if self.owner_write {
            mode |= 0o200;
        }
        if self.owner_execute {
            mode |= 0o100;
        }
        if self.group_read {
            mode |= 0o040;
        }
        if self.group_write {
            mode |= 0o020;
        }
        if self.group_execute {
            mode |= 0o010;
        }
        if self.other_read {
            mode |= 0o004;
        }
        if self.other_write {
            mode |= 0o002;
        }
        if self.other_execute {
            mode |= 0o001;
        }
        mode
    }
}

/// Metadata for a file in the virtual filesystem
///
/// # Requirements Coverage
/// - Requirement 5.5: File metadata serialization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Virtual path in the filesystem
    pub path: PathBuf,
    /// File type
    pub file_type: FileType,
    /// File size in bytes
    pub size: u64,
    /// Content hash (for files)
    pub content_hash: Option<ContentHash>,
    /// List of chunk IDs that make up the file
    pub chunks: Vec<ContentHash>,
    /// File permissions
    pub permissions: Permissions,
    /// Creation time
    pub created: DateTime<Utc>,
    /// Last modification time
    pub modified: DateTime<Utc>,
    /// Last access time
    pub accessed: DateTime<Utc>,
    /// Symlink target (for symlinks)
    pub symlink_target: Option<PathBuf>,
    /// Version number for conflict resolution
    pub version: u64,
    /// Node ID that last modified this file
    pub modified_by: Option<String>,
}

impl FileMetadata {
    /// Create metadata for a new file
    pub fn new_file(
        path: PathBuf,
        size: u64,
        content_hash: ContentHash,
        chunks: Vec<ContentHash>,
    ) -> Self {
        let now = Utc::now();
        Self {
            path,
            file_type: FileType::File,
            size,
            content_hash: Some(content_hash),
            chunks,
            permissions: Permissions::default(),
            created: now,
            modified: now,
            accessed: now,
            symlink_target: None,
            version: 1,
            modified_by: None,
        }
    }

    /// Create metadata for a new directory
    pub fn new_directory(path: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            path,
            file_type: FileType::Directory,
            size: 0,
            content_hash: None,
            chunks: Vec::new(),
            permissions: Permissions::from_mode(0o755),
            created: now,
            modified: now,
            accessed: now,
            symlink_target: None,
            version: 1,
            modified_by: None,
        }
    }

    /// Create metadata for a symlink
    pub fn new_symlink(path: PathBuf, target: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            path,
            file_type: FileType::Symlink,
            size: 0,
            content_hash: None,
            chunks: Vec::new(),
            permissions: Permissions::from_mode(0o777),
            created: now,
            modified: now,
            accessed: now,
            symlink_target: Some(target),
            version: 1,
            modified_by: None,
        }
    }

    /// Check if this is a file
    pub fn is_file(&self) -> bool {
        self.file_type == FileType::File
    }

    /// Check if this is a directory
    pub fn is_directory(&self) -> bool {
        self.file_type == FileType::Directory
    }

    /// Check if this is a symlink
    pub fn is_symlink(&self) -> bool {
        self.file_type == FileType::Symlink
    }

    /// Update modification time and increment version
    pub fn touch(&mut self) {
        self.modified = Utc::now();
        self.version += 1;
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissions_mode_roundtrip() {
        let modes = [0o644, 0o755, 0o777, 0o600, 0o400];
        for mode in modes {
            let perms = Permissions::from_mode(mode);
            assert_eq!(perms.to_mode(), mode);
        }
    }

    #[test]
    fn file_metadata_serialization_roundtrip() {
        use crate::encryption::hash::hash_data;

        let content_hash = hash_data(b"test content");
        let chunk_hash = hash_data(b"chunk data");

        let metadata = FileMetadata::new_file(
            PathBuf::from("/test/file.txt"),
            100,
            content_hash,
            vec![chunk_hash],
        );

        let json = metadata.to_json().unwrap();
        let restored = FileMetadata::from_json(&json).unwrap();

        assert_eq!(restored.path, metadata.path);
        assert_eq!(restored.size, metadata.size);
        assert_eq!(restored.content_hash, metadata.content_hash);
        assert_eq!(restored.chunks.len(), metadata.chunks.len());
    }

    #[test]
    fn directory_metadata() {
        let metadata = FileMetadata::new_directory(PathBuf::from("/test/dir"));

        assert!(metadata.is_directory());
        assert!(!metadata.is_file());
        assert_eq!(metadata.permissions.to_mode(), 0o755);
    }

    #[test]
    fn symlink_metadata() {
        let metadata =
            FileMetadata::new_symlink(PathBuf::from("/test/link"), PathBuf::from("/test/target"));

        assert!(metadata.is_symlink());
        assert_eq!(metadata.symlink_target, Some(PathBuf::from("/test/target")));
    }
}
