//! Encryption and decryption utilities
//!
//! This module provides symmetric encryption using AES-256-GCM via ring.
//! While the design mentions OCKAM, we use ring for the core encryption
//! primitives as it provides the same security guarantees.

use crate::error::EncryptionError;
use crate::encryption::hash::{hash_data, ContentHash};
use ring::aead::{self, Aad, BoundKey, Nonce, NonceSequence, NONCE_LEN, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};

/// Size of the encryption key in bytes (256 bits)
pub const KEY_SIZE: usize = 32;

/// Size of the nonce in bytes (96 bits for AES-GCM)
pub const NONCE_SIZE: usize = NONCE_LEN;

/// Size of the authentication tag in bytes
pub const TAG_SIZE: usize = 16;

/// Encrypted message wrapper containing ciphertext, nonce, and content hash
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedMessage {
    /// The encrypted data including authentication tag
    pub ciphertext: Vec<u8>,
    /// The nonce used for encryption
    pub nonce: [u8; NONCE_SIZE],
    /// BLAKE3 hash of the original plaintext for integrity verification
    pub plaintext_hash: ContentHash,
}

impl EncryptedMessage {
    /// Get the size of the encrypted message
    pub fn size(&self) -> usize {
        self.ciphertext.len()
    }
}

impl serde::Serialize for EncryptedMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let mut state = serializer.serialize_struct("EncryptedMessage", 3)?;
        state.serialize_field("ciphertext", &STANDARD.encode(&self.ciphertext))?;
        state.serialize_field("nonce", &STANDARD.encode(self.nonce))?;
        state.serialize_field("plaintext_hash", &self.plaintext_hash)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for EncryptedMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        
        #[derive(serde::Deserialize)]
        struct Helper {
            ciphertext: String,
            nonce: String,
            plaintext_hash: ContentHash,
        }

        let helper = Helper::deserialize(deserializer)?;
        let ciphertext = STANDARD.decode(&helper.ciphertext)
            .map_err(serde::de::Error::custom)?;
        let nonce_bytes = STANDARD.decode(&helper.nonce)
            .map_err(serde::de::Error::custom)?;

        if nonce_bytes.len() != NONCE_SIZE {
            return Err(serde::de::Error::custom("Invalid nonce length"));
        }

        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&nonce_bytes);

        Ok(EncryptedMessage {
            ciphertext,
            nonce,
            plaintext_hash: helper.plaintext_hash,
        })
    }
}

/// Encryption key wrapper
#[derive(Clone)]
pub struct EncryptionKey {
    key_bytes: [u8; KEY_SIZE],
}

impl EncryptionKey {
    /// Generate a new random encryption key
    pub fn generate() -> Result<Self, EncryptionError> {
        let rng = SystemRandom::new();
        let mut key_bytes = [0u8; KEY_SIZE];
        rng.fill(&mut key_bytes)
            .map_err(|_| EncryptionError::KeyGeneration("Failed to generate random key".into()))?;
        Ok(Self { key_bytes })
    }

    /// Create a key from raw bytes
    pub fn from_bytes(bytes: [u8; KEY_SIZE]) -> Self {
        Self { key_bytes: bytes }
    }

