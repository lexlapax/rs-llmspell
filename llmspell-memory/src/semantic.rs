//! Semantic memory implementations (bi-temporal knowledge graph)
//!
//! This module will contain:
//! - `SurrealDBSemanticMemory` (default, using `SurrealDB`)
//! - `Neo4jSemanticMemory` (optional external service)
//! - `InMemorySemanticMemory` (testing/development)
//!
//! Features:
//! - Bi-temporal tracking (`event_time` + `ingestion_time`)
//! - Entity extraction and relationship mapping
//! - Temporal queries (point-in-time, time-travel)
//!
//! To be implemented in Phase 13.2
