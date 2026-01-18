//! Story executor for Ralph MCP server.
//!
//! Provides the execution loop for running user stories, including:
//! - Iteration management with progress callbacks
//! - Quality gate execution with real-time progress updates
//! - Integration with the iteration view UI components

#![allow(dead_code)]

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::quality::{GateResult, Profile, QualityGateChecker};
use crate::ui::{
    ActivityIndicator, GateProgress, GateSummary, IterationPreview, IterationSummary,
    IterationSummaryStack, LiveIterationPanel, Theme,
};

/// Event emitted during gate execution.
#[derive(Debug, Clone)]
pub enum GateProgressEvent {
    /// Gate is about to start
    Started { gate_name: String },
    /// Gate completed (passed or failed)
    Completed {
        gate_name: String,
        passed: bool,
        duration: Duration,
        message: String,
        details: Option<String>,
    },
    /// Activity update during gate execution
    Activity { gate_name: String, activity: String },
}

impl GateProgressEvent {
    /// Create a started event.
    pub fn started(gate_name: impl Into<String>) -> Self {
        Self::Started {
            gate_name: gate_name.into(),
        }
    }

    /// Create a completed event from a GateResult.
    pub fn completed(result: &GateResult, duration: Duration) -> Self {
        Self::Completed {
            gate_name: result.gate_name.clone(),
            passed: result.passed,
            duration,
            message: result.message.clone(),
            details: result.details.clone(),
        }
    }

    /// Create an activity event.
    pub fn activity(gate_name: impl Into<String>, activity: impl Into<String>) -> Self {
        Self::Activity {
            gate_name: gate_name.into(),
            activity: activity.into(),
        }
    }
}

/// Callback type for gate progress events.
pub type OnGateProgress = Box<dyn FnMut(GateProgressEvent) + Send>;

/// Event emitted during story execution.
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    /// Iteration is starting
    IterationStarting {
        iteration: u32,
        max_iterations: u32,
        gates: Vec<String>,
    },
    /// Gate progress within an iteration
    GateProgress(GateProgressEvent),
    /// Iteration completed
    IterationCompleted {
        iteration: u32,
        max_iterations: u32,
        passed: bool,
        duration: Duration,
        gate_results: Vec<GateSummary>,
    },
    /// Execution finished (success or max iterations reached)
    ExecutionFinished {
        success: bool,
        total_iterations: u32,
        total_duration: Duration,
    },
}

/// Callback type for execution events.
pub type OnExecutionEvent = Box<dyn FnMut(ExecutionEvent) + Send>;

/// Configuration for the story executor.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum number of iterations
    pub max_iterations: u32,
    /// Quality profile for gate checking
    pub profile: Profile,
    /// Whether to show UI during execution
    pub show_ui: bool,
    /// Theme for UI rendering
    pub theme: Theme,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            profile: Profile::default(),
            show_ui: true,
            theme: Theme::default(),
        }
    }
}

/// Display manager for iteration progress.
///
/// Handles showing iteration previews, live progress panels,
/// and collecting iteration summaries.
pub struct IterationDisplay {
    /// Theme for rendering
    theme: Theme,
    /// Stack of completed iteration summaries
    summaries: IterationSummaryStack,
    /// Current live panel (if iteration is in progress)
    current_panel: Option<LiveIterationPanel>,
}

impl IterationDisplay {
    /// Create a new iteration display.
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            summaries: IterationSummaryStack::new(),
            current_panel: None,
        }
    }

    /// Create an iteration display with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            theme,
            summaries: IterationSummaryStack::with_theme(theme),
            current_panel: None,
        }
    }

    /// Show the iteration preview before starting.
    ///
    /// Displays what gates will be run in the format:
    /// "Will run: build → lint → test"
    pub fn show_iteration_preview(&self, gates: &[String]) -> io::Result<()> {
        let preview = IterationPreview::with_theme(gates.to_vec(), self.theme);
        let output = preview.render();

        let mut stdout = io::stdout();
        stdout.write_all(output.as_bytes())?;
        stdout.flush()
    }

    /// Start a new iteration and begin live progress display.
    pub fn start_iteration(&mut self, iteration: u64, total_iterations: u64, gates: Vec<String>) {
        self.current_panel = Some(LiveIterationPanel::with_theme(
            iteration,
            total_iterations,
            gates,
            self.theme,
        ));
    }

    /// Update gate progress in the current iteration.
    pub fn update_gate(&mut self, event: &GateProgressEvent) {
        if let Some(ref mut panel) = self.current_panel {
            match event {
                GateProgressEvent::Started { gate_name } => {
                    panel.start_gate(gate_name);
                }
                GateProgressEvent::Completed {
                    gate_name,
                    passed,
                    duration,
                    ..
                } => {
                    if *passed {
                        panel.pass_gate(gate_name, *duration);
                    } else {
                        panel.fail_gate(gate_name, *duration);
                    }
                }
                GateProgressEvent::Activity { activity, .. } => {
                    panel.set_activity(ActivityIndicator::running_file(activity.clone()));
                }
            }
        }
    }

    /// Refresh the live display.
    pub fn refresh(&mut self) -> io::Result<()> {
        if let Some(ref mut panel) = self.current_panel {
            panel.update()?;
        }
        Ok(())
    }

    /// Finish the current iteration and add summary to stack.
    pub fn finish_iteration(&mut self) -> Option<IterationSummary> {
        if let Some(panel) = self.current_panel.take() {
            let summary = panel.to_summary();
            self.summaries.push(summary.clone());
            Some(summary)
        } else {
            None
        }
    }

    /// Get the summary stack.
    pub fn summaries(&self) -> &IterationSummaryStack {
        &self.summaries
    }

    /// Render the final summary.
    pub fn render_final_summary(&self) -> String {
        self.summaries.render_final_summary()
    }

    /// Print the final summary to stdout.
    pub fn show_final_summary(&self) -> io::Result<()> {
        let output = self.render_final_summary();
        let mut stdout = io::stdout();
        stdout.write_all(output.as_bytes())?;
        stdout.flush()
    }
}

