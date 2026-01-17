//! Quality gate visualization for Ralph's terminal UI.
//!
//! Displays quality gate status with pass/fail indicators,
//! error details, and summary statistics.

#![allow(dead_code)]

use owo_colors::OwoColorize;

use crate::quality::gates::GateResult;
use crate::ui::colors::Theme;

/// Status of a quality gate in the visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateStatus {
    /// Gate is waiting to run
    Pending,
    /// Gate is currently running
    Running,
    /// Gate passed successfully
    Passed,
    /// Gate failed
    Failed,
    /// Gate was skipped (not enabled in profile)
    Skipped,
}

impl GateStatus {
    /// Get the status icon for this gate state.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Running => "◉",
            Self::Passed => "✓",
            Self::Failed => "✗",
            Self::Skipped => "⊘",
        }
    }

    /// Get the status label for this gate state.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Passed => "Passed",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
        }
    }

    /// Create a GateStatus from a GateResult.
    pub fn from_gate_result(result: &GateResult) -> Self {
        if result.message.contains("Skipped") {
            Self::Skipped
        } else if result.passed {
            Self::Passed
        } else {
            Self::Failed
        }
    }
}

/// View model for a quality gate in the visualization.
#[derive(Debug, Clone)]
pub struct QualityGateView {
    /// Name of the quality gate
    pub name: String,
    /// Current status of the gate
    pub status: GateStatus,
    /// Human-readable message about the result
    pub message: String,
    /// Optional error details (shown for failed gates)
    pub details: Option<String>,
}

impl QualityGateView {
    /// Create a new quality gate view.
    pub fn new(name: impl Into<String>, status: GateStatus, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status,
            message: message.into(),
            details: None,
        }
    }

    /// Create a quality gate view with error details.
    pub fn with_details(
        name: impl Into<String>,
        status: GateStatus,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            status,
            message: message.into(),
            details: Some(details.into()),
        }
    }

    /// Create a QualityGateView from a GateResult.
    pub fn from_gate_result(result: &GateResult) -> Self {
        Self {
            name: result.gate_name.clone(),
            status: GateStatus::from_gate_result(result),
            message: result.message.clone(),
            details: result.details.clone(),
        }
    }

    /// Create a pending gate view.
    pub fn pending(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: GateStatus::Pending,
            message: "Waiting to run...".to_string(),
            details: None,
        }
    }

    /// Create a running gate view.
    pub fn running(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: GateStatus::Running,
            message: "Running...".to_string(),
            details: None,
        }
    }
}

/// Renders quality gate status panels to the terminal.
#[derive(Debug)]
pub struct QualityGateRenderer {
    /// Color theme for rendering
    theme: Theme,
    /// Panel width (characters)
    width: usize,
}

