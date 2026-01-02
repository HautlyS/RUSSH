//! Property-based tests for connection state management
//!
//! Feature: russh-ssh
//! These tests validate the correctness properties of the state machine.

use russh_ssh::connection::state::{ConnectionState, StateManager};
use proptest::prelude::*;

/// Strategy for generating valid connection states
fn connection_state_strategy() -> impl Strategy<Value = ConnectionState> {
    prop_oneof![
        Just(ConnectionState::Disconnected),
        Just(ConnectionState::Connecting),
        Just(ConnectionState::Connected),
        (1u32..10).prop_map(|attempt| ConnectionState::Reconnecting { attempt }),
        "[a-z]{5,20}".prop_map(|reason| ConnectionState::Failed { reason }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property: State Transition Validity
    ///
    /// *For any* valid state transition, the state manager SHALL accept it
    /// and update the state accordingly.
    ///
    /// **Validates: Requirements 2.4 - State management**
    #[test]
    fn valid_transitions_succeed(
        initial in connection_state_strategy(),
    ) {
        let manager = StateManager::with_state(initial.clone());
        
        // Get valid next states based on current state
        let valid_next_states: Vec<ConnectionState> = match &initial {
            ConnectionState::Disconnected => vec![
                ConnectionState::Connecting,
                ConnectionState::Failed { reason: "test".to_string() },
            ],
            ConnectionState::Connecting => vec![
                ConnectionState::Connected,
                ConnectionState::Failed { reason: "test".to_string() },
                ConnectionState::Disconnected,
            ],
            ConnectionState::Connected => vec![
                ConnectionState::Disconnected,
                ConnectionState::Failed { reason: "test".to_string() },
                ConnectionState::Reconnecting { attempt: 1 },
            ],
            ConnectionState::Reconnecting { .. } => vec![
                ConnectionState::Connected,
                ConnectionState::Failed { reason: "test".to_string() },
                ConnectionState::Reconnecting { attempt: 2 },
                ConnectionState::Disconnected,
            ],
            ConnectionState::Failed { .. } => vec![
                ConnectionState::Connecting,
                ConnectionState::Disconnected,
            ],
        };
        
        for next_state in valid_next_states {
            let result = manager.try_transition(next_state.clone());
            prop_assert!(
                result.is_ok(),
                "Valid transition from {:?} to {:?} must succeed",
                initial, next_state
            );
            
            // Reset for next test
            let _ = manager.set_state(initial.clone());
        }
    }

    /// Feature: russh-ssh, Property: Invalid Transitions Rejected
    ///
    /// *For any* invalid state transition, the state manager SHALL reject it
    /// and maintain the current state.
    ///
    /// **Validates: Requirements 2.4 - State integrity**
    #[test]
    fn invalid_transitions_rejected(
        initial in connection_state_strategy(),
    ) {
        let manager = StateManager::with_state(initial.clone());
        
        // Get invalid next states based on current state
        let invalid_next_states: Vec<ConnectionState> = match &initial {
            ConnectionState::Disconnected => vec![
                ConnectionState::Connected, // Must go through Connecting
                ConnectionState::Reconnecting { attempt: 1 }, // Must be connected first
            ],
            ConnectionState::Connecting => vec![
                ConnectionState::Reconnecting { attempt: 1 }, // Must be connected first
            ],
            ConnectionState::Connected => vec![
                ConnectionState::Connecting, // Already connected
            ],
            ConnectionState::Reconnecting { .. } => vec![
                ConnectionState::Connecting, // Must succeed or fail
            ],
            ConnectionState::Failed { .. } => vec![
                ConnectionState::Connected, // Must go through Connecting
                ConnectionState::Reconnecting { attempt: 1 }, // Must be connected first
            ],
        };
        
        for next_state in invalid_next_states {
            let initial_clone = initial.clone();
            let result = manager.try_transition(next_state.clone());
            prop_assert!(
                result.is_err(),
                "Invalid transition from {:?} to {:?} must fail",
                initial_clone, next_state
            );
            
            // State should remain unchanged
            prop_assert_eq!(
                manager.state(), initial.clone(),
                "State must remain unchanged after invalid transition"
            );
        }
    }

    /// Feature: russh-ssh, Property: Same State No-Op
    ///
    /// *For any* state, transitioning to the same state SHALL be a no-op
    /// and return false.
    ///
    /// **Validates: Requirements 2.4 - Idempotent state setting**
    #[test]
    fn same_state_is_noop(
        state in connection_state_strategy(),
    ) {
        let manager = StateManager::with_state(state.clone());
        
        let changed = manager.set_state(state.clone());
        
        prop_assert!(!changed, "Setting same state must return false");
        prop_assert_eq!(manager.state(), state, "State must remain unchanged");
    }

    /// Feature: russh-ssh, Property: State Query Consistency
    ///
    /// *For any* state, the query methods SHALL return consistent results.
    ///
    /// **Validates: Requirements 2.4 - State query consistency**
    #[test]
    fn state_queries_consistent(
        state in connection_state_strategy(),
    ) {
        let manager = StateManager::with_state(state.clone());
        let current = manager.state();
        
        match &current {
            ConnectionState::Disconnected => {
                prop_assert!(current.is_disconnected());
                prop_assert!(!current.is_connected());
                prop_assert!(!current.is_connecting());
                prop_assert!(!current.is_failed());
            }
            ConnectionState::Connecting => {
                prop_assert!(!current.is_disconnected());
                prop_assert!(!current.is_connected());
                prop_assert!(current.is_connecting());
                prop_assert!(!current.is_failed());
            }
            ConnectionState::Connected => {
                prop_assert!(!current.is_disconnected());
                prop_assert!(current.is_connected());
                prop_assert!(!current.is_connecting());
                prop_assert!(!current.is_failed());
            }
            ConnectionState::Reconnecting { attempt } => {
                prop_assert!(!current.is_disconnected());
                prop_assert!(!current.is_connected());
                prop_assert!(current.is_connecting());
                prop_assert!(!current.is_failed());
                prop_assert_eq!(current.reconnect_attempt(), Some(*attempt));
            }
            ConnectionState::Failed { reason } => {
                prop_assert!(!current.is_disconnected());
                prop_assert!(!current.is_connected());
                prop_assert!(!current.is_connecting());
                prop_assert!(current.is_failed());
                prop_assert_eq!(current.failure_reason(), Some(reason.as_str()));
            }
        }
    }

    /// Feature: russh-ssh, Property: State Serialization Roundtrip
    ///
    /// *For any* state, serializing and deserializing SHALL produce
    /// an equivalent state.
    ///
    /// **Validates: Requirements 2.4 - State persistence**
    #[test]
    fn state_serialization_roundtrip(
        state in connection_state_strategy(),
    ) {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ConnectionState = serde_json::from_str(&json).unwrap();
        
        prop_assert_eq!(state, deserialized, "Serialization roundtrip must preserve state");
    }
}

/// Tests for state broadcasting
#[cfg(test)]
mod broadcast_tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn state_changes_are_broadcast() {
        let manager = Arc::new(StateManager::new());
        let mut receiver = manager.subscribe();
        
        // Change state
        manager.set_state(ConnectionState::Connecting);
        
        // Should receive the event
        let event = receiver.recv().await.unwrap();
        assert_eq!(event.old_state, ConnectionState::Disconnected);
        assert_eq!(event.new_state, ConnectionState::Connecting);
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_events() {
        let manager = Arc::new(StateManager::new());
        let mut receiver1 = manager.subscribe();
        let mut receiver2 = manager.subscribe();
        
        // Change state
        manager.set_state(ConnectionState::Connecting);
        
        // Both should receive the event
        let event1 = receiver1.recv().await.unwrap();
        let event2 = receiver2.recv().await.unwrap();
        
        assert_eq!(event1.new_state, ConnectionState::Connecting);
        assert_eq!(event2.new_state, ConnectionState::Connecting);
    }
}
