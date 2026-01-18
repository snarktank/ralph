// load_prd MCP tool implementation
// This tool loads a PRD file into the Ralph MCP server

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Request parameters for the load_prd tool.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LoadPrdRequest {
    /// Path to the PRD JSON file to load.
    /// Can be absolute or relative to the current working directory.
    #[schemars(description = "Path to the PRD JSON file to load")]
    pub path: String,
}

/// Response from the load_prd tool.
#[derive(Debug, Serialize)]
pub struct LoadPrdResponse {
    /// Whether the PRD was loaded successfully
    pub success: bool,
    /// Number of user stories in the PRD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_count: Option<usize>,
    /// The project name from the PRD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// The branch name from the PRD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    /// Success or error message
    pub message: String,
}

/// PRD structure for validation.
#[derive(Debug, Deserialize)]
pub struct PrdFile {
    /// Project name
    pub project: String,
    /// Branch name for the feature
    #[serde(rename = "branchName")]
    pub branch_name: String,
    /// Project description
    #[serde(default)]
    pub description: String,
    /// List of user stories
    #[serde(rename = "userStories")]
    pub user_stories: Vec<PrdUserStory>,
    /// Configuration for parallel story execution
    #[serde(default)]
    pub parallel: Option<ParallelConfig>,
}

/// Strategy for handling conflicts in parallel execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ParallelConflictStrategy {
    /// Detect conflicts based on file paths (target_files).
    /// Stories modifying the same files cannot run concurrently.
    #[default]
    FileBased,
    /// Detect conflicts based on entity references.
    /// Stories referencing the same entities cannot run concurrently.
    EntityBased,
    /// No conflict detection. All ready stories can run concurrently.
    None,
}

/// Mode for inferring dependencies between stories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InferenceMode {
    /// Automatically infer dependencies from target file patterns.
    #[default]
    Auto,
    /// Only use explicitly declared dependencies.
    Explicit,
    /// Disable dependency inference entirely.
    Disabled,
}

/// Configuration for parallel story execution in the PRD.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParallelConfig {
    /// Whether parallel execution is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Maximum number of stories to execute concurrently.
    #[serde(rename = "maxConcurrency", default = "default_max_concurrency")]
    pub max_concurrency: u32,
    /// Strategy for detecting and handling conflicts.
    #[serde(rename = "conflictStrategy", default)]
    pub conflict_strategy: ParallelConflictStrategy,
    /// Mode for inferring dependencies between stories.
    #[serde(rename = "inferenceMode", default)]
    pub inference_mode: InferenceMode,
}

fn default_max_concurrency() -> u32 {
    3
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_concurrency: default_max_concurrency(),
            conflict_strategy: ParallelConflictStrategy::default(),
            inference_mode: InferenceMode::default(),
        }
    }
}

/// User story structure for validation.
#[derive(Debug, Deserialize)]
pub struct PrdUserStory {
    /// Story ID (e.g., "US-001")
    pub id: String,
    /// Story title
    pub title: String,
    /// Story description
    #[serde(default)]
    pub description: String,
    /// Acceptance criteria
    #[serde(rename = "acceptanceCriteria", default)]
    pub acceptance_criteria: Vec<String>,
    /// Priority (lower is higher priority)
    pub priority: u32,
    /// Whether the story passes
    pub passes: bool,
    /// IDs of stories this story depends on
    #[serde(rename = "dependsOn", default)]
    pub depends_on: Vec<String>,
    /// Files that this story will modify (for conflict detection)
    #[serde(rename = "targetFiles", default)]
    pub target_files: Vec<String>,
}

/// Validation error types for PRD files.
#[derive(Debug)]
pub enum PrdValidationError {
    /// File does not exist
    FileNotFound(String),
    /// File cannot be read
    ReadError(String),
    /// JSON parsing failed
    ParseError(String),
    /// PRD structure is invalid
    StructureError(String),
}

impl std::fmt::Display for PrdValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrdValidationError::FileNotFound(path) => {
                write!(f, "PRD file not found: {}", path)
            }
            PrdValidationError::ReadError(msg) => {
                write!(f, "Failed to read PRD file: {}", msg)
            }
            PrdValidationError::ParseError(msg) => {
                write!(f, "Failed to parse PRD JSON: {}", msg)
            }
            PrdValidationError::StructureError(msg) => {
                write!(f, "Invalid PRD structure: {}", msg)
            }
        }
    }
}

