//! Stream Handler
//!
//! High-level stream handling with seeking and resumption support.
//!
//! # Requirements Coverage
//! - Requirement 6.1: Seeking support
//! - Requirement 6.5: Stream resumption

use super::buffer::{AdaptiveBuffer, BufferConfig};
use crate::error::StreamError;
use std::time::Instant;

/// Stream position information
#[derive(Debug, Clone)]
pub struct StreamPosition {
    /// Current byte position
    pub position: u64,
    /// Total stream size (if known)
    pub total_size: Option<u64>,
    /// Percentage complete (0.0 - 1.0)
    pub progress: Option<f64>,
}

impl StreamPosition {
    /// Create a new stream position
    pub fn new(position: u64, total_size: Option<u64>) -> Self {
        let progress = total_size.map(|size| {
            if size == 0 {
                1.0
            } else {
                position as f64 / size as f64
            }
        });
        
        Self {
            position,
            total_size,
            progress,
        }
    }
}

/// Stream state for resumption
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StreamState {
    /// Stream identifier
    pub stream_id: String,
    /// Last known position
    pub position: u64,
    /// Total size if known
    pub total_size: Option<u64>,
    /// Timestamp of last activity (as Unix timestamp)
    pub last_activity: i64,
}

impl StreamState {
    /// Create a new stream state
    pub fn new(stream_id: String, position: u64, total_size: Option<u64>) -> Self {
        Self {
            stream_id,
            position,
            total_size,
            last_activity: chrono::Utc::now().timestamp(),
        }
    }

    /// Update the position
    pub fn update_position(&mut self, position: u64) {
        self.position = position;
        self.last_activity = chrono::Utc::now().timestamp();
    }
}

/// Stream handler for media streaming
///
/// Provides buffering, seeking, and resumption capabilities.
pub struct StreamHandler {
    /// Stream identifier
    stream_id: String,
    /// Adaptive buffer
    buffer: AdaptiveBuffer,
    /// Stream state for resumption
    state: StreamState,
    /// Start time
    start_time: Instant,
    /// Bytes read
    bytes_read: u64,
    /// Is stream active
    active: bool,
}

impl StreamHandler {
    /// Create a new stream handler
    pub fn new(stream_id: String, config: BufferConfig) -> Self {
        let buffer = AdaptiveBuffer::new(config);
        let state = StreamState::new(stream_id.clone(), 0, None);
        
        Self {
            stream_id,
            buffer,
            state,
            start_time: Instant::now(),
            bytes_read: 0,
            active: true,
        }
    }

    /// Create with known stream size
    pub fn with_size(mut self, size: u64) -> Self {
        self.buffer = self.buffer.with_stream_size(size);
        self.state.total_size = Some(size);
        self
    }

    /// Resume from a saved state
    pub fn resume(stream_id: String, config: BufferConfig, state: StreamState) -> Self {
        let mut buffer = AdaptiveBuffer::new(config);
        if let Some(size) = state.total_size {
            buffer = buffer.with_stream_size(size);
        }
        buffer.seek(state.position);
        
        Self {
            stream_id,
            buffer,
            state,
            start_time: Instant::now(),
            bytes_read: 0,
            active: true,
        }
    }

    /// Get the stream ID
    pub fn stream_id(&self) -> &str {
        &self.stream_id
    }

    /// Get current position
    pub fn position(&self) -> StreamPosition {
        StreamPosition::new(self.buffer.position(), self.buffer.stream_size())
    }

    /// Get the current state for resumption
    pub fn state(&self) -> &StreamState {
        &self.state
    }

    /// Check if stream is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if buffer needs more data
    pub fn needs_data(&self) -> bool {
        self.buffer.needs_data()
    }

    /// Check if buffer is full
    pub fn is_buffer_full(&self) -> bool {
        self.buffer.is_full()
    }

    /// Add data to the buffer
    pub fn add_data(&mut self, position: u64, data: Vec<u8>) {
        self.buffer.add_data(position, data);
    }

    /// Read data from the stream
    pub fn read(&mut self, len: usize) -> Result<Vec<u8>, StreamError> {
        if !self.active {
            return Err(StreamError::NotFound(self.stream_id.clone()));
        }

        match self.buffer.read(len) {
            Some(data) => {
                self.bytes_read += data.len() as u64;
                self.state.update_position(self.buffer.position());
                Ok(data)
            }
            None => Err(StreamError::BufferUnderrun),
        }
    }

    /// Seek to a position
    ///
    /// # Requirements Coverage
    /// - Requirement 6.1: Seeking support
    pub fn seek(&mut self, position: u64) -> Result<(), StreamError> {
        if !self.active {
            return Err(StreamError::NotFound(self.stream_id.clone()));
        }

        if let Some(size) = self.buffer.stream_size() {
            if position > size {
                return Err(StreamError::SeekOutOfBounds { position, size });
            }
        }

        self.buffer.seek(position);
        self.state.update_position(position);
        Ok(())
    }

    /// Check if a position is buffered
    pub fn is_buffered(&self, position: u64) -> bool {
        self.buffer.is_buffered(position)
    }

    /// Get buffered ranges
    pub fn buffered_ranges(&self) -> Vec<std::ops::Range<u64>> {
        self.buffer.buffered_ranges()
    }

    /// Get buffered bytes
    pub fn buffered_bytes(&self) -> usize {
        self.buffer.buffered_bytes()
    }

    /// Get bytes read
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get average read rate (bytes per second)
    pub fn read_rate(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.bytes_read as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Stop the stream
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Clear the buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_position_progress() {
        let pos = StreamPosition::new(50, Some(100));
        assert_eq!(pos.progress, Some(0.5));
        
        let pos_unknown = StreamPosition::new(50, None);
        assert_eq!(pos_unknown.progress, None);
    }

    #[test]
    fn stream_handler_basic() {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let handler = StreamHandler::new("test-stream".to_string(), config);
        
        assert_eq!(handler.stream_id(), "test-stream");
        assert!(handler.is_active());
        assert!(handler.needs_data());
    }

    #[test]
    fn stream_handler_read_write() {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut handler = StreamHandler::new("test-stream".to_string(), config)
            .with_size(1000);
        
        handler.add_data(0, vec![1, 2, 3, 4, 5]);
        
        let data = handler.read(3).unwrap();
        assert_eq!(data, vec![1, 2, 3]);
        assert_eq!(handler.position().position, 3);
    }

    #[test]
    fn stream_handler_seek() {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut handler = StreamHandler::new("test-stream".to_string(), config)
            .with_size(100);
        
        handler.add_data(0, vec![0; 100]);
        
        handler.seek(50).unwrap();
        assert_eq!(handler.position().position, 50);
        
        // Out of bounds
        let result = handler.seek(150);
        assert!(result.is_err());
    }

    #[test]
    fn stream_handler_resume() {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let state = StreamState::new("test-stream".to_string(), 50, Some(100));
        
        let handler = StreamHandler::resume("test-stream".to_string(), config, state);
        
        assert_eq!(handler.position().position, 50);
        assert_eq!(handler.state().total_size, Some(100));
    }

    #[test]
    fn stream_state_serialization() {
        let state = StreamState::new("test".to_string(), 100, Some(1000));
        
        let json = serde_json::to_string(&state).unwrap();
        let restored: StreamState = serde_json::from_str(&json).unwrap();
        
        assert_eq!(restored.stream_id, state.stream_id);
        assert_eq!(restored.position, state.position);
        assert_eq!(restored.total_size, state.total_size);
    }
}
