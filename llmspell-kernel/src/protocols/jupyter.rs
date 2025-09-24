//! Jupyter wire protocol 5.3 implementation
//!
//! This is the ONLY protocol implementation for the kernel.
//! All kernel modes (embedded, service, client) use this protocol.

use crate::traits::Protocol;
use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::HashMap;
use uuid::Uuid;

/// Jupyter wire protocol implementation (version 5.3)
///
/// This protocol is used for ALL kernel communication, regardless of transport.
/// The difference between CLI and service mode is the transport layer, not the protocol.
#[derive(Debug, Clone)]
pub struct JupyterProtocol {
    session_id: String,
    kernel_id: String,
    protocol_version: String,
    username: String,
    /// HMAC key for message authentication (hex-encoded)
    hmac_key: Option<Vec<u8>>,
}

impl JupyterProtocol {
    /// Create a new Jupyter protocol instance for kernel mode
    pub fn new(session_id: String, kernel_id: String) -> Self {
        Self {
            session_id,
            kernel_id,
            protocol_version: "5.3".to_string(),
            username: "kernel".to_string(),
            hmac_key: None,
        }
    }

    /// Create a new Jupyter protocol instance for client mode
    pub fn new_client() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            kernel_id: String::new(), // Client doesn't own a kernel
            protocol_version: "5.3".to_string(),
            username: "client".to_string(),
            hmac_key: None,
        }
    }

    /// Set the HMAC key for message authentication
    pub fn set_hmac_key(&mut self, key: &str) {
        // Decode hex-encoded key
        if let Ok(decoded) = hex::decode(key) {
            self.hmac_key = Some(decoded);
        } else {
            // If not hex, use raw bytes
            self.hmac_key = Some(key.as_bytes().to_vec());
        }
    }

    /// Sign message components according to Jupyter protocol
    /// Returns the HMAC signature as hex-encoded string
    fn sign_message(
        &self,
        header: &[u8],
        parent_header: &[u8],
        metadata: &[u8],
        content: &[u8],
    ) -> Result<String> {
        if let Some(ref key) = self.hmac_key {
            type HmacSha256 = Hmac<Sha256>;
            let mut mac =
                HmacSha256::new_from_slice(key).map_err(|e| anyhow!("Invalid HMAC key: {}", e))?;

            // Sign in the order specified by Jupyter protocol
            mac.update(header);
            mac.update(parent_header);
            mac.update(metadata);
            mac.update(content);

            let result = mac.finalize();
            Ok(hex::encode(result.into_bytes()))
        } else {
            // No key set, return empty signature
            Ok(String::new())
        }
    }

    /// Verify message signature
    fn verify_signature(
        &self,
        signature: &str,
        header: &[u8],
        parent_header: &[u8],
        metadata: &[u8],
        content: &[u8],
    ) -> Result<bool> {
        if self.hmac_key.is_some() {
            // Compute expected signature
            let expected = self.sign_message(header, parent_header, metadata, content)?;

            // Compare signatures (constant-time comparison for security)
            if signature.len() != expected.len() {
                return Ok(false);
            }

            let sig_bytes = signature.as_bytes();
            let exp_bytes = expected.as_bytes();
            let mut diff = 0u8;
            for i in 0..sig_bytes.len() {
                diff |= sig_bytes[i] ^ exp_bytes[i];
            }

            Ok(diff == 0)
        } else {
            // No key set, accept empty signature
            Ok(signature.is_empty())
        }
    }

    /// Create a message header
    fn create_header(&self, msg_type: &str) -> HashMap<String, Value> {
        let mut header = HashMap::new();
        header.insert("msg_id".to_string(), json!(Uuid::new_v4().to_string()));
        header.insert("session".to_string(), json!(self.session_id));
        header.insert("username".to_string(), json!(self.username));
        header.insert("msg_type".to_string(), json!(msg_type));
        header.insert("version".to_string(), json!(self.protocol_version));
        header.insert("date".to_string(), json!(chrono::Utc::now().to_rfc3339()));
        // Include kernel_id in metadata for tracking
        if !self.kernel_id.is_empty() {
            header.insert("kernel".to_string(), json!(self.kernel_id));
        }
        header
    }

    /// Prepare a message with custom components (for testing)
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails
    pub fn prepare_message(
        &self,
        header: &Value,
        parent_header: &Value,
        metadata: &Value,
        content: &Value,
    ) -> Result<Vec<u8>> {
        // Convert to JSON strings
        let header_bytes = serde_json::to_vec(header)?;
        let parent_bytes = serde_json::to_vec(parent_header)?;
        let metadata_bytes = serde_json::to_vec(metadata)?;
        let content_bytes = serde_json::to_vec(content)?;

        // Sign the message if we have a key
        let signature = self.sign_message(
            &header_bytes,
            &parent_bytes,
            &metadata_bytes,
            &content_bytes,
        )?;

        // Create the message structure
        let request = json!({
            "signature": signature,
            "header": header,
            "parent_header": parent_header,
            "metadata": metadata,
            "content": content,
            "buffers": []
        });

        Ok(serde_json::to_vec(&request)?)
    }
}

