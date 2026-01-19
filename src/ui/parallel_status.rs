//! Header and status sections for parallel execution UI.
//!
//! This module provides rendering functions for the overall execution status
//! display during parallel story execution, including:
//! - Header with PRD name, agent, and worker count
//! - Overall progress bar
//! - Status counts (running, completed, pending, failed)
//! - In Flight section with active stories
//! - Pending section with blocked stories
//! - Completed section (collapsible)
//! - Toggle hint bar for keyboard controls

#![allow(dead_code)]

use owo_colors::OwoColorize;

use crate::ui::colors::Theme;
use crate::ui::keyboard::ToggleState;
use crate::ui::parallel_events::{StoryDisplayInfo, StoryStatus};
use crate::ui::spinner::progress_chars;

/// State of a story during parallel execution for status display.
#[derive(Debug, Clone)]
pub struct StoryExecutionState {
    /// Story display information
    pub info: StoryDisplayInfo,
    /// Current status
    pub status: StoryStatus,
    /// Current iteration (1-indexed)
    pub iteration: u32,
    /// Maximum iterations allowed
    pub max_iterations: u32,
    /// Blocking story ID (if deferred)
    pub blocked_by: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Commit hash (if completed)
    pub commit_hash: Option<String>,
}

impl StoryExecutionState {
    /// Create a new pending story state.
    pub fn new_pending(info: StoryDisplayInfo, max_iterations: u32) -> Self {
        Self {
            info,
            status: StoryStatus::Pending,
            iteration: 0,
            max_iterations,
            blocked_by: None,
            error: None,
            commit_hash: None,
        }
    }

    /// Create a new in-progress story state.
    pub fn new_in_progress(info: StoryDisplayInfo, iteration: u32, max_iterations: u32) -> Self {
        Self {
            info,
            status: StoryStatus::InProgress,
            iteration,
            max_iterations,
            blocked_by: None,
            error: None,
            commit_hash: None,
        }
    }

    /// Mark as in progress with iteration.
    pub fn start(&mut self, iteration: u32) {
        self.status = StoryStatus::InProgress;
        self.iteration = iteration;
    }

    /// Mark as completed.
    pub fn complete(&mut self, commit_hash: Option<String>) {
        self.status = StoryStatus::Completed;
        self.commit_hash = commit_hash;
    }

    /// Mark as failed.
    pub fn fail(&mut self, error: String) {
        self.status = StoryStatus::Failed;
        self.error = Some(error);
    }

    /// Mark as deferred.
    pub fn defer(&mut self, blocked_by: String) {
        self.status = StoryStatus::Deferred;
        self.blocked_by = Some(blocked_by);
    }
}

/// Overall execution state for parallel story execution.
#[derive(Debug, Clone)]
pub struct ParallelExecutionState {
    /// PRD name being executed
    pub prd_name: String,
    /// Agent being used
    pub agent: String,
    /// Number of parallel workers
    pub worker_count: usize,
    /// All story states
    pub stories: Vec<StoryExecutionState>,
    /// Whether the completed section is collapsed
    pub completed_collapsed: bool,
}

impl ParallelExecutionState {
    /// Create a new parallel execution state.
    pub fn new(prd_name: impl Into<String>, agent: impl Into<String>, worker_count: usize) -> Self {
        Self {
            prd_name: prd_name.into(),
            agent: agent.into(),
            worker_count,
            stories: Vec::new(),
            completed_collapsed: true,
        }
    }

    /// Add a story to the execution state.
    pub fn add_story(&mut self, info: StoryDisplayInfo, max_iterations: u32) {
        self.stories
            .push(StoryExecutionState::new_pending(info, max_iterations));
    }

    /// Get a mutable reference to a story by ID.
    pub fn get_story_mut(&mut self, story_id: &str) -> Option<&mut StoryExecutionState> {
        self.stories.iter_mut().find(|s| s.info.id == story_id)
    }

    /// Get a reference to a story by ID.
    pub fn get_story(&self, story_id: &str) -> Option<&StoryExecutionState> {
        self.stories.iter().find(|s| s.info.id == story_id)
    }

    /// Count stories by status.
    pub fn count_by_status(&self, status: StoryStatus) -> usize {
        self.stories.iter().filter(|s| s.status == status).count()
    }

    /// Get running stories count (InProgress + SequentialRetry).
    pub fn running_count(&self) -> usize {
        self.stories
            .iter()
            .filter(|s| {
                matches!(
                    s.status,
                    StoryStatus::InProgress | StoryStatus::SequentialRetry
                )
            })
            .count()
    }

