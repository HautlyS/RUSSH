//! P2P Connection management
//!
//! This module handles peer connections with NAT traversal and relay fallback.
//!
//! # Requirements Coverage
//! - Requirement 3.2: NAT hole-punching for direct connections
//! - Requirement 3.3: Relay server fallback
//! - Requirement 3.5: Connection metadata (latency, type)

use crate::error::P2PError;
use crate::p2p::endpoint::{P2PEndpoint, RUSSH_ALPN};
use iroh::{endpoint::{Connection, ConnectionType as IrohConnectionType}, NodeAddr, NodeId};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Type of P2P connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    /// Direct connection via hole-punching
    Direct,
    /// Connection through relay server
    Relayed,
    /// Connection type unknown or mixed
    Unknown,
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionType::Direct => write!(f, "Direct"),
            ConnectionType::Relayed => write!(f, "Relayed"),
            ConnectionType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Metadata about a P2P connection
#[derive(Debug, Clone)]
pub struct P2PConnectionInfo {
    /// The peer's node ID
    pub peer_id: NodeId,
    /// Type of connection (direct or relayed)
    pub connection_type: ConnectionType,
    /// Measured round-trip latency
    pub latency: Option<Duration>,
    /// Remote address (if direct)
    pub remote_addr: Option<SocketAddr>,
    /// Relay URL (if relayed)
    pub relay_url: Option<iroh::RelayUrl>,
    /// When the connection was established
    pub connected_at: Instant,
}

impl P2PConnectionInfo {
    /// Create new connection info
    pub fn new(peer_id: NodeId) -> Self {
        Self {
            peer_id,
            connection_type: ConnectionType::Unknown,
            latency: None,
            remote_addr: None,
            relay_url: None,
            connected_at: Instant::now(),
        }
    }

    /// Get connection uptime
    pub fn uptime(&self) -> Duration {
        self.connected_at.elapsed()
    }
}

/// A managed P2P connection to a peer
pub struct P2PConnection {
    /// The underlying QUIC connection
    connection: Connection,
    /// The peer's node ID (cached)
    peer_id: NodeId,
    /// Connection metadata
    info: Arc<RwLock<P2PConnectionInfo>>,
    /// Reference to the endpoint for connection type queries
    endpoint: Arc<P2PEndpoint>,
}

impl P2PConnection {
    /// Create a new P2P connection wrapper
    fn new(connection: Connection, peer_id: NodeId, endpoint: Arc<P2PEndpoint>) -> Self {
        Self {
            connection,
            peer_id,
            info: Arc::new(RwLock::new(P2PConnectionInfo::new(peer_id))),
            endpoint,
        }
    }

    /// Get the peer's node ID
    pub fn peer_id(&self) -> NodeId {
        self.peer_id
    }

    /// Get the underlying QUIC connection
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Get connection info
    pub async fn info(&self) -> P2PConnectionInfo {
        self.info.read().await.clone()
    }

    /// Update connection type based on current state
    /// 
    /// Uses the Endpoint's conn_type() method to get the connection type
    /// for this peer, as Iroh tracks connection types at the endpoint level.
    pub async fn update_connection_type(&self) {
        let mut info = self.info.write().await;
        
        // Get connection type from the endpoint (Iroh tracks this at endpoint level)
        if let Ok(conn_type_watcher) = self.endpoint.endpoint().conn_type(self.peer_id) {
            if let Some(conn_type) = conn_type_watcher.get().ok() {
                info.connection_type = match conn_type {
                    IrohConnectionType::Direct(addr) => {
                        info.remote_addr = Some(addr);
                        ConnectionType::Direct
                    }
                    IrohConnectionType::Relay(url) => {
                        info.relay_url = Some(url);
                        ConnectionType::Relayed
                    }
                    IrohConnectionType::Mixed(addr, url) => {
                        info.remote_addr = Some(addr);
                        info.relay_url = Some(url);
                        ConnectionType::Direct // Prefer direct
                    }
                    IrohConnectionType::None => ConnectionType::Unknown,
                };
            }
        }
    }

    /// Measure and update latency
    pub async fn measure_latency(&self) -> Option<Duration> {
        let rtt = self.connection.rtt();
        let mut info = self.info.write().await;
        info.latency = Some(rtt);
        Some(rtt)
    }

    /// Close the connection gracefully
    pub fn close(&self, code: u32, reason: &[u8]) {
        self.connection.close(code.into(), reason);
    }

    /// Check if the connection is still alive
    pub fn is_alive(&self) -> bool {
        !self.connection.close_reason().is_some()
    }
}

/// P2P Connection Manager
///
/// Manages connections to peers with automatic NAT traversal and relay fallback.
/// Implements Drop to ensure proper cleanup of connections.
pub struct P2PConnectionManager {
    /// The P2P endpoint
    endpoint: Arc<P2PEndpoint>,
    /// Active connections
    connections: Arc<RwLock<std::collections::HashMap<NodeId, Arc<P2PConnection>>>>,
}

impl Drop for P2PConnectionManager {
    fn drop(&mut self) {
        // Synchronously close all connections on drop
        // Note: This is best-effort cleanup since we can't await in drop
        if let Ok(connections) = self.connections.try_write() {
            for (peer_id, conn) in connections.iter() {
                conn.close(0, b"manager_dropped");
                tracing::debug!(peer_id = %peer_id, "Connection closed on manager drop");
            }
        }
    }
}

