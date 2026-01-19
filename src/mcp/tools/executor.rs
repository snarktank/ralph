// Story execution engine for Ralph
// This module handles the actual execution of user stories including:
// - Spawning Claude Code or Amp CLI to implement stories
// - Running quality gates after implementation
// - Updating PRD files on success
// - Appending to progress.txt
// - Creating git commits

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{watch, Mutex};

use crate::checkpoint::{Checkpoint, CheckpointManager, PauseReason, StoryCheckpoint};
use crate::error::classification::{ErrorCategory, TimeoutReason};
use crate::iteration::{
    context::{ErrorCategory as IterErrorCategory, IterationContext, IterationError},
    futility::{FutileRetryDetector, FutilityConfig, FutilityVerdict},
};
use crate::metrics::MetricsCollector;
use crate::timeout::{HeartbeatEvent, HeartbeatMonitor, TimeoutConfig};

use crate::mcp::tools::load_prd::{PrdFile, PrdUserStory};
use crate::quality::{GateResult, Profile, QualityGateChecker};

/// Result of story execution
#[derive(Debug)]
pub struct ExecutionResult {
    /// Whether the story was successfully implemented
    pub success: bool,
    /// Git commit hash if a commit was created
    pub commit_hash: Option<String>,
    /// Error message if execution failed
    pub error: Option<String>,
    /// Number of iterations used
    pub iterations_used: u32,
    /// Quality gate results
    pub gate_results: Vec<GateResult>,
    /// Files that were changed
    pub files_changed: Vec<String>,
    /// Futility verdict if execution was stopped early
    pub futility_verdict: Option<FutilityVerdict>,
    /// Iteration context with error history and learnings
    pub iteration_context: Option<IterationContext>,
    /// Whether user guidance is needed to continue
    pub needs_guidance: bool,
}

/// Error types for story execution
#[derive(Debug)]
pub enum ExecutorError {
    /// Story not found in PRD
    StoryNotFound(String),
    /// PRD file operation failed
    PrdError(String),
    /// Git operation failed
    GitError(String),
    /// Quality gates failed
    QualityGateFailed(String),
    /// Agent execution failed
    AgentError(String),
    /// Execution was cancelled
    Cancelled,
    /// IO error
    IoError(String),
    /// Execution timed out
    Timeout(String),
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutorError::StoryNotFound(id) => write!(f, "Story not found: {}", id),
            ExecutorError::PrdError(msg) => write!(f, "PRD error: {}", msg),
            ExecutorError::GitError(msg) => write!(f, "Git error: {}", msg),
            ExecutorError::QualityGateFailed(msg) => write!(f, "Quality gate failed: {}", msg),
            ExecutorError::AgentError(msg) => write!(f, "Agent execution error: {}", msg),
            ExecutorError::Cancelled => write!(f, "Execution was cancelled"),
            ExecutorError::IoError(msg) => write!(f, "IO error: {}", msg),
            ExecutorError::Timeout(msg) => write!(f, "Execution timed out: {}", msg),
        }
    }
}

impl std::error::Error for ExecutorError {}

impl ExecutorError {
    /// Classify this error into an ErrorCategory for recovery decisions.
    pub fn classify(&self) -> ErrorCategory {
        use crate::error::classification::{FatalReason, TransientReason};

        match self {
            ExecutorError::Timeout(_) => ErrorCategory::Timeout(TimeoutReason::ProcessTimeout),
            ExecutorError::Cancelled => ErrorCategory::Fatal(FatalReason::InternalError),
            ExecutorError::StoryNotFound(_) => ErrorCategory::Fatal(FatalReason::ResourceNotFound),
            ExecutorError::PrdError(_) => ErrorCategory::Fatal(FatalReason::ConfigurationError),
            ExecutorError::GitError(_) => ErrorCategory::Transient(TransientReason::ResourceLocked),
            ExecutorError::QualityGateFailed(_) => ErrorCategory::Fatal(FatalReason::InternalError),
            ExecutorError::AgentError(_) => ErrorCategory::Transient(TransientReason::ServerError),
            ExecutorError::IoError(_) => ErrorCategory::Transient(TransientReason::NetworkError),
        }
    }
}

/// Configuration for the story executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Path to the PRD file
    pub prd_path: PathBuf,
    /// Project root directory
    pub project_root: PathBuf,
    /// Path to progress.txt file
    pub progress_path: PathBuf,
    /// Quality profile for gate checking
    pub quality_profile: Option<Profile>,
    /// Agent command to use (e.g., "claude" or "amp")
    pub agent_command: String,
    /// Maximum iterations per story
    pub max_iterations: u32,
    /// Optional mutex for serializing git operations across parallel executions
    pub git_mutex: Option<Arc<Mutex<()>>>,
    /// Timeout configuration for execution limits
    pub timeout_config: TimeoutConfig,
    /// Enable futile retry detection to stop early on hopeless patterns
    pub enable_futility_detection: bool,
    /// Configuration for futility detection thresholds
    pub futility_config: FutilityConfig,
    /// Optional metrics collector for tracking execution statistics
    pub metrics_collector: Option<MetricsCollector>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            prd_path: PathBuf::from("prd.json"),
            project_root: PathBuf::from("."),
            progress_path: PathBuf::from("progress.txt"),
            quality_profile: None,
            agent_command: "claude".to_string(),
            max_iterations: 10,
            git_mutex: None,
            timeout_config: TimeoutConfig::default(),
            enable_futility_detection: true,
            futility_config: FutilityConfig::default(),
            metrics_collector: None,
        }
    }
}

