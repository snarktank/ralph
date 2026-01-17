//! Linear integration provider
//!
//! This module implements the ProjectTracker trait for Linear,
//! allowing Ralph to sync story progress to Linear issue boards.

#![allow(dead_code)]

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::traits::{
    CreateItemRequest, FailureIssueRequest, ItemInfo, ItemStatus, ProjectTracker, TrackerError,
    TrackerResult, UpdateItemRequest,
};

/// Linear GraphQL API endpoint
const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

/// Configuration for Linear provider
#[derive(Debug, Clone)]
pub struct LinearConfig {
    /// Linear API key
    pub api_key: String,
    /// Team ID to create issues in
    pub team_id: String,
}

impl LinearConfig {
    /// Create a new LinearConfig from environment variables
    ///
    /// Expects:
    /// - LINEAR_API_KEY: Linear API key
    /// - LINEAR_TEAM_ID: Team ID to create issues in
    pub fn from_env() -> TrackerResult<Self> {
        let api_key = std::env::var("LINEAR_API_KEY").map_err(|_| {
            TrackerError::ConfigError("LINEAR_API_KEY environment variable not set".to_string())
        })?;

        let team_id = std::env::var("LINEAR_TEAM_ID").map_err(|_| {
            TrackerError::ConfigError("LINEAR_TEAM_ID environment variable not set".to_string())
        })?;

        Ok(Self { api_key, team_id })
    }

    /// Create a new LinearConfig with explicit values
    pub fn new(api_key: String, team_id: String) -> Self {
        Self { api_key, team_id }
    }
}

/// Linear provider
///
/// Implements the ProjectTracker trait using Linear's GraphQL API
/// to manage issues in Linear.
pub struct LinearProvider {
    /// HTTP client for API requests
    client: Client,
    /// Provider configuration
    config: LinearConfig,
}

impl LinearProvider {
    /// Create a new Linear provider
    pub fn new(config: LinearConfig) -> TrackerResult<Self> {
        let client = Client::builder().build().map_err(|e| {
            TrackerError::ConfigError(format!("Failed to create HTTP client: {}", e))
        })?;

        Ok(Self { client, config })
    }

    /// Create a new provider from environment variables
    pub fn from_env() -> TrackerResult<Self> {
        let config = LinearConfig::from_env()?;
        Self::new(config)
    }

