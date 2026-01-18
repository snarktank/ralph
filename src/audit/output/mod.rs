//! Output formatters for audit reports.
//!
//! This module provides various output formats for audit reports,
//! enabling both human-readable and machine-consumable outputs.

pub mod agent_context;
pub mod markdown;
pub mod structured;

pub use agent_context::{AgentContext, AgentContextError, AgentContextWriter};
pub use markdown::{MarkdownOutputError, MarkdownReportWriter};
pub use structured::{JsonOutputError, JsonReportWriter};
