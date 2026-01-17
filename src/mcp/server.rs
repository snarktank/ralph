// MCP Server implementation for Ralph
// This module provides the core MCP server struct

#![allow(dead_code)]

use crate::mcp::resources::{
    list_ralph_resources, read_prd_resource, read_status_resource, ResourceError, PRD_RESOURCE_URI,
    STATUS_RESOURCE_URI,
};
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
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    Implementation, ListResourcesResult, PaginatedRequestParam, ReadResourceRequestParam,
    ReadResourceResult, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
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
}

/// Shared server state that can be accessed across async contexts.
#[derive(Debug)]
pub struct ServerState {
    /// Path to the currently loaded PRD file
    pub prd_path: Option<PathBuf>,
    /// Current execution state
    pub execution_state: ExecutionState,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            prd_path: None,
            execution_state: ExecutionState::Idle,
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
}

impl RalphMcpServer {
    /// Create a new RalphMcpServer instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use ralph::mcp::RalphMcpServer;
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
    /// use ralph::mcp::RalphMcpServer;
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
            })),
            config: Arc::new(None),
            cancel_sender: Arc::new(cancel_sender),
            cancel_receiver,
            tool_router: Self::tool_router(),
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
    /// use ralph::mcp::RalphMcpServer;
    /// use ralph::quality::QualityConfig;
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

        // Create started response
        // Note: In a real implementation, this would spawn an async task to do the actual work.
        // For now, we just return that execution has started.
        // The actual execution logic would involve:
        // 1. Checking out the correct branch
        // 2. Running the agent to implement the story
        // 3. Running quality checks
        // 4. Committing changes
        // 5. Updating the PRD
        // The client can poll get_status to check progress.
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
}
