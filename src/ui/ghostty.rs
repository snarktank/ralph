//! Ghostty-specific terminal features.
//!
//! Provides advanced terminal features that leverage Ghostty's capabilities
//! including OSC 8 hyperlinks, terminal title updates, and synchronized output.

#![allow(dead_code)]

use std::io::{self, Write};

// ============================================================================
// Terminal Capability Detection
// ============================================================================

/// Terminal capability detection and feature flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCapabilities {
    /// Whether the terminal is Ghostty
    pub is_ghostty: bool,
    /// Whether 24-bit color is supported
    pub true_color: bool,
    /// Whether OSC 8 hyperlinks are supported
    pub hyperlinks: bool,
    /// Whether synchronized output is supported
    pub synchronized_output: bool,
    /// Whether title updates are supported
    pub title_updates: bool,
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self::detect()
    }
}

impl TerminalCapabilities {
    /// Detect terminal capabilities from environment.
    pub fn detect() -> Self {
        let is_ghostty = std::env::var("GHOSTTY_RESOURCES_DIR").is_ok();

        // Check for truecolor support
        let true_color = is_ghostty
            || std::env::var("COLORTERM")
                .map(|v| v == "truecolor" || v == "24bit")
                .unwrap_or(false)
            || std::env::var("TERM")
                .map(|v| v.contains("truecolor") || v.contains("256color"))
                .unwrap_or(false);

        // Ghostty and modern terminals support these features
        // For non-Ghostty terminals, we check for common modern terminals
        let is_modern_terminal = is_ghostty
            || std::env::var("TERM_PROGRAM")
                .map(|v| {
                    v == "iTerm.app"
                        || v == "WezTerm"
                        || v == "Alacritty"
                        || v == "kitty"
                        || v == "vscode"
                })
                .unwrap_or(false);

        Self {
            is_ghostty,
            true_color,
            // Hyperlinks are supported by Ghostty and most modern terminals
            hyperlinks: is_ghostty || is_modern_terminal,
            // Synchronized output is a Ghostty feature (also supported by some others)
            synchronized_output: is_ghostty,
            // Title updates are widely supported
            title_updates: true,
        }
    }

    /// Create capabilities with all features enabled (for testing).
    pub fn all_enabled() -> Self {
        Self {
            is_ghostty: true,
            true_color: true,
            hyperlinks: true,
            synchronized_output: true,
            title_updates: true,
        }
    }

    /// Create capabilities with all features disabled (minimal mode).
    pub fn minimal() -> Self {
        Self {
            is_ghostty: false,
            true_color: false,
            hyperlinks: false,
            synchronized_output: false,
            title_updates: false,
        }
    }
}

// ============================================================================
// OSC 8 Hyperlinks
// ============================================================================

/// Create an OSC 8 hyperlink.
///
/// OSC 8 format: `\x1b]8;;URL\x07TEXT\x1b]8;;\x07`
///
/// # Arguments
/// * `url` - The URL to link to (can be file://, https://, etc.)
/// * `text` - The visible text to display
/// * `capabilities` - Terminal capabilities to check for hyperlink support
///
/// # Returns
/// If hyperlinks are supported, returns the text wrapped in OSC 8 escape sequences.
/// Otherwise, returns just the text.
pub fn hyperlink(url: &str, text: &str, capabilities: &TerminalCapabilities) -> String {
    if capabilities.hyperlinks {
        format!("\x1b]8;;{}\x07{}\x1b]8;;\x07", url, text)
    } else {
        text.to_string()
    }
}

/// Create a file hyperlink from a file path.
///
/// Converts the path to a file:// URL and creates an OSC 8 hyperlink.
///
/// # Arguments
/// * `path` - The file path (can be absolute or relative)
/// * `display_text` - Optional display text. If None, uses the filename.
/// * `capabilities` - Terminal capabilities to check for hyperlink support
///
/// # Returns
/// The path wrapped in an OSC 8 hyperlink if supported, otherwise the display text.
pub fn file_hyperlink(
    path: &str,
    display_text: Option<&str>,
    capabilities: &TerminalCapabilities,
) -> String {
    let text = display_text.unwrap_or(path);

    if !capabilities.hyperlinks {
        return text.to_string();
    }

    // Convert to absolute path if possible for file:// URL
    let url = if let Ok(abs_path) = std::fs::canonicalize(path) {
        format!("file://{}", abs_path.display())
    } else {
        // Fall back to the original path
        format!("file://{}", path)
    };

    hyperlink(&url, text, capabilities)
}

