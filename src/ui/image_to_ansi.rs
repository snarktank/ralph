//! Image-to-ANSI art converter inspired by ansizalizer.
//!
//! Converts images to colored ASCII/ANSI text art for terminal display.
//! Uses pixel sampling, color analysis, and character brightness mapping.

use image::{DynamicImage, GenericImageView, Rgba};
use std::fmt::Write as FmtWrite;

// ============================================================================
// Character Sets
// ============================================================================

/// ASCII characters sorted by visual density (dark to light)
const ASCII_CHARS: &str =
    " `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";

/// Shorter gradient for simpler output
const ASCII_SIMPLE: &str = " .:-=+*#%@";

/// Block characters for higher resolution output
const BLOCK_CHARS: &[char] = &[' ', '░', '▒', '▓', '█'];

// ============================================================================
// Configuration
// ============================================================================

/// Character rendering mode
#[derive(Debug, Clone, Copy, Default)]
pub enum CharacterMode {
    /// Full ASCII gradient (94 characters)
    #[default]
    Ascii,
    /// Simple ASCII gradient (10 characters)
    AsciiSimple,
    /// Unicode block characters
    Blocks,
}

/// Color rendering mode
#[derive(Debug, Clone, Copy, Default)]
pub enum ColorMode {
    /// Full 24-bit true color
    #[default]
    TrueColor,
    /// 256 color palette
    Color256,
    /// No color (brightness only)
    Grayscale,
}

/// Configuration for image-to-ANSI conversion
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// Target width in characters
    pub width: u32,
    /// Target height in characters (auto-calculated if None)
    pub height: Option<u32>,
    /// Character aspect ratio compensation (terminal chars are taller than wide)
    pub char_ratio: f32,
    /// Character mode for output
    pub char_mode: CharacterMode,
    /// Color mode for output
    pub color_mode: ColorMode,
    /// Use background colors (two-color mode)
    pub use_background: bool,
    /// Invert brightness mapping
    pub invert: bool,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            width: 40,
            height: None,
            char_ratio: 0.5, // Terminal chars are ~2x taller than wide
            char_mode: CharacterMode::Ascii,
            color_mode: ColorMode::TrueColor,
            use_background: false,
            invert: false,
        }
    }
}

impl ConversionConfig {
    /// Create config for small mascot display
    pub fn mascot() -> Self {
        Self {
            width: 20,
            height: Some(12),
            char_ratio: 0.5,
            char_mode: CharacterMode::Ascii,
            color_mode: ColorMode::TrueColor,
            use_background: true,
            invert: false,
        }
    }

    /// Create config for larger display
    pub fn large() -> Self {
        Self {
            width: 80,
            height: None,
            char_ratio: 0.5,
            char_mode: CharacterMode::Ascii,
            color_mode: ColorMode::TrueColor,
            use_background: true,
            invert: false,
        }
    }
}

// ============================================================================
// Color Utilities
// ============================================================================

/// RGB color with floating point components
#[derive(Debug, Clone, Copy)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    fn from_rgba(rgba: Rgba<u8>) -> Self {
        Self {
            r: rgba[0] as f32 / 255.0,
            g: rgba[1] as f32 / 255.0,
            b: rgba[2] as f32 / 255.0,
        }
    }

    fn to_rgb8(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.0).clamp(0.0, 255.0) as u8,
            (self.g * 255.0).clamp(0.0, 255.0) as u8,
            (self.b * 255.0).clamp(0.0, 255.0) as u8,
        )
    }

    /// Calculate perceived brightness using luminance formula
    fn brightness(&self) -> f32 {
        // ITU-R BT.709 luminance coefficients
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Calculate HSL lightness
    fn lightness(&self) -> f32 {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        (max + min) / 2.0
    }

    /// Average multiple colors
    fn average(colors: &[Color]) -> Color {
        if colors.is_empty() {
            return Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            };
        }
        let n = colors.len() as f32;
        Color {
            r: colors.iter().map(|c| c.r).sum::<f32>() / n,
            g: colors.iter().map(|c| c.g).sum::<f32>() / n,
            b: colors.iter().map(|c| c.b).sum::<f32>() / n,
        }
    }

    /// Find lightest and darkest colors
    fn light_dark(colors: &[Color]) -> (Color, Color) {
        let mut light = colors[0];
        let mut dark = colors[0];
        let mut max_l = light.lightness();
        let mut min_l = max_l;

        for c in colors.iter().skip(1) {
            let l = c.lightness();
            if l > max_l {
                max_l = l;
                light = *c;
            }
            if l < min_l {
                min_l = l;
                dark = *c;
            }
        }
        (light, dark)
    }

    /// Color distance in RGB space
    fn distance(&self, other: &Color) -> f32 {
        let dr = self.r - other.r;
        let dg = self.g - other.g;
        let db = self.b - other.b;
        (dr * dr + dg * dg + db * db).sqrt()
    }
}

