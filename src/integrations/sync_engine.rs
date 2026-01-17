//! Sync engine for coordinating story synchronization with external providers
//!
//! This module provides the `SyncEngine` for managing synchronization between
//! Ralph's PRD stories and external project management systems (GitHub Projects, Linear, etc.).

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::registry::ProviderRegistry;
use super::traits::{
    CreateItemRequest, ItemInfo, ItemStatus, TrackerError, TrackerResult, UpdateItemRequest,
};

/// Conflict resolution strategy for bidirectional sync
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    /// PRD changes overwrite external changes
    PrdWins,
    /// External changes overwrite PRD changes
    ExternalWins,
    /// Conflicts are flagged for manual resolution
    #[default]
    Manual,
    /// Most recently modified version wins
    NewestWins,
}

/// Configuration for the sync engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Enable bidirectional sync (external changes update PRD)
    #[serde(default)]
    pub bidirectional: bool,

    /// Conflict resolution strategy when bidirectional sync is enabled
    #[serde(default)]
    pub conflict_strategy: ConflictStrategy,

    /// Sync on story completion
    #[serde(default = "default_true")]
    pub sync_on_completion: bool,

    /// Create items for new stories automatically
    #[serde(default = "default_true")]
    pub auto_create_items: bool,

    /// Update status when story passes/fails
    #[serde(default = "default_true")]
    pub sync_status: bool,
}

fn default_true() -> bool {
    true
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            bidirectional: false,
            conflict_strategy: ConflictStrategy::Manual,
            sync_on_completion: true,
            auto_create_items: true,
            sync_status: true,
        }
    }
}

/// A user story from the PRD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    /// Story identifier (e.g., "US-001")
    pub id: String,
    /// Story title
    pub title: String,
    /// Story description
    #[serde(default)]
    pub description: String,
    /// Whether the story passes its acceptance criteria
    #[serde(default)]
    pub passes: bool,
    /// Priority (lower is higher priority)
    #[serde(default)]
    pub priority: u32,
}

impl Story {
    /// Get the appropriate status based on pass/fail state
    pub fn status(&self) -> ItemStatus {
        if self.passes {
            ItemStatus::Done
        } else {
            ItemStatus::Todo
        }
    }
}

/// Result of a sync operation for a single story
#[derive(Debug, Clone, Serialize)]
pub struct SyncResult {
    /// Story ID that was synced
    pub story_id: String,
    /// Whether the sync was successful
    pub success: bool,
    /// Provider item ID (if successful)
    pub item_id: Option<String>,
    /// Provider item URL (if available)
    pub item_url: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Action taken (created, updated, skipped)
    pub action: SyncAction,
}

/// Type of action taken during sync
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncAction {
    /// A new item was created in the provider
    Created,
    /// An existing item was updated
    Updated,
    /// The story was skipped (no changes needed)
    Skipped,
    /// The sync failed
    Failed,
}

/// Summary of a batch sync operation
#[derive(Debug, Clone, Serialize)]
pub struct SyncSummary {
    /// Total number of stories processed
    pub total: usize,
    /// Number of items created
    pub created: usize,
    /// Number of items updated
    pub updated: usize,
    /// Number of stories skipped
    pub skipped: usize,
    /// Number of failures
    pub failed: usize,
    /// Individual results for each story
    pub results: Vec<SyncResult>,
}

impl SyncSummary {
    /// Create a new empty summary
    fn new() -> Self {
        Self {
            total: 0,
            created: 0,
            updated: 0,
            skipped: 0,
            failed: 0,
            results: Vec::new(),
        }
    }

    /// Add a result to the summary
    fn add_result(&mut self, result: SyncResult) {
        self.total += 1;
        match result.action {
            SyncAction::Created => self.created += 1,
            SyncAction::Updated => self.updated += 1,
            SyncAction::Skipped => self.skipped += 1,
            SyncAction::Failed => self.failed += 1,
        }
        self.results.push(result);
    }

    /// Check if all syncs were successful
    pub fn all_successful(&self) -> bool {
        self.failed == 0
    }
}