/// Create a file hyperlink with line number.
///
/// Some terminals (including Ghostty) support opening files at specific lines.
///
/// # Arguments
/// * `path` - The file path
/// * `line` - The line number (1-indexed)
/// * `display_text` - Optional display text. If None, uses "path:line" format.
/// * `capabilities` - Terminal capabilities to check for hyperlink support
pub fn file_hyperlink_with_line(
    path: &str,
    line: u32,
    display_text: Option<&str>,
    capabilities: &TerminalCapabilities,
) -> String {
    let default_text = format!("{}:{}", path, line);
    let text = display_text.unwrap_or(&default_text);

    if !capabilities.hyperlinks {
        return text.to_string();
    }

    // Convert to absolute path if possible for file:// URL
    let url = if let Ok(abs_path) = std::fs::canonicalize(path) {
        format!("file://{}#{}", abs_path.display(), line)
    } else {
        format!("file://{}#{}", path, line)
    };

    hyperlink(&url, text, capabilities)
}

// ============================================================================
// Terminal Title
// ============================================================================

/// Execution status for terminal title display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleStatus {
    /// Idle, no active execution
    Idle,
    /// Running a story
    Running,
    /// Story completed successfully
    Success,
    /// Story failed
    Failed,
    /// Execution was interrupted
    Interrupted,
}

impl TitleStatus {
    /// Get the indicator character for this status.
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Running => "\u{25CF}",     // ●
            Self::Success => "\u{2713}",     // ✓
            Self::Failed => "\u{2717}",      // ✗
            Self::Interrupted => "\u{26A0}", // ⚠
        }
    }
}

/// Build a terminal title string.
///
/// Format: `Ralph | [story_id] | Iter X/Y | status_indicator`
///
/// # Arguments
/// * `story_id` - Optional current story ID
/// * `iteration` - Optional tuple of (current, total) iterations
/// * `status` - The current execution status
pub fn build_title(
    story_id: Option<&str>,
    iteration: Option<(u64, u64)>,
    status: TitleStatus,
) -> String {
    let mut parts = vec!["Ralph".to_string()];

    if let Some(id) = story_id {
        parts.push(format!("[{}]", id));
    }

    if let Some((current, total)) = iteration {
        parts.push(format!("Iter {}/{}", current, total));
    }

    let indicator = status.indicator();
    if !indicator.is_empty() {
        parts.push(indicator.to_string());
    }

    parts.join(" | ")
}

/// Set the terminal title using OSC escape sequences.
///
/// OSC 0 format: `\x1b]0;TITLE\x07`
///
/// # Arguments
/// * `title` - The title to set
/// * `capabilities` - Terminal capabilities to check for title support
pub fn set_title(title: &str, capabilities: &TerminalCapabilities) -> io::Result<()> {
    if !capabilities.title_updates {
        return Ok(());
    }

    let sequence = format!("\x1b]0;{}\x07", title);
    let mut stdout = io::stdout();
    stdout.write_all(sequence.as_bytes())?;
    stdout.flush()
}

/// Update the terminal title with Ralph's current state.
///
/// # Arguments
/// * `story_id` - Optional current story ID
/// * `iteration` - Optional tuple of (current, total) iterations
/// * `status` - The current execution status
/// * `capabilities` - Terminal capabilities to check for title support
pub fn update_title(
    story_id: Option<&str>,
    iteration: Option<(u64, u64)>,
    status: TitleStatus,
    capabilities: &TerminalCapabilities,
) -> io::Result<()> {
    let title = build_title(story_id, iteration, status);
    set_title(&title, capabilities)
}

/// Reset the terminal title to the default.
pub fn reset_title(capabilities: &TerminalCapabilities) -> io::Result<()> {
    set_title("Ralph", capabilities)
}

// ============================================================================
// Synchronized Output
// ============================================================================

