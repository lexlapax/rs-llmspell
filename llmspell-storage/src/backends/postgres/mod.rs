//! ABOUTME: PostgreSQL storage backend (Phase 13b.2 infrastructure)
//! ABOUTME: Connection pooling, tenant context, and configuration for PostgreSQL with VectorChord

pub mod artifacts; // Phase 13b.10.3: Artifact backend with automatic BYTEA/Large Object routing
mod backend;
mod config;
mod error;
pub mod event_log; // Phase 13b.11.2: Event log storage with monthly partitioning
pub mod graph; // Phase 13b.5.2: Bi-temporal graph storage with time-travel queries
pub mod large_objects; // Phase 13b.10.2: Large Object streaming API for artifacts >=1MB
mod migrations;
mod pool;
pub mod procedural; // Phase 13b.6.2: Procedural memory pattern storage
pub mod rls; // Phase 13b.3.1: RLS policy generation helpers
mod vector; // Phase 13b.4.2: PostgreSQL vector storage with dimension routing

pub use artifacts::ArtifactStats; // Phase 13b.10.3
pub use backend::PostgresBackend;
pub use config::PostgresConfig;
pub use error::{PostgresError, Result};
pub use event_log::{EventStorageStats, PostgresEventLogStorage}; // Phase 13b.11.2
pub use graph::PostgresGraphStorage; // Phase 13b.5.2
pub use large_objects::LargeObjectStream; // Phase 13b.10.2
pub use pool::PostgresPool;
pub use procedural::{PostgresProceduralStorage, StoredPattern}; // Phase 13b.6.2
pub use vector::PostgreSQLVectorStorage; // Phase 13b.4.2
