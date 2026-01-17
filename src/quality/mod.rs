//! Quality framework module for Ralph.
//!
//! This module contains quality profiles and gate checking functionality.

pub mod blog_generator;
pub mod gates;
pub mod profiles;

// Re-exports for convenience - will be used by CLI and MCP in future stories
#[allow(unused_imports)]
pub use blog_generator::{slugify, BlogContext, BlogGenerator, BlogGeneratorError, BlogResult};
#[allow(unused_imports)]
pub use gates::{GateResult, QualityGateChecker};
#[allow(unused_imports)]
pub use profiles::{
    BlogConfig, CiConfig, DocumentationConfig, Profile, ProfileLevel, QualityConfig,
    QualityConfigError, SecurityConfig, TestingConfig,
};
