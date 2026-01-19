//! Error pattern detector for Claude Code errors
//!
//! This module provides regex-based pattern matching to detect and classify errors
//! from Claude Code agent output. It supports detection of rate limits, usage limits,
//! authentication errors, and timeout conditions.

use regex::Regex;
use std::time::Duration;

use super::{
    ClassifiedError, ErrorCategory, FatalReason, RecoveryHint, TimeoutReason, TransientReason,
    UsageLimitReason,
};

/// A pattern for matching errors in text output.
#[derive(Debug)]
pub struct ErrorPattern {
    /// The compiled regex pattern.
    regex: Regex,
    /// The error category to assign when this pattern matches.
    category: ErrorCategory,
    /// The recovery hint for this type of error.
    recovery_hint: RecoveryHint,
    /// A human-readable description of what this pattern detects.
    description: String,
}

impl ErrorPattern {
    /// Creates a new error pattern.
    ///
    /// # Arguments
    /// * `pattern` - The regex pattern string
    /// * `category` - The error category to assign on match
    /// * `recovery_hint` - The suggested recovery action
    /// * `description` - A description of what this pattern detects
    ///
    /// # Panics
    /// Panics if the regex pattern is invalid.
    pub fn new(
        pattern: &str,
        category: ErrorCategory,
        recovery_hint: RecoveryHint,
        description: impl Into<String>,
    ) -> Self {
        Self {
            regex: Regex::new(pattern).expect("Invalid regex pattern"),
            category,
            recovery_hint,
            description: description.into(),
        }
    }

    /// Creates a new error pattern with a pre-compiled regex.
    pub fn with_regex(
        regex: Regex,
        category: ErrorCategory,
        recovery_hint: RecoveryHint,
        description: impl Into<String>,
    ) -> Self {
        Self {
            regex,
            category,
            recovery_hint,
            description: description.into(),
        }
    }

    /// Returns the regex pattern.
    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    /// Returns the error category.
    pub fn category(&self) -> &ErrorCategory {
        &self.category
    }

    /// Returns the recovery hint.
    pub fn recovery_hint(&self) -> &RecoveryHint {
        &self.recovery_hint
    }

    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Checks if this pattern matches the given text.
    pub fn matches(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    /// Finds the first match in the text and returns the matched string.
    pub fn find<'a>(&self, text: &'a str) -> Option<&'a str> {
        self.regex.find(text).map(|m| m.as_str())
    }
}

/// Error detector that classifies errors from agent output using pattern matching.
#[derive(Debug)]
pub struct ErrorDetector {
    /// The list of patterns to match against, in priority order.
    patterns: Vec<ErrorPattern>,
}

impl Default for ErrorDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorDetector {
    /// Creates a new error detector with pre-configured patterns for Claude Code errors.
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    /// Creates an error detector with custom patterns.
    pub fn with_patterns(patterns: Vec<ErrorPattern>) -> Self {
        Self { patterns }
    }

