//! Webhooks module
//!
//! This module contains the webhook server for receiving events from
//! external project management tools (GitHub, Linear, etc.).

#![allow(dead_code)]

pub mod server;

pub use server::{
    create_webhook_router, health_handler, AppState, WebhookConfig, WebhookError, WebhookResult,
};
