//! Stream Handler Implementation
//!
//! Provides adaptive buffering, seeking, and stream resumption for media streaming.
//!
//! # Requirements Coverage
//! - Requirement 6.1: Seeking support
//! - Requirement 6.2: Adaptive buffering
//! - Requirement 6.5: Stream resumption

pub mod buffer;
pub mod handler;
pub mod video;

pub use buffer::{AdaptiveBuffer, BufferConfig};
pub use handler::{StreamHandler, StreamPosition, StreamState};
pub use video::{StreamSession, StreamRoom, StreamSource, PlaybackState, SyncEvent, HttpVideoStream};
