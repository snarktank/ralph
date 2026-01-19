//! Execution metrics collection for Ralph.
//!
//! This module provides infrastructure for collecting and analyzing
//! execution metrics across story executions, iterations, and quality gates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::iteration::context::ErrorCategory;

/// Metrics for a single story execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryMetrics {
    /// Story ID
    pub story_id: String,
    /// Number of iterations used
    pub iterations_used: u32,
    /// Maximum iterations allowed
    pub max_iterations: u32,
    /// Total execution duration
    pub total_duration: Duration,
    /// Whether the story succeeded
    pub success: bool,
    /// Gate results with durations
    pub gate_durations: HashMap<String, Duration>,
    /// Error categories encountered
    pub error_categories: Vec<ErrorCategory>,
    /// Final error message if failed
    pub final_error: Option<String>,
    /// Timestamp when execution started
    pub started_at: std::time::SystemTime,
    /// Timestamp when execution completed
    pub completed_at: std::time::SystemTime,
}

impl StoryMetrics {
    /// Create a new story metrics instance.
    pub fn new(story_id: impl Into<String>, max_iterations: u32) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            story_id: story_id.into(),
            iterations_used: 0,
            max_iterations,
            success: false,
            total_duration: Duration::ZERO,
            gate_durations: HashMap::new(),
            error_categories: Vec::new(),
            final_error: None,
            started_at: now,
            completed_at: now,
        }
    }

    /// Get the iteration efficiency (lower is better).
    /// Returns the ratio of iterations used to max iterations.
    pub fn iteration_efficiency(&self) -> f64 {
        if self.max_iterations == 0 {
            return 0.0;
        }
        self.iterations_used as f64 / self.max_iterations as f64
    }

    /// Get the average gate duration.
    pub fn average_gate_duration(&self) -> Duration {
        if self.gate_durations.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.gate_durations.values().sum();
        total / self.gate_durations.len() as u32
    }

    /// Mark the story as completed.
    pub fn complete(&mut self, success: bool, duration: Duration) {
        self.success = success;
        self.total_duration = duration;
        self.completed_at = std::time::SystemTime::now();
    }
}

/// Aggregated metrics across multiple story executions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Average iterations per story
    pub avg_iterations_per_story: f64,
    /// Parallelism efficiency (actual throughput / theoretical max)
    pub parallelism_efficiency: f64,
    /// Gate durations aggregated across all stories
    pub gate_durations: HashMap<String, GateDurationStats>,
    /// Error frequency by category
    pub error_frequency: HashMap<ErrorCategory, u32>,
    /// Total stories executed
    pub total_stories: u32,
    /// Successful stories
    pub successful_stories: u32,
    /// Failed stories
    pub failed_stories: u32,
    /// Total execution time
    pub total_execution_time: Duration,
    /// First-time success rate (stories that passed on first iteration)
    pub first_time_success_rate: f64,
}

impl ExecutionMetrics {
    /// Calculate the overall success rate.
    pub fn success_rate(&self) -> f64 {
        if self.total_stories == 0 {
            return 0.0;
        }
        self.successful_stories as f64 / self.total_stories as f64
    }

    /// Get the most common error category.
    pub fn most_common_error(&self) -> Option<ErrorCategory> {
        self.error_frequency
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cat, _)| *cat)
    }

    /// Get the slowest gate on average.
    pub fn slowest_gate(&self) -> Option<&str> {
        self.gate_durations
            .iter()
            .max_by_key(|(_, stats)| stats.mean)
            .map(|(name, _)| name.as_str())
    }
}

/// Duration statistics for a quality gate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GateDurationStats {
    /// Number of samples
    pub count: u32,
    /// Mean duration
    pub mean: Duration,
    /// Minimum duration
    pub min: Duration,
    /// Maximum duration
    pub max: Duration,
    /// Sum of all durations (for calculating mean)
    pub total: Duration,
}

