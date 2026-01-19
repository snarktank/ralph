//! Parallel display controller for concurrent story execution.
//!
//! This module provides a display controller that manages multiple concurrent
//! progress indicators for parallel story execution. It uses indicatif's
//! MultiProgress for thread-safe progress management.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

use crate::ui::colors::Theme;
use crate::ui::display::DisplayOptions;
use crate::ui::parallel_events::{StoryDisplayInfo, StoryStatus};
use crate::ui::spinner::spinner_chars;

/// Progress state for a single story in parallel execution.
#[derive(Debug)]
struct StoryProgressState {
    /// The progress bar for this story
    progress_bar: ProgressBar,
    /// Story display information
    info: StoryDisplayInfo,
    /// Current status
    status: StoryStatus,
    /// Current iteration (1-indexed)
    iteration: u32,
    /// Maximum iterations allowed
    max_iterations: u32,
}

/// Display controller for parallel story execution.
///
/// Manages multiple concurrent progress indicators using indicatif's
/// MultiProgress, providing real-time visual feedback for all in-flight
/// stories during parallel execution.
#[derive(Debug)]
pub struct ParallelRunnerDisplay {
    /// Thread-safe multi-progress manager
    multi_progress: Arc<MultiProgress>,
    /// Per-story progress bars indexed by story ID
    story_progress: HashMap<String, ProgressBar>,
    /// Color theme for consistent styling
    theme: Theme,
    /// Display options (colors, verbosity, etc.)
    display_options: DisplayOptions,
    /// Whether colors are enabled
    colors_enabled: bool,
    /// Maximum concurrent workers (for display purposes)
    max_workers: u32,
}

