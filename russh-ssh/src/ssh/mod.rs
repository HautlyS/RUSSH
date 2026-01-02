//! SSH Client Module
//!
//! This module provides the SSH client functionality, including:
//! - Connection management
//! - Authentication (Password, Public Key, Agent)
//! - Command execution
//! - Interactive shell
//! - Port forwarding
//!
//! # Requirements Coverage
//! - Requirement 1: Async SSH Connection Management
//! - Requirement 9: Command Execution
//! - Requirement 10: Port Forwarding

pub mod client;
pub mod command;
pub mod forward;

pub use client::SshClient;
pub use command::CommandResult;
pub use forward::{PortForward, PortForwarder};

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
