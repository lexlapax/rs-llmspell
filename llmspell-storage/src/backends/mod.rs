//! ABOUTME: Storage backend implementations
//! ABOUTME: Provides memory, sled, vector, postgres, and sqlite backends

pub mod memory;
pub mod sled_backend;
pub mod vector;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use memory::MemoryBackend;
pub use sled_backend::SledBackend;

#[cfg(feature = "postgres")]
pub use postgres::{PostgresBackend, PostgresConfig, PostgresError, PostgresPool};

#[cfg(feature = "sqlite")]
pub use sqlite::{
    HealthStatus, SqliteBackend, SqliteConfig, SqliteError, SqlitePool, TenantContext,
};
