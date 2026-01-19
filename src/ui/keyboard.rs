//! Keyboard input handling for live toggle controls.
//!
//! Provides non-blocking keyboard input to toggle display options
//! during execution (e.g., show/hide streaming output).

#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;

use crate::pause::PauseController;

/// Key bindings for toggle controls.
#[derive(Debug, Clone, Copy)]
pub struct KeyBindings {
    /// Toggle streaming output visibility
    pub toggle_streaming: KeyCode,
    /// Toggle detail expansion
    pub toggle_expand: KeyCode,
    /// Quit/interrupt
    pub quit: KeyCode,
    /// Pause execution
    pub pause: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            toggle_streaming: KeyCode::Char('s'),
            toggle_expand: KeyCode::Char('e'),
            quit: KeyCode::Char('q'),
            pause: KeyCode::Char('p'),
        }
    }
}

/// Live toggle state that can be shared across threads.
#[derive(Debug)]
pub struct ToggleState {
    /// Whether streaming output is visible
    pub show_streaming: AtomicBool,
    /// Whether details are expanded
    pub expand_details: AtomicBool,
    /// Whether graceful quit was requested (finish current stories, then exit)
    pub quit_requested: AtomicBool,
    /// Whether immediate interrupt was requested (Ctrl+C)
    pub immediate_interrupt: AtomicBool,
}

impl Default for ToggleState {
    fn default() -> Self {
        Self::new(false, false) // Streaming off by default, expand off by default
    }
}

impl ToggleState {
    /// Create a new toggle state with initial values.
    pub fn new(show_streaming: bool, expand_details: bool) -> Self {
        Self {
            show_streaming: AtomicBool::new(show_streaming),
            expand_details: AtomicBool::new(expand_details),
            quit_requested: AtomicBool::new(false),
            immediate_interrupt: AtomicBool::new(false),
        }
    }

    /// Check if streaming should be shown.
    pub fn should_show_streaming(&self) -> bool {
        self.show_streaming.load(Ordering::Relaxed)
    }

    /// Check if details should be expanded.
    pub fn should_expand_details(&self) -> bool {
        self.expand_details.load(Ordering::Relaxed)
    }

    /// Check if quit was requested.
    pub fn is_quit_requested(&self) -> bool {
        self.quit_requested.load(Ordering::Relaxed)
    }

    /// Toggle streaming visibility.
    pub fn toggle_streaming(&self) -> bool {
        let old = self.show_streaming.fetch_xor(true, Ordering::Relaxed);
        !old // Return new value
    }

    /// Toggle detail expansion.
    pub fn toggle_expand(&self) -> bool {
        let old = self.expand_details.fetch_xor(true, Ordering::Relaxed);
        !old
    }

    /// Request graceful quit (finish current stories, then exit).
    pub fn request_quit(&self) {
        self.quit_requested.store(true, Ordering::Relaxed);
    }

    /// Check if immediate interrupt was requested (Ctrl+C).
    pub fn is_immediate_interrupt(&self) -> bool {
        self.immediate_interrupt.load(Ordering::Relaxed)
    }

    /// Request immediate interrupt (Ctrl+C).
    pub fn request_immediate_interrupt(&self) {
        self.immediate_interrupt.store(true, Ordering::Relaxed);
    }

    /// Check if any form of quit was requested (graceful or immediate).
    pub fn should_stop(&self) -> bool {
        self.is_quit_requested() || self.is_immediate_interrupt()
    }
}

/// Keyboard event that was detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleEvent {
    /// Streaming visibility toggled (new state)
    StreamingToggled(bool),
    /// Detail expansion toggled (new state)
    ExpandToggled(bool),
    /// Quit requested
    QuitRequested,
    /// Ctrl+E pressed (expand shortcut)
    CtrlE,
}

/// Keyboard listener for toggle controls.
///
/// Runs in a background thread and updates shared toggle state.
pub struct KeyboardListener {
    /// Shared toggle state
    state: Arc<ToggleState>,
    /// Key bindings
    bindings: KeyBindings,
    /// Whether the listener is running
    running: Arc<AtomicBool>,
    /// Optional pause controller for pause functionality
    pause_controller: Option<PauseController>,
}

impl KeyboardListener {
    /// Create a new keyboard listener with default bindings.
    pub fn new(state: Arc<ToggleState>) -> Self {
        Self {
            state,
            bindings: KeyBindings::default(),
            running: Arc::new(AtomicBool::new(false)),
            pause_controller: None,
        }
    }

    /// Create a listener with custom key bindings.
    pub fn with_bindings(state: Arc<ToggleState>, bindings: KeyBindings) -> Self {
        Self {
            state,
            bindings,
            running: Arc::new(AtomicBool::new(false)),
            pause_controller: None,
        }
    }

    /// Set the pause controller for pause functionality.
    pub fn with_pause_controller(mut self, pause_controller: PauseController) -> Self {
        self.pause_controller = Some(pause_controller);
        self
    }

