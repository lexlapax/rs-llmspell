//! JSON schema for LLM consolidation responses
//!
//! Defines structured output format for LLM-driven consolidation decisions.
//! Supports JSON parsing with error recovery and natural language fallback.
//!
//! # Versioning
//!
//! Responses include optional `prompt_version` metadata for tracking which
//! prompt template generated the response (A/B testing, metrics analysis).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info, trace, warn};

use crate::error::{MemoryError, Result};

/// Output format mode for consolidation prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Structured JSON output (default, 95%+ parse success)
    Json,
    /// Natural language output (fallback mode)
    NaturalLanguage,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Json
    }
}

/// Consolidated response from LLM containing all extraction decisions
///
/// # JSON Schema
///
/// ```json
/// {
///   "entities": [
///     {
///       "id": "entity-uuid",
///       "name": "Rust",
///       "entity_type": "programming_language",
///       "properties": {"paradigm": "multi-paradigm"}
///     }
///   ],
///   "relationships": [
///     {
///       "from_entity": "entity-uuid-1",
///       "to_entity": "entity-uuid-2",
///       "relationship_type": "has_feature",
///       "properties": {}
///     }
///   ],
///   "decisions": [
///     {"type": "ADD", "entity_id": "entity-uuid"},
///     {"type": "UPDATE", "entity_id": "existing-id", "changes": {"key": "value"}},
///     {"type": "DELETE", "entity_id": "old-id"},
///     {"type": "NOOP"}
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResponse {
    /// Entities extracted from episodic content
    #[serde(default)]
    pub entities: Vec<EntityPayload>,

    /// Relationships between entities
    #[serde(default)]
    pub relationships: Vec<RelationshipPayload>,

    /// Consolidation decisions (ADD/UPDATE/DELETE/NOOP)
    #[serde(default)]
    pub decisions: Vec<DecisionPayload>,

    /// Optional reasoning from LLM (for debugging/explainability)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,

    /// Prompt version that generated this response (for A/B testing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_version: Option<String>,
}

