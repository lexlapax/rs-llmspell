//! ABOUTME: Type conversions between Rust and script types for sessions
//! ABOUTME: Handles conversion of session data structures to/from script representations

use crate::{
    artifact::{ArtifactMetadata, ArtifactType},
    types::{CreateSessionOptions, SessionQuery, SessionSortBy, SessionStatus},
    SessionId, SessionMetadata,
};
use std::str::FromStr;

/// Convert a script value (JSON) to `CreateSessionOptions`
pub fn json_to_create_options(value: &serde_json::Value) -> Result<CreateSessionOptions, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "Expected object for session options".to_string())?;

    let mut options = CreateSessionOptions::default();

    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
        options.name = Some(name.to_string());
    }

    if let Some(desc) = obj.get("description").and_then(|v| v.as_str()) {
        options.description = Some(desc.to_string());
    }

    if let Some(tags) = obj.get("tags").and_then(|v| v.as_array()) {
        options.tags = tags
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }

    if let Some(metadata) = obj.get("metadata").and_then(|v| v.as_object()) {
        options.metadata = metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
    }

    if let Some(parent) = obj.get("parent_session_id").and_then(|v| v.as_str()) {
        options.parent_session_id = Some(
            SessionId::from_str(parent).map_err(|e| format!("Invalid parent session ID: {}", e))?,
        );
    }

    Ok(options)
}

/// Convert a script value (JSON) to `SessionQuery`
pub fn json_to_session_query(value: &serde_json::Value) -> Result<SessionQuery, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "Expected object for session query".to_string())?;

    let mut query = SessionQuery::default();

    if let Some(status) = obj.get("status").and_then(|v| v.as_str()) {
        query.status = Some(parse_session_status(status)?);
    }

    if let Some(tags) = obj.get("tags").and_then(|v| v.as_array()) {
        query.tags = tags
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }

    if let Some(parent) = obj.get("parent_session_id").and_then(|v| v.as_str()) {
        query.parent_session_id =
            Some(SessionId::from_str(parent).map_err(|e| format!("Invalid parent ID: {}", e))?);
    }

    if let Some(created_after) = obj.get("created_after").and_then(|v| v.as_str()) {
        query.created_after = Some(
            chrono::DateTime::parse_from_rfc3339(created_after)
                .map_err(|e| format!("Invalid created_after date: {}", e))?
                .with_timezone(&chrono::Utc),
        );
    }

    if let Some(created_before) = obj.get("created_before").and_then(|v| v.as_str()) {
        query.created_before = Some(
            chrono::DateTime::parse_from_rfc3339(created_before)
                .map_err(|e| format!("Invalid created_before date: {}", e))?
                .with_timezone(&chrono::Utc),
        );
    }

    if let Some(limit) = obj.get("limit").and_then(|v| v.as_u64()) {
        query.limit = Some(
            limit
                .try_into()
                .map_err(|_| "Limit value too large for platform")?,
        );
    }

    if let Some(offset) = obj.get("offset").and_then(|v| v.as_u64()) {
        query.offset = Some(
            offset
                .try_into()
                .map_err(|_| "Offset value too large for platform")?,
        );
    }

    if let Some(sort_by) = obj.get("sort_by").and_then(|v| v.as_str()) {
        query.sort_by = parse_sort_by(sort_by)?;
    }

    // Sort order is determined by sort_by enum value

    Ok(query)
}

/// Convert `SessionMetadata` to JSON value
pub fn session_metadata_to_json(metadata: &SessionMetadata) -> serde_json::Value {
    serde_json::json!({
        "id": metadata.id.to_string(),
        "name": metadata.name,
        "description": metadata.description,
        "tags": metadata.tags,
        "created_at": metadata.created_at.to_rfc3339(),
        "updated_at": metadata.updated_at.to_rfc3339(),
        "parent_session_id": metadata.parent_session_id.as_ref().map(|id| id.to_string()),
        "custom_metadata": metadata.custom_metadata,
    })
}

/// Convert `ArtifactMetadata` to JSON value
pub fn artifact_metadata_to_json(metadata: &ArtifactMetadata) -> serde_json::Value {
    serde_json::json!({
        "name": metadata.name,
        "description": metadata.description,
        "artifact_type": format!("{:?}", metadata.artifact_type),
        "mime_type": metadata.mime_type,
        "size": metadata.size,
        "tags": metadata.tags,
        "custom": metadata.custom,
        "version": metadata.version.version,
        "created_at": metadata.created_at.to_rfc3339(),
        "created_by": metadata.created_by,
        "parent_artifact": metadata.parent_artifact.as_ref().map(|id| id.to_string()),
        "is_compressed": metadata.is_compressed,
        "original_size": metadata.original_size,
    })
}

/// Parse session status from string
fn parse_session_status(status: &str) -> Result<SessionStatus, String> {
    match status.to_lowercase().as_str() {
        "active" => Ok(SessionStatus::Active),
        "suspended" => Ok(SessionStatus::Suspended),
        "completed" => Ok(SessionStatus::Completed),
        "failed" => Ok(SessionStatus::Failed),
        _ => Err(format!("Unknown session status: {}", status)),
    }
}

/// Parse sort by option from string
fn parse_sort_by(sort_by: &str) -> Result<SessionSortBy, String> {
    match sort_by.to_lowercase().as_str() {
        "created_at" => Ok(SessionSortBy::CreatedAt),
        "updated_at" => Ok(SessionSortBy::UpdatedAt),
        "name" => Ok(SessionSortBy::Name),
        _ => Err(format!("Unknown sort by option: {}", sort_by)),
    }
}

/// Parse artifact type from string
pub fn parse_artifact_type(type_str: &str) -> Result<ArtifactType, String> {
    match type_str.to_lowercase().as_str() {
        "agent_output" => Ok(ArtifactType::AgentOutput),
        "tool_result" => Ok(ArtifactType::ToolResult),
        "user_input" => Ok(ArtifactType::UserInput),
        "system_generated" => Ok(ArtifactType::SystemGenerated),
        _ => {
            // Check if it's a custom type
            if type_str.is_empty() {
                Err("Artifact type cannot be empty".to_string())
            } else {
                Ok(ArtifactType::Custom(type_str.to_string()))
            }
        }
    }
}
