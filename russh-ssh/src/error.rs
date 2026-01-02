//! Error types for the russh SSH library
//!
//! This module defines all error types used throughout the library,
//! ensuring descriptive error messages for all failure scenarios.

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during connection operations
#[derive(Debug, Error)]
pub enum ConnectionError {
    /// Connection timed out
    #[error("Connection timeout after {0:?}")]
    Timeout(Duration),

    /// DNS resolution failed
    #[error("DNS resolution failed for host '{host}': {reason}")]
    DnsResolution { host: String, reason: String },

    /// Connection was refused by the remote host
    #[error("Connection refused by {host}:{port}")]
    ConnectionRefused { host: String, port: u16 },

    /// Network is unreachable
    #[error("Network unreachable: {0}")]
    NetworkUnreachable(String),

    /// TLS handshake failed
    #[error("TLS handshake failed: {0}")]
    TlsHandshake(String),

    /// Generic I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Connection was closed unexpectedly
    #[error("Connection closed unexpectedly: {0}")]
    ConnectionClosed(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Errors that can occur during SSH operations
#[derive(Debug, Error)]
pub enum SshError {
    /// Authentication failed
    #[error("Authentication failed for user '{user}': {reason}")]
    AuthenticationFailed { user: String, reason: String },

    /// Host key verification failed
    #[error("Host key verification failed for {host}")]
    HostKeyVerification { host: String },

    /// Failed to open SSH channel
    #[error("Channel open failed: {0}")]
    ChannelOpen(String),

    /// Command execution failed
    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    /// Session is not connected
    #[error("Session not connected")]
    NotConnected,

    /// Command timed out
    #[error("Command timed out after {0:?}")]
    CommandTimeout(Duration),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
}

/// Errors that can occur during encryption operations
#[derive(Debug, Error)]
pub enum EncryptionError {
    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),

    /// Encryption operation failed
    #[error("Encryption failed: {0}")]
    Encryption(String),

    /// Decryption operation failed
    #[error("Decryption failed: invalid ciphertext or wrong key")]
    Decryption,

    /// Authentication tag mismatch (message may be tampered)
    #[error("Authentication tag mismatch: message may be tampered")]
    AuthenticationFailed,

    /// Secure channel establishment failed
    #[error("Secure channel establishment failed: {0}")]
    ChannelEstablishment(String),

    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
}

/// Errors that can occur during VDFS operations
#[derive(Debug, Error)]
pub enum VdfsError {
    /// File not found
    #[error("File not found: {0}")]
    NotFound(PathBuf),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// Content hash mismatch
    #[error("Content hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    /// Sync conflict detected
    #[error("Sync conflict at {path}: local={local}, remote={remote}")]
    SyncConflict {
        path: PathBuf,
        local: String,
        remote: String,
    },

    /// Peer not connected
    #[error("Peer not connected: {0}")]
    PeerNotConnected(String),

    /// Generic I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Chunk not found
    #[error("Chunk not found: {0}")]
    ChunkNotFound(String),
}

/// Errors that can occur during reconnection
#[derive(Debug, Error)]
pub enum ReconnectionError {
    /// All reconnection attempts exhausted
    #[error("All {attempts} reconnection attempts failed. Last error: {last_error}")]
    AttemptsExhausted { attempts: u32, last_error: String },

    /// Reconnection was cancelled
    #[error("Reconnection cancelled by user")]
    Cancelled,

    /// Connection error during reconnection
    #[error("Connection error during reconnection: {0}")]
    Connection(#[from] ConnectionError),
}

/// Errors that can occur during session operations
#[derive(Debug, Error)]
pub enum SessionError {
    /// Session not found
    #[error("Session not found: {0}")]
    NotFound(String),

    /// Profile not found
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    /// Profile already exists
    #[error("Profile already exists: {0}")]
    ProfileExists(String),

    /// SSH error
    #[error("SSH error: {0}")]
    Ssh(#[from] SshError),

    /// I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Errors that can occur during port forwarding
#[derive(Debug, Error)]
pub enum ForwardError {
    /// Failed to bind local port
    #[error("Failed to bind local port {port}: {reason}")]
    BindFailed { port: u16, reason: String },

    /// Failed to connect to remote destination
    #[error("Failed to connect to {host}:{port}: {reason}")]
    RemoteConnectFailed {
        host: String,
        port: u16,
        reason: String,
    },

    /// Forward not found
    #[error("Forward not found: {0}")]
    NotFound(String),

    /// SSH error
    #[error("SSH error: {0}")]
    Ssh(#[from] SshError),

    /// I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur during P2P operations
#[derive(Debug, Error)]
pub enum P2PError {
    /// Failed to connect to peer
    #[error("Failed to connect to peer {peer_id}: {reason}")]
    ConnectionFailed { peer_id: String, reason: String },

    /// Peer not found
    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    /// Stream error
    #[error("Stream error: {0}")]
    Stream(String),

    /// Relay connection failed
    #[error("Relay connection failed: {0}")]
    RelayFailed(String),

    /// NAT traversal failed
    #[error("NAT traversal failed: {0}")]
    NatTraversalFailed(String),
}

/// Errors that can occur during streaming operations
#[derive(Debug, Error)]
pub enum StreamError {
    /// Stream not found
    #[error("Stream not found: {0}")]
    NotFound(String),

    /// Seek position out of bounds
    #[error("Seek position {position} out of bounds (file size: {size})")]
    SeekOutOfBounds { position: u64, size: u64 },

    /// Buffer underrun
    #[error("Buffer underrun: not enough data buffered")]
    BufferUnderrun,

    /// I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
}

impl ConnectionError {
    /// Check if this error has a descriptive message
    pub fn is_descriptive(&self) -> bool {
        !self.to_string().is_empty()
    }

    /// Get the error reason/description
    pub fn reason(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_error_is_descriptive() {
        let errors = vec![
            ConnectionError::Timeout(Duration::from_secs(30)),
            ConnectionError::DnsResolution {
                host: "example.com".to_string(),
                reason: "NXDOMAIN".to_string(),
            },
            ConnectionError::ConnectionRefused {
                host: "localhost".to_string(),
                port: 22,
            },
            ConnectionError::NetworkUnreachable("No route to host".to_string()),
            ConnectionError::TlsHandshake("Certificate expired".to_string()),
            ConnectionError::ConnectionClosed("Remote closed".to_string()),
            ConnectionError::InvalidConfig("Missing host".to_string()),
        ];

        for error in errors {
            assert!(error.is_descriptive(), "Error should be descriptive: {:?}", error);
            assert!(!error.reason().is_empty(), "Error reason should not be empty: {:?}", error);
        }
    }
}
