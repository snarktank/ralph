// Story execution engine for Ralph
// This module handles the actual execution of user stories including:
// - Spawning Claude Code or Amp CLI to implement stories
// - Running quality gates after implementation
// - Updating PRD files on success
// - Appending to progress.txt
// - Creating git commits

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, Mutex};

use crate::mcp::tools::load_prd::{PrdFile, PrdUserStory};
use crate::quality::{GateResult, Profile, QualityGateChecker};

/// Result of story execution
#[derive(Debug)]
pub struct ExecutionResult {
    /// Whether the story was successfully implemented
    pub success: bool,
    /// Git commit hash if a commit was created
    pub commit_hash: Option<String>,
    /// Error message if execution failed
    pub error: Option<String>,
    /// Number of iterations used
    pub iterations_used: u32,
    /// Quality gate results
    pub gate_results: Vec<GateResult>,
    /// Files that were changed
    pub files_changed: Vec<String>,
}

/// Error types for story execution
#[derive(Debug)]
pub enum ExecutorError {
    /// Story not found in PRD
    StoryNotFound(String),
    /// PRD file operation failed
    PrdError(String),
    /// Git operation failed
    GitError(String),
    /// Quality gates failed
    QualityGateFailed(String),
    /// Agent execution failed
    AgentError(String),
    /// Execution was cancelled
    Cancelled,
    /// IO error
    IoError(String),
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutorError::StoryNotFound(id) => write!(f, "Story not found: {}", id),
            ExecutorError::PrdError(msg) => write!(f, "PRD error: {}", msg),
            ExecutorError::GitError(msg) => write!(f, "Git error: {}", msg),
            ExecutorError::QualityGateFailed(msg) => write!(f, "Quality gate failed: {}", msg),
            ExecutorError::AgentError(msg) => write!(f, "Agent execution error: {}", msg),
            ExecutorError::Cancelled => write!(f, "Execution was cancelled"),
            ExecutorError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ExecutorError {}

/// Configuration for the story executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Path to the PRD file
    pub prd_path: PathBuf,
    /// Project root directory
    pub project_root: PathBuf,
    /// Path to progress.txt file
    pub progress_path: PathBuf,
    /// Quality profile for gate checking
    pub quality_profile: Option<Profile>,
    /// Agent command to use (e.g., "claude" or "amp")
    pub agent_command: String,
    /// Maximum iterations per story
    pub max_iterations: u32,
    /// Optional mutex for serializing git operations across parallel executions
    pub git_mutex: Option<Arc<Mutex<()>>>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            prd_path: PathBuf::from("prd.json"),
            project_root: PathBuf::from("."),
            progress_path: PathBuf::from("progress.txt"),
            quality_profile: None,
            agent_command: "claude".to_string(),
            max_iterations: 10,
            git_mutex: None,
        }
    }
}

/// Story executor that handles the end-to-end execution of user stories
pub struct StoryExecutor {
    config: ExecutorConfig,
}

impl StoryExecutor {
    /// Create a new story executor with the given configuration
    pub fn new(config: ExecutorConfig) -> Self {
        Self { config }
    }

    /// Execute a single story by ID
    ///
    /// This is the main entry point for story execution. It:
    /// 1. Loads the story from the PRD
    /// 2. Runs the agent to implement the story (with iteration loop)
    /// 3. Runs quality gates
    /// 4. Updates the PRD on success
    /// 5. Appends to progress.txt
    /// 6. Creates a git commit
    ///
    /// # Arguments
    ///
    /// * `story_id` - The ID of the story to execute
    /// * `cancel_receiver` - Watch channel to check for cancellation
    /// * `on_iteration` - Callback called after each iteration with (current, max)
    ///
    /// # Returns
    ///
    /// Result containing the execution result or an error
    pub async fn execute_story<F>(
        &self,
        story_id: &str,
        cancel_receiver: watch::Receiver<bool>,
        mut on_iteration: F,
    ) -> Result<ExecutionResult, ExecutorError>
    where
        F: FnMut(u32, u32),
    {
        // Load the PRD and find the story
        let prd = self.load_prd()?;
        let story = self.find_story(&prd, story_id)?;

        // Build the prompt for the agent
        let prompt = self.build_agent_prompt(story, &prd);

        let mut iterations_used = 0;
        let mut last_error: Option<String> = None;
        let mut files_changed: Vec<String>;

        // Iteration loop
        for iteration in 1..=self.config.max_iterations {
            iterations_used = iteration;
            on_iteration(iteration, self.config.max_iterations);

            // Check for cancellation
            if *cancel_receiver.borrow() {
                return Err(ExecutorError::Cancelled);
            }

            // Run the agent
            match self.run_agent(&prompt, iteration).await {
                Ok(changed) => {
                    files_changed = changed;
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    continue; // Try next iteration
                }
            }

            // Check for cancellation before quality gates
            if cancel_receiver.has_changed().unwrap_or(false) && *cancel_receiver.borrow() {
                return Err(ExecutorError::Cancelled);
            }

            // Run quality gates
            let gate_results = self.run_quality_gates();
            let all_passed = QualityGateChecker::all_passed(&gate_results);

            if all_passed {
                // Success! Create commit and update PRD
                let commit_hash = self.create_commit(story).await?;
                self.update_prd_passes(story_id)?;
                self.append_progress(story, &files_changed, iteration)?;

                return Ok(ExecutionResult {
                    success: true,
                    commit_hash: Some(commit_hash),
                    error: None,
                    iterations_used,
                    gate_results,
                    files_changed,
                });
            }

            // Quality gates failed, prepare error message for next iteration
            let failed_gates: Vec<&str> = gate_results
                .iter()
                .filter(|g| !g.passed)
                .map(|g| g.gate_name.as_str())
                .collect();
            last_error = Some(format!("Quality gates failed: {}", failed_gates.join(", ")));
        }

        // Max iterations reached without success
        Err(ExecutorError::AgentError(format!(
            "Failed after {} iterations. Last error: {}",
            iterations_used,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )))
    }

