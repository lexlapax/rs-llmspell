//! ABOUTME: Tests for global registry initialization and dependency ordering
//! ABOUTME: Verifies Session and Artifact globals are registered with correct dependencies

#[cfg(test)]
mod registry_tests {
    use llmspell_bridge::globals::{create_standard_registry, GlobalContext};
    use llmspell_bridge::{ComponentRegistry, ProviderManager};
    use llmspell_config::providers::ProviderManagerConfig;
    use llmspell_events::bus::EventBus;
    use llmspell_hooks::{HookExecutor, HookRegistry};
    use llmspell_kernel::sessions::{SessionManager, SessionManagerConfig};
    use llmspell_kernel::state::StateManager;
    use llmspell_storage::MemoryBackend;
    use std::sync::Arc;
    #[tokio::test]
    async fn test_session_artifact_registration_order() {
        // Create minimal infrastructure
        let registry = Arc::new(ComponentRegistry::new());
        let config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(config).await.unwrap());

        // Create infrastructure for session manager
        let storage_backend = Arc::new(MemoryBackend::new());
        let state_manager = Arc::new(StateManager::new(None).await.unwrap());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());

        let session_config = SessionManagerConfig::default();
        let session_manager = Arc::new(
            SessionManager::new(
                state_manager.clone(),
                storage_backend,
                hook_registry,
                hook_executor,
                &event_bus,
                session_config,
            )
            .unwrap(),
        );

        // Create infrastructure registries (Phase 12.8.2.13)
        let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());
        let agent_registry = Arc::new(llmspell_agents::FactoryRegistry::new());
        let workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory> =
            Arc::new(llmspell_workflows::factory::DefaultWorkflowFactory::new());

        // Create context with necessary bridges
        let context = GlobalContext::new(registry, providers);
        context.set_bridge("session_manager", session_manager);
        context.set_bridge("state_manager", state_manager);
        context.set_bridge("tool_registry", tool_registry);
        context.set_bridge("agent_registry", agent_registry);
        context.set_bridge("workflow_factory", Arc::new(workflow_factory));

        let context = Arc::new(context);

        // Create standard registry
        let registry = create_standard_registry(context).await.unwrap();

        // Verify all globals are present
        let globals = registry.list_globals();
        let global_names: Vec<&str> = globals.iter().map(|g| g.name.as_str()).collect();

        // Verify Session and Artifact are registered
        assert!(
            global_names.contains(&"Session"),
            "Session global should be registered"
        );
        assert!(
            global_names.contains(&"Artifact"),
            "Artifact global should be registered"
        );
        assert!(
            global_names.contains(&"State"),
            "State global should be registered"
        );

        // Find the globals to check dependencies
        let session_global = globals.iter().find(|g| g.name == "Session").unwrap();
        let artifact_global = globals.iter().find(|g| g.name == "Artifact").unwrap();

        // Verify dependencies
        assert_eq!(
            session_global.dependencies,
            vec!["State"],
            "Session should depend on State"
        );
        assert_eq!(
            artifact_global.dependencies,
            vec!["Session"],
            "Artifact should depend on Session"
        );

        // Verify registration order by checking indices
        let session_idx = global_names.iter().position(|&n| n == "Session").unwrap();
        let artifact_idx = global_names.iter().position(|&n| n == "Artifact").unwrap();
        let state_idx = global_names.iter().position(|&n| n == "State").unwrap();

        assert!(
            state_idx < session_idx,
            "State should be registered before Session"
        );
        assert!(
            session_idx < artifact_idx,
            "Session should be registered before Artifact"
        );
    }
    #[tokio::test]
    async fn test_registry_without_session_manager() {
        // Create infrastructure registries (Phase 12.8.2.13)
        let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());
        let agent_registry = Arc::new(llmspell_agents::FactoryRegistry::new());
        let workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory> =
            Arc::new(llmspell_workflows::factory::DefaultWorkflowFactory::new());

        // Create context without session manager
        let registry = Arc::new(ComponentRegistry::new());
        let config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(config).await.unwrap());

        let context = GlobalContext::new(registry, providers);
        context.set_bridge("tool_registry", tool_registry);
        context.set_bridge("agent_registry", agent_registry);
        context.set_bridge("workflow_factory", Arc::new(workflow_factory));
        let context = Arc::new(context);

        // Create standard registry
        let registry = create_standard_registry(context).await.unwrap();

        // Verify Session and Artifact are NOT registered without SessionManager
        let globals = registry.list_globals();
        let global_names: Vec<&str> = globals.iter().map(|g| g.name.as_str()).collect();

        assert!(
            !global_names.contains(&"Session"),
            "Session global should not be registered without SessionManager"
        );
        assert!(
            !global_names.contains(&"Artifact"),
            "Artifact global should not be registered without SessionManager"
        );
    }
}
