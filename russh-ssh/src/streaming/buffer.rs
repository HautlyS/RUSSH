//! Adaptive Buffering
//!
//! Implements adaptive buffering for media streaming with configurable
//! buffer sizes and duration tracking.
//!
//! # Requirements Coverage
//! - Requirement 6.2: Adaptive buffering

use std::collections::BTreeMap;
use std::ops::Range;
use std::time::Duration;

/// Buffer configuration
#[derive(Debug, Clone)]
pub struct BufferConfig {
    /// Minimum buffer size in bytes
    pub min_buffer_size: usize,
    /// Maximum buffer size in bytes
    pub max_buffer_size: usize,
    /// Target buffer duration (for media)
    pub target_duration: Duration,
    /// Low watermark - start buffering when below this
    pub low_watermark: usize,
    /// High watermark - stop buffering when above this
    pub high_watermark: usize,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            min_buffer_size: 64 * 1024,      // 64 KB
            max_buffer_size: 16 * 1024 * 1024, // 16 MB
            target_duration: Duration::from_secs(10),
            low_watermark: 256 * 1024,       // 256 KB
            high_watermark: 8 * 1024 * 1024, // 8 MB
        }
    }
}

impl BufferConfig {
    /// Create a new buffer config with custom sizes
    pub fn new(min: usize, max: usize) -> Self {
        Self {
            min_buffer_size: min,
            max_buffer_size: max,
            ..Default::default()
        }
    }

    /// Set target duration
    pub fn with_target_duration(mut self, duration: Duration) -> Self {
        self.target_duration = duration;
        self
    }

    /// Set watermarks
    pub fn with_watermarks(mut self, low: usize, high: usize) -> Self {
        self.low_watermark = low;
        self.high_watermark = high;
        self
    }
}

/// A buffered range of data
#[derive(Debug, Clone)]
struct BufferedRange {
    /// Start position in the stream
    start: u64,
    /// The buffered data
    data: Vec<u8>,
}

impl BufferedRange {
    fn end(&self) -> u64 {
        self.start + self.data.len() as u64
    }

    fn range(&self) -> Range<u64> {
        self.start..self.end()
    }

    fn contains(&self, pos: u64) -> bool {
        pos >= self.start && pos < self.end()
    }
}

/// Adaptive buffer for streaming data
///
/// Maintains buffered ranges and adapts buffer size based on
/// consumption patterns.
#[derive(Debug)]
pub struct AdaptiveBuffer {
    /// Configuration
    config: BufferConfig,
    /// Buffered ranges (keyed by start position)
    ranges: BTreeMap<u64, BufferedRange>,
    /// Total buffered bytes
    total_buffered: usize,
    /// Total stream size (if known)
    stream_size: Option<u64>,
    /// Current read position
    read_position: u64,
    /// Bytes consumed since last adaptation
    bytes_consumed: usize,
    /// Current adaptive buffer target
    adaptive_target: usize,
}

impl AdaptiveBuffer {
    /// Create a new adaptive buffer
    pub fn new(config: BufferConfig) -> Self {
        let adaptive_target = config.min_buffer_size;
        Self {
            config,
            ranges: BTreeMap::new(),
            total_buffered: 0,
            stream_size: None,
            read_position: 0,
            bytes_consumed: 0,
            adaptive_target,
        }
    }

    /// Create with known stream size
    pub fn with_stream_size(mut self, size: u64) -> Self {
        self.stream_size = Some(size);
        self
    }

    /// Get the current read position
    pub fn position(&self) -> u64 {
        self.read_position
    }

    /// Get total buffered bytes
    pub fn buffered_bytes(&self) -> usize {
        self.total_buffered
    }

    /// Get the stream size if known
    pub fn stream_size(&self) -> Option<u64> {
        self.stream_size
    }

    /// Check if we need more data
    pub fn needs_data(&self) -> bool {
        self.total_buffered < self.config.low_watermark
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.total_buffered >= self.config.high_watermark
    }

    /// Get the current adaptive buffer target
    pub fn adaptive_target(&self) -> usize {
        self.adaptive_target
    }

    /// Add data to the buffer at a specific position
    /// 
    /// Memory safety: Evicts old data BEFORE adding new data to prevent memory spikes.
    /// The buffer will never exceed `max_buffer_size` even temporarily.
    pub fn add_data(&mut self, position: u64, data: Vec<u8>) {
        if data.is_empty() {
            return;
        }

        let data_len = data.len();
        
        // CRITICAL: Evict old data BEFORE adding new data to prevent memory spikes
        // This ensures we never exceed max_buffer_size even temporarily
        if self.total_buffered + data_len > self.config.max_buffer_size {
            self.evict_to_make_room(data_len);
        }
        
        // If data is larger than max buffer size, truncate it
        let data = if data_len > self.config.max_buffer_size {
            tracing::warn!(
                "Data chunk ({} bytes) exceeds max buffer size ({} bytes), truncating",
                data_len, self.config.max_buffer_size
            );
            data[..self.config.max_buffer_size].to_vec()
        } else {
            data
        };
        let actual_len = data.len();
        
        // Check if this overlaps with existing ranges
        // For simplicity, we just add as a new range
        // A production implementation would merge overlapping ranges
        let range = BufferedRange {
            start: position,
            data,
        };

        self.ranges.insert(position, range);
        self.total_buffered += actual_len;
    }
    
