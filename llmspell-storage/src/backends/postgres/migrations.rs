//! ABOUTME: Database migration management using Refinery
//! ABOUTME: Versioned schema migrations for PostgreSQL with VectorChord

use super::PostgresBackend;
use refinery::embed_migrations;

// Embed PostgreSQL migrations from migrations/postgres/ directory at compile time
// Phase 13c.2.1: Migrated from migrations/ to migrations/postgres/ for backend-specific SQL
embed_migrations!("migrations/postgres");

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
    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        // Get connection from pool
        let mut client = self.get_client().await?;

        // Set search_path to llmspell schema so refinery_schema_history is created there
        // This ensures llmspell_app role can access the migrations table
        client
            .execute("SET search_path TO llmspell, public", &[])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set search_path: {}", e))?;

        // Run migrations using refinery
        self::migrations::runner()
            .run_async(&mut **client)
            .await
            .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

        Ok(())
    }

    /// Get current migration version
    ///
    /// Returns the version number of the last applied migration,
    /// or 0 if no migrations have been run.
    ///
    /// # Returns
    /// * `Result<usize>` - Current migration version
    pub async fn migration_version(&self) -> anyhow::Result<usize> {
        let client = self.get_client().await?;

        // Query refinery_schema_history table for latest version (in llmspell schema)
        let row = client
            .query_opt(
                "SELECT version FROM llmspell.refinery_schema_history ORDER BY version DESC LIMIT 1",
                &[],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query migration version: {}", e))?;

        if let Some(row) = row {
            let version: i32 = row.get(0);
            Ok(version as usize)
        } else {
            Ok(0) // No migrations run yet
        }
    }
}