    /// Get completed stories count.
    pub fn completed_count(&self) -> usize {
        self.count_by_status(StoryStatus::Completed)
    }

    /// Get pending stories count (Pending + Deferred).
    pub fn pending_count(&self) -> usize {
        self.stories
            .iter()
            .filter(|s| matches!(s.status, StoryStatus::Pending | StoryStatus::Deferred))
            .count()
    }

    /// Get failed stories count.
    pub fn failed_count(&self) -> usize {
        self.count_by_status(StoryStatus::Failed)
    }

    /// Get total stories count.
    pub fn total_count(&self) -> usize {
        self.stories.len()
    }

    /// Toggle the collapsed state of the completed section.
    pub fn toggle_completed_collapsed(&mut self) {
        self.completed_collapsed = !self.completed_collapsed;
    }

    /// Get stories that are in flight (running).
    pub fn in_flight_stories(&self) -> Vec<&StoryExecutionState> {
        self.stories
            .iter()
            .filter(|s| {
                matches!(
                    s.status,
                    StoryStatus::InProgress | StoryStatus::SequentialRetry
                )
            })
            .collect()
    }

    /// Get stories that are pending (including deferred).
    pub fn pending_stories(&self) -> Vec<&StoryExecutionState> {
        self.stories
            .iter()
            .filter(|s| matches!(s.status, StoryStatus::Pending | StoryStatus::Deferred))
            .collect()
    }

    /// Get completed stories.
    pub fn completed_stories(&self) -> Vec<&StoryExecutionState> {
        self.stories
            .iter()
            .filter(|s| s.status == StoryStatus::Completed)
            .collect()
    }

    /// Get failed stories.
    pub fn failed_stories(&self) -> Vec<&StoryExecutionState> {
        self.stories
            .iter()
            .filter(|s| s.status == StoryStatus::Failed)
            .collect()
    }
}

/// Renderer for parallel execution status display.
#[derive(Debug)]
pub struct ParallelStatusRenderer {
    /// Color theme for rendering
    theme: Theme,
    /// Panel width (characters)
    width: usize,
    /// Whether colors are enabled
    colors_enabled: bool,
}

