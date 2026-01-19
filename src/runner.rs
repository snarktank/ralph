// Terminal runner for Ralph
// This module implements the default "run all stories until complete" behavior

use std::io::{self, Write};
use std::path::PathBuf;
use tokio::sync::watch;

use chrono::Utc;

use crate::checkpoint::{Checkpoint, CheckpointManager, PauseReason, StoryCheckpoint};
use crate::error::classification::ErrorCategory;
use crate::mcp::tools::executor::{detect_agent, ExecutorConfig, StoryExecutor};
use crate::mcp::tools::load_prd::{PrdFile, PrdUserStory};
use crate::notification::Notification;
use crate::parallel::scheduler::ParallelRunnerConfig;
use crate::ui::{DisplayOptions, TuiRunnerDisplay};

/// User's choice when prompted about an existing checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeChoice {
    /// Resume execution from the checkpoint.
    Resume,
    /// Discard the checkpoint and start fresh.
    Discard,
    /// Show detailed checkpoint information.
    ViewDetails,
}

/// Configuration for the runner
#[derive(Debug, Clone)]
#[allow(dead_code)] // parallel fields will be used in future stories
pub struct RunnerConfig {
    /// Path to the PRD file (defaults to "prd.json" in current dir)
    pub prd_path: PathBuf,
    /// Working directory (defaults to current dir)
    pub working_dir: PathBuf,
    /// Maximum iterations per story
    pub max_iterations_per_story: u32,
    /// Maximum total iterations across all stories (0 = unlimited)
    pub max_total_iterations: u32,
    /// Agent command to use (auto-detect if None)
    pub agent_command: Option<String>,
    /// Display options for UI rendering (includes quiet mode, verbosity, etc.)
    pub display_options: DisplayOptions,
    /// Enable parallel execution mode
    pub parallel: bool,
    /// Configuration for parallel execution (used when parallel is true)
    pub parallel_config: Option<ParallelRunnerConfig>,
    /// Resume from checkpoint if available
    pub resume: bool,
    /// Skip checkpoint prompt (do not resume)
    pub no_resume: bool,
    /// Agent timeout override in seconds (None = use default)
    pub timeout_seconds: Option<u64>,
    /// Disable checkpointing
    pub no_checkpoint: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            prd_path: PathBuf::from("prd.json"),
            working_dir: PathBuf::from("."),
            max_iterations_per_story: 10,
            max_total_iterations: 0, // unlimited
            agent_command: None,
            display_options: DisplayOptions::default(),
            parallel: false,
            parallel_config: None,
            resume: false,
            no_resume: false,
            timeout_seconds: None,
            no_checkpoint: false,
        }
    }
}

/// Result of running all stories
#[derive(Debug)]
#[allow(dead_code)] // Fields may be used by callers
pub struct RunResult {
    /// Whether all stories passed
    pub all_passed: bool,
    /// Number of stories that passed
    pub stories_passed: usize,
    /// Total number of stories
    pub total_stories: usize,
    /// Total iterations used
    pub total_iterations: u32,
    /// Error message if failed
    pub error: Option<String>,
}

/// The main runner that iterates through stories
pub struct Runner {
    config: RunnerConfig,
    /// Optional checkpoint manager (None if checkpointing is disabled)
    checkpoint_manager: Option<CheckpointManager>,
}

impl Runner {
    /// Create a new runner with the given configuration
    pub fn new(config: RunnerConfig) -> Self {
        // Initialize checkpoint manager if checkpointing is enabled
        let checkpoint_manager = if config.no_checkpoint {
            None
        } else {
            match CheckpointManager::new(&config.working_dir) {
                Ok(manager) => Some(manager),
                Err(e) => {
                    // Log warning but continue without checkpointing
                    eprintln!("Warning: Failed to initialize checkpoint manager: {}", e);
                    None
                }
            }
        };

        Self {
            config,
            checkpoint_manager,
        }
    }