    /// Execute a GraphQL query against the Linear API
    async fn execute_graphql<T: for<'de> Deserialize<'de>>(&self, query: &str) -> TrackerResult<T> {
        let request_body = serde_json::json!({
            "query": query
        });

        let response = self
            .client
            .post(LINEAR_API_URL)
            // Linear API uses the API key directly without "Bearer" prefix
            .header("Authorization", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| TrackerError::ApiError(format!("HTTP request failed: {}", e)))?;

        // Check for HTTP errors
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(TrackerError::AuthenticationError(
                "Invalid Linear API key".to_string(),
            ));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(TrackerError::RateLimitError(
                "Linear API rate limit exceeded".to_string(),
            ));
        }
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TrackerError::ApiError(format!(
                "HTTP {} error: {}",
                status, error_text
            )));
        }

        let result: T = response
            .json()
            .await
            .map_err(|e| TrackerError::ApiError(format!("Failed to parse response: {}", e)))?;

        Ok(result)
    }

    /// Create an issue using the issueCreate mutation
    async fn create_issue(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> TrackerResult<LinearIssue> {
        let description_field = description
            .map(|d| format!(r#", description: "{}""#, escape_graphql_string(d)))
            .unwrap_or_default();

        let mutation = format!(
            r#"mutation {{
                issueCreate(input: {{
                    teamId: "{team_id}",
                    title: "{title}"{description}
                }}) {{
                    success
                    issue {{
                        id
                        identifier
                        title
                        url
                    }}
                }}
            }}"#,
            team_id = self.config.team_id,
            title = escape_graphql_string(title),
            description = description_field
        );

        let response: GraphQLResponse<IssueCreateData> = self.execute_graphql(&mutation).await?;

        // Check for GraphQL errors
        if let Some(errors) = response.errors {
            let error_msg = errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown GraphQL error".to_string());
            return Err(TrackerError::ApiError(error_msg));
        }

        let data = response
            .data
            .ok_or_else(|| TrackerError::ApiError("No data in response".to_string()))?;

        if !data.issue_create.success {
            return Err(TrackerError::ApiError("Issue creation failed".to_string()));
        }

        data.issue_create
            .issue
            .ok_or_else(|| TrackerError::ApiError("No issue in response".to_string()))
    }

    /// Update an issue using the issueUpdate mutation
    async fn update_issue(
        &self,
        issue_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        state_id: Option<&str>,
    ) -> TrackerResult<LinearIssue> {
        let mut fields = Vec::new();

        if let Some(t) = title {
            fields.push(format!(r#"title: "{}""#, escape_graphql_string(t)));
        }
        if let Some(d) = description {
            fields.push(format!(r#"description: "{}""#, escape_graphql_string(d)));
        }
        if let Some(s) = state_id {
            fields.push(format!(r#"stateId: "{}""#, s));
        }

        if fields.is_empty() {
            return Err(TrackerError::InvalidInput(
                "No fields to update".to_string(),
            ));
        }

        let mutation = format!(
            r#"mutation {{
                issueUpdate(id: "{issue_id}", input: {{
                    {fields}
                }}) {{
                    success
                    issue {{
                        id
                        identifier
                        title
                        url
                    }}
                }}
            }}"#,
            issue_id = issue_id,
            fields = fields.join(", ")
        );

        let response: GraphQLResponse<IssueUpdateData> = self.execute_graphql(&mutation).await?;

        // Check for GraphQL errors
        if let Some(errors) = response.errors {
            let error_msg = errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown GraphQL error".to_string());
            return Err(TrackerError::ApiError(error_msg));
        }

        let data = response
            .data
            .ok_or_else(|| TrackerError::ApiError("No data in response".to_string()))?;

        if !data.issue_update.success {
            return Err(TrackerError::ApiError("Issue update failed".to_string()));
        }

        data.issue_update
            .issue
            .ok_or_else(|| TrackerError::ApiError("No issue in response".to_string()))
    }

    /// Fetch workflow states for the team to map ItemStatus to Linear state IDs
    async fn fetch_workflow_states(&self) -> TrackerResult<Vec<WorkflowState>> {
        let query = format!(
            r#"query {{
                team(id: "{team_id}") {{
                    states {{
                        nodes {{
                            id
                            name
                            type
                        }}
                    }}
                }}
            }}"#,
            team_id = self.config.team_id
        );

        let response: GraphQLResponse<TeamStatesData> = self.execute_graphql(&query).await?;

        // Check for GraphQL errors
        if let Some(errors) = response.errors {
            let error_msg = errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown GraphQL error".to_string());
            return Err(TrackerError::ApiError(error_msg));
        }

        let data = response
            .data
            .ok_or_else(|| TrackerError::ApiError("No data in response".to_string()))?;

        Ok(data.team.states.nodes)
    }

    /// Map an ItemStatus to a Linear workflow state ID
    ///
    /// Linear uses workflow states with types: backlog, unstarted, started, completed, canceled
    async fn find_state_id_for_status(&self, status: ItemStatus) -> TrackerResult<String> {
        let states = self.fetch_workflow_states().await?;

        // Map our ItemStatus to Linear state types and names
        let (expected_type, expected_names) = match status {
            ItemStatus::Todo => ("unstarted", vec!["todo", "to do", "backlog"]),
            ItemStatus::InProgress => ("started", vec!["in progress", "doing", "started"]),
            ItemStatus::InReview => ("started", vec!["in review", "review", "reviewing"]),
            ItemStatus::Done => ("completed", vec!["done", "completed", "closed"]),
            ItemStatus::Blocked => ("started", vec!["blocked", "on hold", "paused"]),
            ItemStatus::Cancelled => ("canceled", vec!["cancelled", "canceled", "won't do"]),
        };

        // First, try to find a state with matching name
        for state in &states {
            let state_name_lower = state.name.to_lowercase();
            for expected_name in &expected_names {
                if state_name_lower.contains(expected_name) {
                    return Ok(state.id.clone());
                }
            }
        }

        // If no name match, try to find a state with matching type
        for state in &states {
            if state.state_type == expected_type {
                return Ok(state.id.clone());
            }
        }

        // If still no match, return error with available states
        let available: Vec<String> = states
            .iter()
            .map(|s| format!("{} ({})", s.name, s.state_type))
            .collect();
        Err(TrackerError::ApiError(format!(
            "No matching state found for '{}'. Available states: {}",
            status,
            available.join(", ")
        )))
    }

    /// Add a comment to an issue using the commentCreate mutation
    async fn create_comment(&self, issue_id: &str, body: &str) -> TrackerResult<LinearComment> {
        let mutation = format!(
            r#"mutation {{
                commentCreate(input: {{
                    issueId: "{issue_id}",
                    body: "{body}"
                }}) {{
                    success
                    comment {{
                        id
                        body
                        url
                    }}
                }}
            }}"#,
            issue_id = issue_id,
            body = escape_graphql_string(body)
        );

        let response: GraphQLResponse<CommentCreateData> = self.execute_graphql(&mutation).await?;

        // Check for GraphQL errors
        if let Some(errors) = response.errors {
            let error_msg = errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown GraphQL error".to_string());
            return Err(TrackerError::ApiError(error_msg));
        }

        let data = response
            .data
            .ok_or_else(|| TrackerError::ApiError("No data in response".to_string()))?;

        if !data.comment_create.success {
            return Err(TrackerError::ApiError(
                "Comment creation failed".to_string(),
            ));
        }

        data.comment_create
            .comment
            .ok_or_else(|| TrackerError::ApiError("No comment in response".to_string()))
    }
}

