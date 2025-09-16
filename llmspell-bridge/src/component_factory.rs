//! ABOUTME: Config-driven component factory for language-agnostic GlobalContext creation
//! ABOUTME: Creates components based on LLMSpellConfig, serving ALL engines (Lua, JavaScript, Python)

use crate::globals::GlobalContext;
use crate::{ComponentRegistry, ProviderManager};
use llmspell_config::LLMSpellConfig;
use llmspell_core::error::LLMSpellError;
use std::sync::Arc;
use tracing::{debug, info};

/// Language-agnostic component factory
pub struct ComponentFactory;

impl ComponentFactory {
    /// Create language-agnostic GlobalContext based on LLMSpellConfig
    /// Used by Lua, JavaScript, Python - all get same components
    pub async fn from_config(config: &LLMSpellConfig) -> Result<Arc<GlobalContext>, LLMSpellError> {
        info!("Creating language-agnostic GlobalContext from config");

        // Create component registry with event support based on config
        let registry = if config.events.enabled {
            let event_bus = Arc::new(llmspell_events::EventBus::new());
            let event_config = llmspell_core::traits::event::EventConfig {
                enabled: config.events.enabled,
                include_types: config.events.filtering.include_types.clone(),
                exclude_types: config.events.filtering.exclude_types.clone(),
                emit_timing_events: config.events.emit_timing_events,
                emit_state_events: config.events.emit_state_events,
                emit_debug_events: config.events.emit_debug_events,
                max_events_per_second: config.events.max_events_per_second,
            };
            Arc::new(ComponentRegistry::with_event_bus(event_bus, event_config))
        } else {
            Arc::new(ComponentRegistry::new())
        };

        // Register all tools with the registry
        crate::tools::register_all_tools(&registry, &config.tools).map_err(|e| LLMSpellError::Component {
            message: format!("Failed to register tools: {e}"),
            source: None,
        })?;

        // Create provider manager
        let providers = Arc::new(ProviderManager::new(config.providers.clone()).await?);

        // Create GlobalContext - same for ALL languages
        let global_context = GlobalContext::new(registry, providers);

        // NOTE: SessionManager creation temporarily disabled - complex dependencies
        // TODO: In Step 5, properly wire SessionManager through kernel API layer
        if config.runtime.sessions.enabled {
            debug!("Sessions enabled in config - will be wired by kernel API layer");
            info!("SessionManager integration deferred to kernel API layer for proper dependency injection");
        } else {
            debug!("Sessions disabled in config");
        }

        // NOTE: StateManager creation temporarily disabled - complex setup
        // TODO: In Step 5, properly wire StateManager through kernel API layer
        if config.runtime.state_persistence.enabled {
            debug!("State persistence enabled in config - will be wired by kernel API layer");
            info!("StateManager integration deferred to kernel API layer for proper dependency injection");
        } else {
            debug!("State persistence disabled in config");
        }

        // NOTE: RAG infrastructure creation temporarily disabled - complex setup
        // TODO: In Step 5, properly wire RAG through kernel API layer
        if config.rag.enabled {
            debug!("RAG enabled in config - will be wired by kernel API layer");
            info!("RAG infrastructure integration deferred to kernel API layer for proper dependency injection");
        } else {
            debug!("RAG disabled in config");
        }

        info!("Basic language-agnostic GlobalContext created successfully");
        info!("JavaScript, Lua, Python all get SAME component registry and providers");
        // JavaScript, Lua, Python all get SAME components automatically
        Ok(Arc::new(global_context))
    }
}