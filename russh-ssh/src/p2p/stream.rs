//! P2P Stream management
//!
//! This module handles bidirectional QUIC streams for P2P communication.
//!
//! # Requirements Coverage
//! - Requirement 3.4: Multiplexed bidirectional streams
//! - Requirement 3.5: Connection metadata (latency, type)

use crate::error::P2PError;
use crate::p2p::connection::{P2PConnection, P2PConnectionInfo};
use iroh::endpoint::{RecvStream, SendStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Statistics for a bidirectional stream
#[derive(Debug, Default)]
pub struct StreamStats {
    /// Bytes sent through this stream
    pub bytes_sent: AtomicU64,
    /// Bytes received through this stream
    pub bytes_received: AtomicU64,
}

impl StreamStats {
    /// Create new stream statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Get total bytes sent
    pub fn get_bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    /// Get total bytes received
    pub fn get_bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }

    /// Add to bytes sent counter
    pub fn add_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Add to bytes received counter
    pub fn add_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }
}

/// A bidirectional stream for P2P communication
pub struct BiStream {
    /// Send half of the stream
    send: SendStream,
    /// Receive half of the stream
    recv: RecvStream,
    /// Stream statistics
    stats: Arc<StreamStats>,
}

impl BiStream {
    /// Create a new bidirectional stream
    pub fn new(send: SendStream, recv: RecvStream) -> Self {
        Self {
            send,
            recv,
            stats: Arc::new(StreamStats::new()),
        }
    }

    /// Split into send and receive halves
    pub fn split(self) -> (SendStream, RecvStream) {
        (self.send, self.recv)
    }

    /// Get mutable reference to send stream
    pub fn send_mut(&mut self) -> &mut SendStream {
        &mut self.send
    }

    /// Get mutable reference to receive stream
    pub fn recv_mut(&mut self) -> &mut RecvStream {
        &mut self.recv
    }

    /// Get stream statistics
    pub fn stats(&self) -> &StreamStats {
        &self.stats
    }

    /// Write data to the stream
    pub async fn write(&mut self, data: &[u8]) -> Result<(), P2PError> {
        self.send
            .write_all(data)
            .await
            .map_err(|e| P2PError::Stream(format!("Write failed: {}", e)))?;
        self.stats.add_bytes_sent(data.len() as u64);
        Ok(())
    }

    /// Write data and finish the send side
    pub async fn write_and_finish(&mut self, data: &[u8]) -> Result<(), P2PError> {
        self.write(data).await?;
        self.finish().await
    }

    /// Finish the send side (signal end of data)
    pub async fn finish(&mut self) -> Result<(), P2PError> {
        self.send
            .finish()
            .map_err(|e| P2PError::Stream(format!("Finish failed: {}", e)))
    }

    /// Read data from the stream
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, P2PError> {
        let bytes_read = self
            .recv
            .read(buf)
            .await
            .map_err(|e| P2PError::Stream(format!("Read failed: {}", e)))?
            .unwrap_or(0);
        self.stats.add_bytes_received(bytes_read as u64);
        Ok(bytes_read)
    }

    /// Read all data until end of stream
    pub async fn read_to_end(&mut self, max_size: usize) -> Result<Vec<u8>, P2PError> {
        let data = self
            .recv
            .read_to_end(max_size)
            .await
            .map_err(|e| P2PError::Stream(format!("Read to end failed: {}", e)))?;
        self.stats.add_bytes_received(data.len() as u64);
        Ok(data)
    }

    /// Read exact number of bytes
    pub async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), P2PError> {
        self.recv
            .read_exact(buf)
            .await
            .map_err(|e| P2PError::Stream(format!("Read exact failed: {}", e)))?;
        self.stats.add_bytes_received(buf.len() as u64);
        Ok(())
    }
}

/// Stream manager for a P2P connection
///
/// Provides methods to open and accept multiplexed streams,
/// and access connection metadata.
pub struct StreamManager {
    /// The underlying connection
    connection: Arc<P2PConnection>,
}

impl StreamManager {
    /// Create a new stream manager for a connection
    pub fn new(connection: Arc<P2PConnection>) -> Self {
        Self { connection }
    }

    /// Open a new bidirectional stream
    ///
    /// # Requirements Coverage
    /// - Requirement 3.4: Multiplexed bidirectional streams
    pub async fn open_bi(&self) -> Result<BiStream, P2PError> {
        let (send, recv) = self
            .connection
            .connection()
            .open_bi()
            .await
            .map_err(|e| P2PError::Stream(format!("Failed to open stream: {}", e)))?;

        Ok(BiStream::new(send, recv))
    }

    /// Accept an incoming bidirectional stream
    pub async fn accept_bi(&self) -> Result<BiStream, P2PError> {
        let (send, recv) = self
            .connection
            .connection()
            .accept_bi()
            .await
            .map_err(|e| P2PError::Stream(format!("Failed to accept stream: {}", e)))?;

        Ok(BiStream::new(send, recv))
    }

    /// Open a unidirectional send stream
    pub async fn open_uni(&self) -> Result<SendStream, P2PError> {
        self.connection
            .connection()
            .open_uni()
            .await
            .map_err(|e| P2PError::Stream(format!("Failed to open uni stream: {}", e)))
    }

    /// Accept an incoming unidirectional receive stream
    pub async fn accept_uni(&self) -> Result<RecvStream, P2PError> {
        self.connection
            .connection()
            .accept_uni()
            .await
            .map_err(|e| P2PError::Stream(format!("Failed to accept uni stream: {}", e)))
    }

    /// Get the peer's node ID
    pub fn peer_id(&self) -> iroh::NodeId {
        self.connection.peer_id()
    }

    /// Get connection info including latency and connection type
    ///
    /// # Requirements Coverage
    /// - Requirement 3.5: Connection metadata (latency, type)
    pub async fn connection_info(&self) -> P2PConnectionInfo {
        self.connection.info().await
    }

    /// Get the underlying connection
    pub fn connection(&self) -> &Arc<P2PConnection> {
        &self.connection
    }
}

/// Helper trait for stream operations
pub trait StreamExt {
    /// Send a length-prefixed message
    fn send_message(
        &mut self,
        data: &[u8],
    ) -> impl std::future::Future<Output = Result<(), P2PError>> + Send;

    /// Receive a length-prefixed message
    fn recv_message(
        &mut self,
        max_size: usize,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, P2PError>> + Send;
}

impl StreamExt for BiStream {
    async fn send_message(&mut self, data: &[u8]) -> Result<(), P2PError> {
        // Send length prefix (4 bytes, big-endian)
        let len = data.len() as u32;
        self.write(&len.to_be_bytes()).await?;
        // Send data
        self.write(data).await
    }

    async fn recv_message(&mut self, max_size: usize) -> Result<Vec<u8>, P2PError> {
        // Read length prefix
        let mut len_buf = [0u8; 4];
        self.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > max_size {
            return Err(P2PError::Stream(format!(
                "Message too large: {} > {}",
                len, max_size
            )));
        }

        // Read data
        let mut data = vec![0u8; len];
        self.read_exact(&mut data).await?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full stream tests require actual P2P connections
    // These are placeholder tests for the module structure

    #[test]
    fn stream_ext_trait_exists() {
        // Verify the trait is properly defined
        fn _assert_send<T: StreamExt + Send>() {}
    }
}
