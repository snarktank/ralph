//! ASCII art mascots and peek animations for Ralph's terminal UI.
//!
//! Provides fun personality through randomly selected mascots that
//! "peek" into the terminal during startup. Optimized for Ghostty
//! with synchronized output for flicker-free animation.
//!
//! Supports both static ASCII art and dynamic image-to-ANSI conversion.

#![allow(dead_code)]

use std::io::{self, Write};
use std::time::Duration;

use owo_colors::OwoColorize;

use crate::ui::colors::Theme;
use crate::ui::ghostty::TerminalCapabilities;
use crate::ui::image_to_ansi::{self, ConversionConfig};

// ============================================================================
// Mascot Definitions
// ============================================================================

/// Available mascot characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mascot {
    /// Ralph Wiggum style - simple cute face
    Wiggum,
    /// Ralph Macchio / Karate Kid style
    KarateKid,
    /// Simple peeking eyes
    PeekingEyes,
    /// Thumbs up gesture
    ThumbsUp,
}

impl Mascot {
    /// Get all available mascots.
    pub const fn all() -> &'static [Mascot] {
        &[
            Mascot::Wiggum,
            Mascot::KarateKid,
            Mascot::PeekingEyes,
            Mascot::ThumbsUp,
        ]
    }

    /// Select a random mascot based on current time.
    /// Uses simple time-based selection to avoid adding rand dependency.
    pub fn random() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);

        let all = Self::all();
        all[(nanos as usize) % all.len()]
    }

    /// Get the ASCII art for this mascot.
    pub fn art(&self) -> &'static str {
        match self {
            Mascot::Wiggum => MASCOT_WIGGUM,
            Mascot::KarateKid => MASCOT_KARATE_KID,
            Mascot::PeekingEyes => MASCOT_PEEKING_EYES,
            Mascot::ThumbsUp => MASCOT_THUMBS_UP,
        }
    }

    /// Get the peek animation frames for this mascot.
    pub fn peek_frames(&self) -> &'static [&'static str] {
        match self {
            Mascot::Wiggum => &PEEK_FRAMES_WIGGUM,
            Mascot::KarateKid => &PEEK_FRAMES_KARATE,
            Mascot::PeekingEyes => &PEEK_FRAMES_EYES,
            Mascot::ThumbsUp => &PEEK_FRAMES_THUMBS,
        }
    }

    /// Get a random quote for this mascot.
    pub fn random_quote(&self) -> &'static str {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);

        let quotes = self.quotes();
        quotes[(nanos as usize) % quotes.len()]
    }

    /// Get all quotes for this mascot.
    pub fn quotes(&self) -> &'static [&'static str] {
        match self {
            Mascot::Wiggum => &QUOTES_WIGGUM,
            Mascot::KarateKid => &QUOTES_KARATE,
            Mascot::PeekingEyes => &QUOTES_EYES,
            Mascot::ThumbsUp => &QUOTES_THUMBS,
        }
    }

    /// Get the embedded image filename for this mascot.
    pub fn image_name(&self) -> &'static str {
        match self {
            Mascot::Wiggum => "ralph_wiggum.png",
            Mascot::KarateKid => "karate_kid.png",
            _ => "ralph_wiggum.png", // Fallback for mascots without images
        }
    }

    /// Get image-based ANSI art for this mascot.
    ///
    /// Returns the mascot rendered from its embedded image, or falls back
    /// to static ASCII art if the image is not available.
    pub fn image_art(&self, config: Option<ConversionConfig>) -> String {
        image_to_ansi::load_mascot_ansi(self.image_name(), config)
            .unwrap_or_else(|| self.art().to_string())
    }
}

/// Get a random image-based mascot as ANSI art.
///
/// This will randomly select from available mascot images and convert
/// to ANSI art with the specified configuration.
pub fn random_image_mascot(config: Option<ConversionConfig>) -> Option<String> {
    image_to_ansi::random_mascot_ansi(config)
}

// ============================================================================
// ASCII Art Constants
// ============================================================================

/// Ralph Wiggum style mascot - simple cute face
pub const MASCOT_WIGGUM: &str = r#"
   ╭───────╮
   │  ◕ ◕  │
   │  ‿‿‿  │
   ╰───────╯
"#;

/// Ralph Macchio / Karate Kid style mascot
pub const MASCOT_KARATE_KID: &str = r#"
      ___
     /   \
    | • • |
    |  ▽  | 🥋
     \___/
"#;

/// Simple peeking eyes mascot
pub const MASCOT_PEEKING_EYES: &str = r#"
   ╭──╮
   │👀│
   ╰──╯
"#;

/// Thumbs up mascot
pub const MASCOT_THUMBS_UP: &str = r#"
    👍
   ╱
  ╱
"#;

// ============================================================================
// Peek Animation Frames
// ============================================================================

