//! Property-based tests for encryption layer
//!
//! Feature: russh-ssh
//! These tests validate the correctness properties of the encryption layer.

use russh_ssh::encryption::hash::{hash_data, ContentHash, IncrementalHasher, verify_hash};
use russh_ssh::encryption::cipher::{encrypt, decrypt, EncryptionKey};
use russh_ssh::encryption::secure_channel::SecureChannelBuilder;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 4: BLAKE3 Hash Determinism
    ///
    /// *For any* byte sequence, computing the BLAKE3 hash multiple times
    /// SHALL always produce the same hash value.
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn blake3_hash_determinism(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let hash1 = hash_data(&data);
        let hash2 = hash_data(&data);
        let hash3 = hash_data(&data);

        prop_assert_eq!(hash1, hash2, "Hash should be deterministic (1 vs 2)");
        prop_assert_eq!(hash2, hash3, "Hash should be deterministic (2 vs 3)");
    }

    /// Feature: russh-ssh, Property 4 (continued): Different data produces different hashes
    ///
    /// *For any* two different byte sequences, the BLAKE3 hashes SHALL be different
    /// (with cryptographic probability).
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn different_data_different_hash(
        data1 in prop::collection::vec(any::<u8>(), 1..1000),
        data2 in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        // Only test when data is actually different
        prop_assume!(data1 != data2);

        let hash1 = hash_data(&data1);
        let hash2 = hash_data(&data2);

        prop_assert_ne!(
            hash1, hash2,
            "Different data should produce different hashes"
        );
    }

    /// Feature: russh-ssh, Property 4 (continued): Hash hex roundtrip
    ///
    /// *For any* byte sequence, converting the hash to hex and back
    /// SHALL produce the same hash.
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn hash_hex_roundtrip(data in prop::collection::vec(any::<u8>(), 0..5000)) {
        let hash = hash_data(&data);
        let hex = hash.to_hex();
        let parsed = ContentHash::from_hex(&hex).unwrap();

        prop_assert_eq!(hash, parsed, "Hash hex roundtrip should preserve value");
    }

    /// Feature: russh-ssh, Property 4 (continued): Incremental hashing matches direct
    ///
    /// *For any* byte sequence split into chunks, incremental hashing
    /// SHALL produce the same result as direct hashing.
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn incremental_hash_matches_direct(
        chunks in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 0..500),
            1..10
        )
    ) {
        // Combine all chunks
        let combined: Vec<u8> = chunks.iter().flatten().copied().collect();
        let direct_hash = hash_data(&combined);

        // Hash incrementally
        let mut incremental = IncrementalHasher::new();
        for chunk in &chunks {
            incremental.update(chunk);
        }
        let incremental_hash = incremental.finalize();

        prop_assert_eq!(
            direct_hash, incremental_hash,
            "Incremental hash should match direct hash"
        );
    }

    /// Feature: russh-ssh, Property 4 (continued): Verify hash correctness
    ///
    /// *For any* byte sequence, verify_hash SHALL return true for the correct hash
    /// and false for any modified data.
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn verify_hash_correctness(
        data in prop::collection::vec(any::<u8>(), 1..1000),
        modification_index in any::<usize>(),
    ) {
        let hash = hash_data(&data);

        // Verify correct data
        prop_assert!(
            verify_hash(&data, &hash),
            "verify_hash should return true for correct data"
        );

        // Modify data and verify it fails
        let mut modified = data.clone();
        let idx = modification_index % modified.len();
        modified[idx] = modified[idx].wrapping_add(1);

        prop_assert!(
            !verify_hash(&modified, &hash),
            "verify_hash should return false for modified data"
        );
    }

    /// Feature: russh-ssh, Property 4 (continued): Hash serialization roundtrip
    ///
    /// *For any* byte sequence, serializing and deserializing the hash
    /// SHALL produce the same hash.
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn hash_serialization_roundtrip(data in prop::collection::vec(any::<u8>(), 0..5000)) {
        let hash = hash_data(&data);
        let json = serde_json::to_string(&hash).unwrap();
        let deserialized: ContentHash = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(hash, deserialized, "Hash serialization roundtrip should preserve value");
    }

    /// Feature: russh-ssh, Property 3: Encryption Round-Trip
    ///
    /// *For any* valid plaintext data and encryption key, encrypting then
    /// decrypting the data SHALL produce the original plaintext exactly.
    ///
    /// **Validates: Requirements 4.1**
    #[test]
    fn encryption_roundtrip(plaintext in prop::collection::vec(any::<u8>(), 0..10000)) {
        let key = EncryptionKey::generate().unwrap();

        let encrypted = encrypt(&key, &plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        prop_assert_eq!(
            plaintext, decrypted,
            "Encryption roundtrip should preserve plaintext"
        );
    }

    /// Feature: russh-ssh, Property 3 (continued): Encryption produces different ciphertext
    ///
    /// *For any* plaintext, encrypting it twice SHALL produce different ciphertexts
    /// (due to random nonce generation).
    ///
    /// **Validates: Requirements 4.1**
    #[test]
    fn encryption_produces_different_ciphertext(
        plaintext in prop::collection::vec(any::<u8>(), 1..1000)
    ) {
        let key = EncryptionKey::generate().unwrap();

        let encrypted1 = encrypt(&key, &plaintext).unwrap();
        let encrypted2 = encrypt(&key, &plaintext).unwrap();

        // Nonces should be different
        prop_assert_ne!(
            encrypted1.nonce, encrypted2.nonce,
            "Different encryptions should use different nonces"
        );

        // Ciphertexts should be different (due to different nonces)
        prop_assert_ne!(
            &encrypted1.ciphertext, &encrypted2.ciphertext,
            "Different encryptions should produce different ciphertexts"
        );

        // But both should decrypt to the same plaintext
        let decrypted1 = decrypt(&key, &encrypted1).unwrap();
        let decrypted2 = decrypt(&key, &encrypted2).unwrap();

        prop_assert_eq!(&decrypted1, &decrypted2, "Both should decrypt to same plaintext");
        prop_assert_eq!(&plaintext, &decrypted1, "Decrypted should match original");
    }

    /// Feature: russh-ssh, Property 3 (continued): Secure channel encryption roundtrip
    ///
    /// *For any* plaintext and established secure channel, encrypting then
    /// decrypting through the channel SHALL produce the original plaintext.
    ///
    /// **Validates: Requirements 4.1**
    #[test]
    fn secure_channel_encryption_roundtrip(
        plaintext in prop::collection::vec(any::<u8>(), 0..5000)
    ) {
        // Establish secure channel
        let initiator_builder = SecureChannelBuilder::new().unwrap();
        let init_msg = initiator_builder.create_init_message();

        let responder_builder = SecureChannelBuilder::new().unwrap();
        let (responder_channel, response_msg) = responder_builder.process_init(init_msg).unwrap();

        let initiator_channel = initiator_builder.process_response(response_msg).unwrap();

        // Test initiator -> responder
        let encrypted = initiator_channel.encrypt(&plaintext).unwrap();
        let decrypted = responder_channel.decrypt(&encrypted).unwrap();

        prop_assert_eq!(
            &plaintext, &decrypted,
            "Secure channel encryption roundtrip should preserve plaintext (initiator -> responder)"
        );

        // Test responder -> initiator
        let encrypted2 = responder_channel.encrypt(&plaintext).unwrap();
        let decrypted2 = initiator_channel.decrypt(&encrypted2).unwrap();

        prop_assert_eq!(
            &plaintext, &decrypted2,
            "Secure channel encryption roundtrip should preserve plaintext (responder -> initiator)"
        );
    }

    /// Feature: russh-ssh, Property 3 (continued): Key derivation is deterministic
    ///
    /// *For any* password and salt, deriving a key SHALL always produce
    /// the same key bytes.
    ///
    /// **Validates: Requirements 4.1**
    #[test]
    fn key_derivation_deterministic(
        password in prop::collection::vec(any::<u8>(), 1..100),
        salt in prop::collection::vec(any::<u8>(), 16..50),
    ) {
        let key1 = EncryptionKey::from_password(&password, &salt);
        let key2 = EncryptionKey::from_password(&password, &salt);

        prop_assert_eq!(
            key1.as_bytes(), key2.as_bytes(),
            "Key derivation should be deterministic"
        );
    }

    /// Feature: russh-ssh, Property 3 (continued): Different passwords produce different keys
    ///
    /// *For any* two different passwords with the same salt, the derived keys
    /// SHALL be different.
    ///
    /// **Validates: Requirements 4.1**
    #[test]
    fn different_passwords_different_keys(
        password1 in prop::collection::vec(any::<u8>(), 1..100),
        password2 in prop::collection::vec(any::<u8>(), 1..100),
        salt in prop::collection::vec(any::<u8>(), 16..50),
    ) {
        prop_assume!(password1 != password2);

        let key1 = EncryptionKey::from_password(&password1, &salt);
        let key2 = EncryptionKey::from_password(&password2, &salt);

        prop_assert_ne!(
            key1.as_bytes(), key2.as_bytes(),
            "Different passwords should produce different keys"
        );
    }
}


