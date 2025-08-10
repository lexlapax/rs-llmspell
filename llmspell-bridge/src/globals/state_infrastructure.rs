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

/// Helper function to get or create state infrastructure from `GlobalContext`
#[allow(clippy::too_many_lines)]
pub async fn get_or_create_state_infrastructure(
    context: &GlobalContext,
    config: &StatePersistenceConfig,
) -> Result<StateInfrastructure> {
    // Check if already initialized
    if let Some(state_manager) = context.get_bridge::<StateManager>("state_manager") {
        debug!("Using existing state infrastructure from GlobalContext");

        let migration_engine = context.get_bridge::<MigrationEngine>("migration_engine");
        let schema_registry = context.get_bridge::<SchemaRegistry>("schema_registry");

        let backup_manager = context
            .get_bridge::<llmspell_state_persistence::backup::BackupManager>("backup_manager");

        return Ok(StateInfrastructure {
            state_manager,
            migration_engine,
            schema_registry,
            backup_manager,
        });
    }

    // Initialize new infrastructure
    info!("Initializing state persistence infrastructure");

    // Create storage backend type based on config
    let backend_type = create_backend_type(config);

    // Create StateManager with default persistence config
    let persistence_config = PersistenceConfig {
        enabled: config.enabled,
        backend_type: backend_type.clone(),
        ..Default::default()
    };

    let state_manager = Arc::new(
        StateManager::with_backend(backend_type.clone(), persistence_config.clone())
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create StateManager: {e}"),
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
        let migration_backend_type = create_backend_type(config);
        let migration_backend =
            llmspell_state_persistence::backend_adapter::create_storage_backend(
                &migration_backend_type,
            )
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create migration backend: {e}"),
                source: None,
            })?;

        let storage_adapter = Arc::new(StateStorageAdapter::new(
            migration_backend,
            "llmspell_migration".to_string(),
        ));

        // Get or create event infrastructure
        let event_bus = get_or_create_event_bus(context);
        let correlation_tracker = get_or_create_correlation_tracker(context);
        let hook_executor = get_or_create_hook_executor(context);

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

    // Initialize backup infrastructure if enabled
    let backup_manager = if config.backup_enabled {
        debug!("Initializing backup infrastructure");

        // Get backup config or use defaults
        #[allow(clippy::option_if_let_else)] // Complex pattern
        let backup_config = if let Some(ref backup_cfg) = config.backup {
            // Convert from runtime BackupConfig to state-persistence BackupConfig
            llmspell_state_persistence::config::BackupConfig {
                backup_dir: std::path::PathBuf::from(
                    backup_cfg.backup_dir.as_deref().unwrap_or("./backups"),
                ),
                compression_enabled: backup_cfg.compression_enabled,
                compression_type: match backup_cfg.compression_type.as_str() {
                    "gzip" => llmspell_state_persistence::config::CompressionType::Gzip,
                    "lz4" => llmspell_state_persistence::config::CompressionType::Lz4,
                    "brotli" => llmspell_state_persistence::config::CompressionType::Brotli,
                    _ => llmspell_state_persistence::config::CompressionType::Zstd,
                },
                compression_level: backup_cfg.compression_level,
                encryption_enabled: false, // Not exposed in runtime config yet
                max_backups: backup_cfg.max_backups,
                max_backup_age: backup_cfg
                    .max_backup_age
                    .map(std::time::Duration::from_secs),
                incremental_enabled: backup_cfg.incremental_enabled,
                full_backup_interval: std::time::Duration::from_secs(86400), // 24 hours default
            }
        } else {
            llmspell_state_persistence::config::BackupConfig::default()
        };

        // Create backup manager using the same StateManager instance
        match llmspell_state_persistence::backup::BackupManager::new(
            backup_config,
            state_manager.clone(),
        ) {
            Ok(mgr) => {
                let backup_mgr = Arc::new(mgr);
                context.set_bridge("backup_manager", backup_mgr.clone());
                Some(backup_mgr)
            }
            Err(e) => {
                warn!(
                    "Failed to create backup manager: {}, backup functionality disabled",
                    e
                );
                None
            }
        }
    } else {
        debug!("Backup support disabled in configuration");
        None
    };

    Ok(StateInfrastructure {
        state_manager,
        migration_engine,
        schema_registry,
        backup_manager,
    })
}

