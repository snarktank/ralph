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
    /// Active/important text color - orange (255, 165, 0)
    pub active: Rgb,
    /// Completed/done text color - dim gray for strikethrough (128, 128, 128)
    pub completed: Rgb,
    /// Primary text color - white (255, 255, 255)
    pub primary: Rgb,
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
            active: Rgb(255, 165, 0), // Orange for active/important items
            completed: Rgb(128, 128, 128), // Dim gray for completed items
            primary: Rgb(255, 255, 255), // White for primary text
        }
    }
}

impl Theme {
    /// Create a new theme with default colors.
    pub fn new() -> Self {
        Self::default()
    }
}

/// ANSI escape codes for text styling.
pub mod ansi {
    /// Start strikethrough text
    pub const STRIKETHROUGH_START: &str = "\x1b[9m";
    /// End strikethrough text
    pub const STRIKETHROUGH_END: &str = "\x1b[29m";
    /// Start blinking text
    pub const BLINK_START: &str = "\x1b[5m";
    /// End blinking text
    pub const BLINK_END: &str = "\x1b[25m";
    /// Start dim text
    pub const DIM_START: &str = "\x1b[2m";
    /// End dim text
    pub const DIM_END: &str = "\x1b[22m";
    /// Reset all attributes
    pub const RESET: &str = "\x1b[0m";
}

/// Text styling utilities for terminal output.
#[derive(Debug, Clone)]
pub struct StyledText {
    text: String,
    color: Option<Rgb>,
    strikethrough: bool,
    blink: bool,
    dim: bool,
}

impl StyledText {
    /// Create a new styled text with the given content.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            strikethrough: false,
            blink: false,
            dim: false,
        }
    }

    /// Set the text color.
    pub fn color(mut self, color: Rgb) -> Self {
        self.color = Some(color);
        self
    }

    /// Enable strikethrough styling.
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Enable blinking text.
    pub fn blink(mut self) -> Self {
        self.blink = true;
        self
    }

    /// Enable dim text.
    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Render the styled text as a string with ANSI codes.
    pub fn render(&self) -> String {
        use owo_colors::OwoColorize;

        let mut result = String::new();

        // Apply ANSI styles first
        if self.blink {
            result.push_str(ansi::BLINK_START);
        }
        if self.dim {
            result.push_str(ansi::DIM_START);
        }
        if self.strikethrough {
            result.push_str(ansi::STRIKETHROUGH_START);
        }

        // Apply color if set
        if let Some(rgb) = self.color {
            result.push_str(&format!("{}", self.text.color(rgb)));
        } else {
            result.push_str(&self.text);
        }

        // Reset styles
        if self.strikethrough {
            result.push_str(ansi::STRIKETHROUGH_END);
        }
        if self.dim {
            result.push_str(ansi::DIM_END);
        }
        if self.blink {
            result.push_str(ansi::BLINK_END);
        }

        result
    }
}

impl std::fmt::Display for StyledText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}

/// Helper functions for creating styled text.
pub fn active_text(text: impl Into<String>) -> StyledText {
    StyledText::new(text).color(Theme::default().active)
}

pub fn completed_text(text: impl Into<String>) -> StyledText {
    StyledText::new(text)
        .color(Theme::default().completed)
        .strikethrough()
        .dim()
}

pub fn muted_text(text: impl Into<String>) -> StyledText {
    StyledText::new(text).color(Theme::default().muted)
}

pub fn primary_text(text: impl Into<String>) -> StyledText {
    StyledText::new(text).color(Theme::default().primary)
}

pub fn blinking_text(text: impl Into<String>) -> StyledText {
    StyledText::new(text).blink()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_text_basic() {
        let styled = StyledText::new("Hello");
        assert_eq!(styled.text, "Hello");
    }

    #[test]
    fn test_styled_text_with_color() {
        let theme = Theme::default();
        let styled = StyledText::new("Active").color(theme.active);
        let rendered = styled.render();
        assert!(rendered.contains("Active"));
    }

    #[test]
    fn test_styled_text_strikethrough() {
        let styled = StyledText::new("Done").strikethrough();
        let rendered = styled.render();
        assert!(rendered.contains(ansi::STRIKETHROUGH_START));
        assert!(rendered.contains(ansi::STRIKETHROUGH_END));
    }

    #[test]
    fn test_completed_text_helper() {
        let styled = completed_text("Finished task");
        let rendered = styled.render();
        assert!(rendered.contains(ansi::STRIKETHROUGH_START));
        assert!(rendered.contains(ansi::DIM_START));
    }

    #[test]
    fn test_active_text_helper() {
        let styled = active_text("Important");
        let rendered = styled.render();
        // Should contain the orange color
        assert!(rendered.contains("Important"));
    }
}
