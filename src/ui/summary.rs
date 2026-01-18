//! Completion summary view for Ralph's terminal UI.
//!
//! Displays a comprehensive summary when all stories complete,
//! including story results, quality gate statistics, and execution metrics.

#![allow(dead_code)]

use std::time::Duration;

use owo_colors::OwoColorize;

use crate::ui::colors::Theme;

/// Result of a single story execution.
#[derive(Debug, Clone)]
pub struct StoryResult {
    /// Story identifier (e.g., "US-001")
    pub id: String,
    /// Story title
    pub title: String,
    /// Whether the story passed
    pub passed: bool,
    /// Number of iterations taken to complete
    pub iterations: u32,
}

impl StoryResult {
    /// Create a new story result.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        passed: bool,
        iterations: u32,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            passed,
            iterations,
        }
    }

    /// Create a passed story result.
    pub fn passed(id: impl Into<String>, title: impl Into<String>, iterations: u32) -> Self {
        Self::new(id, title, true, iterations)
    }

    /// Create a failed story result.
    pub fn failed(id: impl Into<String>, title: impl Into<String>, iterations: u32) -> Self {
        Self::new(id, title, false, iterations)
    }
}

/// Quality gate aggregate statistics.
#[derive(Debug, Clone, Default)]
pub struct GateStatistics {
    /// Total gates run across all stories
    pub total_runs: u32,
    /// Total passes
    pub total_passes: u32,
    /// Total failures
    pub total_failures: u32,
    /// Total skipped
    pub total_skipped: u32,
}

impl GateStatistics {
    /// Create new gate statistics.
    pub fn new(
        total_runs: u32,
        total_passes: u32,
        total_failures: u32,
        total_skipped: u32,
    ) -> Self {
        Self {
            total_runs,
            total_passes,
            total_failures,
            total_skipped,
        }
    }

    /// Calculate the pass rate as a percentage.
    pub fn pass_rate(&self) -> f64 {
        if self.total_runs == 0 {
            100.0
        } else {
            (self.total_passes as f64 / self.total_runs as f64) * 100.0
        }
    }

    /// Calculate the effective pass rate (passes + skipped).
    pub fn effective_pass_rate(&self) -> f64 {
        if self.total_runs == 0 {
            100.0
        } else {
            ((self.total_passes + self.total_skipped) as f64 / self.total_runs as f64) * 100.0
        }
    }
}

/// Comprehensive execution summary for display.
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    /// Results for each completed story
    pub story_results: Vec<StoryResult>,
    /// Total number of iterations across all stories
    pub total_iterations: u32,
    /// Total execution duration
    pub duration: Duration,
    /// Number of commits created
    pub commit_count: u32,
    /// Quality gate statistics
    pub gate_stats: GateStatistics,
}

impl ExecutionSummary {
    /// Create a new execution summary.
    pub fn new(
        story_results: Vec<StoryResult>,
        total_iterations: u32,
        duration: Duration,
        commit_count: u32,
        gate_stats: GateStatistics,
    ) -> Self {
        Self {
            story_results,
            total_iterations,
            duration,
            commit_count,
            gate_stats,
        }
    }

    /// Get the number of stories completed successfully.
    pub fn stories_passed(&self) -> usize {
        self.story_results.iter().filter(|s| s.passed).count()
    }

    /// Get the total number of stories.
    pub fn stories_total(&self) -> usize {
        self.story_results.len()
    }

    /// Check if all stories passed.
    pub fn all_passed(&self) -> bool {
        self.story_results.iter().all(|s| s.passed)
    }
}

/// Renders completion summary to the terminal.
#[derive(Debug)]
pub struct SummaryRenderer {
    /// Color theme for rendering
    theme: Theme,
    /// Panel width (characters)
    width: usize,
}

impl Default for SummaryRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl SummaryRenderer {
    /// Create a new summary renderer with default settings.
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

