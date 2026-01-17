//! Provider registry for managing project management providers
//!
//! This module provides a registry for registering and managing multiple
//! project management providers (GitHub Projects, Linear, etc.) with the
//! ability to set an active provider for operations.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use super::traits::{ProjectTracker, TrackerError, TrackerResult};

/// Registry for managing multiple project management providers
///
/// The registry allows registering multiple providers by name and selecting
/// one as the active provider for operations.
pub struct ProviderRegistry {
    /// Registered providers by name
    providers: HashMap<String, Arc<dyn ProjectTracker>>,
    /// Name of the currently active provider
    active_provider: Option<String>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    /// Create a new empty provider registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            active_provider: None,
        }
    }

    /// Register a new provider
    ///
    /// If a provider with the same name already exists, it will be replaced.
    ///
    /// # Arguments
    /// * `provider` - The provider to register
    ///
    /// # Returns
    /// The name of the registered provider
    pub fn register(&mut self, provider: Arc<dyn ProjectTracker>) -> String {
        let name = provider.name().to_string();
        self.providers.insert(name.clone(), provider);
        name
    }

    /// Get a provider by name
    ///
    /// # Arguments
    /// * `name` - The name of the provider
    ///
    /// # Returns
    /// The provider if found, None otherwise
    pub fn get(&self, name: &str) -> Option<Arc<dyn ProjectTracker>> {
        self.providers.get(name).cloned()
    }

    /// Set the active provider by name
    ///
    /// # Arguments
    /// * `name` - The name of the provider to set as active
    ///
    /// # Returns
    /// Ok(()) if the provider exists, Err if not found
    pub fn set_active(&mut self, name: &str) -> TrackerResult<()> {
        if self.providers.contains_key(name) {
            self.active_provider = Some(name.to_string());
            Ok(())
        } else {
            Err(TrackerError::ConfigError(format!(
                "Provider '{}' not found in registry",
                name
            )))
        }
    }

    /// Get the currently active provider
    ///
    /// # Returns
    /// The active provider if one is set, None otherwise
    pub fn get_active(&self) -> Option<Arc<dyn ProjectTracker>> {
        self.active_provider
            .as_ref()
            .and_then(|name| self.providers.get(name).cloned())
    }

    /// Get the name of the active provider
    ///
    /// # Returns
    /// The name of the active provider if one is set
    pub fn active_name(&self) -> Option<&str> {
        self.active_provider.as_deref()
    }

    /// Clear the active provider selection
    pub fn clear_active(&mut self) {
        self.active_provider = None;
    }

    /// Check if a provider is registered
    ///
    /// # Arguments
    /// * `name` - The name to check
    ///
    /// # Returns
    /// true if a provider with this name is registered
    pub fn has(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Remove a provider from the registry
    ///
    /// If the removed provider was the active provider, the active provider
    /// will be cleared.
    ///
    /// # Arguments
    /// * `name` - The name of the provider to remove
    ///
    /// # Returns
    /// The removed provider if it existed
    pub fn remove(&mut self, name: &str) -> Option<Arc<dyn ProjectTracker>> {
        // Clear active if we're removing the active provider
        if self.active_provider.as_deref() == Some(name) {
            self.active_provider = None;
        }
        self.providers.remove(name)
    }

    /// Get all registered provider names
    ///
    /// # Returns
    /// A vector of all provider names
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of registered providers
    ///
    /// # Returns
    /// The count of registered providers
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if the registry is empty
    ///
    /// # Returns
    /// true if no providers are registered
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::traits::{
        CreateItemRequest, FailureIssueRequest, ItemInfo, ItemStatus, UpdateItemRequest,
    };
    use async_trait::async_trait;

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

        async fn create_item(&self, _request: CreateItemRequest) -> TrackerResult<ItemInfo> {
            Ok(ItemInfo {
                id: "mock-id".to_string(),
                title: "Mock Item".to_string(),
                url: None,
            })
        }

        async fn update_item(
            &self,
            item_id: &str,
            _request: UpdateItemRequest,
        ) -> TrackerResult<ItemInfo> {
            Ok(ItemInfo {
                id: item_id.to_string(),
                title: "Updated Item".to_string(),
                url: None,
            })
        }

        async fn create_failure_issue(
            &self,
            request: FailureIssueRequest,
        ) -> TrackerResult<ItemInfo> {
            Ok(ItemInfo {
                id: "failure-id".to_string(),
                title: format!("[Ralph Failure] {}", request.story_title),
                url: None,
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
                url: None,
            })
        }
    }

    #[test]
    fn test_new_registry() {
        let registry = ProviderRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.get_active().is_none());
        assert!(registry.active_name().is_none());
    }

    #[test]
    fn test_default_registry() {
        let registry = ProviderRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_register_provider() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider::new("github"));

        let name = registry.register(provider);

        assert_eq!(name, "github");
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert!(registry.has("github"));
        assert!(!registry.has("linear"));
    }

    #[test]
    fn test_register_multiple_providers() {
        let mut registry = ProviderRegistry::new();
        let github = Arc::new(MockProvider::new("github"));
        let linear = Arc::new(MockProvider::new("linear"));

        registry.register(github);
        registry.register(linear);

        assert_eq!(registry.len(), 2);
        assert!(registry.has("github"));
        assert!(registry.has("linear"));

        let names = registry.provider_names();
        assert!(names.contains(&"github"));
        assert!(names.contains(&"linear"));
    }

    #[test]
    fn test_get_provider() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider::new("github"));
        registry.register(provider);

        let retrieved = registry.get("github");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "github");

        let not_found = registry.get("linear");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_set_active_provider() {
        let mut registry = ProviderRegistry::new();
        let github = Arc::new(MockProvider::new("github"));
        let linear = Arc::new(MockProvider::new("linear"));
        registry.register(github);
        registry.register(linear);

        // Set github as active
        let result = registry.set_active("github");
        assert!(result.is_ok());
        assert_eq!(registry.active_name(), Some("github"));

        // Get active provider
        let active = registry.get_active();
        assert!(active.is_some());
        assert_eq!(active.unwrap().name(), "github");

        // Switch to linear
        let result = registry.set_active("linear");
        assert!(result.is_ok());
        assert_eq!(registry.active_name(), Some("linear"));
    }

    #[test]
    fn test_set_active_nonexistent() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider::new("github"));
        registry.register(provider);

        let result = registry.set_active("nonexistent");
        assert!(result.is_err());

        match result.unwrap_err() {
            TrackerError::ConfigError(msg) => {
                assert!(msg.contains("nonexistent"));
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_clear_active() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider::new("github"));
        registry.register(provider);
        registry.set_active("github").unwrap();

        assert!(registry.get_active().is_some());

        registry.clear_active();

        assert!(registry.get_active().is_none());
        assert!(registry.active_name().is_none());
    }

    #[test]
    fn test_remove_provider() {
        let mut registry = ProviderRegistry::new();
        let github = Arc::new(MockProvider::new("github"));
        let linear = Arc::new(MockProvider::new("linear"));
        registry.register(github);
        registry.register(linear);
        registry.set_active("github").unwrap();

        // Remove non-active provider
        let removed = registry.remove("linear");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name(), "linear");
        assert!(!registry.has("linear"));
        assert_eq!(registry.active_name(), Some("github"));

        // Remove active provider - should clear active
        let removed = registry.remove("github");
        assert!(removed.is_some());
        assert!(registry.active_name().is_none());
        assert!(registry.is_empty());

        // Remove non-existent
        let removed = registry.remove("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_replace_provider() {
        let mut registry = ProviderRegistry::new();
        let github1 = Arc::new(MockProvider::new("github"));
        let github2 = Arc::new(MockProvider::new("github"));

        registry.register(github1);
        assert_eq!(registry.len(), 1);

        // Register with same name replaces
        registry.register(github2);
        assert_eq!(registry.len(), 1);
        assert!(registry.has("github"));
    }

    #[tokio::test]
    async fn test_active_provider_operations() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider::new("github"));
        registry.register(provider);
        registry.set_active("github").unwrap();

        let active = registry.get_active().unwrap();

        // Test create_item
        let item = active
            .create_item(CreateItemRequest {
                title: "Test".to_string(),
                description: None,
                status: None,
                labels: vec![],
            })
            .await
            .unwrap();
        assert_eq!(item.id, "mock-id");

        // Test update_status
        let updated = active
            .update_status("item-1", ItemStatus::Done)
            .await
            .unwrap();
        assert_eq!(updated.id, "item-1");
    }
}
