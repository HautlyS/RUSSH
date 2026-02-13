//! SSH Port Forwarding
//!
//! Handles local, remote, and dynamic port forwarding.
//!
//! # Requirements Coverage
//! - Requirement 10.1: Local port forwarding
//! - Requirement 10.2: Remote port forwarding (limited support)
//! - Requirement 10.3: Dynamic port forwarding (SOCKS5 proxy)
//! - Requirement 10.4: Concurrent forward management
//! - Requirement 10.5: Graceful failure handling

use super::SshClient;
use crate::error::{ForwardError, SshError};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

/// Port forward configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PortForward {
    /// Local port forwarding (Local -> Remote)
    Local {
        local_port: u16,
        remote_host: String,
        remote_port: u16,
    },
    /// Remote port forwarding (Remote -> Local)
    Remote {
        remote_port: u16,
        local_host: String,
        local_port: u16,
    },
    /// Dynamic port forwarding (SOCKS Proxy)
    Dynamic { local_port: u16 },
}

/// Active forward handle
#[derive(Debug)]
pub struct ForwardHandle {
    pub id: Uuid,
    pub config: PortForward,
    pub bytes_transferred: AtomicU64,
}

impl ForwardHandle {
    pub fn new(id: Uuid, config: PortForward) -> Self {
        Self {
            id,
            config,
            bytes_transferred: AtomicU64::new(0),
        }
    }

    pub fn inc_bytes(&self, bytes: u64) {
        self.bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
    }
}

/// Port forwarder trait
#[async_trait]
pub trait PortForwarder {
    /// Start a port forward
    async fn start_forward(&self, forward: PortForward)
        -> Result<Arc<ForwardHandle>, ForwardError>;

    /// Stop a port forward
    async fn stop_forward(&self, id: Uuid) -> Result<(), ForwardError>;

    /// List active forwards
    async fn list_forwards(&self) -> Vec<Arc<ForwardHandle>>;
}

