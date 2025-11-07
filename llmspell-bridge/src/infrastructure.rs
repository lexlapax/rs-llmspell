//! ABOUTME: Central infrastructure creation module for self-contained `ScriptRuntime`
//! ABOUTME: Creates ALL infrastructure components from config (Phase 13b.16.1)
//!
//! This module implements the Phase 9/10 architecture principle: "`IntegratedKernel` as
//! self-contained component". It centralizes creation of all infrastructure components
//! that `ScriptRuntime` needs, ensuring:
//!
//! 1. Single infrastructure creation path for all modes (embedded + daemon)
//! 2. CLI remains thin (no infrastructure creation in llmspell-cli)
//! 3. Future services (web server daemon) can use `ScriptRuntime` directly
//! 4. Config-driven creation (RAG, `MemoryManager` optional based on config)
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────┐
//! │  Infrastructure::from_config(config)   │
//! │  Creates ALL 9 components:             │
//! │  ├─ ProviderManager                    │
//! │  ├─ StateManager                       │
//! │  ├─ SessionManager                     │
//! │  ├─ RAG (if enabled)                   │
//! │  ├─ MemoryManager (if enabled)         │
//! │  ├─ ToolRegistry                       │
//! │  ├─ AgentRegistry                      │
//! │  ├─ WorkflowFactory                    │
//! │  └─ ComponentRegistry                  │
//! └────────────────────────────────────────┘
//! ```
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_bridge::infrastructure::Infrastructure;
//! use llmspell_config::LLMSpellConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = LLMSpellConfig::default();
//! let infrastructure = Infrastructure::from_config(&config).await?;
//!
//! // All components available:
//! let provider_manager = infrastructure.provider_manager.clone();
//! let session_manager = infrastructure.session_manager.clone();
//! let rag = infrastructure.rag.clone(); // Option<Arc<...>>
//! let memory = infrastructure.memory_manager.clone(); // Option<Arc<...>>
//! # Ok(())
//! # }
//! ```

use crate::{providers::ProviderManager, registry::ComponentRegistry};
use llmspell_config::LLMSpellConfig;
use llmspell_core::error::LLMSpellError;
use llmspell_storage::backends::vector::HNSWVectorStorage;
use std::sync::Arc;
use tracing::{debug, info};

/// Central infrastructure container with all `ScriptRuntime` dependencies
///
/// Contains all 9 core infrastructure components that `ScriptRuntime` needs.
/// Created via `Infrastructure::from_config()` which handles conditional
/// creation based on configuration (e.g., RAG only if enabled).
pub struct Infrastructure {
    /// Provider manager for LLM access
    pub provider_manager: Arc<ProviderManager>,

    /// State manager for persistent state
    pub state_manager: Arc<llmspell_kernel::state::StateManager>,

    /// Session manager for session and artifact management
    pub session_manager: Arc<llmspell_kernel::sessions::SessionManager>,

    /// RAG system (optional, created if config.rag.enabled)
    pub rag: Option<Arc<llmspell_rag::multi_tenant_integration::MultiTenantRAG>>,

    /// Memory manager (optional, created if config.memory.enabled)
    pub memory_manager: Option<Arc<llmspell_memory::DefaultMemoryManager>>,

    /// Tool registry for tool management
    pub tool_registry: Arc<llmspell_tools::ToolRegistry>,

    /// Agent factory registry
    pub agent_registry: Arc<llmspell_agents::FactoryRegistry>,

    /// Workflow factory
    pub workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory>,

    /// Component registry (lightweight script access layer)
    pub component_registry: Arc<ComponentRegistry>,
}

impl Infrastructure {
    /// Create complete infrastructure from configuration
    ///
    /// This is THE entry point for infrastructure creation. It creates all 9 components
    /// based on `LLMSpellConfig`, conditionally creating optional components (RAG, Memory).
    ///
    /// # Architecture Principle (Phase 9/10)
    ///
    /// This function embodies the "self-contained kernel" principle:
    /// - CLI layer NEVER creates infrastructure
    /// - `ScriptRuntime` calls this to get everything it needs
    /// - Daemon mode and embedded mode use SAME path
    ///
    /// # Errors
    ///
    /// Returns an error if any component fails to initialize
    #[allow(clippy::cognitive_complexity)]
    pub async fn from_config(config: &LLMSpellConfig) -> Result<Self, LLMSpellError> {
        info!("Creating infrastructure from config");

        // 1. Create provider manager
        let provider_manager = create_provider_manager(config).await?;

        // 2. Create state manager
        let state_manager = create_state_manager(config).await?;

        // 3. Create session manager (depends on state_manager)
        let session_manager = create_session_manager(state_manager.clone(), config)?;

        // 4. Create RAG if enabled
        let rag = if config.rag.enabled {
            Some(create_rag(config))
        } else {
            debug!("RAG disabled in config, skipping creation");
            None
        };

        // 5. Create memory manager if enabled
        let memory_manager = if config.runtime.memory.enabled {
            Some(create_memory_manager(config).await?)
        } else {
            debug!("Memory disabled in config, skipping creation");
            None
        };

        // 6. Create tool registry
        let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());