/// Begin synchronized output mode.
///
/// This tells the terminal to buffer output until `end_sync()` is called,
/// preventing flicker during complex updates.
///
/// DCS sequence: `\x1b[?2026h` (begin synchronized update)
///
/// # Arguments
/// * `capabilities` - Terminal capabilities to check for sync support
pub fn begin_sync(capabilities: &TerminalCapabilities) -> io::Result<()> {
    if !capabilities.synchronized_output {
        return Ok(());
    }

    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b[?2026h")?;
    stdout.flush()
}

/// End synchronized output mode.
///
/// This tells the terminal to display the buffered output.
///
/// DCS sequence: `\x1b[?2026l` (end synchronized update)
///
/// # Arguments
/// * `capabilities` - Terminal capabilities to check for sync support
pub fn end_sync(capabilities: &TerminalCapabilities) -> io::Result<()> {
    if !capabilities.synchronized_output {
        return Ok(());
    }

    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b[?2026l")?;
    stdout.flush()
}

/// Execute a closure with synchronized output.
///
/// Wraps the closure in begin_sync/end_sync calls for flicker-free output.
///
/// # Arguments
/// * `capabilities` - Terminal capabilities to check for sync support
/// * `f` - The closure to execute with synchronized output
///
/// # Returns
/// The result of the closure, or an io::Error if syncing fails.
pub fn with_sync<F, T>(capabilities: &TerminalCapabilities, f: F) -> io::Result<T>
where
    F: FnOnce() -> T,
{
    begin_sync(capabilities)?;
    let result = f();
    end_sync(capabilities)?;
    Ok(result)
}

/// RAII guard for synchronized output.
///
/// Automatically calls begin_sync on creation and end_sync on drop.
pub struct SyncGuard<'a> {
    capabilities: &'a TerminalCapabilities,
    active: bool,
}

impl<'a> SyncGuard<'a> {
    /// Create a new sync guard.
    ///
    /// Immediately begins synchronized output if supported.
    pub fn new(capabilities: &'a TerminalCapabilities) -> io::Result<Self> {
        begin_sync(capabilities)?;
        Ok(Self {
            capabilities,
            active: capabilities.synchronized_output,
        })
    }
}

impl Drop for SyncGuard<'_> {
    fn drop(&mut self) {
        if self.active {
            // Best effort to end sync mode
            let _ = end_sync(self.capabilities);
        }
    }
}

// ============================================================================
// GhosttyFeatures - Main Interface
// ============================================================================

/// Main interface for Ghostty terminal features.
///
/// Provides a unified API for all Ghostty-specific features with
/// automatic capability detection and graceful fallbacks.
#[derive(Debug, Clone)]
pub struct GhosttyFeatures {
    /// Detected terminal capabilities
    capabilities: TerminalCapabilities,
}

impl Default for GhosttyFeatures {
    fn default() -> Self {
        Self::new()
    }
}

impl GhosttyFeatures {
    /// Create a new GhosttyFeatures instance with auto-detected capabilities.
    pub fn new() -> Self {
        Self {
            capabilities: TerminalCapabilities::detect(),
        }
    }

    /// Create with specific capabilities (useful for testing).
    pub fn with_capabilities(capabilities: TerminalCapabilities) -> Self {
        Self { capabilities }
    }

    /// Get the detected terminal capabilities.
    pub fn capabilities(&self) -> &TerminalCapabilities {
        &self.capabilities
    }

    /// Check if running in Ghostty terminal.
    pub fn is_ghostty(&self) -> bool {
        self.capabilities.is_ghostty
    }

    // -------------------------------------------------------------------------
    // Hyperlinks
    // -------------------------------------------------------------------------

    /// Create a hyperlink.
    pub fn hyperlink(&self, url: &str, text: &str) -> String {
        hyperlink(url, text, &self.capabilities)
    }

    /// Create a file hyperlink.
    pub fn file_hyperlink(&self, path: &str, display_text: Option<&str>) -> String {
        file_hyperlink(path, display_text, &self.capabilities)
    }

    /// Create a file hyperlink with line number.
    pub fn file_hyperlink_with_line(
        &self,
        path: &str,
        line: u32,
        display_text: Option<&str>,
    ) -> String {
        file_hyperlink_with_line(path, line, display_text, &self.capabilities)
    }

    // -------------------------------------------------------------------------
    // Terminal Title
    // -------------------------------------------------------------------------

    /// Set the terminal title.
    pub fn set_title(&self, title: &str) -> io::Result<()> {
        set_title(title, &self.capabilities)
    }

    /// Update the terminal title with Ralph's state.
    pub fn update_title(
        &self,
        story_id: Option<&str>,
        iteration: Option<(u64, u64)>,
        status: TitleStatus,
    ) -> io::Result<()> {
        update_title(story_id, iteration, status, &self.capabilities)
    }

    /// Reset the terminal title.
    pub fn reset_title(&self) -> io::Result<()> {
        reset_title(&self.capabilities)
    }

    // -------------------------------------------------------------------------
    // Synchronized Output
    // -------------------------------------------------------------------------

    /// Begin synchronized output mode.
    pub fn begin_sync(&self) -> io::Result<()> {
        begin_sync(&self.capabilities)
    }

    /// End synchronized output mode.
    pub fn end_sync(&self) -> io::Result<()> {
        end_sync(&self.capabilities)
    }

    /// Execute a closure with synchronized output.
    pub fn with_sync<F, T>(&self, f: F) -> io::Result<T>
    where
        F: FnOnce() -> T,
    {
        with_sync(&self.capabilities, f)
    }

