//! Terminal UI module for Ralph.
//!
//! Provides rich terminal output with 24-bit color support,
//! progress indicators, and interactive displays.

#![allow(unused_imports)]

mod colors;
mod display;

pub use colors::Theme;
pub use display::RalphDisplay;
