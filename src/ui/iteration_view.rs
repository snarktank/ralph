//! Iteration view component for Ralph's terminal UI.
//!
//! Displays what's planned before each iteration starts,
//! showing the gates that will run. Also provides live progress
//! display during iteration execution.

#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use owo_colors::OwoColorize;

use crate::ui::colors::Theme;
use crate::ui::ghostty::{self, file_hyperlink_with_line, TerminalCapabilities};
use crate::ui::spinner::spinner_chars;

/// Preview of what will happen in an iteration.
///
/// Shows the list of quality gates that will be executed.
#[derive(Debug, Clone)]
pub struct IterationPreview {
    /// Names of the gates to be run
    gates: Vec<String>,
    /// Color theme for rendering
    theme: Theme,
}

impl IterationPreview {
    /// Create a new iteration preview with the given gates.
    pub fn new(gates: Vec<String>) -> Self {
        Self {
            gates,
            theme: Theme::default(),
        }
    }

    /// Create an iteration preview with a custom theme.
    pub fn with_theme(gates: Vec<String>, theme: Theme) -> Self {
        Self { gates, theme }
    }

    /// Get the list of gates to be run.
    pub fn gates(&self) -> &[String] {
        &self.gates
    }

    /// Render the pre-iteration header showing gates to run.
    ///
    /// Format: "Will run: build → lint → test"
    pub fn render(&self) -> String {
        if self.gates.is_empty() {
            return format!("{} No gates configured\n", "○".color(self.theme.muted));
        }

        let gate_chain = self
            .gates
            .iter()
            .map(|g| g.as_str())
            .collect::<Vec<_>>()
            .join(" → ");

        format!(
            "{} {}\n",
            "Will run:".color(self.theme.muted),
            gate_chain.color(self.theme.in_progress)
        )
    }

    /// Render as a compact inline format.
    ///
    /// Format: "build → lint → test"
    pub fn render_compact(&self) -> String {
        if self.gates.is_empty() {
            return "No gates".to_string();
        }

        self.gates
            .iter()
            .map(|g| g.as_str())
            .collect::<Vec<_>>()
            .join(" → ")
    }
}

// ============================================================================
// Gate Progress
// ============================================================================

/// Progress state for a quality gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateProgress {
    /// Gate has not started yet
    Pending,
    /// Gate is currently running
    Running,
    /// Gate completed successfully
    Passed,
    /// Gate failed
    Failed,
}

impl GateProgress {
    /// Get the status indicator character for this gate state.
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Pending => "○", // Empty circle
            Self::Running => "◐", // Half-filled circle (will animate)
            Self::Passed => "✓",  // Checkmark
            Self::Failed => "✗",  // X mark
        }
    }

    /// Check if the gate is in a finished state.
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Passed | Self::Failed)
    }
}

// ============================================================================
// Gate Progress Info
// ============================================================================

/// Progress information for a single gate during iteration execution.
///
/// Tracks the current state, timing, and duration of a gate.
#[derive(Debug, Clone)]
pub struct GateProgressInfo {
    /// Name of the gate
    pub name: String,
    /// Current progress state
    pub progress: GateProgress,
    /// Duration if completed (None if pending/running)
    pub duration: Option<Duration>,
    /// When this gate started running (for elapsed time calculation)
    started_at: Option<Instant>,
}

impl GateProgressInfo {
    /// Create a new pending gate progress info.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            progress: GateProgress::Pending,
            duration: None,
            started_at: None,
        }
    }

    /// Mark the gate as running.
    pub fn start(&mut self) {
        self.progress = GateProgress::Running;
        self.started_at = Some(Instant::now());
    }

    /// Mark the gate as passed with the given duration.
    pub fn pass(&mut self, duration: Duration) {
        self.progress = GateProgress::Passed;
        self.duration = Some(duration);
    }

    /// Mark the gate as failed with the given duration.
    pub fn fail(&mut self, duration: Duration) {
        self.progress = GateProgress::Failed;
        self.duration = Some(duration);
    }

    /// Get the elapsed time since the gate started running.
    pub fn elapsed(&self) -> Option<Duration> {
        self.started_at.map(|start| start.elapsed())
    }

    /// Format the duration for display.
    pub fn format_duration(&self) -> Option<String> {
        self.duration.map(|d| {
            if d.as_secs() >= 60 {
                format!(
                    "{}m{:.1}s",
                    d.as_secs() / 60,
                    (d.as_millis() % 60000) as f64 / 1000.0
                )
            } else {
                format!("{:.1}s", d.as_secs_f64())
            }
        })
    }
}

// ============================================================================
// Activity Indicator
// ============================================================================

/// Represents the current activity during a long-running operation.
///
/// Used to show what file or action is happening during quality gate execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityIndicator {
    /// The activity description (e.g., "Running:")
    prefix: String,
    /// The activity target (e.g., file path or action name)
    target: String,
    /// Optional line number for file-based activities
    line_number: Option<u32>,
}

impl ActivityIndicator {
    /// Create a new activity indicator.
    ///
    /// # Arguments
    /// * `prefix` - The activity prefix (e.g., "Running:", "Testing:")
    /// * `target` - The target of the activity (e.g., file path, test name)
    pub fn new(prefix: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            target: target.into(),
            line_number: None,
        }
    }

    /// Create an activity indicator with a line number.
    ///
    /// Useful for showing file:line format like "src/tests/auth.rs:142"
    pub fn with_line(prefix: impl Into<String>, target: impl Into<String>, line: u32) -> Self {
        Self {
            prefix: prefix.into(),
            target: target.into(),
            line_number: Some(line),
        }
    }

    /// Create a "Running" activity indicator for a file.
    pub fn running_file(path: impl Into<String>) -> Self {
        Self::new("Running:", path)
    }

    /// Create a "Running" activity indicator for a file with line number.
    pub fn running_file_at_line(path: impl Into<String>, line: u32) -> Self {
        Self::with_line("Running:", path, line)
    }

    /// Create a "Testing" activity indicator.
    pub fn testing(target: impl Into<String>) -> Self {
        Self::new("Testing:", target)
    }

    /// Create a "Compiling" activity indicator.
    pub fn compiling(target: impl Into<String>) -> Self {
        Self::new("Compiling:", target)
    }

    /// Create a "Linting" activity indicator.
    pub fn linting(target: impl Into<String>) -> Self {
        Self::new("Linting:", target)
    }

    /// Get the prefix.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Get the target.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Get the line number if set.
    pub fn line_number(&self) -> Option<u32> {
        self.line_number
    }

    /// Check if this is a file-based activity (has a path-like target).
    pub fn is_file_activity(&self) -> bool {
        // Simple heuristic: contains path separator or common file extensions
        self.target.contains('/')
            || self.target.contains('\\')
            || self.target.ends_with(".rs")
            || self.target.ends_with(".ts")
            || self.target.ends_with(".js")
            || self.target.ends_with(".py")
    }

    /// Render the activity indicator as a string.
    ///
    /// # Arguments
    /// * `theme` - Color theme for rendering
    /// * `capabilities` - Terminal capabilities for hyperlink support
    pub fn render(&self, theme: &Theme, capabilities: &TerminalCapabilities) -> String {
        let target_display = if let Some(line) = self.line_number {
            // File with line number - make it a clickable hyperlink if supported
            file_hyperlink_with_line(&self.target, line, None, capabilities)
        } else if self.is_file_activity() {
            // Just a file path without line number
            crate::ui::ghostty::file_hyperlink(&self.target, None, capabilities)
        } else {
            // Not a file, just use the target as-is
            self.target.clone()
        };

        format!(
            "{} {}",
            self.prefix.color(theme.muted),
            target_display.color(theme.in_progress)
        )
    }

    /// Render the activity indicator without hyperlinks (plain text).
    pub fn render_plain(&self, theme: &Theme) -> String {
        let target_display = if let Some(line) = self.line_number {
            format!("{}:{}", self.target, line)
        } else {
            self.target.clone()
        };

        format!(
            "{} {}",
            self.prefix.color(theme.muted),
            target_display.color(theme.in_progress)
        )
    }
}

