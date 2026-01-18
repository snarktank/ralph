//! Ratatui-based TUI module for Ralph.
//!
//! Provides a proper terminal UI framework with:
//! - Responsive layouts
//! - Smooth animations with easing
//! - Rich progress indicators
//! - Interactive widgets

pub mod animation;
pub mod app;
pub mod progress;
pub mod widgets;

pub use animation::{AnimationState, Easing, Tween};
pub use app::{App, AppState, CompletionData, StoryInfo};
pub use progress::{RichProgress, StoryProgressWidget, StoryState};
pub use widgets::{
    CompletionSummaryWidget, GateChainWidget, GateInfo, GateStatus, GitSummary, IterationWidget,
    StoryHeaderWidget,
};
