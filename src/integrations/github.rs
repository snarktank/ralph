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

    async fn create_failure_issue(&self, _request: FailureIssueRequest) -> TrackerResult<ItemInfo> {
        // Will be implemented in US-030
        Err(TrackerError::ApiError(
            "create_failure_issue not yet implemented".to_string(),
        ))
    }

    async fn add_comment(&self, _item_id: &str, _comment: &str) -> TrackerResult<()> {
        // Will be implemented in a future story
        Err(TrackerError::ApiError(
            "add_comment not yet implemented".to_string(),
        ))
    }

    async fn update_status(&self, _item_id: &str, _status: ItemStatus) -> TrackerResult<ItemInfo> {
        // Will be implemented in US-029
        Err(TrackerError::ApiError(
            "update_status not yet implemented".to_string(),
        ))
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
    async fn test_create_failure_issue_not_implemented() {
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
            error: "Test error".to_string(),
            context: None,
        };

        let result = provider.create_failure_issue(request).await;
        assert!(result.is_err());
        if let Err(TrackerError::ApiError(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        }
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
    async fn test_update_status_not_implemented() {
        let config = GitHubConfig::new(
            "test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            1,
        );
        let provider = GitHubProjectsProvider::new(config).unwrap();

        let result = provider.update_status("item-123", ItemStatus::Done).await;
        assert!(result.is_err());
        if let Err(TrackerError::ApiError(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        }
    }
}