impl P2PConnectionManager {
    /// Create a new connection manager
    pub fn new(endpoint: Arc<P2PEndpoint>) -> Self {
        Self {
            endpoint,
            connections: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Connect to a peer by NodeId
    ///
    /// Uses discovery to find the peer's address if not provided.
    ///
    /// # Requirements Coverage
    /// - Requirement 3.2: NAT hole-punching
    /// - Requirement 3.3: Relay fallback
    pub async fn connect(&self, peer_id: NodeId) -> Result<Arc<P2PConnection>, P2PError> {
        // Check if already connected
        {
            let connections = self.connections.read().await;
            if let Some(conn) = connections.get(&peer_id) {
                if conn.is_alive() {
                    return Ok(conn.clone());
                }
            }
        }

        tracing::info!(peer_id = %peer_id, "Connecting to peer");

        // Connect using discovery
        let connection = self.endpoint.endpoint()
            .connect(peer_id, RUSSH_ALPN)
            .await
            .map_err(|e| P2PError::ConnectionFailed {
                peer_id: peer_id.to_string(),
                reason: e.to_string(),
            })?;

        let p2p_conn = Arc::new(P2PConnection::new(connection, peer_id, self.endpoint.clone()));
        
        // Update connection info
        p2p_conn.update_connection_type().await;
        p2p_conn.measure_latency().await;

        let info = p2p_conn.info().await;
        tracing::info!(
            peer_id = %peer_id,
            connection_type = %info.connection_type,
            latency = ?info.latency,
            "Connected to peer"
        );

        // Store connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(peer_id, p2p_conn.clone());
        }

        Ok(p2p_conn)
    }

    /// Connect to a peer with explicit address information
    pub async fn connect_with_addr(&self, addr: NodeAddr) -> Result<Arc<P2PConnection>, P2PError> {
        let peer_id = addr.node_id;

        // Check if already connected
        {
            let connections = self.connections.read().await;
            if let Some(conn) = connections.get(&peer_id) {
                if conn.is_alive() {
                    return Ok(conn.clone());
                }
            }
        }

        tracing::info!(
            peer_id = %peer_id,
            relay = ?addr.relay_url,
            direct_addrs = ?addr.direct_addresses,
            "Connecting to peer with address"
        );

        let connection = self.endpoint.endpoint()
            .connect(addr, RUSSH_ALPN)
            .await
            .map_err(|e| P2PError::ConnectionFailed {
                peer_id: peer_id.to_string(),
                reason: e.to_string(),
            })?;

        let p2p_conn = Arc::new(P2PConnection::new(connection, peer_id, self.endpoint.clone()));
        p2p_conn.update_connection_type().await;
        p2p_conn.measure_latency().await;

        {
            let mut connections = self.connections.write().await;
            connections.insert(peer_id, p2p_conn.clone());
        }

        Ok(p2p_conn)
    }

    /// Get an existing connection to a peer
    pub async fn get_connection(&self, peer_id: &NodeId) -> Option<Arc<P2PConnection>> {
        let connections = self.connections.read().await;
        connections.get(peer_id).cloned()
    }

    /// Get connection info for a peer
    pub async fn connection_info(&self, peer_id: &NodeId) -> Option<P2PConnectionInfo> {
        let connections = self.connections.read().await;
        if let Some(conn) = connections.get(peer_id) {
            Some(conn.info().await)
        } else {
            None
        }
    }

    /// List all connected peers
    pub async fn connected_peers(&self) -> Vec<NodeId> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// Disconnect from a peer
    pub async fn disconnect(&self, peer_id: &NodeId) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.remove(peer_id) {
            conn.close(0, b"disconnect");
            tracing::info!(peer_id = %peer_id, "Disconnected from peer");
        }
    }

    /// Disconnect from all peers
    pub async fn disconnect_all(&self) {
        let mut connections = self.connections.write().await;
        for (peer_id, conn) in connections.drain() {
            conn.close(0, b"shutdown");
            tracing::info!(peer_id = %peer_id, "Disconnected from peer");
        }
    }

    /// Get our node ID
    pub fn local_node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_type_display() {
        assert_eq!(ConnectionType::Direct.to_string(), "Direct");
        assert_eq!(ConnectionType::Relayed.to_string(), "Relayed");
        assert_eq!(ConnectionType::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn connection_info_new() {
        let node_id = iroh::SecretKey::generate(rand::rngs::OsRng).public();
        let info = P2PConnectionInfo::new(node_id);
        
        assert_eq!(info.peer_id, node_id);
        assert_eq!(info.connection_type, ConnectionType::Unknown);
        assert!(info.latency.is_none());
        assert!(info.remote_addr.is_none());
        assert!(info.relay_url.is_none());
    }

    #[test]
    fn connection_info_uptime() {
        let node_id = iroh::SecretKey::generate(rand::rngs::OsRng).public();
        let info = P2PConnectionInfo::new(node_id);
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(info.uptime() >= std::time::Duration::from_millis(10));
    }
}