// ============================================================================
// Live Iteration Panel
// ============================================================================

/// Live progress display for iteration execution.
///
/// Shows real-time gate progress with checkmarks, spinners, and timing.
/// Uses Ghostty synchronized output for flicker-free updates.
#[derive(Debug)]
pub struct LiveIterationPanel {
    /// Current iteration number
    iteration: u64,
    /// Total number of iterations
    total_iterations: u64,
    /// Gate progress information in order
    gates: Vec<GateProgressInfo>,
    /// Gate name to index mapping
    gate_indices: HashMap<String, usize>,
    /// When the iteration started
    started_at: Instant,
    /// Spinner animation frame
    spinner_frame: usize,
    /// Color theme
    theme: Theme,
    /// Terminal capabilities for Ghostty features
    capabilities: TerminalCapabilities,
    /// Number of lines rendered (for clearing)
    rendered_lines: usize,
    /// Current activity indicator (shown beneath running gate)
    activity: Option<ActivityIndicator>,
}

impl LiveIterationPanel {
    /// Create a new live iteration panel.
    pub fn new(iteration: u64, total_iterations: u64, gate_names: Vec<String>) -> Self {
        let mut gate_indices = HashMap::new();
        let mut gates = Vec::with_capacity(gate_names.len());

        for (idx, name) in gate_names.iter().enumerate() {
            gate_indices.insert(name.clone(), idx);
            gates.push(GateProgressInfo::new(name.clone()));
        }

        Self {
            iteration,
            total_iterations,
            gates,
            gate_indices,
            started_at: Instant::now(),
            spinner_frame: 0,
            theme: Theme::default(),
            capabilities: TerminalCapabilities::detect(),
            rendered_lines: 0,
            activity: None,
        }
    }

    /// Create a live iteration panel with a custom theme.
    pub fn with_theme(
        iteration: u64,
        total_iterations: u64,
        gate_names: Vec<String>,
        theme: Theme,
    ) -> Self {
        let mut panel = Self::new(iteration, total_iterations, gate_names);
        panel.theme = theme;
        panel
    }

    /// Create a live iteration panel with specific terminal capabilities.
    pub fn with_capabilities(
        iteration: u64,
        total_iterations: u64,
        gate_names: Vec<String>,
        capabilities: TerminalCapabilities,
    ) -> Self {
        let mut panel = Self::new(iteration, total_iterations, gate_names);
        panel.capabilities = capabilities;
        panel
    }

    /// Get the elapsed time since the iteration started.
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Format elapsed time for display.
    pub fn format_elapsed(&self) -> String {
        let elapsed = self.elapsed();
        if elapsed.as_secs() >= 60 {
            format!(
                "{}m{:.1}s",
                elapsed.as_secs() / 60,
                (elapsed.as_millis() % 60000) as f64 / 1000.0
            )
        } else {
            format!("{:.1}s", elapsed.as_secs_f64())
        }
    }

    /// Mark a gate as running.
    pub fn start_gate(&mut self, name: &str) {
        if let Some(&idx) = self.gate_indices.get(name) {
            self.gates[idx].start();
        }
    }

    /// Mark a gate as passed.
    ///
    /// Also clears the current activity indicator since the gate has completed.
    pub fn pass_gate(&mut self, name: &str, duration: Duration) {
        if let Some(&idx) = self.gate_indices.get(name) {
            self.gates[idx].pass(duration);
            self.clear_activity();
        }
    }

    /// Mark a gate as failed.
    ///
    /// Also clears the current activity indicator since the gate has completed.
    pub fn fail_gate(&mut self, name: &str, duration: Duration) {
        if let Some(&idx) = self.gate_indices.get(name) {
            self.gates[idx].fail(duration);
            self.clear_activity();
        }
    }

    /// Set the current activity indicator.
    ///
    /// Shows what file or action is happening during a long-running gate.
    /// Displayed beneath the running gate in the UI.
    ///
    /// # Arguments
    /// * `activity` - The activity indicator to display
    pub fn set_activity(&mut self, activity: ActivityIndicator) {
        self.activity = Some(activity);
    }

    /// Clear the current activity indicator.
    ///
    /// Called automatically when a gate passes or fails.
    pub fn clear_activity(&mut self) {
        self.activity = None;
    }

    /// Get the current activity indicator if set.
    pub fn activity(&self) -> Option<&ActivityIndicator> {
        self.activity.as_ref()
    }