    /// Load the PRD file
    fn load_prd(&self) -> Result<PrdFile, ExecutorError> {
        let content = std::fs::read_to_string(&self.config.prd_path)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to read PRD: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to parse PRD: {}", e)))
    }

    /// Find a story by ID in the PRD
    fn find_story<'a>(
        &self,
        prd: &'a PrdFile,
        story_id: &str,
    ) -> Result<&'a PrdUserStory, ExecutorError> {
        prd.user_stories
            .iter()
            .find(|s| s.id == story_id)
            .ok_or_else(|| ExecutorError::StoryNotFound(story_id.to_string()))
    }

    /// Build the agent prompt for implementing a story
    fn build_agent_prompt(&self, story: &PrdUserStory, prd: &PrdFile) -> String {
        let mut prompt = format!("# Implement User Story: {} - {}\n\n", story.id, story.title);

        if !story.description.is_empty() {
            prompt.push_str(&format!("## Description\n{}\n\n", story.description));
        }

        if !story.acceptance_criteria.is_empty() {
            prompt.push_str("## Acceptance Criteria\n");
            for (i, criterion) in story.acceptance_criteria.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, criterion));
            }
            prompt.push('\n');
        }

        prompt.push_str(&format!(
            "## Project Context\n\
            - Project: {}\n\
            - Branch: {}\n\
            - Story Priority: {}\n\n\
            ## Instructions\n\
            1. Implement all acceptance criteria\n\
            2. Ensure code compiles without errors (cargo check)\n\
            3. Ensure no clippy warnings (cargo clippy -- -D warnings)\n\
            4. Ensure proper formatting (cargo fmt)\n\
            5. Keep changes focused and minimal\n",
            prd.project, prd.branch_name, story.priority
        ));

        prompt
    }

    /// Run the agent (Claude Code or Amp CLI) to implement the story
    async fn run_agent(&self, prompt: &str, iteration: u32) -> Result<Vec<String>, ExecutorError> {
        let agent_cmd = &self.config.agent_command;

        // Detect which agent to use
        let (program, args) = if agent_cmd == "claude" || agent_cmd.contains("claude") {
            // Claude Code CLI - use --print for non-interactive mode
            // and --dangerously-skip-permissions to allow file changes
            (
                "claude",
                vec!["--print", "--dangerously-skip-permissions", prompt],
            )
        } else if agent_cmd == "amp" || agent_cmd.contains("amp") {
            // Amp CLI
            ("amp", vec!["--prompt", prompt])
        } else {
            // Custom agent command
            (agent_cmd.as_str(), vec![prompt])
        };

        // Check if the agent is available (cross-platform)
        if !is_program_in_path(program) {
            return Err(ExecutorError::AgentError(format!(
                "Agent '{}' not found in PATH. Install Claude Code CLI or Amp CLI.",
                program
            )));
        }

        // Run the agent with the prompt
        let output = tokio::process::Command::new(program)
            .args(&args)
            .current_dir(&self.config.project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| ExecutorError::AgentError(format!("Failed to run {}: {}", program, e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExecutorError::AgentError(format!(
                "{} failed (iteration {}): {}",
                program, iteration, stderr
            )));
        }

        // Get list of changed files from git
        let files_changed = self.get_changed_files()?;
        Ok(files_changed)
    }

    /// Get the list of files changed according to git
    fn get_changed_files(&self) -> Result<Vec<String>, ExecutorError> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.config.project_root)
            .output()
            .map_err(|e| ExecutorError::GitError(format!("Failed to run git status: {}", e)))?;

        if !output.status.success() {
            return Err(ExecutorError::GitError("git status failed".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                // Git status format: "XY filename" where X and Y are status codes
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

    /// Run quality gates and return results
    fn run_quality_gates(&self) -> Vec<GateResult> {
        let profile = self.config.quality_profile.clone().unwrap_or_default();
        let checker = QualityGateChecker::new(profile, &self.config.project_root);
        checker.run_all()
    }

    /// Create a git commit with the proper format
    ///
    /// If a git_mutex is configured, this method will acquire the lock before
    /// performing git operations to prevent concurrent git operations that could
    /// corrupt the repository.
    async fn create_commit(&self, story: &PrdUserStory) -> Result<String, ExecutorError> {
        // Acquire git mutex if configured (for parallel execution)
        let _guard = if let Some(ref mutex) = self.config.git_mutex {
            Some(mutex.lock().await)
        } else {
            None
        };

        // Stage all changes
        let status = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.config.project_root)
            .status()
            .map_err(|e| ExecutorError::GitError(format!("Failed to stage changes: {}", e)))?;

        if !status.success() {
            return Err(ExecutorError::GitError("git add failed".to_string()));
        }

        // Create commit with proper message format: feat: [ID] - [Title]
        let commit_message = format!("feat: {} - {}", story.id, story.title);

        let status = Command::new("git")
            .args(["commit", "-m", &commit_message])
            .current_dir(&self.config.project_root)
            .status()
            .map_err(|e| ExecutorError::GitError(format!("Failed to create commit: {}", e)))?;

        if !status.success() {
            return Err(ExecutorError::GitError("git commit failed".to_string()));
        }

        // Get the commit hash
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.config.project_root)
            .output()
            .map_err(|e| ExecutorError::GitError(format!("Failed to get commit hash: {}", e)))?;

        if !output.status.success() {
            return Err(ExecutorError::GitError(
                "Failed to get commit hash".to_string(),
            ));
        }

        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(hash)
        // _guard is dropped here, releasing the mutex lock
    }

    /// Update the PRD file to set passes: true for the story
    fn update_prd_passes(&self, story_id: &str) -> Result<(), ExecutorError> {
        // Read the PRD as raw JSON to preserve structure
        let content = std::fs::read_to_string(&self.config.prd_path)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to read PRD: {}", e)))?;

        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to parse PRD: {}", e)))?;

        // Find and update the story
        if let Some(stories) = json.get_mut("userStories").and_then(|s| s.as_array_mut()) {
            for story in stories {
                if story.get("id").and_then(|id| id.as_str()) == Some(story_id) {
                    story["passes"] = serde_json::Value::Bool(true);
                    break;
                }
            }
        }

        // Write back with pretty formatting
        let updated_content = serde_json::to_string_pretty(&json)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to serialize PRD: {}", e)))?;

        std::fs::write(&self.config.prd_path, updated_content)
            .map_err(|e| ExecutorError::PrdError(format!("Failed to write PRD: {}", e)))?;

        Ok(())
    }

    /// Append progress entry to progress.txt
    fn append_progress(
        &self,
        story: &PrdUserStory,
        files_changed: &[String],
        iterations: u32,
    ) -> Result<(), ExecutorError> {
        use std::io::Write;

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M");

        let mut entry = format!(
            "\n## {} - {}\n\
            - **What was implemented**: {}\n\
            - **Files changed**:\n",
            timestamp, story.id, story.title
        );

        for file in files_changed.iter().take(20) {
            entry.push_str(&format!("  - {}\n", file));
        }
        if files_changed.len() > 20 {
            entry.push_str(&format!(
                "  - ... and {} more files\n",
                files_changed.len() - 20
            ));
        }

        entry.push_str(&format!(
            "- **Iterations used**: {}\n\
            - **Learnings for future iterations:**\n\
              - Story completed successfully via automated execution\n\
            ---\n",
            iterations
        ));

        // Append to progress file
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.progress_path)
            .map_err(|e| ExecutorError::IoError(format!("Failed to open progress file: {}", e)))?;

        file.write_all(entry.as_bytes())
            .map_err(|e| ExecutorError::IoError(format!("Failed to write progress: {}", e)))?;

        Ok(())
    }
}

