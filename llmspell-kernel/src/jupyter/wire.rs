//! Jupyter wire protocol implementation
//!
//! Handles the wire format for Jupyter messages including:
//! - Message serialization/deserialization
//! - HMAC signature generation and validation
//! - Identity frame handling for routing
//! - Delimiter handling (<IDS|MSG>)

use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use super::protocol::{JupyterMessage, MessageContent, MessageHeader};

/// Jupyter wire message format
///
/// Wire format: [identities, <IDS|MSG>, hmac, header, `parent_header`, metadata, content]
pub struct WireMessage {
    pub identities: Vec<Vec<u8>>,
    pub header: Vec<u8>,
    pub parent_header: Vec<u8>,
    pub metadata: Vec<u8>,
    pub content: Vec<u8>,
    pub hmac: Vec<u8>,
}

/// Handles Jupyter wire protocol encoding/decoding
pub struct WireProtocol {
    signing_key: String,
}

impl WireProtocol {
    /// Create new wire protocol handler with signing key
    #[must_use]
    pub const fn new(signing_key: String) -> Self {
        Self { signing_key }
    }

    /// Decode multipart ZMQ message into `JupyterMessage`
    ///
    /// # Errors
    ///
    /// Returns an error if the message format is invalid or HMAC verification fails.
    pub fn decode_message(&self, parts: &[Vec<u8>], channel: &str) -> Result<JupyterMessage> {
        tracing::debug!("Decoding {} parts on {} channel", parts.len(), channel);

        if parts.len() < 4 {
            return Err(anyhow::anyhow!(
                "Invalid message format on {} channel",
                channel
            ));
        }

        // Find the delimiter (<IDS|MSG>)
        let mut delim_idx = None;
        for (i, part) in parts.iter().enumerate() {
            if part == b"<IDS|MSG>" {
                delim_idx = Some(i);
                break;
            }
        }

        let Some(delim_idx) = delim_idx else {
            return Err(anyhow::anyhow!(
                "No delimiter found in message on {} channel",
                channel
            ));
        };

        if delim_idx + 5 >= parts.len() {
            return Err(anyhow::anyhow!("Incomplete message on {} channel", channel));
        }

        // Extract identities (everything before delimiter)
        let identities: Vec<Vec<u8>> = parts[..delim_idx].to_vec();

        // Extract message parts after delimiter
        let received_hmac = &parts[delim_idx + 1];
        let header = &parts[delim_idx + 2];
        let parent_header = &parts[delim_idx + 3];
        let metadata = &parts[delim_idx + 4];
        let content = &parts[delim_idx + 5];

        // Verify HMAC signature
        let expected_hmac = self.calculate_hmac(&[header, parent_header, metadata, content]);
        if !self.verify_hmac_signature(received_hmac, &expected_hmac) {
            return Err(anyhow::anyhow!(
                "HMAC signature verification failed on {} channel",
                channel
            ));
        }

        // Deserialize header
        let header: MessageHeader =
            serde_json::from_slice(header).context("Failed to deserialize header")?;

        // Handle parent_header - Jupyter sends {} for empty parent
        let parent_header: Option<MessageHeader> =
            if parent_header.is_empty() || parent_header == b"{}" {
                None
            } else {
                Some(
                    serde_json::from_slice(parent_header)
                        .context("Failed to deserialize parent_header")?,
                )
            };

        // Deserialize metadata
        let mut metadata: Value =
            serde_json::from_slice(metadata).context("Failed to deserialize metadata")?;

        // Store identities in metadata for reply routing (temporary hack)
        if identities.is_empty() {
            tracing::warn!(
                "No identities found in received message on {} channel",
                channel
            );
        } else {
            let hex_identities: Vec<String> = identities.iter().map(hex::encode).collect();
            tracing::info!(
                "Storing {} identities for reply on {} channel",
                hex_identities.len(),
                channel
            );
            metadata["__identities"] = serde_json::json!(hex_identities);
        }

        // Deserialize content based on msg_type
        let content = Self::deserialize_content(&header.msg_type, content)
            .context("Failed to deserialize content")?;

        Ok(JupyterMessage {
            header,
            parent_header,
            metadata,
            content,
        })
    }

    /// Encode `JupyterMessage` into multipart ZMQ message
    ///
    /// # Errors
    ///
    /// Returns an error if message serialization or encoding fails.
    pub fn encode_message(&self, msg: &JupyterMessage, channel: &str) -> Result<Vec<Vec<u8>>> {
        let mut parts = Vec::new();

        // IOPub is a PUB socket - no identities needed
        if channel != "iopub" {
            // Extract and add identities for ROUTER sockets
            if let Some(identities) = msg.metadata.get("__identities") {
                if let Some(id_array) = identities.as_array() {
                    tracing::info!(
                        "Encoding with {} identities for {} channel",
                        id_array.len(),
                        channel
                    );
                    for id in id_array {
                        if let Some(id_str) = id.as_str() {
                            if let Ok(id_bytes) = hex::decode(id_str) {
                                parts.push(id_bytes);
                            }
                        }
                    }
                }
            }

            // Must have identities for ROUTER sockets (shell, control)
            if parts.is_empty() {
                tracing::error!("No identities for ROUTER socket on {} channel", channel);
                return Err(anyhow::anyhow!(
                    "No routing identity available for reply on {} channel",
                    channel
                ));
            }
        }

        // Add delimiter
        parts.push(b"<IDS|MSG>".to_vec());

        // Serialize message components
        let header = serde_json::to_vec(&msg.header).context("Failed to serialize header")?;
        let parent_header = match &msg.parent_header {
            Some(ph) => serde_json::to_vec(ph).context("Failed to serialize parent_header")?,
            None => b"{}".to_vec(), // Empty JSON object for Jupyter compatibility
        };

        // Filter out __identities from metadata before sending
        let mut clean_metadata = msg.metadata.clone();
        if let Some(obj) = clean_metadata.as_object_mut() {
            obj.remove("__identities");
        }
        let metadata =
            serde_json::to_vec(&clean_metadata).context("Failed to serialize metadata")?;
        let content = serde_json::to_vec(&msg.content).context("Failed to serialize content")?;

        // Calculate HMAC
        let hmac = self.calculate_hmac(&[&header, &parent_header, &metadata, &content]);

        // Add message parts in order
        parts.push(hmac);
        parts.push(header);
        parts.push(parent_header);
        parts.push(metadata);
        parts.push(content);

        tracing::info!("Encoded {} parts for {} channel", parts.len(), channel);
        Ok(parts)
    }

