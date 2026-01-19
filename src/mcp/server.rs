// MCP Server implementation for Ralph
// This module provides the core MCP server struct

#![allow(dead_code)]

use crate::audit::prd_converter::{PrdConverter, PrdConverterConfig};
use crate::audit::prd_generator::{PrdGenerator, PrdGeneratorConfig};
use crate::mcp::resources::{
    list_ralph_resources, read_prd_resource, read_status_resource, ResourceError, PRD_RESOURCE_URI,
    STATUS_RESOURCE_URI,
};
use crate::mcp::tools::audit::{
    all_sections, create_error_response as create_audit_error_response,
    create_generate_prd_error_response, create_generate_prd_success_response,
    create_results_error_response, create_results_success_response, create_status_error_response,
    create_status_success_response, create_success_response as create_audit_success_response,
    generate_audit_id, get_audit_status_from_state, resolve_audit_path, AuditOutputFormat,
    AuditState, AuditStatus, GeneratePrdFromAuditError, GeneratePrdFromAuditRequest,
    GetAuditResultsError, GetAuditResultsRequest, GetAuditStatusError, GetAuditStatusRequest,
    StartAuditError, StartAuditRequest,
};
use crate::mcp::tools::executor::{detect_agent, ExecutorConfig, StoryExecutor};
use crate::mcp::tools::get_status::{GetStatusRequest, GetStatusResponse};
use crate::mcp::tools::list_stories::{load_stories, ListStoriesRequest, ListStoriesResponse};
use crate::mcp::tools::load_prd::{
    create_error_response, create_success_response, validate_prd, LoadPrdRequest,
};
use crate::mcp::tools::run_story::{
    check_already_running, create_error_response as create_run_error_response,
    create_started_response, current_timestamp, find_story, RunStoryError, RunStoryRequest,
};
use crate::mcp::tools::stop_execution::{
    create_cancelled_response, create_not_running_response, get_running_story_id,
    state_description, StopExecutionRequest,
};
use crate::quality::QualityConfig;
use crate::ui::{DisplayOptions, RalphDisplay};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    Implementation, ListResourcesResult, PaginatedRequestParam, ReadResourceRequestParam,
    ReadResourceResult, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};

/// Execution state of the Ralph agent.
///
/// This enum tracks the current state of story execution,
/// allowing MCP clients to monitor progress and respond appropriately.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ExecutionState {
    /// No execution in progress
    #[default]
    Idle,
    /// Currently executing a story
    Running {
        /// The story ID being executed
        story_id: String,
        /// When execution started (Unix timestamp)
        started_at: u64,
        /// Current iteration number
        iteration: u32,
        /// Maximum iterations allowed
        max_iterations: u32,
    },
    /// Execution completed successfully
    Completed {
        /// The story ID that completed
        story_id: String,
        /// The git commit hash (if any)
        commit_hash: Option<String>,
    },
    /// Execution failed
    Failed {
        /// The story ID that failed
        story_id: String,
        /// Error message describing the failure
        error: String,
    },
    /// Execution paused (e.g., waiting for user input or external event)
    Paused {
        /// The story ID that is paused
        story_id: String,
        /// When execution was paused (Unix timestamp)
        paused_at: u64,
        /// Reason for the pause
        pause_reason: String,
    },
    /// Waiting for retry after a transient failure
    WaitingForRetry {
        /// The story ID waiting for retry
        story_id: String,
        /// When the retry will be attempted (Unix timestamp)
        retry_at: u64,
        /// Current attempt number
        attempt: u32,
        /// Maximum number of attempts allowed
        max_attempts: u32,
    },
}

/// Shared server state that can be accessed across async contexts.
#[derive(Debug)]
pub struct ServerState {
    /// Path to the currently loaded PRD file
    pub prd_path: Option<PathBuf>,
    /// Current execution state
    pub execution_state: ExecutionState,
    /// Audit states indexed by audit ID
    pub audit_states: HashMap<String, AuditState>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            prd_path: None,
            execution_state: ExecutionState::Idle,
            audit_states: HashMap::new(),
        }
    }
}

/// RalphMcpServer - The main MCP server struct for Ralph
///
/// This server exposes Ralph's functionality via the Model Context Protocol,
/// allowing AI assistants to interact with Ralph's PRD management, story execution,
/// and quality checking capabilities.
///
/// # Thread Safety
///
/// The server uses `Arc<RwLock<_>>` for shared state to allow safe concurrent access
/// from multiple MCP tools and async tasks.
#[derive(Clone)]
pub struct RalphMcpServer {
    /// Shared mutable state protected by RwLock
    state: Arc<RwLock<ServerState>>,
    /// Quality configuration for running quality gates
    config: Arc<Option<QualityConfig>>,
    /// Cancellation signal sender - send true to cancel execution
    cancel_sender: Arc<watch::Sender<bool>>,
    /// Cancellation signal receiver - tools check this to know if cancelled
    cancel_receiver: watch::Receiver<bool>,
    /// Tool router for MCP tools
    tool_router: ToolRouter<Self>,
    /// Display controller for terminal UI
    display: Arc<RwLock<RalphDisplay>>,
    /// Test-only: Override agent detection with a mock agent name
    #[cfg(test)]
    test_agent_override: Option<String>,
}

impl RalphMcpServer {
    /// Create a new RalphMcpServer instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use ralphmacchio::mcp::RalphMcpServer;
    ///
    /// let server = RalphMcpServer::new();
    /// ```
    pub fn new() -> Self {
        let (cancel_sender, cancel_receiver) = watch::channel(false);
        Self {
            state: Arc::new(RwLock::new(ServerState::default())),
            config: Arc::new(None),
            cancel_sender: Arc::new(cancel_sender),
            cancel_receiver,
            tool_router: Self::tool_router(),
            display: Arc::new(RwLock::new(RalphDisplay::new())),
            #[cfg(test)]
            test_agent_override: None,
        }
    }

    /// Create a new RalphMcpServer for testing with a mock agent.
    ///
    /// This bypasses the real agent detection and uses the provided agent name.
    #[cfg(test)]
    pub fn new_for_test(agent_name: &str) -> Self {
        let (cancel_sender, cancel_receiver) = watch::channel(false);
        Self {
            state: Arc::new(RwLock::new(ServerState::default())),
            config: Arc::new(None),
            cancel_sender: Arc::new(cancel_sender),
            cancel_receiver,
            tool_router: Self::tool_router(),
            display: Arc::new(RwLock::new(RalphDisplay::new())),
            test_agent_override: Some(agent_name.to_string()),
        }
    }

    /// Create a new RalphMcpServer with a preloaded PRD path.
    ///
    /// # Arguments
    ///
    /// * `prd_path` - Path to the PRD file to preload
    ///
    /// # Examples
    ///
    /// ```
    /// use ralphmacchio::mcp::RalphMcpServer;
    /// use std::path::PathBuf;
    ///
    /// let server = RalphMcpServer::with_prd(PathBuf::from("prd.json"));
    /// ```
    pub fn with_prd(prd_path: PathBuf) -> Self {
        let (cancel_sender, cancel_receiver) = watch::channel(false);
        Self {
            state: Arc::new(RwLock::new(ServerState {
                prd_path: Some(prd_path),
                execution_state: ExecutionState::Idle,
                audit_states: HashMap::new(),
            })),
            config: Arc::new(None),
            cancel_sender: Arc::new(cancel_sender),
            cancel_receiver,
            tool_router: Self::tool_router(),
            display: Arc::new(RwLock::new(RalphDisplay::new())),
            #[cfg(test)]
            test_agent_override: None,
        }
    }

    /// Create a new RalphMcpServer with quality configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Quality configuration for running quality gates
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ralphmacchio::mcp::RalphMcpServer;
    /// use ralphmacchio::quality::QualityConfig;
    ///
    /// let config = QualityConfig::load("quality/ralph-quality.toml").unwrap();
    /// let server = RalphMcpServer::with_config(config);
    /// ```
    pub fn with_config(config: QualityConfig) -> Self {
        let (cancel_sender, cancel_receiver) = watch::channel(false);
        Self {
            state: Arc::new(RwLock::new(ServerState::default())),
            config: Arc::new(Some(config)),
            cancel_sender: Arc::new(cancel_sender),
            cancel_receiver,
            tool_router: Self::tool_router(),
            display: Arc::new(RwLock::new(RalphDisplay::new())),
            #[cfg(test)]
            test_agent_override: None,
        }
    }

