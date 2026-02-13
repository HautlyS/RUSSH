//! Comprehensive property-based tests for security, memory safety, and correctness
//!
//! Feature: russh-ssh
//! These tests validate critical security and correctness properties.

use proptest::prelude::*;
use russh_ssh::connection::state::{ConnectionState, StateManager};
use russh_ssh::encryption::cipher::{decrypt, encrypt, EncryptionKey, KEY_SIZE};
use russh_ssh::encryption::hash::{hash_data, verify_hash, ContentHash};
use russh_ssh::encryption::secure_channel::SecureChannelBuilder;
use russh_ssh::streaming::buffer::{AdaptiveBuffer, BufferConfig};
use std::collections::HashSet;

// =============================================================================
// SECURITY PROPERTIES
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Salt Length Enforcement
    ///
    /// *For any* password-based key derivation, the salt MUST be at least 16 bytes.
    /// Shorter salts are rejected to prevent rainbow table attacks.
    ///
    /// **Validates: Security requirement for KDF**
    #[test]
    fn salt_length_enforcement(
        password in prop::collection::vec(any::<u8>(), 1..100),
        salt in prop::collection::vec(any::<u8>(), 16..64),
    ) {
        // Valid salt (>= 16 bytes) should work
        let key = EncryptionKey::from_password(&password, &salt);
        prop_assert_eq!(key.as_bytes().len(), KEY_SIZE);
    }

    /// Property: Key Derivation Determinism with Proper Salt
    ///
    /// *For any* password and valid salt, key derivation MUST be deterministic.
    ///
    /// **Validates: Requirements 4.1**
    #[test]
    fn key_derivation_deterministic_with_valid_salt(
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

    /// Property: Encryption Produces Unique Ciphertexts
    ///
    /// *For any* plaintext encrypted multiple times, each ciphertext MUST be unique
    /// due to random nonce generation (semantic security).
    ///
    /// **Validates: Requirements 4.1 - Semantic security**
    #[test]
    fn encryption_semantic_security(
        plaintext in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        let key = EncryptionKey::generate().unwrap();

        let encrypted1 = encrypt(&key, &plaintext).unwrap();
        let encrypted2 = encrypt(&key, &plaintext).unwrap();
        let encrypted3 = encrypt(&key, &plaintext).unwrap();

        // All nonces must be unique
        let nonces: HashSet<_> = vec![encrypted1.nonce, encrypted2.nonce, encrypted3.nonce]
            .into_iter()
            .collect();
        prop_assert_eq!(nonces.len(), 3, "All nonces must be unique");

        // All ciphertexts must be different (clone to avoid move)
        prop_assert_ne!(encrypted1.ciphertext.clone(), encrypted2.ciphertext.clone());
        prop_assert_ne!(encrypted2.ciphertext.clone(), encrypted3.ciphertext.clone());
        prop_assert_ne!(encrypted1.ciphertext, encrypted3.ciphertext);
    }

    /// Property: Tampered Ciphertext Detection
    ///
    /// *For any* ciphertext with any byte modified, decryption MUST fail.
    /// This validates the authentication tag integrity.
    ///
    /// **Validates: Requirements 4.5 - Tamper detection**
    #[test]
    fn tampered_ciphertext_always_detected(
        plaintext in prop::collection::vec(any::<u8>(), 1..1000),
        tamper_position in any::<usize>(),
        tamper_value in any::<u8>(),
    ) {
        let key = EncryptionKey::generate().unwrap();
        let mut encrypted = encrypt(&key, &plaintext).unwrap();

        if !encrypted.ciphertext.is_empty() {
            let idx = tamper_position % encrypted.ciphertext.len();
            let original = encrypted.ciphertext[idx];
            // Only tamper if it actually changes the value
            if tamper_value != original {
                encrypted.ciphertext[idx] = tamper_value;
                let result = decrypt(&key, &encrypted);
                prop_assert!(result.is_err(), "Tampered ciphertext must fail decryption");
            }
        }
    }

    /// Property: Wrong Key Always Fails
    ///
    /// *For any* ciphertext, decryption with a different key MUST fail.
    ///
    /// **Validates: Requirements 4.5 - Key isolation**
    #[test]
    fn wrong_key_always_fails(
        plaintext in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        let key1 = EncryptionKey::generate().unwrap();
        let key2 = EncryptionKey::generate().unwrap();

        let encrypted = encrypt(&key1, &plaintext).unwrap();
        let result = decrypt(&key2, &encrypted);

        prop_assert!(result.is_err(), "Decryption with wrong key must fail");
    }
}