    /// Calculate HMAC-SHA256 signature for message authentication
    pub fn calculate_hmac(&self, parts: &[&[u8]]) -> Vec<u8> {
        // Use the signing key as raw ASCII bytes (Jupyter convention)
        let key_bytes = self.signing_key.as_bytes();

        // Create HMAC instance
        let mut mac = match Hmac::<Sha256>::new_from_slice(key_bytes) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Failed to create HMAC: {}, using empty signature", e);
                return vec![];
            }
        };

        // Update with all message parts
        for part in parts {
            mac.update(part);
        }

        // Return signature as hex string bytes
        let result = mac.finalize();
        hex::encode(result.into_bytes()).into_bytes()
    }

    /// Verify HMAC signature using constant-time comparison
    #[must_use]
    pub fn verify_hmac_signature(&self, received: &[u8], expected: &[u8]) -> bool {
        // Convert to strings for comparison (Jupyter uses hex-encoded HMAC)
        let received_str = String::from_utf8_lossy(received);
        let expected_str = String::from_utf8_lossy(expected);

        // Use constant-time comparison to prevent timing attacks

        if received_str.len() != expected_str.len() {
            return false;
        }

        // Compare hex strings in constant time
        received_str
            .as_bytes()
            .ct_eq(expected_str.as_bytes())
            .into()
    }

    /// Deserialize content based on message type
    fn deserialize_content(msg_type: &str, content_bytes: &[u8]) -> Result<MessageContent> {
        // For most request messages, the content structure varies
        match msg_type {
            "kernel_info_request" => Ok(MessageContent::KernelInfoRequest {}),
            "execute_request" => {
                let value: serde_json::Value = serde_json::from_slice(content_bytes)?;
                let code = value["code"].as_str().unwrap_or("").to_string();
                let silent = value["silent"].as_bool().unwrap_or(false);
                let store_history = value
                    .get("store_history")
                    .and_then(serde_json::Value::as_bool);
                let user_expressions = None; // TODO: Parse if needed
                let allow_stdin = value
                    .get("allow_stdin")
                    .and_then(serde_json::Value::as_bool);
                let stop_on_error = value
                    .get("stop_on_error")
                    .and_then(serde_json::Value::as_bool);

                Ok(MessageContent::ExecuteRequest {
                    code,
                    silent,
                    store_history,
                    user_expressions,
                    allow_stdin,
                    stop_on_error,
                })
            }
            "shutdown_request" => {
                let value: serde_json::Value = serde_json::from_slice(content_bytes)?;
                let restart = value["restart"].as_bool().unwrap_or(false);
                Ok(MessageContent::ShutdownRequest { restart })
            }
            "interrupt_request" => Ok(MessageContent::InterruptRequest {}),
            "comm_open" => {
                let value: serde_json::Value = serde_json::from_slice(content_bytes)?;
                Ok(MessageContent::CommOpen {
                    comm_id: value["comm_id"].as_str().unwrap_or("").to_string(),
                    target_name: value["target_name"].as_str().unwrap_or("").to_string(),
                    data: value
                        .get("data")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({})),
                    metadata: value
                        .get("metadata")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                })
            }
            "comm_msg" => {
                let value: serde_json::Value = serde_json::from_slice(content_bytes)?;
                Ok(MessageContent::CommMsg {
                    comm_id: value["comm_id"].as_str().unwrap_or("").to_string(),
                    data: value
                        .get("data")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({})),
                    metadata: value
                        .get("metadata")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                })
            }
            "comm_close" => {
                let value: serde_json::Value = serde_json::from_slice(content_bytes)?;
                Ok(MessageContent::CommClose {
                    comm_id: value["comm_id"].as_str().unwrap_or("").to_string(),
                    data: value.get("data").cloned(),
                    metadata: value
                        .get("metadata")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                })
            }
            "comm_info_request" => {
                let value: serde_json::Value = serde_json::from_slice(content_bytes)?;
                Ok(MessageContent::CommInfoRequest {
                    target_name: value
                        .get("target_name")
                        .and_then(|v| v.as_str())
                        .map(std::string::ToString::to_string),
                })
            }
            // Add more message types as needed
            _ => {
                // For unknown types, try generic deserialization
                serde_json::from_slice(content_bytes)
                    .with_context(|| format!("Unknown message type: {msg_type}"))
            }
        }
    }
}
