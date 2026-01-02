//! Connection state tracking
//!
//! This module defines connection states and state transitions,
//! including state change broadcasting for reactive updates.

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Default channel capacity for state change broadcasts
const STATE_CHANNEL_CAPACITY: usize = 16;

/// Connection state tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConnectionState {
    /// Not connected to any host
    #[default]
    Disconnected,
    /// Currently attempting to connect
    Connecting,
    /// Successfully connected
    Connected,
    /// Attempting to reconnect after disconnection
    Reconnecting {
        /// Current reconnection attempt number (1-based)
        attempt: u32,
    },
    /// Connection failed
    Failed {
        /// Reason for the failure
        reason: String,
    },
}

impl ConnectionState {
    /// Check if the connection is active
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionState::Connected)
    }

    /// Check if a connection attempt is in progress
    pub fn is_connecting(&self) -> bool {
        matches!(self, ConnectionState::Connecting | ConnectionState::Reconnecting { .. })
    }

    /// Check if the connection has failed
    pub fn is_failed(&self) -> bool {
        matches!(self, ConnectionState::Failed { .. })
    }

    /// Check if disconnected (not connected and not attempting)
    pub fn is_disconnected(&self) -> bool {
        matches!(self, ConnectionState::Disconnected)
    }

    /// Get the reconnection attempt number if reconnecting
    pub fn reconnect_attempt(&self) -> Option<u32> {
        match self {
            ConnectionState::Reconnecting { attempt } => Some(*attempt),
            _ => None,
        }
    }

    /// Get the failure reason if failed
    pub fn failure_reason(&self) -> Option<&str> {
        match self {
            ConnectionState::Failed { reason } => Some(reason),
            _ => None,
        }
    }

    /// Check if transition to the new state is valid
    pub fn can_transition_to(&self, new_state: &ConnectionState) -> bool {
        match (self, new_state) {
            // From Disconnected: can connect
            (ConnectionState::Disconnected, ConnectionState::Connecting) => true,
            (ConnectionState::Disconnected, ConnectionState::Failed { .. }) => true,

            // From Connecting: can succeed, fail, or disconnect
            (ConnectionState::Connecting, ConnectionState::Connected) => true,
            (ConnectionState::Connecting, ConnectionState::Failed { .. }) => true,
            (ConnectionState::Connecting, ConnectionState::Disconnected) => true,

            // From Connected: can disconnect, fail, or start reconnecting
            (ConnectionState::Connected, ConnectionState::Disconnected) => true,
            (ConnectionState::Connected, ConnectionState::Failed { .. }) => true,
            (ConnectionState::Connected, ConnectionState::Reconnecting { .. }) => true,

            // From Reconnecting: can succeed, fail, continue reconnecting, or disconnect
            (ConnectionState::Reconnecting { .. }, ConnectionState::Connected) => true,
            (ConnectionState::Reconnecting { .. }, ConnectionState::Failed { .. }) => true,
            (ConnectionState::Reconnecting { .. }, ConnectionState::Reconnecting { .. }) => true,
            (ConnectionState::Reconnecting { .. }, ConnectionState::Disconnected) => true,

            // From Failed: can try to connect again or stay disconnected
            (ConnectionState::Failed { .. }, ConnectionState::Connecting) => true,
            (ConnectionState::Failed { .. }, ConnectionState::Disconnected) => true,

            // Same state transitions are always valid (no-op)
            _ if std::mem::discriminant(self) == std::mem::discriminant(new_state) => true,

            // All other transitions are invalid
            _ => false,
        }
    }
}



impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "Disconnected"),
            ConnectionState::Connecting => write!(f, "Connecting"),
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Reconnecting { attempt } => {
                write!(f, "Reconnecting (attempt {})", attempt)
            }
            ConnectionState::Failed { reason } => write!(f, "Failed: {}", reason),
        }
    }
}

/// State change event containing old and new states
#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    /// The previous state
    pub old_state: ConnectionState,
    /// The new state
    pub new_state: ConnectionState,
}

/// Manager for connection state with broadcasting capabilities
/// 
/// Thread-safe state management with proper handling of poisoned locks.
pub struct StateManager {
    /// Current state
    state: std::sync::RwLock<ConnectionState>,
    /// Broadcast sender for state changes
    sender: broadcast::Sender<StateChangeEvent>,
}

impl StateManager {
    /// Create a new state manager starting in Disconnected state
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(STATE_CHANNEL_CAPACITY);
        Self {
            state: std::sync::RwLock::new(ConnectionState::Disconnected),
            sender,
        }
    }

    /// Create a new state manager with a specific initial state
    pub fn with_state(initial_state: ConnectionState) -> Self {
        let (sender, _) = broadcast::channel(STATE_CHANNEL_CAPACITY);
        Self {
            state: std::sync::RwLock::new(initial_state),
            sender,
        }
    }

    /// Get the current state
    /// 
    /// Handles poisoned locks gracefully by recovering the inner value.
    pub fn state(&self) -> ConnectionState {
        self.state.read()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("State lock was poisoned, recovering");
                poisoned.into_inner()
            })
            .clone()
    }

    /// Set the state, broadcasting the change to all subscribers
    ///
    /// Returns true if the state was changed, false if it was the same.
    /// Handles poisoned locks gracefully.
    pub fn set_state(&self, new_state: ConnectionState) -> bool {
        let mut state = self.state.write()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("State lock was poisoned, recovering");
                poisoned.into_inner()
            });
        if *state == new_state {
            return false;
        }

        let old_state = std::mem::replace(&mut *state, new_state.clone());
        drop(state); // Release lock before broadcasting

        // Broadcast the change (ignore errors if no receivers)
        let _ = self.sender.send(StateChangeEvent {
            old_state,
            new_state,
        });

        true
    }

    /// Try to transition to a new state, checking if the transition is valid
    ///
    /// Returns Ok(true) if transitioned, Ok(false) if same state, Err if invalid transition.
    /// Handles poisoned locks gracefully.
    pub fn try_transition(&self, new_state: ConnectionState) -> Result<bool, InvalidTransition> {
        let mut state = self.state.write()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("State lock was poisoned, recovering");
                poisoned.into_inner()
            });

        if *state == new_state {
            return Ok(false);
        }

        if !state.can_transition_to(&new_state) {
            return Err(InvalidTransition {
                from: state.clone(),
                to: new_state,
            });
        }

        let old_state = std::mem::replace(&mut *state, new_state.clone());
        drop(state);

        let _ = self.sender.send(StateChangeEvent {
            old_state,
            new_state,
        });

        Ok(true)
    }

    /// Subscribe to state changes
    pub fn subscribe(&self) -> broadcast::Receiver<StateChangeEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Error for invalid state transitions