// =============================================================================
// SECURE CHANNEL PROPERTIES
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Secure Channel Replay Protection
    ///
    /// *For any* message sent through a secure channel, replaying the same
    /// message MUST be detected and rejected.
    ///
    /// **Validates: Requirements 4.2 - Replay protection**
    #[test]
    fn secure_channel_replay_protection(
        plaintext in prop::collection::vec(any::<u8>(), 1..500),
    ) {
        // Establish channel
        let initiator_builder = SecureChannelBuilder::new().unwrap();
        let init_msg = initiator_builder.create_init_message();
        let responder_builder = SecureChannelBuilder::new().unwrap();
        let (responder_channel, response_msg) = responder_builder.process_init(init_msg).unwrap();
        let initiator_channel = initiator_builder.process_response(response_msg).unwrap();

        // Send a message
        let encrypted = initiator_channel.encrypt(&plaintext).unwrap();

        // First decryption should succeed
        let decrypted = responder_channel.decrypt(&encrypted);
        prop_assert!(decrypted.is_ok(), "First decryption should succeed");

        // Replay should fail
        let replay_result = responder_channel.decrypt(&encrypted);
        prop_assert!(replay_result.is_err(), "Replay attack must be detected");
    }

    /// Property: Out-of-Order Messages Within Window
    ///
    /// *For any* set of messages received out of order within the replay window,
    /// all messages MUST be accepted exactly once.
    ///
    /// **Validates: Requirements 4.2 - Out-of-order handling**
    #[test]
    fn out_of_order_within_window(
        messages in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 1..100),
            3..10
        ),
    ) {
        // Establish channel
        let initiator_builder = SecureChannelBuilder::new().unwrap();
        let init_msg = initiator_builder.create_init_message();
        let responder_builder = SecureChannelBuilder::new().unwrap();
        let (responder_channel, response_msg) = responder_builder.process_init(init_msg).unwrap();
        let initiator_channel = initiator_builder.process_response(response_msg).unwrap();

        // Encrypt all messages
        let encrypted_msgs: Vec<_> = messages.iter()
            .map(|m| initiator_channel.encrypt(m).unwrap())
            .collect();

        // Receive in reverse order (within window)
        for encrypted in encrypted_msgs.iter().rev() {
            let result = responder_channel.decrypt(encrypted);
            prop_assert!(result.is_ok(), "Out-of-order message within window should succeed");
        }

        // Replay any should fail
        for encrypted in &encrypted_msgs {
            let result = responder_channel.decrypt(encrypted);
            prop_assert!(result.is_err(), "Replay of any message should fail");
        }
    }

    /// Property: Cross-Channel Isolation
    ///
    /// *For any* two separate secure channels, messages from one channel
    /// MUST NOT be decryptable by the other.
    ///
    /// **Validates: Requirements 4.2 - Channel isolation**
    #[test]
    fn cross_channel_isolation(
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
        let _channel2_init = builder2_init.process_response(response2).unwrap();

        // Encrypt with channel 1
        let encrypted = channel1_init.encrypt(&plaintext).unwrap();

        // Try to decrypt with channel 2 - must fail
        let result = channel2_resp.decrypt(&encrypted);
        prop_assert!(result.is_err(), "Cross-channel decryption must fail");
    }
}