/// Engine for coordinating synchronization between PRD stories and external providers
pub struct SyncEngine {
    /// Provider registry for accessing project management providers
    registry: ProviderRegistry,
    /// Sync configuration
    config: SyncConfig,
    /// Mapping from story IDs to provider item IDs
    story_item_map: HashMap<String, String>,
}

impl SyncEngine {
    /// Create a new sync engine with the given registry and config
    ///
    /// # Arguments
    /// * `registry` - Provider registry with registered providers
    /// * `config` - Sync configuration settings
    pub fn new(registry: ProviderRegistry, config: SyncConfig) -> Self {
        Self {
            registry,
            config,
            story_item_map: HashMap::new(),
        }
    }

    /// Create a sync engine with default configuration
    ///
    /// # Arguments
    /// * `registry` - Provider registry with registered providers
    pub fn with_default_config(registry: ProviderRegistry) -> Self {
        Self::new(registry, SyncConfig::default())
    }

    /// Get a reference to the sync configuration
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    /// Get a mutable reference to the sync configuration
    pub fn config_mut(&mut self) -> &mut SyncConfig {
        &mut self.config
    }

    /// Get a reference to the provider registry
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    /// Get a mutable reference to the provider registry
    pub fn registry_mut(&mut self) -> &mut ProviderRegistry {
        &mut self.registry
    }

    /// Set the mapping from story ID to provider item ID
    ///
    /// This is used to track which provider items correspond to which stories
    /// for subsequent sync operations.
    pub fn set_item_mapping(&mut self, story_id: &str, item_id: &str) {
        self.story_item_map
            .insert(story_id.to_string(), item_id.to_string());
    }

    /// Get the provider item ID for a story
    pub fn get_item_id(&self, story_id: &str) -> Option<&str> {
        self.story_item_map.get(story_id).map(|s| s.as_str())
    }

    /// Clear all item mappings
    pub fn clear_mappings(&mut self) {
        self.story_item_map.clear();
    }

    /// Perform initial sync, pushing all stories to the active provider
    ///
    /// This creates items in the provider for all stories that don't already
    /// have corresponding items.
    ///
    /// # Arguments
    /// * `stories` - List of stories to sync
    ///
    /// # Returns
    /// Summary of the sync operation
    pub async fn initial_sync(&mut self, stories: &[Story]) -> TrackerResult<SyncSummary> {
        let provider = self.registry.get_active().ok_or_else(|| {
            TrackerError::ConfigError("No active provider configured".to_string())
        })?;

        let mut summary = SyncSummary::new();

        for story in stories {
            let result = if self.story_item_map.contains_key(&story.id) {
                // Story already has an item, skip or update based on config
                SyncResult {
                    story_id: story.id.clone(),
                    success: true,
                    item_id: self.story_item_map.get(&story.id).cloned(),
                    item_url: None,
                    error: None,
                    action: SyncAction::Skipped,
                }
            } else if self.config.auto_create_items {
                // Create new item for this story
                match self.create_item_for_story(&provider, story).await {
                    Ok(info) => {
                        self.story_item_map
                            .insert(story.id.clone(), info.id.clone());
                        SyncResult {
                            story_id: story.id.clone(),
                            success: true,
                            item_id: Some(info.id),
                            item_url: info.url,
                            error: None,
                            action: SyncAction::Created,
                        }
                    }
                    Err(e) => SyncResult {
                        story_id: story.id.clone(),
                        success: false,
                        item_id: None,
                        item_url: None,
                        error: Some(e.to_string()),
                        action: SyncAction::Failed,
                    },
                }
            } else {
                // Auto-create disabled, skip
                SyncResult {
                    story_id: story.id.clone(),
                    success: true,
                    item_id: None,
                    item_url: None,
                    error: None,
                    action: SyncAction::Skipped,
                }
            };

            summary.add_result(result);
        }

        Ok(summary)
    }

