//! Property-based tests for Session Management
//!
//! Feature: russh-ssh
//! These tests validate the correctness properties of session management.

use russh_ssh::session::{SessionProfile, SessionManager};
use russh_ssh::session::profile::AuthConfig;
use russh_ssh::ssh::PortForward;
use proptest::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

/// Strategy for generating arbitrary host names
fn arb_host() -> impl Strategy<Value = String> {
    "[a-z]{3,10}\\.[a-z]{2,5}".prop_map(|s| s)
}

/// Strategy for generating arbitrary usernames
fn arb_username() -> impl Strategy<Value = String> {
    "[a-z_][a-z0-9_]{2,15}".prop_map(|s| s)
}

/// Strategy for generating arbitrary profile names
fn arb_profile_name() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 _-]{2,20}".prop_map(|s| s)
}

/// Strategy for generating arbitrary AuthConfig
fn arb_auth_config() -> impl Strategy<Value = AuthConfig> {
    prop_oneof![
        Just(AuthConfig::Agent),
        prop::option::of("[a-zA-Z0-9]{8,20}")
            .prop_map(|password| AuthConfig::Password { password }),
        ("[a-z/_]{5,30}", any::<bool>())
            .prop_map(|(path, encrypted)| AuthConfig::PublicKey {
                key_path: PathBuf::from(path),
                encrypted,
            }),
    ]
}

/// Strategy for generating arbitrary port forwards
fn arb_port_forward() -> impl Strategy<Value = PortForward> {
    prop_oneof![
        (1024u16..65535, arb_host(), 1u16..65535)
            .prop_map(|(local_port, remote_host, remote_port)| PortForward::Local {
                local_port,
                remote_host,
                remote_port,
            }),
        (1u16..65535, "localhost|127\\.0\\.0\\.1".prop_map(|s| s.to_string()), 1024u16..65535)
            .prop_map(|(remote_port, local_host, local_port)| PortForward::Remote {
                remote_port,
                local_host,
                local_port,
            }),
        (1024u16..65535).prop_map(|local_port| PortForward::Dynamic { local_port }),
    ]
}

