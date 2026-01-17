//! Webhooks module
//!
//! This module contains the webhook server for receiving events from
//! external project management tools (GitHub, Linear, etc.).

#![allow(dead_code)]

pub mod github;
pub mod linear;
pub mod server;

pub use github::GitHubWebhookHandler;
pub use linear::LinearWebhookHandler;
pub use server::{
    create_webhook_router, health_handler, AppState, WebhookConfig, WebhookError, WebhookResult,
};
