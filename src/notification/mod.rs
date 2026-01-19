//! Notification types for error recovery and status updates.
//!
//! This module provides notification types for communicating error recovery
//! actions and status changes to users. Each notification variant includes
//! relevant context data for display purposes.

mod renderer;

pub use renderer::NotificationRenderer;

use std::time::Duration;

/// A notification about an error recovery action or status change.
///
/// Notifications are used to inform users about what the system is doing
/// in response to errors or state changes. Each variant includes the
/// relevant data needed to display a meaningful message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notification {
    /// The API rate limit has been exceeded.
    ///
    /// Contains the countdown duration until the rate limit resets.
    RateLimited {
        /// Time remaining until the rate limit resets.
        countdown: Duration,
    },

    /// The usage limit (quota) has been exceeded.
    ///
    /// Contains the reason for the limit being exceeded.
    UsageLimitExceeded {
        /// Human-readable reason for the usage limit.
        reason: String,
    },

    /// An operation timed out.
    ///
    /// Contains the duration that was exceeded and what operation timed out.
    Timeout {
        /// The timeout duration that was exceeded.
        duration: Duration,
        /// Description of what operation timed out.
        operation: String,
    },

    /// The system is retrying an operation.
    ///
    /// Contains the current attempt number and delay before retry.
    Retrying {
        /// The current attempt number (1-based).
        attempt: u32,
        /// Maximum number of attempts configured.
        max_attempts: u32,
        /// Delay before the next retry attempt.
        delay: Duration,
        /// Reason for the retry.
        reason: String,
    },

    /// Execution has been paused.
    ///
    /// Contains the reason for pausing.
    Paused {
        /// Reason why execution was paused.
        reason: String,
    },

    /// Execution is resuming from a paused state.
    ///
    /// Contains information about what is being resumed.
    Resuming {
        /// Description of what is being resumed.
        context: String,
    },
}

impl Notification {
    /// Creates a new RateLimited notification.
    pub fn rate_limited(countdown: Duration) -> Self {
        Self::RateLimited { countdown }
    }

    /// Creates a new UsageLimitExceeded notification.
    pub fn usage_limit_exceeded(reason: impl Into<String>) -> Self {
        Self::UsageLimitExceeded {
            reason: reason.into(),
        }
    }

    /// Creates a new Timeout notification.
    pub fn timeout(duration: Duration, operation: impl Into<String>) -> Self {
        Self::Timeout {
            duration,
            operation: operation.into(),
        }
    }

    /// Creates a new Retrying notification.
    pub fn retrying(
        attempt: u32,
        max_attempts: u32,
        delay: Duration,
        reason: impl Into<String>,
    ) -> Self {
        Self::Retrying {
            attempt,
            max_attempts,
            delay,
            reason: reason.into(),
        }
    }

    /// Creates a new Paused notification.
    pub fn paused(reason: impl Into<String>) -> Self {
        Self::Paused {
            reason: reason.into(),
        }
    }

    /// Creates a new Resuming notification.
    pub fn resuming(context: impl Into<String>) -> Self {
        Self::Resuming {
            context: context.into(),
        }
    }

    /// Returns true if this notification indicates an error condition.
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::UsageLimitExceeded { .. } | Self::Timeout { .. }
        )
    }

    /// Returns true if this notification indicates a recovery action.
    pub fn is_recovery(&self) -> bool {
        matches!(self, Self::Retrying { .. } | Self::Resuming { .. })
    }

    /// Returns true if this notification indicates a paused state.
    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }
}

