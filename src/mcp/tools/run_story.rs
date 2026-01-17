// run_story MCP tool implementation
// This tool executes a user story from the loaded PRD

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::mcp::server::ExecutionState;

/// Request parameters for the run_story tool.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RunStoryRequest {
    /// The ID of the story to execute (e.g., "US-001").
    #[schemars(description = "The ID of the story to execute")]
    pub story_id: String,
    /// Maximum number of iterations to attempt (default: 10).
    #[schemars(description = "Maximum iterations to attempt (default: 10)")]
    #[serde(default)]
    pub max_iterations: Option<u32>,
}

/// Response from the run_story tool.
#[derive(Debug, Serialize)]
pub struct RunStoryResponse {
    /// Whether the execution was started/completed successfully
    pub success: bool,
    /// The story ID being executed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_id: Option<String>,
    /// The story title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_title: Option<String>,
    /// Git commit hash if completed successfully
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
    /// Message describing the result
    pub message: String,
}

/// Minimal story structure for finding a story by ID.
#[derive(Debug, Deserialize)]
pub struct PrdStory {
    /// Story ID
    pub id: String,
    /// Story title
    pub title: String,
    /// Whether the story passes
    pub passes: bool,
}

/// Minimal PRD structure for finding stories.
#[derive(Debug, Deserialize)]
pub struct Prd {
    /// List of user stories
    #[serde(rename = "userStories")]
    pub user_stories: Vec<PrdStory>,
}

/// Error types for run_story operations.
#[derive(Debug)]
pub enum RunStoryError {
    /// No PRD is loaded
    NoPrdLoaded,
    /// Story not found in PRD
    StoryNotFound(String),
    /// Already executing a story
    AlreadyRunning(String),
    /// PRD file read error
    PrdReadError(String),
    /// PRD parse error
    PrdParseError(String),
    /// Execution error
    ExecutionError(String),
}

impl std::fmt::Display for RunStoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStoryError::NoPrdLoaded => {
                write!(
                    f,
                    "No PRD loaded. Use load_prd tool to load a PRD file first."
                )
            }
            RunStoryError::StoryNotFound(id) => {
                write!(f, "Story '{}' not found in the loaded PRD", id)
            }
            RunStoryError::AlreadyRunning(id) => {
                write!(
                    f,
                    "Already executing story '{}'. Use stop_execution to cancel first.",
                    id
                )
            }
            RunStoryError::PrdReadError(msg) => {
                write!(f, "Failed to read PRD file: {}", msg)
            }
            RunStoryError::PrdParseError(msg) => {
                write!(f, "Failed to parse PRD file: {}", msg)
            }
            RunStoryError::ExecutionError(msg) => {
                write!(f, "Execution error: {}", msg)
            }
        }
    }
}

/// Find a story by ID in the PRD file.
pub fn find_story(prd_path: &Path, story_id: &str) -> Result<PrdStory, RunStoryError> {
    // Read the PRD file
    let content =
        fs::read_to_string(prd_path).map_err(|e| RunStoryError::PrdReadError(e.to_string()))?;

    // Parse the PRD JSON
    let prd: Prd =
        serde_json::from_str(&content).map_err(|e| RunStoryError::PrdParseError(e.to_string()))?;

    // Find the story with matching ID
    prd.user_stories
        .into_iter()
        .find(|s| s.id == story_id)
        .ok_or_else(|| RunStoryError::StoryNotFound(story_id.to_string()))
}

/// Check if execution is currently running and return the story ID if so.
pub fn check_already_running(state: &ExecutionState) -> Option<String> {
    match state {
        ExecutionState::Running { story_id, .. } => Some(story_id.clone()),
        _ => None,
    }
}

/// Get the current Unix timestamp.
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Create a success response for run_story.
pub fn create_success_response(story: &PrdStory, commit_hash: Option<String>) -> RunStoryResponse {
    RunStoryResponse {
        success: true,
        story_id: Some(story.id.clone()),
        story_title: Some(story.title.clone()),
        commit_hash,
        message: format!(
            "Successfully executed story '{}': {}",
            story.id, story.title
        ),
    }
}

/// Create an error response for run_story.
pub fn create_error_response(error: &RunStoryError) -> RunStoryResponse {
    RunStoryResponse {
        success: false,
        story_id: None,
        story_title: None,
        commit_hash: None,
        message: error.to_string(),
    }
}

