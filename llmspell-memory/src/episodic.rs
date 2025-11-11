//! Episodic memory implementations (vector-indexed interaction history)
//!
//! This module provides multiple storage backends for episodic memory:
//! - `HNSWEpisodicMemory` (production default, O(log n) search via llmspell-storage) - **DEPRECATED: will be removed in Task 13c.2.8**
//! - `SqliteEpisodicMemory` (production default, O(log n) search with SQLite + HNSW, persistent local storage)
//! - `InMemoryEpisodicMemory` (testing/development, simple `HashMap` with O(n) search)
//! - `PostgreSQLEpisodicMemory` (production, O(log n) search with pgvector, RLS multi-tenancy)
//! - `EpisodicBackend` (enum dispatch over backends, selected via config)
//! - `ChromaDBEpisodicMemory` (future, optional external service)
//! - `QdrantEpisodicMemory` (future, optional external service)

pub mod backend;
pub mod hnsw_backend;
pub mod in_memory;
#[cfg(feature = "postgres")]
pub mod postgresql_backend;
pub mod sqlite_backend;

pub use backend::EpisodicBackend;
pub use hnsw_backend::HNSWEpisodicMemory;
pub use in_memory::InMemoryEpisodicMemory;
#[cfg(feature = "postgres")]
pub use postgresql_backend::PostgreSQLEpisodicMemory;
pub use sqlite_backend::SqliteEpisodicMemory;
