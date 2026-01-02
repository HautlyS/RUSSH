//! Secure Channel implementation following OCKAM principles
//!
//! This module provides end-to-end encrypted secure channels with:
//! - Mutual authentication between peers
//! - Key agreement using X25519 Diffie-Hellman
//! - Message encryption using AES-256-GCM
//! - BLAKE3 for key derivation and integrity
//! - Replay protection with sliding window

use crate::encryption::cipher::{decrypt, encrypt, EncryptedMessage, EncryptionKey, KEY_SIZE};
use crate::encryption::hash::{hash_data, ContentHash};
use crate::error::EncryptionError;
use ring::agreement::{self, EphemeralPrivateKey, UnparsedPublicKey, X25519};
use ring::rand::SystemRandom;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// Size of X25519 public key in bytes
pub const PUBLIC_KEY_SIZE: usize = 32;

/// Size of the replay protection window
const REPLAY_WINDOW_SIZE: u64 = 64;

/// A cryptographic identity for secure channel establishment
#[derive(Clone)]
pub struct Identity {
    /// The public key for this identity
    pub public_key: [u8; PUBLIC_KEY_SIZE],
    /// Unique identifier derived from public key
    pub identifier: ContentHash,
}

impl Identity {
    /// Create an identity from a public key
    pub fn from_public_key(public_key: [u8; PUBLIC_KEY_SIZE]) -> Self {
        let identifier = hash_data(&public_key);
        Self {
            public_key,
            identifier,
        }
    }

    /// Get the identifier as a hex string
    pub fn identifier_hex(&self) -> String {
        self.identifier.to_hex()
    }
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("identifier", &self.identifier_hex())
            .finish()
    }
}

impl Serialize for Identity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Identity", 2)?;
        state.serialize_field("public_key", &hex::encode(&self.public_key))?;
        state.serialize_field("identifier", &self.identifier)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Identity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            public_key: String,
            identifier: ContentHash,
        }

        let helper = Helper::deserialize(deserializer)?;
        let public_key_bytes = hex::decode(&helper.public_key)
            .map_err(serde::de::Error::custom)?;

        if public_key_bytes.len() != PUBLIC_KEY_SIZE {
            return Err(serde::de::Error::custom("Invalid public key length"));
        }

        let mut public_key = [0u8; PUBLIC_KEY_SIZE];
        public_key.copy_from_slice(&public_key_bytes);

        Ok(Self {
            public_key,
            identifier: helper.identifier,
        })
    }
}

/// Key pair for secure channel establishment
pub struct KeyPair {
    /// The private key (ephemeral, used for key agreement)
    private_key: EphemeralPrivateKey,
    /// The public key bytes
    public_key_bytes: [u8; PUBLIC_KEY_SIZE],
}

impl KeyPair {
    /// Generate a new X25519 key pair
    pub fn generate() -> Result<Self, EncryptionError> {
        let rng = SystemRandom::new();
        let private_key = EphemeralPrivateKey::generate(&X25519, &rng)
            .map_err(|_| EncryptionError::KeyGeneration("Failed to generate X25519 key pair".into()))?;

        let public_key = private_key.compute_public_key()
            .map_err(|_| EncryptionError::KeyGeneration("Failed to compute public key".into()))?;

        let mut public_key_bytes = [0u8; PUBLIC_KEY_SIZE];
        public_key_bytes.copy_from_slice(public_key.as_ref());

        Ok(Self {
            private_key,
            public_key_bytes,
        })
    }

    /// Get the public key bytes
    pub fn public_key(&self) -> &[u8; PUBLIC_KEY_SIZE] {
        &self.public_key_bytes
    }

    /// Create an Identity from this key pair
    pub fn identity(&self) -> Identity {
        Identity::from_public_key(self.public_key_bytes)
    }

