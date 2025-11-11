//! Session storage types
//!
//! Types for persistent session data with expiration tracking and artifact references.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session status
///
/// Tracks the lifecycle of a session from active to terminal states.
///
/// # State Transitions
///
/// ```text
/// Active → Completed
///       ↘ Expired
/// ```
///
/// # Examples
///
/// ```
/// use llmspell_core::types::storage::SessionStatus;
///
/// let status = SessionStatus::Active;
/// assert_eq!(status.is_active(), true);
///
/// let status = SessionStatus::Expired;
/// assert_eq!(status.is_active(), false);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is active and can be used
    Active,
    /// Session completed normally
    Completed,
    /// Session expired (past `expires_at` timestamp)
    Expired,
}

impl SessionStatus {
    /// Check if session is active
    ///
    /// # Returns
    ///
    /// `true` if status is Active, `false` otherwise
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if session is terminal (completed or expired)
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Expired)
    }
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

/// Persistent session data
///
/// Stores session state including metadata, expiration, and artifact tracking.
/// The `session_data` field contains session-specific state as JSON.
///
/// # Expiration
///
/// Sessions can have optional expiration times. When `expires_at` is set,
/// the session should be marked as Expired after that time.
///
/// # Artifact Tracking
///
/// The `artifact_count` field tracks the number of artifacts associated
/// with this session for cleanup and quota management.
///
/// # Examples
///
/// ```
/// use llmspell_core::types::storage::{SessionData, SessionStatus};
/// use chrono::Utc;
/// use serde_json::json;
///
/// let data = SessionData {
///     session_id: "sess-123".to_string(),
///     status: SessionStatus::Active,
///     session_data: json!({"user_id": "user-456", "context": "chat"}),
///     created_at: Utc::now(),
///     expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
///     artifact_count: 5,
/// };
///
/// assert_eq!(data.status, SessionStatus::Active);
/// assert!(data.expires_at.is_some());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Unique session identifier
    pub session_id: String,

    /// Current session status
    pub status: SessionStatus,

    /// Session-specific data (stored as JSONB in database)
    ///
    /// This field contains all session-specific context such as user preferences,
    /// conversation history, or application state.
    pub session_data: serde_json::Value,

    /// Timestamp when session was created
    pub created_at: DateTime<Utc>,

    /// Optional expiration timestamp
    ///
    /// When set, the session should be marked as Expired after this time.
    /// `None` means the session never expires.
    pub expires_at: Option<DateTime<Utc>>,

    /// Number of artifacts associated with this session
    ///
    /// Used for cleanup coordination and quota tracking.
    pub artifact_count: usize,
}

impl SessionData {
    /// Create a new session in Active status
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the session
    ///
    /// # Examples
    ///
    /// ```
    /// use llmspell_core::types::storage::SessionData;
    ///
    /// let data = SessionData::new("sess-123");
    /// assert_eq!(data.session_id, "sess-123");
    /// assert_eq!(data.artifact_count, 0);
    /// ```
    #[must_use]
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            status: SessionStatus::Active,
            session_data: serde_json::Value::Object(serde_json::Map::new()),
            created_at: Utc::now(),
            expires_at: None,
            artifact_count: 0,
        }
    }

    /// Create session with expiration time
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier
    /// * `expires_at` - Expiration timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use llmspell_core::types::storage::SessionData;
    /// use chrono::Utc;
    ///
    /// let expires = Utc::now() + chrono::Duration::hours(1);
    /// let data = SessionData::with_expiration("sess-123", expires);
    /// assert!(data.expires_at.is_some());
    /// ```
    #[must_use]
    pub fn with_expiration(session_id: impl Into<String>, expires_at: DateTime<Utc>) -> Self {
        Self {
            session_id: session_id.into(),
            status: SessionStatus::Active,
            session_data: serde_json::Value::Object(serde_json::Map::new()),
            created_at: Utc::now(),
            expires_at: Some(expires_at),
            artifact_count: 0,
        }
    }

    /// Check if session has expired
    ///
    /// # Returns
    ///
    /// `true` if `expires_at` is set and current time is past it
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Mark session as expired if past expiration time
    ///
    /// Updates status to Expired if current time is past `expires_at`.
    /// No-op if no expiration time set or not yet expired.
    pub fn check_and_mark_expired(&mut self) {
        if self.is_expired() && self.status == SessionStatus::Active {
            self.status = SessionStatus::Expired;
        }
    }

    /// Increment artifact count
    pub fn increment_artifacts(&mut self) {
        self.artifact_count = self.artifact_count.saturating_add(1);
    }

    /// Decrement artifact count
    pub fn decrement_artifacts(&mut self) {
        self.artifact_count = self.artifact_count.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_session_status() {
        assert!(SessionStatus::Active.is_active());
        assert!(!SessionStatus::Completed.is_active());
        assert!(!SessionStatus::Expired.is_active());

        assert!(SessionStatus::Completed.is_terminal());
        assert!(SessionStatus::Expired.is_terminal());
        assert!(!SessionStatus::Active.is_terminal());
    }

    #[test]
    fn test_session_data_new() {
        let data = SessionData::new("sess-1");
        assert_eq!(data.status, SessionStatus::Active);
        assert_eq!(data.artifact_count, 0);
        assert!(data.expires_at.is_none());
    }

    #[test]
    fn test_session_expiration() {
        // Not expired
        let future = Utc::now() + Duration::hours(1);
        let data = SessionData::with_expiration("sess-1", future);
        assert!(!data.is_expired());

        // Expired
        let past = Utc::now() - Duration::hours(1);
        let data = SessionData::with_expiration("sess-1", past);
        assert!(data.is_expired());
    }

    #[test]
    fn test_check_and_mark_expired() {
        let past = Utc::now() - Duration::hours(1);
        let mut data = SessionData::with_expiration("sess-1", past);
        assert_eq!(data.status, SessionStatus::Active);

        data.check_and_mark_expired();
        assert_eq!(data.status, SessionStatus::Expired);
    }

    #[test]
    fn test_artifact_count() {
        let mut data = SessionData::new("sess-1");
        assert_eq!(data.artifact_count, 0);

        data.increment_artifacts();
        assert_eq!(data.artifact_count, 1);

        data.increment_artifacts();
        assert_eq!(data.artifact_count, 2);

        data.decrement_artifacts();
        assert_eq!(data.artifact_count, 1);
    }
}