    /// Render the complete summary.
    pub fn render(&self, summary: &ExecutionSummary) -> String {
        let mut output = String::new();

        // Completion banner
        output.push_str(&self.render_completion_banner(summary));
        output.push('\n');

        // Story results table
        output.push_str(&self.render_story_table(summary));
        output.push('\n');

        // Quality gate statistics
        output.push_str(&self.render_gate_statistics(summary));
        output.push('\n');

        // Execution metrics
        output.push_str(&self.render_execution_metrics(summary));
        output.push('\n');

        // Progress tip
        output.push_str(&self.render_progress_tip());

        output
    }

    /// Render the success/failure completion banner.
    pub fn render_completion_banner(&self, summary: &ExecutionSummary) -> String {
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Top border
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));

        // Banner content
        let (icon, message, color) = if summary.all_passed() {
            (
                "✓",
                "All Stories Completed!".to_string(),
                self.theme.success,
            )
        } else {
            let failed = summary.stories_total() - summary.stories_passed();
            ("✗", format!("{} Stories Failed", failed), self.theme.error)
        };

        let banner_text = format!(" {} {} ", icon, message);
        let banner_padding = (inner_width.saturating_sub(banner_text.len())) / 2;
        let banner_remainder = inner_width
            .saturating_sub(banner_text.len())
            .saturating_sub(banner_padding);

        output.push_str(&format!(
            "│{}{}{}│\n",
            " ".repeat(banner_padding),
            banner_text.color(color).bold(),
            " ".repeat(banner_remainder)
        ));

        // Subtitle
        let subtitle = format!(
            "{}/{} stories passed",
            summary.stories_passed(),
            summary.stories_total()
        );
        let subtitle_padding = (inner_width.saturating_sub(subtitle.len())) / 2;
        let subtitle_remainder = inner_width
            .saturating_sub(subtitle.len())
            .saturating_sub(subtitle_padding);

        output.push_str(&format!(
            "│{}{}{}│\n",
            " ".repeat(subtitle_padding),
            subtitle.color(self.theme.muted),
            " ".repeat(subtitle_remainder)
        ));

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Render the story results table.
    pub fn render_story_table(&self, summary: &ExecutionSummary) -> String {
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Header
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));
        output.push_str(&format!(
            "│ {:<width$} │\n",
            "Story Results",
            width = inner_width - 2
        ));
        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Column headers
        let id_col = 8;
        let iter_col = 6;
        let status_col = 8;
        let title_col = inner_width.saturating_sub(id_col + iter_col + status_col + 7);

        let header = format!(
            " {:<id_w$} {:<title_w$} {:>iter_w$} {:>status_w$} ",
            "ID",
            "Title",
            "Iters",
            "Status",
            id_w = id_col,
            title_w = title_col,
            iter_w = iter_col,
            status_w = status_col
        );
        output.push_str(&format!("│{}│\n", header.color(self.theme.muted)));
        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Story rows
        for story in &summary.story_results {
            let status_icon = if story.passed { "✓" } else { "✗" };
            let status_text = if story.passed { "Passed" } else { "Failed" };
            let status_color = if story.passed {
                self.theme.success
            } else {
                self.theme.error
            };

            let title = self.truncate_text(&story.title, title_col);

            let id_display = format!("{:<width$}", story.id, width = id_col);
            let title_display = format!("{:<width$}", title, width = title_col);
            let iter_display = format!("{:>width$}", story.iterations, width = iter_col);
            let status_display = format!("{} {:<6}", status_icon, status_text);

            output.push_str(&format!(
                "│ {} {} {} {} │\n",
                id_display.color(self.theme.story_id),
                title_display,
                iter_display.color(self.theme.muted),
                status_display.color(status_color)
            ));
        }

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Render quality gate aggregate statistics.
    pub fn render_gate_statistics(&self, summary: &ExecutionSummary) -> String {
        let mut output = String::new();
        let inner_width = self.width - 2;
        let stats = &summary.gate_stats;

        // Header
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));
        output.push_str(&format!(
            "│ {:<width$} │\n",
            "Quality Gate Statistics",
            width = inner_width - 2
        ));
        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Statistics rows
        let pass_rate = format!("{:.1}%", stats.pass_rate());
        let pass_rate_color = if stats.pass_rate() >= 100.0 {
            self.theme.success
        } else if stats.pass_rate() >= 80.0 {
            self.theme.warning
        } else {
            self.theme.error
        };

        // Gate runs row
        let runs_label = "Total Gate Runs:";
        let runs_value = format!("{}", stats.total_runs);
        let runs_padding = inner_width.saturating_sub(runs_label.len() + runs_value.len() + 4);
        output.push_str(&format!(
            "│ {}{}{} │\n",
            runs_label.color(self.theme.muted),
            " ".repeat(runs_padding),
            runs_value
        ));

        // Passes row
        let passes_label = "Passed:";
        let passes_value = format!("{}", stats.total_passes);
        let passes_padding =
            inner_width.saturating_sub(passes_label.len() + passes_value.len() + 4);
        output.push_str(&format!(
            "│ {}{}{} │\n",
            passes_label.color(self.theme.muted),
            " ".repeat(passes_padding),
            passes_value.color(self.theme.success)
        ));

        // Failures row
        let failures_label = "Failed:";
        let failures_value = format!("{}", stats.total_failures);
        let failures_padding =
            inner_width.saturating_sub(failures_label.len() + failures_value.len() + 4);
        let failures_color = if stats.total_failures > 0 {
            self.theme.error
        } else {
            self.theme.muted
        };
        output.push_str(&format!(
            "│ {}{}{} │\n",
            failures_label.color(self.theme.muted),
            " ".repeat(failures_padding),
            failures_value.color(failures_color)
        ));

        // Skipped row
        let skipped_label = "Skipped:";
        let skipped_value = format!("{}", stats.total_skipped);
        let skipped_padding =
            inner_width.saturating_sub(skipped_label.len() + skipped_value.len() + 4);
        output.push_str(&format!(
            "│ {}{}{} │\n",
            skipped_label.color(self.theme.muted),
            " ".repeat(skipped_padding),
            skipped_value.color(self.theme.muted)
        ));

        // Separator
        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Pass rate row
        let rate_label = "Pass Rate:";
        let rate_padding = inner_width.saturating_sub(rate_label.len() + pass_rate.len() + 4);
        output.push_str(&format!(
            "│ {}{}{} │\n",
            rate_label.color(self.theme.muted),
            " ".repeat(rate_padding),
            pass_rate.color(pass_rate_color).bold()
        ));

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Render execution metrics (duration, iterations, commits).
    pub fn render_execution_metrics(&self, summary: &ExecutionSummary) -> String {
        let mut output = String::new();
        let inner_width = self.width - 2;

        // Header
        output.push_str(&format!("╭{}╮\n", "─".repeat(inner_width)));
        output.push_str(&format!(
            "│ {:<width$} │\n",
            "Execution Metrics",
            width = inner_width - 2
        ));
        output.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        // Duration row
        let duration_label = "Total Duration:";
        let duration_value = Self::format_duration(summary.duration);
        let duration_padding =
            inner_width.saturating_sub(duration_label.len() + duration_value.len() + 4);
        output.push_str(&format!(
            "│ {}{}{} │\n",
            duration_label.color(self.theme.muted),
            " ".repeat(duration_padding),
            duration_value.color(self.theme.in_progress)
        ));

        // Iterations row
        let iter_label = "Total Iterations:";
        let iter_value = format!("{}", summary.total_iterations);
        let iter_padding = inner_width.saturating_sub(iter_label.len() + iter_value.len() + 4);
        output.push_str(&format!(
            "│ {}{}{} │\n",
            iter_label.color(self.theme.muted),
            " ".repeat(iter_padding),
            iter_value
        ));

        // Commits row
        let commits_label = "Commits Created:";
        let commits_value = format!("{}", summary.commit_count);
        let commits_padding =
            inner_width.saturating_sub(commits_label.len() + commits_value.len() + 4);
        output.push_str(&format!(
            "│ {}{}{} │\n",
            commits_label.color(self.theme.muted),
            " ".repeat(commits_padding),
            commits_value.color(self.theme.success)
        ));

        // Bottom border
        output.push_str(&format!("╰{}╯", "─".repeat(inner_width)));

        output
    }

    /// Render a tip about progress.txt.
    pub fn render_progress_tip(&self) -> String {
        let tip_icon = "i";
        let tip_text = "See progress.txt for detailed execution history and learnings.";
        format!(
            "{} {}",
            tip_icon.color(self.theme.in_progress).bold(),
            tip_text.color(self.theme.muted)
        )
    }

    /// Format a duration in human-readable format.
    ///
    /// Examples: "1h 23m 45s", "5m 30s", "45s", "< 1s"
    pub fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();

        if total_secs == 0 {
            return "< 1s".to_string();
        }

        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        let mut parts = Vec::new();

        if hours > 0 {
            parts.push(format!("{}h", hours));
        }
        if minutes > 0 {
            parts.push(format!("{}m", minutes));
        }
        if seconds > 0 || parts.is_empty() {
            parts.push(format!("{}s", seconds));
        }

        parts.join(" ")
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
    fn test_story_result_creation() {
        let result = StoryResult::new("US-001", "Test Story", true, 3);
        assert_eq!(result.id, "US-001");
        assert_eq!(result.title, "Test Story");
        assert!(result.passed);
        assert_eq!(result.iterations, 3);
    }

    #[test]
    fn test_story_result_passed() {
        let result = StoryResult::passed("US-001", "Test", 2);
        assert!(result.passed);
    }

    #[test]
    fn test_story_result_failed() {
        let result = StoryResult::failed("US-001", "Test", 5);
        assert!(!result.passed);
    }

    #[test]
    fn test_gate_statistics_pass_rate() {
        let stats = GateStatistics::new(10, 8, 2, 0);
        assert!((stats.pass_rate() - 80.0).abs() < 0.1);
    }

    #[test]
    fn test_gate_statistics_pass_rate_zero_runs() {
        let stats = GateStatistics::new(0, 0, 0, 0);
        assert!((stats.pass_rate() - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_gate_statistics_effective_pass_rate() {
        let stats = GateStatistics::new(10, 6, 2, 2);
        assert!((stats.effective_pass_rate() - 80.0).abs() < 0.1);
    }

    #[test]
    fn test_execution_summary_stories_passed() {
        let summary = ExecutionSummary::new(
            vec![
                StoryResult::passed("US-001", "Story 1", 1),
                StoryResult::failed("US-002", "Story 2", 3),
                StoryResult::passed("US-003", "Story 3", 2),
            ],
            6,
            Duration::from_secs(300),
            2,
            GateStatistics::default(),
        );

        assert_eq!(summary.stories_passed(), 2);
        assert_eq!(summary.stories_total(), 3);
        assert!(!summary.all_passed());
    }

    #[test]
    fn test_execution_summary_all_passed() {
        let summary = ExecutionSummary::new(
            vec![
                StoryResult::passed("US-001", "Story 1", 1),
                StoryResult::passed("US-002", "Story 2", 2),
            ],
            3,
            Duration::from_secs(120),
            2,
            GateStatistics::default(),
        );

        assert!(summary.all_passed());
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(
            SummaryRenderer::format_duration(Duration::from_secs(45)),
            "45s"
        );
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(
            SummaryRenderer::format_duration(Duration::from_secs(330)),
            "5m 30s"
        );
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(
            SummaryRenderer::format_duration(Duration::from_secs(5025)),
            "1h 23m 45s"
        );
    }

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(
            SummaryRenderer::format_duration(Duration::from_secs(0)),
            "< 1s"
        );
    }

    #[test]
    fn test_format_duration_exact_hour() {
        assert_eq!(
            SummaryRenderer::format_duration(Duration::from_secs(3600)),
            "1h"
        );
    }

    #[test]
    fn test_format_duration_exact_minute() {
        assert_eq!(
            SummaryRenderer::format_duration(Duration::from_secs(60)),
            "1m"
        );
    }

    #[test]
    fn test_render_completion_banner_success() {
        let renderer = SummaryRenderer::new();
        let summary = ExecutionSummary::new(
            vec![
                StoryResult::passed("US-001", "Story 1", 1),
                StoryResult::passed("US-002", "Story 2", 2),
            ],
            3,
            Duration::from_secs(120),
            2,
            GateStatistics::default(),
        );

        let output = renderer.render_completion_banner(&summary);
        assert!(output.contains("All Stories Completed"));
        assert!(output.contains("2/2 stories passed"));
    }

    #[test]
    fn test_render_completion_banner_failure() {
        let renderer = SummaryRenderer::new();
        let summary = ExecutionSummary::new(
            vec![
                StoryResult::passed("US-001", "Story 1", 1),
                StoryResult::failed("US-002", "Story 2", 3),
            ],
            4,
            Duration::from_secs(180),
            1,
            GateStatistics::default(),
        );

        let output = renderer.render_completion_banner(&summary);
        assert!(output.contains("1 Stories Failed"));
        assert!(output.contains("1/2 stories passed"));
    }

    #[test]
    fn test_render_story_table() {
        let renderer = SummaryRenderer::new().with_width(70);
        let summary = ExecutionSummary::new(
            vec![
                StoryResult::passed("US-001", "First Story", 2),
                StoryResult::failed("US-002", "Second Story", 5),
            ],
            7,
            Duration::from_secs(300),
            1,
            GateStatistics::default(),
        );

        let output = renderer.render_story_table(&summary);
        assert!(output.contains("Story Results"));
        assert!(output.contains("US-001"));
        assert!(output.contains("US-002"));
        assert!(output.contains("Passed"));
        assert!(output.contains("Failed"));
    }

    #[test]
    fn test_render_gate_statistics() {
        let renderer = SummaryRenderer::new();
        let summary = ExecutionSummary::new(
            vec![StoryResult::passed("US-001", "Story", 1)],
            1,
            Duration::from_secs(60),
            1,
            GateStatistics::new(20, 18, 2, 0),
        );

        let output = renderer.render_gate_statistics(&summary);
        assert!(output.contains("Quality Gate Statistics"));
        assert!(output.contains("20"));
        assert!(output.contains("18"));
        assert!(output.contains("90.0%"));
    }

    #[test]
    fn test_render_execution_metrics() {
        let renderer = SummaryRenderer::new();
        let summary = ExecutionSummary::new(
            vec![StoryResult::passed("US-001", "Story", 1)],
            5,
            Duration::from_secs(3665), // 1h 1m 5s
            3,
            GateStatistics::default(),
        );

        let output = renderer.render_execution_metrics(&summary);
        assert!(output.contains("Execution Metrics"));
        assert!(output.contains("1h 1m 5s"));
        assert!(output.contains("5"));
        assert!(output.contains("3"));
    }

    #[test]
    fn test_render_progress_tip() {
        let renderer = SummaryRenderer::new();
        let output = renderer.render_progress_tip();
        assert!(output.contains("progress.txt"));
    }

    #[test]
    fn test_full_render() {
        let renderer = SummaryRenderer::new().with_width(60);
        let summary = ExecutionSummary::new(
            vec![
                StoryResult::passed("US-001", "Create UI module", 1),
                StoryResult::passed("US-002", "Add spinners", 2),
            ],
            3,
            Duration::from_secs(180),
            2,
            GateStatistics::new(8, 8, 0, 0),
        );

        let output = renderer.render(&summary);

        // Check all sections are present
        assert!(output.contains("All Stories Completed"));
        assert!(output.contains("Story Results"));
        assert!(output.contains("Quality Gate Statistics"));
        assert!(output.contains("Execution Metrics"));
        assert!(output.contains("progress.txt"));
    }
}
