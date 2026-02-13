//! Iroh Endpoint initialization and management
//!
//! This module provides the P2P endpoint using Iroh's QUIC transport.
//!
//! # Requirements Coverage
//! - Requirement 3.1: Iroh QUIC implementation for transport
//! - Requirement 3.3: Relay server configuration for fallback

use crate::error::P2PError;
use iroh::{Endpoint, NodeId, RelayMode, SecretKey};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Application-Level Protocol Negotiation identifier for russh
pub const RUSSH_ALPN: &[u8] = b"russh/1";

/// Configuration for the P2P endpoint
#[derive(Debug, Clone)]
pub struct P2PConfig {
    /// Secret key for node identity (generated if not provided)
    pub secret_key: Option<SecretKey>,
    /// Relay mode configuration
    pub relay_mode: P2PRelayMode,
    /// Enable n0 DNS discovery
    pub enable_discovery: bool,
    /// Custom ALPN protocols (russh/1 is always included)
    pub alpns: Vec<Vec<u8>>,
}

/// Relay mode configuration
#[derive(Debug, Clone, Default)]
pub enum P2PRelayMode {
    /// Use default n0 relay servers
    #[default]
    Default,
    /// Disable relay (direct connections only)
    Disabled,
    /// Custom relay configuration (URLs)
    Custom(Vec<String>),
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            secret_key: None,
            relay_mode: P2PRelayMode::Default,
            enable_discovery: true,
            alpns: vec![RUSSH_ALPN.to_vec()],
        }
    }
}

impl P2PConfig {
    /// Create a new P2P config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a specific secret key for node identity
    pub fn with_secret_key(mut self, key: SecretKey) -> Self {
        self.secret_key = Some(key);
        self
    }

    /// Set relay mode
    pub fn with_relay_mode(mut self, mode: P2PRelayMode) -> Self {
        self.relay_mode = mode;
        self
    }

    /// Enable or disable discovery
    pub fn with_discovery(mut self, enabled: bool) -> Self {
        self.enable_discovery = enabled;
        self
    }

    /// Add additional ALPN protocols
    pub fn with_alpn(mut self, alpn: Vec<u8>) -> Self {
        if !self.alpns.contains(&alpn) {
            self.alpns.push(alpn);
        }
        self
    }
}

/// P2P Endpoint wrapper around Iroh's Endpoint
///
/// Provides a high-level interface for P2P networking with automatic
/// relay fallback and NAT traversal.
pub struct P2PEndpoint {
    /// The underlying Iroh endpoint
    endpoint: Endpoint,
    /// Our node ID
    node_id: NodeId,
    /// Configuration used to create this endpoint
    #[allow(dead_code)]
    config: P2PConfig,
    /// Whether the endpoint is online (connected to relay)
    online: Arc<RwLock<bool>>,
}

impl P2PEndpoint {
    /// Create and bind a new P2P endpoint
    ///
    /// # Requirements Coverage
    /// - Requirement 3.1: Creates Iroh Endpoint with QUIC transport
    /// - Requirement 3.3: Configures relay servers for fallback
    pub async fn bind(config: P2PConfig) -> Result<Self, P2PError> {
        let secret_key = config
            .secret_key
            .clone()
            .unwrap_or_else(|| SecretKey::generate(rand::rngs::OsRng));
        let node_id = secret_key.public();

        let mut builder = Endpoint::builder()
            .secret_key(secret_key)
            .alpns(config.alpns.clone());

        // Configure relay mode
        builder = match &config.relay_mode {
            P2PRelayMode::Default => builder.relay_mode(RelayMode::Default),
            P2PRelayMode::Disabled => builder.relay_mode(RelayMode::Disabled),
            P2PRelayMode::Custom(urls) => {
                if urls.is_empty() {
                    tracing::warn!("Empty custom relay list, using default");
                    builder.relay_mode(RelayMode::Default)
                } else {
                    // Parse URLs and create relay map
                    let relay_urls: Vec<iroh::RelayUrl> = urls
                        .iter()
                        .filter_map(|url_str| {
                            url_str
                                .parse::<iroh::RelayUrl>()
                                .map_err(|e| {
                                    tracing::warn!("Invalid relay URL '{}': {}", url_str, e);
                                    e
                                })
                                .ok()
                        })
                        .collect();

                    if relay_urls.is_empty() {
                        tracing::warn!("No valid relay URLs, using default");
                        builder.relay_mode(RelayMode::Default)
                    } else {
                        tracing::info!("Using {} custom relay server(s)", relay_urls.len());
                        let relay_map =
                            iroh::RelayMap::from_nodes(relay_urls.into_iter().map(|url| {
                                iroh::RelayNode {
                                    url: url.clone(),
                                    stun_only: false,
                                    stun_port: 3478,
                                    quic: None,
                                }
                            }))
                            .map_err(|e| {
                                P2PError::ConnectionFailed {
                                    peer_id: node_id.to_string(),
                                    reason: format!("Failed to create relay map: {}", e),
                                }
                            })?;
                        builder.relay_mode(RelayMode::Custom(relay_map))
                    }
                }
            }
        };

        // Configure discovery
        if config.enable_discovery {
            builder = builder.discovery_n0();
        }

        let endpoint = builder
            .bind()
            .await
            .map_err(|e| P2PError::ConnectionFailed {
                peer_id: node_id.to_string(),
                reason: format!("Failed to bind endpoint: {}", e),
            })?;

        tracing::info!(
            node_id = %node_id,
            "P2P endpoint created"
        );

        Ok(Self {
            endpoint,
            node_id,
            config,
            online: Arc::new(RwLock::new(false)),
        })
    }

