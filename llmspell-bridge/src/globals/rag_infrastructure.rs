//! ABOUTME: Shared RAG infrastructure initialization for multi-tenant vector storage
//! ABOUTME: Provides `get_or_create` pattern for RAG management following session infrastructure pattern
//!
//! Phase 13c.2.8.13a: Migrated from HNSWVectorStorage to SqliteVectorStorage (vectorlite-rs)

use crate::globals::GlobalContext;
use llmspell_config::RAGConfig;
use llmspell_core::{error::LLMSpellError, Result};
use llmspell_events::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_kernel::sessions::{SessionManager, SessionManagerConfig};
use llmspell_kernel::state::StateManager;
use llmspell_rag::multi_tenant_integration::MultiTenantRAG;
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};
use llmspell_storage::{MemoryBackend, StorageBackend, VectorStorage};
use llmspell_tenancy::MultiTenantVectorManager;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Helper function to get or create RAG infrastructure from `GlobalContext`
///
/// # Errors
///
/// Returns an error if:
/// - Vector storage creation fails
/// - State manager creation fails
/// - Session manager creation fails
/// - Multi-tenant RAG initialization fails
pub async fn get_or_create_rag_infrastructure(
    context: &GlobalContext,
    config: &RAGConfig,
) -> Result<RAGInfrastructure> {
    // Check if already initialized
    if let Some(multi_tenant_rag) = context.get_bridge::<MultiTenantRAG>("multi_tenant_rag") {
        debug!("Using existing RAG infrastructure from GlobalContext");
        return Ok(RAGInfrastructure {
            multi_tenant_rag,
            vector_storage: None, // Storage is embedded in multi_tenant_rag
            sqlite_storage: context.get_bridge::<SqliteVectorStorage>("sqlite_storage"),
        });
    }

    // Initialize new infrastructure
    info!("Initializing RAG infrastructure");

    // Get or create required dependencies
    let state_manager = get_or_create_state_manager(context).await?;
    let session_manager = get_or_create_session_manager(context, &state_manager)?;

    // Create vector storage based on configuration
    let (vector_storage, sqlite_storage) = create_vector_storage(config, context).await?;

    // Create multi-tenant vector manager
    let tenant_manager = Arc::new(MultiTenantVectorManager::new(vector_storage.clone()));

    // Create multi-tenant RAG
    let multi_tenant_rag = Arc::new(MultiTenantRAG::new(tenant_manager));

    // Store dependencies in context for auto-detection by register_rag_global
    context.set_bridge("state_manager", state_manager);
    context.set_bridge("session_manager", session_manager);
    context.set_bridge("multi_tenant_rag", multi_tenant_rag.clone());

    Ok(RAGInfrastructure {
        multi_tenant_rag,
        vector_storage: Some(vector_storage),
        sqlite_storage: Some(sqlite_storage),
    })
}

/// Get or create `StateManager`
async fn get_or_create_state_manager(context: &GlobalContext) -> Result<Arc<StateManager>> {
    if let Some(state_manager) = context.get_bridge::<StateManager>("state_manager") {
        return Ok(state_manager);
    }

    // Create StateManager for RAG
    debug!("Creating StateManager for RAG infrastructure");
    let state_manager =
        Arc::new(
            StateManager::new(None)
                .await
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to create StateManager for RAG: {e}"),
                    source: None,
                })?,
        );

    Ok(state_manager)
}

/// Get or create `SessionManager`
fn get_or_create_session_manager(
    context: &GlobalContext,
    state_manager: &Arc<StateManager>,
) -> Result<Arc<SessionManager>> {
    if let Some(session_manager) = context.get_bridge::<SessionManager>("session_manager") {
        return Ok(session_manager);
    }

    // Create dependencies for SessionManager
    debug!("Creating SessionManager for RAG infrastructure");
    let hook_registry = get_or_create_hook_registry(context);
    let hook_executor = get_or_create_hook_executor(context);
    let event_bus = get_or_create_event_bus(context);
    let storage_backend = create_storage_backend("memory")?;

    // Create SessionManagerConfig with defaults suitable for RAG
    let session_config = SessionManagerConfig::builder()
        .max_active_sessions(100)
        .default_session_timeout(chrono::Duration::hours(1))
        .storage_path(std::path::PathBuf::from("./rag_sessions"))
        .auto_persist(false) // RAG doesn't need session persistence by default
        .track_activity(true)
        .max_storage_size_bytes(100 * 1024 * 1024) // 100MB for RAG sessions
        .enable_compression(false)
        .enable_deduplication(false)
        .build();

    // Create SessionManager
    let session_manager = Arc::new(
        SessionManager::new(
            state_manager.clone(),
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            session_config,
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create SessionManager for RAG: {e}"),
            source: None,
        })?,
    );

    Ok(session_manager)
}

/// Get or create `HookRegistry`
fn get_or_create_hook_registry(context: &GlobalContext) -> Arc<HookRegistry> {
    if let Some(registry) = context.get_bridge::<HookRegistry>("hook_registry") {
        return registry;
    }

    // Create new registry
    Arc::new(HookRegistry::new())
}

/// Get or create `HookExecutor`
fn get_or_create_hook_executor(context: &GlobalContext) -> Arc<HookExecutor> {
    if let Some(executor) = context.get_bridge::<HookExecutor>("hook_executor") {
        return executor;
    }

    // Create new executor
    Arc::new(HookExecutor::new())
}