    /// Get the raw key bytes
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.key_bytes
    }

    /// Derive a key from a password using PBKDF2-HMAC-SHA256
    /// 
    /// Uses PBKDF2 with OWASP-recommended iteration count for password-based key derivation.
    /// 
    /// # Security
    /// - Uses PBKDF2-HMAC-SHA256
    /// - 600,000 iterations (OWASP recommended minimum for PBKDF2-SHA256)
    /// - Salt MUST be at least 16 bytes and cryptographically random
    /// 
    /// # Panics
    /// Panics if salt is less than 16 bytes (security requirement)
    pub fn from_password(password: &[u8], salt: &[u8]) -> Self {
        use ring::pbkdf2;
        
        // Enforce minimum salt length for security
        assert!(salt.len() >= 16, "Salt must be at least 16 bytes for security");
        
        // OWASP recommended minimum for PBKDF2-SHA256 (as of 2023)
        const ITERATIONS: u32 = 600_000;
        
        let mut key_bytes = [0u8; KEY_SIZE];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(ITERATIONS).unwrap(),
            salt,
            password,
            &mut key_bytes,
        );
        Self { key_bytes }
    }
    
    /// Generate a cryptographically secure random salt for password-based key derivation
    /// 
    /// Returns a 32-byte random salt suitable for use with `from_password`.
    pub fn generate_salt() -> Result<[u8; 32], EncryptionError> {
        let rng = SystemRandom::new();
        let mut salt = [0u8; 32];
        rng.fill(&mut salt)
            .map_err(|_| EncryptionError::KeyGeneration("Failed to generate random salt".into()))?;
        Ok(salt)
    }
    
    /// Derive a key from a password using BLAKE3 (fast, for non-password use cases)
    /// 
    /// WARNING: This is NOT suitable for password-based key derivation.
    /// Use `from_password` for password-based keys.
    /// This method is suitable for deriving keys from high-entropy secrets.
    pub fn from_high_entropy_secret(secret: &[u8], context: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new_derive_key("russh-ssh encryption key");
        hasher.update(secret);
        hasher.update(context);
        let mut key_bytes = [0u8; KEY_SIZE];
        hasher.finalize_xof().fill(&mut key_bytes);
        Self { key_bytes }
    }
}

impl std::fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionKey")
            .field("key_bytes", &"[REDACTED]")
            .finish()
    }
}

/// Nonce sequence that uses a single nonce
struct SingleNonce(Option<Nonce>);

impl NonceSequence for SingleNonce {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        self.0.take().ok_or(ring::error::Unspecified)
    }
}

/// Encrypt plaintext using AES-256-GCM
///
/// Returns an EncryptedMessage containing the ciphertext, nonce, and plaintext hash.
pub fn encrypt(key: &EncryptionKey, plaintext: &[u8]) -> Result<EncryptedMessage, EncryptionError> {
    let rng = SystemRandom::new();

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| EncryptionError::Encryption("Failed to generate nonce".into()))?;

    // Compute plaintext hash before encryption
    let plaintext_hash = hash_data(plaintext);

    // Create sealing key
    let unbound_key = UnboundKey::new(&aead::AES_256_GCM, key.as_bytes())
        .map_err(|_| EncryptionError::Encryption("Failed to create encryption key".into()))?;

    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut sealing_key = aead::SealingKey::new(unbound_key, SingleNonce(Some(nonce)));

    // Encrypt in place
    let mut ciphertext = plaintext.to_vec();
    sealing_key
        .seal_in_place_append_tag(Aad::empty(), &mut ciphertext)
        .map_err(|_| EncryptionError::Encryption("Encryption failed".into()))?;

    Ok(EncryptedMessage {
        ciphertext,
        nonce: nonce_bytes,
        plaintext_hash,
    })
}

/// Decrypt ciphertext using AES-256-GCM
///
/// Returns the original plaintext if decryption and verification succeed.
pub fn decrypt(key: &EncryptionKey, message: &EncryptedMessage) -> Result<Vec<u8>, EncryptionError> {
    // Create opening key
    let unbound_key = UnboundKey::new(&aead::AES_256_GCM, key.as_bytes())
        .map_err(|_| EncryptionError::Decryption)?;

    let nonce = Nonce::assume_unique_for_key(message.nonce);
    let mut opening_key = aead::OpeningKey::new(unbound_key, SingleNonce(Some(nonce)));

    // Decrypt in place
    let mut plaintext = message.ciphertext.clone();
    let decrypted = opening_key
        .open_in_place(Aad::empty(), &mut plaintext)
        .map_err(|_| EncryptionError::Decryption)?;

    // Verify plaintext hash
    let computed_hash = hash_data(decrypted);
    if computed_hash != message.plaintext_hash {
        return Err(EncryptionError::AuthenticationFailed);
    }

    Ok(decrypted.to_vec())
}

/// Encrypt plaintext and return only the ciphertext bytes (without metadata)
pub fn encrypt_raw(key: &EncryptionKey, nonce: &[u8; NONCE_SIZE], plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let unbound_key = UnboundKey::new(&aead::AES_256_GCM, key.as_bytes())
        .map_err(|_| EncryptionError::Encryption("Failed to create encryption key".into()))?;

    let nonce = Nonce::assume_unique_for_key(*nonce);
    let mut sealing_key = aead::SealingKey::new(unbound_key, SingleNonce(Some(nonce)));

    let mut ciphertext = plaintext.to_vec();
    sealing_key
        .seal_in_place_append_tag(Aad::empty(), &mut ciphertext)
        .map_err(|_| EncryptionError::Encryption("Encryption failed".into()))?;

    Ok(ciphertext)
}

