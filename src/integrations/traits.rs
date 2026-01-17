//! Project management provider traits
//!
//! This module defines the core traits for integrating with project management
//! systems like GitHub Projects and Linear.

#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Status of an item in a project management system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    /// Item is waiting to be started
    Todo,
    /// Item is currently being worked on
    InProgress,
    /// Item is waiting for review
    InReview,
    /// Item has been completed successfully
    Done,
    /// Item has been blocked or failed
    Blocked,
    /// Item has been cancelled
    Cancelled,
}

impl fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemStatus::Todo => write!(f, "todo"),
            ItemStatus::InProgress => write!(f, "in_progress"),
            ItemStatus::InReview => write!(f, "in_review"),
            ItemStatus::Done => write!(f, "done"),
            ItemStatus::Blocked => write!(f, "blocked"),
            ItemStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Error type for project tracker operations
#[derive(Debug)]
pub enum TrackerError {
    /// Authentication failed
    AuthenticationError(String),
    /// The requested item was not found
    ItemNotFound(String),
    /// Network or API error
    ApiError(String),
    /// Configuration error
    ConfigError(String),
    /// Rate limit exceeded
    RateLimitError(String),
    /// Invalid input provided
    InvalidInput(String),
}

impl std::error::Error for TrackerError {}

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackerError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            TrackerError::ItemNotFound(msg) => write!(f, "Item not found: {}", msg),
            TrackerError::ApiError(msg) => write!(f, "API error: {}", msg),
            TrackerError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            TrackerError::RateLimitError(msg) => write!(f, "Rate limit exceeded: {}", msg),
            TrackerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

/// Result type for project tracker operations
pub type TrackerResult<T> = Result<T, TrackerError>;

/// Information about a created or updated item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemInfo {
    /// Unique identifier from the provider
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// URL to view the item (if available)
    pub url: Option<String>,
}

/// Request to create a new item
#[derive(Debug, Clone)]
pub struct CreateItemRequest {
    /// Title of the item
    pub title: String,
    /// Description or body content
    pub description: Option<String>,
    /// Initial status
    pub status: Option<ItemStatus>,
    /// Labels/tags to apply
    pub labels: Vec<String>,
}

/// Request to update an existing item
#[derive(Debug, Clone)]
pub struct UpdateItemRequest {
    /// New title (if changing)
    pub title: Option<String>,
    /// New description (if changing)
    pub description: Option<String>,
    /// New status (if changing)
    pub status: Option<ItemStatus>,
    /// Labels to add
    pub add_labels: Vec<String>,
    /// Labels to remove
    pub remove_labels: Vec<String>,
}

/// Request to create a failure issue
#[derive(Debug, Clone)]
pub struct FailureIssueRequest {
    /// Story ID that failed
    pub story_id: String,
    /// Story title
    pub story_title: String,
    /// Error message
    pub error: String,
    /// Additional context (logs, stack traces, etc.)
    pub context: Option<String>,
}

/// Trait for project management system providers
///
/// This trait defines the interface for integrating Ralph with external
/// project management systems like GitHub Projects, Linear, Jira, etc.
#[async_trait]
pub trait ProjectTracker: Send + Sync {
    /// Returns the name of this provider (e.g., "github", "linear")
    fn name(&self) -> &str;

    /// Create a new item in the project management system
    ///
    /// # Arguments
    /// * `request` - Details for the item to create
    ///
    /// # Returns
    /// Information about the created item including its ID
    async fn create_item(&self, request: CreateItemRequest) -> TrackerResult<ItemInfo>;

    /// Update an existing item
    ///
    /// # Arguments
    /// * `item_id` - The provider-specific item ID
    /// * `request` - Fields to update
    ///
    /// # Returns
    /// Updated item information
    async fn update_item(
        &self,
        item_id: &str,
        request: UpdateItemRequest,
    ) -> TrackerResult<ItemInfo>;

    /// Create an issue for a failed story execution
    ///
    /// This creates a detailed issue with error information, useful for
    /// tracking failures that need manual intervention.
    ///
    /// # Arguments
    /// * `request` - Details about the failure
    ///
    /// # Returns
    /// Information about the created issue
    async fn create_failure_issue(&self, request: FailureIssueRequest) -> TrackerResult<ItemInfo>;

