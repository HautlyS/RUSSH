//! Connection Manager implementation
//!
//! This module provides async connection management using Tokio,
//! including TCP connection establishment with timeout and keepalive.
//!
//! # Features
//! - Async connection establishment using Tokio runtime
//! - TCP keepalive configuration to prevent timeout disconnections
//! - Automatic reconnection with exponential backoff
//! - Connection state tracking and broadcasting
//!
//! # Requirements Coverage
//! - Requirement 1.1: Async SSH connection establishment
//! - Requirement 1.5: TCP keepalive for idle connections

use crate::config::{ConnectionConfig, ReconnectionStrategy};
use crate::connection::state::{ConnectionState, StateManager, StateChangeEvent};
use crate::connection::reconnection::ReconnectionController;
use crate::error::{ConnectionError, ReconnectionError};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::time::timeout;

/// A managed TCP connection with state tracking
///
/// This struct wraps a TCP stream with additional metadata and utilities
/// for connection management. It provides access to the underlying stream
/// while tracking connection information.
///
/// # Requirements Coverage
/// - Requirement 1.1: Async SSH connection establishment
pub struct ManagedConnection {
    /// The underlying TCP stream
    stream: TcpStream,
    /// Remote address
    remote_addr: SocketAddr,
    /// Connection established timestamp
    connected_at: std::time::Instant,
}

impl ManagedConnection {
    /// Create a new managed connection
    fn new(stream: TcpStream, remote_addr: SocketAddr) -> Self {
        Self {
            stream,
            remote_addr,
            connected_at: std::time::Instant::now(),
        }
    }

    /// Get the remote address
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Get a reference to the underlying stream
    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }

    /// Get a mutable reference to the underlying stream
    pub fn stream_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    /// Consume and return the underlying stream
    pub fn into_stream(self) -> TcpStream {
        self.stream
    }

    /// Split the stream into read and write halves
    pub fn split(&mut self) -> (tokio::net::tcp::ReadHalf<'_>, tokio::net::tcp::WriteHalf<'_>) {
        self.stream.split()
    }

    /// Get the duration since connection was established
    pub fn uptime(&self) -> Duration {
        self.connected_at.elapsed()
    }

    /// Check if the connection is still alive by checking the stream
    pub fn is_alive(&self) -> bool {
        // Check if the peer address is still accessible
        self.stream.peer_addr().is_ok()
    }
}

/// Connection manager for establishing and managing TCP connections
///
/// The ConnectionManager provides async connection establishment using Tokio,
/// with support for automatic reconnection, state tracking, and TCP keepalive.
///
/// # Requirements Coverage
/// - Requirement 1.1: Async SSH connection establishment using Tokio runtime
/// - Requirement 1.5: TCP keepalive to prevent timeout disconnections
/// - Requirement 2.1-2.5: Automatic reconnection with exponential backoff
pub struct ConnectionManager {
    /// Connection configuration
    config: ConnectionConfig,
    /// State manager for tracking connection state
    state_manager: Arc<StateManager>,
    /// Reconnection controller
    reconnection_controller: Arc<ReconnectionController>,
}