/// Story executor that handles the end-to-end execution of user stories
pub struct StoryExecutor {
    config: ExecutorConfig,
    checkpoint_manager: Option<CheckpointManager>,
}

impl StoryExecutor {
    /// Create a new story executor with the given configuration
    pub fn new(config: ExecutorConfig) -> Self {
        // Attempt to create a checkpoint manager for the project root
        let checkpoint_manager = CheckpointManager::new(&config.project_root).ok();
        Self {
            config,
            checkpoint_manager,
        }
    }

    /// Create a new story executor with an explicit checkpoint manager
    pub fn with_checkpoint_manager(
        config: ExecutorConfig,
        checkpoint_manager: Option<CheckpointManager>,
    ) -> Self {
        Self {
            config,
            checkpoint_manager,
        }
    }

    /// Continue execution of a story with user-provided steering guidance.
    ///
    /// This method resumes execution from a previous iteration context,
    /// incorporating user guidance to help overcome stuck situations.
    ///
    /// # Arguments
    ///
    /// * `story_id` - The ID of the story to continue
    /// * `context` - Previous iteration context with error history
    /// * `guidance` - User-provided steering guidance
    /// * `cancel_receiver` - Watch channel to check for cancellation
    /// * `on_iteration` - Callback called after each iteration with (current, max)
    ///
    /// # Returns
    ///
    /// Result containing the execution result or an error
    pub async fn continue_with_guidance<F>(
        &self,
        story_id: &str,
        mut context: IterationContext,
        guidance: crate::iteration::context::SteeringGuidance,
        cancel_receiver: watch::Receiver<bool>,
        on_iteration: F,
    ) -> Result<ExecutionResult, ExecutorError>
    where
        F: FnMut(u32, u32),
    {
        // Inject the steering guidance into the context
        context.set_steering_guidance(guidance);

        // Continue execution from current iteration
        self.execute_story_with_context(story_id, context, cancel_receiver, on_iteration)
            .await
    }

    /// Execute a single story by ID
    ///
    /// This is the main entry point for story execution. It:
    /// 1. Loads the story from the PRD
    /// 2. Runs the agent to implement the story (with iteration loop)
    /// 3. Runs quality gates
    /// 4. Updates the PRD on success
    /// 5. Appends to progress.txt
    /// 6. Creates a git commit
    ///
    /// # Arguments
    ///
    /// * `story_id` - The ID of the story to execute
    /// * `cancel_receiver` - Watch channel to check for cancellation
    /// * `on_iteration` - Callback called after each iteration with (current, max)
    ///
    /// # Returns
    ///
    /// Result containing the execution result or an error
    pub async fn execute_story<F>(
        &self,
        story_id: &str,
        cancel_receiver: watch::Receiver<bool>,
        on_iteration: F,
    ) -> Result<ExecutionResult, ExecutorError>
    where
        F: FnMut(u32, u32),
    {
        // Create new iteration context
        let iter_context = IterationContext::new(story_id, self.config.max_iterations);

        self.execute_story_with_context(story_id, iter_context, cancel_receiver, on_iteration)
            .await
    }

