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
use crate::ui::ghostty::{self, TerminalCapabilities};
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
            return format!(
                "{} No gates configured\n",
                "○".color(self.theme.muted)
            );
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
            Self::Pending => "○",  // Empty circle
            Self::Running => "◐",  // Half-filled circle (will animate)
            Self::Passed => "✓",   // Checkmark
            Self::Failed => "✗",   // X mark
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
                format!("{}m{:.1}s", d.as_secs() / 60, (d.as_millis() % 60000) as f64 / 1000.0)
            } else {
                format!("{:.1}s", d.as_secs_f64())
            }
        })
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
    pub fn pass_gate(&mut self, name: &str, duration: Duration) {
        if let Some(&idx) = self.gate_indices.get(name) {
            self.gates[idx].pass(duration);
        }
    }

    /// Mark a gate as failed.
    pub fn fail_gate(&mut self, name: &str, duration: Duration) {
        if let Some(&idx) = self.gate_indices.get(name) {
            self.gates[idx].fail(duration);
        }
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
        self.gates.iter().any(|g| g.progress == GateProgress::Failed)
    }

    /// Render a single gate progress info.
    fn render_gate(&self, gate: &GateProgressInfo) -> String {
        let indicator = match gate.progress {
            GateProgress::Pending => format!("{}", "○".color(self.theme.muted)),
            GateProgress::Running => format!("{}", self.spinner_char().color(self.theme.in_progress)),
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
            format!("{} {} ({})", indicator, name, duration_str.color(self.theme.muted))
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
                    GateProgress::Running => format!("{}", self.spinner_char().color(self.theme.in_progress)),
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
}