    /// Perform key agreement with a peer's public key
    pub fn agree(self, peer_public_key: &[u8; PUBLIC_KEY_SIZE]) -> Result<SharedSecret, EncryptionError> {
        let peer_public = UnparsedPublicKey::new(&X25519, peer_public_key);

        agreement::agree_ephemeral(self.private_key, &peer_public, |shared_secret| {
            let mut secret_bytes = [0u8; 32];
            secret_bytes.copy_from_slice(shared_secret);
            SharedSecret(secret_bytes)
        })
        .map_err(|_| EncryptionError::ChannelEstablishment("Key agreement failed".into()))
    }
}

/// Shared secret from key agreement
pub struct SharedSecret([u8; 32]);

impl SharedSecret {
    /// Derive encryption keys from the shared secret
    pub fn derive_keys(&self, context: &[u8]) -> DerivedKeys {
        // Use BLAKE3 key derivation
        let mut hasher = blake3::Hasher::new_derive_key("russh-ssh secure channel keys");
        hasher.update(&self.0);
        hasher.update(context);

        let mut output = [0u8; 64]; // 32 bytes for each direction
        hasher.finalize_xof().fill(&mut output);

        let mut initiator_key = [0u8; KEY_SIZE];
        let mut responder_key = [0u8; KEY_SIZE];
        initiator_key.copy_from_slice(&output[..32]);
        responder_key.copy_from_slice(&output[32..]);

        DerivedKeys {
            initiator_key: EncryptionKey::from_bytes(initiator_key),
            responder_key: EncryptionKey::from_bytes(responder_key),
        }
    }
}

/// Keys derived from shared secret for bidirectional communication
pub struct DerivedKeys {
    /// Key for messages from initiator to responder
    pub initiator_key: EncryptionKey,
    /// Key for messages from responder to initiator
    pub responder_key: EncryptionKey,
}

/// Role in the secure channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelRole {
    /// The party that initiated the channel
    Initiator,
    /// The party that responded to the channel request
    Responder,
}

/// Replay protection window using a bitmap
/// 
/// Implements a sliding window algorithm to detect replay attacks while
/// allowing for out-of-order message delivery within the window.
struct ReplayWindow {
    /// The highest counter value seen
    highest_seen: u64,
    /// Bitmap of seen messages within the window
    /// Bit i is set if message (highest_seen - i) has been seen
    bitmap: u64,
}

impl ReplayWindow {
    fn new() -> Self {
        Self {
            highest_seen: 0,
            bitmap: 0,
        }
    }
    
    /// Check if a message counter is valid (not a replay) and mark it as seen
    /// Returns true if the message is valid, false if it's a replay
    fn check_and_mark(&mut self, counter: u64) -> bool {
        if counter > self.highest_seen {
            // New highest counter - shift the window
            let shift = counter - self.highest_seen;
            if shift >= REPLAY_WINDOW_SIZE {
                // Counter is way ahead, reset the bitmap
                self.bitmap = 1; // Mark the new highest as seen
            } else {
                // Shift the bitmap and mark the new highest
                self.bitmap = (self.bitmap << shift) | 1;
            }
            self.highest_seen = counter;
            true
        } else if self.highest_seen - counter >= REPLAY_WINDOW_SIZE {
            // Counter is too old (outside the window)
            false
        } else {
            // Counter is within the window - check if already seen
            let bit_index = self.highest_seen - counter;
            let bit_mask = 1u64 << bit_index;
            if self.bitmap & bit_mask != 0 {
                // Already seen - replay attack
                false
            } else {
                // Not seen - mark as seen
                self.bitmap |= bit_mask;
                true
            }
        }
    }
}

/// An established secure channel for encrypted communication
pub struct SecureChannel {
    /// Our role in the channel
    role: ChannelRole,
    /// Key for encrypting outgoing messages
    encrypt_key: EncryptionKey,
    /// Key for decrypting incoming messages
    decrypt_key: EncryptionKey,
    /// Our identity
    local_identity: Identity,
    /// Peer's identity
    peer_identity: Identity,
    /// Message counter for replay protection
    send_counter: AtomicU64,
    /// Replay protection window
    replay_window: RwLock<ReplayWindow>,
}