/// Feature: russh-ssh, Property 5: Invalid Ciphertext Error Handling
///
/// Tests for invalid ciphertext handling - these are separate from the proptest
/// block because they test error conditions.
#[cfg(test)]
mod invalid_ciphertext_tests {
    use russh_ssh::encryption::cipher::{encrypt, decrypt, EncryptionKey, NONCE_SIZE};
    use russh_ssh::encryption::hash::hash_data;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: russh-ssh, Property 5: Invalid Ciphertext Error Handling
        ///
        /// *For any* malformed or tampered ciphertext, the decryption operation
        /// SHALL return an error rather than producing invalid plaintext.
        ///
        /// **Validates: Requirements 4.5**
        #[test]
        fn tampered_ciphertext_returns_error(
            plaintext in prop::collection::vec(any::<u8>(), 1..1000),
            tamper_index in any::<usize>(),
        ) {
            let key = EncryptionKey::generate().unwrap();
            let mut encrypted = encrypt(&key, &plaintext).unwrap();

            // Tamper with the ciphertext
            if !encrypted.ciphertext.is_empty() {
                let idx = tamper_index % encrypted.ciphertext.len();
                encrypted.ciphertext[idx] = encrypted.ciphertext[idx].wrapping_add(1);
            }

            // Decryption should fail
            let result = decrypt(&key, &encrypted);
            prop_assert!(
                result.is_err(),
                "Tampered ciphertext should fail to decrypt"
            );
        }

