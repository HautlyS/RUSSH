//! Security-focused property-based tests
//!
//! Feature: russh-ssh
//! These tests validate security properties of the encryption and authentication layers.

use russh_ssh::encryption::cipher::{encrypt, decrypt, EncryptionKey};
use russh_ssh::encryption::hash::hash_data;
use russh_ssh::encryption::secure_channel::SecureChannelBuilder;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property: Key Derivation Security
    ///
    /// *For any* password and salt combination, the derived key SHALL be
    /// deterministic and different passwords SHALL produce different keys.
    ///
    /// **Validates: Requirements 4.1 - Secure key derivation**
    #[test]
    fn key_derivation_is_deterministic(
        password in prop::collection::vec(any::<u8>(), 1..100),
        salt in prop::collection::vec(any::<u8>(), 16..64),
    ) {
        let key1 = EncryptionKey::from_password(&password, &salt);
        let key2 = EncryptionKey::from_password(&password, &salt);
        
        prop_assert_eq!(
            key1.as_bytes(), key2.as_bytes(),
            "Key derivation must be deterministic"
        );
    }

    /// Feature: russh-ssh, Property: Different Passwords Different Keys
    ///
    /// *For any* two different passwords with the same salt, the derived keys
    /// SHALL be different (with cryptographic probability).
    ///
    /// **Validates: Requirements 4.1 - Key uniqueness**
    #[test]
    fn different_passwords_produce_different_keys(
        password1 in prop::collection::vec(any::<u8>(), 1..100),
        password2 in prop::collection::vec(any::<u8>(), 1..100),
        salt in prop::collection::vec(any::<u8>(), 16..64),
    ) {
        prop_assume!(password1 != password2);
        
        let key1 = EncryptionKey::from_password(&password1, &salt);
        let key2 = EncryptionKey::from_password(&password2, &salt);
        
        prop_assert_ne!(
            key1.as_bytes(), key2.as_bytes(),
            "Different passwords must produce different keys"
        );
    }

    /// Feature: russh-ssh, Property: Different Salts Different Keys
    ///
    /// *For any* password with two different salts, the derived keys
    /// SHALL be different.
    ///
    /// **Validates: Requirements 4.1 - Salt effectiveness**
    #[test]
    fn different_salts_produce_different_keys(
        password in prop::collection::vec(any::<u8>(), 1..100),
        salt1 in prop::collection::vec(any::<u8>(), 16..64),
        salt2 in prop::collection::vec(any::<u8>(), 16..64),
    ) {
        prop_assume!(salt1 != salt2);
        
        let key1 = EncryptionKey::from_password(&password, &salt1);
        let key2 = EncryptionKey::from_password(&password, &salt2);
        
        prop_assert_ne!(
            key1.as_bytes(), key2.as_bytes(),
            "Different salts must produce different keys"
        );
    }

    /// Feature: russh-ssh, Property: Nonce Uniqueness
    ///
    /// *For any* plaintext encrypted multiple times, each encryption SHALL
    /// use a different nonce (with overwhelming probability).
    ///
    /// **Validates: Requirements 4.1 - Nonce uniqueness for AES-GCM**
    #[test]
    fn encryption_uses_unique_nonces(
        plaintext in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        let key = EncryptionKey::generate().unwrap();
        
        let encrypted1 = encrypt(&key, &plaintext).unwrap();
        let encrypted2 = encrypt(&key, &plaintext).unwrap();
        let encrypted3 = encrypt(&key, &plaintext).unwrap();
        
        // All nonces should be different
        prop_assert_ne!(encrypted1.nonce, encrypted2.nonce, "Nonces must be unique (1 vs 2)");
        prop_assert_ne!(encrypted2.nonce, encrypted3.nonce, "Nonces must be unique (2 vs 3)");
        prop_assert_ne!(encrypted1.nonce, encrypted3.nonce, "Nonces must be unique (1 vs 3)");
    }

    /// Feature: russh-ssh, Property: Ciphertext Indistinguishability
    ///
    /// *For any* plaintext encrypted multiple times, the ciphertexts SHALL
    /// be different (semantic security).
    ///
    /// **Validates: Requirements 4.1 - Semantic security**
    #[test]
    fn ciphertexts_are_indistinguishable(
        plaintext in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        let key = EncryptionKey::generate().unwrap();
        
        let encrypted1 = encrypt(&key, &plaintext).unwrap();
        let encrypted2 = encrypt(&key, &plaintext).unwrap();
        
        prop_assert_ne!(
            encrypted1.ciphertext, encrypted2.ciphertext,
            "Same plaintext must produce different ciphertexts"
        );
    }

    /// Feature: russh-ssh, Property: Secure Channel Key Independence
    ///
    /// *For any* two secure channel establishments, the derived keys SHALL
    /// be different (due to ephemeral key pairs).
    ///
    /// **Validates: Requirements 4.2 - Forward secrecy**
    #[test]
    fn secure_channels_have_independent_keys(
        plaintext in prop::collection::vec(any::<u8>(), 1..500),
    ) {
        // Establish first channel
        let builder1_init = SecureChannelBuilder::new().unwrap();
        let init1 = builder1_init.create_init_message();
        let builder1_resp = SecureChannelBuilder::new().unwrap();
        let (_channel1_resp, response1) = builder1_resp.process_init(init1).unwrap();
        let channel1_init = builder1_init.process_response(response1).unwrap();
        
        // Establish second channel
        let builder2_init = SecureChannelBuilder::new().unwrap();
        let init2 = builder2_init.create_init_message();
        let builder2_resp = SecureChannelBuilder::new().unwrap();
        let (channel2_resp, response2) = builder2_resp.process_init(init2).unwrap();
        let channel2_init = builder2_init.process_response(response2).unwrap();
        
        // Encrypt same plaintext with both channels
        let encrypted1 = channel1_init.encrypt(&plaintext).unwrap();
        let encrypted2 = channel2_init.encrypt(&plaintext).unwrap();
        
        // Ciphertexts should be different (different keys)
        prop_assert_ne!(
            encrypted1.encrypted.ciphertext.clone(), encrypted2.encrypted.ciphertext,
            "Different channels must use different keys"
        );
        
        // Cross-channel decryption should fail
        let result = channel2_resp.decrypt(&encrypted1);
        prop_assert!(result.is_err(), "Cross-channel decryption must fail");
    }

    /// Feature: russh-ssh, Property: Bidirectional Channel Security
    ///
    /// *For any* secure channel, messages encrypted by one party SHALL only
    /// be decryptable by the other party.
    ///
    /// **Validates: Requirements 4.2 - Mutual authentication**
    #[test]
    fn bidirectional_channel_security(
        plaintext1 in prop::collection::vec(any::<u8>(), 1..500),
        plaintext2 in prop::collection::vec(any::<u8>(), 1..500),
    ) {
        // Establish channel
        let builder_init = SecureChannelBuilder::new().unwrap();
        let init = builder_init.create_init_message();
        let builder_resp = SecureChannelBuilder::new().unwrap();
        let (channel_resp, response) = builder_resp.process_init(init).unwrap();
        let channel_init = builder_init.process_response(response).unwrap();
        
        // Initiator -> Responder
        let encrypted1 = channel_init.encrypt(&plaintext1).unwrap();
        let decrypted1 = channel_resp.decrypt(&encrypted1).unwrap();
        prop_assert_eq!(&plaintext1, &decrypted1, "Initiator->Responder must work");
        
        // Responder -> Initiator
        let encrypted2 = channel_resp.encrypt(&plaintext2).unwrap();
        let decrypted2 = channel_init.decrypt(&encrypted2).unwrap();
        prop_assert_eq!(&plaintext2, &decrypted2, "Responder->Initiator must work");
        
        // Initiator cannot decrypt its own messages
        let result = channel_init.decrypt(&encrypted1);
        prop_assert!(result.is_err(), "Initiator must not decrypt its own messages");
        
        // Responder cannot decrypt its own messages
        let result = channel_resp.decrypt(&encrypted2);
        prop_assert!(result.is_err(), "Responder must not decrypt its own messages");
    }
}

