//! Futile retry detection for the iteration system.
//!
//! This module provides the `FutileRetryDetector` which analyzes error patterns
//! to determine when further retries are unlikely to succeed and should be
//! stopped early.

use super::context::{ErrorCategory, IterationContext};

/// Verdict from the futility detector about whether to continue retrying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FutilityVerdict {
    /// Continue with the next iteration
    Continue,
    /// Pause and request guidance from the user
    PauseForGuidance {
        /// Reason for pausing
        reason: String,
        /// Suggested actions
        suggestions: Vec<String>,
    },
    /// Defer this story and move to the next one
    DeferStory {
        /// Reason for deferring
        reason: String,
    },
    /// Fatal error that cannot be recovered from
    Fatal {
        /// Reason for the fatal verdict
        reason: String,
    },
}

impl FutilityVerdict {
    /// Check if this verdict allows continuing.
    pub fn should_continue(&self) -> bool {
        matches!(self, FutilityVerdict::Continue)
    }

    /// Get the reason if this is not a Continue verdict.
    pub fn reason(&self) -> Option<&str> {
        match self {
            FutilityVerdict::Continue => None,
            FutilityVerdict::PauseForGuidance { reason, .. } => Some(reason),
            FutilityVerdict::DeferStory { reason } => Some(reason),
            FutilityVerdict::Fatal { reason } => Some(reason),
        }
    }
}

/// Configuration for the futility detector.
#[derive(Debug, Clone)]
pub struct FutilityConfig {
    /// Number of consecutive same errors before flagging oscillation
    pub oscillation_threshold: u32,
    /// Number of consecutive errors without progress before flagging stagnation
    pub stagnation_threshold: u32,
    /// Error categories that are considered fatal (cannot be retried)
    pub fatal_categories: Vec<ErrorCategory>,
    /// Whether to enable pattern detection
    pub enable_pattern_detection: bool,
}

impl Default for FutilityConfig {
    fn default() -> Self {
        Self {
            oscillation_threshold: 3,
            stagnation_threshold: 4,
            fatal_categories: vec![ErrorCategory::Environment],
            enable_pattern_detection: true,
        }
    }
}

/// Detector for futile retry patterns.
///
/// Analyzes the error history from an iteration context to determine
/// if further retries are unlikely to succeed.
pub struct FutileRetryDetector {
    config: FutilityConfig,
}

impl FutileRetryDetector {
    /// Create a new futility detector with default configuration.
    pub fn new() -> Self {
        Self {
            config: FutilityConfig::default(),
        }
    }

    /// Create a new futility detector with custom configuration.
    pub fn with_config(config: FutilityConfig) -> Self {
        Self { config }
    }

    /// Analyze the iteration context and return a verdict.
    pub fn analyze(&self, context: &IterationContext) -> FutilityVerdict {
        // Check for fatal errors first
        if let Some(verdict) = self.check_fatal_errors(context) {
            return verdict;
        }

        // Check for oscillation patterns (A -> B -> A -> B)
        if self.config.enable_pattern_detection {
            if let Some(verdict) = self.check_oscillation(context) {
                return verdict;
            }
        }

        // Check for stagnation (same error repeatedly)
        if let Some(verdict) = self.check_stagnation(context) {
            return verdict;
        }

        // Check for error rate acceleration
        if let Some(verdict) = self.check_acceleration(context) {
            return verdict;
        }

        FutilityVerdict::Continue
    }

    /// Check for fatal errors that cannot be recovered from.
    fn check_fatal_errors(&self, context: &IterationContext) -> Option<FutilityVerdict> {
        for error in &context.error_history {
            if self.config.fatal_categories.contains(&error.category) {
                return Some(FutilityVerdict::Fatal {
                    reason: format!("Fatal {} error: {}", error.category.as_str(), error.message),
                });
            }
        }
        None
    }

    /// Check for oscillation pattern (alternating between two error types).
    fn check_oscillation(&self, context: &IterationContext) -> Option<FutilityVerdict> {
        let signatures = context.error_signature_sequence();
        if signatures.len() < 4 {
            return None;
        }

        // Check for A-B-A-B pattern in the last 4 errors
        let len = signatures.len();
        if signatures[len - 1] == signatures[len - 3]
            && signatures[len - 2] == signatures[len - 4]
            && signatures[len - 1] != signatures[len - 2]
        {
            let error_a = &signatures[len - 1];
            let error_b = &signatures[len - 2];
            return Some(FutilityVerdict::PauseForGuidance {
                reason: format!(
                    "Detected oscillating error pattern: {} <-> {}. \
                     Fixing one issue causes the other to reappear.",
                    error_a, error_b
                ),
                suggestions: vec![
                    "Review the conflicting requirements".to_string(),
                    "Consider addressing both issues simultaneously".to_string(),
                    "Check if there's a design issue causing the oscillation".to_string(),
                ],
            });
        }

        None
    }

