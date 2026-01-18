//! Story display panels for Ralph's terminal UI.
//!
//! Provides visual representation of user stories with Unicode box drawing,
//! status indicators, and acceptance criteria checklists.

#![allow(dead_code)]

use owo_colors::OwoColorize;

use crate::ui::colors::{ansi, Theme};

/// Information about a user story for display purposes.
#[derive(Debug, Clone)]
pub struct StoryInfo {
    /// Story identifier (e.g., "US-001")
    pub id: String,
    /// Story title
    pub title: String,
    /// Priority level (1 = highest)
    pub priority: u32,
    /// Whether the story has passed all quality gates
    pub passes: bool,
    /// List of acceptance criteria
    pub acceptance_criteria: Vec<String>,
}

/// State of a story in the execution view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryViewState {
    /// Story is waiting to be executed
    Pending,
    /// Story is currently being executed
    InProgress,
    /// Story completed successfully
    Completed,
    /// Story execution failed
    Failed,
}

impl StoryViewState {
    /// Get the status icon for this state.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::InProgress => "◉",
            Self::Completed => "✓",
            Self::Failed => "✗",
        }
    }

    /// Get the status label for this state.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::InProgress => "In Progress",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }
}

/// Renders story display panels to the terminal.
#[derive(Debug)]
pub struct StoryView {
    /// Color theme for rendering
    theme: Theme,
    /// Panel width (characters)
    width: usize,
}

impl Default for StoryView {
    fn default() -> Self {
        Self::new()
    }
}

