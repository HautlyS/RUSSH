//! Property-based tests for reconnection strategy
//!
//! Feature: russh-ssh
//! These tests validate the correctness properties of the reconnection strategy.

use proptest::prelude::*;
use russh_ssh::config::ReconnectionStrategy;
use std::time::Duration;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: russh-ssh, Property 1: Exponential Backoff Calculation
    ///
    /// *For any* reconnection attempt number n and base delay d, the calculated
    /// delay SHALL equal d × 2^n (capped at max_delay), ensuring predictable
    /// backoff behavior.
    ///
    /// **Validates: Requirements 2.1**
    #[test]
    fn exponential_backoff_calculation(
        attempt in 0u32..20,
        base_delay_ms in 100u64..5000,
        max_delay_ms in 10000u64..60000,
    ) {
        // Ensure max_delay >= base_delay for valid strategy
        let max_delay_ms = max_delay_ms.max(base_delay_ms);

        let strategy = ReconnectionStrategy::new(
            10,
            Duration::from_millis(base_delay_ms),
            Duration::from_millis(max_delay_ms),
        ).without_jitter(); // Disable jitter for deterministic testing

        // Calculate expected delay: base_delay * 2^attempt, capped at max_delay
        // Cap exponent at 10 to match implementation
        let capped_attempt = attempt.min(10);
        let multiplier = 2u64.pow(capped_attempt);
        let expected_ms = base_delay_ms.saturating_mul(multiplier);
        let expected_capped_ms = expected_ms.min(max_delay_ms);
        let expected = Duration::from_millis(expected_capped_ms);

        let actual = strategy.delay_for_attempt(attempt);

        prop_assert_eq!(
            actual, expected,
            "For attempt {}, base_delay {}ms, max_delay {}ms: expected {:?}, got {:?}",
            attempt, base_delay_ms, max_delay_ms, expected, actual
        );
    }

    /// Feature: russh-ssh, Property 1 (continued): Delay is always capped
    ///
    /// *For any* reconnection attempt, the delay SHALL never exceed max_delay.
    ///
    /// **Validates: Requirements 2.1**
    #[test]
    fn delay_never_exceeds_max(
        attempt in 0u32..100,
        base_delay_ms in 1u64..10000,
        max_delay_ms in 1u64..120000,
    ) {
        let strategy = ReconnectionStrategy::new(
            100,
            Duration::from_millis(base_delay_ms),
            Duration::from_millis(max_delay_ms),
        ).without_jitter();

        let delay = strategy.delay_for_attempt(attempt);
        let max_delay = Duration::from_millis(max_delay_ms);

        prop_assert!(
            delay <= max_delay,
            "Delay {:?} exceeded max_delay {:?} for attempt {}",
            delay, max_delay, attempt
        );
    }

    /// Feature: russh-ssh, Property 1 (continued): Delay is monotonically increasing
    ///
    /// *For any* two consecutive attempts, the delay for the later attempt
    /// SHALL be greater than or equal to the delay for the earlier attempt
    /// (until max_delay is reached).
    ///
    /// **Validates: Requirements 2.1**
    #[test]
    fn delay_is_monotonically_increasing(
        attempt in 0u32..19,
        base_delay_ms in 100u64..5000,
        max_delay_ms in 10000u64..60000,
    ) {
        let max_delay_ms = max_delay_ms.max(base_delay_ms);

        let strategy = ReconnectionStrategy::new(
            20,
            Duration::from_millis(base_delay_ms),
            Duration::from_millis(max_delay_ms),
        ).without_jitter();

        let delay_current = strategy.delay_for_attempt(attempt);
        let delay_next = strategy.delay_for_attempt(attempt + 1);

        prop_assert!(
            delay_next >= delay_current,
            "Delay decreased from {:?} (attempt {}) to {:?} (attempt {})",
            delay_current, attempt, delay_next, attempt + 1
        );
    }

    /// Feature: russh-ssh, Property 1 (continued): First attempt uses base delay
    ///
    /// *For any* reconnection strategy, the delay for attempt 0 SHALL equal
    /// the base delay (or max_delay if base > max).
    ///
    /// **Validates: Requirements 2.1**
    #[test]
    fn first_attempt_uses_base_delay(
        base_delay_ms in 1u64..10000,
        max_delay_ms in 1u64..60000,
    ) {
        let strategy = ReconnectionStrategy::new(
            10,
            Duration::from_millis(base_delay_ms),
            Duration::from_millis(max_delay_ms),
        ).without_jitter();

        let delay = strategy.delay_for_attempt(0);
        let expected = Duration::from_millis(base_delay_ms.min(max_delay_ms));

        prop_assert_eq!(
            delay, expected,
            "First attempt delay {:?} != expected base delay {:?}",
            delay, expected
        );
    }

    /// Feature: russh-ssh, Property 1 (continued): Jitter adds bounded randomness
    ///
    /// *For any* reconnection attempt with jitter enabled, the delay SHALL be
    /// between the base exponential delay and base + 25% of base.
    ///
    /// **Validates: Requirements 2.1**
    #[test]
    fn jitter_is_bounded(
        attempt in 0u32..10,
        base_delay_ms in 100u64..5000,
        max_delay_ms in 10000u64..60000,
    ) {
        let max_delay_ms = max_delay_ms.max(base_delay_ms);

        let strategy_no_jitter = ReconnectionStrategy::new(
            10,
            Duration::from_millis(base_delay_ms),
            Duration::from_millis(max_delay_ms),
        ).without_jitter();

        let strategy_with_jitter = ReconnectionStrategy::new(
            10,
            Duration::from_millis(base_delay_ms),
            Duration::from_millis(max_delay_ms),
        ).with_jitter();

        let base_delay = strategy_no_jitter.delay_for_attempt(attempt);
        let jittered_delay = strategy_with_jitter.delay_for_attempt(attempt);

        // Jitter adds up to 25% of the base delay
        let max_jitter = base_delay.as_millis() as u64 / 4;
        let max_with_jitter = base_delay + Duration::from_millis(max_jitter);

        prop_assert!(
            jittered_delay >= base_delay,
            "Jittered delay {:?} < base delay {:?}",
            jittered_delay, base_delay
        );

        prop_assert!(
            jittered_delay <= max_with_jitter,
            "Jittered delay {:?} > max allowed {:?} (base {:?} + 25%)",
            jittered_delay, max_with_jitter, base_delay
        );
    }
}

