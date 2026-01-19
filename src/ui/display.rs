//! Main display controller for Ralph's terminal UI.
//!
//! Coordinates all UI components and manages terminal output.

#![allow(dead_code)]

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::mcp::server::ExecutionState;
use crate::quality::gates::GateResult;
use crate::ui::colors::Theme;
use crate::ui::ghostty::{GhosttyFeatures, TitleStatus};
use crate::ui::interrupt::InterruptHandler;
use crate::ui::quality_gates::{QualityGateRenderer, QualityGateView};
use crate::ui::spinner::{IterationProgress, ProgressManager, RalphSpinner};
use crate::ui::story_view::{StoryInfo, StoryView, StoryViewState};
use crate::ui::summary::{ExecutionSummary, GateStatistics, StoryResult, SummaryRenderer};

/// UI mode for terminal display.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum UiMode {
    /// Auto-detect based on terminal capabilities
    #[default]
    Auto,
    /// Force enable rich terminal UI
    Enabled,
    /// Force disable rich terminal UI (plain text only)
    Disabled,
}

/// Configuration options for RalphDisplay.
///
/// Used to configure display behavior from CLI flags or programmatically.
#[derive(Debug, Clone, Default)]
pub struct DisplayOptions {
    /// UI mode (auto, enabled, or disabled)
    pub ui_mode: UiMode,
    /// Whether colors are explicitly enabled/disabled (None = auto-detect)
    pub color: Option<bool>,
    /// Whether to suppress all non-error output
    pub quiet: bool,
    /// Whether to show streaming output (under the hood view)
    pub show_streaming: bool,
    /// Whether to expand detailed sections by default
    pub expand_details: bool,
    /// Verbosity level (0 = normal, 1 = verbose, 2 = very verbose)
    pub verbosity: u8,
}

impl DisplayOptions {
    /// Create new display options with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the UI mode.
    pub fn with_ui_mode(mut self, mode: UiMode) -> Self {
        self.ui_mode = mode;
        self
    }

    /// Set whether colors are enabled.
    ///
    /// Pass `true` to enable colors, `false` to disable.
    /// This overrides auto-detection and NO_COLOR environment variable.
    pub fn with_color(mut self, enabled: bool) -> Self {
        // If --no-color is passed, disable colors. Otherwise, use auto-detect (None).
        self.color = if enabled { None } else { Some(false) };
        self
    }

    /// Explicitly set the color option (Some(true), Some(false), or None for auto).
    pub fn with_color_option(mut self, color: Option<bool>) -> Self {
        self.color = color;
        self
    }

    /// Set quiet mode.
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Enable streaming output (under the hood view).
    pub fn with_streaming(mut self, show: bool) -> Self {
        self.show_streaming = show;
        self
    }

    /// Expand detailed sections by default.
    pub fn with_expand_details(mut self, expand: bool) -> Self {
        self.expand_details = expand;
        self
    }

    /// Set verbosity level.
    ///
    /// - 0: Normal output
    /// - 1: Verbose (show streaming, expand details)
    /// - 2: Very verbose (show all internal details)
    pub fn with_verbosity(mut self, level: u8) -> Self {
        self.verbosity = level;
        if level >= 1 {
            self.show_streaming = true;
            self.expand_details = true;
        }
        self
    }

    /// Check if streaming output should be shown.
    pub fn should_show_streaming(&self) -> bool {
        self.show_streaming || self.verbosity >= 1
    }

    /// Check if details should be expanded.
    pub fn should_expand_details(&self) -> bool {
        self.expand_details || self.verbosity >= 1
    }

    /// Check if colors should be enabled based on options and environment.
    ///
    /// Priority:
    /// 1. Explicit color option from CLI (--no-color)
    /// 2. NO_COLOR environment variable
    /// 3. Default to enabled
    pub fn should_enable_colors(&self) -> bool {
        match self.color {
            Some(enabled) => enabled,
            None => {
                // Check NO_COLOR environment variable
                std::env::var("NO_COLOR").is_err()
            }
        }
    }

    /// Check if rich UI should be enabled based on options and terminal capabilities.
    ///
    /// Priority:
    /// 1. Explicit UI mode (Enabled/Disabled)
    /// 2. Auto-detect based on terminal capabilities
    pub fn should_enable_rich_ui(&self) -> bool {
        match self.ui_mode {
            UiMode::Enabled => true,
            UiMode::Disabled => false,
            UiMode::Auto => Self::detect_rich_ui_support(),
        }
    }

