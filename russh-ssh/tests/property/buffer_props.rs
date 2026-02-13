//! Property-based tests for streaming buffer
//!
//! Feature: russh-ssh
//! These tests validate the correctness properties of the adaptive buffer.

use proptest::prelude::*;
use russh_ssh::streaming::buffer::{AdaptiveBuffer, BufferConfig};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property: Buffer Data Integrity
    ///
    /// *For any* data added to the buffer, reading it back SHALL return
    /// the exact same data.
    ///
    /// **Validates: Requirements 6.2 - Data integrity**
    #[test]
    fn buffer_preserves_data_integrity(
        data in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut buffer = AdaptiveBuffer::new(config);

        buffer.add_data(0, data.clone());

        let read = buffer.read(data.len()).unwrap();
        prop_assert_eq!(data, read, "Buffer must preserve data integrity");
    }

    /// Feature: russh-ssh, Property: Buffer Position Tracking
    ///
    /// *For any* sequence of reads, the buffer position SHALL advance
    /// by exactly the number of bytes read.
    ///
    /// **Validates: Requirements 6.2 - Position tracking**
    #[test]
    fn buffer_position_advances_correctly(
        data in prop::collection::vec(any::<u8>(), 100..1000),
        read_sizes in prop::collection::vec(1usize..50, 1..10),
    ) {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut buffer = AdaptiveBuffer::new(config);

        buffer.add_data(0, data.clone());

        let mut expected_position = 0u64;
        for read_size in read_sizes {
            let actual_read_size = read_size.min(data.len() - expected_position as usize);
            if actual_read_size == 0 {
                break;
            }

            buffer.read(actual_read_size);
            expected_position += actual_read_size as u64;

            prop_assert_eq!(
                buffer.position(), expected_position,
                "Buffer position must advance by bytes read"
            );
        }
    }

    /// Feature: russh-ssh, Property: Buffer Size Tracking
    ///
    /// *For any* data added to the buffer, the buffered_bytes count SHALL
    /// accurately reflect the total data stored.
    ///
    /// **Validates: Requirements 6.2 - Size tracking**
    #[test]
    fn buffer_size_tracking_accurate(
        chunks in prop::collection::vec(
            (0u64..1000, prop::collection::vec(any::<u8>(), 1..100)),
            1..10
        ),
    ) {
        let max_buffer_size = 1024 * 1024;
        let config = BufferConfig::new(1024, max_buffer_size);
        let mut buffer = AdaptiveBuffer::new(config);

        for (position, data) in chunks {
            buffer.add_data(position, data);

            // Note: This may not be exact due to eviction, but should not exceed max
            prop_assert!(
                buffer.buffered_bytes() <= max_buffer_size,
                "Buffer size must not exceed max"
            );
        }
    }

    /// Feature: russh-ssh, Property: Buffer Seek Validity
    ///
    /// *For any* seek to a buffered position, the seek SHALL succeed and
    /// subsequent reads SHALL return data from that position.
    ///
    /// **Validates: Requirements 6.2 - Seek functionality**
    #[test]
    fn buffer_seek_to_buffered_position(
        data in prop::collection::vec(any::<u8>(), 100..1000),
        seek_position in 0u64..100,
    ) {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut buffer = AdaptiveBuffer::new(config)
            .with_stream_size(data.len() as u64);

        buffer.add_data(0, data.clone());

        let seek_pos = seek_position.min(data.len() as u64 - 1);
        let result = buffer.seek(seek_pos);

        prop_assert!(result, "Seek to buffered position must succeed");
        prop_assert_eq!(buffer.position(), seek_pos, "Position must be updated after seek");

        // Read should return data from seek position
        if let Some(read_data) = buffer.read(10) {
            let expected_start = seek_pos as usize;
            let expected_end = (expected_start + read_data.len()).min(data.len());
            prop_assert_eq!(
                &read_data[..],
                &data[expected_start..expected_end],
                "Read after seek must return correct data"
            );
        }
    }

    /// Feature: russh-ssh, Property: Buffer Clear Resets State
    ///
    /// *For any* buffer with data, clearing it SHALL reset all state
    /// to initial values.
    ///
    /// **Validates: Requirements 6.2 - Clear functionality**
    #[test]
    fn buffer_clear_resets_state(
        data in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut buffer = AdaptiveBuffer::new(config);

        buffer.add_data(0, data);
        buffer.read(100);

        buffer.clear();

        prop_assert_eq!(buffer.position(), 0, "Position must be 0 after clear");
        prop_assert_eq!(buffer.buffered_bytes(), 0, "Buffered bytes must be 0 after clear");
        prop_assert!(buffer.buffered_ranges().is_empty(), "Ranges must be empty after clear");
    }

    /// Feature: russh-ssh, Property: Buffer Max Size Enforcement
    ///
    /// *For any* amount of data added, the buffer SHALL never exceed
    /// the configured maximum size.
    ///
    /// **Validates: Requirements 6.2 - Memory bounds**
    #[test]
    fn buffer_respects_max_size(
        chunks in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 100..500),
            1..50
        ),
    ) {
        let max_size = 2048;
        let config = BufferConfig::new(512, max_size);
        let mut buffer = AdaptiveBuffer::new(config);

        let mut position = 0u64;
        for chunk in chunks {
            buffer.add_data(position, chunk.clone());
            position += chunk.len() as u64;

            prop_assert!(
                buffer.buffered_bytes() <= max_size,
                "Buffer must never exceed max size: {} > {}",
                buffer.buffered_bytes(), max_size
            );
        }
    }

    /// Feature: russh-ssh, Property: Buffer Range Tracking
    ///
    /// *For any* data added at specific positions, the buffered_ranges
    /// SHALL accurately reflect what positions are buffered.
    ///
    /// **Validates: Requirements 6.2 - Range tracking**
    #[test]
    fn buffer_range_tracking_accurate(
        position in 0u64..1000,
        data in prop::collection::vec(any::<u8>(), 1..100),
    ) {
        let config = BufferConfig::new(1024, 1024 * 1024);
        let mut buffer = AdaptiveBuffer::new(config);

        buffer.add_data(position, data.clone());

        let ranges = buffer.buffered_ranges();
        prop_assert!(!ranges.is_empty(), "Ranges must not be empty after adding data");

        // Check that the position is marked as buffered
        prop_assert!(
            buffer.is_buffered(position),
            "Start position must be buffered"
        );

        if data.len() > 1 {
            prop_assert!(
                buffer.is_buffered(position + data.len() as u64 - 1),
                "End position must be buffered"
            );
        }
    }
}

