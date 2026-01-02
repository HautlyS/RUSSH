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
pub mod metadata;
pub mod sync;
pub mod filesystem;

pub use chunk::{Chunk, ChunkStore, ChunkId, chunk_data, reassemble_chunks};
pub use metadata::FileMetadata;
pub use sync::{SyncState, SyncEngine};
pub use filesystem::VirtualFs;
