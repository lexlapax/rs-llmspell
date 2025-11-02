//! ABOUTME: Database migration management using Refinery
//! ABOUTME: Versioned schema migrations for PostgreSQL with VectorChord

use super::error::{PostgresError, Result};
use super::PostgresBackend;
use refinery::embed_migrations;

// Embed migrations from migrations/ directory at compile time
embed_migrations!("migrations");

impl PostgresBackend {
    /// Run all pending database migrations
    ///
    /// Uses Refinery to apply versioned SQL migrations from `migrations/` directory.
    /// Migrations are idempotent and tracked in `refinery_schema_history` table.
    ///
    /// # Returns
    /// * `Result<()>` - Success or migration error
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = PostgresConfig::new("postgresql://localhost/llmspell_dev");
    ///     let backend = PostgresBackend::new(config).await.unwrap();
    ///     backend.run_migrations().await.unwrap();
    /// }
    /// ```
    pub async fn run_migrations(&self) -> Result<()> {
        // Get connection from pool
        let mut client = self.get_client().await?;

        // Run migrations using refinery
        self::migrations::runner()
            .run_async(&mut **client)
            .await
            .map_err(|e| PostgresError::Migration(format!("Migration failed: {}", e)))?;

        Ok(())
    }

    /// Get current migration version
    ///
    /// Returns the version number of the last applied migration,
    /// or 0 if no migrations have been run.
    ///
    /// # Returns
    /// * `Result<usize>` - Current migration version
    pub async fn migration_version(&self) -> Result<usize> {
        let client = self.get_client().await?;

        // Query refinery_schema_history table for latest version
        let row = client
            .query_opt(
                "SELECT version FROM refinery_schema_history ORDER BY version DESC LIMIT 1",
                &[],
            )
            .await
            .map_err(|e| PostgresError::Query(format!("Failed to query migration version: {}", e)))?;

        if let Some(row) = row {
            let version: i32 = row.get(0);
            Ok(version as usize)
        } else {
            Ok(0) // No migrations run yet
        }
    }
}