    /// Sync a single story update to the provider
    ///
    /// This updates the provider item for the given story if one exists,
    /// or creates a new item if auto_create_items is enabled.
    ///
    /// # Arguments
    /// * `story` - The story that was updated
    ///
    /// # Returns
    /// Result of the sync operation
    pub async fn sync_story_update(&mut self, story: &Story) -> TrackerResult<SyncResult> {
        let provider = self.registry.get_active().ok_or_else(|| {
            TrackerError::ConfigError("No active provider configured".to_string())
        })?;

        // Check if we have an existing item for this story
        if let Some(item_id) = self.story_item_map.get(&story.id).cloned() {
            // Update existing item
            let request = UpdateItemRequest {
                title: Some(format!("[{}] {}", story.id, story.title)),
                description: if story.description.is_empty() {
                    None
                } else {
                    Some(story.description.clone())
                },
                status: if self.config.sync_status {
                    Some(story.status())
                } else {
                    None
                },
                add_labels: vec![],
                remove_labels: vec![],
            };

            match provider.update_item(&item_id, request).await {
                Ok(info) => Ok(SyncResult {
                    story_id: story.id.clone(),
                    success: true,
                    item_id: Some(info.id),
                    item_url: info.url,
                    error: None,
                    action: SyncAction::Updated,
                }),
                Err(e) => Ok(SyncResult {
                    story_id: story.id.clone(),
                    success: false,
                    item_id: Some(item_id),
                    item_url: None,
                    error: Some(e.to_string()),
                    action: SyncAction::Failed,
                }),
            }
        } else if self.config.auto_create_items {
            // Create new item
            match self.create_item_for_story(&provider, story).await {
                Ok(info) => {
                    self.story_item_map
                        .insert(story.id.clone(), info.id.clone());
                    Ok(SyncResult {
                        story_id: story.id.clone(),
                        success: true,
                        item_id: Some(info.id),
                        item_url: info.url,
                        error: None,
                        action: SyncAction::Created,
                    })
                }
                Err(e) => Ok(SyncResult {
                    story_id: story.id.clone(),
                    success: false,
                    item_id: None,
                    item_url: None,
                    error: Some(e.to_string()),
                    action: SyncAction::Failed,
                }),
            }
        } else {
            // No existing item and auto-create disabled
            Ok(SyncResult {
                story_id: story.id.clone(),
                success: true,
                item_id: None,
                item_url: None,
                error: None,
                action: SyncAction::Skipped,
            })
        }
    }

    /// Sync story completion status to the provider
    ///
    /// This is a convenience method for syncing just the pass/fail status
    /// of a story after execution.
    ///
    /// # Arguments
    /// * `story_id` - The story ID
    /// * `passes` - Whether the story passes
    ///
    /// # Returns
    /// Result of the sync operation
    pub async fn sync_story_completion(
        &mut self,
        story_id: &str,
        passes: bool,
    ) -> TrackerResult<SyncResult> {
        if !self.config.sync_on_completion {
            return Ok(SyncResult {
                story_id: story_id.to_string(),
                success: true,
                item_id: None,
                item_url: None,
                error: None,
                action: SyncAction::Skipped,
            });
        }

        let provider = self.registry.get_active().ok_or_else(|| {
            TrackerError::ConfigError("No active provider configured".to_string())
        })?;

        let item_id = self.story_item_map.get(story_id).cloned();

        if let Some(item_id) = item_id {
            let status = if passes {
                ItemStatus::Done
            } else {
                ItemStatus::Blocked
            };

            match provider.update_status(&item_id, status).await {
                Ok(info) => Ok(SyncResult {
                    story_id: story_id.to_string(),
                    success: true,
                    item_id: Some(info.id),
                    item_url: info.url,
                    error: None,
                    action: SyncAction::Updated,
                }),
                Err(e) => Ok(SyncResult {
                    story_id: story_id.to_string(),
                    success: false,
                    item_id: Some(item_id),
                    item_url: None,
                    error: Some(e.to_string()),
                    action: SyncAction::Failed,
                }),
            }
        } else {
            // No item exists for this story
            Ok(SyncResult {
                story_id: story_id.to_string(),
                success: true,
                item_id: None,
                item_url: None,
                error: None,
                action: SyncAction::Skipped,
            })
        }
    }