    /// Execute a story with an existing iteration context.
    ///
    /// This is the internal method that handles both fresh starts and resumptions.
    async fn execute_story_with_context<F>(
        &self,
        story_id: &str,
        mut iter_context: IterationContext,
        cancel_receiver: watch::Receiver<bool>,
        mut on_iteration: F,
    ) -> Result<ExecutionResult, ExecutorError>
    where
        F: FnMut(u32, u32),
    {
        // Load the PRD and find the story
        let prd = self.load_prd()?;
        let story = self.find_story(&prd, story_id)?;

        // Update iteration context (may already be initialized if resuming)
        if iter_context.max_iterations == 0 {
            iter_context.max_iterations = self.config.max_iterations;
        }

        // Initialize futility detector if enabled
        let futility_detector = if self.config.enable_futility_detection {
            Some(FutileRetryDetector::with_config(
                self.config.futility_config.clone(),
            ))
        } else {
            None
        };

        // Record metrics start if collector is available
        if let Some(ref collector) = self.config.metrics_collector {
            collector.start_story(story_id, self.config.max_iterations);
        }

        let execution_start = std::time::Instant::now();
        let mut iterations_used = 0;
        let mut last_error: Option<String> = None;
        let mut files_changed: Vec<String> = Vec::new();
        let mut last_gate_results: Vec<GateResult> = Vec::new();

        // Iteration loop
        for iteration in 1..=self.config.max_iterations {
            iterations_used = iteration;
            iter_context.start_iteration(iteration);
            on_iteration(iteration, self.config.max_iterations);

            // Record iteration in metrics
            if let Some(ref collector) = self.config.metrics_collector {
                collector.record_iteration(iteration);
            }

            // Check for cancellation
            if *cancel_receiver.borrow() {
                return Err(ExecutorError::Cancelled);
            }

            // Build the prompt with iteration context if we have previous errors
            let prompt = if iter_context.error_history.is_empty() {
                self.build_agent_prompt(story, &prd)
            } else {
                self.build_agent_prompt_with_context(story, &prd, &iter_context)
            };

            // Run the agent
            match self.run_agent(&prompt, iteration).await {
                Ok(changed) => {
                    files_changed = changed;
                }
                Err(ExecutorError::Timeout(msg)) => {
                    // Record timeout error in context
                    iter_context.record_error(IterationError::new(
                        iteration,
                        IterErrorCategory::AgentExecution,
                        &msg,
                    ));

                    // Record in metrics
                    if let Some(ref collector) = self.config.metrics_collector {
                        collector.record_error(IterErrorCategory::AgentExecution);
                    }

                    // On timeout, save checkpoint before returning error
                    self.save_timeout_checkpoint(story_id, iteration);
                    return Err(ExecutorError::Timeout(msg));
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let category = IterErrorCategory::from_error_message(&error_msg, None);

                    // Record error in iteration context
                    iter_context.record_error(IterationError::new(iteration, category, &error_msg));

                    // Record in metrics
                    if let Some(ref collector) = self.config.metrics_collector {
                        collector.record_error(category);
                    }

                    last_error = Some(error_msg);

                    // Check for futility before continuing
                    if let Some(ref detector) = futility_detector {
                        let verdict = detector.analyze(&iter_context);
                        if !verdict.should_continue() {
                            // Check if this is a pause for guidance scenario
                            let needs_guidance =
                                matches!(verdict, FutilityVerdict::PauseForGuidance { .. });

                            // Record metrics completion (only if not pausing for guidance)
                            if !needs_guidance {
                                if let Some(ref collector) = self.config.metrics_collector {
                                    collector.complete_story(
                                        false,
                                        execution_start.elapsed(),
                                        Some(format!("Futile: {:?}", verdict.reason())),
                                    );
                                }
                            }

                            return Ok(ExecutionResult {
                                success: false,
                                commit_hash: None,
                                error: verdict.reason().map(String::from),
                                iterations_used,
                                gate_results: last_gate_results,
                                files_changed,
                                futility_verdict: Some(verdict.clone()),
                                iteration_context: Some(iter_context),
                                needs_guidance,
                            });
                        }
                    }

                    continue; // Try next iteration
                }
            }

            // Check for cancellation before quality gates
            if cancel_receiver.has_changed().unwrap_or(false) && *cancel_receiver.borrow() {
                return Err(ExecutorError::Cancelled);
            }

            // Run quality gates with timing
            let gate_start = std::time::Instant::now();
            let gate_results = self.run_quality_gates();
            let gate_duration = gate_start.elapsed();

            // Record gate durations in metrics
            if let Some(ref collector) = self.config.metrics_collector {
                for gate in &gate_results {
                    collector.record_gate_duration(
                        &gate.gate_name,
                        gate_duration / gate_results.len() as u32,
                    );
                }
            }

            last_gate_results = gate_results.clone();
            let all_passed = QualityGateChecker::all_passed(&gate_results);

            if all_passed {
                // Success! Create commit and update PRD
                let commit_hash = self.create_commit(story).await?;
                self.update_prd_passes(story_id)?;
                self.append_progress(story, &files_changed, iteration)?;

                // Record successful completion in metrics
                if let Some(ref collector) = self.config.metrics_collector {
                    collector.complete_story(true, execution_start.elapsed(), None);
                }

                return Ok(ExecutionResult {
                    success: true,
                    commit_hash: Some(commit_hash),
                    error: None,
                    iterations_used,
                    gate_results,
                    files_changed,
                    futility_verdict: None,
                    iteration_context: Some(iter_context),
                    needs_guidance: false,
                });
            }

            // Quality gates failed, record in iteration context
            let failed_gates: Vec<&str> = gate_results
                .iter()
                .filter(|g| !g.passed)
                .map(|g| g.gate_name.as_str())
                .collect();

            // Record each failed gate as an error
            for gate_name in &failed_gates {
                let category = IterErrorCategory::from_error_message("", Some(gate_name));
                iter_context.record_error(
                    IterationError::new(
                        iteration,
                        category,
                        format!("Gate '{}' failed", gate_name),
                    )
                    .with_gate(*gate_name)
                    .with_files(files_changed.clone()),
                );

                // Record in metrics
                if let Some(ref collector) = self.config.metrics_collector {
                    collector.record_error(category);
                }
            }

            last_error = Some(format!("Quality gates failed: {}", failed_gates.join(", ")));

            // Check for futility after gate failures
            if let Some(ref detector) = futility_detector {
                let verdict = detector.analyze(&iter_context);
                if !verdict.should_continue() {
                    // Check if this is a pause for guidance scenario
                    let needs_guidance =
                        matches!(verdict, FutilityVerdict::PauseForGuidance { .. });

                    // Record metrics completion (only if not pausing for guidance)
                    if !needs_guidance {
                        if let Some(ref collector) = self.config.metrics_collector {
                            collector.complete_story(
                                false,
                                execution_start.elapsed(),
                                Some(format!("Futile: {:?}", verdict.reason())),
                            );
                        }
                    }

                    return Ok(ExecutionResult {
                        success: false,
                        commit_hash: None,
                        error: verdict.reason().map(String::from),
                        iterations_used,
                        gate_results,
                        files_changed,
                        futility_verdict: Some(verdict.clone()),
                        iteration_context: Some(iter_context),
                        needs_guidance,
                    });
                }
            }
        }

        // Max iterations reached without success
        // Record metrics completion
        if let Some(ref collector) = self.config.metrics_collector {
            collector.complete_story(false, execution_start.elapsed(), last_error.clone());
        }

        // Build detailed failure summary
        let failure_summary = self.build_failure_summary(
            story_id,
            iterations_used,
            &last_error,
            &iter_context,
            &last_gate_results,
        );

        Err(ExecutorError::AgentError(failure_summary))
    }