/// Create storage backend type based on configuration
///
/// # Errors
///
/// Returns an error if:
/// - Unknown backend type is specified
/// - Backend configuration is invalid
fn create_backend_type(config: &StatePersistenceConfig) -> StorageBackendType {
    match config.backend_type.as_str() {
        "memory" => {
            debug!("Creating in-memory storage backend type");
            StorageBackendType::Memory
        }
        "sled" => {
            debug!("Creating sled storage backend type");
            let path = std::env::var("LLMSPELL_STATE_PATH")
                .unwrap_or_else(|_| "./llmspell_state".to_string());
            StorageBackendType::Sled(SledConfig {
                path: std::path::PathBuf::from(path),
                cache_capacity: 64 * 1024 * 1024, // 64MB
                use_compression: true,
            })
        }
        backend => {
            warn!("Unknown backend type '{}', falling back to memory", backend);
            StorageBackendType::Memory
        }
    }
}

/// Get or create `EventBus`
fn get_or_create_event_bus(context: &GlobalContext) -> Arc<EventBus> {
    if let Some(event_bus) = context.get_bridge::<EventBus>("event_bus") {
        return event_bus;
    }

    // Create new EventBus
    let event_bus = Arc::new(EventBus::new());
    context.set_bridge("event_bus", event_bus.clone());
    event_bus
}

/// Get or create `EventCorrelationTracker`
fn get_or_create_correlation_tracker(context: &GlobalContext) -> Arc<EventCorrelationTracker> {
    if let Some(tracker) = context.get_bridge::<EventCorrelationTracker>("correlation_tracker") {
        return tracker;
    }

    // Create new tracker
    let tracker = Arc::new(EventCorrelationTracker::default());
    context.set_bridge("correlation_tracker", tracker.clone());
    tracker
}

/// Get or create `HookExecutor`
fn get_or_create_hook_executor(context: &GlobalContext) -> Arc<HookExecutor> {
    if let Some(executor) = context.get_bridge::<HookExecutor>("hook_executor") {
        return executor;
    }

    // Create new executor
    let executor = Arc::new(HookExecutor::new());
    context.set_bridge("hook_executor", executor.clone());
    executor
}

/// Container for state infrastructure components
pub struct StateInfrastructure {
    pub state_manager: Arc<StateManager>,
    pub migration_engine: Option<Arc<MigrationEngine>>,
    pub schema_registry: Option<Arc<SchemaRegistry>>,
    pub backup_manager: Option<Arc<llmspell_state_persistence::backup::BackupManager>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentRegistry, ProviderManager, ProviderManagerConfig};
    #[tokio::test]
    async fn test_state_infrastructure_creation() {
        let context = GlobalContext::new(
            Arc::new(ComponentRegistry::new()),
            Arc::new(
                ProviderManager::new(ProviderManagerConfig::default())
                    .await
                    .unwrap(),
            ),
        );

        let config = StatePersistenceConfig {
            enabled: true,
            ..Default::default()
        };

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
            Arc::new(
                ProviderManager::new(ProviderManagerConfig::default())
                    .await
                    .unwrap(),
            ),
        );

        let config = StatePersistenceConfig {
            enabled: true,
            migration_enabled: true,
            ..Default::default()
        };

        let infrastructure = get_or_create_state_infrastructure(&context, &config)
            .await
            .unwrap();

        assert!(infrastructure.migration_engine.is_some());
        assert!(infrastructure.schema_registry.is_some());
    }
}
