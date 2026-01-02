//! Configuration types for the russh SSH library
//!
//! This module defines configuration structs for connections,
//! reconnection strategies, and other configurable behaviors.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for connection behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Remote host address
    pub host: String,
    /// Remote port number
    pub port: u16,
    /// Connection timeout duration
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    /// TCP keepalive interval
    #[serde(with = "duration_serde")]
    pub keepalive_interval: Duration,
    /// Maximum number of reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Base delay for reconnection backoff
    #[serde(with = "duration_serde")]
    pub reconnect_base_delay: Duration,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            timeout: Duration::from_secs(30),
            keepalive_interval: Duration::from_secs(60),
            max_reconnect_attempts: 5,
            reconnect_base_delay: Duration::from_secs(1),
        }
    }
}

impl ConnectionConfig {
    /// Create a new connection config with the given host and port
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            ..Default::default()
        }
    }

    /// Set the connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the keepalive interval
    pub fn with_keepalive(mut self, interval: Duration) -> Self {
        self.keepalive_interval = interval;
        self
    }

    /// Set the maximum reconnection attempts
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }
}

/// Reconnection strategy configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectionStrategy {
    /// Maximum number of reconnection attempts
    pub max_attempts: u32,
    /// Base delay between reconnection attempts
    #[serde(with = "duration_serde")]
    pub base_delay: Duration,
    /// Maximum delay between reconnection attempts
    #[serde(with = "duration_serde")]
    pub max_delay: Duration,
    /// Whether to add random jitter to delays
    pub jitter: bool,
}

impl Default for ReconnectionStrategy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            jitter: true,
        }
    }
}

impl ReconnectionStrategy {
    /// Create a new reconnection strategy
    pub fn new(max_attempts: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay,
            jitter: true,
        }
    }

    /// Disable jitter for deterministic delays
    pub fn without_jitter(mut self) -> Self {
        self.jitter = false;
        self
    }

    /// Enable jitter for randomized delays
    pub fn with_jitter(mut self) -> Self {
        self.jitter = true;
        self
    }

    /// Calculate delay for given attempt number using exponential backoff
    ///
    /// The delay is calculated as: base_delay × 2^attempt, capped at max_delay.
    /// If jitter is enabled, random jitter up to 25% of the delay is added.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        // Cap the exponent to prevent overflow (2^10 = 1024 is reasonable)
        let capped_attempt = attempt.min(10);
        
        // Calculate exponential delay: base_delay * 2^attempt
        let multiplier = 2u32.saturating_pow(capped_attempt);
        let exponential = self.base_delay.saturating_mul(multiplier);
        
        // Cap at max_delay
        let capped = if exponential > self.max_delay {
            self.max_delay
        } else {
            exponential
        };

        // Add jitter if enabled (up to 25% of delay)
        if self.jitter {
            let jitter_max = capped.as_millis() as u64 / 4;
            if jitter_max > 0 {
                let jitter = rand::random::<u64>() % jitter_max;
                capped + Duration::from_millis(jitter)
            } else {
                capped
            }
        } else {
            capped
        }
    }

    /// Check if more attempts are allowed
    pub fn should_retry(&self, current_attempt: u32) -> bool {
        current_attempt < self.max_attempts
    }
}

/// SSH authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Password-based authentication
    Password(String),
    /// Public key authentication
    PublicKey {
        /// Path to the private key file
        key_path: PathBuf,
        /// Optional passphrase for encrypted keys
        passphrase: Option<String>,
    },
    /// SSH agent authentication
    Agent,
}

/// SSH session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    /// Remote host address
    pub host: String,
    /// Remote port number
    pub port: u16,
    /// Username for authentication
    pub username: String,
    /// Authentication method
    pub auth: AuthMethod,
    /// Connection timeout
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
}

impl SshConfig {
    /// Create a new SSH config with password authentication
    pub fn with_password(
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            username: username.into(),
            auth: AuthMethod::Password(password.into()),
            timeout: Duration::from_secs(30),
        }
    }

    /// Create a new SSH config with public key authentication
    pub fn with_key(
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
        key_path: impl Into<PathBuf>,
        passphrase: Option<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            username: username.into(),
            auth: AuthMethod::PublicKey {
                key_path: key_path.into(),
                passphrase,
            },
            timeout: Duration::from_secs(30),
        }
    }

    /// Create a new SSH config with SSH agent authentication
    pub fn with_agent(
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            username: username.into(),
            auth: AuthMethod::Agent,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set the connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Serde helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    #[derive(Serialize, Deserialize)]
    struct DurationHelper {
        secs: u64,
        nanos: u32,
    }

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let helper = DurationHelper {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        };
        helper.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = DurationHelper::deserialize(deserializer)?;
        Ok(Duration::new(helper.secs, helper.nanos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_config_default() {
        let config = ConnectionConfig::default();
        assert_eq!(config.port, 22);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn connection_config_builder() {
        let config = ConnectionConfig::new("example.com", 2222)
            .with_timeout(Duration::from_secs(60))
            .with_keepalive(Duration::from_secs(30))
            .with_max_reconnect_attempts(10);

        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 2222);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.keepalive_interval, Duration::from_secs(30));
        assert_eq!(config.max_reconnect_attempts, 10);
    }

    #[test]
    fn reconnection_strategy_default() {
        let strategy = ReconnectionStrategy::default();
        assert_eq!(strategy.max_attempts, 5);
        assert!(strategy.jitter);
    }

    #[test]
    fn ssh_config_with_password() {
        let config = SshConfig::with_password("host.com", 22, "user", "pass");
        assert_eq!(config.host, "host.com");
        assert_eq!(config.username, "user");
        assert!(matches!(config.auth, AuthMethod::Password(_)));
    }

    #[test]
    fn ssh_config_serialization_roundtrip() {
        let config = SshConfig::with_password("host.com", 22, "user", "pass")
            .with_timeout(Duration::from_secs(45));

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SshConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.host, deserialized.host);
        assert_eq!(config.port, deserialized.port);
        assert_eq!(config.username, deserialized.username);
        assert_eq!(config.timeout, deserialized.timeout);
    }
}
