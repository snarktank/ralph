//! Notification renderer for themed error recovery panels.
//!
//! Provides visual notification panels for different error states during
//! Ralph's execution, using the existing color scheme from the UI module.

use std::time::Duration;

use owo_colors::OwoColorize;

use crate::ui::{RalphDisplay, Theme};

/// Renderer for themed notification panels during error recovery.
///
/// This struct holds a reference to a RalphDisplay and uses its theme
/// to render consistent notification panels for various error states.
pub struct NotificationRenderer<'a> {
    /// Reference to the RalphDisplay for theming
    display: &'a RalphDisplay,
    /// Panel width (characters)
    width: usize,
}

impl<'a> NotificationRenderer<'a> {
    /// Creates a new NotificationRenderer with a reference to a RalphDisplay.
    pub fn new(display: &'a RalphDisplay) -> Self {
        Self { display, width: 60 }
    }

    /// Sets the panel width.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Gets the theme from the display.
    fn theme(&self) -> &Theme {
        self.display.theme()
    }

    /// Helper to create a horizontal border line.
    fn border_line(&self, width: usize) -> String {
        "─".repeat(width)
    }

    /// Safely truncates a string to the given maximum character count.
    /// This handles multi-byte UTF-8 characters correctly.
    fn safe_truncate(s: &str, max_chars: usize) -> String {
        if s.chars().count() <= max_chars {
            s.to_string()
        } else {
            let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
            format!("{}...", truncated)
        }
    }

    /// Renders a rate limit notification with countdown timer display.
    ///
    /// # Arguments
    /// * `retry_after` - Duration until the rate limit resets.
    /// * `message` - Optional custom message to display.
    ///
    /// # Returns
    /// A string containing the formatted notification panel.
    pub fn render_rate_limit(&self, retry_after: Duration, message: Option<&str>) -> String {
        let theme = self.theme();
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border
        let top_border = self.border_line(inner_width);
        output.push_str(&format!("{}\n", top_border.color(theme.warning)));

        // Header with warning icon
        let header = "Rate Limit Exceeded";
        let header_padding = inner_width.saturating_sub(header.chars().count() + 2);
        let header_colored = header.color(theme.warning);
        let header_line = format!(" {} {}", header_colored.bold(), " ".repeat(header_padding));
        output.push_str(&format!("{header_line}\n"));

        // Separator
        let separator = self.border_line(inner_width);
        output.push_str(&format!("{}\n", separator.color(theme.warning)));

        // Empty line for spacing
        output.push('\n');

        // Custom message or default
        let msg = message.unwrap_or("API rate limit reached. Waiting for reset...");
        let msg_padding = inner_width.saturating_sub(msg.len() + 1);
        output.push_str(&format!(" {}{}\n", msg, " ".repeat(msg_padding)));

        // Countdown timer display
        let secs = retry_after.as_secs();
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        let countdown = if mins > 0 {
            format!("Retry in: {}m {}s", mins, remaining_secs)
        } else {
            format!("Retry in: {}s", secs)
        };
        let countdown_padding = inner_width.saturating_sub(countdown.len() + 1);
        output.push_str(&format!(
            " {}{}\n",
            countdown.color(theme.in_progress).bold(),
            " ".repeat(countdown_padding)
        ));

        // Empty line for spacing
        output.push('\n');

        // Bottom border
        let bottom_border = self.border_line(inner_width);
        output.push_str(&format!("{}", bottom_border.color(theme.warning)));

        output
    }

    /// Renders a usage limit notification with action required message.
    ///
    /// # Arguments
    /// * `limit_type` - Description of the limit type (e.g., "quota", "tokens").
    /// * `action` - The action required from the user.
    ///
    /// # Returns
    /// A string containing the formatted notification panel.
    pub fn render_usage_limit(&self, limit_type: &str, action: &str) -> String {
        let theme = self.theme();
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border
        let top_border = self.border_line(inner_width);
        output.push_str(&format!("{}\n", top_border.color(theme.error)));

        // Header with error icon
        let header = "Usage Limit Reached";
        let header_padding = inner_width.saturating_sub(header.chars().count() + 2);
        let header_colored = header.color(theme.error);
        let header_line = format!(" {} {}", header_colored.bold(), " ".repeat(header_padding));
        output.push_str(&format!("{header_line}\n"));

        // Separator
        let separator = self.border_line(inner_width);
        output.push_str(&format!("{}\n", separator.color(theme.error)));

        // Empty line for spacing
        output.push('\n');

        // Limit type message
        let limit_msg = format!("Limit type: {}", limit_type);
        let limit_padding = inner_width.saturating_sub(limit_msg.len() + 1);
        output.push_str(&format!(" {}{}\n", limit_msg, " ".repeat(limit_padding)));

        // Empty line for spacing
        output.push('\n');

        // Action required header
        let action_header = "Action Required:";
        let action_header_padding = inner_width.saturating_sub(action_header.len() + 1);
        output.push_str(&format!(
            " {}{}\n",
            action_header.color(theme.active).bold(),
            " ".repeat(action_header_padding)
        ));

        // Action message
        let action_padding = inner_width.saturating_sub(action.len() + 1);
        output.push_str(&format!(" {}{}\n", action, " ".repeat(action_padding)));

        // Empty line for spacing
        output.push('\n');

        // Bottom border
        let bottom_border = self.border_line(inner_width);
        output.push_str(&format!("{}", bottom_border.color(theme.error)));

        output
    }