    /// Returns the default patterns for Claude Code error detection.
    fn default_patterns() -> Vec<ErrorPattern> {
        vec![
            // Rate limit patterns (highest priority for usage limits)
            ErrorPattern::new(
                r"(?i)\b429\b",
                ErrorCategory::UsageLimit(UsageLimitReason::RateLimited),
                RecoveryHint::RetryAfter(Duration::from_secs(60)),
                "HTTP 429 status code",
            ),
            ErrorPattern::new(
                r"(?i)\brate[\s\-]?limit",
                ErrorCategory::UsageLimit(UsageLimitReason::RateLimited),
                RecoveryHint::RetryAfter(Duration::from_secs(60)),
                "Rate limit error message",
            ),
            ErrorPattern::new(
                r"(?i)too\s+many\s+requests",
                ErrorCategory::UsageLimit(UsageLimitReason::RateLimited),
                RecoveryHint::RetryAfter(Duration::from_secs(60)),
                "Too many requests error",
            ),
            // Usage limit patterns
            ErrorPattern::new(
                r"(?i)plan\s*limit",
                ErrorCategory::UsageLimit(UsageLimitReason::QuotaExhausted),
                RecoveryHint::WaitForUser,
                "Plan limit reached",
            ),
            ErrorPattern::new(
                r"(?i)usage\s*limit",
                ErrorCategory::UsageLimit(UsageLimitReason::QuotaExhausted),
                RecoveryHint::WaitForUser,
                "Usage limit exceeded",
            ),
            ErrorPattern::new(
                r"(?i)quota\s*(exceeded|exhausted)",
                ErrorCategory::UsageLimit(UsageLimitReason::QuotaExhausted),
                RecoveryHint::WaitForUser,
                "Quota exceeded",
            ),
            ErrorPattern::new(
                r"(?i)token\s*limit",
                ErrorCategory::UsageLimit(UsageLimitReason::TokenLimitExceeded),
                RecoveryHint::WaitForUser,
                "Token limit exceeded",
            ),
            ErrorPattern::new(
                r"(?i)concurrency\s*limit",
                ErrorCategory::UsageLimit(UsageLimitReason::ConcurrencyLimit),
                RecoveryHint::RetryAfter(Duration::from_secs(30)),
                "Concurrency limit reached",
            ),
            // Authentication error patterns
            ErrorPattern::new(
                r"(?i)\bunauthorized\b",
                ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
                RecoveryHint::WaitForUser,
                "Unauthorized access",
            ),
            ErrorPattern::new(
                r"(?i)auth(entication)?\s*(failed|error)",
                ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
                RecoveryHint::WaitForUser,
                "Authentication failed",
            ),
            ErrorPattern::new(
                r"(?i)invalid\s*(api\s*)?token",
                ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
                RecoveryHint::WaitForUser,
                "Invalid token",
            ),
            ErrorPattern::new(
                r"(?i)invalid\s*(api\s*)?key",
                ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
                RecoveryHint::WaitForUser,
                "Invalid API key",
            ),
            ErrorPattern::new(
                r"(?i)permission\s*denied",
                ErrorCategory::Fatal(FatalReason::PermissionDenied),
                RecoveryHint::WaitForUser,
                "Permission denied",
            ),
            ErrorPattern::new(
                r"(?i)access\s*denied",
                ErrorCategory::Fatal(FatalReason::PermissionDenied),
                RecoveryHint::WaitForUser,
                "Access denied",
            ),
            ErrorPattern::new(
                r"(?i)\b403\b",
                ErrorCategory::Fatal(FatalReason::PermissionDenied),
                RecoveryHint::WaitForUser,
                "HTTP 403 Forbidden",
            ),
            ErrorPattern::new(
                r"(?i)\b401\b",
                ErrorCategory::Fatal(FatalReason::AuthenticationFailed),
                RecoveryHint::WaitForUser,
                "HTTP 401 Unauthorized",
            ),
            // Network/transient error patterns
            ErrorPattern::new(
                r"(?i)connection\s*(refused|reset|timed?\s*out)",
                ErrorCategory::Transient(TransientReason::ConnectionReset),
                RecoveryHint::RetryNow,
                "Connection error",
            ),
            ErrorPattern::new(
                r"(?i)network\s*(error|failure)",
                ErrorCategory::Transient(TransientReason::NetworkError),
                RecoveryHint::RetryNow,
                "Network error",
            ),
            ErrorPattern::new(
                r"(?i)\b503\b",
                ErrorCategory::Transient(TransientReason::ServiceUnavailable),
                RecoveryHint::RetryAfter(Duration::from_secs(30)),
                "HTTP 503 Service Unavailable",
            ),
            ErrorPattern::new(
                r"(?i)service\s*unavailable",
                ErrorCategory::Transient(TransientReason::ServiceUnavailable),
                RecoveryHint::RetryAfter(Duration::from_secs(30)),
                "Service unavailable",
            ),
            ErrorPattern::new(
                r"(?i)\b5[0-9]{2}\b",
                ErrorCategory::Transient(TransientReason::ServerError),
                RecoveryHint::RetryAfter(Duration::from_secs(10)),
                "HTTP 5xx server error",
            ),
            // Timeout patterns
            ErrorPattern::new(
                r"(?i)request\s*time(d?\s*)?out",
                ErrorCategory::Timeout(TimeoutReason::RequestTimeout),
                RecoveryHint::RetryNow,
                "Request timeout",
            ),
            ErrorPattern::new(
                r"(?i)operation\s*time(d?\s*)?out",
                ErrorCategory::Timeout(TimeoutReason::OperationDeadline),
                RecoveryHint::RetryNow,
                "Operation timeout",
            ),
            ErrorPattern::new(
                r"(?i)deadline\s*(exceeded|expired)",
                ErrorCategory::Timeout(TimeoutReason::OperationDeadline),
                RecoveryHint::RetryNow,
                "Deadline exceeded",
            ),
            // Resource errors
            ErrorPattern::new(
                r"(?i)\b404\b",
                ErrorCategory::Fatal(FatalReason::ResourceNotFound),
                RecoveryHint::StopExecution,
                "HTTP 404 Not Found",
            ),
            ErrorPattern::new(
                r"(?i)not\s*found",
                ErrorCategory::Fatal(FatalReason::ResourceNotFound),
                RecoveryHint::StopExecution,
                "Resource not found",
            ),
            ErrorPattern::new(
                r"(?i)invalid\s*request",
                ErrorCategory::Fatal(FatalReason::InvalidRequest),
                RecoveryHint::StopExecution,
                "Invalid request",
            ),
            ErrorPattern::new(
                r"(?i)\b400\b",
                ErrorCategory::Fatal(FatalReason::InvalidRequest),
                RecoveryHint::StopExecution,
                "HTTP 400 Bad Request",
            ),
        ]
    }

