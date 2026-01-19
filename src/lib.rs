//! Ralph - Enterprise-ready autonomous AI agent framework
//!
//! This library exposes Ralph's internal modules for integration testing
//! and potential use as a library.

pub mod audit;
pub mod integrations;
pub mod logging;
pub mod mcp;
pub mod parallel;
pub mod quality;
pub mod runner;
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