/// Validate a PRD file and return parsed content.
///
/// # Arguments
///
/// * `path` - Path to the PRD JSON file
///
/// # Returns
///
/// Result containing the parsed PRD or a validation error
pub fn validate_prd(path: &Path) -> Result<PrdFile, PrdValidationError> {
    // Check if file exists
    if !path.exists() {
        return Err(PrdValidationError::FileNotFound(path.display().to_string()));
    }

    // Read file content
    let content =
        fs::read_to_string(path).map_err(|e| PrdValidationError::ReadError(e.to_string()))?;

    // Parse JSON
    let prd: PrdFile = serde_json::from_str(&content)
        .map_err(|e| PrdValidationError::ParseError(e.to_string()))?;

    // Validate structure
    validate_prd_structure(&prd)?;

    Ok(prd)
}

/// Validate the structure of a parsed PRD.
fn validate_prd_structure(prd: &PrdFile) -> Result<(), PrdValidationError> {
    // Check project name is not empty
    if prd.project.trim().is_empty() {
        return Err(PrdValidationError::StructureError(
            "Project name is empty".to_string(),
        ));
    }

    // Check branch name is not empty
    if prd.branch_name.trim().is_empty() {
        return Err(PrdValidationError::StructureError(
            "Branch name is empty".to_string(),
        ));
    }

    // Check there's at least one user story
    if prd.user_stories.is_empty() {
        return Err(PrdValidationError::StructureError(
            "No user stories found".to_string(),
        ));
    }

    // Validate each user story
    for story in &prd.user_stories {
        if story.id.trim().is_empty() {
            return Err(PrdValidationError::StructureError(
                "User story has empty ID".to_string(),
            ));
        }
        if story.title.trim().is_empty() {
            return Err(PrdValidationError::StructureError(format!(
                "User story {} has empty title",
                story.id
            )));
        }
    }

    Ok(())
}

/// Create a success response for load_prd.
pub fn create_success_response(prd: &PrdFile) -> LoadPrdResponse {
    LoadPrdResponse {
        success: true,
        story_count: Some(prd.user_stories.len()),
        project: Some(prd.project.clone()),
        branch_name: Some(prd.branch_name.clone()),
        message: format!(
            "Successfully loaded PRD '{}' with {} user stories",
            prd.project,
            prd.user_stories.len()
        ),
    }
}