impl ConnectionManager {
    /// Create a new connection manager with the given configuration
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            config,
            state_manager: Arc::new(StateManager::new()),
            reconnection_controller: Arc::new(ReconnectionController::new()),
        }
    }

    /// Get the current connection state
    pub fn state(&self) -> ConnectionState {
        self.state_manager.state()
    }

    /// Subscribe to connection state changes
    ///
    /// Returns a broadcast receiver that will receive state change events
    /// whenever the connection state changes.
    pub fn subscribe_state_changes(&self) -> broadcast::Receiver<StateChangeEvent> {
        self.state_manager.subscribe()
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        self.state_manager.state().is_connected()
    }

    /// Check if reconnection is in progress
    pub fn is_reconnecting(&self) -> bool {
        self.reconnection_controller.is_reconnecting()
    }

    /// Get the connection configuration
    pub fn config(&self) -> &ConnectionConfig {
        &self.config
    }

    /// Establish a new connection to the configured host
    ///
    /// This method establishes an async TCP connection using Tokio,
    /// with timeout and keepalive configuration.
    ///
    /// # Requirements Coverage
    /// - Requirement 1.1: Async connection establishment
    /// - Requirement 1.4: Descriptive error on failure
    /// - Requirement 1.5: TCP keepalive configuration
    pub async fn connect(&self) -> Result<ManagedConnection, ConnectionError> {
        // Update state to connecting
        self.state_manager.set_state(ConnectionState::Connecting);

        match self.connect_internal().await {
            Ok(conn) => {
                self.state_manager.set_state(ConnectionState::Connected);
                tracing::info!(
                    host = %self.config.host,
                    port = %self.config.port,
                    "Connection established successfully"
                );
                Ok(conn)
            }
            Err(e) => {
                let reason = e.to_string();
                self.state_manager.set_state(ConnectionState::Failed {
                    reason: reason.clone(),
                });
                tracing::error!(
                    host = %self.config.host,
                    port = %self.config.port,
                    error = %reason,
                    "Connection failed"
                );
                Err(e)
            }
        }
    }

    /// Internal connection logic
    ///
    /// Performs DNS resolution, TCP connection with timeout, and keepalive setup.
    async fn connect_internal(&self) -> Result<ManagedConnection, ConnectionError> {
        // Resolve the address
        let addr = self.resolve_address().await?;
        tracing::debug!(address = %addr, "Resolved address");

        // Connect with timeout
        let stream = timeout(self.config.timeout, TcpStream::connect(addr))
            .await
            .map_err(|_| {
                tracing::warn!(
                    timeout = ?self.config.timeout,
                    "Connection timed out"
                );
                ConnectionError::Timeout(self.config.timeout)
            })?
            .map_err(|e| self.map_io_error(e, &addr))?;

        // Configure TCP keepalive to prevent idle disconnections
        self.configure_keepalive(&stream)?;
        tracing::debug!(
            keepalive_interval = ?self.config.keepalive_interval,
            "TCP keepalive configured"
        );

        // Configure TCP nodelay for lower latency
        stream.set_nodelay(true).map_err(ConnectionError::Io)?;

        Ok(ManagedConnection::new(stream, addr))
    }

    /// Resolve the host address
    ///
    /// Attempts to parse the address directly first, then falls back to DNS resolution.
    async fn resolve_address(&self) -> Result<SocketAddr, ConnectionError> {
        let addr_str = format!("{}:{}", self.config.host, self.config.port);

        // Try to parse as socket address first (for IP addresses)
        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
            return Ok(addr);
        }

        // Otherwise, perform DNS resolution
        let addrs: Vec<SocketAddr> = tokio::net::lookup_host(&addr_str)
            .await
            .map_err(|e| {
                tracing::warn!(
                    host = %self.config.host,
                    error = %e,
                    "DNS resolution failed"
                );
                ConnectionError::DnsResolution {
                    host: self.config.host.clone(),
                    reason: e.to_string(),
                }
            })?
            .collect();

        addrs.into_iter().next().ok_or_else(|| {
            tracing::warn!(host = %self.config.host, "No addresses found for host");
            ConnectionError::DnsResolution {
                host: self.config.host.clone(),
                reason: "No addresses found".to_string(),
            }
        })
    }

    /// Configure TCP keepalive on the stream
    ///
    /// This prevents idle connections from being dropped by intermediate
    /// network devices (firewalls, NAT, etc.).
    ///
    /// # Requirements Coverage
    /// - Requirement 1.5: TCP keepalive for idle connections
    fn configure_keepalive(&self, stream: &TcpStream) -> Result<(), ConnectionError> {
        let socket = socket2::SockRef::from(stream);

        let keepalive = socket2::TcpKeepalive::new()
            .with_time(self.config.keepalive_interval);

        socket.set_tcp_keepalive(&keepalive)
            .map_err(|e| {
                tracing::warn!(error = %e, "Failed to configure TCP keepalive");
                ConnectionError::Io(e.into())
            })?;

        Ok(())
    }

    /// Map IO errors to more specific connection errors
    ///
    /// Provides descriptive error messages for common failure scenarios.
    ///
    /// # Requirements Coverage
    /// - Requirement 1.4: Descriptive error with failure reason
    fn map_io_error(&self, error: std::io::Error, addr: &SocketAddr) -> ConnectionError {
        match error.kind() {
            std::io::ErrorKind::ConnectionRefused => {
                tracing::warn!(
                    host = %addr.ip(),
                    port = %addr.port(),
                    "Connection refused"
                );
                ConnectionError::ConnectionRefused {
                    host: addr.ip().to_string(),
                    port: addr.port(),
                }
            }
            std::io::ErrorKind::NetworkUnreachable => {
                tracing::warn!(error = %error, "Network unreachable");
                ConnectionError::NetworkUnreachable(error.to_string())
            }
            std::io::ErrorKind::TimedOut => {
                tracing::warn!(timeout = ?self.config.timeout, "Connection timed out");
                ConnectionError::Timeout(self.config.timeout)
            }
            std::io::ErrorKind::HostUnreachable => {
                tracing::warn!(host = %addr.ip(), "Host unreachable");
                ConnectionError::NetworkUnreachable(format!("Host unreachable: {}", addr.ip()))
            }
            _ => {
                tracing::warn!(error = %error, "IO error during connection");
                ConnectionError::Io(error)
            }
        }
    }

    /// Attempt to reconnect with the configured strategy
    ///
    /// Uses the default reconnection strategy based on the connection config.
    ///
    /// # Requirements Coverage
    /// - Requirement 2.1: Automatic reconnection with exponential backoff
    /// - Requirement 2.2: Configurable maximum attempts
    pub async fn reconnect(&self) -> Result<ManagedConnection, ReconnectionError> {
        let strategy = ReconnectionStrategy::new(
            self.config.max_reconnect_attempts,
            self.config.reconnect_base_delay,
            Duration::from_secs(60), // Max delay of 60 seconds
        );

        self.reconnect_with_strategy(&strategy).await
    }

    /// Attempt to reconnect with a custom strategy
    ///
    /// # Requirements Coverage
    /// - Requirement 2.1: Automatic reconnection with exponential backoff
    /// - Requirement 2.2: Configurable maximum attempts
    /// - Requirement 2.3: Session state restoration (handled at higher level)
    /// - Requirement 2.4: User notification of reconnection status
    /// - Requirement 2.5: Manual reconnection option on failure
    pub async fn reconnect_with_strategy(
        &self,
        strategy: &ReconnectionStrategy,
    ) -> Result<ManagedConnection, ReconnectionError> {
        tracing::info!(
            host = %self.config.host,
            port = %self.config.port,
            max_attempts = %strategy.max_attempts,
            "Starting reconnection"
        );

        let state_manager = self.state_manager.clone();
        let config = self.config.clone();

        self.reconnection_controller
            .reconnect(strategy, || {
                let sm = state_manager.clone();
                let attempt = self.reconnection_controller.current_attempt();
                let cfg = config.clone();
                async move {
                    sm.set_state(ConnectionState::Reconnecting { attempt });
                    tracing::info!(
                        attempt = %attempt,
                        host = %cfg.host,
                        "Reconnection attempt"
                    );
                    self.connect_internal().await
                }
            })
            .await
            .map(|conn| {
                self.state_manager.set_state(ConnectionState::Connected);
                tracing::info!(
                    host = %self.config.host,
                    "Reconnection successful"
                );
                conn
            })
            .map_err(|e| {
                let reason = e.to_string();
                self.state_manager.set_state(ConnectionState::Failed {
                    reason: reason.clone(),
                });
                tracing::error!(
                    host = %self.config.host,
                    error = %reason,
                    "Reconnection failed"
                );
                e
            })
    }

    /// Cancel any ongoing reconnection attempts
    ///
    /// # Requirements Coverage
    /// - Requirement 2.5: Manual reconnection option (cancel allows manual control)
    pub fn cancel_reconnection(&self) {
        tracing::info!("Cancelling reconnection");
        self.reconnection_controller.cancel_reconnection();
    }

    /// Mark the connection as disconnected
    pub fn disconnect(&self) {
        tracing::info!(
            host = %self.config.host,
            "Disconnecting"
        );
        self.state_manager.set_state(ConnectionState::Disconnected);
    }

    /// Get the state manager for external state monitoring
    pub fn state_manager(&self) -> Arc<StateManager> {
        self.state_manager.clone()
    }
}

