//! Application state management

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::AppError;
use super::session_state::{SessionState, SessionInfo};

/// Profile data for saved connections
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileData {
    pub id: Option<String>,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub key_path: Option<String>,
    pub tags: Vec<String>,
    pub folder: Option<String>,
    pub color: Option<String>,
    pub auto_reconnect: bool,
    #[serde(default)]
    pub use_count: u32,
    pub last_connected: Option<String>,
}

/// Application settings
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub terminal: TerminalSettings,
    pub appearance: AppearanceSettings,
    pub keyboard: KeyboardSettings,
    pub notifications: NotificationSettings,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeneralSettings {
    pub start_minimized: bool,
    pub minimize_to_tray: bool,
    pub check_updates: bool,
    pub language: String,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            start_minimized: false,
            minimize_to_tray: true,
            check_updates: true,
            language: "en".to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerminalSettings {
    pub font_family: String,
    pub font_size: u32,
    pub line_height: f32,
    pub cursor_style: String,
    pub cursor_blink: bool,
    pub scrollback: u32,
    pub copy_on_select: bool,
    pub right_click_paste: bool,
    pub bell_sound: bool,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono, Menlo, Monaco, monospace".to_string(),
            font_size: 14,
            line_height: 1.2,
            cursor_style: "block".to_string(),
            cursor_blink: true,
            scrollback: 10000,
            copy_on_select: false,
            right_click_paste: true,
            bell_sound: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppearanceSettings {
    pub theme: String,
    pub accent_color: String,
    pub terminal_theme: String,
    pub sidebar_position: String,
    pub compact_mode: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            accent_color: "#3b82f6".to_string(),
            terminal_theme: "dracula".to_string(),
            sidebar_position: "left".to_string(),
            compact_mode: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyboardSettings {
    pub shortcuts: HashMap<String, String>,
    pub enable_global_shortcuts: bool,
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("newConnection".to_string(), "Ctrl+N".to_string());
        shortcuts.insert("newTab".to_string(), "Ctrl+T".to_string());
        shortcuts.insert("closeTab".to_string(), "Ctrl+W".to_string());
        shortcuts.insert("nextTab".to_string(), "Ctrl+Tab".to_string());
        shortcuts.insert("prevTab".to_string(), "Ctrl+Shift+Tab".to_string());
        shortcuts.insert("commandPalette".to_string(), "Ctrl+K".to_string());
        shortcuts.insert("settings".to_string(), "Ctrl+,".to_string());
        shortcuts.insert("toggleSidebar".to_string(), "Ctrl+B".to_string());
        
        Self {
            shortcuts,
            enable_global_shortcuts: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub sound: bool,
    pub connection_events: bool,
    pub transfer_complete: bool,
    pub errors: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sound: false,
            connection_events: true,
            transfer_complete: true,
            errors: true,
        }
    }
}

/// P2P Node information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct P2PNodeInfo {
    pub node_id: String,
    pub relay_url: Option<String>,
    pub direct_addresses: Vec<String>,
    pub is_online: bool,
}

/// P2P Peer information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct P2PPeerInfo {
    pub peer_id: String,
    pub connection_type: String,
    pub latency_ms: u64,
    pub connected_at: String,
}

