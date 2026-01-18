//! Rich progress indicators using ratatui.
//!
//! Provides visual progress bars with smooth animations,
//! status indicators, and gradient colors.

#![allow(dead_code)]

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use super::animation::AnimationState;

/// A rich progress bar with smooth animation and gradient colors.
#[derive(Debug, Clone)]
pub struct RichProgress {
    /// Current progress (0.0 to 1.0)
    progress: f64,
    /// Label to show
    label: String,
    /// Start color of gradient
    color_start: Color,
    /// End color of gradient
    color_end: Color,
    /// Background color
    bg_color: Color,
    /// Whether to show percentage
    show_percentage: bool,
    /// Whether to show the label
    show_label: bool,
}

impl Default for RichProgress {
    fn default() -> Self {
        Self {
            progress: 0.0,
            label: String::new(),
            color_start: Color::Rgb(34, 197, 94), // Green
            color_end: Color::Rgb(59, 130, 246),  // Blue
            bg_color: Color::Rgb(55, 65, 81),     // Gray
            show_percentage: true,
            show_label: true,
        }
    }
}

impl RichProgress {
    /// Create a new rich progress bar.
    pub fn new(progress: f64) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Set the progress value.
    pub fn progress(mut self, progress: f64) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Set the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the gradient colors.
    pub fn colors(mut self, start: Color, end: Color) -> Self {
        self.color_start = start;
        self.color_end = end;
        self
    }

    /// Set the background color.
    pub fn bg(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Enable/disable percentage display.
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Enable/disable label display.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Render as a formatted string.
    pub fn render_string(&self, width: usize) -> String {
        let bar_width = width.saturating_sub(10);
        let filled = (self.progress * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);

        let mut output = format!("[{}{}]", "━".repeat(filled), "─".repeat(empty));

        if self.show_percentage {
            output.push_str(&format!(" {}%", (self.progress * 100.0) as u32));
        }

        output
    }
}

impl Widget for RichProgress {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 1 {
            return;
        }

        // Use Paragraph for simpler rendering
        let content = self.render_string(area.width as usize);
        let paragraph = Paragraph::new(content);
        paragraph.render(area, buf);
    }
}

/// Status indicator for story progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryState {
    /// Story is pending
    Pending,
    /// Story is running
    Running,
    /// Story passed
    Passed,
    /// Story failed
    Failed,
}

impl StoryState {
    /// Get the emoji for this state.
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Pending => "⚪",
            Self::Running => "🔵",
            Self::Passed => "🟢",
            Self::Failed => "🔴",
        }
    }

    /// Get the icon for this state.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Running => "◐",
            Self::Passed => "✓",
            Self::Failed => "✗",
        }
    }

    /// Get the color for this state.
    pub fn color(&self) -> Color {
        match self {
            Self::Pending => Color::Rgb(107, 114, 128), // Gray
            Self::Running => Color::Rgb(59, 130, 246),  // Blue
            Self::Passed => Color::Rgb(34, 197, 94),    // Green
            Self::Failed => Color::Rgb(239, 68, 68),    // Red
        }
    }
}

/// Story progress widget showing all stories with status.
#[derive(Debug, Clone)]
pub struct StoryProgressWidget {
    /// Stories with their IDs and states
    stories: Vec<(String, StoryState)>,
    /// Current progress (completed / total)
    current: usize,
    /// Total stories
    total: usize,
    /// Animation state for running indicator
    animation: Option<AnimationState>,
}

impl StoryProgressWidget {
    /// Create a new story progress widget.
    pub fn new(stories: Vec<(String, StoryState)>) -> Self {
        let current = stories
            .iter()
            .filter(|(_, s)| *s == StoryState::Passed)
            .count();
        let total = stories.len();
        Self {
            stories,
            current,
            total,
            animation: None,
        }
    }

    /// Set the animation state.
    pub fn with_animation(mut self, animation: AnimationState) -> Self {
        self.animation = Some(animation);
        self
    }

    /// Render as a formatted string (for non-TUI use).
    pub fn render_string(&self) -> String {
        let mut output = String::new();

        // Progress bar
        let bar_width = 20;
        let filled = if self.total > 0 {
            (self.current as f64 / self.total as f64 * bar_width as f64) as usize
        } else {
            0
        };
        let empty = bar_width - filled;

        output.push_str(&format!(
            "Progress: [{}{}] {}/{} stories ({}%)\n",
            "━".repeat(filled),
            "─".repeat(empty),
            self.current,
            self.total,
            if self.total > 0 {
                (self.current * 100) / self.total
            } else {
                0
            }
        ));

        // Story list
        output.push_str("         └── ");
        for (i, (id, state)) in self.stories.iter().take(6).enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            output.push_str(&format!("{} {} {}", state.emoji(), id, state.icon()));
        }
        if self.stories.len() > 6 {
            output.push_str("...");
        }

        output
    }
}

impl Widget for StoryProgressWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 1 {
            return;
        }

        // Use Paragraph for simpler rendering
        let content = self.render_string();
        let lines: Vec<Line> = content.lines().map(|s| Line::from(s.to_string())).collect();
        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rich_progress_new() {
        let progress = RichProgress::new(0.5);
        assert!((progress.progress - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_rich_progress_clamp() {
        let progress = RichProgress::new(1.5);
        assert!((progress.progress - 1.0).abs() < 0.001);

        let progress = RichProgress::new(-0.5);
        assert!((progress.progress - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_story_state_emoji() {
        assert_eq!(StoryState::Pending.emoji(), "⚪");
        assert_eq!(StoryState::Running.emoji(), "🔵");
        assert_eq!(StoryState::Passed.emoji(), "🟢");
        assert_eq!(StoryState::Failed.emoji(), "🔴");
    }

    #[test]
    fn test_story_state_icon() {
        assert_eq!(StoryState::Pending.icon(), "○");
        assert_eq!(StoryState::Running.icon(), "◐");
        assert_eq!(StoryState::Passed.icon(), "✓");
        assert_eq!(StoryState::Failed.icon(), "✗");
    }

    #[test]
    fn test_story_progress_widget_new() {
        let stories = vec![
            ("US-001".to_string(), StoryState::Passed),
            ("US-002".to_string(), StoryState::Running),
            ("US-003".to_string(), StoryState::Pending),
        ];
        let widget = StoryProgressWidget::new(stories);
        assert_eq!(widget.current, 1);
        assert_eq!(widget.total, 3);
    }

    #[test]
    fn test_story_progress_render_string() {
        let stories = vec![
            ("US-001".to_string(), StoryState::Passed),
            ("US-002".to_string(), StoryState::Running),
        ];
        let widget = StoryProgressWidget::new(stories);
        let output = widget.render_string();
        assert!(output.contains("Progress:"));
        assert!(output.contains("1/2"));
        assert!(output.contains("US-001"));
        assert!(output.contains("US-002"));
    }
}
