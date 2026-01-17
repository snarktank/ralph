//! GitHub Projects V2 integration provider
//!
//! This module implements the ProjectTracker trait for GitHub Projects V2,
//! allowing Ralph to sync story progress to GitHub project boards.

#![allow(dead_code)]

use async_trait::async_trait;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

use super::traits::{
    CreateItemRequest, FailureIssueRequest, ItemInfo, ItemStatus, ProjectTracker, TrackerError,
    TrackerResult, UpdateItemRequest,
};

/// Configuration for GitHub Projects provider
#[derive(Debug, Clone)]
pub struct GitHubConfig {
    /// GitHub personal access token (PAT) with project permissions
    pub token: String,
    /// Repository owner (user or organization)
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// GitHub Project V2 number
    pub project_number: u64,
}

impl GitHubConfig {
    /// Create a new GitHubConfig from environment variables
    ///
    /// Expects:
    /// - GITHUB_TOKEN: Personal access token
    /// - GITHUB_OWNER: Repository owner
    /// - GITHUB_REPO: Repository name
    /// - GITHUB_PROJECT_NUMBER: Project V2 number
    pub fn from_env() -> TrackerResult<Self> {
        let token = std::env::var("GITHUB_TOKEN").map_err(|_| {
            TrackerError::ConfigError("GITHUB_TOKEN environment variable not set".to_string())
        })?;

        let owner = std::env::var("GITHUB_OWNER").map_err(|_| {
            TrackerError::ConfigError("GITHUB_OWNER environment variable not set".to_string())
        })?;

        let repo = std::env::var("GITHUB_REPO").map_err(|_| {
            TrackerError::ConfigError("GITHUB_REPO environment variable not set".to_string())
        })?;

        let project_number = std::env::var("GITHUB_PROJECT_NUMBER")
            .map_err(|_| {
                TrackerError::ConfigError(
                    "GITHUB_PROJECT_NUMBER environment variable not set".to_string(),
                )
            })?
            .parse::<u64>()
            .map_err(|_| {
                TrackerError::ConfigError(
                    "GITHUB_PROJECT_NUMBER must be a valid number".to_string(),
                )
            })?;

        Ok(Self {
            token,
            owner,
            repo,
            project_number,
        })
    }

    /// Create a new GitHubConfig with explicit values
    pub fn new(token: String, owner: String, repo: String, project_number: u64) -> Self {
        Self {
            token,
            owner,
            repo,
            project_number,
        }
    }
}

/// GitHub Projects V2 provider
///
/// Implements the ProjectTracker trait using GitHub's GraphQL API
/// to manage items in GitHub Projects V2 boards.
pub struct GitHubProjectsProvider {
    /// Octocrab client for GitHub API requests
    client: Octocrab,
    /// Provider configuration
    config: GitHubConfig,
    /// Cached project ID (fetched on first use)
    project_id: Option<String>,
}

impl GitHubProjectsProvider {
    /// Create a new GitHub Projects provider
    pub fn new(config: GitHubConfig) -> TrackerResult<Self> {
        let client = Octocrab::builder()
            .personal_token(config.token.clone())
            .build()
            .map_err(|e| {
                TrackerError::ConfigError(format!("Failed to create GitHub client: {}", e))
            })?;

        Ok(Self {
            client,
            config,
            project_id: None,
        })
    }

    /// Create a new provider from environment variables
    pub fn from_env() -> TrackerResult<Self> {
        let config = GitHubConfig::from_env()?;
        Self::new(config)
    }

    /// Get the project ID, fetching it if not cached
    async fn get_project_id(&mut self) -> TrackerResult<String> {
        if let Some(ref id) = self.project_id {
            return Ok(id.clone());
        }

        let project_id = self.fetch_project_id().await?;
        self.project_id = Some(project_id.clone());
        Ok(project_id)
    }

