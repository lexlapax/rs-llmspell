//! Consolidation engine trait and decision types
//!
//! The consolidation engine transforms episodic memories (interaction history)
//! into semantic knowledge (entities and relationships in the knowledge graph).
//!
//! # Consolidation Process
//!
//! ```text
//! Episodic Entry → LLM Analysis → ConsolidationDecision → Knowledge Graph Update
//!
//! Decision Types:
//! - ADD: Create new entity/relationship
//! - UPDATE: Modify existing entity properties
//! - DELETE: Remove entity (with tombstone)
//! - NOOP: No changes needed
//! ```
//!
//! # Implementation Status
//!
//! This trait will be fully implemented in Phase 13.5 (Consolidation Engine).

use std::collections::HashMap;

use serde_json::Value;

use super::semantic::Entity;

/// Consolidation decision from LLM analysis
///
/// After analyzing an episodic entry, the LLM determines what knowledge
/// graph operations to perform.
///
/// # Example
///
/// ```rust,ignore
/// use llmspell_memory::traits::ConsolidationDecision;
/// use serde_json::json;
///
/// // From "Alice works at Acme Corp", extract:
/// let decisions = vec![
///     ConsolidationDecision::Add(Entity {
///         id: "person-alice".into(),
///         entity_type: "person".into(),
///         name: "Alice".into(),
///         properties: json!({}),
///         event_time: Utc::now(),
///         ingestion_time: Utc::now(),
///     }),
///     ConsolidationDecision::Add(Entity {
///         id: "company-acme".into(),
///         entity_type: "company".into(),
///         name: "Acme Corp".into(),
///         properties: json!({}),
///         event_time: Utc::now(),
///         ingestion_time: Utc::now(),
///     }),
///     // Relationship handled separately
/// ];
/// ```
#[derive(Debug, Clone)]
pub enum ConsolidationDecision {
    /// Add a new entity to the knowledge graph
    Add(Entity),

    /// Update an existing entity's properties
    Update {
        /// Entity ID to update
        entity_id: String,

        /// Property changes to apply (merge with existing)
        changes: HashMap<String, Value>,
    },

    /// Delete an entity (soft delete with tombstone)
    Delete {
        /// Entity ID to delete
        entity_id: String,
    },

    /// No operation needed (entry doesn't add new knowledge)
    Noop,
}

// Consolidation engine trait will be added in Phase 13.5
