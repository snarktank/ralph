//! Progress spinners and iteration bars for Ralph's terminal UI.
//!
//! Provides animated progress indicators using the indicatif crate,
//! with custom styling that matches Ralph's color theme.

#![allow(dead_code)]

use std::sync::Arc;
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

use crate::ui::colors::{ansi, Theme};

/// Custom spinner character sequences.
pub mod spinner_chars {
    /// Braille spinner pattern - smooth animation
    pub const BRAILLE: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    /// Dots spinner pattern - simple animation
    pub const DOTS: &[&str] = &["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"];

    /// Arc spinner pattern - circular motion
    pub const ARC: &[&str] = &["◜", "◠", "◝", "◞", "◡", "◟"];

    /// Circle spinner pattern - filling circle
    pub const CIRCLE: &[&str] = &["◐", "◓", "◑", "◒"];

    /// Block spinner pattern - block rotation
    pub const BLOCK: &[&str] = &["▖", "▘", "▝", "▗"];

    /// Pulse spinner pattern - pulsing animation
    pub const PULSE: &[&str] = &["●", "◉", "○", "◉"];

    /// Arrow spinner pattern - rotating arrow
    pub const ARROW: &[&str] = &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"];

    /// Clock spinner pattern - clock hands
    pub const CLOCK: &[&str] = &[
        "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛",
    ];
}

/// Blinking indicator styles.
pub mod blink_chars {
    /// Simple blink - alternating visibility
    pub const SIMPLE: &[&str] = &["●", " "];

    /// Pulse blink - size variation
    pub const PULSE: &[&str] = &["●", "◉", "○", "◉"];

    /// Signal blink - radio signal style
    pub const SIGNAL: &[&str] = &["◯", "◔", "◑", "◕", "●"];

    /// Heartbeat blink
    pub const HEARTBEAT: &[&str] = &["♡", "♥", "♥", "♡"];
}

/// Progress bar characters for iteration visualization.
pub mod progress_chars {
    /// Filled block character
    pub const FILLED: &str = "█";
    /// Partially filled block
    pub const PARTIAL: &str = "▓";
    /// Empty block character
    pub const EMPTY: &str = "░";
    /// Progress bar left cap
    pub const LEFT_CAP: &str = "▐";
    /// Progress bar right cap
    pub const RIGHT_CAP: &str = "▌";
}

/// Spinner style configuration.
#[derive(Debug, Clone)]
pub struct SpinnerStyle {
    /// Spinner character sequence
    pub chars: Vec<String>,
    /// Tick duration in milliseconds
    pub tick_ms: u64,
}

impl Default for SpinnerStyle {
    fn default() -> Self {
        Self {
            chars: spinner_chars::BRAILLE
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tick_ms: 80,
        }
    }
}

impl SpinnerStyle {
    /// Create a new spinner style with braille characters.
    pub fn braille() -> Self {
        Self::default()
    }

    /// Create a spinner style with dot characters.
    pub fn dots() -> Self {
        Self {
            chars: spinner_chars::DOTS.iter().map(|s| s.to_string()).collect(),
            tick_ms: 80,
        }
    }

    /// Create a spinner style with arc characters.
    pub fn arc() -> Self {
        Self {
            chars: spinner_chars::ARC.iter().map(|s| s.to_string()).collect(),
            tick_ms: 100,
        }
    }

    /// Create a spinner style with circle characters.
    pub fn circle() -> Self {
        Self {
            chars: spinner_chars::CIRCLE
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tick_ms: 120,
        }
    }

    /// Create a spinner style with block characters.
    pub fn block() -> Self {
        Self {
            chars: spinner_chars::BLOCK.iter().map(|s| s.to_string()).collect(),
            tick_ms: 100,
        }
    }
}

/// A managed spinner instance.
#[derive(Debug, Clone)]
pub struct RalphSpinner {
    /// The underlying progress bar
    progress_bar: ProgressBar,
    /// The current action message
    message: String,
    /// Color theme
    theme: Theme,
}

impl RalphSpinner {
    /// Create a new spinner with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        let theme = Theme::default();

        let pb = ProgressBar::new_spinner();
        pb.set_style(Self::create_style(&theme));
        pb.set_message(message.clone());
        pb.enable_steady_tick(Duration::from_millis(80));