    /// Wait for the endpoint to be online (connected to relay)
    pub async fn wait_online(&self) {
        // Check if we have a home relay
        let watcher = self.endpoint.home_relay();
        if watcher.get().ok().flatten().is_some() {
            *self.online.write().await = true;
            tracing::info!(node_id = %self.node_id, "P2P endpoint online");
        }
    }

    /// Check if the endpoint is online
    pub async fn is_online(&self) -> bool {
        *self.online.read().await
    }

    /// Get our node ID
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Get the underlying Iroh endpoint
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Get our node address for sharing with peers
    pub async fn node_addr(&self) -> Result<iroh::NodeAddr, P2PError> {
        self.endpoint
            .node_addr()
            .await
            .map_err(|e| P2PError::ConnectionFailed {
                peer_id: self.node_id.to_string(),
                reason: format!("Failed to get node address: {}", e),
            })
    }

    /// Get direct addresses (if any)
    pub fn direct_addresses(&self) -> Vec<std::net::SocketAddr> {
        // let watcher = self.endpoint.direct_addresses();
        // watcher.get()
        //     .map(|addrs| addrs.into_iter().map(|da| da.clone().into()).collect())
        //     .unwrap_or_default()
        vec![]
    }

    /// Get the relay URL (if connected to a relay)
    pub fn relay_url(&self) -> Option<iroh::RelayUrl> {
        self.endpoint.home_relay().get().ok().flatten()
    }

    /// Close the endpoint gracefully
    pub async fn close(self) {
        tracing::info!(node_id = %self.node_id, "Closing P2P endpoint");
        self.endpoint.close().await;
    }
}

/// Builder for P2P endpoints
pub struct P2PEndpointBuilder {
    config: P2PConfig,
}

impl P2PEndpointBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: P2PConfig::default(),
        }
    }

    /// Set a specific secret key
    pub fn secret_key(mut self, key: SecretKey) -> Self {
        self.config.secret_key = Some(key);
        self
    }

    /// Use default relay servers
    pub fn relay_default(mut self) -> Self {
        self.config.relay_mode = P2PRelayMode::Default;
        self
    }

    /// Disable relay (direct connections only)
    pub fn relay_disabled(mut self) -> Self {
        self.config.relay_mode = P2PRelayMode::Disabled;
        self
    }

    /// Enable n0 DNS discovery
    pub fn discovery(mut self, enabled: bool) -> Self {
        self.config.enable_discovery = enabled;
        self
    }

    /// Add an ALPN protocol
    pub fn alpn(mut self, alpn: Vec<u8>) -> Self {
        if !self.config.alpns.contains(&alpn) {
            self.config.alpns.push(alpn);
        }
        self
    }

    /// Build and bind the endpoint
    pub async fn bind(self) -> Result<P2PEndpoint, P2PError> {
        P2PEndpoint::bind(self.config).await
    }
}

impl Default for P2PEndpointBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p2p_config_default() {
        let config = P2PConfig::default();
        assert!(config.secret_key.is_none());
        assert!(config.enable_discovery);
        assert!(config.alpns.contains(&RUSSH_ALPN.to_vec()));
    }

    #[test]
    fn p2p_config_builder() {
        let key = SecretKey::generate(rand::rngs::OsRng);
        let config = P2PConfig::new()
            .with_secret_key(key.clone())
            .with_discovery(false)
            .with_relay_mode(P2PRelayMode::Disabled);

        assert!(config.secret_key.is_some());
        assert!(!config.enable_discovery);
        assert!(matches!(config.relay_mode, P2PRelayMode::Disabled));
    }

    #[tokio::test]
    async fn endpoint_builder_creates_config() {
        // Just test the builder pattern, not actual binding
        let builder = P2PEndpointBuilder::new()
            .relay_default()
            .discovery(true)
            .alpn(b"test/1".to_vec());

        assert!(builder.config.enable_discovery);
        assert!(builder.config.alpns.contains(&b"test/1".to_vec()));
    }
}
