//! Collapsible section component for terminal output.
//!
//! Provides expandable/collapsible sections for detailed information
//! like iteration summaries and streaming output.

#![allow(dead_code)]

use owo_colors::OwoColorize;

use crate::ui::colors::{ansi, Theme};

/// State of a collapsible section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CollapsibleState {
    /// Section is collapsed, showing only header
    #[default]
    Collapsed,
    /// Section is expanded, showing all content
    Expanded,
}

impl CollapsibleState {
    /// Toggle between collapsed and expanded states.
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Collapsed => Self::Expanded,
            Self::Expanded => Self::Collapsed,
        };
    }

    /// Check if the section is expanded.
    pub fn is_expanded(&self) -> bool {
        matches!(self, Self::Expanded)
    }
}

/// A collapsible section with header and content.
#[derive(Debug, Clone)]
pub struct CollapsibleSection {
    /// Section header text
    header: String,
    /// Collapsed content lines (stored but not displayed when collapsed)
    content: Vec<String>,
    /// Current state
    state: CollapsibleState,
    /// Number of lines hidden when collapsed
    hidden_count: usize,
    /// Color theme
    theme: Theme,
    /// Keyboard hint for expanding (e.g., "ctrl+e")
    expand_hint: Option<String>,
}

impl CollapsibleSection {
    /// Create a new collapsible section.
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            content: Vec::new(),
            state: CollapsibleState::Collapsed,
            hidden_count: 0,
            theme: Theme::default(),
            expand_hint: Some("ctrl+e".to_string()),
        }
    }

    /// Create a collapsible section with a custom theme.
    pub fn with_theme(header: impl Into<String>, theme: Theme) -> Self {
        Self {
            header: header.into(),
            content: Vec::new(),
            state: CollapsibleState::Collapsed,
            hidden_count: 0,
            theme,
            expand_hint: Some("ctrl+e".to_string()),
        }
    }

    /// Add content lines to the section.
    pub fn add_content(mut self, lines: Vec<String>) -> Self {
        self.hidden_count = lines.len();
        self.content = lines;
        self
    }

    /// Add a single content line.
    pub fn add_line(mut self, line: impl Into<String>) -> Self {
        self.content.push(line.into());
        self.hidden_count = self.content.len();
        self
    }

    /// Set the initial state.
    pub fn with_state(mut self, state: CollapsibleState) -> Self {
        self.state = state;
        self
    }

    /// Set the expand hint.
    pub fn with_expand_hint(mut self, hint: impl Into<String>) -> Self {
        self.expand_hint = Some(hint.into());
        self
    }

    /// Remove the expand hint.
    pub fn without_expand_hint(mut self) -> Self {
        self.expand_hint = None;
        self
    }

    /// Start expanded.
    pub fn expanded(mut self) -> Self {
        self.state = CollapsibleState::Expanded;
        self
    }

    /// Start collapsed.
    pub fn collapsed(mut self) -> Self {
        self.state = CollapsibleState::Collapsed;
        self
    }

    /// Get the current state.
    pub fn state(&self) -> CollapsibleState {
        self.state
    }

    /// Toggle the section state.
    pub fn toggle(&mut self) {
        self.state.toggle();
    }

    /// Expand the section.
    pub fn expand(&mut self) {
        self.state = CollapsibleState::Expanded;
    }

    /// Collapse the section.
    pub fn collapse(&mut self) {
        self.state = CollapsibleState::Collapsed;
    }

    /// Get the number of hidden lines.
    pub fn hidden_count(&self) -> usize {
        if self.state.is_expanded() {
            0
        } else {
            self.hidden_count
        }
    }

    /// Render the section as a string.
    pub fn render(&self) -> String {
        match self.state {
            CollapsibleState::Collapsed => self.render_collapsed(),
            CollapsibleState::Expanded => self.render_expanded(),
        }
    }

    /// Render the collapsed view.
    fn render_collapsed(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("{}\n", self.header.color(self.theme.primary)));

        // Collapsed indicator
        if self.hidden_count > 0 {
            let hint = self
                .expand_hint
                .as_ref()
                .map(|h| format!(" ({} to expand)", h))
                .unwrap_or_default();

            let collapsed_msg = format!(
                "  {} +{} lines{}",
                "▶".color(self.theme.muted),
                self.hidden_count,
                hint
            );
            output.push_str(&format!("{}\n", collapsed_msg.color(self.theme.muted)));
        }

        output
    }

    /// Render the expanded view.
    fn render_expanded(&self) -> String {
        let mut output = String::new();

        // Header with collapse indicator
        output.push_str(&format!(
            "{} {}\n",
            "▼".color(self.theme.active),
            self.header.color(self.theme.primary)
        ));

        // Content
        for line in &self.content {
            output.push_str(&format!("  {}\n", line));
        }

        output
    }
}