// ============================================================================
// ANSI Code Generation
// ============================================================================

/// Generate ANSI escape code for 24-bit foreground color
fn ansi_fg_24bit(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

/// Generate ANSI escape code for 24-bit background color
fn ansi_bg_24bit(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[48;2;{};{};{}m", r, g, b)
}

/// Generate ANSI escape code for 256-color foreground
fn ansi_fg_256(color: u8) -> String {
    format!("\x1b[38;5;{}m", color)
}

/// Generate ANSI escape code for 256-color background
fn ansi_bg_256(color: u8) -> String {
    format!("\x1b[48;5;{}m", color)
}

/// ANSI reset code
const ANSI_RESET: &str = "\x1b[0m";

/// Convert RGB to closest 256-color palette index
fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    // Check grayscale ramp first (232-255)
    if r == g && g == b {
        if r < 8 {
            return 16; // black
        }
        if r > 248 {
            return 231; // white
        }
        return ((r as u16 - 8) / 10 + 232) as u8;
    }

    // 6x6x6 color cube (16-231)
    let r_idx = (r as u16 * 6 / 256) as u8;
    let g_idx = (g as u16 * 6 / 256) as u8;
    let b_idx = (b as u16 * 6 / 256) as u8;

    16 + 36 * r_idx + 6 * g_idx + b_idx
}

// ============================================================================
// Image Converter
// ============================================================================

/// Converts images to ANSI text art
pub struct ImageConverter {
    config: ConversionConfig,
}

impl ImageConverter {
    /// Create a new converter with the given configuration
    pub fn new(config: ConversionConfig) -> Self {
        Self { config }
    }

    /// Create a converter with default mascot settings
    pub fn mascot() -> Self {
        Self::new(ConversionConfig::mascot())
    }

    /// Convert an image to ANSI art string
    pub fn convert(&self, img: &DynamicImage) -> String {
        // Calculate target dimensions
        let (orig_width, orig_height) = img.dimensions();
        let aspect = orig_width as f32 / orig_height as f32;

        let target_width = self.config.width;
        let target_height = self
            .config
            .height
            .unwrap_or_else(|| (target_width as f32 / aspect / self.config.char_ratio) as u32);

        // Resize image to 2x target for pixel grouping
        let resized = img.resize_exact(
            target_width * 2,
            target_height * 2,
            image::imageops::FilterType::Lanczos3,
        );

        let mut output = String::new();

        // Process 2x2 pixel groups
        for y in 0..target_height {
            for x in 0..target_width {
                let px = x * 2;
                let py = y * 2;

                // Get 4 pixels in the 2x2 block
                let colors = [
                    Color::from_rgba(resized.get_pixel(px, py)),
                    Color::from_rgba(resized.get_pixel(px + 1, py)),
                    Color::from_rgba(resized.get_pixel(px, py + 1)),
                    Color::from_rgba(resized.get_pixel(px + 1, py + 1)),
                ];

                let (ch, fg, bg) = self.process_pixel_group(&colors);

                // Generate ANSI codes
                match self.config.color_mode {
                    ColorMode::TrueColor => {
                        let (r, g, b) = fg.to_rgb8();
                        let _ = write!(output, "{}", ansi_fg_24bit(r, g, b));
                        if self.config.use_background {
                            let (br, bg_color, bb) = bg.to_rgb8();
                            let _ = write!(output, "{}", ansi_bg_24bit(br, bg_color, bb));
                        }
                    }
                    ColorMode::Color256 => {
                        let (r, g, b) = fg.to_rgb8();
                        let _ = write!(output, "{}", ansi_fg_256(rgb_to_256(r, g, b)));
                        if self.config.use_background {
                            let (br, bg_color, bb) = bg.to_rgb8();
                            let _ = write!(output, "{}", ansi_bg_256(rgb_to_256(br, bg_color, bb)));
                        }
                    }
                    ColorMode::Grayscale => {
                        // No color codes
                    }
                }

                output.push(ch);
            }

            // Reset at end of line and add newline
            if matches!(
                self.config.color_mode,
                ColorMode::TrueColor | ColorMode::Color256
            ) {
                output.push_str(ANSI_RESET);
            }
            output.push('\n');
        }

        output
    }

    /// Process a 2x2 pixel group and return character + colors
    fn process_pixel_group(&self, colors: &[Color; 4]) -> (char, Color, Color) {
        let avg = Color::average(colors);
        let (light, dark) = Color::light_dark(colors);

        // Calculate brightness for character selection
        let brightness = if self.config.use_background {
            // Two-color mode: brightness is distance from dark to average
            let total_dist = light.distance(&dark);
            if total_dist < 0.01 {
                0.5 // Uniform color
            } else {
                avg.distance(&dark) / total_dist
            }
        } else {
            // One-color mode: absolute brightness
            avg.brightness()
        };

        let brightness = if self.config.invert {
            1.0 - brightness
        } else {
            brightness
        };

        let ch = self.brightness_to_char(brightness);
        let fg = if self.config.use_background {
            light
        } else {
            avg
        };
        let bg = dark;

        (ch, fg, bg)
    }

    /// Map brightness value to character
    fn brightness_to_char(&self, brightness: f32) -> char {
        let chars: Vec<char> = match self.config.char_mode {
            CharacterMode::Ascii => ASCII_CHARS.chars().collect(),
            CharacterMode::AsciiSimple => ASCII_SIMPLE.chars().collect(),
            CharacterMode::Blocks => BLOCK_CHARS.to_vec(),
        };

        let idx = ((brightness * (chars.len() - 1) as f32).round() as usize).min(chars.len() - 1);
        chars[idx]
    }

    /// Convert image bytes to ANSI art
    pub fn convert_bytes(&self, bytes: &[u8]) -> Result<String, image::ImageError> {
        let img = image::load_from_memory(bytes)?;
        Ok(self.convert(&img))
    }
}

// ============================================================================
// Embedded Ralph Images
// ============================================================================

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/mascots/"]
#[include = "*.png"]
#[include = "*.jpg"]
#[include = "*.jpeg"]
pub struct MascotAssets;

/// Get list of available mascot image names
pub fn list_mascot_images() -> Vec<String> {
    <MascotAssets as Embed>::iter()
        .map(|s| s.to_string())
        .collect()
}

/// Load a random mascot image and convert to ANSI art
pub fn random_mascot_ansi(config: Option<ConversionConfig>) -> Option<String> {
    let images: Vec<_> = <MascotAssets as Embed>::iter().collect();
    if images.is_empty() {
        return None;
    }

    // Time-based random selection
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);

    let idx = (nanos as usize) % images.len();
    let name = &images[idx];

    <MascotAssets as Embed>::get(name).and_then(|file| {
        let converter = ImageConverter::new(config.unwrap_or_else(ConversionConfig::mascot));
        converter.convert_bytes(&file.data).ok()
    })
}