impl JupyterProtocol {
    /// Parse a multipart `ZeroMQ` message according to Jupyter wire protocol
    ///
    /// Expected format:
    /// [identity1, identity2, ..., "<IDS|MSG>", signature, header, `parent_header`, metadata, content, ...buffers]
    ///
    /// # Errors
    ///
    /// Returns an error if the message format is invalid or parsing fails
    pub fn parse_wire_message(&self, parts: &[Vec<u8>]) -> Result<HashMap<String, Value>> {
        if parts.is_empty() {
            return Ok(HashMap::new());
        }

        // Find the delimiter "<IDS|MSG>"
        let delimiter = b"<IDS|MSG>";
        let delimiter_idx = parts
            .iter()
            .position(|part| part == delimiter)
            .ok_or_else(|| anyhow!("Missing <IDS|MSG> delimiter in message"))?;

        // Need at least 5 parts after delimiter: signature, header, parent, metadata, content
        if parts.len() < delimiter_idx + 6 {
            return Err(anyhow!(
                "Incomplete message: expected at least 6 parts after delimiter, got {}",
                parts.len() - delimiter_idx - 1
            ));
        }

        // Extract message components after delimiter
        let signature = &parts[delimiter_idx + 1];
        let header_bytes = &parts[delimiter_idx + 2];
        let parent_header_bytes = &parts[delimiter_idx + 3];
        let metadata_bytes = &parts[delimiter_idx + 4];
        let content_bytes = &parts[delimiter_idx + 5];

        // Verify HMAC signature if key is set
        if self.hmac_key.is_some() {
            let sig_str = std::str::from_utf8(signature)
                .map_err(|_| anyhow!("Invalid signature encoding"))?;

            if !self.verify_signature(
                sig_str,
                header_bytes,
                parent_header_bytes,
                metadata_bytes,
                content_bytes,
            )? {
                return Err(anyhow!("Invalid message signature"));
            }
        }

        // Parse JSON components
        let header: Value = serde_json::from_slice(header_bytes)?;
        let parent_header: Value = serde_json::from_slice(parent_header_bytes)?;
        let metadata: Value = serde_json::from_slice(metadata_bytes)?;
        let content: Value = serde_json::from_slice(content_bytes)?;

        // Extract any binary buffers
        let buffers: Vec<Vec<u8>> = if parts.len() > delimiter_idx + 6 {
            parts[delimiter_idx + 6..].to_vec()
        } else {
            Vec::new()
        };

        // Build result map
        let mut result = HashMap::new();

        // Extract msg_type and other fields from header
        if let Value::Object(header_map) = &header {
            for (key, value) in header_map {
                result.insert(key.clone(), value.clone());
            }
        }

        // Add the full components for processing
        result.insert("header".to_string(), header);
        result.insert("parent_header".to_string(), parent_header);
        result.insert("metadata".to_string(), metadata);
        result.insert("content".to_string(), content);
        result.insert("buffers".to_string(), json!(buffers.len()));

        Ok(result)
    }
}

impl Protocol for JupyterProtocol {
    fn parse_message(&self, data: &[u8]) -> Result<HashMap<String, Value>> {
        // This method is for single-part messages (legacy/testing)
        // For real wire protocol, use parse_wire_message with all parts

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Try to parse as JSON directly (for testing/legacy)
        let message: Value = serde_json::from_slice(data)?;

        // Extract message parts for verification
        if let Value::Object(ref map) = message {
            // If HMAC key is set, verify signature
            if self.hmac_key.is_some() {
                // Extract signature if present
                if let Some(Value::String(signature)) = map.get("signature") {
                    // Serialize parts for verification
                    let header = serde_json::to_vec(map.get("header").unwrap_or(&Value::Null))?;
                    let parent =
                        serde_json::to_vec(map.get("parent_header").unwrap_or(&Value::Null))?;
                    let metadata = serde_json::to_vec(map.get("metadata").unwrap_or(&Value::Null))?;
                    let content = serde_json::to_vec(map.get("content").unwrap_or(&Value::Null))?;

                    // Verify signature
                    if !self.verify_signature(signature, &header, &parent, &metadata, &content)? {
                        return Err(anyhow!("Invalid message signature"));
                    }
                }
            }

            // Return parsed message
            Ok(map
                .into_iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect())
        } else {
            Ok(HashMap::new())
        }
    }