    /// Evict data to make room for new data
    fn evict_to_make_room(&mut self, needed_space: usize) {
        let target_size = self.config.max_buffer_size.saturating_sub(needed_space);
        
        while self.total_buffered > target_size {
            // Remove the oldest range that's before the read position first
            let to_remove = self.ranges.iter()
                .filter(|(_, r)| r.end() <= self.read_position)
                .map(|(k, r)| (*k, r.data.len()))
                .next();

            if let Some((key, size)) = to_remove {
                self.ranges.remove(&key);
                self.total_buffered -= size;
            } else {
                // No old ranges to remove, remove the oldest anyway
                if let Some((&key, range)) = self.ranges.iter().next() {
                    let size = range.data.len();
                    self.ranges.remove(&key);
                    self.total_buffered -= size;
                } else {
                    break;
                }
            }
        }
    }

    /// Read data from the buffer
    ///
    /// Returns the data if available, or None if the position is not buffered.
    pub fn read(&mut self, len: usize) -> Option<Vec<u8>> {
        let pos = self.read_position;
        
        // Find the range containing this position
        let range = self.ranges.iter()
            .find(|(_, r)| r.contains(pos))?;
        
        let range_start = *range.0;
        let range_data = &range.1.data;
        
        // Calculate offset within the range
        let offset = (pos - range_start) as usize;
        let available = range_data.len() - offset;
        let to_read = len.min(available);
        
        let data = range_data[offset..offset + to_read].to_vec();
        
        // Update position
        self.read_position += to_read as u64;
        self.bytes_consumed += to_read;
        
        // Adapt buffer size based on consumption
        self.adapt_buffer_size();
        
        Some(data)
    }

    /// Seek to a position
    ///
    /// Returns true if the position is buffered, false otherwise.
    pub fn seek(&mut self, position: u64) -> bool {
        // Check bounds
        if let Some(size) = self.stream_size {
            if position > size {
                return false;
            }
        }

        self.read_position = position;
        
        // Check if position is buffered
        self.ranges.iter().any(|(_, r)| r.contains(position))
    }

    /// Get buffered ranges
    pub fn buffered_ranges(&self) -> Vec<Range<u64>> {
        self.ranges.values().map(|r| r.range()).collect()
    }

    /// Check if a position is buffered
    pub fn is_buffered(&self, position: u64) -> bool {
        self.ranges.iter().any(|(_, r)| r.contains(position))
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.ranges.clear();
        self.total_buffered = 0;
        self.read_position = 0;
    }

    /// Adapt buffer size based on consumption patterns
    fn adapt_buffer_size(&mut self) {
        // Simple adaptation: increase target if consuming fast
        // Make this configurable via BufferConfig
        let adaptation_threshold = self.config.high_watermark / 8; // 1/8 of high watermark
        
        if self.bytes_consumed >= adaptation_threshold {
            // Increase adaptive target
            self.adaptive_target = (self.adaptive_target * 3 / 2)
                .min(self.config.max_buffer_size);
            self.bytes_consumed = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_config_default() {
        let config = BufferConfig::default();
        assert!(config.min_buffer_size < config.max_buffer_size);
        assert!(config.low_watermark < config.high_watermark);
    }

    #[test]
    fn adaptive_buffer_basic() {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let buffer = AdaptiveBuffer::new(config);
        
        assert_eq!(buffer.position(), 0);
        assert_eq!(buffer.buffered_bytes(), 0);
        assert!(buffer.needs_data());
    }

    #[test]
    fn adaptive_buffer_add_and_read() {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut buffer = AdaptiveBuffer::new(config);
        
        let data = vec![1, 2, 3, 4, 5];
        buffer.add_data(0, data.clone());
        
        assert_eq!(buffer.buffered_bytes(), 5);
        
        let read = buffer.read(3).unwrap();
        assert_eq!(read, vec![1, 2, 3]);
        assert_eq!(buffer.position(), 3);
    }

    #[test]
    fn adaptive_buffer_seek() {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let buffer = AdaptiveBuffer::new(config)
            .with_stream_size(100);
        
        // Need to make it mutable for add_data and seek
        let mut buffer = buffer;
        buffer.add_data(0, vec![0; 50]);
        buffer.add_data(50, vec![0; 50]);
        
        assert!(buffer.seek(25));
        assert_eq!(buffer.position(), 25);
        
        assert!(buffer.seek(75));
        assert_eq!(buffer.position(), 75);
        
        // Out of bounds
        assert!(!buffer.seek(150));
    }

    #[test]
    fn adaptive_buffer_ranges() {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut buffer = AdaptiveBuffer::new(config);
        
        buffer.add_data(0, vec![0; 100]);
        buffer.add_data(200, vec![0; 100]);
        
        let ranges = buffer.buffered_ranges();
        assert_eq!(ranges.len(), 2);
        assert!(buffer.is_buffered(50));
        assert!(!buffer.is_buffered(150));
        assert!(buffer.is_buffered(250));
    }
}
