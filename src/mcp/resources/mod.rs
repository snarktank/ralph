// MCP Resources module for Ralph
// This module contains the MCP resource implementations

#![allow(dead_code)]

use crate::mcp::server::{ExecutionState, RalphMcpServer};
use rmcp::model::{
    Annotated, ListResourcesResult, RawResource, ReadResourceRequestParam, ReadResourceResult,
    Resource, ResourceContents,
};
use serde::Serialize;

/// URI for the current PRD resource
pub const PRD_RESOURCE_URI: &str = "ralph://prd/current";
/// URI for the execution status resource
pub const STATUS_RESOURCE_URI: &str = "ralph://status";

/// Create the list of available Ralph resources.
///
/// This returns the resources that can be accessed via the MCP resources/read method:
/// - `ralph://prd/current` - The currently loaded PRD file contents
/// - `ralph://status` - The current execution status
pub fn list_ralph_resources() -> ListResourcesResult {
    let prd_resource = Resource {
        raw: RawResource {
            uri: PRD_RESOURCE_URI.to_string(),
            name: "prd".to_string(),
            title: Some("Current PRD".to_string()),
            description: Some(
                "The currently loaded PRD (Product Requirements Document) file contents"
                    .to_string(),
            ),
            mime_type: Some("application/json".to_string()),
            size: None,
            icons: None,
        },
        annotations: None,
    };

    let status_resource = Resource {
        raw: RawResource {
            uri: STATUS_RESOURCE_URI.to_string(),
            name: "status".to_string(),
            title: Some("Execution Status".to_string()),
            description: Some(
                "Current Ralph execution status including state and progress information"
                    .to_string(),
            ),
            mime_type: Some("application/json".to_string()),
            size: None,
            icons: None,
        },
        annotations: None,
    };

    ListResourcesResult {
        resources: vec![prd_resource, status_resource],
        next_cursor: None,
    }
}

/// Status response for the ralph://status resource
#[derive(Debug, Clone, Serialize)]
pub struct StatusResource {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iteration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
}

