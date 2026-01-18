//! Custom ratatui widgets for Ralph's terminal UI.
//!
//! Provides rich visual components for:
//! - Story headers with priority badges
//! - Iteration progress with gate chain
//! - Completion summaries with git info

#![allow(dead_code)]

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use super::animation::AnimationState;

// ============================================================================
// Color Constants
// ============================================================================

mod colors {
    use ratatui::style::Color;

    pub const CYAN: Color = Color::Rgb(34, 211, 238);
    pub const GREEN: Color = Color::Rgb(34, 197, 94);
    pub const YELLOW: Color = Color::Rgb(234, 179, 8);
    pub const RED: Color = Color::Rgb(239, 68, 68);
    pub const BLUE: Color = Color::Rgb(59, 130, 246);
    pub const GRAY: Color = Color::Rgb(107, 114, 128);
    pub const MUTED: Color = Color::Rgb(75, 85, 99);
    pub const WHITE: Color = Color::Rgb(255, 255, 255);
}

// ============================================================================
// Story Header Widget
// ============================================================================

/// Enhanced story header widget with box drawing and priority badge.
#[derive(Debug, Clone)]
pub struct StoryHeaderWidget {
    /// Story ID
    story_id: String,
    /// Story title
    title: String,
    /// Priority (1-5)
    priority: u32,
    /// Whether the story is complete
    complete: bool,
}

impl StoryHeaderWidget {
    /// Create a new story header widget.
    pub fn new(story_id: impl Into<String>, title: impl Into<String>, priority: u32) -> Self {
        Self {
            story_id: story_id.into(),
            title: title.into(),
            priority,
            complete: false,
        }
    }

    /// Mark as complete.
    pub fn complete(mut self, complete: bool) -> Self {
        self.complete = complete;
        self
    }

    /// Render as a formatted string.
    pub fn render_string(&self, width: usize) -> String {
        let mut output = String::new();
        let inner = width.saturating_sub(2);

        // Top border
        output.push_str(&format!("╔{}╗\n", "═".repeat(inner)));

        // Header line: 📖 Story: US-001 - Title [P1]
        let emoji = "📖";
        let priority_badge = format!("[P{}]", self.priority);
        let header_base = format!("{} Story: {} - ", emoji, self.story_id);
        let available = inner.saturating_sub(header_base.len() + priority_badge.len() + 2);
        let title = if self.title.len() > available && available > 3 {
            format!("{}...", &self.title[..available.saturating_sub(3)])
        } else {
            self.title.clone()
        };

        let content = format!("{}{}", header_base, title);
        let padding = inner.saturating_sub(content.len() + priority_badge.len() + 1);

        output.push_str(&format!(
            "║ {}{}{} ║\n",
            content,
            " ".repeat(padding),
            priority_badge
        ));

        // Bottom border
        output.push_str(&format!("╚{}╝", "═".repeat(inner)));

        output
    }
}

impl Widget for StoryHeaderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 3 {
            return;
        }

        let content = self.render_string(area.width as usize);
        let lines: Vec<Line> = content
            .lines()
            .map(|s| {
                Line::from(Span::styled(
                    s.to_string(),
                    Style::default().fg(colors::CYAN),
                ))
            })
            .collect();
        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }
}

// ============================================================================
// Gate Chain Widget
// ============================================================================

/// Gate status for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateStatus {
    /// Gate is pending
    Pending,
    /// Gate is running
    Running,
    /// Gate passed
    Passed,
    /// Gate failed
    Failed,
    /// Gate was skipped
    Skipped,
}

impl GateStatus {
    /// Get the icon for this status.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Running => "◐",
            Self::Passed => "✓",
            Self::Failed => "✗",
            Self::Skipped => "⊘",
        }
    }

    /// Get the color for this status.
    pub fn color(&self) -> Color {
        match self {
            Self::Pending => colors::GRAY,
            Self::Running => colors::BLUE,
            Self::Passed => colors::GREEN,
            Self::Failed => colors::RED,
            Self::Skipped => colors::MUTED,
        }
    }
}

/// A gate in the chain.
#[derive(Debug, Clone)]
pub struct GateInfo {
    /// Gate name
    pub name: String,
    /// Gate status
    pub status: GateStatus,
    /// Optional duration in seconds
    pub duration: Option<f64>,
}

impl GateInfo {
    /// Create a new gate info.
    pub fn new(name: impl Into<String>, status: GateStatus) -> Self {
        Self {
            name: name.into(),
            status,
            duration: None,
        }
    }

    /// Set the duration.
    pub fn with_duration(mut self, seconds: f64) -> Self {
        self.duration = Some(seconds);
        self
    }
}

/// Widget showing gate chain progress.
#[derive(Debug, Clone)]
pub struct GateChainWidget {
    /// Gates in the chain
    gates: Vec<GateInfo>,
    /// Animation state for running indicators
    animation: Option<AnimationState>,
}

impl GateChainWidget {
    /// Create a new gate chain widget.
    pub fn new(gates: Vec<GateInfo>) -> Self {
        Self {
            gates,
            animation: None,
        }
    }

