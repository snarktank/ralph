//! TUI application state and management.
//!
//! Provides the main application loop and state management
//! for the ratatui-based terminal UI.

#![allow(dead_code)]

use std::io::{self, stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};

use super::animation::AnimationState;
use super::progress::{StoryProgressWidget, StoryState};
use super::widgets::{
    CompletionSummaryWidget, GateChainWidget, GateInfo, GateStatus, GitSummary, IterationWidget,
    StoryHeaderWidget,
};

/// Application state for the TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// Initial startup
    Starting,
    /// Running stories
    Running,
    /// Paused (user interrupt)
    Paused,
    /// Completed successfully
    Completed,
    /// Failed
    Failed,
    /// Exiting
    Exiting,
}

/// The main TUI application.
pub struct App {
    /// Current state
    state: AppState,
    /// Animation state
    animation: AnimationState,
    /// Current story info
    current_story: Option<StoryInfo>,
    /// All stories with status
    stories: Vec<(String, StoryState)>,
    /// Current iteration
    current_iteration: u32,
    /// Max iterations
    max_iterations: u32,
    /// Current gates
    gates: Vec<GateInfo>,
    /// Whether to show completion summary
    show_completion: bool,
    /// Completion summary data
    completion: Option<CompletionData>,
    /// Whether running in alternate screen mode
    alternate_screen: bool,
}

/// Story information for display.
#[derive(Debug, Clone)]
pub struct StoryInfo {
    /// Story ID
    pub id: String,
    /// Story title
    pub title: String,
    /// Priority
    pub priority: u32,
}

/// Completion data for summary.
#[derive(Debug, Clone)]
pub struct CompletionData {
    /// Story ID
    pub story_id: String,
    /// Whether passed
    pub passed: bool,
    /// Duration in seconds
    pub duration_secs: f64,
    /// Iterations used
    pub iterations_used: u32,
    /// Max iterations
    pub max_iterations: u32,
    /// Gates
    pub gates: Vec<GateInfo>,
    /// Git info
    pub git: GitSummary,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new application.
    pub fn new() -> Self {
        Self {
            state: AppState::Starting,
            animation: AnimationState::new(30), // 30 FPS for smoother animations
            current_story: None,
            stories: Vec::new(),
            current_iteration: 0,
            max_iterations: 10,
            gates: Vec::new(),
            show_completion: false,
            completion: None,
            alternate_screen: false,
        }
    }

    /// Set stories.
    pub fn with_stories(mut self, stories: Vec<(String, StoryState)>) -> Self {
        self.stories = stories;
        self
    }

    /// Set current story.
    pub fn with_current_story(mut self, story: StoryInfo) -> Self {
        self.current_story = Some(story);
        self
    }

    /// Set iteration progress.
    pub fn with_iteration(mut self, current: u32, max: u32) -> Self {
        self.current_iteration = current;
        self.max_iterations = max;
        self
    }

    /// Set gates.
    pub fn with_gates(mut self, gates: Vec<GateInfo>) -> Self {
        self.gates = gates;
        self
    }

    /// Set completion data.
    pub fn with_completion(mut self, completion: CompletionData) -> Self {
        self.completion = Some(completion);
        self.show_completion = true;
        self
    }

    /// Update animation state.
    pub fn tick(&mut self) {
        self.animation.tick();
    }

    /// Check if should continue running.
    pub fn is_running(&self) -> bool {
        !matches!(self.state, AppState::Exiting)
    }

    /// Get current state.
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Set state.
    pub fn set_state(&mut self, state: AppState) {
        self.state = state;
    }