/// Tests for authentication tag integrity
#[cfg(test)]
mod authentication_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: russh-ssh, Property: Authentication Tag Integrity
        ///
        /// *For any* ciphertext with a modified authentication tag, decryption
        /// SHALL fail.
        ///
        /// **Validates: Requirements 4.5 - Tamper detection**
        #[test]
        fn modified_auth_tag_fails(
            plaintext in prop::collection::vec(any::<u8>(), 1..1000),
            tag_byte_index in 0usize..16,
        ) {
            let key = EncryptionKey::generate().unwrap();
            let mut encrypted = encrypt(&key, &plaintext).unwrap();
            
            // The auth tag is the last 16 bytes of the ciphertext
            if encrypted.ciphertext.len() >= 16 {
                let tag_start = encrypted.ciphertext.len() - 16;
                let idx = tag_start + (tag_byte_index % 16);
                encrypted.ciphertext[idx] = encrypted.ciphertext[idx].wrapping_add(1);
                
                let result = decrypt(&key, &encrypted);
                prop_assert!(result.is_err(), "Modified auth tag must fail decryption");
            }
        }

        /// Feature: russh-ssh, Property: Ciphertext Integrity
        ///
        /// *For any* ciphertext with modified content (not tag), decryption
        /// SHALL fail.
        ///
        /// **Validates: Requirements 4.5 - Tamper detection**
        #[test]
        fn modified_ciphertext_content_fails(
            plaintext in prop::collection::vec(any::<u8>(), 17..1000),
            content_byte_index in any::<usize>(),
        ) {
            let key = EncryptionKey::generate().unwrap();
            let mut encrypted = encrypt(&key, &plaintext).unwrap();
            
            // Modify a byte in the content (not the tag)
            if encrypted.ciphertext.len() > 16 {
                let content_len = encrypted.ciphertext.len() - 16;
                let idx = content_byte_index % content_len;
                encrypted.ciphertext[idx] = encrypted.ciphertext[idx].wrapping_add(1);
                
                let result = decrypt(&key, &encrypted);
                prop_assert!(result.is_err(), "Modified ciphertext content must fail decryption");
            }
        }
    }
}

/// Tests for hash collision resistance
#[cfg(test)]
mod hash_collision_tests {
    use super::*;
    use std::collections::HashSet;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: russh-ssh, Property: Hash Collision Resistance
        ///
        /// *For any* set of different inputs, the hashes SHALL all be different
        /// (with cryptographic probability).
        ///
        /// **Validates: Requirements 4.3 - Collision resistance**
        #[test]
        fn no_hash_collisions_in_batch(
            inputs in prop::collection::vec(
                prop::collection::vec(any::<u8>(), 1..100),
                2..20
            )
        ) {
            // Filter to unique inputs
            let unique_inputs: Vec<_> = inputs.into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            
            if unique_inputs.len() < 2 {
                return Ok(());
            }
            
            let hashes: Vec<_> = unique_inputs.iter()
                .map(|input| hash_data(input))
                .collect();
            
            let unique_hashes: HashSet<_> = hashes.iter().collect();
            
            prop_assert_eq!(
                unique_inputs.len(), unique_hashes.len(),
                "All unique inputs must produce unique hashes"
            );
        }
    }
}