/// Check if a program exists in PATH (cross-platform)
fn is_program_in_path(program: &str) -> bool {
    #[cfg(target_os = "windows")]
    let check_cmd = "where";
    #[cfg(not(target_os = "windows"))]
    let check_cmd = "which";

    Command::new(check_cmd)
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a specific agent CLI is available
pub fn is_agent_available(agent: &str) -> bool {
    is_program_in_path(agent)
}

/// Detect the best available agent CLI
pub fn detect_agent() -> Option<String> {
    // Prefer Claude Code, fall back to Amp
    if is_agent_available("claude") {
        Some("claude".to_string())
    } else if is_agent_available("amp") {
        Some("amp".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_prd() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "description": "Test PRD",
            "userStories": [
                {
                    "id": "US-001",
                    "title": "First story",
                    "description": "A test story",
                    "acceptanceCriteria": ["AC1", "AC2"],
                    "priority": 1,
                    "passes": false
                },
                {
                    "id": "US-002",
                    "title": "Second story",
                    "priority": 2,
                    "passes": true
                }
            ]
        }"#;
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.agent_command, "claude");
        assert_eq!(config.max_iterations, 10);
    }

    #[test]
    fn test_load_prd() {
        let prd_file = create_test_prd();
        let config = ExecutorConfig {
            prd_path: prd_file.path().to_path_buf(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        let prd = executor.load_prd().unwrap();
        assert_eq!(prd.project, "TestProject");
        assert_eq!(prd.user_stories.len(), 2);
    }

    #[test]
    fn test_find_story_success() {
        let prd_file = create_test_prd();
        let config = ExecutorConfig {
            prd_path: prd_file.path().to_path_buf(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        let prd = executor.load_prd().unwrap();
        let story = executor.find_story(&prd, "US-001").unwrap();
        assert_eq!(story.id, "US-001");
        assert_eq!(story.title, "First story");
    }

    #[test]
    fn test_find_story_not_found() {
        let prd_file = create_test_prd();
        let config = ExecutorConfig {
            prd_path: prd_file.path().to_path_buf(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        let prd = executor.load_prd().unwrap();
        let result = executor.find_story(&prd, "US-999");
        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutorError::StoryNotFound(id) => assert_eq!(id, "US-999"),
            _ => panic!("Expected StoryNotFound error"),
        }
    }

    #[test]
    fn test_build_agent_prompt() {
        let prd_file = create_test_prd();
        let config = ExecutorConfig {
            prd_path: prd_file.path().to_path_buf(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        let prd = executor.load_prd().unwrap();
        let story = executor.find_story(&prd, "US-001").unwrap();
        let prompt = executor.build_agent_prompt(story, &prd);

        assert!(prompt.contains("US-001"));
        assert!(prompt.contains("First story"));
        assert!(prompt.contains("AC1"));
        assert!(prompt.contains("AC2"));
        assert!(prompt.contains("TestProject"));
        assert!(prompt.contains("cargo check"));
    }

    #[test]
    fn test_executor_error_display() {
        assert!(ExecutorError::StoryNotFound("US-001".to_string())
            .to_string()
            .contains("US-001"));
        assert!(ExecutorError::Cancelled.to_string().contains("cancelled"));
        assert!(ExecutorError::GitError("test".to_string())
            .to_string()
            .contains("Git error"));
    }

    #[test]
    fn test_update_prd_passes() {
        let prd_file = create_test_prd();
        let prd_path = prd_file.path().to_path_buf();

        // Copy to a temp file we can modify
        let temp_dir = TempDir::new().unwrap();
        let test_prd_path = temp_dir.path().join("prd.json");
        std::fs::copy(&prd_path, &test_prd_path).unwrap();

        let config = ExecutorConfig {
            prd_path: test_prd_path.clone(),
            ..Default::default()
        };
        let executor = StoryExecutor::new(config);

        // Update US-001 to passes: true
        executor.update_prd_passes("US-001").unwrap();

        // Verify the change
        let content = std::fs::read_to_string(&test_prd_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        let stories = json.get("userStories").unwrap().as_array().unwrap();
        let us001 = stories
            .iter()
            .find(|s| s.get("id").unwrap() == "US-001")
            .unwrap();
        assert_eq!(us001.get("passes").unwrap(), &serde_json::Value::Bool(true));
    }

    #[test]
    fn test_detect_agent() {
        // This test may pass or fail depending on installed tools
        let agent = detect_agent();
        // Just verify it returns a valid option or None
        if let Some(a) = agent {
            assert!(a == "claude" || a == "amp");
        }
    }
}