    /// Create a RAII guard for synchronized output.
    pub fn sync_guard(&self) -> io::Result<SyncGuard<'_>> {
        SyncGuard::new(&self.capabilities)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_capabilities_detect() {
        // Just verify it doesn't panic
        let caps = TerminalCapabilities::detect();
        // Title updates should always be true
        assert!(caps.title_updates);
    }

    #[test]
    fn test_terminal_capabilities_all_enabled() {
        let caps = TerminalCapabilities::all_enabled();
        assert!(caps.is_ghostty);
        assert!(caps.true_color);
        assert!(caps.hyperlinks);
        assert!(caps.synchronized_output);
        assert!(caps.title_updates);
    }

    #[test]
    fn test_terminal_capabilities_minimal() {
        let caps = TerminalCapabilities::minimal();
        assert!(!caps.is_ghostty);
        assert!(!caps.true_color);
        assert!(!caps.hyperlinks);
        assert!(!caps.synchronized_output);
        assert!(!caps.title_updates);
    }

    #[test]
    fn test_hyperlink_with_support() {
        let caps = TerminalCapabilities::all_enabled();
        let result = hyperlink("https://example.com", "Example", &caps);
        assert_eq!(result, "\x1b]8;;https://example.com\x07Example\x1b]8;;\x07");
    }

    #[test]
    fn test_hyperlink_without_support() {
        let caps = TerminalCapabilities::minimal();
        let result = hyperlink("https://example.com", "Example", &caps);
        assert_eq!(result, "Example");
    }

    #[test]
    fn test_file_hyperlink_without_support() {
        let caps = TerminalCapabilities::minimal();
        let result = file_hyperlink("/path/to/file.rs", None, &caps);
        assert_eq!(result, "/path/to/file.rs");
    }

    #[test]
    fn test_file_hyperlink_with_display_text() {
        let caps = TerminalCapabilities::minimal();
        let result = file_hyperlink("/path/to/file.rs", Some("file.rs"), &caps);
        assert_eq!(result, "file.rs");
    }

    #[test]
    fn test_file_hyperlink_with_line_without_support() {
        let caps = TerminalCapabilities::minimal();
        let result = file_hyperlink_with_line("/path/to/file.rs", 42, None, &caps);
        assert_eq!(result, "/path/to/file.rs:42");
    }

    #[test]
    fn test_title_status_indicators() {
        assert_eq!(TitleStatus::Idle.indicator(), "");
        assert_eq!(TitleStatus::Running.indicator(), "\u{25CF}");
        assert_eq!(TitleStatus::Success.indicator(), "\u{2713}");
        assert_eq!(TitleStatus::Failed.indicator(), "\u{2717}");
        assert_eq!(TitleStatus::Interrupted.indicator(), "\u{26A0}");
    }

    #[test]
    fn test_build_title_basic() {
        let title = build_title(None, None, TitleStatus::Idle);
        assert_eq!(title, "Ralph");
    }

    #[test]
    fn test_build_title_with_story() {
        let title = build_title(Some("US-001"), None, TitleStatus::Running);
        assert_eq!(title, "Ralph | [US-001] | \u{25CF}");
    }

    #[test]
    fn test_build_title_full() {
        let title = build_title(Some("US-001"), Some((3, 10)), TitleStatus::Running);
        assert_eq!(title, "Ralph | [US-001] | Iter 3/10 | \u{25CF}");
    }

    #[test]
    fn test_build_title_success() {
        let title = build_title(Some("US-001"), Some((10, 10)), TitleStatus::Success);
        assert_eq!(title, "Ralph | [US-001] | Iter 10/10 | \u{2713}");
    }

    #[test]
    fn test_ghostty_features_new() {
        // Just verify it doesn't panic
        let features = GhosttyFeatures::new();
        let _ = features.capabilities();
    }

    #[test]
    fn test_ghostty_features_with_capabilities() {
        let caps = TerminalCapabilities::all_enabled();
        let features = GhosttyFeatures::with_capabilities(caps);
        assert!(features.is_ghostty());
    }

    #[test]
    fn test_ghostty_features_hyperlink() {
        let features = GhosttyFeatures::with_capabilities(TerminalCapabilities::all_enabled());
        let result = features.hyperlink("https://example.com", "Test");
        assert!(result.contains("\x1b]8;;"));
    }

    #[test]
    fn test_ghostty_features_fallback() {
        let features = GhosttyFeatures::with_capabilities(TerminalCapabilities::minimal());
        let result = features.hyperlink("https://example.com", "Test");
        assert_eq!(result, "Test");
    }
}