    /// Load stories from a PRD file
    ///
    /// # Arguments
    /// * `prd_path` - Path to the PRD JSON file
    ///
    /// # Returns
    /// Vector of stories parsed from the PRD
    pub fn load_stories_from_prd(prd_path: &Path) -> TrackerResult<Vec<Story>> {
        let content = std::fs::read_to_string(prd_path)
            .map_err(|e| TrackerError::ConfigError(format!("Failed to read PRD file: {}", e)))?;

        let prd: PrdFile = serde_json::from_str(&content)
            .map_err(|e| TrackerError::ConfigError(format!("Failed to parse PRD JSON: {}", e)))?;

        Ok(prd
            .user_stories
            .into_iter()
            .map(|s| Story {
                id: s.id,
                title: s.title,
                description: s.description,
                passes: s.passes,
                priority: s.priority,
            })
            .collect())
    }

    /// Helper to create an item for a story
    async fn create_item_for_story(
        &self,
        provider: &std::sync::Arc<dyn super::traits::ProjectTracker>,
        story: &Story,
    ) -> TrackerResult<ItemInfo> {
        let request = CreateItemRequest {
            title: format!("[{}] {}", story.id, story.title),
            description: if story.description.is_empty() {
                None
            } else {
                Some(story.description.clone())
            },
            status: if self.config.sync_status {
                Some(story.status())
            } else {
                None
            },
            labels: vec![],
        };

        provider.create_item(request).await
    }
}

/// PRD file structure for parsing
#[derive(Debug, Deserialize)]
struct PrdFile {
    #[serde(rename = "userStories")]
    user_stories: Vec<PrdStory>,
}

/// Story structure in PRD file
#[derive(Debug, Deserialize)]
struct PrdStory {
    id: String,
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    passes: bool,
    #[serde(default)]
    priority: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::traits::{
        CreateItemRequest, FailureIssueRequest, ItemInfo, ProjectTracker, UpdateItemRequest,
    };
    use async_trait::async_trait;
    use std::sync::Arc;

    /// Mock provider for testing
    struct MockProvider {
        name: String,
    }

    impl MockProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl ProjectTracker for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn create_item(&self, request: CreateItemRequest) -> TrackerResult<ItemInfo> {
            Ok(ItemInfo {
                id: format!("item-{}", request.title.len()),
                title: request.title,
                url: Some("https://example.com/item".to_string()),
            })
        }

        async fn update_item(
            &self,
            item_id: &str,
            request: UpdateItemRequest,
        ) -> TrackerResult<ItemInfo> {
            Ok(ItemInfo {
                id: item_id.to_string(),
                title: request.title.unwrap_or_else(|| "Updated".to_string()),
                url: Some("https://example.com/item".to_string()),
            })
        }

        async fn create_failure_issue(
            &self,
            request: FailureIssueRequest,
        ) -> TrackerResult<ItemInfo> {
            Ok(ItemInfo {
                id: "failure-123".to_string(),
                title: format!("[Ralph Failure] {}", request.story_title),
                url: Some("https://example.com/issue".to_string()),
            })
        }

        async fn add_comment(&self, _item_id: &str, _comment: &str) -> TrackerResult<()> {
            Ok(())
        }

