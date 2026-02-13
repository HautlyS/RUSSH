//! Encryption module
//!
//! This module provides encryption utilities using BLAKE3 and secure encryption.
//! The encryption layer follows OCKAM's secure channel principles:
//! - End-to-end encryption using AES-256-GCM
//! - Mutual authentication between peers
//! - BLAKE3 for high-performance cryptographic hashing
//! - Zero-knowledge key storage principles

pub mod cipher;
pub mod hash;
pub mod secure_channel;

pub use cipher::*;
pub use hash::*;
pub use secure_channel::*;