/// Get or create `EventBus`
fn get_or_create_event_bus(context: &GlobalContext) -> Arc<EventBus> {
    if let Some(event_bus) = context.get_bridge::<EventBus>("event_bus") {
        return event_bus;
    }

    // Create new EventBus
    Arc::new(EventBus::new())
}

/// Create vector storage based on RAG configuration
///
/// Returns both the trait object and concrete SqliteVectorStorage for save operations
///
/// # Errors
///
/// Returns an error if SQLite backend or vector storage initialization fails
async fn create_vector_storage(
    config: &RAGConfig,
    _context: &GlobalContext,
) -> Result<(Arc<dyn VectorStorage>, Arc<SqliteVectorStorage>)> {
    debug!("Creating SQLite vector storage for RAG");

    let dimensions = config.vector_storage.dimensions;

    // Get database path from config or use default
    let db_path = config
        .vector_storage
        .persistence_path
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("./data/rag_vectors.db"));

    debug!("Creating SQLite vector storage at: {:?}", db_path);

    // Create SQLite backend
    let config = SqliteConfig::new(db_path);
    let backend = SqliteBackend::new(config)
        .await
        .map_err(|e| LLMSpellError::Storage {
            message: format!("Failed to create SQLite backend for RAG: {e}"),
            operation: Some("create_backend".to_string()),
            source: None,
        })?;

    // Create vector storage with SQLite backend
    let storage = SqliteVectorStorage::new(Arc::new(backend), dimensions)
        .await
        .map_err(|e| LLMSpellError::Storage {
            message: format!("Failed to create SQLite vector storage for RAG: {e}"),
            operation: Some("create_vector_storage".to_string()),
            source: None,
        })?;

    let storage_arc = Arc::new(storage);

    // Return both trait object and concrete type
    Ok((storage_arc.clone(), storage_arc))
}

/// Create storage backend for sessions
///
/// # Errors
///
/// Returns an error if:
/// - Unknown backend type is specified
/// - Backend creation fails
fn create_storage_backend(backend_type: &str) -> Result<Arc<dyn StorageBackend>> {
    match backend_type {
        "memory" => {
            debug!("Creating in-memory storage backend for RAG sessions");
            Ok(Arc::new(MemoryBackend::new()))
        }
        backend => {
            warn!(
                "Backend type '{}' not supported via Lua bridge for RAG (only 'memory'), falling back to memory. \
                 For persistent storage, use Rust API with SQLite or PostgreSQL.",
                backend
            );
            Ok(Arc::new(MemoryBackend::new()))
        }
    }
}

/// Container for RAG infrastructure components
pub struct RAGInfrastructure {
    pub multi_tenant_rag: Arc<MultiTenantRAG>,
    pub vector_storage: Option<Arc<dyn VectorStorage>>,
    /// Keep a reference to the concrete SQLite storage for save/load operations
    pub sqlite_storage: Option<Arc<SqliteVectorStorage>>,
}

impl RAGInfrastructure {
    /// Save vector storage to disk if persistence is configured
    ///
    /// Note: SQLite storage auto-persists via transactions, so this is a no-op.
    /// Kept for API compatibility with existing code.
    ///
    /// # Errors
    ///
    /// Returns an error if the save operation fails
    pub async fn save(&self) -> Result<()> {
        if self.sqlite_storage.is_some() {
            debug!(
                "SQLite vector storage auto-persists via transactions - no explicit save needed"
            );
        } else {
            debug!("No vector storage to save");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentRegistry, ProviderManager};
    use llmspell_config::providers::ProviderManagerConfig;

    #[tokio::test]
    async fn test_rag_infrastructure_creation() {
        let context = GlobalContext::new(
            Arc::new(ComponentRegistry::new()),
            Arc::new(
                ProviderManager::new(ProviderManagerConfig::default())
                    .await
                    .unwrap(),
            ),
        );

        let mut config = RAGConfig::builder()
            .enabled(true)
            .dimensions(384)
            .backend(llmspell_config::VectorBackend::HNSW)
            .build();

        // Use in-memory database for testing
        config.vector_storage.persistence_path = Some(std::path::PathBuf::from(":memory:"));

        let infrastructure = get_or_create_rag_infrastructure(&context, &config)
            .await
            .unwrap();

        // Should reuse on second call
        let infrastructure2 = get_or_create_rag_infrastructure(&context, &config)
            .await
            .unwrap();
        assert!(Arc::ptr_eq(
            &infrastructure.multi_tenant_rag,
            &infrastructure2.multi_tenant_rag
        ));
    }

    #[tokio::test]
    async fn test_rag_hnsw_storage_creation() {
        let context = GlobalContext::new(
            Arc::new(ComponentRegistry::new()),
            Arc::new(
                ProviderManager::new(ProviderManagerConfig::default())
                    .await
                    .unwrap(),
            ),
        );

        let mut config = RAGConfig::builder()
            .enabled(true)
            .dimensions(768)
            .backend(llmspell_config::VectorBackend::HNSW)
            .build();

        // Use in-memory database for testing
        config.vector_storage.persistence_path = Some(std::path::PathBuf::from(":memory:"));

        let infrastructure = get_or_create_rag_infrastructure(&context, &config)
            .await
            .unwrap();

        // Verify infrastructure was created
        assert!(Arc::strong_count(&infrastructure.multi_tenant_rag) >= 1);
    }
}