impl SecureChannel {
    /// Create a new secure channel from derived keys
    pub fn new(
        role: ChannelRole,
        keys: DerivedKeys,
        local_identity: Identity,
        peer_identity: Identity,
    ) -> Self {
        let (encrypt_key, decrypt_key) = match role {
            ChannelRole::Initiator => (keys.initiator_key, keys.responder_key),
            ChannelRole::Responder => (keys.responder_key, keys.initiator_key),
        };

        Self {
            role,
            encrypt_key,
            decrypt_key,
            local_identity,
            peer_identity,
            send_counter: AtomicU64::new(0),
            replay_window: RwLock::new(ReplayWindow::new()),
        }
    }

    /// Get our role in the channel
    pub fn role(&self) -> ChannelRole {
        self.role
    }

    /// Get our identity
    pub fn local_identity(&self) -> &Identity {
        &self.local_identity
    }

    /// Get the peer's identity
    pub fn peer_identity(&self) -> &Identity {
        &self.peer_identity
    }

    /// Encrypt a message for sending through the channel
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<SecureMessage, EncryptionError> {
        let counter = self.send_counter.fetch_add(1, Ordering::SeqCst);
        let encrypted = encrypt(&self.encrypt_key, plaintext)?;

        Ok(SecureMessage {
            encrypted,
            counter,
            sender: self.local_identity.identifier.clone(),
        })
    }

    /// Decrypt a message received through the channel
    pub fn decrypt(&self, message: &SecureMessage) -> Result<Vec<u8>, EncryptionError> {
        // Verify sender
        if message.sender != self.peer_identity.identifier {
            return Err(EncryptionError::AuthenticationFailed);
        }

        // Check counter for replay protection using sliding window
        {
            let mut window = self.replay_window.write()
                .map_err(|_| EncryptionError::ChannelEstablishment("Lock poisoned".into()))?;
            if !window.check_and_mark(message.counter) {
                return Err(EncryptionError::AuthenticationFailed);
            }
        }

        decrypt(&self.decrypt_key, &message.encrypted)
    }
}

/// A message sent through a secure channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureMessage {
    /// The encrypted payload
    pub encrypted: EncryptedMessage,
    /// Message counter for ordering and replay protection
    pub counter: u64,
    /// Sender's identifier
    pub sender: ContentHash,
}

/// Handshake message for establishing a secure channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeMessage {
    /// Initial message from initiator containing their public key
    Init {
        /// Initiator's ephemeral public key
        public_key: [u8; PUBLIC_KEY_SIZE],
        /// Initiator's identity
        identity: Identity,
    },
    /// Response from responder containing their public key
    Response {
        /// Responder's ephemeral public key
        public_key: [u8; PUBLIC_KEY_SIZE],
        /// Responder's identity
        identity: Identity,
    },
}

/// Builder for establishing secure channels
pub struct SecureChannelBuilder {
    local_keypair: KeyPair,
    local_identity: Identity,
}

impl SecureChannelBuilder {
    /// Create a new secure channel builder
    pub fn new() -> Result<Self, EncryptionError> {
        let local_keypair = KeyPair::generate()?;
        let local_identity = local_keypair.identity();

        Ok(Self {
            local_keypair,
            local_identity,
        })
    }

    /// Get the local identity
    pub fn local_identity(&self) -> &Identity {
        &self.local_identity
    }

    /// Create the initial handshake message (for initiator)
    pub fn create_init_message(&self) -> HandshakeMessage {
        HandshakeMessage::Init {
            public_key: *self.local_keypair.public_key(),
            identity: self.local_identity.clone(),
        }
    }

    /// Process an init message and create a response (for responder)
    pub fn process_init(
        self,
        init: HandshakeMessage,
    ) -> Result<(SecureChannel, HandshakeMessage), EncryptionError> {
        let (peer_public_key, peer_identity) = match init {
            HandshakeMessage::Init { public_key, identity } => (public_key, identity),
            _ => return Err(EncryptionError::ChannelEstablishment("Expected Init message".into())),
        };

        // Create response message
        let response = HandshakeMessage::Response {
            public_key: *self.local_keypair.public_key(),
            identity: self.local_identity.clone(),
        };

        // Perform key agreement
        let shared_secret = self.local_keypair.agree(&peer_public_key)?;

        // Derive keys with context including both identities
        let mut context = Vec::new();
        context.extend_from_slice(&peer_identity.public_key);
        context.extend_from_slice(&self.local_identity.public_key);
        let keys = shared_secret.derive_keys(&context);

        // Create secure channel as responder
        let channel = SecureChannel::new(
            ChannelRole::Responder,
            keys,
            self.local_identity,
            peer_identity,
        );

        Ok((channel, response))
    }

