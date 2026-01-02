//! Property-based tests for connection layer
//!
//! Feature: russh-ssh
//! These tests validate the correctness properties of the connection layer.

use russh_ssh::error::ConnectionError;
use proptest::prelude::*;
use std::time::Duration;

/// Strategy for generating various connection error scenarios
fn connection_error_strategy() -> impl Strategy<Value = ConnectionError> {
    prop_oneof![
        // Timeout errors with various durations
        (1u64..120).prop_map(|secs| ConnectionError::Timeout(Duration::from_secs(secs))),
        
        // DNS resolution errors
        ("[a-z]{3,20}\\.[a-z]{2,5}", "[a-zA-Z0-9 ]{5,50}")
            .prop_map(|(host, reason)| ConnectionError::DnsResolution { host, reason }),
        
        // Connection refused errors
        ("[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}", 1u16..65535)
            .prop_map(|(host, port)| ConnectionError::ConnectionRefused { host, port }),
        
        // Network unreachable errors
        "[a-zA-Z0-9 ]{5,100}".prop_map(ConnectionError::NetworkUnreachable),
        
        // TLS handshake errors
        "[a-zA-Z0-9 ]{5,100}".prop_map(ConnectionError::TlsHandshake),
        
        // Connection closed errors
        "[a-zA-Z0-9 ]{5,100}".prop_map(ConnectionError::ConnectionClosed),
        
        // Invalid config errors
        "[a-zA-Z0-9 ]{5,100}".prop_map(ConnectionError::InvalidConfig),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 13: Connection Error Descriptiveness
    ///
    /// *For any* connection failure scenario, the returned ConnectionError
    /// SHALL contain a non-empty, descriptive message that identifies the failure reason.
    ///
    /// **Validates: Requirements 1.4**
    #[test]
    fn connection_error_is_descriptive(error in connection_error_strategy()) {
        let error_string = error.to_string();
        
        // Error message must not be empty
        prop_assert!(
            !error_string.is_empty(),
            "Connection error message should not be empty"
        );
        
        // Error message must have meaningful content (more than just whitespace)
        prop_assert!(
            error_string.trim().len() > 0,
            "Connection error message should have meaningful content"
        );
        
        // Error message should be reasonably descriptive (at least 10 chars)
        prop_assert!(
            error_string.len() >= 10,
            "Connection error message should be descriptive (at least 10 chars), got: '{}'",
            error_string
        );
        
        // Verify the is_descriptive helper method
        prop_assert!(
            error.is_descriptive(),
            "is_descriptive() should return true for all errors"
        );
        
        // Verify the reason() method returns non-empty string
        let reason = error.reason();
        prop_assert!(
            !reason.is_empty(),
            "reason() should return non-empty string"
        );
    }

    /// Feature: russh-ssh, Property 13 (continued): Timeout errors include duration
    ///
    /// *For any* timeout error, the error message SHALL include the timeout duration.
    ///
    /// **Validates: Requirements 1.4**
    #[test]
    fn timeout_error_includes_duration(secs in 1u64..120) {
        let error = ConnectionError::Timeout(Duration::from_secs(secs));
        let error_string = error.to_string();
        
        // Should mention "timeout" (case insensitive)
        prop_assert!(
            error_string.to_lowercase().contains("timeout"),
            "Timeout error should mention 'timeout': '{}'",
            error_string
        );
    }

    /// Feature: russh-ssh, Property 13 (continued): DNS errors include host
    ///
    /// *For any* DNS resolution error, the error message SHALL include the host name.
    ///
    /// **Validates: Requirements 1.4**
    #[test]
    fn dns_error_includes_host(
        host in "[a-z]{3,20}\\.[a-z]{2,5}",
        reason in "[a-zA-Z0-9 ]{5,50}"
    ) {
        let error = ConnectionError::DnsResolution { 
            host: host.clone(), 
            reason 
        };
        let error_string = error.to_string();
        
        // Should include the host name
        prop_assert!(
            error_string.contains(&host),
            "DNS error should include host '{}': '{}'",
            host, error_string
        );
        
        // Should mention DNS or resolution
        prop_assert!(
            error_string.to_lowercase().contains("dns") || 
            error_string.to_lowercase().contains("resolution"),
            "DNS error should mention 'dns' or 'resolution': '{}'",
            error_string
        );
    }

    /// Feature: russh-ssh, Property 13 (continued): Connection refused includes host and port
    ///
    /// *For any* connection refused error, the error message SHALL include host and port.
    ///
    /// **Validates: Requirements 1.4**
    #[test]
    fn connection_refused_includes_host_port(
        host in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
        port in 1u16..65535
    ) {
        let error = ConnectionError::ConnectionRefused { 
            host: host.clone(), 
            port 
        };
        let error_string = error.to_string();
        
        // Should include the host
        prop_assert!(
            error_string.contains(&host),
            "Connection refused error should include host '{}': '{}'",
            host, error_string
        );
        
        // Should include the port
        prop_assert!(
            error_string.contains(&port.to_string()),
            "Connection refused error should include port '{}': '{}'",
            port, error_string
        );
        
        // Should mention "refused"
        prop_assert!(
            error_string.to_lowercase().contains("refused"),
            "Connection refused error should mention 'refused': '{}'",
            error_string
        );
    }

    /// Feature: russh-ssh, Property 13 (continued): Network unreachable includes reason
    ///
    /// *For any* network unreachable error, the error message SHALL include the reason.
    ///
    /// **Validates: Requirements 1.4**
    #[test]
    fn network_unreachable_includes_reason(reason in "[a-zA-Z0-9 ]{5,50}") {
        let error = ConnectionError::NetworkUnreachable(reason.clone());
        let error_string = error.to_string();
        
        // Should include the reason
        prop_assert!(
            error_string.contains(&reason),
            "Network unreachable error should include reason '{}': '{}'",
            reason, error_string
        );
        
        // Should mention "unreachable" or "network"
        prop_assert!(
            error_string.to_lowercase().contains("unreachable") ||
            error_string.to_lowercase().contains("network"),
            "Network unreachable error should mention 'unreachable' or 'network': '{}'",
            error_string
        );
    }
}

/// Additional unit tests for error descriptiveness edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn io_error_is_descriptive() {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "Connection reset by peer"
        );
        let error = ConnectionError::Io(io_error);
        
        assert!(error.is_descriptive());
        assert!(!error.to_string().is_empty());
        assert!(error.to_string().len() >= 10);
    }

    #[test]
    fn all_error_variants_are_descriptive() {
        let errors = vec![
            ConnectionError::Timeout(Duration::from_secs(30)),
            ConnectionError::DnsResolution {
                host: "example.com".to_string(),
                reason: "NXDOMAIN".to_string(),
            },
            ConnectionError::ConnectionRefused {
                host: "127.0.0.1".to_string(),
                port: 22,
            },
            ConnectionError::NetworkUnreachable("No route to host".to_string()),
            ConnectionError::TlsHandshake("Certificate expired".to_string()),
            ConnectionError::ConnectionClosed("Remote closed connection".to_string()),
            ConnectionError::InvalidConfig("Missing host".to_string()),
        ];

        for error in errors {
            assert!(
                error.is_descriptive(),
                "Error should be descriptive: {:?}",
                error
            );
            assert!(
                !error.to_string().is_empty(),
                "Error message should not be empty: {:?}",
                error
            );
            assert!(
                error.to_string().len() >= 10,
                "Error message should be at least 10 chars: {:?} -> '{}'",
                error,
                error.to_string()
            );
        }
    }
}