    /// Fetch the project ID using GraphQL
    async fn fetch_project_id(&self) -> TrackerResult<String> {
        // GraphQL query to get project ID by owner and project number
        let query = format!(
            r#"query {{
                user(login: "{owner}") {{
                    projectV2(number: {number}) {{
                        id
                    }}
                }}
            }}"#,
            owner = self.config.owner,
            number = self.config.project_number
        );

        let response: serde_json::Value = self
            .client
            .graphql(&serde_json::json!({ "query": query }))
            .await
            .map_err(|e| TrackerError::ApiError(format!("GraphQL query failed: {}", e)))?;

        // Try user first, then organization
        if let Some(id) = response
            .get("data")
            .and_then(|d| d.get("user"))
            .and_then(|u| u.get("projectV2"))
            .and_then(|p| p.get("id"))
            .and_then(|i| i.as_str())
        {
            return Ok(id.to_string());
        }

        // Try organization
        let org_query = format!(
            r#"query {{
                organization(login: "{owner}") {{
                    projectV2(number: {number}) {{
                        id
                    }}
                }}
            }}"#,
            owner = self.config.owner,
            number = self.config.project_number
        );

        let org_response: serde_json::Value = self
            .client
            .graphql(&serde_json::json!({ "query": org_query }))
            .await
            .map_err(|e| TrackerError::ApiError(format!("GraphQL query failed: {}", e)))?;

        if let Some(id) = org_response
            .get("data")
            .and_then(|d| d.get("organization"))
            .and_then(|o| o.get("projectV2"))
            .and_then(|p| p.get("id"))
            .and_then(|i| i.as_str())
        {
            return Ok(id.to_string());
        }

        Err(TrackerError::ItemNotFound(format!(
            "Project {} not found for owner {}",
            self.config.project_number, self.config.owner
        )))
    }

    /// Fetch the status field information from the project
    ///
    /// Returns the field ID and available status options
    async fn fetch_status_field(&self, project_id: &str) -> TrackerResult<ProjectFieldInfo> {
        let query = format!(
            r#"query {{
                node(id: "{project_id}") {{
                    ... on ProjectV2 {{
                        fields(first: 20) {{
                            nodes {{
                                ... on ProjectV2SingleSelectField {{
                                    id
                                    name
                                    options {{
                                        id
                                        name
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
            }}"#,
            project_id = project_id
        );

        let response: serde_json::Value = self
            .client
            .graphql(&serde_json::json!({ "query": query }))
            .await
            .map_err(|e| TrackerError::ApiError(format!("GraphQL query failed: {}", e)))?;

        // Check for errors
        if let Some(errors) = response.get("errors") {
            let error_msg = errors
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown GraphQL error");
            return Err(TrackerError::ApiError(error_msg.to_string()));
        }

        // Find the Status field
        let fields = response
            .get("data")
            .and_then(|d| d.get("node"))
            .and_then(|n| n.get("fields"))
            .and_then(|f| f.get("nodes"))
            .and_then(|n| n.as_array())
            .ok_or_else(|| TrackerError::ApiError("Failed to get project fields".to_string()))?;

        for field in fields {
            let field_name = field.get("name").and_then(|n| n.as_str()).unwrap_or("");

            // Look for a field named "Status" (case-insensitive)
            if field_name.eq_ignore_ascii_case("status") {
                let field_id = field
                    .get("id")
                    .and_then(|i| i.as_str())
                    .ok_or_else(|| TrackerError::ApiError("Status field missing ID".to_string()))?;

                let options = field
                    .get("options")
                    .and_then(|o| o.as_array())
                    .map(|opts| {
                        opts.iter()
                            .filter_map(|opt| {
                                let id = opt.get("id").and_then(|i| i.as_str())?;
                                let name = opt.get("name").and_then(|n| n.as_str())?;
                                Some(FieldOption {
                                    id: id.to_string(),
                                    name: name.to_string(),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                return Ok(ProjectFieldInfo {
                    id: field_id.to_string(),
                    name: field_name.to_string(),
                    options,
                });
            }
        }

        Err(TrackerError::ItemNotFound(
            "Status field not found in project".to_string(),
        ))
    }

    /// Map ItemStatus to GitHub Project status option name
    fn map_status_to_github(&self, status: ItemStatus) -> &'static str {
        match status {
            ItemStatus::Todo => "Todo",
            ItemStatus::InProgress => "In Progress",
            ItemStatus::InReview => "In Review",
            ItemStatus::Done => "Done",
            ItemStatus::Blocked => "Blocked",
            ItemStatus::Cancelled => "Cancelled",
        }
    }

    /// Find the option ID for a given status in the field info
    fn find_status_option(
        &self,
        field_info: &ProjectFieldInfo,
        status: ItemStatus,
    ) -> TrackerResult<String> {
        let github_status = self.map_status_to_github(status);

        // Try exact match first
        if let Some(option) = field_info.options.iter().find(|o| o.name == github_status) {
            return Ok(option.id.clone());
        }

        // Try case-insensitive match
        let github_status_lower = github_status.to_lowercase();
        if let Some(option) = field_info
            .options
            .iter()
            .find(|o| o.name.to_lowercase() == github_status_lower)
        {
            return Ok(option.id.clone());
        }

        // Try matching without spaces (e.g., "InProgress" vs "In Progress")
        let github_status_no_spaces = github_status_lower.replace(' ', "");
        if let Some(option) = field_info
            .options
            .iter()
            .find(|o| o.name.to_lowercase().replace(' ', "") == github_status_no_spaces)
        {
            return Ok(option.id.clone());
        }

        Err(TrackerError::InvalidInput(format!(
            "Status option '{}' not found in project. Available options: {}",
            github_status,
            field_info
                .options
                .iter()
                .map(|o| o.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )))
    }

    /// Update a project item's field value using GraphQL mutation
    async fn update_item_field_value(
        &self,
        project_id: &str,
        item_id: &str,
        field_id: &str,
        option_id: &str,
    ) -> TrackerResult<()> {
        let mutation = format!(
            r#"mutation {{
                updateProjectV2ItemFieldValue(input: {{
                    projectId: "{project_id}",
                    itemId: "{item_id}",
                    fieldId: "{field_id}",
                    value: {{
                        singleSelectOptionId: "{option_id}"
                    }}
                }}) {{
                    projectV2Item {{
                        id
                    }}
                }}
            }}"#,
            project_id = project_id,
            item_id = item_id,
            field_id = field_id,
            option_id = option_id
        );

        let response: serde_json::Value = self
            .client
            .graphql(&serde_json::json!({ "query": mutation }))
            .await
            .map_err(|e| TrackerError::ApiError(format!("GraphQL mutation failed: {}", e)))?;

        // Check for errors
        if let Some(errors) = response.get("errors") {
            let error_msg = errors
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown GraphQL error");
            return Err(TrackerError::ApiError(error_msg.to_string()));
        }

        // Verify the update was successful
        if response
            .get("data")
            .and_then(|d| d.get("updateProjectV2ItemFieldValue"))
            .and_then(|u| u.get("projectV2Item"))
            .and_then(|p| p.get("id"))
            .is_none()
        {
            return Err(TrackerError::ApiError(
                "Failed to verify field update".to_string(),
            ));
        }

        Ok(())
    }

    /// Format the body of a failure issue
    fn format_failure_issue_body(&self, request: &FailureIssueRequest) -> String {
        let mut body = String::new();

        body.push_str("## Ralph Execution Failure\n\n");

        body.push_str("### Story Information\n\n");
        body.push_str(&format!("- **Story ID:** {}\n", request.story_id));
        body.push_str(&format!("- **Story Title:** {}\n\n", request.story_title));

        body.push_str("### Error Details\n\n");
        body.push_str("```\n");
        body.push_str(&request.error);
        body.push_str("\n```\n\n");

        if let Some(ref context) = request.context {
            body.push_str("### Additional Context\n\n");
            body.push_str("<details>\n<summary>Click to expand</summary>\n\n");
            body.push_str("```\n");
            body.push_str(context);
            body.push_str("\n```\n\n");
            body.push_str("</details>\n\n");
        }

        body.push_str("---\n");
        body.push_str("*This issue was automatically created by Ralph autonomous agent.*\n");

        body
    }

    /// Add a draft item to the project using GraphQL addProjectV2DraftIssue mutation
    async fn add_draft_item(
        &self,
        project_id: &str,
        title: &str,
        body: Option<&str>,
    ) -> TrackerResult<AddDraftItemResponse> {
        let body_value = body
            .map(|b| format!(r#", body: "{}""#, escape_graphql_string(b)))
            .unwrap_or_default();

        let mutation = format!(
            r#"mutation {{
                addProjectV2DraftIssue(input: {{
                    projectId: "{project_id}",
                    title: "{title}"{body}
                }}) {{
                    projectItem {{
                        id
                    }}
                }}
            }}"#,
            project_id = project_id,
            title = escape_graphql_string(title),
            body = body_value
        );

        let response: serde_json::Value = self
            .client
            .graphql(&serde_json::json!({ "query": mutation }))
            .await
            .map_err(|e| TrackerError::ApiError(format!("GraphQL mutation failed: {}", e)))?;

        // Check for errors
        if let Some(errors) = response.get("errors") {
            let error_msg = errors
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown GraphQL error");
            return Err(TrackerError::ApiError(error_msg.to_string()));
        }

        let item_id = response
            .get("data")
            .and_then(|d| d.get("addProjectV2DraftIssue"))
            .and_then(|a| a.get("projectItem"))
            .and_then(|p| p.get("id"))
            .and_then(|i| i.as_str())
            .ok_or_else(|| {
                TrackerError::ApiError("Failed to get item ID from response".to_string())
            })?;

        Ok(AddDraftItemResponse {
            item_id: item_id.to_string(),
        })
    }
}

/// Response from addProjectV2DraftIssue mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddDraftItemResponse {
    item_id: String,
}

/// Information about a project field
#[derive(Debug, Clone)]
struct ProjectFieldInfo {
    /// Field ID
    id: String,
    /// Field name
    name: String,
    /// Field options (for single select fields)
    options: Vec<FieldOption>,
}

/// A single select field option
#[derive(Debug, Clone)]
struct FieldOption {
    /// Option ID
    id: String,
    /// Option name
    name: String,
}

/// Escape special characters for GraphQL string values
fn escape_graphql_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[async_trait]
impl ProjectTracker for GitHubProjectsProvider {
    fn name(&self) -> &str {
        "github"
    }

    async fn create_item(&self, request: CreateItemRequest) -> TrackerResult<ItemInfo> {
        // We need mutable access for caching, but the trait requires &self
        // For now, fetch project ID each time (can be optimized with interior mutability)
        let project_id = self.fetch_project_id().await?;

        let response = self
            .add_draft_item(&project_id, &request.title, request.description.as_deref())
            .await?;

        Ok(ItemInfo {
            id: response.item_id,
            title: request.title,
            url: None, // Draft items don't have a direct URL
        })
    }

    async fn update_item(
        &self,
        _item_id: &str,
        _request: UpdateItemRequest,
    ) -> TrackerResult<ItemInfo> {
        // Will be implemented in US-029
        Err(TrackerError::ApiError(
            "update_item not yet implemented".to_string(),
        ))
    }

    async fn create_failure_issue(&self, request: FailureIssueRequest) -> TrackerResult<ItemInfo> {
        // Create issue using GitHub REST API
        let title = format!("[Ralph Failure] {}", request.story_title);

        // Build issue body with error details and context
        let body = self.format_failure_issue_body(&request);

        // Create the issue using octocrab's issues API
        let issue = self
            .client
            .issues(&self.config.owner, &self.config.repo)
            .create(&title)
            .body(&body)
            .labels(vec!["ralph-failure".to_string()])
            .send()
            .await
            .map_err(|e| TrackerError::ApiError(format!("Failed to create issue: {}", e)))?;

        Ok(ItemInfo {
            id: issue.number.to_string(),
            title: issue.title.clone(),
            url: Some(issue.html_url.to_string()),
        })
    }

    async fn add_comment(&self, _item_id: &str, _comment: &str) -> TrackerResult<()> {
        // Will be implemented in a future story
        Err(TrackerError::ApiError(
            "add_comment not yet implemented".to_string(),
        ))
    }

    async fn update_status(&self, item_id: &str, status: ItemStatus) -> TrackerResult<ItemInfo> {
        // Fetch project ID
        let project_id = self.fetch_project_id().await?;

        // Fetch status field information
        let status_field = self.fetch_status_field(&project_id).await?;

        // Find the option ID for the requested status
        let option_id = self.find_status_option(&status_field, status)?;

        // Update the field value
        self.update_item_field_value(&project_id, item_id, &status_field.id, &option_id)
            .await?;

        Ok(ItemInfo {
            id: item_id.to_string(),
            title: format!("Status updated to {}", self.map_status_to_github(status)),
            url: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config_new() {
        let config = GitHubConfig::new(
            "ghp_test_token".to_string(),
            "test-owner".to_string(),
            "test-repo".to_string(),
            42,
        );

        assert_eq!(config.token, "ghp_test_token");
        assert_eq!(config.owner, "test-owner");
        assert_eq!(config.repo, "test-repo");
        assert_eq!(config.project_number, 42);
    }

    #[test]
    fn test_escape_graphql_string() {
        assert_eq!(escape_graphql_string("hello"), "hello");
        assert_eq!(escape_graphql_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_graphql_string("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_graphql_string("tab\there"), "tab\\there");
        assert_eq!(escape_graphql_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_escape_graphql_string_complex() {
        let input = "Line 1\nLine 2\r\nWith \"quotes\" and \\backslash";
        let escaped = escape_graphql_string(input);
        assert_eq!(
            escaped,
            "Line 1\\nLine 2\\r\\nWith \\\"quotes\\\" and \\\\backslash"
        );
    }

    #[test]
    fn test_add_draft_item_response() {
        let response = AddDraftItemResponse {
            item_id: "PVTI_test123".to_string(),
        };
        assert_eq!(response.item_id, "PVTI_test123");
    }

    #[tokio::test]
    async fn test_provider_name() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();
        assert_eq!(provider.name(), "github");
    }

    #[test]
    fn test_github_config_from_env_missing_vars() {
        // Test that creating config with no env vars fails appropriately
        // Note: tests run in parallel, so we can't reliably test specific env var order
        // Instead, test with explicit config which is deterministic
        let config = GitHubConfig::new(
            "".to_string(), // empty token
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        // Empty token is technically valid for the struct, but octocrab will reject it
        assert!(config.token.is_empty());

        // Test that from_env returns ConfigError when any var is missing
        // by temporarily clearing just GITHUB_TOKEN (the first one checked)
        let orig_token = std::env::var("GITHUB_TOKEN").ok();
        std::env::remove_var("GITHUB_TOKEN");

        let result = GitHubConfig::from_env();

        // Restore original value
        if let Some(v) = orig_token {
            std::env::set_var("GITHUB_TOKEN", v);
        }

        // Should get a ConfigError
        match result {
            Err(TrackerError::ConfigError(_)) => {} // Expected
            _ => panic!("Expected ConfigError, got: {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_create_item_request_construction() {
        let request = CreateItemRequest {
            title: "Test Story".to_string(),
            description: Some("Test description".to_string()),
            status: Some(ItemStatus::Todo),
            labels: vec!["enhancement".to_string()],
        };

        assert_eq!(request.title, "Test Story");
        assert_eq!(request.description, Some("Test description".to_string()));
        assert_eq!(request.status, Some(ItemStatus::Todo));
        assert_eq!(request.labels, vec!["enhancement".to_string()]);
    }

    #[test]
    fn test_github_config_parse_project_number() {
        // Test that invalid project numbers are properly rejected
        // Instead of using env vars (which causes race conditions in parallel tests),
        // we test the parsing logic directly
        let invalid_num = "not_a_number".parse::<u64>();
        assert!(invalid_num.is_err());

        let valid_num = "42".parse::<u64>();
        assert!(valid_num.is_ok());
        assert_eq!(valid_num.unwrap(), 42);

        // Verify the config struct accepts valid numbers
        let config = GitHubConfig::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            42,
        );
        assert_eq!(config.project_number, 42);
    }

    #[tokio::test]
    async fn test_update_item_not_implemented() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();
        let request = UpdateItemRequest {
            title: None,
            description: None,
            status: Some(ItemStatus::Done),
            add_labels: vec![],
            remove_labels: vec![],
        };

        let result = provider.update_item("item-123", request).await;
        assert!(result.is_err());
        if let Err(TrackerError::ApiError(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        }
    }

    #[tokio::test]
    async fn test_format_failure_issue_body_basic() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();
        let request = FailureIssueRequest {
            story_id: "US-001".to_string(),
            story_title: "Test story".to_string(),
            error: "Test error message".to_string(),
            context: None,
        };

        let body = provider.format_failure_issue_body(&request);

        // Check that body contains expected sections
        assert!(body.contains("## Ralph Execution Failure"));
        assert!(body.contains("### Story Information"));
        assert!(body.contains("**Story ID:** US-001"));
        assert!(body.contains("**Story Title:** Test story"));
        assert!(body.contains("### Error Details"));
        assert!(body.contains("Test error message"));
        assert!(body.contains("*This issue was automatically created by Ralph autonomous agent.*"));

        // Should NOT contain context section when context is None
        assert!(!body.contains("### Additional Context"));
    }

    #[tokio::test]
    async fn test_format_failure_issue_body_with_context() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();
        let request = FailureIssueRequest {
            story_id: "US-002".to_string(),
            story_title: "Another story".to_string(),
            error: "Compilation failed".to_string(),
            context: Some("Stack trace:\n  at main.rs:42".to_string()),
        };

        let body = provider.format_failure_issue_body(&request);

        // Check that body contains context section
        assert!(body.contains("### Additional Context"));
        assert!(body.contains("<details>"));
        assert!(body.contains("<summary>Click to expand</summary>"));
        assert!(body.contains("Stack trace:\n  at main.rs:42"));
        assert!(body.contains("</details>"));
    }

    #[test]
    fn test_failure_issue_title_format() {
        // Test that title is formatted correctly
        let story_title = "Create user authentication";
        let expected_title = format!("[Ralph Failure] {}", story_title);
        assert_eq!(expected_title, "[Ralph Failure] Create user authentication");
    }

    #[tokio::test]
    async fn test_format_failure_issue_body_special_characters() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();
        let request = FailureIssueRequest {
            story_id: "US-003".to_string(),
            story_title: "Test with \"quotes\" and <html>".to_string(),
            error: "Error with `backticks` and *asterisks*".to_string(),
            context: None,
        };

        let body = provider.format_failure_issue_body(&request);

        // Body should contain the special characters as-is (markdown rendering will handle them)
        assert!(body.contains("\"quotes\""));
        assert!(body.contains("<html>"));
        assert!(body.contains("`backticks`"));
        assert!(body.contains("*asterisks*"));
    }

    #[tokio::test]
    async fn test_add_comment_not_implemented() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();

        let result = provider.add_comment("item-123", "Test comment").await;
        assert!(result.is_err());
        if let Err(TrackerError::ApiError(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        }
    }

    #[tokio::test]
    async fn test_map_status_to_github() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();

        assert_eq!(provider.map_status_to_github(ItemStatus::Todo), "Todo");
        assert_eq!(
            provider.map_status_to_github(ItemStatus::InProgress),
            "In Progress"
        );
        assert_eq!(
            provider.map_status_to_github(ItemStatus::InReview),
            "In Review"
        );
        assert_eq!(provider.map_status_to_github(ItemStatus::Done), "Done");
        assert_eq!(
            provider.map_status_to_github(ItemStatus::Blocked),
            "Blocked"
        );
        assert_eq!(
            provider.map_status_to_github(ItemStatus::Cancelled),
            "Cancelled"
        );
    }

    #[tokio::test]
    async fn test_find_status_option_exact_match() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();

        let field_info = ProjectFieldInfo {
            id: "PVTSSF_test".to_string(),
            name: "Status".to_string(),
            options: vec![
                FieldOption {
                    id: "opt_1".to_string(),
                    name: "Todo".to_string(),
                },
                FieldOption {
                    id: "opt_2".to_string(),
                    name: "In Progress".to_string(),
                },
                FieldOption {
                    id: "opt_3".to_string(),
                    name: "Done".to_string(),
                },
            ],
        };

        let result = provider.find_status_option(&field_info, ItemStatus::Todo);
        assert_eq!(result.unwrap(), "opt_1");

        let result = provider.find_status_option(&field_info, ItemStatus::InProgress);
        assert_eq!(result.unwrap(), "opt_2");

        let result = provider.find_status_option(&field_info, ItemStatus::Done);
        assert_eq!(result.unwrap(), "opt_3");
    }

    #[tokio::test]
    async fn test_find_status_option_case_insensitive() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();

        let field_info = ProjectFieldInfo {
            id: "PVTSSF_test".to_string(),
            name: "Status".to_string(),
            options: vec![
                FieldOption {
                    id: "opt_1".to_string(),
                    name: "TODO".to_string(), // uppercase
                },
                FieldOption {
                    id: "opt_2".to_string(),
                    name: "in progress".to_string(), // lowercase
                },
            ],
        };

        let result = provider.find_status_option(&field_info, ItemStatus::Todo);
        assert_eq!(result.unwrap(), "opt_1");

        let result = provider.find_status_option(&field_info, ItemStatus::InProgress);
        assert_eq!(result.unwrap(), "opt_2");
    }

    #[tokio::test]
    async fn test_find_status_option_no_spaces() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();

        let field_info = ProjectFieldInfo {
            id: "PVTSSF_test".to_string(),
            name: "Status".to_string(),
            options: vec![FieldOption {
                id: "opt_1".to_string(),
                name: "InProgress".to_string(), // no space
            }],
        };

        let result = provider.find_status_option(&field_info, ItemStatus::InProgress);
        assert_eq!(result.unwrap(), "opt_1");
    }

    #[tokio::test]
    async fn test_find_status_option_not_found() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();

        let field_info = ProjectFieldInfo {
            id: "PVTSSF_test".to_string(),
            name: "Status".to_string(),
            options: vec![FieldOption {
                id: "opt_1".to_string(),
                name: "Todo".to_string(),
            }],
        };

        let result = provider.find_status_option(&field_info, ItemStatus::Done);
        assert!(result.is_err());
        if let Err(TrackerError::InvalidInput(msg)) = result {
            assert!(msg.contains("Done"));
            assert!(msg.contains("not found"));
            assert!(msg.contains("Todo")); // Available options
        }
    }

    #[test]
    fn test_project_field_info_construction() {
        let field_info = ProjectFieldInfo {
            id: "PVTSSF_abc123".to_string(),
            name: "Status".to_string(),
            options: vec![
                FieldOption {
                    id: "opt_1".to_string(),
                    name: "Todo".to_string(),
                },
                FieldOption {
                    id: "opt_2".to_string(),
                    name: "Done".to_string(),
                },
            ],
        };

        assert_eq!(field_info.id, "PVTSSF_abc123");
        assert_eq!(field_info.name, "Status");
        assert_eq!(field_info.options.len(), 2);
        assert_eq!(field_info.options[0].name, "Todo");
        assert_eq!(field_info.options[1].name, "Done");
    }

    #[test]
    fn test_field_option_construction() {
        let option = FieldOption {
            id: "opt_test".to_string(),
            name: "In Progress".to_string(),
        };

        assert_eq!(option.id, "opt_test");
        assert_eq!(option.name, "In Progress");
    }
}
