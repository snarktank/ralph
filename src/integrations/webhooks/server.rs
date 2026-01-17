//! Webhook server implementation
//!
//! Provides HTTP endpoints for receiving webhook events from GitHub and Linear.
//! Includes signature verification for secure webhook handling.

#![allow(dead_code)]

use super::github::GitHubWebhookHandler;
use super::linear::LinearWebhookHandler;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Webhook server configuration
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    /// Port to listen on
    pub port: u16,
    /// Address to bind to (e.g., "0.0.0.0" or "127.0.0.1")
    pub bind_address: String,
    /// Secret for verifying GitHub webhook signatures
    pub github_secret: Option<String>,
    /// Secret for verifying Linear webhook signatures
    pub linear_secret: Option<String>,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            bind_address: "127.0.0.1".to_string(),
            github_secret: None,
            linear_secret: None,
        }
    }
}

impl WebhookConfig {
    /// Create a new WebhookConfig with the given port
    pub fn new(port: u16) -> Self {
        Self {
            port,
            ..Default::default()
        }
    }

    /// Set the bind address
    pub fn with_bind_address(mut self, address: impl Into<String>) -> Self {
        self.bind_address = address.into();
        self
    }

    /// Set the GitHub webhook secret
    pub fn with_github_secret(mut self, secret: impl Into<String>) -> Self {
        self.github_secret = Some(secret.into());
        self
    }

    /// Set the Linear webhook secret
    pub fn with_linear_secret(mut self, secret: impl Into<String>) -> Self {
        self.linear_secret = Some(secret.into());
        self
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("WEBHOOK_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            bind_address: std::env::var("WEBHOOK_BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            github_secret: std::env::var("GITHUB_WEBHOOK_SECRET").ok(),
            linear_secret: std::env::var("LINEAR_WEBHOOK_SECRET").ok(),
        }
    }

    /// Get the full bind address (ip:port)
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
}

/// Shared application state for the webhook server
#[derive(Debug, Clone)]
pub struct AppState {
    /// Webhook configuration
    pub config: WebhookConfig,
    /// Indicates if the server is healthy
    pub healthy: Arc<RwLock<bool>>,
}

impl AppState {
    /// Create a new AppState with the given config
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            config,
            healthy: Arc::new(RwLock::new(true)),
        }
    }

    /// Set the health status
    pub async fn set_healthy(&self, healthy: bool) {
        let mut guard = self.healthy.write().await;
        *guard = healthy;
    }

    /// Check if the server is healthy
    pub async fn is_healthy(&self) -> bool {
        *self.healthy.read().await
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(WebhookConfig::default())
    }
}

/// Webhook error types
#[derive(Debug, Clone, Serialize)]
pub enum WebhookError {
    /// Invalid or missing signature
    InvalidSignature,
    /// Failed to parse webhook payload
    ParseError(String),
    /// Internal server error
    InternalError(String),
    /// Unsupported event type
    UnsupportedEvent(String),
}

impl std::fmt::Display for WebhookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookError::InvalidSignature => write!(f, "Invalid or missing webhook signature"),
            WebhookError::ParseError(msg) => write!(f, "Failed to parse webhook payload: {}", msg),
            WebhookError::InternalError(msg) => write!(f, "Internal server error: {}", msg),
            WebhookError::UnsupportedEvent(event) => {
                write!(f, "Unsupported webhook event type: {}", event)
            }
        }
    }
}

impl std::error::Error for WebhookError {}