impl GateDurationStats {
    /// Add a new duration sample.
    pub fn add_sample(&mut self, duration: Duration) {
        self.count += 1;
        self.total += duration;
        self.mean = self.total / self.count;

        if self.count == 1 {
            self.min = duration;
            self.max = duration;
        } else {
            self.min = self.min.min(duration);
            self.max = self.max.max(duration);
        }
    }
}

/// Builder for tracking an execution session.
#[derive(Debug)]
pub struct MetricsBuilder {
    /// Current story being tracked
    current_story: Option<StoryMetrics>,
    /// Completed story metrics
    completed_stories: Vec<StoryMetrics>,
    /// Parallel execution start times for efficiency calculation
    parallel_start: Option<Instant>,
    /// Total wall-clock time for parallel execution
    parallel_wall_time: Duration,
    /// Sum of individual story durations (for parallelism calculation)
    parallel_sum_time: Duration,
}

impl MetricsBuilder {
    /// Create a new metrics builder.
    pub fn new() -> Self {
        Self {
            current_story: None,
            completed_stories: Vec::new(),
            parallel_start: None,
            parallel_wall_time: Duration::ZERO,
            parallel_sum_time: Duration::ZERO,
        }
    }

    /// Start tracking a new story.
    pub fn start_story(&mut self, story_id: impl Into<String>, max_iterations: u32) {
        self.current_story = Some(StoryMetrics::new(story_id, max_iterations));
    }

    /// Record an iteration for the current story.
    pub fn record_iteration(&mut self, iteration: u32) {
        if let Some(ref mut story) = self.current_story {
            story.iterations_used = iteration;
        }
    }

    /// Record a gate duration for the current story.
    pub fn record_gate_duration(&mut self, gate_name: impl Into<String>, duration: Duration) {
        if let Some(ref mut story) = self.current_story {
            story.gate_durations.insert(gate_name.into(), duration);
        }
    }

    /// Record an error category for the current story.
    pub fn record_error(&mut self, category: ErrorCategory) {
        if let Some(ref mut story) = self.current_story {
            story.error_categories.push(category);
        }
    }

    /// Complete the current story.
    pub fn complete_story(&mut self, success: bool, duration: Duration, error: Option<String>) {
        if let Some(mut story) = self.current_story.take() {
            story.complete(success, duration);
            story.final_error = error;
            self.parallel_sum_time += duration;
            self.completed_stories.push(story);
        }
    }

    /// Start tracking parallel execution.
    pub fn start_parallel(&mut self) {
        self.parallel_start = Some(Instant::now());
    }

    /// End parallel execution tracking.
    pub fn end_parallel(&mut self) {
        if let Some(start) = self.parallel_start.take() {
            self.parallel_wall_time = start.elapsed();
        }
    }

    /// Build the final aggregated metrics.
    pub fn build(self) -> ExecutionMetrics {
        let total_stories = self.completed_stories.len() as u32;
        if total_stories == 0 {
            return ExecutionMetrics::default();
        }

        let successful_stories = self.completed_stories.iter().filter(|s| s.success).count() as u32;
        let failed_stories = total_stories - successful_stories;

        // Calculate average iterations
        let total_iterations: u32 = self
            .completed_stories
            .iter()
            .map(|s| s.iterations_used)
            .sum();
        let avg_iterations = total_iterations as f64 / total_stories as f64;

        // Calculate first-time success rate
        let first_time_successes = self
            .completed_stories
            .iter()
            .filter(|s| s.success && s.iterations_used == 1)
            .count() as f64;
        let first_time_success_rate = first_time_successes / total_stories as f64;

        // Aggregate gate durations
        let mut gate_durations: HashMap<String, GateDurationStats> = HashMap::new();
        for story in &self.completed_stories {
            for (gate, duration) in &story.gate_durations {
                gate_durations
                    .entry(gate.clone())
                    .or_default()
                    .add_sample(*duration);
            }
        }

        // Aggregate error frequencies
        let mut error_frequency: HashMap<ErrorCategory, u32> = HashMap::new();
        for story in &self.completed_stories {
            for category in &story.error_categories {
                *error_frequency.entry(*category).or_insert(0) += 1;
            }
        }

        // Calculate total execution time
        let total_execution_time: Duration = self
            .completed_stories
            .iter()
            .map(|s| s.total_duration)
            .sum();

        // Calculate parallelism efficiency
        let parallelism_efficiency = if self.parallel_wall_time > Duration::ZERO {
            self.parallel_sum_time.as_secs_f64() / self.parallel_wall_time.as_secs_f64()
        } else {
            1.0 // Sequential execution
        };

        ExecutionMetrics {
            avg_iterations_per_story: avg_iterations,
            parallelism_efficiency,
            gate_durations,
            error_frequency,
            total_stories,
            successful_stories,
            failed_stories,
            total_execution_time,
            first_time_success_rate,
        }
    }
}

