//! Session Management System for Kernel
//!
//! This module provides comprehensive session lifecycle management, artifact storage,
//! and policy enforcement integrated with the kernel's execution flow.
//!
//! Migrated from llmspell-sessions crate, preserving sophisticated session capabilities
//! while integrating with kernel-specific message handling and state management.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

pub mod artifact;
pub mod events;
pub mod manager;
pub mod policies;
pub mod security;

// Re-export key types
pub use artifact::{ArtifactStorage, ArtifactType, SessionArtifact};
pub use events::{SessionEvent, SessionEventType};
pub use manager::SessionManager;
pub use policies::{RateLimitPolicy, SessionPolicy, TimeoutPolicy};
pub use security::{AccessControl, SessionSecurity};

/// Session identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new session ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get as string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session is active
    Active,
    /// Session is paused
    Paused,
    /// Session is being resumed
    Resuming,
    /// Session is archived
    Archived,
    /// Session has expired
    Expired,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session ID
    pub id: SessionId,
    /// Session status
    pub status: SessionStatus,
    /// Creation time
    pub created_at: SystemTime,
    /// Last activity time
    pub last_activity: SystemTime,
    /// Session tags
    pub tags: Vec<String>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// TTL for session expiration
    pub ttl: Option<Duration>,
    /// Owner/user ID for multi-tenant isolation
    pub owner: Option<String>,
}

impl SessionMetadata {
    /// Create new session metadata
    pub fn new(id: SessionId) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            status: SessionStatus::Active,
            created_at: now,
            last_activity: now,
            tags: Vec::new(),
            metadata: HashMap::new(),
            ttl: None,
            owner: None,
        }
    }

    /// Update last activity time
    pub fn touch(&mut self) {
        self.last_activity = SystemTime::now();
    }

    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            if let Ok(elapsed) = self.last_activity.elapsed() {
                return elapsed > ttl;
            }
        }
        false
    }

    /// Set session owner for multi-tenant isolation
    pub fn set_owner(&mut self, owner: String) {
        self.owner = Some(owner);
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Enable artifact storage
    pub enable_artifacts: bool,
    /// Enable event tracking
    pub enable_events: bool,
    /// Enable security policies
    pub enable_security: bool,
    /// Maximum artifacts per session
    pub max_artifacts: usize,
    /// Session TTL
    pub session_ttl: Option<Duration>,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enable_artifacts: true,
            enable_events: true,
            enable_security: true,
            max_artifacts: 1000,
            session_ttl: Some(Duration::from_secs(3600)), // 1 hour default
            rate_limit: Some(RateLimitConfig::default()),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60), // 100 requests per minute
        }
    }
}

/// Session metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total messages processed
    pub messages_processed: u64,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Number of artifacts
    pub artifact_count: usize,
    /// Number of errors
    pub error_count: u64,
    /// Memory usage bytes
    pub memory_bytes: u64,
}

/// Complete session with all components
#[derive(Debug, Clone)]
pub struct Session {
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Session configuration
    pub config: SessionConfig,
    /// Session metrics
    pub metrics: SessionMetrics,
    /// Session artifacts
    pub artifacts: Vec<SessionArtifact>,
    /// Active policies
    pub policies: Vec<Arc<dyn SessionPolicy>>,
}

impl Session {
    /// Create a new session
    pub fn new(id: SessionId, config: SessionConfig) -> Self {
        Self {
            metadata: SessionMetadata::new(id),
            config,
            metrics: SessionMetrics::default(),
            artifacts: Vec::new(),
            policies: Vec::new(),
        }
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.metadata.status == SessionStatus::Active && !self.metadata.is_expired()
    }

    /// Pause the session
    pub fn pause(&mut self) {
        self.metadata.status = SessionStatus::Paused;
    }

    /// Resume the session
    pub fn resume(&mut self) {
        self.metadata.status = SessionStatus::Active;
        self.metadata.touch();
    }

    /// Archive the session
    pub fn archive(&mut self) {
        self.metadata.status = SessionStatus::Archived;
    }

    /// Add an artifact to the session
    ///
    /// # Errors
    ///
    /// Returns an error if the maximum artifacts limit is reached
    pub fn add_artifact(&mut self, artifact: SessionArtifact) -> Result<()> {
        if self.artifacts.len() >= self.config.max_artifacts {
            return Err(anyhow::anyhow!("Maximum artifacts limit reached"));
        }
        self.artifacts.push(artifact);
        self.metrics.artifact_count = self.artifacts.len();
        Ok(())
    }

    /// Update session metrics
    pub fn update_metrics<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut SessionMetrics),
    {
        updater(&mut self.metrics);
    }
}

/// Integration point with kernel message flow
pub trait KernelSessionIntegration {
    /// Handle kernel message in session context
    ///
    /// # Errors
    ///
    /// Returns an error if message handling fails
    fn handle_kernel_message(&mut self, msg: serde_json::Value) -> Result<()>;

    /// Apply session policies to message
    ///
    /// # Errors
    ///
    /// Returns an error if policy evaluation fails
    fn apply_policies(&self, msg: &serde_json::Value) -> Result<bool>;

    /// Track message for correlation
    fn track_message(&mut self, msg: &serde_json::Value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(SessionId::new(), SessionConfig::default());
        assert!(session.is_active());
        assert_eq!(session.metadata.status, SessionStatus::Active);
    }

    #[test]
    fn test_session_lifecycle() {
        let mut session = Session::new(SessionId::new(), SessionConfig::default());

        // Pause
        session.pause();
        assert_eq!(session.metadata.status, SessionStatus::Paused);

        // Resume
        session.resume();
        assert_eq!(session.metadata.status, SessionStatus::Active);

        // Archive
        session.archive();
        assert_eq!(session.metadata.status, SessionStatus::Archived);
    }

    #[test]
    fn test_session_expiration() {
        let mut metadata = SessionMetadata::new(SessionId::new());
        metadata.ttl = Some(Duration::from_millis(1));

        std::thread::sleep(Duration::from_millis(2));
        assert!(metadata.is_expired());
    }
}