/// Main application state
pub struct AppState {
    /// Active SSH sessions
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    /// Saved connection profiles
    profiles: Arc<RwLock<HashMap<String, ProfileData>>>,
    /// Application settings
    settings: Arc<RwLock<AppSettings>>,
    /// P2P peers
    p2p_peers: Arc<RwLock<HashMap<String, P2PPeerInfo>>>,
    /// Data directory path
    data_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("russh-client");
        
        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&data_dir).ok();
        
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(AppSettings::default())),
            p2p_peers: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
        }
    }

    // Session management
    pub async fn create_session(&self, host: String, username: String) -> String {
        let session_id = Uuid::new_v4().to_string();
        let session = SessionState::new(session_id.clone(), host, username);
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        session_id
    }

    pub async fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| s.info.clone())
    }

    pub async fn set_session_connected(&self, session_id: &str) -> Result<(), AppError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;
        session.set_connected();
        Ok(())
    }

    pub async fn remove_session(&self, session_id: &str) -> Result<(), AppError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id)
            .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;
        Ok(())
    }

    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.values().map(|s| s.info.clone()).collect()
    }

    // Profile management
    pub async fn save_profile(&self, id: String, mut profile: ProfileData) -> Result<(), AppError> {
        profile.id = Some(id.clone());
        
        let mut profiles = self.profiles.write().await;
        profiles.insert(id, profile);
        
        self.persist_profiles(&profiles).await?;
        Ok(())
    }

    pub async fn update_profile(&self, id: String, profile: ProfileData) -> Result<(), AppError> {
        let mut profiles = self.profiles.write().await;
        if !profiles.contains_key(&id) {
            return Err(AppError::ProfileNotFound(id));
        }
        profiles.insert(id, profile);
        
        self.persist_profiles(&profiles).await?;
        Ok(())
    }

    pub async fn delete_profile(&self, id: &str) -> Result<(), AppError> {
        let mut profiles = self.profiles.write().await;
        profiles.remove(id)
            .ok_or_else(|| AppError::ProfileNotFound(id.to_string()))?;
        
        self.persist_profiles(&profiles).await?;
        Ok(())
    }

    pub async fn list_profiles(&self) -> Vec<ProfileData> {
        let profiles = self.profiles.read().await;
        profiles.values().cloned().collect()
    }

    pub async fn load_profiles(&self) -> Result<(), AppError> {
        let path = self.data_dir.join("profiles.json");
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let profiles: HashMap<String, ProfileData> = serde_json::from_str(&content)?;
            let mut state_profiles = self.profiles.write().await;
            *state_profiles = profiles;
        }
        Ok(())
    }

    async fn persist_profiles(&self, profiles: &HashMap<String, ProfileData>) -> Result<(), AppError> {
        let path = self.data_dir.join("profiles.json");
        let content = serde_json::to_string_pretty(profiles)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub async fn export_profiles(&self, include_credentials: bool) -> Result<String, AppError> {
        let profiles = self.profiles.read().await;
        let export_profiles: Vec<ProfileData> = profiles.values()
            .map(|p| {
                let mut profile = p.clone();
                if !include_credentials {
                    profile.key_path = None;
                }
                profile
            })
            .collect();
        
        Ok(serde_json::to_string_pretty(&export_profiles)?)
    }

    pub async fn import_profiles(&self, json_data: &str) -> Result<usize, AppError> {
        let import_profiles: Vec<ProfileData> = serde_json::from_str(json_data)?;
        let count = import_profiles.len();
        
        let mut profiles = self.profiles.write().await;
        for mut profile in import_profiles {
            let id = profile.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
            profile.id = Some(id.clone());
            profiles.insert(id, profile);
        }
        
        self.persist_profiles(&profiles).await?;
        Ok(count)
    }

    // Settings management
    pub async fn load_settings(&self) -> Result<AppSettings, AppError> {
        let path = self.data_dir.join("settings.json");
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let settings: AppSettings = serde_json::from_str(&content)?;
            let mut state_settings = self.settings.write().await;
            *state_settings = settings.clone();
            Ok(settings)
        } else {
            Ok(AppSettings::default())
        }
    }

    pub async fn save_settings(&self, settings: AppSettings) -> Result<(), AppError> {
        let path = self.data_dir.join("settings.json");
        let content = serde_json::to_string_pretty(&settings)?;
        std::fs::write(path, content)?;
        
        let mut state_settings = self.settings.write().await;
        *state_settings = settings;
        Ok(())
    }

    // P2P management
    pub async fn get_p2p_node_info(&self) -> P2PNodeInfo {
        // Generate a node ID (in real implementation, this would come from iroh)
        P2PNodeInfo {
            node_id: Uuid::new_v4().to_string(),
            relay_url: None,
            direct_addresses: vec![],
            is_online: true,
        }
    }

    pub async fn add_p2p_peer(&self, peer_id: String, peer_info: P2PPeerInfo) {
        let mut peers = self.p2p_peers.write().await;
        peers.insert(peer_id, peer_info);
    }

    pub async fn remove_p2p_peer(&self, peer_id: &str) -> Result<(), AppError> {
        let mut peers = self.p2p_peers.write().await;
        peers.remove(peer_id)
            .ok_or_else(|| AppError::PeerNotFound(peer_id.to_string()))?;
        Ok(())
    }

    pub async fn list_p2p_peers(&self) -> Vec<P2PPeerInfo> {
        let peers = self.p2p_peers.read().await;
        peers.values().cloned().collect()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
