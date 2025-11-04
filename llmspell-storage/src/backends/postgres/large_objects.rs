//! ABOUTME: PostgreSQL Large Object streaming API (Phase 13b.10.2)
//! ABOUTME: Efficient streaming upload/download for artifacts >=1MB using PostgreSQL Large Objects
//!
//! # Architecture
//!
//! - **Streaming Upload**: Chunks data and writes via `lowrite()` in transaction
//! - **Streaming Download**: Reads chunks via `loread()` in transaction
//! - **Cleanup**: Orphaned Large Object detection and removal
//! - **Chunk Size**: 1MB default (configurable)
//!
//! # Large Object Modes
//!
//! - `INV_WRITE` (0x20000): Write mode
//! - `INV_READ` (0x40000): Read mode
//!
//! # Transaction Requirements
//!
//! All Large Object operations must occur within a transaction.
//! The API handles transaction management automatically.

use super::error::{PostgresError, Result};
use deadpool_postgres::Object as PoolClient;
use tokio_postgres::types::Oid;

/// Default chunk size for streaming (1MB)
const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

/// Large Object mode constants
const INV_WRITE: i32 = 0x20000;
const INV_READ: i32 = 0x40000;

/// Large Object streaming API
pub struct LargeObjectStream {
    /// Database connection pool client
    client: PoolClient,
    /// Chunk size for streaming operations
    chunk_size: usize,
}

