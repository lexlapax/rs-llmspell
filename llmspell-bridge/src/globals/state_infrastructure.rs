// ABOUTME: Shared state infrastructure initialization for lazy creation of StateManager and migration components
// ABOUTME: Provides get_or_create pattern for state persistence infrastructure following EventBridge pattern

use crate::globals::GlobalContext;
use crate::runtime::StatePersistenceConfig;
use llmspell_core::{error::LLMSpellError, Result};
use llmspell_events::{EventBus, EventCorrelationTracker};
use llmspell_hooks::HookExecutor;
use llmspell_state_persistence::{
    backend_adapter::StateStorageAdapter,
    config::{PersistenceConfig, SledConfig, StorageBackendType},
    migration::MigrationEngine,
    schema::SchemaRegistry,
    StateManager,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Helper function to get or create state infrastructure from GlobalContext
pub async fn get_or_create_state_infrastructure(
    context: &GlobalContext,
    config: &StatePersistenceConfig,
) -> Result<StateInfrastructure> {
    // Check if already initialized
    if let Some(state_manager) = context.get_bridge::<StateManager>("state_manager") {
        debug!("Using existing state infrastructure from GlobalContext");

        let migration_engine = context.get_bridge::<MigrationEngine>("migration_engine");
        let schema_registry = context.get_bridge::<SchemaRegistry>("schema_registry");

        return Ok(StateInfrastructure {
            state_manager,
            migration_engine,
            schema_registry,
        });
    }

    // Initialize new infrastructure
    info!("Initializing state persistence infrastructure");

    // Create storage backend type based on config
    let backend_type = create_backend_type(config)?;

    // Create StateManager with default persistence config
    let persistence_config = PersistenceConfig {
        enabled: config.enabled,
        backend_type: backend_type.clone(),
        ..Default::default()
    };

    let state_manager = Arc::new(
        StateManager::with_backend(backend_type, persistence_config)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create StateManager: {}", e),
                source: None,
            })?,
    );

    // Store StateManager in context
    context.set_bridge("state_manager", state_manager.clone());

    // Initialize migration infrastructure if enabled
    let (migration_engine, schema_registry) = if config.migration_enabled {
        debug!("Initializing migration infrastructure");

        // Create schema registry that will be shared
        let schema_registry = SchemaRegistry::new();

        // Get state manager's storage adapter for migration engine
        // Note: StateManager already has a storage adapter, we need to share it
        // For now, create a new one with the same backend type
        let migration_backend_type = create_backend_type(config)?;
        let migration_backend =
            llmspell_state_persistence::backend_adapter::create_storage_backend(
                &migration_backend_type,
            )
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create migration backend: {}", e),
                source: None,
            })?;

        let storage_adapter = Arc::new(StateStorageAdapter::new(
            migration_backend,
            "llmspell_migration".to_string(),
        ));

        // Get or create event infrastructure
        let event_bus = get_or_create_event_bus(context).await?;
        let correlation_tracker = get_or_create_correlation_tracker(context)?;
        let hook_executor = get_or_create_hook_executor(context)?;

        // Create migration engine
        let migration_engine = Arc::new(MigrationEngine::new(
            storage_adapter,
            schema_registry.clone(), // Pass the registry, MigrationEngine will wrap it in Arc<RwLock>
            hook_executor,
            correlation_tracker,
            event_bus,
        ));

        // Store in context
        let schema_registry_arc = Arc::new(schema_registry);
        context.set_bridge("migration_engine", migration_engine.clone());
        context.set_bridge("schema_registry", schema_registry_arc.clone());

        (Some(migration_engine), Some(schema_registry_arc))
    } else {
        debug!("Migration support disabled in configuration");
        (None, None)
    };

    Ok(StateInfrastructure {
        state_manager,
        migration_engine,
        schema_registry,
    })
}

/// Create storage backend type based on configuration
fn create_backend_type(config: &StatePersistenceConfig) -> Result<StorageBackendType> {
    match config.backend_type.as_str() {
        "memory" => {
            debug!("Creating in-memory storage backend type");
            Ok(StorageBackendType::Memory)
        }
        "sled" => {
            debug!("Creating sled storage backend type");
            let path = std::env::var("LLMSPELL_STATE_PATH")
                .unwrap_or_else(|_| "./llmspell_state".to_string());
            Ok(StorageBackendType::Sled(SledConfig {
                path: std::path::PathBuf::from(path),
                cache_capacity: 64 * 1024 * 1024, // 64MB
                use_compression: true,
            }))
        }
        backend => {
            warn!("Unknown backend type '{}', falling back to memory", backend);
            Ok(StorageBackendType::Memory)
        }
    }
}

/// Get or create EventBus
async fn get_or_create_event_bus(context: &GlobalContext) -> Result<Arc<EventBus>> {
    if let Some(event_bus) = context.get_bridge::<EventBus>("event_bus") {
        return Ok(event_bus);
    }

    // Create new EventBus
    let event_bus = Arc::new(EventBus::new());
    context.set_bridge("event_bus", event_bus.clone());
    Ok(event_bus)
}

/// Get or create EventCorrelationTracker
fn get_or_create_correlation_tracker(
    context: &GlobalContext,
) -> Result<Arc<EventCorrelationTracker>> {
    if let Some(tracker) = context.get_bridge::<EventCorrelationTracker>("correlation_tracker") {
        return Ok(tracker);
    }

    // Create new tracker
    let tracker = Arc::new(EventCorrelationTracker::default());
    context.set_bridge("correlation_tracker", tracker.clone());
    Ok(tracker)
}

/// Get or create HookExecutor
fn get_or_create_hook_executor(context: &GlobalContext) -> Result<Arc<HookExecutor>> {
    if let Some(executor) = context.get_bridge::<HookExecutor>("hook_executor") {
        return Ok(executor);
    }

    // Create new executor
    let executor = Arc::new(HookExecutor::new());
    context.set_bridge("hook_executor", executor.clone());
    Ok(executor)
}

/// Container for state infrastructure components
pub struct StateInfrastructure {
    pub state_manager: Arc<StateManager>,
    pub migration_engine: Option<Arc<MigrationEngine>>,
    pub schema_registry: Option<Arc<SchemaRegistry>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentRegistry, ProviderManager};

    #[tokio::test]
    async fn test_state_infrastructure_creation() {
        let context = GlobalContext::new(
            Arc::new(ComponentRegistry::new()),
            Arc::new(ProviderManager::new(Default::default()).await.unwrap()),
        );

        let mut config = StatePersistenceConfig::default();
        config.enabled = true;

        let infrastructure = get_or_create_state_infrastructure(&context, &config)
            .await
            .unwrap();

        assert!(infrastructure.migration_engine.is_none());
        assert!(infrastructure.schema_registry.is_none());

        // Should reuse on second call
        let infrastructure2 = get_or_create_state_infrastructure(&context, &config)
            .await
            .unwrap();
        assert!(Arc::ptr_eq(
            &infrastructure.state_manager,
            &infrastructure2.state_manager
        ));
    }

    #[tokio::test]
    async fn test_migration_infrastructure_creation() {
        let context = GlobalContext::new(
            Arc::new(ComponentRegistry::new()),
            Arc::new(ProviderManager::new(Default::default()).await.unwrap()),
        );

        let mut config = StatePersistenceConfig::default();
        config.enabled = true;
        config.migration_enabled = true;

        let infrastructure = get_or_create_state_infrastructure(&context, &config)
            .await
            .unwrap();

        assert!(infrastructure.migration_engine.is_some());
        assert!(infrastructure.schema_registry.is_some());
    }
}
