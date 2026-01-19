//! Event-based communication system for parallel execution UI.
//!
//! This module provides an event system that decouples the parallel scheduler
//! from the display layer, enabling real-time UI updates during concurrent
//! story execution.

use std::path::PathBuf;

/// Information about a story for display purposes in parallel execution.
///
/// This is a lightweight struct containing only the information needed
/// for UI rendering during parallel execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryDisplayInfo {
    /// Story identifier (e.g., "US-001")
    pub id: String,
    /// Story title
    pub title: String,
    /// Priority level (1 = highest)
    pub priority: u32,
}

impl StoryDisplayInfo {
    /// Creates a new StoryDisplayInfo.
    pub fn new(id: impl Into<String>, title: impl Into<String>, priority: u32) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            priority,
        }
    }
}

/// Status of a story in the parallel execution pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryStatus {
    /// Story is waiting for dependencies or resources.
    Pending,
    /// Story is currently executing.
    InProgress,
    /// Story completed successfully.
    Completed,
    /// Story execution failed.
    Failed,
    /// Story was deferred due to conflicts.
    Deferred,
    /// Story is retrying in sequential mode.
    SequentialRetry,
}

impl StoryStatus {
    /// Get the status icon for this state.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::InProgress => "◉",
            Self::Completed => "✓",
            Self::Failed => "✗",
            Self::Deferred => "⊘",
            Self::SequentialRetry => "↻",
        }
    }

    /// Get the status label for this state.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::InProgress => "In Progress",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Deferred => "Deferred",
            Self::SequentialRetry => "Retrying",
        }
    }
}

/// Events emitted during parallel story execution for UI updates.
///
/// These events allow the UI to track the progress of multiple concurrent
/// story executions without tight coupling to the scheduler implementation.
#[derive(Clone, Debug, PartialEq)]
pub enum ParallelUIEvent {
    /// A story has started execution.
    StoryStarted {
        /// Information about the story that started.
        story: StoryDisplayInfo,
        /// Current iteration number (1-indexed).
        iteration: u32,
        /// Number of concurrent stories currently executing.
        concurrent_count: usize,
    },

    /// Progress update for a story's iteration.
    IterationUpdate {
        /// Story identifier.
        story_id: String,
        /// Current iteration number (1-indexed).
        iteration: u32,
        /// Maximum allowed iterations.
        max_iterations: u32,
        /// Optional progress message.
        message: Option<String>,
    },

    /// Quality gate status update for a story.
    GateUpdate {
        /// Story identifier.
        story_id: String,
        /// Name of the quality gate.
        gate_name: String,
        /// Whether the gate passed.
        passed: bool,
        /// Optional message or details.
        message: Option<String>,
    },

    /// A story has completed successfully.
    StoryCompleted {
        /// Story identifier.
        story_id: String,
        /// Total iterations taken.
        iterations_used: u32,
        /// Total duration in milliseconds.
        duration_ms: u64,
    },

    /// A story has failed after exhausting retries.
    StoryFailed {
        /// Story identifier.
        story_id: String,
        /// Error message describing the failure.
        error: String,
        /// Iteration at which the failure occurred.
        iteration: u32,
    },

    /// A story was deferred due to file conflicts with another story.
    ConflictDeferred {
        /// Story identifier that was deferred.
        story_id: String,
        /// Story identifier that caused the conflict.
        blocking_story_id: String,
        /// Files that caused the conflict.
        conflicting_files: Vec<PathBuf>,
    },

    /// Status update from the reconciliation engine.
    ReconciliationStatus {
        /// Whether reconciliation was successful (no issues found).
        success: bool,
        /// Number of issues found (0 if successful).
        issues_count: usize,
        /// Summary message about the reconciliation result.
        message: String,
    },

    /// A story is being retried in sequential mode after parallel failure.
    SequentialRetryStarted {
        /// Story identifier being retried.
        story_id: String,
        /// Reason for falling back to sequential execution.
        reason: String,
    },

    /// Keyboard toggle event (streaming, expand, or quit).
    KeyboardToggle {
        /// Type of toggle: "streaming", "expand", or "quit"
        toggle_type: String,
        /// New state (true = on, false = off)
        new_state: bool,
    },

    /// Graceful quit requested (finish current stories, then exit).
    GracefulQuitRequested,

    /// Immediate interrupt requested (Ctrl+C).
    ImmediateInterrupt,
}

impl ParallelUIEvent {
    /// Returns the story ID associated with this event, if any.
    pub fn story_id(&self) -> Option<&str> {
        match self {
            Self::StoryStarted { story, .. } => Some(&story.id),
            Self::IterationUpdate { story_id, .. } => Some(story_id),
            Self::GateUpdate { story_id, .. } => Some(story_id),
            Self::StoryCompleted { story_id, .. } => Some(story_id),
            Self::StoryFailed { story_id, .. } => Some(story_id),
            Self::ConflictDeferred { story_id, .. } => Some(story_id),
            Self::ReconciliationStatus { .. } => None,
            Self::SequentialRetryStarted { story_id, .. } => Some(story_id),
            Self::KeyboardToggle { .. } => None,
            Self::GracefulQuitRequested => None,
            Self::ImmediateInterrupt => None,
        }
    }

