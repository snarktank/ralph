//! Kitty Graphics Protocol implementation for inline image display.
//!
//! This module implements the Kitty terminal graphics protocol, which allows
//! displaying actual images inline in the terminal. This provides much higher
//! quality than ANSI block-character approximations.
//!
//! Protocol specification: https://sw.kovidgoyal.net/kitty/graphics-protocol/
//!
//! Supported terminals:
//! - Kitty
//! - Ghostty
//! - WezTerm
//! - iTerm2 (partial)

#![allow(dead_code)]

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::{self, Write};

/// Maximum chunk size for image data transmission (4096 bytes recommended)
const CHUNK_SIZE: usize = 4096;

/// Kitty graphics protocol action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsAction {
    /// Transmit image data
    Transmit,
    /// Transmit and display
    TransmitAndDisplay,
    /// Query terminal support
    Query,
    /// Delete images
    Delete,
}

impl GraphicsAction {
    fn as_char(self) -> char {
        match self {
            Self::Transmit => 't',
            Self::TransmitAndDisplay => 'T',
            Self::Query => 'q',
            Self::Delete => 'd',
        }
    }
}

/// Image transmission format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat_ {
    /// PNG format (recommended, lossless)
    Png,
    /// RGB raw pixels (24-bit)
    Rgb,
    /// RGBA raw pixels (32-bit)
    Rgba,
}

impl ImageFormat_ {
    fn format_code(self) -> u32 {
        match self {
            Self::Png => 100,
            Self::Rgb => 24,
            Self::Rgba => 32,
        }
    }
}

/// Image placement configuration
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    /// Unique image ID (for caching/deletion)
    pub id: Option<u32>,
    /// Display width in cells (None = auto)
    pub width_cells: Option<u32>,
    /// Display height in cells (None = auto)
    pub height_cells: Option<u32>,
    /// Display width in pixels (None = auto)
    pub width_pixels: Option<u32>,
    /// Display height in pixels (None = auto)
    pub height_pixels: Option<u32>,
    /// X offset in pixels within cell
    pub x_offset: Option<u32>,
    /// Y offset in pixels within cell
    pub y_offset: Option<u32>,
    /// Z-index for layering
    pub z_index: Option<i32>,
    /// Whether to preserve aspect ratio
    pub preserve_aspect: bool,
}

impl Default for ImagePlacement {
    fn default() -> Self {
        Self {
            id: None,
            width_cells: None,
            height_cells: None,
            width_pixels: None,
            height_pixels: None,
            x_offset: None,
            y_offset: None,
            z_index: None,
            preserve_aspect: true,
        }
    }
}

impl ImagePlacement {
    /// Create a new placement with cell dimensions
    pub fn with_cells(width: u32, height: u32) -> Self {
        Self {
            width_cells: Some(width),
            height_cells: Some(height),
            ..Default::default()
        }
    }

    /// Create a new placement with pixel dimensions
    pub fn with_pixels(width: u32, height: u32) -> Self {
        Self {
            width_pixels: Some(width),
            height_pixels: Some(height),
            ..Default::default()
        }
    }

    /// Set the image ID for caching
    pub fn with_id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }
}

/// Kitty graphics protocol handler
#[derive(Debug)]
pub struct KittyGraphics {
    /// Whether Kitty graphics is supported
    supported: bool,
    /// Next available image ID
    next_id: u32,
}

impl Default for KittyGraphics {
    fn default() -> Self {
        Self::new()
    }
}

impl KittyGraphics {
    /// Create a new Kitty graphics handler
    pub fn new() -> Self {
        Self {
            supported: Self::detect_support(),
            next_id: 1,
        }
    }

    /// Detect if the terminal supports Kitty graphics protocol
    pub fn detect_support() -> bool {
        // Check for known supporting terminals
        if let Ok(term) = std::env::var("TERM") {
            let term_lower = term.to_lowercase();
            if term_lower.contains("kitty")
                || term_lower.contains("ghostty")
                || term_lower.contains("wezterm")
            {
                return true;
            }
        }

        // Check TERM_PROGRAM
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            let prog_lower = term_program.to_lowercase();
            if prog_lower.contains("kitty")
                || prog_lower.contains("ghostty")
                || prog_lower.contains("wezterm")
                || prog_lower.contains("iterm")
            {
                return true;
            }
        }