#[derive(Debug, Clone)]
pub struct InvalidTransition {
    /// The state we tried to transition from
    pub from: ConnectionState,
    /// The state we tried to transition to
    pub to: ConnectionState,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid state transition from {} to {}",
            self.from, self.to
        )
    }
}

impl std::error::Error for InvalidTransition {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn connection_state_is_connected() {
        assert!(!ConnectionState::Disconnected.is_connected());
        assert!(!ConnectionState::Connecting.is_connected());
        assert!(ConnectionState::Connected.is_connected());
        assert!(!ConnectionState::Reconnecting { attempt: 1 }.is_connected());
        assert!(!ConnectionState::Failed { reason: "test".to_string() }.is_connected());
    }

    #[test]
    fn connection_state_is_connecting() {
        assert!(!ConnectionState::Disconnected.is_connecting());
        assert!(ConnectionState::Connecting.is_connecting());
        assert!(!ConnectionState::Connected.is_connecting());
        assert!(ConnectionState::Reconnecting { attempt: 1 }.is_connecting());
        assert!(!ConnectionState::Failed { reason: "test".to_string() }.is_connecting());
    }

    #[test]
    fn connection_state_display() {
        assert_eq!(ConnectionState::Disconnected.to_string(), "Disconnected");
        assert_eq!(ConnectionState::Connecting.to_string(), "Connecting");
        assert_eq!(ConnectionState::Connected.to_string(), "Connected");
        assert_eq!(
            ConnectionState::Reconnecting { attempt: 3 }.to_string(),
            "Reconnecting (attempt 3)"
        );
        assert_eq!(
            ConnectionState::Failed { reason: "timeout".to_string() }.to_string(),
            "Failed: timeout"
        );
    }

    #[test]
    fn connection_state_serialization() {
        let state = ConnectionState::Reconnecting { attempt: 2 };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ConnectionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }

    #[test]
    fn valid_state_transitions() {
        // Disconnected -> Connecting
        assert!(ConnectionState::Disconnected.can_transition_to(&ConnectionState::Connecting));

        // Connecting -> Connected
        assert!(ConnectionState::Connecting.can_transition_to(&ConnectionState::Connected));

        // Connected -> Reconnecting
        assert!(ConnectionState::Connected.can_transition_to(&ConnectionState::Reconnecting { attempt: 1 }));

        // Reconnecting -> Connected
        assert!(ConnectionState::Reconnecting { attempt: 1 }.can_transition_to(&ConnectionState::Connected));

        // Failed -> Connecting (retry)
        assert!(ConnectionState::Failed { reason: "test".to_string() }.can_transition_to(&ConnectionState::Connecting));
    }

    #[test]
    fn invalid_state_transitions() {
        // Disconnected -> Connected (must go through Connecting)
        assert!(!ConnectionState::Disconnected.can_transition_to(&ConnectionState::Connected));

        // Disconnected -> Reconnecting (must be connected first)
        assert!(!ConnectionState::Disconnected.can_transition_to(&ConnectionState::Reconnecting { attempt: 1 }));
    }

    #[test]
    fn state_manager_basic() {
        let manager = StateManager::new();
        assert_eq!(manager.state(), ConnectionState::Disconnected);

        manager.set_state(ConnectionState::Connecting);
        assert_eq!(manager.state(), ConnectionState::Connecting);

        manager.set_state(ConnectionState::Connected);
        assert_eq!(manager.state(), ConnectionState::Connected);
    }

    #[tokio::test]
    async fn state_manager_broadcasting() {
        let manager = Arc::new(StateManager::new());
        let mut receiver = manager.subscribe();

        // Change state
        manager.set_state(ConnectionState::Connecting);

        // Receive the event
        let event = receiver.recv().await.unwrap();
        assert_eq!(event.old_state, ConnectionState::Disconnected);
        assert_eq!(event.new_state, ConnectionState::Connecting);
    }

    #[test]
    fn state_manager_try_transition() {
        let manager = StateManager::new();

        // Valid transition
        assert!(manager.try_transition(ConnectionState::Connecting).is_ok());

        // Invalid transition (Connecting -> Reconnecting is not valid)
        let result = manager.try_transition(ConnectionState::Reconnecting { attempt: 1 });
        assert!(result.is_err());
    }

    #[test]
    fn state_manager_same_state_no_broadcast() {
        let manager = StateManager::new();
        manager.set_state(ConnectionState::Connecting);

        // Setting same state should return false
        assert!(!manager.set_state(ConnectionState::Connecting));
    }
}