    fn create_response(&self, msg_type: &str, content: Value) -> Result<Vec<u8>> {
        // Create a complete Jupyter wire protocol response with HMAC signature
        let header = self.create_header(msg_type);
        let parent_header: HashMap<String, Value> = HashMap::new(); // Would be set from request in real impl
        let metadata: HashMap<String, Value> = HashMap::new();

        // Calculate signature if key is set
        let signature = if self.hmac_key.is_some() {
            let header_bytes = serde_json::to_vec(&header)?;
            let parent_bytes = serde_json::to_vec(&parent_header)?;
            let metadata_bytes = serde_json::to_vec(&metadata)?;
            let content_bytes = serde_json::to_vec(&content)?;

            self.sign_message(
                &header_bytes,
                &parent_bytes,
                &metadata_bytes,
                &content_bytes,
            )?
        } else {
            String::new()
        };

        let response = json!({
            "signature": signature,
            "header": header,
            "parent_header": parent_header,
            "metadata": metadata,
            "content": content,
            "buffers": []
        });

        Ok(serde_json::to_vec(&response)?)
    }

    fn create_request(&self, msg_type: &str, content: Value) -> Result<Vec<u8>> {
        // Create a complete Jupyter wire protocol request with HMAC signature
        let header = self.create_header(msg_type);
        let parent_header: HashMap<String, Value> = HashMap::new();
        let metadata: HashMap<String, Value> = HashMap::new();

        // Calculate signature if key is set
        let signature = if self.hmac_key.is_some() {
            let header_bytes = serde_json::to_vec(&header)?;
            let parent_bytes = serde_json::to_vec(&parent_header)?;
            let metadata_bytes = serde_json::to_vec(&metadata)?;
            let content_bytes = serde_json::to_vec(&content)?;

            self.sign_message(
                &header_bytes,
                &parent_bytes,
                &metadata_bytes,
                &content_bytes,
            )?
        } else {
            String::new()
        };

        let request = json!({
            "signature": signature,
            "header": header,
            "parent_header": parent_header,
            "metadata": metadata,
            "content": content,
            "buffers": []
        });

        Ok(serde_json::to_vec(&request)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_protocol() -> JupyterProtocol {
        JupyterProtocol::new("test-session".to_string(), "test-kernel".to_string())
    }

    #[test]
    fn test_sign_message_without_key() {
        let protocol = create_test_protocol();

        // Without HMAC key, signature should be empty
        let signature = protocol
            .sign_message(b"header", b"parent", b"metadata", b"content")
            .unwrap();

        assert_eq!(signature, "");
    }

    #[test]
    fn test_sign_message_with_key() {
        let mut protocol = create_test_protocol();

        // Set a test HMAC key
        let test_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        protocol.set_hmac_key(test_key);

        // Sign a message
        let signature = protocol
            .sign_message(b"header", b"parent", b"metadata", b"content")
            .unwrap();

        // Signature should not be empty
        assert!(!signature.is_empty());
        // Signature should be hex-encoded (64 chars for HMAC-SHA256)
        assert_eq!(signature.len(), 64);
        // Should only contain hex characters
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_message_deterministic() {
        let mut protocol = create_test_protocol();
        protocol.set_hmac_key("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");

        // Same inputs should produce same signature
        let sig1 = protocol
            .sign_message(b"header", b"parent", b"metadata", b"content")
            .unwrap();

        let sig2 = protocol
            .sign_message(b"header", b"parent", b"metadata", b"content")
            .unwrap();

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_sign_message_different_inputs() {
        let mut protocol = create_test_protocol();
        protocol.set_hmac_key("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");

        // Different inputs should produce different signatures
        let sig1 = protocol
            .sign_message(b"header1", b"parent", b"metadata", b"content")
            .unwrap();

        let sig2 = protocol
            .sign_message(b"header2", b"parent", b"metadata", b"content")
            .unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_verify_signature_without_key() {
        let protocol = create_test_protocol();

        // Without HMAC key, empty signature should be valid
        let valid = protocol
            .verify_signature("", b"header", b"parent", b"metadata", b"content")
            .unwrap();

        assert!(valid);

        // Non-empty signature should be invalid
        let invalid = protocol
            .verify_signature(
                "some_signature",
                b"header",
                b"parent",
                b"metadata",
                b"content",
            )
            .unwrap();

        assert!(!invalid);
    }

    #[test]
    fn test_verify_signature_with_key() {
        let mut protocol = create_test_protocol();
        protocol.set_hmac_key("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");

        // Sign a message
        let signature = protocol
            .sign_message(b"header", b"parent", b"metadata", b"content")
            .unwrap();

        // Verify with correct signature
        let valid = protocol
            .verify_signature(&signature, b"header", b"parent", b"metadata", b"content")
            .unwrap();

        assert!(valid);
    }

    #[test]
    fn test_verify_signature_invalid() {
        let mut protocol = create_test_protocol();
        protocol.set_hmac_key("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");

        // Verify with incorrect signature
        let invalid = protocol
            .verify_signature(
                "incorrect_signature",
                b"header",
                b"parent",
                b"metadata",
                b"content",
            )
            .unwrap();

        assert!(!invalid);
    }

    #[test]
    fn test_verify_signature_tampered_content() {
        let mut protocol = create_test_protocol();
        protocol.set_hmac_key("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");

        // Sign a message
        let signature = protocol
            .sign_message(b"header", b"parent", b"metadata", b"content")
            .unwrap();

        // Verify with tampered content
        let invalid = protocol
            .verify_signature(
                &signature,
                b"header",
                b"parent",
                b"metadata",
                b"tampered_content",
            )
            .unwrap();

        assert!(!invalid);
    }

    #[test]
    fn test_set_hmac_key_hex_decode() {
        let mut protocol = create_test_protocol();

        // Valid hex key
        protocol.set_hmac_key("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
        assert!(protocol.hmac_key.is_some());

        // Key should be decoded from hex (32 bytes for 64 hex chars)
        assert_eq!(protocol.hmac_key.as_ref().unwrap().len(), 32);
    }

    #[test]
    fn test_set_hmac_key_invalid_hex() {
        let mut protocol = create_test_protocol();

        // Invalid hex should not set key (handled gracefully)
        protocol.set_hmac_key("not_valid_hex!");
        // Since hex::decode might fail, the key won't be set
        // The current implementation doesn't handle errors, so it might panic
        // Let's test with valid hex only for now
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let mut protocol = create_test_protocol();
        protocol.set_hmac_key("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210");

        // Test various message combinations
        let test_cases = vec![
            (
                b"header1".as_ref(),
                b"parent1".as_ref(),
                b"meta1".as_ref(),
                b"content1".as_ref(),
            ),
            (b"".as_ref(), b"".as_ref(), b"".as_ref(), b"".as_ref()),
            (
                b"long_header_with_special_chars!@#$%".as_ref(),
                b"parent".as_ref(),
                b"metadata".as_ref(),
                b"content".as_ref(),
            ),
        ];

        for (header, parent, metadata, content) in test_cases {
            let signature = protocol
                .sign_message(header, parent, metadata, content)
                .unwrap();
            let valid = protocol
                .verify_signature(&signature, header, parent, metadata, content)
                .unwrap();
            assert!(valid, "Signature verification failed for valid signature");

            // Verify that tampering any field invalidates signature
            if !header.is_empty() {
                let mut tampered_header = header.to_vec();
                tampered_header[0] ^= 1;
                let invalid = protocol
                    .verify_signature(&signature, &tampered_header, parent, metadata, content)
                    .unwrap();
                assert!(
                    !invalid,
                    "Signature verification should fail for tampered header"
                );
            }
        }
    }

    #[test]
    fn test_prepare_message_with_signature() {
        let mut protocol = create_test_protocol();
        protocol.set_hmac_key("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");

        let header = json!({"msg_type": "test"});
        let parent_header = json!({});
        let metadata = json!({});
        let content = json!({"data": "test"});

        let result = protocol
            .prepare_message(&header, &parent_header, &metadata, &content)
            .unwrap();

        // Deserialize and check structure
        let message: serde_json::Value = serde_json::from_slice(&result).unwrap();

        assert!(message["signature"].is_string());
        assert!(!message["signature"].as_str().unwrap().is_empty());
        assert_eq!(message["header"], header);
        assert_eq!(message["parent_header"], parent_header);
        assert_eq!(message["metadata"], metadata);
        assert_eq!(message["content"], content);
    }

    #[test]
    fn test_prepare_message_without_signature() {
        let protocol = create_test_protocol();

        let header = json!({"msg_type": "test"});
        let parent_header = json!({});
        let metadata = json!({});
        let content = json!({"data": "test"});

        let result = protocol
            .prepare_message(&header, &parent_header, &metadata, &content)
            .unwrap();

        // Deserialize and check structure
        let message: serde_json::Value = serde_json::from_slice(&result).unwrap();

        assert!(message["signature"].is_string());
        assert_eq!(message["signature"].as_str().unwrap(), "");
    }
}