    /// Renders a timeout notification with checkpoint confirmation.
    ///
    /// # Arguments
    /// * `operation` - Description of the operation that timed out.
    /// * `checkpoint_saved` - Whether a checkpoint was successfully saved.
    /// * `checkpoint_path` - Optional path to the saved checkpoint.
    ///
    /// # Returns
    /// A string containing the formatted notification panel.
    pub fn render_timeout(
        &self,
        operation: &str,
        checkpoint_saved: bool,
        checkpoint_path: Option<&str>,
    ) -> String {
        let theme = self.theme();
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border
        let top_border = self.border_line(inner_width);
        output.push_str(&format!("{}\n", top_border.color(theme.warning)));

        // Header with warning icon
        let header = "Operation Timeout";
        let header_padding = inner_width.saturating_sub(header.chars().count() + 2);
        let header_colored = header.color(theme.warning);
        let header_line = format!(" {} {}", header_colored.bold(), " ".repeat(header_padding));
        output.push_str(&format!("{header_line}\n"));

        // Separator
        let separator = self.border_line(inner_width);
        output.push_str(&format!("{}\n", separator.color(theme.warning)));

        // Empty line for spacing
        output.push('\n');

        // Operation message
        let op_msg = format!("Operation: {}", operation);
        let op_padding = inner_width.saturating_sub(op_msg.len() + 1);
        output.push_str(&format!(" {}{}\n", op_msg, " ".repeat(op_padding)));

        // Empty line for spacing
        output.push('\n');

        // Checkpoint status
        let (checkpoint_status, status_icon, status_color) = if checkpoint_saved {
            ("Checkpoint saved successfully", "✓", theme.success)
        } else {
            ("No checkpoint saved", "✗", theme.error)
        };
        let status_msg = format!("{} {}", status_icon, checkpoint_status);
        let status_padding = inner_width.saturating_sub(status_msg.chars().count() + 1);
        output.push_str(&format!(
            " {}{}\n",
            status_msg.color(status_color),
            " ".repeat(status_padding)
        ));

        // Checkpoint path if available
        if let Some(path) = checkpoint_path {
            let path_msg = format!("Path: {}", path);
            // Truncate path if too long (UTF-8 safe)
            let truncated_path = if path_msg.chars().count() > inner_width - 2 {
                let path_chars: Vec<char> = path.chars().collect();
                let max_path_chars = inner_width.saturating_sub(12);
                let start_idx = path_chars.len().saturating_sub(max_path_chars);
                let truncated_path_part: String = path_chars[start_idx..].iter().collect();
                format!("Path: ...{}", truncated_path_part)
            } else {
                path_msg
            };
            let path_padding = inner_width.saturating_sub(truncated_path.len() + 1);
            output.push_str(&format!(
                " {}{}\n",
                truncated_path.color(theme.muted),
                " ".repeat(path_padding)
            ));
        }

        // Resume hint
        output.push('\n');
        let resume_hint = "Run again to resume from checkpoint.";
        let resume_padding = inner_width.saturating_sub(resume_hint.len() + 1);
        output.push_str(&format!(
            " {}{}\n",
            resume_hint.color(theme.muted),
            " ".repeat(resume_padding)
        ));

        // Bottom border
        let bottom_border = self.border_line(inner_width);
        output.push_str(&format!("{}", bottom_border.color(theme.warning)));

        output
    }