/// Create an error response for load_prd.
pub fn create_error_response(error: &PrdValidationError) -> LoadPrdResponse {
    LoadPrdResponse {
        success: false,
        story_count: None,
        project: None,
        branch_name: None,
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_valid_prd() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "description": "Test PRD",
            "userStories": [
                {
                    "id": "US-001",
                    "title": "First story",
                    "description": "A test story",
                    "acceptanceCriteria": ["AC1", "AC2"],
                    "priority": 1,
                    "passes": false
                },
                {
                    "id": "US-002",
                    "title": "Second story",
                    "description": "Another test story",
                    "acceptanceCriteria": ["AC3"],
                    "priority": 2,
                    "passes": true
                }
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_validate_prd_success() {
        let prd_file = create_valid_prd();
        let result = validate_prd(prd_file.path());
        assert!(result.is_ok());

        let prd = result.unwrap();
        assert_eq!(prd.project, "TestProject");
        assert_eq!(prd.branch_name, "feature/test");
        assert_eq!(prd.user_stories.len(), 2);
    }

    #[test]
    fn test_validate_prd_file_not_found() {
        let result = validate_prd(Path::new("/nonexistent/path.json"));
        assert!(result.is_err());

        match result.unwrap_err() {
            PrdValidationError::FileNotFound(_) => {}
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_validate_prd_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"not valid json").unwrap();

        let result = validate_prd(file.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            PrdValidationError::ParseError(_) => {}
            _ => panic!("Expected ParseError error"),
        }
    }

    #[test]
    fn test_validate_prd_empty_project() {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "project": "",
            "branchName": "main",
            "userStories": [{"id": "US-001", "title": "Test", "priority": 1, "passes": false}]
        }"#;
        file.write_all(content.as_bytes()).unwrap();

        let result = validate_prd(file.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            PrdValidationError::StructureError(msg) => {
                assert!(msg.contains("Project name is empty"));
            }
            _ => panic!("Expected StructureError error"),
        }
    }

    #[test]
    fn test_validate_prd_empty_branch_name() {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "project": "Test",
            "branchName": "",
            "userStories": [{"id": "US-001", "title": "Test", "priority": 1, "passes": false}]
        }"#;
        file.write_all(content.as_bytes()).unwrap();

        let result = validate_prd(file.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            PrdValidationError::StructureError(msg) => {
                assert!(msg.contains("Branch name is empty"));
            }
            _ => panic!("Expected StructureError error"),
        }
    }

    #[test]
    fn test_validate_prd_no_stories() {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "project": "Test",
            "branchName": "main",
            "userStories": []
        }"#;
        file.write_all(content.as_bytes()).unwrap();

        let result = validate_prd(file.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            PrdValidationError::StructureError(msg) => {
                assert!(msg.contains("No user stories found"));
            }
            _ => panic!("Expected StructureError error"),
        }
    }

    #[test]
    fn test_validate_prd_empty_story_id() {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "project": "Test",
            "branchName": "main",
            "userStories": [{"id": "", "title": "Test", "priority": 1, "passes": false}]
        }"#;
        file.write_all(content.as_bytes()).unwrap();

        let result = validate_prd(file.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            PrdValidationError::StructureError(msg) => {
                assert!(msg.contains("empty ID"));
            }
            _ => panic!("Expected StructureError error"),
        }
    }

    #[test]
    fn test_validate_prd_empty_story_title() {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "project": "Test",
            "branchName": "main",
            "userStories": [{"id": "US-001", "title": "", "priority": 1, "passes": false}]
        }"#;
        file.write_all(content.as_bytes()).unwrap();

        let result = validate_prd(file.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            PrdValidationError::StructureError(msg) => {
                assert!(msg.contains("empty title"));
            }
            _ => panic!("Expected StructureError error"),
        }
    }

    #[test]
    fn test_create_success_response() {
        let prd_file = create_valid_prd();
        let prd = validate_prd(prd_file.path()).unwrap();
        let response = create_success_response(&prd);

        assert!(response.success);
        assert_eq!(response.story_count, Some(2));
        assert_eq!(response.project, Some("TestProject".to_string()));
        assert_eq!(response.branch_name, Some("feature/test".to_string()));
        assert!(response.message.contains("Successfully loaded"));
    }

    #[test]
    fn test_create_error_response() {
        let error = PrdValidationError::FileNotFound("/test/path.json".to_string());
        let response = create_error_response(&error);

        assert!(!response.success);
        assert!(response.story_count.is_none());
        assert!(response.project.is_none());
        assert!(response.branch_name.is_none());
        assert!(response.message.contains("not found"));
    }

    #[test]
    fn test_load_prd_response_serialization() {
        let response = LoadPrdResponse {
            success: true,
            story_count: Some(5),
            project: Some("Test".to_string()),
            branch_name: Some("main".to_string()),
            message: "Success".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"story_count\":5"));
        assert!(json.contains("\"project\":\"Test\""));
    }

    #[test]
    fn test_load_prd_response_none_fields_not_serialized() {
        let response = LoadPrdResponse {
            success: false,
            story_count: None,
            project: None,
            branch_name: None,
            message: "Error".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("story_count"));
        assert!(!json.contains("project"));
        assert!(!json.contains("branch_name"));
    }

    #[test]
    fn test_prd_validation_error_display() {
        let error = PrdValidationError::FileNotFound("/test.json".to_string());
        assert_eq!(error.to_string(), "PRD file not found: /test.json");

        let error = PrdValidationError::ReadError("Permission denied".to_string());
        assert_eq!(
            error.to_string(),
            "Failed to read PRD file: Permission denied"
        );

        let error = PrdValidationError::ParseError("Unexpected token".to_string());
        assert_eq!(
            error.to_string(),
            "Failed to parse PRD JSON: Unexpected token"
        );

        let error = PrdValidationError::StructureError("Missing field".to_string());
        assert_eq!(error.to_string(), "Invalid PRD structure: Missing field");
    }

    #[test]
    fn test_deserialize_story_without_depends_on() {
        let json = r#"{
            "id": "US-001",
            "title": "Test Story",
            "description": "A story without dependencies",
            "acceptanceCriteria": ["AC1"],
            "priority": 1,
            "passes": false
        }"#;

        let story: PrdUserStory = serde_json::from_str(json).unwrap();
        assert_eq!(story.id, "US-001");
        assert_eq!(story.title, "Test Story");
        assert!(story.depends_on.is_empty());
    }

    #[test]
    fn test_deserialize_story_with_depends_on() {
        let json = r#"{
            "id": "US-002",
            "title": "Dependent Story",
            "description": "A story with dependencies",
            "acceptanceCriteria": ["AC1"],
            "priority": 2,
            "passes": false,
            "dependsOn": ["US-001", "US-003"]
        }"#;

        let story: PrdUserStory = serde_json::from_str(json).unwrap();
        assert_eq!(story.id, "US-002");
        assert_eq!(story.title, "Dependent Story");
        assert_eq!(story.depends_on, vec!["US-001", "US-003"]);
    }

    #[test]
    fn test_deserialize_story_with_empty_depends_on() {
        let json = r#"{
            "id": "US-001",
            "title": "Story with empty deps",
            "priority": 1,
            "passes": false,
            "dependsOn": []
        }"#;

        let story: PrdUserStory = serde_json::from_str(json).unwrap();
        assert!(story.depends_on.is_empty());
    }

    #[test]
    fn test_deserialize_story_without_target_files() {
        let json = r#"{
            "id": "US-001",
            "title": "Test Story",
            "description": "A story without target files",
            "acceptanceCriteria": ["AC1"],
            "priority": 1,
            "passes": false
        }"#;

        let story: PrdUserStory = serde_json::from_str(json).unwrap();
        assert_eq!(story.id, "US-001");
        assert_eq!(story.title, "Test Story");
        assert!(story.target_files.is_empty());
    }

    #[test]
    fn test_deserialize_story_with_target_files() {
        let json = r#"{
            "id": "US-002",
            "title": "Story with target files",
            "description": "A story that modifies specific files",
            "acceptanceCriteria": ["AC1"],
            "priority": 2,
            "passes": false,
            "targetFiles": ["src/main.rs", "src/lib.rs", "Cargo.toml"]
        }"#;

        let story: PrdUserStory = serde_json::from_str(json).unwrap();
        assert_eq!(story.id, "US-002");
        assert_eq!(story.title, "Story with target files");
        assert_eq!(
            story.target_files,
            vec!["src/main.rs", "src/lib.rs", "Cargo.toml"]
        );
    }

    #[test]
    fn test_deserialize_story_with_empty_target_files() {
        let json = r#"{
            "id": "US-001",
            "title": "Story with empty target files",
            "priority": 1,
            "passes": false,
            "targetFiles": []
        }"#;

        let story: PrdUserStory = serde_json::from_str(json).unwrap();
        assert!(story.target_files.is_empty());
    }

    #[test]
    fn test_deserialize_prd_without_parallel() {
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "description": "Test PRD",
            "userStories": [
                {
                    "id": "US-001",
                    "title": "First story",
                    "priority": 1,
                    "passes": false
                }
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        let prd = validate_prd(file.path()).unwrap();
        assert!(prd.parallel.is_none());
    }

    #[test]
    fn test_deserialize_prd_with_parallel_enabled() {
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "description": "Test PRD",
            "userStories": [
                {
                    "id": "US-001",
                    "title": "First story",
                    "priority": 1,
                    "passes": false
                }
            ],
            "parallel": {
                "enabled": true,
                "maxConcurrency": 5,
                "conflictStrategy": "entity_based",
                "inferenceMode": "explicit"
            }
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        let prd = validate_prd(file.path()).unwrap();
        assert!(prd.parallel.is_some());
        let parallel = prd.parallel.unwrap();
        assert!(parallel.enabled);
        assert_eq!(parallel.max_concurrency, 5);
        assert_eq!(
            parallel.conflict_strategy,
            ParallelConflictStrategy::EntityBased
        );
        assert_eq!(parallel.inference_mode, InferenceMode::Explicit);
    }

    #[test]
    fn test_deserialize_prd_with_partial_parallel_config() {
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "description": "Test PRD",
            "userStories": [
                {
                    "id": "US-001",
                    "title": "First story",
                    "priority": 1,
                    "passes": false
                }
            ],
            "parallel": {
                "enabled": true
            }
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        let prd = validate_prd(file.path()).unwrap();
        assert!(prd.parallel.is_some());
        let parallel = prd.parallel.unwrap();
        assert!(parallel.enabled);
        // Check defaults are applied
        assert_eq!(parallel.max_concurrency, 3);
        assert_eq!(
            parallel.conflict_strategy,
            ParallelConflictStrategy::FileBased
        );
        assert_eq!(parallel.inference_mode, InferenceMode::Auto);
    }

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_concurrency, 3);
        assert_eq!(
            config.conflict_strategy,
            ParallelConflictStrategy::FileBased
        );
        assert_eq!(config.inference_mode, InferenceMode::Auto);
    }

    #[test]
    fn test_parallel_conflict_strategy_serialize() {
        let strategy = ParallelConflictStrategy::FileBased;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"file_based\"");

        let strategy = ParallelConflictStrategy::EntityBased;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"entity_based\"");

        let strategy = ParallelConflictStrategy::None;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"none\"");
    }

    #[test]
    fn test_inference_mode_serialize() {
        let mode = InferenceMode::Auto;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"auto\"");

        let mode = InferenceMode::Explicit;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"explicit\"");

        let mode = InferenceMode::Disabled;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"disabled\"");
    }
}