/// Create a started response when execution begins.
pub fn create_started_response(story: &PrdStory, max_iterations: u32) -> RunStoryResponse {
    RunStoryResponse {
        success: true,
        story_id: Some(story.id.clone()),
        story_title: Some(story.title.clone()),
        commit_hash: None,
        message: format!(
            "Started execution of story '{}': {} (max {} iterations)",
            story.id, story.title, max_iterations
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_prd() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "project": "Test",
            "branchName": "main",
            "userStories": [
                {"id": "US-001", "title": "First story", "priority": 1, "passes": false},
                {"id": "US-002", "title": "Second story", "priority": 2, "passes": true}
            ]
        }"#;
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_find_story_success() {
        let prd_file = create_test_prd();
        let result = find_story(prd_file.path(), "US-001");
        assert!(result.is_ok());

        let story = result.unwrap();
        assert_eq!(story.id, "US-001");
        assert_eq!(story.title, "First story");
        assert!(!story.passes);
    }

    #[test]
    fn test_find_story_second() {
        let prd_file = create_test_prd();
        let result = find_story(prd_file.path(), "US-002");
        assert!(result.is_ok());

        let story = result.unwrap();
        assert_eq!(story.id, "US-002");
        assert_eq!(story.title, "Second story");
        assert!(story.passes);
    }

    #[test]
    fn test_find_story_not_found() {
        let prd_file = create_test_prd();
        let result = find_story(prd_file.path(), "US-999");
        assert!(result.is_err());

        match result.unwrap_err() {
            RunStoryError::StoryNotFound(id) => {
                assert_eq!(id, "US-999");
            }
            _ => panic!("Expected StoryNotFound error"),
        }
    }

    #[test]
    fn test_find_story_file_not_found() {
        let result = find_story(Path::new("/nonexistent/path.json"), "US-001");
        assert!(result.is_err());

        match result.unwrap_err() {
            RunStoryError::PrdReadError(_) => {}
            _ => panic!("Expected PrdReadError"),
        }
    }

    #[test]
    fn test_find_story_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"not valid json").unwrap();

        let result = find_story(file.path(), "US-001");
        assert!(result.is_err());

        match result.unwrap_err() {
            RunStoryError::PrdParseError(_) => {}
            _ => panic!("Expected PrdParseError"),
        }
    }

    #[test]
    fn test_check_already_running_idle() {
        let state = ExecutionState::Idle;
        assert!(check_already_running(&state).is_none());
    }

    #[test]
    fn test_check_already_running_running() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 1,
            max_iterations: 10,
        };
        let result = check_already_running(&state);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "US-001");
    }

    #[test]
    fn test_check_already_running_completed() {
        let state = ExecutionState::Completed {
            story_id: "US-001".to_string(),
            commit_hash: Some("abc123".to_string()),
        };
        assert!(check_already_running(&state).is_none());
    }

    #[test]
    fn test_check_already_running_failed() {
        let state = ExecutionState::Failed {
            story_id: "US-001".to_string(),
            error: "Test error".to_string(),
        };
        assert!(check_already_running(&state).is_none());
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        // Should be after 2024-01-01 (Unix timestamp 1704067200)
        assert!(ts > 1704067200);
    }

    #[test]
    fn test_create_success_response() {
        let story = PrdStory {
            id: "US-001".to_string(),
            title: "Test story".to_string(),
            passes: false,
        };
        let response = create_success_response(&story, Some("abc123def".to_string()));

        assert!(response.success);
        assert_eq!(response.story_id, Some("US-001".to_string()));
        assert_eq!(response.story_title, Some("Test story".to_string()));
        assert_eq!(response.commit_hash, Some("abc123def".to_string()));
        assert!(response.message.contains("Successfully executed"));
    }

    #[test]
    fn test_create_success_response_no_commit() {
        let story = PrdStory {
            id: "US-001".to_string(),
            title: "Test story".to_string(),
            passes: false,
        };
        let response = create_success_response(&story, None);

        assert!(response.success);
        assert!(response.commit_hash.is_none());
    }

    #[test]
    fn test_create_error_response() {
        let error = RunStoryError::StoryNotFound("US-999".to_string());
        let response = create_error_response(&error);

        assert!(!response.success);
        assert!(response.story_id.is_none());
        assert!(response.story_title.is_none());
        assert!(response.commit_hash.is_none());
        assert!(response.message.contains("US-999"));
        assert!(response.message.contains("not found"));
    }

    #[test]
    fn test_create_started_response() {
        let story = PrdStory {
            id: "US-001".to_string(),
            title: "Test story".to_string(),
            passes: false,
        };
        let response = create_started_response(&story, 10);

        assert!(response.success);
        assert_eq!(response.story_id, Some("US-001".to_string()));
        assert_eq!(response.story_title, Some("Test story".to_string()));
        assert!(response.commit_hash.is_none());
        assert!(response.message.contains("Started execution"));
        assert!(response.message.contains("10 iterations"));
    }

    #[test]
    fn test_run_story_error_display() {
        assert!(RunStoryError::NoPrdLoaded
            .to_string()
            .contains("No PRD loaded"));

        assert!(RunStoryError::StoryNotFound("US-001".to_string())
            .to_string()
            .contains("US-001"));

        assert!(RunStoryError::AlreadyRunning("US-002".to_string())
            .to_string()
            .contains("Already executing"));

        assert!(RunStoryError::PrdReadError("Permission denied".to_string())
            .to_string()
            .contains("Permission denied"));

        assert!(RunStoryError::PrdParseError("Syntax error".to_string())
            .to_string()
            .contains("Syntax error"));

        assert!(RunStoryError::ExecutionError("Build failed".to_string())
            .to_string()
            .contains("Build failed"));
    }

    #[test]
    fn test_run_story_response_serialization() {
        let response = RunStoryResponse {
            success: true,
            story_id: Some("US-001".to_string()),
            story_title: Some("Test".to_string()),
            commit_hash: Some("abc123".to_string()),
            message: "Success".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"story_id\":\"US-001\""));
        assert!(json.contains("\"commit_hash\":\"abc123\""));
    }

    #[test]
    fn test_run_story_response_none_fields_not_serialized() {
        let response = RunStoryResponse {
            success: false,
            story_id: None,
            story_title: None,
            commit_hash: None,
            message: "Error".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("story_id"));
        assert!(!json.contains("story_title"));
        assert!(!json.contains("commit_hash"));
    }

    #[test]
    fn test_run_story_request_deserialization() {
        let json = r#"{"story_id": "US-001", "max_iterations": 5}"#;
        let req: RunStoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.story_id, "US-001");
        assert_eq!(req.max_iterations, Some(5));
    }

    #[test]
    fn test_run_story_request_default_max_iterations() {
        let json = r#"{"story_id": "US-001"}"#;
        let req: RunStoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.story_id, "US-001");
        assert!(req.max_iterations.is_none());
    }
}