impl Default for ParallelStatusRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelStatusRenderer {
    /// Create a new status renderer with default settings.
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            width: 60,
            colors_enabled: true,
        }
    }

    /// Create a renderer with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            theme,
            width: 60,
            colors_enabled: true,
        }
    }

    /// Set the panel width.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set whether colors are enabled.
    pub fn with_colors(mut self, enabled: bool) -> Self {
        self.colors_enabled = enabled;
        self
    }

    /// Get the theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Render the header showing PRD name, agent, and worker count.
    pub fn render_header(&self, state: &ParallelExecutionState) -> String {
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));

        // PRD name line (centered, bold)
        let prd_display = format!(" {} ", state.prd_name);
        let prd_padding = (inner_width.saturating_sub(prd_display.len())) / 2;
        let prd_remainder = inner_width
            .saturating_sub(prd_display.len())
            .saturating_sub(prd_padding);

        if self.colors_enabled {
            output.push_str(&format!(
                "│{}{}{}│\n",
                " ".repeat(prd_padding),
                prd_display.color(self.theme.primary).bold(),
                " ".repeat(prd_remainder)
            ));
        } else {
            output.push_str(&format!(
                "│{}{}{}│\n",
                " ".repeat(prd_padding),
                prd_display,
                " ".repeat(prd_remainder)
            ));
        }

        // Agent and worker count line
        let agent_text = format!("Agent: {}", state.agent);
        let worker_text = format!("Workers: {}", state.worker_count);
        let info_padding = inner_width.saturating_sub(agent_text.len() + worker_text.len() + 4);

        if self.colors_enabled {
            output.push_str(&format!(
                "│ {}{}{}│\n",
                agent_text.color(self.theme.muted),
                " ".repeat(info_padding),
                format!("{} ", worker_text).color(self.theme.in_progress)
            ));
        } else {
            output.push_str(&format!(
                "│ {}{}{} │\n",
                agent_text,
                " ".repeat(info_padding),
                worker_text
            ));
        }

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Render the overall progress bar.
    ///
    /// Format: Stories: [████████░░░░░░░░] 4/10 (40%)
    pub fn render_overall_progress(&self, state: &ParallelExecutionState) -> String {
        let completed = state.completed_count();
        let total = state.total_count();
        let percentage = if total == 0 {
            100
        } else {
            // Use saturating_mul to prevent overflow for large values
            completed.saturating_mul(100) / total
        };

        // Calculate bar width (leaving room for label and counts)
        let label = "Stories: ";
        let count_display = format!(" {}/{} ({}%)", completed, total, percentage);
        let bar_width = self
            .width
            .saturating_sub(label.len() + count_display.len() + 2); // +2 for brackets

        // Calculate filled/empty portions
        let filled_count = if total == 0 {
            bar_width
        } else {
            (completed * bar_width) / total
        };
        let empty_count = bar_width.saturating_sub(filled_count);

        let filled_str = progress_chars::FILLED.repeat(filled_count);
        let empty_str = progress_chars::EMPTY.repeat(empty_count);

        if self.colors_enabled {
            format!(
                "{}[{}{}]{}",
                label.color(self.theme.muted),
                filled_str.color(self.theme.success),
                empty_str.color(self.theme.muted),
                count_display.color(self.theme.primary)
            )
        } else {
            format!("{}[{}{}]{}", label, filled_str, empty_str, count_display)
        }
    }

    /// Render the status counts (running, completed, pending, failed).
    pub fn render_status_counts(&self, state: &ParallelExecutionState) -> String {
        let running = state.running_count();
        let completed = state.completed_count();
        let pending = state.pending_count();
        let failed = state.failed_count();

        if self.colors_enabled {
            format!(
                "{} {} {} {} {} {} {} {}",
                format!("{}:", StoryStatus::InProgress.icon())
                    .color(self.theme.in_progress)
                    .bold(),
                running.to_string().color(self.theme.in_progress),
                format!("{}:", StoryStatus::Completed.icon())
                    .color(self.theme.success)
                    .bold(),
                completed.to_string().color(self.theme.success),
                format!("{}:", StoryStatus::Pending.icon())
                    .color(self.theme.muted)
                    .bold(),
                pending.to_string().color(self.theme.muted),
                format!("{}:", StoryStatus::Failed.icon())
                    .color(self.theme.error)
                    .bold(),
                if failed > 0 {
                    failed.to_string().color(self.theme.error).to_string()
                } else {
                    failed.to_string().color(self.theme.muted).to_string()
                }
            )
        } else {
            format!(
                "{}: {} {}: {} {}: {} {}: {}",
                StoryStatus::InProgress.icon(),
                running,
                StoryStatus::Completed.icon(),
                completed,
                StoryStatus::Pending.icon(),
                pending,
                StoryStatus::Failed.icon(),
                failed
            )
        }
    }

    /// Render the In Flight section with active stories.
    pub fn render_in_flight_section(&self, state: &ParallelExecutionState) -> String {
        let mut output = String::new();
        let in_flight = state.in_flight_stories();

        if in_flight.is_empty() {
            return output;
        }

        let inner_width = self.width - 2;

        // Section header
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));

        let header = " In Flight ";
        let count_display = format!("({})", in_flight.len());
        let header_padding = inner_width.saturating_sub(header.len() + count_display.len() + 2);

        if self.colors_enabled {
            output.push_str(&format!(
                "│{}{}{}{}│\n",
                header.color(self.theme.in_progress).bold(),
                " ".repeat(header_padding),
                count_display.color(self.theme.muted),
                " "
            ));
        } else {
            output.push_str(&format!(
                "│{}{}{}{}│\n",
                header,
                " ".repeat(header_padding),
                count_display,
                " "
            ));
        }

        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Render each in-flight story
        for story in in_flight {
            let line = self.format_in_flight_story(story, inner_width);
            output.push_str(&format!("│{}│\n", line));
        }

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Format a single in-flight story line.
    fn format_in_flight_story(&self, story: &StoryExecutionState, width: usize) -> String {
        let spinner_icon = story.status.icon();
        let id = &story.info.id;
        let iteration_info = format!("[{}/{}]", story.iteration, story.max_iterations);

        // Calculate title width
        let prefix_len = 3 + id.len() + 3 + iteration_info.len() + 1; // " ◉ ID - [x/y] "
        let title_width = width.saturating_sub(prefix_len + 1);
        let title = self.truncate_text(&story.info.title, title_width);

        // Calculate padding
        let content_len = prefix_len + title.len();
        let padding = width.saturating_sub(content_len);

        if self.colors_enabled {
            let status_color = match story.status {
                StoryStatus::InProgress => self.theme.in_progress,
                StoryStatus::SequentialRetry => self.theme.active,
                _ => self.theme.muted,
            };

            format!(
                " {} {} - {} {}{}",
                spinner_icon.color(status_color),
                id.color(self.theme.story_id),
                title,
                iteration_info.color(self.theme.muted),
                " ".repeat(padding)
            )
        } else {
            format!(
                " {} {} - {} {}{}",
                spinner_icon,
                id,
                title,
                iteration_info,
                " ".repeat(padding)
            )
        }
    }

    /// Render the Pending section showing blocked stories with blocking reason.
    pub fn render_pending_section(&self, state: &ParallelExecutionState) -> String {
        let mut output = String::new();
        let pending = state.pending_stories();

        if pending.is_empty() {
            return output;
        }

        let inner_width = self.width - 2;

        // Section header
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));

        let header = " Pending ";
        let count_display = format!("({})", pending.len());
        let header_padding = inner_width.saturating_sub(header.len() + count_display.len() + 2);

        if self.colors_enabled {
            output.push_str(&format!(
                "│{}{}{}{}│\n",
                header.color(self.theme.muted).bold(),
                " ".repeat(header_padding),
                count_display.color(self.theme.muted),
                " "
            ));
        } else {
            output.push_str(&format!(
                "│{}{}{}{}│\n",
                header,
                " ".repeat(header_padding),
                count_display,
                " "
            ));
        }

        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Render each pending story
        for story in pending {
            let line = self.format_pending_story(story, inner_width);
            output.push_str(&format!("│{}│\n", line));
        }

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Format a single pending story line.
    fn format_pending_story(&self, story: &StoryExecutionState, width: usize) -> String {
        let status_icon = story.status.icon();
        let id = &story.info.id;

        // Build blocking reason if deferred
        let blocking_info = if story.status == StoryStatus::Deferred {
            if let Some(ref blocked_by) = story.blocked_by {
                format!("(blocked by {})", blocked_by)
            } else {
                "(deferred)".to_string()
            }
        } else {
            String::new()
        };

        // Calculate title width
        let prefix_len = 3 + id.len() + 3; // " ○ ID - "
        let suffix_len = if blocking_info.is_empty() {
            0
        } else {
            blocking_info.len() + 1
        };
        let title_width = width.saturating_sub(prefix_len + suffix_len + 1);
        let title = self.truncate_text(&story.info.title, title_width);

        // Calculate padding
        let content_len = prefix_len + title.len() + suffix_len;
        let padding = width.saturating_sub(content_len);

        if self.colors_enabled {
            let status_color = if story.status == StoryStatus::Deferred {
                self.theme.warning
            } else {
                self.theme.muted
            };

            if blocking_info.is_empty() {
                format!(
                    " {} {} - {}{}",
                    status_icon.color(status_color),
                    id.color(self.theme.story_id),
                    title.color(self.theme.muted),
                    " ".repeat(padding)
                )
            } else {
                format!(
                    " {} {} - {} {}{}",
                    status_icon.color(status_color),
                    id.color(self.theme.story_id),
                    title.color(self.theme.muted),
                    blocking_info.color(self.theme.warning),
                    " ".repeat(padding)
                )
            }
        } else if blocking_info.is_empty() {
            format!(" {} {} - {}{}", status_icon, id, title, " ".repeat(padding))
        } else {
            format!(
                " {} {} - {} {}{}",
                status_icon,
                id,
                title,
                blocking_info,
                " ".repeat(padding)
            )
        }
    }

    /// Render the Completed section (collapsible).
    pub fn render_completed_section(&self, state: &ParallelExecutionState) -> String {
        let mut output = String::new();
        let completed = state.completed_stories();

        if completed.is_empty() {
            return output;
        }

        let inner_width = self.width - 2;

        // Section header
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));

        let collapse_icon = if state.completed_collapsed {
            "▸"
        } else {
            "▾"
        };
        let header = " Completed ";
        let count_display = format!("({})", completed.len());
        let header_padding = inner_width
            .saturating_sub(collapse_icon.len() + header.len() + count_display.len() + 3);

        if self.colors_enabled {
            output.push_str(&format!(
                "│ {}{}{}{}{}│\n",
                collapse_icon.color(self.theme.muted),
                header.color(self.theme.success).bold(),
                " ".repeat(header_padding),
                count_display.color(self.theme.muted),
                " "
            ));
        } else {
            output.push_str(&format!(
                "│ {}{}{}{}{}│\n",
                collapse_icon,
                header,
                " ".repeat(header_padding),
                count_display,
                " "
            ));
        }

        // If not collapsed, show the stories
        if !state.completed_collapsed {
            output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

            for story in completed {
                let line = self.format_completed_story(story, inner_width);
                output.push_str(&format!("│{}│\n", line));
            }
        }

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Format a single completed story line.
    fn format_completed_story(&self, story: &StoryExecutionState, width: usize) -> String {
        let status_icon = StoryStatus::Completed.icon();
        let id = &story.info.id;

        // Build commit hash if available
        let commit_info = if let Some(ref hash) = story.commit_hash {
            // Show short hash (first 7 chars) - use chars() for UTF-8 safety
            let short_hash: String = hash.chars().take(7).collect();
            format!("[{}]", short_hash)
        } else {
            String::new()
        };

        // Calculate title width
        let prefix_len = 3 + id.len() + 3; // " ✓ ID - "
        let suffix_len = if commit_info.is_empty() {
            0
        } else {
            commit_info.len() + 1
        };
        let title_width = width.saturating_sub(prefix_len + suffix_len + 1);
        let title = self.truncate_text(&story.info.title, title_width);

        // Calculate padding
        let content_len = prefix_len + title.len() + suffix_len;
        let padding = width.saturating_sub(content_len);

        if self.colors_enabled {
            if commit_info.is_empty() {
                format!(
                    " {} {} - {}{}",
                    status_icon.color(self.theme.success),
                    id.color(self.theme.story_id),
                    title.color(self.theme.completed),
                    " ".repeat(padding)
                )
            } else {
                format!(
                    " {} {} - {} {}{}",
                    status_icon.color(self.theme.success),
                    id.color(self.theme.story_id),
                    title.color(self.theme.completed),
                    commit_info.color(self.theme.story_id),
                    " ".repeat(padding)
                )
            }
        } else if commit_info.is_empty() {
            format!(" {} {} - {}{}", status_icon, id, title, " ".repeat(padding))
        } else {
            format!(
                " {} {} - {} {}{}",
                status_icon,
                id,
                title,
                commit_info,
                " ".repeat(padding)
            )
        }
    }

    /// Render the Failed section.
    pub fn render_failed_section(&self, state: &ParallelExecutionState) -> String {
        let mut output = String::new();
        let failed = state.failed_stories();

        if failed.is_empty() {
            return output;
        }

        let inner_width = self.width - 2;

        // Section header
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));

        let header = " Failed ";
        let count_display = format!("({})", failed.len());
        let header_padding = inner_width.saturating_sub(header.len() + count_display.len() + 2);

        if self.colors_enabled {
            output.push_str(&format!(
                "│{}{}{}{}│\n",
                header.color(self.theme.error).bold(),
                " ".repeat(header_padding),
                count_display.color(self.theme.muted),
                " "
            ));
        } else {
            output.push_str(&format!(
                "│{}{}{}{}│\n",
                header,
                " ".repeat(header_padding),
                count_display,
                " "
            ));
        }

        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Render each failed story
        for story in failed {
            let line = self.format_failed_story(story, inner_width);
            output.push_str(&format!("│{}│\n", line));
        }

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Format a single failed story line.
    fn format_failed_story(&self, story: &StoryExecutionState, width: usize) -> String {
        let status_icon = StoryStatus::Failed.icon();
        let id = &story.info.id;

        // Build error info if available
        let error_info = if let Some(ref error) = story.error {
            let truncated_error = self.truncate_text(error, 30);
            format!("- {}", truncated_error)
        } else {
            String::new()
        };

        // Calculate title width
        let prefix_len = 3 + id.len() + 3; // " ✗ ID - "
        let suffix_len = if error_info.is_empty() {
            0
        } else {
            error_info.len() + 1
        };
        let title_width = width.saturating_sub(prefix_len + suffix_len + 1);
        let title = self.truncate_text(&story.info.title, title_width);

        // Calculate padding
        let content_len = prefix_len + title.len() + suffix_len;
        let padding = width.saturating_sub(content_len);

        if self.colors_enabled {
            if error_info.is_empty() {
                format!(
                    " {} {} - {}{}",
                    status_icon.color(self.theme.error),
                    id.color(self.theme.story_id),
                    title,
                    " ".repeat(padding)
                )
            } else {
                format!(
                    " {} {} - {} {}{}",
                    status_icon.color(self.theme.error),
                    id.color(self.theme.story_id),
                    title,
                    error_info.color(self.theme.error),
                    " ".repeat(padding)
                )
            }
        } else if error_info.is_empty() {
            format!(" {} {} - {}{}", status_icon, id, title, " ".repeat(padding))
        } else {
            format!(
                " {} {} - {} {}{}",
                status_icon,
                id,
                title,
                error_info,
                " ".repeat(padding)
            )
        }
    }

    /// Render the complete status display.
    pub fn render(&self, state: &ParallelExecutionState) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.render_header(state));
        output.push('\n');

        // Overall progress
        output.push_str(&self.render_overall_progress(state));
        output.push('\n');

        // Status counts
        output.push_str(&self.render_status_counts(state));
        output.push('\n');
        output.push('\n');

        // In Flight section
        let in_flight = self.render_in_flight_section(state);
        if !in_flight.is_empty() {
            output.push_str(&in_flight);
            output.push('\n');
            output.push('\n');
        }

        // Pending section
        let pending = self.render_pending_section(state);
        if !pending.is_empty() {
            output.push_str(&pending);
            output.push('\n');
            output.push('\n');
        }

        // Failed section
        let failed = self.render_failed_section(state);
        if !failed.is_empty() {
            output.push_str(&failed);
            output.push('\n');
            output.push('\n');
        }

        // Completed section
        let completed = self.render_completed_section(state);
        if !completed.is_empty() {
            output.push_str(&completed);
        }

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

    /// Render the toggle hint bar showing keyboard controls.
    ///
    /// Format: [s] stream: off | [e] expand: off | [q] quit
    pub fn render_hint_bar(&self, toggle_state: &ToggleState) -> String {
        let streaming_status = if toggle_state.should_show_streaming() {
            "on"
        } else {
            "off"
        };
        let expand_status = if toggle_state.should_expand_details() {
            "on"
        } else {
            "off"
        };

        if self.colors_enabled {
            format!(
                "{} {} {} {} {} {} {} {}",
                "[s]".color(self.theme.active).bold(),
                format!("stream: {}", streaming_status).color(self.theme.muted),
                "│".color(self.theme.muted),
                "[e]".color(self.theme.active).bold(),
                format!("expand: {}", expand_status).color(self.theme.muted),
                "│".color(self.theme.muted),
                "[q]".color(self.theme.warning).bold(),
                "quit".color(self.theme.muted)
            )
        } else {
            format!(
                "[s] stream: {} │ [e] expand: {} │ [q] quit",
                streaming_status, expand_status
            )
        }
    }

    /// Render the complete status display with hint bar.
    pub fn render_with_hints(
        &self,
        state: &ParallelExecutionState,
        toggle_state: &ToggleState,
    ) -> String {
        let mut output = self.render(state);
        output.push('\n');
        output.push_str(&self.render_hint_bar(toggle_state));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> ParallelExecutionState {
        let mut state = ParallelExecutionState::new("Test PRD", "claude-code", 4);

        state.add_story(
            StoryDisplayInfo::new("US-001", "First story implementation", 1),
            5,
        );
        state.add_story(
            StoryDisplayInfo::new("US-002", "Second story implementation", 2),
            5,
        );
        state.add_story(
            StoryDisplayInfo::new("US-003", "Third story implementation", 3),
            5,
        );
        state.add_story(
            StoryDisplayInfo::new("US-004", "Fourth story implementation", 4),
            5,
        );

        state
    }

    #[test]
    fn test_parallel_execution_state_new() {
        let state = ParallelExecutionState::new("My PRD", "claude-code", 4);
        assert_eq!(state.prd_name, "My PRD");
        assert_eq!(state.agent, "claude-code");
        assert_eq!(state.worker_count, 4);
        assert!(state.stories.is_empty());
        assert!(state.completed_collapsed);
    }

    #[test]
    fn test_parallel_execution_state_add_story() {
        let state = create_test_state();
        assert_eq!(state.total_count(), 4);
        assert_eq!(state.pending_count(), 4);
        assert_eq!(state.running_count(), 0);
        assert_eq!(state.completed_count(), 0);
        assert_eq!(state.failed_count(), 0);
    }

    #[test]
    fn test_story_state_transitions() {
        let mut state = create_test_state();

        // Start first story
        if let Some(story) = state.get_story_mut("US-001") {
            story.start(1);
        }
        assert_eq!(state.running_count(), 1);
        assert_eq!(state.pending_count(), 3);

        // Complete first story
        if let Some(story) = state.get_story_mut("US-001") {
            story.complete(Some("abc1234".to_string()));
        }
        assert_eq!(state.completed_count(), 1);
        assert_eq!(state.running_count(), 0);

        // Fail second story
        if let Some(story) = state.get_story_mut("US-002") {
            story.start(1);
            story.fail("Quality gates failed".to_string());
        }
        assert_eq!(state.failed_count(), 1);

        // Defer third story
        if let Some(story) = state.get_story_mut("US-003") {
            story.defer("US-004".to_string());
        }
        assert_eq!(state.pending_count(), 2); // US-003 (deferred) + US-004 (pending)
    }

    #[test]
    fn test_render_header() {
        let state = create_test_state();
        let renderer = ParallelStatusRenderer::new().with_width(60);

        let output = renderer.render_header(&state);
        assert!(output.contains("Test PRD"));
        assert!(output.contains("Agent: claude-code"));
        assert!(output.contains("Workers: 4"));
    }

    #[test]
    fn test_render_overall_progress() {
        let mut state = create_test_state();
        let renderer = ParallelStatusRenderer::new().with_colors(false);

        // No progress yet
        let output = renderer.render_overall_progress(&state);
        assert!(output.contains("0/4"));
        assert!(output.contains("0%"));

        // Complete one story
        if let Some(story) = state.get_story_mut("US-001") {
            story.complete(None);
        }
        let output = renderer.render_overall_progress(&state);
        assert!(output.contains("1/4"));
        assert!(output.contains("25%"));
    }

    #[test]
    fn test_render_status_counts() {
        let mut state = create_test_state();
        let renderer = ParallelStatusRenderer::new().with_colors(false);

        // Start some stories
        if let Some(story) = state.get_story_mut("US-001") {
            story.start(1);
        }
        if let Some(story) = state.get_story_mut("US-002") {
            story.complete(None);
        }
        if let Some(story) = state.get_story_mut("US-003") {
            story.fail("Error".to_string());
        }

        let output = renderer.render_status_counts(&state);
        // Should show: 1 running, 1 completed, 1 pending, 1 failed
        assert!(output.contains("1")); // At least shows counts
    }

    #[test]
    fn test_render_in_flight_section() {
        let mut state = create_test_state();
        let renderer = ParallelStatusRenderer::new().with_colors(false);

        // No in-flight stories yet
        let output = renderer.render_in_flight_section(&state);
        assert!(output.is_empty());

        // Start a story
        if let Some(story) = state.get_story_mut("US-001") {
            story.start(2);
        }

        let output = renderer.render_in_flight_section(&state);
        assert!(output.contains("In Flight"));
        assert!(output.contains("US-001"));
        assert!(output.contains("[2/5]"));
    }

    #[test]
    fn test_render_pending_section() {
        let mut state = create_test_state();
        let renderer = ParallelStatusRenderer::new().with_colors(false);

        // Defer a story
        if let Some(story) = state.get_story_mut("US-002") {
            story.defer("US-001".to_string());
        }

        let output = renderer.render_pending_section(&state);
        assert!(output.contains("Pending"));
        assert!(output.contains("US-002"));
        assert!(output.contains("blocked by US-001"));
    }

    #[test]
    fn test_render_completed_section_collapsed() {
        let mut state = create_test_state();
        let renderer = ParallelStatusRenderer::new().with_colors(false);

        // Complete a story
        if let Some(story) = state.get_story_mut("US-001") {
            story.complete(Some("abc1234def".to_string()));
        }

        // Collapsed by default - should only show header
        let output = renderer.render_completed_section(&state);
        assert!(output.contains("Completed"));
        assert!(output.contains("(1)"));
        assert!(output.contains("▸")); // Collapsed indicator
        assert!(!output.contains("US-001")); // Story not shown when collapsed
    }

    #[test]
    fn test_render_completed_section_expanded() {
        let mut state = create_test_state();
        state.completed_collapsed = false;
        let renderer = ParallelStatusRenderer::new().with_colors(false);

        // Complete a story
        if let Some(story) = state.get_story_mut("US-001") {
            story.complete(Some("abc1234def".to_string()));
        }

        let output = renderer.render_completed_section(&state);
        assert!(output.contains("▾")); // Expanded indicator
        assert!(output.contains("US-001"));
        assert!(output.contains("[abc1234]")); // Short hash
    }

    #[test]
    fn test_render_failed_section() {
        let mut state = create_test_state();
        let renderer = ParallelStatusRenderer::new().with_colors(false);

        // Fail a story
        if let Some(story) = state.get_story_mut("US-001") {
            story.fail("Quality gates failed".to_string());
        }

        let output = renderer.render_failed_section(&state);
        assert!(output.contains("Failed"));
        assert!(output.contains("US-001"));
        assert!(output.contains("Quality gates failed"));
    }

    #[test]
    fn test_truncate_text() {
        let renderer = ParallelStatusRenderer::new();

        assert_eq!(renderer.truncate_text("short", 10), "short");
        assert_eq!(
            renderer.truncate_text("this is a long text", 10),
            "this is..."
        );
        assert_eq!(renderer.truncate_text("abc", 3), "abc");
    }

    #[test]
    fn test_story_execution_state_new_pending() {
        let info = StoryDisplayInfo::new("US-001", "Test Story", 1);
        let state = StoryExecutionState::new_pending(info, 5);

        assert_eq!(state.status, StoryStatus::Pending);
        assert_eq!(state.iteration, 0);
        assert_eq!(state.max_iterations, 5);
        assert!(state.blocked_by.is_none());
        assert!(state.error.is_none());
        assert!(state.commit_hash.is_none());
    }

    #[test]
    fn test_story_execution_state_new_in_progress() {
        let info = StoryDisplayInfo::new("US-001", "Test Story", 1);
        let state = StoryExecutionState::new_in_progress(info, 2, 5);

        assert_eq!(state.status, StoryStatus::InProgress);
        assert_eq!(state.iteration, 2);
        assert_eq!(state.max_iterations, 5);
    }

    #[test]
    fn test_toggle_completed_collapsed() {
        let mut state = ParallelExecutionState::new("PRD", "agent", 2);
        assert!(state.completed_collapsed);

        state.toggle_completed_collapsed();
        assert!(!state.completed_collapsed);

        state.toggle_completed_collapsed();
        assert!(state.completed_collapsed);
    }

    #[test]
    fn test_full_render() {
        let mut state = create_test_state();

        // Set up various states
        if let Some(story) = state.get_story_mut("US-001") {
            story.start(2);
        }
        if let Some(story) = state.get_story_mut("US-002") {
            story.complete(Some("abc1234".to_string()));
        }
        if let Some(story) = state.get_story_mut("US-003") {
            story.fail("Error".to_string());
        }
        // US-004 stays pending

        state.completed_collapsed = false;

        let renderer = ParallelStatusRenderer::new().with_colors(false);
        let output = renderer.render(&state);

        // Should contain all sections
        assert!(output.contains("Test PRD"));
        assert!(output.contains("In Flight"));
        assert!(output.contains("Pending"));
        assert!(output.contains("Completed"));
        assert!(output.contains("Failed"));
    }

    #[test]
    fn test_render_hint_bar_default() {
        let renderer = ParallelStatusRenderer::new().with_colors(false);
        let toggle_state = ToggleState::default();

        let hint = renderer.render_hint_bar(&toggle_state);
        assert!(hint.contains("[s]"));
        assert!(hint.contains("stream: off"));
        assert!(hint.contains("[e]"));
        assert!(hint.contains("expand: off"));
        assert!(hint.contains("[q]"));
        assert!(hint.contains("quit"));
    }

    #[test]
    fn test_render_hint_bar_streaming_on() {
        let renderer = ParallelStatusRenderer::new().with_colors(false);
        let toggle_state = ToggleState::new(true, false);

        let hint = renderer.render_hint_bar(&toggle_state);
        assert!(hint.contains("stream: on"));
        assert!(hint.contains("expand: off"));
    }

    #[test]
    fn test_render_hint_bar_expand_on() {
        let renderer = ParallelStatusRenderer::new().with_colors(false);
        let toggle_state = ToggleState::new(false, true);

        let hint = renderer.render_hint_bar(&toggle_state);
        assert!(hint.contains("stream: off"));
        assert!(hint.contains("expand: on"));
    }

    #[test]
    fn test_render_hint_bar_both_on() {
        let renderer = ParallelStatusRenderer::new().with_colors(false);
        let toggle_state = ToggleState::new(true, true);

        let hint = renderer.render_hint_bar(&toggle_state);
        assert!(hint.contains("stream: on"));
        assert!(hint.contains("expand: on"));
    }

    #[test]
    fn test_render_with_hints() {
        let state = create_test_state();
        let toggle_state = ToggleState::default();
        let renderer = ParallelStatusRenderer::new().with_colors(false);

        let output = renderer.render_with_hints(&state, &toggle_state);

        // Should contain the main content
        assert!(output.contains("Test PRD"));
        // Should contain the hint bar
        assert!(output.contains("[s]"));
        assert!(output.contains("[e]"));
        assert!(output.contains("[q]"));
    }
}