        Self {
            progress_bar: pb,
            message,
            theme,
        }
    }

    /// Create a spinner with a custom theme.
    pub fn with_theme(message: impl Into<String>, theme: Theme) -> Self {
        let message = message.into();

        let pb = ProgressBar::new_spinner();
        pb.set_style(Self::create_style(&theme));
        pb.set_message(message.clone());
        pb.enable_steady_tick(Duration::from_millis(80));

        Self {
            progress_bar: pb,
            message,
            theme,
        }
    }

    /// Create a spinner attached to a MultiProgress.
    pub fn with_multi(mp: &MultiProgress, message: impl Into<String>) -> Self {
        let message = message.into();
        let theme = Theme::default();

        let pb = mp.add(ProgressBar::new_spinner());
        pb.set_style(Self::create_style(&theme));
        pb.set_message(message.clone());
        pb.enable_steady_tick(Duration::from_millis(80));

        Self {
            progress_bar: pb,
            message,
            theme,
        }
    }

    /// Create the progress style with theme colors.
    fn create_style(theme: &Theme) -> ProgressStyle {
        let spinner_chars = spinner_chars::BRAILLE.join("");
        let in_progress_rgb = theme.in_progress;

        // Format: spinner [elapsed] message
        ProgressStyle::with_template(&format!(
            "{{spinner:.color({},{},{})}} [{{elapsed_precise}}] {{msg}}",
            in_progress_rgb.0, in_progress_rgb.1, in_progress_rgb.2
        ))
        .unwrap_or_else(|_| ProgressStyle::default_spinner())
        .tick_strings(&[
            &spinner_chars,
            "✓", // Final state
        ])
    }

    /// Update the spinner message.
    pub fn set_message(&self, message: impl Into<String>) {
        self.progress_bar.set_message(message.into());
    }

    /// Get the current message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the underlying progress bar.
    pub fn progress_bar(&self) -> &ProgressBar {
        &self.progress_bar
    }

    /// Mark the spinner as finished with a success message.
    pub fn finish_with_success(&self, message: impl Into<String>) {
        let msg = message.into();
        let styled_msg = format!("{} {}", "✓".color(self.theme.success), msg);
        self.progress_bar.finish_with_message(styled_msg);
    }

    /// Mark the spinner as finished with an error message.
    pub fn finish_with_error(&self, message: impl Into<String>) {
        let msg = message.into();
        let styled_msg = format!("{} {}", "✗".color(self.theme.error), msg);
        self.progress_bar.finish_with_message(styled_msg);
    }

    /// Mark the spinner as finished and clear it.
    pub fn finish_and_clear(&self) {
        self.progress_bar.finish_and_clear();
    }

    /// Stop the spinner without clearing.
    pub fn finish(&self) {
        self.progress_bar.finish();
    }
}

/// Iteration progress bar showing X/Y progress with percentage.
#[derive(Debug, Clone)]
pub struct IterationProgress {
    /// The underlying progress bar
    progress_bar: ProgressBar,
    /// Total iterations
    total: u64,
    /// Current iteration
    current: u64,
    /// Color theme
    theme: Theme,
    /// Bar width in characters
    width: usize,
}

impl IterationProgress {
    /// Create a new iteration progress bar.
    pub fn new(total: u64) -> Self {
        let theme = Theme::default();
        let width = 30;

        let pb = ProgressBar::new(total);
        pb.set_style(Self::create_style(&theme, width));

        Self {
            progress_bar: pb,
            total,
            current: 0,
            theme,
            width,
        }
    }

    /// Create an iteration progress bar with a custom theme.
    pub fn with_theme(total: u64, theme: Theme) -> Self {
        let width = 30;

        let pb = ProgressBar::new(total);
        pb.set_style(Self::create_style(&theme, width));

        Self {
            progress_bar: pb,
            total,
            current: 0,
            theme,
            width,
        }
    }

    /// Create an iteration progress bar attached to a MultiProgress.
    pub fn with_multi(mp: &MultiProgress, total: u64) -> Self {
        let theme = Theme::default();
        let width = 30;

        let pb = mp.add(ProgressBar::new(total));
        pb.set_style(Self::create_style(&theme, width));

        Self {
            progress_bar: pb,
            total,
            current: 0,
            theme,
            width,
        }
    }

