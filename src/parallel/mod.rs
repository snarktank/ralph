//! Parallel execution module
//!
//! This module provides infrastructure for parallel story execution,
//! including dependency analysis, scheduling, conflict detection, and reconciliation.

pub mod conflict;
pub mod dependency;
pub mod inference;
pub mod reconcile;
pub mod scheduler;

// Re-export UI events for external use
pub use crate::ui::parallel_events::{ParallelUIEvent, StoryDisplayInfo, StoryStatus};
