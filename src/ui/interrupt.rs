//! Ctrl+C interruption handling UI for Ralph.
//!
//! Provides graceful interruption handling with visual feedback,
//! showing the user what story will be retried on the next run.

#![allow(dead_code)]

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use owo_colors::OwoColorize;

use crate::ui::colors::Theme;

/// Global flag indicating whether an interrupt has been requested.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Handles Ctrl+C interruption with graceful shutdown and visual feedback.
#[derive(Debug, Clone)]
pub struct InterruptHandler {
    /// Color theme for rendering
    theme: Theme,
    /// Panel width (characters)
    width: usize,
    /// Cancellation token for cooperative cancellation
    cancel_flag: Arc<AtomicBool>,
}

impl Default for InterruptHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InterruptHandler {
    /// Create a new interrupt handler with default settings.
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            width: 60,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create an interrupt handler with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            theme,
            width: 60,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set the panel width.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Get the cancellation flag for cooperative cancellation.
    ///
    /// Pass this to long-running operations so they can check for cancellation.
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel_flag)
    }

    /// Check if an interrupt has been requested.
    pub fn is_interrupted(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }

    /// Install the Ctrl+C signal handler.
    ///
    /// This sets up a global handler that will set the interrupt flag
    /// when Ctrl+C is pressed.
    pub fn install_handler(&self) -> io::Result<()> {
        let cancel_flag = Arc::clone(&self.cancel_flag);

        ctrlc::set_handler(move || {
            cancel_flag.store(true, Ordering::SeqCst);
            INTERRUPTED.store(true, Ordering::SeqCst);
        })
        .map_err(|e| io::Error::other(e.to_string()))
    }

    /// Reset the interrupt state (for testing or retry scenarios).
    pub fn reset(&self) {
        self.cancel_flag.store(false, Ordering::SeqCst);
        INTERRUPTED.store(false, Ordering::SeqCst);
    }

    /// Trigger an interrupt programmatically (useful for testing).
    pub fn trigger_interrupt(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
        INTERRUPTED.store(true, Ordering::SeqCst);
    }

    /// Render the interruption notification panel.
    ///
    /// Shows a warning-styled panel with the story that will be
    /// retried on the next run.
    pub fn render_interrupt_panel(&self, current_story_id: Option<&str>) -> String {
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border
        let top_border = format!("╭{}╮", "─".repeat(inner_width));
        output.push_str(&format!("{}\n", top_border.color(self.theme.warning)));

        // Header with warning icon
        let header = "⚠ Interruption Received";
        let header_padding = inner_width.saturating_sub(header.chars().count() + 2);
        let header_colored = header.color(self.theme.warning);
        let header_line = format!(
            "│ {}{}│",
            header_colored.bold(),
            " ".repeat(header_padding + 1)
        );
        output.push_str(&format!("{}\n", header_line));

        // Separator
        let separator = format!("├{}┤", "─".repeat(inner_width));
        output.push_str(&format!("{}\n", separator.color(self.theme.warning)));

        // Empty line for spacing
        output.push_str(&format!("│{}│\n", " ".repeat(inner_width)));

        // Cleanup message
        let cleanup_msg = "Performing graceful cleanup...";
        let cleanup_padding = inner_width.saturating_sub(cleanup_msg.len() + 2);
        output.push_str(&format!(
            "│ {}{}│\n",
            cleanup_msg,
            " ".repeat(cleanup_padding + 1)
        ));

        // Story retry information
        if let Some(story_id) = current_story_id {
            let retry_label = "Story to retry:";
            let retry_padding = inner_width.saturating_sub(retry_label.len() + story_id.len() + 4);
            let story_id_colored = story_id.color(self.theme.story_id);
            output.push_str(&format!(
                "│ {} {}{}│\n",
                retry_label,
                story_id_colored.bold(),
                " ".repeat(retry_padding)
            ));
        } else {
            let no_story_msg = "No story in progress";
            let no_story_padding = inner_width.saturating_sub(no_story_msg.len() + 2);
            output.push_str(&format!(
                "│ {}{}│\n",
                no_story_msg.color(self.theme.muted),
                " ".repeat(no_story_padding + 1)
            ));
        }

        // Empty line for spacing
        output.push_str(&format!("│{}│\n", " ".repeat(inner_width)));

        // State saving message
        let state_msg = "State saved. Run again to continue.";
        let state_padding = inner_width.saturating_sub(state_msg.len() + 2);
        output.push_str(&format!(
            "│ {}{}│\n",
            state_msg.color(self.theme.muted),
            " ".repeat(state_padding + 1)
        ));

        // Bottom border
        let bottom_border = format!("╰{}╯", "─".repeat(inner_width));
        output.push_str(&format!("{}", bottom_border.color(self.theme.warning)));

        output
    }

    /// Render a compact interruption message (single line).
    pub fn render_interrupt_message(&self, current_story_id: Option<&str>) -> String {
        let icon = "⚠".color(self.theme.warning);
        let interrupted_colored = "Interrupted".color(self.theme.warning);
        let msg = interrupted_colored.bold();

        if let Some(story_id) = current_story_id {
            format!(
                "{} {} │ Retry: {}",
                icon,
                msg,
                story_id.color(self.theme.story_id)
            )
        } else {
            format!("{} {}", icon, msg)
        }
    }

    /// Render the cleanup progress message.
    pub fn render_cleanup_progress(&self, step: &str) -> String {
        format!(
            "{} {}",
            "◉".color(self.theme.in_progress),
            step.color(self.theme.muted)
        )
    }

    /// Display the interruption panel and cleanup progress.
    ///
    /// This is a convenience method that prints directly to stdout.
    pub fn display_interrupt(&self, current_story_id: Option<&str>) {
        let panel = self.render_interrupt_panel(current_story_id);
        println!("\n{}\n", panel);
    }

    /// Display cleanup progress message.
    ///
    /// This is a convenience method that prints directly to stdout.
    pub fn display_cleanup_step(&self, step: &str) {
        let msg = self.render_cleanup_progress(step);
        println!("{}", msg);
    }
}

