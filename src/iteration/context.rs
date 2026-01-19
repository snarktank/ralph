//! Iteration context for learning transfer between retries.
//!
//! This module provides the `IterationContext` struct which accumulates
//! information across iterations to help subsequent attempts learn from
//! previous failures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error information from a single iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationError {
    /// Which iteration this error occurred in (1-indexed)
    pub iteration: u32,
    /// Category of the error (e.g., "compilation", "quality_gate", "agent")
    pub category: ErrorCategory,
    /// Human-readable error message
    pub message: String,
    /// Specific gate that failed (if applicable)
    pub failed_gate: Option<String>,
    /// Files involved in the error (if known)
    pub affected_files: Vec<String>,
}

impl IterationError {
    /// Create a new iteration error.
    pub fn new(iteration: u32, category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            iteration,
            category,
            message: message.into(),
            failed_gate: None,
            affected_files: Vec::new(),
        }
    }

    /// Set the failed gate name.
    pub fn with_gate(mut self, gate: impl Into<String>) -> Self {
        self.failed_gate = Some(gate.into());
        self
    }

    /// Set the affected files.
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.affected_files = files;
        self
    }

    /// Get a normalized representation for pattern matching.
    /// This is used to detect oscillating or repeating errors.
    pub fn signature(&self) -> String {
        let gate_part = self.failed_gate.as_deref().unwrap_or("none");
        format!("{}:{}", self.category.as_str(), gate_part)
    }
}

/// Categories of errors that can occur during iteration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Rust compilation errors (cargo check)
    Compilation,
    /// Clippy lint failures
    Lint,
    /// Formatting check failures
    Format,
    /// Test failures
    Test,
    /// Coverage threshold not met
    Coverage,
    /// Security audit failures
    SecurityAudit,
    /// Agent execution errors (timeout, crash, etc.)
    AgentExecution,
    /// Git operation failures
    Git,
    /// Environment/configuration issues
    Environment,
    /// Other/unknown errors
    Other,
}

impl ErrorCategory {
    /// Get a string representation of the category.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::Compilation => "compilation",
            ErrorCategory::Lint => "lint",
            ErrorCategory::Format => "format",
            ErrorCategory::Test => "test",
            ErrorCategory::Coverage => "coverage",
            ErrorCategory::SecurityAudit => "security_audit",
            ErrorCategory::AgentExecution => "agent_execution",
            ErrorCategory::Git => "git",
            ErrorCategory::Environment => "environment",
            ErrorCategory::Other => "other",
        }
    }

    /// Parse error message to determine category.
    pub fn from_error_message(message: &str, gate_name: Option<&str>) -> Self {
        // First check if we have a gate name
        if let Some(gate) = gate_name {
            return match gate {
                "coverage" => ErrorCategory::Coverage,
                "lint" => ErrorCategory::Lint,
                "format" => ErrorCategory::Format,
                "security_audit" => ErrorCategory::SecurityAudit,
                _ => ErrorCategory::Other,
            };
        }

        // Otherwise, try to infer from the message
        let lower = message.to_lowercase();
        if lower.contains("cargo check") || lower.contains("compile") || lower.contains("rustc") {
            ErrorCategory::Compilation
        } else if lower.contains("clippy") || lower.contains("lint") {
            ErrorCategory::Lint
        } else if lower.contains("fmt") || lower.contains("format") {
            ErrorCategory::Format
        } else if lower.contains("test") {
            ErrorCategory::Test
        } else if lower.contains("coverage") {
            ErrorCategory::Coverage
        } else if lower.contains("audit") || lower.contains("security") {
            ErrorCategory::SecurityAudit
        } else if lower.contains("git") {
            ErrorCategory::Git
        } else if lower.contains("agent") || lower.contains("timeout") || lower.contains("claude") {
            ErrorCategory::AgentExecution
        } else if lower.contains("environment") || lower.contains("config") {
            ErrorCategory::Environment
        } else {
            ErrorCategory::Other
        }
    }
}

/// Hints about approaches that have worked for similar errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproachHint {
    /// Description of the approach
    pub description: String,
    /// Success rate of this approach (0.0 to 1.0)
    pub success_rate: f64,
    /// Number of times this approach was tried
    pub sample_count: u32,
}

impl ApproachHint {
    /// Create a new approach hint.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            success_rate: 0.0,
            sample_count: 0,
        }
    }

    /// Update the hint with a new success/failure result.
    pub fn record_result(&mut self, succeeded: bool) {
        let successes = (self.success_rate * self.sample_count as f64) as u32;
        self.sample_count += 1;
        let new_successes = if succeeded { successes + 1 } else { successes };
        self.success_rate = new_successes as f64 / self.sample_count as f64;
    }
}

