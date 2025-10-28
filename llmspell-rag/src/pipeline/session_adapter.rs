//! Session RAG Adapter - wraps `SessionAwareRAGPipeline` to implement `RAGRetriever` trait
//!
//! Provides session-agnostic interface by extracting session from `StateScope`.

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use llmspell_core::state::StateScope;
use llmspell_kernel::sessions::SessionId;
use std::sync::Arc;
use tracing::{debug, warn};

use super::rag_trait::{RAGRetriever, RAGResult};
use crate::session_integration::{SessionAwareRAGPipeline, SessionVectorResult};

/// Adapter wrapping `SessionAwareRAGPipeline` to implement `RAGRetriever` trait
///
/// Extracts session context from `StateScope` and converts session-specific
/// results to generic `RAGResult` format.
#[derive(Debug)]
pub struct SessionRAGAdapter {
    /// Wrapped session-aware pipeline
    inner: Arc<SessionAwareRAGPipeline>,
    /// Default session ID used when scope doesn't specify one
    default_session: SessionId,
}

impl SessionRAGAdapter {
    /// Create new adapter
    #[must_use]
    pub fn new(inner: Arc<SessionAwareRAGPipeline>, default_session: SessionId) -> Self {
        debug!(
            "Creating SessionRAGAdapter with default_session={}",
            default_session
        );
        Self {
            inner,
            default_session,
        }
    }

    /// Get reference to wrapped pipeline
    #[must_use]
    pub const fn inner(&self) -> &Arc<SessionAwareRAGPipeline> {
        &self.inner
    }
}

#[async_trait]
impl RAGRetriever for SessionRAGAdapter {
    async fn retrieve(
        &self,
        query: &str,
        k: usize,
        scope: Option<StateScope>,
    ) -> Result<Vec<RAGResult>> {
        // Extract session from scope or use default
        let session_id = extract_session_from_scope(scope.as_ref()).unwrap_or(self.default_session);

        debug!(
            "SessionRAGAdapter: retrieving query=\"{}\" k={} session_id={}",
            query, k, session_id
        );

        // Call wrapped pipeline
        let results = self.inner.retrieve_in_session(query, session_id, k).await?;

        debug!(
            "SessionRAGAdapter: got {} results, converting to RAGResult",
            results.len()
        );

        // Convert SessionVectorResult → RAGResult
        Ok(results
            .into_iter()
            .map(convert_to_rag_result)
            .collect())
    }
}

/// Extract session ID from `StateScope`
///
/// Parses `StateScope::Custom("session:abc123")` → `SessionId("abc123")`
///
/// # Returns
/// `Some(SessionId)` if scope encodes session, `None` otherwise
fn extract_session_from_scope(scope: Option<&StateScope>) -> Option<SessionId> {
    match scope {
        Some(StateScope::Custom(s)) if s.starts_with("session:") => {
            let session_str = s.strip_prefix("session:")?;
            match session_str.parse::<SessionId>() {
                Ok(id) => {
                    debug!("Extracted session_id={} from scope", id);
                    Some(id)
                }
                Err(e) => {
                    warn!("Failed to parse session_id from scope \"{}\": {}", s, e);
                    None
                }
            }
        }
        _ => None,
    }
}

/// Convert `SessionVectorResult` to `RAGResult`
///
/// Preserves all relevant fields: id, content, score, metadata
fn convert_to_rag_result(session_result: SessionVectorResult) -> RAGResult {
    RAGResult {
        id: session_result.id,
        content: session_result.text,
        score: session_result.score,
        metadata: session_result.metadata,
        // SessionVectorResult doesn't have timestamp, use current time
        timestamp: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_session_from_scope() {
        // Valid session scope (SessionId wraps a UUID)
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let scope = StateScope::Custom(format!("session:{}", uuid_str));
        let session_id = extract_session_from_scope(Some(&scope));
        assert!(session_id.is_some());
        assert_eq!(session_id.unwrap().to_string(), uuid_str);

        // Invalid UUID format
        let scope = StateScope::Custom("session:not-a-uuid".to_string());
        assert!(extract_session_from_scope(Some(&scope)).is_none());

        // Different prefix
        let scope = StateScope::Custom("tenant:abc".to_string());
        assert!(extract_session_from_scope(Some(&scope)).is_none());

        // Global scope
        let scope = StateScope::Global;
        assert!(extract_session_from_scope(Some(&scope)).is_none());

        // None
        assert!(extract_session_from_scope(None).is_none());
    }

    #[test]
    #[allow(clippy::float_cmp)] // Test needs exact float comparison
    fn test_convert_to_rag_result() {
        use std::collections::HashMap;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::json!("test.txt"));

        let session_result = SessionVectorResult {
            id: "result-1".to_string(),
            text: "test content".to_string(),
            score: 0.95,
            metadata,
        };

        let rag_result = convert_to_rag_result(session_result);

        assert_eq!(rag_result.id, "result-1");
        assert_eq!(rag_result.content, "test content");
        assert_eq!(rag_result.score, 0.95);
        assert_eq!(
            rag_result.metadata.get("source"),
            Some(&serde_json::json!("test.txt"))
        );
    }
}