    /// Renders a retry notification with attempt number and delay.
    ///
    /// # Arguments
    /// * `attempt` - Current retry attempt number (1-based).
    /// * `max_attempts` - Maximum number of retry attempts.
    /// * `delay` - Duration until the next retry.
    /// * `error_summary` - Brief summary of the error that triggered the retry.
    ///
    /// # Returns
    /// A string containing the formatted notification panel.
    pub fn render_retry(
        &self,
        attempt: u32,
        max_attempts: u32,
        delay: Duration,
        error_summary: &str,
    ) -> String {
        let theme = self.theme();
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border
        let top_border = self.border_line(inner_width);
        output.push_str(&format!("{}\n", top_border.color(theme.in_progress)));

        // Header with retry icon
        let header = format!("Retry Attempt {}/{}", attempt, max_attempts);
        let header_padding = inner_width.saturating_sub(header.chars().count() + 2);
        let header_colored = header.color(theme.in_progress);
        let header_line = format!(" {} {}", header_colored.bold(), " ".repeat(header_padding));
        output.push_str(&format!("{header_line}\n"));

        // Separator
        let separator = self.border_line(inner_width);
        output.push_str(&format!("{}\n", separator.color(theme.in_progress)));

        // Empty line for spacing
        output.push('\n');

        // Error summary
        let error_label = "Error:";
        let error_padding = inner_width.saturating_sub(error_label.len() + 1);
        output.push_str(&format!(
            " {}{}\n",
            error_label.color(theme.muted),
            " ".repeat(error_padding)
        ));

        // Truncate error summary if too long (UTF-8 safe)
        let truncated_error = Self::safe_truncate(error_summary, inner_width.saturating_sub(2));
        let err_padding = inner_width.saturating_sub(truncated_error.chars().count() + 1);
        output.push_str(&format!(
            " {}{}\n",
            truncated_error,
            " ".repeat(err_padding)
        ));

        // Empty line for spacing
        output.push('\n');

        // Delay display
        let secs = delay.as_secs();
        let delay_msg = format!("Retrying in {}s...", secs);
        let delay_padding = inner_width.saturating_sub(delay_msg.len() + 1);
        output.push_str(&format!(
            " {}{}\n",
            delay_msg.color(theme.in_progress).bold(),
            " ".repeat(delay_padding)
        ));

        // Progress indicator (visual attempt progress)
        let filled = attempt as usize;
        let empty = (max_attempts - attempt) as usize;
        let progress_bar = format!("{}{}", "●".repeat(filled), "○".repeat(empty));
        let progress_padding = inner_width.saturating_sub(progress_bar.chars().count() + 1);
        output.push_str(&format!(
            " {}{}\n",
            progress_bar.color(theme.in_progress),
            " ".repeat(progress_padding)
        ));

        // Bottom border
        let bottom_border = self.border_line(inner_width);
        output.push_str(&format!("{}", bottom_border.color(theme.in_progress)));

        output
    }