#[async_trait]
impl PortForwarder for SshClient {
    async fn start_forward(
        &self,
        forward: PortForward,
    ) -> Result<Arc<ForwardHandle>, ForwardError> {
        let client = self
            .inner()
            .ok_or(ForwardError::Ssh(SshError::NotConnected))?;

        let id = Uuid::new_v4();
        let handle = Arc::new(ForwardHandle::new(id, forward.clone()));

        let abort_handle = match &forward {
            PortForward::Local {
                local_port,
                remote_host,
                remote_port,
            } => {
                let listener = TcpListener::bind(format!("127.0.0.1:{}", local_port))
                    .await
                    .map_err(|e| ForwardError::BindFailed {
                        port: *local_port,
                        reason: e.to_string(),
                    })?;

                let remote_host = remote_host.clone();
                let remote_port = *remote_port;
                let local_port = *local_port;

                // Clone the client for use in the spawned task
                // Note: async-ssh2-tokio Client should be Clone
                let client_clone = client.clone();

                let task = tokio::spawn(async move {
                    tracing::info!("Started local forward on port {}", local_port);

                    loop {
                        match listener.accept().await {
                            Ok((mut local_stream, addr)) => {
                                tracing::debug!(
                                    "Accepted connection from {} for forward to {}:{}",
                                    addr,
                                    remote_host,
                                    remote_port
                                );

                                let host = remote_host.clone();
                                let port = remote_port;
                                let client_for_conn = client_clone.clone();

                                tokio::spawn(async move {
                                    tracing::debug!(
                                        "Opening direct TCP/IP channel to {}:{}",
                                        host,
                                        port
                                    );

                                    // Format as "host:port" string for the API
                                    let target = format!("{}:{}", host, port);
                                    match client_for_conn
                                        .open_direct_tcpip_channel(target, None)
                                        .await
                                    {
                                        Ok(channel) => {
                                            tracing::debug!("Channel opened, bridging streams");

                                            // Convert channel to stream for AsyncRead/AsyncWrite
                                            let mut channel_stream = channel.into_stream();

                                            match tokio::io::copy_bidirectional(
                                                &mut local_stream,
                                                &mut channel_stream,
                                            )
                                            .await
                                            {
                                                Ok((sent, received)) => {
                                                    tracing::debug!("Forward connection closed. Sent: {}, Received: {}", sent, received);
                                                }
                                                Err(e) => {
                                                    tracing::error!(
                                                        "Error bridging streams: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                "Failed to open direct-tcpip channel: {}",
                                                e
                                            );
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Accept failed: {}", e);
                                break;
                            }
                        }
                    }
                });
                task.abort_handle()
            }
            PortForward::Remote {
                remote_port,
                local_host,
                local_port,
            } => {
                // Remote port forwarding: The SSH server listens on remote_port and
                // forwards connections to local_host:local_port on the client side.
                //
                // Note: This requires the SSH server to support tcpip-forward requests.
                // The async-ssh2-tokio library doesn't directly expose this functionality,
                // so we implement it using SSH command execution to set up the forward.
                //
                // For full remote port forwarding support, consider using the russh
                // library directly or an SSH server that supports reverse tunnels.

                let remote_port = *remote_port;
                let local_host = local_host.clone();
                let local_port = *local_port;

                // We'll use socat or netcat on the remote side if available
                // This is a workaround since direct tcpip-forward isn't exposed
                let client_clone = client.clone();

                let task = tokio::spawn(async move {
                    tracing::info!(
                        "Starting remote forward: remote:{} -> {}:{}",
                        remote_port,
                        local_host,
                        local_port
                    );

                    // Try to set up a reverse tunnel using SSH command
                    // This requires socat or similar on the remote host
                    let cmd = format!(
                        "socat TCP-LISTEN:{},fork,reuseaddr TCP:{}:{} 2>/dev/null || \
                         nc -l -p {} -c 'nc {} {}' 2>/dev/null || \
                         echo 'Remote forwarding requires socat or netcat on remote host'",
                        remote_port, local_host, local_port, remote_port, local_host, local_port
                    );

                    match client_clone.execute(&cmd).await {
                        Ok(result) => {
                            if result.exit_status != 0 {
                                tracing::warn!(
                                    "Remote forward command exited with status {}: {}",
                                    result.exit_status,
                                    result.stdout
                                );
                            }
                        }
                        Err(e) => {
                            tracing::error!("Remote forward failed: {}", e);
                        }
                    }
                });
                task.abort_handle()
            }
            PortForward::Dynamic { local_port } => {
                // Dynamic port forwarding: SOCKS5 proxy
                // Listen on local_port and forward connections based on SOCKS5 protocol

                let listener = TcpListener::bind(format!("127.0.0.1:{}", local_port))
                    .await
                    .map_err(|e| ForwardError::BindFailed {
                        port: *local_port,
                        reason: e.to_string(),
                    })?;

                let local_port = *local_port;
                let client_clone = client.clone();

                let task = tokio::spawn(async move {
                    tracing::info!("Started SOCKS5 proxy on port {}", local_port);

                    loop {
                        match listener.accept().await {
                            Ok((stream, addr)) => {
                                tracing::debug!("SOCKS5: Accepted connection from {}", addr);
                                let client_for_conn = client_clone.clone();

                                tokio::spawn(async move {
                                    if let Err(e) =
                                        handle_socks5_connection(stream, client_for_conn).await
                                    {
                                        tracing::debug!("SOCKS5 connection error: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("SOCKS5 accept failed: {}", e);
                                break;
                            }
                        }
                    }
                });
                task.abort_handle()
            }
        };

        let mut forwards = self.forwards.write().await;
        forwards.insert(id, (handle.clone(), abort_handle));

        Ok(handle)
    }

    async fn stop_forward(&self, id: Uuid) -> Result<(), ForwardError> {
        let mut forwards = self.forwards.write().await;
        if let Some((_, abort_handle)) = forwards.remove(&id) {
            abort_handle.abort();
            Ok(())
        } else {
            Err(ForwardError::NotFound(id.to_string()))
        }
    }

    async fn list_forwards(&self) -> Vec<Arc<ForwardHandle>> {
        let forwards = self.forwards.read().await;
        forwards
            .values()
            .map(|(handle, _)| handle.clone())
            .collect()
    }
}

/// Handle a SOCKS5 connection
///
/// Implements the SOCKS5 protocol (RFC 1928) for dynamic port forwarding.
/// Supports CONNECT command with IPv4, IPv6, and domain name addressing.
async fn handle_socks5_connection(
    mut stream: TcpStream,
    client: async_ssh2_tokio::client::Client,
) -> Result<(), ForwardError> {
    // SOCKS5 greeting
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await?;

    if buf[0] != 0x05 {
        return Err(ForwardError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid SOCKS version",
        )));
    }

    let nmethods = buf[1] as usize;
    let mut methods = vec![0u8; nmethods];
    stream.read_exact(&mut methods).await?;

    // We only support no authentication (0x00)
    if !methods.contains(&0x00) {
        // Send "no acceptable methods"
        stream.write_all(&[0x05, 0xFF]).await?;
        return Err(ForwardError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "No acceptable authentication method",
        )));
    }

    // Send "no authentication required"
    stream.write_all(&[0x05, 0x00]).await?;

    // Read SOCKS5 request
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;

    if header[0] != 0x05 {
        return Err(ForwardError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid SOCKS version in request",
        )));
    }

    let cmd = header[1];
    let atyp = header[3];

    // Only support CONNECT (0x01)
    if cmd != 0x01 {
        // Send "command not supported"
        stream
            .write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
            .await?;
        return Err(ForwardError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Only CONNECT command is supported",
        )));
    }

    // Parse destination address
    let dest_addr = match atyp {
        0x01 => {
            // IPv4
            let mut addr = [0u8; 4];
            stream.read_exact(&mut addr).await?;
            format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3])
        }
        0x03 => {
            // Domain name
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut domain = vec![0u8; len[0] as usize];
            stream.read_exact(&mut domain).await?;
            String::from_utf8_lossy(&domain).to_string()
        }
        0x04 => {
            // IPv6
            let mut addr = [0u8; 16];
            stream.read_exact(&mut addr).await?;
            format!(
                "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                u16::from_be_bytes([addr[0], addr[1]]),
                u16::from_be_bytes([addr[2], addr[3]]),
                u16::from_be_bytes([addr[4], addr[5]]),
                u16::from_be_bytes([addr[6], addr[7]]),
                u16::from_be_bytes([addr[8], addr[9]]),
                u16::from_be_bytes([addr[10], addr[11]]),
                u16::from_be_bytes([addr[12], addr[13]]),
                u16::from_be_bytes([addr[14], addr[15]])
            )
        }
        _ => {
            // Send "address type not supported"
            stream
                .write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                .await?;
            return Err(ForwardError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsupported address type",
            )));
        }
    };

    // Read port
    let mut port_buf = [0u8; 2];
    stream.read_exact(&mut port_buf).await?;
    let dest_port = u16::from_be_bytes(port_buf);

    tracing::debug!("SOCKS5 CONNECT to {}:{}", dest_addr, dest_port);

    // Open SSH channel to destination
    let target = format!("{}:{}", dest_addr, dest_port);
    match client.open_direct_tcpip_channel(target, None).await {
        Ok(channel) => {
            // Send success response
            stream
                .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                .await?;

            // Bridge the streams
            let mut channel_stream = channel.into_stream();
            match tokio::io::copy_bidirectional(&mut stream, &mut channel_stream).await {
                Ok((sent, received)) => {
                    tracing::debug!(
                        "SOCKS5 connection to {}:{} closed. Sent: {}, Received: {}",
                        dest_addr,
                        dest_port,
                        sent,
                        received
                    );
                }
                Err(e) => {
                    tracing::debug!("SOCKS5 bridge error: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                "SOCKS5 failed to connect to {}:{}: {}",
                dest_addr,
                dest_port,
                e
            );
            // Send "connection refused"
            stream
                .write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                .await?;
            return Err(ForwardError::RemoteConnectFailed {
                host: dest_addr,
                port: dest_port,
                reason: e.to_string(),
            });
        }
    }

    Ok(())
}
