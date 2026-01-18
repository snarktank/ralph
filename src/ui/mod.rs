//! Terminal UI module for Ralph.
//!
//! Provides rich terminal output with 24-bit color support,
//! progress indicators, and interactive displays.

#![allow(unused_imports)]

mod colors;
mod display;
mod ghostty;
mod help;
mod image_to_ansi;
mod interrupt;
mod iteration_view;
mod mascot;
mod quality_gates;
mod spinner;
mod story_view;
mod summary;

pub use colors::Theme;
pub use display::{DisplayOptions, RalphDisplay, UiMode};
pub use ghostty::{
    file_hyperlink, file_hyperlink_with_line, hyperlink, GhosttyFeatures, SyncGuard,
    TerminalCapabilities, TitleStatus,
};
pub use help::{BuildInfo, CommandInfo, HelpRenderer, COMMANDS, GLOBAL_OPTIONS, RALPH_BANNER};
pub use image_to_ansi::{
    list_mascot_images, load_mascot_ansi, random_mascot_ansi, CharacterMode, ColorMode,
    ConversionConfig, ImageConverter,
};
pub use interrupt::{
    is_globally_interrupted, render_interrupt_panel, reset_global_interrupt, InterruptHandler,
};
pub use iteration_view::{
    ActivityIndicator, GateProgress, GateProgressInfo, GateSummary, IterationPreview,
    IterationSummary, IterationSummaryStack, LiveIterationPanel,
};
pub use mascot::{random_image_mascot, AnimationConfig, Mascot, MascotRenderer, PeekAnimation};
pub use quality_gates::{GateStatus, QualityGateRenderer, QualityGateView};
pub use spinner::{
    progress_chars, spinner_chars, IterationProgress, ProgressManager, RalphSpinner, SpinnerStyle,
};
pub use story_view::{StoryInfo, StoryView, StoryViewState};
pub use summary::{ExecutionSummary, GateStatistics, StoryResult, SummaryRenderer};
