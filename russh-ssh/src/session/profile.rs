//! Session Profile
//!
//! Defines session profiles with all connection parameters.
//!
//! # Requirements Coverage
//! - Requirement 8.1: Session parameter completeness
//! - Requirement 8.2: Session profile serialization

use crate::ssh::{AuthMethod, PortForward};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// Session profile containing all connection parameters
///
/// # Requirements Coverage
/// - Requirement 8.1: Session parameter completeness
/// - Requirement 8.2: Session profile serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProfile {
    /// Unique profile identifier
    pub id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Remote host address
    pub host: String,
    /// Remote port
    pub port: u16,
    /// Username
    pub username: String,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Connection timeout
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    /// Keep-alive interval
    #[serde(with = "option_duration_serde")]
    pub keepalive_interval: Option<Duration>,
    /// Port forwards to establish
    pub port_forwards: Vec<PortForward>,
    /// Environment variables to set
    pub environment: Vec<(String, String)>,
    /// Startup command to run
    pub startup_command: Option<String>,
    /// Working directory on remote
    pub working_directory: Option<String>,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// Use count
    pub use_count: u64,
}

/// Authentication configuration (serializable version)
/// 
/// # Security Warning
/// The `Password` variant stores passwords in plain text when serialized.
/// For production use, consider:
/// - Using `PublicKey` or `Agent` authentication instead
/// - Storing passwords in a secure keyring/credential manager
/// - Prompting for passwords at runtime rather than storing them
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    /// Password authentication
    /// 
    /// WARNING: Passwords are stored in plain text. Use with caution.
    /// Consider using `AuthConfig::password_prompt()` for interactive use.
    Password {
        /// Store password (NOT RECOMMENDED for production - use keyring instead)
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    /// Public key authentication (RECOMMENDED)
    PublicKey {
        /// Path to private key
        key_path: PathBuf,
        /// Whether key is encrypted
        encrypted: bool,
    },
    /// SSH Agent authentication (RECOMMENDED)
    Agent,
}

impl AuthConfig {
    /// Create a password auth config that prompts at runtime (no stored password)
    /// 
    /// This is safer than storing passwords as it requires user input each time.
    pub fn password_prompt() -> Self {
        AuthConfig::Password { password: None }
    }
    
    /// Create a public key auth config
    pub fn public_key(key_path: impl Into<PathBuf>, encrypted: bool) -> Self {
        AuthConfig::PublicKey {
            key_path: key_path.into(),
            encrypted,
        }
    }
    
    /// Create an SSH agent auth config
    pub fn agent() -> Self {
        AuthConfig::Agent
    }
    
    /// Check if this auth config stores sensitive data
    pub fn stores_sensitive_data(&self) -> bool {
        matches!(self, AuthConfig::Password { password: Some(_) })
    }
    
    /// Convert to AuthMethod for connection
    pub fn to_auth_method(&self, password_prompt: Option<&str>) -> Option<AuthMethod> {
        match self {
            AuthConfig::Password { password } => {
                password.as_ref()
                    .or(password_prompt.map(|s| s.to_string()).as_ref())
                    .map(|p| AuthMethod::Password(p.clone()))
            }
            AuthConfig::PublicKey { key_path, encrypted } => {
                Some(AuthMethod::PublicKey {
                    key_path: key_path.clone(),
                    passphrase: if *encrypted { password_prompt.map(|s| s.to_string()) } else { None },
                })
            }
            AuthConfig::Agent => Some(AuthMethod::Agent),
        }
    }
}

impl SessionProfile {
    /// Create a new session profile
    pub fn new(name: String, host: String, username: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            host,
            port: 22,
            username,
            auth: AuthConfig::Agent,
            timeout: Duration::from_secs(30),
            keepalive_interval: Some(Duration::from_secs(60)),
            port_forwards: Vec::new(),
            environment: Vec::new(),
            startup_command: None,
            working_directory: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            last_used: None,
            use_count: 0,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set authentication
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add port forward
    pub fn with_port_forward(mut self, forward: PortForward) -> Self {
        self.port_forwards.push(forward);
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.push((key, value));
        self
    }

    /// Set startup command
    pub fn with_startup_command(mut self, command: String) -> Self {
        self.startup_command = Some(command);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Record usage
    pub fn record_use(&mut self) {
        self.last_used = Some(chrono::Utc::now());
        self.use_count += 1;
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Check if profile has all required parameters
    pub fn is_complete(&self) -> bool {
        !self.name.is_empty() 
            && !self.host.is_empty() 
            && !self.username.is_empty()
            && self.port > 0
    }
}

/// Serde helper for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// Serde helper for Option<Duration>
mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.map(|d| d.as_millis() as u64).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis: Option<u64> = Option::deserialize(deserializer)?;
        Ok(millis.map(Duration::from_millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_profile_creation() {
        let profile = SessionProfile::new(
            "My Server".to_string(),
            "example.com".to_string(),
            "user".to_string(),
        );

        assert!(!profile.id.is_nil());
        assert_eq!(profile.name, "My Server");
        assert_eq!(profile.host, "example.com");
        assert_eq!(profile.username, "user");
        assert_eq!(profile.port, 22);
        assert!(profile.is_complete());
    }

    #[test]
    fn session_profile_builder() {
        let profile = SessionProfile::new(
            "Dev Server".to_string(),
            "dev.example.com".to_string(),
            "developer".to_string(),
        )
        .with_port(2222)
        .with_description("Development server".to_string())
        .with_auth(AuthConfig::PublicKey {
            key_path: PathBuf::from("~/.ssh/id_rsa"),
            encrypted: true,
        })
        .with_tag("development".to_string());

        assert_eq!(profile.port, 2222);
        assert_eq!(profile.description, Some("Development server".to_string()));
        assert!(profile.tags.contains(&"development".to_string()));
    }

    #[test]
    fn session_profile_serialization_roundtrip() {
        let profile = SessionProfile::new(
            "Test Server".to_string(),
            "test.example.com".to_string(),
            "tester".to_string(),
        )
        .with_port(22)
        .with_timeout(Duration::from_secs(60));

        let json = profile.to_json().unwrap();
        let restored = SessionProfile::from_json(&json).unwrap();

        assert_eq!(restored.id, profile.id);
        assert_eq!(restored.name, profile.name);
        assert_eq!(restored.host, profile.host);
        assert_eq!(restored.username, profile.username);
        assert_eq!(restored.port, profile.port);
        assert_eq!(restored.timeout, profile.timeout);
    }

    #[test]
    fn session_profile_completeness() {
        let complete = SessionProfile::new(
            "Complete".to_string(),
            "host.com".to_string(),
            "user".to_string(),
        );
        assert!(complete.is_complete());

        let mut incomplete = complete.clone();
        incomplete.name = String::new();
        assert!(!incomplete.is_complete());
    }

    #[test]
    fn session_profile_usage_tracking() {
        let mut profile = SessionProfile::new(
            "Server".to_string(),
            "host.com".to_string(),
            "user".to_string(),
        );

        assert_eq!(profile.use_count, 0);
        assert!(profile.last_used.is_none());

        profile.record_use();
        assert_eq!(profile.use_count, 1);
        assert!(profile.last_used.is_some());

        profile.record_use();
        assert_eq!(profile.use_count, 2);
    }
}
