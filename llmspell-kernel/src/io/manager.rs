//! Enhanced I/O Manager with Multi-Channel Routing
//!
//! This module provides the core I/O management functionality for the kernel,
//! handling stdout/stderr capture and routing to appropriate Jupyter channels
//! with parent header tracking for message correlation.

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tracing::{debug, instrument, trace, warn};
use uuid::Uuid;

use crate::runtime::tracing::TracingInstrumentation;

/// I/O stream type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamType {
    /// Standard output stream
    Stdout,
    /// Standard error stream
    Stderr,
}

impl StreamType {
    /// Get stream name for Jupyter protocol
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

/// Message header for correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Message ID
    pub msg_id: String,
    /// Session ID
    pub session: String,
    /// Username
    pub username: String,
    /// Message type
    pub msg_type: String,
    /// Protocol version
    pub version: String,
    /// Timestamp
    pub date: String,
}

impl MessageHeader {
    /// Create a new message header
    pub fn new(msg_type: &str, session: &str) -> Self {
        Self {
            msg_id: Uuid::new_v4().to_string(),
            session: session.to_string(),
            username: "kernel".to_string(),
            msg_type: msg_type.to_string(),
            version: crate::PROTOCOL_VERSION.to_string(),
            date: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// `IOPub` message for output publishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOPubMessage {
    /// Parent header for correlation
    pub parent_header: Option<MessageHeader>,
    /// Message header
    pub header: MessageHeader,
    /// Message metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Message content
    pub content: HashMap<String, serde_json::Value>,
}

/// Configuration for I/O manager
#[derive(Debug, Clone)]
pub struct IOConfig {
    /// Buffer size for stdout
    pub stdout_buffer_size: usize,
    /// Buffer size for stderr
    pub stderr_buffer_size: usize,
    /// Flush interval in milliseconds
    pub flush_interval_ms: u64,
    /// Enable parent header tracking
    pub track_parent_headers: bool,
}

impl Default for IOConfig {
    fn default() -> Self {
        Self {
            stdout_buffer_size: 8192,
            stderr_buffer_size: 8192,
            flush_interval_ms: 100,
            track_parent_headers: true,
        }
    }
}

use crate::events::correlation::{KernelEvent, KernelEventCorrelator};

/// Enhanced I/O Manager for multi-channel routing
pub struct EnhancedIOManager {
    /// `IOPub` channel sender
    iopub_sender: Option<Sender<IOPubMessage>>,
    /// Stdout buffer
    stdout_buffer: Arc<RwLock<String>>,
    /// Stderr buffer
    stderr_buffer: Arc<RwLock<String>>,
    /// Parent headers for message correlation
    parent_headers: Arc<RwLock<HashMap<String, MessageHeader>>>,
    /// Current execution parent header
    current_parent: Arc<RwLock<Option<MessageHeader>>>,
    /// Event correlator for direct broadcasting
    event_correlator: Option<Arc<KernelEventCorrelator>>,
    /// Configuration
    config: IOConfig,
    /// Session ID
    session_id: String,
    /// Tracing instrumentation
    tracing: Option<TracingInstrumentation>,
}

impl EnhancedIOManager {
    /// Create a new enhanced I/O manager
    #[instrument(level = "debug", skip_all)]
    pub fn new(config: IOConfig, session_id: String) -> Self {
        debug!("Creating EnhancedIOManager for session {}", session_id);

        Self {
            iopub_sender: None,
            stdout_buffer: Arc::new(RwLock::new(String::with_capacity(
                config.stdout_buffer_size,
            ))),
            stderr_buffer: Arc::new(RwLock::new(String::with_capacity(
                config.stderr_buffer_size,
            ))),
            parent_headers: Arc::new(RwLock::new(HashMap::new())),
            current_parent: Arc::new(RwLock::new(None)),
            event_correlator: None,
            config,
            session_id,
            tracing: None,
        }
    }

    /// Set tracing instrumentation
    pub fn set_tracing(&mut self, tracing: TracingInstrumentation) {
        self.tracing = Some(tracing);
    }

    /// Set event correlator for direct broadcasting
    pub fn set_event_correlator(&mut self, correlator: Arc<KernelEventCorrelator>) {
        self.event_correlator = Some(correlator);
    }

    /// Set the `IOPub` channel sender
    pub fn set_iopub_sender(&mut self, sender: Sender<IOPubMessage>) {
        self.iopub_sender = Some(sender);
    }

    /// Set the current parent header for message correlation
    #[instrument(level = "trace", skip(self))]
    pub fn set_parent_header(&self, parent: MessageHeader) {
        if self.config.track_parent_headers {
            trace!("Setting parent header: msg_id={}", parent.msg_id);
            *self.current_parent.write() = Some(parent.clone());
            self.parent_headers
                .write()
                .insert(parent.msg_id.clone(), parent);
        }
    }

    /// Clear the current parent header
    pub fn clear_parent_header(&self) {
        *self.current_parent.write() = None;
    }

    /// Write to stdout
    ///
    /// # Errors
    ///
    /// Returns an error if the write operation fails
    #[instrument(level = "trace", skip(self, data))]
    pub async fn write_stdout(&self, data: &str) -> Result<()> {
        self.write_stream(StreamType::Stdout, data).await
    }

    /// Write to stderr
    ///
    /// # Errors
    ///
    /// Returns an error if the write operation fails
    #[instrument(level = "trace", skip(self, data))]
    pub async fn write_stderr(&self, data: &str) -> Result<()> {
        self.write_stream(StreamType::Stderr, data).await
    }

    /// Write to a specific stream
    async fn write_stream(&self, stream_type: StreamType, data: &str) -> Result<()> {
        // Buffer the data
        let buffer = match stream_type {
            StreamType::Stdout => &self.stdout_buffer,
            StreamType::Stderr => &self.stderr_buffer,
        };

        buffer.write().push_str(data);

        // Check if we should flush (contains newline or buffer is large)
        if data.contains('\n') || buffer.read().len() > self.config.stdout_buffer_size / 2 {
            self.flush_stream(stream_type).await?;
        }

        Ok(())
    }

    /// Flush a specific stream
    ///
    /// # Errors
    ///
    /// Returns an error if the flush operation fails
    #[instrument(level = "debug", skip(self))]
    pub async fn flush_stream(&self, stream_type: StreamType) -> Result<()> {
        let buffer = match stream_type {
            StreamType::Stdout => &self.stdout_buffer,
            StreamType::Stderr => &self.stderr_buffer,
        };

        let content = {
            let mut buf = buffer.write();
            if buf.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *buf)
        };

        // Send to IOPub channel
        self.publish_stream(stream_type, &content).await
    }

    /// Flush all streams
    ///
    /// # Errors
    ///
    /// Returns an error if any flush operation fails
    pub async fn flush_all(&self) -> Result<()> {
        self.flush_stream(StreamType::Stdout).await?;
        self.flush_stream(StreamType::Stderr).await?;
        Ok(())
    }

    /// Publish stream output to `IOPub` channel
    #[instrument(level = "trace", skip(self, text))]
    async fn publish_stream(&self, stream_type: StreamType, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        let Some(ref sender) = self.iopub_sender else {
            debug!("No IOPub sender configured, dropping output");
            return Ok(());
        };

        // Create stream message
        let header = MessageHeader::new("stream", &self.session_id);
        let parent_header = self.current_parent.read().clone();

        let mut content = HashMap::new();
        content.insert(
            "name".to_string(),
            serde_json::Value::String(stream_type.as_str().to_string()),
        );
        content.insert(
            "text".to_string(),
            serde_json::Value::String(text.to_string()),
        );

        let message = IOPubMessage {
            parent_header,
            header,
            metadata: HashMap::new(),
            content,
        };

        // Trace the operation
        if let Some(ref tracing) = self.tracing {
            tracing.trace_transport_operation("jupyter", "iopub", "stream");
        }

        // Broadcast via Correlator if available (Direct Path)
        if let Some(ref correlator) = self.event_correlator {
            let event = KernelEvent::IOPubMessage(message.clone());
            if let Err(e) = correlator.track_event(event).await {
                warn!("Failed to track stream event via correlator: {}", e);
            }
        }

        // Send message to channel (for IntegratedKernel loop processing/logging)
        sender
            .send(message)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send to IOPub channel: {e}"))?;

        trace!(
            "Published {} bytes to {} stream",
            text.len(),
            stream_type.as_str()
        );
        Ok(())
    }

    /// Publish a status update
    ///
    /// # Errors
    ///
    /// Returns an error if publishing fails
    #[instrument(level = "debug", skip(self))]
    pub async fn publish_status(&self, status: &str) -> Result<()> {
        let Some(ref sender) = self.iopub_sender else {
            return Ok(());
        };

        let header = MessageHeader::new("status", &self.session_id);
        let parent_header = self.current_parent.read().clone();

        let mut content = HashMap::new();
        content.insert(
            "execution_state".to_string(),
            serde_json::Value::String(status.to_string()),
        );

        let message = IOPubMessage {
            parent_header,
            header,
            metadata: HashMap::new(),
            content,
        };

        // Broadcast via Correlator if available (Direct Path)
        if let Some(ref correlator) = self.event_correlator {
            let event = KernelEvent::IOPubMessage(message.clone());
            if let Err(e) = correlator.track_event(event).await {
                warn!("Failed to track status event via correlator: {}", e);
            }
        }

        // Use try_send to avoid blocking and handle channel full/closed gracefully
        match sender.try_send(message) {
            Ok(()) => {
                debug!("Published status: {}", status);
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("IOPub channel full, dropping status message: {}", status);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                // Channel closed is not fatal during startup/shutdown
                debug!("IOPub channel closed, ignoring status: {}", status);
            }
        }

        Ok(())
    }

    /// Publish an execute result
    ///
    /// # Errors
    ///
    /// Returns an error if publishing fails
    #[instrument(level = "debug", skip(self, data))]
    pub async fn publish_execute_result(
        &self,
        execution_count: i32,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let Some(ref sender) = self.iopub_sender else {
            return Ok(());
        };

        let header = MessageHeader::new("execute_result", &self.session_id);
        let parent_header = self.current_parent.read().clone();

        let mut content = HashMap::new();
        content.insert(
            "execution_count".to_string(),
            serde_json::Value::Number(execution_count.into()),
        );
        content.insert(
            "data".to_string(),
            serde_json::Value::Object(data.into_iter().collect()),
        );
        content.insert(
            "metadata".to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        );

        let message = IOPubMessage {
            parent_header,
            header,
            metadata: HashMap::new(),
            content,
        };

        // Broadcast via Correlator if available (Direct Path)
        if let Some(ref correlator) = self.event_correlator {
            let event = KernelEvent::IOPubMessage(message.clone());
            if let Err(e) = correlator.track_event(event).await {
                warn!("Failed to track execute_result event via correlator: {}", e);
            }
        }

        sender
            .send(message)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send execute result: {e}"))?;

        debug!("Published execute result for execution {}", execution_count);
        Ok(())
    }

    /// Publish display data to `IOPub` channel
    ///
    /// # Errors
    ///
    /// Returns an error if publishing fails
    #[instrument(level = "debug", skip(self, data))]
    pub async fn publish_display_data(
        &self,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let Some(ref sender) = self.iopub_sender else {
            return Ok(());
        };

        let header = MessageHeader::new("display_data", &self.session_id);
        let parent_header = self.current_parent.read().clone();

        let mut content = HashMap::new();
        content.insert(
            "data".to_string(),
            serde_json::Value::Object(data.into_iter().collect()),
        );
        content.insert(
            "metadata".to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        );

        let message = IOPubMessage {
            parent_header,
            header,
            metadata: HashMap::new(),
            content,
        };

        // Broadcast via Correlator if available (Direct Path)
        if let Some(ref correlator) = self.event_correlator {
            let event = KernelEvent::IOPubMessage(message.clone());
            if let Err(e) = correlator.track_event(event).await {
                warn!("Failed to track display_data event via correlator: {}", e);
            }
        }

        sender
            .send(message)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send display data: {e}"))?;

        debug!("Published display data");
        Ok(())
    }

    /// Create an `IOPub` channel
    pub fn create_iopub_channel(&mut self) -> mpsc::Receiver<IOPubMessage> {
        let (tx, rx) = mpsc::channel(100);
        self.iopub_sender = Some(tx);
        rx
    }

    /// Get parent header by message ID
    pub fn get_parent_header(&self, msg_id: &str) -> Option<MessageHeader> {
        self.parent_headers.read().get(msg_id).cloned()
    }

    /// Clean up old parent headers
    #[instrument(level = "debug", skip(self))]
    pub fn cleanup_parent_headers(&self, keep_recent: usize) {
        let mut headers = self.parent_headers.write();
        if headers.len() > keep_recent * 2 {
            // Simple cleanup: remove oldest entries
            let to_remove = headers.len() - keep_recent;
            let keys: Vec<String> = headers.keys().take(to_remove).cloned().collect();
            for key in keys {
                headers.remove(&key);
            }
            debug!("Cleaned up {} parent headers", to_remove);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_io_manager_creation() {
        let config = IOConfig::default();
        let manager = EnhancedIOManager::new(config, "test-session".to_string());

        assert!(manager.iopub_sender.is_none());
        assert!(manager.current_parent.read().is_none());
    }

    #[tokio::test]
    async fn test_stream_buffering() {
        let config = IOConfig::default();
        let mut manager = EnhancedIOManager::new(config, "test-session".to_string());

        // Create channel
        let mut rx = manager.create_iopub_channel();

        // Write without newline - should buffer
        manager.write_stdout("Hello").await.unwrap();

        // Buffer should contain data
        assert_eq!(manager.stdout_buffer.read().as_str(), "Hello");

        // Write with newline - should flush
        manager.write_stdout(" World\n").await.unwrap();

        // Buffer should be empty after flush
        assert!(manager.stdout_buffer.read().is_empty());

        // Should receive message
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg.header.msg_type, "stream");
        let text = msg.content.get("text").unwrap().as_str().unwrap();
        assert_eq!(text, "Hello World\n");
    }

    #[tokio::test]
    async fn test_parent_header_tracking() {
        let config = IOConfig {
            track_parent_headers: true,
            ..Default::default()
        };

        let manager = EnhancedIOManager::new(config, "test-session".to_string());

        let parent = MessageHeader::new("execute_request", "test-session");
        let parent_id = parent.msg_id.clone();

        manager.set_parent_header(parent);

        // Should be set as current
        assert!(manager.current_parent.read().is_some());

        // Should be stored
        assert!(manager.get_parent_header(&parent_id).is_some());

        // Clear should remove current but not stored
        manager.clear_parent_header();
        assert!(manager.current_parent.read().is_none());
        assert!(manager.get_parent_header(&parent_id).is_some());
    }

    #[tokio::test]
    async fn test_status_publishing() {
        let config = IOConfig::default();
        let mut manager = EnhancedIOManager::new(config, "test-session".to_string());

        let mut rx = manager.create_iopub_channel();

        manager.publish_status("busy").await.unwrap();

        let msg = rx.recv().await.unwrap();
        assert_eq!(msg.header.msg_type, "status");
        assert_eq!(
            msg.content
                .get("execution_state")
                .unwrap()
                .as_str()
                .unwrap(),
            "busy"
        );
    }
}