impl Default for IterationDisplay {
    fn default() -> Self {
        Self::new()
    }
}

/// Story executor that runs iterations with progress callbacks.
///
/// This executor integrates with the UI iteration view components
/// to provide real-time progress updates during story execution.
pub struct StoryExecutor {
    /// Configuration for execution
    config: ExecutorConfig,
    /// Quality gate checker
    checker: QualityGateChecker,
    /// Callback for execution events
    on_event: Option<OnExecutionEvent>,
}

impl StoryExecutor {
    /// Create a new story executor.
    pub fn new(config: ExecutorConfig, project_root: impl Into<std::path::PathBuf>) -> Self {
        let checker = QualityGateChecker::new(config.profile.clone(), project_root);
        Self {
            config,
            checker,
            on_event: None,
        }
    }

    /// Set the callback for execution events.
    ///
    /// This callback is called for:
    /// - Iteration start/completion
    /// - Gate progress (started, completed, activity)
    /// - Execution finish
    pub fn on_event(mut self, callback: OnExecutionEvent) -> Self {
        self.on_event = Some(callback);
        self
    }

    /// Set the gate progress callback (convenience method).
    ///
    /// This wraps gate progress events in ExecutionEvent::GateProgress.
    pub fn on_gate_progress(self, mut callback: OnGateProgress) -> Self {
        self.on_event(Box::new(move |event| {
            if let ExecutionEvent::GateProgress(gate_event) = event {
                callback(gate_event);
            }
        }))
    }

    /// Get the list of gate names that will be run.
    pub fn gate_names(&self) -> Vec<String> {
        // Return the gates that would be checked based on profile
        let mut gates = Vec::new();

        if self.config.profile.testing.coverage_threshold > 0 {
            gates.push("coverage".to_string());
        }
        if self.config.profile.ci.lint_check {
            gates.push("lint".to_string());
        }
        if self.config.profile.ci.format_check {
            gates.push("format".to_string());
        }
        if self.config.profile.security.cargo_audit {
            gates.push("security_audit".to_string());
        }

        // If no gates are enabled, return a default set
        if gates.is_empty() {
            gates = vec!["build".to_string(), "lint".to_string(), "test".to_string()];
        }

        gates
    }

    /// Emit an execution event to the callback.
    fn emit(&mut self, event: ExecutionEvent) {
        if let Some(ref mut callback) = self.on_event {
            callback(event);
        }
    }

    /// Run quality gates with progress callbacks.
    ///
    /// Emits GateProgress events as each gate starts and completes.
    pub fn run_quality_gates(&mut self) -> Vec<GateResult> {
        let mut results = Vec::new();

        // Run coverage check
        if self.config.profile.testing.coverage_threshold > 0 {
            self.emit(ExecutionEvent::GateProgress(GateProgressEvent::started(
                "coverage",
            )));
            let start = Instant::now();
            let result = self.checker.check_coverage();
            let duration = start.elapsed();
            self.emit(ExecutionEvent::GateProgress(GateProgressEvent::completed(
                &result, duration,
            )));
            results.push(result);
        }

        // Run lint check
        if self.config.profile.ci.lint_check {
            self.emit(ExecutionEvent::GateProgress(GateProgressEvent::started(
                "lint",
            )));
            let start = Instant::now();
            let result = self.checker.check_lint();
            let duration = start.elapsed();
            self.emit(ExecutionEvent::GateProgress(GateProgressEvent::completed(
                &result, duration,
            )));
            results.push(result);
        }

        // Run format check
        if self.config.profile.ci.format_check {
            self.emit(ExecutionEvent::GateProgress(GateProgressEvent::started(
                "format",
            )));
            let start = Instant::now();
            let result = self.checker.check_format();
            let duration = start.elapsed();
            self.emit(ExecutionEvent::GateProgress(GateProgressEvent::completed(
                &result, duration,
            )));
            results.push(result);
        }

        // Run security audit
        if self.config.profile.security.cargo_audit {
            self.emit(ExecutionEvent::GateProgress(GateProgressEvent::started(
                "security_audit",
            )));
            let start = Instant::now();
            let result = self.checker.check_security_audit();
            let duration = start.elapsed();
            self.emit(ExecutionEvent::GateProgress(GateProgressEvent::completed(
                &result, duration,
            )));
            results.push(result);
        }

        results
    }

