//! ABOUTME: PostgreSQL storage backend (Phase 13b.2 infrastructure)
//! ABOUTME: Connection pooling, tenant context, and configuration for PostgreSQL with VectorChord

mod backend;
mod config;
mod error;
pub mod graph; // Phase 13b.5.2: Bi-temporal graph storage with time-travel queries
mod migrations;
mod pool;
pub mod rls; // Phase 13b.3.1: RLS policy generation helpers
mod vector; // Phase 13b.4.2: PostgreSQL vector storage with dimension routing

pub use backend::PostgresBackend;
pub use config::PostgresConfig;
pub use error::{PostgresError, Result};
pub use graph::PostgresGraphStorage; // Phase 13b.5.2
pub use pool::PostgresPool;
pub use vector::PostgreSQLVectorStorage; // Phase 13b.4.2
