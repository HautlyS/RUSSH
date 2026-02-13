//! Reconnection controller implementation

use crate::config::ReconnectionStrategy;
use crate::error::ReconnectionError;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::sleep;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconnectionStatus {
    Idle,
    InProgress { attempt: u32, max_attempts: u32 },
    Succeeded,
    Failed { attempts: u32, last_error: String },
    Cancelled,
}

impl std::fmt::Display for ReconnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconnectionStatus::Idle => write!(f, "Idle"),
            ReconnectionStatus::InProgress {
                attempt,
                max_attempts,
            } => write!(f, "Reconnecting ({}/{})", attempt, max_attempts),
            ReconnectionStatus::Succeeded => write!(f, "Succeeded"),
            ReconnectionStatus::Failed {
                attempts,
                last_error,
            } => write!(f, "Failed after {}: {}", attempts, last_error),
            ReconnectionStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

pub struct ReconnectionController {
    current_attempt: AtomicU32,
    max_attempts: AtomicU32,
    cancelled: AtomicBool,
    cancel_notify: Arc<Notify>,
}

impl Default for ReconnectionController {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconnectionController {
    pub fn new() -> Self {
        Self {
            current_attempt: AtomicU32::new(0),
            max_attempts: AtomicU32::new(0),
            cancelled: AtomicBool::new(false),
            cancel_notify: Arc::new(Notify::new()),
        }
    }

    pub fn is_reconnecting(&self) -> bool {
        self.current_attempt.load(Ordering::SeqCst) > 0
    }
    pub fn current_attempt(&self) -> u32 {
        self.current_attempt.load(Ordering::SeqCst)
    }

    pub fn status(&self) -> ReconnectionStatus {
        let attempt = self.current_attempt.load(Ordering::SeqCst);
        let max = self.max_attempts.load(Ordering::SeqCst);
        if self.cancelled.load(Ordering::SeqCst) {
            ReconnectionStatus::Cancelled
        } else if attempt > 0 {
            ReconnectionStatus::InProgress {
                attempt,
                max_attempts: max,
            }
        } else {
            ReconnectionStatus::Idle
        }
    }

    pub fn cancel_reconnection(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.cancel_notify.notify_waiters();
    }

    pub fn reset(&self) {
        self.current_attempt.store(0, Ordering::SeqCst);
        self.max_attempts.store(0, Ordering::SeqCst);
        self.cancelled.store(false, Ordering::SeqCst);
    }

    pub async fn reconnect<F, Fut, T, E>(
        &self,
        strategy: &ReconnectionStrategy,
        mut connect: F,
    ) -> Result<T, ReconnectionError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        self.reset();
        self.max_attempts
            .store(strategy.max_attempts, Ordering::SeqCst);
        let mut last_error = String::new();
        for attempt in 0..strategy.max_attempts {
            if self.cancelled.load(Ordering::SeqCst) {
                return Err(ReconnectionError::Cancelled);
            }
            self.current_attempt.store(attempt + 1, Ordering::SeqCst);
            if attempt > 0 {
                let delay = strategy.delay_for_attempt(attempt - 1);
                tokio::select! {
                    _ = sleep(delay) => {}
                    _ = self.cancel_notify.notified() => { return Err(ReconnectionError::Cancelled); }
                }
            }
            match connect().await {
                Ok(result) => {
                    self.current_attempt.store(0, Ordering::SeqCst);
                    return Ok(result);
                }
                Err(e) => {
                    last_error = e.to_string();
                }
            }
        }
        self.current_attempt.store(0, Ordering::SeqCst);
        Err(ReconnectionError::AttemptsExhausted {
            attempts: strategy.max_attempts,
            last_error,
        })
    }

    pub async fn try_once<F, Fut, T, E>(&self, connect: F) -> Result<T, ReconnectionError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        self.reset();
        self.current_attempt.store(1, Ordering::SeqCst);
        self.max_attempts.store(1, Ordering::SeqCst);
        match connect().await {
            Ok(result) => {
                self.current_attempt.store(0, Ordering::SeqCst);
                Ok(result)
            }
            Err(e) => {
                self.current_attempt.store(0, Ordering::SeqCst);
                Err(ReconnectionError::AttemptsExhausted {
                    attempts: 1,
                    last_error: e.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn reconnection_succeeds_on_first_attempt() {
        let controller = ReconnectionController::new();
        let strategy =
            ReconnectionStrategy::new(3, Duration::from_millis(10), Duration::from_millis(100))
                .without_jitter();
        let result: Result<i32, ReconnectionError> = controller
            .reconnect(&strategy, || async { Ok::<_, &str>(42) })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn reconnection_succeeds_after_failures() {
        let controller = ReconnectionController::new();
        let strategy =
            ReconnectionStrategy::new(5, Duration::from_millis(1), Duration::from_millis(10))
                .without_jitter();
        let attempt_count = Arc::new(AtomicU32::new(0));
        let ac = attempt_count.clone();
        let result: Result<i32, ReconnectionError> = controller
            .reconnect(&strategy, || {
                let c = ac.clone();
                async move {
                    let cur = c.fetch_add(1, Ordering::SeqCst);
                    if cur < 2 {
                        Err("not yet")
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;
        assert!(result.is_ok());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn reconnection_exhausts_attempts() {
        let controller = ReconnectionController::new();
        let strategy =
            ReconnectionStrategy::new(3, Duration::from_millis(1), Duration::from_millis(10))
                .without_jitter();
        let attempt_count = Arc::new(AtomicU32::new(0));
        let ac = attempt_count.clone();
        let result: Result<i32, ReconnectionError> = controller
            .reconnect(&strategy, || {
                let c = ac.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("fail")
                }
            })
            .await;
        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn reconnection_can_be_cancelled() {
        let controller = Arc::new(ReconnectionController::new());
        let strategy =
            ReconnectionStrategy::new(10, Duration::from_secs(10), Duration::from_secs(60))
                .without_jitter();
        let cc = controller.clone();
        let handle = tokio::spawn(async move {
            cc.reconnect(&strategy, || async { Err::<i32, _>("fail") })
                .await
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
        controller.cancel_reconnection();
        assert!(matches!(
            handle.await.unwrap(),
            Err(ReconnectionError::Cancelled)
        ));
    }

    #[tokio::test]
    async fn try_once_succeeds() {
        let controller = ReconnectionController::new();
        let result: Result<i32, ReconnectionError> =
            controller.try_once(|| async { Ok::<_, &str>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn try_once_fails() {
        let controller = ReconnectionController::new();
        let result: Result<i32, ReconnectionError> = controller
            .try_once(|| async { Err::<i32, _>("fail") })
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn status_tracking() {
        let controller = ReconnectionController::new();
        assert_eq!(controller.status(), ReconnectionStatus::Idle);
        controller.current_attempt.store(2, Ordering::SeqCst);
        controller.max_attempts.store(5, Ordering::SeqCst);
        match controller.status() {
            ReconnectionStatus::InProgress {
                attempt,
                max_attempts,
            } => {
                assert_eq!(attempt, 2);
                assert_eq!(max_attempts, 5);
            }
            _ => panic!("Expected InProgress"),
        }
        controller.cancelled.store(true, Ordering::SeqCst);
        assert_eq!(controller.status(), ReconnectionStatus::Cancelled);
    }
}
