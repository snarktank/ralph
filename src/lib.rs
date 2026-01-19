//! Ralph - Enterprise-ready autonomous AI agent framework
//!
//! This library exposes Ralph's internal modules for integration testing
//! and potential use as a library.

pub mod audit;
pub mod checkpoint;
pub mod error;
pub mod integrations;
pub mod interactive_guidance;
pub mod iteration;
pub mod logging;
pub mod mcp;
pub mod metrics;
pub mod notification;
pub mod parallel;
pub mod pause;
pub mod quality;
pub mod runner;
pub mod timeout;
pub mod ui;

/// Returns a greeting message.
pub fn hello_world() -> &'static str {
    "Hello, World!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello, World!");
    }
}