    /// Check for stagnation (same error repeated multiple times).
    fn check_stagnation(&self, context: &IterationContext) -> Option<FutilityVerdict> {
        let signatures = context.error_signature_sequence();
        if signatures.is_empty() {
            return None;
        }

        let last_signature = signatures.last().unwrap();
        let consecutive_count = context.repeated_error_count(last_signature);

        if consecutive_count >= self.config.stagnation_threshold {
            return Some(FutilityVerdict::DeferStory {
                reason: format!(
                    "Same error '{}' occurred {} times consecutively. \
                     The agent may not be able to resolve this issue without guidance.",
                    last_signature, consecutive_count
                ),
            });
        }

        // Also check for near-stagnation (3+ times)
        if consecutive_count >= self.config.oscillation_threshold {
            return Some(FutilityVerdict::PauseForGuidance {
                reason: format!(
                    "Error '{}' has occurred {} times. \
                     Consider providing additional context or breaking down the task.",
                    last_signature, consecutive_count
                ),
                suggestions: vec![
                    "Provide more specific implementation guidance".to_string(),
                    "Break the story into smaller subtasks".to_string(),
                    "Check for missing dependencies or prerequisites".to_string(),
                ],
            });
        }

        None
    }

    /// Check for error rate acceleration (errors happening faster).
    fn check_acceleration(&self, context: &IterationContext) -> Option<FutilityVerdict> {
        // If we have many errors relative to iterations, might be getting worse
        let error_rate =
            context.error_history.len() as f64 / context.current_iteration.max(1) as f64;

        // If every iteration is producing an error and we're past iteration 5
        if context.current_iteration >= 5 && error_rate > 0.9 {
            let remaining = context.max_iterations - context.current_iteration;
            if remaining <= 2 {
                return Some(FutilityVerdict::PauseForGuidance {
                    reason: format!(
                        "High error rate ({:.0}%) with only {} iterations remaining.",
                        error_rate * 100.0,
                        remaining
                    ),
                    suggestions: vec![
                        "Consider if the task is appropriately scoped".to_string(),
                        "Review error patterns to identify root cause".to_string(),
                    ],
                });
            }
        }

        None
    }

    /// Get a summary of the error patterns detected.
    pub fn summarize_patterns(&self, context: &IterationContext) -> PatternSummary {
        // Count by category
        let errors_by_category = context.error_count_by_category();

        // Check for repeated signatures
        let signatures = context.error_signature_sequence();
        let mut signature_counts: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        for sig in &signatures {
            *signature_counts.entry(sig.clone()).or_insert(0) += 1;
        }

        let most_frequent_error = signature_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(sig, count)| (sig.clone(), *count));

        // Detect patterns
        let has_oscillation = self.check_oscillation(context).is_some();
        let has_stagnation = self.check_stagnation(context).is_some();

        // Calculate progress
        let total_errors = context.error_history.len() as u32;
        let total_iterations = context.current_iteration;

        PatternSummary {
            errors_by_category,
            most_frequent_error,
            has_oscillation,
            has_stagnation,
            total_errors,
            total_iterations,
        }
    }
}

impl Default for FutileRetryDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of detected patterns in the error history.
#[derive(Debug, Default)]
pub struct PatternSummary {
    /// Errors grouped by category
    pub errors_by_category: std::collections::HashMap<ErrorCategory, u32>,
    /// Most frequently occurring error (signature, count)
    pub most_frequent_error: Option<(String, u32)>,
    /// Whether oscillation pattern was detected
    pub has_oscillation: bool,
    /// Whether stagnation pattern was detected
    pub has_stagnation: bool,
    /// Total number of errors
    pub total_errors: u32,
    /// Total iterations so far
    pub total_iterations: u32,
}