impl std::fmt::Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited { countdown } => {
                write!(f, "Rate limited. Retry in {} seconds.", countdown.as_secs())
            }
            Self::UsageLimitExceeded { reason } => {
                write!(f, "Usage limit exceeded: {}", reason)
            }
            Self::Timeout {
                duration,
                operation,
            } => {
                write!(
                    f,
                    "Operation '{}' timed out after {} seconds.",
                    operation,
                    duration.as_secs()
                )
            }
            Self::Retrying {
                attempt,
                max_attempts,
                delay,
                reason,
            } => {
                write!(
                    f,
                    "Retrying ({}/{}) in {} seconds: {}",
                    attempt,
                    max_attempts,
                    delay.as_secs(),
                    reason
                )
            }
            Self::Paused { reason } => {
                write!(f, "Paused: {}", reason)
            }
            Self::Resuming { context } => {
                write!(f, "Resuming: {}", context)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limited_notification() {
        let notification = Notification::rate_limited(Duration::from_secs(30));
        assert!(matches!(
            notification,
            Notification::RateLimited { countdown } if countdown == Duration::from_secs(30)
        ));
        assert!(notification.is_error());
        assert!(!notification.is_recovery());
        assert!(!notification.is_paused());
    }

    #[test]
    fn test_usage_limit_exceeded_notification() {
        let notification = Notification::usage_limit_exceeded("API quota exhausted");
        assert!(matches!(
            notification,
            Notification::UsageLimitExceeded { ref reason } if reason == "API quota exhausted"
        ));
        assert!(notification.is_error());
        assert!(!notification.is_recovery());
    }

    #[test]
    fn test_timeout_notification() {
        let notification = Notification::timeout(Duration::from_secs(60), "API request");
        assert!(matches!(
            notification,
            Notification::Timeout { duration, ref operation }
                if duration == Duration::from_secs(60) && operation == "API request"
        ));
        assert!(notification.is_error());
        assert!(!notification.is_recovery());
    }

    #[test]
    fn test_retrying_notification() {
        let notification =
            Notification::retrying(2, 5, Duration::from_secs(4), "Connection failed");
        assert!(matches!(
            notification,
            Notification::Retrying { attempt, max_attempts, delay, ref reason }
                if attempt == 2
                    && max_attempts == 5
                    && delay == Duration::from_secs(4)
                    && reason == "Connection failed"
        ));
        assert!(!notification.is_error());
        assert!(notification.is_recovery());
    }

    #[test]
    fn test_paused_notification() {
        let notification = Notification::paused("User requested pause");
        assert!(matches!(
            notification,
            Notification::Paused { ref reason } if reason == "User requested pause"
        ));
        assert!(!notification.is_error());
        assert!(!notification.is_recovery());
        assert!(notification.is_paused());
    }

    #[test]
    fn test_resuming_notification() {
        let notification = Notification::resuming("Continuing from checkpoint");
        assert!(matches!(
            notification,
            Notification::Resuming { ref context } if context == "Continuing from checkpoint"
        ));
        assert!(!notification.is_error());
        assert!(notification.is_recovery());
        assert!(!notification.is_paused());
    }

    #[test]
    fn test_display_rate_limited() {
        let notification = Notification::rate_limited(Duration::from_secs(30));
        assert_eq!(
            format!("{}", notification),
            "Rate limited. Retry in 30 seconds."
        );
    }

    #[test]
    fn test_display_usage_limit_exceeded() {
        let notification = Notification::usage_limit_exceeded("API quota exhausted");
        assert_eq!(
            format!("{}", notification),
            "Usage limit exceeded: API quota exhausted"
        );
    }

    #[test]
    fn test_display_timeout() {
        let notification = Notification::timeout(Duration::from_secs(60), "API request");
        assert_eq!(
            format!("{}", notification),
            "Operation 'API request' timed out after 60 seconds."
        );
    }

    #[test]
    fn test_display_retrying() {
        let notification =
            Notification::retrying(2, 5, Duration::from_secs(4), "Connection failed");
        assert_eq!(
            format!("{}", notification),
            "Retrying (2/5) in 4 seconds: Connection failed"
        );
    }

    #[test]
    fn test_display_paused() {
        let notification = Notification::paused("Rate limit exceeded");
        assert_eq!(format!("{}", notification), "Paused: Rate limit exceeded");
    }

    #[test]
    fn test_display_resuming() {
        let notification = Notification::resuming("Story execution");
        assert_eq!(format!("{}", notification), "Resuming: Story execution");
    }

    #[test]
    fn test_notification_clone() {
        let notification = Notification::retrying(1, 3, Duration::from_secs(2), "Test");
        let cloned = notification.clone();
        assert_eq!(notification, cloned);
    }

    #[test]
    fn test_notification_equality() {
        let n1 = Notification::rate_limited(Duration::from_secs(30));
        let n2 = Notification::rate_limited(Duration::from_secs(30));
        let n3 = Notification::rate_limited(Duration::from_secs(60));

        assert_eq!(n1, n2);
        assert_ne!(n1, n3);
    }

    #[test]
    fn test_notification_debug() {
        let notification = Notification::paused("Test pause");
        let debug_str = format!("{:?}", notification);
        assert!(debug_str.contains("Paused"));
        assert!(debug_str.contains("Test pause"));
    }
}