    /// Run all stories until all pass or an error occurs.
    ///
    /// Routes to parallel or sequential execution based on config.parallel.
    pub async fn run(&self) -> RunResult {
        if self.config.parallel {
            // Use parallel execution
            let parallel_config = self.config.parallel_config.clone().unwrap_or_default();
            let parallel_runner = crate::parallel::scheduler::ParallelRunner::new(
                parallel_config,
                self.config.clone(),
            );
            parallel_runner.run().await
        } else {
            // Use sequential execution
            self.run_sequential().await
        }
    }

    /// Run all stories sequentially until all pass or an error occurs
    async fn run_sequential(&self) -> RunResult {
        let mut total_iterations: u32 = 0;

        // Create TUI display with display options
        let mut display =
            TuiRunnerDisplay::with_display_options(self.config.display_options.clone());

        // Handle checkpoint resume at startup
        let resume_from = self.handle_checkpoint_resume();

        // Load and validate PRD
        let prd = match self.load_prd() {
            Ok(prd) => prd,
            Err(e) => {
                return RunResult {
                    all_passed: false,
                    stories_passed: 0,
                    total_stories: 0,
                    total_iterations: 0,
                    error: Some(format!("Failed to load PRD: {}", e)),
                };
            }
        };

        let total_stories = prd.user_stories.len();

        // Initialize display with story list
        let story_status: Vec<(String, bool)> = prd
            .user_stories
            .iter()
            .map(|s| (s.id.clone(), s.passes))
            .collect();
        display.init_stories(story_status);

        // Check if all stories already pass
        let passing_count = prd.user_stories.iter().filter(|s| s.passes).count();
        if passing_count == total_stories {
            display.display_all_complete(total_stories);
            return RunResult {
                all_passed: true,
                stories_passed: total_stories,
                total_stories,
                total_iterations: 0,
                error: None,
            };
        }

        // Detect agent (only needed if there are failing stories)
        let agent = match self.config.agent_command.clone().or_else(detect_agent) {
            Some(a) => a,
            None => {
                return RunResult {
                    all_passed: false,
                    stories_passed: passing_count,
                    total_stories,
                    total_iterations: 0,
                    error: Some("No agent found. Install Claude Code CLI or Amp CLI.".to_string()),
                };
            }
        };

        // Display startup banner (with resume info if applicable)
        if let Some(ref checkpoint) = resume_from {
            println!(
                "Resuming from checkpoint: {} (iteration {}/{})",
                checkpoint.story_id, checkpoint.iteration, checkpoint.max_iterations
            );
        }
        display.display_startup(
            &self.config.prd_path.display().to_string(),
            &agent,
            passing_count,
            total_stories,
        );

        // Track if we're resuming and need to start from a specific iteration
        let mut resume_state = resume_from;

        // Main loop - continue until all stories pass
        loop {
            // Reload PRD each iteration to get updated passes status
            let prd = match self.load_prd() {
                Ok(prd) => prd,
                Err(e) => {
                    return RunResult {
                        all_passed: false,
                        stories_passed: self.count_passing_stories().unwrap_or(0),
                        total_stories,
                        total_iterations,
                        error: Some(format!("Failed to reload PRD: {}", e)),
                    };
                }
            };

            // Update display with current story states
            let story_status: Vec<(String, bool)> = prd
                .user_stories
                .iter()
                .map(|s| (s.id.clone(), s.passes))
                .collect();
            display.init_stories(story_status);

            // Determine which story to work on and the starting iteration
            let (next_story, start_iteration) = if let Some(resume_checkpoint) = resume_state.take()
            {
                // Resuming from checkpoint: find the specific story
                let story = prd
                    .user_stories
                    .iter()
                    .find(|s| s.id == resume_checkpoint.story_id && !s.passes);

                match story {
                    Some(s) => (Some(s), resume_checkpoint.iteration),
                    None => {
                        // Story not found or already passes, fall back to normal selection
                        (self.find_next_story(&prd), 1)
                    }
                }
            } else {
                // Normal operation: find next story by priority
                (self.find_next_story(&prd), 1)
            };

            match next_story {
                None => {
                    // All stories pass! Clear checkpoint on full completion.
                    self.clear_checkpoint();
                    display.display_all_complete(total_stories);
                    return RunResult {
                        all_passed: true,
                        stories_passed: total_stories,
                        total_stories,
                        total_iterations,
                        error: None,
                    };
                }
                Some(story) => {
                    // Check total iteration limit
                    if self.config.max_total_iterations > 0
                        && total_iterations >= self.config.max_total_iterations
                    {
                        // Save checkpoint on reaching iteration limit
                        self.save_checkpoint(
                            &story.id,
                            start_iteration,
                            self.config.max_iterations_per_story,
                            PauseReason::Error(format!(
                                "Max total iterations ({}) reached",
                                self.config.max_total_iterations
                            )),
                        );
                        return RunResult {
                            all_passed: false,
                            stories_passed: self.count_passing_stories().unwrap_or(0),
                            total_stories,
                            total_iterations,
                            error: Some(format!(
                                "Max total iterations ({}) reached",
                                self.config.max_total_iterations
                            )),
                        };
                    }

                    // Display story start (indicate if resuming)
                    if start_iteration > 1 {
                        println!("  Resuming from iteration {}", start_iteration);
                    }
                    display.start_story(&story.id, &story.title, story.priority);

                    // Calculate remaining iterations when resuming
                    let max_iterations = self.config.max_iterations_per_story;
                    let remaining_iterations = max_iterations.saturating_sub(start_iteration - 1);

                    // Execute the story
                    let executor_config = ExecutorConfig {
                        prd_path: self.config.prd_path.clone(),
                        project_root: self.config.working_dir.clone(),
                        progress_path: self.config.working_dir.join("progress.txt"),
                        quality_profile: None,
                        agent_command: agent.clone(),
                        max_iterations: remaining_iterations,
                        git_mutex: None, // Sequential execution doesn't need mutex
                        timeout_config: crate::timeout::TimeoutConfig::default(),
                        ..Default::default()
                    };

                    let executor = StoryExecutor::new(executor_config);
                    let (_cancel_tx, cancel_rx) = watch::channel(false);

                    let story_id = story.id.clone();

                    // Save checkpoint before starting story execution (for recovery if interrupted)
                    self.save_checkpoint(
                        &story_id,
                        start_iteration,
                        max_iterations,
                        PauseReason::IterationBoundary,
                    );

                    let result = executor
                        .execute_story(&story_id, cancel_rx, |iter, _max| {
                            // Adjust iteration display to account for resume offset
                            let adjusted_iter = iter + start_iteration - 1;
                            display.update_iteration(adjusted_iter, max_iterations);
                        })
                        .await;

                    // Calculate total iterations used (including those before resume)
                    let iterations_this_run =
                        result.as_ref().map(|r| r.iterations_used).unwrap_or(1);
                    total_iterations += iterations_this_run;

                    match result {
                        Ok(exec_result) => {
                            if exec_result.success {
                                // Clear checkpoint on successful story completion
                                self.clear_checkpoint();
                                display
                                    .complete_story(&story_id, exec_result.commit_hash.as_deref());
                            } else {
                                // Save checkpoint on story failure (quality gates didn't pass)
                                let final_iteration =
                                    start_iteration + exec_result.iterations_used - 1;
                                self.save_checkpoint(
                                    &story_id,
                                    final_iteration,
                                    max_iterations,
                                    PauseReason::Error(
                                        exec_result
                                            .error
                                            .clone()
                                            .unwrap_or_else(|| "Quality gates failed".to_string()),
                                    ),
                                );
                                display.fail_story(
                                    &story_id,
                                    exec_result.error.as_deref().unwrap_or("unknown"),
                                );
                            }
                        }
                        Err(e) => {
                            // Classify the error using ErrorDetector
                            let category = e.classify();

                            // Handle based on error category
                            match &category {
                                ErrorCategory::Transient(_) => {
                                    // For transient errors, save checkpoint and pause
                                    // Will retry on next run
                                    let notification = Notification::paused(format!(
                                        "Transient error (will retry on next run): {}",
                                        e
                                    ));
                                    println!("{}", notification);
                                    self.save_checkpoint(
                                        &story_id,
                                        start_iteration,
                                        max_iterations,
                                        PauseReason::Error(e.to_string()),
                                    );
                                    display.fail_story(&story_id, &e.to_string());
                                    // Continue to next story, will retry on next run
                                }
                                ErrorCategory::UsageLimit(_) => {
                                    // For usage limit errors, save checkpoint and pause
                                    let notification = Notification::paused(format!(
                                        "Usage limit exceeded: {}",
                                        e
                                    ));
                                    println!("{}", notification);
                                    self.save_checkpoint(
                                        &story_id,
                                        start_iteration,
                                        max_iterations,
                                        PauseReason::UsageLimitExceeded,
                                    );
                                    display.fail_story(&story_id, &e.to_string());
                                    // Return immediately - user needs to wait or upgrade
                                    return RunResult {
                                        all_passed: false,
                                        stories_passed: self.count_passing_stories().unwrap_or(0),
                                        total_stories,
                                        total_iterations,
                                        error: Some(
                                            "Usage limit exceeded. Checkpoint saved. Resume later with: ralph --resume".to_string()
                                        ),
                                    };
                                }
                                ErrorCategory::Fatal(_) => {
                                    // For fatal errors, stop execution with clear message
                                    self.save_checkpoint(
                                        &story_id,
                                        start_iteration,
                                        max_iterations,
                                        PauseReason::Error(e.to_string()),
                                    );
                                    display.fail_story(&story_id, &e.to_string());
                                    // Return immediately - error is unrecoverable
                                    return RunResult {
                                        all_passed: false,
                                        stories_passed: self.count_passing_stories().unwrap_or(0),
                                        total_stories,
                                        total_iterations,
                                        error: Some(format!("Fatal error: {}", e)),
                                    };
                                }
                                ErrorCategory::Timeout(_) => {
                                    // For timeout errors, save checkpoint and report
                                    let notification = Notification::timeout(
                                        std::time::Duration::from_secs(0),
                                        format!("Story {} execution", story_id),
                                    );
                                    println!("{}", notification);
                                    self.save_checkpoint(
                                        &story_id,
                                        start_iteration,
                                        max_iterations,
                                        PauseReason::Timeout,
                                    );
                                    display.fail_story(&story_id, &e.to_string());
                                    // Return with checkpoint info
                                    return RunResult {
                                        all_passed: false,
                                        stories_passed: self.count_passing_stories().unwrap_or(0),
                                        total_stories,
                                        total_iterations,
                                        error: Some(format!(
                                            "Timeout: {}. Checkpoint saved. Resume with: ralph --resume",
                                            e
                                        )),
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Load the PRD file
    fn load_prd(&self) -> Result<PrdFile, String> {
        let content = std::fs::read_to_string(&self.config.prd_path)
            .map_err(|e| format!("Failed to read {}: {}", self.config.prd_path.display(), e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse PRD: {}", e))
    }

    /// Find the next story to work on (highest priority where passes: false)
    fn find_next_story<'a>(&self, prd: &'a PrdFile) -> Option<&'a PrdUserStory> {
        prd.user_stories
            .iter()
            .filter(|s| !s.passes)
            .min_by_key(|s| s.priority) // Lower priority number = higher priority
    }

    /// Count stories that are currently passing
    fn count_passing_stories(&self) -> Result<usize, String> {
        let prd = self.load_prd()?;
        Ok(prd.user_stories.iter().filter(|s| s.passes).count())
    }

    /// Save a checkpoint with the current execution state.
    ///
    /// Does nothing if checkpointing is disabled.
    fn save_checkpoint(
        &self,
        story_id: &str,
        iteration: u32,
        max_iterations: u32,
        pause_reason: PauseReason,
    ) {
        if let Some(ref manager) = self.checkpoint_manager {
            let uncommitted_files = self.get_uncommitted_files().unwrap_or_default();
            let checkpoint = Checkpoint::new(
                Some(StoryCheckpoint::new(story_id, iteration, max_iterations)),
                pause_reason,
                uncommitted_files,
            );

            if let Err(e) = manager.save(&checkpoint) {
                eprintln!("Warning: Failed to save checkpoint: {}", e);
            }
        }
    }

    /// Clear the checkpoint (called on successful completion).
    ///
    /// Does nothing if checkpointing is disabled.
    fn clear_checkpoint(&self) {
        if let Some(ref manager) = self.checkpoint_manager {
            if let Err(e) = manager.clear() {
                eprintln!("Warning: Failed to clear checkpoint: {}", e);
            }
        }
    }

    /// Get list of uncommitted files from git.
    fn get_uncommitted_files(&self) -> Result<Vec<String>, String> {
        use std::process::Command;

        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.config.working_dir)
            .output()
            .map_err(|e| format!("Failed to run git status: {}", e))?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.len() > 3 {
                    Some(line[3..].to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(files)
    }

    /// Check if a checkpoint exists and return it if found.
    fn check_for_checkpoint(&self) -> Option<Checkpoint> {
        self.checkpoint_manager
            .as_ref()
            .and_then(|manager| manager.load().ok().flatten())
    }

    /// Display a summary of the checkpoint.
    fn display_checkpoint_summary(&self, checkpoint: &Checkpoint) {
        println!();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║               Checkpoint Found                               ║");
        println!("╠══════════════════════════════════════════════════════════════╣");

        // Story information
        if let Some(ref story) = checkpoint.current_story {
            println!("║  Story:      {:<48} ║", story.story_id);
            println!(
                "║  Iteration:  {}/{:<44} ║",
                story.iteration, story.max_iterations
            );
        } else {
            println!("║  Story:      (none)                                          ║");
        }

        // Pause reason
        let reason_str = match &checkpoint.pause_reason {
            PauseReason::UsageLimitExceeded => "Usage limit exceeded".to_string(),
            PauseReason::RateLimited => "Rate limited".to_string(),
            PauseReason::UserRequested => "User requested".to_string(),
            PauseReason::Timeout => "Timeout".to_string(),
            PauseReason::IterationBoundary => "Iteration boundary".to_string(),
            PauseReason::Error(msg) => {
                let truncated = if msg.len() > 40 {
                    format!("{}...", &msg[..37])
                } else {
                    msg.clone()
                };
                format!("Error: {}", truncated)
            }
        };
        println!("║  Reason:     {:<48} ║", reason_str);

        // Age
        let now = Utc::now();
        let age = now.signed_duration_since(checkpoint.created_at);
        let age_str = Self::format_duration(age);
        println!("║  Age:        {:<48} ║", age_str);

        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
    }

    /// Display detailed checkpoint information.
    fn display_checkpoint_details(&self, checkpoint: &Checkpoint) {
        println!();
        println!("═══════════════════════════════════════════════════════════════");
        println!("                    Checkpoint Details                          ");
        println!("═══════════════════════════════════════════════════════════════");
        println!();

        // Basic info
        println!("Version:       {}", checkpoint.version);
        println!(
            "Created:       {}",
            checkpoint.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );

        // Age
        let now = Utc::now();
        let age = now.signed_duration_since(checkpoint.created_at);
        println!("Age:           {}", Self::format_duration(age));
        println!();

        // Story checkpoint details
        if let Some(ref story) = checkpoint.current_story {
            println!("Story Information:");
            println!("  ID:          {}", story.story_id);
            println!(
                "  Iteration:   {} of {}",
                story.iteration, story.max_iterations
            );
            println!(
                "  Remaining:   {} iterations",
                story.max_iterations.saturating_sub(story.iteration)
            );
            println!();
        }

        // Pause reason
        println!("Pause Reason:");
        match &checkpoint.pause_reason {
            PauseReason::UsageLimitExceeded => {
                println!("  Type:        Usage Limit Exceeded");
                println!("  Details:     API usage quota has been exhausted");
            }
            PauseReason::RateLimited => {
                println!("  Type:        Rate Limited");
                println!("  Details:     Too many requests, rate limit applied");
            }
            PauseReason::UserRequested => {
                println!("  Type:        User Requested");
                println!("  Details:     Execution was paused by user action");
            }
            PauseReason::Timeout => {
                println!("  Type:        Timeout");
                println!("  Details:     Operation exceeded configured timeout");
            }
            PauseReason::IterationBoundary => {
                println!("  Type:        Iteration Boundary");
                println!("  Details:     Checkpoint saved at iteration start for recovery");
            }
            PauseReason::Error(msg) => {
                println!("  Type:        Error");
                println!("  Details:     {}", msg);
            }
        }
        println!();

        // Uncommitted files
        if !checkpoint.uncommitted_files.is_empty() {
            println!(
                "Uncommitted Files ({}):",
                checkpoint.uncommitted_files.len()
            );
            for file in &checkpoint.uncommitted_files {
                println!("  - {}", file);
            }
            println!();
        }

        println!("═══════════════════════════════════════════════════════════════");
        println!();
    }

    /// Prompt the user for their choice regarding the checkpoint.
    fn prompt_resume_choice(&self) -> ResumeChoice {
        loop {
            print!("Choose action: [R]esume / [D]iscard / [V]iew details: ");
            let _ = io::stdout().flush();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                // If we can't read stdin, default to discard
                return ResumeChoice::Discard;
            }

            let input = input.trim().to_lowercase();
            match input.as_str() {
                "r" | "resume" => return ResumeChoice::Resume,
                "d" | "discard" => return ResumeChoice::Discard,
                "v" | "view" | "view details" => return ResumeChoice::ViewDetails,
                "" => {
                    // Default to resume on empty input
                    return ResumeChoice::Resume;
                }
                _ => {
                    println!("Invalid choice. Please enter R, D, or V.");
                }
            }
        }
    }

    /// Handle the checkpoint resume flow at startup.
    ///
    /// Returns the story checkpoint to resume from, if any.
    fn handle_checkpoint_resume(&self) -> Option<StoryCheckpoint> {
        // Check if checkpointing is disabled
        if self.config.no_checkpoint {
            return None;
        }

        // Check for existing checkpoint
        let checkpoint = self.check_for_checkpoint()?;

        // Handle --no-resume flag: discard without prompt
        if self.config.no_resume {
            self.clear_checkpoint();
            return None;
        }

        // Handle --resume flag: auto-resume without prompt
        if self.config.resume {
            return checkpoint.current_story;
        }

        // Interactive mode: prompt user
        self.display_checkpoint_summary(&checkpoint);

        loop {
            let choice = self.prompt_resume_choice();
            match choice {
                ResumeChoice::Resume => {
                    return checkpoint.current_story;
                }
                ResumeChoice::Discard => {
                    self.clear_checkpoint();
                    return None;
                }
                ResumeChoice::ViewDetails => {
                    self.display_checkpoint_details(&checkpoint);
                    // Continue the loop to prompt again
                }
            }
        }
    }

    /// Format a duration in a human-readable way.
    fn format_duration(duration: chrono::Duration) -> String {
        let total_seconds = duration.num_seconds().unsigned_abs();

        if total_seconds < 60 {
            format!("{} seconds ago", total_seconds)
        } else if total_seconds < 3600 {
            let minutes = total_seconds / 60;
            format!(
                "{} minute{} ago",
                minutes,
                if minutes == 1 { "" } else { "s" }
            )
        } else if total_seconds < 86400 {
            let hours = total_seconds / 3600;
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else {
            let days = total_seconds / 86400;
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        }
    }
}
