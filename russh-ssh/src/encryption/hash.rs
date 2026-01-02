//! BLAKE3 hashing utilities
//!
//! This module provides content hashing using BLAKE3 for:
//! - Content-addressed storage
//! - File integrity verification
//! - Cryptographic hashing operations

use std::io::{self, Read};
use std::path::Path;

/// The size of a BLAKE3 hash in bytes
pub const HASH_SIZE: usize = 32;

/// A content hash wrapper for BLAKE3 hashes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(blake3::Hash);

impl ContentHash {
    /// Create a ContentHash from raw bytes
    pub fn from_bytes(bytes: [u8; HASH_SIZE]) -> Self {
        ContentHash(blake3::Hash::from_bytes(bytes))
    }

    /// Get the raw bytes of the hash
    pub fn as_bytes(&self) -> &[u8; HASH_SIZE] {
        self.0.as_bytes()
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        self.0.to_hex().to_string()
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self, HashError> {
        if hex.len() != HASH_SIZE * 2 {
            return Err(HashError::InvalidHexLength {
                expected: HASH_SIZE * 2,
                actual: hex.len(),
            });
        }

        let mut bytes = [0u8; HASH_SIZE];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let hex_byte = std::str::from_utf8(chunk)
                .map_err(|_| HashError::InvalidHexCharacter)?;
            bytes[i] = u8::from_str_radix(hex_byte, 16)
                .map_err(|_| HashError::InvalidHexCharacter)?;
        }

        Ok(ContentHash(blake3::Hash::from_bytes(bytes)))
    }

    /// Get the inner blake3::Hash
    pub fn inner(&self) -> &blake3::Hash {
        &self.0
    }
}

impl From<blake3::Hash> for ContentHash {
    fn from(hash: blake3::Hash) -> Self {
        ContentHash(hash)
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl serde::Serialize for ContentHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for ContentHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        ContentHash::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}

/// Errors that can occur during hashing operations
#[derive(Debug, thiserror::Error)]
pub enum HashError {
    /// Invalid hex string length
    #[error("Invalid hex length: expected {expected}, got {actual}")]
    InvalidHexLength { expected: usize, actual: usize },

    /// Invalid hex character
    #[error("Invalid hex character in hash string")]
    InvalidHexCharacter,

    /// IO error during file hashing
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Compute BLAKE3 hash of data
pub fn hash_data(data: &[u8]) -> ContentHash {
    ContentHash(blake3::hash(data))
}

/// Compute BLAKE3 hash and return as hex string
pub fn hash_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Compute BLAKE3 hash of a file (synchronous - use hash_file_async for async contexts)
/// 
/// WARNING: This function uses blocking I/O. In async contexts, use `hash_file_async` instead
/// to avoid blocking the async runtime.
pub fn hash_file(path: &Path) -> Result<ContentHash, HashError> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(ContentHash(hasher.finalize()))
}

/// Compute BLAKE3 hash of a file asynchronously
/// 
/// This uses tokio's async file I/O to avoid blocking the runtime.
/// Preferred over `hash_file` in async contexts.
pub async fn hash_file_async(path: &Path) -> Result<ContentHash, HashError> {
    use tokio::io::AsyncReadExt;
    
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(ContentHash(hasher.finalize()))
}

/// Compute BLAKE3 hash of a file using spawn_blocking (for use in async contexts with sync file access)
/// 
/// This spawns the blocking file read on a dedicated thread pool, preventing
/// blocking of the async runtime while still using efficient synchronous I/O.
pub async fn hash_file_spawn_blocking(path: impl AsRef<Path> + Send + 'static) -> Result<ContentHash, HashError> {
    tokio::task::spawn_blocking(move || hash_file(path.as_ref()))
        .await
        .map_err(|e| HashError::Io(std::io::Error::other(e.to_string())))?
}

/// Compute BLAKE3 hash from a reader
pub fn hash_reader<R: Read>(reader: &mut R) -> Result<ContentHash, HashError> {
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(ContentHash(hasher.finalize()))
}

/// Incremental hasher for streaming data
pub struct IncrementalHasher {
    hasher: blake3::Hasher,
    bytes_processed: u64,
}

impl IncrementalHasher {
    /// Create a new incremental hasher
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
            bytes_processed: 0,
        }
    }

    /// Update the hasher with more data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
        self.bytes_processed += data.len() as u64;
    }

    /// Get the number of bytes processed so far
    pub fn bytes_processed(&self) -> u64 {
        self.bytes_processed
    }

    /// Finalize and return the hash
    pub fn finalize(self) -> ContentHash {
        ContentHash(self.hasher.finalize())
    }

    /// Finalize but keep the hasher for continued use
    pub fn finalize_reset(&mut self) -> ContentHash {
        let hash = ContentHash(self.hasher.finalize());
        self.hasher.reset();
        self.bytes_processed = 0;
        hash
    }
}

impl Default for IncrementalHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify that data matches an expected hash
pub fn verify_hash(data: &[u8], expected: &ContentHash) -> bool {
    hash_data(data) == *expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_determinism() {
        let data = b"hello world";
        let hash1 = hash_data(data);
        let hash2 = hash_data(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn different_data_different_hash() {
        let hash1 = hash_data(b"hello");
        let hash2 = hash_data(b"world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn hash_hex_roundtrip() {
        let data = b"test data for hashing";
        let hash = hash_data(data);
        let hex = hash.to_hex();
        let parsed = ContentHash::from_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn incremental_hasher_matches_direct() {
        let data = b"hello world this is a test";
        let direct_hash = hash_data(data);

        let mut incremental = IncrementalHasher::new();
        incremental.update(b"hello ");
        incremental.update(b"world ");
        incremental.update(b"this is a test");
        let incremental_hash = incremental.finalize();

        assert_eq!(direct_hash, incremental_hash);
    }

    #[test]
    fn verify_hash_works() {
        let data = b"verify me";
        let hash = hash_data(data);
        assert!(verify_hash(data, &hash));
        assert!(!verify_hash(b"wrong data", &hash));
    }

    #[test]
    fn content_hash_serialization() {
        let hash = hash_data(b"serialize me");
        let json = serde_json::to_string(&hash).unwrap();
        let deserialized: ContentHash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, deserialized);
    }

    #[test]
    fn empty_data_hash() {
        let hash1 = hash_data(b"");
        let hash2 = hash_data(b"");
        assert_eq!(hash1, hash2);
        // Empty data should still produce a valid hash
        assert_eq!(hash1.as_bytes().len(), HASH_SIZE);
    }
}