    /// Process a response message and complete channel establishment (for initiator)
    pub fn process_response(
        self,
        response: HandshakeMessage,
    ) -> Result<SecureChannel, EncryptionError> {
        let (peer_public_key, peer_identity) = match response {
            HandshakeMessage::Response { public_key, identity } => (public_key, identity),
            _ => return Err(EncryptionError::ChannelEstablishment("Expected Response message".into())),
        };

        // Perform key agreement
        let shared_secret = self.local_keypair.agree(&peer_public_key)?;

        // Derive keys with context including both identities
        let mut context = Vec::new();
        context.extend_from_slice(&self.local_identity.public_key);
        context.extend_from_slice(&peer_identity.public_key);
        let keys = shared_secret.derive_keys(&context);

        // Create secure channel as initiator
        let channel = SecureChannel::new(
            ChannelRole::Initiator,
            keys,
            self.local_identity,
            peer_identity,
        );

        Ok(channel)
    }
}

impl Default for SecureChannelBuilder {
    fn default() -> Self {
        Self::new().expect("Failed to create SecureChannelBuilder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_generation() {
        let keypair = KeyPair::generate().unwrap();
        assert_eq!(keypair.public_key().len(), PUBLIC_KEY_SIZE);
    }

    #[test]
    fn identity_from_public_key() {
        let keypair = KeyPair::generate().unwrap();
        let identity = keypair.identity();
        assert_eq!(identity.public_key, *keypair.public_key());
    }

    #[test]
    fn secure_channel_establishment() {
        // Initiator creates builder and init message
        let initiator_builder = SecureChannelBuilder::new().unwrap();
        let init_msg = initiator_builder.create_init_message();

        // Responder processes init and creates response
        let responder_builder = SecureChannelBuilder::new().unwrap();
        let (responder_channel, response_msg) = responder_builder.process_init(init_msg).unwrap();

        // Initiator processes response
        let initiator_channel = initiator_builder.process_response(response_msg).unwrap();

        // Verify roles
        assert_eq!(initiator_channel.role(), ChannelRole::Initiator);
        assert_eq!(responder_channel.role(), ChannelRole::Responder);

        // Verify identities match
        assert_eq!(
            initiator_channel.peer_identity().identifier,
            responder_channel.local_identity().identifier
        );
        assert_eq!(
            responder_channel.peer_identity().identifier,
            initiator_channel.local_identity().identifier
        );
    }

    #[test]
    fn secure_channel_encryption_roundtrip() {
        // Establish channel
        let initiator_builder = SecureChannelBuilder::new().unwrap();
        let init_msg = initiator_builder.create_init_message();

        let responder_builder = SecureChannelBuilder::new().unwrap();
        let (responder_channel, response_msg) = responder_builder.process_init(init_msg).unwrap();

        let initiator_channel = initiator_builder.process_response(response_msg).unwrap();

        // Test initiator -> responder
        let plaintext = b"Hello from initiator!";
        let encrypted = initiator_channel.encrypt(plaintext).unwrap();
        let decrypted = responder_channel.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());

        // Test responder -> initiator
        let plaintext2 = b"Hello from responder!";
        let encrypted2 = responder_channel.encrypt(plaintext2).unwrap();
        let decrypted2 = initiator_channel.decrypt(&encrypted2).unwrap();
        assert_eq!(plaintext2.as_slice(), decrypted2.as_slice());
    }