    /// Returns true if this is a terminal event for a story (completed or failed).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::StoryCompleted { .. } | Self::StoryFailed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_display_info_new() {
        let info = StoryDisplayInfo::new("US-001", "Test Story", 1);
        assert_eq!(info.id, "US-001");
        assert_eq!(info.title, "Test Story");
        assert_eq!(info.priority, 1);
    }

    #[test]
    fn test_story_display_info_equality() {
        let info1 = StoryDisplayInfo::new("US-001", "Test", 1);
        let info2 = StoryDisplayInfo::new("US-001", "Test", 1);
        let info3 = StoryDisplayInfo::new("US-002", "Test", 1);

        assert_eq!(info1, info2);
        assert_ne!(info1, info3);
    }

    #[test]
    fn test_story_status_icons() {
        assert_eq!(StoryStatus::Pending.icon(), "○");
        assert_eq!(StoryStatus::InProgress.icon(), "◉");
        assert_eq!(StoryStatus::Completed.icon(), "✓");
        assert_eq!(StoryStatus::Failed.icon(), "✗");
        assert_eq!(StoryStatus::Deferred.icon(), "⊘");
        assert_eq!(StoryStatus::SequentialRetry.icon(), "↻");
    }

    #[test]
    fn test_story_status_labels() {
        assert_eq!(StoryStatus::Pending.label(), "Pending");
        assert_eq!(StoryStatus::InProgress.label(), "In Progress");
        assert_eq!(StoryStatus::Completed.label(), "Completed");
        assert_eq!(StoryStatus::Failed.label(), "Failed");
        assert_eq!(StoryStatus::Deferred.label(), "Deferred");
        assert_eq!(StoryStatus::SequentialRetry.label(), "Retrying");
    }

    #[test]
    fn test_event_story_started() {
        let story = StoryDisplayInfo::new("US-001", "Test Story", 1);
        let event = ParallelUIEvent::StoryStarted {
            story: story.clone(),
            iteration: 1,
            concurrent_count: 3,
        };

        assert_eq!(event.story_id(), Some("US-001"));
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_event_iteration_update() {
        let event = ParallelUIEvent::IterationUpdate {
            story_id: "US-001".to_string(),
            iteration: 2,
            max_iterations: 5,
            message: Some("Running quality gates".to_string()),
        };

        assert_eq!(event.story_id(), Some("US-001"));
        assert!(!event.is_terminal());

        if let ParallelUIEvent::IterationUpdate {
            iteration,
            max_iterations,
            message,
            ..
        } = event
        {
            assert_eq!(iteration, 2);
            assert_eq!(max_iterations, 5);
            assert_eq!(message, Some("Running quality gates".to_string()));
        }
    }

    #[test]
    fn test_event_gate_update() {
        let event = ParallelUIEvent::GateUpdate {
            story_id: "US-001".to_string(),
            gate_name: "lint".to_string(),
            passed: true,
            message: Some("No warnings".to_string()),
        };

        assert_eq!(event.story_id(), Some("US-001"));
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_event_story_completed() {
        let event = ParallelUIEvent::StoryCompleted {
            story_id: "US-001".to_string(),
            iterations_used: 3,
            duration_ms: 5000,
        };

        assert_eq!(event.story_id(), Some("US-001"));
        assert!(event.is_terminal());
    }

    #[test]
    fn test_event_story_failed() {
        let event = ParallelUIEvent::StoryFailed {
            story_id: "US-001".to_string(),
            error: "Quality gates failed".to_string(),
            iteration: 5,
        };

        assert_eq!(event.story_id(), Some("US-001"));
        assert!(event.is_terminal());
    }

    #[test]
    fn test_event_conflict_deferred() {
        let event = ParallelUIEvent::ConflictDeferred {
            story_id: "US-002".to_string(),
            blocking_story_id: "US-001".to_string(),
            conflicting_files: vec![PathBuf::from("src/lib.rs")],
        };

        assert_eq!(event.story_id(), Some("US-002"));
        assert!(!event.is_terminal());

        if let ParallelUIEvent::ConflictDeferred {
            blocking_story_id,
            conflicting_files,
            ..
        } = event
        {
            assert_eq!(blocking_story_id, "US-001");
            assert_eq!(conflicting_files.len(), 1);
        }
    }

    #[test]
    fn test_event_reconciliation_status() {
        let event = ParallelUIEvent::ReconciliationStatus {
            success: true,
            issues_count: 0,
            message: "No conflicts detected".to_string(),
        };

        assert_eq!(event.story_id(), None);
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_event_sequential_retry_started() {
        let event = ParallelUIEvent::SequentialRetryStarted {
            story_id: "US-001".to_string(),
            reason: "Conflict with US-002".to_string(),
        };

        assert_eq!(event.story_id(), Some("US-001"));
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_story_display_info_clone() {
        let info = StoryDisplayInfo::new("US-001", "Test", 1);
        let cloned = info.clone();
        assert_eq!(info, cloned);
    }

    #[test]
    fn test_story_status_copy() {
        let status = StoryStatus::InProgress;
        let copied = status;
        assert_eq!(status, copied);
    }

    #[test]
    fn test_event_clone() {
        let event = ParallelUIEvent::StoryCompleted {
            story_id: "US-001".to_string(),
            iterations_used: 2,
            duration_ms: 1000,
        };
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }
}