impl StatusResource {
    /// Create a StatusResource from an ExecutionState
    pub fn from_execution_state(state: &ExecutionState) -> Self {
        match state {
            ExecutionState::Idle => StatusResource {
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
                let progress = if *max_iterations > 0 {
                    ((*iteration as f32 / *max_iterations as f32) * 100.0) as u32
                } else {
                    0
                };
                StatusResource {
                    state: "running".to_string(),
                    story_id: Some(story_id.clone()),
                    started_at: Some(*started_at),
                    iteration: Some(*iteration),
                    max_iterations: Some(*max_iterations),
                    progress_percent: Some(progress),
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
            } => StatusResource {
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
            ExecutionState::Failed { story_id, error } => StatusResource {
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
            } => StatusResource {
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
            } => StatusResource {
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

/// Error types for resource reading
#[derive(Debug)]
pub enum ResourceError {
    /// The requested resource URI is not recognized
    UnknownResource(String),
    /// No PRD is currently loaded
    NoPrdLoaded,
    /// Failed to read the PRD file
    PrdReadError(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::UnknownResource(uri) => write!(f, "Unknown resource: {}", uri),
            ResourceError::NoPrdLoaded => {
                write!(
                    f,
                    "No PRD loaded. Use the load_prd tool to load a PRD first."
                )
            }
            ResourceError::PrdReadError(msg) => write!(f, "Failed to read PRD: {}", msg),
        }
    }
}

impl std::error::Error for ResourceError {}

/// Read the ralph://prd/current resource.
///
/// Returns the contents of the currently loaded PRD file as JSON.
pub fn read_prd_resource(
    prd_path: &Option<std::path::PathBuf>,
) -> Result<ResourceContents, ResourceError> {
    match prd_path {
        Some(path) => {
            let contents = std::fs::read_to_string(path)
                .map_err(|e| ResourceError::PrdReadError(e.to_string()))?;

            Ok(ResourceContents::TextResourceContents {
                uri: PRD_RESOURCE_URI.to_string(),
                mime_type: Some("application/json".to_string()),
                text: contents,
                meta: None,
            })
        }
        None => Err(ResourceError::NoPrdLoaded),
    }
}

/// Read the ralph://status resource.
///
/// Returns the current execution status as JSON.
pub fn read_status_resource(execution_state: &ExecutionState) -> ResourceContents {
    let status = StatusResource::from_execution_state(execution_state);
    let json = serde_json::to_string_pretty(&status).unwrap_or_else(|_| "{}".to_string());

    ResourceContents::TextResourceContents {
        uri: STATUS_RESOURCE_URI.to_string(),
        mime_type: Some("application/json".to_string()),
        text: json,
        meta: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_list_ralph_resources() {
        let result = list_ralph_resources();

        assert_eq!(result.resources.len(), 2);

        // Check PRD resource
        let prd_resource = &result.resources[0];
        assert_eq!(prd_resource.raw.uri, PRD_RESOURCE_URI);
        assert_eq!(prd_resource.raw.name, "prd");
        assert_eq!(
            prd_resource.raw.mime_type,
            Some("application/json".to_string())
        );

        // Check status resource
        let status_resource = &result.resources[1];
        assert_eq!(status_resource.raw.uri, STATUS_RESOURCE_URI);
        assert_eq!(status_resource.raw.name, "status");
        assert_eq!(
            status_resource.raw.mime_type,
            Some("application/json".to_string())
        );
    }

    #[test]
    fn test_status_resource_idle() {
        let state = ExecutionState::Idle;
        let status = StatusResource::from_execution_state(&state);

        assert_eq!(status.state, "idle");
        assert!(status.story_id.is_none());
        assert!(status.progress_percent.is_none());
    }

    #[test]
    fn test_status_resource_running() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 5,
            max_iterations: 10,
        };
        let status = StatusResource::from_execution_state(&state);

        assert_eq!(status.state, "running");
        assert_eq!(status.story_id, Some("US-001".to_string()));
        assert_eq!(status.started_at, Some(1234567890));
        assert_eq!(status.iteration, Some(5));
        assert_eq!(status.max_iterations, Some(10));
        assert_eq!(status.progress_percent, Some(50));
    }

    #[test]
    fn test_status_resource_completed() {
        let state = ExecutionState::Completed {
            story_id: "US-001".to_string(),
            commit_hash: Some("abc123".to_string()),
        };
        let status = StatusResource::from_execution_state(&state);

        assert_eq!(status.state, "completed");
        assert_eq!(status.story_id, Some("US-001".to_string()));
        assert_eq!(status.commit_hash, Some("abc123".to_string()));
        assert_eq!(status.progress_percent, Some(100));
    }

    #[test]
    fn test_status_resource_failed() {
        let state = ExecutionState::Failed {
            story_id: "US-001".to_string(),
            error: "Build failed".to_string(),
        };
        let status = StatusResource::from_execution_state(&state);

        assert_eq!(status.state, "failed");
        assert_eq!(status.story_id, Some("US-001".to_string()));
        assert_eq!(status.error, Some("Build failed".to_string()));
        assert!(status.progress_percent.is_none());
    }

    #[test]
    fn test_read_prd_resource_no_prd_loaded() {
        let result = read_prd_resource(&None);
        assert!(matches!(result, Err(ResourceError::NoPrdLoaded)));
    }

    #[test]
    fn test_read_prd_resource_success() {
        // Create a temporary PRD file
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{"project": "Test", "branchName": "main", "userStories": []}"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        let result = read_prd_resource(&Some(file.path().to_path_buf()));
        assert!(result.is_ok());

        let contents = result.unwrap();
        match contents {
            ResourceContents::TextResourceContents {
                uri,
                mime_type,
                text,
                ..
            } => {
                assert_eq!(uri, PRD_RESOURCE_URI);
                assert_eq!(mime_type, Some("application/json".to_string()));
                assert_eq!(text, prd_content);
            }
            _ => panic!("Expected TextResourceContents"),
        }
    }

    #[test]
    fn test_read_prd_resource_file_not_found() {
        let result = read_prd_resource(&Some(std::path::PathBuf::from("/nonexistent/prd.json")));
        assert!(matches!(result, Err(ResourceError::PrdReadError(_))));
    }

    #[test]
    fn test_read_status_resource_idle() {
        let state = ExecutionState::Idle;
        let contents = read_status_resource(&state);

        match contents {
            ResourceContents::TextResourceContents {
                uri,
                mime_type,
                text,
                ..
            } => {
                assert_eq!(uri, STATUS_RESOURCE_URI);
                assert_eq!(mime_type, Some("application/json".to_string()));

                // Parse and verify the JSON
                let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(json["state"], "idle");
            }
            _ => panic!("Expected TextResourceContents"),
        }
    }

    #[test]
    fn test_read_status_resource_running() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 3,
            max_iterations: 10,
        };
        let contents = read_status_resource(&state);

        match contents {
            ResourceContents::TextResourceContents { text, .. } => {
                let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(json["state"], "running");
                assert_eq!(json["story_id"], "US-001");
                assert_eq!(json["iteration"], 3);
                assert_eq!(json["progress_percent"], 30);
            }
            _ => panic!("Expected TextResourceContents"),
        }
    }

    #[test]
    fn test_resource_error_display() {
        let err = ResourceError::UnknownResource("invalid://uri".to_string());
        assert!(err.to_string().contains("Unknown resource"));

        let err = ResourceError::NoPrdLoaded;
        assert!(err.to_string().contains("No PRD loaded"));

        let err = ResourceError::PrdReadError("file not found".to_string());
        assert!(err.to_string().contains("Failed to read PRD"));
    }
}