    /// Set the bar width in characters.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self.progress_bar
            .set_style(Self::create_style(&self.theme, width));
        self
    }

    /// Create the progress style with block characters.
    fn create_style(theme: &Theme, width: usize) -> ProgressStyle {
        let in_progress_rgb = theme.in_progress;
        let muted_rgb = theme.muted;

        // Use filled/empty block characters for progress visualization
        // Format: Iteration X/Y [████░░░░░░] XX%
        ProgressStyle::with_template(&format!(
            "Iteration {{pos}}/{{len}} [{{bar:{width}.color({},{},{}).color({},{},{})}}] {{percent:>3}}%",
            in_progress_rgb.0, in_progress_rgb.1, in_progress_rgb.2,
            muted_rgb.0, muted_rgb.1, muted_rgb.2,
            width = width
        ))
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars(&format!(
            "{}{}{}",
            progress_chars::FILLED,
            progress_chars::PARTIAL,
            progress_chars::EMPTY
        ))
    }

    /// Increment the progress by one.
    pub fn inc(&mut self) {
        self.current = (self.current + 1).min(self.total);
        self.progress_bar.inc(1);
    }

    /// Set the current progress position.
    pub fn set_position(&mut self, pos: u64) {
        self.current = pos.min(self.total);
        self.progress_bar.set_position(pos);
    }

    /// Get the current iteration number.
    pub fn current(&self) -> u64 {
        self.current
    }

    /// Get the total number of iterations.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Get the progress percentage.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }

    /// Get the underlying progress bar.
    pub fn progress_bar(&self) -> &ProgressBar {
        &self.progress_bar
    }

    /// Mark the progress as finished.
    pub fn finish(&self) {
        self.progress_bar.finish();
    }

    /// Mark the progress as finished with a message.
    pub fn finish_with_message(&self, message: impl Into<String>) {
        self.progress_bar.finish_with_message(message.into());
    }

    /// Mark the progress as finished and clear it.
    pub fn finish_and_clear(&self) {
        self.progress_bar.finish_and_clear();
    }

    /// Render progress bar as a string (for non-interactive use).
    pub fn render(&self) -> String {
        let filled_count = if self.total == 0 {
            self.width
        } else {
            ((self.current as f64 / self.total as f64) * self.width as f64) as usize
        };
        let empty_count = self.width.saturating_sub(filled_count);

        let filled_str = progress_chars::FILLED.repeat(filled_count);
        let empty_str = progress_chars::EMPTY.repeat(empty_count);

        let filled = filled_str.color(self.theme.in_progress);
        let empty = empty_str.color(self.theme.muted);

        format!(
            "Iteration {}/{} [{}{}] {:>3}%",
            self.current,
            self.total,
            filled,
            empty,
            self.percentage() as u32
        )
    }
}

/// Manager for multiple parallel progress indicators.
#[derive(Debug, Clone)]
pub struct ProgressManager {
    /// The indicatif MultiProgress instance
    multi_progress: Arc<MultiProgress>,
    /// Color theme
    theme: Theme,
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressManager {
    /// Create a new progress manager.
    pub fn new() -> Self {
        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            theme: Theme::default(),
        }
    }

    /// Create a progress manager with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            theme,
        }
    }

    /// Get the underlying MultiProgress instance.
    pub fn multi_progress(&self) -> &MultiProgress {
        &self.multi_progress
    }

    /// Create a new spinner attached to this manager.
    pub fn add_spinner(&self, message: impl Into<String>) -> RalphSpinner {
        RalphSpinner::with_multi(&self.multi_progress, message)
    }

    /// Create a new iteration progress bar attached to this manager.
    pub fn add_iteration_progress(&self, total: u64) -> IterationProgress {
        IterationProgress::with_multi(&self.multi_progress, total)
    }

    /// Add a custom progress bar to this manager.
    pub fn add(&self, pb: ProgressBar) -> ProgressBar {
        self.multi_progress.add(pb)
    }

    /// Remove a progress bar from this manager.
    pub fn remove(&self, pb: &ProgressBar) {
        self.multi_progress.remove(pb);
    }

    /// Clear all progress bars.
    pub fn clear(&self) -> std::io::Result<()> {
        self.multi_progress.clear()
    }

    /// Create a blinking indicator attached to this manager.
    pub fn add_blinking_indicator(&self, label: impl Into<String>) -> BlinkingIndicator {
        BlinkingIndicator::with_multi(&self.multi_progress, label)
    }
}