    /// Create a new RalphMcpServer with display options.
    ///
    /// # Arguments
    ///
    /// * `options` - Display options for configuring terminal UI behavior
    ///
    /// # Examples
    ///
    /// ```
    /// use ralphmacchio::mcp::RalphMcpServer;
    /// use ralphmacchio::ui::{DisplayOptions, UiMode};
    ///
    /// let options = DisplayOptions::new()
    ///     .with_ui_mode(UiMode::Enabled)
    ///     .with_color(true)
    ///     .with_quiet(false);
    /// let server = RalphMcpServer::with_display(options);
    /// ```
    pub fn with_display(options: DisplayOptions) -> Self {
        let (cancel_sender, cancel_receiver) = watch::channel(false);
        Self {
            state: Arc::new(RwLock::new(ServerState::default())),
            config: Arc::new(None),
            cancel_sender: Arc::new(cancel_sender),
            cancel_receiver,
            tool_router: Self::tool_router(),
            display: Arc::new(RwLock::new(RalphDisplay::with_options(options))),
            #[cfg(test)]
            test_agent_override: None,
        }
    }

    /// Create a new RalphMcpServer with a preloaded PRD and display options.
    ///
    /// # Arguments
    ///
    /// * `prd_path` - Path to the PRD file to preload
    /// * `options` - Display options for configuring terminal UI behavior
    ///
    /// # Examples
    ///
    /// ```
    /// use ralphmacchio::mcp::RalphMcpServer;
    /// use ralphmacchio::ui::{DisplayOptions, UiMode};
    /// use std::path::PathBuf;
    ///
    /// let options = DisplayOptions::new().with_quiet(true);
    /// let server = RalphMcpServer::with_prd_and_display(PathBuf::from("prd.json"), options);
    /// ```
    pub fn with_prd_and_display(prd_path: PathBuf, options: DisplayOptions) -> Self {
        let (cancel_sender, cancel_receiver) = watch::channel(false);
        Self {
            state: Arc::new(RwLock::new(ServerState {
                prd_path: Some(prd_path),
                execution_state: ExecutionState::Idle,
                audit_states: HashMap::new(),
            })),
            config: Arc::new(None),
            cancel_sender: Arc::new(cancel_sender),
            cancel_receiver,
            tool_router: Self::tool_router(),
            display: Arc::new(RwLock::new(RalphDisplay::with_options(options))),
            #[cfg(test)]
            test_agent_override: None,
        }
    }

    /// Get read access to the shared state.
    ///
    /// Returns a read guard that provides immutable access to the server state.
    pub async fn state(&self) -> tokio::sync::RwLockReadGuard<'_, ServerState> {
        self.state.read().await
    }

    /// Get write access to the shared state.
    ///
    /// Returns a write guard that provides mutable access to the server state.
    pub async fn state_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, ServerState> {
        self.state.write().await
    }

    /// Get the quality configuration.
    pub fn config(&self) -> Option<&QualityConfig> {
        self.config.as_ref().as_ref()
    }

    /// Signal cancellation of the current execution.
    ///
    /// This sends a cancellation signal that can be checked by running tasks.
    /// The signal is a watch channel, so all receivers will be notified.
    pub fn cancel(&self) {
        let _ = self.cancel_sender.send(true);
    }

    /// Reset the cancellation signal.
    ///
    /// This should be called before starting a new execution to clear
    /// any previous cancellation state.
    pub fn reset_cancel(&self) {
        let _ = self.cancel_sender.send(false);
    }

    /// Check if cancellation has been requested.
    ///
    /// Returns true if `cancel()` has been called since the last `reset_cancel()`.
    pub fn is_cancelled(&self) -> bool {
        *self.cancel_receiver.borrow()
    }

    /// Get a clone of the cancellation receiver.
    ///
    /// This can be passed to async tasks that need to check for cancellation.
    pub fn cancel_receiver(&self) -> watch::Receiver<bool> {
        self.cancel_receiver.clone()
    }

    /// Get read access to the display controller.
    ///
    /// Returns a read guard that provides immutable access to the display.
    pub async fn display(&self) -> tokio::sync::RwLockReadGuard<'_, RalphDisplay> {
        self.display.read().await
    }

    /// Get write access to the display controller.
    ///
    /// Returns a write guard that provides mutable access to the display.
    pub async fn display_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, RalphDisplay> {
        self.display.write().await
    }

    /// Update the display based on the current execution state.
    ///
    /// This method reads the current execution state and updates the UI accordingly.
    /// Call this after state transitions to keep the terminal display in sync.
    pub async fn update_display(&self) {
        let state = {
            let server_state = self.state.read().await;
            server_state.execution_state.clone()
        };

        let mut display = self.display.write().await;
        display.update_from_state(&state, None);
    }

    /// Update the display with story information.
    ///
    /// This method reads the current execution state and updates the UI,
    /// including displaying story details when available.
    pub async fn update_display_with_story(&self, story_info: Option<&crate::ui::StoryInfo>) {
        let state = {
            let server_state = self.state.read().await;
            server_state.execution_state.clone()
        };

        let mut display = self.display.write().await;
        display.update_from_state(&state, story_info);
    }
}