/// Decrypt raw ciphertext bytes
pub fn decrypt_raw(key: &EncryptionKey, nonce: &[u8; NONCE_SIZE], ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let unbound_key = UnboundKey::new(&aead::AES_256_GCM, key.as_bytes())
        .map_err(|_| EncryptionError::Decryption)?;

    let nonce = Nonce::assume_unique_for_key(*nonce);
    let mut opening_key = aead::OpeningKey::new(unbound_key, SingleNonce(Some(nonce)));

    let mut plaintext = ciphertext.to_vec();
    let decrypted = opening_key
        .open_in_place(Aad::empty(), &mut plaintext)
        .map_err(|_| EncryptionError::Decryption)?;

    Ok(decrypted.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = EncryptionKey::generate().unwrap();
        let plaintext = b"Hello, World!";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn encrypt_decrypt_empty() {
        let key = EncryptionKey::generate().unwrap();
        let plaintext = b"";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = EncryptionKey::generate().unwrap();
        let key2 = EncryptionKey::generate().unwrap();
        let plaintext = b"Secret message";

        let encrypted = encrypt(&key1, plaintext).unwrap();
        let result = decrypt(&key2, &encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = EncryptionKey::generate().unwrap();
        let plaintext = b"Secret message";

        let mut encrypted = encrypt(&key, plaintext).unwrap();
        // Tamper with ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }

        let result = decrypt(&key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn key_from_password() {
        let password = b"my secret password";
        let salt = b"random_salt_value_16"; // Salt must be at least 16 bytes

        let key1 = EncryptionKey::from_password(password, salt);
        let key2 = EncryptionKey::from_password(password, salt);

        assert_eq!(key1.as_bytes(), key2.as_bytes());

        // Different password should produce different key
        let key3 = EncryptionKey::from_password(b"different password", salt);
        assert_ne!(key1.as_bytes(), key3.as_bytes());
        
        // Different salt should produce different key
        let key4 = EncryptionKey::from_password(password, b"different_salt_16!!");
        assert_ne!(key1.as_bytes(), key4.as_bytes());
    }
    
    #[test]
    #[should_panic(expected = "Salt must be at least 16 bytes")]
    fn key_from_password_short_salt_panics() {
        let password = b"my secret password";
        let short_salt = b"short"; // Less than 16 bytes
        let _ = EncryptionKey::from_password(password, short_salt);
    }
    
    #[test]
    fn generate_salt_produces_unique_values() {
        let salt1 = EncryptionKey::generate_salt().unwrap();
        let salt2 = EncryptionKey::generate_salt().unwrap();
        assert_ne!(salt1, salt2, "Generated salts should be unique");
        assert_eq!(salt1.len(), 32, "Salt should be 32 bytes");
    }
    
    #[test]
    fn key_from_high_entropy_secret() {
        let secret = b"high entropy secret key material";
        let context = b"test context";

        let key1 = EncryptionKey::from_high_entropy_secret(secret, context);
        let key2 = EncryptionKey::from_high_entropy_secret(secret, context);

        assert_eq!(key1.as_bytes(), key2.as_bytes());

        // Different context should produce different key
        let key3 = EncryptionKey::from_high_entropy_secret(secret, b"other context");
        assert_ne!(key1.as_bytes(), key3.as_bytes());
    }

    #[test]
    fn encrypted_message_serialization() {
        let key = EncryptionKey::generate().unwrap();
        let plaintext = b"Test message for serialization";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let json = serde_json::to_string(&encrypted).unwrap();
        let deserialized: EncryptedMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(encrypted.ciphertext, deserialized.ciphertext);
        assert_eq!(encrypted.nonce, deserialized.nonce);
        assert_eq!(encrypted.plaintext_hash, deserialized.plaintext_hash);

        // Verify we can still decrypt
        let decrypted = decrypt(&key, &deserialized).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }
}
