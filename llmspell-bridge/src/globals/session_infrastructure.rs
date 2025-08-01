// ABOUTME: Shared session infrastructure initialization for SessionManager creation
// ABOUTME: Provides get_or_create pattern for session management following state infrastructure pattern

use crate::globals::GlobalContext;
use crate::runtime::SessionConfig;
use llmspell_core::{error::LLMSpellError, Result};
use llmspell_events::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_sessions::{SessionManager, SessionManagerConfig};
use llmspell_state_persistence::StateManager;
use llmspell_storage::{MemoryBackend, SledBackend, StorageBackend};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Helper function to get or create session infrastructure from GlobalContext
pub async fn get_or_create_session_infrastructure(
    context: &GlobalContext,
    config: &SessionConfig,
) -> Result<SessionInfrastructure> {
    // Check if already initialized
    if let Some(session_manager) = context.get_bridge::<SessionManager>("session_manager") {
        debug!("Using existing session infrastructure from GlobalContext");
        return Ok(SessionInfrastructure { session_manager });
    }

    // Initialize new infrastructure
    info!("Initializing session management infrastructure");

    // Get or create required dependencies
    let state_manager = get_or_create_state_manager(context).await?;
    let hook_registry = get_or_create_hook_registry(context)?;
    let hook_executor = get_or_create_hook_executor(context)?;
    let event_bus = get_or_create_event_bus(context).await?;

    // Create storage backend based on configuration
    let storage_backend = create_storage_backend(&config.storage_backend).await?;

    // Create SessionManagerConfig from runtime config
    let session_config = SessionManagerConfig {
        max_active_sessions: config.max_sessions,
        default_session_timeout: chrono::Duration::seconds(config.session_timeout_seconds as i64),
        storage_path: std::path::PathBuf::from("./sessions"),
        auto_persist: true,
        persist_interval_secs: 300,
        track_activity: true,
        max_storage_size_bytes: 10 * 1024 * 1024 * 1024, // 10GB
        enable_compression: config.artifact_compression_threshold > 0,
        compression_level: 3,
        enable_deduplication: true,
        cleanup_config: Default::default(),
        hook_config: Default::default(),
        event_config: Default::default(),
    };

    // Create SessionManager
    let session_manager = Arc::new(
        SessionManager::new(
            state_manager,
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            session_config,
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create SessionManager: {}", e),
            source: None,
        })?,
    );

    // Store SessionManager in context
    context.set_bridge("session_manager", session_manager.clone());

    Ok(SessionInfrastructure { session_manager })
}

/// Get or create StateManager
async fn get_or_create_state_manager(context: &GlobalContext) -> Result<Arc<StateManager>> {
    if let Some(state_manager) = context.get_bridge::<StateManager>("state_manager") {
        return Ok(state_manager);
    }

    // Create basic in-memory StateManager if not available
    warn!("StateManager not found, creating in-memory instance for SessionManager");
    let state_manager =
        Arc::new(
            StateManager::new()
                .await
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to create StateManager: {}", e),
                    source: None,
                })?,
        );

    context.set_bridge("state_manager", state_manager.clone());
    Ok(state_manager)
}

/// Get or create HookRegistry
fn get_or_create_hook_registry(context: &GlobalContext) -> Result<Arc<HookRegistry>> {
    if let Some(registry) = context.get_bridge::<HookRegistry>("hook_registry") {
        return Ok(registry);
    }

    // Create new registry
    let registry = Arc::new(HookRegistry::new());
    context.set_bridge("hook_registry", registry.clone());
    Ok(registry)
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

/// Create storage backend based on configuration
async fn create_storage_backend(backend_type: &str) -> Result<Arc<dyn StorageBackend>> {
    match backend_type {
        "memory" => {
            debug!("Creating in-memory storage backend for sessions");
            Ok(Arc::new(MemoryBackend::new()))
        }
        "sled" => {
            debug!("Creating sled storage backend for sessions");
            let _path = std::env::var("LLMSPELL_SESSION_PATH")
                .unwrap_or_else(|_| "./llmspell_sessions".to_string());
            let backend = SledBackend::new().map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create sled backend: {}", e),
                source: None,
            })?;
            Ok(Arc::new(backend))
        }
        backend => {
            warn!("Unknown backend type '{}', falling back to memory", backend);
            Ok(Arc::new(MemoryBackend::new()))
        }
    }
}

/// Container for session infrastructure components
pub struct SessionInfrastructure {
    pub session_manager: Arc<SessionManager>,
}

#[cfg(test)]
#[cfg_attr(test_category = "bridge")]
mod tests {
    use super::*;
    use crate::{ComponentRegistry, ProviderManager};

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_session_infrastructure_creation() {
        let context = GlobalContext::new(
            Arc::new(ComponentRegistry::new()),
            Arc::new(ProviderManager::new(Default::default()).await.unwrap()),
        );

        let config = SessionConfig {
            enabled: true,
            max_sessions: 100,
            max_artifacts_per_session: 1000,
            artifact_compression_threshold: 10240,
            session_timeout_seconds: 3600,
            storage_backend: "memory".to_string(),
        };

        let infrastructure = get_or_create_session_infrastructure(&context, &config)
            .await
            .unwrap();

        // Should reuse on second call
        let infrastructure2 = get_or_create_session_infrastructure(&context, &config)
            .await
            .unwrap();
        assert!(Arc::ptr_eq(
            &infrastructure.session_manager,
            &infrastructure2.session_manager
        ));
    }
}