    /// Adds a custom pattern to the detector.
    pub fn add_pattern(&mut self, pattern: ErrorPattern) {
        self.patterns.push(pattern);
    }

    /// Returns the number of patterns configured.
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Returns a reference to all configured patterns.
    pub fn patterns(&self) -> &[ErrorPattern] {
        &self.patterns
    }

    /// Classifies an error from text output.
    ///
    /// Matches the text against all patterns in order and returns a `ClassifiedError`
    /// for the first matching pattern. If no pattern matches, returns `None`.
    ///
    /// # Arguments
    /// * `text` - The error text to classify
    ///
    /// # Returns
    /// An optional `ClassifiedError` if a pattern matches
    pub fn classify_error(&self, text: &str) -> Option<ClassifiedError> {
        for pattern in &self.patterns {
            if pattern.matches(text) {
                // Safe to unwrap since matches() returned true
                let matched = pattern
                    .find(text)
                    .expect("pattern should match after matches() returned true");
                return Some(
                    ClassifiedError::new(
                        pattern.category.clone(),
                        pattern.description.clone(),
                        pattern.recovery_hint.clone(),
                    )
                    .add_context("matched_pattern", matched)
                    .add_context("original_text", text),
                );
            }
        }
        None
    }

    /// Classifies an error based on a process exit code.
    ///
    /// This is useful for detecting timeout conditions when a process is killed
    /// or terminates abnormally.
    ///
    /// # Arguments
    /// * `exit_code` - The process exit code
    ///
    /// # Returns
    /// An optional `ClassifiedError` for known exit codes
    ///
    /// # Exit Code Mappings
    /// - 124: Timeout (coreutils timeout command)
    /// - 137: SIGKILL (128 + 9, often used for timeout)
    /// - 143: SIGTERM (128 + 15)
    pub fn classify_exit_code(&self, exit_code: i32) -> Option<ClassifiedError> {
        match exit_code {
            // Exit code 124: timeout command (coreutils)
            124 => Some(
                ClassifiedError::new(
                    ErrorCategory::Timeout(TimeoutReason::ProcessTimeout),
                    "Process timed out (exit code 124)",
                    RecoveryHint::ResumeFromCheckpoint,
                )
                .add_context("exit_code", "124"),
            ),
            // Exit code 137: SIGKILL (128 + 9), often used for timeout
            137 => Some(
                ClassifiedError::new(
                    ErrorCategory::Timeout(TimeoutReason::ProcessTimeout),
                    "Process killed by SIGKILL (exit code 137)",
                    RecoveryHint::ResumeFromCheckpoint,
                )
                .add_context("exit_code", "137")
                .add_context("signal", "SIGKILL"),
            ),
            // Exit code 143: SIGTERM (128 + 15)
            143 => Some(
                ClassifiedError::new(
                    ErrorCategory::Timeout(TimeoutReason::ProcessTimeout),
                    "Process terminated by SIGTERM (exit code 143)",
                    RecoveryHint::ResumeFromCheckpoint,
                )
                .add_context("exit_code", "143")
                .add_context("signal", "SIGTERM"),
            ),
            // Other non-zero exit codes are not classified
            _ => None,
        }
    }

