//! ABOUTME: Core artifact bridge providing language-agnostic artifact operations
//! ABOUTME: Wraps `ArtifactStorage` for script access with async operations

use llmspell_core::{error::LLMSpellError, Result};
use llmspell_kernel::sessions::{
    artifact::{
        access::Permission, ArtifactId, ArtifactMetadata, ArtifactQuery, ArtifactType,
        SessionArtifact,
    },
    SessionId, SessionManager,
};
use std::path::Path;
use std::sync::Arc;

/// Helper macro to convert `SessionError` to `LLMSpellError`
macro_rules! convert_err {
    ($expr:expr) => {
        $expr.map_err(|e| LLMSpellError::Component {
            message: format!("Artifact error: {}", e),
            source: None,
        })
    };
}

/// Core artifact bridge for language-agnostic artifact operations
///
/// This bridge wraps artifact operations from `SessionManager` and provides
/// async interfaces for script languages.
pub struct ArtifactBridge {
    /// Reference to the session manager (which contains artifact storage)
    session_manager: Arc<SessionManager>,
}

impl ArtifactBridge {
    /// Create a new artifact bridge
    #[must_use]
    pub const fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Store an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if artifact storage fails
    pub async fn store_artifact(
        &self,
        session_id: &SessionId,
        artifact_type: ArtifactType,
        name: String,
        content: Vec<u8>,
        metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<ArtifactId> {
        convert_err!(
            self.session_manager
                .store_artifact(session_id, artifact_type, name, content, metadata)
                .await
        )
    }

    /// Get an artifact with metadata
    ///
    /// # Errors
    ///
    /// Returns an error if artifact retrieval fails
    pub async fn get_artifact(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<SessionArtifact> {
        convert_err!(
            self.session_manager
                .get_artifact(session_id, artifact_id)
                .await
        )
    }

    /// Get artifact content only
    ///
    /// # Errors
    ///
    /// Returns an error if content retrieval fails
    pub async fn get_artifact_content(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<Vec<u8>> {
        convert_err!(
            self.session_manager
                .get_artifact_content(session_id, artifact_id)
                .await
        )
    }

    /// List artifacts for a session
    ///
    /// # Errors
    ///
    /// Returns an error if artifact listing fails
    pub async fn list_artifacts(&self, session_id: &SessionId) -> Result<Vec<ArtifactMetadata>> {
        convert_err!(self.session_manager.list_artifacts(session_id).await)
    }

    /// Delete an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if artifact deletion fails
    pub async fn delete_artifact(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<()> {
        convert_err!(
            self.session_manager
                .delete_artifact(session_id, artifact_id)
                .await
        )
    }

    /// Query artifacts across sessions
    ///
    /// # Errors
    ///
    /// Returns an error if artifact query fails
    pub async fn query_artifacts(&self, query: ArtifactQuery) -> Result<Vec<ArtifactMetadata>> {
        convert_err!(self.session_manager.query_artifacts(query).await)
    }

    /// Store a file as an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File reading fails
    /// - Artifact storage fails
    pub async fn store_file_artifact(
        &self,
        session_id: &SessionId,
        file_path: &Path,
        artifact_type: ArtifactType,
        metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<ArtifactId> {
        convert_err!(
            self.session_manager
                .store_file_artifact(session_id, file_path, artifact_type, metadata)
                .await
        )
    }

    /// Grant permission on an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if permission granting fails
    pub async fn grant_permission(
        &self,
        granting_session_id: &SessionId,
        artifact_id: &ArtifactId,
        target_session_id: SessionId,
        permission: Permission,
    ) -> Result<()> {
        convert_err!(
            self.session_manager
                .grant_artifact_permission(
                    granting_session_id,
                    artifact_id,
                    target_session_id,
                    permission,
                )
                .await
        )
    }

    /// Revoke permission on an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if permission revocation fails
    pub async fn revoke_permission(
        &self,
        revoking_session_id: &SessionId,
        artifact_id: &ArtifactId,
        target_session_id: &SessionId,
    ) -> Result<()> {
        convert_err!(
            self.session_manager
                .revoke_artifact_permission(revoking_session_id, artifact_id, target_session_id,)
                .await
        )
    }

    // TODO: Add more operations like getPermissions, checkAccess, getAuditLog in subsequent tasks
}