    /// Build a detailed failure summary for a story that failed all iterations.
    ///
    /// This generates a comprehensive report showing:
    /// - Overall failure reason
    /// - Iteration-by-iteration breakdown
    /// - Error patterns detected
    /// - Suggested next steps
    fn build_failure_summary(
        &self,
        story_id: &str,
        iterations_used: u32,
        last_error: &Option<String>,
        context: &IterationContext,
        gate_results: &[GateResult],
    ) -> String {
        use crate::iteration::futility::FutileRetryDetector;

        let mut summary = String::new();

        // Header
        summary.push_str(&format!(
            "Story {} FAILED after {} iterations\n\n",
            story_id, iterations_used
        ));

        // Last error (if available)
        if let Some(ref err) = last_error {
            // Strip "Agent execution error:" prefix if present to avoid nesting
            let clean_error = err.strip_prefix("Agent execution error: ").unwrap_or(err);
            summary.push_str(&format!("Last Error:\n{}\n\n", clean_error));
        }

        // Quality gate status
        if !gate_results.is_empty() {
            summary.push_str("Quality Gate Results (Last Iteration):\n");
            for gate in gate_results {
                let status = if gate.passed { "PASS" } else { "FAIL" };
                summary.push_str(&format!("  - {}: {}\n", gate.gate_name, status));
                if !gate.passed {
                    if let Some(ref details) = gate.details {
                        // Show first 3 lines of details
                        let detail_lines: Vec<&str> = details.lines().take(3).collect();
                        for line in detail_lines {
                            summary.push_str(&format!("    {}\n", line));
                        }
                    }
                }
            }
            summary.push('\n');
        }

        // Error history breakdown
        if !context.error_history.is_empty() {
            summary.push_str("Error History:\n");
            let error_counts = context.error_count_by_category();
            for (category, count) in error_counts.iter() {
                summary.push_str(&format!(
                    "  - {}: {} occurrence(s)\n",
                    category.as_str(),
                    count
                ));
            }
            summary.push('\n');

            // Show last 5 iteration errors
            summary.push_str("Recent Iteration Errors:\n");
            for error in context.error_history.iter().rev().take(5).rev() {
                summary.push_str(&format!(
                    "  Iteration {}: [{}] {}\n",
                    error.iteration,
                    error.category.as_str(),
                    error.message
                ));
                if let Some(ref gate) = error.failed_gate {
                    summary.push_str(&format!("    Failed gate: {}\n", gate));
                }
            }
            summary.push('\n');
        }

        // Pattern analysis
        let detector = FutileRetryDetector::with_config(self.config.futility_config.clone());
        let pattern_summary = detector.summarize_patterns(context);

        summary.push_str("Pattern Analysis:\n");
        summary.push_str(&format!(
            "  - Error rate: {:.0}% ({} errors in {} iterations)\n",
            pattern_summary.error_rate(),
            pattern_summary.total_errors,
            pattern_summary.total_iterations
        ));

        if let Some((sig, count)) = pattern_summary.most_frequent_error {
            summary.push_str(&format!("  - Most frequent: '{}' ({} times)\n", sig, count));
        }

        if pattern_summary.has_oscillation {
            summary.push_str("  - ⚠ Oscillating errors detected (fixing one breaks another)\n");
        }

        if pattern_summary.has_stagnation {
            summary.push_str("  - ⚠ Stagnation detected (same error repeating)\n");
        }

        summary.push('\n');

        // Suggested next steps
        summary.push_str("Suggested Next Steps:\n");
        if pattern_summary.has_oscillation {
            summary.push_str("  1. Review conflicting requirements that cause oscillation\n");
            summary.push_str("  2. Address both issues simultaneously rather than sequentially\n");
        } else if pattern_summary.has_stagnation {
            summary
                .push_str("  1. Review the recurring error and provide more specific guidance\n");
            summary.push_str("  2. Consider breaking down the story into smaller subtasks\n");
            summary.push_str("  3. Check for missing dependencies or prerequisites\n");
        } else {
            summary.push_str("  1. Review the error history to identify root cause\n");
            summary.push_str("  2. Check if the story requirements are clear and achievable\n");
            summary.push_str("  3. Consider if quality gates are too strict or misconfigured\n");
        }

        summary
    }

    /// Build an agent prompt that includes iteration context from previous failures.
    fn build_agent_prompt_with_context(
        &self,
        story: &PrdUserStory,
        prd: &PrdFile,
        context: &IterationContext,
    ) -> String {
        let base_prompt = self.build_agent_prompt(story, prd);
        let context_section = context.build_prompt_context();
        format!("{}{}", base_prompt, context_section)
    }