impl PatternSummary {
    /// Get the error rate as a percentage.
    pub fn error_rate(&self) -> f64 {
        if self.total_iterations == 0 {
            0.0
        } else {
            (self.total_errors as f64 / self.total_iterations as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iteration::context::IterationError;

    fn make_error(iteration: u32, category: ErrorCategory, gate: &str) -> IterationError {
        IterationError::new(iteration, category, "test error").with_gate(gate)
    }

    #[test]
    fn test_futility_verdict_should_continue() {
        assert!(FutilityVerdict::Continue.should_continue());
        assert!(!FutilityVerdict::Fatal {
            reason: "test".to_string()
        }
        .should_continue());
        assert!(!FutilityVerdict::DeferStory {
            reason: "test".to_string()
        }
        .should_continue());
    }

    #[test]
    fn test_futility_verdict_reason() {
        assert!(FutilityVerdict::Continue.reason().is_none());
        assert_eq!(
            FutilityVerdict::Fatal {
                reason: "test".to_string()
            }
            .reason(),
            Some("test")
        );
    }

    #[test]
    fn test_futility_config_default() {
        let config = FutilityConfig::default();
        assert_eq!(config.oscillation_threshold, 3);
        assert_eq!(config.stagnation_threshold, 4);
        assert!(config.enable_pattern_detection);
    }

    #[test]
    fn test_detector_new() {
        let detector = FutileRetryDetector::new();
        assert_eq!(detector.config.oscillation_threshold, 3);
    }

    #[test]
    fn test_detector_with_config() {
        let config = FutilityConfig {
            oscillation_threshold: 5,
            ..Default::default()
        };
        let detector = FutileRetryDetector::with_config(config);
        assert_eq!(detector.config.oscillation_threshold, 5);
    }

    #[test]
    fn test_detector_continue_empty_context() {
        let detector = FutileRetryDetector::new();
        let context = IterationContext::new("US-001", 10);
        assert_eq!(detector.analyze(&context), FutilityVerdict::Continue);
    }

    #[test]
    fn test_detector_continue_few_errors() {
        let detector = FutileRetryDetector::new();
        let mut context = IterationContext::new("US-001", 10);
        context.start_iteration(2);
        context.record_error(make_error(1, ErrorCategory::Lint, "lint"));
        assert_eq!(detector.analyze(&context), FutilityVerdict::Continue);
    }

    #[test]
    fn test_detector_fatal_error() {
        let detector = FutileRetryDetector::new();
        let mut context = IterationContext::new("US-001", 10);
        context.start_iteration(1);
        context.record_error(make_error(1, ErrorCategory::Environment, "env"));

        let verdict = detector.analyze(&context);
        assert!(matches!(verdict, FutilityVerdict::Fatal { .. }));
    }

    #[test]
    fn test_detector_oscillation() {
        let detector = FutileRetryDetector::new();
        let mut context = IterationContext::new("US-001", 10);
        context.start_iteration(5);

        // Create A-B-A-B pattern
        context.record_error(make_error(1, ErrorCategory::Lint, "lint"));
        context.record_error(make_error(2, ErrorCategory::Format, "format"));
        context.record_error(make_error(3, ErrorCategory::Lint, "lint"));
        context.record_error(make_error(4, ErrorCategory::Format, "format"));

        let verdict = detector.analyze(&context);
        assert!(
            matches!(verdict, FutilityVerdict::PauseForGuidance { reason, .. } if reason.contains("oscillating"))
        );
    }

    #[test]
    fn test_detector_stagnation() {
        let detector = FutileRetryDetector::new();
        let mut context = IterationContext::new("US-001", 10);
        context.start_iteration(5);

        // Same error 4 times (stagnation threshold)
        for i in 1..=4 {
            context.record_error(make_error(i, ErrorCategory::Lint, "lint"));
        }

        let verdict = detector.analyze(&context);
        assert!(matches!(verdict, FutilityVerdict::DeferStory { .. }));
    }

    #[test]
    fn test_detector_near_stagnation() {
        let detector = FutileRetryDetector::new();
        let mut context = IterationContext::new("US-001", 10);
        context.start_iteration(4);

        // Same error 3 times (oscillation threshold but not stagnation)
        for i in 1..=3 {
            context.record_error(make_error(i, ErrorCategory::Lint, "lint"));
        }

        let verdict = detector.analyze(&context);
        assert!(matches!(verdict, FutilityVerdict::PauseForGuidance { .. }));
    }

    #[test]
    fn test_detector_high_error_rate() {
        let detector = FutileRetryDetector::new();
        let mut context = IterationContext::new("US-001", 10);
        context.start_iteration(9); // 9th iteration, only 1 remaining

        // Different errors each time to avoid stagnation detection
        context.record_error(make_error(1, ErrorCategory::Lint, "lint"));
        context.record_error(make_error(2, ErrorCategory::Format, "format"));
        context.record_error(make_error(3, ErrorCategory::Coverage, "coverage"));
        context.record_error(make_error(4, ErrorCategory::Compilation, "compile"));
        context.record_error(make_error(5, ErrorCategory::Test, "test"));
        context.record_error(make_error(6, ErrorCategory::Lint, "lint2"));
        context.record_error(make_error(7, ErrorCategory::Format, "format2"));
        context.record_error(make_error(8, ErrorCategory::SecurityAudit, "audit"));
        context.record_error(make_error(9, ErrorCategory::Git, "git"));

        let verdict = detector.analyze(&context);
        assert!(matches!(verdict, FutilityVerdict::PauseForGuidance { .. }));
    }

    #[test]
    fn test_pattern_summary() {
        let detector = FutileRetryDetector::new();
        let mut context = IterationContext::new("US-001", 10);
        context.start_iteration(3);

        context.record_error(make_error(1, ErrorCategory::Lint, "lint"));
        context.record_error(make_error(2, ErrorCategory::Lint, "lint"));
        context.record_error(make_error(3, ErrorCategory::Format, "format"));

        let summary = detector.summarize_patterns(&context);
        assert_eq!(summary.total_errors, 3);
        assert_eq!(summary.total_iterations, 3);
        assert_eq!(
            summary.errors_by_category.get(&ErrorCategory::Lint),
            Some(&2)
        );
        assert!(summary.most_frequent_error.is_some());
        assert_eq!(summary.error_rate(), 100.0);
    }

    #[test]
    fn test_pattern_summary_error_rate() {
        let summary = PatternSummary {
            total_errors: 5,
            total_iterations: 10,
            ..Default::default()
        };
        assert_eq!(summary.error_rate(), 50.0);
    }

    #[test]
    fn test_pattern_summary_error_rate_zero_iterations() {
        let summary = PatternSummary::default();
        assert_eq!(summary.error_rate(), 0.0);
    }
}