    /// Detect if the terminal supports rich UI features.
    fn detect_rich_ui_support() -> bool {
        // Check for Ghostty
        if std::env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
            return true;
        }

        // Check for other modern terminals that support 24-bit color
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") || term.contains("truecolor") {
                return true;
            }
        }

        // Check COLORTERM for truecolor support
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return true;
            }
        }

        // Check for common modern terminal emulators
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            let modern_terminals = ["iTerm.app", "WezTerm", "Alacritty", "kitty", "vscode"];
            if modern_terminals.iter().any(|t| term_program.contains(t)) {
                return true;
            }
        }

        false
    }
}

/// Main display controller for Ralph's terminal output.
///
/// Coordinates rendering of story panels, progress indicators,
/// quality gates, and other UI components.
#[derive(Debug)]
pub struct RalphDisplay {
    /// Color theme for terminal output
    theme: Theme,
    /// Display options (UI mode, color, quiet)
    options: DisplayOptions,
    /// Whether colors are enabled (computed from options)
    colors_enabled: bool,
    /// Whether rich UI is enabled (computed from options)
    rich_ui_enabled: bool,
    /// Whether the terminal supports advanced features
    advanced_features: bool,
    /// Progress manager for handling multiple progress indicators
    progress_manager: ProgressManager,
    /// Current active spinner (if any)
    active_spinner: Option<RalphSpinner>,
    /// Current iteration progress bar (if any)
    iteration_progress: Option<IterationProgress>,
    /// Interrupt handler for Ctrl+C handling
    interrupt_handler: InterruptHandler,
    /// Current story ID being processed (for interrupt display)
    current_story_id: Option<String>,
    /// Ghostty terminal features
    ghostty: GhosttyFeatures,
    /// Last execution state seen (for detecting transitions)
    last_state: Option<ExecutionState>,
    /// Story view renderer
    story_view: StoryView,
    /// Quality gate renderer
    gate_renderer: QualityGateRenderer,
    /// Summary renderer
    summary_renderer: SummaryRenderer,
    /// Accumulated story results for summary
    story_results: Vec<StoryResult>,
    /// Accumulated gate statistics
    gate_stats: GateStatistics,
    /// Execution start time
    execution_start: Option<Instant>,
    /// Commit count
    commit_count: u32,
}

impl Default for RalphDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl RalphDisplay {
    /// Create a new RalphDisplay with default settings.
    pub fn new() -> Self {
        Self::with_options(DisplayOptions::default())
    }

    /// Create a RalphDisplay with the given options.
    pub fn with_options(options: DisplayOptions) -> Self {
        let theme = Theme::default();
        let colors_enabled = options.should_enable_colors();
        let rich_ui_enabled = options.should_enable_rich_ui();
        Self {
            theme,
            options,
            colors_enabled,
            rich_ui_enabled,
            advanced_features: Self::detect_advanced_features(),
            progress_manager: ProgressManager::with_theme(theme),
            active_spinner: None,
            iteration_progress: None,
            interrupt_handler: InterruptHandler::with_theme(theme),
            current_story_id: None,
            ghostty: GhosttyFeatures::new(),
            last_state: None,
            story_view: StoryView::with_theme(theme),
            gate_renderer: QualityGateRenderer::with_theme(theme),
            summary_renderer: SummaryRenderer::with_theme(theme),
            story_results: Vec::new(),
            gate_stats: GateStatistics::default(),
            execution_start: None,
            commit_count: 0,
        }
    }