impl Default for RalphMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP tool implementations for RalphMcpServer.
///
/// This impl block contains all the MCP tools exposed by the server.
/// Tools are registered using the `#[tool]` attribute macro from rmcp.
#[tool_router]
impl RalphMcpServer {
    /// List stories from the loaded PRD.
    ///
    /// This tool returns a list of user stories from the currently loaded PRD file.
    /// Stories can be filtered by their pass/fail status using the optional status_filter parameter.
    ///
    /// # Parameters
    ///
    /// * `status_filter` - Optional filter: "passing" for stories where passes=true,
    ///   "failing" for stories where passes=false, or omit for all stories.
    ///
    /// # Returns
    ///
    /// JSON object containing:
    /// - `stories`: Array of {id, title, passes} objects
    /// - `count`: Total number of stories returned
    ///
    /// # Errors
    ///
    /// Returns an error message if:
    /// - No PRD is loaded
    /// - The PRD file cannot be read
    /// - The PRD JSON is invalid
    #[tool(
        name = "list_stories",
        description = "List user stories from the loaded PRD. Returns an array of story objects with id, title, and passes status. Optionally filter by 'passing' or 'failing' status."
    )]
    pub async fn list_stories(&self, Parameters(req): Parameters<ListStoriesRequest>) -> String {
        // Get the PRD path from state
        let prd_path = {
            let state = self.state.read().await;
            state.prd_path.clone()
        };

        match prd_path {
            Some(path) => {
                match load_stories(&path, req.status_filter.as_deref()) {
                    Ok(response) => {
                        // Serialize the response to JSON
                        serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                            format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                        })
                    }
                    Err(e) => {
                        format!("{{\"error\": \"{}\"}}", e)
                    }
                }
            }
            None => r#"{"error": "No PRD loaded. Use load_prd tool to load a PRD file first."}"#
                .to_string(),
        }
    }

    /// Get the current execution status of Ralph.
    ///
    /// This tool returns the current state of story execution, including:
    /// - Whether Ralph is idle, running, completed, or failed
    /// - For running state: story ID, start time, iteration progress
    /// - For completed state: story ID and commit hash
    /// - For failed state: story ID and error message
    ///
    /// # Returns
    ///
    /// JSON object containing:
    /// - `state`: Current state ("idle", "running", "completed", "failed")
    /// - `story_id`: ID of the story being processed (if applicable)
    /// - `started_at`: Unix timestamp when execution started (for running state)
    /// - `iteration`: Current iteration number (for running state)
    /// - `max_iterations`: Maximum iterations allowed (for running state)
    /// - `progress_percent`: Progress percentage (for running state)
    /// - `commit_hash`: Git commit hash (for completed state)
    /// - `error`: Error message (for failed state)
    #[tool(
        name = "get_status",
        description = "Get the current execution status of Ralph. Returns state (idle, running, completed, failed) along with progress info for running tasks and results for completed/failed tasks."
    )]
    pub async fn get_status(&self, Parameters(_req): Parameters<GetStatusRequest>) -> String {
        // Get the execution state from server state
        let execution_state = {
            let state = self.state.read().await;
            state.execution_state.clone()
        };

        // Convert ExecutionState to GetStatusResponse
        let response = GetStatusResponse::from_execution_state(&execution_state);

        // Serialize to JSON
        serde_json::to_string_pretty(&response)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize response: {}\"}}", e))
    }

    /// Load a PRD file into the Ralph MCP server.
    ///
    /// This tool loads a PRD JSON file from the specified path, validates its structure,
    /// and makes it available for other tools like list_stories and run_story.
    ///
    /// # Parameters
    ///
    /// * `path` - Path to the PRD JSON file to load. Can be absolute or relative.
    ///
    /// # Returns
    ///
    /// JSON object containing:
    /// - `success`: Whether the PRD was loaded successfully
    /// - `story_count`: Number of user stories in the PRD (if successful)
    /// - `project`: Project name from the PRD (if successful)
    /// - `branch_name`: Branch name from the PRD (if successful)
    /// - `message`: Success or error message
    ///
    /// # Errors
    ///
    /// Returns an error message if:
    /// - The file does not exist
    /// - The file cannot be read
    /// - The JSON is invalid
    /// - The PRD structure is invalid (missing required fields)
    #[tool(
        name = "load_prd",
        description = "Load a PRD file into Ralph. Validates the PRD JSON structure and returns story count on success, or an error message on failure. The PRD must have project, branchName, and userStories fields."
    )]
    pub async fn load_prd(&self, Parameters(req): Parameters<LoadPrdRequest>) -> String {
        // Convert path string to PathBuf
        let path = std::path::PathBuf::from(&req.path);

        // Canonicalize the path to handle relative paths
        let canonical_path = if path.is_absolute() {
            path.clone()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&path))
                .unwrap_or(path.clone())
        };

        // Validate the PRD file
        match validate_prd(&canonical_path) {
            Ok(prd) => {
                // Update server state with the new PRD path
                {
                    let mut state = self.state.write().await;
                    state.prd_path = Some(canonical_path);
                }

                // Create success response
                let response = create_success_response(&prd);
                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                })
            }
            Err(e) => {
                // Create error response
                let response = create_error_response(&e);
                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                })
            }
        }
    }

    /// Execute a user story from the loaded PRD.
    ///
    /// This tool starts execution of a specified story. It prevents concurrent execution -
    /// only one story can be running at a time. Use stop_execution to cancel a running story.
    ///
    /// # Parameters
    ///
    /// * `story_id` - The ID of the story to execute (e.g., "US-001")
    /// * `max_iterations` - Optional maximum number of iterations (default: 10)
    ///
    /// # Returns
    ///
    /// JSON object containing:
    /// - `success`: Whether execution started successfully
    /// - `story_id`: The story ID being executed
    /// - `story_title`: The story title
    /// - `commit_hash`: Git commit hash (if completed successfully)
    /// - `message`: Status message
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No PRD is loaded
    /// - Story ID not found in PRD
    /// - Another story is already executing
    #[tool(
        name = "run_story",
        description = "Execute a user story from the loaded PRD. Accepts story_id and optional max_iterations. Returns error if no PRD loaded, story not found, or another story is already running."
    )]
    pub async fn run_story(&self, Parameters(req): Parameters<RunStoryRequest>) -> String {
        let max_iterations = req.max_iterations.unwrap_or(10);

        // Get the PRD path and current execution state
        let (prd_path, current_state) = {
            let state = self.state.read().await;
            (state.prd_path.clone(), state.execution_state.clone())
        };

        // Check if a PRD is loaded
        let prd_path = match prd_path {
            Some(path) => path,
            None => {
                let response = create_run_error_response(&RunStoryError::NoPrdLoaded);
                return serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                });
            }
        };

        // Check if already running
        if let Some(running_id) = check_already_running(&current_state) {
            let response = create_run_error_response(&RunStoryError::AlreadyRunning(running_id));
            return serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            });
        }

        // Find the story in the PRD
        let story = match find_story(&prd_path, &req.story_id) {
            Ok(story) => story,
            Err(e) => {
                let response = create_run_error_response(&e);
                return serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                });
            }
        };

        // Reset cancellation and update state to Running
        self.reset_cancel();
        {
            let mut state = self.state.write().await;
            state.execution_state = ExecutionState::Running {
                story_id: req.story_id.clone(),
                started_at: current_timestamp(),
                iteration: 1,
                max_iterations,
            };
        }

        // Detect available agent (use test override if available)
        #[cfg(test)]
        let detected_agent = self.test_agent_override.clone().or_else(detect_agent);
        #[cfg(not(test))]
        let detected_agent = detect_agent();

        let agent_command = match detected_agent {
            Some(agent) => agent,
            None => {
                // Reset state to idle since we can't run
                {
                    let mut state = self.state.write().await;
                    state.execution_state = ExecutionState::Idle;
                }
                let response = create_run_error_response(&RunStoryError::ExecutionError(
                    "No agent CLI found. Install Claude Code CLI (claude) or Amp CLI (amp)."
                        .to_string(),
                ));
                return serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                });
            }
        };

        // Get project root from PRD path
        let project_root = prd_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Progress file path (in same directory as PRD)
        let progress_path = project_root.join("progress.txt");

        // Create executor config
        let executor_config = ExecutorConfig {
            prd_path: prd_path.clone(),
            project_root: project_root.clone(),
            progress_path,
            quality_profile: self
                .config
                .as_ref()
                .as_ref()
                .map(|c| c.profiles.get("standard").cloned().unwrap_or_default()),
            agent_command,
            max_iterations,
            git_mutex: None, // MCP server executes single story at a time
            timeout_config: crate::timeout::TimeoutConfig::default(),
            ..Default::default()
        };

        // Clone necessary data for the spawned task
        let story_id = req.story_id.clone();
        let state = self.state.clone();
        let cancel_receiver = self.cancel_receiver();
        let display = self.display.clone();

        // Spawn async task to execute the story
        tokio::spawn(async move {
            let executor = StoryExecutor::new(executor_config);

            // Iteration callback to update state
            let state_for_callback = state.clone();
            let on_iteration = move |iteration: u32, _max: u32| {
                let state_clone = state_for_callback.clone();
                tokio::spawn(async move {
                    let mut server_state = state_clone.write().await;
                    if let ExecutionState::Running {
                        iteration: iter, ..
                    } = &mut server_state.execution_state
                    {
                        *iter = iteration;
                    }
                });
            };

            // Execute the story
            match executor
                .execute_story(&story_id, cancel_receiver, on_iteration)
                .await
            {
                Ok(result) => {
                    // Update state to Completed
                    let mut server_state = state.write().await;
                    server_state.execution_state = ExecutionState::Completed {
                        story_id: story_id.clone(),
                        commit_hash: result.commit_hash,
                    };

                    // Update display
                    drop(server_state);
                    if let Ok(mut disp) = display.try_write() {
                        let completed_state = ExecutionState::Completed {
                            story_id: story_id.clone(),
                            commit_hash: None,
                        };
                        disp.update_from_state(&completed_state, None);
                    }
                }
                Err(e) => {
                    // Update state to Failed
                    let mut server_state = state.write().await;
                    server_state.execution_state = ExecutionState::Failed {
                        story_id: story_id.clone(),
                        error: e.to_string(),
                    };

                    // Update display
                    drop(server_state);
                    if let Ok(mut disp) = display.try_write() {
                        let failed_state = ExecutionState::Failed {
                            story_id: story_id.clone(),
                            error: e.to_string(),
                        };
                        disp.update_from_state(&failed_state, None);
                    }
                }
            }
        });

        // Return started response immediately (execution continues in background)
        let response = create_started_response(&story, max_iterations);
        serde_json::to_string_pretty(&response)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize response: {}\"}}", e))
    }

    /// Stop the currently executing story.
    ///
    /// This tool sends a cancellation signal to stop the currently running story execution.
    /// If no execution is in progress, it returns a message indicating the current state.
    ///
    /// # Returns
    ///
    /// JSON object containing:
    /// - `success`: Always true (the operation itself succeeded)
    /// - `was_running`: Whether an execution was actually cancelled
    /// - `story_id`: The ID of the story that was cancelled (if any)
    /// - `message`: Description of what happened
    ///
    /// # Notes
    ///
    /// The cancellation is cooperative - the executing task will stop at the next safe point
    /// (e.g., between iterations). It does not forcibly terminate the execution.
    #[tool(
        name = "stop_execution",
        description = "Stop the currently executing story. Sends a cancellation signal that will stop execution at the next safe point. Returns confirmation whether anything was running."
    )]
    pub async fn stop_execution(
        &self,
        Parameters(_req): Parameters<StopExecutionRequest>,
    ) -> String {
        // Get the current execution state
        let current_state = {
            let state = self.state.read().await;
            state.execution_state.clone()
        };

        // Check if anything is running
        if let Some(story_id) = get_running_story_id(&current_state) {
            // Signal cancellation
            self.cancel();

            // Create response indicating cancellation was sent
            let response = create_cancelled_response(&story_id);
            serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            })
        } else {
            // Nothing running - return appropriate message
            let state_desc = state_description(&current_state);
            let response = create_not_running_response(state_desc);
            serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
            })
        }
    }

    /// Start a codebase audit.
    ///
    /// This tool initiates an audit of the codebase, analyzing various aspects
    /// such as file structure, dependencies, architecture patterns, and more.
    ///
    /// # Parameters
    ///
    /// * `path` - Optional path to the directory to audit. Defaults to the PRD directory
    ///   if a PRD is loaded, otherwise uses the current working directory.
    /// * `sections` - Optional list of sections to analyze. If not provided, all sections
    ///   are analyzed. Valid sections: inventory, dependencies, architecture, testing,
    ///   documentation, api, tech_debt, opportunities.
    /// * `format` - Optional output format: json (default), markdown, or agent_context.
    ///
    /// # Returns
    ///
    /// JSON object containing:
    /// - `success`: Whether the audit was started successfully
    /// - `audit_id`: Unique identifier for checking audit status
    /// - `path`: The directory being audited
    /// - `sections`: List of sections being analyzed
    /// - `format`: The output format
    /// - `message`: Status message
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The specified path does not exist
    /// - The specified path is not a directory
    /// - Audit initialization fails
    #[tool(
        name = "start_audit",
        description = "Start a codebase audit. Analyzes file structure, dependencies, architecture patterns, testing, documentation, and identifies opportunities. Returns an audit_id for status checking."
    )]
    pub async fn start_audit(&self, Parameters(req): Parameters<StartAuditRequest>) -> String {
        // Get the PRD path from state for path resolution
        let prd_path = {
            let state = self.state.read().await;
            state.prd_path.clone()
        };

        // Resolve the audit path
        let audit_path = match resolve_audit_path(req.path.as_deref(), prd_path.as_ref()) {
            Ok(path) => path,
            Err(e) => {
                let response = create_audit_error_response(&e);
                return serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                });
            }
        };

        // Determine sections to analyze
        let sections = req.sections.unwrap_or_else(all_sections);

        // Determine output format
        let format = req.format.unwrap_or(AuditOutputFormat::Json);

        // Generate audit ID
        let audit_id = generate_audit_id();

        // Create audit state
        let audit_state = AuditState {
            audit_id: audit_id.clone(),
            path: audit_path,
            sections,
            format,
            started_at: crate::mcp::tools::audit::current_timestamp(),
            completed: false,
            error: None,
            progress: 0,
            report: None,
        };

        // Store the audit state
        {
            let mut state = self.state.write().await;
            state
                .audit_states
                .insert(audit_id.clone(), audit_state.clone());
        }

        // Create success response
        let response = create_audit_success_response(&audit_state);
        serde_json::to_string_pretty(&response)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize response: {}\"}}", e))
    }

    /// Get the status of a codebase audit.
    ///
    /// This tool returns the current status and progress of an audit started with start_audit.
    ///
    /// # Parameters
    ///
    /// * `audit_id` - The audit ID returned from start_audit.
    ///
    /// # Returns
    ///
    /// JSON object containing:
    /// - `success`: Whether the request was successful
    /// - `audit_id`: The audit ID
    /// - `status`: Current status: pending, running, completed, failed
    /// - `progress`: Progress percentage (0-100) if running
    /// - `error`: Error message if failed
    /// - `message`: Status message
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The audit ID is not found
    #[tool(
        name = "get_audit_status",
        description = "Get the status of a codebase audit. Returns status (pending, running, completed, failed) and progress percentage if running."
    )]
    pub async fn get_audit_status(
        &self,
        Parameters(req): Parameters<GetAuditStatusRequest>,
    ) -> String {
        // Get the audit state from server state
        let audit_state = {
            let state = self.state.read().await;
            state.audit_states.get(&req.audit_id).cloned()
        };

        match audit_state {
            Some(state) => {
                let response = create_status_success_response(&state);
                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                })
            }
            None => {
                let error = GetAuditStatusError::AuditNotFound(req.audit_id);
                let response = create_status_error_response(&error);
                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                })
            }
        }
    }

    /// Get the results of a completed codebase audit.
    ///
    /// This tool returns the full audit report for a completed audit.
    /// The audit must be complete before results can be retrieved.
    ///
    /// # Parameters
    ///
    /// * `audit_id` - The audit ID returned from start_audit.
    ///
    /// # Returns
    ///
    /// JSON object containing:
    /// - `success`: Whether the request was successful
    /// - `audit_id`: The audit ID
    /// - `report`: The full AuditReport object (if completed)
    /// - `error`: Error message if failed
    /// - `message`: Status message
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The audit ID is not found
    /// - The audit is not yet complete (pending or running)
    /// - The audit failed
    #[tool(
        name = "get_audit_results",
        description = "Get the results of a completed codebase audit. Returns the full AuditReport as JSON. Returns an error if the audit is not complete."
    )]
    pub async fn get_audit_results(
        &self,
        Parameters(req): Parameters<GetAuditResultsRequest>,
    ) -> String {
        // Get the audit state from server state
        let audit_state = {
            let state = self.state.read().await;
            state.audit_states.get(&req.audit_id).cloned()
        };

        match audit_state {
            Some(state) => {
                let status = get_audit_status_from_state(&state);

                match status {
                    AuditStatus::Completed => {
                        // Return the report if available
                        match state.report {
                            Some(report) => {
                                let response =
                                    create_results_success_response(&state.audit_id, report);
                                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                                    format!(
                                        "{{\"error\": \"Failed to serialize response: {}\"}}",
                                        e
                                    )
                                })
                            }
                            None => {
                                // Audit completed but no report (shouldn't happen normally)
                                let error = GetAuditResultsError::AuditFailed(
                                    req.audit_id,
                                    "Audit completed but no report available".to_string(),
                                );
                                let response = create_results_error_response(&error);
                                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                                    format!(
                                        "{{\"error\": \"Failed to serialize response: {}\"}}",
                                        e
                                    )
                                })
                            }
                        }
                    }
                    AuditStatus::Failed => {
                        let error = GetAuditResultsError::AuditFailed(
                            req.audit_id,
                            state.error.unwrap_or_else(|| "Unknown error".to_string()),
                        );
                        let response = create_results_error_response(&error);
                        serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                            format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                        })
                    }
                    _ => {
                        // Audit is pending or running
                        let error = GetAuditResultsError::AuditNotComplete(req.audit_id, status);
                        let response = create_results_error_response(&error);
                        serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                            format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                        })
                    }
                }
            }
            None => {
                let error = GetAuditResultsError::AuditNotFound(req.audit_id);
                let response = create_results_error_response(&error);
                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                })
            }
        }
    }

    /// Generate a PRD from completed audit results.
    ///
    /// This tool generates a Product Requirements Document (PRD) in both markdown
    /// and prd.json formats from a completed audit. The generated PRD can be used
    /// directly with Ralph to implement the improvements identified by the audit.
    ///
    /// # Parameters
    ///
    /// * `audit_id` - The audit ID returned from start_audit. The audit must be completed.
    /// * `user_answers` - Optional user answers from the interactive Q&A session.
    /// * `project_name` - Optional project name override.
    /// * `output_dir` - Optional output directory for generated files.
    ///
    /// # Returns
    ///
    /// JSON object containing:
    /// - `success`: Whether the generation was successful
    /// - `audit_id`: The audit ID
    /// - `prd_markdown_path`: Path to the generated PRD markdown file
    /// - `prd_json_path`: Path to the generated prd.json file
    /// - `story_count`: Number of user stories generated
    /// - `error`: Error message if failed
    /// - `message`: Status message
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The audit ID is not found
    /// - The audit is not yet complete (pending or running)
    /// - The audit failed
    /// - PRD generation fails
    /// - PRD conversion fails
    #[tool(
        name = "generate_prd_from_audit",
        description = "Generate a PRD from completed audit results. Creates both markdown and prd.json files that can be used with Ralph to implement improvements. The audit must be completed before calling this tool."
    )]
    pub async fn generate_prd_from_audit(
        &self,
        Parameters(req): Parameters<GeneratePrdFromAuditRequest>,
    ) -> String {
        // Get the audit state from server state
        let audit_state = {
            let state = self.state.read().await;
            state.audit_states.get(&req.audit_id).cloned()
        };

        match audit_state {
            Some(state) => {
                let status = get_audit_status_from_state(&state);

                match status {
                    AuditStatus::Completed => {
                        // Get the report
                        match state.report {
                            Some(report) => {
                                // Determine output directory
                                let output_dir = req
                                    .output_dir
                                    .map(PathBuf::from)
                                    .unwrap_or_else(|| state.path.clone());

                                // Create tasks directory if it doesn't exist
                                let tasks_dir = output_dir.join("tasks");

                                // Configure PRD generator
                                let mut generator_config = PrdGeneratorConfig::new()
                                    .with_skip_prompt(true)
                                    .with_output_dir(tasks_dir.clone());

                                if let Some(project_name) = req.project_name.clone() {
                                    generator_config =
                                        generator_config.with_project_name(project_name);
                                }

                                // Generate PRD markdown
                                let generator = PrdGenerator::with_config(generator_config);
                                let prd_result = match generator.generate(&report) {
                                    Ok(result) => result,
                                    Err(e) => {
                                        let error = GeneratePrdFromAuditError::GenerationFailed(
                                            e.to_string(),
                                        );
                                        let response = create_generate_prd_error_response(&error);
                                        return serde_json::to_string_pretty(&response)
                                            .unwrap_or_else(|e| {
                                                format!(
                                                "{{\"error\": \"Failed to serialize response: {}\"}}",
                                                e
                                            )
                                            });
                                    }
                                };

                                // Configure PRD converter
                                let mut converter_config = PrdConverterConfig::new()
                                    .with_skip_prompt(true)
                                    .with_output_dir(output_dir.clone());

                                if let Some(project_name) = req.project_name {
                                    converter_config =
                                        converter_config.with_project_name(project_name);
                                }

                                // Convert PRD to prd.json
                                let converter = PrdConverter::with_config(converter_config);
                                let conversion_result = match converter
                                    .convert(&prd_result.prd_path)
                                {
                                    Ok(result) => result,
                                    Err(e) => {
                                        let error = GeneratePrdFromAuditError::ConversionFailed(
                                            e.to_string(),
                                        );
                                        let response = create_generate_prd_error_response(&error);
                                        return serde_json::to_string_pretty(&response)
                                                .unwrap_or_else(|e| {
                                                    format!(
                                                "{{\"error\": \"Failed to serialize response: {}\"}}",
                                                e
                                            )
                                                });
                                    }
                                };

                                // Create success response
                                let response = create_generate_prd_success_response(
                                    &req.audit_id,
                                    &prd_result.prd_path,
                                    &conversion_result.prd_json_path,
                                    conversion_result.story_count,
                                );
                                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                                    format!(
                                        "{{\"error\": \"Failed to serialize response: {}\"}}",
                                        e
                                    )
                                })
                            }
                            None => {
                                // Audit completed but no report
                                let error = GeneratePrdFromAuditError::AuditFailed(
                                    req.audit_id,
                                    "Audit completed but no report available".to_string(),
                                );
                                let response = create_generate_prd_error_response(&error);
                                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                                    format!(
                                        "{{\"error\": \"Failed to serialize response: {}\"}}",
                                        e
                                    )
                                })
                            }
                        }
                    }
                    AuditStatus::Failed => {
                        let error = GeneratePrdFromAuditError::AuditFailed(
                            req.audit_id,
                            state.error.unwrap_or_else(|| "Unknown error".to_string()),
                        );
                        let response = create_generate_prd_error_response(&error);
                        serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                            format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                        })
                    }
                    _ => {
                        // Audit is pending or running
                        let error =
                            GeneratePrdFromAuditError::AuditNotComplete(req.audit_id, status);
                        let response = create_generate_prd_error_response(&error);
                        serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                            format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                        })
                    }
                }
            }
            None => {
                let error = GeneratePrdFromAuditError::AuditNotFound(req.audit_id);
                let response = create_generate_prd_error_response(&error);
                serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize response: {}\"}}", e)
                })
            }
        }
    }
}