/// Generic GraphQL response wrapper
#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error
#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

/// Response data from issueCreate mutation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssueCreateData {
    issue_create: IssueCreateResult,
}

/// Result of issueCreate mutation
#[derive(Debug, Deserialize)]
struct IssueCreateResult {
    success: bool,
    issue: Option<LinearIssue>,
}

/// Linear issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearIssue {
    /// Internal Linear ID (UUID)
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123")
    pub identifier: String,
    /// Issue title
    pub title: String,
    /// URL to view the issue
    pub url: String,
}

/// Response data from issueUpdate mutation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssueUpdateData {
    issue_update: IssueUpdateResult,
}

/// Result of issueUpdate mutation
#[derive(Debug, Deserialize)]
struct IssueUpdateResult {
    success: bool,
    issue: Option<LinearIssue>,
}

/// Response data from team states query
#[derive(Debug, Deserialize)]
struct TeamStatesData {
    team: TeamData,
}

/// Team data containing workflow states
#[derive(Debug, Deserialize)]
struct TeamData {
    states: StatesConnection,
}

/// States connection for pagination
#[derive(Debug, Deserialize)]
struct StatesConnection {
    nodes: Vec<WorkflowState>,
}

/// Linear workflow state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// State ID
    pub id: String,
    /// State name (e.g., "Todo", "In Progress", "Done")
    pub name: String,
    /// State type (backlog, unstarted, started, completed, canceled)
    #[serde(rename = "type")]
    pub state_type: String,
}

/// Response data from commentCreate mutation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommentCreateData {
    comment_create: CommentCreateResult,
}

/// Result of commentCreate mutation
#[derive(Debug, Deserialize)]
struct CommentCreateResult {
    success: bool,
    comment: Option<LinearComment>,
}

/// Linear comment representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearComment {
    /// Comment ID
    pub id: String,
    /// Comment body
    pub body: String,
    /// URL to view the comment
    pub url: String,
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
impl ProjectTracker for LinearProvider {
    fn name(&self) -> &str {
        "linear"
    }

    async fn create_item(&self, request: CreateItemRequest) -> TrackerResult<ItemInfo> {
        let issue = self
            .create_issue(&request.title, request.description.as_deref())
            .await?;

        Ok(ItemInfo {
            id: issue.id,
            title: issue.title,
            url: Some(issue.url),
        })
    }

    async fn update_item(
        &self,
        item_id: &str,
        request: UpdateItemRequest,
    ) -> TrackerResult<ItemInfo> {
        // If status is provided, get the state ID
        let state_id = if let Some(status) = request.status {
            Some(self.find_state_id_for_status(status).await?)
        } else {
            None
        };

        let issue = self
            .update_issue(
                item_id,
                request.title.as_deref(),
                request.description.as_deref(),
                state_id.as_deref(),
            )
            .await?;

        Ok(ItemInfo {
            id: issue.id,
            title: issue.title,
            url: Some(issue.url),
        })
    }

    async fn create_failure_issue(&self, request: FailureIssueRequest) -> TrackerResult<ItemInfo> {
        let title = format!("[Ralph Failure] {}", request.story_title);
        let body = format_failure_issue_body(&request);

        let issue = self.create_issue(&title, Some(&body)).await?;

        Ok(ItemInfo {
            id: issue.id,
            title: issue.title,
            url: Some(issue.url),
        })
    }

    async fn add_comment(&self, item_id: &str, comment: &str) -> TrackerResult<()> {
        self.create_comment(item_id, comment).await?;
        Ok(())
    }

    async fn update_status(&self, item_id: &str, status: ItemStatus) -> TrackerResult<ItemInfo> {
        // Find the state ID for the given status
        let state_id = self.find_state_id_for_status(status).await?;

        // Update the issue with the new state
        let issue = self
            .update_issue(item_id, None, None, Some(&state_id))
            .await?;

        Ok(ItemInfo {
            id: issue.id,
            title: issue.title,
            url: Some(issue.url),
        })
    }
}

