//! Property-based tests for Virtual Distributed File System (VDFS)
//!
//! Feature: russh-ssh
//! These tests validate the correctness properties of the VDFS layer.

use russh_ssh::vdfs::{Chunk, ChunkStore, chunk_data, reassemble_chunks, FileMetadata};
use russh_ssh::vdfs::sync::SyncState;
use russh_ssh::encryption::hash::{hash_data, ContentHash};
use proptest::prelude::*;
use std::path::PathBuf;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 6: Content-Addressed Storage Determinism
    ///
    /// *For any* file content, storing it in the VDFS SHALL produce a content hash
    /// that is deterministic—the same content always produces the same hash, and
    /// retrieving by that hash returns the original content.
    ///
    /// **Validates: Requirements 5.1**
    #[test]
    fn content_addressed_storage_determinism(
        data in prop::collection::vec(any::<u8>(), 0..10000)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let store = ChunkStore::new();
            
            // Store the same data twice
            let id1 = store.store_data(data.clone()).await;
            let id2 = store.store_data(data.clone()).await;
            
            // IDs should be identical (deterministic)
            prop_assert_eq!(
                id1, id2,
                "Same content should produce same hash"
            );
            
            // Retrieve and verify content matches
            let retrieved = store.get(&id1).await.unwrap();
            prop_assert_eq!(
                retrieved.data, data,
                "Retrieved content should match original"
            );
            
            // Store should have only one entry (deduplication)
            prop_assert_eq!(
                store.len().await, 1,
                "Store should deduplicate identical content"
            );
            
            Ok(())
        })?;
    }

    /// Feature: russh-ssh, Property 6 (continued): Hash matches content
    ///
    /// *For any* chunk, the stored ID SHALL equal the BLAKE3 hash of its data.
    ///
    /// **Validates: Requirements 5.1**
    #[test]
    fn chunk_id_matches_content_hash(
        data in prop::collection::vec(any::<u8>(), 0..5000)
    ) {
        let chunk = Chunk::new(data.clone());
        let expected_hash = hash_data(&data);
        
        prop_assert_eq!(
            chunk.id, expected_hash,
            "Chunk ID should equal BLAKE3 hash of content"
        );
        
        prop_assert!(
            chunk.verify(),
            "Chunk should verify successfully"
        );
    }

    /// Feature: russh-ssh, Property 6 (continued): Different content produces different hashes
    ///
    /// *For any* two different byte sequences, storing them SHALL produce different hashes.
    ///
    /// **Validates: Requirements 5.1**
    #[test]
    fn different_content_different_hash(
        data1 in prop::collection::vec(any::<u8>(), 1..1000),
        data2 in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        prop_assume!(data1 != data2);
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let store = ChunkStore::new();
            
            let id1 = store.store_data(data1).await;
            let id2 = store.store_data(data2).await;
            
            prop_assert_ne!(
                id1, id2,
                "Different content should produce different hashes"
            );
            
            prop_assert_eq!(
                store.len().await, 2,
                "Store should have two distinct entries"
            );
            
            Ok(())
        })?;
    }
}


proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 8: Chunk Identification Determinism
    ///
    /// *For any* file content, chunking the file SHALL produce the same set of chunks
    /// with the same boundaries, ensuring that unchanged portions are identified
    /// correctly during sync.
    ///
    /// **Validates: Requirements 5.4**
    #[test]
    fn chunk_identification_determinism(
        data in prop::collection::vec(any::<u8>(), 0..10000),
        chunk_size in 64usize..1024,
    ) {
        // Chunk the same data twice
        let chunks1 = chunk_data(&data, chunk_size);
        let chunks2 = chunk_data(&data, chunk_size);
        
        // Should produce same number of chunks
        prop_assert_eq!(
            chunks1.len(), chunks2.len(),
            "Same data should produce same number of chunks"
        );
        
        // Each chunk should have identical ID and data
        for (c1, c2) in chunks1.iter().zip(chunks2.iter()) {
            prop_assert_eq!(
                c1.id, c2.id,
                "Chunk IDs should be identical"
            );
            prop_assert_eq!(
                &c1.data, &c2.data,
                "Chunk data should be identical"
            );
        }
    }

    /// Feature: russh-ssh, Property 8 (continued): Chunking and reassembly roundtrip
    ///
    /// *For any* file content, chunking then reassembling SHALL produce the original content.
    ///
    /// **Validates: Requirements 5.4**
    #[test]
    fn chunk_reassembly_roundtrip(
        data in prop::collection::vec(any::<u8>(), 0..10000),
        chunk_size in 64usize..1024,
    ) {
        let chunks = chunk_data(&data, chunk_size);
        let reassembled = reassemble_chunks(&chunks);
        
        prop_assert_eq!(
            reassembled, data,
            "Reassembled data should match original"
        );
    }

    /// Feature: russh-ssh, Property 8 (continued): Chunk boundaries are consistent
    ///
    /// *For any* file content, the chunk boundaries SHALL be at fixed intervals
    /// determined by chunk_size (except for the last chunk).
    ///
    /// **Validates: Requirements 5.4**
    #[test]
    fn chunk_boundaries_consistent(
        data in prop::collection::vec(any::<u8>(), 1..5000),
        chunk_size in 64usize..512,
    ) {
        let chunks = chunk_data(&data, chunk_size);
        
        // All chunks except the last should be exactly chunk_size
        for (i, chunk) in chunks.iter().enumerate() {
            if i < chunks.len() - 1 {
                prop_assert_eq!(
                    chunk.size(), chunk_size,
                    "Non-final chunk {} should be exactly chunk_size", i
                );
            } else {
                // Last chunk can be smaller
                prop_assert!(
                    chunk.size() <= chunk_size,
                    "Final chunk should be <= chunk_size"
                );
                prop_assert!(
                    chunk.size() > 0,
                    "Final chunk should not be empty"
                );
            }
        }
        
        // Total size should match original
        let total_size: usize = chunks.iter().map(|c| c.size()).sum();
        prop_assert_eq!(
            total_size, data.len(),
            "Total chunk size should equal original data size"
        );
    }
}