    /// Check if the listener is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Start listening for keyboard events in a background thread.
    ///
    /// Returns a handle that can be used to stop the listener.
    pub fn start(&self) -> ListenerHandle {
        let state = Arc::clone(&self.state);
        let bindings = self.bindings;
        let running = Arc::clone(&self.running);
        let pause_controller = self.pause_controller.clone();

        running.store(true, Ordering::Relaxed);

        let handle = thread::spawn(move || {
            // Try to enable raw mode for direct key input
            let raw_mode_enabled = terminal::enable_raw_mode().is_ok();

            while running.load(Ordering::Relaxed) {
                // Poll for events with timeout
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    if let Ok(Event::Key(key_event)) = event::read() {
                        Self::handle_key_event(
                            &state,
                            &bindings,
                            pause_controller.as_ref(),
                            key_event,
                        );
                    }
                }
            }

            // Restore terminal state
            if raw_mode_enabled {
                let _ = terminal::disable_raw_mode();
            }
        });

        ListenerHandle {
            running: Arc::clone(&self.running),
            _handle: handle,
        }
    }

    /// Handle a key event.
    fn handle_key_event(
        state: &ToggleState,
        bindings: &KeyBindings,
        pause_controller: Option<&PauseController>,
        event: KeyEvent,
    ) {
        // Check for Ctrl+C (immediate interrupt)
        if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('c') {
            state.request_immediate_interrupt();
            return;
        }

        // Check for Ctrl+E (expand toggle - common shortcut)
        if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('e') {
            state.toggle_expand();
            return;
        }

        // Check regular key bindings (without modifiers)
        if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT {
            match event.code {
                code if code == bindings.toggle_streaming => {
                    state.toggle_streaming();
                }
                code if code == bindings.toggle_expand => {
                    state.toggle_expand();
                }
                code if code == bindings.quit => {
                    state.request_quit();
                }
                code if code == bindings.pause => {
                    if let Some(controller) = pause_controller {
                        // Only print message if pause was successfully requested
                        // (i.e., not already paused or pause already requested)
                        if controller.request_pause() {
                            // Print message on a new line to avoid corrupting current output
                            println!("\r\nPausing after current iteration...");
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Handle to a running keyboard listener.
pub struct ListenerHandle {
    running: Arc<AtomicBool>,
    _handle: thread::JoinHandle<()>,
}

impl ListenerHandle {
    /// Stop the listener.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Render the toggle hint bar showing available controls.
pub fn render_toggle_hint(state: &ToggleState) -> String {
    let streaming_status = if state.should_show_streaming() {
        "on"
    } else {
        "off"
    };
    let expand_status = if state.should_expand_details() {
        "on"
    } else {
        "off"
    };

    format!(
        "[s] stream: {} | [e] expand: {} | [p] pause | [q] quit",
        streaming_status, expand_status
    )
}

/// Render a compact toggle hint.
pub fn render_compact_hint() -> &'static str {
    "[s]tream [e]xpand [p]ause [q]uit"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_state_default() {
        let state = ToggleState::default();
        assert!(!state.should_show_streaming()); // Streaming off by default
        assert!(!state.should_expand_details());
        assert!(!state.is_quit_requested());
        assert!(!state.is_immediate_interrupt());
    }

    #[test]
    fn test_toggle_streaming() {
        let state = ToggleState::new(true, false);
        assert!(state.should_show_streaming());

        let new_state = state.toggle_streaming();
        assert!(!new_state);
        assert!(!state.should_show_streaming());

        let new_state = state.toggle_streaming();
        assert!(new_state);
        assert!(state.should_show_streaming());
    }

    #[test]
    fn test_toggle_expand() {
        let state = ToggleState::new(true, false);
        assert!(!state.should_expand_details());

        let new_state = state.toggle_expand();
        assert!(new_state);
        assert!(state.should_expand_details());
    }

    #[test]
    fn test_quit_request() {
        let state = ToggleState::default();
        assert!(!state.is_quit_requested());
        assert!(!state.should_stop());

        state.request_quit();
        assert!(state.is_quit_requested());
        assert!(state.should_stop());
    }

    #[test]
    fn test_immediate_interrupt() {
        let state = ToggleState::default();
        assert!(!state.is_immediate_interrupt());
        assert!(!state.should_stop());

        state.request_immediate_interrupt();
        assert!(state.is_immediate_interrupt());
        assert!(state.should_stop());
    }

    #[test]
    fn test_should_stop_either_quit() {
        // Test that should_stop returns true for either quit type
        let state1 = ToggleState::default();
        state1.request_quit();
        assert!(state1.should_stop());

        let state2 = ToggleState::default();
        state2.request_immediate_interrupt();
        assert!(state2.should_stop());
    }

    #[test]
    fn test_render_toggle_hint() {
        let state = ToggleState::new(true, false);
        let hint = render_toggle_hint(&state);
        assert!(hint.contains("stream: on"));
        assert!(hint.contains("expand: off"));
        assert!(hint.contains("[q] quit"));
    }

    #[test]
    fn test_render_toggle_hint_off() {
        let state = ToggleState::new(false, false);
        let hint = render_toggle_hint(&state);
        assert!(hint.contains("stream: off"));
        assert!(hint.contains("expand: off"));
    }

    #[test]
    fn test_render_toggle_hint_expand_on() {
        let state = ToggleState::new(false, true);
        let hint = render_toggle_hint(&state);
        assert!(hint.contains("stream: off"));
        assert!(hint.contains("expand: on"));
    }
}