/// Peek frames for Wiggum mascot (appears from right)
const PEEK_FRAMES_WIGGUM: [&str; 5] = [
    "",             // Empty
    "          ╭─", // Top edge peeks
    "          │◕", // Eye peeks
    "          │‿", // Smile peeks
    "          ╰─", // Bottom edge
];

/// Peek frames for Karate Kid mascot
const PEEK_FRAMES_KARATE: [&str; 5] = [
    "",
    "           _",
    "          |•",
    "          |▽",
    "           ‾",
];

/// Peek frames for peeking eyes
const PEEK_FRAMES_EYES: [&str; 5] = ["", "         ╭─", "         │👀", "         ╰─", ""];

/// Peek frames for thumbs up
const PEEK_FRAMES_THUMBS: [&str; 5] = ["", "         ╱", "        👍", "         ╲", ""];

// ============================================================================
// Quotes
// ============================================================================

/// Quotes for Wiggum mascot
const QUOTES_WIGGUM: [&str; 5] = [
    "I'm helping!",
    "Me fail English? That's unpossible!",
    "I bent my coverage report!",
    "My code is stuck in the cat.",
    "The tests go in, the tests go out!",
];

/// Quotes for Karate Kid mascot
const QUOTES_KARATE: [&str; 5] = [
    "Wax on, wax off...",
    "Sweep the leg!",
    "No mercy!",
    "Balance, Daniel-san.",
    "First learn stand, then learn fly.",
];

/// Quotes for peeking eyes
const QUOTES_EYES: [&str; 4] = [
    "I see you...",
    "Watching your builds.",
    "Quality is watching.",
    "Always observing.",
];

/// Quotes for thumbs up
const QUOTES_THUMBS: [&str; 4] = [
    "You got this!",
    "Looking good!",
    "Ship it!",
    "Quality gates? More like quality GREATS!",
];

// ============================================================================
// Animation Engine
// ============================================================================

/// Configuration for peek animation.
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Delay between frames in milliseconds
    pub frame_delay_ms: u64,
    /// Whether animation is enabled
    pub enabled: bool,
    /// Terminal capabilities for optimization
    pub capabilities: TerminalCapabilities,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            frame_delay_ms: 80,
            enabled: true,
            capabilities: TerminalCapabilities::detect(),
        }
    }
}

impl AnimationConfig {
    /// Create config with animation disabled.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Set frame delay.
    pub fn with_frame_delay(mut self, ms: u64) -> Self {
        self.frame_delay_ms = ms;
        self
    }
}

/// Peek animation player.
pub struct PeekAnimation {
    mascot: Mascot,
    config: AnimationConfig,
    theme: Theme,
}

impl PeekAnimation {
    /// Create a new peek animation for a random mascot.
    pub fn new() -> Self {
        Self {
            mascot: Mascot::random(),
            config: AnimationConfig::default(),
            theme: Theme::default(),
        }
    }

    /// Create animation for a specific mascot.
    pub fn with_mascot(mascot: Mascot) -> Self {
        Self {
            mascot,
            config: AnimationConfig::default(),
            theme: Theme::default(),
        }
    }

    /// Set animation configuration.
    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Get the mascot.
    pub fn mascot(&self) -> Mascot {
        self.mascot
    }

    /// Play the peek animation.
    ///
    /// The mascot peeks in from the right side of a specified column,
    /// pauses, then retreats.
    pub fn play(&self, at_column: usize) -> io::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut stdout = io::stdout();
        let frames = self.mascot.peek_frames();

        // Hide cursor during animation
        write!(stdout, "\x1b[?25l")?;
        stdout.flush()?;

        // Play peek-in animation
        for frame in frames.iter().take(3) {
            self.draw_frame(&mut stdout, frame, at_column)?;
            std::thread::sleep(Duration::from_millis(self.config.frame_delay_ms));
        }

        // Pause at full peek
        std::thread::sleep(Duration::from_millis(self.config.frame_delay_ms * 3));

        // Play retreat animation
        for frame in frames.iter().rev().skip(2) {
            self.draw_frame(&mut stdout, frame, at_column)?;
            std::thread::sleep(Duration::from_millis(self.config.frame_delay_ms));
        }

        // Clear the peek area
        self.clear_peek_area(&mut stdout, at_column)?;

        // Show cursor
        write!(stdout, "\x1b[?25h")?;
        stdout.flush()?;

