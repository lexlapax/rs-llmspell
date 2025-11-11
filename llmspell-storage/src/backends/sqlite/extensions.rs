//! SQLite extension management for vector search
//!
//! This module provides integration with sqlite-vec for brute-force vector search.
//! For HNSW-indexed search (3-100x faster), see Task 13c.2.2a (vectorlite-rs).
//!
//! # Architecture
//!
//! - **sqlite-vec**: Brute-force K-NN search via vec0 virtual table
//! - **Performance**: O(N) complexity, suitable for <100K vectors
//! - **Future**: vectorlite-rs will provide HNSW indexing as optional optimization
//!
//! # Usage
//!
//! ```no_run
//! use llmspell_storage::backends::sqlite::SqliteVecExtension;
//!
//! // Extension is registered automatically in SqliteBackend::new()
//! // Check availability:
//! let available = SqliteVecExtension::is_available(&conn)?;
//! ```

use anyhow::{Context, Result};
#[cfg(feature = "sqlite")]
use libsql::Connection;

/// Vector extension wrapper for sqlite-vec
///
/// NOTE: sqlite-vec uses brute-force search (O(N) complexity).
/// For HNSW-indexed search (3-100x faster), see Task 13c.2.2a (vectorlite-rs).
pub struct SqliteVecExtension;

impl SqliteVecExtension {
    /// Check if vec0 virtual table module is available
    ///
    /// # Errors
    ///
    /// Returns error if query fails or extension not loaded
    ///
    /// # Example
    ///
    /// ```no_run
    /// use llmspell_storage::backends::sqlite::SqliteVecExtension;
    /// # use anyhow::Result;
    ///
    /// # fn example(conn: &libsql::Connection) -> Result<()> {
    /// let available = SqliteVecExtension::is_available(conn)?;
    /// assert!(available, "sqlite-vec extension should be loaded");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "sqlite")]
    pub async fn is_available(conn: &Connection) -> Result<bool> {
        let mut rows = conn
            .query("SELECT vec_version()", ())
            .await
            .context("Failed to query vec_version()")?;

        let row = rows
            .next()
            .await
            .context("No row returned from vec_version()")?;

        if let Some(row) = row {
            let version: String = row
                .get(0)
                .context("Failed to get version from vec_version()")?;
            Ok(!version.is_empty())
        } else {
            Ok(false)
        }
    }

    /// Get supported vector dimensions (384, 768, 1536, 3072)
    ///
    /// These dimensions cover all common embedding models:
    /// - 384: sentence-transformers/all-MiniLM-L6-v2
    /// - 768: OpenAI ada-002, BERT-base
    /// - 1536: OpenAI text-embedding-3-small
    /// - 3072: OpenAI text-embedding-3-large
    ///
    /// # Example
    ///
    /// ```
    /// use llmspell_storage::backends::sqlite::SqliteVecExtension;
    ///
    /// let dims = SqliteVecExtension::supported_dimensions();
    /// assert_eq!(dims, &[384, 768, 1536, 3072]);
    /// ```
    #[must_use]
    pub const fn supported_dimensions() -> &'static [usize] {
        &[384, 768, 1536, 3072]
    }

    /// Check if a dimension is supported
    ///
    /// # Example
    ///
    /// ```
    /// use llmspell_storage::backends::sqlite::SqliteVecExtension;
    ///
    /// assert!(SqliteVecExtension::is_dimension_supported(768));
    /// assert!(!SqliteVecExtension::is_dimension_supported(512));
    /// ```
    #[must_use]
    pub fn is_dimension_supported(dimension: usize) -> bool {
        Self::supported_dimensions().contains(&dimension)
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;

    #[test]
    fn test_supported_dimensions() {
        let dims = SqliteVecExtension::supported_dimensions();
        assert_eq!(dims.len(), 4);
        assert_eq!(dims, &[384, 768, 1536, 3072]);
    }

    #[test]
    fn test_dimension_validation() {
        assert!(SqliteVecExtension::is_dimension_supported(384));
        assert!(SqliteVecExtension::is_dimension_supported(768));
        assert!(SqliteVecExtension::is_dimension_supported(1536));
        assert!(SqliteVecExtension::is_dimension_supported(3072));

        assert!(!SqliteVecExtension::is_dimension_supported(128));
        assert!(!SqliteVecExtension::is_dimension_supported(512));
        assert!(!SqliteVecExtension::is_dimension_supported(2048));
    }

    // NOTE: Integration tests for actual extension loading are deferred to Task 13c.2.2 completion
    // due to complexity of sqlite-vec + libsql integration:
    //
    // - sqlite-vec uses sqlite3_auto_extension (rusqlite FFI)
    // - libsql has its own embedded SQLite
    // - Need to use libsql::Connection::load_extension_enable() and load custom .so
    // - Requires compiling sqlite-vec as loadable extension
    //
    // Current approach: Tests validate API surface, actual extension loading will be tested
    // via SqliteBackend integration tests once extension loading is properly configured.
    //
    // See TODO.md Task 13c.2.2 for tracking remaining work:
    // - Compile sqlite-vec as loadable extension (.so/.dylib/.dll)
    // - Configure libsql to load extension via load_extension() API
    // - Integration tests in SqliteBackend with vec0 virtual table creation
}