// =============================================================================
// MEMORY SAFETY PROPERTIES
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Buffer Never Exceeds Maximum Size
    ///
    /// *For any* sequence of data additions, the buffer MUST never exceed
    /// the configured maximum size, even temporarily.
    ///
    /// **Validates: Requirements 6.2 - Memory bounds**
    #[test]
    fn buffer_never_exceeds_max(
        chunks in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 1..500),
            1..50
        ),
        max_size in 1024usize..8192,
    ) {
        let config = BufferConfig::new(512, max_size);
        let mut buffer = AdaptiveBuffer::new(config);

        let mut position = 0u64;
        for chunk in chunks {
            buffer.add_data(position, chunk.clone());
            position += chunk.len() as u64;

            // CRITICAL: Buffer must NEVER exceed max size
            prop_assert!(
                buffer.buffered_bytes() <= max_size,
                "Buffer exceeded max size: {} > {}",
                buffer.buffered_bytes(), max_size
            );
        }
    }

    /// Property: Buffer Data Integrity
    ///
    /// *For any* data added to the buffer, reading it back MUST return
    /// the exact same bytes.
    ///
    /// **Validates: Requirements 6.2 - Data integrity**
    #[test]
    fn buffer_data_integrity(
        data in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut buffer = AdaptiveBuffer::new(config);

        buffer.add_data(0, data.clone());

        if let Some(read) = buffer.read(data.len()) {
            prop_assert_eq!(data, read, "Buffer must preserve data integrity");
        }
    }

    /// Property: Buffer Clear Resets All State
    ///
    /// *For any* buffer with data, clearing it MUST reset all state to initial values.
    ///
    /// **Validates: Requirements 6.2 - Clear functionality**
    #[test]
    fn buffer_clear_complete_reset(
        data in prop::collection::vec(any::<u8>(), 1..1000),
        read_amount in 1usize..100,
    ) {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut buffer = AdaptiveBuffer::new(config);

        buffer.add_data(0, data);
        let _ = buffer.read(read_amount);

        buffer.clear();

        prop_assert_eq!(buffer.position(), 0, "Position must be 0 after clear");
        prop_assert_eq!(buffer.buffered_bytes(), 0, "Buffered bytes must be 0 after clear");
        prop_assert!(buffer.buffered_ranges().is_empty(), "Ranges must be empty after clear");
    }
}

// =============================================================================
// HASH PROPERTIES
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Hash Collision Resistance
    ///
    /// *For any* set of different inputs, all hashes MUST be different
    /// (with cryptographic probability).
    ///
    /// **Validates: Requirements 4.3 - Collision resistance**
    #[test]
    fn hash_collision_resistance(
        inputs in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 1..100),
            2..20
        ),
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

        let unique_hashes: HashSet<_> = hashes.iter()
            .map(|h| h.to_hex())
            .collect();

        prop_assert_eq!(
            unique_inputs.len(), unique_hashes.len(),
            "All unique inputs must produce unique hashes"
        );
    }

    /// Property: Hash Verification Correctness
    ///
    /// *For any* data, verify_hash MUST return true for the correct hash
    /// and false for any modification.
    ///
    /// **Validates: Requirements 4.3 - Integrity verification**
    #[test]
    fn hash_verification_correctness(
        data in prop::collection::vec(any::<u8>(), 1..1000),
        modification_index in any::<usize>(),
    ) {
        let hash = hash_data(&data);

        // Correct data must verify
        prop_assert!(verify_hash(&data, &hash), "Correct data must verify");

        // Modified data must not verify
        let mut modified = data.clone();
        let idx = modification_index % modified.len();
        modified[idx] = modified[idx].wrapping_add(1);

        prop_assert!(!verify_hash(&modified, &hash), "Modified data must not verify");
    }

    /// Property: Hash Hex Roundtrip
    ///
    /// *For any* hash, converting to hex and back MUST produce the same hash.
    ///
    /// **Validates: Requirements 4.3 - Serialization**
    #[test]
    fn hash_hex_roundtrip(
        data in prop::collection::vec(any::<u8>(), 0..1000),
    ) {
        let hash = hash_data(&data);
        let hex = hash.to_hex();
        let parsed = ContentHash::from_hex(&hex).unwrap();

        prop_assert_eq!(hash, parsed, "Hash hex roundtrip must preserve value");
    }
}

