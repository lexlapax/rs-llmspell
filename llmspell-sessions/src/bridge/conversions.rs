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
        "status": metadata.status.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_to_create_options_full() {
        let json = json!({
            "name": "Test Session",
            "description": "A test session",
            "tags": ["tag1", "tag2"],
            "metadata": {
                "key1": "value1",
                "key2": 42
            },
            "parent_session_id": "00000000-0000-0000-0000-000000000001"
        });

        let options = json_to_create_options(&json).expect("Failed to parse options");

        assert_eq!(options.name, Some("Test Session".to_string()));
        assert_eq!(options.description, Some("A test session".to_string()));
        assert_eq!(options.tags, vec!["tag1", "tag2"]);
        assert_eq!(options.metadata.get("key1"), Some(&json!("value1")));
        assert_eq!(options.metadata.get("key2"), Some(&json!(42)));
        assert!(options.parent_session_id.is_some());
    }

    #[test]
    fn test_json_to_create_options_minimal() {
        let json = json!({});
        let options = json_to_create_options(&json).expect("Failed to parse options");

        assert!(options.name.is_none());
        assert!(options.description.is_none());
        assert!(options.tags.is_empty());
        assert!(options.metadata.is_empty());
        assert!(options.parent_session_id.is_none());
    }

    #[test]
    fn test_json_to_create_options_invalid_parent_id() {
        let json = json!({
            "parent_session_id": "invalid-uuid"
        });

        let result = json_to_create_options(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid parent session ID"));
    }

    #[test]
    fn test_json_to_create_options_non_object() {
        let json = json!("not an object");
        let result = json_to_create_options(&json);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Expected object for session options");
    }

    #[test]
    fn test_json_to_create_options_invalid_tags() {
        let json = json!({
            "tags": [123, "valid-tag", null, true]
        });

        let options = json_to_create_options(&json).expect("Failed to parse options");
        // Only string values should be kept
        assert_eq!(options.tags, vec!["valid-tag"]);
    }

    #[test]
    fn test_json_to_session_query_full() {
        let json = json!({
            "status": "active",
            "tags": ["tag1", "tag2"],
            "parent_session_id": "00000000-0000-0000-0000-000000000001",
            "created_after": "2023-01-01T00:00:00Z",
            "created_before": "2023-12-31T23:59:59Z",
            "limit": 10,
            "offset": 5,
            "sort_by": "created_at"
        });

        let query = json_to_session_query(&json).expect("Failed to parse query");

        assert_eq!(query.status, Some(SessionStatus::Active));
        assert_eq!(query.tags, vec!["tag1", "tag2"]);
        assert!(query.parent_session_id.is_some());
        assert!(query.created_after.is_some());
        assert!(query.created_before.is_some());
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(5));
        assert_eq!(query.sort_by, SessionSortBy::CreatedAt);
    }

    #[test]
    fn test_json_to_session_query_invalid_status() {
        let json = json!({
            "status": "invalid_status"
        });

        let result = json_to_session_query(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown session status"));
    }

    #[test]
    fn test_json_to_session_query_invalid_dates() {
        let json = json!({
            "created_after": "not-a-date"
        });

        let result = json_to_session_query(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid created_after date"));
    }

    #[test]
    fn test_json_to_session_query_invalid_sort_by() {
        let json = json!({
            "sort_by": "invalid_field"
        });

        let result = json_to_session_query(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown sort by option"));
    }

    #[test]
    fn test_session_metadata_to_json() {
        let metadata = SessionMetadata {
            id: SessionId::new(),
            name: Some("Test Session".to_string()),
            description: Some("A test session".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            status: SessionStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            ended_at: None,
            created_by: Some("test_user".to_string()),
            parent_session_id: Some(SessionId::new()),
            custom_metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("key".to_string(), json!("value"));
                map
            },
            artifact_count: 0,
            total_artifact_size: 0,
            operation_count: 0,
        };

        let json = session_metadata_to_json(&metadata);

        assert_eq!(json["name"], "Test Session");
        assert_eq!(json["description"], "A test session");
        assert_eq!(json["tags"], json!(["tag1", "tag2"]));
        assert_eq!(json["status"], "active");
        assert!(json["created_at"].is_string());
        assert!(json["updated_at"].is_string());
        assert!(json["parent_session_id"].is_string());
        assert_eq!(json["custom_metadata"]["key"], "value");
    }

    #[test]
    fn test_artifact_metadata_to_json() {
        let artifact_metadata = ArtifactMetadata {
            name: "test.txt".to_string(),
            description: Some("Test artifact".to_string()),
            artifact_type: ArtifactType::UserInput,
            mime_type: "text/plain".to_string(),
            size: 1024,
            tags: vec!["test".to_string()],
            custom: std::collections::HashMap::new(),
            version: crate::artifact::ArtifactVersion {
                version: 1,
                previous_hash: None,
                created_at: chrono::Utc::now(),
            },
            created_at: chrono::Utc::now(),
            created_by: Some("user".to_string()),
            parent_artifact: None,
            is_compressed: false,
            original_size: None,
        };

        let json = artifact_metadata_to_json(&artifact_metadata);

        assert_eq!(json["name"], "test.txt");
        assert_eq!(json["description"], "Test artifact");
        assert_eq!(json["artifact_type"], "UserInput");
        assert_eq!(json["mime_type"], "text/plain");
        assert_eq!(json["size"], 1024);
        assert_eq!(json["tags"], json!(["test"]));
        assert_eq!(json["version"], 1);
        assert_eq!(json["created_by"], "user");
        assert_eq!(json["is_compressed"], false);
    }

    #[test]
    fn test_parse_session_status_all_variants() {
        assert_eq!(
            parse_session_status("active").unwrap(),
            SessionStatus::Active
        );
        assert_eq!(
            parse_session_status("ACTIVE").unwrap(),
            SessionStatus::Active
        );
        assert_eq!(
            parse_session_status("suspended").unwrap(),
            SessionStatus::Suspended
        );
        assert_eq!(
            parse_session_status("completed").unwrap(),
            SessionStatus::Completed
        );
        assert_eq!(
            parse_session_status("failed").unwrap(),
            SessionStatus::Failed
        );
    }

    #[test]
    fn test_parse_session_status_invalid() {
        assert!(parse_session_status("invalid").is_err());
        assert!(parse_session_status("").is_err());
    }

    #[test]
    fn test_parse_sort_by_all_variants() {
        assert_eq!(
            parse_sort_by("created_at").unwrap(),
            SessionSortBy::CreatedAt
        );
        assert_eq!(
            parse_sort_by("CREATED_AT").unwrap(),
            SessionSortBy::CreatedAt
        );
        assert_eq!(
            parse_sort_by("updated_at").unwrap(),
            SessionSortBy::UpdatedAt
        );
        assert_eq!(parse_sort_by("name").unwrap(), SessionSortBy::Name);
    }

    #[test]
    fn test_parse_artifact_type_all_variants() {
        assert_eq!(
            parse_artifact_type("agent_output").unwrap(),
            ArtifactType::AgentOutput
        );
        assert_eq!(
            parse_artifact_type("tool_result").unwrap(),
            ArtifactType::ToolResult
        );
        assert_eq!(
            parse_artifact_type("user_input").unwrap(),
            ArtifactType::UserInput
        );
        assert_eq!(
            parse_artifact_type("system_generated").unwrap(),
            ArtifactType::SystemGenerated
        );
    }

    #[test]
    fn test_parse_artifact_type_custom() {
        match parse_artifact_type("custom_type").unwrap() {
            ArtifactType::Custom(name) => assert_eq!(name, "custom_type"),
            _ => panic!("Expected custom type"),
        }
    }

    #[test]
    fn test_parse_artifact_type_empty() {
        assert!(parse_artifact_type("").is_err());
        assert_eq!(
            parse_artifact_type("").unwrap_err(),
            "Artifact type cannot be empty"
        );
    }

    #[test]
    fn test_json_to_session_query_large_values() {
        let json = json!({
            "limit": u64::MAX,
            "offset": u64::MAX
        });

        let result = json_to_session_query(&json);
        // On platforms where usize < u64, this should fail
        if std::mem::size_of::<usize>() < std::mem::size_of::<u64>() {
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_edge_case_mixed_type_arrays() {
        // Test that non-string values in tag arrays are filtered out
        let json = json!({
            "tags": [123, "tag1", null, {"object": "value"}, "tag2", true]
        });

        let options = json_to_create_options(&json).expect("Failed to parse options");
        assert_eq!(options.tags, vec!["tag1", "tag2"]);

        let query = json_to_session_query(&json).expect("Failed to parse query");
        assert_eq!(query.tags, vec!["tag1", "tag2"]);
    }
}
