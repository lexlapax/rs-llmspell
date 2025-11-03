//! ABOUTME: PostgreSQL storage backend (Phase 13b.2 infrastructure)
//! ABOUTME: Connection pooling, tenant context, and configuration for PostgreSQL with VectorChord

mod backend;
mod config;
mod error;
mod migrations;
mod pool;
pub mod rls; // Phase 13b.3.1: RLS policy generation helpers

pub use backend::PostgresBackend;
pub use config::PostgresConfig;
pub use error::{PostgresError, Result};
pub use pool::PostgresPool;