/// User-provided steering guidance for a failing story.
///
/// This allows users to provide additional context and instructions
/// when the system detects that a story is stuck or failing repeatedly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringGuidance {
    /// Additional context or instructions from the user
    pub guidance_text: String,
    /// Optional modified acceptance criteria
    pub modified_acceptance_criteria: Option<Vec<String>>,
    /// Files to focus on
    pub focus_files: Vec<String>,
    /// Files to avoid modifying
    pub avoid_files: Vec<String>,
    /// Quality gates to temporarily relax
    pub relaxed_gates: Vec<String>,
    /// When this guidance was provided
    pub provided_at_iteration: u32,
}

impl SteeringGuidance {
    /// Create new steering guidance with just text.
    pub fn new(guidance_text: impl Into<String>, iteration: u32) -> Self {
        Self {
            guidance_text: guidance_text.into(),
            modified_acceptance_criteria: None,
            focus_files: Vec::new(),
            avoid_files: Vec::new(),
            relaxed_gates: Vec::new(),
            provided_at_iteration: iteration,
        }
    }

    /// Add files to focus on.
    pub fn with_focus_files(mut self, files: Vec<String>) -> Self {
        self.focus_files = files;
        self
    }

    /// Add files to avoid.
    pub fn with_avoid_files(mut self, files: Vec<String>) -> Self {
        self.avoid_files = files;
        self
    }

    /// Add gates to relax.
    pub fn with_relaxed_gates(mut self, gates: Vec<String>) -> Self {
        self.relaxed_gates = gates;
        self
    }

    /// Build a prompt section for this guidance.
    pub fn build_prompt_section(&self) -> String {
        let mut section = String::from("\n## User Steering Guidance\n\n");
        section.push_str(&format!(
            "The user has reviewed the errors and provided the following guidance:\n\n{}\n\n",
            self.guidance_text
        ));

        if let Some(ref criteria) = self.modified_acceptance_criteria {
            section.push_str("### Updated Acceptance Criteria\n");
            for (i, criterion) in criteria.iter().enumerate() {
                section.push_str(&format!("{}. {}\n", i + 1, criterion));
            }
            section.push('\n');
        }

        if !self.focus_files.is_empty() {
            section.push_str("### Files to Focus On\n");
            for file in &self.focus_files {
                section.push_str(&format!("- {}\n", file));
            }
            section.push('\n');
        }

        if !self.avoid_files.is_empty() {
            section.push_str("### Files to Avoid Modifying\n");
            for file in &self.avoid_files {
                section.push_str(&format!("- {}\n", file));
            }
            section.push('\n');
        }

        section.push_str("**IMPORTANT**: Follow the user's guidance carefully and address the specific issues mentioned.\n");

        section
    }
}

/// Context that accumulates across iterations to help learning.
///
/// This struct is passed between iterations and accumulates information
/// about what has been tried and what has failed, allowing subsequent
/// iterations to make more informed decisions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IterationContext {
    /// History of errors from previous iterations
    pub error_history: Vec<IterationError>,
    /// Files that have already passed quality gates (keyed by gate name)
    pub partial_progress: HashMap<String, Vec<String>>,
    /// Hints about approaches that have worked for similar errors
    pub approach_hints: Vec<ApproachHint>,
    /// Current iteration number (1-indexed)
    pub current_iteration: u32,
    /// Maximum iterations allowed
    pub max_iterations: u32,
    /// Story ID being executed
    pub story_id: String,
    /// User-provided steering guidance (if any)
    pub steering_guidance: Option<SteeringGuidance>,
}

impl IterationContext {
    /// Create a new iteration context for a story.
    pub fn new(story_id: impl Into<String>, max_iterations: u32) -> Self {
        Self {
            error_history: Vec::new(),
            partial_progress: HashMap::new(),
            approach_hints: Vec::new(),
            current_iteration: 0,
            max_iterations,
            story_id: story_id.into(),
            steering_guidance: None,
        }
    }

    /// Start a new iteration.
    pub fn start_iteration(&mut self, iteration: u32) {
        self.current_iteration = iteration;
    }

    /// Record an error from the current iteration.
    pub fn record_error(&mut self, error: IterationError) {
        self.error_history.push(error);
    }

    /// Record that certain files passed a specific gate.
    pub fn record_partial_progress(&mut self, gate_name: impl Into<String>, files: Vec<String>) {
        self.partial_progress
            .entry(gate_name.into())
            .or_default()
            .extend(files);
    }

    /// Add an approach hint.
    pub fn add_hint(&mut self, hint: ApproachHint) {
        self.approach_hints.push(hint);
    }

    /// Set steering guidance from the user.
    pub fn set_steering_guidance(&mut self, guidance: SteeringGuidance) {
        self.steering_guidance = Some(guidance);
    }