    /// Load the PRD file
    fn load_prd(&self) -> Result<PrdFile, ExecutorError> {
        let content = std::fs::read_to_string(&self.config.prd_path)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to read PRD: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to parse PRD: {}", e)))
    }

    /// Find a story by ID in the PRD
    fn find_story<'a>(
        &self,
        prd: &'a PrdFile,
        story_id: &str,
    ) -> Result<&'a PrdUserStory, ExecutorError> {
        prd.user_stories
            .iter()
            .find(|s| s.id == story_id)
            .ok_or_else(|| ExecutorError::StoryNotFound(story_id.to_string()))
    }

    /// Build the agent prompt for implementing a story
    fn build_agent_prompt(&self, story: &PrdUserStory, prd: &PrdFile) -> String {
        let mut prompt = format!("# Implement User Story: {} - {}\n\n", story.id, story.title);

        if !story.description.is_empty() {
            prompt.push_str(&format!("## Description\n{}\n\n", story.description));
        }

        if !story.acceptance_criteria.is_empty() {
            prompt.push_str("## Acceptance Criteria\n");
            for (i, criterion) in story.acceptance_criteria.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, criterion));
            }
            prompt.push('\n');
        }

        prompt.push_str(&format!(
            "## Project Context\n\
            - Project: {}\n\
            - Branch: {}\n\
            - Story Priority: {}\n\n\
            ## Instructions\n\
            1. Implement all acceptance criteria\n\
            2. Ensure code compiles without errors (cargo check)\n\
            3. Ensure no clippy warnings (cargo clippy -- -D warnings)\n\
            4. Ensure proper formatting (cargo fmt)\n\
            5. Keep changes focused and minimal\n",
            prd.project, prd.branch_name, story.priority
        ));

        prompt
    }

    /// Run the agent (Claude Code or Amp CLI) to implement the story
    ///
    /// This method integrates heartbeat monitoring to detect stalled agents.
    /// The heartbeat is updated whenever the agent produces output, and stall
    /// detection triggers a graceful timeout.
    async fn run_agent(&self, prompt: &str, iteration: u32) -> Result<Vec<String>, ExecutorError> {
        let agent_cmd = &self.config.agent_command;

        // Detect which agent to use
        let (program, args) = if agent_cmd == "claude" || agent_cmd.contains("claude") {
            // Claude Code CLI - use --print for non-interactive mode
            // and --dangerously-skip-permissions to allow file changes
            (
                "claude",
                vec!["--print", "--dangerously-skip-permissions", prompt],
            )
        } else if agent_cmd == "amp" || agent_cmd.contains("amp") {
            // Amp CLI
            ("amp", vec!["--prompt", prompt])
        } else {
            // Custom agent command
            (agent_cmd.as_str(), vec![prompt])
        };

        // Check if the agent is available (cross-platform)
        if !is_program_in_path(program) {
            return Err(ExecutorError::AgentError(format!(
                "Agent '{}' not found in PATH. Install Claude Code CLI or Amp CLI.",
                program
            )));
        }

        // Create heartbeat monitor for stall detection
        let (heartbeat_monitor, mut heartbeat_receiver) =
            HeartbeatMonitor::new(self.config.timeout_config.clone());

        // Start heartbeat monitoring before agent execution
        heartbeat_monitor.start_monitoring().await;

        // Spawn the agent process with piped stdout/stderr for streaming
        let mut child = tokio::process::Command::new(program)
            .args(&args)
            .current_dir(&self.config.project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                ExecutorError::AgentError(format!("Failed to spawn {}: {}", program, e))
            })?;

        // Take ownership of stdout and stderr
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Create readers for stdout and stderr
        let mut stdout_reader = stdout.map(|s| BufReader::new(s).lines());
        let mut stderr_reader = stderr.map(|s| BufReader::new(s).lines());

        // Collect both stdout and stderr for error reporting
        let mut stderr_output = String::new();
        let mut stdout_output = String::new();

        // Track if we received a stall detection
        let mut stall_detected = false;

        // Overall timeout for the agent execution
        let timeout_duration = self.config.timeout_config.agent_timeout;
        let timeout_deadline = tokio::time::Instant::now() + timeout_duration;

        // Main loop: process output, heartbeat events, and wait for completion
        loop {
            tokio::select! {
                // Check for heartbeat events
                event = heartbeat_receiver.recv() => {
                    match event {
                        Some(HeartbeatEvent::Warning(missed)) => {
                            // Log warning about missed heartbeats
                            eprintln!(
                                "Warning: Agent stall detected - {} missed heartbeats (iteration {})",
                                missed, iteration
                            );
                        }
                        Some(HeartbeatEvent::StallDetected(missed)) => {
                            // Stall detected - trigger graceful timeout
                            eprintln!(
                                "Agent stall detected: {} missed heartbeats, triggering timeout (iteration {})",
                                missed, iteration
                            );
                            stall_detected = true;
                            // Kill the child process gracefully
                            let _ = child.kill().await;
                            break;
                        }
                        None => {
                            // Channel closed, continue processing
                        }
                    }
                }

                // Read stdout line
                line = async {
                    if let Some(ref mut reader) = stdout_reader {
                        reader.next_line().await
                    } else {
                        Ok(None)
                    }
                } => {
                    match line {
                        Ok(Some(text)) => {
                            // Activity detected - update heartbeat
                            heartbeat_monitor.pulse().await;
                            // Collect stdout for error diagnostics
                            stdout_output.push_str(&text);
                            stdout_output.push('\n');
                        }
                        Ok(None) => {
                            // EOF on stdout
                            stdout_reader = None;
                        }
                        Err(_) => {
                            stdout_reader = None;
                        }
                    }
                }

                // Read stderr line
                line = async {
                    if let Some(ref mut reader) = stderr_reader {
                        reader.next_line().await
                    } else {
                        Ok(None)
                    }
                } => {
                    match line {
                        Ok(Some(text)) => {
                            // Activity detected - update heartbeat
                            heartbeat_monitor.pulse().await;
                            // Collect stderr for error reporting
                            stderr_output.push_str(&text);
                            stderr_output.push('\n');
                        }
                        Ok(None) => {
                            // EOF on stderr
                            stderr_reader = None;
                        }
                        Err(_) => {
                            stderr_reader = None;
                        }
                    }
                }

                // Check for process completion
                status = child.wait() => {
                    match status {
                        Ok(exit_status) => {
                            // Stop heartbeat monitoring
                            heartbeat_monitor.stop().await;

                            if !exit_status.success() {
                                // Build comprehensive error message from both streams
                                let error_details = self.build_agent_error_message(
                                    &stdout_output,
                                    &stderr_output,
                                    exit_status.code()
                                );
                                return Err(ExecutorError::AgentError(error_details));
                            }
                            // Process completed successfully
                            let files_changed = self.get_changed_files()?;
                            return Ok(files_changed);
                        }
                        Err(e) => {
                            heartbeat_monitor.stop().await;
                            return Err(ExecutorError::AgentError(format!(
                                "Failed to wait for {}: {}", program, e
                            )));
                        }
                    }
                }

                // Overall timeout
                _ = tokio::time::sleep_until(timeout_deadline) => {
                    heartbeat_monitor.stop().await;
                    let _ = child.kill().await;
                    return Err(ExecutorError::Timeout(format!(
                        "Agent '{}' timed out after {:?} (iteration {})",
                        program, timeout_duration, iteration
                    )));
                }
            }

            // Check if both readers are done and process hasn't exited yet
            if stdout_reader.is_none() && stderr_reader.is_none() {
                // Wait for process to exit
                match child.wait().await {
                    Ok(exit_status) => {
                        heartbeat_monitor.stop().await;

                        if stall_detected {
                            return Err(ExecutorError::Timeout(format!(
                                "Agent '{}' stalled (no output for {:?}) (iteration {})",
                                program,
                                self.config.timeout_config.heartbeat_interval
                                    * self.config.timeout_config.missed_heartbeats_threshold,
                                iteration
                            )));
                        }

                        if !exit_status.success() {
                            let error_details = self.build_agent_error_message(
                                &stdout_output,
                                &stderr_output,
                                exit_status.code(),
                            );
                            return Err(ExecutorError::AgentError(error_details));
                        }

                        let files_changed = self.get_changed_files()?;
                        return Ok(files_changed);
                    }
                    Err(e) => {
                        heartbeat_monitor.stop().await;
                        return Err(ExecutorError::AgentError(format!(
                            "Failed to wait for {}: {}",
                            program, e
                        )));
                    }
                }
            }
        }

        // Stop heartbeat monitoring after execution completes
        heartbeat_monitor.stop().await;

        if stall_detected {
            return Err(ExecutorError::Timeout(format!(
                "Agent '{}' stalled (no output for {:?}) (iteration {})",
                program,
                self.config.timeout_config.heartbeat_interval
                    * self.config.timeout_config.missed_heartbeats_threshold,
                iteration
            )));
        }

        // Get list of changed files from git
        let files_changed = self.get_changed_files()?;
        Ok(files_changed)
    }

    /// Build a comprehensive error message from agent output.
    ///
    /// Extracts the most relevant error information from stdout and stderr,
    /// avoiding truncation and providing context.
    fn build_agent_error_message(
        &self,
        stdout: &str,
        stderr: &str,
        exit_code: Option<i32>,
    ) -> String {
        let mut error_parts = Vec::new();

        // Add exit code
        if let Some(code) = exit_code {
            error_parts.push(format!("Exit code: {}", code));
        }

        // Extract last few lines of stderr (most likely to contain error)
        let stderr_lines: Vec<&str> = stderr.lines().collect();
        if !stderr_lines.is_empty() {
            let relevant_stderr = stderr_lines
                .iter()
                .rev()
                .take(10)
                .rev()
                .copied()
                .collect::<Vec<_>>()
                .join("\n");
            if !relevant_stderr.trim().is_empty() {
                error_parts.push(format!("stderr:\n{}", relevant_stderr.trim()));
            }
        }

        // If stderr is empty, check stdout for error indicators
        if stderr.trim().is_empty() {
            let stdout_lines: Vec<&str> = stdout.lines().collect();
            let error_indicators = [
                "error:", "Error:", "ERROR", "failed", "Failed", "FAILED", "panic",
            ];

            let relevant_stdout: Vec<&str> = stdout_lines
                .iter()
                .rev()
                .take(20)
                .rev()
                .filter(|line| {
                    error_indicators
                        .iter()
                        .any(|indicator| line.contains(indicator))
                })
                .copied()
                .collect();

            if !relevant_stdout.is_empty() {
                error_parts.push(format!("stdout (errors):\n{}", relevant_stdout.join("\n")));
            } else if !stdout_lines.is_empty() {
                // No error indicators, just show last few lines
                let last_lines = stdout_lines
                    .iter()
                    .rev()
                    .take(5)
                    .rev()
                    .copied()
                    .collect::<Vec<_>>()
                    .join("\n");
                if !last_lines.trim().is_empty() {
                    error_parts.push(format!("stdout (last lines):\n{}", last_lines.trim()));
                }
            }
        }

        if error_parts.is_empty() {
            "Agent failed with no output".to_string()
        } else {
            error_parts.join("\n\n")
        }
    }

    /// Get the list of files changed according to git
    fn get_changed_files(&self) -> Result<Vec<String>, ExecutorError> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.config.project_root)
            .output()
            .map_err(|e| ExecutorError::GitError(format!("Failed to run git status: {}", e)))?;

        if !output.status.success() {
            return Err(ExecutorError::GitError("git status failed".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                // Git status format: "XY filename" where X and Y are status codes
                let line = line.trim();
                if line.len() > 3 {
                    Some(line[3..].to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(files)
    }

    /// Save a checkpoint when execution times out.
    ///
    /// This captures the current execution state so the story can be resumed later.
    /// Errors during checkpoint saving are logged but not propagated.
    fn save_timeout_checkpoint(&self, story_id: &str, iteration: u32) {
        if let Some(ref manager) = self.checkpoint_manager {
            // Get uncommitted files for checkpoint
            let uncommitted_files = self.get_changed_files().unwrap_or_default();

            let checkpoint = Checkpoint::new(
                Some(StoryCheckpoint::new(
                    story_id,
                    iteration,
                    self.config.max_iterations,
                )),
                PauseReason::Timeout,
                uncommitted_files,
            );

            // Save checkpoint with error logging (best effort, but warn on failure)
            if let Err(e) = manager.save(&checkpoint) {
                eprintln!(
                    "Warning: Failed to save timeout checkpoint for story '{}': {}",
                    story_id, e
                );
            }
        }
    }

    /// Run quality gates and return results
    fn run_quality_gates(&self) -> Vec<GateResult> {
        let profile = self.config.quality_profile.clone().unwrap_or_default();
        let checker = QualityGateChecker::new(profile, &self.config.project_root);
        checker.run_all()
    }

    /// Create a git commit with the proper format
    ///
    /// If a git_mutex is configured, this method will acquire the lock before
    /// performing git operations to prevent concurrent git operations that could
    /// corrupt the repository.
    async fn create_commit(&self, story: &PrdUserStory) -> Result<String, ExecutorError> {
        // Acquire git mutex if configured (for parallel execution)
        let _guard = if let Some(ref mutex) = self.config.git_mutex {
            Some(mutex.lock().await)
        } else {
            None
        };

        // Stage all changes
        let status = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.config.project_root)
            .status()
            .map_err(|e| ExecutorError::GitError(format!("Failed to stage changes: {}", e)))?;

        if !status.success() {
            return Err(ExecutorError::GitError("git add failed".to_string()));
        }

        // Create commit with proper message format: feat: [ID] - [Title]
        let commit_message = format!("feat: {} - {}", story.id, story.title);

        let status = Command::new("git")
            .args(["commit", "-m", &commit_message])
            .current_dir(&self.config.project_root)
            .status()
            .map_err(|e| ExecutorError::GitError(format!("Failed to create commit: {}", e)))?;

        if !status.success() {
            return Err(ExecutorError::GitError("git commit failed".to_string()));
        }

        // Get the commit hash
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.config.project_root)
            .output()
            .map_err(|e| ExecutorError::GitError(format!("Failed to get commit hash: {}", e)))?;

        if !output.status.success() {
            return Err(ExecutorError::GitError(
                "Failed to get commit hash".to_string(),
            ));
        }

        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(hash)
        // _guard is dropped here, releasing the mutex lock
    }

    /// Update the PRD file to set passes: true for the story
    fn update_prd_passes(&self, story_id: &str) -> Result<(), ExecutorError> {
        // Read the PRD as raw JSON to preserve structure
        let content = std::fs::read_to_string(&self.config.prd_path)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to read PRD: {}", e)))?;

        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to parse PRD: {}", e)))?;

        // Find and update the story
        if let Some(stories) = json.get_mut("userStories").and_then(|s| s.as_array_mut()) {
            for story in stories {
                if story.get("id").and_then(|id| id.as_str()) == Some(story_id) {
                    story["passes"] = serde_json::Value::Bool(true);
                    break;
                }
            }
        }

        // Write back with pretty formatting
        let updated_content = serde_json::to_string_pretty(&json)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to serialize PRD: {}", e)))?;

        std::fs::write(&self.config.prd_path, updated_content)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to write PRD: {}", e)))?;

        Ok(())
    }

    /// Append progress entry to progress.txt
    fn append_progress(
        &self,
        story: &PrdUserStory,
        files_changed: &[String],
        iterations: u32,
    ) -> Result<(), ExecutorError> {
        use std::io::Write;

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M");

        let mut entry = format!(
            "\n## {} - {}\n\
            - **What was implemented**: {}\n\
            - **Files changed**:\n",
            timestamp, story.id, story.title
        );

        for file in files_changed.iter().take(20) {
            entry.push_str(&format!("  - {}\n", file));
        }
        if files_changed.len() > 20 {
            entry.push_str(&format!(
                "  - ... and {} more files\n",
                files_changed.len() - 20
            ));
        }

        entry.push_str(&format!(
            "- **Iterations used**: {}\n\
            - **Learnings for future iterations:**\n\
              - Story completed successfully via automated execution\n\
            ---\n",
            iterations
        ));

        // Append to progress file
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.progress_path)
            .map_err(|e| ExecutorError::IoError(format!("Failed to open progress file: {}", e)))?;

        file.write_all(entry.as_bytes())
            .map_err(|e| ExecutorError::IoError(format!("Failed to write progress: {}", e)))?;

        Ok(())
    }
}