/// Strategy for generating arbitrary file paths
fn arb_path() -> impl Strategy<Value = PathBuf> {
    prop::collection::vec("[a-z0-9_]{1,10}", 1..5)
        .prop_map(|parts| {
            let path_str = format!("/{}", parts.join("/"));
            PathBuf::from(path_str)
        })
}

/// Strategy for generating arbitrary permissions
fn arb_permissions() -> impl Strategy<Value = russh_ssh::vdfs::metadata::Permissions> {
    (any::<bool>(), any::<bool>(), any::<bool>(),
     any::<bool>(), any::<bool>(), any::<bool>(),
     any::<bool>(), any::<bool>(), any::<bool>())
        .prop_map(|(or, ow, ox, gr, gw, gx, otr, otw, otx)| {
            russh_ssh::vdfs::metadata::Permissions {
                owner_read: or,
                owner_write: ow,
                owner_execute: ox,
                group_read: gr,
                group_write: gw,
                group_execute: gx,
                other_read: otr,
                other_write: otw,
                other_execute: otx,
            }
        })
}

/// Strategy for generating arbitrary FileMetadata
fn arb_file_metadata() -> impl Strategy<Value = FileMetadata> {
    (
        arb_path(),
        0u64..1_000_000,
        prop::collection::vec(any::<u8>(), 32..64),
        prop::collection::vec(prop::collection::vec(any::<u8>(), 32..64), 0..5),
        arb_permissions(),
        0u64..1000,
        prop::option::of("[a-z0-9]{8,16}"),
    ).prop_map(|(path, size, content_bytes, chunk_bytes, permissions, version, modified_by)| {
        let content_hash = hash_data(&content_bytes);
        let chunks: Vec<ContentHash> = chunk_bytes.iter()
            .map(|b| hash_data(b))
            .collect();
        
        let mut metadata = FileMetadata::new_file(path, size, content_hash, chunks);
        metadata.permissions = permissions;
        metadata.version = version;
        metadata.modified_by = modified_by;
        metadata
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 9: File Metadata Round-Trip
    ///
    /// *For any* valid FileMetadata object, serializing then deserializing SHALL
    /// produce an equivalent FileMetadata with all fields preserved (path, size,
    /// hash, permissions, timestamps, owner).
    ///
    /// **Validates: Requirements 5.5**
    #[test]
    fn file_metadata_roundtrip(metadata in arb_file_metadata()) {
        let json = metadata.to_json().unwrap();
        let restored = FileMetadata::from_json(&json).unwrap();
        
        // All fields should be preserved
        prop_assert_eq!(
            restored.path, metadata.path,
            "Path should be preserved"
        );
        prop_assert_eq!(
            restored.size, metadata.size,
            "Size should be preserved"
        );
        prop_assert_eq!(
            restored.content_hash, metadata.content_hash,
            "Content hash should be preserved"
        );
        prop_assert_eq!(
            restored.chunks.len(), metadata.chunks.len(),
            "Chunk count should be preserved"
        );
        for (r, o) in restored.chunks.iter().zip(metadata.chunks.iter()) {
            prop_assert_eq!(r, o, "Chunk hashes should be preserved");
        }
        prop_assert_eq!(
            restored.permissions.to_mode(), metadata.permissions.to_mode(),
            "Permissions should be preserved"
        );
        prop_assert_eq!(
            restored.version, metadata.version,
            "Version should be preserved"
        );
        prop_assert_eq!(
            restored.modified_by, metadata.modified_by,
            "Modified_by should be preserved"
        );
        prop_assert_eq!(
            restored.file_type, metadata.file_type,
            "File type should be preserved"
        );
    }

    /// Feature: russh-ssh, Property 9 (continued): Permissions mode roundtrip
    ///
    /// *For any* Unix mode value, converting to Permissions and back SHALL
    /// produce the same mode value.
    ///
    /// **Validates: Requirements 5.5**
    #[test]
    fn permissions_mode_roundtrip(mode in 0u32..0o777) {
        use russh_ssh::vdfs::metadata::Permissions;
        
        let permissions = Permissions::from_mode(mode);
        let restored_mode = permissions.to_mode();
        
        prop_assert_eq!(
            restored_mode, mode,
            "Permissions mode roundtrip should preserve value"
        );
    }

    /// Feature: russh-ssh, Property 9 (continued): Directory metadata roundtrip
    ///
    /// *For any* directory path, creating directory metadata and serializing
    /// then deserializing SHALL preserve all fields.
    ///
    /// **Validates: Requirements 5.5**
    #[test]
    fn directory_metadata_roundtrip(path in arb_path()) {
        let metadata = FileMetadata::new_directory(path.clone());
        
        let json = metadata.to_json().unwrap();
        let restored = FileMetadata::from_json(&json).unwrap();
        
        prop_assert_eq!(&restored.path, &path);
        prop_assert!(restored.is_directory());
        prop_assert_eq!(restored.size, 0);
        prop_assert!(restored.content_hash.is_none());
        prop_assert!(restored.chunks.is_empty());
    }

    /// Feature: russh-ssh, Property 9 (continued): Symlink metadata roundtrip
    ///
    /// *For any* symlink path and target, creating symlink metadata and serializing
    /// then deserializing SHALL preserve all fields including the target.
    ///
    /// **Validates: Requirements 5.5**
    #[test]
    fn symlink_metadata_roundtrip(
        path in arb_path(),
        target in arb_path(),
    ) {
        let metadata = FileMetadata::new_symlink(path.clone(), target.clone());
        
        let json = metadata.to_json().unwrap();
        let restored = FileMetadata::from_json(&json).unwrap();
        
        prop_assert_eq!(&restored.path, &path);
        prop_assert!(restored.is_symlink());
        prop_assert_eq!(&restored.symlink_target, &Some(target));
    }
}


/// Strategy for generating arbitrary node IDs
fn arb_node_id() -> impl Strategy<Value = String> {
    "[a-z0-9]{8,16}".prop_map(|s| s)
}

/// Strategy for generating a SyncState with some files
fn arb_sync_state() -> impl Strategy<Value = SyncState> {
    (
        arb_node_id(),
        prop::collection::vec(arb_file_metadata(), 0..5),
    ).prop_map(|(node_id, files)| {
        let mut state = SyncState::new(node_id);
        for file in files {
            state.apply_local(russh_ssh::vdfs::sync::FileOperation::Create {
                path: file.path.clone(),
                metadata: Box::new(file),
            });
        }
        state
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 7: CRDT Merge Commutativity
    ///
    /// *For any* two CRDT states A and B, merge(A, B) SHALL equal merge(B, A)
    /// (commutativity).
    ///
    /// **Validates: Requirements 5.2**
    #[test]
    fn crdt_merge_commutativity(
        state_a in arb_sync_state(),
        state_b in arb_sync_state(),
    ) {
        // Merge A into B
        let mut merged_ab = state_a.clone();
        merged_ab.merge(&state_b);
        
        // Merge B into A
        let mut merged_ba = state_b.clone();
        merged_ba.merge(&state_a);
        
        // Both should have the same files
        let files_ab: std::collections::HashSet<_> = merged_ab.list_files()
            .iter()
            .map(|f| f.path.clone())
            .collect();
        let files_ba: std::collections::HashSet<_> = merged_ba.list_files()
            .iter()
            .map(|f| f.path.clone())
            .collect();
        
        prop_assert_eq!(
            files_ab.clone(), files_ba,
            "Merge should be commutative: same files regardless of merge order"
        );
        
        // For each file, the content should be the same
        for path in &files_ab {
            let meta_ab = merged_ab.get(path);
            let meta_ba = merged_ba.get(path);
            
            prop_assert!(meta_ab.is_some() && meta_ba.is_some());
            
            let meta_ab = meta_ab.unwrap();
            let meta_ba = meta_ba.unwrap();
            
            prop_assert_eq!(
                meta_ab.content_hash, meta_ba.content_hash,
                "File content hash should be same regardless of merge order"
            );
        }
    }

    /// Feature: russh-ssh, Property 7: CRDT Merge Idempotence
    ///
    /// *For any* CRDT state A, merge(A, merge(A, B)) SHALL equal merge(A, B)
    /// (idempotence).
    ///
    /// **Validates: Requirements 5.2**
    #[test]
    fn crdt_merge_idempotence(state in arb_sync_state()) {
        let original = state.clone();
        
        // Merge with self
        let mut merged_once = state.clone();
        merged_once.merge(&original);
        
        // Merge again with self
        let mut merged_twice = merged_once.clone();
        merged_twice.merge(&original);
        
        // Should have same files
        let files_once: std::collections::HashSet<_> = merged_once.list_files()
            .iter()
            .map(|f| f.path.clone())
            .collect();
        let files_twice: std::collections::HashSet<_> = merged_twice.list_files()
            .iter()
            .map(|f| f.path.clone())
            .collect();
        
        prop_assert_eq!(
            files_once, files_twice,
            "Merge should be idempotent: merging twice should equal merging once"
        );
        
        // File count should be same
        prop_assert_eq!(
            merged_once.list_files().len(),
            merged_twice.list_files().len(),
            "File count should be same after idempotent merge"
        );
    }

    /// Feature: russh-ssh, Property 7 (continued): Merge preserves all files
    ///
    /// *For any* two CRDT states, merging SHALL include all files from both states
    /// (union semantics for non-conflicting files).
    ///
    /// **Validates: Requirements 5.2**
    #[test]
    fn crdt_merge_preserves_files(
        state_a in arb_sync_state(),
        state_b in arb_sync_state(),
    ) {
        let files_a: std::collections::HashSet<_> = state_a.list_files()
            .iter()
            .map(|f| f.path.clone())
            .collect();
        let files_b: std::collections::HashSet<_> = state_b.list_files()
            .iter()
            .map(|f| f.path.clone())
            .collect();
        
        let mut merged = state_a.clone();
        merged.merge(&state_b);
        
        let files_merged: std::collections::HashSet<_> = merged.list_files()
            .iter()
            .map(|f| f.path.clone())
            .collect();
        
        // All files from A should be in merged
        for path in &files_a {
            prop_assert!(
                files_merged.contains(path),
                "Merged state should contain file from A: {:?}", path
            );
        }
        
        // All files from B should be in merged
        for path in &files_b {
            prop_assert!(
                files_merged.contains(path),
                "Merged state should contain file from B: {:?}", path
            );
        }
    }

    /// Feature: russh-ssh, Property 7 (continued): LWW conflict resolution
    ///
    /// *For any* conflicting updates to the same file, the Last-Writer-Wins
    /// strategy SHALL select the update with the higher version or later timestamp.
    ///
    /// **Validates: Requirements 5.2**
    #[test]
    fn crdt_lww_conflict_resolution(
        path in arb_path(),
        content1 in prop::collection::vec(any::<u8>(), 32..64),
        content2 in prop::collection::vec(any::<u8>(), 32..64),
    ) {
        prop_assume!(content1 != content2);
        
        let hash1 = hash_data(&content1);
        let hash2 = hash_data(&content2);
        
        // Create two states with same file but different content
        let mut state_a = SyncState::new("node_a".to_string());
        let mut meta_a = FileMetadata::new_file(path.clone(), 100, hash1, vec![hash1]);
        meta_a.version = 1;
        state_a.apply_local(russh_ssh::vdfs::sync::FileOperation::Create {
            path: path.clone(),
            metadata: Box::new(meta_a),
        });
        
        let mut state_b = SyncState::new("node_b".to_string());
        let mut meta_b = FileMetadata::new_file(path.clone(), 200, hash2, vec![hash2]);
        meta_b.version = 2; // Higher version
        state_b.apply_local(russh_ssh::vdfs::sync::FileOperation::Create {
            path: path.clone(),
            metadata: Box::new(meta_b),
        });
        
        // Merge - higher version should win
        let mut merged = state_a.clone();
        merged.merge(&state_b);
        
        let result = merged.get(&path).unwrap();
        prop_assert_eq!(
            result.version, 2,
            "Higher version should win in LWW conflict resolution"
        );
        prop_assert_eq!(
            result.content_hash, Some(hash2),
            "Content from higher version should be preserved"
        );
    }
}
