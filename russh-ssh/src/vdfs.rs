//! Virtual Distributed File System (VDFS)
//!
//! Content-addressed storage with CRDT-based synchronization.
//!
//! # Requirements Coverage
//! - Requirement 5.1: Content-addressed storage using BLAKE3
//! - Requirement 5.2: CRDT-based sync for conflict resolution
//! - Requirement 5.3: Virtual filesystem interface
//! - Requirement 5.4: Deterministic chunking
//! - Requirement 5.5: File metadata serialization

pub mod chunk;
pub mod filesystem;
pub mod metadata;
pub mod sync;

pub use chunk::{chunk_data, reassemble_chunks, Chunk, ChunkId, ChunkStore};
pub use filesystem::VirtualFs;
pub use metadata::FileMetadata;
pub use sync::{SyncEngine, SyncState};