/// A collapsible iteration summary with detailed information.
#[derive(Debug, Clone)]
pub struct CollapsibleIterationSummary {
    /// Story ID
    story_id: String,
    /// Story title (optional)
    title: Option<String>,
    /// Iteration number
    iteration: u32,
    /// Max iterations
    max_iterations: u32,
    /// Duration in seconds
    duration_secs: f64,
    /// Git commit hash (if committed)
    commit_hash: Option<String>,
    /// Files changed
    files_changed: u32,
    /// Lines added
    lines_added: u32,
    /// Lines deleted
    lines_deleted: u32,
    /// Quality gates summary
    gates: Vec<(String, bool)>, // (gate_name, passed)
    /// Detailed output lines (collapsible)
    detailed_output: Vec<String>,
    /// Color theme
    theme: Theme,
    /// Whether to show detailed output
    show_details: bool,
}

impl CollapsibleIterationSummary {
    /// Create a new iteration summary.
    pub fn new(story_id: impl Into<String>, iteration: u32, max_iterations: u32) -> Self {
        Self {
            story_id: story_id.into(),
            title: None,
            iteration,
            max_iterations,
            duration_secs: 0.0,
            commit_hash: None,
            files_changed: 0,
            lines_added: 0,
            lines_deleted: 0,
            gates: Vec::new(),
            detailed_output: Vec::new(),
            theme: Theme::default(),
            show_details: false,
        }
    }

    /// Set the story title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the duration.
    pub fn with_duration(mut self, seconds: f64) -> Self {
        self.duration_secs = seconds;
        self
    }

    /// Set the commit hash.
    pub fn with_commit(mut self, hash: impl Into<String>) -> Self {
        self.commit_hash = Some(hash.into());
        self
    }

    /// Set file change statistics.
    pub fn with_changes(mut self, files: u32, added: u32, deleted: u32) -> Self {
        self.files_changed = files;
        self.lines_added = added;
        self.lines_deleted = deleted;
        self
    }

    /// Add a quality gate result.
    pub fn add_gate(mut self, name: impl Into<String>, passed: bool) -> Self {
        self.gates.push((name.into(), passed));
        self
    }

    /// Add detailed output lines.
    pub fn with_detailed_output(mut self, lines: Vec<String>) -> Self {
        self.detailed_output = lines;
        self
    }

    /// Set whether to show details.
    pub fn show_details(mut self, show: bool) -> Self {
        self.show_details = show;
        self
    }

    /// Format the duration.
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

    /// Render the summary header (always visible).
    pub fn render_header(&self) -> String {
        let status_icon = "✓";
        let status = "COMPLETE";

        let title_part = self
            .title
            .as_ref()
            .map(|t| format!(" - {}", t))
            .unwrap_or_default();

        format!(
            "{} Story {}{} {}",
            status_icon.color(self.theme.success),
            self.story_id.color(self.theme.story_id),
            title_part,
            status.color(self.theme.success).bold()
        )
    }

    /// Render the summary details.
    pub fn render_details(&self) -> String {
        let mut output = String::new();

        // Duration and iterations
        output.push_str(&format!(
            "Duration: {} | Iterations: {}/{}\n",
            self.format_duration().color(self.theme.muted),
            self.iteration,
            self.max_iterations
        ));

        // Commit info if present
        if let Some(ref hash) = self.commit_hash {
            output.push_str(&format!("Commit: {}\n", hash.color(self.theme.story_id)));
        }

        // File changes if present
        if self.files_changed > 0 {
            output.push_str(&format!(
                "{} files changed, {} insertions(+), {} deletions(-)\n",
                self.files_changed,
                format!("+{}", self.lines_added).color(self.theme.success),
                format!("-{}", self.lines_deleted).color(self.theme.error)
            ));
        }

        output
    }

    /// Render the gate summary as a chain.
    pub fn render_gates(&self) -> String {
        if self.gates.is_empty() {
            return String::new();
        }

        let gate_parts: Vec<String> = self
            .gates
            .iter()
            .map(|(name, passed)| {
                let icon = if *passed { "✓" } else { "✗" };
                let color = if *passed {
                    self.theme.success
                } else {
                    self.theme.error
                };
                format!("{} {}", name, icon.color(color))
            })
            .collect();

        gate_parts.join(" → ")
    }

    /// Render the full summary.
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Box top
        let inner_width = 58;
        output.push_str(&format!(
            "{}\n",
            format!("╭{}╮", "─".repeat(inner_width)).color(self.theme.muted)
        ));

        // Header
        let header = self.render_header();
        output.push_str(&format!(
            "│ {:<width$} │\n",
            header,
            width = inner_width - 2
        ));