    /// Create a RalphDisplay with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self::with_theme_and_options(theme, DisplayOptions::default())
    }

    /// Create a RalphDisplay with a custom theme and options.
    pub fn with_theme_and_options(theme: Theme, options: DisplayOptions) -> Self {
        let colors_enabled = options.should_enable_colors();
        let rich_ui_enabled = options.should_enable_rich_ui();
        Self {
            theme,
            options,
            colors_enabled,
            rich_ui_enabled,
            advanced_features: Self::detect_advanced_features(),
            progress_manager: ProgressManager::with_theme(theme),
            active_spinner: None,
            iteration_progress: None,
            interrupt_handler: InterruptHandler::with_theme(theme),
            current_story_id: None,
            ghostty: GhosttyFeatures::new(),
            last_state: None,
            story_view: StoryView::with_theme(theme),
            gate_renderer: QualityGateRenderer::with_theme(theme),
            summary_renderer: SummaryRenderer::with_theme(theme),
            story_results: Vec::new(),
            gate_stats: GateStatistics::default(),
            execution_start: None,
            commit_count: 0,
        }
    }

    /// Get the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Get the display options.
    pub fn options(&self) -> &DisplayOptions {
        &self.options
    }

    /// Check if colors are enabled.
    pub fn colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    /// Enable or disable colors.
    pub fn set_colors_enabled(&mut self, enabled: bool) {
        self.colors_enabled = enabled;
    }

    /// Check if rich UI is enabled.
    pub fn rich_ui_enabled(&self) -> bool {
        self.rich_ui_enabled
    }

    /// Check if quiet mode is enabled.
    pub fn is_quiet(&self) -> bool {
        self.options.quiet
    }

    /// Check if advanced terminal features are available.
    pub fn advanced_features(&self) -> bool {
        self.advanced_features
    }

    /// Detect if advanced terminal features are available.
    ///
    /// Checks for Ghostty or other modern terminal emulators.
    fn detect_advanced_features() -> bool {
        // Check for Ghostty
        if std::env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
            return true;
        }

        // Check for other modern terminals that support 24-bit color
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") || term.contains("truecolor") {
                return true;
            }
        }

        // Check COLORTERM for truecolor support
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return true;
            }
        }

        false
    }

    // =========================================================================
    // Spinner Management
    // =========================================================================

    /// Start a spinner with the given action message.
    ///
    /// If a spinner is already active, it will be stopped first.
    /// The spinner displays elapsed time and the current action.
    pub fn start_spinner(&mut self, message: impl Into<String>) {
        // Stop any existing spinner first
        self.stop_spinner();

        let spinner = self.progress_manager.add_spinner(message);
        self.active_spinner = Some(spinner);
    }

    /// Stop the current spinner with a success message.
    ///
    /// If no spinner is active, this is a no-op.
    pub fn stop_spinner_with_success(&mut self, message: impl Into<String>) {
        if let Some(spinner) = self.active_spinner.take() {
            spinner.finish_with_success(message);
        }
    }

    /// Stop the current spinner with an error message.
    ///
    /// If no spinner is active, this is a no-op.
    pub fn stop_spinner_with_error(&mut self, message: impl Into<String>) {
        if let Some(spinner) = self.active_spinner.take() {
            spinner.finish_with_error(message);
        }
    }

    /// Stop the current spinner and clear it from the display.
    ///
    /// If no spinner is active, this is a no-op.
    pub fn stop_spinner(&mut self) {
        if let Some(spinner) = self.active_spinner.take() {
            spinner.finish_and_clear();
        }
    }

    /// Update the message on the current spinner.
    ///
    /// If no spinner is active, this is a no-op.
    pub fn update_spinner_message(&self, message: impl Into<String>) {
        if let Some(ref spinner) = self.active_spinner {
            spinner.set_message(message);
        }
    }

    /// Check if a spinner is currently active.
    pub fn has_active_spinner(&self) -> bool {
        self.active_spinner.is_some()
    }

    // =========================================================================
    // Iteration Progress Management
    // =========================================================================

    /// Start an iteration progress bar with the given total iterations.
    ///
    /// If a progress bar is already active, it will be stopped first.
    pub fn start_iteration_progress(&mut self, total: u64) {
        // Stop any existing progress bar first
        self.stop_iteration_progress();

        let progress = self.progress_manager.add_iteration_progress(total);
        self.iteration_progress = Some(progress);
    }

    /// Increment the iteration progress by one.
    ///
    /// If no progress bar is active, this is a no-op.
    pub fn inc_iteration(&mut self) {
        if let Some(ref mut progress) = self.iteration_progress {
            progress.inc();
        }
    }

    /// Set the current iteration position.
    ///
    /// If no progress bar is active, this is a no-op.
    pub fn set_iteration(&mut self, pos: u64) {
        if let Some(ref mut progress) = self.iteration_progress {
            progress.set_position(pos);
        }
    }

    /// Get the current iteration count.
    ///
    /// Returns 0 if no progress bar is active.
    pub fn current_iteration(&self) -> u64 {
        self.iteration_progress
            .as_ref()
            .map(|p| p.current())
            .unwrap_or(0)
    }

    /// Get the total iteration count.
    ///
    /// Returns 0 if no progress bar is active.
    pub fn total_iterations(&self) -> u64 {
        self.iteration_progress
            .as_ref()
            .map(|p| p.total())
            .unwrap_or(0)
    }

    /// Stop and clear the iteration progress bar.
    ///
    /// If no progress bar is active, this is a no-op.
    pub fn stop_iteration_progress(&mut self) {
        if let Some(progress) = self.iteration_progress.take() {
            progress.finish_and_clear();
        }
    }

    /// Finish the iteration progress bar (keeps it visible).
    ///
    /// If no progress bar is active, this is a no-op.
    pub fn finish_iteration_progress(&mut self) {
        if let Some(progress) = self.iteration_progress.take() {
            progress.finish();
        }
    }

    /// Get the progress manager for advanced multi-progress use.
    pub fn progress_manager(&self) -> &ProgressManager {
        &self.progress_manager
    }

    // =========================================================================
    // Interrupt Handling
    // =========================================================================

    /// Install the Ctrl+C signal handler.
    ///
    /// This should be called once at startup to enable graceful interruption.
    pub fn install_interrupt_handler(&self) -> std::io::Result<()> {
        self.interrupt_handler.install_handler()
    }

    /// Check if an interrupt has been requested.
    pub fn is_interrupted(&self) -> bool {
        self.interrupt_handler.is_interrupted()
    }

    /// Get the cancellation flag for cooperative cancellation.
    ///
    /// Pass this to long-running operations so they can check for cancellation.
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.interrupt_handler.cancel_flag()
    }

    /// Set the current story ID for interrupt display.
    pub fn set_current_story(&mut self, story_id: impl Into<String>) {
        self.current_story_id = Some(story_id.into());
    }

    /// Clear the current story ID.
    pub fn clear_current_story(&mut self) {
        self.current_story_id = None;
    }

    /// Get the current story ID.
    pub fn current_story_id(&self) -> Option<&str> {
        self.current_story_id.as_deref()
    }

    /// Display the interruption panel with current story info.
    ///
    /// This should be called when an interrupt is detected to show
    /// the user what story will be retried on the next run.
    pub fn display_interrupt(&self) {
        self.interrupt_handler
            .display_interrupt(self.current_story_id.as_deref());
    }

    /// Render the interruption panel as a string.
    pub fn render_interrupt_panel(&self) -> String {
        self.interrupt_handler
            .render_interrupt_panel(self.current_story_id.as_deref())
    }

    /// Display a cleanup progress step.
    pub fn display_cleanup_step(&self, step: &str) {
        self.interrupt_handler.display_cleanup_step(step);
    }

    /// Reset the interrupt state.
    pub fn reset_interrupt(&self) {
        self.interrupt_handler.reset();
    }

    /// Get the interrupt handler for direct access.
    pub fn interrupt_handler(&self) -> &InterruptHandler {
        &self.interrupt_handler
    }

    // =========================================================================
    // State Change Handling
    // =========================================================================

    /// Update the display based on a new execution state.
    ///
    /// This method detects state transitions and updates the UI accordingly:
    /// - Idle -> Running: Start spinner, show story panel, update title
    /// - Running -> Running: Update iteration progress
    /// - Running -> Completed: Stop spinner with success, record result
    /// - Running -> Failed: Stop spinner with error, record result
    /// - Any -> Idle: Reset UI state
    ///
    /// # Arguments
    /// * `state` - The new execution state
    /// * `story_info` - Optional story information for display
    pub fn update_from_state(&mut self, state: &ExecutionState, story_info: Option<&StoryInfo>) {
        // Detect state transition
        let transition = self.detect_transition(state);

        match transition {
            StateTransition::ToRunning {
                story_id,
                max_iterations,
            } => {
                self.handle_start_running(&story_id, max_iterations, story_info);
            }
            StateTransition::IterationUpdate { iteration, max } => {
                self.handle_iteration_update(iteration, max);
            }
            StateTransition::ToCompleted {
                story_id,
                commit_hash,
            } => {
                self.handle_completed(&story_id, commit_hash.as_deref(), story_info);
            }
            StateTransition::ToFailed { story_id, error } => {
                self.handle_failed(&story_id, &error, story_info);
            }
            StateTransition::ToPaused {
                story_id,
                pause_reason,
            } => {
                self.handle_paused(&story_id, &pause_reason);
            }
            StateTransition::ToWaitingForRetry {
                story_id,
                attempt,
                max_attempts,
            } => {
                self.handle_waiting_for_retry(&story_id, attempt, max_attempts);
            }
            StateTransition::ToIdle => {
                self.handle_idle();
            }
            StateTransition::None => {
                // No transition, nothing to update
            }
        }

        // Update the last seen state
        self.last_state = Some(state.clone());
    }

    /// Detect the type of state transition.
    fn detect_transition(&self, new_state: &ExecutionState) -> StateTransition {
        match (&self.last_state, new_state) {
            // Transition to Running (from various states including Paused and WaitingForRetry)
            (
                None | Some(ExecutionState::Idle),
                ExecutionState::Running {
                    story_id,
                    max_iterations,
                    ..
                },
            )
            | (
                Some(ExecutionState::Completed { .. }),
                ExecutionState::Running {
                    story_id,
                    max_iterations,
                    ..
                },
            )
            | (
                Some(ExecutionState::Failed { .. }),
                ExecutionState::Running {
                    story_id,
                    max_iterations,
                    ..
                },
            )
            | (
                Some(ExecutionState::Paused { .. }),
                ExecutionState::Running {
                    story_id,
                    max_iterations,
                    ..
                },
            )
            | (
                Some(ExecutionState::WaitingForRetry { .. }),
                ExecutionState::Running {
                    story_id,
                    max_iterations,
                    ..
                },
            ) => StateTransition::ToRunning {
                story_id: story_id.clone(),
                max_iterations: *max_iterations,
            },
            // Iteration update (still running)
            (
                Some(ExecutionState::Running {
                    iteration: old_iter,
                    ..
                }),
                ExecutionState::Running {
                    iteration: new_iter,
                    max_iterations,
                    ..
                },
            ) if old_iter != new_iter => StateTransition::IterationUpdate {
                iteration: *new_iter,
                max: *max_iterations,
            },
            // Transition to Completed
            (
                Some(ExecutionState::Running { .. }),
                ExecutionState::Completed {
                    story_id,
                    commit_hash,
                },
            ) => StateTransition::ToCompleted {
                story_id: story_id.clone(),
                commit_hash: commit_hash.clone(),
            },
            // Transition to Failed
            (Some(ExecutionState::Running { .. }), ExecutionState::Failed { story_id, error }) => {
                StateTransition::ToFailed {
                    story_id: story_id.clone(),
                    error: error.clone(),
                }
            }
            // Transition to Paused
            (
                Some(ExecutionState::Running { .. }),
                ExecutionState::Paused {
                    story_id,
                    pause_reason,
                    ..
                },
            ) => StateTransition::ToPaused {
                story_id: story_id.clone(),
                pause_reason: pause_reason.clone(),
            },
            // Transition to WaitingForRetry
            (
                Some(ExecutionState::Running { .. }),
                ExecutionState::WaitingForRetry {
                    story_id,
                    attempt,
                    max_attempts,
                    ..
                },
            ) => StateTransition::ToWaitingForRetry {
                story_id: story_id.clone(),
                attempt: *attempt,
                max_attempts: *max_attempts,
            },
            // Transition to Idle (reset)
            (Some(_), ExecutionState::Idle) => StateTransition::ToIdle,
            // No transition (same state)
            _ => StateTransition::None,
        }
    }

    /// Handle transition to Running state.
    fn handle_start_running(
        &mut self,
        story_id: &str,
        max_iterations: u32,
        story_info: Option<&StoryInfo>,
    ) {
        // Record execution start time
        self.execution_start = Some(Instant::now());

        // Set the current story for interrupt display
        self.set_current_story(story_id);

        // Start the iteration progress bar
        self.start_iteration_progress(max_iterations as u64);
        self.set_iteration(1);

        // Start a spinner for the current action
        self.start_spinner(format!("Running story {}...", story_id));

        // Display the story panel if info is available
        if let Some(info) = story_info {
            let panel = self
                .story_view
                .render_current_story(info, StoryViewState::InProgress);
            println!("{}", panel);
        }

        // Update terminal title
        let _ = self.ghostty.update_title(
            Some(story_id),
            Some((1, max_iterations as u64)),
            TitleStatus::Running,
        );
    }

    /// Handle iteration progress update.
    fn handle_iteration_update(&mut self, iteration: u32, max: u32) {
        // Update the progress bar
        self.set_iteration(iteration as u64);

        // Update the spinner message
        self.update_spinner_message(format!("Iteration {}/{}...", iteration, max));

        // Update terminal title
        let story_id = self.current_story_id.as_deref();
        let _ = self.ghostty.update_title(
            story_id,
            Some((iteration as u64, max as u64)),
            TitleStatus::Running,
        );
    }

    /// Handle transition to Completed state.
    fn handle_completed(
        &mut self,
        story_id: &str,
        commit_hash: Option<&str>,
        story_info: Option<&StoryInfo>,
    ) {
        // Stop the spinner with success
        self.stop_spinner_with_success(format!("Story {} completed!", story_id));

        // Finish the iteration progress
        self.finish_iteration_progress();

        // Record the story result
        let iterations = self.current_iteration() as u32;
        let title = story_info.map(|i| i.title.clone()).unwrap_or_default();
        self.story_results
            .push(StoryResult::passed(story_id, title, iterations));

        // Increment commit count if we have a commit
        if commit_hash.is_some() {
            self.commit_count += 1;
        }

        // Display completed story panel
        if let Some(info) = story_info {
            let panel = self
                .story_view
                .render_current_story(info, StoryViewState::Completed);
            println!("{}", panel);
        }

        // Update terminal title
        let _ = self
            .ghostty
            .update_title(Some(story_id), None, TitleStatus::Success);

        // Clear the current story
        self.clear_current_story();
    }

    /// Handle transition to Failed state.
    fn handle_failed(&mut self, story_id: &str, error: &str, story_info: Option<&StoryInfo>) {
        // Stop the spinner with error
        self.stop_spinner_with_error(format!("Story {} failed: {}", story_id, error));

        // Stop the iteration progress
        self.stop_iteration_progress();

        // Record the story result
        let iterations = self.current_iteration() as u32;
        let title = story_info.map(|i| i.title.clone()).unwrap_or_default();
        self.story_results
            .push(StoryResult::failed(story_id, title, iterations));

        // Display failed story panel
        if let Some(info) = story_info {
            let panel = self
                .story_view
                .render_current_story(info, StoryViewState::Failed);
            println!("{}", panel);
        }

        // Update terminal title
        let _ = self
            .ghostty
            .update_title(Some(story_id), None, TitleStatus::Failed);

        // Clear the current story
        self.clear_current_story();
    }

    /// Handle transition to Paused state.
    fn handle_paused(&mut self, story_id: &str, pause_reason: &str) {
        // Update the spinner to show paused state
        self.update_spinner_message(format!("Story {} paused: {}", story_id, pause_reason));

        // Update terminal title to show paused state
        let _ = self
            .ghostty
            .update_title(Some(story_id), None, TitleStatus::Running);
    }

    /// Handle transition to WaitingForRetry state.
    fn handle_waiting_for_retry(&mut self, story_id: &str, attempt: u32, max_attempts: u32) {
        // Update the spinner to show retry waiting state
        self.update_spinner_message(format!(
            "Story {} waiting for retry (attempt {}/{})",
            story_id, attempt, max_attempts
        ));

        // Update terminal title to show retry state
        let _ = self
            .ghostty
            .update_title(Some(story_id), None, TitleStatus::Running);
    }

    /// Handle transition to Idle state.
    fn handle_idle(&mut self) {
        // Stop any active spinners
        self.stop_spinner();
        self.stop_iteration_progress();

        // Reset terminal title
        let _ = self.ghostty.reset_title();

        // Clear the current story
        self.clear_current_story();
    }

    // =========================================================================
    // Quality Gate Display
    // =========================================================================

    /// Display quality gates during the gate checking phase.
    ///
    /// Shows the current state of all quality gates with pass/fail indicators.
    pub fn display_quality_gates(&self, gates: &[QualityGateView]) {
        let output = self.gate_renderer.render_gates(gates);
        println!("{}", output);
    }

    /// Display quality gate results from GateResult slice.
    pub fn display_gate_results(&mut self, results: &[GateResult]) {
        let output = self.gate_renderer.render_from_results(results);
        println!("{}", output);

        // Accumulate gate statistics
        for result in results {
            self.gate_stats.total_runs += 1;
            if result.message.contains("Skipped") {
                self.gate_stats.total_skipped += 1;
            } else if result.passed {
                self.gate_stats.total_passes += 1;
            } else {
                self.gate_stats.total_failures += 1;
            }
        }
    }

    /// Display a compact summary bar for quality gates.
    pub fn display_gate_summary(&self, gates: &[QualityGateView]) {
        let output = self.gate_renderer.render_summary_bar(gates);
        println!("{}", output);
    }

    /// Start a spinner for quality gate checking.
    pub fn start_gate_checking_spinner(&mut self) {
        self.start_spinner("Running quality gates...");
    }

    /// Stop the quality gate spinner with results.
    pub fn stop_gate_checking_spinner(&mut self, all_passed: bool) {
        if all_passed {
            self.stop_spinner_with_success("All quality gates passed!");
        } else {
            self.stop_spinner_with_error("Some quality gates failed");
        }
    }

    // =========================================================================
    // Summary Display
    // =========================================================================

    /// Display the completion summary.
    ///
    /// Shows comprehensive results including story outcomes,
    /// quality gate statistics, and execution metrics.
    pub fn display_summary(&self) {
        let duration = self
            .execution_start
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO);

        let total_iterations: u32 = self.story_results.iter().map(|s| s.iterations).sum();

        let summary = ExecutionSummary::new(
            self.story_results.clone(),
            total_iterations,
            duration,
            self.commit_count,
            self.gate_stats.clone(),
        );

        let output = self.summary_renderer.render(&summary);
        println!("{}", output);
    }

    /// Trigger the summary view when all stories are complete.
    ///
    /// Call this method when the PRD indicates all stories have passed.
    pub fn trigger_completion_summary(&self) {
        // Update terminal title to show completion
        let all_passed = self.story_results.iter().all(|s| s.passed);
        let status = if all_passed {
            TitleStatus::Success
        } else {
            TitleStatus::Failed
        };
        let _ = self.ghostty.update_title(None, None, status);

        // Display the summary
        self.display_summary();

        // Reset the terminal title after a moment
        let _ = self.ghostty.reset_title();
    }

    /// Reset the display state for a new execution session.
    pub fn reset_session(&mut self) {
        self.story_results.clear();
        self.gate_stats = GateStatistics::default();
        self.execution_start = None;
        self.commit_count = 0;
        self.last_state = None;
        self.current_story_id = None;
        self.stop_spinner();
        self.stop_iteration_progress();
        let _ = self.ghostty.reset_title();
    }

    /// Get the accumulated story results.
    pub fn story_results(&self) -> &[StoryResult] {
        &self.story_results
    }

    /// Get the accumulated gate statistics.
    pub fn gate_statistics(&self) -> &GateStatistics {
        &self.gate_stats
    }

    /// Get the Ghostty features interface.
    pub fn ghostty(&self) -> &GhosttyFeatures {
        &self.ghostty
    }

    // =========================================================================
    // Story Panel Display
    // =========================================================================

    /// Display the current story panel.
    pub fn display_current_story(&self, story: &StoryInfo, state: StoryViewState) {
        let panel = self.story_view.render_current_story(story, state);
        println!("{}", panel);
    }

    /// Display the next story preview panel.
    pub fn display_next_story(&self, story: &StoryInfo) {
        let panel = self.story_view.render_next_story(story);
        println!("{}", panel);
    }
}

/// Internal enum representing state transitions.
#[derive(Debug)]
enum StateTransition {
    /// Transitioning to Running state
    ToRunning {
        story_id: String,
        max_iterations: u32,
    },
    /// Iteration count updated while running
    IterationUpdate { iteration: u32, max: u32 },
    /// Transitioning to Completed state
    ToCompleted {
        story_id: String,
        commit_hash: Option<String>,
    },
    /// Transitioning to Failed state
    ToFailed { story_id: String, error: String },
    /// Transitioning to Paused state
    ToPaused {
        story_id: String,
        pause_reason: String,
    },
    /// Transitioning to WaitingForRetry state
    ToWaitingForRetry {
        story_id: String,
        attempt: u32,
        max_attempts: u32,
    },
    /// Transitioning to Idle state
    ToIdle,
    /// No transition (same state)
    None,
}