/// Builder for creating ConnectionManager instances
pub struct ConnectionManagerBuilder {
    config: ConnectionConfig,
}

impl ConnectionManagerBuilder {
    /// Create a new builder with the given host and port
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            config: ConnectionConfig::new(host, port),
        }
    }

    /// Set the connection timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the keepalive interval
    pub fn keepalive_interval(mut self, interval: Duration) -> Self {
        self.config.keepalive_interval = interval;
        self
    }

    /// Set the maximum reconnection attempts
    pub fn max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.config.max_reconnect_attempts = attempts;
        self
    }

    /// Set the base delay for reconnection backoff
    pub fn reconnect_base_delay(mut self, delay: Duration) -> Self {
        self.config.reconnect_base_delay = delay;
        self
    }

    /// Build the ConnectionManager
    pub fn build(self) -> ConnectionManager {
        ConnectionManager::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_manager_builder() {
        let manager = ConnectionManagerBuilder::new("localhost", 22)
            .timeout(Duration::from_secs(10))
            .keepalive_interval(Duration::from_secs(30))
            .max_reconnect_attempts(3)
            .build();

        assert_eq!(manager.config().host, "localhost");
        assert_eq!(manager.config().port, 22);
        assert_eq!(manager.config().timeout, Duration::from_secs(10));
        assert_eq!(manager.config().keepalive_interval, Duration::from_secs(30));
        assert_eq!(manager.config().max_reconnect_attempts, 3);
    }

    #[test]
    fn connection_manager_initial_state() {
        let manager = ConnectionManager::new(ConnectionConfig::new("localhost", 22));
        assert_eq!(manager.state(), ConnectionState::Disconnected);
        assert!(!manager.is_connected());
        assert!(!manager.is_reconnecting());
    }

    #[tokio::test]
    async fn connection_manager_state_subscription() {
        let manager = ConnectionManager::new(ConnectionConfig::new("localhost", 22));
        let mut receiver = manager.subscribe_state_changes();

        // Manually set state for testing
        manager.state_manager.set_state(ConnectionState::Connecting);

        let event = receiver.recv().await.unwrap();
        assert_eq!(event.old_state, ConnectionState::Disconnected);
        assert_eq!(event.new_state, ConnectionState::Connecting);
    }

    #[tokio::test]
    async fn connection_to_invalid_host_fails() {
        let manager = ConnectionManagerBuilder::new("invalid.host.that.does.not.exist.local", 22)
            .timeout(Duration::from_millis(100))
            .build();

        let result = manager.connect().await;
        assert!(result.is_err());
        assert!(manager.state().is_failed());
        
        // Verify error is descriptive (Requirement 1.4)
        let error = result.unwrap_err();
        assert!(!error.to_string().is_empty());
    }

    #[tokio::test]
    async fn connection_refused_error() {
        // Try to connect to a port that's likely not listening
        let manager = ConnectionManagerBuilder::new("127.0.0.1", 59999)
            .timeout(Duration::from_secs(1))
            .build();

        let result = manager.connect().await;
        assert!(result.is_err());

        // Should be either ConnectionRefused or Timeout depending on OS behavior
        let error = result.unwrap_err();
        match error {
            ConnectionError::ConnectionRefused { host, port } => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 59999);
            }
            ConnectionError::Timeout(_) => {}
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn managed_connection_uptime() {
        // This test verifies the uptime tracking works
        // We can't easily test with a real connection, but we can verify the struct
        let config = ConnectionConfig::new("localhost", 22);
        let _manager = ConnectionManager::new(config);
        // The actual uptime test would require a real connection
    }

    #[test]
    fn disconnect_sets_state() {
        let manager = ConnectionManager::new(ConnectionConfig::new("localhost", 22));
        manager.state_manager.set_state(ConnectionState::Connected);
        assert!(manager.is_connected());
        
        manager.disconnect();
        assert_eq!(manager.state(), ConnectionState::Disconnected);
    }
}