/// Format the body for a failure issue
fn format_failure_issue_body(request: &FailureIssueRequest) -> String {
    let mut body = format!(
        "## Story Information\n\n\
         - **Story ID:** {}\n\
         - **Story Title:** {}\n\n\
         ## Error Details\n\n\
         ```\n{}\n```",
        request.story_id, request.story_title, request.error
    );

    if let Some(context) = &request.context {
        body.push_str(&format!(
            "\n\n<details>\n<summary>Additional Context</summary>\n\n{}\n</details>",
            context
        ));
    }

    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_config_new() {
        let config = LinearConfig::new("lin_api_test_key".to_string(), "team_test_id".to_string());

        assert_eq!(config.api_key, "lin_api_test_key");
        assert_eq!(config.team_id, "team_test_id");
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
    fn test_linear_issue_serialize() {
        let issue = LinearIssue {
            id: "abc123".to_string(),
            identifier: "ENG-42".to_string(),
            title: "Test Issue".to_string(),
            url: "https://linear.app/team/issue/ENG-42".to_string(),
        };

        let json = serde_json::to_string(&issue).unwrap();
        assert!(json.contains("\"id\":\"abc123\""));
        assert!(json.contains("\"identifier\":\"ENG-42\""));
        assert!(json.contains("\"title\":\"Test Issue\""));
        assert!(json.contains("\"url\":\"https://linear.app/team/issue/ENG-42\""));
    }

    #[test]
    fn test_linear_issue_deserialize() {
        let json = r#"{
            "id": "uuid-123",
            "identifier": "ENG-100",
            "title": "Deserialized Issue",
            "url": "https://linear.app/test"
        }"#;

        let issue: LinearIssue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.id, "uuid-123");
        assert_eq!(issue.identifier, "ENG-100");
        assert_eq!(issue.title, "Deserialized Issue");
        assert_eq!(issue.url, "https://linear.app/test");
    }

    #[test]
    fn test_linear_config_from_env_missing_vars() {
        // Save original values
        let orig_api_key = std::env::var("LINEAR_API_KEY").ok();
        let orig_team_id = std::env::var("LINEAR_TEAM_ID").ok();

        // Remove env vars
        std::env::remove_var("LINEAR_API_KEY");
        std::env::remove_var("LINEAR_TEAM_ID");

        let result = LinearConfig::from_env();

        // Restore original values
        if let Some(v) = orig_api_key {
            std::env::set_var("LINEAR_API_KEY", v);
        }
        if let Some(v) = orig_team_id {
            std::env::set_var("LINEAR_TEAM_ID", v);
        }

        // Should get a ConfigError
        match result {
            Err(TrackerError::ConfigError(msg)) => {
                assert!(msg.contains("LINEAR_API_KEY"));
            }
            _ => panic!("Expected ConfigError, got: {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_provider_name() {
        let config = LinearConfig::new("test_key".to_string(), "team_id".to_string());
        let provider = LinearProvider::new(config).unwrap();
        assert_eq!(provider.name(), "linear");
    }

    #[test]
    fn test_issue_update_data_deserialize() {
        let json = r#"{
            "data": {
                "issueUpdate": {
                    "success": true,
                    "issue": {
                        "id": "updated-id",
                        "identifier": "ENG-42",
                        "title": "Updated Title",
                        "url": "https://linear.app/test/ENG-42"
                    }
                }
            }
        }"#;

        let response: GraphQLResponse<IssueUpdateData> = serde_json::from_str(json).unwrap();
        assert!(response.errors.is_none());
        let data = response.data.unwrap();
        assert!(data.issue_update.success);
        let issue = data.issue_update.issue.unwrap();
        assert_eq!(issue.id, "updated-id");
        assert_eq!(issue.identifier, "ENG-42");
        assert_eq!(issue.title, "Updated Title");
    }

    #[test]
    fn test_workflow_state_deserialize() {
        let json = r#"{
            "id": "state-123",
            "name": "In Progress",
            "type": "started"
        }"#;

        let state: WorkflowState = serde_json::from_str(json).unwrap();
        assert_eq!(state.id, "state-123");
        assert_eq!(state.name, "In Progress");
        assert_eq!(state.state_type, "started");
    }

    #[test]
    fn test_team_states_data_deserialize() {
        let json = r#"{
            "data": {
                "team": {
                    "states": {
                        "nodes": [
                            {"id": "s1", "name": "Todo", "type": "unstarted"},
                            {"id": "s2", "name": "In Progress", "type": "started"},
                            {"id": "s3", "name": "Done", "type": "completed"}
                        ]
                    }
                }
            }
        }"#;

        let response: GraphQLResponse<TeamStatesData> = serde_json::from_str(json).unwrap();
        let data = response.data.unwrap();
        let states = &data.team.states.nodes;
        assert_eq!(states.len(), 3);
        assert_eq!(states[0].name, "Todo");
        assert_eq!(states[1].name, "In Progress");
        assert_eq!(states[2].name, "Done");
    }

    #[test]
    fn test_linear_comment_deserialize() {
        let json = r#"{
            "id": "comment-123",
            "body": "Test comment body",
            "url": "https://linear.app/test/comment"
        }"#;

        let comment: LinearComment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "comment-123");
        assert_eq!(comment.body, "Test comment body");
        assert_eq!(comment.url, "https://linear.app/test/comment");
    }

    #[test]
    fn test_comment_create_data_deserialize() {
        let json = r#"{
            "data": {
                "commentCreate": {
                    "success": true,
                    "comment": {
                        "id": "c-456",
                        "body": "New comment",
                        "url": "https://linear.app/comment/c-456"
                    }
                }
            }
        }"#;

        let response: GraphQLResponse<CommentCreateData> = serde_json::from_str(json).unwrap();
        let data = response.data.unwrap();
        assert!(data.comment_create.success);
        let comment = data.comment_create.comment.unwrap();
        assert_eq!(comment.id, "c-456");
        assert_eq!(comment.body, "New comment");
    }

    #[test]
    fn test_format_failure_issue_body_basic() {
        let request = FailureIssueRequest {
            story_id: "US-001".to_string(),
            story_title: "Test story".to_string(),
            error: "Test error message".to_string(),
            context: None,
        };

        let body = format_failure_issue_body(&request);
        assert!(body.contains("## Story Information"));
        assert!(body.contains("**Story ID:** US-001"));
        assert!(body.contains("**Story Title:** Test story"));
        assert!(body.contains("## Error Details"));
        assert!(body.contains("Test error message"));
        assert!(!body.contains("<details>"));
    }

    #[test]
    fn test_format_failure_issue_body_with_context() {
        let request = FailureIssueRequest {
            story_id: "US-002".to_string(),
            story_title: "Another story".to_string(),
            error: "Error occurred".to_string(),
            context: Some("Stack trace here".to_string()),
        };

        let body = format_failure_issue_body(&request);
        assert!(body.contains("<details>"));
        assert!(body.contains("<summary>Additional Context</summary>"));
        assert!(body.contains("Stack trace here"));
        assert!(body.contains("</details>"));
    }

    #[test]
    fn test_format_failure_issue_body_special_characters() {
        let request = FailureIssueRequest {
            story_id: "US-003".to_string(),
            story_title: "Story with \"quotes\"".to_string(),
            error: "Error with\nnewlines".to_string(),
            context: None,
        };

        let body = format_failure_issue_body(&request);
        // The body should contain the special characters (not escaped in markdown)
        assert!(body.contains("Story with \"quotes\""));
        assert!(body.contains("Error with\nnewlines"));
    }

    #[test]
    fn test_graphql_response_deserialize() {
        let json = r#"{
            "data": {
                "issueCreate": {
                    "success": true,
                    "issue": {
                        "id": "test-id",
                        "identifier": "ENG-1",
                        "title": "Test",
                        "url": "https://linear.app/test"
                    }
                }
            }
        }"#;

        let response: GraphQLResponse<IssueCreateData> = serde_json::from_str(json).unwrap();
        assert!(response.errors.is_none());
        let data = response.data.unwrap();
        assert!(data.issue_create.success);
        let issue = data.issue_create.issue.unwrap();
        assert_eq!(issue.id, "test-id");
        assert_eq!(issue.identifier, "ENG-1");
    }

    #[test]
    fn test_graphql_error_deserialize() {
        let json = r#"{
            "errors": [
                {"message": "Invalid API key"}
            ]
        }"#;

        let response: GraphQLResponse<IssueCreateData> = serde_json::from_str(json).unwrap();
        assert!(response.data.is_none());
        let errors = response.errors.unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Invalid API key");
    }

    #[test]
    fn test_create_item_request_construction() {
        let request = CreateItemRequest {
            title: "New Linear Issue".to_string(),
            description: Some("Issue description".to_string()),
            status: Some(ItemStatus::Todo),
            labels: vec!["bug".to_string()],
        };

        assert_eq!(request.title, "New Linear Issue");
        assert_eq!(request.description, Some("Issue description".to_string()));
        assert_eq!(request.status, Some(ItemStatus::Todo));
        assert_eq!(request.labels, vec!["bug".to_string()]);
    }
}