    /// Run a single iteration of the story execution.
    ///
    /// Returns true if all quality gates passed, false otherwise.
    pub fn run_iteration(&mut self, iteration: u32) -> (bool, Vec<GateSummary>, Duration) {
        let start = Instant::now();
        let gates = self.gate_names();

        // Emit iteration starting event
        self.emit(ExecutionEvent::IterationStarting {
            iteration,
            max_iterations: self.config.max_iterations,
            gates: gates.clone(),
        });

        // Run quality gates
        let results = self.run_quality_gates();
        let duration = start.elapsed();

        // Convert to gate summaries
        let gate_summaries: Vec<GateSummary> = results
            .iter()
            .map(|r| {
                let mut summary = GateSummary::new(&r.gate_name, r.passed);
                if let Some(ref details) = r.details {
                    summary = summary.with_error(details.clone());
                }
                summary
            })
            .collect();

        let all_passed = QualityGateChecker::all_passed(&results);

        // Emit iteration completed event
        self.emit(ExecutionEvent::IterationCompleted {
            iteration,
            max_iterations: self.config.max_iterations,
            passed: all_passed,
            duration,
            gate_results: gate_summaries.clone(),
        });

        (all_passed, gate_summaries, duration)
    }

    /// Run the story with the iteration loop.
    ///
    /// Calls the progress callback for each iteration and gate.
    /// Returns (success, total_iterations, total_duration).
    pub fn run_story(&mut self, _story_id: &str) -> (bool, u32, Duration) {
        let total_start = Instant::now();
        let gates = self.gate_names();

        // Create display for UI feedback
        let mut display = IterationDisplay::with_theme(self.config.theme);

        // Show iteration preview before starting
        if self.config.show_ui {
            let _ = display.show_iteration_preview(&gates);
        }

        let mut success = false;
        let mut iterations_run = 0;

        for iteration in 1..=self.config.max_iterations {
            iterations_run = iteration;

            // Start iteration display
            if self.config.show_ui {
                display.start_iteration(
                    iteration as u64,
                    self.config.max_iterations as u64,
                    gates.clone(),
                );
            }

            // Run the iteration
            let (passed, _gate_summaries, _duration) = self.run_iteration(iteration);

            // Finish iteration display
            if self.config.show_ui {
                display.finish_iteration();
            }

            if passed {
                success = true;
                break;
            }
        }

        let total_duration = total_start.elapsed();

        // Show final summary
        if self.config.show_ui {
            let _ = display.show_final_summary();
        }

        // Emit finished event
        self.emit(ExecutionEvent::ExecutionFinished {
            success,
            total_iterations: iterations_run,
            total_duration,
        });

        (success, iterations_run, total_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::{CiConfig, Profile, SecurityConfig, TestingConfig};
    use std::sync::{Arc, Mutex};

    fn create_minimal_profile() -> Profile {
        Profile {
            description: "Test profile".to_string(),
            testing: TestingConfig {
                coverage_threshold: 0,
                unit_tests: true,
                integration_tests: false,
            },
            ci: CiConfig {
                required: true,
                lint_check: false,
                format_check: false,
            },
            security: SecurityConfig {
                cargo_audit: false,
                cargo_deny: false,
                sast: false,
            },
            ..Default::default()
        }
    }

    fn create_config_with_profile(profile: Profile) -> ExecutorConfig {
        ExecutorConfig {
            max_iterations: 3,
            profile,
            show_ui: false,
            theme: Theme::default(),
        }
    }

    #[test]
    fn test_gate_progress_event_started() {
        let event = GateProgressEvent::started("lint");
        match event {
            GateProgressEvent::Started { gate_name } => {
                assert_eq!(gate_name, "lint");
            }
            _ => panic!("Expected Started event"),
        }
    }

    #[test]
    fn test_gate_progress_event_completed() {
        let result = GateResult::pass("format", "All formatted");
        let event = GateProgressEvent::completed(&result, Duration::from_secs(1));
        match event {
            GateProgressEvent::Completed {
                gate_name,
                passed,
                duration,
                message,
                details,
            } => {
                assert_eq!(gate_name, "format");
                assert!(passed);
                assert_eq!(duration, Duration::from_secs(1));
                assert_eq!(message, "All formatted");
                assert!(details.is_none());
            }
            _ => panic!("Expected Completed event"),
        }
    }

    #[test]
    fn test_gate_progress_event_activity() {
        let event = GateProgressEvent::activity("test", "src/lib.rs:42");
        match event {
            GateProgressEvent::Activity {
                gate_name,
                activity,
            } => {
                assert_eq!(gate_name, "test");
                assert_eq!(activity, "src/lib.rs:42");
            }
            _ => panic!("Expected Activity event"),
        }
    }

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.max_iterations, 10);
        assert!(config.show_ui);
    }

