//! Pause controller module for pause/resume state management.
//!
//! This module provides thread-safe pause and resume functionality
//! for agent execution, allowing users to manually pause execution
//! and resume later.

use std::sync::{Arc, RwLock};

/// State of the pause controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseState {
    /// Execution is running normally.
    Running,
    /// A pause has been requested but not yet executed.
    PauseRequested,
    /// Execution is paused.
    Paused,
}

impl Default for PauseState {
    fn default() -> Self {
        Self::Running
    }
}

/// Controller for managing pause/resume state.
///
/// This struct provides thread-safe access to pause state,
/// allowing multiple threads to check and modify the pause state.
#[derive(Debug, Clone)]
pub struct PauseController {
    state: Arc<RwLock<PauseState>>,
}

impl Default for PauseController {
    fn default() -> Self {
        Self::new()
    }
}

impl PauseController {
    /// Creates a new PauseController in the Running state.
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PauseState::Running)),
        }
    }

    /// Requests a pause. Transitions from Running to PauseRequested.
    ///
    /// Returns true if the pause was requested successfully,
    /// false if already paused or pause already requested.
    pub fn request_pause(&self) -> bool {
        let mut state = self.state.write().unwrap();
        if *state == PauseState::Running {
            *state = PauseState::PauseRequested;
            true
        } else {
            false
        }
    }

    /// Checks if a pause has been requested.
    ///
    /// Returns true if the state is PauseRequested.
    pub fn is_pause_requested(&self) -> bool {
        let state = self.state.read().unwrap();
        *state == PauseState::PauseRequested
    }

    /// Executes the pause. Transitions from PauseRequested to Paused.
    ///
    /// Returns true if the pause was executed successfully,
    /// false if not in PauseRequested state.
    pub fn execute_pause(&self) -> bool {
        let mut state = self.state.write().unwrap();
        if *state == PauseState::PauseRequested {
            *state = PauseState::Paused;
            true
        } else {
            false
        }
    }

    /// Resumes execution. Transitions from Paused to Running.
    ///
    /// Returns true if resume was successful,
    /// false if not in Paused state.
    pub fn resume(&self) -> bool {
        let mut state = self.state.write().unwrap();
        if *state == PauseState::Paused {
            *state = PauseState::Running;
            true
        } else {
            false
        }
    }

    /// Returns the current pause state.
    pub fn state(&self) -> PauseState {
        let state = self.state.read().unwrap();
        *state
    }

    /// Checks if execution is currently paused.
    pub fn is_paused(&self) -> bool {
        let state = self.state.read().unwrap();
        *state == PauseState::Paused
    }

    /// Checks if execution is currently running.
    pub fn is_running(&self) -> bool {
        let state = self.state.read().unwrap();
        *state == PauseState::Running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_running() {
        let controller = PauseController::new();
        assert_eq!(controller.state(), PauseState::Running);
        assert!(controller.is_running());
        assert!(!controller.is_paused());
        assert!(!controller.is_pause_requested());
    }

    #[test]
    fn test_request_pause_from_running() {
        let controller = PauseController::new();

        assert!(controller.request_pause());
        assert_eq!(controller.state(), PauseState::PauseRequested);
        assert!(controller.is_pause_requested());
        assert!(!controller.is_running());
        assert!(!controller.is_paused());
    }

    #[test]
    fn test_request_pause_when_already_requested() {
        let controller = PauseController::new();

        assert!(controller.request_pause());
        assert!(!controller.request_pause()); // Should return false
        assert_eq!(controller.state(), PauseState::PauseRequested);
    }

    #[test]
    fn test_request_pause_when_paused() {
        let controller = PauseController::new();

        controller.request_pause();
        controller.execute_pause();

        assert!(!controller.request_pause()); // Should return false
        assert_eq!(controller.state(), PauseState::Paused);
    }

    #[test]
    fn test_execute_pause_from_pause_requested() {
        let controller = PauseController::new();

        controller.request_pause();
        assert!(controller.execute_pause());
        assert_eq!(controller.state(), PauseState::Paused);
        assert!(controller.is_paused());
        assert!(!controller.is_running());
        assert!(!controller.is_pause_requested());
    }

    #[test]
    fn test_execute_pause_from_running() {
        let controller = PauseController::new();

        assert!(!controller.execute_pause()); // Should return false
        assert_eq!(controller.state(), PauseState::Running);
    }

    #[test]
    fn test_execute_pause_when_already_paused() {
        let controller = PauseController::new();

        controller.request_pause();
        controller.execute_pause();

        assert!(!controller.execute_pause()); // Should return false
        assert_eq!(controller.state(), PauseState::Paused);
    }

    #[test]
    fn test_resume_from_paused() {
        let controller = PauseController::new();

        controller.request_pause();
        controller.execute_pause();

        assert!(controller.resume());
        assert_eq!(controller.state(), PauseState::Running);
        assert!(controller.is_running());
        assert!(!controller.is_paused());
    }

    #[test]
    fn test_resume_from_running() {
        let controller = PauseController::new();

        assert!(!controller.resume()); // Should return false
        assert_eq!(controller.state(), PauseState::Running);
    }

    #[test]
    fn test_resume_from_pause_requested() {
        let controller = PauseController::new();

        controller.request_pause();

        assert!(!controller.resume()); // Should return false
        assert_eq!(controller.state(), PauseState::PauseRequested);
    }

    #[test]
    fn test_full_pause_resume_cycle() {
        let controller = PauseController::new();

        // Start running
        assert_eq!(controller.state(), PauseState::Running);

        // Request pause
        assert!(controller.request_pause());
        assert_eq!(controller.state(), PauseState::PauseRequested);

        // Execute pause
        assert!(controller.execute_pause());
        assert_eq!(controller.state(), PauseState::Paused);

        // Resume
        assert!(controller.resume());
        assert_eq!(controller.state(), PauseState::Running);
    }

    #[test]
    fn test_multiple_pause_resume_cycles() {
        let controller = PauseController::new();

        for _ in 0..3 {
            assert!(controller.request_pause());
            assert!(controller.execute_pause());
            assert!(controller.resume());
            assert_eq!(controller.state(), PauseState::Running);
        }
    }

    #[test]
    fn test_clone_shares_state() {
        let controller1 = PauseController::new();
        let controller2 = controller1.clone();

        controller1.request_pause();
        assert!(controller2.is_pause_requested());

        controller2.execute_pause();
        assert!(controller1.is_paused());

        controller1.resume();
        assert!(controller2.is_running());
    }

    #[test]
    fn test_default_is_running() {
        let controller = PauseController::default();
        assert_eq!(controller.state(), PauseState::Running);
    }

    #[test]
    fn test_pause_state_default_is_running() {
        let state = PauseState::default();
        assert_eq!(state, PauseState::Running);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let controller = PauseController::new();
        let controller_clone = controller.clone();

        let handle = thread::spawn(move || {
            controller_clone.request_pause();
            controller_clone.is_pause_requested()
        });

        let result = handle.join().unwrap();
        assert!(result);
        assert!(controller.is_pause_requested());
    }
}
