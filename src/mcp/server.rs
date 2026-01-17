// MCP Server implementation for Ralph
// This module provides the core MCP server struct

#![allow(dead_code)]

use crate::mcp::tools::get_status::{GetStatusRequest, GetStatusResponse};
use crate::mcp::tools::list_stories::{load_stories, ListStoriesRequest, ListStoriesResponse};
use crate::quality::QualityConfig;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};

/// Execution state of the Ralph agent.
///
/// This enum tracks the current state of story execution,
/// allowing MCP clients to monitor progress and respond appropriately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionState {
    /// No execution in progress
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

impl Default for ExecutionState {
    fn default() -> Self {
        Self::Idle
    }
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
}