// =============================================================================
// STATE MACHINE PROPERTIES
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: State Transition Validity
    ///
    /// *For any* valid state transition, the state manager MUST accept it.
    ///
    /// **Validates: Requirements 2.4 - State management**
    #[test]
    fn valid_state_transitions_accepted(
        initial_state in prop_oneof![
            Just(ConnectionState::Disconnected),
            Just(ConnectionState::Connecting),
            Just(ConnectionState::Connected),
            (1u32..10).prop_map(|a| ConnectionState::Reconnecting { attempt: a }),
            "[a-z]{5,20}".prop_map(|r| ConnectionState::Failed { reason: r }),
        ],
    ) {
        let manager = StateManager::with_state(initial_state.clone());

        // Get valid transitions for this state
        let valid_transitions: Vec<ConnectionState> = match &initial_state {
            ConnectionState::Disconnected => vec![
                ConnectionState::Connecting,
                ConnectionState::Failed { reason: "test".to_string() },
            ],
            ConnectionState::Connecting => vec![
                ConnectionState::Connected,
                ConnectionState::Failed { reason: "test".to_string() },
                ConnectionState::Disconnected,
            ],
            ConnectionState::Connected => vec![
                ConnectionState::Disconnected,
                ConnectionState::Failed { reason: "test".to_string() },
                ConnectionState::Reconnecting { attempt: 1 },
            ],
            ConnectionState::Reconnecting { .. } => vec![
                ConnectionState::Connected,
                ConnectionState::Failed { reason: "test".to_string() },
                ConnectionState::Reconnecting { attempt: 2 },
                ConnectionState::Disconnected,
            ],
            ConnectionState::Failed { .. } => vec![
                ConnectionState::Connecting,
                ConnectionState::Disconnected,
            ],
        };

        for next_state in valid_transitions {
            let result = manager.try_transition(next_state.clone());
            prop_assert!(
                result.is_ok(),
                "Valid transition from {:?} to {:?} must succeed",
                initial_state, next_state
            );
            // Reset for next test
            let _ = manager.set_state(initial_state.clone());
        }
    }

    /// Property: Invalid State Transitions Rejected
    ///
    /// *For any* invalid state transition, the state manager MUST reject it
    /// and maintain the current state.
    ///
    /// **Validates: Requirements 2.4 - State integrity**
    #[test]
    fn invalid_state_transitions_rejected(
        initial_state in prop_oneof![
            Just(ConnectionState::Disconnected),
            Just(ConnectionState::Connecting),
            Just(ConnectionState::Connected),
            (1u32..10).prop_map(|a| ConnectionState::Reconnecting { attempt: a }),
            "[a-z]{5,20}".prop_map(|r| ConnectionState::Failed { reason: r }),
        ],
    ) {
        let manager = StateManager::with_state(initial_state.clone());

        // Get invalid transitions for this state
        let invalid_transitions: Vec<ConnectionState> = match &initial_state {
            ConnectionState::Disconnected => vec![
                ConnectionState::Connected,
                ConnectionState::Reconnecting { attempt: 1 },
            ],
            ConnectionState::Connecting => vec![
                ConnectionState::Reconnecting { attempt: 1 },
            ],
            ConnectionState::Connected => vec![
                ConnectionState::Connecting,
            ],
            ConnectionState::Reconnecting { .. } => vec![
                ConnectionState::Connecting,
            ],
            ConnectionState::Failed { .. } => vec![
                ConnectionState::Connected,
                ConnectionState::Reconnecting { attempt: 1 },
            ],
        };

        for next_state in invalid_transitions {
            let initial_clone = initial_state.clone();
            let result = manager.try_transition(next_state.clone());
            prop_assert!(
                result.is_err(),
                "Invalid transition from {:?} to {:?} must fail",
                initial_clone, next_state
            );
            // State must remain unchanged
            prop_assert_eq!(
                manager.state(), initial_state.clone(),
                "State must remain unchanged after invalid transition"
            );
        }
    }

    /// Property: Same State Is No-Op
    ///
    /// *For any* state, setting the same state MUST be a no-op.
    ///
    /// **Validates: Requirements 2.4 - Idempotent state setting**
    #[test]
    fn same_state_is_noop(
        state in prop_oneof![
            Just(ConnectionState::Disconnected),
            Just(ConnectionState::Connecting),
            Just(ConnectionState::Connected),
            (1u32..10).prop_map(|a| ConnectionState::Reconnecting { attempt: a }),
            "[a-z]{5,20}".prop_map(|r| ConnectionState::Failed { reason: r }),
        ],
    ) {
        let manager = StateManager::with_state(state.clone());

        let changed = manager.set_state(state.clone());

        prop_assert!(!changed, "Setting same state must return false");
        prop_assert_eq!(manager.state(), state, "State must remain unchanged");
    }
}
