//! ABOUTME: Core artifact bridge providing language-agnostic artifact operations
//! ABOUTME: Wraps ArtifactStorage for script access with type conversions

use llmspell_core::{error::LLMSpellError, Result};
use llmspell_sessions::{
    artifact::{
        access::Permission, ArtifactId, ArtifactMetadata, ArtifactQuery, ArtifactType,
        SessionArtifact,
    },
    SessionId, SessionManager,
};
use std::path::Path;
use std::sync::Arc;

/// Convert mlua error to LLMSpellError
fn lua_to_llmspell_error(e: mlua::Error) -> LLMSpellError {
    LLMSpellError::Script {
        message: e.to_string(),
        language: Some("lua".to_string()),
        line: None,
        source: None,
    }
}

/// Core artifact bridge for language-agnostic artifact operations
///
/// This bridge wraps artifact operations from SessionManager and provides
/// synchronous interfaces for script languages.
pub struct ArtifactBridge {
    /// Reference to the session manager (which contains artifact storage)
    session_manager: Arc<SessionManager>,
}

impl ArtifactBridge {
    /// Create a new artifact bridge
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Store an artifact
    pub fn store_artifact(
        &self,
        session_id: &SessionId,
        artifact_type: ArtifactType,
        name: String,
        content: Vec<u8>,
        metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<ArtifactId> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "artifact_store",
            async move {
                manager
                    .store_artifact(&session_id, artifact_type, name, content, metadata)
                    .await
            },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Get an artifact with metadata
    pub fn get_artifact(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<SessionArtifact> {
        let session_id = *session_id;
        let artifact_id = artifact_id.clone();
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "artifact_get",
            async move { manager.get_artifact(&session_id, &artifact_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Get artifact content only
    pub fn get_artifact_content(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<Vec<u8>> {
        let session_id = *session_id;
        let artifact_id = artifact_id.clone();
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "artifact_get_content",
            async move {
                manager
                    .get_artifact_content(&session_id, &artifact_id)
                    .await
            },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// List artifacts for a session
    pub fn list_artifacts(&self, session_id: &SessionId) -> Result<Vec<ArtifactMetadata>> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "artifact_list",
            async move { manager.list_artifacts(&session_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Delete an artifact
    pub fn delete_artifact(&self, session_id: &SessionId, artifact_id: &ArtifactId) -> Result<()> {
        let session_id = *session_id;
        let artifact_id = artifact_id.clone();
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "artifact_delete",
            async move { manager.delete_artifact(&session_id, &artifact_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Query artifacts across sessions
    pub fn query_artifacts(&self, query: ArtifactQuery) -> Result<Vec<ArtifactMetadata>> {
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "artifact_query",
            async move { manager.query_artifacts(query).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Store a file as an artifact
    pub fn store_file_artifact(
        &self,
        session_id: &SessionId,
        file_path: &str,
        artifact_type: ArtifactType,
        metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<ArtifactId> {
        let session_id = *session_id;
        let file_path_owned = file_path.to_string();
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "artifact_store_file",
            async move {
                manager
                    .store_file_artifact(
                        &session_id,
                        Path::new(&file_path_owned),
                        artifact_type,
                        metadata,
                    )
                    .await
            },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Grant permission on an artifact
    pub fn grant_permission(
        &self,
        granting_session_id: &SessionId,
        artifact_id: &ArtifactId,
        target_session_id: SessionId,
        permission: Permission,
    ) -> Result<()> {
        let granting_session_id = *granting_session_id;
        let artifact_id = artifact_id.clone();
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "artifact_grant_permission",
            async move {
                manager
                    .grant_artifact_permission(
                        &granting_session_id,
                        &artifact_id,
                        target_session_id,
                        permission,
                    )
                    .await
            },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Revoke permission on an artifact
    pub fn revoke_permission(
        &self,
        revoking_session_id: &SessionId,
        artifact_id: &ArtifactId,
        target_session_id: &SessionId,
    ) -> Result<()> {
        let revoking_session_id = *revoking_session_id;
        let artifact_id = artifact_id.clone();
        let target_session_id = *target_session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "artifact_revoke_permission",
            async move {
                manager
                    .revoke_artifact_permission(
                        &revoking_session_id,
                        &artifact_id,
                        &target_session_id,
                    )
                    .await
            },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    // TODO: Add more operations like getPermissions, checkAccess, getAuditLog in subsequent tasks
}