        // Check for Ghostty specifically
        if std::env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
            return true;
        }

        false
    }

    /// Check if Kitty graphics is supported
    pub fn is_supported(&self) -> bool {
        self.supported
    }

    /// Generate a unique image ID
    pub fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Display an image from a DynamicImage
    pub fn display_image(
        &mut self,
        image: &DynamicImage,
        placement: &ImagePlacement,
    ) -> io::Result<()> {
        if !self.supported {
            return Ok(());
        }

        // Encode image as PNG
        let mut png_data = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_data);
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(io::Error::other)?;

        self.display_png_data(&png_data, placement)
    }

    /// Display an image from raw PNG data
    pub fn display_png_data(&mut self, data: &[u8], placement: &ImagePlacement) -> io::Result<()> {
        if !self.supported {
            return Ok(());
        }

        let encoded = BASE64.encode(data);
        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|c| std::str::from_utf8(c).unwrap_or(""))
            .collect();

        let mut stdout = io::stdout();
        let id = placement.id.unwrap_or_else(|| self.next_id());

        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i == chunks.len() - 1;
            let is_first = i == 0;

            // Build control parameters
            let mut params = Vec::new();

            if is_first {
                params.push(format!(
                    "a={}",
                    GraphicsAction::TransmitAndDisplay.as_char()
                ));
                params.push(format!("f={}", ImageFormat_::Png.format_code()));
                params.push(format!("i={}", id));

                if let Some(w) = placement.width_cells {
                    params.push(format!("c={}", w));
                }
                if let Some(h) = placement.height_cells {
                    params.push(format!("r={}", h));
                }
                if let Some(w) = placement.width_pixels {
                    params.push(format!("w={}", w));
                }
                if let Some(h) = placement.height_pixels {
                    params.push(format!("h={}", h));
                }
                if let Some(x) = placement.x_offset {
                    params.push(format!("X={}", x));
                }
                if let Some(y) = placement.y_offset {
                    params.push(format!("Y={}", y));
                }
                if let Some(z) = placement.z_index {
                    params.push(format!("z={}", z));
                }
            }

            // m=1 means more chunks coming, m=0 means last chunk
            params.push(format!("m={}", if is_last { 0 } else { 1 }));

            let params_str = params.join(",");

            // Write the escape sequence
            // Format: ESC_G<params>;<base64_data>ESC\
            write!(stdout, "\x1b_G{};{}\x1b\\", params_str, chunk)?;
        }

        stdout.flush()
    }

    /// Display an image from embedded asset data
    pub fn display_embedded(&mut self, data: &[u8], placement: &ImagePlacement) -> io::Result<()> {
        if !self.supported {
            return Ok(());
        }

        // Try to load as image first to validate
        let img = image::load_from_memory(data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.display_image(&img, placement)
    }

    /// Delete an image by ID
    pub fn delete_image(&self, id: u32) -> io::Result<()> {
        if !self.supported {
            return Ok(());
        }

        let mut stdout = io::stdout();
        write!(stdout, "\x1b_Ga=d,d=I,i={};\x1b\\", id)?;
        stdout.flush()
    }

    /// Delete all images
    pub fn delete_all(&self) -> io::Result<()> {
        if !self.supported {
            return Ok(());
        }

        let mut stdout = io::stdout();
        write!(stdout, "\x1b_Ga=d,d=A;\x1b\\")?;
        stdout.flush()
    }

    /// Render image to a string (for use in formatted output)
    pub fn render_image_string(
        &mut self,
        image: &DynamicImage,
        placement: &ImagePlacement,
    ) -> io::Result<String> {
        if !self.supported {
            return Ok(String::new());
        }

        // Encode image as PNG
        let mut png_data = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_data);
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(io::Error::other)?;

        self.render_png_string(&png_data, placement)
    }

    /// Render PNG data to a string
    pub fn render_png_string(
        &mut self,
        data: &[u8],
        placement: &ImagePlacement,
    ) -> io::Result<String> {
        if !self.supported {
            return Ok(String::new());
        }

        let encoded = BASE64.encode(data);
        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|c| std::str::from_utf8(c).unwrap_or(""))
            .collect();

        let mut output = String::new();
        let id = placement.id.unwrap_or_else(|| self.next_id());

        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i == chunks.len() - 1;
            let is_first = i == 0;

            let mut params = Vec::new();

            if is_first {
                params.push(format!(
                    "a={}",
                    GraphicsAction::TransmitAndDisplay.as_char()
                ));
                params.push(format!("f={}", ImageFormat_::Png.format_code()));
                params.push(format!("i={}", id));

                if let Some(w) = placement.width_cells {
                    params.push(format!("c={}", w));
                }
                if let Some(h) = placement.height_cells {
                    params.push(format!("r={}", h));
                }
                if let Some(w) = placement.width_pixels {
                    params.push(format!("w={}", w));
                }
                if let Some(h) = placement.height_pixels {
                    params.push(format!("h={}", h));
                }
            }

            params.push(format!("m={}", if is_last { 0 } else { 1 }));

            let params_str = params.join(",");
            output.push_str(&format!("\x1b_G{};{}\x1b\\", params_str, chunk));
        }

        Ok(output)
    }
}