    /// Get the current spinner character for animation.
    fn spinner_char(&self) -> &'static str {
        spinner_chars::BRAILLE[self.spinner_frame % spinner_chars::BRAILLE.len()]
    }

    /// Advance the spinner animation frame.
    pub fn tick(&mut self) {
        self.spinner_frame = self.spinner_frame.wrapping_add(1);
    }

    /// Check if all gates have finished.
    pub fn is_finished(&self) -> bool {
        self.gates.iter().all(|g| g.progress.is_finished())
    }

    /// Check if any gate has failed.
    pub fn has_failure(&self) -> bool {
        self.gates
            .iter()
            .any(|g| g.progress == GateProgress::Failed)
    }

    /// Render a single gate progress info.
    fn render_gate(&self, gate: &GateProgressInfo) -> String {
        let indicator = match gate.progress {
            GateProgress::Pending => format!("{}", "○".color(self.theme.muted)),
            GateProgress::Running => {
                format!("{}", self.spinner_char().color(self.theme.in_progress))
            }
            GateProgress::Passed => format!("{}", "✓".color(self.theme.success)),
            GateProgress::Failed => format!("{}", "✗".color(self.theme.error)),
        };

        let name = match gate.progress {
            GateProgress::Pending => format!("{}", gate.name.color(self.theme.muted)),
            GateProgress::Running => format!("{}", gate.name.color(self.theme.in_progress)),
            GateProgress::Passed => format!("{}", gate.name.color(self.theme.success)),
            GateProgress::Failed => format!("{}", gate.name.color(self.theme.error)),
        };

        // Add timing for completed gates
        if let Some(duration_str) = gate.format_duration() {
            format!(
                "{} {} ({})",
                indicator,
                name,
                duration_str.color(self.theme.muted)
            )
        } else {
            format!("{} {}", indicator, name)
        }
    }

    /// Render the complete panel as a string.
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Header line: "Iteration 1/5 (2.3s)"
        let header = format!(
            "Iteration {}/{} ({})",
            self.iteration,
            self.total_iterations,
            self.format_elapsed()
        );
        output.push_str(&format!("{}\n", header.color(self.theme.muted)));

        // Gate progress lines
        for gate in &self.gates {
            output.push_str(&format!("  {}\n", self.render_gate(gate)));

            // Show activity beneath running gate
            if gate.progress == GateProgress::Running {
                if let Some(ref activity) = self.activity {
                    output.push_str(&format!(
                        "    {}\n",
                        activity.render(&self.theme, &self.capabilities)
                    ));
                }
            }
        }

        output
    }

    /// Render the panel inline (single line format).
    ///
    /// Format: "Iteration 1/5: ✓ build ◐ lint ○ test (2.3s)"
    pub fn render_inline(&self) -> String {
        let gate_parts: Vec<String> = self
            .gates
            .iter()
            .map(|g| {
                let indicator = match g.progress {
                    GateProgress::Pending => format!("{}", "○".color(self.theme.muted)),
                    GateProgress::Running => {
                        format!("{}", self.spinner_char().color(self.theme.in_progress))
                    }
                    GateProgress::Passed => format!("{}", "✓".color(self.theme.success)),
                    GateProgress::Failed => format!("{}", "✗".color(self.theme.error)),
                };

                if let Some(duration_str) = g.format_duration() {
                    format!("{} {}({})", indicator, g.name, duration_str)
                } else {
                    format!("{} {}", indicator, g.name)
                }
            })
            .collect();

        format!(
            "Iteration {}/{}: {} ({})",
            self.iteration,
            self.total_iterations,
            gate_parts.join(" "),
            self.format_elapsed()
        )
    }

    /// Clear previously rendered output from the terminal.
    pub fn clear(&self) -> io::Result<()> {
        if self.rendered_lines == 0 {
            return Ok(());
        }

        let mut stdout = io::stdout();
        // Move cursor up and clear each line
        for _ in 0..self.rendered_lines {
            stdout.write_all(b"\x1b[1A\x1b[2K")?;
        }
        stdout.flush()
    }

    /// Render the panel to stdout with Ghostty synchronized output.
    pub fn display(&mut self) -> io::Result<()> {
        // Clear previous output
        self.clear()?;

        let output = self.render();
        let line_count = output.lines().count();

        // Use synchronized output if available
        ghostty::begin_sync(&self.capabilities)?;

        let mut stdout = io::stdout();
        stdout.write_all(output.as_bytes())?;
        stdout.flush()?;

        ghostty::end_sync(&self.capabilities)?;

        self.rendered_lines = line_count;
        Ok(())
    }

    /// Update display with a tick (for spinner animation).
    pub fn update(&mut self) -> io::Result<()> {
        self.tick();
        self.display()
    }

    /// Convert to an iteration summary for display after completion.
    pub fn to_summary(&self) -> IterationSummary {
        let gate_results: Vec<GateSummary> = self
            .gates
            .iter()
            .map(|g| GateSummary {
                name: g.name.clone(),
                passed: g.progress == GateProgress::Passed,
                duration: g.duration,
                error_details: None, // Error details can be added later
            })
            .collect();

        let passed = !self.has_failure();
        let total_duration = self.elapsed();

        IterationSummary::new(
            self.iteration,
            self.total_iterations,
            passed,
            total_duration,
        )
        .with_gates(gate_results)
    }
}

// ============================================================================
// Iteration Summary
// ============================================================================

/// Summary of a single gate result for display in iteration summary.
#[derive(Debug, Clone)]
pub struct GateSummary {
    /// Name of the gate
    pub name: String,
    /// Whether the gate passed
    pub passed: bool,
    /// Duration of the gate execution
    pub duration: Option<Duration>,
    /// Error details if the gate failed (for expanded view)
    pub error_details: Option<String>,
}

impl GateSummary {
    /// Create a new gate summary.
    pub fn new(name: impl Into<String>, passed: bool) -> Self {
        Self {
            name: name.into(),
            passed,
            duration: None,
            error_details: None,
        }
    }

    /// Create a gate summary with duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Create a gate summary with error details.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_details = Some(error.into());
        self
    }

    /// Format the duration for display.
    pub fn format_duration(&self) -> Option<String> {
        self.duration.map(|d| {
            if d.as_secs() >= 60 {
                format!(
                    "{}m{:.1}s",
                    d.as_secs() / 60,
                    (d.as_millis() % 60000) as f64 / 1000.0
                )
            } else {
                format!("{:.1}s", d.as_secs_f64())
            }
        })
    }
}

/// Summary of a completed iteration.
///
/// Renders as a collapsed single line for successful iterations,
/// or auto-expands to show error details for failed iterations.
#[derive(Debug, Clone)]
pub struct IterationSummary {
    /// Iteration number
    iteration: u64,
    /// Total number of iterations
    total_iterations: u64,
    /// Whether the iteration passed (all gates successful)
    passed: bool,
    /// Total duration of the iteration
    duration: Duration,
    /// Individual gate summaries
    gates: Vec<GateSummary>,
    /// Color theme for rendering
    theme: Theme,
}