/// Strategy for generating arbitrary SessionProfile
fn arb_session_profile() -> impl Strategy<Value = SessionProfile> {
    (
        arb_profile_name(),
        arb_host(),
        arb_username(),
        1u16..65535,
        arb_auth_config(),
        1000u64..120000,
        prop::collection::vec(arb_port_forward(), 0..3),
        prop::collection::vec(("[A-Z_]{3,10}", "[a-zA-Z0-9]{1,20}"), 0..3),
        prop::option::of("[a-z ]{5,50}"),
        prop::collection::vec("[a-z]{3,10}", 0..5),
    ).prop_map(|(name, host, username, port, auth, timeout_ms, forwards, env, desc, tags)| {
        let mut profile = SessionProfile::new(name, host, username)
            .with_port(port)
            .with_auth(auth)
            .with_timeout(Duration::from_millis(timeout_ms));
        
        for forward in forwards {
            profile = profile.with_port_forward(forward);
        }
        
        for (key, value) in env {
            profile = profile.with_env(key, value);
        }
        
        if let Some(description) = desc {
            profile = profile.with_description(description);
        }
        
        for tag in tags {
            profile = profile.with_tag(tag);
        }
        
        profile
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 10: Session Parameter Completeness
    ///
    /// *For any* valid session creation request, the resulting Session SHALL
    /// contain all specified connection parameters (host, port, authentication
    /// method) accessible and unchanged.
    ///
    /// **Validates: Requirements 8.1**
    #[test]
    fn session_parameter_completeness(
        name in arb_profile_name(),
        host in arb_host(),
        username in arb_username(),
        port in 1u16..65535,
        timeout_ms in 1000u64..120000,
    ) {
        let profile = SessionProfile::new(name.clone(), host.clone(), username.clone())
            .with_port(port)
            .with_timeout(Duration::from_millis(timeout_ms));
        
        // All parameters should be accessible and unchanged
        prop_assert_eq!(
            &profile.name, &name,
            "Name should be preserved"
        );
        prop_assert_eq!(
            &profile.host, &host,
            "Host should be preserved"
        );
        prop_assert_eq!(
            &profile.username, &username,
            "Username should be preserved"
        );
        prop_assert_eq!(
            profile.port, port,
            "Port should be preserved"
        );
        prop_assert_eq!(
            profile.timeout, Duration::from_millis(timeout_ms),
            "Timeout should be preserved"
        );
        
        // Profile should be complete
        prop_assert!(
            profile.is_complete(),
            "Profile with all required fields should be complete"
        );
    }

    /// Feature: russh-ssh, Property 10 (continued): Auth config preservation
    ///
    /// *For any* authentication configuration, the session SHALL preserve
    /// the authentication method exactly as specified.
    ///
    /// **Validates: Requirements 8.1**
    #[test]
    fn session_auth_config_preserved(
        name in arb_profile_name(),
        host in arb_host(),
        username in arb_username(),
        auth in arb_auth_config(),
    ) {
        let profile = SessionProfile::new(name, host, username)
            .with_auth(auth.clone());
        
        // Auth config should match
        match (&profile.auth, &auth) {
            (AuthConfig::Agent, AuthConfig::Agent) => {}
            (AuthConfig::Password { password: p1 }, AuthConfig::Password { password: p2 }) => {
                prop_assert_eq!(p1, p2, "Password should be preserved");
            }
            (AuthConfig::PublicKey { key_path: k1, encrypted: e1 }, 
             AuthConfig::PublicKey { key_path: k2, encrypted: e2 }) => {
                prop_assert_eq!(k1, k2, "Key path should be preserved");
                prop_assert_eq!(e1, e2, "Encrypted flag should be preserved");
            }
            _ => prop_assert!(false, "Auth config type mismatch"),
        }
    }

    /// Feature: russh-ssh, Property 10 (continued): Port forwards preservation
    ///
    /// *For any* set of port forwards, the session SHALL preserve all forwards
    /// with their exact configuration.
    ///
    /// **Validates: Requirements 8.1**
    #[test]
    fn session_port_forwards_preserved(
        name in arb_profile_name(),
        host in arb_host(),
        username in arb_username(),
        forwards in prop::collection::vec(arb_port_forward(), 0..5),
    ) {
        let mut profile = SessionProfile::new(name, host, username);
        
        for forward in &forwards {
            profile = profile.with_port_forward(forward.clone());
        }
        
        prop_assert_eq!(
            profile.port_forwards.len(), forwards.len(),
            "Number of port forwards should be preserved"
        );
        
        for (stored, original) in profile.port_forwards.iter().zip(forwards.iter()) {
            match (stored, original) {
                (PortForward::Local { local_port: lp1, remote_host: rh1, remote_port: rp1 },
                 PortForward::Local { local_port: lp2, remote_host: rh2, remote_port: rp2 }) => {
                    prop_assert_eq!(lp1, lp2);
                    prop_assert_eq!(rh1, rh2);
                    prop_assert_eq!(rp1, rp2);
                }
                (PortForward::Remote { remote_port: rp1, local_host: lh1, local_port: lp1 },
                 PortForward::Remote { remote_port: rp2, local_host: lh2, local_port: lp2 }) => {
                    prop_assert_eq!(rp1, rp2);
                    prop_assert_eq!(lh1, lh2);
                    prop_assert_eq!(lp1, lp2);
                }
                (PortForward::Dynamic { local_port: lp1 },
                 PortForward::Dynamic { local_port: lp2 }) => {
                    prop_assert_eq!(lp1, lp2);
                }
                _ => prop_assert!(false, "Port forward type mismatch"),
            }
        }
    }
}


proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 11: Session Profile Round-Trip
    ///
    /// *For any* valid SessionProfile, saving then loading the profile by name
    /// SHALL produce an equivalent SessionProfile with all fields preserved.
    ///
    /// **Validates: Requirements 8.2**
    #[test]
    fn session_profile_roundtrip(profile in arb_session_profile()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create a temporary directory for storage
            let temp_dir = tempfile::tempdir().unwrap();
            let storage_path = temp_dir.path().join("profiles.json");
            
            let manager = SessionManager::with_storage(storage_path);
            
            // Add profile
            let id = manager.add_profile(profile.clone()).await;
            
            // Save to disk
            manager.save().await.unwrap();
            
            // Create new manager and load
            let manager2 = SessionManager::with_storage(temp_dir.path().join("profiles.json"));
            manager2.load().await.unwrap();
            
            // Retrieve by ID
            let loaded = manager2.get_profile(&id).await;
            prop_assert!(loaded.is_some(), "Profile should be loadable after save");
            
            let loaded = loaded.unwrap();
            
            // All fields should be preserved
            prop_assert_eq!(loaded.id, profile.id, "ID should be preserved");
            prop_assert_eq!(loaded.name, profile.name, "Name should be preserved");
            prop_assert_eq!(loaded.host, profile.host, "Host should be preserved");
            prop_assert_eq!(loaded.username, profile.username, "Username should be preserved");
            prop_assert_eq!(loaded.port, profile.port, "Port should be preserved");
            prop_assert_eq!(loaded.timeout, profile.timeout, "Timeout should be preserved");
            prop_assert_eq!(
                loaded.port_forwards.len(), profile.port_forwards.len(),
                "Port forwards count should be preserved"
            );
            prop_assert_eq!(
                loaded.environment.len(), profile.environment.len(),
                "Environment count should be preserved"
            );
            prop_assert_eq!(loaded.tags, profile.tags, "Tags should be preserved");
            prop_assert_eq!(loaded.description, profile.description, "Description should be preserved");
            
            Ok(())
        })?;
    }

    /// Feature: russh-ssh, Property 11 (continued): Profile retrieval by name
    ///
    /// *For any* saved profile, retrieving by name SHALL return the same profile.
    ///
    /// **Validates: Requirements 8.2**
    #[test]
    fn session_profile_retrieval_by_name(profile in arb_session_profile()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = SessionManager::new();
            
            let original_name = profile.name.clone();
            manager.add_profile(profile.clone()).await;
            
            // Retrieve by name
            let retrieved = manager.get_profile_by_name(&original_name).await;
            prop_assert!(retrieved.is_some(), "Profile should be retrievable by name");
            
            let retrieved = retrieved.unwrap();
            prop_assert_eq!(retrieved.id, profile.id, "Retrieved profile should have same ID");
            prop_assert_eq!(retrieved.name, original_name, "Retrieved profile should have same name");
            
            Ok(())
        })?;
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 12: Session Serialization Round-Trip
    ///
    /// *For any* valid Session object, serializing to JSON then deserializing
    /// SHALL produce an equivalent Session object with all state preserved.
    ///
    /// **Validates: Requirements 8.7**
    #[test]
    fn session_serialization_roundtrip(profile in arb_session_profile()) {
        // Serialize to JSON
        let json = profile.to_json().unwrap();
        
        // Deserialize back
        let restored = SessionProfile::from_json(&json).unwrap();
        
        // All fields should be preserved
        prop_assert_eq!(restored.id, profile.id, "ID should be preserved");
        prop_assert_eq!(restored.name, profile.name, "Name should be preserved");
        prop_assert_eq!(restored.host, profile.host, "Host should be preserved");
        prop_assert_eq!(restored.username, profile.username, "Username should be preserved");
        prop_assert_eq!(restored.port, profile.port, "Port should be preserved");
        prop_assert_eq!(restored.timeout, profile.timeout, "Timeout should be preserved");
        prop_assert_eq!(
            restored.port_forwards.len(), profile.port_forwards.len(),
            "Port forwards count should be preserved"
        );
        prop_assert_eq!(
            restored.environment.len(), profile.environment.len(),
            "Environment count should be preserved"
        );
        prop_assert_eq!(restored.tags, profile.tags, "Tags should be preserved");
        prop_assert_eq!(restored.description, profile.description, "Description should be preserved");
        prop_assert_eq!(restored.use_count, profile.use_count, "Use count should be preserved");
    }

    /// Feature: russh-ssh, Property 12 (continued): JSON format is valid
    ///
    /// *For any* SessionProfile, the serialized JSON SHALL be valid JSON
    /// that can be parsed by standard JSON parsers.
    ///
    /// **Validates: Requirements 8.7**
    #[test]
    fn session_json_is_valid(profile in arb_session_profile()) {
        let json = profile.to_json().unwrap();
        
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        // Should be an object
        prop_assert!(parsed.is_object(), "Serialized profile should be a JSON object");
        
        // Should contain required fields
        let obj = parsed.as_object().unwrap();
        prop_assert!(obj.contains_key("id"), "JSON should contain 'id' field");
        prop_assert!(obj.contains_key("name"), "JSON should contain 'name' field");
        prop_assert!(obj.contains_key("host"), "JSON should contain 'host' field");
        prop_assert!(obj.contains_key("username"), "JSON should contain 'username' field");
        prop_assert!(obj.contains_key("port"), "JSON should contain 'port' field");
    }

    /// Feature: russh-ssh, Property 12 (continued): Multiple serialization roundtrips
    ///
    /// *For any* SessionProfile, multiple serialization/deserialization cycles
    /// SHALL produce identical results (stability).
    ///
    /// **Validates: Requirements 8.7**
    #[test]
    fn session_multiple_roundtrips_stable(profile in arb_session_profile()) {
        // First roundtrip
        let json1 = profile.to_json().unwrap();
        let restored1 = SessionProfile::from_json(&json1).unwrap();
        
        // Second roundtrip
        let json2 = restored1.to_json().unwrap();
        let restored2 = SessionProfile::from_json(&json2).unwrap();
        
        // Third roundtrip
        let json3 = restored2.to_json().unwrap();
        let restored3 = SessionProfile::from_json(&json3).unwrap();
        
        // All should be equivalent
        prop_assert_eq!(restored1.id, restored2.id, "ID should be stable across roundtrips");
        prop_assert_eq!(restored2.id, restored3.id, "ID should be stable across roundtrips");
        prop_assert_eq!(restored1.name, restored3.name, "Name should be stable across roundtrips");
        prop_assert_eq!(restored1.host, restored3.host, "Host should be stable across roundtrips");
        
        // JSON should be identical after first roundtrip
        prop_assert_eq!(json2, json3, "JSON should be stable after first roundtrip");
    }
}