/// A blinking progress indicator for showing active processes.
///
/// Provides a visual pulsing/blinking animation to indicate
/// that something is actively running.
#[derive(Debug, Clone)]
pub struct BlinkingIndicator {
    /// The underlying progress bar
    progress_bar: ProgressBar,
    /// The label text
    label: String,
    /// Color theme
    theme: Theme,
    /// Blink style
    blink_style: BlinkStyle,
}

/// Style of blinking animation.
#[derive(Debug, Clone, Copy, Default)]
pub enum BlinkStyle {
    /// Simple on/off blink
    #[default]
    Simple,
    /// Pulsing size change
    Pulse,
    /// Radio signal style
    Signal,
    /// Heartbeat style
    Heartbeat,
}

impl BlinkingIndicator {
    /// Create a new blinking indicator with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let theme = Theme::default();

        let pb = ProgressBar::new_spinner();
        pb.set_style(Self::create_style(&theme, BlinkStyle::default()));
        pb.set_message(label.clone());
        pb.enable_steady_tick(Duration::from_millis(250)); // Slower tick for blink effect

        Self {
            progress_bar: pb,
            label,
            theme,
            blink_style: BlinkStyle::default(),
        }
    }

    /// Create a blinking indicator with a custom theme.
    pub fn with_theme(label: impl Into<String>, theme: Theme) -> Self {
        let label = label.into();

        let pb = ProgressBar::new_spinner();
        pb.set_style(Self::create_style(&theme, BlinkStyle::default()));
        pb.set_message(label.clone());
        pb.enable_steady_tick(Duration::from_millis(250));

        Self {
            progress_bar: pb,
            label,
            theme,
            blink_style: BlinkStyle::default(),
        }
    }

    /// Create a blinking indicator attached to a MultiProgress.
    pub fn with_multi(mp: &MultiProgress, label: impl Into<String>) -> Self {
        let label = label.into();
        let theme = Theme::default();

        let pb = mp.add(ProgressBar::new_spinner());
        pb.set_style(Self::create_style(&theme, BlinkStyle::default()));
        pb.set_message(label.clone());
        pb.enable_steady_tick(Duration::from_millis(250));

        Self {
            progress_bar: pb,
            label,
            theme,
            blink_style: BlinkStyle::default(),
        }
    }

    /// Set the blink style.
    pub fn with_blink_style(mut self, style: BlinkStyle) -> Self {
        self.blink_style = style;
        self.progress_bar
            .set_style(Self::create_style(&self.theme, style));
        self
    }

    /// Create the progress style for blinking.
    fn create_style(theme: &Theme, style: BlinkStyle) -> ProgressStyle {
        let chars = match style {
            BlinkStyle::Simple => blink_chars::SIMPLE,
            BlinkStyle::Pulse => blink_chars::PULSE,
            BlinkStyle::Signal => blink_chars::SIGNAL,
            BlinkStyle::Heartbeat => blink_chars::HEARTBEAT,
        };
        let spinner_chars = chars.join("");
        let active_rgb = theme.active;

        // Format: blinking indicator followed by message
        ProgressStyle::with_template(&format!(
            "{{spinner:.color({},{},{})}} {{msg}}",
            active_rgb.0, active_rgb.1, active_rgb.2
        ))
        .unwrap_or_else(|_| ProgressStyle::default_spinner())
        .tick_strings(&[&spinner_chars, "●"])
    }

    /// Update the label.
    pub fn set_label(&self, label: impl Into<String>) {
        self.progress_bar.set_message(label.into());
    }

    /// Get the current label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Finish and clear the indicator.
    pub fn finish_and_clear(&self) {
        self.progress_bar.finish_and_clear();
    }

    /// Finish the indicator without clearing.
    pub fn finish(&self) {
        self.progress_bar.finish();
    }

    /// Finish with a success message.
    pub fn finish_with_success(&self, message: impl Into<String>) {
        let msg = message.into();
        let styled_msg = format!("{} {}", "✓".color(self.theme.success), msg);
        self.progress_bar.finish_with_message(styled_msg);
    }

    /// Finish with an error message.
    pub fn finish_with_error(&self, message: impl Into<String>) {
        let msg = message.into();
        let styled_msg = format!("{} {}", "✗".color(self.theme.error), msg);
        self.progress_bar.finish_with_message(styled_msg);
    }
}