/// Implementation of the MCP ServerHandler trait for RalphMcpServer.
///
/// This implementation provides the server information including name, version,
/// and enabled capabilities (tools and resources).
#[tool_handler(router = self.tool_router)]
impl ServerHandler for RalphMcpServer {
    /// Returns server information for MCP initialization.
    ///
    /// The returned `ServerInfo` includes:
    /// - Server name: "ralph"
    /// - Version: from Cargo.toml (CARGO_PKG_VERSION)
    /// - Capabilities: tools and resources enabled
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "ralph".to_string(),
                title: Some("Ralph Autonomous Agent".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Ralph is an autonomous AI agent framework for executing PRD-based user stories."
                    .to_string(),
            ),
        }
    }

    /// List available resources.
    ///
    /// Returns the list of resources that can be accessed via MCP:
    /// - `ralph://prd/current` - The currently loaded PRD file contents
    /// - `ralph://status` - The current execution status
    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        std::future::ready(Ok(list_ralph_resources()))
    }

    /// Read a specific resource by URI.
    ///
    /// Supported URIs:
    /// - `ralph://prd/current` - Returns the contents of the currently loaded PRD file as JSON
    /// - `ralph://status` - Returns the current execution status as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URI is not recognized
    /// - For `ralph://prd/current`: no PRD is loaded or the file cannot be read
    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        let state = self.state.clone();

        async move {
            let server_state = state.read().await;

            match request.uri.as_str() {
                PRD_RESOURCE_URI => match read_prd_resource(&server_state.prd_path) {
                    Ok(contents) => Ok(ReadResourceResult {
                        contents: vec![contents],
                    }),
                    Err(ResourceError::NoPrdLoaded) => Err(McpError::invalid_request(
                        "No PRD loaded. Use the load_prd tool to load a PRD first.",
                        None,
                    )),
                    Err(ResourceError::PrdReadError(msg)) => Err(McpError::invalid_request(
                        format!("Failed to read PRD: {}", msg),
                        None,
                    )),
                    Err(ResourceError::UnknownResource(uri)) => Err(McpError::invalid_request(
                        format!("Unknown resource: {}", uri),
                        None,
                    )),
                },
                STATUS_RESOURCE_URI => {
                    let contents = read_status_resource(&server_state.execution_state);
                    Ok(ReadResourceResult {
                        contents: vec![contents],
                    })
                }
                _ => Err(McpError::invalid_request(
                    format!("Unknown resource URI: {}. Available resources: ralph://prd/current, ralph://status", request.uri),
                    None,
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_state_default() {
        let state = ExecutionState::default();
        assert_eq!(state, ExecutionState::Idle);
    }

    #[test]
    fn test_execution_state_running() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 1,
            max_iterations: 10,
        };
        match state {
            ExecutionState::Running {
                story_id,
                iteration,
                ..
            } => {
                assert_eq!(story_id, "US-001");
                assert_eq!(iteration, 1);
            }
            _ => panic!("Expected Running state"),
        }
    }

    #[test]
    fn test_execution_state_completed() {
        let state = ExecutionState::Completed {
            story_id: "US-001".to_string(),
            commit_hash: Some("abc123".to_string()),
        };
        match state {
            ExecutionState::Completed {
                story_id,
                commit_hash,
            } => {
                assert_eq!(story_id, "US-001");
                assert_eq!(commit_hash, Some("abc123".to_string()));
            }
            _ => panic!("Expected Completed state"),
        }
    }

    #[test]
    fn test_execution_state_failed() {
        let state = ExecutionState::Failed {
            story_id: "US-001".to_string(),
            error: "Test failure".to_string(),
        };
        match state {
            ExecutionState::Failed { story_id, error } => {
                assert_eq!(story_id, "US-001");
                assert_eq!(error, "Test failure");
            }
            _ => panic!("Expected Failed state"),
        }
    }

    #[test]
    fn test_server_state_default() {
        let state = ServerState::default();
        assert!(state.prd_path.is_none());
        assert_eq!(state.execution_state, ExecutionState::Idle);
    }

    #[test]
    fn test_ralph_mcp_server_new() {
        let server = RalphMcpServer::new();
        assert!(!server.is_cancelled());
    }

    #[test]
    fn test_ralph_mcp_server_with_prd() {
        let server = RalphMcpServer::with_prd(PathBuf::from("test.json"));
        assert!(!server.is_cancelled());
    }

    #[test]
    fn test_ralph_mcp_server_cancel() {
        let server = RalphMcpServer::new();
        assert!(!server.is_cancelled());

        server.cancel();
        assert!(server.is_cancelled());

        server.reset_cancel();
        assert!(!server.is_cancelled());
    }

    #[test]
    fn test_ralph_mcp_server_clone() {
        let server = RalphMcpServer::new();
        let cloned = server.clone();

        // Both should share the same cancel state
        server.cancel();
        assert!(cloned.is_cancelled());
    }

    #[tokio::test]
    async fn test_ralph_mcp_server_state_access() {
        let server = RalphMcpServer::new();

        // Test read access
        {
            let state = server.state().await;
            assert!(state.prd_path.is_none());
            assert_eq!(state.execution_state, ExecutionState::Idle);
        }

        // Test write access
        {
            let mut state = server.state_mut().await;
            state.prd_path = Some(PathBuf::from("test.json"));
        }

        // Verify the change persisted
        {
            let state = server.state().await;
            assert_eq!(state.prd_path, Some(PathBuf::from("test.json")));
        }
    }

    #[tokio::test]
    async fn test_ralph_mcp_server_state_mutation() {
        let server = RalphMcpServer::new();

        {
            let mut state = server.state_mut().await;
            state.execution_state = ExecutionState::Running {
                story_id: "US-001".to_string(),
                started_at: 1234567890,
                iteration: 1,
                max_iterations: 10,
            };
        }

        {
            let state = server.state().await;
            match &state.execution_state {
                ExecutionState::Running { story_id, .. } => {
                    assert_eq!(story_id, "US-001");
                }
                _ => panic!("Expected Running state"),
            }
        }
    }

    #[test]
    fn test_server_handler_get_info() {
        let server = RalphMcpServer::new();
        let info = server.get_info();

        // Check server name
        assert_eq!(info.server_info.name, "ralph");

        // Check version is set (matches Cargo.toml version)
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));

        // Check title is set
        assert_eq!(
            info.server_info.title,
            Some("Ralph Autonomous Agent".to_string())
        );

        // Check instructions are set
        assert!(info.instructions.is_some());
        assert!(info
            .instructions
            .as_ref()
            .unwrap()
            .contains("autonomous AI agent"));
    }

    #[test]
    fn test_server_handler_capabilities() {
        let server = RalphMcpServer::new();
        let info = server.get_info();

        // Check tools capability is enabled
        assert!(info.capabilities.tools.is_some());

        // Check resources capability is enabled
        assert!(info.capabilities.resources.is_some());
    }

    #[tokio::test]
    async fn test_get_status_idle() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();
        let result = server.get_status(Parameters(GetStatusRequest {})).await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["state"], "idle");
        assert!(json.get("story_id").is_none());
    }

    #[tokio::test]
    async fn test_get_status_running() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        // Set the state to running
        {
            let mut state = server.state_mut().await;
            state.execution_state = ExecutionState::Running {
                story_id: "US-001".to_string(),
                started_at: 1234567890,
                iteration: 5,
                max_iterations: 10,
            };
        }

        let result = server.get_status(Parameters(GetStatusRequest {})).await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["state"], "running");
        assert_eq!(json["story_id"], "US-001");
        assert_eq!(json["started_at"], 1234567890);
        assert_eq!(json["iteration"], 5);
        assert_eq!(json["max_iterations"], 10);
        assert_eq!(json["progress_percent"], 50);
    }

    #[tokio::test]
    async fn test_get_status_completed() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        // Set the state to completed
        {
            let mut state = server.state_mut().await;
            state.execution_state = ExecutionState::Completed {
                story_id: "US-001".to_string(),
                commit_hash: Some("abc123def456".to_string()),
            };
        }

        let result = server.get_status(Parameters(GetStatusRequest {})).await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["state"], "completed");
        assert_eq!(json["story_id"], "US-001");
        assert_eq!(json["commit_hash"], "abc123def456");
        assert_eq!(json["progress_percent"], 100);
    }

    #[tokio::test]
    async fn test_get_status_failed() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        // Set the state to failed
        {
            let mut state = server.state_mut().await;
            state.execution_state = ExecutionState::Failed {
                story_id: "US-001".to_string(),
                error: "Build failed: syntax error".to_string(),
            };
        }

        let result = server.get_status(Parameters(GetStatusRequest {})).await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["state"], "failed");
        assert_eq!(json["story_id"], "US-001");
        assert_eq!(json["error"], "Build failed: syntax error");
    }

    #[tokio::test]
    async fn test_load_prd_success() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new();

        // Create a valid PRD file
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "description": "Test PRD",
            "userStories": [
                {"id": "US-001", "title": "First story", "priority": 1, "passes": false}
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        let result = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["story_count"], 1);
        assert_eq!(json["project"], "TestProject");
        assert_eq!(json["branch_name"], "feature/test");
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("Successfully loaded"));

        // Verify the PRD path was set in server state
        let state = server.state().await;
        assert!(state.prd_path.is_some());
    }

    #[tokio::test]
    async fn test_load_prd_file_not_found() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        let result = server
            .load_prd(Parameters(LoadPrdRequest {
                path: "/nonexistent/path/to/prd.json".to_string(),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("not found"));

        // Verify the PRD path was NOT set in server state
        let state = server.state().await;
        assert!(state.prd_path.is_none());
    }

    #[tokio::test]
    async fn test_load_prd_invalid_json() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new();

        // Create an invalid JSON file
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"not valid json").unwrap();

        let result = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("parse"));

        // Verify the PRD path was NOT set in server state
        let state = server.state().await;
        assert!(state.prd_path.is_none());
    }

    #[tokio::test]
    async fn test_load_prd_invalid_structure() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new();

        // Create a JSON file with invalid PRD structure (empty userStories)
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "project": "Test",
            "branchName": "main",
            "userStories": []
        }"#;
        file.write_all(content.as_bytes()).unwrap();

        let result = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("No user stories"));
    }

    #[tokio::test]
    async fn test_load_prd_updates_state() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new();

        // Verify initial state has no PRD
        {
            let state = server.state().await;
            assert!(state.prd_path.is_none());
        }

        // Create a valid PRD file
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "userStories": [
                {"id": "US-001", "title": "Test", "priority": 1, "passes": false}
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        // Load the PRD
        let _ = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Verify state was updated
        {
            let state = server.state().await;
            assert!(state.prd_path.is_some());
            // The path should contain our temp file path
            let prd_path = state.prd_path.as_ref().unwrap();
            assert!(prd_path.exists());
        }
    }

    #[tokio::test]
    async fn test_run_story_no_prd_loaded() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        let result = server
            .run_story(Parameters(RunStoryRequest {
                story_id: "US-001".to_string(),
                max_iterations: None,
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("No PRD loaded"));
    }

    #[tokio::test]
    async fn test_run_story_story_not_found() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new();

        // Create and load a valid PRD file
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "userStories": [
                {"id": "US-001", "title": "First story", "priority": 1, "passes": false}
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        // Load the PRD
        let _ = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Try to run a non-existent story
        let result = server
            .run_story(Parameters(RunStoryRequest {
                story_id: "US-999".to_string(),
                max_iterations: None,
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("not found"));
        assert!(json["message"].as_str().unwrap().contains("US-999"));
    }

    #[tokio::test]
    async fn test_run_story_already_running() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new();

        // Create and load a valid PRD file
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "userStories": [
                {"id": "US-001", "title": "First story", "priority": 1, "passes": false},
                {"id": "US-002", "title": "Second story", "priority": 2, "passes": false}
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        // Load the PRD
        let _ = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Set state to already running
        {
            let mut state = server.state_mut().await;
            state.execution_state = ExecutionState::Running {
                story_id: "US-001".to_string(),
                started_at: 1234567890,
                iteration: 5,
                max_iterations: 10,
            };
        }

        // Try to run another story
        let result = server
            .run_story(Parameters(RunStoryRequest {
                story_id: "US-002".to_string(),
                max_iterations: None,
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("Already executing"));
        assert!(json["message"].as_str().unwrap().contains("US-001"));
    }

    #[tokio::test]
    async fn test_run_story_success() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new_for_test("mock-agent");

        // Create and load a valid PRD file
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "userStories": [
                {"id": "US-001", "title": "First story", "priority": 1, "passes": false}
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        // Load the PRD
        let _ = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Run the story
        let result = server
            .run_story(Parameters(RunStoryRequest {
                story_id: "US-001".to_string(),
                max_iterations: Some(5),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["story_id"], "US-001");
        assert_eq!(json["story_title"], "First story");
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("Started execution"));
        assert!(json["message"].as_str().unwrap().contains("5 iterations"));
    }

    #[tokio::test]
    async fn test_run_story_updates_state() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new_for_test("mock-agent");

        // Create and load a valid PRD file
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "userStories": [
                {"id": "US-001", "title": "First story", "priority": 1, "passes": false}
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        // Load the PRD
        let _ = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Verify initial state is idle
        {
            let state = server.state().await;
            assert_eq!(state.execution_state, ExecutionState::Idle);
        }

        // Run the story
        let _ = server
            .run_story(Parameters(RunStoryRequest {
                story_id: "US-001".to_string(),
                max_iterations: Some(10),
            }))
            .await;

        // Verify state changed to running
        {
            let state = server.state().await;
            match &state.execution_state {
                ExecutionState::Running {
                    story_id,
                    max_iterations,
                    iteration,
                    ..
                } => {
                    assert_eq!(story_id, "US-001");
                    assert_eq!(*max_iterations, 10);
                    assert_eq!(*iteration, 1);
                }
                _ => panic!("Expected Running state"),
            }
        }
    }

    #[tokio::test]
    async fn test_run_story_default_max_iterations() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new_for_test("mock-agent");

        // Create and load a valid PRD file
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "userStories": [
                {"id": "US-001", "title": "First story", "priority": 1, "passes": false}
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        // Load the PRD
        let _ = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Run the story without specifying max_iterations
        let result = server
            .run_story(Parameters(RunStoryRequest {
                story_id: "US-001".to_string(),
                max_iterations: None,
            }))
            .await;

        // Parse the result as JSON - should default to 10 iterations
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert!(json["message"].as_str().unwrap().contains("10 iterations"));

        // Verify state has default max_iterations
        {
            let state = server.state().await;
            match &state.execution_state {
                ExecutionState::Running { max_iterations, .. } => {
                    assert_eq!(*max_iterations, 10);
                }
                _ => panic!("Expected Running state"),
            }
        }
    }

    #[tokio::test]
    async fn test_run_story_resets_cancel() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let server = RalphMcpServer::new_for_test("mock-agent");

        // Create and load a valid PRD file
        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{
            "project": "TestProject",
            "branchName": "feature/test",
            "userStories": [
                {"id": "US-001", "title": "First story", "priority": 1, "passes": false}
            ]
        }"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        // Load the PRD
        let _ = server
            .load_prd(Parameters(LoadPrdRequest {
                path: file.path().to_string_lossy().to_string(),
            }))
            .await;

        // Set cancel flag
        server.cancel();
        assert!(server.is_cancelled());

        // Run the story - should reset cancel
        let _ = server
            .run_story(Parameters(RunStoryRequest {
                story_id: "US-001".to_string(),
                max_iterations: None,
            }))
            .await;

        // Verify cancel was reset
        assert!(!server.is_cancelled());
    }

    #[tokio::test]
    async fn test_stop_execution_nothing_running() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        let result = server
            .stop_execution(Parameters(StopExecutionRequest {}))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["was_running"], false);
        assert!(json.get("story_id").is_none());
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("No execution in progress"));
        assert!(json["message"].as_str().unwrap().contains("idle"));
    }

    #[tokio::test]
    async fn test_stop_execution_while_running() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        // Set state to running
        {
            let mut state = server.state_mut().await;
            state.execution_state = ExecutionState::Running {
                story_id: "US-001".to_string(),
                started_at: 1234567890,
                iteration: 5,
                max_iterations: 10,
            };
        }

        // Verify not cancelled initially
        assert!(!server.is_cancelled());

        let result = server
            .stop_execution(Parameters(StopExecutionRequest {}))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["was_running"], true);
        assert_eq!(json["story_id"], "US-001");
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("Cancellation signal sent"));
        assert!(json["message"].as_str().unwrap().contains("US-001"));

        // Verify cancel signal was sent
        assert!(server.is_cancelled());
    }

    #[tokio::test]
    async fn test_stop_execution_after_completed() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        // Set state to completed
        {
            let mut state = server.state_mut().await;
            state.execution_state = ExecutionState::Completed {
                story_id: "US-001".to_string(),
                commit_hash: Some("abc123".to_string()),
            };
        }

        let result = server
            .stop_execution(Parameters(StopExecutionRequest {}))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["was_running"], false);
        assert!(json["message"].as_str().unwrap().contains("completed"));
    }

    #[tokio::test]
    async fn test_stop_execution_after_failed() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        // Set state to failed
        {
            let mut state = server.state_mut().await;
            state.execution_state = ExecutionState::Failed {
                story_id: "US-001".to_string(),
                error: "Build failed".to_string(),
            };
        }

        let result = server
            .stop_execution(Parameters(StopExecutionRequest {}))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["was_running"], false);
        assert!(json["message"].as_str().unwrap().contains("failed"));
    }

    #[tokio::test]
    async fn test_stop_execution_sets_cancel_flag() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        // Set state to running
        {
            let mut state = server.state_mut().await;
            state.execution_state = ExecutionState::Running {
                story_id: "US-002".to_string(),
                started_at: 1234567890,
                iteration: 3,
                max_iterations: 10,
            };
        }

        // Initially not cancelled
        assert!(!server.is_cancelled());

        // Stop execution
        let _ = server
            .stop_execution(Parameters(StopExecutionRequest {}))
            .await;

        // Cancel flag should now be set
        assert!(server.is_cancelled());

        // Clone should also see the cancel flag (shared state)
        let cloned = server.clone();
        assert!(cloned.is_cancelled());
    }

    // Note: Tests for list_resources and read_resource functionality
    // are in src/mcp/resources/mod.rs, which tests the helper functions directly.
    // Testing the ServerHandler trait methods would require constructing RequestContext,
    // which uses private rmcp internals. The helper functions provide comprehensive coverage.

    #[test]
    fn test_list_resources_helper() {
        // Test the helper function directly
        let resources = list_ralph_resources();
        assert_eq!(resources.resources.len(), 2);

        // Check PRD resource
        assert_eq!(resources.resources[0].raw.uri, "ralph://prd/current");
        assert_eq!(resources.resources[0].raw.name, "prd");
        assert_eq!(
            resources.resources[0].raw.mime_type,
            Some("application/json".to_string())
        );

        // Check status resource
        assert_eq!(resources.resources[1].raw.uri, "ralph://status");
        assert_eq!(resources.resources[1].raw.name, "status");
        assert_eq!(
            resources.resources[1].raw.mime_type,
            Some("application/json".to_string())
        );
    }

    #[test]
    fn test_read_status_resource_idle() {
        let state = ExecutionState::Idle;
        let contents = read_status_resource(&state);

        match contents {
            rmcp::model::ResourceContents::TextResourceContents {
                uri,
                mime_type,
                text,
                ..
            } => {
                assert_eq!(uri, "ralph://status");
                assert_eq!(mime_type, Some("application/json".to_string()));
                let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(json["state"], "idle");
            }
            _ => panic!("Expected TextResourceContents"),
        }
    }

    #[test]
    fn test_read_status_resource_running() {
        let state = ExecutionState::Running {
            story_id: "US-001".to_string(),
            started_at: 1234567890,
            iteration: 5,
            max_iterations: 10,
        };
        let contents = read_status_resource(&state);

        match contents {
            rmcp::model::ResourceContents::TextResourceContents { text, .. } => {
                let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(json["state"], "running");
                assert_eq!(json["story_id"], "US-001");
                assert_eq!(json["progress_percent"], 50);
            }
            _ => panic!("Expected TextResourceContents"),
        }
    }

    #[test]
    fn test_read_prd_resource_no_prd() {
        let result = read_prd_resource(&None);
        assert!(matches!(result, Err(ResourceError::NoPrdLoaded)));
    }

    #[test]
    fn test_read_prd_resource_success() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        let prd_content = r#"{"project": "Test", "branchName": "main", "userStories": []}"#;
        file.write_all(prd_content.as_bytes()).unwrap();

        let result = read_prd_resource(&Some(file.path().to_path_buf()));
        assert!(result.is_ok());

        match result.unwrap() {
            rmcp::model::ResourceContents::TextResourceContents {
                uri,
                mime_type,
                text,
                ..
            } => {
                assert_eq!(uri, "ralph://prd/current");
                assert_eq!(mime_type, Some("application/json".to_string()));
                assert_eq!(text, prd_content);
            }
            _ => panic!("Expected TextResourceContents"),
        }
    }

    #[tokio::test]
    async fn test_start_audit_with_path() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        let result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: None,
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert!(json["audit_id"].as_str().unwrap().starts_with("audit-"));
        assert!(json["path"]
            .as_str()
            .unwrap()
            .contains(temp_dir.path().to_str().unwrap()));
        assert!(json["sections"].is_array());
        assert_eq!(json["format"], "json");
    }

    #[tokio::test]
    async fn test_start_audit_with_sections() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        use crate::mcp::tools::audit::AuditSection;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        let result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: Some(vec![AuditSection::Inventory, AuditSection::Dependencies]),
                format: None,
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        let sections = json["sections"].as_array().unwrap();
        assert_eq!(sections.len(), 2);
        assert!(sections.contains(&serde_json::json!("inventory")));
        assert!(sections.contains(&serde_json::json!("dependencies")));
    }

    #[tokio::test]
    async fn test_start_audit_with_format() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        use crate::mcp::tools::audit::AuditOutputFormat;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        let result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: Some(AuditOutputFormat::Markdown),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["format"], "markdown");
    }

    #[tokio::test]
    async fn test_start_audit_invalid_path() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        let result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some("/nonexistent/path/to/directory".to_string()),
                sections: None,
                format: None,
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn test_start_audit_uses_prd_directory() {
        use rmcp::handler::server::wrapper::Parameters;
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let prd_path = temp_dir.path().join("prd.json");

        // Create a PRD file
        let mut file = std::fs::File::create(&prd_path).unwrap();
        file.write_all(
            br#"{"project": "Test", "branchName": "main", "userStories": [{"id": "US-001", "title": "Test", "priority": 1, "passes": false}]}"#,
        )
        .unwrap();

        // Create server with PRD
        let server = RalphMcpServer::with_prd(prd_path);

        // Start audit without specifying path
        let result = server
            .start_audit(Parameters(StartAuditRequest {
                path: None,
                sections: None,
                format: None,
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        // Path should be the temp directory (parent of PRD)
        assert!(json["path"]
            .as_str()
            .unwrap()
            .contains(temp_dir.path().to_str().unwrap()));
    }

    #[tokio::test]
    async fn test_start_audit_fallback_to_cwd() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        // Start audit without path or PRD - should use current directory
        let result = server
            .start_audit(Parameters(StartAuditRequest {
                path: None,
                sections: None,
                format: None,
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        // Should use current working directory
        let cwd = std::env::current_dir().unwrap();
        assert!(json["path"]
            .as_str()
            .unwrap()
            .contains(cwd.to_str().unwrap()));
    }

    #[tokio::test]
    async fn test_start_audit_all_sections_by_default() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        let result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: None,
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        let sections = json["sections"].as_array().unwrap();
        // Should have all 8 sections
        assert_eq!(sections.len(), 8);
    }

    #[tokio::test]
    async fn test_start_audit_unique_ids() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        // Start two audits
        let result1 = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: None,
            }))
            .await;

        let result2 = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: None,
            }))
            .await;

        // Parse results
        let json1: serde_json::Value = serde_json::from_str(&result1).unwrap();
        let json2: serde_json::Value = serde_json::from_str(&result2).unwrap();

        // IDs should be different
        assert_ne!(json1["audit_id"], json2["audit_id"]);
    }

    #[tokio::test]
    async fn test_get_audit_status_not_found() {
        use rmcp::handler::server::wrapper::Parameters;

        let server = RalphMcpServer::new();

        let result = server
            .get_audit_status(Parameters(GetAuditStatusRequest {
                audit_id: "audit-nonexistent".to_string(),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("not found"));
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("audit-nonexistent"));
    }

    #[tokio::test]
    async fn test_get_audit_status_pending() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        // Start an audit
        let start_result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: None,
            }))
            .await;

        let start_json: serde_json::Value = serde_json::from_str(&start_result).unwrap();
        let audit_id = start_json["audit_id"].as_str().unwrap().to_string();

        // Get the status
        let result = server
            .get_audit_status(Parameters(GetAuditStatusRequest {
                audit_id: audit_id.clone(),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["audit_id"], audit_id);
        assert_eq!(json["status"], "pending");
        assert!(json.get("progress").is_none()); // Progress not shown for pending
        assert!(json["message"].as_str().unwrap().contains("pending"));
    }

    #[tokio::test]
    async fn test_get_audit_status_running() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        use crate::mcp::tools::audit::AuditSection;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        // Start an audit
        let start_result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: None,
            }))
            .await;

        let start_json: serde_json::Value = serde_json::from_str(&start_result).unwrap();
        let audit_id = start_json["audit_id"].as_str().unwrap().to_string();

        // Update the audit state to running with progress
        {
            let mut state = server.state_mut().await;
            if let Some(audit_state) = state.audit_states.get_mut(&audit_id) {
                audit_state.progress = 50;
            }
        }

        // Get the status
        let result = server
            .get_audit_status(Parameters(GetAuditStatusRequest {
                audit_id: audit_id.clone(),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["audit_id"], audit_id);
        assert_eq!(json["status"], "running");
        assert_eq!(json["progress"], 50);
        assert!(json["message"].as_str().unwrap().contains("running"));
        assert!(json["message"].as_str().unwrap().contains("50%"));
    }

    #[tokio::test]
    async fn test_get_audit_status_completed() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        // Start an audit
        let start_result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: None,
            }))
            .await;

        let start_json: serde_json::Value = serde_json::from_str(&start_result).unwrap();
        let audit_id = start_json["audit_id"].as_str().unwrap().to_string();

        // Update the audit state to completed
        {
            let mut state = server.state_mut().await;
            if let Some(audit_state) = state.audit_states.get_mut(&audit_id) {
                audit_state.completed = true;
                audit_state.progress = 100;
            }
        }

        // Get the status
        let result = server
            .get_audit_status(Parameters(GetAuditStatusRequest {
                audit_id: audit_id.clone(),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["audit_id"], audit_id);
        assert_eq!(json["status"], "completed");
        assert!(json.get("progress").is_none()); // Progress not shown for completed
        assert!(json["message"].as_str().unwrap().contains("completed"));
    }

    #[tokio::test]
    async fn test_get_audit_status_failed() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        // Start an audit
        let start_result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: None,
            }))
            .await;

        let start_json: serde_json::Value = serde_json::from_str(&start_result).unwrap();
        let audit_id = start_json["audit_id"].as_str().unwrap().to_string();

        // Update the audit state to failed
        {
            let mut state = server.state_mut().await;
            if let Some(audit_state) = state.audit_states.get_mut(&audit_id) {
                audit_state.error = Some("Test error message".to_string());
            }
        }

        // Get the status
        let result = server
            .get_audit_status(Parameters(GetAuditStatusRequest {
                audit_id: audit_id.clone(),
            }))
            .await;

        // Parse the result as JSON
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["audit_id"], audit_id);
        assert_eq!(json["status"], "failed");
        assert_eq!(json["error"], "Test error message");
        assert!(json["message"].as_str().unwrap().contains("failed"));
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("Test error message"));
    }

    #[tokio::test]
    async fn test_start_audit_stores_state() {
        use rmcp::handler::server::wrapper::Parameters;
        use tempfile::TempDir;

        let server = RalphMcpServer::new();
        let temp_dir = TempDir::new().unwrap();

        // Start an audit
        let result = server
            .start_audit(Parameters(StartAuditRequest {
                path: Some(temp_dir.path().to_string_lossy().to_string()),
                sections: None,
                format: None,
            }))
            .await;

        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let audit_id = json["audit_id"].as_str().unwrap().to_string();

        // Verify state was stored
        {
            let state = server.state().await;
            assert!(state.audit_states.contains_key(&audit_id));
            let audit_state = state.audit_states.get(&audit_id).unwrap();
            assert_eq!(audit_state.audit_id, audit_id);
            assert_eq!(audit_state.progress, 0);
            assert!(!audit_state.completed);
            assert!(audit_state.error.is_none());
        }
    }
}
