//! Error handling and classification module
//!
//! This module provides infrastructure for classifying errors into actionable
//! recovery strategies. It supports categorizing errors as transient, usage-limited,
//! fatal, or timeout-related, with specific reasons and recovery hints.

pub mod classification;
pub mod detector;

// Re-export main types for convenient access
pub use classification::{
    ClassifiedError, ErrorCategory, FatalReason, RecoveryHint, TimeoutReason, TransientReason,
    UsageLimitReason,
};
pub use detector::{ErrorDetector, ErrorPattern};
