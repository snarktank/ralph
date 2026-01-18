// Terminal runner for Ralph
// This module implements the default "run all stories until complete" behavior

use std::path::PathBuf;
use tokio::sync::watch;

use crate::mcp::tools::executor::{detect_agent, ExecutorConfig, StoryExecutor};
use crate::mcp::tools::load_prd::{PrdFile, PrdUserStory};

/// Configuration for the runner
#[derive(Debug, Clone)]
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
    /// Quiet mode - suppress output
    pub quiet: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            prd_path: PathBuf::from("prd.json"),
            working_dir: PathBuf::from("."),
            max_iterations_per_story: 10,
            max_total_iterations: 0, // unlimited
            agent_command: None,
            quiet: false,
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
}

impl Runner {
    /// Create a new runner with the given configuration
    pub fn new(config: RunnerConfig) -> Self {
        Self { config }
    }

    /// Run all stories until all pass or an error occurs
    pub async fn run(&self) -> RunResult {
        let mut total_iterations: u32 = 0;

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

        // Check if all stories already pass
        let passing_count = prd.user_stories.iter().filter(|s| s.passes).count();
        if passing_count == total_stories {
            if !self.config.quiet {
                println!("All {} stories passed!", total_stories);
                println!("<promise>COMPLETE</promise>");
            }
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

        if !self.config.quiet {
            println!("Starting Ralph iteration loop...");
            println!("PRD: {}", self.config.prd_path.display());
            println!("Agent: {}", agent);
            println!("Stories: {}/{} passing", passing_count, total_stories);
            println!();
        }

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

            // Find next story to work on (highest priority where passes: false)
            let next_story = self.find_next_story(&prd);

            match next_story {
                None => {
                    // All stories pass!
                    if !self.config.quiet {
                        println!();
                        println!("All {} stories passed!", total_stories);
                        println!("<promise>COMPLETE</promise>");
                    }
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

                    if !self.config.quiet {
                        let passing = self.count_passing_stories().unwrap_or(0);
                        println!(
                            "=== Story: {} - {} (priority {}) ===",
                            story.id, story.title, story.priority
                        );
                        println!("Progress: {}/{} stories passing", passing, total_stories);
                    }

                    // Execute the story
                    let executor_config = ExecutorConfig {
                        prd_path: self.config.prd_path.clone(),
                        project_root: self.config.working_dir.clone(),
                        progress_path: self.config.working_dir.join("progress.txt"),
                        quality_profile: None,
                        agent_command: agent.clone(),
                        max_iterations: self.config.max_iterations_per_story,
                    };

                    let executor = StoryExecutor::new(executor_config);
                    let (_cancel_tx, cancel_rx) = watch::channel(false);

                    let story_id = story.id.clone();
                    let quiet = self.config.quiet;

                    let result = executor
                        .execute_story(&story_id, cancel_rx, |iter, max| {
                            if !quiet {
                                println!("  Iteration {}/{}", iter, max);
                            }
                        })
                        .await;

                    total_iterations += result.as_ref().map(|r| r.iterations_used).unwrap_or(1);

                    match result {
                        Ok(exec_result) => {
                            if exec_result.success {
                                if !self.config.quiet {
                                    println!(
                                        "  ✓ Story {} completed (commit: {})",
                                        story_id,
                                        exec_result.commit_hash.as_deref().unwrap_or("none")
                                    );
                                    println!();
                                }
                            } else if !self.config.quiet {
                                println!(
                                    "  ✗ Story {} failed: {}",
                                    story_id,
                                    exec_result.error.as_deref().unwrap_or("unknown")
                                );
                                println!("  Continuing to next story...");
                                println!();
                            }
                        }
                        Err(e) => {
                            if !self.config.quiet {
                                println!("  ✗ Story {} error: {}", story_id, e);
                                println!("  Continuing to next story...");
                                println!();
                            }
                            // Don't fail the whole run, just continue
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
}