        // 7. Create agent registry
        let agent_registry = Arc::new(llmspell_agents::FactoryRegistry::new());

        // 8. Create workflow factory
        let workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory> =
            Arc::new(llmspell_workflows::factory::DefaultWorkflowFactory::new());

        // 9. Create component registry (with EventBus if enabled)
        let component_registry = create_component_registry(config)?;

        info!("Infrastructure created successfully");

        Ok(Self {
            provider_manager,
            state_manager,
            session_manager,
            rag,
            memory_manager,
            tool_registry,
            agent_registry,
            workflow_factory,
            component_registry,
        })
    }
}

/// Create provider manager from config
///
/// # Errors
///
/// Returns an error if provider initialization fails
async fn create_provider_manager(
    config: &LLMSpellConfig,
) -> Result<Arc<ProviderManager>, LLMSpellError> {
    debug!("Creating provider manager");
    let manager = ProviderManager::new(config.providers.clone()).await?;
    Ok(Arc::new(manager))
}

/// Create state manager from config
///
/// # Errors
///
/// Returns an error if state manager initialization fails
async fn create_state_manager(
    _config: &LLMSpellConfig,
) -> Result<Arc<llmspell_kernel::state::StateManager>, LLMSpellError> {
    debug!("Creating state manager");
    let manager = llmspell_kernel::state::StateManager::new(None)
        .await
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create state manager: {e}"),
            source: None,
        })?;
    Ok(Arc::new(manager))
}

/// Create session manager from state manager and config
///
/// # Errors
///
/// Returns an error if session manager initialization fails
fn create_session_manager(
    state_manager: Arc<llmspell_kernel::state::StateManager>,
    config: &LLMSpellConfig,
) -> Result<Arc<llmspell_kernel::sessions::SessionManager>, LLMSpellError> {
    debug!("Creating session manager");

    // Create session storage backend based on config (Phase 13b.16.9 - Fix lock contention)
    let session_storage_backend: Arc<dyn llmspell_storage::StorageBackend> =
        match config.runtime.sessions.storage_backend.as_str() {
            "memory" => {
                debug!("Using memory backend for session storage");
                Arc::new(llmspell_storage::MemoryBackend::new())
            }
            "sled" | _ => {
                debug!("Using Sled backend for session storage at ./sessions");
                Arc::new(
                    llmspell_storage::SledBackend::new_with_path("./sessions").map_err(|e| {
                        LLMSpellError::Component {
                            message: format!("Failed to create session storage backend: {e}"),
                            source: None,
                        }
                    })?,
                )
            }
        };

    // Create hook infrastructure
    let hook_registry = Arc::new(llmspell_hooks::HookRegistry::new());
    let hook_executor = Arc::new(llmspell_hooks::HookExecutor::new());

    // Create event bus
    let event_bus = Arc::new(llmspell_events::bus::EventBus::new());

    // Use default session config
    let session_config = llmspell_kernel::sessions::SessionManagerConfig::default();

    // Create session manager
    let manager = llmspell_kernel::sessions::SessionManager::new(
        state_manager,
        session_storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        session_config,
    )
    .map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create session manager: {e}"),
        source: None,
    })?;

    Ok(Arc::new(manager))
}

