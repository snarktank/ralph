//! Pause controller module for pause/resume state management.
//!
//! This module provides thread-safe pause and resume functionality
//! for agent execution, allowing users to manually pause execution
//! and resume later. Also includes retry strategies with exponential backoff.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::error::ErrorCategory;

/// State of the pause controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PauseState {
    /// Execution is running normally.
    #[default]
    Running,
    /// A pause has been requested but not yet executed.
    PauseRequested,
    /// Execution is paused.
    Paused,
}

/// Controller for managing pause/resume state.
///
/// This struct provides thread-safe access to pause state,
/// allowing multiple threads to check and modify the pause state.
#[derive(Debug, Clone)]
pub struct PauseController {
    state: Arc<RwLock<PauseState>>,
}

impl Default for PauseController {
    fn default() -> Self {
        Self::new()
    }
}

impl PauseController {
    /// Creates a new PauseController in the Running state.
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PauseState::Running)),
        }
    }

    /// Requests a pause. Transitions from Running to PauseRequested.
    ///
    /// Returns true if the pause was requested successfully,
    /// false if already paused or pause already requested.
    pub fn request_pause(&self) -> bool {
        let mut state = self.state.write().unwrap();
        if *state == PauseState::Running {
            *state = PauseState::PauseRequested;
            true
        } else {
            false
        }
    }

    /// Checks if a pause has been requested.
    ///
    /// Returns true if the state is PauseRequested.
    pub fn is_pause_requested(&self) -> bool {
        let state = self.state.read().unwrap();
        *state == PauseState::PauseRequested
    }

    /// Executes the pause. Transitions from PauseRequested to Paused.
    ///
    /// Returns true if the pause was executed successfully,
    /// false if not in PauseRequested state.
    pub fn execute_pause(&self) -> bool {
        let mut state = self.state.write().unwrap();
        if *state == PauseState::PauseRequested {
            *state = PauseState::Paused;
            true
        } else {
            false
        }
    }

    /// Resumes execution. Transitions from Paused to Running.
    ///
    /// Returns true if resume was successful,
    /// false if not in Paused state.
    pub fn resume(&self) -> bool {
        let mut state = self.state.write().unwrap();
        if *state == PauseState::Paused {
            *state = PauseState::Running;
            true
        } else {
            false
        }
    }

    /// Returns the current pause state.
    pub fn state(&self) -> PauseState {
        let state = self.state.read().unwrap();
        *state
    }

    /// Checks if execution is currently paused.
    pub fn is_paused(&self) -> bool {
        let state = self.state.read().unwrap();
        *state == PauseState::Paused
    }

    /// Checks if execution is currently running.
    pub fn is_running(&self) -> bool {
        let state = self.state.read().unwrap();
        *state == PauseState::Running
    }
}

/// Configuration for retry behavior with exponential backoff.
#[derive(Clone, Debug)]
pub struct RetryStrategy {
    /// Base delay for the first retry attempt.
    pub base_delay: Duration,
    /// Maximum delay cap for any retry attempt.
    pub max_delay: Duration,
    /// Maximum number of retry attempts allowed.
    pub max_attempts: u32,
    /// Percentage of jitter to add to delays (0-100).
    pub jitter_percent: u8,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_attempts: 5,
            jitter_percent: 10,
        }
    }
}

impl RetryStrategy {
    /// Creates a new retry strategy with the specified parameters.
    pub fn new(
        base_delay: Duration,
        max_delay: Duration,
        max_attempts: u32,
        jitter_percent: u8,
    ) -> Self {
        Self {
            base_delay,
            max_delay,
            max_attempts,
            jitter_percent: jitter_percent.min(100),
        }
    }