        /// Feature: russh-ssh, Property 5 (continued): Wrong key returns error
        ///
        /// *For any* ciphertext encrypted with one key, attempting to decrypt
        /// with a different key SHALL return an error.
        ///
        /// **Validates: Requirements 4.5**
        #[test]
        fn wrong_key_returns_error(
            plaintext in prop::collection::vec(any::<u8>(), 1..1000),
        ) {
            let key1 = EncryptionKey::generate().unwrap();
            let key2 = EncryptionKey::generate().unwrap();

            let encrypted = encrypt(&key1, &plaintext).unwrap();

            // Decryption with wrong key should fail
            let result = decrypt(&key2, &encrypted);
            prop_assert!(
                result.is_err(),
                "Decryption with wrong key should fail"
            );
        }

        /// Feature: russh-ssh, Property 5 (continued): Tampered nonce returns error
        ///
        /// *For any* ciphertext with a tampered nonce, decryption SHALL fail.
        ///
        /// **Validates: Requirements 4.5**
        #[test]
        fn tampered_nonce_returns_error(
            plaintext in prop::collection::vec(any::<u8>(), 1..1000),
            tamper_index in 0usize..NONCE_SIZE,
        ) {
            let key = EncryptionKey::generate().unwrap();
            let mut encrypted = encrypt(&key, &plaintext).unwrap();

            // Tamper with the nonce
            encrypted.nonce[tamper_index] = encrypted.nonce[tamper_index].wrapping_add(1);

            // Decryption should fail
            let result = decrypt(&key, &encrypted);
            prop_assert!(
                result.is_err(),
                "Tampered nonce should fail to decrypt"
            );
        }

        /// Feature: russh-ssh, Property 5 (continued): Tampered hash returns error
        ///
        /// *For any* ciphertext with a tampered plaintext hash, decryption SHALL fail
        /// (even if the ciphertext itself is valid).
        ///
        /// **Validates: Requirements 4.5**
        #[test]
        fn tampered_hash_returns_error(
            plaintext in prop::collection::vec(any::<u8>(), 1..1000),
        ) {
            let key = EncryptionKey::generate().unwrap();
            let mut encrypted = encrypt(&key, &plaintext).unwrap();

            // Tamper with the plaintext hash by computing hash of different data
            encrypted.plaintext_hash = hash_data(b"wrong data");

            // Decryption should fail due to hash mismatch
            let result = decrypt(&key, &encrypted);
            prop_assert!(
                result.is_err(),
                "Tampered plaintext hash should fail to decrypt"
            );
        }

        /// Feature: russh-ssh, Property 5 (continued): Truncated ciphertext returns error
        ///
        /// *For any* ciphertext that is truncated, decryption SHALL fail.
        ///
        /// **Validates: Requirements 4.5**
        #[test]
        fn truncated_ciphertext_returns_error(
            plaintext in prop::collection::vec(any::<u8>(), 10..1000),
            truncate_amount in 1usize..10,
        ) {
            let key = EncryptionKey::generate().unwrap();
            let mut encrypted = encrypt(&key, &plaintext).unwrap();

            // Truncate the ciphertext
            let truncate_to = encrypted.ciphertext.len().saturating_sub(truncate_amount);
            if truncate_to > 0 {
                encrypted.ciphertext.truncate(truncate_to);
            }

            // Decryption should fail
            let result = decrypt(&key, &encrypted);
            prop_assert!(
                result.is_err(),
                "Truncated ciphertext should fail to decrypt"
            );
        }

        /// Feature: russh-ssh, Property 5 (continued): Extended ciphertext returns error
        ///
        /// *For any* ciphertext with extra bytes appended, decryption SHALL fail.
        ///
        /// **Validates: Requirements 4.5**
        #[test]
        fn extended_ciphertext_returns_error(
            plaintext in prop::collection::vec(any::<u8>(), 1..1000),
            extra_bytes in prop::collection::vec(any::<u8>(), 1..10),
        ) {
            let key = EncryptionKey::generate().unwrap();
            let mut encrypted = encrypt(&key, &plaintext).unwrap();

            // Append extra bytes to ciphertext
            encrypted.ciphertext.extend(extra_bytes);

            // Decryption should fail
            let result = decrypt(&key, &encrypted);
            prop_assert!(
                result.is_err(),
                "Extended ciphertext should fail to decrypt"
            );
        }
    }
}
