//! Error classification types for the error recovery system
//!
//! This module provides types for categorizing errors into actionable recovery strategies.
//! Errors are classified by category (transient, usage limit, fatal, timeout) and each
//! category has specific reasons and recovery hints.

use std::collections::HashMap;
use std::time::Duration;

/// The primary category of an error, determining the general recovery strategy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Temporary errors that may resolve on retry (network issues, temporary service unavailability).
    Transient(TransientReason),
    /// Errors due to API rate limits or quota exhaustion.
    UsageLimit(UsageLimitReason),
    /// Unrecoverable errors that require user intervention or code changes.
    Fatal(FatalReason),
    /// Errors caused by operations exceeding time limits.
    Timeout(TimeoutReason),
}

/// Reasons for transient errors that may resolve on retry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransientReason {
    /// Network connectivity issues (DNS, TCP, TLS).
    NetworkError,
    /// Service temporarily unavailable (HTTP 503).
    ServiceUnavailable,
    /// Server-side error that may be temporary (HTTP 5xx).
    ServerError,
    /// Connection was reset or dropped.
    ConnectionReset,
    /// Resource temporarily locked by another process.
    ResourceLocked,
}

/// Reasons for usage limit errors related to API quotas and rate limits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UsageLimitReason {
    /// Rate limit exceeded (HTTP 429).
    RateLimited,
    /// API quota exhausted for the billing period.
    QuotaExhausted,
    /// Token limit exceeded for a single request.
    TokenLimitExceeded,
    /// Concurrent request limit reached.
    ConcurrencyLimit,
}

/// Reasons for fatal errors that cannot be automatically recovered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FatalReason {
    /// Invalid API credentials or authentication failure.
    AuthenticationFailed,
    /// Insufficient permissions for the requested operation.
    PermissionDenied,
    /// The requested resource does not exist.
    ResourceNotFound,
    /// Invalid request parameters or payload.
    InvalidRequest,
    /// Unsupported operation or feature.
    UnsupportedOperation,
    /// Internal error in the application logic.
    InternalError,
    /// Configuration error preventing operation.
    ConfigurationError,
}

/// Reasons for timeout errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimeoutReason {
    /// HTTP request timed out waiting for response.
    RequestTimeout,
    /// Operation exceeded the configured deadline.
    OperationDeadline,
    /// Process execution exceeded time limit.
    ProcessTimeout,
    /// Idle connection timed out.
    IdleTimeout,
}

/// Hints for how to recover from an error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryHint {
    /// Retry immediately with the same parameters.
    RetryNow,
    /// Retry after a specified duration.
    RetryAfter(Duration),
    /// Wait for user input or intervention before proceeding.
    WaitForUser,
    /// Stop execution entirely; error is unrecoverable.
    StopExecution,
    /// Resume from a previously saved checkpoint.
    ResumeFromCheckpoint,
}

/// A classified error with category, message, recovery hint, and additional context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassifiedError {
    /// The category of the error.
    pub category: ErrorCategory,
    /// Human-readable error message.
    pub message: String,
    /// Suggested recovery action.
    pub recovery_hint: RecoveryHint,
    /// Additional context as key-value pairs.
    pub context: HashMap<String, String>,
}

impl ClassifiedError {
    /// Creates a new classified error.
    pub fn new(
        category: ErrorCategory,
        message: impl Into<String>,
        recovery_hint: RecoveryHint,
    ) -> Self {
        Self {
            category,
            message: message.into(),
            recovery_hint,
            context: HashMap::new(),
        }
    }

    /// Creates a new classified error with context.
    pub fn with_context(
        category: ErrorCategory,
        message: impl Into<String>,
        recovery_hint: RecoveryHint,
        context: HashMap<String, String>,
    ) -> Self {
        Self {
            category,
            message: message.into(),
            recovery_hint,
            context,
        }
    }

    /// Adds a context key-value pair to the error.
    pub fn add_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Returns true if this error is transient and may resolve on retry.
    pub fn is_transient(&self) -> bool {
        matches!(self.category, ErrorCategory::Transient(_))
    }

    /// Returns true if this error is due to usage limits.
    pub fn is_usage_limit(&self) -> bool {
        matches!(self.category, ErrorCategory::UsageLimit(_))
    }

    /// Returns true if this error is fatal and cannot be recovered.
    pub fn is_fatal(&self) -> bool {
        matches!(self.category, ErrorCategory::Fatal(_))
    }

    /// Returns true if this error is a timeout.
    pub fn is_timeout(&self) -> bool {
        matches!(self.category, ErrorCategory::Timeout(_))
    }