impl LargeObjectStream {
    /// Create a new Large Object stream handler
    ///
    /// # Arguments
    /// * `client` - Database connection from pool
    ///
    /// # Returns
    /// * `Self` - New Large Object stream handler
    pub fn new(client: PoolClient) -> Self {
        Self {
            client,
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }

    /// Set custom chunk size for streaming operations
    ///
    /// # Arguments
    /// * `size` - Chunk size in bytes
    ///
    /// # Returns
    /// * `Self` - Self for method chaining
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Upload data as a Large Object with streaming
    ///
    /// # Arguments
    /// * `data` - Data to upload
    ///
    /// # Returns
    /// * `Result<u32>` - Large Object OID on success
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, LargeObjectStream};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = PostgresConfig::new("postgresql://localhost/llmspell_dev");
    ///     let backend = PostgresBackend::new(config).await.unwrap();
    ///     let client = backend.get_client().await.unwrap();
    ///
    ///     let stream = LargeObjectStream::new(client);
    ///     let data = vec![0u8; 10_000_000]; // 10MB
    ///     let oid = stream.upload(&data).await.unwrap();
    ///     println!("Uploaded Large Object: {}", oid);
    /// }
    /// ```
    pub async fn upload(&mut self, data: &[u8]) -> Result<u32> {
        // Start transaction
        let tx = self
            .client
            .transaction()
            .await
            .map_err(|e| PostgresError::Connection(format!("Failed to start transaction: {}", e)))?;

        // Create new Large Object
        let row = tx
            .query_one("SELECT lo_create(0)", &[])
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to create Large Object: {}", e))
            })?;
        let oid: u32 = row.get(0);

        // Open Large Object for writing
        let row = tx
            .query_one("SELECT lo_open($1, $2)", &[&Oid::from(oid), &INV_WRITE])
            .await
            .map_err(|e| PostgresError::Query(format!("Failed to open Large Object: {}", e)))?;
        let fd: i32 = row.get(0);

        // Write data in chunks
        for chunk in data.chunks(self.chunk_size) {
            tx.execute("SELECT lowrite($1, $2)", &[&fd, &chunk])
                .await
                .map_err(|e| {
                    PostgresError::Query(format!("Failed to write to Large Object: {}", e))
                })?;
        }

        // Close Large Object
        tx.execute("SELECT lo_close($1)", &[&fd])
            .await
            .map_err(|e| PostgresError::Query(format!("Failed to close Large Object: {}", e)))?;

        // Commit transaction
        tx.commit().await.map_err(|e| {
            PostgresError::Connection(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(oid)
    }

    /// Download Large Object with streaming
    ///
    /// # Arguments
    /// * `oid` - Large Object OID to download
    ///
    /// # Returns
    /// * `Result<Vec<u8>>` - Downloaded data on success
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, LargeObjectStream};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = PostgresConfig::new("postgresql://localhost/llmspell_dev");
    ///     let backend = PostgresBackend::new(config).await.unwrap();
    ///     let client = backend.get_client().await.unwrap();
    ///
    ///     let stream = LargeObjectStream::new(client);
    ///     let oid = 12345;
    ///     let data = stream.download(oid).await.unwrap();
    ///     println!("Downloaded {} bytes", data.len());
    /// }
    /// ```
    pub async fn download(&mut self, oid: u32) -> Result<Vec<u8>> {
        // Start transaction
        let tx = self
            .client
            .transaction()
            .await
            .map_err(|e| PostgresError::Connection(format!("Failed to start transaction: {}", e)))?;

        // Open Large Object for reading
        let row = tx
            .query_one("SELECT lo_open($1, $2)", &[&Oid::from(oid), &INV_READ])
            .await
            .map_err(|e| PostgresError::Query(format!("Failed to open Large Object: {}", e)))?;
        let fd: i32 = row.get(0);

        // Read data in chunks
        let mut buffer = Vec::new();
        loop {
            let row = tx
                .query_one(
                    "SELECT loread($1, $2)",
                    &[&fd, &(self.chunk_size as i32)],
                )
                .await
                .map_err(|e| {
                    PostgresError::Query(format!("Failed to read from Large Object: {}", e))
                })?;

            let chunk: Vec<u8> = row.get(0);
            if chunk.is_empty() {
                break;
            }
            buffer.extend_from_slice(&chunk);
        }

        // Close Large Object
        tx.execute("SELECT lo_close($1)", &[&fd])
            .await
            .map_err(|e| PostgresError::Query(format!("Failed to close Large Object: {}", e)))?;

        // Commit transaction
        tx.commit().await.map_err(|e| {
            PostgresError::Connection(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(buffer)
    }

    /// Delete a Large Object
    ///
    /// # Arguments
    /// * `oid` - Large Object OID to delete
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, LargeObjectStream};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = PostgresConfig::new("postgresql://localhost/llmspell_dev");
    ///     let backend = PostgresBackend::new(config).await.unwrap();
    ///     let client = backend.get_client().await.unwrap();
    ///
    ///     let stream = LargeObjectStream::new(client);
    ///     let oid = 12345;
    ///     stream.delete(oid).await.unwrap();
    ///     println!("Deleted Large Object: {}", oid);
    /// }
    /// ```
    pub async fn delete(&mut self, oid: u32) -> Result<()> {
        // Start transaction
        let tx = self
            .client
            .transaction()
            .await
            .map_err(|e| PostgresError::Connection(format!("Failed to start transaction: {}", e)))?;

        // Delete Large Object
        tx.execute("SELECT lo_unlink($1)", &[&Oid::from(oid)])
            .await
            .map_err(|e| PostgresError::Query(format!("Failed to delete Large Object: {}", e)))?;

        // Commit transaction
        tx.commit().await.map_err(|e| {
            PostgresError::Connection(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    /// Find orphaned Large Objects not referenced in artifact_content table
    ///
    /// # Returns
    /// * `Result<Vec<u32>>` - List of orphaned Large Object OIDs
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, LargeObjectStream};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = PostgresConfig::new("postgresql://localhost/llmspell_dev");
    ///     let backend = PostgresBackend::new(config).await.unwrap();
    ///     let client = backend.get_client().await.unwrap();
    ///
    ///     let stream = LargeObjectStream::new(client);
    ///     let orphaned = stream.find_orphaned_objects().await.unwrap();
    ///     println!("Found {} orphaned Large Objects", orphaned.len());
    /// }
    /// ```
    pub async fn find_orphaned_objects(&self) -> Result<Vec<u32>> {
        let rows = self
            .client
            .query(
                "SELECT oid FROM pg_largeobject_metadata
                 WHERE oid NOT IN (
                     SELECT large_object_oid FROM llmspell.artifact_content
                     WHERE large_object_oid IS NOT NULL
                 )",
                &[],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to find orphaned Large Objects: {}", e))
            })?;

        Ok(rows.iter().map(|row| row.get::<_, u32>(0)).collect())
    }

    /// Cleanup orphaned Large Objects
    ///
    /// # Returns
    /// * `Result<usize>` - Number of Large Objects cleaned up
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, LargeObjectStream};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = PostgresConfig::new("postgresql://localhost/llmspell_dev");
    ///     let backend = PostgresBackend::new(config).await.unwrap();
    ///     let client = backend.get_client().await.unwrap();
    ///
    ///     let mut stream = LargeObjectStream::new(client);
    ///     let count = stream.cleanup_orphaned_objects().await.unwrap();
    ///     println!("Cleaned up {} orphaned Large Objects", count);
    /// }
    /// ```
    pub async fn cleanup_orphaned_objects(&mut self) -> Result<usize> {
        let orphaned = self.find_orphaned_objects().await?;
        let mut cleaned = 0;

        for oid in orphaned {
            // Ignore errors for already-deleted objects
            if self.delete(oid).await.is_ok() {
                cleaned += 1;
            }
        }

        Ok(cleaned)
    }

    /// Check if a Large Object exists
    ///
    /// # Arguments
    /// * `oid` - Large Object OID to check
    ///
    /// # Returns
    /// * `Result<bool>` - True if Large Object exists
    pub async fn exists(&self, oid: u32) -> Result<bool> {
        let row = self
            .client
            .query_opt(
                "SELECT 1 FROM pg_largeobject_metadata WHERE oid = $1",
                &[&Oid::from(oid)],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to check Large Object existence: {}", e))
            })?;

        Ok(row.is_some())
    }

    /// Get Large Object size in bytes
    ///
    /// # Arguments
    /// * `oid` - Large Object OID
    ///
    /// # Returns
    /// * `Result<i64>` - Size in bytes
    pub async fn size(&mut self, oid: u32) -> Result<i64> {
        // Start transaction
        let tx = self
            .client
            .transaction()
            .await
            .map_err(|e| PostgresError::Connection(format!("Failed to start transaction: {}", e)))?;

        // Open Large Object for reading
        let row = tx
            .query_one("SELECT lo_open($1, $2)", &[&Oid::from(oid), &INV_READ])
            .await
            .map_err(|e| PostgresError::Query(format!("Failed to open Large Object: {}", e)))?;
        let fd: i32 = row.get(0);

        // Seek to end to get size
        let row = tx
            .query_one("SELECT lo_lseek64($1, 0, 2)", &[&fd])
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to seek in Large Object: {}", e))
            })?;
        let size: i64 = row.get(0);

        // Close Large Object
        tx.execute("SELECT lo_close($1)", &[&fd])
            .await
            .map_err(|e| PostgresError::Query(format!("Failed to close Large Object: {}", e)))?;

        // Commit transaction
        tx.commit().await.map_err(|e| {
            PostgresError::Connection(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests will be in separate test file: tests/postgres_large_objects_tests.rs
}