    /// Renders a paused notification with resume instructions.
    ///
    /// # Arguments
    /// * `story_id` - Optional story ID that was paused.
    /// * `reason` - Reason for the pause.
    ///
    /// # Returns
    /// A string containing the formatted notification panel.
    pub fn render_paused(&self, story_id: Option<&str>, reason: &str) -> String {
        let theme = self.theme();
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border
        let top_border = self.border_line(inner_width);
        output.push_str(&format!("{}\n", top_border.color(theme.active)));

        // Header with pause icon
        let header = "Execution Paused";
        let header_padding = inner_width.saturating_sub(header.chars().count() + 2);
        let header_colored = header.color(theme.active);
        let header_line = format!(" {} {}", header_colored.bold(), " ".repeat(header_padding));
        output.push_str(&format!("{header_line}\n"));

        // Separator
        let separator = self.border_line(inner_width);
        output.push_str(&format!("{}\n", separator.color(theme.active)));

        // Empty line for spacing
        output.push('\n');

        // Story ID if available
        if let Some(id) = story_id {
            let story_msg = format!("Story: {}", id);
            let story_padding = inner_width.saturating_sub(story_msg.len() + 1);
            output.push_str(&format!(
                " {}{}\n",
                story_msg.color(theme.story_id),
                " ".repeat(story_padding)
            ));
        }

        // Reason
        let reason_label = "Reason:";
        let reason_padding = inner_width.saturating_sub(reason_label.len() + 1);
        output.push_str(&format!(
            " {}{}\n",
            reason_label.color(theme.muted),
            " ".repeat(reason_padding)
        ));

        // Reason text (may need truncation, UTF-8 safe)
        let truncated_reason = Self::safe_truncate(reason, inner_width.saturating_sub(2));
        let reason_text_padding = inner_width.saturating_sub(truncated_reason.chars().count() + 1);
        output.push_str(&format!(
            " {}{}\n",
            truncated_reason,
            " ".repeat(reason_text_padding)
        ));

        // Empty line for spacing
        output.push('\n');

        // Resume instructions header
        let instructions_header = "Resume Instructions:";
        let instr_padding = inner_width.saturating_sub(instructions_header.len() + 1);
        output.push_str(&format!(
            " {}{}\n",
            instructions_header.color(theme.success).bold(),
            " ".repeat(instr_padding)
        ));

        // Resume instructions
        let instr1 = "Press Enter to resume execution";
        let instr1_padding = inner_width.saturating_sub(instr1.len() + 1);
        output.push_str(&format!(" {}{}\n", instr1, " ".repeat(instr1_padding)));

        let instr2 = "Press Ctrl+C to cancel and save state";
        let instr2_padding = inner_width.saturating_sub(instr2.len() + 1);
        output.push_str(&format!(
            " {}{}\n",
            instr2.color(theme.muted),
            " ".repeat(instr2_padding)
        ));

        // Bottom border
        let bottom_border = self.border_line(inner_width);
        output.push_str(&format!("{}", bottom_border.color(theme.active)));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::RalphDisplay;

    fn create_test_display() -> RalphDisplay {
        RalphDisplay::new()
    }

    #[test]
    fn test_new_notification_renderer() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);
        assert_eq!(renderer.width, 60);
    }

    #[test]
    fn test_with_width() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display).with_width(80);
        assert_eq!(renderer.width, 80);
    }

    #[test]
    fn test_render_rate_limit() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output = renderer.render_rate_limit(Duration::from_secs(30), None);

        assert!(output.contains("Rate Limit Exceeded"));
        assert!(output.contains("Retry in:"));
        assert!(output.contains("30s"));
    }

    #[test]
    fn test_render_rate_limit_with_minutes() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output = renderer.render_rate_limit(Duration::from_secs(90), None);

        assert!(output.contains("1m 30s"));
    }

    #[test]
    fn test_render_rate_limit_with_custom_message() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output =
            renderer.render_rate_limit(Duration::from_secs(60), Some("Custom rate limit message"));

        assert!(output.contains("Custom rate limit message"));
    }

    #[test]
    fn test_render_usage_limit() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output =
            renderer.render_usage_limit("API quota", "Upgrade your plan or wait for reset");

        assert!(output.contains("Usage Limit Reached"));
        assert!(output.contains("API quota"));
        assert!(output.contains("Action Required"));
        assert!(output.contains("Upgrade your plan"));
    }

    #[test]
    fn test_render_timeout_with_checkpoint() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output = renderer.render_timeout("Story execution", true, Some("/tmp/checkpoint.json"));

        assert!(output.contains("Operation Timeout"));
        assert!(output.contains("Story execution"));
        assert!(output.contains("Checkpoint saved successfully"));
        assert!(output.contains("checkpoint.json"));
        assert!(output.contains("resume from checkpoint"));
    }

    #[test]
    fn test_render_timeout_without_checkpoint() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output = renderer.render_timeout("API request", false, None);

        assert!(output.contains("Operation Timeout"));
        assert!(output.contains("API request"));
        assert!(output.contains("No checkpoint saved"));
    }

    #[test]
    fn test_render_retry() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output = renderer.render_retry(2, 5, Duration::from_secs(4), "Connection refused");

        assert!(output.contains("Retry Attempt 2/5"));
        assert!(output.contains("Error:"));
        assert!(output.contains("Connection refused"));
        assert!(output.contains("Retrying in 4s"));
    }

    #[test]
    fn test_render_retry_progress_bar() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output = renderer.render_retry(3, 5, Duration::from_secs(8), "Timeout");

        // Should have 3 filled circles and 2 empty
        assert!(output.contains("●"));
        assert!(output.contains("○"));
    }

    #[test]
    fn test_render_paused_with_story() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output = renderer.render_paused(Some("US-010"), "User requested pause");

        assert!(output.contains("Execution Paused"));
        assert!(output.contains("US-010"));
        assert!(output.contains("User requested pause"));
        assert!(output.contains("Resume Instructions"));
        assert!(output.contains("Press Enter"));
        assert!(output.contains("Ctrl+C"));
    }

    #[test]
    fn test_render_paused_without_story() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display);

        let output = renderer.render_paused(None, "Rate limit wait");

        assert!(output.contains("Execution Paused"));
        assert!(output.contains("Rate limit wait"));
        assert!(!output.contains("Story:"));
    }

    #[test]
    fn test_long_error_truncation() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display).with_width(40);

        let long_error =
            "This is a very long error message that should be truncated when rendered in the panel";
        let output = renderer.render_retry(1, 3, Duration::from_secs(2), long_error);

        assert!(output.contains("..."));
    }

    #[test]
    fn test_long_reason_truncation() {
        let display = create_test_display();
        let renderer = NotificationRenderer::new(&display).with_width(40);

        let long_reason = "This is a very long reason message that exceeds the panel width";
        let output = renderer.render_paused(Some("US-001"), long_reason);

        assert!(output.contains("..."));
    }
}
