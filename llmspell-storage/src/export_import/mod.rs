//! Export/Import functionality for PostgreSQL ↔ SQLite data migration
//!
//! This module provides bidirectional data migration between PostgreSQL and SQLite
//! backends, enabling growth paths (SQLite → PostgreSQL) and edge deployment
//! (PostgreSQL → SQLite).
//!
//! # Architecture
//!
//! - **Type Converters**: Handle PostgreSQL-specific types → SQLite equivalents
//! - **Exporter**: Extract data from any backend to standardized JSON format
//! - **Importer**: Load JSON data into any backend
//! - **Format**: Versioned JSON schema with metadata
//!
//! # Example
//!
//! ```ignore
//! use llmspell_storage::export_import::{StorageExporter, StorageImporter};
//!
//! // Export PostgreSQL to JSON
//! let exporter = StorageExporter::new(postgres_backend).await?;
//! exporter.export_to_file("dump.json").await?;
//!
//! // Import JSON to SQLite
//! let importer = StorageImporter::new(sqlite_backend).await?;
//! importer.import_from_file("dump.json").await?;
//! ```

pub mod converters;
pub mod exporter;
pub mod format;
pub mod importer;

pub use converters::{TypeConverter, TypeConverters};
#[cfg(feature = "postgres")]
pub use exporter::PostgresExporter;
#[cfg(feature = "sqlite")]
pub use exporter::SqliteExporter;
pub use format::ExportFormat;
pub use importer::StorageImporter;