    #[test]
    fn wrong_channel_cannot_decrypt() {
        // Establish two separate channels
        let builder1 = SecureChannelBuilder::new().unwrap();
        let init1 = builder1.create_init_message();
        let builder2 = SecureChannelBuilder::new().unwrap();
        let (channel1_responder, response1) = builder2.process_init(init1).unwrap();
        let channel1_initiator = builder1.process_response(response1).unwrap();

        let builder3 = SecureChannelBuilder::new().unwrap();
        let init3 = builder3.create_init_message();
        let builder4 = SecureChannelBuilder::new().unwrap();
        let (channel2_responder, response3) = builder4.process_init(init3).unwrap();
        let _channel2_initiator = builder3.process_response(response3).unwrap();

        // Encrypt with channel 1
        let plaintext = b"Secret message";
        let encrypted = channel1_initiator.encrypt(plaintext).unwrap();

        // Verify correct channel can decrypt
        let decrypted = channel1_responder.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());

        // Encrypt another message for cross-channel test
        let encrypted2 = channel1_initiator.encrypt(plaintext).unwrap();

        // Try to decrypt with channel 2 - should fail
        let result = channel2_responder.decrypt(&encrypted2);
        assert!(result.is_err());
    }

    #[test]
    fn identity_serialization() {
        let keypair = KeyPair::generate().unwrap();
        let identity = keypair.identity();

        let json = serde_json::to_string(&identity).unwrap();
        let deserialized: Identity = serde_json::from_str(&json).unwrap();

        assert_eq!(identity.public_key, deserialized.public_key);
        assert_eq!(identity.identifier, deserialized.identifier);
    }

    #[test]
    fn handshake_message_serialization() {
        let builder = SecureChannelBuilder::new().unwrap();
        let init_msg = builder.create_init_message();

        let json = serde_json::to_string(&init_msg).unwrap();
        let deserialized: HandshakeMessage = serde_json::from_str(&json).unwrap();

        match (init_msg, deserialized) {
            (
                HandshakeMessage::Init { public_key: pk1, identity: id1 },
                HandshakeMessage::Init { public_key: pk2, identity: id2 },
            ) => {
                assert_eq!(pk1, pk2);
                assert_eq!(id1.identifier, id2.identifier);
            }
            _ => panic!("Deserialization produced wrong variant"),
        }
    }
    
    #[test]
    fn replay_attack_prevention() {
        // Establish channel
        let initiator_builder = SecureChannelBuilder::new().unwrap();
        let init_msg = initiator_builder.create_init_message();

        let responder_builder = SecureChannelBuilder::new().unwrap();
        let (responder_channel, response_msg) = responder_builder.process_init(init_msg).unwrap();

        let initiator_channel = initiator_builder.process_response(response_msg).unwrap();

        // Send a message
        let plaintext = b"Original message";
        let encrypted = initiator_channel.encrypt(plaintext).unwrap();
        
        // First decryption should succeed
        let decrypted = responder_channel.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
        
        // Replay attack - same message again should fail
        let result = responder_channel.decrypt(&encrypted);
        assert!(result.is_err(), "Replay attack should be detected");
    }
    
    #[test]
    fn out_of_order_messages_within_window() {
        // Establish channel
        let initiator_builder = SecureChannelBuilder::new().unwrap();
        let init_msg = initiator_builder.create_init_message();

        let responder_builder = SecureChannelBuilder::new().unwrap();
        let (responder_channel, response_msg) = responder_builder.process_init(init_msg).unwrap();

        let initiator_channel = initiator_builder.process_response(response_msg).unwrap();

        // Send multiple messages
        let msg1 = initiator_channel.encrypt(b"Message 1").unwrap();
        let msg2 = initiator_channel.encrypt(b"Message 2").unwrap();
        let msg3 = initiator_channel.encrypt(b"Message 3").unwrap();
        
        // Receive out of order (3, 1, 2) - all should succeed
        assert!(responder_channel.decrypt(&msg3).is_ok());
        assert!(responder_channel.decrypt(&msg1).is_ok());
        assert!(responder_channel.decrypt(&msg2).is_ok());
        
        // Replay any of them should fail
        assert!(responder_channel.decrypt(&msg1).is_err());
        assert!(responder_channel.decrypt(&msg2).is_err());
        assert!(responder_channel.decrypt(&msg3).is_err());
    }
}
