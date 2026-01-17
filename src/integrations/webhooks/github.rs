//! GitHub webhook handler
//!
//! Provides signature verification for GitHub webhook payloads using HMAC-SHA256.

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Type alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

/// GitHub webhook handler for processing and verifying webhook events
#[derive(Debug, Clone)]
pub struct GitHubWebhookHandler {
    /// Secret key for HMAC signature verification
    secret: String,
}

impl GitHubWebhookHandler {
    /// Create a new GitHubWebhookHandler with the given secret
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Verify the signature of a GitHub webhook payload
    ///
    /// GitHub sends the signature in the `X-Hub-Signature-256` header in the format:
    /// `sha256=<hex-encoded-hmac>`
    ///
    /// # Arguments
    /// * `payload` - The raw webhook payload bytes
    /// * `signature` - The signature header value (e.g., "sha256=abc123...")
    ///
    /// # Returns
    /// `true` if the signature is valid, `false` otherwise
    pub fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        // GitHub signature format: "sha256=<hex>"
        let signature = match signature.strip_prefix("sha256=") {
            Some(sig) => sig,
            None => return false,
        };

        // Decode the hex signature
        let expected_signature = match hex::decode(signature) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        // Compute HMAC-SHA256
        let mut mac = match HmacSha256::new_from_slice(self.secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };
        mac.update(payload);

        // Verify the signature using constant-time comparison
        mac.verify_slice(&expected_signature).is_ok()
    }

    /// Compute the expected signature for a payload
    ///
    /// This is useful for testing and debugging.
    ///
    /// # Arguments
    /// * `payload` - The raw webhook payload bytes
    ///
    /// # Returns
    /// The signature in GitHub's format: `sha256=<hex-encoded-hmac>`
    pub fn compute_signature(&self, payload: &[u8]) -> String {
        let mut mac =
            HmacSha256::new_from_slice(self.secret.as_bytes()).expect("HMAC can take any key size");
        mac.update(payload);
        let result = mac.finalize();
        format!("sha256={}", hex::encode(result.into_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_webhook_handler_new() {
        let handler = GitHubWebhookHandler::new("my-secret");
        assert_eq!(handler.secret, "my-secret");
    }

    #[test]
    fn test_verify_signature_valid() {
        let handler = GitHubWebhookHandler::new("test-secret");
        let payload = b"test payload";

        // Compute the expected signature
        let signature = handler.compute_signature(payload);

        // Verify it
        assert!(handler.verify_signature(payload, &signature));
    }

    #[test]
    fn test_verify_signature_invalid() {
        let handler = GitHubWebhookHandler::new("test-secret");
        let payload = b"test payload";

        // Try with wrong signature
        let wrong_signature =
            "sha256=0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!handler.verify_signature(payload, wrong_signature));
    }

    #[test]
    fn test_verify_signature_missing_prefix() {
        let handler = GitHubWebhookHandler::new("test-secret");
        let payload = b"test payload";

        // Signature without sha256= prefix
        let signature = "abc123";
        assert!(!handler.verify_signature(payload, signature));
    }

    #[test]
    fn test_verify_signature_invalid_hex() {
        let handler = GitHubWebhookHandler::new("test-secret");
        let payload = b"test payload";

        // Invalid hex string
        let signature = "sha256=not-valid-hex";
        assert!(!handler.verify_signature(payload, signature));
    }

    #[test]
    fn test_verify_signature_wrong_secret() {
        let handler1 = GitHubWebhookHandler::new("secret-one");
        let handler2 = GitHubWebhookHandler::new("secret-two");
        let payload = b"test payload";

        // Compute signature with handler1's secret
        let signature = handler1.compute_signature(payload);

        // It should not verify with handler2's different secret
        assert!(!handler2.verify_signature(payload, &signature));
    }

    #[test]
    fn test_verify_signature_different_payload() {
        let handler = GitHubWebhookHandler::new("test-secret");

        let payload1 = b"payload one";
        let payload2 = b"payload two";

        // Compute signature for payload1
        let signature = handler.compute_signature(payload1);

        // It should not verify for payload2
        assert!(!handler.verify_signature(payload2, &signature));
    }

    #[test]
    fn test_compute_signature_format() {
        let handler = GitHubWebhookHandler::new("test-secret");
        let payload = b"test";

        let signature = handler.compute_signature(payload);

        // Should start with "sha256="
        assert!(signature.starts_with("sha256="));

        // The hex part should be 64 characters (32 bytes * 2)
        let hex_part = signature.strip_prefix("sha256=").unwrap();
        assert_eq!(hex_part.len(), 64);
    }

    #[test]
    fn test_verify_signature_empty_payload() {
        let handler = GitHubWebhookHandler::new("test-secret");
        let payload = b"";

        let signature = handler.compute_signature(payload);
        assert!(handler.verify_signature(payload, &signature));
    }

    #[test]
    fn test_verify_signature_real_github_example() {
        // This is a test case that mimics GitHub's actual webhook format
        let secret = "webhook_secret";
        let handler = GitHubWebhookHandler::new(secret);

        let payload = br#"{"action":"opened","number":1}"#;
        let signature = handler.compute_signature(payload);

        assert!(handler.verify_signature(payload, &signature));
    }
}
