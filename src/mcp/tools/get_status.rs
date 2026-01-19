// get_status MCP tool implementation
// This tool returns the current Ralph execution status

use crate::mcp::server::ExecutionState;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Request parameters for the get_status tool.
/// This tool takes no parameters.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetStatusRequest {}

/// Response from the get_status tool.
#[derive(Debug, Serialize)]
pub struct GetStatusResponse {
    /// Current state: "idle", "running", "completed", "failed", "paused", or "waiting_for_retry"
    pub state: String,
    /// Story ID being processed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_id: Option<String>,
    /// Timestamp when execution started (Unix timestamp, for running state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    /// Current iteration number (for running state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iteration: Option<u32>,
    /// Maximum iterations allowed (for running state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<u32>,
    /// Progress percentage (for running state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<u32>,
    /// Commit hash (for completed state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
    /// Error message (for failed state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Timestamp when execution was paused (Unix timestamp, for paused state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused_at: Option<u64>,
    /// Reason for the pause (for paused state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_reason: Option<String>,
    /// Timestamp when retry will be attempted (Unix timestamp, for waiting_for_retry state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_at: Option<u64>,
    /// Current retry attempt number (for waiting_for_retry state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    /// Maximum retry attempts allowed (for waiting_for_retry state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
}

impl GetStatusResponse {
    /// Create a response from an ExecutionState.
    pub fn from_execution_state(state: &ExecutionState) -> Self {
        match state {
            ExecutionState::Idle => Self {
                state: "idle".to_string(),
                story_id: None,
                started_at: None,
                iteration: None,
                max_iterations: None,
                progress_percent: None,
                commit_hash: None,
                error: None,
                paused_at: None,
                pause_reason: None,
                retry_at: None,
                attempt: None,
                max_attempts: None,
            },
            ExecutionState::Running {
                story_id,
                started_at,
                iteration,
                max_iterations,
            } => {
                // Calculate progress percentage
                let progress_percent = if *max_iterations > 0 {
                    Some(((*iteration as f64 / *max_iterations as f64) * 100.0) as u32)
                } else {
                    Some(0)
                };

                Self {
                    state: "running".to_string(),
                    story_id: Some(story_id.clone()),
                    started_at: Some(*started_at),
                    iteration: Some(*iteration),
                    max_iterations: Some(*max_iterations),
                    progress_percent,
                    commit_hash: None,
                    error: None,
                    paused_at: None,
                    pause_reason: None,
                    retry_at: None,
                    attempt: None,
                    max_attempts: None,
                }
            }
            ExecutionState::Completed {
                story_id,
                commit_hash,
            } => Self {
                state: "completed".to_string(),
                story_id: Some(story_id.clone()),
                started_at: None,
                iteration: None,
                max_iterations: None,
                progress_percent: Some(100),
                commit_hash: commit_hash.clone(),
                error: None,
                paused_at: None,
                pause_reason: None,
                retry_at: None,
                attempt: None,
                max_attempts: None,
            },
            ExecutionState::Failed { story_id, error } => Self {
                state: "failed".to_string(),
                story_id: Some(story_id.clone()),
                started_at: None,
                iteration: None,
                max_iterations: None,
                progress_percent: None,
                commit_hash: None,
                error: Some(error.clone()),
                paused_at: None,
                pause_reason: None,
                retry_at: None,
                attempt: None,
                max_attempts: None,
            },
            ExecutionState::Paused {
                story_id,
                paused_at,
                pause_reason,
            } => Self {
                state: "paused".to_string(),
                story_id: Some(story_id.clone()),
                started_at: None,
                iteration: None,
                max_iterations: None,
                progress_percent: None,
                commit_hash: None,
                error: None,
                paused_at: Some(*paused_at),
                pause_reason: Some(pause_reason.clone()),
                retry_at: None,
                attempt: None,
                max_attempts: None,
            },
            ExecutionState::WaitingForRetry {
                story_id,
                retry_at,
                attempt,
                max_attempts,
            } => Self {
                state: "waiting_for_retry".to_string(),
                story_id: Some(story_id.clone()),
                started_at: None,
                iteration: None,
                max_iterations: None,
                progress_percent: None,
                commit_hash: None,
                error: None,
                paused_at: None,
                pause_reason: None,
                retry_at: Some(*retry_at),
                attempt: Some(*attempt),
                max_attempts: Some(*max_attempts),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_status_response_idle() {
        let state = ExecutionState::Idle;
        let response = GetStatusResponse::from_execution_state(&state);

        assert_eq!(response.state, "idle");
        assert!(response.story_id.is_none());
        assert!(response.started_at.is_none());
        assert!(response.iteration.is_none());
        assert!(response.max_iterations.is_none());
        assert!(response.progress_percent.is_none());
        assert!(response.commit_hash.is_none());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_get_status_response_running() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 3,
            max_iterations: 10,
        };
        let response = GetStatusResponse::from_execution_state(&state);

        assert_eq!(response.state, "running");
        assert_eq!(response.story_id, Some("US-001".to_string()));
        assert_eq!(response.started_at, Some(1234567890));
        assert_eq!(response.iteration, Some(3));
        assert_eq!(response.max_iterations, Some(10));
        assert_eq!(response.progress_percent, Some(30)); // 3/10 = 30%
        assert!(response.commit_hash.is_none());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_get_status_response_running_zero_max() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 1,
            max_iterations: 0,
        };
        let response = GetStatusResponse::from_execution_state(&state);

        assert_eq!(response.state, "running");
        assert_eq!(response.progress_percent, Some(0)); // Avoid divide by zero
    }

    #[test]
    fn test_get_status_response_completed() {
        let state = ExecutionState::Completed {
            story_id: "US-001".to_string(),
            commit_hash: Some("abc123def".to_string()),
        };
        let response = GetStatusResponse::from_execution_state(&state);

        assert_eq!(response.state, "completed");
        assert_eq!(response.story_id, Some("US-001".to_string()));
        assert!(response.started_at.is_none());
        assert!(response.iteration.is_none());
        assert!(response.max_iterations.is_none());
        assert_eq!(response.progress_percent, Some(100));
        assert_eq!(response.commit_hash, Some("abc123def".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_get_status_response_completed_no_commit() {
        let state = ExecutionState::Completed {
            story_id: "US-001".to_string(),
            commit_hash: None,
        };
        let response = GetStatusResponse::from_execution_state(&state);

        assert_eq!(response.state, "completed");
        assert!(response.commit_hash.is_none());
    }

    #[test]
    fn test_get_status_response_failed() {
        let state = ExecutionState::Failed {
            story_id: "US-001".to_string(),
            error: "Quality checks failed".to_string(),
        };
        let response = GetStatusResponse::from_execution_state(&state);

        assert_eq!(response.state, "failed");
        assert_eq!(response.story_id, Some("US-001".to_string()));
        assert!(response.started_at.is_none());
        assert!(response.iteration.is_none());
        assert!(response.max_iterations.is_none());
        assert!(response.progress_percent.is_none());
        assert!(response.commit_hash.is_none());
        assert_eq!(response.error, Some("Quality checks failed".to_string()));
    }

    #[test]
    fn test_get_status_response_serialization_idle() {
        let state = ExecutionState::Idle;
        let response = GetStatusResponse::from_execution_state(&state);
        let json = serde_json::to_string(&response).unwrap();

        // Verify that optional fields are not included when None
        assert!(json.contains("\"state\":\"idle\""));
        assert!(!json.contains("story_id"));
        assert!(!json.contains("started_at"));
    }

    #[test]
    fn test_get_status_response_serialization_running() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 5,
            max_iterations: 10,
        };
        let response = GetStatusResponse::from_execution_state(&state);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"state\":\"running\""));
        assert!(json.contains("\"story_id\":\"US-001\""));
        assert!(json.contains("\"started_at\":1234567890"));
        assert!(json.contains("\"iteration\":5"));
        assert!(json.contains("\"max_iterations\":10"));
        assert!(json.contains("\"progress_percent\":50"));
        // Should not contain error or commit_hash
        assert!(!json.contains("\"error\""));
        assert!(!json.contains("\"commit_hash\""));
    }
}
