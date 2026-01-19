//! Timeout configuration and management module.
//!
//! This module provides types and functionality for configuring timeouts
//! for agent execution, including agent-level and iteration-level limits,
//! as well as heartbeat monitoring.

pub mod heartbeat;

use std::time::Duration;

// Re-export heartbeat types for convenient access
pub use heartbeat::{HeartbeatEvent, HeartbeatMonitor};

/// Configuration for timeout behavior during agent execution.
///
/// This struct holds all timeout-related settings including agent execution
/// limits, iteration limits, and heartbeat monitoring parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeoutConfig {
    /// Maximum time allowed for overall agent execution.
    /// Default: 600 seconds (10 minutes)
    pub agent_timeout: Duration,

    /// Maximum time allowed for a single iteration.
    /// Default: 900 seconds (15 minutes)
    pub iteration_timeout: Duration,

    /// Interval between heartbeat checks.
    /// Default: 30 seconds
    pub heartbeat_interval: Duration,

    /// Number of missed heartbeats before considering execution stalled.
    /// Default: 3
    pub missed_heartbeats_threshold: u32,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            agent_timeout: Duration::from_secs(600),
            iteration_timeout: Duration::from_secs(900),
            heartbeat_interval: Duration::from_secs(30),
            missed_heartbeats_threshold: 3,
        }
    }
}

impl TimeoutConfig {
    /// Creates a new TimeoutConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a TimeoutConfig with custom values.
    pub fn with_values(
        agent_timeout: Duration,
        iteration_timeout: Duration,
        heartbeat_interval: Duration,
        missed_heartbeats_threshold: u32,
    ) -> Self {
        Self {
            agent_timeout,
            iteration_timeout,
            heartbeat_interval,
            missed_heartbeats_threshold,
        }
    }

    /// Sets the agent timeout duration.
    pub fn with_agent_timeout(mut self, timeout: Duration) -> Self {
        self.agent_timeout = timeout;
        self
    }

    /// Sets the iteration timeout duration.
    pub fn with_iteration_timeout(mut self, timeout: Duration) -> Self {
        self.iteration_timeout = timeout;
        self
    }

    /// Sets the heartbeat interval.
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Sets the missed heartbeats threshold.
    pub fn with_missed_heartbeats_threshold(mut self, threshold: u32) -> Self {
        self.missed_heartbeats_threshold = threshold;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_agent_timeout() {
        let config = TimeoutConfig::default();
        assert_eq!(config.agent_timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_default_iteration_timeout() {
        let config = TimeoutConfig::default();
        assert_eq!(config.iteration_timeout, Duration::from_secs(900));
    }

    #[test]
    fn test_default_heartbeat_interval() {
        let config = TimeoutConfig::default();
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
    }

    #[test]
    fn test_default_missed_heartbeats_threshold() {
        let config = TimeoutConfig::default();
        assert_eq!(config.missed_heartbeats_threshold, 3);
    }

    #[test]
    fn test_new_returns_default() {
        let config = TimeoutConfig::new();
        let default_config = TimeoutConfig::default();
        assert_eq!(config, default_config);
    }

    #[test]
    fn test_with_values() {
        let config = TimeoutConfig::with_values(
            Duration::from_secs(120),
            Duration::from_secs(300),
            Duration::from_secs(10),
            5,
        );

        assert_eq!(config.agent_timeout, Duration::from_secs(120));
        assert_eq!(config.iteration_timeout, Duration::from_secs(300));
        assert_eq!(config.heartbeat_interval, Duration::from_secs(10));
        assert_eq!(config.missed_heartbeats_threshold, 5);
    }

    #[test]
    fn test_builder_pattern() {
        let config = TimeoutConfig::new()
            .with_agent_timeout(Duration::from_secs(300))
            .with_iteration_timeout(Duration::from_secs(600))
            .with_heartbeat_interval(Duration::from_secs(15))
            .with_missed_heartbeats_threshold(5);

        assert_eq!(config.agent_timeout, Duration::from_secs(300));
        assert_eq!(config.iteration_timeout, Duration::from_secs(600));
        assert_eq!(config.heartbeat_interval, Duration::from_secs(15));
        assert_eq!(config.missed_heartbeats_threshold, 5);
    }

    #[test]
    fn test_config_equality() {
        let config1 = TimeoutConfig::default();
        let config2 = TimeoutConfig::default();
        assert_eq!(config1, config2);
    }

    #[test]
    fn test_config_clone() {
        let config1 = TimeoutConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1, config2);
    }
}