/// Check if a program exists in PATH (cross-platform)
fn is_program_in_path(program: &str) -> bool {
    #[cfg(target_os = "windows")]
    let check_cmd = "where";
    #[cfg(not(target_os = "windows"))]
    let check_cmd = "which";

    Command::new(check_cmd)
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a specific agent CLI is available
pub fn is_agent_available(agent: &str) -> bool {
    is_program_in_path(agent)
}

/// Detect the best available agent CLI
pub fn detect_agent() -> Option<String> {
    // Prefer Claude Code, fall back to Amp
    if is_agent_available("claude") {
        Some("claude".to_string())
    } else if is_agent_available("amp") {
        Some("amp".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_prd() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "description": "Test PRD",
            "userStories": [
                {
                    "id": "US-001",
                    "title": "First story",
                    "description": "A test story",
                    "acceptanceCriteria": ["AC1", "AC2"],
                    "priority": 1,
                    "passes": false
                },
                {
                    "id": "US-002",
                    "title": "Second story",
                    "priority": 2,
                    "passes": true
                }
            ]
        }"#;
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.agent_command, "claude");
        assert_eq!(config.max_iterations, 10);
    }

    #[test]
    fn test_load_prd() {
        let prd_file = create_test_prd();
        let config = ExecutorConfig {
            prd_path: prd_file.path().to_path_buf(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        let prd = executor.load_prd().unwrap();
        assert_eq!(prd.project, "TestProject");
        assert_eq!(prd.user_stories.len(), 2);
    }

    #[test]
    fn test_find_story_success() {
        let prd_file = create_test_prd();
        let config = ExecutorConfig {
            prd_path: prd_file.path().to_path_buf(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        let prd = executor.load_prd().unwrap();
        let story = executor.find_story(&prd, "US-001").unwrap();
        assert_eq!(story.id, "US-001");
        assert_eq!(story.title, "First story");
    }

    #[test]
    fn test_find_story_not_found() {
        let prd_file = create_test_prd();
        let config = ExecutorConfig {
            prd_path: prd_file.path().to_path_buf(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        let prd = executor.load_prd().unwrap();
        let result = executor.find_story(&prd, "US-999");
        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutorError::StoryNotFound(id) => assert_eq!(id, "US-999"),
            _ => panic!("Expected StoryNotFound error"),
        }
    }

    #[test]
    fn test_build_agent_prompt() {
        let prd_file = create_test_prd();
        let config = ExecutorConfig {
            prd_path: prd_file.path().to_path_buf(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        let prd = executor.load_prd().unwrap();
        let story = executor.find_story(&prd, "US-001").unwrap();
        let prompt = executor.build_agent_prompt(story, &prd);

        assert!(prompt.contains("US-001"));
        assert!(prompt.contains("First story"));
        assert!(prompt.contains("AC1"));
        assert!(prompt.contains("AC2"));
        assert!(prompt.contains("TestProject"));
        assert!(prompt.contains("cargo check"));
    }

    #[test]
    fn test_executor_error_display() {
        assert!(ExecutorError::StoryNotFound("US-001".to_string())
            .to_string()
            .contains("US-001"));
        assert!(ExecutorError::Cancelled.to_string().contains("cancelled"));
        assert!(ExecutorError::GitError("test".to_string())
            .to_string()
            .contains("Git error"));
    }

    #[test]
    fn test_update_prd_passes() {
        let prd_file = create_test_prd();
        let prd_path = prd_file.path().to_path_buf();

        // Copy to a temp file we can modify
        let temp_dir = TempDir::new().unwrap();
        let test_prd_path = temp_dir.path().join("prd.json");
        std::fs::copy(&prd_path, &test_prd_path).unwrap();

        let config = ExecutorConfig {
            prd_path: test_prd_path.clone(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        // Update US-001 to passes: true
        executor.update_prd_passes("US-001").unwrap();

        // Verify the change
        let content = std::fs::read_to_string(&test_prd_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        let stories = json.get("userStories").unwrap().as_array().unwrap();
        let us001 = stories
            .iter()
            .find(|s| s.get("id").unwrap() == "US-001")
            .unwrap();
        assert_eq!(us001.get("passes").unwrap(), &serde_json::Value::Bool(true));
    }

    #[test]
    fn test_detect_agent() {
        // This test may pass or fail depending on installed tools
        let agent = detect_agent();
        // Just verify it returns a valid option or None
        if let Some(a) = agent {
            assert!(a == "claude" || a == "amp");
        }
    }
}
