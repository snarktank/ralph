//! Parallel execution scheduler

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{watch, Mutex, RwLock, Semaphore};

use crate::mcp::tools::executor::{detect_agent, ExecutorConfig, StoryExecutor};
use crate::mcp::tools::load_prd::{validate_prd, PrdFile};
use crate::parallel::dependency::{DependencyGraph, StoryNode};
use crate::parallel::reconcile::{ReconciliationEngine, ReconciliationIssue, ReconciliationResult};
use crate::runner::{RunResult, RunnerConfig};

/// Strategy for detecting conflicts between parallel story executions.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Detect conflicts based on file paths (target_files).
    /// Stories modifying the same files cannot run concurrently.
    #[default]
    FileBased,
    /// Detect conflicts based on entity references.
    /// Stories referencing the same entities cannot run concurrently.
    EntityBased,
    /// No conflict detection. All ready stories can run concurrently.
    None,
}

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
    /// Strategy for detecting conflicts between parallel stories.
    pub conflict_strategy: ConflictStrategy,
}

impl Default for ParallelRunnerConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 3,
            infer_dependencies: true,
            fallback_to_sequential: true,
            conflict_strategy: ConflictStrategy::default(),
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

impl ParallelExecutionState {
    /// Attempts to acquire locks on the given file patterns for a story.
    ///
    /// Returns `true` if all locks were acquired successfully, `false` if any
    /// file is already locked by another story. If acquisition fails, no locks
    /// are added (atomic behavior).
    ///
    /// # Arguments
    ///
    /// * `story_id` - The ID of the story requesting the locks
    /// * `target_files` - List of file patterns that the story will modify
    pub fn acquire_locks(&mut self, story_id: &str, target_files: &[String]) -> bool {
        // First, check if any file is already locked by another story
        for file_pattern in target_files {
            let path = PathBuf::from(file_pattern);
            if let Some(locking_story) = self.locked_files.get(&path) {
                if locking_story != story_id {
                    // File is locked by another story
                    return false;
                }
            }
        }

        // All files are available, acquire all locks
        for file_pattern in target_files {
            let path = PathBuf::from(file_pattern);
            self.locked_files.insert(path, story_id.to_string());
        }

        true
    }

    /// Releases all file locks held by a story.
    ///
    /// This should be called when a story completes (success or failure).
    ///
    /// # Arguments
    ///
    /// * `story_id` - The ID of the story releasing its locks
    pub fn release_locks(&mut self, story_id: &str) {
        self.locked_files
            .retain(|_path, locking_story| locking_story != story_id);
    }
}

/// Detects pre-execution conflicts between ready stories based on overlapping target files.
///
/// Returns a list of story ID pairs that conflict (have overlapping target_files).
/// The first element of each pair is always the lower-priority story (higher priority number).
fn detect_preexecution_conflicts(stories: &[StoryNode]) -> Vec<(String, String)> {
    let mut conflicts = Vec::new();

    for i in 0..stories.len() {
        for j in (i + 1)..stories.len() {
            let story_a = &stories[i];
            let story_b = &stories[j];

            // Check for overlapping target_files
            let files_a: HashSet<&String> = story_a.target_files.iter().collect();
            let files_b: HashSet<&String> = story_b.target_files.iter().collect();

            if !files_a.is_disjoint(&files_b) {
                // There is an overlap - determine which is lower priority
                // Lower priority number = higher priority
                if story_a.priority > story_b.priority {
                    // story_a has lower priority (higher number), story_b runs first
                    conflicts.push((story_a.id.clone(), story_b.id.clone()));
                } else if story_b.priority > story_a.priority {
                    // story_b has lower priority (higher number), story_a runs first
                    conflicts.push((story_b.id.clone(), story_a.id.clone()));
                } else {
                    // Same priority - use lexicographic order as tiebreaker
                    // Earlier ID runs first
                    if story_a.id > story_b.id {
                        conflicts.push((story_a.id.clone(), story_b.id.clone()));
                    } else {
                        conflicts.push((story_b.id.clone(), story_a.id.clone()));
                    }
                }
            }
        }
    }

    conflicts
}

