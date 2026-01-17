//! Terminal UI module for Ralph.
//!
//! Provides rich terminal output with 24-bit color support,
//! progress indicators, and interactive displays.

#![allow(unused_imports)]

mod colors;
mod display;
mod interrupt;
mod quality_gates;
mod spinner;
mod story_view;

pub use colors::Theme;
pub use display::RalphDisplay;
pub use interrupt::{
    is_globally_interrupted, render_interrupt_panel, reset_global_interrupt, InterruptHandler,
};
pub use quality_gates::{GateStatus, QualityGateRenderer, QualityGateView};
pub use spinner::{
    progress_chars, spinner_chars, IterationProgress, ProgressManager, RalphSpinner, SpinnerStyle,
};
pub use story_view::{StoryInfo, StoryView, StoryViewState};