impl StoryView {
    /// Create a new story view with default settings.
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            width: 60,
        }
    }

    /// Create a story view with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self { theme, width: 60 }
    }

    /// Set the panel width.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Render the current story panel.
    ///
    /// Displays a visually prominent panel with story details,
    /// status, and acceptance criteria checklist.
    pub fn render_current_story(&self, story: &StoryInfo, state: StoryViewState) -> String {
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border with rounded corners
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));

        // Story ID and status line
        let status_icon = state.icon();
        let status_label = state.label();
        let id_display = format!(" {} ", story.id);
        let status_display = format!("{} {}", status_icon, status_label);

        // Calculate padding for right-aligned status
        let content_len = id_display.len() + status_display.len();
        let padding = if inner_width > content_len {
            inner_width - content_len
        } else {
            1
        };

        let id_colored = id_display.color(self.theme.story_id);
        let status_colored = match state {
            StoryViewState::Pending => status_display.color(self.theme.muted).to_string(),
            StoryViewState::InProgress => {
                // Use orange for active/in-progress state
                status_display.color(self.theme.active).bold().to_string()
            }
            StoryViewState::Completed => {
                // Use strikethrough for completed items
                format!(
                    "{}{}{}",
                    ansi::STRIKETHROUGH_START,
                    status_display.color(self.theme.completed),
                    ansi::STRIKETHROUGH_END
                )
            }
            StoryViewState::Failed => status_display.color(self.theme.error).to_string(),
        };

        output.push_str(&format!(
            "│{}{}{}│\n",
            id_colored,
            " ".repeat(padding),
            status_colored
        ));

        // Title line
        let title = self.truncate_text(&story.title, inner_width - 2);
        output.push_str(&format!("│ {:<width$} │\n", title, width = inner_width - 2));

        // Priority line
        let priority_text = format!("Priority: {}", story.priority);
        output.push_str(&format!(
            "│ {:<width$} │\n",
            priority_text.color(self.theme.muted),
            width = inner_width - 2
        ));

        // Separator
        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Acceptance criteria header
        output.push_str(&format!(
            "│ {:<width$} │\n",
            "Acceptance Criteria:",
            width = inner_width - 2
        ));

        // Render each criterion with circle marker
        for criterion in &story.acceptance_criteria {
            let marker = if story.passes { "●" } else { "○" };
            let marker_colored = if story.passes {
                marker.color(self.theme.success).to_string()
            } else {
                marker.color(self.theme.muted).to_string()
            };

            // Truncate criterion to fit in panel
            let max_criterion_len = inner_width - 5; // 2 for padding, 2 for marker, 1 for space
            let criterion_text = self.truncate_text(criterion, max_criterion_len);

            output.push_str(&format!(
                "│  {} {:<width$}│\n",
                marker_colored,
                criterion_text,
                width = max_criterion_len
            ));
        }

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Render a preview panel for the next story.
    ///
    /// Uses lighter styling to indicate it's not the active story.
    pub fn render_next_story(&self, story: &StoryInfo) -> String {
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border with rounded corners (muted)
        output.push_str(&format!(
            "{}\n",
            format!("╭{}╮", "─".repeat(inner_width)).color(self.theme.muted)
        ));

        // Header
        let header = " Next Story ";
        let header_padding = (inner_width - header.len()) / 2;
        output.push_str(&format!(
            "{}{}{}{}│\n",
            "│".color(self.theme.muted),
            " ".repeat(header_padding),
            header.color(self.theme.muted),
            " ".repeat(inner_width - header_padding - header.len()),
        ));

        // Story ID line
        let id_display = format!(" {} ", story.id);
        let title = self.truncate_text(&story.title, inner_width - id_display.len() - 3);

        output.push_str(&format!(
            "{}{}  {}{}\n",
            "│".color(self.theme.muted),
            id_display.color(self.theme.story_id),
            title.color(self.theme.muted),
            format!(
                "{:>width$}│",
                "",
                width = inner_width - id_display.len() - title.len() - 2
            )
            .color(self.theme.muted)
        ));

        // Priority line
        let priority_text = format!(
            "Priority: {} • {} criteria",
            story.priority,
            story.acceptance_criteria.len()
        );
        output.push_str(&format!(
            "{} {:<width$}{}\n",
            "│".color(self.theme.muted),
            priority_text.color(self.theme.muted),
            "│".color(self.theme.muted),
            width = inner_width - 2
        ));

        // Bottom border
        output.push_str(&format!(
            "{}",
            format!("╰{}╯", "─".repeat(inner_width)).color(self.theme.muted)
        ));

        output
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_view_state_icons() {
        assert_eq!(StoryViewState::Pending.icon(), "○");
        assert_eq!(StoryViewState::InProgress.icon(), "◉");
        assert_eq!(StoryViewState::Completed.icon(), "✓");
        assert_eq!(StoryViewState::Failed.icon(), "✗");
    }

    #[test]
    fn test_story_view_state_labels() {
        assert_eq!(StoryViewState::Pending.label(), "Pending");
        assert_eq!(StoryViewState::InProgress.label(), "In Progress");
        assert_eq!(StoryViewState::Completed.label(), "Completed");
        assert_eq!(StoryViewState::Failed.label(), "Failed");
    }

    #[test]
    fn test_story_info_creation() {
        let story = StoryInfo {
            id: "US-001".to_string(),
            title: "Test Story".to_string(),
            priority: 1,
            passes: false,
            acceptance_criteria: vec!["Criterion 1".to_string(), "Criterion 2".to_string()],
        };

        assert_eq!(story.id, "US-001");
        assert_eq!(story.priority, 1);
        assert!(!story.passes);
        assert_eq!(story.acceptance_criteria.len(), 2);
    }

    #[test]
    fn test_render_current_story() {
        let view = StoryView::new().with_width(50);
        let story = StoryInfo {
            id: "US-001".to_string(),
            title: "Test Story".to_string(),
            priority: 1,
            passes: false,
            acceptance_criteria: vec!["First criterion".to_string()],
        };

        let output = view.render_current_story(&story, StoryViewState::InProgress);

        // Check that output contains expected elements
        assert!(output.contains("US-001"));
        assert!(output.contains("Test Story"));
        assert!(output.contains("Priority: 1"));
        assert!(output.contains("Acceptance Criteria:"));
        assert!(output.contains("First criterion"));
    }

    #[test]
    fn test_render_next_story() {
        let view = StoryView::new().with_width(50);
        let story = StoryInfo {
            id: "US-002".to_string(),
            title: "Next Story".to_string(),
            priority: 2,
            passes: false,
            acceptance_criteria: vec!["Criterion A".to_string(), "Criterion B".to_string()],
        };

        let output = view.render_next_story(&story);

        // Check that output contains expected elements
        assert!(output.contains("Next Story"));
        assert!(output.contains("US-002"));
        assert!(output.contains("Priority: 2"));
        assert!(output.contains("2 criteria"));
    }
}
