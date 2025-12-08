//! ABOUTME: Database migration management for SQLite
//! ABOUTME: Applies versioned SQL migrations from migrations/sqlite/ directory

use super::SqliteBackend;
use rusqlite::params;

impl SqliteBackend {
    /// Run all database migrations
    ///
    /// Applies all SQL migration files from `migrations/sqlite/` directory.
    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        let conn = self.get_connection().await?;

        // Apply migrations in order
        // V1: Initial setup (PRAGMA, _migrations table)
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V1__initial_setup.sql"
        ))
        .map_err(|e| anyhow::anyhow!("V1 migration failed: {}", e))?;

        // V3: Vector embeddings
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V3__vector_embeddings.sql"
        ))
        .map_err(|e| anyhow::anyhow!("V3 migration failed: {}", e))?;

        // V4: Temporal graph
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V4__temporal_graph.sql"
        ))
        .map_err(|e| anyhow::anyhow!("V4 migration failed: {}", e))?;

        // V5: Procedural memory
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V5__procedural_memory.sql"
        ))
        .map_err(|e| anyhow::anyhow!("V5 migration failed: {}", e))?;

        // V6: Agent state
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V6__agent_state.sql"
        ))
        .map_err(|e| anyhow::anyhow!("V6 migration failed: {}", e))?;

        // V7: KV store
        conn.execute_batch(include_str!("../../../migrations/sqlite/V7__kv_store.sql"))
            .map_err(|e| anyhow::anyhow!("V7 migration failed: {}", e))?;

        // V8: Workflow states
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V8__workflow_states.sql"
        ))
        .map_err(|e| anyhow::anyhow!("V8 migration failed: {}", e))?;

        // V9: Sessions
        conn.execute_batch(include_str!("../../../migrations/sqlite/V9__sessions.sql"))
            .map_err(|e| anyhow::anyhow!("V9 migration failed: {}", e))?;

        // V10: Artifacts
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V10__artifacts.sql"
        ))
        .map_err(|e| anyhow::anyhow!("V10 migration failed: {}", e))?;

        // V11: Event log
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V11__event_log.sql"
        ))
        .map_err(|e| anyhow::anyhow!("V11 migration failed: {}", e))?;

        // V13: Hook history
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V13__hook_history.sql"
        ))
        .map_err(|e| anyhow::anyhow!("V13 migration failed: {}", e))?;

        Ok(())
    }

    /// Get current migration version
    pub async fn migration_version(&self) -> anyhow::Result<usize> {
        let conn = self.get_connection().await?;

        // Query _migrations table for highest version
        // Rusqlite sync call
        let version: Option<i64> = conn.query_row(
            "SELECT MAX(version) FROM _migrations",
            [],
            |row| row.get(0)
        )
        .map_err(|e| anyhow::anyhow!("Failed to query migration version: {}", e))?;

        Ok(version.unwrap_or(0) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::sqlite::SqliteConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_run_migrations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_migrations.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap());
        let backend = SqliteBackend::new(config).await.unwrap();

        // Run migrations
        backend.run_migrations().await.unwrap();

        // Verify migration version
        let version = backend.migration_version().await.unwrap();
        assert_eq!(version, 13, "Should have applied migration V13");

        // Verify tables exist
        let conn = backend.get_connection().await.unwrap();

        // Check kv_store table
        let exists: bool = conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='kv_store'",
            [],
            |_| Ok(true)
        ).unwrap_or(false);
        assert!(exists, "kv_store table should exist");

        // Check agent_states table
        let exists: bool = conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='agent_states'",
            [],
            |_| Ok(true)
        ).unwrap_or(false);
        assert!(exists, "agent_states table should exist");
    }

    #[tokio::test]
    async fn test_migrations_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_idempotent.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap());
        let backend = SqliteBackend::new(config).await.unwrap();

        // Run migrations twice
        backend.run_migrations().await.unwrap();
        backend.run_migrations().await.unwrap();

        // Should still be at version 13
        let version = backend.migration_version().await.unwrap();
        assert_eq!(version, 13);
    }
}

