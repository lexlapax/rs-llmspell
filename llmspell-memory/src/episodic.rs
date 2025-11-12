//! Episodic memory implementations (vector-indexed interaction history)
//!
//! This module provides multiple storage backends for episodic memory:
//! - `SqliteEpisodicMemory` (production default, O(log n) search with SQLite + vectorlite HNSW, persistent local storage)
//! - `PostgreSQLEpisodicMemory` (production, O(log n) search with pgvector, RLS multi-tenancy)
//! - `InMemoryEpisodicMemory` (testing/development, simple `HashMap` with O(n) search)
//! - `EpisodicBackend` (enum dispatch over backends, selected via config)
//! - `ChromaDBEpisodicMemory` (future, optional external service)
//! - `QdrantEpisodicMemory` (future, optional external service)

pub mod backend;
pub mod in_memory;
#[cfg(feature = "postgres")]
pub mod postgresql_backend;
pub mod sqlite_backend;

pub use backend::EpisodicBackend;
pub use in_memory::InMemoryEpisodicMemory;
#[cfg(feature = "postgres")]
pub use postgresql_backend::PostgreSQLEpisodicMemory;
pub use sqlite_backend::SqliteEpisodicMemory;
