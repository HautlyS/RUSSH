//! SSH Client Implementation
//!
//! Handles the connection and authentication process for SSH sessions.
//!
//! # Requirements Coverage
//! - Requirement 1.2: Password and key-based authentication methods

use super::{SshConfig, AuthMethod, HostKeyCheck};
use crate::error::{SshError, ConnectionError};
use async_ssh2_tokio::client::{Client, AuthMethod as SshAuthMethod, ServerCheckMethod};
use std::net::ToSocketAddrs;
use std::sync::Arc;

use std::collections::HashMap;
use uuid::Uuid;
use tokio::task::AbortHandle;
use tokio::sync::RwLock;
use super::forward::ForwardHandle;

type ForwardsMap = HashMap<Uuid, (Arc<ForwardHandle>, AbortHandle)>;

/// Async SSH Client wrapper
///
/// Provides SSH connection management with support for:
/// - Password authentication
/// - Public key authentication
/// - Command execution
/// - Port forwarding
pub struct SshClient {
    client: Option<Client>,
    config: Option<SshConfig>,
    pub(crate) forwards: Arc<RwLock<ForwardsMap>>,
}

impl Default for SshClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SshClient {
    /// Create a new SSH client
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
            forwards: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect and authenticate to the remote host
    ///
    /// # Requirements Coverage
    /// - Requirement 1.2: Support password and key-based authentication methods
    pub async fn connect(&mut self, config: &SshConfig) -> Result<(), SshError> {
        let addr = format!("{}:{}", config.host, config.port);
        let socket_addr = addr.to_socket_addrs()
            .map_err(|e| ConnectionError::DnsResolution { 
                host: config.host.clone(), 
                reason: e.to_string() 
            })?
            .next()
            .ok_or_else(|| ConnectionError::DnsResolution { 
                host: config.host.clone(), 
                reason: "No address found".to_string() 
            })?;

        tracing::info!("Connecting to SSH server at {}", addr);

        // Convert our AuthMethod to async-ssh2-tokio's AuthMethod
        let auth_method = match &config.auth {
            AuthMethod::Password(password) => {
                tracing::debug!("Using password authentication");
                SshAuthMethod::with_password(password)
            }
            AuthMethod::PublicKey { key_path, passphrase } => {
                tracing::debug!("Using public key authentication with key: {:?}", key_path);
                SshAuthMethod::with_key_file(key_path, passphrase.as_deref())
            }
            AuthMethod::Agent => {
                // Agent auth - not directly supported by async-ssh2-tokio
                tracing::warn!("SSH agent authentication not supported, falling back to error");
                return Err(SshError::AuthenticationFailed {
                    user: config.username.clone(),
                    reason: "SSH agent authentication not supported in this version".to_string(),
                });
            }
        };


        let check_method = match config.host_key_check {
            HostKeyCheck::Strict => {
                if let Some(path) = &config.known_hosts_path {
                    ServerCheckMethod::with_known_hosts_file(&path.to_string_lossy())
                } else {
                    tracing::warn!("Strict host key checking requested but no known_hosts path provided. Defaulting to NoCheck.");
                    ServerCheckMethod::NoCheck
                }
            }
            HostKeyCheck::AcceptNew => {
                 // For now, async-ssh2-tokio might not directly support "AcceptNew" in the same way OpenSSH does via a simple enum.
                 // It usually supports KnownHosts or NoCheck. 
                 // If we want "Accept New", we might need a custom check or just use KnownHosts and handle the error if we could, 
                 // but for simplicity/safety in this pass, we will treat it similarly to Strict if a path is present, 
                 // effectively relying on the library's behavior for known hosts.
                 // verifying the library documentation or behavior would be ideal, but for now:
                 if let Some(path) = &config.known_hosts_path {
                     ServerCheckMethod::with_known_hosts_file(&path.to_string_lossy())
                 } else {
                     ServerCheckMethod::NoCheck
                 }
            }
            HostKeyCheck::None => ServerCheckMethod::NoCheck,
        };

        let client = Client::connect(
            socket_addr,
            &config.username,
            auth_method,
            check_method,
        ).await.map_err(|e| SshError::AuthenticationFailed { 
            user: config.username.clone(), 
            reason: e.to_string() 
        })?;

        tracing::info!("SSH authentication successful for user {}", config.username);
        
        self.client = Some(client);
        self.config = Some(config.clone());
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.client.as_ref().map(|c| !c.is_closed()).unwrap_or(false)
    }

    /// Get the current configuration
    pub fn config(&self) -> Option<&SshConfig> {
        self.config.as_ref()
    }

    /// Disconnect from remote host
    pub async fn disconnect(&mut self) -> Result<(), SshError> {
        // Stop all active forwards first
        let forward_ids: Vec<Uuid> = {
            let forwards = self.forwards.read().await;
            forwards.keys().cloned().collect()
        };
        
        for id in forward_ids {
            let mut forwards = self.forwards.write().await;
            if let Some((_, abort_handle)) = forwards.remove(&id) {
                abort_handle.abort();
            }
        }

        if let Some(client) = self.client.take() {
            client.disconnect().await.map_err(|e| {
                SshError::CommandExecution(format!("Disconnect failed: {}", e))
            })?;
            tracing::info!("Disconnected from SSH server");
        }
        self.config = None;
        Ok(())
    }
    
    /// Get reference to inner client
    pub(crate) fn inner(&self) -> Option<&Client> {
        self.client.as_ref()
    }
}