    /// Returns true if this error suggests retrying.
    pub fn should_retry(&self) -> bool {
        matches!(
            self.recovery_hint,
            RecoveryHint::RetryNow | RecoveryHint::RetryAfter(_)
        )
    }
}

impl std::fmt::Display for ClassifiedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ClassifiedError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_transient() {
        let category = ErrorCategory::Transient(TransientReason::NetworkError);
        assert!(matches!(category, ErrorCategory::Transient(_)));
    }

    #[test]
    fn test_error_category_usage_limit() {
        let category = ErrorCategory::UsageLimit(UsageLimitReason::RateLimited);
        assert!(matches!(category, ErrorCategory::UsageLimit(_)));
    }

    #[test]
    fn test_error_category_fatal() {
        let category = ErrorCategory::Fatal(FatalReason::AuthenticationFailed);
        assert!(matches!(category, ErrorCategory::Fatal(_)));
    }

    #[test]
    fn test_error_category_timeout() {
        let category = ErrorCategory::Timeout(TimeoutReason::RequestTimeout);
        assert!(matches!(category, ErrorCategory::Timeout(_)));
    }

    #[test]
    fn test_transient_reasons() {
        let reasons = vec![
            TransientReason::NetworkError,
            TransientReason::ServiceUnavailable,
            TransientReason::ServerError,
            TransientReason::ConnectionReset,
            TransientReason::ResourceLocked,
        ];
        assert_eq!(reasons.len(), 5);
    }

    #[test]
    fn test_usage_limit_reasons() {
        let reasons = vec![
            UsageLimitReason::RateLimited,
            UsageLimitReason::QuotaExhausted,
            UsageLimitReason::TokenLimitExceeded,
            UsageLimitReason::ConcurrencyLimit,
        ];
        assert_eq!(reasons.len(), 4);
    }

    #[test]
    fn test_fatal_reasons() {
        let reasons = vec![
            FatalReason::AuthenticationFailed,
            FatalReason::PermissionDenied,
            FatalReason::ResourceNotFound,
            FatalReason::InvalidRequest,
            FatalReason::UnsupportedOperation,
            FatalReason::InternalError,
            FatalReason::ConfigurationError,
        ];
        assert_eq!(reasons.len(), 7);
    }

    #[test]
    fn test_timeout_reasons() {
        let reasons = vec![
            TimeoutReason::RequestTimeout,
            TimeoutReason::OperationDeadline,
            TimeoutReason::ProcessTimeout,
            TimeoutReason::IdleTimeout,
        ];
        assert_eq!(reasons.len(), 4);
    }

    #[test]
    fn test_recovery_hint_retry_now() {
        let hint = RecoveryHint::RetryNow;
        assert!(matches!(hint, RecoveryHint::RetryNow));
    }

    #[test]
    fn test_recovery_hint_retry_after() {
        let hint = RecoveryHint::RetryAfter(Duration::from_secs(30));
        if let RecoveryHint::RetryAfter(duration) = hint {
            assert_eq!(duration, Duration::from_secs(30));
        } else {
            panic!("Expected RetryAfter variant");
        }
    }

    #[test]
    fn test_recovery_hint_wait_for_user() {
        let hint = RecoveryHint::WaitForUser;
        assert!(matches!(hint, RecoveryHint::WaitForUser));
    }

    #[test]
    fn test_recovery_hint_stop_execution() {
        let hint = RecoveryHint::StopExecution;
        assert!(matches!(hint, RecoveryHint::StopExecution));
    }

    #[test]
    fn test_recovery_hint_resume_from_checkpoint() {
        let hint = RecoveryHint::ResumeFromCheckpoint;
        assert!(matches!(hint, RecoveryHint::ResumeFromCheckpoint));
    }

    #[test]
    fn test_classified_error_new() {
        let error = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Connection failed",
            RecoveryHint::RetryNow,
        );

        assert!(matches!(
            error.category,
            ErrorCategory::Transient(TransientReason::NetworkError)
        ));
        assert_eq!(error.message, "Connection failed");
        assert!(matches!(error.recovery_hint, RecoveryHint::RetryNow));
        assert!(error.context.is_empty());
    }

    #[test]
    fn test_classified_error_with_context() {
        let mut context = HashMap::new();
        context.insert("url".to_string(), "https://api.example.com".to_string());
        context.insert("status_code".to_string(), "503".to_string());

        let error = ClassifiedError::with_context(
            ErrorCategory::Transient(TransientReason::ServiceUnavailable),
            "Service temporarily unavailable",
            RecoveryHint::RetryAfter(Duration::from_secs(60)),
            context,
        );

        assert_eq!(error.context.len(), 2);
        assert_eq!(
            error.context.get("url"),
            Some(&"https://api.example.com".to_string())
        );
        assert_eq!(error.context.get("status_code"), Some(&"503".to_string()));
    }

    #[test]
    fn test_classified_error_add_context() {
        let error = ClassifiedError::new(
            ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
            "Invalid API key",
            RecoveryHint::WaitForUser,
        )
        .add_context("api_key_prefix", "sk-...")
        .add_context("endpoint", "/v1/chat/completions");

        assert_eq!(error.context.len(), 2);
        assert_eq!(
            error.context.get("api_key_prefix"),
            Some(&"sk-...".to_string())
        );
    }

    #[test]
    fn test_classified_error_is_transient() {
        let transient = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Network error",
            RecoveryHint::RetryNow,
        );
        let fatal = ClassifiedError::new(
            ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
            "Auth failed",
            RecoveryHint::StopExecution,
        );

        assert!(transient.is_transient());
        assert!(!fatal.is_transient());
    }

    #[test]
    fn test_classified_error_is_usage_limit() {
        let rate_limited = ClassifiedError::new(
            ErrorCategory::UsageLimit(UsageLimitReason::RateLimited),
            "Rate limited",
            RecoveryHint::RetryAfter(Duration::from_secs(60)),
        );
        let transient = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Network error",
            RecoveryHint::RetryNow,
        );

        assert!(rate_limited.is_usage_limit());
        assert!(!transient.is_usage_limit());
    }

    #[test]
    fn test_classified_error_is_fatal() {
        let fatal = ClassifiedError::new(
            ErrorCategory::Fatal(FatalReason::PermissionDenied),
            "Permission denied",
            RecoveryHint::WaitForUser,
        );
        let transient = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Network error",
            RecoveryHint::RetryNow,
        );

        assert!(fatal.is_fatal());
        assert!(!transient.is_fatal());
    }

    #[test]
    fn test_classified_error_is_timeout() {
        let timeout = ClassifiedError::new(
            ErrorCategory::Timeout(TimeoutReason::RequestTimeout),
            "Request timed out",
            RecoveryHint::RetryNow,
        );
        let fatal = ClassifiedError::new(
            ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
            "Auth failed",
            RecoveryHint::StopExecution,
        );

        assert!(timeout.is_timeout());
        assert!(!fatal.is_timeout());
    }

    #[test]
    fn test_classified_error_should_retry() {
        let retry_now = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Network error",
            RecoveryHint::RetryNow,
        );
        let retry_after = ClassifiedError::new(
            ErrorCategory::UsageLimit(UsageLimitReason::RateLimited),
            "Rate limited",
            RecoveryHint::RetryAfter(Duration::from_secs(60)),
        );
        let stop = ClassifiedError::new(
            ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
            "Auth failed",
            RecoveryHint::StopExecution,
        );

        assert!(retry_now.should_retry());
        assert!(retry_after.should_retry());
        assert!(!stop.should_retry());
    }

    #[test]
    fn test_classified_error_display() {
        let error = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Connection failed: timeout",
            RecoveryHint::RetryNow,
        );

        assert_eq!(format!("{}", error), "Connection failed: timeout");
    }

    #[test]
    fn test_classified_error_clone() {
        let error = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Network error",
            RecoveryHint::RetryNow,
        )
        .add_context("key", "value");

        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_classified_error_equality() {
        let error1 = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Network error",
            RecoveryHint::RetryNow,
        );
        let error2 = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Network error",
            RecoveryHint::RetryNow,
        );
        let error3 = ClassifiedError::new(
            ErrorCategory::Transient(TransientReason::NetworkError),
            "Different message",
            RecoveryHint::RetryNow,
        );

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_error_category_equality() {
        let cat1 = ErrorCategory::Transient(TransientReason::NetworkError);
        let cat2 = ErrorCategory::Transient(TransientReason::NetworkError);
        let cat3 = ErrorCategory::Transient(TransientReason::ServerError);

        assert_eq!(cat1, cat2);
        assert_ne!(cat1, cat3);
    }

    #[test]
    fn test_recovery_hint_equality() {
        let hint1 = RecoveryHint::RetryAfter(Duration::from_secs(30));
        let hint2 = RecoveryHint::RetryAfter(Duration::from_secs(30));
        let hint3 = RecoveryHint::RetryAfter(Duration::from_secs(60));

        assert_eq!(hint1, hint2);
        assert_ne!(hint1, hint3);
    }
}