impl Default for MetricsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe metrics collector for concurrent story execution.
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    inner: Arc<RwLock<MetricsBuilder>>,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MetricsBuilder::new())),
        }
    }

    /// Start tracking a new story (thread-safe).
    pub fn start_story(&self, story_id: impl Into<String>, max_iterations: u32) {
        if let Ok(mut builder) = self.inner.write() {
            builder.start_story(story_id, max_iterations);
        }
    }

    /// Record an iteration (thread-safe).
    pub fn record_iteration(&self, iteration: u32) {
        if let Ok(mut builder) = self.inner.write() {
            builder.record_iteration(iteration);
        }
    }

    /// Record a gate duration (thread-safe).
    pub fn record_gate_duration(&self, gate_name: impl Into<String>, duration: Duration) {
        if let Ok(mut builder) = self.inner.write() {
            builder.record_gate_duration(gate_name, duration);
        }
    }

    /// Record an error (thread-safe).
    pub fn record_error(&self, category: ErrorCategory) {
        if let Ok(mut builder) = self.inner.write() {
            builder.record_error(category);
        }
    }

    /// Complete the current story (thread-safe).
    pub fn complete_story(&self, success: bool, duration: Duration, error: Option<String>) {
        if let Ok(mut builder) = self.inner.write() {
            builder.complete_story(success, duration, error);
        }
    }

    /// Start tracking parallel execution (thread-safe).
    pub fn start_parallel(&self) {
        if let Ok(mut builder) = self.inner.write() {
            builder.start_parallel();
        }
    }

    /// End parallel execution tracking (thread-safe).
    pub fn end_parallel(&self) {
        if let Ok(mut builder) = self.inner.write() {
            builder.end_parallel();
        }
    }

    /// Build the final metrics (consumes the inner builder).
    pub fn build(&self) -> ExecutionMetrics {
        if let Ok(builder) = self.inner.read() {
            // Clone the builder's data to build metrics
            let mut new_builder = MetricsBuilder::new();
            new_builder.completed_stories = builder.completed_stories.clone();
            new_builder.parallel_wall_time = builder.parallel_wall_time;
            new_builder.parallel_sum_time = builder.parallel_sum_time;
            new_builder.build()
        } else {
            ExecutionMetrics::default()
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Format metrics for display.
pub fn format_metrics(metrics: &ExecutionMetrics) -> String {
    let mut output = String::from("## Execution Metrics\n\n");

    // Summary statistics
    output.push_str("### Summary\n");
    output.push_str(&format!(
        "- **Total Stories**: {} ({} successful, {} failed)\n",
        metrics.total_stories, metrics.successful_stories, metrics.failed_stories
    ));
    output.push_str(&format!(
        "- **Success Rate**: {:.1}%\n",
        metrics.success_rate() * 100.0
    ));
    output.push_str(&format!(
        "- **First-Time Success Rate**: {:.1}%\n",
        metrics.first_time_success_rate * 100.0
    ));
    output.push_str(&format!(
        "- **Average Iterations**: {:.2}\n",
        metrics.avg_iterations_per_story
    ));
    output.push_str(&format!(
        "- **Parallelism Efficiency**: {:.2}x\n",
        metrics.parallelism_efficiency
    ));
    output.push_str(&format!(
        "- **Total Execution Time**: {:.1}s\n",
        metrics.total_execution_time.as_secs_f64()
    ));

    // Gate durations
    if !metrics.gate_durations.is_empty() {
        output.push_str("\n### Gate Durations\n");
        for (gate, stats) in &metrics.gate_durations {
            output.push_str(&format!(
                "- **{}**: mean={:.2}s, min={:.2}s, max={:.2}s (n={})\n",
                gate,
                stats.mean.as_secs_f64(),
                stats.min.as_secs_f64(),
                stats.max.as_secs_f64(),
                stats.count
            ));
        }
    }

    // Error frequencies
    if !metrics.error_frequency.is_empty() {
        output.push_str("\n### Error Frequencies\n");
        let mut errors: Vec<_> = metrics.error_frequency.iter().collect();
        errors.sort_by(|a, b| b.1.cmp(a.1));
        for (category, count) in errors {
            output.push_str(&format!(
                "- **{}**: {} occurrences\n",
                category.as_str(),
                count
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_metrics_new() {
        let metrics = StoryMetrics::new("US-001", 10);
        assert_eq!(metrics.story_id, "US-001");
        assert_eq!(metrics.max_iterations, 10);
        assert_eq!(metrics.iterations_used, 0);
        assert!(!metrics.success);
    }

    #[test]
    fn test_story_metrics_iteration_efficiency() {
        let mut metrics = StoryMetrics::new("US-001", 10);
        metrics.iterations_used = 5;
        assert_eq!(metrics.iteration_efficiency(), 0.5);
    }

    #[test]
    fn test_story_metrics_iteration_efficiency_zero() {
        let metrics = StoryMetrics::new("US-001", 0);
        assert_eq!(metrics.iteration_efficiency(), 0.0);
    }

    #[test]
    fn test_story_metrics_complete() {
        let mut metrics = StoryMetrics::new("US-001", 10);
        metrics.complete(true, Duration::from_secs(60));
        assert!(metrics.success);
        assert_eq!(metrics.total_duration, Duration::from_secs(60));
    }

    #[test]
    fn test_gate_duration_stats_add_sample() {
        let mut stats = GateDurationStats::default();
        stats.add_sample(Duration::from_secs(1));
        assert_eq!(stats.count, 1);
        assert_eq!(stats.mean, Duration::from_secs(1));
        assert_eq!(stats.min, Duration::from_secs(1));
        assert_eq!(stats.max, Duration::from_secs(1));

        stats.add_sample(Duration::from_secs(3));
        assert_eq!(stats.count, 2);
        assert_eq!(stats.mean, Duration::from_secs(2));
        assert_eq!(stats.min, Duration::from_secs(1));
        assert_eq!(stats.max, Duration::from_secs(3));
    }

    #[test]
    fn test_metrics_builder_new() {
        let builder = MetricsBuilder::new();
        assert!(builder.current_story.is_none());
        assert!(builder.completed_stories.is_empty());
    }

    #[test]
    fn test_metrics_builder_track_story() {
        let mut builder = MetricsBuilder::new();
        builder.start_story("US-001", 10);
        assert!(builder.current_story.is_some());

        builder.record_iteration(1);
        builder.record_gate_duration("lint", Duration::from_secs(5));
        builder.record_error(ErrorCategory::Lint);
        builder.complete_story(true, Duration::from_secs(30), None);

        assert!(builder.current_story.is_none());
        assert_eq!(builder.completed_stories.len(), 1);
        assert!(builder.completed_stories[0].success);
    }

    #[test]
    fn test_metrics_builder_build() {
        let mut builder = MetricsBuilder::new();

        // Add two stories
        builder.start_story("US-001", 10);
        builder.record_iteration(1);
        builder.complete_story(true, Duration::from_secs(30), None);

        builder.start_story("US-002", 10);
        builder.record_iteration(3);
        builder.complete_story(false, Duration::from_secs(60), Some("Failed".to_string()));

        let metrics = builder.build();
        assert_eq!(metrics.total_stories, 2);
        assert_eq!(metrics.successful_stories, 1);
        assert_eq!(metrics.failed_stories, 1);
        assert_eq!(metrics.avg_iterations_per_story, 2.0);
        assert_eq!(metrics.success_rate(), 0.5);
    }

    #[test]
    fn test_metrics_builder_build_empty() {
        let builder = MetricsBuilder::new();
        let metrics = builder.build();
        assert_eq!(metrics.total_stories, 0);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_metrics_builder_first_time_success() {
        let mut builder = MetricsBuilder::new();

        // First-time success
        builder.start_story("US-001", 10);
        builder.record_iteration(1);
        builder.complete_story(true, Duration::from_secs(30), None);

        // Not first-time success (2 iterations)
        builder.start_story("US-002", 10);
        builder.record_iteration(2);
        builder.complete_story(true, Duration::from_secs(60), None);

        let metrics = builder.build();
        assert_eq!(metrics.first_time_success_rate, 0.5);
    }

    #[test]
    fn test_metrics_builder_parallelism() {
        let mut builder = MetricsBuilder::new();

        builder.start_parallel();

        builder.start_story("US-001", 10);
        builder.complete_story(true, Duration::from_secs(30), None);

        builder.start_story("US-002", 10);
        builder.complete_story(true, Duration::from_secs(40), None);

        // Simulate some wall time
        builder.parallel_wall_time = Duration::from_secs(35); // Less than sum (70s)
        builder.parallel_sum_time = Duration::from_secs(70);

        let metrics = builder.build();
        assert_eq!(metrics.parallelism_efficiency, 2.0); // 70/35 = 2x
    }

    #[test]
    fn test_execution_metrics_most_common_error() {
        let mut metrics = ExecutionMetrics::default();
        metrics.error_frequency.insert(ErrorCategory::Lint, 5);
        metrics.error_frequency.insert(ErrorCategory::Format, 3);
        assert_eq!(metrics.most_common_error(), Some(ErrorCategory::Lint));
    }

    #[test]
    fn test_execution_metrics_slowest_gate() {
        let mut metrics = ExecutionMetrics::default();
        metrics.gate_durations.insert(
            "lint".to_string(),
            GateDurationStats {
                mean: Duration::from_secs(10),
                ..Default::default()
            },
        );
        metrics.gate_durations.insert(
            "coverage".to_string(),
            GateDurationStats {
                mean: Duration::from_secs(60),
                ..Default::default()
            },
        );
        assert_eq!(metrics.slowest_gate(), Some("coverage"));
    }

    #[test]
    fn test_metrics_collector_thread_safe() {
        let collector = MetricsCollector::new();

        collector.start_story("US-001", 10);
        collector.record_iteration(1);
        collector.complete_story(true, Duration::from_secs(30), None);

        let metrics = collector.build();
        assert_eq!(metrics.total_stories, 1);
    }

    #[test]
    fn test_format_metrics() {
        let mut metrics = ExecutionMetrics::default();
        metrics.total_stories = 10;
        metrics.successful_stories = 8;
        metrics.failed_stories = 2;
        metrics.avg_iterations_per_story = 1.5;
        metrics.first_time_success_rate = 0.7;
        metrics.parallelism_efficiency = 2.5;
        metrics.total_execution_time = Duration::from_secs(120);

        let output = format_metrics(&metrics);
        assert!(output.contains("Total Stories"));
        assert!(output.contains("80.0%")); // Success rate
        assert!(output.contains("70.0%")); // First-time success rate
        assert!(output.contains("1.50")); // Avg iterations
        assert!(output.contains("2.50x")); // Parallelism efficiency
    }
}
