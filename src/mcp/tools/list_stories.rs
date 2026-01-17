// list_stories MCP tool implementation
// This tool lists stories from the loaded PRD

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;

/// Request parameters for the list_stories tool.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListStoriesRequest {
    /// Optional filter for story status.
    /// If "passing", only return stories where passes=true.
    /// If "failing", only return stories where passes=false.
    /// If not specified or any other value, return all stories.
    #[schemars(description = "Filter stories by status: 'passing', 'failing', or omit for all")]
    pub status_filter: Option<String>,
}

/// A single story in the response.
#[derive(Debug, Serialize)]
pub struct StoryInfo {
    /// Story ID (e.g., "US-001")
    pub id: String,
    /// Story title
    pub title: String,
    /// Whether the story passes (true) or not (false)
    pub passes: bool,
}

/// Response from the list_stories tool.
#[derive(Debug, Serialize)]
pub struct ListStoriesResponse {
    /// List of stories matching the filter
    pub stories: Vec<StoryInfo>,
    /// Total count of stories returned
    pub count: usize,
}

/// PRD structure for parsing the PRD file.
#[derive(Debug, Deserialize)]
struct Prd {
    #[serde(rename = "userStories")]
    user_stories: Vec<PrdStory>,
}

/// Story structure from the PRD file.
#[derive(Debug, Deserialize)]
struct PrdStory {
    id: String,
    title: String,
    passes: bool,
}

/// Load and filter stories from a PRD file.
///
/// # Arguments
///
/// * `prd_path` - Path to the PRD JSON file
/// * `status_filter` - Optional filter: "passing", "failing", or None for all
///
/// # Returns
///
/// Result containing the list of stories or an error message
pub fn load_stories(
    prd_path: &std::path::Path,
    status_filter: Option<&str>,
) -> Result<ListStoriesResponse, String> {
    // Read the PRD file
    let content =
        fs::read_to_string(prd_path).map_err(|e| format!("Failed to read PRD file: {}", e))?;

    // Parse the PRD JSON
    let prd: Prd =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse PRD JSON: {}", e))?;

    // Filter stories based on status_filter
    let stories: Vec<StoryInfo> = prd
        .user_stories
        .into_iter()
        .filter(|story| {
            match status_filter {
                Some("passing") => story.passes,
                Some("failing") => !story.passes,
                _ => true, // No filter or unknown filter = return all
            }
        })
        .map(|story| StoryInfo {
            id: story.id,
            title: story.title,
            passes: story.passes,
        })
        .collect();

    let count = stories.len();
    Ok(ListStoriesResponse { stories, count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_prd() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "Test",
            "branchName": "main",
            "description": "Test PRD",
            "userStories": [
                {"id": "US-001", "title": "First story", "passes": true, "priority": 1},
                {"id": "US-002", "title": "Second story", "passes": false, "priority": 2},
                {"id": "US-003", "title": "Third story", "passes": true, "priority": 3}
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_load_stories_all() {
        let prd_file = create_test_prd();
        let result = load_stories(prd_file.path(), None).unwrap();

        assert_eq!(result.count, 3);
        assert_eq!(result.stories.len(), 3);
        assert_eq!(result.stories[0].id, "US-001");
        assert_eq!(result.stories[1].id, "US-002");
        assert_eq!(result.stories[2].id, "US-003");
    }

    #[test]
    fn test_load_stories_passing() {
        let prd_file = create_test_prd();
        let result = load_stories(prd_file.path(), Some("passing")).unwrap();

        assert_eq!(result.count, 2);
        assert!(result.stories.iter().all(|s| s.passes));
        assert_eq!(result.stories[0].id, "US-001");
        assert_eq!(result.stories[1].id, "US-003");
    }

    #[test]
    fn test_load_stories_failing() {
        let prd_file = create_test_prd();
        let result = load_stories(prd_file.path(), Some("failing")).unwrap();

        assert_eq!(result.count, 1);
        assert!(result.stories.iter().all(|s| !s.passes));
        assert_eq!(result.stories[0].id, "US-002");
    }

    #[test]
    fn test_load_stories_unknown_filter() {
        let prd_file = create_test_prd();
        let result = load_stories(prd_file.path(), Some("unknown")).unwrap();

        // Unknown filter should return all stories
        assert_eq!(result.count, 3);
    }

    #[test]
    fn test_load_stories_file_not_found() {
        let result = load_stories(std::path::Path::new("/nonexistent/path.json"), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read PRD file"));
    }

    #[test]
    fn test_load_stories_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"not valid json").unwrap();

        let result = load_stories(file.path(), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse PRD JSON"));
    }

    #[test]
    fn test_story_info_serialization() {
        let story = StoryInfo {
            id: "US-001".to_string(),
            title: "Test Story".to_string(),
            passes: true,
        };

        let json = serde_json::to_string(&story).unwrap();
        assert!(json.contains("\"id\":\"US-001\""));
        assert!(json.contains("\"title\":\"Test Story\""));
        assert!(json.contains("\"passes\":true"));
    }
}