impl Default for QualityGateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityGateRenderer {
    /// Create a new quality gate renderer with default settings.
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            width: 60,
        }
    }

    /// Create a renderer with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self { theme, width: 60 }
    }

    /// Set the panel width.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Render a single gate with status icon and message.
    pub fn render_gate(&self, gate: &QualityGateView) -> String {
        let mut output = String::new();

        // Format: [icon] gate_name: message
        let icon = gate.status.icon();
        let icon_colored = match gate.status {
            GateStatus::Pending => icon.color(self.theme.muted).to_string(),
            GateStatus::Running => icon.color(self.theme.in_progress).to_string(),
            GateStatus::Passed => icon.color(self.theme.success).to_string(),
            GateStatus::Failed => icon.color(self.theme.error).to_string(),
            GateStatus::Skipped => icon.color(self.theme.muted).to_string(),
        };

        // Gate name with proper capitalization
        let gate_name = Self::format_gate_name(&gate.name);

        // Truncate message if needed
        let max_msg_len = self.width.saturating_sub(gate_name.len() + 5);
        let message = self.truncate_text(&gate.message, max_msg_len);

        output.push_str(&format!("{} {}: {}\n", icon_colored, gate_name, message));

        // Show details for failed gates (indented)
        if gate.status == GateStatus::Failed {
            if let Some(ref details) = gate.details {
                let detail_lines: Vec<&str> = details.lines().take(10).collect();
                for line in detail_lines {
                    let indented = format!("    {}", line);
                    let truncated = self.truncate_text(&indented, self.width - 2);
                    output.push_str(&format!("{}\n", truncated.color(self.theme.error)));
                }
                if details.lines().count() > 10 {
                    output.push_str(&format!(
                        "{}\n",
                        "    ... (truncated)".color(self.theme.muted)
                    ));
                }
            }
        }

        output
    }

    /// Render a list of quality gates.
    pub fn render_gates(&self, gates: &[QualityGateView]) -> String {
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Header
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));
        output.push_str(&format!(
            "│ {:<width$} │\n",
            "Quality Gates",
            width = inner_width - 2
        ));
        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Render each gate
        for gate in gates {
            let gate_output = self.render_gate(gate);
            for line in gate_output.lines() {
                // Pad and wrap in box
                let visible_len = Self::visible_length(line);
                let padding = if inner_width > visible_len + 2 {
                    inner_width - visible_len - 2
                } else {
                    0
                };
                output.push_str(&format!("│ {}{} │\n", line, " ".repeat(padding)));
            }
        }

        // Summary bar
        let (passed, total, failed_names) = Self::count_results(gates);
        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        let summary = if failed_names.is_empty() {
            format!("All {} gates passed", total)
                .color(self.theme.success)
                .to_string()
        } else {
            format!("{}/{} gates passed", passed, total)
                .color(self.theme.warning)
                .to_string()
        };

        let summary_visible_len = if failed_names.is_empty() {
            format!("All {} gates passed", total).len()
        } else {
            format!("{}/{} gates passed", passed, total).len()
        };
        let summary_padding = if inner_width > summary_visible_len + 2 {
            inner_width - summary_visible_len - 2
        } else {
            0
        };

        output.push_str(&format!("│ {}{} │\n", summary, " ".repeat(summary_padding)));

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Render a compact summary bar only.
    pub fn render_summary_bar(&self, gates: &[QualityGateView]) -> String {
        let (passed, total, failed_names) = Self::count_results(gates);

        if failed_names.is_empty() {
            format!(
                "{} {} All {} gates passed",
                "✓".color(self.theme.success),
                "│".color(self.theme.muted),
                total
            )
        } else {
            format!(
                "{} {} {}/{} gates passed. Failed: {}",
                "✗".color(self.theme.error),
                "│".color(self.theme.muted),
                passed,
                total,
                failed_names.join(", ")
            )
        }
    }

    /// Render gates from GateResult slice directly.
    pub fn render_from_results(&self, results: &[GateResult]) -> String {
        let gates: Vec<QualityGateView> = results
            .iter()
            .map(QualityGateView::from_gate_result)
            .collect();
        self.render_gates(&gates)
    }

    /// Format gate name for display (capitalize and replace underscores).
    fn format_gate_name(name: &str) -> String {
        name.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Truncate text to fit within max length, adding ellipsis if needed.
    fn truncate_text(&self, text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else if max_len > 3 {
            format!("{}...", &text[..max_len - 3])
        } else {
            text[..max_len].to_string()
        }
    }

    /// Calculate the visible length of a string (excluding ANSI escape codes).
    fn visible_length(s: &str) -> usize {
        // Simple ANSI escape code stripping
        let mut in_escape = false;
        let mut len = 0;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c == 'm' {
                    in_escape = false;
                }
            } else {
                len += 1;
            }
        }
        len
    }

    /// Count passed, total, and get names of failed gates.
    fn count_results(gates: &[QualityGateView]) -> (usize, usize, Vec<String>) {
        let passed = gates
            .iter()
            .filter(|g| g.status == GateStatus::Passed || g.status == GateStatus::Skipped)
            .count();
        let total = gates.len();
        let failed_names: Vec<String> = gates
            .iter()
            .filter(|g| g.status == GateStatus::Failed)
            .map(|g| g.name.clone())
            .collect();

        (passed, total, failed_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_status_icons() {
        assert_eq!(GateStatus::Pending.icon(), "○");
        assert_eq!(GateStatus::Running.icon(), "◉");
        assert_eq!(GateStatus::Passed.icon(), "✓");
        assert_eq!(GateStatus::Failed.icon(), "✗");
        assert_eq!(GateStatus::Skipped.icon(), "⊘");
    }

    #[test]
    fn test_gate_status_labels() {
        assert_eq!(GateStatus::Pending.label(), "Pending");
        assert_eq!(GateStatus::Running.label(), "Running");
        assert_eq!(GateStatus::Passed.label(), "Passed");
        assert_eq!(GateStatus::Failed.label(), "Failed");
        assert_eq!(GateStatus::Skipped.label(), "Skipped");
    }

    #[test]
    fn test_quality_gate_view_new() {
        let gate = QualityGateView::new("lint", GateStatus::Passed, "No warnings");
        assert_eq!(gate.name, "lint");
        assert_eq!(gate.status, GateStatus::Passed);
        assert_eq!(gate.message, "No warnings");
        assert!(gate.details.is_none());
    }

    #[test]
    fn test_quality_gate_view_with_details() {
        let gate = QualityGateView::with_details(
            "coverage",
            GateStatus::Failed,
            "Coverage too low",
            "60% < 80% threshold",
        );
        assert_eq!(gate.name, "coverage");
        assert_eq!(gate.status, GateStatus::Failed);
        assert!(gate.details.is_some());
        assert!(gate.details.unwrap().contains("60%"));
    }

    #[test]
    fn test_quality_gate_view_pending() {
        let gate = QualityGateView::pending("format");
        assert_eq!(gate.name, "format");
        assert_eq!(gate.status, GateStatus::Pending);
        assert!(gate.message.contains("Waiting"));
    }

    #[test]
    fn test_quality_gate_view_running() {
        let gate = QualityGateView::running("security_audit");
        assert_eq!(gate.name, "security_audit");
        assert_eq!(gate.status, GateStatus::Running);
        assert!(gate.message.contains("Running"));
    }

    #[test]
    fn test_from_gate_result_passed() {
        let result = GateResult::pass("lint", "No warnings found");
        let view = QualityGateView::from_gate_result(&result);
        assert_eq!(view.status, GateStatus::Passed);
    }

    #[test]
    fn test_from_gate_result_failed() {
        let result = GateResult::fail("coverage", "Below threshold", Some("Details".to_string()));
        let view = QualityGateView::from_gate_result(&result);
        assert_eq!(view.status, GateStatus::Failed);
        assert!(view.details.is_some());
    }

    #[test]
    fn test_from_gate_result_skipped() {
        let result = GateResult::skipped("security_audit", "Not enabled");
        let view = QualityGateView::from_gate_result(&result);
        assert_eq!(view.status, GateStatus::Skipped);
    }

    #[test]
    fn test_format_gate_name() {
        assert_eq!(
            QualityGateRenderer::format_gate_name("security_audit"),
            "Security Audit"
        );
        assert_eq!(QualityGateRenderer::format_gate_name("lint"), "Lint");
        assert_eq!(QualityGateRenderer::format_gate_name("format"), "Format");
        assert_eq!(
            QualityGateRenderer::format_gate_name("code_coverage_check"),
            "Code Coverage Check"
        );
    }

    #[test]
    fn test_render_gate_passed() {
        let renderer = QualityGateRenderer::new();
        let gate = QualityGateView::new("lint", GateStatus::Passed, "No warnings");
        let output = renderer.render_gate(&gate);

        assert!(output.contains("Lint"));
        assert!(output.contains("No warnings"));
    }

    #[test]
    fn test_render_gate_failed_with_details() {
        let renderer = QualityGateRenderer::new();
        let gate = QualityGateView::with_details(
            "coverage",
            GateStatus::Failed,
            "Coverage too low",
            "Line 1\nLine 2",
        );
        let output = renderer.render_gate(&gate);

        assert!(output.contains("Coverage"));
        assert!(output.contains("Coverage too low"));
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
    }

    #[test]
    fn test_render_gates() {
        let renderer = QualityGateRenderer::new().with_width(50);
        let gates = vec![
            QualityGateView::new("lint", GateStatus::Passed, "No warnings"),
            QualityGateView::new("format", GateStatus::Passed, "All formatted"),
            QualityGateView::new("coverage", GateStatus::Failed, "Below threshold"),
        ];

        let output = renderer.render_gates(&gates);

        // Check box drawing characters
        assert!(output.contains("╭"));
        assert!(output.contains("╯"));
        assert!(output.contains("Quality Gates"));

        // Check gates are rendered
        assert!(output.contains("Lint"));
        assert!(output.contains("Format"));
        assert!(output.contains("Coverage"));

        // Check summary
        assert!(output.contains("2/3 gates passed"));
    }

    #[test]
    fn test_render_gates_all_passed() {
        let renderer = QualityGateRenderer::new();
        let gates = vec![
            QualityGateView::new("lint", GateStatus::Passed, "No warnings"),
            QualityGateView::new("format", GateStatus::Passed, "All formatted"),
        ];

        let output = renderer.render_gates(&gates);
        assert!(output.contains("All 2 gates passed"));
    }

    #[test]
    fn test_render_summary_bar_all_passed() {
        let renderer = QualityGateRenderer::new();
        let gates = vec![
            QualityGateView::new("lint", GateStatus::Passed, "OK"),
            QualityGateView::new("format", GateStatus::Passed, "OK"),
        ];

        let output = renderer.render_summary_bar(&gates);
        assert!(output.contains("All 2 gates passed"));
    }

    #[test]
    fn test_render_summary_bar_with_failures() {
        let renderer = QualityGateRenderer::new();
        let gates = vec![
            QualityGateView::new("lint", GateStatus::Passed, "OK"),
            QualityGateView::new("coverage", GateStatus::Failed, "Low"),
        ];

        let output = renderer.render_summary_bar(&gates);
        assert!(output.contains("1/2 gates passed"));
        assert!(output.contains("coverage"));
    }

    #[test]
    fn test_visible_length() {
        // Plain text
        assert_eq!(QualityGateRenderer::visible_length("hello"), 5);

        // Text with ANSI codes (simulated)
        assert_eq!(
            QualityGateRenderer::visible_length("\x1b[32mhello\x1b[0m"),
            5
        );
    }

    #[test]
    fn test_count_results() {
        let gates = vec![
            QualityGateView::new("a", GateStatus::Passed, "OK"),
            QualityGateView::new("b", GateStatus::Failed, "Fail"),
            QualityGateView::new("c", GateStatus::Skipped, "Skip"),
            QualityGateView::new("d", GateStatus::Failed, "Fail"),
        ];

        let (passed, total, failed_names) = QualityGateRenderer::count_results(&gates);
        assert_eq!(passed, 2); // Passed + Skipped
        assert_eq!(total, 4);
        assert_eq!(failed_names.len(), 2);
        assert!(failed_names.contains(&"b".to_string()));
        assert!(failed_names.contains(&"d".to_string()));
    }
}
