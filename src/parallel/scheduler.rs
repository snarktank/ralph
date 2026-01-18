//! Parallel execution scheduler

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{watch, RwLock, Semaphore};

use crate::mcp::tools::executor::{detect_agent, ExecutorConfig, StoryExecutor};
use crate::mcp::tools::load_prd::{validate_prd, PrdFile};
use crate::parallel::dependency::DependencyGraph;
use crate::runner::{RunResult, RunnerConfig};

/// Configuration options for parallel story execution.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ParallelRunnerConfig {
    /// Maximum number of stories to execute concurrently.
    pub max_concurrency: u32,
    /// Whether to automatically infer dependencies from file patterns.
    pub infer_dependencies: bool,
    /// Whether to fall back to sequential execution on errors.
    pub fallback_to_sequential: bool,
}

impl Default for ParallelRunnerConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 3,
            infer_dependencies: true,
            fallback_to_sequential: true,
        }
    }
}

/// Tracks execution state across parallel story executions.
///
/// This struct maintains the runtime state for the parallel scheduler,
/// including which stories are currently executing, which have completed,
/// which have failed, and which files are locked.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct ParallelExecutionState {
    /// Stories currently being executed, mapped by story ID.
    pub in_flight: HashSet<String>,
    /// Stories that have completed successfully, mapped by story ID.
    pub completed: HashSet<String>,
    /// Stories that have failed, mapped by story ID to error message.
    pub failed: HashMap<String, String>,
    /// Files currently locked by stories, mapped from file path to story ID.
    pub locked_files: HashMap<PathBuf, String>,
}

/// The main parallel runner that executes multiple stories concurrently.
///
/// This struct manages parallel story execution with concurrency limiting
/// via a semaphore and shared execution state protected by a read-write lock.
#[allow(dead_code)]
pub struct ParallelRunner {
    /// Configuration for parallel execution settings.
    config: ParallelRunnerConfig,
    /// Base runner configuration (paths, limits, etc.).
    base_config: RunnerConfig,
    /// Semaphore for limiting concurrent story executions.
    semaphore: Arc<Semaphore>,
    /// Shared execution state tracking in-flight, completed, and failed stories.
    execution_state: Arc<RwLock<ParallelExecutionState>>,
}

