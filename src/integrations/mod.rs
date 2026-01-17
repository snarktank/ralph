//! Integrations module
//!
//! This module contains project management integrations (GitHub Projects, Linear, etc.)
//! and the core traits for implementing providers.

#![allow(unused_imports)]

pub mod traits;

pub use traits::{
    CreateItemRequest, FailureIssueRequest, ItemInfo, ItemStatus, ProjectTracker, TrackerError,
    TrackerResult, UpdateItemRequest,
};