    /// Render the main view to a string (for non-TUI output).
    pub fn render_to_string(&self, width: usize) -> String {
        let mut output = String::new();

        // Story header
        if let Some(ref story) = self.current_story {
            let header = StoryHeaderWidget::new(&story.id, &story.title, story.priority);
            output.push_str(&header.render_string(width));
            output.push('\n');
        }

        // Progress
        if !self.stories.is_empty() {
            let progress = StoryProgressWidget::new(self.stories.clone());
            output.push_str(&progress.render_string());
            output.push('\n');
        }

        // Iteration
        if self.current_iteration > 0 {
            let iteration = IterationWidget::new(self.current_iteration, self.max_iterations)
                .with_gates(self.gates.clone())
                .with_animation(self.animation.clone());
            output.push_str(&iteration.render_string());
            output.push('\n');
        }

        // Completion summary
        if self.show_completion {
            if let Some(ref completion) = self.completion {
                let summary = CompletionSummaryWidget::new(
                    &completion.story_id,
                    completion.passed,
                    completion.duration_secs,
                    completion.iterations_used,
                    completion.max_iterations,
                )
                .with_gates(completion.gates.clone())
                .with_git(completion.git.clone());
                output.push('\n');
                output.push_str(&summary.render_string(width));
            }
        }

        output
    }

    /// Render to a ratatui frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Story header
                Constraint::Length(2), // Progress
                Constraint::Length(2), // Iteration
                Constraint::Min(0),    // Content/Completion
            ])
            .split(area);

        // Story header
        if let Some(ref story) = self.current_story {
            let header = StoryHeaderWidget::new(&story.id, &story.title, story.priority);
            frame.render_widget(header, chunks[0]);
        }

        // Progress
        if !self.stories.is_empty() {
            let progress = StoryProgressWidget::new(self.stories.clone())
                .with_animation(self.animation.clone());
            frame.render_widget(progress, chunks[1]);
        }

        // Iteration
        if self.current_iteration > 0 {
            let iteration = IterationWidget::new(self.current_iteration, self.max_iterations)
                .with_gates(self.gates.clone())
                .with_animation(self.animation.clone());
            frame.render_widget(iteration, chunks[2]);
        }

        // Completion summary (centered overlay)
        if self.show_completion {
            if let Some(ref completion) = self.completion {
                let summary = CompletionSummaryWidget::new(
                    &completion.story_id,
                    completion.passed,
                    completion.duration_secs,
                    completion.iterations_used,
                    completion.max_iterations,
                )
                .with_gates(completion.gates.clone())
                .with_git(completion.git.clone());

                // Center the summary
                let summary_width = 50_u16.min(area.width.saturating_sub(4));
                let summary_height = 10_u16.min(area.height.saturating_sub(4));
                let summary_area = Rect {
                    x: (area.width - summary_width) / 2,
                    y: (area.height - summary_height) / 2,
                    width: summary_width,
                    height: summary_height,
                };

                // Clear background
                frame.render_widget(Clear, summary_area);
                frame.render_widget(summary, summary_area);
            }
        }
    }
}

/// Initialize terminal for TUI mode.
pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore terminal from TUI mode.
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

/// Simple output without full TUI (for inline display).
pub fn render_inline(app: &App, width: usize) -> String {
    app.render_to_string(width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.state(), &AppState::Starting);
        assert!(app.is_running());
    }

    #[test]
    fn test_app_with_stories() {
        let stories = vec![
            ("US-001".to_string(), StoryState::Passed),
            ("US-002".to_string(), StoryState::Running),
        ];
        let app = App::new().with_stories(stories);
        assert_eq!(app.stories.len(), 2);
    }

    #[test]
    fn test_app_with_current_story() {
        let story = StoryInfo {
            id: "US-001".to_string(),
            title: "Test Story".to_string(),
            priority: 1,
        };
        let app = App::new().with_current_story(story);
        assert!(app.current_story.is_some());
    }

    #[test]
    fn test_app_render_to_string() {
        let story = StoryInfo {
            id: "US-001".to_string(),
            title: "Test Story".to_string(),
            priority: 1,
        };
        let app = App::new().with_current_story(story).with_iteration(3, 10);

        let output = app.render_to_string(60);
        assert!(output.contains("US-001"));
        assert!(output.contains("3/10"));
    }

    #[test]
    fn test_app_state_transitions() {
        let mut app = App::new();
        assert_eq!(app.state(), &AppState::Starting);

        app.set_state(AppState::Running);
        assert_eq!(app.state(), &AppState::Running);

        app.set_state(AppState::Completed);
        assert_eq!(app.state(), &AppState::Completed);

        app.set_state(AppState::Exiting);
        assert!(!app.is_running());
    }
}