#[allow(dead_code)]
impl ParallelRunner {
    /// Create a new parallel runner with the given configurations.
    ///
    /// # Arguments
    /// * `config` - Parallel execution settings (concurrency, inference, fallback)
    /// * `base_config` - Base runner configuration (paths, iteration limits)
    pub fn new(config: ParallelRunnerConfig, base_config: RunnerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency as usize));
        let execution_state = Arc::new(RwLock::new(ParallelExecutionState::default()));

        Self {
            config,
            base_config,
            semaphore,
            execution_state,
        }
    }

    /// Run all stories in parallel until all pass or an error occurs.
    ///
    /// This method implements the main parallel execution loop:
    /// 1. Loads the PRD and builds a DependencyGraph
    /// 2. Optionally infers dependencies from target files
    /// 3. Spawns concurrent tasks for ready stories (limited by semaphore)
    /// 4. Waits for any task to complete and updates state
    /// 5. Repeats until all stories pass or cannot make progress
    pub async fn run(&self) -> RunResult {
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

        // Build dependency graph
        let mut graph = DependencyGraph::from_stories(&prd.user_stories);

        // Optionally infer dependencies from file patterns
        if self.config.infer_dependencies {
            graph.infer_dependencies();
        }

        // Validate graph for cycles
        if let Err(e) = graph.validate() {
            return RunResult {
                all_passed: false,
                stories_passed: 0,
                total_stories,
                total_iterations: 0,
                error: Some(format!("Invalid dependency graph: {}", e)),
            };
        }

        // Count already passing stories
        let initially_passing: HashSet<String> = prd
            .user_stories
            .iter()
            .filter(|s| s.passes)
            .map(|s| s.id.clone())
            .collect();

        // Initialize completed set with already passing stories
        {
            let mut state = self.execution_state.write().await;
            state.completed = initially_passing.clone();
        }

        // Detect agent
        let agent = match self.base_config.agent_command.clone().or_else(detect_agent) {
            Some(a) => a,
            None => {
                return RunResult {
                    all_passed: false,
                    stories_passed: initially_passing.len(),
                    total_stories,
                    total_iterations: 0,
                    error: Some("No agent found. Install Claude Code CLI or Amp CLI.".to_string()),
                };
            }
        };

        // Check if all stories already pass
        if initially_passing.len() == total_stories {
            return RunResult {
                all_passed: true,
                stories_passed: total_stories,
                total_stories,
                total_iterations: 0,
                error: None,
            };
        }

        let mut total_iterations: u32 = 0;

        // Main execution loop
        loop {
            // Get current state snapshot
            let state = self.execution_state.read().await;
            let completed = state.completed.clone();
            let in_flight = state.in_flight.clone();
            drop(state);

            // Get stories ready to execute (dependencies satisfied, not completed, not in flight)
            let ready_stories: Vec<String> = graph
                .get_ready_stories(&completed)
                .into_iter()
                .filter(|s| !in_flight.contains(&s.id))
                .map(|s| s.id.clone())
                .collect();

            // Check if we're done or stuck
            if ready_stories.is_empty() && in_flight.is_empty() {
                // No more stories to run and none in flight
                let state = self.execution_state.read().await;
                let stories_passed = state.completed.len();
                let has_failures = !state.failed.is_empty();
                drop(state);

                return RunResult {
                    all_passed: stories_passed == total_stories,
                    stories_passed,
                    total_stories,
                    total_iterations,
                    error: if has_failures {
                        Some("Some stories failed".to_string())
                    } else {
                        None
                    },
                };
            }

            // If no ready stories but some in flight, wait for them
            if ready_stories.is_empty() {
                // Wait for any in-flight task to complete
                tokio::task::yield_now().await;
                continue;
            }

            // Spawn tasks for ready stories (up to available semaphore permits)
            let mut handles = Vec::new();

            for story_id in ready_stories {
                // Try to acquire semaphore permit
                let permit = match self.semaphore.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => break, // No more permits available
                };

                // Mark story as in-flight
                {
                    let mut state = self.execution_state.write().await;
                    state.in_flight.insert(story_id.clone());
                }

                // Clone values for the spawned task
                let executor_config = ExecutorConfig {
                    prd_path: self.base_config.prd_path.clone(),
                    project_root: self.base_config.working_dir.clone(),
                    progress_path: self.base_config.working_dir.join("progress.txt"),
                    quality_profile: None,
                    agent_command: agent.clone(),
                    max_iterations: self.base_config.max_iterations_per_story,
                };

                let execution_state = self.execution_state.clone();
                let story_id_clone = story_id.clone();

                let handle = tokio::spawn(async move {
                    // Hold the permit until the task completes (RAII)
                    let _permit = permit;

                    let executor = StoryExecutor::new(executor_config);
                    let (_cancel_tx, cancel_rx) = watch::channel(false);

                    let result = executor
                        .execute_story(&story_id_clone, cancel_rx, |_iter, _max| {})
                        .await;

                    // Update state based on result
                    let mut state = execution_state.write().await;
                    state.in_flight.remove(&story_id_clone);

                    match result {
                        Ok(exec_result) if exec_result.success => {
                            state.completed.insert(story_id_clone.clone());
                            (story_id_clone, true, exec_result.iterations_used)
                        }
                        Ok(exec_result) => {
                            let error_msg = exec_result
                                .error
                                .unwrap_or_else(|| "Unknown error".to_string());
                            state.failed.insert(story_id_clone.clone(), error_msg);
                            (story_id_clone, false, exec_result.iterations_used)
                        }
                        Err(e) => {
                            state.failed.insert(story_id_clone.clone(), e.to_string());
                            (story_id_clone, false, 1)
                        }
                    }
                    // Permit is dropped here, releasing the semaphore slot
                });

                handles.push(handle);
            }

            // Wait for at least one task to complete
            if !handles.is_empty() {
                // Use select to wait for any task to complete
                let (result, _index, remaining) = futures::future::select_all(handles).await;

                if let Ok((_story_id, _success, iterations)) = result {
                    total_iterations += iterations;
                }

                // Let remaining tasks continue (they'll complete in future iterations)
                for handle in remaining {
                    // Spawn a task to await the remaining handles
                    tokio::spawn(async move {
                        // Just await the handle to let it complete; state is already updated inside the task
                        let _ = handle.await;
                    });
                }
            }
        }
    }

    /// Load the PRD file.
    fn load_prd(&self) -> Result<PrdFile, String> {
        validate_prd(&self.base_config.prd_path).map_err(|e| e.to_string())
    }
}