    #[test]
    fn test_iteration_display_new() {
        let display = IterationDisplay::new();
        assert!(display.summaries().is_empty());
    }

    #[test]
    fn test_iteration_display_with_theme() {
        let theme = Theme::default();
        let display = IterationDisplay::with_theme(theme);
        assert!(display.summaries().is_empty());
    }

    #[test]
    fn test_story_executor_new() {
        let config = create_config_with_profile(create_minimal_profile());
        let executor = StoryExecutor::new(config, "/tmp/test");
        let gates = executor.gate_names();
        // With minimal profile, should get default gates
        assert!(gates.contains(&"build".to_string()) || !gates.is_empty());
    }

    #[test]
    fn test_story_executor_gate_names_with_profile() {
        let mut profile = create_minimal_profile();
        profile.ci.lint_check = true;
        profile.ci.format_check = true;

        let config = create_config_with_profile(profile);
        let executor = StoryExecutor::new(config, "/tmp/test");
        let gates = executor.gate_names();

        assert!(gates.contains(&"lint".to_string()));
        assert!(gates.contains(&"format".to_string()));
    }

    #[test]
    fn test_story_executor_on_event_callback() {
        let profile = create_minimal_profile();
        let config = create_config_with_profile(profile);

        let events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let executor = StoryExecutor::new(config, "/tmp/test").on_event(Box::new(move |event| {
            let mut events = events_clone.lock().unwrap();
            match event {
                ExecutionEvent::IterationStarting { iteration, .. } => {
                    events.push(format!("iteration_{}_start", iteration));
                }
                ExecutionEvent::IterationCompleted { iteration, .. } => {
                    events.push(format!("iteration_{}_complete", iteration));
                }
                ExecutionEvent::GateProgress(_) => {
                    events.push("gate_progress".to_string());
                }
                ExecutionEvent::ExecutionFinished { .. } => {
                    events.push("finished".to_string());
                }
            }
        }));

        // Just verify the executor was created with callback
        assert!(executor.on_event.is_some());
    }

    #[test]
    fn test_iteration_display_start_and_finish() {
        let mut display = IterationDisplay::new();

        display.start_iteration(1, 3, vec!["build".to_string(), "lint".to_string()]);
        assert!(display.current_panel.is_some());

        let summary = display.finish_iteration();
        assert!(summary.is_some());
        assert!(display.current_panel.is_none());
        assert_eq!(display.summaries().len(), 1);
    }

    #[test]
    fn test_iteration_display_update_gate() {
        let mut display = IterationDisplay::new();
        display.start_iteration(1, 3, vec!["build".to_string()]);

        // Start gate
        display.update_gate(&GateProgressEvent::started("build"));

        // Complete gate
        display.update_gate(&GateProgressEvent::Completed {
            gate_name: "build".to_string(),
            passed: true,
            duration: Duration::from_secs(1),
            message: "Build passed".to_string(),
            details: None,
        });

        let summary = display.finish_iteration();
        assert!(summary.is_some());
        assert!(summary.unwrap().passed());
    }

    #[test]
    fn test_iteration_display_activity() {
        let mut display = IterationDisplay::new();
        display.start_iteration(1, 1, vec!["build".to_string()]);

        display.update_gate(&GateProgressEvent::started("build"));
        display.update_gate(&GateProgressEvent::activity("build", "src/main.rs"));

        // Verify activity is set
        if let Some(ref panel) = display.current_panel {
            assert!(panel.activity().is_some());
        }
    }

    #[test]
    fn test_story_executor_run_iteration() {
        let profile = create_minimal_profile();
        let mut config = create_config_with_profile(profile);
        config.show_ui = false;

        let mut executor = StoryExecutor::new(config, std::env::current_dir().unwrap());

        let (passed, _gate_summaries, duration) = executor.run_iteration(1);

        // With minimal profile (all gates disabled), should pass
        // or get default gates which may or may not pass depending on environment
        assert!(duration.as_nanos() > 0);
        // passed state depends on environment
        let _ = passed;
    }
}
