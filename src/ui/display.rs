//! Main display controller for Ralph's terminal UI.
//!
//! Coordinates all UI components and manages terminal output.

#![allow(dead_code)]

use crate::ui::colors::Theme;

/// Main display controller for Ralph's terminal output.
///
/// Coordinates rendering of story panels, progress indicators,
/// quality gates, and other UI components.
#[derive(Debug)]
pub struct RalphDisplay {
    /// Color theme for terminal output
    theme: Theme,
    /// Whether colors are enabled (respects NO_COLOR env var)
    colors_enabled: bool,
    /// Whether the terminal supports advanced features
    advanced_features: bool,
}

impl Default for RalphDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl RalphDisplay {
    /// Create a new RalphDisplay with default settings.
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            colors_enabled: Self::detect_color_support(),
            advanced_features: Self::detect_advanced_features(),
        }
    }

    /// Create a RalphDisplay with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            theme,
            colors_enabled: Self::detect_color_support(),
            advanced_features: Self::detect_advanced_features(),
        }
    }

    /// Get the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Check if colors are enabled.
    pub fn colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    /// Enable or disable colors.
    pub fn set_colors_enabled(&mut self, enabled: bool) {
        self.colors_enabled = enabled;
    }

    /// Check if advanced terminal features are available.
    pub fn advanced_features(&self) -> bool {
        self.advanced_features
    }

    /// Detect if color output should be enabled.
    ///
    /// Respects the NO_COLOR environment variable.
    fn detect_color_support() -> bool {
        std::env::var("NO_COLOR").is_err()
    }

    /// Detect if advanced terminal features are available.
    ///
    /// Checks for Ghostty or other modern terminal emulators.
    fn detect_advanced_features() -> bool {
        // Check for Ghostty
        if std::env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
            return true;
        }

        // Check for other modern terminals that support 24-bit color
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") || term.contains("truecolor") {
                return true;
            }
        }

        // Check COLORTERM for truecolor support
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return true;
            }
        }

        false
    }
}
