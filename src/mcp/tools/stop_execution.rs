// stop_execution MCP tool implementation
// This tool cancels the currently running story execution

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::mcp::server::ExecutionState;

/// Request parameters for the stop_execution tool.
///
/// This tool takes no parameters - it simply stops whatever is currently running.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct StopExecutionRequest {}

/// Response from the stop_execution tool.
#[derive(Debug, Serialize)]
pub struct StopExecutionResponse {
    /// Whether the stop was successful or there was nothing to stop
    pub success: bool,
    /// Whether an execution was actually cancelled (false if nothing was running)
    pub was_running: bool,
    /// The story ID that was cancelled (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_id: Option<String>,
    /// Message describing the result
    pub message: String,
}

/// Create a response when execution is successfully cancelled.
pub fn create_cancelled_response(story_id: &str) -> StopExecutionResponse {
    StopExecutionResponse {
        success: true,
        was_running: true,
        story_id: Some(story_id.to_string()),
        message: format!(
            "Cancellation signal sent for story '{}'. Execution will stop at the next safe point.",
            story_id
        ),
    }
}

/// Create a response when nothing is running.
pub fn create_not_running_response(current_state: &str) -> StopExecutionResponse {
    StopExecutionResponse {
        success: true,
        was_running: false,
        story_id: None,
        message: format!("No execution in progress. Current state: {}", current_state),
    }
}

/// Get the story ID if execution is currently running.
pub fn get_running_story_id(state: &ExecutionState) -> Option<String> {
    match state {
        ExecutionState::Running { story_id, .. } => Some(story_id.clone()),
        _ => None,
    }
}

/// Get a human-readable description of the current state.
pub fn state_description(state: &ExecutionState) -> &'static str {
    match state {
        ExecutionState::Idle => "idle",
        ExecutionState::Running { .. } => "running",
        ExecutionState::Completed { .. } => "completed",
        ExecutionState::Failed { .. } => "failed",
        ExecutionState::Paused { .. } => "paused",
        ExecutionState::WaitingForRetry { .. } => "waiting_for_retry",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_execution_request_empty() {
        // Verify the request struct can be deserialized from empty JSON
        let json = "{}";
        let req: StopExecutionRequest = serde_json::from_str(json).unwrap();
        // The struct exists and can be created
        let _ = req;
    }

    #[test]
    fn test_stop_execution_request_schema() {
        // Verify the request implements JsonSchema
        let schema = schemars::schema_for!(StopExecutionRequest);
        let json = serde_json::to_string_pretty(&schema).unwrap();
        // Schema should be a valid JSON object
        assert!(json.contains("$schema"));
    }

    #[test]
    fn test_create_cancelled_response() {
        let response = create_cancelled_response("US-001");

        assert!(response.success);
        assert!(response.was_running);
        assert_eq!(response.story_id, Some("US-001".to_string()));
        assert!(response.message.contains("Cancellation signal sent"));
        assert!(response.message.contains("US-001"));
    }

    #[test]
    fn test_create_not_running_response_idle() {
        let response = create_not_running_response("idle");

        assert!(response.success);
        assert!(!response.was_running);
        assert!(response.story_id.is_none());
        assert!(response.message.contains("No execution in progress"));
        assert!(response.message.contains("idle"));
    }

    #[test]
    fn test_create_not_running_response_completed() {
        let response = create_not_running_response("completed");

        assert!(response.success);
        assert!(!response.was_running);
        assert!(response.message.contains("completed"));
    }

    #[test]
    fn test_create_not_running_response_failed() {
        let response = create_not_running_response("failed");

        assert!(response.success);
        assert!(!response.was_running);
        assert!(response.message.contains("failed"));
    }

    #[test]
    fn test_get_running_story_id_idle() {
        let state = ExecutionState::Idle;
        assert!(get_running_story_id(&state).is_none());
    }

    #[test]
    fn test_get_running_story_id_running() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 5,
            max_iterations: 10,
        };
        let result = get_running_story_id(&state);
        assert_eq!(result, Some("US-001".to_string()));
    }

    #[test]
    fn test_get_running_story_id_completed() {
        let state = ExecutionState::Completed {
            story_id: "US-001".to_string(),
            commit_hash: Some("abc123".to_string()),
        };
        assert!(get_running_story_id(&state).is_none());
    }

    #[test]
    fn test_get_running_story_id_failed() {
        let state = ExecutionState::Failed {
            story_id: "US-001".to_string(),
            error: "Test error".to_string(),
        };
        assert!(get_running_story_id(&state).is_none());
    }

    #[test]
    fn test_state_description_idle() {
        let state = ExecutionState::Idle;
        assert_eq!(state_description(&state), "idle");
    }

    #[test]
    fn test_state_description_running() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 1,
            max_iterations: 10,
        };
        assert_eq!(state_description(&state), "running");
    }

    #[test]
    fn test_state_description_completed() {
        let state = ExecutionState::Completed {
            story_id: "US-001".to_string(),
            commit_hash: Some("abc123".to_string()),
        };
        assert_eq!(state_description(&state), "completed");
    }

    #[test]
    fn test_state_description_failed() {
        let state = ExecutionState::Failed {
            story_id: "US-001".to_string(),
            error: "Test error".to_string(),
        };
        assert_eq!(state_description(&state), "failed");
    }

    #[test]
    fn test_response_serialization_with_story_id() {
        let response = StopExecutionResponse {
            success: true,
            was_running: true,
            story_id: Some("US-001".to_string()),
            message: "Cancelled".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"was_running\":true"));
        assert!(json.contains("\"story_id\":\"US-001\""));
        assert!(json.contains("\"message\":\"Cancelled\""));
    }

    #[test]
    fn test_response_serialization_without_story_id() {
        let response = StopExecutionResponse {
            success: true,
            was_running: false,
            story_id: None,
            message: "Nothing running".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"was_running\":false"));
        // story_id should be omitted when None
        assert!(!json.contains("story_id"));
        assert!(json.contains("\"message\":\"Nothing running\""));
    }
}