impl Default for ParallelRunnerDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelRunnerDisplay {
    /// Create a new ParallelRunnerDisplay with default settings.
    pub fn new() -> Self {
        let theme = Theme::default();
        let display_options = DisplayOptions::default();
        let colors_enabled = display_options.should_enable_colors();

        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            story_progress: HashMap::new(),
            theme,
            display_options,
            colors_enabled,
            max_workers: 3, // Default concurrency
        }
    }

    /// Create a ParallelRunnerDisplay with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        let display_options = DisplayOptions::default();
        let colors_enabled = display_options.should_enable_colors();

        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            story_progress: HashMap::new(),
            theme,
            display_options,
            colors_enabled,
            max_workers: 3,
        }
    }

    /// Create a ParallelRunnerDisplay with custom display options.
    pub fn with_display_options(options: DisplayOptions) -> Self {
        let theme = Theme::default();
        let colors_enabled = options.should_enable_colors();

        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            story_progress: HashMap::new(),
            theme,
            display_options: options,
            colors_enabled,
            max_workers: 3,
        }
    }

    /// Create a ParallelRunnerDisplay with both custom theme and display options.
    pub fn with_theme_and_options(theme: Theme, options: DisplayOptions) -> Self {
        let colors_enabled = options.should_enable_colors();

        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            story_progress: HashMap::new(),
            theme,
            display_options: options,
            colors_enabled,
            max_workers: 3,
        }
    }

    /// Set the maximum number of concurrent workers.
    pub fn with_max_workers(mut self, max_workers: u32) -> Self {
        self.max_workers = max_workers;
        self
    }

    /// Set the maximum number of concurrent workers (mutable).
    pub fn set_max_workers(&mut self, max_workers: u32) {
        self.max_workers = max_workers;
    }

    /// Get the maximum number of concurrent workers.
    pub fn max_workers(&self) -> u32 {
        self.max_workers
    }

    /// Get the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Get the display options.
    pub fn display_options(&self) -> &DisplayOptions {
        &self.display_options
    }

    /// Check if colors are enabled.
    pub fn colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    /// Get the underlying MultiProgress instance.
    pub fn multi_progress(&self) -> &MultiProgress {
        &self.multi_progress
    }

    /// Get an Arc reference to the MultiProgress for sharing across threads.
    pub fn multi_progress_arc(&self) -> Arc<MultiProgress> {
        Arc::clone(&self.multi_progress)
    }

    /// Display the parallel execution header with worker count.
    ///
    /// Shows a banner indicating parallel mode is active and the number of workers.
    pub fn display_header(&self, story_count: usize) {
        if self.display_options.quiet {
            return;
        }

        // Format worker count display (handle "unlimited" case)
        let workers_display = if self.max_workers == u32::MAX {
            "unlimited".to_string()
        } else {
            self.max_workers.to_string()
        };

        println!();
        if self.colors_enabled {
            let header_rgb = self.theme.active;
            println!(
                "\x1b[38;2;{};{};{}m╔════════════════════════════════════════════════════════════╗\x1b[0m",
                header_rgb.0, header_rgb.1, header_rgb.2
            );
            println!(
                "\x1b[38;2;{};{};{}m║               🥋 RALPH PARALLEL MODE                        ║\x1b[0m",
                header_rgb.0, header_rgb.1, header_rgb.2
            );
            println!(
                "\x1b[38;2;{};{};{}m╚════════════════════════════════════════════════════════════╝\x1b[0m",
                header_rgb.0, header_rgb.1, header_rgb.2
            );
        } else {
            println!("╔════════════════════════════════════════════════════════════╗");
            println!("║               RALPH PARALLEL MODE                          ║");
            println!("╚════════════════════════════════════════════════════════════╝");
        }
        println!();
        println!(
            "  Workers: {}  |  Stories: {}",
            workers_display, story_count
        );
        println!();
    }

    /// Display completion message for parallel execution.
    ///
    /// Shows a summary of completed execution including story counts and iterations.
    pub fn display_completion(&self, completed: usize, total: usize, iterations: u32) {
        if self.display_options.quiet {
            return;
        }

        println!();
        if self.colors_enabled {
            let success_rgb = self.theme.success;
            if completed == total {
                println!(
                    "\x1b[38;2;{};{};{}m╔═══════════════════════════════════════════════════════════╗\x1b[0m",
                    success_rgb.0, success_rgb.1, success_rgb.2
                );
                println!(
                    "\x1b[38;2;{};{};{}m║  🎉 ALL {} STORIES COMPLETE! 🎉                           ║\x1b[0m",
                    success_rgb.0, success_rgb.1, success_rgb.2, total
                );
                println!(
                    "\x1b[38;2;{};{};{}m╚═══════════════════════════════════════════════════════════╝\x1b[0m",
                    success_rgb.0, success_rgb.1, success_rgb.2
                );
            } else {
                let warning_rgb = self.theme.warning;
                println!(
                    "\x1b[38;2;{};{};{}m╔═══════════════════════════════════════════════════════════╗\x1b[0m",
                    warning_rgb.0, warning_rgb.1, warning_rgb.2
                );
                println!(
                    "\x1b[38;2;{};{};{}m║  ⚠️  {}/{} STORIES COMPLETE                                ║\x1b[0m",
                    warning_rgb.0, warning_rgb.1, warning_rgb.2, completed, total
                );
                println!(
                    "\x1b[38;2;{};{};{}m╚═══════════════════════════════════════════════════════════╝\x1b[0m",
                    warning_rgb.0, warning_rgb.1, warning_rgb.2
                );
            }
        } else if completed == total {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!(
                "║  ALL {} STORIES COMPLETE!                                 ║",
                total
            );
            println!("╚═══════════════════════════════════════════════════════════╝");
        } else {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!(
                "║  {}/{} STORIES COMPLETE                                   ║",
                completed, total
            );
            println!("╚═══════════════════════════════════════════════════════════╝");
        }

        if iterations > 0 {
            println!();
            println!("  Total iterations: {}", iterations);
        }
        println!();
    }

    /// Initialize progress bars for all stories that will be executed.
    ///
    /// This sets up a progress bar for each story in the execution queue,
    /// allowing them to be updated independently as stories progress.
    ///
    /// # Arguments
    /// * `stories` - List of story display information for all stories to track
    pub fn init_stories(&mut self, stories: &[StoryDisplayInfo]) {
        // Clear any existing progress bars first
        self.clear();

        // Display header with worker count
        self.display_header(stories.len());

        // Create a progress bar for each story
        for story in stories {
            let pb = self.create_story_progress_bar(&story.id, &story.title);
            let pb = self.multi_progress.add(pb);
            self.story_progress.insert(story.id.clone(), pb);
        }
    }

    /// Create a styled progress bar for a story.
    fn create_story_progress_bar(&self, story_id: &str, title: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();

        // Create style with theme colors
        let style = self.create_pending_style();
        pb.set_style(style);

        // Set initial message with story ID and title
        let message = self.format_story_message(story_id, title, StoryStatus::Pending, None);
        pb.set_message(message);

        pb
    }

    /// Create a progress style for pending stories.
    fn create_pending_style(&self) -> ProgressStyle {
        let spinner_chars = spinner_chars::BRAILLE.join("");

        // Fall back to simple output when colors are not enabled
        let template = if self.colors_enabled {
            let muted_rgb = self.theme.muted;
            format!(
                "{{spinner:.color({},{},{})}} {{msg}}",
                muted_rgb.0, muted_rgb.1, muted_rgb.2
            )
        } else {
            "{spinner} {msg}".to_string()
        };

        ProgressStyle::with_template(&template)
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
            .tick_strings(&[&spinner_chars, StoryStatus::Pending.icon()])
    }

    /// Create a progress style for in-progress stories.
    fn create_in_progress_style(&self) -> ProgressStyle {
        let spinner_chars = spinner_chars::BRAILLE.join("");

        // Fall back to simple output when colors are not enabled
        let template = if self.colors_enabled {
            let in_progress_rgb = self.theme.in_progress;
            format!(
                "{{spinner:.color({},{},{})}} [{{elapsed_precise}}] {{msg}}",
                in_progress_rgb.0, in_progress_rgb.1, in_progress_rgb.2
            )
        } else {
            "{spinner} [{elapsed_precise}] {msg}".to_string()
        };

        ProgressStyle::with_template(&template)
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
            .tick_strings(&[&spinner_chars, StoryStatus::InProgress.icon()])
    }

    /// Format the message for a story progress bar.
    fn format_story_message(
        &self,
        story_id: &str,
        title: &str,
        status: StoryStatus,
        iteration_info: Option<(u32, u32)>,
    ) -> String {
        let status_icon = status.icon();
        let status_color = self.get_status_color(status);

        let styled_icon = if self.colors_enabled {
            format!("{}", status_icon.color(status_color))
        } else {
            status_icon.to_string()
        };

        let styled_id = if self.colors_enabled {
            format!("{}", story_id.color(self.theme.story_id))
        } else {
            story_id.to_string()
        };

        // Truncate title if too long
        let max_title_len = 40;
        let display_title = if title.len() > max_title_len {
            format!("{}...", &title[..max_title_len - 3])
        } else {
            title.to_string()
        };

        let mut message = format!("{} {} - {}", styled_icon, styled_id, display_title);

        // Add iteration info with visual progress bar if available
        if let Some((current, max)) = iteration_info {
            let iter_text = format!("[{}/{}]", current, max);
            let progress_bar = Self::format_progress_bar(current, max);
            let styled_iter = if self.colors_enabled {
                format!(
                    "{} {}",
                    iter_text.color(self.theme.muted),
                    progress_bar.color(self.theme.in_progress)
                )
            } else {
                format!("{} {}", iter_text, progress_bar)
            };
            message.push_str(&format!(" {}", styled_iter));
        }

        message
    }

    /// Format a visual progress bar string.
    ///
    /// # Arguments
    /// * `current` - Current iteration (1-indexed)
    /// * `max` - Maximum iterations
    ///
    /// # Returns
    /// A string like "====--" representing progress
    fn format_progress_bar(current: u32, max: u32) -> String {
        // Use a fixed width for the progress bar
        const BAR_WIDTH: u32 = 6;

        if max == 0 {
            return "-".repeat(BAR_WIDTH as usize);
        }

        // Calculate filled portion (current is 1-indexed, so current-1 iterations are complete)
        let completed = current.saturating_sub(1);
        let filled = ((completed as f64 / max as f64) * BAR_WIDTH as f64).round() as u32;
        let empty = BAR_WIDTH.saturating_sub(filled);

        format!(
            "{}{}",
            "=".repeat(filled as usize),
            "-".repeat(empty as usize)
        )
    }

    /// Get the theme color for a story status.
    fn get_status_color(&self, status: StoryStatus) -> owo_colors::Rgb {
        match status {
            StoryStatus::Pending => self.theme.muted,
            StoryStatus::InProgress => self.theme.in_progress,
            StoryStatus::Completed => self.theme.success,
            StoryStatus::Failed => self.theme.error,
            StoryStatus::Deferred => self.theme.warning,
            StoryStatus::SequentialRetry => self.theme.active,
        }
    }

    /// Update a story's status to in-progress.
    ///
    /// # Arguments
    /// * `story_id` - The story identifier
    /// * `title` - The story title
    /// * `iteration` - Current iteration number (1-indexed)
    /// * `max_iterations` - Maximum allowed iterations
    pub fn story_started(&self, story_id: &str, title: &str, iteration: u32, max_iterations: u32) {
        if let Some(pb) = self.story_progress.get(story_id) {
            pb.set_style(self.create_in_progress_style());
            pb.enable_steady_tick(Duration::from_millis(80));

            let message = self.format_story_message(
                story_id,
                title,
                StoryStatus::InProgress,
                Some((iteration, max_iterations)),
            );
            pb.set_message(message);
        }
    }

    /// Update a story's iteration progress.
    ///
    /// # Arguments
    /// * `story_id` - The story identifier
    /// * `title` - The story title
    /// * `iteration` - Current iteration number (1-indexed)
    /// * `max_iterations` - Maximum allowed iterations
    pub fn update_iteration(
        &self,
        story_id: &str,
        title: &str,
        iteration: u32,
        max_iterations: u32,
    ) {
        if let Some(pb) = self.story_progress.get(story_id) {
            let message = self.format_story_message(
                story_id,
                title,
                StoryStatus::InProgress,
                Some((iteration, max_iterations)),
            );
            pb.set_message(message);
        }
    }

    /// Mark a story as completed successfully.
    ///
    /// # Arguments
    /// * `story_id` - The story identifier
    /// * `title` - The story title
    /// * `iterations_used` - Total iterations taken
    /// * `commit_hash` - Optional commit hash for the completed story
    pub fn story_completed(
        &self,
        story_id: &str,
        title: &str,
        iterations_used: u32,
        commit_hash: Option<&str>,
    ) {
        if let Some(pb) = self.story_progress.get(story_id) {
            let message = self.format_story_message(story_id, title, StoryStatus::Completed, None);

            // Build iteration info
            let iter_info = if self.colors_enabled {
                format!(
                    "{}",
                    format!("({} iterations)", iterations_used).color(self.theme.muted)
                )
            } else {
                format!("({} iterations)", iterations_used)
            };

            // Add commit hash if provided
            let final_message = if let Some(hash) = commit_hash {
                let commit_display = if self.colors_enabled {
                    format!("{}", hash.color(self.theme.story_id))
                } else {
                    hash.to_string()
                };
                format!("{} {} [{}]", message, iter_info, commit_display)
            } else {
                format!("{} {}", message, iter_info)
            };

            pb.finish_with_message(final_message);
        }
    }

    /// Mark a story as failed.
    ///
    /// # Arguments
    /// * `story_id` - The story identifier
    /// * `title` - The story title
    /// * `error` - Error message
    pub fn story_failed(&self, story_id: &str, title: &str, error: &str) {
        if let Some(pb) = self.story_progress.get(story_id) {
            let message = self.format_story_message(story_id, title, StoryStatus::Failed, None);
            // Truncate error if too long
            let max_error_len = 50;
            let display_error = if error.len() > max_error_len {
                format!("{}...", &error[..max_error_len - 3])
            } else {
                error.to_string()
            };
            let final_message = format!(
                "{} - {}",
                message,
                if self.colors_enabled {
                    format!("{}", display_error.color(self.theme.error))
                } else {
                    display_error
                }
            );
            pb.finish_with_message(final_message);
        }
    }

    /// Mark a story as deferred due to conflicts.
    ///
    /// # Arguments
    /// * `story_id` - The story identifier
    /// * `title` - The story title
    /// * `blocking_story_id` - ID of the story causing the conflict
    pub fn story_deferred(&self, story_id: &str, title: &str, blocking_story_id: &str) {
        if let Some(pb) = self.story_progress.get(story_id) {
            let message = self.format_story_message(story_id, title, StoryStatus::Deferred, None);
            let final_message = format!(
                "{} {}",
                message,
                if self.colors_enabled {
                    format!(
                        "{}",
                        format!("(blocked by {})", blocking_story_id).color(self.theme.warning)
                    )
                } else {
                    format!("(blocked by {})", blocking_story_id)
                }
            );
            pb.set_message(final_message);
        }
    }

    /// Mark a story as retrying in sequential mode.
    ///
    /// # Arguments
    /// * `story_id` - The story identifier
    /// * `title` - The story title
    /// * `reason` - Reason for sequential retry
    pub fn story_sequential_retry(&self, story_id: &str, title: &str, reason: &str) {
        if let Some(pb) = self.story_progress.get(story_id) {
            pb.set_style(self.create_in_progress_style());
            pb.enable_steady_tick(Duration::from_millis(80));

            let message =
                self.format_story_message(story_id, title, StoryStatus::SequentialRetry, None);
            let final_message = format!(
                "{} {}",
                message,
                if self.colors_enabled {
                    format!("{}", format!("({})", reason).color(self.theme.active))
                } else {
                    format!("({})", reason)
                }
            );
            pb.set_message(final_message);
        }
    }

    /// Get a progress bar for a specific story.
    pub fn get_story_progress(&self, story_id: &str) -> Option<&ProgressBar> {
        self.story_progress.get(story_id)
    }

    /// Check if a story has a progress bar.
    pub fn has_story(&self, story_id: &str) -> bool {
        self.story_progress.contains_key(story_id)
    }

    /// Get the number of tracked stories.
    pub fn story_count(&self) -> usize {
        self.story_progress.len()
    }

    /// Clear all progress bars.
    pub fn clear(&mut self) {
        // Finish and clear all progress bars
        for pb in self.story_progress.values() {
            pb.finish_and_clear();
        }
        self.story_progress.clear();

        // Clear the multi-progress
        let _ = self.multi_progress.clear();
    }

    /// Finish all progress bars without clearing them.
    pub fn finish_all(&self) {
        for pb in self.story_progress.values() {
            pb.finish();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default() {
        let display = ParallelRunnerDisplay::new();
        assert_eq!(display.story_count(), 0);
        assert!(display.colors_enabled()); // Default should have colors enabled
    }

    #[test]
    fn test_with_theme() {
        let theme = Theme::default();
        let display = ParallelRunnerDisplay::with_theme(theme);
        assert_eq!(display.theme().success, theme.success);
    }

    #[test]
    fn test_with_display_options() {
        let options = DisplayOptions::new().with_quiet(true);
        let display = ParallelRunnerDisplay::with_display_options(options);
        assert!(display.display_options().quiet);
    }

    #[test]
    fn test_init_stories() {
        let mut display = ParallelRunnerDisplay::new();
        let stories = vec![
            StoryDisplayInfo::new("US-001", "First Story", 1),
            StoryDisplayInfo::new("US-002", "Second Story", 2),
            StoryDisplayInfo::new("US-003", "Third Story", 3),
        ];

        display.init_stories(&stories);

        assert_eq!(display.story_count(), 3);
        assert!(display.has_story("US-001"));
        assert!(display.has_story("US-002"));
        assert!(display.has_story("US-003"));
        assert!(!display.has_story("US-999"));
    }

    #[test]
    fn test_init_stories_clears_previous() {
        let mut display = ParallelRunnerDisplay::new();

        // Initialize with first set
        let stories1 = vec![StoryDisplayInfo::new("US-001", "First", 1)];
        display.init_stories(&stories1);
        assert_eq!(display.story_count(), 1);

        // Initialize with second set - should clear previous
        let stories2 = vec![
            StoryDisplayInfo::new("US-002", "Second", 1),
            StoryDisplayInfo::new("US-003", "Third", 2),
        ];
        display.init_stories(&stories2);
        assert_eq!(display.story_count(), 2);
        assert!(!display.has_story("US-001"));
        assert!(display.has_story("US-002"));
    }

    #[test]
    fn test_clear() {
        let mut display = ParallelRunnerDisplay::new();
        let stories = vec![StoryDisplayInfo::new("US-001", "Test", 1)];
        display.init_stories(&stories);

        assert_eq!(display.story_count(), 1);

        display.clear();
        assert_eq!(display.story_count(), 0);
    }

    #[test]
    fn test_format_story_message() {
        let display = ParallelRunnerDisplay::new();

        let message =
            display.format_story_message("US-001", "Test Story", StoryStatus::Pending, None);
        assert!(message.contains("US-001"));
        assert!(message.contains("Test Story"));

        let message_with_iter = display.format_story_message(
            "US-001",
            "Test Story",
            StoryStatus::InProgress,
            Some((2, 5)),
        );
        assert!(message_with_iter.contains("[2/5]"));
    }

    #[test]
    fn test_format_story_message_truncates_long_title() {
        let display = ParallelRunnerDisplay::new();
        let long_title = "This is a very long story title that should be truncated to fit";

        let message =
            display.format_story_message("US-001", long_title, StoryStatus::Pending, None);
        assert!(message.contains("..."));
        assert!(!message.contains("truncated to fit"));
    }

    #[test]
    fn test_multi_progress_arc() {
        let display = ParallelRunnerDisplay::new();
        let arc1 = display.multi_progress_arc();
        let arc2 = display.multi_progress_arc();

        // Both should point to the same MultiProgress
        assert!(Arc::ptr_eq(&arc1, &arc2));
    }

    #[test]
    fn test_get_status_color() {
        let display = ParallelRunnerDisplay::new();

        assert_eq!(
            display.get_status_color(StoryStatus::Pending),
            display.theme.muted
        );
        assert_eq!(
            display.get_status_color(StoryStatus::InProgress),
            display.theme.in_progress
        );
        assert_eq!(
            display.get_status_color(StoryStatus::Completed),
            display.theme.success
        );
        assert_eq!(
            display.get_status_color(StoryStatus::Failed),
            display.theme.error
        );
        assert_eq!(
            display.get_status_color(StoryStatus::Deferred),
            display.theme.warning
        );
        assert_eq!(
            display.get_status_color(StoryStatus::SequentialRetry),
            display.theme.active
        );
    }
}
