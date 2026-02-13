//! Session Management
//!
//! Provides session profiles, persistence, and management.
//!
//! # Requirements Coverage
//! - Requirement 8.1: Session parameter completeness
//! - Requirement 8.2: Session profile serialization
//! - Requirement 8.3: Session management
//! - Requirement 8.4: Session persistence
//! - Requirement 8.7: Session serialization round-trip

pub mod manager;
pub mod profile;

pub use manager::SessionManager;
pub use profile::SessionProfile;
