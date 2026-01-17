//! Linear webhook handler
//!
//! Provides signature verification for Linear webhook payloads using HMAC-SHA256.

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Type alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

/// Linear webhook handler for processing and verifying webhook events
#[derive(Debug, Clone)]
pub struct LinearWebhookHandler {
    /// Secret key for HMAC signature verification
    secret: String,
}

impl LinearWebhookHandler {
    /// Create a new LinearWebhookHandler with the given secret
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Verify the signature of a Linear webhook payload
    ///
    /// Linear sends the signature in the `Linear-Signature` header.
    /// The signature is a hex-encoded HMAC-SHA256 of the raw request body.
    ///
    /// # Arguments
    /// * `payload` - The raw webhook payload bytes
    /// * `signature` - The signature header value (hex-encoded HMAC)
    ///
    /// # Returns
    /// `true` if the signature is valid, `false` otherwise
    pub fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
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
    /// The signature as hex-encoded HMAC-SHA256
    pub fn compute_signature(&self, payload: &[u8]) -> String {
        let mut mac =
            HmacSha256::new_from_slice(self.secret.as_bytes()).expect("HMAC can take any key size");
        mac.update(payload);
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_webhook_handler_new() {
        let handler = LinearWebhookHandler::new("my-secret");
        assert_eq!(handler.secret, "my-secret");
    }

    #[test]
    fn test_verify_signature_valid() {
        let handler = LinearWebhookHandler::new("test-secret");
        let payload = b"test payload";

        // Compute the expected signature
        let signature = handler.compute_signature(payload);

        // Verify it
        assert!(handler.verify_signature(payload, &signature));
    }

    #[test]
    fn test_verify_signature_invalid() {
        let handler = LinearWebhookHandler::new("test-secret");
        let payload = b"test payload";

        // Try with wrong signature (64 hex chars = 32 bytes)
        let wrong_signature = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!handler.verify_signature(payload, wrong_signature));
    }

    #[test]
    fn test_verify_signature_invalid_hex() {
        let handler = LinearWebhookHandler::new("test-secret");
        let payload = b"test payload";

        // Invalid hex string
        let signature = "not-valid-hex";
        assert!(!handler.verify_signature(payload, signature));
    }

    #[test]
    fn test_verify_signature_wrong_secret() {
        let handler1 = LinearWebhookHandler::new("secret-one");
        let handler2 = LinearWebhookHandler::new("secret-two");
        let payload = b"test payload";

        // Compute signature with handler1's secret
        let signature = handler1.compute_signature(payload);

        // It should not verify with handler2's different secret
        assert!(!handler2.verify_signature(payload, &signature));
    }

    #[test]
    fn test_verify_signature_different_payload() {
        let handler = LinearWebhookHandler::new("test-secret");

        let payload1 = b"payload one";
        let payload2 = b"payload two";

        // Compute signature for payload1
        let signature = handler.compute_signature(payload1);

        // It should not verify for payload2
        assert!(!handler.verify_signature(payload2, &signature));
    }

    #[test]
    fn test_compute_signature_format() {
        let handler = LinearWebhookHandler::new("test-secret");
        let payload = b"test";

        let signature = handler.compute_signature(payload);

        // Should be 64 hex characters (32 bytes * 2)
        assert_eq!(signature.len(), 64);

        // Should be valid hex
        assert!(hex::decode(&signature).is_ok());
    }

    #[test]
    fn test_verify_signature_empty_payload() {
        let handler = LinearWebhookHandler::new("test-secret");
        let payload = b"";

        let signature = handler.compute_signature(payload);
        assert!(handler.verify_signature(payload, &signature));
    }

    #[test]
    fn test_verify_signature_real_linear_example() {
        // This is a test case that mimics Linear's actual webhook format
        let secret = "webhook_secret";
        let handler = LinearWebhookHandler::new(secret);

        let payload = br#"{"action":"create","type":"Issue","data":{"id":"123"}}"#;
        let signature = handler.compute_signature(payload);

        assert!(handler.verify_signature(payload, &signature));
    }

    #[test]
    fn test_verify_signature_case_insensitive_hex() {
        let handler = LinearWebhookHandler::new("test-secret");
        let payload = b"test payload";

        // Compute signature (lowercase hex)
        let signature_lower = handler.compute_signature(payload);

        // Convert to uppercase
        let signature_upper = signature_lower.to_uppercase();

        // Both should verify (hex decoding is case-insensitive)
        assert!(handler.verify_signature(payload, &signature_lower));
        assert!(handler.verify_signature(payload, &signature_upper));
    }
}