        // Separator
        output.push_str(&format!(
            "{}\n",
            format!("├{}┤", "─".repeat(inner_width)).color(self.theme.muted)
        ));

        // Details
        let details = self.render_details();
        for line in details.lines() {
            output.push_str(&format!("│ {:<width$} │\n", line, width = inner_width - 2));
        }

        // Gates
        if !self.gates.is_empty() {
            let gates = self.render_gates();
            output.push_str(&format!("│ {:<width$} │\n", gates, width = inner_width - 2));
        }

        // Collapsible detailed output
        if !self.detailed_output.is_empty() {
            output.push_str(&format!(
                "{}\n",
                format!("├{}┤", "─".repeat(inner_width)).color(self.theme.muted)
            ));

            if self.show_details {
                // Show detailed output
                for line in &self.detailed_output {
                    let truncated = if line.len() > inner_width - 4 {
                        format!("{}...", &line[..inner_width - 7])
                    } else {
                        line.clone()
                    };
                    output.push_str(&format!(
                        "│ {:<width$} │\n",
                        truncated.color(self.theme.muted),
                        width = inner_width - 2
                    ));
                }
            } else {
                // Show collapsed indicator
                let collapsed_msg = format!(
                    "{} +{} lines (ctrl+e to expand)",
                    "▶",
                    self.detailed_output.len()
                );
                output.push_str(&format!(
                    "│ {:<width$} │\n",
                    collapsed_msg.color(self.theme.muted),
                    width = inner_width - 2
                ));
            }
        }

        // Box bottom
        output.push_str(&format!(
            "{}",
            format!("╰{}╯", "─".repeat(inner_width)).color(self.theme.muted)
        ));

        output
    }
}

/// Display options for streaming/verbose output.
#[derive(Debug, Clone, Copy, Default)]
pub struct StreamingDisplayOptions {
    /// Whether to show streaming output
    pub show_streaming: bool,
    /// Whether sections are expanded by default
    pub expand_by_default: bool,
    /// Maximum lines to show when collapsed
    pub collapsed_preview_lines: usize,
}

impl StreamingDisplayOptions {
    /// Create new options with all output shown.
    pub fn verbose() -> Self {
        Self {
            show_streaming: true,
            expand_by_default: true,
            collapsed_preview_lines: 5,
        }
    }

    /// Create new options with minimal output.
    pub fn quiet() -> Self {
        Self {
            show_streaming: false,
            expand_by_default: false,
            collapsed_preview_lines: 0,
        }
    }

    /// Create new options with partial preview.
    pub fn preview(lines: usize) -> Self {
        Self {
            show_streaming: true,
            expand_by_default: false,
            collapsed_preview_lines: lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapsible_state_toggle() {
        let mut state = CollapsibleState::Collapsed;
        state.toggle();
        assert_eq!(state, CollapsibleState::Expanded);
        state.toggle();
        assert_eq!(state, CollapsibleState::Collapsed);
    }

    #[test]
    fn test_collapsible_section_new() {
        let section = CollapsibleSection::new("Test Header");
        assert_eq!(section.header, "Test Header");
        assert_eq!(section.state, CollapsibleState::Collapsed);
    }

    #[test]
    fn test_collapsible_section_with_content() {
        let section = CollapsibleSection::new("Header")
            .add_content(vec!["Line 1".to_string(), "Line 2".to_string()]);
        assert_eq!(section.hidden_count, 2);
    }

    #[test]
    fn test_collapsible_section_render_collapsed() {
        let section = CollapsibleSection::new("Test")
            .add_content(vec!["Hidden".to_string()])
            .collapsed();
        let output = section.render();
        assert!(output.contains("+1 lines"));
    }

    #[test]
    fn test_collapsible_section_render_expanded() {
        let section = CollapsibleSection::new("Test")
            .add_content(vec!["Visible".to_string()])
            .expanded();
        let output = section.render();
        assert!(output.contains("Visible"));
    }

    #[test]
    fn test_iteration_summary_render() {
        let summary = CollapsibleIterationSummary::new("US-001", 3, 10)
            .with_title("Test Story")
            .with_duration(65.0)
            .with_commit("abc123f")
            .with_changes(5, 100, 50)
            .add_gate("build", true)
            .add_gate("test", true);

        let output = summary.render();
        assert!(output.contains("US-001"));
        assert!(output.contains("1m 5s"));
        assert!(output.contains("abc123f"));
        assert!(output.contains("build"));
    }

    #[test]
    fn test_streaming_options_verbose() {
        let opts = StreamingDisplayOptions::verbose();
        assert!(opts.show_streaming);
        assert!(opts.expand_by_default);
    }
}