/// Load and display a mascot image using Kitty graphics
pub fn display_mascot(image_data: &[u8], width_cells: u32, height_cells: u32) -> io::Result<bool> {
    let mut graphics = KittyGraphics::new();

    if !graphics.is_supported() {
        return Ok(false);
    }

    let placement = ImagePlacement::with_cells(width_cells, height_cells);
    graphics.display_embedded(image_data, &placement)?;

    Ok(true)
}

/// Get mascot image as a string for inline display
pub fn mascot_inline_string(
    image_data: &[u8],
    width_cells: u32,
    height_cells: u32,
) -> io::Result<Option<String>> {
    let mut graphics = KittyGraphics::new();

    if !graphics.is_supported() {
        return Ok(None);
    }

    let img = image::load_from_memory(image_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let placement = ImagePlacement::with_cells(width_cells, height_cells);
    let result = graphics.render_image_string(&img, &placement)?;

    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kitty_graphics_new() {
        let graphics = KittyGraphics::new();
        // Just verify it doesn't panic
        let _ = graphics.is_supported();
    }

    #[test]
    fn test_image_placement_default() {
        let placement = ImagePlacement::default();
        assert!(placement.preserve_aspect);
        assert!(placement.id.is_none());
    }

    #[test]
    fn test_image_placement_with_cells() {
        let placement = ImagePlacement::with_cells(20, 10);
        assert_eq!(placement.width_cells, Some(20));
        assert_eq!(placement.height_cells, Some(10));
    }

    #[test]
    fn test_image_placement_with_pixels() {
        let placement = ImagePlacement::with_pixels(200, 100);
        assert_eq!(placement.width_pixels, Some(200));
        assert_eq!(placement.height_pixels, Some(100));
    }

    #[test]
    fn test_graphics_action_char() {
        assert_eq!(GraphicsAction::Transmit.as_char(), 't');
        assert_eq!(GraphicsAction::TransmitAndDisplay.as_char(), 'T');
        assert_eq!(GraphicsAction::Query.as_char(), 'q');
        assert_eq!(GraphicsAction::Delete.as_char(), 'd');
    }

    #[test]
    fn test_image_format_code() {
        assert_eq!(ImageFormat_::Png.format_code(), 100);
        assert_eq!(ImageFormat_::Rgb.format_code(), 24);
        assert_eq!(ImageFormat_::Rgba.format_code(), 32);
    }

    #[test]
    fn test_next_id() {
        let mut graphics = KittyGraphics::new();
        let id1 = graphics.next_id();
        let id2 = graphics.next_id();
        assert_eq!(id2, id1 + 1);
    }
}
