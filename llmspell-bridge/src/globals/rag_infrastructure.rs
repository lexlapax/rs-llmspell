//! ABOUTME: Shared RAG infrastructure initialization for multi-tenant vector storage
//! ABOUTME: Provides `get_or_create` pattern for RAG management following session infrastructure pattern

use crate::globals::GlobalContext;
use llmspell_config::RAGConfig;
use llmspell_core::{error::LLMSpellError, Result};
use llmspell_events::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_rag::multi_tenant_integration::MultiTenantRAG;
use llmspell_sessions::{SessionManager, SessionManagerConfig};
use llmspell_state_persistence::StateManager;
use llmspell_storage::{MemoryBackend, SledBackend, StorageBackend, VectorStorage};
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
            hnsw_storage: context
                .get_bridge::<llmspell_storage::backends::vector::hnsw::HNSWVectorStorage>(
                    "hnsw_storage",
                ),
        });
    }

    // Initialize new infrastructure
    info!("Initializing RAG infrastructure");

    // Get or create required dependencies
    let state_manager = get_or_create_state_manager(context).await?;
    let session_manager = get_or_create_session_manager(context, &state_manager)?;

    // Create vector storage based on configuration
    let vector_storage = create_vector_storage(config, context).await;

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
        hnsw_storage: context
            .get_bridge::<llmspell_storage::backends::vector::hnsw::HNSWVectorStorage>(
                "hnsw_storage",
            ),
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
            StateManager::new()
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

/// Convert config distance metric to storage distance metric
fn convert_distance_metric(
    metric: &llmspell_config::DistanceMetric,
) -> llmspell_storage::vector_storage::DistanceMetric {
    match metric {
        llmspell_config::DistanceMetric::Cosine => {
            llmspell_storage::vector_storage::DistanceMetric::Cosine
        }
        llmspell_config::DistanceMetric::Euclidean => {
            llmspell_storage::vector_storage::DistanceMetric::Euclidean
        }
        llmspell_config::DistanceMetric::InnerProduct => {
            llmspell_storage::vector_storage::DistanceMetric::InnerProduct
        }
    }
}

/// Convert config HNSW configuration to storage HNSW configuration
fn convert_hnsw_config(
    config: &llmspell_config::HNSWConfig,
) -> llmspell_storage::vector_storage::HNSWConfig {
    llmspell_storage::vector_storage::HNSWConfig {
        m: config.m,
        ef_construction: config.ef_construction,
        ef_search: config.ef_search,
        max_elements: config.max_elements,
        seed: config.seed,
        metric: convert_distance_metric(&config.metric),
        allow_replace_deleted: config.allow_replace_deleted,
        num_threads: config.num_threads,
    }
}

/// Create HNSW storage with real implementation
#[cfg(feature = "hnsw-real")]
async fn create_real_hnsw_storage(
    config: &RAGConfig,
    hnsw_config: llmspell_storage::vector_storage::HNSWConfig,
) -> Arc<dyn VectorStorage> {
    use llmspell_storage::backends::vector::hnsw_real::RealHNSWVectorStorage;

    debug!("Creating real HNSW vector storage for RAG");

    if let Some(ref path) = config.vector_storage.persistence_path {
        debug!(
            "Loading or creating real HNSW storage with persistence at: {:?}",
            path
        );

        let mut storage = RealHNSWVectorStorage::new(config.vector_storage.dimensions, hnsw_config)
            .with_persistence(path.clone());

        if let Err(e) = storage.load().await {
            warn!(
                "Failed to load existing real HNSW index: {}, starting fresh",
                e
            );
        }

        Arc::new(storage)
    } else {
        warn!("Real HNSW storage created without persistence - data will not survive restarts");
        Arc::new(RealHNSWVectorStorage::new(
            config.vector_storage.dimensions,
            hnsw_config,
        ))
    }
}