/// Load a specific mascot image by name
pub fn load_mascot_ansi(name: &str, config: Option<ConversionConfig>) -> Option<String> {
    <MascotAssets as Embed>::get(name).and_then(|file| {
        let converter = ImageConverter::new(config.unwrap_or_else(ConversionConfig::mascot));
        converter.convert_bytes(&file.data).ok()
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_brightness() {
        let white = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        };
        let black = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let gray = Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
        };

        assert!((white.brightness() - 1.0).abs() < 0.01);
        assert!(black.brightness().abs() < 0.01);
        assert!((gray.brightness() - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_color_average() {
        let colors = [
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
            },
        ];
        let avg = Color::average(&colors);
        assert!((avg.r - 0.5).abs() < 0.01);
        assert!((avg.g - 0.5).abs() < 0.01);
        assert!(avg.b.abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_256() {
        assert_eq!(rgb_to_256(0, 0, 0), 16); // black
        assert_eq!(rgb_to_256(255, 255, 255), 231); // white
        assert_eq!(rgb_to_256(128, 128, 128), 244); // gray
    }

    #[test]
    fn test_converter_creation() {
        let converter = ImageConverter::mascot();
        assert_eq!(converter.config.width, 20);
    }

    #[test]
    fn test_brightness_to_char() {
        let converter = ImageConverter::new(ConversionConfig {
            char_mode: CharacterMode::AsciiSimple,
            ..Default::default()
        });

        let dark = converter.brightness_to_char(0.0);
        let light = converter.brightness_to_char(1.0);

        assert_eq!(dark, ' ');
        assert_eq!(light, '@');
    }

    #[test]
    fn test_list_mascot_images() {
        // This will be empty until we add images
        let _ = list_mascot_images();
    }
}
