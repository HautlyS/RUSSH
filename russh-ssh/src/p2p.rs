//! P2P Network Layer using Iroh
//!
//! This module provides peer-to-peer networking capabilities using Iroh's
//! QUIC-based transport with NAT traversal and relay fallback.
//!
//! # Features
//! - QUIC transport for reliable, multiplexed connections
//! - NAT traversal via hole-punching
//! - Relay server fallback when direct connection fails
//! - Bidirectional stream support
//!
//! # Requirements Coverage
//! - Requirement 3.1: Iroh QUIC implementation for transport
//! - Requirement 3.2: NAT hole-punching for direct connections
//! - Requirement 3.3: Relay server fallback
//! - Requirement 3.4: Multiplexed bidirectional streams
//! - Requirement 3.5: Connection metadata (latency, type)

pub mod endpoint;
pub mod connection;
pub mod stream;

pub use endpoint::*;
pub use connection::*;
pub use stream::*;