    /// Classifies an error from both text output and exit code.
    ///
    /// Text-based classification takes priority over exit code classification.
    /// This allows for more specific error detection from output messages.
    ///
    /// # Arguments
    /// * `text` - The error text output (can be empty)
    /// * `exit_code` - Optional process exit code
    ///
    /// # Returns
    /// An optional `ClassifiedError` if either method produces a match
    pub fn classify(&self, text: &str, exit_code: Option<i32>) -> Option<ClassifiedError> {
        // Try text classification first (higher priority)
        if !text.is_empty() {
            if let Some(error) = self.classify_error(text) {
                return Some(error);
            }
        }

        // Fall back to exit code classification
        exit_code.and_then(|code| self.classify_exit_code(code))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a detector for testing
    fn test_detector() -> ErrorDetector {
        ErrorDetector::new()
    }

    // ==================== ErrorPattern Tests ====================

    #[test]
    fn test_error_pattern_new() {
        let pattern = ErrorPattern::new(
            r"(?i)test",
            ErrorCategory::Fatal(FatalReason::InternalError),
            RecoveryHint::StopExecution,
            "Test pattern",
        );

        assert_eq!(pattern.description(), "Test pattern");
        assert!(pattern.matches("This is a TEST"));
        assert!(!pattern.matches("No match here"));
    }

    #[test]
    fn test_error_pattern_with_regex() {
        let regex = Regex::new(r"(?i)custom").unwrap();
        let pattern = ErrorPattern::with_regex(
            regex,
            ErrorCategory::Transient(TransientReason::NetworkError),
            RecoveryHint::RetryNow,
            "Custom regex pattern",
        );

        assert!(pattern.matches("CUSTOM error"));
        assert!(pattern.matches("custom error"));
    }

    #[test]
    fn test_error_pattern_find() {
        let pattern = ErrorPattern::new(
            r"\d{3}",
            ErrorCategory::Fatal(FatalReason::InternalError),
            RecoveryHint::StopExecution,
            "Status code pattern",
        );

        assert_eq!(pattern.find("Error 429 occurred"), Some("429"));
        assert_eq!(pattern.find("No code here"), None);
    }

    #[test]
    fn test_error_pattern_accessors() {
        let pattern = ErrorPattern::new(
            r"test",
            ErrorCategory::Fatal(FatalReason::InternalError),
            RecoveryHint::StopExecution,
            "Test",
        );

        assert!(pattern.regex().is_match("test"));
        assert!(matches!(
            pattern.category(),
            ErrorCategory::Fatal(FatalReason::InternalError)
        ));
        assert!(matches!(
            pattern.recovery_hint(),
            RecoveryHint::StopExecution
        ));
    }

    // ==================== ErrorDetector Construction Tests ====================

    #[test]
    fn test_error_detector_new() {
        let detector = ErrorDetector::new();
        assert!(detector.pattern_count() > 0);
    }

    #[test]
    fn test_error_detector_default() {
        let detector = ErrorDetector::default();
        assert!(detector.pattern_count() > 0);
    }

    #[test]
    fn test_error_detector_with_patterns() {
        let patterns = vec![ErrorPattern::new(
            r"custom",
            ErrorCategory::Fatal(FatalReason::InternalError),
            RecoveryHint::StopExecution,
            "Custom",
        )];
        let detector = ErrorDetector::with_patterns(patterns);
        assert_eq!(detector.pattern_count(), 1);
    }

    #[test]
    fn test_error_detector_add_pattern() {
        let mut detector = ErrorDetector::with_patterns(vec![]);
        assert_eq!(detector.pattern_count(), 0);

        detector.add_pattern(ErrorPattern::new(
            r"added",
            ErrorCategory::Fatal(FatalReason::InternalError),
            RecoveryHint::StopExecution,
            "Added pattern",
        ));
        assert_eq!(detector.pattern_count(), 1);
    }

    #[test]
    fn test_error_detector_patterns() {
        let detector = ErrorDetector::new();
        let patterns = detector.patterns();
        assert!(!patterns.is_empty());
    }

    // ==================== Rate Limit Pattern Tests ====================

    #[test]
    fn test_detect_rate_limit_429() {
        let detector = test_detector();
        let result = detector.classify_error("Error: 429 Too Many Requests");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::RateLimited)
        ));
        assert!(matches!(
            error.recovery_hint,
            RecoveryHint::RetryAfter(d) if d == Duration::from_secs(60)
        ));
    }

    #[test]
    fn test_detect_rate_limit_text() {
        let detector = test_detector();

        let test_cases = vec![
            "rate limit exceeded",
            "Rate Limit Error",
            "RateLimit reached",
            "rate-limit hit",
            "RATE LIMIT",
        ];

        for text in test_cases {
            let result = detector.classify_error(text);
            assert!(result.is_some(), "Should detect rate limit in: '{}'", text);
            let error = result.unwrap();
            assert!(
                matches!(
                    error.category,
                    ErrorCategory::UsageLimit(UsageLimitReason::RateLimited)
                ),
                "Wrong category for: '{}'",
                text
            );
        }
    }

    #[test]
    fn test_detect_too_many_requests() {
        let detector = test_detector();
        let result = detector.classify_error("Too many requests, please slow down");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::RateLimited)
        ));
    }

    // ==================== Usage Limit Pattern Tests ====================

    #[test]
    fn test_detect_plan_limit() {
        let detector = test_detector();
        let result = detector.classify_error("You have reached your plan limit");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::QuotaExhausted)
        ));
        assert!(matches!(error.recovery_hint, RecoveryHint::WaitForUser));
    }

    #[test]
    fn test_detect_usage_limit() {
        let detector = test_detector();
        let result = detector.classify_error("Usage limit exceeded for this month");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::QuotaExhausted)
        ));
    }

    #[test]
    fn test_detect_quota_exceeded() {
        let detector = test_detector();

        for text in ["Quota exceeded", "quota exhausted", "QUOTA EXCEEDED"] {
            let result = detector.classify_error(text);
            assert!(result.is_some(), "Should detect quota in: '{}'", text);
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::UsageLimit(UsageLimitReason::QuotaExhausted)
            ));
        }
    }

    #[test]
    fn test_detect_token_limit() {
        let detector = test_detector();
        let result = detector.classify_error("Token limit exceeded for this request");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::TokenLimitExceeded)
        ));
    }

    #[test]
    fn test_detect_concurrency_limit() {
        let detector = test_detector();
        let result = detector.classify_error("Concurrency limit reached");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::ConcurrencyLimit)
        ));
    }

    // ==================== Authentication Error Pattern Tests ====================

    #[test]
    fn test_detect_unauthorized() {
        let detector = test_detector();

        for text in ["Unauthorized access", "401 Unauthorized", "UNAUTHORIZED"] {
            let result = detector.classify_error(text);
            assert!(
                result.is_some(),
                "Should detect unauthorized in: '{}'",
                text
            );
            let error = result.unwrap();
            assert!(
                matches!(
                    error.category,
                    ErrorCategory::Fatal(FatalReason::AuthenticationFailed)
                ),
                "Wrong category for: '{}'",
                text
            );
        }
    }

    #[test]
    fn test_detect_authentication_failed() {
        let detector = test_detector();

        for text in [
            "Authentication failed",
            "authentication error",
            "Auth Error",
        ] {
            let result = detector.classify_error(text);
            assert!(result.is_some(), "Should detect auth error in: '{}'", text);
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Fatal(FatalReason::AuthenticationFailed)
            ));
        }
    }

    #[test]
    fn test_detect_invalid_token() {
        let detector = test_detector();

        for text in [
            "Invalid token",
            "invalid api token",
            "Invalid API Token",
            "INVALID TOKEN",
        ] {
            let result = detector.classify_error(text);
            assert!(
                result.is_some(),
                "Should detect invalid token in: '{}'",
                text
            );
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Fatal(FatalReason::AuthenticationFailed)
            ));
        }
    }

    #[test]
    fn test_detect_invalid_api_key() {
        let detector = test_detector();

        for text in ["Invalid API key", "invalid key", "Invalid api key provided"] {
            let result = detector.classify_error(text);
            assert!(result.is_some(), "Should detect invalid key in: '{}'", text);
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Fatal(FatalReason::AuthenticationFailed)
            ));
        }
    }

    #[test]
    fn test_detect_permission_denied() {
        let detector = test_detector();

        for text in [
            "Permission denied",
            "PERMISSION DENIED",
            "access denied",
            "403 Forbidden",
        ] {
            let result = detector.classify_error(text);
            assert!(
                result.is_some(),
                "Should detect permission error in: '{}'",
                text
            );
            let error = result.unwrap();
            assert!(
                matches!(
                    error.category,
                    ErrorCategory::Fatal(FatalReason::PermissionDenied)
                        | ErrorCategory::Fatal(FatalReason::AuthenticationFailed)
                ),
                "Wrong category for: '{}'",
                text
            );
        }
    }

    #[test]
    fn test_detect_http_401() {
        let detector = test_detector();
        let result = detector.classify_error("HTTP 401 - Authentication Required");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::Fatal(FatalReason::AuthenticationFailed)
        ));
    }

    #[test]
    fn test_detect_http_403() {
        let detector = test_detector();
        let result = detector.classify_error("HTTP 403 Forbidden");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::Fatal(FatalReason::PermissionDenied)
        ));
    }

    // ==================== Network/Transient Error Pattern Tests ====================

    #[test]
    fn test_detect_connection_errors() {
        let detector = test_detector();

        for text in [
            "Connection refused",
            "connection reset",
            "Connection timed out",
            "connection timeout",
        ] {
            let result = detector.classify_error(text);
            assert!(
                result.is_some(),
                "Should detect connection error in: '{}'",
                text
            );
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Transient(TransientReason::ConnectionReset)
            ));
        }
    }

    #[test]
    fn test_detect_network_error() {
        let detector = test_detector();

        for text in ["Network error occurred", "network failure", "NETWORK ERROR"] {
            let result = detector.classify_error(text);
            assert!(
                result.is_some(),
                "Should detect network error in: '{}'",
                text
            );
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Transient(TransientReason::NetworkError)
            ));
        }
    }

    #[test]
    fn test_detect_service_unavailable() {
        let detector = test_detector();

        for text in [
            "503 Service Unavailable",
            "Service unavailable",
            "service unavailable",
        ] {
            let result = detector.classify_error(text);
            assert!(
                result.is_some(),
                "Should detect service unavailable in: '{}'",
                text
            );
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Transient(TransientReason::ServiceUnavailable)
            ));
        }
    }

    #[test]
    fn test_detect_server_error_5xx() {
        let detector = test_detector();

        // 5xx errors should be detected as server errors
        for code in ["500", "502", "504", "520"] {
            let text = format!("HTTP {} Internal Server Error", code);
            let result = detector.classify_error(&text);
            assert!(result.is_some(), "Should detect 5xx error in: '{}'", text);
            let error = result.unwrap();
            assert!(
                matches!(
                    error.category,
                    ErrorCategory::Transient(TransientReason::ServerError)
                        | ErrorCategory::Transient(TransientReason::ServiceUnavailable)
                ),
                "Wrong category for: '{}'",
                text
            );
        }
    }

    // ==================== Timeout Pattern Tests ====================

    #[test]
    fn test_detect_request_timeout() {
        let detector = test_detector();

        for text in ["Request timeout", "request timed out", "REQUEST TIMEOUT"] {
            let result = detector.classify_error(text);
            assert!(result.is_some(), "Should detect timeout in: '{}'", text);
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Timeout(TimeoutReason::RequestTimeout)
            ));
        }
    }

    #[test]
    fn test_detect_operation_timeout() {
        let detector = test_detector();

        for text in ["Operation timeout", "operation timed out"] {
            let result = detector.classify_error(text);
            assert!(result.is_some(), "Should detect timeout in: '{}'", text);
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Timeout(TimeoutReason::OperationDeadline)
            ));
        }
    }

    #[test]
    fn test_detect_deadline_exceeded() {
        let detector = test_detector();

        for text in ["Deadline exceeded", "deadline expired", "DEADLINE EXCEEDED"] {
            let result = detector.classify_error(text);
            assert!(result.is_some(), "Should detect deadline in: '{}'", text);
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Timeout(TimeoutReason::OperationDeadline)
            ));
        }
    }

    // ==================== Resource Error Pattern Tests ====================

    #[test]
    fn test_detect_not_found() {
        let detector = test_detector();

        for text in ["404 Not Found", "Resource not found", "NOT FOUND"] {
            let result = detector.classify_error(text);
            assert!(result.is_some(), "Should detect not found in: '{}'", text);
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Fatal(FatalReason::ResourceNotFound)
            ));
        }
    }

    #[test]
    fn test_detect_invalid_request() {
        let detector = test_detector();

        for text in ["Invalid request", "400 Bad Request", "INVALID REQUEST"] {
            let result = detector.classify_error(text);
            assert!(
                result.is_some(),
                "Should detect invalid request in: '{}'",
                text
            );
            let error = result.unwrap();
            assert!(matches!(
                error.category,
                ErrorCategory::Fatal(FatalReason::InvalidRequest)
            ));
        }
    }

    // ==================== Exit Code Classification Tests ====================

    #[test]
    fn test_classify_exit_code_124() {
        let detector = test_detector();
        let result = detector.classify_exit_code(124);

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::Timeout(TimeoutReason::ProcessTimeout)
        ));
        assert!(matches!(
            error.recovery_hint,
            RecoveryHint::ResumeFromCheckpoint
        ));
        assert_eq!(error.context.get("exit_code"), Some(&"124".to_string()));
    }

    #[test]
    fn test_classify_exit_code_137() {
        let detector = test_detector();
        let result = detector.classify_exit_code(137);

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::Timeout(TimeoutReason::ProcessTimeout)
        ));
        assert_eq!(error.context.get("signal"), Some(&"SIGKILL".to_string()));
    }

    #[test]
    fn test_classify_exit_code_143() {
        let detector = test_detector();
        let result = detector.classify_exit_code(143);

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::Timeout(TimeoutReason::ProcessTimeout)
        ));
        assert_eq!(error.context.get("signal"), Some(&"SIGTERM".to_string()));
    }

    #[test]
    fn test_classify_exit_code_unknown() {
        let detector = test_detector();

        // Regular non-zero exit codes should not be classified
        for code in [0, 1, 2, 127, 128, 255] {
            let result = detector.classify_exit_code(code);
            assert!(
                result.is_none(),
                "Exit code {} should not be classified",
                code
            );
        }
    }

    // ==================== Combined Classification Tests ====================

    #[test]
    fn test_classify_text_priority_over_exit_code() {
        let detector = test_detector();

        // Text classification should take priority
        let result = detector.classify("rate limit exceeded", Some(124));
        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::RateLimited)
        ));
    }

    #[test]
    fn test_classify_fallback_to_exit_code() {
        let detector = test_detector();

        // With empty or non-matching text, should fall back to exit code
        let result = detector.classify("", Some(124));
        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::Timeout(TimeoutReason::ProcessTimeout)
        ));
    }

    #[test]
    fn test_classify_no_match() {
        let detector = test_detector();

        let result = detector.classify("Everything is fine", None);
        assert!(result.is_none());

        let result = detector.classify("Normal output", Some(0));
        assert!(result.is_none());
    }

    // ==================== Context Tests ====================

    #[test]
    fn test_classified_error_has_context() {
        let detector = test_detector();
        let result = detector.classify_error("Error 429: rate limit exceeded");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(error.context.contains_key("matched_pattern"));
        assert!(error.context.contains_key("original_text"));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_classify_empty_text() {
        let detector = test_detector();
        let result = detector.classify_error("");
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_whitespace_only() {
        let detector = test_detector();
        let result = detector.classify_error("   \n\t  ");
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_mixed_case() {
        let detector = test_detector();
        let result = detector.classify_error("RaTe LiMiT ExCeEdEd");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::RateLimited)
        ));
    }

    #[test]
    fn test_classify_multiline_text() {
        let detector = test_detector();
        let result = detector.classify_error("Line 1\nError: rate limit exceeded\nLine 3");

        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::RateLimited)
        ));
    }

    #[test]
    fn test_classify_partial_matches() {
        let detector = test_detector();

        // Should match "rate limit" even in longer strings
        let result = detector.classify_error("The server returned a rate limit error message");
        assert!(result.is_some());

        // Should not match partial words
        let result = detector.classify_error("rated highly");
        assert!(result.is_none());
    }

    #[test]
    fn test_pattern_priority() {
        let detector = test_detector();

        // 429 is more specific than general 5xx pattern
        let result = detector.classify_error("HTTP 429");
        assert!(result.is_some());
        let error = result.unwrap();
        assert!(matches!(
            error.category,
            ErrorCategory::UsageLimit(UsageLimitReason::RateLimited)
        ));
    }
}