/// A live status indicator that shows real-time progress with blinking.
///
/// Combines a blinking indicator with status text that can be updated.
#[derive(Debug)]
pub struct LiveStatusIndicator {
    /// Primary status text
    status: String,
    /// Secondary detail text
    detail: Option<String>,
    /// Progress counter (current/total)
    progress: Option<(u64, u64)>,
    /// Whether actively processing
    active: bool,
    /// Color theme
    theme: Theme,
    /// Frame counter for animation
    frame: usize,
}

impl LiveStatusIndicator {
    /// Create a new live status indicator.
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            detail: None,
            progress: None,
            active: true,
            theme: Theme::default(),
            frame: 0,
        }
    }

    /// Set the detail text.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Set the progress counter.
    pub fn with_progress(mut self, current: u64, total: u64) -> Self {
        self.progress = Some((current, total));
        self
    }

    /// Set whether the indicator is active.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Update the status text.
    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    /// Update the detail text.
    pub fn set_detail(&mut self, detail: impl Into<String>) {
        self.detail = Some(detail.into());
    }

    /// Clear the detail text.
    pub fn clear_detail(&mut self) {
        self.detail = None;
    }

    /// Update the progress.
    pub fn set_progress(&mut self, current: u64, total: u64) {
        self.progress = Some((current, total));
    }

    /// Advance the animation frame.
    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % 4;
    }

    /// Render the indicator as a string.
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Blinking indicator
        if self.active {
            let blink_chars = blink_chars::PULSE;
            let blink = blink_chars[self.frame % blink_chars.len()];
            output.push_str(&format!("{} ", blink.color(self.theme.active)));
        } else {
            output.push_str(&format!("{} ", "○".color(self.theme.muted)));
        }

        // Status text (orange when active, white otherwise)
        if self.active {
            output.push_str(&format!("{}", self.status.color(self.theme.active)));
        } else {
            output.push_str(&format!("{}", self.status.color(self.theme.primary)));
        }

        // Progress if present
        if let Some((current, total)) = self.progress {
            output.push_str(&format!(
                " [{}/{}]",
                current.to_string().color(self.theme.primary),
                total.to_string().color(self.theme.muted)
            ));
        }

        // Detail if present
        if let Some(ref detail) = self.detail {
            output.push_str(&format!(" - {}", detail.color(self.theme.muted)));
        }

        output
    }

    /// Render with ANSI blinking enabled.
    pub fn render_with_blink(&self) -> String {
        if self.active {
            format!("{}{}{}", ansi::BLINK_START, self.render(), ansi::BLINK_END)
        } else {
            self.render()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_style_default() {
        let style = SpinnerStyle::default();
        assert_eq!(style.chars.len(), 10);
        assert_eq!(style.tick_ms, 80);
    }

    #[test]
    fn test_spinner_style_variants() {
        let braille = SpinnerStyle::braille();
        assert_eq!(braille.chars.len(), 10);

        let dots = SpinnerStyle::dots();
        assert_eq!(dots.chars.len(), 10);

        let arc = SpinnerStyle::arc();
        assert_eq!(arc.chars.len(), 6);

        let circle = SpinnerStyle::circle();
        assert_eq!(circle.chars.len(), 4);

        let block = SpinnerStyle::block();
        assert_eq!(block.chars.len(), 4);
    }

    #[test]
    fn test_iteration_progress_percentage() {
        let mut progress = IterationProgress::new(10);
        assert_eq!(progress.percentage(), 0.0);

        progress.set_position(5);
        assert_eq!(progress.percentage(), 50.0);

        progress.set_position(10);
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_iteration_progress_zero_total() {
        let progress = IterationProgress::new(0);
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_iteration_progress_render() {
        let progress = IterationProgress::new(10);
        let output = progress.render();
        assert!(output.contains("Iteration 0/10"));
        assert!(output.contains("0%"));
    }

    #[test]
    fn test_progress_manager_creation() {
        let manager = ProgressManager::new();
        // Just verify it creates without panic
        assert!(manager.multi_progress().is_hidden() || !manager.multi_progress().is_hidden());
    }
}