    /// Add a comment to an existing item
    ///
    /// # Arguments
    /// * `item_id` - The provider-specific item ID
    /// * `comment` - The comment text to add
    ///
    /// # Returns
    /// Ok(()) on success
    async fn add_comment(&self, item_id: &str, comment: &str) -> TrackerResult<()>;

    /// Update only the status of an item
    ///
    /// This is a convenience method for the common case of just changing status.
    ///
    /// # Arguments
    /// * `item_id` - The provider-specific item ID
    /// * `status` - The new status
    ///
    /// # Returns
    /// Updated item information
    async fn update_status(&self, item_id: &str, status: ItemStatus) -> TrackerResult<ItemInfo>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_status_display() {
        assert_eq!(ItemStatus::Todo.to_string(), "todo");
        assert_eq!(ItemStatus::InProgress.to_string(), "in_progress");
        assert_eq!(ItemStatus::InReview.to_string(), "in_review");
        assert_eq!(ItemStatus::Done.to_string(), "done");
        assert_eq!(ItemStatus::Blocked.to_string(), "blocked");
        assert_eq!(ItemStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_item_status_serialize() {
        let status = ItemStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");
    }

    #[test]
    fn test_item_status_deserialize() {
        let status: ItemStatus = serde_json::from_str("\"in_review\"").unwrap();
        assert_eq!(status, ItemStatus::InReview);
    }

    #[test]
    fn test_tracker_error_display() {
        let err = TrackerError::AuthenticationError("Invalid token".to_string());
        assert_eq!(err.to_string(), "Authentication error: Invalid token");

        let err = TrackerError::ItemNotFound("item-123".to_string());
        assert_eq!(err.to_string(), "Item not found: item-123");

        let err = TrackerError::ApiError("Connection refused".to_string());
        assert_eq!(err.to_string(), "API error: Connection refused");

        let err = TrackerError::ConfigError("Missing API key".to_string());
        assert_eq!(err.to_string(), "Configuration error: Missing API key");

        let err = TrackerError::RateLimitError("Too many requests".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: Too many requests");

        let err = TrackerError::InvalidInput("Empty title".to_string());
        assert_eq!(err.to_string(), "Invalid input: Empty title");
    }

    #[test]
    fn test_item_info_serialize() {
        let info = ItemInfo {
            id: "123".to_string(),
            title: "Test Item".to_string(),
            url: Some("https://example.com/item/123".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"id\":\"123\""));
        assert!(json.contains("\"title\":\"Test Item\""));
        assert!(json.contains("\"url\":\"https://example.com/item/123\""));
    }

    #[test]
    fn test_create_item_request() {
        let request = CreateItemRequest {
            title: "New Feature".to_string(),
            description: Some("Implement new feature".to_string()),
            status: Some(ItemStatus::Todo),
            labels: vec!["enhancement".to_string(), "p1".to_string()],
        };
        assert_eq!(request.title, "New Feature");
        assert_eq!(request.description.unwrap(), "Implement new feature");
        assert_eq!(request.status.unwrap(), ItemStatus::Todo);
        assert_eq!(request.labels.len(), 2);
    }

    #[test]
    fn test_update_item_request() {
        let request = UpdateItemRequest {
            title: None,
            description: None,
            status: Some(ItemStatus::Done),
            add_labels: vec!["completed".to_string()],
            remove_labels: vec!["in-progress".to_string()],
        };
        assert!(request.title.is_none());
        assert_eq!(request.status.unwrap(), ItemStatus::Done);
        assert_eq!(request.add_labels.len(), 1);
        assert_eq!(request.remove_labels.len(), 1);
    }

    #[test]
    fn test_failure_issue_request() {
        let request = FailureIssueRequest {
            story_id: "US-001".to_string(),
            story_title: "Create feature X".to_string(),
            error: "Test failed: assertion error".to_string(),
            context: Some("Stack trace here...".to_string()),
        };
        assert_eq!(request.story_id, "US-001");
        assert_eq!(request.story_title, "Create feature X");
        assert_eq!(request.error, "Test failed: assertion error");
        assert!(request.context.is_some());
    }
}