    /// Set animation state.
    pub fn with_animation(mut self, animation: AnimationState) -> Self {
        self.animation = Some(animation);
        self
    }

    /// Render as a formatted string.
    pub fn render_string(&self) -> String {
        self.gates
            .iter()
            .map(|g| {
                let icon = g.status.icon();
                let duration = g
                    .duration
                    .map(|d| format!(" ({:.1}s)", d))
                    .unwrap_or_default();
                format!("{} {}{}", g.name, icon, duration)
            })
            .collect::<Vec<_>>()
            .join(" → ")
    }
}

impl Widget for GateChainWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 1 {
            return;
        }

        let content = self.render_string();
        let paragraph = Paragraph::new(content);
        paragraph.render(area, buf);
    }
}

// ============================================================================
// Iteration Widget
// ============================================================================

/// Widget showing iteration progress.
#[derive(Debug, Clone)]
pub struct IterationWidget {
    /// Current iteration number
    current: u32,
    /// Maximum iterations
    max: u32,
    /// Gates in this iteration
    gates: Vec<GateInfo>,
    /// Animation state
    animation: Option<AnimationState>,
}

impl IterationWidget {
    /// Create a new iteration widget.
    pub fn new(current: u32, max: u32) -> Self {
        Self {
            current,
            max,
            gates: Vec::new(),
            animation: None,
        }
    }

    /// Set the gates.
    pub fn with_gates(mut self, gates: Vec<GateInfo>) -> Self {
        self.gates = gates;
        self
    }

    /// Set animation state.
    pub fn with_animation(mut self, animation: AnimationState) -> Self {
        self.animation = Some(animation);
        self
    }

    /// Render as a formatted string.
    pub fn render_string(&self) -> String {
        let spinner = self
            .animation
            .as_ref()
            .map(|a| {
                let chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                chars[(a.frame as usize) % chars.len()]
            })
            .unwrap_or("⟳");

        let bar_width = 15;
        let filled = if self.max > 0 {
            (self.current as f64 / self.max as f64 * bar_width as f64) as usize
        } else {
            0
        };
        let empty = bar_width - filled;

        let mut output = format!(
            "{} Iteration [{}/{}] {}{}",
            spinner,
            self.current,
            self.max,
            "█".repeat(filled),
            "░".repeat(empty),
        );

        if !self.gates.is_empty() {
            let gate_chain = GateChainWidget::new(self.gates.clone());
            output.push_str(&format!(
                "\n         └── Running: {}",
                gate_chain.render_string()
            ));
        }

        output
    }
}

impl Widget for IterationWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height < 1 {
            return;
        }

        let content = self.render_string();
        let lines: Vec<Line> = content.lines().map(|s| Line::from(s.to_string())).collect();
        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }
}

// ============================================================================
// Completion Summary Widget
// ============================================================================

/// Git information for completion summary.
#[derive(Debug, Clone, Default)]
pub struct GitSummary {
    /// Branch name
    pub branch: Option<String>,
    /// Commit hash (short)
    pub commit: Option<String>,
    /// Files changed
    pub files_changed: u32,
    /// Lines added
    pub lines_added: u32,
    /// Lines deleted
    pub lines_deleted: u32,
}

impl GitSummary {
    /// Create a new git summary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set branch.
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// Set commit.
    pub fn with_commit(mut self, commit: impl Into<String>) -> Self {
        self.commit = Some(commit.into());
        self
    }

    /// Set file changes.
    pub fn with_changes(mut self, files: u32, added: u32, deleted: u32) -> Self {
        self.files_changed = files;
        self.lines_added = added;
        self.lines_deleted = deleted;
        self
    }
}

/// Completion summary widget with rich formatting.
#[derive(Debug, Clone)]
pub struct CompletionSummaryWidget {
    /// Story ID
    story_id: String,
    /// Whether passed
    passed: bool,
    /// Duration in seconds
    duration_secs: f64,
    /// Iterations used
    iterations_used: u32,
    /// Max iterations
    max_iterations: u32,
    /// Gate results
    gates: Vec<GateInfo>,
    /// Git information
    git: GitSummary,
}

impl CompletionSummaryWidget {
    /// Create a new completion summary.
    pub fn new(
        story_id: impl Into<String>,
        passed: bool,
        duration_secs: f64,
        iterations_used: u32,
        max_iterations: u32,
    ) -> Self {
        Self {
            story_id: story_id.into(),
            passed,
            duration_secs,
            iterations_used,
            max_iterations,
            gates: Vec::new(),
            git: GitSummary::default(),
        }
    }

    /// Set gate results.
    pub fn with_gates(mut self, gates: Vec<GateInfo>) -> Self {
        self.gates = gates;
        self
    }

    /// Set git information.
    pub fn with_git(mut self, git: GitSummary) -> Self {
        self.git = git;
        self
    }

