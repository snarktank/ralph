//! Checkpoint module for execution state persistence.
//!
//! This module provides types and functionality for saving and loading
//! execution state, enabling resumption after interruptions.

pub mod manager;

pub use manager::{CheckpointError, CheckpointManager, CheckpointResult};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Reason why execution was paused and a checkpoint was created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PauseReason {
    /// API usage limit was exceeded
    UsageLimitExceeded,
    /// Rate limited by external service
    RateLimited,
    /// User requested pause
    UserRequested,
    /// Execution timed out
    Timeout,
    /// An error occurred during execution
    Error(String),
    /// Checkpoint saved at iteration boundary (for recovery if interrupted)
    IterationBoundary,
}

/// Checkpoint data for a single story's execution state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryCheckpoint {
    /// Unique identifier for the story
    pub story_id: String,
    /// Current iteration number (1-indexed)
    pub iteration: u32,
    /// Maximum number of iterations allowed
    pub max_iterations: u32,
}

/// Main checkpoint structure containing full execution state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint format version for future compatibility
    pub version: u32,
    /// When this checkpoint was created
    pub created_at: DateTime<Utc>,
    /// Current story being executed (if any)
    pub current_story: Option<StoryCheckpoint>,
    /// Reason execution was paused
    pub pause_reason: PauseReason,
    /// List of files with uncommitted changes
    pub uncommitted_files: Vec<String>,
}

impl Checkpoint {
    /// Current checkpoint format version
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new checkpoint with the current timestamp.
    pub fn new(
        current_story: Option<StoryCheckpoint>,
        pause_reason: PauseReason,
        uncommitted_files: Vec<String>,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            created_at: Utc::now(),
            current_story,
            pause_reason,
            uncommitted_files,
        }
    }
}

impl StoryCheckpoint {
    /// Create a new story checkpoint.
    pub fn new(story_id: impl Into<String>, iteration: u32, max_iterations: u32) -> Self {
        Self {
            story_id: story_id.into(),
            iteration,
            max_iterations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_reason_serialization_roundtrip() {
        let reasons = vec![
            PauseReason::UsageLimitExceeded,
            PauseReason::RateLimited,
            PauseReason::UserRequested,
            PauseReason::Timeout,
            PauseReason::Error("Connection failed".to_string()),
        ];

        for reason in reasons {
            let json = serde_json::to_string(&reason).expect("Failed to serialize PauseReason");
            let deserialized: PauseReason =
                serde_json::from_str(&json).expect("Failed to deserialize PauseReason");
            assert_eq!(reason, deserialized);
        }
    }

    #[test]
    fn test_story_checkpoint_serialization_roundtrip() {
        let checkpoint = StoryCheckpoint::new("US-001", 3, 5);

        let json = serde_json::to_string(&checkpoint).expect("Failed to serialize StoryCheckpoint");
        let deserialized: StoryCheckpoint =
            serde_json::from_str(&json).expect("Failed to deserialize StoryCheckpoint");

        assert_eq!(checkpoint, deserialized);
    }

    #[test]
    fn test_checkpoint_serialization_roundtrip() {
        let checkpoint = Checkpoint::new(
            Some(StoryCheckpoint::new("US-002", 2, 10)),
            PauseReason::RateLimited,
            vec!["src/main.rs".to_string(), "Cargo.toml".to_string()],
        );

        let json = serde_json::to_string(&checkpoint).expect("Failed to serialize Checkpoint");
        let deserialized: Checkpoint =
            serde_json::from_str(&json).expect("Failed to deserialize Checkpoint");

        assert_eq!(checkpoint.version, deserialized.version);
        assert_eq!(checkpoint.current_story, deserialized.current_story);
        assert_eq!(checkpoint.pause_reason, deserialized.pause_reason);
        assert_eq!(checkpoint.uncommitted_files, deserialized.uncommitted_files);
    }

    #[test]
    fn test_checkpoint_without_current_story() {
        let checkpoint = Checkpoint::new(None, PauseReason::UserRequested, vec![]);

        let json = serde_json::to_string(&checkpoint).expect("Failed to serialize Checkpoint");
        let deserialized: Checkpoint =
            serde_json::from_str(&json).expect("Failed to deserialize Checkpoint");

        assert!(deserialized.current_story.is_none());
        assert_eq!(checkpoint.pause_reason, deserialized.pause_reason);
    }

    #[test]
    fn test_checkpoint_version() {
        let checkpoint = Checkpoint::new(None, PauseReason::Timeout, vec![]);
        assert_eq!(checkpoint.version, Checkpoint::CURRENT_VERSION);
    }

    #[test]
    fn test_pause_reason_json_format() {
        // Verify snake_case serialization
        let json = serde_json::to_string(&PauseReason::UsageLimitExceeded).unwrap();
        assert_eq!(json, "\"usage_limit_exceeded\"");

        let json = serde_json::to_string(&PauseReason::RateLimited).unwrap();
        assert_eq!(json, "\"rate_limited\"");

        let json = serde_json::to_string(&PauseReason::UserRequested).unwrap();
        assert_eq!(json, "\"user_requested\"");

        let json = serde_json::to_string(&PauseReason::Error("test".to_string())).unwrap();
        assert!(json.contains("error"));
    }
}