impl IterationSummary {
    /// Create a new iteration summary.
    pub fn new(iteration: u64, total_iterations: u64, passed: bool, duration: Duration) -> Self {
        Self {
            iteration,
            total_iterations,
            passed,
            duration,
            gates: Vec::new(),
            theme: Theme::default(),
        }
    }

    /// Create an iteration summary with a custom theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set the gate summaries for this iteration.
    pub fn with_gates(mut self, gates: Vec<GateSummary>) -> Self {
        self.gates = gates;
        self
    }

    /// Get whether this iteration passed.
    pub fn passed(&self) -> bool {
        self.passed
    }

    /// Get the iteration number.
    pub fn iteration(&self) -> u64 {
        self.iteration
    }

    /// Get the total duration.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Format the total duration for display.
    pub fn format_duration(&self) -> String {
        if self.duration.as_secs() >= 60 {
            format!(
                "{}m{:.1}s",
                self.duration.as_secs() / 60,
                (self.duration.as_millis() % 60000) as f64 / 1000.0
            )
        } else {
            format!("{:.1}s", self.duration.as_secs_f64())
        }
    }

    /// Render the collapsed summary line.
    ///
    /// Format: "▸ Iteration 1/5: ✓ build ✓ lint ✓ test (3.2s)"
    pub fn render_collapsed(&self) -> String {
        let gate_parts: Vec<String> = self
            .gates
            .iter()
            .map(|g| {
                let indicator = if g.passed {
                    format!("{}", "✓".color(self.theme.success))
                } else {
                    format!("{}", "✗".color(self.theme.error))
                };
                format!("{} {}", indicator, g.name)
            })
            .collect();

        let status_color = if self.passed {
            self.theme.success
        } else {
            self.theme.error
        };

        format!(
            "{} Iteration {}/{}: {} ({})",
            "▸".color(self.theme.muted),
            self.iteration.to_string().color(status_color),
            self.total_iterations,
            gate_parts.join(" "),
            self.format_duration().color(self.theme.muted)
        )
    }

    /// Render the expanded view showing error details.
    ///
    /// Used when an iteration failed to show what went wrong.
    pub fn render_expanded(&self) -> String {
        let mut output = String::new();

        // Header line with expanded indicator
        let status_color = if self.passed {
            self.theme.success
        } else {
            self.theme.error
        };

        output.push_str(&format!(
            "{} Iteration {}/{} ({})\n",
            "▾".color(self.theme.muted),
            self.iteration.to_string().color(status_color),
            self.total_iterations,
            self.format_duration().color(self.theme.muted)
        ));

        // Gate details
        for gate in &self.gates {
            let indicator = if gate.passed {
                format!("{}", "✓".color(self.theme.success))
            } else {
                format!("{}", "✗".color(self.theme.error))
            };

            let name_color = if gate.passed {
                self.theme.success
            } else {
                self.theme.error
            };

            let timing = gate
                .format_duration()
                .map(|d| format!(" ({})", d.color(self.theme.muted)))
                .unwrap_or_default();

            output.push_str(&format!(
                "  {} {}{}\n",
                indicator,
                gate.name.color(name_color),
                timing
            ));

            // Show error details for failed gates
            if !gate.passed {
                if let Some(ref error) = gate.error_details {
                    for line in error.lines() {
                        output.push_str(&format!("    {}\n", line.color(self.theme.error)));
                    }
                }
            }
        }

        output
    }

    /// Render the summary, auto-expanding if failed.
    ///
    /// Successful iterations are collapsed, failed ones are expanded.
    pub fn render(&self) -> String {
        if self.passed {
            format!("{}\n", self.render_collapsed())
        } else {
            self.render_expanded()
        }
    }
}

// ============================================================================
// Iteration Summary Stack
// ============================================================================

/// A stack of iteration summaries for displaying multiple completed iterations.
///
/// Renders all summaries stacked vertically, with failed iterations expanded.
#[derive(Debug, Clone)]
pub struct IterationSummaryStack {
    /// Completed iteration summaries in order
    summaries: Vec<IterationSummary>,
    /// Color theme for rendering
    theme: Theme,
}

impl IterationSummaryStack {
    /// Create a new empty summary stack.
    pub fn new() -> Self {
        Self {
            summaries: Vec::new(),
            theme: Theme::default(),
        }
    }

    /// Create a summary stack with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            summaries: Vec::new(),
            theme,
        }
    }

    /// Add a completed iteration summary to the stack.
    pub fn push(&mut self, summary: IterationSummary) {
        self.summaries.push(summary);
    }

    /// Get the number of summaries in the stack.
    pub fn len(&self) -> usize {
        self.summaries.len()
    }

    /// Check if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.summaries.is_empty()
    }

    /// Get all summaries.
    pub fn summaries(&self) -> &[IterationSummary] {
        &self.summaries
    }

    /// Check if all iterations passed.
    pub fn all_passed(&self) -> bool {
        self.summaries.iter().all(|s| s.passed())
    }

    /// Get the total duration of all iterations.
    pub fn total_duration(&self) -> Duration {
        self.summaries.iter().map(|s| s.duration()).sum()
    }

    /// Format the total duration for display.
    pub fn format_total_duration(&self) -> String {
        let total = self.total_duration();
        if total.as_secs() >= 60 {
            format!(
                "{}m{:.1}s",
                total.as_secs() / 60,
                (total.as_millis() % 60000) as f64 / 1000.0
            )
        } else {
            format!("{:.1}s", total.as_secs_f64())
        }
    }

    /// Render all summaries stacked vertically.
    pub fn render(&self) -> String {
        if self.summaries.is_empty() {
            return String::new();
        }

        self.summaries
            .iter()
            .map(|s| s.render())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Render a final summary line after all iterations complete.
    pub fn render_final_summary(&self) -> String {
        if self.summaries.is_empty() {
            return String::new();
        }

        let passed_count = self.summaries.iter().filter(|s| s.passed()).count();
        let total_count = self.summaries.len();

        let status = if self.all_passed() {
            format!("{}", "All iterations passed".color(self.theme.success))
        } else {
            format!(
                "{}/{} iterations passed",
                passed_count.to_string().color(if passed_count > 0 {
                    self.theme.success
                } else {
                    self.theme.error
                }),
                total_count
            )
        };

        format!(
            "\n{} (total: {})\n",
            status,
            self.format_total_duration().color(self.theme.muted)
        )
    }
}