    /// Get the count of errors by category.
    pub fn error_count_by_category(&self) -> HashMap<ErrorCategory, u32> {
        let mut counts = HashMap::new();
        for error in &self.error_history {
            *counts.entry(error.category).or_insert(0) += 1;
        }
        counts
    }

    /// Get the most recent error of a given category.
    pub fn last_error_of_category(&self, category: ErrorCategory) -> Option<&IterationError> {
        self.error_history
            .iter()
            .rev()
            .find(|e| e.category == category)
    }

    /// Check if the same error has occurred in the last N iterations.
    pub fn repeated_error_count(&self, signature: &str) -> u32 {
        self.error_history
            .iter()
            .filter(|e| e.signature() == signature)
            .count() as u32
    }

    /// Get a sequence of error signatures for pattern detection.
    pub fn error_signature_sequence(&self) -> Vec<String> {
        self.error_history.iter().map(|e| e.signature()).collect()
    }

    /// Build a context string to inject into agent prompts.
    ///
    /// This provides the agent with information about previous failures
    /// to help it avoid repeating the same mistakes.
    pub fn build_prompt_context(&self) -> String {
        if self.error_history.is_empty() {
            return String::new();
        }

        let mut context = String::from("\n## Previous Iteration Context\n\n");

        // Add error history summary
        context.push_str("### Previous Errors\n\n");
        for error in &self.error_history {
            context.push_str(&format!(
                "- **Iteration {}** ({}): {}\n",
                error.iteration,
                error.category.as_str(),
                error.message
            ));
            if let Some(gate) = &error.failed_gate {
                context.push_str(&format!("  - Failed gate: {}\n", gate));
            }
            if !error.affected_files.is_empty() {
                let files = error
                    .affected_files
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>();
                context.push_str(&format!("  - Affected files: {}\n", files.join(", ")));
            }
        }

        // Add hints if available
        if !self.approach_hints.is_empty() {
            context.push_str("\n### Suggested Approaches\n\n");
            for hint in &self.approach_hints {
                context.push_str(&format!(
                    "- {} (success rate: {:.0}%)\n",
                    hint.description,
                    hint.success_rate * 100.0
                ));
            }
        }

        // Add partial progress if available
        if !self.partial_progress.is_empty() {
            context.push_str("\n### Partial Progress\n\n");
            for (gate, files) in &self.partial_progress {
                if !files.is_empty() {
                    context.push_str(&format!(
                        "- Gate '{}': {} files already passing\n",
                        gate,
                        files.len()
                    ));
                }
            }
        }

        // Add steering guidance if provided
        if let Some(ref guidance) = self.steering_guidance {
            context.push_str(&guidance.build_prompt_section());
        }

        context.push_str(&format!(
            "\n**Note**: This is iteration {} of {}. Please address the issues above.\n",
            self.current_iteration, self.max_iterations
        ));

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iteration_error_new() {
        let error = IterationError::new(1, ErrorCategory::Compilation, "Failed to compile");
        assert_eq!(error.iteration, 1);
        assert_eq!(error.category, ErrorCategory::Compilation);
        assert_eq!(error.message, "Failed to compile");
        assert!(error.failed_gate.is_none());
        assert!(error.affected_files.is_empty());
    }

    #[test]
    fn test_iteration_error_with_gate() {
        let error = IterationError::new(1, ErrorCategory::Lint, "Clippy failed").with_gate("lint");
        assert_eq!(error.failed_gate, Some("lint".to_string()));
    }

    #[test]
    fn test_iteration_error_with_files() {
        let error = IterationError::new(1, ErrorCategory::Format, "Formatting failed")
            .with_files(vec!["src/main.rs".to_string()]);
        assert_eq!(error.affected_files, vec!["src/main.rs"]);
    }

    #[test]
    fn test_iteration_error_signature() {
        let error = IterationError::new(1, ErrorCategory::Lint, "test").with_gate("lint");
        assert_eq!(error.signature(), "lint:lint");

        let error2 = IterationError::new(1, ErrorCategory::Compilation, "test");
        assert_eq!(error2.signature(), "compilation:none");
    }

    #[test]
    fn test_error_category_as_str() {
        assert_eq!(ErrorCategory::Compilation.as_str(), "compilation");
        assert_eq!(ErrorCategory::Lint.as_str(), "lint");
        assert_eq!(ErrorCategory::Format.as_str(), "format");
        assert_eq!(ErrorCategory::Coverage.as_str(), "coverage");
    }

    #[test]
    fn test_error_category_from_message() {
        assert_eq!(
            ErrorCategory::from_error_message("cargo check failed", None),
            ErrorCategory::Compilation
        );
        assert_eq!(
            ErrorCategory::from_error_message("clippy warnings", None),
            ErrorCategory::Lint
        );
        assert_eq!(
            ErrorCategory::from_error_message("unknown error", Some("coverage")),
            ErrorCategory::Coverage
        );
    }

    #[test]
    fn test_approach_hint_new() {
        let hint = ApproachHint::new("Try adding tests first");
        assert_eq!(hint.description, "Try adding tests first");
        assert_eq!(hint.success_rate, 0.0);
        assert_eq!(hint.sample_count, 0);
    }

    #[test]
    fn test_approach_hint_record_result() {
        let mut hint = ApproachHint::new("Test");
        hint.record_result(true);
        assert_eq!(hint.sample_count, 1);
        assert_eq!(hint.success_rate, 1.0);

        hint.record_result(false);
        assert_eq!(hint.sample_count, 2);
        assert_eq!(hint.success_rate, 0.5);
    }

    #[test]
    fn test_iteration_context_new() {
        let ctx = IterationContext::new("US-001", 10);
        assert_eq!(ctx.story_id, "US-001");
        assert_eq!(ctx.max_iterations, 10);
        assert_eq!(ctx.current_iteration, 0);
        assert!(ctx.error_history.is_empty());
    }

    #[test]
    fn test_iteration_context_start_iteration() {
        let mut ctx = IterationContext::new("US-001", 10);
        ctx.start_iteration(1);
        assert_eq!(ctx.current_iteration, 1);
    }

    #[test]
    fn test_iteration_context_record_error() {
        let mut ctx = IterationContext::new("US-001", 10);
        ctx.record_error(IterationError::new(1, ErrorCategory::Lint, "test"));
        assert_eq!(ctx.error_history.len(), 1);
    }

    #[test]
    fn test_iteration_context_record_partial_progress() {
        let mut ctx = IterationContext::new("US-001", 10);
        ctx.record_partial_progress("lint", vec!["src/main.rs".to_string()]);
        assert_eq!(ctx.partial_progress.get("lint").unwrap().len(), 1);
    }

    #[test]
    fn test_iteration_context_error_count_by_category() {
        let mut ctx = IterationContext::new("US-001", 10);
        ctx.record_error(IterationError::new(1, ErrorCategory::Lint, "test1"));
        ctx.record_error(IterationError::new(2, ErrorCategory::Lint, "test2"));
        ctx.record_error(IterationError::new(3, ErrorCategory::Compilation, "test3"));

        let counts = ctx.error_count_by_category();
        assert_eq!(counts.get(&ErrorCategory::Lint), Some(&2));
        assert_eq!(counts.get(&ErrorCategory::Compilation), Some(&1));
    }

    #[test]
    fn test_iteration_context_repeated_error_count() {
        let mut ctx = IterationContext::new("US-001", 10);
        ctx.record_error(IterationError::new(1, ErrorCategory::Lint, "test").with_gate("lint"));
        ctx.record_error(IterationError::new(2, ErrorCategory::Lint, "test").with_gate("lint"));
        ctx.record_error(IterationError::new(3, ErrorCategory::Format, "test").with_gate("format"));

        assert_eq!(ctx.repeated_error_count("lint:lint"), 2);
        assert_eq!(ctx.repeated_error_count("format:format"), 1);
    }

    #[test]
    fn test_iteration_context_build_prompt_context_empty() {
        let ctx = IterationContext::new("US-001", 10);
        assert_eq!(ctx.build_prompt_context(), "");
    }

    #[test]
    fn test_iteration_context_build_prompt_context_with_errors() {
        let mut ctx = IterationContext::new("US-001", 10);
        ctx.start_iteration(2);
        ctx.record_error(
            IterationError::new(1, ErrorCategory::Lint, "Clippy found warnings")
                .with_gate("lint")
                .with_files(vec!["src/main.rs".to_string()]),
        );

        let prompt = ctx.build_prompt_context();
        assert!(prompt.contains("Previous Iteration Context"));
        assert!(prompt.contains("Iteration 1"));
        assert!(prompt.contains("lint"));
        assert!(prompt.contains("Clippy found warnings"));
        assert!(prompt.contains("src/main.rs"));
        assert!(prompt.contains("iteration 2 of 10"));
    }

    #[test]
    fn test_iteration_context_build_prompt_context_with_hints() {
        let mut ctx = IterationContext::new("US-001", 10);
        ctx.start_iteration(2);
        ctx.record_error(IterationError::new(1, ErrorCategory::Lint, "test"));

        let mut hint = ApproachHint::new("Fix imports first");
        hint.success_rate = 0.8;
        hint.sample_count = 10;
        ctx.add_hint(hint);

        let prompt = ctx.build_prompt_context();
        assert!(prompt.contains("Suggested Approaches"));
        assert!(prompt.contains("Fix imports first"));
        assert!(prompt.contains("80%"));
    }
}