/// Additional unit tests for edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn empty_profile_name_is_incomplete() {
        let profile = SessionProfile::new(
            String::new(),
            "host.com".to_string(),
            "user".to_string(),
        );
        assert!(!profile.is_complete());
    }

    #[test]
    fn empty_host_is_incomplete() {
        let profile = SessionProfile::new(
            "Name".to_string(),
            String::new(),
            "user".to_string(),
        );
        assert!(!profile.is_complete());
    }

    #[test]
    fn empty_username_is_incomplete() {
        let profile = SessionProfile::new(
            "Name".to_string(),
            "host.com".to_string(),
            String::new(),
        );
        assert!(!profile.is_complete());
    }

    #[test]
    fn usage_tracking_increments() {
        let mut profile = SessionProfile::new(
            "Test".to_string(),
            "host.com".to_string(),
            "user".to_string(),
        );
        
        assert_eq!(profile.use_count, 0);
        assert!(profile.last_used.is_none());
        
        profile.record_use();
        assert_eq!(profile.use_count, 1);
        assert!(profile.last_used.is_some());
        
        let first_use = profile.last_used;
        
        profile.record_use();
        assert_eq!(profile.use_count, 2);
        assert!(profile.last_used >= first_use);
    }

    #[tokio::test]
    async fn session_manager_multiple_profiles() {
        let manager = SessionManager::new();
        
        let profile1 = SessionProfile::new(
            "Server1".to_string(),
            "host1.com".to_string(),
            "user1".to_string(),
        );
        let profile2 = SessionProfile::new(
            "Server2".to_string(),
            "host2.com".to_string(),
            "user2".to_string(),
        );
        
        let id1 = manager.add_profile(profile1).await;
        let id2 = manager.add_profile(profile2).await;
        
        assert_ne!(id1, id2);
        
        let profiles = manager.list_profiles().await;
        assert_eq!(profiles.len(), 2);
    }
}