/// Filters ready stories to avoid pre-execution conflicts.
///
/// When two stories have overlapping target_files, only the higher-priority story
/// (lower priority number) is included in this batch. The lower-priority story
/// is deferred to a subsequent batch.
///
/// Returns a tuple of (stories to run this batch, deferred story IDs for logging).
fn filter_conflicting_stories(stories: Vec<StoryNode>) -> (Vec<StoryNode>, Vec<(String, String)>) {
    if stories.is_empty() {
        return (stories, Vec::new());
    }

    let conflicts = detect_preexecution_conflicts(&stories);

    if conflicts.is_empty() {
        return (stories, Vec::new());
    }

    // Collect IDs of stories that should be deferred (lower-priority stories in conflicts)
    let deferred_ids: HashSet<String> = conflicts
        .iter()
        .map(|(deferred, _)| deferred.clone())
        .collect();

    // Filter out deferred stories
    let filtered: Vec<StoryNode> = stories
        .into_iter()
        .filter(|s| !deferred_ids.contains(&s.id))
        .collect();

    (filtered, conflicts)
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
    /// Mutex to serialize git operations across parallel stories.
    git_mutex: Arc<Mutex<()>>,
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
        let git_mutex = Arc::new(Mutex::new(()));

        Self {
            config,
            base_config,
            semaphore,
            execution_state,
            git_mutex,
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

        // Check if all stories already pass - no agent needed in this case
        if initially_passing.len() == total_stories {
            return RunResult {
                all_passed: true,
                stories_passed: total_stories,
                total_stories,
                total_iterations: 0,
                error: None,
            };
        }

        // Detect agent (only needed if there are failing stories)
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

        let mut total_iterations: u32 = 0;

        // Main execution loop
        loop {
            // Get current state snapshot
            let state = self.execution_state.read().await;
            let completed = state.completed.clone();
            let in_flight = state.in_flight.clone();
            drop(state);

            // Get stories ready to execute (dependencies satisfied, not completed, not in flight)
            // Keep the full StoryNode so we have access to target_files for locking
            let ready_stories: Vec<_> = graph
                .get_ready_stories(&completed)
                .into_iter()
                .filter(|s| !in_flight.contains(&s.id))
                .cloned()
                .collect();

            // Pre-execution conflict detection: filter out lower-priority stories
            // that have overlapping target_files with higher-priority stories
            let (ready_stories, conflicts) = filter_conflicting_stories(ready_stories);

            // Log when sequential fallback is triggered due to conflicts
            for (deferred_id, higher_priority_id) in &conflicts {
                eprintln!(
                    "[parallel] Conflict fallback: {} conflicts with {} over target files. \
                     Running {} first, {} deferred to next batch.",
                    deferred_id, higher_priority_id, higher_priority_id, deferred_id
                );
            }

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

            for story in ready_stories {
                let story_id = story.id.clone();
                let target_files = story.target_files.clone();

                // Try to acquire semaphore permit
                let permit = match self.semaphore.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => break, // No more permits available
                };

                // Try to acquire file locks; skip this story if files are locked
                {
                    let mut state = self.execution_state.write().await;
                    if !state.acquire_locks(&story_id, &target_files) {
                        // Files are locked by another story, skip for now
                        drop(permit); // Release the semaphore permit
                        continue;
                    }
                    // Mark story as in-flight
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
                    git_mutex: Some(self.git_mutex.clone()),
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
                    // Release file locks (success or failure)
                    state.release_locks(&story_id_clone);

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

            // Wait for all tasks in this batch to complete
            if !handles.is_empty() {
                let batch_story_ids: Vec<String> = {
                    let state = self.execution_state.read().await;
                    state.in_flight.iter().cloned().collect()
                };

                // Wait for all handles to complete
                let results = futures::future::join_all(handles).await;

                for (_story_id, _success, iterations) in results.into_iter().flatten() {
                    total_iterations += iterations;
                }

                // Run reconciliation after each batch completes
                let reconciliation_result = self
                    .run_reconciliation(&batch_story_ids, &graph, &agent, &mut total_iterations)
                    .await;

                // If reconciliation failed and we couldn't recover, return error
                if let Some(error) = reconciliation_result {
                    let state = self.execution_state.read().await;
                    return RunResult {
                        all_passed: false,
                        stories_passed: state.completed.len(),
                        total_stories,
                        total_iterations,
                        error: Some(error),
                    };
                }
            }
        }
    }

    /// Runs reconciliation after a batch completes and handles any issues found.
    ///
    /// Returns `None` if reconciliation passed or issues were resolved via sequential retry.
    /// Returns `Some(error)` if reconciliation found issues that couldn't be resolved.
    async fn run_reconciliation(
        &self,
        batch_story_ids: &[String],
        graph: &DependencyGraph,
        agent: &str,
        total_iterations: &mut u32,
    ) -> Option<String> {
        let engine = ReconciliationEngine::new(self.base_config.working_dir.clone());
        let result = engine.reconcile();

        match result {
            ReconciliationResult::Clean => {
                eprintln!("[parallel] Reconciliation: clean after batch");
                None
            }
            ReconciliationResult::IssuesFound(issues) => {
                // Log all issues found
                for issue in &issues {
                    match issue {
                        ReconciliationIssue::GitConflict { affected_files } => {
                            eprintln!(
                                "[parallel] Reconciliation issue: git conflict in files: {}",
                                affected_files.join(", ")
                            );
                        }
                        ReconciliationIssue::TypeMismatch { file, error } => {
                            eprintln!(
                                "[parallel] Reconciliation issue: type error in {}: {}",
                                file, error
                            );
                        }
                        ReconciliationIssue::ImportDuplicate => {
                            eprintln!("[parallel] Reconciliation issue: duplicate import detected");
                        }
                    }
                }

                // If fallback is enabled, retry affected stories sequentially
                if self.config.fallback_to_sequential {
                    eprintln!(
                        "[parallel] Fallback enabled: retrying {} stories sequentially",
                        batch_story_ids.len()
                    );

                    // Get affected stories - for now, we retry all stories from the batch
                    // that have issues (based on file overlap with issue files)
                    let affected_story_ids =
                        self.get_affected_stories(&issues, batch_story_ids, graph);

                    if !affected_story_ids.is_empty() {
                        // Remove affected stories from completed set so they can be retried
                        {
                            let mut state = self.execution_state.write().await;
                            for story_id in &affected_story_ids {
                                state.completed.remove(story_id);
                                state.failed.remove(story_id);
                            }
                        }

                        // Retry affected stories sequentially
                        for story_id in &affected_story_ids {
                            eprintln!("[parallel] Sequential retry: executing story {}", story_id);

                            let executor_config = ExecutorConfig {
                                prd_path: self.base_config.prd_path.clone(),
                                project_root: self.base_config.working_dir.clone(),
                                progress_path: self.base_config.working_dir.join("progress.txt"),
                                quality_profile: None,
                                agent_command: agent.to_string(),
                                max_iterations: self.base_config.max_iterations_per_story,
                                git_mutex: Some(self.git_mutex.clone()),
                            };

                            let executor = StoryExecutor::new(executor_config);
                            let (_cancel_tx, cancel_rx) = watch::channel(false);

                            let result = executor
                                .execute_story(story_id, cancel_rx, |_iter, _max| {})
                                .await;

                            match result {
                                Ok(exec_result) if exec_result.success => {
                                    let mut state = self.execution_state.write().await;
                                    state.completed.insert(story_id.clone());
                                    *total_iterations += exec_result.iterations_used;
                                    eprintln!(
                                        "[parallel] Sequential retry: story {} succeeded",
                                        story_id
                                    );
                                }
                                Ok(exec_result) => {
                                    let mut state = self.execution_state.write().await;
                                    let error_msg = exec_result
                                        .error
                                        .unwrap_or_else(|| "Unknown error".to_string());
                                    state.failed.insert(story_id.clone(), error_msg.clone());
                                    *total_iterations += exec_result.iterations_used;
                                    eprintln!(
                                        "[parallel] Sequential retry: story {} failed: {}",
                                        story_id, error_msg
                                    );
                                }
                                Err(e) => {
                                    let mut state = self.execution_state.write().await;
                                    state.failed.insert(story_id.clone(), e.to_string());
                                    *total_iterations += 1;
                                    eprintln!(
                                        "[parallel] Sequential retry: story {} error: {}",
                                        story_id, e
                                    );
                                }
                            }
                        }

                        // Run reconciliation again after sequential retry
                        let post_retry_result = engine.reconcile();
                        match post_retry_result {
                            ReconciliationResult::Clean => {
                                eprintln!(
                                    "[parallel] Reconciliation: clean after sequential retry"
                                );
                                None
                            }
                            ReconciliationResult::IssuesFound(remaining_issues) => {
                                eprintln!(
                                    "[parallel] Reconciliation: {} issues remain after sequential retry",
                                    remaining_issues.len()
                                );
                                Some(format!(
                                    "Reconciliation failed with {} issues after sequential retry",
                                    remaining_issues.len()
                                ))
                            }
                        }
                    } else {
                        // No affected stories identified, but issues exist
                        Some(format!(
                            "Reconciliation failed with {} issues",
                            issues.len()
                        ))
                    }
                } else {
                    // Fallback disabled, return error
                    Some(format!(
                        "Reconciliation failed with {} issues (fallback disabled)",
                        issues.len()
                    ))
                }
            }
        }
    }

    /// Identifies stories affected by reconciliation issues.
    ///
    /// Returns a list of story IDs that should be retried based on the issues found.
    fn get_affected_stories(
        &self,
        issues: &[ReconciliationIssue],
        batch_story_ids: &[String],
        graph: &DependencyGraph,
    ) -> Vec<String> {
        // Collect all affected files from issues
        let mut affected_files: HashSet<String> = HashSet::new();

        for issue in issues {
            match issue {
                ReconciliationIssue::GitConflict {
                    affected_files: files,
                } => {
                    affected_files.extend(files.iter().cloned());
                }
                ReconciliationIssue::TypeMismatch { file, .. } => {
                    if file != "unknown" {
                        affected_files.insert(file.clone());
                    }
                }
                ReconciliationIssue::ImportDuplicate => {
                    // For import duplicates, we can't easily determine which files are affected
                    // So we mark all batch stories as affected
                    return batch_story_ids.to_vec();
                }
            }
        }

        // If we couldn't identify specific files, retry all batch stories
        if affected_files.is_empty() {
            return batch_story_ids.to_vec();
        }

        // Find stories whose target_files overlap with affected files
        let mut affected_story_ids = Vec::new();

        for story_id in batch_story_ids {
            if let Some(story) = graph.get_story(story_id) {
                for target_file in &story.target_files {
                    // Check if any affected file matches or is contained in target_file pattern
                    for affected_file in &affected_files {
                        if target_file.contains(affected_file)
                            || affected_file.contains(target_file)
                        {
                            affected_story_ids.push(story_id.clone());
                            break;
                        }
                    }
                }
            }
        }

        // If we still couldn't match any stories, retry all
        if affected_story_ids.is_empty() {
            batch_story_ids.to_vec()
        } else {
            affected_story_ids
        }
    }

    /// Load the PRD file.
    fn load_prd(&self) -> Result<PrdFile, String> {
        validate_prd(&self.base_config.prd_path).map_err(|e| e.to_string())
    }
}