        Ok(())
    }

    /// Draw a single animation frame.
    fn draw_frame(&self, stdout: &mut io::Stdout, frame: &str, at_column: usize) -> io::Result<()> {
        // Use synchronized output if available for flicker-free rendering
        if self.config.capabilities.synchronized_output {
            write!(stdout, "\x1b[?2026h")?; // Begin sync
        }

        // Save cursor position
        write!(stdout, "\x1b[s")?;

        // Move to column and draw frame
        write!(
            stdout,
            "\x1b[{}G{}",
            at_column,
            frame.color(self.theme.story_id)
        )?;

        // Restore cursor position
        write!(stdout, "\x1b[u")?;

        if self.config.capabilities.synchronized_output {
            write!(stdout, "\x1b[?2026l")?; // End sync
        }

        stdout.flush()
    }

    /// Clear the peek area after animation.
    fn clear_peek_area(&self, stdout: &mut io::Stdout, at_column: usize) -> io::Result<()> {
        write!(stdout, "\x1b[s")?; // Save position
        write!(stdout, "\x1b[{}G    ", at_column)?; // Clear area
        write!(stdout, "\x1b[u")?; // Restore position
        stdout.flush()
    }
}

impl Default for PeekAnimation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Mascot Renderer
// ============================================================================

/// Renders mascots alongside other content.
#[derive(Debug)]
pub struct MascotRenderer {
    theme: Theme,
    config: AnimationConfig,
}

impl Default for MascotRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MascotRenderer {
    /// Create a new mascot renderer.
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            config: AnimationConfig::default(),
        }
    }

    /// Create with animation disabled.
    pub fn without_animation() -> Self {
        Self {
            theme: Theme::default(),
            config: AnimationConfig::disabled(),
        }
    }

    /// Set the theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set animation config.
    pub fn with_animation_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }

    /// Check if animation is enabled.
    pub fn animation_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Render a mascot with a speech bubble quote.
    pub fn render_with_quote(&self, mascot: Mascot) -> String {
        let quote = mascot.random_quote();
        let art = mascot.art();

        format!(
            "{}\n    {}\n",
            art.color(self.theme.story_id),
            format!("\"{}\"", quote).color(self.theme.muted)
        )
    }

    /// Render mascot positioned next to content.
    ///
    /// Combines the main content (left side) with the mascot (right side).
    pub fn render_beside_content(&self, content: &str, mascot: Mascot) -> String {
        let mascot_lines: Vec<&str> = mascot.art().lines().collect();
        let content_lines: Vec<&str> = content.lines().collect();

        let max_content_width = content_lines.iter().map(|l| l.len()).max().unwrap_or(0);
        let padding = max_content_width + 4;

        let max_lines = content_lines.len().max(mascot_lines.len());
        let mut output = String::new();

        for i in 0..max_lines {
            let content_line = content_lines.get(i).unwrap_or(&"");
            let mascot_line = mascot_lines.get(i).unwrap_or(&"");

            output.push_str(&format!(
                "{:<width$}{}",
                content_line,
                mascot_line.color(self.theme.story_id),
                width = padding
            ));
            output.push('\n');
        }

        output
    }

    /// Play peek animation at the right edge of content.
    pub fn play_peek_beside(&self, content_width: usize) -> io::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let animation = PeekAnimation::new()
            .with_config(self.config.clone())
            .with_theme(self.theme);

        animation.play(content_width + 4)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mascot_all() {
        let all = Mascot::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_mascot_random() {
        // Just verify it doesn't panic
        let _ = Mascot::random();
    }

    #[test]
    fn test_mascot_art() {
        let mascot = Mascot::Wiggum;
        let art = mascot.art();
        assert!(!art.is_empty());
        assert!(art.contains("◕"));
    }

    #[test]
    fn test_mascot_quotes() {
        let mascot = Mascot::Wiggum;
        let quotes = mascot.quotes();
        assert!(!quotes.is_empty());
    }

    #[test]
    fn test_mascot_random_quote() {
        let mascot = Mascot::KarateKid;
        let quote = mascot.random_quote();
        assert!(!quote.is_empty());
    }

    #[test]
    fn test_animation_config_default() {
        let config = AnimationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.frame_delay_ms, 80);
    }

    #[test]
    fn test_animation_config_disabled() {
        let config = AnimationConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_peek_animation_new() {
        let anim = PeekAnimation::new();
        // Just verify it creates without panic
        let _ = anim.mascot();
    }

    #[test]
    fn test_mascot_renderer_new() {
        let renderer = MascotRenderer::new();
        assert!(renderer.animation_enabled());
    }

    #[test]
    fn test_mascot_renderer_without_animation() {
        let renderer = MascotRenderer::without_animation();
        assert!(!renderer.animation_enabled());
    }

    #[test]
    fn test_render_with_quote() {
        let renderer = MascotRenderer::new();
        let output = renderer.render_with_quote(Mascot::Wiggum);
        assert!(output.contains("◕"));
    }

    #[test]
    fn test_render_beside_content() {
        let renderer = MascotRenderer::new();
        let content = "Hello\nWorld";
        let output = renderer.render_beside_content(content, Mascot::PeekingEyes);
        assert!(output.contains("Hello"));
        assert!(output.contains("👀"));
    }
}