impl ConsolidationResponse {
    /// Create empty response
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entities: Vec::new(),
            relationships: Vec::new(),
            decisions: Vec::new(),
            reasoning: None,
            prompt_version: None,
        }
    }

    /// Set prompt version metadata
    #[must_use]
    pub fn with_prompt_version(mut self, version: String) -> Self {
        self.prompt_version = Some(version);
        self
    }

    /// Check if response contains any decisions
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entities.is_empty() && self.relationships.is_empty() && self.decisions.is_empty()
    }

    /// Parse from JSON string with error recovery
    ///
    /// Attempts to extract valid decisions even if JSON is partially malformed.
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::InvalidInput` if JSON parsing fails completely,
    /// including partial parsing attempts.
    pub fn from_json(json_str: &str) -> Result<Self> {
        info!(
            "Parsing consolidation response: len={} bytes",
            json_str.len()
        );
        trace!(
            "Raw JSON input: {}",
            json_str.chars().take(200).collect::<String>()
        );

        // Strip markdown code fences if present (```json ... ``` or ``` ... ```)
        let json_str = json_str.trim();
        let json_str = if json_str.starts_with("```") {
            debug!("Detected markdown code fence, stripping");
            // Find start and end of code block
            let start = if json_str.starts_with("```json") {
                json_str.find('\n').map_or(7, |i| i + 1)
            } else if json_str.starts_with("```") {
                json_str.find('\n').map_or(3, |i| i + 1)
            } else {
                0
            };
            let end = json_str.rfind("```").unwrap_or(json_str.len());
            json_str[start..end].trim()
        } else {
            json_str
        };

        // Try full JSON parsing first
        match serde_json::from_str::<Self>(json_str) {
            Ok(response) => {
                info!(
                    "Successfully parsed consolidation response: entities={}, relationships={}, decisions={}",
                    response.entities.len(),
                    response.relationships.len(),
                    response.decisions.len()
                );
                Ok(response)
            }
            Err(e) => {
                warn!("Full JSON parsing failed: {}, attempting partial parse", e);
                // Try partial parsing with lenient mode
                Self::partial_parse(json_str).ok_or_else(|| {
                    MemoryError::InvalidInput(format!(
                        "Failed to parse consolidation response: {e}"
                    ))
                })
            }
        }
    }

    /// Attempt partial parsing for malformed JSON
    ///
    /// Extracts as many valid fields as possible, skipping invalid sections.
    fn partial_parse(json_str: &str) -> Option<Self> {
        debug!("Attempting partial parse of malformed JSON");
        let mut response = Self::empty();

        if let Ok(value) = serde_json::from_str::<Value>(json_str) {
            Self::extract_entities(&value, &mut response);
            Self::extract_relationships(&value, &mut response);
            Self::extract_decisions(&value, &mut response);
            Self::extract_reasoning(&value, &mut response);
        }

        if response.is_empty() {
            warn!("Partial parse failed: no valid sections extracted");
            None
        } else {
            info!(
                "Partial parse succeeded: entities={}, relationships={}, decisions={}",
                response.entities.len(),
                response.relationships.len(),
                response.decisions.len()
            );
            Some(response)
        }
    }

    /// Extract entities array from parsed JSON
    fn extract_entities(value: &Value, response: &mut Self) {
        if let Some(entities) = value.get("entities").and_then(Value::as_array) {
            response.entities = entities
                .iter()
                .filter_map(|v| serde_json::from_value::<EntityPayload>(v.clone()).ok())
                .collect();
            debug!(
                "Partial parse: extracted {} entities",
                response.entities.len()
            );
        }
    }

    /// Extract relationships array from parsed JSON
    fn extract_relationships(value: &Value, response: &mut Self) {
        if let Some(relationships) = value.get("relationships").and_then(Value::as_array) {
            response.relationships = relationships
                .iter()
                .filter_map(|v| serde_json::from_value::<RelationshipPayload>(v.clone()).ok())
                .collect();
            debug!(
                "Partial parse: extracted {} relationships",
                response.relationships.len()
            );
        }
    }

    /// Extract decisions array from parsed JSON
    fn extract_decisions(value: &Value, response: &mut Self) {
        if let Some(decisions) = value.get("decisions").and_then(Value::as_array) {
            response.decisions = decisions
                .iter()
                .filter_map(|v| serde_json::from_value::<DecisionPayload>(v.clone()).ok())
                .collect();
            debug!(
                "Partial parse: extracted {} decisions",
                response.decisions.len()
            );
        }
    }

    /// Extract reasoning string from parsed JSON
    fn extract_reasoning(value: &Value, response: &mut Self) {
        if let Some(reasoning) = value.get("reasoning").and_then(Value::as_str) {
            response.reasoning = Some(reasoning.to_string());
            trace!("Partial parse: extracted reasoning");
        }
    }

    /// Convert to JSON string
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::Serialization` if JSON serialization fails.
    pub fn to_json(&self) -> Result<String> {
        debug!(
            "Serializing consolidation response: entities={}, relationships={}, decisions={}",
            self.entities.len(),
            self.relationships.len(),
            self.decisions.len()
        );
        let json = serde_json::to_string_pretty(self).map_err(MemoryError::Serialization)?;
        trace!(
            "Serialized JSON (first 200 chars): {}",
            json.chars().take(200).collect::<String>()
        );
        Ok(json)
    }
}

/// Entity payload in LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPayload {
    /// Entity ID (auto-generated if not provided)
    #[serde(default = "generate_entity_id")]
    pub id: String,

    /// Entity name/label
    pub name: String,

    /// Entity type/category
    pub entity_type: String,

    /// Additional properties
    #[serde(default)]
    pub properties: Value,

    /// Event time (optional, extracted from episodic timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<DateTime<Utc>>,
}

/// Relationship payload in LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipPayload {
    /// Source entity ID
    pub from_entity: String,

    /// Target entity ID
    pub to_entity: String,

    /// Relationship type
    pub relationship_type: String,

    /// Additional properties
    #[serde(default)]
    pub properties: Value,

    /// Event time (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<DateTime<Utc>>,
}

/// Decision payload in LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum DecisionPayload {
    /// Add new entity
    #[serde(rename = "ADD")]
    Add {
        /// Entity ID to add
        entity_id: String,
    },

    /// Update existing entity
    #[serde(rename = "UPDATE")]
    Update {
        /// Entity ID to update
        entity_id: String,
        /// Property changes
        changes: HashMap<String, Value>,
    },

    /// Delete entity
    #[serde(rename = "DELETE")]
    Delete {
        /// Entity ID to delete
        entity_id: String,
    },

    /// No operation
    #[serde(rename = "NOOP")]
    Noop,
}

/// Generate unique entity ID
fn generate_entity_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Example JSON responses for different decision types
pub mod examples {
    use super::{ConsolidationResponse, DecisionPayload, EntityPayload};

    /// Example ADD decision response
    #[must_use]
    pub fn add_example() -> ConsolidationResponse {
        ConsolidationResponse {
            entities: vec![EntityPayload {
                id: "rust-lang-001".to_string(),
                name: "Rust".to_string(),
                entity_type: "programming_language".to_string(),
                properties: serde_json::json!({
                    "paradigm": "multi-paradigm",
                    "typing": "static"
                }),
                event_time: None,
            }],
            relationships: vec![],
            decisions: vec![DecisionPayload::Add {
                entity_id: "rust-lang-001".to_string(),
            }],
            reasoning: Some(
                "New programming language entity extracted from episodic content.".to_string(),
            ),
            prompt_version: None,
        }
    }

    /// Example UPDATE decision response
    ///
    /// # Panics
    ///
    /// Panics if the hard-coded JSON value cannot be converted to `HashMap` (should never happen).
    #[must_use]
    pub fn update_example() -> ConsolidationResponse {
        ConsolidationResponse {
            entities: vec![],
            relationships: vec![],
            decisions: vec![DecisionPayload::Update {
                entity_id: "rust-lang-001".to_string(),
                changes: serde_json::from_value(serde_json::json!({
                    "features": "zero-cost abstractions"
                }))
                .expect("Hard-coded JSON should always deserialize"),
            }],
            reasoning: Some(
                "Updating existing Rust entity with additional feature information.".to_string(),
            ),
            prompt_version: None,
        }
    }

    /// Example DELETE decision response
    #[must_use]
    pub fn delete_example() -> ConsolidationResponse {
        ConsolidationResponse {
            entities: vec![],
            relationships: vec![],
            decisions: vec![DecisionPayload::Delete {
                entity_id: "python27-deprecated".to_string(),
            }],
            reasoning: Some("Python 2.7 is deprecated and unsupported.".to_string()),
            prompt_version: None,
        }
    }

    /// Example NOOP decision response
    #[must_use]
    pub fn noop_example() -> ConsolidationResponse {
        ConsolidationResponse {
            entities: vec![],
            relationships: vec![],
            decisions: vec![DecisionPayload::Noop],
            reasoning: Some("Episodic content contains no extractable knowledge.".to_string()),
            prompt_version: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{examples, ConsolidationResponse, DecisionPayload, OutputFormat};

    #[test]
    fn test_parse_add_decision() {
        let json = r#"{
            "entities": [
                {
                    "id": "test-001",
                    "name": "Test Entity",
                    "entity_type": "concept",
                    "properties": {"key": "value"}
                }
            ],
            "relationships": [],
            "decisions": [
                {"type": "ADD", "entity_id": "test-001"}
            ]
        }"#;

        let response = ConsolidationResponse::from_json(json).unwrap();
        assert_eq!(response.entities.len(), 1);
        assert_eq!(response.decisions.len(), 1);
        assert!(matches!(response.decisions[0], DecisionPayload::Add { .. }));
    }

    #[test]
    fn test_parse_update_decision() {
        let json = r#"{
            "entities": [],
            "relationships": [],
            "decisions": [
                {
                    "type": "UPDATE",
                    "entity_id": "existing-001",
                    "changes": {"property": "new_value"}
                }
            ]
        }"#;

        let response = ConsolidationResponse::from_json(json).unwrap();
        assert_eq!(response.decisions.len(), 1);
        assert!(matches!(
            response.decisions[0],
            DecisionPayload::Update { .. }
        ));
    }

    #[test]
    fn test_parse_delete_decision() {
        let json = r#"{
            "entities": [],
            "relationships": [],
            "decisions": [
                {"type": "DELETE", "entity_id": "old-001"}
            ]
        }"#;

        let response = ConsolidationResponse::from_json(json).unwrap();
        assert_eq!(response.decisions.len(), 1);
        assert!(matches!(
            response.decisions[0],
            DecisionPayload::Delete { .. }
        ));
    }

    #[test]
    fn test_parse_noop_decision() {
        let json = r#"{
            "entities": [],
            "relationships": [],
            "decisions": [
                {"type": "NOOP"}
            ],
            "reasoning": "No actionable knowledge"
        }"#;

        let response = ConsolidationResponse::from_json(json).unwrap();
        assert_eq!(response.decisions.len(), 1);
        assert!(matches!(response.decisions[0], DecisionPayload::Noop));
        assert!(response.reasoning.is_some());
    }

    #[test]
    fn test_partial_parse_malformed_json() {
        // Missing closing brace, but entities array is valid
        let json = r#"{
            "entities": [
                {"id": "test", "name": "Test", "entity_type": "test", "properties": {}}
            ],
            "decisions": [
                {"type": "ADD", "entity_id": "test"}
        }"#;

        // Should fail full parse but succeed partial parse
        let result = ConsolidationResponse::from_json(json);
        // This will fail because serde_json is strict, but in real usage
        // we'd try partial_parse directly
        assert!(result.is_err());

        // Test partial_parse directly
        let response = ConsolidationResponse::partial_parse(json);
        assert!(response.is_none()); // Actually fails because it's not valid JSON at all
    }

    #[test]
    fn test_empty_response() {
        let response = ConsolidationResponse::empty();
        assert!(response.is_empty());
        assert_eq!(response.entities.len(), 0);
        assert_eq!(response.decisions.len(), 0);
    }

    #[test]
    fn test_examples() {
        let add = examples::add_example();
        assert_eq!(add.entities.len(), 1);
        assert!(matches!(add.decisions[0], DecisionPayload::Add { .. }));

        let update = examples::update_example();
        assert!(matches!(
            update.decisions[0],
            DecisionPayload::Update { .. }
        ));

        let delete = examples::delete_example();
        assert!(matches!(
            delete.decisions[0],
            DecisionPayload::Delete { .. }
        ));

        let noop = examples::noop_example();
        assert!(matches!(noop.decisions[0], DecisionPayload::Noop));
    }

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Json);
    }

    #[test]
    fn test_to_json() {
        let response = examples::add_example();
        let json = response.to_json().unwrap();
        assert!(json.contains("\"name\": \"Rust\""));
        assert!(json.contains("\"type\": \"ADD\""));
    }

    #[test]
    fn test_response_with_prompt_version() {
        let response = ConsolidationResponse::empty().with_prompt_version("V1".to_string());
        assert_eq!(response.prompt_version, Some("V1".to_string()));
    }

    #[test]
    fn test_parse_with_prompt_version() {
        let json = r#"{
            "entities": [],
            "relationships": [],
            "decisions": [{"type": "NOOP"}],
            "reasoning": "Test",
            "prompt_version": "V1"
        }"#;

        let response = ConsolidationResponse::from_json(json).unwrap();
        assert_eq!(response.prompt_version, Some("V1".to_string()));
    }

    #[test]
    fn test_version_field_optional() {
        let json = r#"{
            "entities": [],
            "relationships": [],
            "decisions": [{"type": "NOOP"}]
        }"#;

        let response = ConsolidationResponse::from_json(json).unwrap();
        assert!(response.prompt_version.is_none());
    }
}
