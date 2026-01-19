//! Iteration management module for Ralph.
//!
//! This module provides infrastructure for managing iteration loops during story execution,
//! including context passing between iterations and futile retry detection.

pub mod context;
pub mod futility;

// Re-exports for convenience
pub use context::{ApproachHint, IterationContext, IterationError};
pub use futility::{FutileRetryDetector, FutilityVerdict};
