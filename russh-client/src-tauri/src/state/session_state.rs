//! Session state management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub profile_id: Option<String>,
    pub host: String,
    pub username: String,
    pub connected_at: DateTime<Utc>,
    pub status: SessionStatus,
    pub stats: SessionStats,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
    Error,
}

/// Session statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub commands_executed: u64,
}

/// Internal session state (not serialized)
pub struct SessionState {
    pub info: SessionInfo,
    // SSH client handle would go here
    // pub client: Option<SshClient>,
}

impl SessionState {
    pub fn new(session_id: String, host: String, username: String) -> Self {
        Self {
            info: SessionInfo {
                session_id,
                profile_id: None,
                host,
                username,
                connected_at: Utc::now(),
                status: SessionStatus::Connecting,
                stats: SessionStats::default(),
            },
        }
    }

    pub fn set_connected(&mut self) {
        self.info.status = SessionStatus::Connected;
    }

    pub fn set_disconnected(&mut self) {
        self.info.status = SessionStatus::Disconnected;
    }

    pub fn set_error(&mut self) {
        self.info.status = SessionStatus::Error;
    }

    pub fn increment_commands(&mut self) {
        self.info.stats.commands_executed += 1;
    }

    pub fn add_bytes_sent(&mut self, bytes: u64) {
        self.info.stats.bytes_sent += bytes;
    }

    pub fn add_bytes_received(&mut self, bytes: u64) {
        self.info.stats.bytes_received += bytes;
    }
}