        async fn update_status(
            &self,
            item_id: &str,
            _status: ItemStatus,
        ) -> TrackerResult<ItemInfo> {
            Ok(ItemInfo {
                id: item_id.to_string(),
                title: "Status Updated".to_string(),
                url: Some("https://example.com/item".to_string()),
            })
        }
    }

    fn create_test_registry() -> ProviderRegistry {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider::new("test"));
        registry.register(provider);
        registry.set_active("test").unwrap();
        registry
    }

    fn create_test_stories() -> Vec<Story> {
        vec![
            Story {
                id: "US-001".to_string(),
                title: "First story".to_string(),
                description: "Description 1".to_string(),
                passes: false,
                priority: 1,
            },
            Story {
                id: "US-002".to_string(),
                title: "Second story".to_string(),
                description: "Description 2".to_string(),
                passes: true,
                priority: 2,
            },
            Story {
                id: "US-003".to_string(),
                title: "Third story".to_string(),
                description: String::new(),
                passes: false,
                priority: 3,
            },
        ]
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert!(!config.bidirectional);
        assert_eq!(config.conflict_strategy, ConflictStrategy::Manual);
        assert!(config.sync_on_completion);
        assert!(config.auto_create_items);
        assert!(config.sync_status);
    }

    #[test]
    fn test_conflict_strategy_serialize() {
        let strategy = ConflictStrategy::PrdWins;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"prd_wins\"");

        let strategy = ConflictStrategy::ExternalWins;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"external_wins\"");
    }

    #[test]
    fn test_conflict_strategy_deserialize() {
        let strategy: ConflictStrategy = serde_json::from_str("\"newest_wins\"").unwrap();
        assert_eq!(strategy, ConflictStrategy::NewestWins);
    }

    #[test]
    fn test_story_status() {
        let passing = Story {
            id: "US-001".to_string(),
            title: "Test".to_string(),
            description: String::new(),
            passes: true,
            priority: 1,
        };
        assert_eq!(passing.status(), ItemStatus::Done);

        let failing = Story {
            id: "US-002".to_string(),
            title: "Test".to_string(),
            description: String::new(),
            passes: false,
            priority: 1,
        };
        assert_eq!(failing.status(), ItemStatus::Todo);
    }

    #[test]
    fn test_sync_summary_new() {
        let summary = SyncSummary::new();
        assert_eq!(summary.total, 0);
        assert_eq!(summary.created, 0);
        assert_eq!(summary.updated, 0);
        assert_eq!(summary.skipped, 0);
        assert_eq!(summary.failed, 0);
        assert!(summary.all_successful());
    }

    #[test]
    fn test_sync_summary_add_results() {
        let mut summary = SyncSummary::new();

        summary.add_result(SyncResult {
            story_id: "US-001".to_string(),
            success: true,
            item_id: Some("item-1".to_string()),
            item_url: None,
            error: None,
            action: SyncAction::Created,
        });

        summary.add_result(SyncResult {
            story_id: "US-002".to_string(),
            success: true,
            item_id: Some("item-2".to_string()),
            item_url: None,
            error: None,
            action: SyncAction::Updated,
        });

        summary.add_result(SyncResult {
            story_id: "US-003".to_string(),
            success: false,
            item_id: None,
            item_url: None,
            error: Some("Error".to_string()),
            action: SyncAction::Failed,
        });

        assert_eq!(summary.total, 3);
        assert_eq!(summary.created, 1);
        assert_eq!(summary.updated, 1);
        assert_eq!(summary.failed, 1);
        assert!(!summary.all_successful());
    }

    #[test]
    fn test_sync_engine_new() {
        let registry = create_test_registry();
        let config = SyncConfig::default();
        let engine = SyncEngine::new(registry, config);

        assert!(engine.config().sync_on_completion);
        assert!(!engine.registry().is_empty());
    }

    #[test]
    fn test_sync_engine_with_default_config() {
        let registry = create_test_registry();
        let engine = SyncEngine::with_default_config(registry);

        assert!(engine.config().auto_create_items);
    }

    #[test]
    fn test_sync_engine_item_mapping() {
        let registry = create_test_registry();
        let mut engine = SyncEngine::with_default_config(registry);

        engine.set_item_mapping("US-001", "item-123");
        assert_eq!(engine.get_item_id("US-001"), Some("item-123"));
        assert_eq!(engine.get_item_id("US-002"), None);

        engine.clear_mappings();
        assert_eq!(engine.get_item_id("US-001"), None);
    }

    #[tokio::test]
    async fn test_initial_sync_creates_items() {
        let registry = create_test_registry();
        let mut engine = SyncEngine::with_default_config(registry);
        let stories = create_test_stories();

        let summary = engine.initial_sync(&stories).await.unwrap();

        assert_eq!(summary.total, 3);
        assert_eq!(summary.created, 3);
        assert_eq!(summary.failed, 0);
        assert!(summary.all_successful());

        // Check mappings were created
        assert!(engine.get_item_id("US-001").is_some());
        assert!(engine.get_item_id("US-002").is_some());
        assert!(engine.get_item_id("US-003").is_some());
    }

    #[tokio::test]
    async fn test_initial_sync_skips_existing() {
        let registry = create_test_registry();
        let mut engine = SyncEngine::with_default_config(registry);
        let stories = create_test_stories();

        // Pre-set a mapping
        engine.set_item_mapping("US-001", "existing-item");

        let summary = engine.initial_sync(&stories).await.unwrap();

        assert_eq!(summary.total, 3);
        assert_eq!(summary.created, 2);
        assert_eq!(summary.skipped, 1);
        assert!(summary.all_successful());
    }

    #[tokio::test]
    async fn test_initial_sync_no_auto_create() {
        let registry = create_test_registry();
        let config = SyncConfig {
            auto_create_items: false,
            ..Default::default()
        };
        let mut engine = SyncEngine::new(registry, config);
        let stories = create_test_stories();

        let summary = engine.initial_sync(&stories).await.unwrap();

        assert_eq!(summary.total, 3);
        assert_eq!(summary.skipped, 3);
        assert_eq!(summary.created, 0);
    }

    #[tokio::test]
    async fn test_sync_story_update_creates_new() {
        let registry = create_test_registry();
        let mut engine = SyncEngine::with_default_config(registry);
        let story = Story {
            id: "US-001".to_string(),
            title: "Test story".to_string(),
            description: "Description".to_string(),
            passes: false,
            priority: 1,
        };

        let result = engine.sync_story_update(&story).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, SyncAction::Created);
        assert!(result.item_id.is_some());

        // Mapping should be set
        assert!(engine.get_item_id("US-001").is_some());
    }

    #[tokio::test]
    async fn test_sync_story_update_updates_existing() {
        let registry = create_test_registry();
        let mut engine = SyncEngine::with_default_config(registry);

        // Pre-set mapping
        engine.set_item_mapping("US-001", "existing-item");

        let story = Story {
            id: "US-001".to_string(),
            title: "Updated story".to_string(),
            description: "New description".to_string(),
            passes: true,
            priority: 1,
        };

        let result = engine.sync_story_update(&story).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, SyncAction::Updated);
        assert_eq!(result.item_id, Some("existing-item".to_string()));
    }

    #[tokio::test]
    async fn test_sync_story_completion() {
        let registry = create_test_registry();
        let mut engine = SyncEngine::with_default_config(registry);

        // Pre-set mapping
        engine.set_item_mapping("US-001", "item-123");

        let result = engine.sync_story_completion("US-001", true).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, SyncAction::Updated);
    }

    #[tokio::test]
    async fn test_sync_story_completion_disabled() {
        let registry = create_test_registry();
        let config = SyncConfig {
            sync_on_completion: false,
            ..Default::default()
        };
        let mut engine = SyncEngine::new(registry, config);

        engine.set_item_mapping("US-001", "item-123");

        let result = engine.sync_story_completion("US-001", true).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, SyncAction::Skipped);
    }

    #[tokio::test]
    async fn test_sync_story_completion_no_item() {
        let registry = create_test_registry();
        let mut engine = SyncEngine::with_default_config(registry);

        // No mapping set
        let result = engine.sync_story_completion("US-001", true).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, SyncAction::Skipped);
    }

    #[tokio::test]
    async fn test_sync_no_active_provider() {
        let registry = ProviderRegistry::new(); // Empty, no active provider
        let mut engine = SyncEngine::with_default_config(registry);
        let stories = create_test_stories();

        let result = engine.initial_sync(&stories).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            TrackerError::ConfigError(msg) => {
                assert!(msg.contains("No active provider"));
            }
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_sync_action_serialize() {
        let action = SyncAction::Created;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"created\"");

        let action = SyncAction::Updated;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"updated\"");
    }
}