impl Default for IterationSummaryStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iteration_preview_new() {
        let gates = vec!["build".to_string(), "lint".to_string(), "test".to_string()];
        let preview = IterationPreview::new(gates.clone());
        assert_eq!(preview.gates(), &gates);
    }

    #[test]
    fn test_iteration_preview_empty() {
        let preview = IterationPreview::new(vec![]);
        assert!(preview.gates().is_empty());
        let output = preview.render();
        assert!(output.contains("No gates configured"));
    }

    #[test]
    fn test_iteration_preview_render() {
        let gates = vec!["build".to_string(), "lint".to_string(), "test".to_string()];
        let preview = IterationPreview::new(gates);
        let output = preview.render();

        // Check that the output contains the gate names
        assert!(output.contains("Will run:"));
        assert!(output.contains("build"));
        assert!(output.contains("lint"));
        assert!(output.contains("test"));
        // Check for arrow separator
        assert!(output.contains("→"));
    }

    #[test]
    fn test_iteration_preview_render_compact() {
        let gates = vec!["build".to_string(), "lint".to_string()];
        let preview = IterationPreview::new(gates);
        let output = preview.render_compact();

        assert_eq!(output, "build → lint");
    }

    #[test]
    fn test_iteration_preview_render_compact_empty() {
        let preview = IterationPreview::new(vec![]);
        let output = preview.render_compact();

        assert_eq!(output, "No gates");
    }

    #[test]
    fn test_iteration_preview_with_theme() {
        let gates = vec!["format".to_string()];
        let theme = Theme::default();
        let preview = IterationPreview::with_theme(gates.clone(), theme);

        assert_eq!(preview.gates(), &gates);
    }

    #[test]
    fn test_iteration_preview_single_gate() {
        let gates = vec!["typecheck".to_string()];
        let preview = IterationPreview::new(gates);
        let output = preview.render_compact();

        // Single gate should not have arrow
        assert_eq!(output, "typecheck");
        assert!(!output.contains("→"));
    }

    // ========================================================================
    // GateProgress Tests
    // ========================================================================

    #[test]
    fn test_gate_progress_indicator() {
        assert_eq!(GateProgress::Pending.indicator(), "○");
        assert_eq!(GateProgress::Running.indicator(), "◐");
        assert_eq!(GateProgress::Passed.indicator(), "✓");
        assert_eq!(GateProgress::Failed.indicator(), "✗");
    }

    #[test]
    fn test_gate_progress_is_finished() {
        assert!(!GateProgress::Pending.is_finished());
        assert!(!GateProgress::Running.is_finished());
        assert!(GateProgress::Passed.is_finished());
        assert!(GateProgress::Failed.is_finished());
    }

    // ========================================================================
    // GateProgressInfo Tests
    // ========================================================================

    #[test]
    fn test_gate_progress_info_new() {
        let info = GateProgressInfo::new("build");
        assert_eq!(info.name, "build");
        assert_eq!(info.progress, GateProgress::Pending);
        assert!(info.duration.is_none());
    }

    #[test]
    fn test_gate_progress_info_start() {
        let mut info = GateProgressInfo::new("lint");
        info.start();
        assert_eq!(info.progress, GateProgress::Running);
        assert!(info.elapsed().is_some());
    }

    #[test]
    fn test_gate_progress_info_pass() {
        let mut info = GateProgressInfo::new("test");
        let duration = Duration::from_secs_f64(1.5);
        info.pass(duration);
        assert_eq!(info.progress, GateProgress::Passed);
        assert_eq!(info.duration, Some(duration));
    }

    #[test]
    fn test_gate_progress_info_fail() {
        let mut info = GateProgressInfo::new("test");
        let duration = Duration::from_secs_f64(2.3);
        info.fail(duration);
        assert_eq!(info.progress, GateProgress::Failed);
        assert_eq!(info.duration, Some(duration));
    }

    #[test]
    fn test_gate_progress_info_format_duration_seconds() {
        let mut info = GateProgressInfo::new("build");
        info.pass(Duration::from_secs_f64(1.234));
        let formatted = info.format_duration().unwrap();
        assert!(formatted.contains("1.2"));
        assert!(formatted.ends_with('s'));
    }

    #[test]
    fn test_gate_progress_info_format_duration_minutes() {
        let mut info = GateProgressInfo::new("test");
        info.pass(Duration::from_secs(125)); // 2m 5s
        let formatted = info.format_duration().unwrap();
        assert!(formatted.contains("2m"));
    }

    // ========================================================================
    // LiveIterationPanel Tests
    // ========================================================================

    #[test]
    fn test_live_iteration_panel_new() {
        let gates = vec!["build".to_string(), "lint".to_string(), "test".to_string()];
        let panel = LiveIterationPanel::new(1, 5, gates);
        assert!(!panel.is_finished());
        assert!(!panel.has_failure());
    }

    #[test]
    fn test_live_iteration_panel_start_gate() {
        let gates = vec!["build".to_string(), "lint".to_string()];
        let mut panel = LiveIterationPanel::new(1, 3, gates);
        panel.start_gate("build");
        assert!(!panel.is_finished());
    }

    #[test]
    fn test_live_iteration_panel_pass_gate() {
        let gates = vec!["build".to_string()];
        let mut panel = LiveIterationPanel::new(1, 3, gates);
        panel.start_gate("build");
        panel.pass_gate("build", Duration::from_secs_f64(1.2));
        assert!(panel.is_finished());
        assert!(!panel.has_failure());
    }

    #[test]
    fn test_live_iteration_panel_fail_gate() {
        let gates = vec!["build".to_string()];
        let mut panel = LiveIterationPanel::new(1, 3, gates);
        panel.start_gate("build");
        panel.fail_gate("build", Duration::from_secs_f64(0.5));
        assert!(panel.is_finished());
        assert!(panel.has_failure());
    }

    #[test]
    fn test_live_iteration_panel_render() {
        let gates = vec!["build".to_string(), "lint".to_string(), "test".to_string()];
        let panel = LiveIterationPanel::new(2, 5, gates);
        let output = panel.render();

        assert!(output.contains("Iteration 2/5"));
        assert!(output.contains("build"));
        assert!(output.contains("lint"));
        assert!(output.contains("test"));
    }

    #[test]
    fn test_live_iteration_panel_render_with_progress() {
        let gates = vec!["build".to_string(), "lint".to_string(), "test".to_string()];
        let mut panel = LiveIterationPanel::new(1, 5, gates);

        // Complete first gate
        panel.start_gate("build");
        panel.pass_gate("build", Duration::from_secs_f64(1.5));

        // Start second gate
        panel.start_gate("lint");

        let output = panel.render();

        // Should show checkmark for passed gate
        assert!(output.contains("✓"));
        assert!(output.contains("build"));
        // Should show timing for completed gate
        assert!(output.contains("1.5s"));
    }

    #[test]
    fn test_live_iteration_panel_render_inline() {
        let gates = vec!["build".to_string(), "lint".to_string()];
        let mut panel = LiveIterationPanel::new(1, 3, gates);
        panel.pass_gate("build", Duration::from_secs_f64(1.0));

        let output = panel.render_inline();
        assert!(output.contains("Iteration 1/3"));
        assert!(output.contains("build"));
        assert!(output.contains("lint"));
    }

    #[test]
    fn test_live_iteration_panel_tick() {
        let gates = vec!["build".to_string()];
        let mut panel = LiveIterationPanel::new(1, 1, gates);
        let initial_frame = panel.spinner_frame;
        panel.tick();
        assert_eq!(panel.spinner_frame, initial_frame + 1);
    }

    #[test]
    fn test_live_iteration_panel_with_theme() {
        let gates = vec!["build".to_string()];
        let theme = Theme::default();
        let panel = LiveIterationPanel::with_theme(1, 1, gates, theme);
        assert_eq!(panel.iteration, 1);
    }

    #[test]
    fn test_live_iteration_panel_with_capabilities() {
        let gates = vec!["build".to_string()];
        let caps = TerminalCapabilities::minimal();
        let panel = LiveIterationPanel::with_capabilities(1, 1, gates, caps);
        assert!(!panel.capabilities.synchronized_output);
    }

    #[test]
    fn test_live_iteration_panel_unknown_gate() {
        let gates = vec!["build".to_string()];
        let mut panel = LiveIterationPanel::new(1, 1, gates);
        // Should not panic on unknown gate
        panel.start_gate("unknown");
        panel.pass_gate("unknown", Duration::from_secs(1));
        panel.fail_gate("unknown", Duration::from_secs(1));
    }

    #[test]
    fn test_live_iteration_panel_to_summary() {
        let gates = vec!["build".to_string(), "lint".to_string()];
        let mut panel = LiveIterationPanel::new(1, 5, gates);
        panel.pass_gate("build", Duration::from_secs_f64(1.0));
        panel.pass_gate("lint", Duration::from_secs_f64(0.5));

        let summary = panel.to_summary();
        assert!(summary.passed());
        assert_eq!(summary.iteration(), 1);
    }

    // ========================================================================
    // GateSummary Tests
    // ========================================================================

    #[test]
    fn test_gate_summary_new() {
        let summary = GateSummary::new("build", true);
        assert_eq!(summary.name, "build");
        assert!(summary.passed);
        assert!(summary.duration.is_none());
        assert!(summary.error_details.is_none());
    }

    #[test]
    fn test_gate_summary_with_duration() {
        let summary = GateSummary::new("lint", true).with_duration(Duration::from_secs_f64(2.5));
        assert_eq!(summary.format_duration(), Some("2.5s".to_string()));
    }

    #[test]
    fn test_gate_summary_with_error() {
        let summary = GateSummary::new("test", false).with_error("assertion failed at line 42");
        assert_eq!(
            summary.error_details,
            Some("assertion failed at line 42".to_string())
        );
    }

    #[test]
    fn test_gate_summary_format_duration_minutes() {
        let summary = GateSummary::new("test", true).with_duration(Duration::from_secs(125));
        let formatted = summary.format_duration().unwrap();
        assert!(formatted.contains("2m"));
    }

    // ========================================================================
    // IterationSummary Tests
    // ========================================================================

    #[test]
    fn test_iteration_summary_new() {
        let summary = IterationSummary::new(1, 5, true, Duration::from_secs_f64(3.2));
        assert!(summary.passed());
        assert_eq!(summary.iteration(), 1);
        assert_eq!(summary.duration(), Duration::from_secs_f64(3.2));
    }

    #[test]
    fn test_iteration_summary_with_theme() {
        let summary =
            IterationSummary::new(1, 5, true, Duration::from_secs(1)).with_theme(Theme::default());
        assert!(summary.passed());
    }

    #[test]
    fn test_iteration_summary_with_gates() {
        let gates = vec![
            GateSummary::new("build", true).with_duration(Duration::from_secs_f64(1.0)),
            GateSummary::new("lint", true).with_duration(Duration::from_secs_f64(0.5)),
        ];
        let summary =
            IterationSummary::new(1, 5, true, Duration::from_secs_f64(1.5)).with_gates(gates);
        assert!(summary.passed());
    }

    #[test]
    fn test_iteration_summary_format_duration_seconds() {
        let summary = IterationSummary::new(1, 5, true, Duration::from_secs_f64(3.2));
        assert_eq!(summary.format_duration(), "3.2s");
    }

    #[test]
    fn test_iteration_summary_format_duration_minutes() {
        let summary = IterationSummary::new(1, 5, true, Duration::from_secs(125));
        let formatted = summary.format_duration();
        assert!(formatted.contains("2m"));
    }

    #[test]
    fn test_iteration_summary_render_collapsed() {
        let gates = vec![
            GateSummary::new("build", true),
            GateSummary::new("lint", true),
            GateSummary::new("test", true),
        ];
        let summary =
            IterationSummary::new(1, 5, true, Duration::from_secs_f64(3.2)).with_gates(gates);
        let output = summary.render_collapsed();

        assert!(output.contains("▸"));
        assert!(output.contains("Iteration"));
        assert!(output.contains("1"));
        assert!(output.contains("5"));
        assert!(output.contains("build"));
        assert!(output.contains("lint"));
        assert!(output.contains("test"));
        assert!(output.contains("✓"));
        assert!(output.contains("3.2s"));
    }

    #[test]
    fn test_iteration_summary_render_collapsed_with_failure() {
        let gates = vec![
            GateSummary::new("build", true),
            GateSummary::new("lint", false),
        ];
        let summary =
            IterationSummary::new(1, 5, false, Duration::from_secs_f64(1.5)).with_gates(gates);
        let output = summary.render_collapsed();

        assert!(output.contains("✓")); // build passed
        assert!(output.contains("✗")); // lint failed
    }

    #[test]
    fn test_iteration_summary_render_expanded() {
        let gates = vec![
            GateSummary::new("build", true).with_duration(Duration::from_secs_f64(1.0)),
            GateSummary::new("test", false)
                .with_duration(Duration::from_secs_f64(0.5))
                .with_error("test failed: expected 1, got 2"),
        ];
        let summary =
            IterationSummary::new(1, 5, false, Duration::from_secs_f64(1.5)).with_gates(gates);
        let output = summary.render_expanded();

        assert!(output.contains("▾")); // expanded indicator
        assert!(output.contains("Iteration"));
        assert!(output.contains("build"));
        assert!(output.contains("test"));
        assert!(output.contains("test failed")); // error details
    }

    #[test]
    fn test_iteration_summary_render_auto_collapse_passed() {
        let gates = vec![GateSummary::new("build", true)];
        let summary =
            IterationSummary::new(1, 5, true, Duration::from_secs_f64(1.0)).with_gates(gates);
        let output = summary.render();

        // Passed iteration should be collapsed (single line with newline)
        assert!(output.contains("▸"));
        assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn test_iteration_summary_render_auto_expand_failed() {
        let gates = vec![GateSummary::new("build", false).with_error("compilation error")];
        let summary =
            IterationSummary::new(1, 5, false, Duration::from_secs_f64(1.0)).with_gates(gates);
        let output = summary.render();

        // Failed iteration should be expanded (multiple lines)
        assert!(output.contains("▾"));
        assert!(output.lines().count() > 1);
    }

    // ========================================================================
    // IterationSummaryStack Tests
    // ========================================================================

    #[test]
    fn test_iteration_summary_stack_new() {
        let stack = IterationSummaryStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_iteration_summary_stack_with_theme() {
        let stack = IterationSummaryStack::with_theme(Theme::default());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_iteration_summary_stack_default() {
        let stack = IterationSummaryStack::default();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_iteration_summary_stack_push() {
        let mut stack = IterationSummaryStack::new();
        let summary = IterationSummary::new(1, 5, true, Duration::from_secs(1));
        stack.push(summary);

        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_iteration_summary_stack_all_passed() {
        let mut stack = IterationSummaryStack::new();
        stack.push(IterationSummary::new(1, 3, true, Duration::from_secs(1)));
        stack.push(IterationSummary::new(2, 3, true, Duration::from_secs(2)));
        stack.push(IterationSummary::new(3, 3, true, Duration::from_secs(1)));

        assert!(stack.all_passed());
    }

    #[test]
    fn test_iteration_summary_stack_not_all_passed() {
        let mut stack = IterationSummaryStack::new();
        stack.push(IterationSummary::new(1, 3, true, Duration::from_secs(1)));
        stack.push(IterationSummary::new(2, 3, false, Duration::from_secs(2)));
        stack.push(IterationSummary::new(3, 3, true, Duration::from_secs(1)));

        assert!(!stack.all_passed());
    }

    #[test]
    fn test_iteration_summary_stack_total_duration() {
        let mut stack = IterationSummaryStack::new();
        stack.push(IterationSummary::new(1, 3, true, Duration::from_secs(1)));
        stack.push(IterationSummary::new(2, 3, true, Duration::from_secs(2)));

        assert_eq!(stack.total_duration(), Duration::from_secs(3));
    }

    #[test]
    fn test_iteration_summary_stack_format_total_duration() {
        let mut stack = IterationSummaryStack::new();
        stack.push(IterationSummary::new(
            1,
            3,
            true,
            Duration::from_secs_f64(1.5),
        ));
        stack.push(IterationSummary::new(
            2,
            3,
            true,
            Duration::from_secs_f64(2.0),
        ));

        assert_eq!(stack.format_total_duration(), "3.5s");
    }

    #[test]
    fn test_iteration_summary_stack_summaries() {
        let mut stack = IterationSummaryStack::new();
        stack.push(IterationSummary::new(1, 3, true, Duration::from_secs(1)));
        stack.push(IterationSummary::new(2, 3, true, Duration::from_secs(2)));

        let summaries = stack.summaries();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].iteration(), 1);
        assert_eq!(summaries[1].iteration(), 2);
    }

    #[test]
    fn test_iteration_summary_stack_render_empty() {
        let stack = IterationSummaryStack::new();
        assert_eq!(stack.render(), "");
    }

    #[test]
    fn test_iteration_summary_stack_render() {
        let mut stack = IterationSummaryStack::new();

        let gates1 = vec![GateSummary::new("build", true)];
        stack.push(IterationSummary::new(1, 3, true, Duration::from_secs(1)).with_gates(gates1));

        let gates2 = vec![GateSummary::new("build", true)];
        stack.push(IterationSummary::new(2, 3, true, Duration::from_secs(2)).with_gates(gates2));

        let output = stack.render();
        assert!(output.contains("Iteration"));
        assert!(output.contains("1"));
        assert!(output.contains("2"));
    }

    #[test]
    fn test_iteration_summary_stack_render_final_summary_empty() {
        let stack = IterationSummaryStack::new();
        assert_eq!(stack.render_final_summary(), "");
    }

    #[test]
    fn test_iteration_summary_stack_render_final_summary_all_passed() {
        let mut stack = IterationSummaryStack::new();
        stack.push(IterationSummary::new(1, 2, true, Duration::from_secs(1)));
        stack.push(IterationSummary::new(2, 2, true, Duration::from_secs(2)));

        let output = stack.render_final_summary();
        assert!(output.contains("All iterations passed"));
        assert!(output.contains("total"));
    }

    #[test]
    fn test_iteration_summary_stack_render_final_summary_some_failed() {
        let mut stack = IterationSummaryStack::new();
        stack.push(IterationSummary::new(1, 3, true, Duration::from_secs(1)));
        stack.push(IterationSummary::new(2, 3, false, Duration::from_secs(2)));
        stack.push(IterationSummary::new(3, 3, true, Duration::from_secs(1)));

        let output = stack.render_final_summary();
        assert!(output.contains("2"));
        assert!(output.contains("3"));
        assert!(output.contains("iterations passed"));
    }

    // ========================================================================
    // ActivityIndicator Tests
    // ========================================================================

    #[test]
    fn test_activity_indicator_new() {
        let activity = ActivityIndicator::new("Running:", "src/main.rs");
        assert_eq!(activity.prefix(), "Running:");
        assert_eq!(activity.target(), "src/main.rs");
        assert!(activity.line_number().is_none());
    }

    #[test]
    fn test_activity_indicator_with_line() {
        let activity = ActivityIndicator::with_line("Running:", "src/tests/auth.rs", 142);
        assert_eq!(activity.prefix(), "Running:");
        assert_eq!(activity.target(), "src/tests/auth.rs");
        assert_eq!(activity.line_number(), Some(142));
    }

    #[test]
    fn test_activity_indicator_running_file() {
        let activity = ActivityIndicator::running_file("src/lib.rs");
        assert_eq!(activity.prefix(), "Running:");
        assert_eq!(activity.target(), "src/lib.rs");
    }

    #[test]
    fn test_activity_indicator_running_file_at_line() {
        let activity = ActivityIndicator::running_file_at_line("src/lib.rs", 42);
        assert_eq!(activity.prefix(), "Running:");
        assert_eq!(activity.target(), "src/lib.rs");
        assert_eq!(activity.line_number(), Some(42));
    }

    #[test]
    fn test_activity_indicator_testing() {
        let activity = ActivityIndicator::testing("test_something");
        assert_eq!(activity.prefix(), "Testing:");
        assert_eq!(activity.target(), "test_something");
    }

    #[test]
    fn test_activity_indicator_compiling() {
        let activity = ActivityIndicator::compiling("my_crate");
        assert_eq!(activity.prefix(), "Compiling:");
        assert_eq!(activity.target(), "my_crate");
    }

    #[test]
    fn test_activity_indicator_linting() {
        let activity = ActivityIndicator::linting("src/main.rs");
        assert_eq!(activity.prefix(), "Linting:");
        assert_eq!(activity.target(), "src/main.rs");
    }

    #[test]
    fn test_activity_indicator_is_file_activity() {
        assert!(ActivityIndicator::new("Running:", "src/main.rs").is_file_activity());
        assert!(ActivityIndicator::new("Running:", "tests/test.rs").is_file_activity());
        assert!(ActivityIndicator::new("Running:", "/absolute/path.rs").is_file_activity());
        assert!(ActivityIndicator::new("Running:", "file.ts").is_file_activity());
        assert!(ActivityIndicator::new("Running:", "file.js").is_file_activity());
        assert!(ActivityIndicator::new("Running:", "file.py").is_file_activity());
        assert!(!ActivityIndicator::new("Running:", "my_crate").is_file_activity());
        assert!(!ActivityIndicator::new("Running:", "test_something").is_file_activity());
    }

    #[test]
    fn test_activity_indicator_render_plain_without_line() {
        let activity = ActivityIndicator::new("Running:", "src/main.rs");
        let theme = Theme::default();
        let output = activity.render_plain(&theme);
        assert!(output.contains("Running:"));
        assert!(output.contains("src/main.rs"));
    }

    #[test]
    fn test_activity_indicator_render_plain_with_line() {
        let activity = ActivityIndicator::with_line("Running:", "src/tests/auth.rs", 142);
        let theme = Theme::default();
        let output = activity.render_plain(&theme);
        assert!(output.contains("Running:"));
        assert!(output.contains("src/tests/auth.rs:142"));
    }

    #[test]
    fn test_activity_indicator_render_with_hyperlinks() {
        let activity = ActivityIndicator::with_line("Running:", "src/tests/auth.rs", 142);
        let theme = Theme::default();
        let caps = TerminalCapabilities::all_enabled();
        let output = activity.render(&theme, &caps);
        assert!(output.contains("Running:"));
        // Should have hyperlink escape codes
        assert!(output.contains("\x1b]8;;"));
    }

    #[test]
    fn test_activity_indicator_render_without_hyperlinks() {
        let activity = ActivityIndicator::with_line("Running:", "src/tests/auth.rs", 142);
        let theme = Theme::default();
        let caps = TerminalCapabilities::minimal();
        let output = activity.render(&theme, &caps);
        assert!(output.contains("Running:"));
        // Should not have hyperlink escape codes
        assert!(!output.contains("\x1b]8;;"));
    }

    #[test]
    fn test_activity_indicator_equality() {
        let a1 = ActivityIndicator::new("Running:", "src/main.rs");
        let a2 = ActivityIndicator::new("Running:", "src/main.rs");
        let a3 = ActivityIndicator::new("Testing:", "src/main.rs");
        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
    }

    // ========================================================================
    // LiveIterationPanel Activity Tests
    // ========================================================================

    #[test]
    fn test_live_iteration_panel_set_activity() {
        let gates = vec!["build".to_string()];
        let mut panel = LiveIterationPanel::new(1, 1, gates);

        assert!(panel.activity().is_none());

        let activity = ActivityIndicator::running_file("src/main.rs");
        panel.set_activity(activity.clone());

        assert!(panel.activity().is_some());
        assert_eq!(panel.activity().unwrap().target(), "src/main.rs");
    }

    #[test]
    fn test_live_iteration_panel_clear_activity() {
        let gates = vec!["build".to_string()];
        let mut panel = LiveIterationPanel::new(1, 1, gates);

        let activity = ActivityIndicator::running_file("src/main.rs");
        panel.set_activity(activity);
        assert!(panel.activity().is_some());

        panel.clear_activity();
        assert!(panel.activity().is_none());
    }

    #[test]
    fn test_live_iteration_panel_pass_gate_clears_activity() {
        let gates = vec!["build".to_string()];
        let mut panel = LiveIterationPanel::new(1, 1, gates);

        panel.start_gate("build");
        panel.set_activity(ActivityIndicator::running_file("src/main.rs"));
        assert!(panel.activity().is_some());

        panel.pass_gate("build", Duration::from_secs(1));
        assert!(panel.activity().is_none());
    }

    #[test]
    fn test_live_iteration_panel_fail_gate_clears_activity() {
        let gates = vec!["build".to_string()];
        let mut panel = LiveIterationPanel::new(1, 1, gates);

        panel.start_gate("build");
        panel.set_activity(ActivityIndicator::running_file("src/main.rs"));
        assert!(panel.activity().is_some());

        panel.fail_gate("build", Duration::from_secs(1));
        assert!(panel.activity().is_none());
    }

    #[test]
    fn test_live_iteration_panel_render_with_activity() {
        let gates = vec!["build".to_string(), "lint".to_string()];
        let mut panel =
            LiveIterationPanel::with_capabilities(1, 3, gates, TerminalCapabilities::minimal());

        panel.start_gate("build");
        panel.set_activity(ActivityIndicator::running_file_at_line(
            "src/tests/auth.rs",
            142,
        ));

        let output = panel.render();

        // Should contain the activity beneath the running gate
        assert!(output.contains("build"));
        assert!(output.contains("Running:"));
        assert!(output.contains("src/tests/auth.rs"));
    }

    #[test]
    fn test_live_iteration_panel_render_no_activity_when_not_running() {
        let gates = vec!["build".to_string()];
        let mut panel =
            LiveIterationPanel::with_capabilities(1, 1, gates, TerminalCapabilities::minimal());

        // Set activity but don't start the gate (should not show)
        panel.set_activity(ActivityIndicator::running_file("src/main.rs"));

        let output = panel.render();

        // Activity should not appear since no gate is running
        assert!(output.contains("build"));
        assert!(!output.contains("Running:"));
    }
}