// Additional tests for Property 2: Max Reconnection Attempts Enforcement

/// Feature: russh-ssh, Property 2: Max Reconnection Attempts Enforcement
///
/// *For any* reconnection strategy with max_attempts = M, the reconnection
/// controller SHALL make exactly M attempts before reporting failure, never
/// more and never fewer (unless connection succeeds).
///
/// **Validates: Requirements 2.2**
#[cfg(test)]
mod max_attempts_tests {
    use proptest::prelude::*;
    use russh_ssh::config::ReconnectionStrategy;
    use russh_ssh::connection::ReconnectionController;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: russh-ssh, Property 2: Exactly M attempts on failure
        ///
        /// *For any* max_attempts value M, when all connection attempts fail,
        /// the controller SHALL make exactly M attempts.
        ///
        /// **Validates: Requirements 2.2**
        #[test]
        fn exactly_m_attempts_on_failure(max_attempts in 1u32..10) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let controller = ReconnectionController::new();
                let strategy = ReconnectionStrategy::new(
                    max_attempts,
                    Duration::from_millis(1),
                    Duration::from_millis(10),
                ).without_jitter();

                let attempt_count = Arc::new(AtomicU32::new(0));
                let attempt_count_clone = attempt_count.clone();

                let result: Result<(), russh_ssh::ReconnectionError> = controller
                    .reconnect(&strategy, || {
                        let count = attempt_count_clone.clone();
                        async move {
                            count.fetch_add(1, Ordering::SeqCst);
                            Err::<(), _>("always fails")
                        }
                    })
                    .await;

                prop_assert!(result.is_err(), "Should fail when all attempts fail");
                prop_assert_eq!(
                    attempt_count.load(Ordering::SeqCst),
                    max_attempts,
                    "Should make exactly {} attempts, made {}",
                    max_attempts,
                    attempt_count.load(Ordering::SeqCst)
                );

                Ok(())
            })?;
        }

        /// Feature: russh-ssh, Property 2: Success stops attempts
        ///
        /// *For any* max_attempts value M and success_at value S < M,
        /// when connection succeeds at attempt S, the controller SHALL
        /// make exactly S attempts (not more).
        ///
        /// **Validates: Requirements 2.2**
        #[test]
        fn success_stops_attempts(
            max_attempts in 3u32..10,
            success_at in 1u32..3,
        ) {
            // Ensure success_at < max_attempts
            let success_at = success_at.min(max_attempts - 1);

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let controller = ReconnectionController::new();
                let strategy = ReconnectionStrategy::new(
                    max_attempts,
                    Duration::from_millis(1),
                    Duration::from_millis(10),
                ).without_jitter();

                let attempt_count = Arc::new(AtomicU32::new(0));
                let attempt_count_clone = attempt_count.clone();

                let result: Result<i32, russh_ssh::ReconnectionError> = controller
                    .reconnect(&strategy, || {
                        let count = attempt_count_clone.clone();
                        let target = success_at;
                        async move {
                            let current = count.fetch_add(1, Ordering::SeqCst);
                            if current + 1 >= target {
                                Ok(42)
                            } else {
                                Err("not yet")
                            }
                        }
                    })
                    .await;

                prop_assert!(result.is_ok(), "Should succeed at attempt {}", success_at);
                prop_assert_eq!(
                    attempt_count.load(Ordering::SeqCst),
                    success_at,
                    "Should stop at attempt {}, made {}",
                    success_at,
                    attempt_count.load(Ordering::SeqCst)
                );

                Ok(())
            })?;
        }

        /// Feature: russh-ssh, Property 2: should_retry respects max_attempts
        ///
        /// *For any* max_attempts value M and current_attempt value C,
        /// should_retry SHALL return true iff C < M.
        ///
        /// **Validates: Requirements 2.2**
        #[test]
        fn should_retry_respects_max(
            max_attempts in 1u32..100,
            current_attempt in 0u32..150,
        ) {
            let strategy = ReconnectionStrategy::new(
                max_attempts,
                Duration::from_millis(100),
                Duration::from_millis(1000),
            );

            let should_retry = strategy.should_retry(current_attempt);
            let expected = current_attempt < max_attempts;

            prop_assert_eq!(
                should_retry, expected,
                "should_retry({}) with max_attempts={} should be {}, got {}",
                current_attempt, max_attempts, expected, should_retry
            );
        }
    }
}