    /// Format duration as human readable.
    fn format_duration(&self) -> String {
        let secs = self.duration_secs as u64;
        let mins = secs / 60;
        let secs = secs % 60;
        if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// Render as formatted string.
    pub fn render_string(&self, width: usize) -> String {
        let mut output = String::new();
        let inner = width.saturating_sub(2);

        let (icon, status) = if self.passed {
            ("✅", "COMPLETE")
        } else {
            ("❌", "FAILED")
        };

        // Top border
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner)));

        // Header
        let header = format!("{} Story {} {}", icon, self.story_id, status);
        let padding = inner.saturating_sub(header.len() + 1);
        output.push_str(&format!("│ {}{} │\n", header, " ".repeat(padding)));

        // Separator
        output.push_str(&format!("├{}┤\n", "─".repeat(inner)));

        // Duration
        let dur_line = format!("Duration: {}", self.format_duration());
        let padding = inner.saturating_sub(dur_line.len() + 1);
        output.push_str(&format!("│ {}{} │\n", dur_line, " ".repeat(padding)));

        // Iterations
        let iter_line = format!(
            "Iterations: {}/{}",
            self.iterations_used, self.max_iterations
        );
        let padding = inner.saturating_sub(iter_line.len() + 1);
        output.push_str(&format!("│ {}{} │\n", iter_line, " ".repeat(padding)));

        // Gates
        if !self.gates.is_empty() {
            let gates_str = self
                .gates
                .iter()
                .map(|g| format!("{} {}", g.name, g.status.icon()))
                .collect::<Vec<_>>()
                .join(" | ");
            let gates_line = format!("Gates: {}", gates_str);
            let display_len = gates_line.len().min(inner);
            let padding = inner.saturating_sub(display_len + 1);
            output.push_str(&format!(
                "│ {}{} │\n",
                &gates_line[..display_len],
                " ".repeat(padding.min(inner))
            ));
        }

        // Git info
        if let Some(ref commit) = self.git.commit {
            let commit_line = format!("Commit: {}", commit);
            let padding = inner.saturating_sub(commit_line.len() + 1);
            output.push_str(&format!("│ {}{} │\n", commit_line, " ".repeat(padding)));
        }

        if self.git.files_changed > 0 {
            let files_line = format!(
                "Files: {} changed, +{}/-{}",
                self.git.files_changed, self.git.lines_added, self.git.lines_deleted
            );
            let padding = inner.saturating_sub(files_line.len() + 1);
            output.push_str(&format!("│ {}{} │\n", files_line, " ".repeat(padding)));
        }

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner)));

        output
    }
}

impl Widget for CompletionSummaryWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height < 6 {
            return;
        }

        let content = self.render_string(area.width as usize);
        let color = if self.passed {
            colors::GREEN
        } else {
            colors::RED
        };
        let lines: Vec<Line> = content
            .lines()
            .map(|s| Line::from(Span::styled(s.to_string(), Style::default().fg(color))))
            .collect();
        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_header_widget_new() {
        let widget = StoryHeaderWidget::new("US-001", "Test Story", 1);
        assert_eq!(widget.story_id, "US-001");
        assert_eq!(widget.title, "Test Story");
        assert_eq!(widget.priority, 1);
    }

    #[test]
    fn test_story_header_render_string() {
        let widget = StoryHeaderWidget::new("US-001", "Test Story", 1);
        let output = widget.render_string(50);
        assert!(output.contains("US-001"));
        assert!(output.contains("[P1]"));
        assert!(output.contains("╔"));
        assert!(output.contains("╝"));
    }

    #[test]
    fn test_gate_status_icon() {
        assert_eq!(GateStatus::Pending.icon(), "○");
        assert_eq!(GateStatus::Running.icon(), "◐");
        assert_eq!(GateStatus::Passed.icon(), "✓");
        assert_eq!(GateStatus::Failed.icon(), "✗");
        assert_eq!(GateStatus::Skipped.icon(), "⊘");
    }

    #[test]
    fn test_gate_chain_render_string() {
        let gates = vec![
            GateInfo::new("build", GateStatus::Passed),
            GateInfo::new("lint", GateStatus::Passed),
            GateInfo::new("test", GateStatus::Running),
        ];
        let widget = GateChainWidget::new(gates);
        let output = widget.render_string();
        assert!(output.contains("build"));
        assert!(output.contains("→"));
        assert!(output.contains("test"));
    }

    #[test]
    fn test_iteration_widget_render_string() {
        let widget = IterationWidget::new(3, 10);
        let output = widget.render_string();
        assert!(output.contains("3/10"));
        assert!(output.contains("█"));
    }

    #[test]
    fn test_completion_summary_render_string() {
        let widget = CompletionSummaryWidget::new("US-001", true, 154.0, 3, 10)
            .with_gates(vec![
                GateInfo::new("build", GateStatus::Passed),
                GateInfo::new("test", GateStatus::Passed),
            ])
            .with_git(
                GitSummary::new()
                    .with_commit("abc123f")
                    .with_changes(3, 212, 208),
            );

        let output = widget.render_string(50);
        assert!(output.contains("US-001"));
        assert!(output.contains("COMPLETE"));
        assert!(output.contains("abc123f"));
        assert!(output.contains("3 changed"));
    }
}