impl IntoResponse for WebhookError {
    fn into_response(self) -> Response {
        let status = match &self {
            WebhookError::InvalidSignature => StatusCode::UNAUTHORIZED,
            WebhookError::ParseError(_) => StatusCode::BAD_REQUEST,
            WebhookError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::UnsupportedEvent(_) => StatusCode::BAD_REQUEST,
        };

        let body = Json(WebhookErrorResponse {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

/// Result type for webhook operations
pub type WebhookResult<T> = Result<T, WebhookError>;

/// Error response body
#[derive(Debug, Serialize)]
struct WebhookErrorResponse {
    error: String,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

/// GitHub webhook response
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubWebhookResponse {
    pub received: bool,
    pub event_type: Option<String>,
    pub message: String,
}

/// Linear webhook response
#[derive(Debug, Serialize, Deserialize)]
pub struct LinearWebhookResponse {
    pub received: bool,
    pub event_type: Option<String>,
    pub message: String,
}

/// GitHub webhook payload (simplified for common events)
#[derive(Debug, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: Option<String>,
    pub sender: Option<GitHubUser>,
    pub repository: Option<GitHubRepository>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRepository {
    pub name: String,
    pub full_name: String,
}

/// Linear webhook payload
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearWebhookPayload {
    pub action: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
    pub url: Option<String>,
    pub created_at: Option<String>,
}

/// Create the webhook router with all routes
pub fn create_webhook_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/webhooks/github", post(github_webhook_handler))
        .route("/webhooks/linear", post(linear_webhook_handler))
        .with_state(state)
}

/// Health check endpoint handler
///
/// GET /health
///
/// Returns the current health status of the webhook server.
pub async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let status = if state.is_healthy().await {
        "healthy"
    } else {
        "unhealthy"
    };

    Json(HealthResponse {
        status: status.to_string(),
        service: "ralph-webhooks".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Verify GitHub webhook signature
///
/// Returns Ok(()) if the signature is valid or no secret is configured.
/// Returns Err(WebhookError::InvalidSignature) if the signature is invalid.
fn verify_github_signature(
    secret: Option<&str>,
    payload: &[u8],
    headers: &HeaderMap,
) -> Result<(), WebhookError> {
    let Some(secret) = secret else {
        // No secret configured, skip verification
        return Ok(());
    };

    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or(WebhookError::InvalidSignature)?;

    let handler = GitHubWebhookHandler::new(secret);
    if handler.verify_signature(payload, signature) {
        Ok(())
    } else {
        Err(WebhookError::InvalidSignature)
    }
}

/// GitHub webhook endpoint handler
///
/// POST /webhooks/github
///
/// Receives webhook events from GitHub. The signature is verified using
/// HMAC-SHA256 if a secret is configured. Returns 401 Unauthorized on
/// invalid signature.
pub async fn github_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<GitHubWebhookResponse>, WebhookError> {
    // Verify the signature if a secret is configured
    verify_github_signature(state.config.github_secret.as_deref(), &body, &headers)?;

    // Extract the event type from headers
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    // Parse the payload
    let payload: GitHubWebhookPayload =
        serde_json::from_slice(&body).map_err(|e| WebhookError::ParseError(e.to_string()))?;

    // Log the received webhook (in a real implementation, this would process the event)
    tracing::info!(
        event_type = ?event_type,
        action = ?payload.action,
        repository = ?payload.repository.as_ref().map(|r| &r.full_name),
        "Received GitHub webhook"
    );

    Ok(Json(GitHubWebhookResponse {
        received: true,
        event_type,
        message: format!(
            "Received GitHub webhook: action={:?}",
            payload.action.as_deref().unwrap_or("unknown")
        ),
    }))
}

/// Verify Linear webhook signature
///
/// Returns Ok(()) if the signature is valid or no secret is configured.
/// Returns Err(WebhookError::InvalidSignature) if the signature is invalid.
fn verify_linear_signature(
    secret: Option<&str>,
    payload: &[u8],
    headers: &HeaderMap,
) -> Result<(), WebhookError> {
    let Some(secret) = secret else {
        // No secret configured, skip verification
        return Ok(());
    };

    let signature = headers
        .get("Linear-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(WebhookError::InvalidSignature)?;

    let handler = LinearWebhookHandler::new(secret);
    if handler.verify_signature(payload, signature) {
        Ok(())
    } else {
        Err(WebhookError::InvalidSignature)
    }
}

/// Linear webhook endpoint handler
///
/// POST /webhooks/linear
///
/// Receives webhook events from Linear. The signature is verified using
/// HMAC-SHA256 if a secret is configured. Returns 401 Unauthorized on
/// invalid signature.
pub async fn linear_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<LinearWebhookResponse>, WebhookError> {
    // Verify the signature if a secret is configured
    verify_linear_signature(state.config.linear_secret.as_deref(), &body, &headers)?;

    // Parse the payload
    let payload: LinearWebhookPayload =
        serde_json::from_slice(&body).map_err(|e| WebhookError::ParseError(e.to_string()))?;

    // Log the received webhook (in a real implementation, this would process the event)
    tracing::info!(
        event_type = %payload.event_type,
        action = %payload.action,
        "Received Linear webhook"
    );

    Ok(Json(LinearWebhookResponse {
        received: true,
        event_type: Some(payload.event_type.clone()),
        message: format!(
            "Received Linear webhook: type={}, action={}",
            payload.event_type, payload.action
        ),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn create_test_state() -> AppState {
        AppState::new(WebhookConfig::default())
    }

    #[test]
    fn test_webhook_config_default() {
        let config = WebhookConfig::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.bind_address, "127.0.0.1");
        assert!(config.github_secret.is_none());
        assert!(config.linear_secret.is_none());
    }

    #[test]
    fn test_webhook_config_new() {
        let config = WebhookConfig::new(8080);
        assert_eq!(config.port, 8080);
        assert_eq!(config.bind_address, "127.0.0.1");
    }

    #[test]
    fn test_webhook_config_builder() {
        let config = WebhookConfig::new(9000)
            .with_bind_address("0.0.0.0")
            .with_github_secret("gh_secret")
            .with_linear_secret("linear_secret");

        assert_eq!(config.port, 9000);
        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.github_secret, Some("gh_secret".to_string()));
        assert_eq!(config.linear_secret, Some("linear_secret".to_string()));
    }

    #[test]
    fn test_webhook_config_socket_addr() {
        let config = WebhookConfig::new(3000).with_bind_address("127.0.0.1");
        assert_eq!(config.socket_addr(), "127.0.0.1:3000");
    }

    #[tokio::test]
    async fn test_app_state_health() {
        let state = AppState::default();
        assert!(state.is_healthy().await);

        state.set_healthy(false).await;
        assert!(!state.is_healthy().await);

        state.set_healthy(true).await;
        assert!(state.is_healthy().await);
    }

    #[test]
    fn test_webhook_error_display() {
        assert_eq!(
            WebhookError::InvalidSignature.to_string(),
            "Invalid or missing webhook signature"
        );
        assert_eq!(
            WebhookError::ParseError("bad json".to_string()).to_string(),
            "Failed to parse webhook payload: bad json"
        );
        assert_eq!(
            WebhookError::InternalError("database error".to_string()).to_string(),
            "Internal server error: database error"
        );
        assert_eq!(
            WebhookError::UnsupportedEvent("unknown".to_string()).to_string(),
            "Unsupported webhook event type: unknown"
        );
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = create_test_state();
        let app = create_webhook_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.status, "healthy");
        assert_eq!(health.service, "ralph-webhooks");
    }

    #[tokio::test]
    async fn test_github_webhook_endpoint() {
        let state = create_test_state();
        let app = create_webhook_router(state);

        let payload =
            r#"{"action": "opened", "repository": {"name": "test", "full_name": "owner/test"}}"#;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/github")
                    .header("Content-Type", "application/json")
                    .header("X-GitHub-Event", "issues")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let result: GitHubWebhookResponse = serde_json::from_slice(&body).unwrap();

        assert!(result.received);
        assert_eq!(result.event_type, Some("issues".to_string()));
    }

    #[tokio::test]
    async fn test_github_webhook_endpoint_invalid_json() {
        let state = create_test_state();
        let app = create_webhook_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/github")
                    .header("Content-Type", "application/json")
                    .body(Body::from("not valid json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_linear_webhook_endpoint() {
        let state = create_test_state();
        let app = create_webhook_router(state);

        let payload = r#"{"action": "create", "type": "Issue", "data": {"id": "123"}, "createdAt": "2024-01-01T00:00:00Z"}"#;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/linear")
                    .header("Content-Type", "application/json")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let result: LinearWebhookResponse = serde_json::from_slice(&body).unwrap();

        assert!(result.received);
        assert_eq!(result.event_type, Some("Issue".to_string()));
    }

    #[tokio::test]
    async fn test_linear_webhook_endpoint_invalid_json() {
        let state = create_test_state();
        let app = create_webhook_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/linear")
                    .header("Content-Type", "application/json")
                    .body(Body::from("invalid"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            service: "ralph-webhooks".to_string(),
            version: "0.1.0".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("ralph-webhooks"));
    }

    #[test]
    fn test_github_webhook_response_serialization() {
        let response = GitHubWebhookResponse {
            received: true,
            event_type: Some("push".to_string()),
            message: "Received push event".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"received\":true"));
        assert!(json.contains("push"));
    }

    #[test]
    fn test_linear_webhook_response_serialization() {
        let response = LinearWebhookResponse {
            received: true,
            event_type: Some("Issue".to_string()),
            message: "Received Issue event".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"received\":true"));
        assert!(json.contains("Issue"));
    }

    #[test]
    fn test_github_webhook_payload_deserialization() {
        let json = r#"{
            "action": "opened",
            "sender": {"login": "testuser", "id": 12345},
            "repository": {"name": "test-repo", "full_name": "owner/test-repo"}
        }"#;

        let payload: GitHubWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.action, Some("opened".to_string()));
        assert_eq!(payload.sender.as_ref().unwrap().login, "testuser");
        assert_eq!(
            payload.repository.as_ref().unwrap().full_name,
            "owner/test-repo"
        );
    }

    #[test]
    fn test_linear_webhook_payload_deserialization() {
        let json = r#"{
            "action": "create",
            "type": "Issue",
            "data": {"id": "abc123", "title": "Test Issue"},
            "url": "https://linear.app/issue/123",
            "createdAt": "2024-01-01T00:00:00Z"
        }"#;

        let payload: LinearWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.action, "create");
        assert_eq!(payload.event_type, "Issue");
        assert_eq!(
            payload.url,
            Some("https://linear.app/issue/123".to_string())
        );
    }

    // ===== Signature Verification Tests =====

    fn create_test_state_with_secrets() -> AppState {
        AppState::new(
            WebhookConfig::new(3000)
                .with_github_secret("github-test-secret")
                .with_linear_secret("linear-test-secret"),
        )
    }

    #[test]
    fn test_verify_github_signature_no_secret() {
        // When no secret is configured, verification should pass
        let headers = HeaderMap::new();
        let payload = b"test";
        let result = verify_github_signature(None, payload, &headers);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_github_signature_valid() {
        let secret = "test-secret";
        let payload = b"test payload";
        let handler = GitHubWebhookHandler::new(secret);
        let signature = handler.compute_signature(payload);

        let mut headers = HeaderMap::new();
        headers.insert("X-Hub-Signature-256", signature.parse().unwrap());

        let result = verify_github_signature(Some(secret), payload, &headers);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_github_signature_invalid() {
        let secret = "test-secret";
        let payload = b"test payload";

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Hub-Signature-256",
            "sha256=0000000000000000000000000000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
        );

        let result = verify_github_signature(Some(secret), payload, &headers);
        assert!(matches!(result, Err(WebhookError::InvalidSignature)));
    }

    #[test]
    fn test_verify_github_signature_missing_header() {
        let secret = "test-secret";
        let payload = b"test payload";
        let headers = HeaderMap::new();

        let result = verify_github_signature(Some(secret), payload, &headers);
        assert!(matches!(result, Err(WebhookError::InvalidSignature)));
    }

    #[test]
    fn test_verify_linear_signature_no_secret() {
        // When no secret is configured, verification should pass
        let headers = HeaderMap::new();
        let payload = b"test";
        let result = verify_linear_signature(None, payload, &headers);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_linear_signature_valid() {
        let secret = "test-secret";
        let payload = b"test payload";
        let handler = LinearWebhookHandler::new(secret);
        let signature = handler.compute_signature(payload);

        let mut headers = HeaderMap::new();
        headers.insert("Linear-Signature", signature.parse().unwrap());

        let result = verify_linear_signature(Some(secret), payload, &headers);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_linear_signature_invalid() {
        let secret = "test-secret";
        let payload = b"test payload";

        let mut headers = HeaderMap::new();
        headers.insert(
            "Linear-Signature",
            "0000000000000000000000000000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
        );

        let result = verify_linear_signature(Some(secret), payload, &headers);
        assert!(matches!(result, Err(WebhookError::InvalidSignature)));
    }

    #[test]
    fn test_verify_linear_signature_missing_header() {
        let secret = "test-secret";
        let payload = b"test payload";
        let headers = HeaderMap::new();

        let result = verify_linear_signature(Some(secret), payload, &headers);
        assert!(matches!(result, Err(WebhookError::InvalidSignature)));
    }

    #[tokio::test]
    async fn test_github_webhook_with_valid_signature() {
        let state = create_test_state_with_secrets();
        let app = create_webhook_router(state.clone());

        let payload =
            r#"{"action": "opened", "repository": {"name": "test", "full_name": "owner/test"}}"#;
        let handler = GitHubWebhookHandler::new("github-test-secret");
        let signature = handler.compute_signature(payload.as_bytes());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/github")
                    .header("Content-Type", "application/json")
                    .header("X-GitHub-Event", "issues")
                    .header("X-Hub-Signature-256", signature)
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_github_webhook_with_invalid_signature() {
        let state = create_test_state_with_secrets();
        let app = create_webhook_router(state);

        let payload =
            r#"{"action": "opened", "repository": {"name": "test", "full_name": "owner/test"}}"#;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/github")
                    .header("Content-Type", "application/json")
                    .header("X-GitHub-Event", "issues")
                    .header(
                        "X-Hub-Signature-256",
                        "sha256=0000000000000000000000000000000000000000000000000000000000000000",
                    )
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_github_webhook_missing_signature_header() {
        let state = create_test_state_with_secrets();
        let app = create_webhook_router(state);

        let payload =
            r#"{"action": "opened", "repository": {"name": "test", "full_name": "owner/test"}}"#;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/github")
                    .header("Content-Type", "application/json")
                    .header("X-GitHub-Event", "issues")
                    // No X-Hub-Signature-256 header
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_linear_webhook_with_valid_signature() {
        let state = create_test_state_with_secrets();
        let app = create_webhook_router(state.clone());

        let payload = r#"{"action": "create", "type": "Issue", "data": {"id": "123"}, "createdAt": "2024-01-01T00:00:00Z"}"#;
        let handler = LinearWebhookHandler::new("linear-test-secret");
        let signature = handler.compute_signature(payload.as_bytes());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/linear")
                    .header("Content-Type", "application/json")
                    .header("Linear-Signature", signature)
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_linear_webhook_with_invalid_signature() {
        let state = create_test_state_with_secrets();
        let app = create_webhook_router(state);

        let payload = r#"{"action": "create", "type": "Issue", "data": {"id": "123"}, "createdAt": "2024-01-01T00:00:00Z"}"#;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/linear")
                    .header("Content-Type", "application/json")
                    .header(
                        "Linear-Signature",
                        "0000000000000000000000000000000000000000000000000000000000000000",
                    )
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_linear_webhook_missing_signature_header() {
        let state = create_test_state_with_secrets();
        let app = create_webhook_router(state);

        let payload = r#"{"action": "create", "type": "Issue", "data": {"id": "123"}, "createdAt": "2024-01-01T00:00:00Z"}"#;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/linear")
                    .header("Content-Type", "application/json")
                    // No Linear-Signature header
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
