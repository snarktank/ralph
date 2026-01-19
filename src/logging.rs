//! Logging middleware for debugging and diagnostics.
//!
//! This module provides a configurable logging system that:
//! - Writes to stderr (to not interfere with stdout-based protocols like MCP)
//! - Supports configurable log levels via `RUST_LOG` or programmatic configuration
//! - Includes timestamps in all log entries

use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

/// Log level configuration for the logging middleware.
#[derive(Debug, Clone, Copy, Default)]
pub enum LogLevel {
    /// Trace level - most verbose
    Trace,
    /// Debug level
    Debug,
    /// Info level (default)
    #[default]
    Info,
    /// Warning level
    Warn,
    /// Error level - least verbose
    Error,
    /// Disable logging entirely
    Off,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
            LogLevel::Off => Level::ERROR, // Will be filtered out by directive
        }
    }
}

impl From<u8> for LogLevel {
    /// Convert verbosity count to log level.
    /// 0 = Info, 1 = Debug, 2+ = Trace
    fn from(verbosity: u8) -> Self {
        match verbosity {
            0 => LogLevel::Info,
            1 => LogLevel::Debug,
            _ => LogLevel::Trace,
        }
    }
}

/// Configuration for the logging middleware.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// The log level to use
    pub level: LogLevel,
    /// Whether to include timestamps
    pub with_timestamps: bool,
    /// Whether to include the target (module path)
    pub with_target: bool,
    /// Whether to include thread IDs
    pub with_thread_ids: bool,
    /// Whether to include file/line information
    pub with_file: bool,
    /// Whether to include line numbers
    pub with_line_number: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            with_timestamps: true,
            with_target: true,
            with_thread_ids: false,
            with_file: false,
            with_line_number: false,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the log level.
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Set whether to include timestamps.
    pub fn with_timestamps(mut self, enabled: bool) -> Self {
        self.with_timestamps = enabled;
        self
    }

    /// Set whether to include the target (module path).
    pub fn with_target(mut self, enabled: bool) -> Self {
        self.with_target = enabled;
        self
    }

    /// Set whether to include thread IDs.
    pub fn with_thread_ids(mut self, enabled: bool) -> Self {
        self.with_thread_ids = enabled;
        self
    }

    /// Set whether to include file information.
    pub fn with_file(mut self, enabled: bool) -> Self {
        self.with_file = enabled;
        self
    }

    /// Set whether to include line numbers.
    pub fn with_line_number(mut self, enabled: bool) -> Self {
        self.with_line_number = enabled;
        self
    }

    /// Create a configuration from verbosity level (0 = info, 1 = debug, 2+ = trace).
    pub fn from_verbosity(verbosity: u8) -> Self {
        Self::default().with_level(LogLevel::from(verbosity))
    }
}

/// Initialize the logging middleware with the given configuration.
///
/// This function should be called once at the start of the application.
/// Logs are written to stderr to avoid interfering with stdout-based
/// protocols like MCP.
///
/// # Examples
///
/// ```no_run
/// use ralphmacchio::logging::{init_logging, LoggingConfig, LogLevel};
///
/// // Initialize with default settings (info level, timestamps enabled)
/// init_logging(LoggingConfig::default());
///
/// // Or with custom configuration
/// init_logging(
///     LoggingConfig::new()
///         .with_level(LogLevel::Debug)
///         .with_timestamps(true)
/// );
/// ```
pub fn init_logging(config: LoggingConfig) {
    // Check for RUST_LOG environment variable first
    let env_filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        let level_str = match config.level {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Off => "off",
        };
        EnvFilter::new(level_str)
    };

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .with_target(config.with_target)
        .with_thread_ids(config.with_thread_ids)
        .with_file(config.with_file)
        .with_line_number(config.with_line_number);

    if config.with_timestamps {
        subscriber.init();
    } else {
        subscriber.without_time().init();
    }
}

/// Initialize logging with default configuration.
///
/// This is a convenience function that initializes logging with:
/// - Info log level (unless RUST_LOG is set)
/// - Timestamps enabled
/// - Target (module path) enabled
/// - Output to stderr
pub fn init_default_logging() {
    init_logging(LoggingConfig::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_verbosity() {
        assert!(matches!(LogLevel::from(0), LogLevel::Info));
        assert!(matches!(LogLevel::from(1), LogLevel::Debug));
        assert!(matches!(LogLevel::from(2), LogLevel::Trace));
        assert!(matches!(LogLevel::from(10), LogLevel::Trace));
    }

    #[test]
    fn test_logging_config_builder() {
        let config = LoggingConfig::new()
            .with_level(LogLevel::Debug)
            .with_timestamps(false)
            .with_target(false)
            .with_thread_ids(true);

        assert!(matches!(config.level, LogLevel::Debug));
        assert!(!config.with_timestamps);
        assert!(!config.with_target);
        assert!(config.with_thread_ids);
    }

    #[test]
    fn test_logging_config_from_verbosity() {
        let config = LoggingConfig::from_verbosity(2);
        assert!(matches!(config.level, LogLevel::Trace));
        assert!(config.with_timestamps);
    }
}
