//! SSH Client Module
//!
//! This module provides the SSH client functionality, including:
//! - Connection management
//! - Authentication (Password, Public Key, Agent)
//! - Command execution
//! - Interactive shell
//! - Port forwarding
//! - SFTP file operations
//!
//! # Requirements Coverage
//! - Requirement 1: Async SSH Connection Management
//! - Requirement 9: Command Execution
//! - Requirement 10: Port Forwarding

pub mod client;
pub mod command;
pub mod forward;
pub mod sftp;

pub use client::SshClient;
pub use command::{CommandResult, Shell};
pub use forward::{PortForward, PortForwarder};
pub use sftp::RemoteFileEntry;

use std::path::PathBuf;
use std::time::Duration;

/// SSH session configuration
#[derive(Debug, Clone)]
pub struct SshConfig {
    /// Remote host address
    pub host: String,
    /// Remote port (default: 22)
    pub port: u16,
    /// Username for authentication
    pub username: String,
    /// Authentication method
    pub auth: AuthMethod,
    /// Connection timeout
    pub timeout: Duration,
    /// Path to known_hosts file
    pub known_hosts_path: Option<PathBuf>,
    /// Host key check policy
    pub host_key_check: HostKeyCheck,
}

/// Host key checking policy
#[derive(Debug, Clone, Default)]
pub enum HostKeyCheck {
    /// Strict checking (reject unknown/changed keys)
    #[default]
    Strict,
    /// Accept new keys (add to known_hosts), reject changed
    AcceptNew,
    /// No checking (insecure)
    None,
}

/// SSH authentication method
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Password authentication
    Password(String),
    /// Public key authentication
    PublicKey {
        /// Path to private key file
        key_path: PathBuf,
        /// Optional passphrase for encrypted keys
        passphrase: Option<String>,
    },
    /// SSH Agent authentication
    Agent,
}