/// Check if an interrupt has been globally signaled.
///
/// This can be called from anywhere in the codebase to check
/// if the user has requested interruption.
pub fn is_globally_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Reset the global interrupt flag.
pub fn reset_global_interrupt() {
    INTERRUPTED.store(false, Ordering::SeqCst);
}

/// Render an interruption notification panel.
///
/// Convenience function for when you don't need an InterruptHandler instance.
pub fn render_interrupt_panel(
    theme: &Theme,
    current_story_id: Option<&str>,
    width: usize,
) -> String {
    let handler = InterruptHandler {
        theme: *theme,
        width,
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };
    handler.render_interrupt_panel(current_story_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_handler_new() {
        let handler = InterruptHandler::new();
        assert!(!handler.is_interrupted());
        assert_eq!(handler.width, 60);
    }

    #[test]
    fn test_interrupt_handler_with_theme() {
        let theme = Theme::default();
        let handler = InterruptHandler::with_theme(theme);
        assert!(!handler.is_interrupted());
    }

    #[test]
    fn test_interrupt_handler_with_width() {
        let handler = InterruptHandler::new().with_width(80);
        assert_eq!(handler.width, 80);
    }

    #[test]
    fn test_trigger_and_check_interrupt() {
        let handler = InterruptHandler::new();
        assert!(!handler.is_interrupted());

        handler.trigger_interrupt();
        assert!(handler.is_interrupted());

        handler.reset();
        assert!(!handler.is_interrupted());
    }

    #[test]
    fn test_cancel_flag_sharing() {
        let handler = InterruptHandler::new();
        let flag1 = handler.cancel_flag();
        let flag2 = handler.cancel_flag();

        // Both flags should reference the same atomic
        flag1.store(true, Ordering::SeqCst);
        assert!(flag2.load(Ordering::SeqCst));
        assert!(handler.is_interrupted());
    }

    #[test]
    fn test_render_interrupt_panel_with_story() {
        let handler = InterruptHandler::new().with_width(50);
        let output = handler.render_interrupt_panel(Some("US-005"));

        // Check structure
        assert!(output.contains("╭"));
        assert!(output.contains("╯"));
        assert!(output.contains("Interruption Received"));

        // Check content
        assert!(output.contains("graceful cleanup"));
        assert!(output.contains("US-005"));
        assert!(output.contains("retry"));
        assert!(output.contains("State saved"));
    }

    #[test]
    fn test_render_interrupt_panel_without_story() {
        let handler = InterruptHandler::new().with_width(50);
        let output = handler.render_interrupt_panel(None);

        // Check structure
        assert!(output.contains("Interruption Received"));

        // Check content
        assert!(output.contains("No story in progress"));
    }

    #[test]
    fn test_render_interrupt_message_with_story() {
        let handler = InterruptHandler::new();
        let output = handler.render_interrupt_message(Some("US-001"));

        assert!(output.contains("Interrupted"));
        assert!(output.contains("US-001"));
        assert!(output.contains("Retry"));
    }

    #[test]
    fn test_render_interrupt_message_without_story() {
        let handler = InterruptHandler::new();
        let output = handler.render_interrupt_message(None);

        assert!(output.contains("Interrupted"));
        assert!(!output.contains("Retry"));
    }

    #[test]
    fn test_render_cleanup_progress() {
        let handler = InterruptHandler::new();
        let output = handler.render_cleanup_progress("Saving state...");

        assert!(output.contains("Saving state..."));
        assert!(output.contains("◉"));
    }

    #[test]
    fn test_global_interrupt_functions() {
        // Reset first to ensure clean state
        reset_global_interrupt();
        assert!(!is_globally_interrupted());

        // Trigger via handler
        let handler = InterruptHandler::new();
        handler.trigger_interrupt();
        assert!(is_globally_interrupted());

        // Reset
        reset_global_interrupt();
        assert!(!is_globally_interrupted());
    }

    #[test]
    fn test_render_interrupt_panel_convenience_function() {
        let theme = Theme::default();
        let output = render_interrupt_panel(&theme, Some("US-010"), 60);

        assert!(output.contains("Interruption Received"));
        assert!(output.contains("US-010"));
    }
}