/// Create HNSW storage with mock implementation
#[cfg(not(feature = "hnsw-real"))]
async fn create_mock_hnsw_storage(
    config: &RAGConfig,
    hnsw_config: llmspell_storage::vector_storage::HNSWConfig,
    context: &GlobalContext,
) -> Arc<dyn VectorStorage> {
    if let Some(ref path) = config.vector_storage.persistence_path {
        debug!(
            "Loading or creating mock HNSW storage with persistence at: {:?}",
            path
        );

        let storage = llmspell_storage::backends::vector::hnsw::HNSWVectorStorage::load(
            path,
            config.vector_storage.dimensions,
            hnsw_config.clone(),
        )
        .await
        .unwrap_or_else(|e| {
            warn!(
                "Failed to load existing mock HNSW index: {}, creating new",
                e
            );
            llmspell_storage::backends::vector::hnsw::HNSWVectorStorage::new(
                config.vector_storage.dimensions,
                hnsw_config,
            )
            .with_persistence(path.clone())
        });

        let storage_arc = Arc::new(storage);
        context.set_bridge("hnsw_storage", storage_arc.clone());
        storage_arc
    } else {
        warn!("Mock HNSW storage created without persistence - data will not survive restarts");
        Arc::new(
            llmspell_storage::backends::vector::hnsw::HNSWVectorStorage::new(
                config.vector_storage.dimensions,
                hnsw_config,
            ),
        )
    }
}

/// Create vector storage based on RAG configuration
async fn create_vector_storage(
    config: &RAGConfig,
    _context: &GlobalContext,
) -> Arc<dyn VectorStorage> {
    match config.vector_storage.backend {
        llmspell_config::VectorBackend::HNSW => {
            debug!("Creating HNSW vector storage for RAG");

            let hnsw_config = convert_hnsw_config(&config.vector_storage.hnsw);

            #[cfg(feature = "hnsw-real")]
            {
                create_real_hnsw_storage(config, hnsw_config).await
            }

            #[cfg(not(feature = "hnsw-real"))]
            {
                create_mock_hnsw_storage(config, hnsw_config, _context).await
            }
        }
        llmspell_config::VectorBackend::Mock => {
            debug!("Creating mock vector storage for RAG (testing)");
            // Use the existing MockVectorStorage from rag_bridge
            let storage = crate::rag_bridge::MockVectorStorage::new();
            Arc::new(storage)
        }
    }
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
        "sled" => {
            debug!("Creating sled storage backend for RAG sessions");
            let backend = SledBackend::new().map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create sled backend for RAG: {e}"),
                source: None,
            })?;
            Ok(Arc::new(backend))
        }
        backend => {
            warn!(
                "Unknown backend type '{}' for RAG, falling back to memory",
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
    /// Keep a reference to the concrete HNSW storage for save/load operations
    pub hnsw_storage: Option<Arc<llmspell_storage::backends::vector::hnsw::HNSWVectorStorage>>,
}

impl RAGInfrastructure {
    /// Save vector storage to disk if persistence is configured
    ///
    /// # Errors
    ///
    /// Returns an error if the save operation fails
    pub async fn save(&self) -> Result<()> {
        if let Some(ref hnsw) = self.hnsw_storage {
            info!("Saving HNSW vector storage to disk");
            hnsw.save().await.map_err(|e| LLMSpellError::Component {
                message: format!("Failed to save HNSW storage: {e}"),
                source: None,
            })?;
        } else {
            debug!("No HNSW storage to save (using different backend or no persistence)");
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

        let config = RAGConfig::builder()
            .enabled(true)
            .dimensions(384)
            .backend(llmspell_config::VectorBackend::Mock)
            .build();

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

        let config = RAGConfig::builder()
            .enabled(true)
            .dimensions(768)
            .backend(llmspell_config::VectorBackend::HNSW)
            .build();

        let infrastructure = get_or_create_rag_infrastructure(&context, &config)
            .await
            .unwrap();

        // Verify infrastructure was created
        assert!(Arc::strong_count(&infrastructure.multi_tenant_rag) >= 1);
    }
}