/// Create RAG system from config
///
/// Only called when `config.rag.enabled` is true.
fn create_rag(
    config: &LLMSpellConfig,
) -> Arc<llmspell_rag::multi_tenant_integration::MultiTenantRAG> {
    debug!("Creating RAG infrastructure (enabled via config)");

    // Read configuration from config.rag
    let dimensions = config.rag.vector_storage.dimensions;

    // Convert llmspell_config::rag::HNSWConfig to llmspell_storage::HNSWConfig
    let storage_hnsw_config = llmspell_storage::HNSWConfig {
        m: config.rag.vector_storage.hnsw.m,
        ef_construction: config.rag.vector_storage.hnsw.ef_construction,
        ef_search: config.rag.vector_storage.hnsw.ef_search,
        max_elements: config.rag.vector_storage.hnsw.max_elements,
        seed: config.rag.vector_storage.hnsw.seed,
        metric: convert_distance_metric(&config.rag.vector_storage.hnsw.metric),
        allow_replace_deleted: config.rag.vector_storage.hnsw.allow_replace_deleted,
        num_threads: config.rag.vector_storage.hnsw.num_threads,
        nb_layers: config.rag.vector_storage.hnsw.nb_layers,
        parallel_batch_size: config.rag.vector_storage.hnsw.parallel_batch_size,
        enable_mmap: config.rag.vector_storage.hnsw.enable_mmap,
        mmap_sync_interval: config.rag.vector_storage.hnsw.mmap_sync_interval,
    };

    // Create vector storage
    let vector_storage = Arc::new(HNSWVectorStorage::new(dimensions, storage_hnsw_config));

    // Create tenant manager
    let tenant_manager = Arc::new(llmspell_tenancy::MultiTenantVectorManager::new(
        vector_storage,
    ));

    // Create multi-tenant RAG
    let rag = Arc::new(llmspell_rag::multi_tenant_integration::MultiTenantRAG::new(
        tenant_manager,
    ));

    debug!("RAG infrastructure created (dimensions={})", dimensions);

    rag
}

/// Create memory manager from config
///
/// Only called when `config.runtime.memory.enabled` is true.
///
/// # Errors
///
/// Returns an error if memory manager initialization fails
async fn create_memory_manager(
    _config: &LLMSpellConfig,
) -> Result<Arc<llmspell_memory::DefaultMemoryManager>, LLMSpellError> {
    debug!("Creating memory manager (enabled via config)");

    // Use in-memory implementation (testing/development mode)
    // For production with HNSW and real embeddings, use DefaultMemoryManager::new_in_memory_with_embeddings()
    let manager = llmspell_memory::DefaultMemoryManager::new_in_memory()
        .await
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create memory manager: {e}"),
            source: None,
        })?;

    debug!("Memory manager created successfully");

    Ok(Arc::new(manager))
}

/// Create component registry with optional `EventBus`
///
/// # Errors
///
/// Returns an error if component registry initialization fails
fn create_component_registry(
    config: &LLMSpellConfig,
) -> Result<Arc<ComponentRegistry>, LLMSpellError> {
    debug!("Creating component registry");

    let registry = if config.events.enabled {
        // Create EventBus
        let event_bus = Arc::new(llmspell_events::EventBus::new());

        // Convert config to EventConfig
        let event_config = llmspell_core::traits::event::EventConfig {
            enabled: config.events.enabled,
            include_types: config.events.filtering.include_types.clone(),
            exclude_types: config.events.filtering.exclude_types.clone(),
            emit_timing_events: config.events.emit_timing_events,
            emit_state_events: config.events.emit_state_events,
            emit_debug_events: config.events.emit_debug_events,
            max_events_per_second: config.events.max_events_per_second,
        };

        ComponentRegistry::with_event_bus_and_templates(event_bus, event_config).map_err(|e| {
            LLMSpellError::Component {
                message: format!("Failed to initialize component registry with events: {e}"),
                source: None,
            }
        })?
    } else {
        ComponentRegistry::with_templates().map_err(|e| LLMSpellError::Component {
            message: format!("Failed to initialize component registry: {e}"),
            source: None,
        })?
    };

    Ok(Arc::new(registry))
}

/// Convert config `DistanceMetric` to storage `DistanceMetric`
const fn convert_distance_metric(
    metric: &llmspell_config::rag::DistanceMetric,
) -> llmspell_storage::DistanceMetric {
    match metric {
        llmspell_config::rag::DistanceMetric::Cosine => llmspell_storage::DistanceMetric::Cosine,
        llmspell_config::rag::DistanceMetric::Euclidean => {
            llmspell_storage::DistanceMetric::Euclidean
        }
        llmspell_config::rag::DistanceMetric::InnerProduct => {
            llmspell_storage::DistanceMetric::InnerProduct
        }
    }
}
