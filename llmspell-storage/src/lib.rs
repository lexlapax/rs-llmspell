//! Storage backends and persistence for rs-llmspell
//!
//! Provides a unified storage interface with multiple backend implementations,
//! enabling flexible data persistence across different storage systems.
//!
//! # Overview
//!
//! The storage module abstracts data persistence through a common `StorageBackend`
//! trait that can be implemented by various storage systems. This allows applications
//! to switch between storage backends without changing business logic.
//!
//! # Available Backends
//!
//! - **MemoryBackend**: In-memory storage for testing and temporary data
//! - **SledBackend**: Embedded database for persistent local storage
//! - Custom backends can be implemented via the `StorageBackend` trait
//!
//! # Examples
//!
//! ## Using Memory Backend
//!
//! ```
//! use llmspell_storage::{MemoryBackend, StorageBackend};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let backend = MemoryBackend::new();
//!
//! // Store data
//! let value = json!({"name": "Alice"});
//! backend.set("user:123", serde_json::to_vec(&value)?).await?;
//!
//! // Retrieve data
//! let data = backend.get("user:123").await?;
//! if let Some(bytes) = data {
//!     let retrieved: serde_json::Value = serde_json::from_slice(&bytes)?;
//!     assert_eq!(retrieved, json!({"name": "Alice"}));
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Using Sled Backend
//!
//! ```no_run
//! use llmspell_storage::{SledBackend, StorageBackend};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let backend = SledBackend::new_with_path("./data/storage")?;
//!
//! // Data persists across restarts
//! let value = json!({"version": "1.0"});
//! backend.set("config:app", serde_json::to_vec(&value)?).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Characteristics
//!
//! ## MemoryBackend
//! - **Read**: O(1) average, <1μs
//! - **Write**: O(1) average, <1μs
//! - **Memory**: All data in RAM
//! - **Persistence**: None (data lost on restart)
//! - **Use Case**: Testing, caching, temporary data
//!
//! ## SledBackend
//! - **Read**: O(log n), <100μs for most operations
//! - **Write**: O(log n), <1ms with fsync
//! - **Memory**: Configurable cache, defaults to 1GB
//! - **Persistence**: ACID compliant, crash-safe
//! - **Use Case**: Production embedded database
//!
//! # Integration Example
//!
//! ```no_run
//! use llmspell_storage::{StorageBackend, StorageBackendType, MemoryBackend, SledBackend};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Factory pattern for backend selection
//! fn create_backend(backend_type: StorageBackendType) -> Result<Arc<dyn StorageBackend>, Box<dyn std::error::Error>> {
//!     match backend_type {
//!         StorageBackendType::Memory => Ok(Arc::new(MemoryBackend::new())),
//!         StorageBackendType::Sled => Ok(Arc::new(SledBackend::new_with_path("./data")?)),
//!         _ => Err("Unsupported backend".into()),
//!     }
//! }
//!
//! // Use backend agnostically
//! let backend = create_backend(StorageBackendType::Memory)?;
//! let value = serde_json::json!({"value": 42});
//! backend.set("key", serde_json::to_vec(&value)?).await?;
//! # Ok(())
//! # }
//! ```

pub mod backends;
pub mod traits;

// Re-export commonly used types
pub use backends::{MemoryBackend, SledBackend};
pub use traits::{StorageBackend, StorageBackendType, StorageCharacteristics, StorageSerialize};
