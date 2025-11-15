//! Storage backend types
//!
//! Domain types for StorageBackend trait including:
//! - `StorageBackendType`: Backend identifier enum (Memory, Sqlite, Postgres)
//! - `StorageCharacteristics`: Capability and performance characteristics
//!
//! Migrated from llmspell-storage/src/traits.rs as part of Phase 13c.3.

use serde::{Deserialize, Serialize};

/// Type of storage backend
///
/// Identifies which storage implementation is being used. Each backend has
/// different characteristics in terms of persistence, performance, and features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackendType {
    /// In-memory storage (for testing/development)
    Memory,

    /// SQLite database (Phase 13c, unified storage)
    Sqlite,

    /// PostgreSQL database with VectorChord (Phase 13b.2+)
    #[cfg(feature = "postgres")]
    Postgres,
}

/// Storage backend characteristics
///
/// Describes the capabilities and performance characteristics of a storage backend.
/// Used for runtime backend selection and performance tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCharacteristics {
    /// Whether the backend persists data
    pub persistent: bool,

    /// Whether the backend supports transactions
    pub transactional: bool,

    /// Whether the backend supports key prefix scanning
    pub supports_prefix_scan: bool,

    /// Whether the backend supports atomic operations
    pub supports_atomic_ops: bool,

    /// Estimated read latency in microseconds
    pub avg_read_latency_us: u64,

    /// Estimated write latency in microseconds
    pub avg_write_latency_us: u64,
}