    /// Calculates the delay for a given attempt number using exponential backoff.
    ///
    /// The delay is calculated as: base_delay * 2^(attempt - 1), capped at max_delay.
    /// Jitter is added based on the configured jitter_percent.
    ///
    /// # Arguments
    /// * `attempt` - The current attempt number (1-based).
    ///
    /// # Returns
    /// The duration to wait before the next retry attempt.
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.base_delay;
        }

        // Calculate exponential backoff: base_delay * 2^(attempt - 1)
        let exponent = (attempt - 1).min(31); // Prevent overflow
        let multiplier = 1u64 << exponent;
        let base_millis = self.base_delay.as_millis() as u64;
        let delay_millis = base_millis.saturating_mul(multiplier);

        // Cap at max_delay
        let capped_millis = delay_millis.min(self.max_delay.as_millis() as u64);

        // Apply jitter: random variation between -jitter_percent% and +jitter_percent%
        let jitter_range = (capped_millis as f64 * self.jitter_percent as f64) / 100.0;
        let jitter = self.deterministic_jitter(attempt, jitter_range);

        let final_millis = (capped_millis as i64 + jitter).max(0) as u64;
        Duration::from_millis(final_millis)
    }

    /// Generates deterministic jitter based on attempt number for testability.
    /// In production, this could be replaced with random jitter.
    fn deterministic_jitter(&self, attempt: u32, jitter_range: f64) -> i64 {
        // Use attempt number to create pseudo-random but deterministic jitter
        // This alternates between positive and negative jitter
        let sign = if attempt % 2 == 0 { 1.0 } else { -1.0 };
        // Scale jitter by a factor based on attempt (creates variation)
        let factor = ((attempt % 5) as f64 + 1.0) / 5.0;
        (jitter_range * sign * factor) as i64
    }

    /// Determines whether a retry should be attempted based on the attempt count
    /// and error category.
    ///
    /// Only `Transient` category errors are eligible for retry.
    ///
    /// # Arguments
    /// * `attempt` - The current attempt number (1-based).
    /// * `category` - The category of the error that occurred.
    ///
    /// # Returns
    /// `true` if the operation should be retried, `false` otherwise.
    pub fn should_retry(&self, attempt: u32, category: &ErrorCategory) -> bool {
        // Only retry transient errors
        if !matches!(category, ErrorCategory::Transient(_)) {
            return false;
        }

        // Check if we have attempts remaining
        attempt < self.max_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_initial_state_is_running() {
        let controller = PauseController::new();
        assert_eq!(controller.state(), PauseState::Running);
        assert!(controller.is_running());
        assert!(!controller.is_paused());
        assert!(!controller.is_pause_requested());
    }

    #[test]
    fn test_request_pause_from_running() {
        let controller = PauseController::new();

        assert!(controller.request_pause());
        assert_eq!(controller.state(), PauseState::PauseRequested);
        assert!(controller.is_pause_requested());
        assert!(!controller.is_running());
        assert!(!controller.is_paused());
    }

    #[test]
    fn test_request_pause_when_already_requested() {
        let controller = PauseController::new();

        assert!(controller.request_pause());
        assert!(!controller.request_pause()); // Should return false
        assert_eq!(controller.state(), PauseState::PauseRequested);
    }

    #[test]
    fn test_request_pause_when_paused() {
        let controller = PauseController::new();

        controller.request_pause();
        controller.execute_pause();

        assert!(!controller.request_pause()); // Should return false
        assert_eq!(controller.state(), PauseState::Paused);
    }

    #[test]
    fn test_execute_pause_from_pause_requested() {
        let controller = PauseController::new();

        controller.request_pause();
        assert!(controller.execute_pause());
        assert_eq!(controller.state(), PauseState::Paused);
        assert!(controller.is_paused());
        assert!(!controller.is_running());
        assert!(!controller.is_pause_requested());
    }

    #[test]
    fn test_execute_pause_from_running() {
        let controller = PauseController::new();

        assert!(!controller.execute_pause()); // Should return false
        assert_eq!(controller.state(), PauseState::Running);
    }

    #[test]
    fn test_execute_pause_when_already_paused() {
        let controller = PauseController::new();

        controller.request_pause();
        controller.execute_pause();

        assert!(!controller.execute_pause()); // Should return false
        assert_eq!(controller.state(), PauseState::Paused);
    }

    #[test]
    fn test_resume_from_paused() {
        let controller = PauseController::new();

        controller.request_pause();
        controller.execute_pause();

        assert!(controller.resume());
        assert_eq!(controller.state(), PauseState::Running);
        assert!(controller.is_running());
        assert!(!controller.is_paused());
    }

    #[test]
    fn test_resume_from_running() {
        let controller = PauseController::new();

        assert!(!controller.resume()); // Should return false
        assert_eq!(controller.state(), PauseState::Running);
    }

    #[test]
    fn test_resume_from_pause_requested() {
        let controller = PauseController::new();

        controller.request_pause();

        assert!(!controller.resume()); // Should return false
        assert_eq!(controller.state(), PauseState::PauseRequested);
    }

    #[test]
    fn test_full_pause_resume_cycle() {
        let controller = PauseController::new();

        // Start running
        assert_eq!(controller.state(), PauseState::Running);

        // Request pause
        assert!(controller.request_pause());
        assert_eq!(controller.state(), PauseState::PauseRequested);

        // Execute pause
        assert!(controller.execute_pause());
        assert_eq!(controller.state(), PauseState::Paused);

        // Resume
        assert!(controller.resume());
        assert_eq!(controller.state(), PauseState::Running);
    }

    #[test]
    fn test_multiple_pause_resume_cycles() {
        let controller = PauseController::new();

        for _ in 0..3 {
            assert!(controller.request_pause());
            assert!(controller.execute_pause());
            assert!(controller.resume());
            assert_eq!(controller.state(), PauseState::Running);
        }
    }

    #[test]
    fn test_clone_shares_state() {
        let controller1 = PauseController::new();
        let controller2 = controller1.clone();

        controller1.request_pause();
        assert!(controller2.is_pause_requested());

        controller2.execute_pause();
        assert!(controller1.is_paused());

        controller1.resume();
        assert!(controller2.is_running());
    }

    #[test]
    fn test_default_is_running() {
        let controller = PauseController::default();
        assert_eq!(controller.state(), PauseState::Running);
    }

    #[test]
    fn test_pause_state_default_is_running() {
        let state = PauseState::default();
        assert_eq!(state, PauseState::Running);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let controller = PauseController::new();
        let controller_clone = controller.clone();

        let handle = thread::spawn(move || {
            controller_clone.request_pause();
            controller_clone.is_pause_requested()
        });

        let result = handle.join().unwrap();
        assert!(result);
        assert!(controller.is_pause_requested());
    }

    // RetryStrategy tests
    use crate::error::{FatalReason, TimeoutReason, TransientReason, UsageLimitReason};

    #[test]
    fn test_retry_strategy_default() {
        let strategy = RetryStrategy::default();
        assert_eq!(strategy.base_delay, Duration::from_secs(1));
        assert_eq!(strategy.max_delay, Duration::from_secs(60));
        assert_eq!(strategy.max_attempts, 5);
        assert_eq!(strategy.jitter_percent, 10);
    }

    #[test]
    fn test_retry_strategy_new() {
        let strategy =
            RetryStrategy::new(Duration::from_millis(500), Duration::from_secs(30), 3, 20);
        assert_eq!(strategy.base_delay, Duration::from_millis(500));
        assert_eq!(strategy.max_delay, Duration::from_secs(30));
        assert_eq!(strategy.max_attempts, 3);
        assert_eq!(strategy.jitter_percent, 20);
    }

    #[test]
    fn test_retry_strategy_jitter_percent_capped_at_100() {
        let strategy = RetryStrategy::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            5,
            150, // Should be capped to 100
        );
        assert_eq!(strategy.jitter_percent, 100);
    }

    #[test]
    fn test_calculate_delay_exponential_growth() {
        let strategy = RetryStrategy::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            5,
            0, // No jitter for predictable testing
        );

        // attempt 1: 1 * 2^0 = 1 second
        let delay1 = strategy.calculate_delay(1);
        assert_eq!(delay1, Duration::from_secs(1));

        // attempt 2: 1 * 2^1 = 2 seconds
        let delay2 = strategy.calculate_delay(2);
        assert_eq!(delay2, Duration::from_secs(2));

        // attempt 3: 1 * 2^2 = 4 seconds
        let delay3 = strategy.calculate_delay(3);
        assert_eq!(delay3, Duration::from_secs(4));

        // attempt 4: 1 * 2^3 = 8 seconds
        let delay4 = strategy.calculate_delay(4);
        assert_eq!(delay4, Duration::from_secs(8));

        // attempt 5: 1 * 2^4 = 16 seconds
        let delay5 = strategy.calculate_delay(5);
        assert_eq!(delay5, Duration::from_secs(16));
    }

    #[test]
    fn test_calculate_delay_respects_max_delay() {
        let strategy = RetryStrategy::new(
            Duration::from_secs(10),
            Duration::from_secs(30),
            10,
            0, // No jitter
        );

        // attempt 1: 10 * 2^0 = 10 seconds
        assert_eq!(strategy.calculate_delay(1), Duration::from_secs(10));

        // attempt 2: 10 * 2^1 = 20 seconds
        assert_eq!(strategy.calculate_delay(2), Duration::from_secs(20));

        // attempt 3: 10 * 2^2 = 40 seconds, capped to 30
        assert_eq!(strategy.calculate_delay(3), Duration::from_secs(30));

        // attempt 4: 10 * 2^3 = 80 seconds, capped to 30
        assert_eq!(strategy.calculate_delay(4), Duration::from_secs(30));
    }

    #[test]
    fn test_calculate_delay_with_jitter() {
        let strategy = RetryStrategy::new(
            Duration::from_secs(10),
            Duration::from_secs(60),
            5,
            10, // 10% jitter
        );

        let delay1 = strategy.calculate_delay(1);
        let delay2 = strategy.calculate_delay(2);

        // With 10% jitter on 10 seconds = ±1 second range
        // Delay1 should be around 10s with some jitter
        assert!(delay1 >= Duration::from_millis(9000));
        assert!(delay1 <= Duration::from_millis(11000));

        // Delay2 base is 20 seconds with ±2 seconds jitter
        assert!(delay2 >= Duration::from_millis(18000));
        assert!(delay2 <= Duration::from_millis(22000));
    }

    #[test]
    fn test_calculate_delay_attempt_zero() {
        let strategy = RetryStrategy::default();
        // Attempt 0 should return base delay
        assert_eq!(strategy.calculate_delay(0), Duration::from_secs(1));
    }

    #[test]
    fn test_calculate_delay_handles_overflow() {
        let strategy = RetryStrategy::new(Duration::from_secs(1), Duration::from_secs(60), 100, 0);

        // Very high attempt numbers should not overflow
        let delay = strategy.calculate_delay(50);
        assert!(delay <= Duration::from_secs(60));

        let delay_max = strategy.calculate_delay(u32::MAX);
        assert!(delay_max <= Duration::from_secs(60));
    }

    #[test]
    fn test_should_retry_transient_errors() {
        let strategy = RetryStrategy::default();

        let transient_network = ErrorCategory::Transient(TransientReason::NetworkError);
        let transient_server = ErrorCategory::Transient(TransientReason::ServerError);
        let transient_unavailable = ErrorCategory::Transient(TransientReason::ServiceUnavailable);

        // Should retry transient errors when under max attempts
        assert!(strategy.should_retry(1, &transient_network));
        assert!(strategy.should_retry(2, &transient_server));
        assert!(strategy.should_retry(4, &transient_unavailable));
    }

    #[test]
    fn test_should_retry_respects_max_attempts() {
        let strategy = RetryStrategy::new(Duration::from_secs(1), Duration::from_secs(60), 3, 10);

        let transient = ErrorCategory::Transient(TransientReason::NetworkError);

        // Attempts 1, 2 should be retried (attempt < max_attempts of 3)
        assert!(strategy.should_retry(1, &transient));
        assert!(strategy.should_retry(2, &transient));

        // Attempt 3 and beyond should not be retried
        assert!(!strategy.should_retry(3, &transient));
        assert!(!strategy.should_retry(4, &transient));
    }

    #[test]
    fn test_should_retry_rejects_non_transient_errors() {
        let strategy = RetryStrategy::default();

        // Usage limit errors should not be retried
        let rate_limited = ErrorCategory::UsageLimit(UsageLimitReason::RateLimited);
        assert!(!strategy.should_retry(1, &rate_limited));

        // Fatal errors should not be retried
        let auth_failed = ErrorCategory::Fatal(FatalReason::AuthenticationFailed);
        assert!(!strategy.should_retry(1, &auth_failed));

        let permission_denied = ErrorCategory::Fatal(FatalReason::PermissionDenied);
        assert!(!strategy.should_retry(1, &permission_denied));

        // Timeout errors should not be retried
        let timeout = ErrorCategory::Timeout(TimeoutReason::RequestTimeout);
        assert!(!strategy.should_retry(1, &timeout));
    }

    #[test]
    fn test_should_retry_all_transient_reasons() {
        let strategy = RetryStrategy::default();

        let transient_reasons = vec![
            ErrorCategory::Transient(TransientReason::NetworkError),
            ErrorCategory::Transient(TransientReason::ServiceUnavailable),
            ErrorCategory::Transient(TransientReason::ServerError),
            ErrorCategory::Transient(TransientReason::ConnectionReset),
            ErrorCategory::Transient(TransientReason::ResourceLocked),
        ];

        for category in transient_reasons {
            assert!(
                strategy.should_retry(1, &category),
                "Should retry transient error: {:?}",
                category
            );
        }
    }

    #[test]
    fn test_should_retry_rejects_all_usage_limit_reasons() {
        let strategy = RetryStrategy::default();

        let usage_limit_reasons = vec![
            ErrorCategory::UsageLimit(UsageLimitReason::RateLimited),
            ErrorCategory::UsageLimit(UsageLimitReason::QuotaExhausted),
            ErrorCategory::UsageLimit(UsageLimitReason::TokenLimitExceeded),
            ErrorCategory::UsageLimit(UsageLimitReason::ConcurrencyLimit),
        ];

        for category in usage_limit_reasons {
            assert!(
                !strategy.should_retry(1, &category),
                "Should NOT retry usage limit error: {:?}",
                category
            );
        }
    }

    #[test]
    fn test_should_retry_rejects_all_fatal_reasons() {
        let strategy = RetryStrategy::default();

        let fatal_reasons = vec![
            ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
            ErrorCategory::Fatal(FatalReason::PermissionDenied),
            ErrorCategory::Fatal(FatalReason::ResourceNotFound),
            ErrorCategory::Fatal(FatalReason::InvalidRequest),
            ErrorCategory::Fatal(FatalReason::UnsupportedOperation),
            ErrorCategory::Fatal(FatalReason::InternalError),
            ErrorCategory::Fatal(FatalReason::ConfigurationError),
        ];

        for category in fatal_reasons {
            assert!(
                !strategy.should_retry(1, &category),
                "Should NOT retry fatal error: {:?}",
                category
            );
        }
    }

    #[test]
    fn test_should_retry_rejects_all_timeout_reasons() {
        let strategy = RetryStrategy::default();

        let timeout_reasons = vec![
            ErrorCategory::Timeout(TimeoutReason::RequestTimeout),
            ErrorCategory::Timeout(TimeoutReason::OperationDeadline),
            ErrorCategory::Timeout(TimeoutReason::ProcessTimeout),
            ErrorCategory::Timeout(TimeoutReason::IdleTimeout),
        ];

        for category in timeout_reasons {
            assert!(
                !strategy.should_retry(1, &category),
                "Should NOT retry timeout error: {:?}",
                category
            );
        }
    }

    #[test]
    fn test_retry_strategy_clone() {
        let strategy =
            RetryStrategy::new(Duration::from_millis(500), Duration::from_secs(30), 3, 15);
        let cloned = strategy.clone();

        assert_eq!(strategy.base_delay, cloned.base_delay);
        assert_eq!(strategy.max_delay, cloned.max_delay);
        assert_eq!(strategy.max_attempts, cloned.max_attempts);
        assert_eq!(strategy.jitter_percent, cloned.jitter_percent);
    }

    #[test]
    fn test_retry_strategy_debug() {
        let strategy = RetryStrategy::default();
        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("RetryStrategy"));
        assert!(debug_str.contains("base_delay"));
        assert!(debug_str.contains("max_delay"));
    }
}
