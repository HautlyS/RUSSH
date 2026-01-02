//! Session Manager
//!
//! Manages session profiles and active sessions.
//!
//! # Requirements Coverage
//! - Requirement 8.3: Session management
//! - Requirement 8.4: Session persistence

use super::profile::SessionProfile;
use crate::error::SessionError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Session statistics
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    /// Total sessions created
    pub total_created: u64,
    /// Currently active sessions
    pub active_count: usize,
    /// Total bytes transferred
    pub bytes_transferred: u64,
    /// Total commands executed
    pub commands_executed: u64,
}

/// Active session information
#[derive(Debug)]
pub struct ActiveSession {
    /// Session ID
    pub id: Uuid,
    /// Profile used
    pub profile_id: Uuid,
    /// Start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Commands executed
    pub commands_executed: u64,
}

impl ActiveSession {
    fn new(profile_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            profile_id,
            started_at: chrono::Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
            commands_executed: 0,
        }
    }

    /// Get session duration
    pub fn duration(&self) -> chrono::Duration {
        chrono::Utc::now() - self.started_at
    }
}

/// Session manager for profiles and active sessions
pub struct SessionManager {
    /// Stored profiles
    profiles: RwLock<HashMap<Uuid, SessionProfile>>,
    /// Active sessions
    active_sessions: RwLock<HashMap<Uuid, ActiveSession>>,
    /// Storage path for persistence
    storage_path: Option<PathBuf>,
    /// Statistics
    stats: RwLock<SessionStats>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            profiles: RwLock::new(HashMap::new()),
            active_sessions: RwLock::new(HashMap::new()),
            storage_path: None,
            stats: RwLock::new(SessionStats::default()),
        }
    }

    /// Create with persistence path
    pub fn with_storage(path: PathBuf) -> Self {
        Self {
            profiles: RwLock::new(HashMap::new()),
            active_sessions: RwLock::new(HashMap::new()),
            storage_path: Some(path),
            stats: RwLock::new(SessionStats::default()),
        }
    }

    /// Add a profile
    pub async fn add_profile(&self, profile: SessionProfile) -> Uuid {
        let id = profile.id;
        let mut profiles = self.profiles.write().await;
        profiles.insert(id, profile);
        id
    }

    /// Get a profile by ID
    pub async fn get_profile(&self, id: &Uuid) -> Option<SessionProfile> {
        let profiles = self.profiles.read().await;
        profiles.get(id).cloned()
    }

    /// Get a profile by name
    pub async fn get_profile_by_name(&self, name: &str) -> Option<SessionProfile> {
        let profiles = self.profiles.read().await;
        profiles.values().find(|p| p.name == name).cloned()
    }

    /// Update a profile
    pub async fn update_profile(&self, profile: SessionProfile) -> Result<(), SessionError> {
        let mut profiles = self.profiles.write().await;
        if profiles.contains_key(&profile.id) {
            profiles.insert(profile.id, profile);
            Ok(())
        } else {
            Err(SessionError::ProfileNotFound(profile.id.to_string()))
        }
    }

    /// Remove a profile
    pub async fn remove_profile(&self, id: &Uuid) -> Result<SessionProfile, SessionError> {
        let mut profiles = self.profiles.write().await;
        profiles.remove(id)
            .ok_or_else(|| SessionError::ProfileNotFound(id.to_string()))
    }

    /// List all profiles
    pub async fn list_profiles(&self) -> Vec<SessionProfile> {
        let profiles = self.profiles.read().await;
        profiles.values().cloned().collect()
    }

    /// Search profiles by tag
    pub async fn search_by_tag(&self, tag: &str) -> Vec<SessionProfile> {
        let profiles = self.profiles.read().await;
        profiles.values()
            .filter(|p| p.tags.iter().any(|t| t.contains(tag)))
            .cloned()
            .collect()
    }

    /// Create a new session from a profile
    pub async fn create_session(&self, profile_id: &Uuid) -> Result<Uuid, SessionError> {
        // Verify profile exists
        {
            let profiles = self.profiles.read().await;
            if !profiles.contains_key(profile_id) {
                return Err(SessionError::ProfileNotFound(profile_id.to_string()));
            }
        }

        // Create active session
        let session = ActiveSession::new(*profile_id);
        let session_id = session.id;

        {
            let mut active = self.active_sessions.write().await;
            active.insert(session_id, session);
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_created += 1;
            stats.active_count += 1;
        }

        // Update profile usage
        {
            let mut profiles = self.profiles.write().await;
            if let Some(profile) = profiles.get_mut(profile_id) {
                profile.record_use();
            }
        }

        Ok(session_id)
    }

    /// Close a session
    pub async fn close_session(&self, session_id: &Uuid) -> Result<(), SessionError> {
        let session = {
            let mut active = self.active_sessions.write().await;
            active.remove(session_id)
        };

        match session {
            Some(s) => {
                let mut stats = self.stats.write().await;
                stats.active_count -= 1;
                stats.bytes_transferred += s.bytes_sent + s.bytes_received;
                stats.commands_executed += s.commands_executed;
                Ok(())
            }
            None => Err(SessionError::NotFound(session_id.to_string())),
        }
    }

    /// Get active session info
    pub async fn get_session(&self, session_id: &Uuid) -> Option<(Uuid, Uuid, chrono::DateTime<chrono::Utc>)> {
        let active = self.active_sessions.read().await;
        active.get(session_id).map(|s| (s.id, s.profile_id, s.started_at))
    }

    /// List active sessions
    pub async fn list_active_sessions(&self) -> Vec<Uuid> {
        let active = self.active_sessions.read().await;
        active.keys().cloned().collect()
    }

    /// Get statistics
    pub async fn stats(&self) -> SessionStats {
        let stats = self.stats.read().await;
        let active = self.active_sessions.read().await;
        SessionStats {
            active_count: active.len(),
            ..*stats
        }
    }

    /// Save profiles to disk
    ///
    /// # Requirements Coverage
    /// - Requirement 8.4: Session persistence
    pub async fn save(&self) -> Result<(), SessionError> {
        let path = self.storage_path.as_ref()
            .ok_or_else(|| SessionError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No storage path configured"
            )))?;

        let profiles = self.profiles.read().await;
        let profiles_vec: Vec<&SessionProfile> = profiles.values().collect();
        let json = serde_json::to_string_pretty(&profiles_vec)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Load profiles from disk
    ///
    /// # Requirements Coverage
    /// - Requirement 8.4: Session persistence
    pub async fn load(&self) -> Result<(), SessionError> {
        let path = self.storage_path.as_ref()
            .ok_or_else(|| SessionError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No storage path configured"
            )))?;

        if !path.exists() {
            return Ok(()); // No saved profiles yet
        }

        let json = tokio::fs::read_to_string(path).await?;
        let profiles_vec: Vec<SessionProfile> = serde_json::from_str(&json)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        let mut profiles = self.profiles.write().await;
        for profile in profiles_vec {
            profiles.insert(profile.id, profile);
        }

        Ok(())
    }

    /// Import profiles from a file
    pub async fn import(&self, path: &Path) -> Result<usize, SessionError> {
        let json = tokio::fs::read_to_string(path).await?;
        let profiles_vec: Vec<SessionProfile> = serde_json::from_str(&json)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        let count = profiles_vec.len();
        let mut profiles = self.profiles.write().await;
        for profile in profiles_vec {
            profiles.insert(profile.id, profile);
        }

        Ok(count)
    }

    /// Export profiles to a file
    pub async fn export(&self, path: &Path) -> Result<usize, SessionError> {
        let profiles = self.profiles.read().await;
        let profiles_vec: Vec<&SessionProfile> = profiles.values().collect();
        let count = profiles_vec.len();

        let json = serde_json::to_string_pretty(&profiles_vec)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        tokio::fs::write(path, json).await?;
        Ok(count)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn session_manager_profile_crud() {
        let manager = SessionManager::new();

        // Create
        let profile = SessionProfile::new(
            "Test".to_string(),
            "host.com".to_string(),
            "user".to_string(),
        );
        let id = manager.add_profile(profile.clone()).await;

        // Read
        let retrieved = manager.get_profile(&id).await.unwrap();
        assert_eq!(retrieved.name, "Test");

        // Update
        let mut updated = retrieved.clone();
        updated.name = "Updated".to_string();
        manager.update_profile(updated).await.unwrap();

        let retrieved = manager.get_profile(&id).await.unwrap();
        assert_eq!(retrieved.name, "Updated");

        // Delete
        manager.remove_profile(&id).await.unwrap();
        assert!(manager.get_profile(&id).await.is_none());
    }

    #[tokio::test]
    async fn session_manager_sessions() {
        let manager = SessionManager::new();

        let profile = SessionProfile::new(
            "Test".to_string(),
            "host.com".to_string(),
            "user".to_string(),
        );
        let profile_id = manager.add_profile(profile).await;

        // Create session
        let session_id = manager.create_session(&profile_id).await.unwrap();
        
        let stats = manager.stats().await;
        assert_eq!(stats.active_count, 1);
        assert_eq!(stats.total_created, 1);

        // Close session
        manager.close_session(&session_id).await.unwrap();
        
        let stats = manager.stats().await;
        assert_eq!(stats.active_count, 0);
    }

    #[tokio::test]
    async fn session_manager_search() {
        let manager = SessionManager::new();

        let profile1 = SessionProfile::new(
            "Dev Server".to_string(),
            "dev.com".to_string(),
            "user".to_string(),
        ).with_tag("development".to_string());

        let profile2 = SessionProfile::new(
            "Prod Server".to_string(),
            "prod.com".to_string(),
            "user".to_string(),
        ).with_tag("production".to_string());

        manager.add_profile(profile1).await;
        manager.add_profile(profile2).await;

        let dev_profiles = manager.search_by_tag("dev").await;
        assert_eq!(dev_profiles.len(), 1);
        assert_eq!(dev_profiles[0].name, "Dev Server");
    }

    #[tokio::test]
    async fn session_manager_get_by_name() {
        let manager = SessionManager::new();

        let profile = SessionProfile::new(
            "Named Server".to_string(),
            "host.com".to_string(),
            "user".to_string(),
        );
        manager.add_profile(profile).await;

        let found = manager.get_profile_by_name("Named Server").await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().host, "host.com");

        let not_found = manager.get_profile_by_name("Unknown").await;
        assert!(not_found.is_none());
    }
}
