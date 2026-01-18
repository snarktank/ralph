//! 24-bit RGB color theme for terminal output.
//!
//! Defines the color palette used throughout Ralph's terminal UI.

#![allow(dead_code)]

use owo_colors::Rgb;

/// 24-bit RGB color theme for Ralph's terminal UI.
///
/// All colors use true 24-bit RGB values optimized for
/// modern terminals like Ghostty.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Success state color - green (34, 197, 94)
    pub success: Rgb,
    /// Error state color - red (239, 68, 68)
    pub error: Rgb,
    /// Warning state color - yellow (234, 179, 8)
    pub warning: Rgb,
    /// In-progress state color - blue (59, 130, 246)
    pub in_progress: Rgb,
    /// Muted/secondary text color - gray (107, 114, 128)
    pub muted: Rgb,
    /// Story ID highlight color - cyan (34, 211, 238)
    pub story_id: Rgb,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            success: Rgb(34, 197, 94),
            error: Rgb(239, 68, 68),
            warning: Rgb(234, 179, 8),
            in_progress: Rgb(59, 130, 246),
            muted: Rgb(107, 114, 128),
            story_id: Rgb(34, 211, 238),
        }
    }
}

impl Theme {
    /// Create a new theme with default colors.
    pub fn new() -> Self {
        Self::default()
    }
}
