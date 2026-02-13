//! russh SSH Library
//!
//! A secure, efficient russh SSH library built in Rust with:
//! - Tokio async runtime for non-blocking operations
//! - Iroh P2P networking with QUIC transport
//! - End-to-end encryption
//! - Virtual distributed filesystem
//! - Media streaming capabilities

pub mod config;
pub mod connection;
pub mod encryption;
pub mod error;
pub mod p2p;
pub mod session;
pub mod streaming;
pub mod vdfs;

pub use error::{ConnectionError, ReconnectionError};

pub mod ssh;
pub use config::*;

// Re-export iroh types needed by consumers
pub use iroh::NodeId;
