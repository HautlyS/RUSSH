//! Session state management

use chrono::{DateTime, Utc};
use russh_ssh::ssh::SshClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Session information (serializable for frontend)
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
    /// SSH client handle
    pub client: Arc<Mutex<SshClient>>,
    /// Terminal output task handle
    pub terminal_task: Option<tokio::task::JoinHandle<()>>,
    /// Terminal input sender
    pub terminal_input_tx: Option<tokio::sync::mpsc::Sender<Vec<u8>>>,
}

impl SessionState {
    pub fn new(session_id: String, host: String, username: String, client: SshClient) -> Self {
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
            client: Arc::new(Mutex::new(client)),
            terminal_task: None,
            terminal_input_tx: None,
        }
    }

    pub fn set_connected(&mut self) {
        self.info.status = SessionStatus::Connected;
    }

    #[allow(dead_code)]
    pub fn set_disconnected(&mut self) {
        self.info.status = SessionStatus::Disconnected;
    }

    #[allow(dead_code)]
    pub fn set_error(&mut self) {
        self.info.status = SessionStatus::Error;
    }

    pub fn increment_commands(&mut self) {
        self.info.stats.commands_executed += 1;
    }

    #[allow(dead_code)]
    pub fn add_bytes_sent(&mut self, bytes: u64) {
        self.info.stats.bytes_sent += bytes;
    }

    pub fn add_bytes_received(&mut self, bytes: u64) {
        self.info.stats.bytes_received += bytes;
    }

    pub fn stop_terminal(&mut self) {
        if let Some(task) = self.terminal_task.take() {
            task.abort();
        }
        self.terminal_input_tx = None;
    }
}