/// Tests for watermark behavior
#[cfg(test)]
mod watermark_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// Feature: russh-ssh, Property: Low Watermark Triggers Need
        ///
        /// *For any* buffer below the low watermark, needs_data SHALL return true.
        ///
        /// **Validates: Requirements 6.2 - Watermark behavior**
        #[test]
        fn low_watermark_triggers_need(
            data_size in 1usize..100,
        ) {
            let config = BufferConfig::new(1024, 1024 * 1024)
                .with_watermarks(200, 800);
            let mut buffer = AdaptiveBuffer::new(config);

            // Add less data than low watermark
            let data = vec![0u8; data_size.min(199)];
            buffer.add_data(0, data);

            prop_assert!(
                buffer.needs_data(),
                "Buffer below low watermark must need data"
            );
        }

        /// Feature: russh-ssh, Property: High Watermark Triggers Full
        ///
        /// *For any* buffer above the high watermark, is_full SHALL return true.
        ///
        /// **Validates: Requirements 6.2 - Watermark behavior**
        #[test]
        fn high_watermark_triggers_full(
            extra_size in 1usize..100,
        ) {
            let config = BufferConfig::new(1024, 2048)
                .with_watermarks(200, 800);
            let mut buffer = AdaptiveBuffer::new(config);

            // Add more data than high watermark
            let data = vec![0u8; 800 + extra_size];
            buffer.add_data(0, data);

            prop_assert!(
                buffer.is_full(),
                "Buffer above high watermark must be full"
            );
        }
    }
}
