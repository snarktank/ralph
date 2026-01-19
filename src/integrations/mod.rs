//! Integrations module
//!
//! This module contains project management integrations (GitHub Projects, Linear, etc.)
//! and the core traits for implementing providers.

#![allow(unused_imports)]

pub mod github;
pub mod linear;
pub mod registry;
pub mod sync_engine;
pub mod traits;
pub mod webhooks;

pub use github::{GitHubConfig, GitHubProjectsProvider};
pub use linear::{LinearConfig, LinearProvider};
pub use registry::ProviderRegistry;
pub use sync_engine::{
    ConflictStrategy, Story, SyncAction, SyncConfig, SyncEngine, SyncResult, SyncSummary,
};
pub use traits::{
    CreateItemRequest, FailureIssueRequest, ItemInfo, ItemStatus, ProjectTracker, TrackerError,
    TrackerResult, UpdateItemRequest,
};
pub use webhooks::{
    create_webhook_router, health_handler, AppState, WebhookConfig, WebhookError, WebhookResult,
};
