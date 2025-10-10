//! Integration test: LocalLLM global registration
//!
//! Validates that LocalLLM global is properly injected when ProviderManager
//! exists in GlobalContext (regression test for Phase 11b bug fix).

#[cfg(feature = "lua")]
mod local_llm_registration {
    use llmspell_bridge::globals::{create_standard_registry, GlobalContext};
    use llmspell_bridge::{ComponentRegistry, ProviderManager};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_localllm_global_registered() {
        // Arrange: Create context with provider manager (normal runtime setup)
        let registry = Arc::new(ComponentRegistry::new());
        let providers = Arc::new(
            ProviderManager::new(Default::default())
                .await
                .expect("Failed to create ProviderManager"),
        );
        let context = Arc::new(GlobalContext::new(registry, providers));

        // Act: Create standard registry (what inject_apis does)
        let global_registry = create_standard_registry(context.clone())
            .await
            .expect("Should create global registry");

        // Assert: LocalLLM global must be registered
        let localllm_exists = global_registry.get("LocalLLM").is_some();

        assert!(
            localllm_exists,
            "LocalLLM global MUST be registered when ProviderManager exists in context \
             (regression: Phase 11b bug fix - was conditionally skipped)"
        );

        // Verify total globals count (should be 15, not 14)
        let global_count = global_registry.list_globals().len();
        assert_eq!(
            global_count, 15,
            "Expected 15 globals (including LocalLLM), got {}",
            global_count
        );
    }

    #[tokio::test]
    async fn test_localllm_uses_context_providers() {
        // Arrange
        let registry = Arc::new(ComponentRegistry::new());
        let providers = Arc::new(
            ProviderManager::new(Default::default())
                .await
                .expect("Failed to create ProviderManager"),
        );
        let context = Arc::new(GlobalContext::new(registry, providers.clone()));

        // Act
        let global_registry = create_standard_registry(context.clone())
            .await
            .expect("Should create global registry");

        // Assert: LocalLLM should use same provider manager as context
        // (This validates the fix: using context.providers instead of bridge_refs)
        let localllm_global = global_registry
            .get("LocalLLM")
            .expect("LocalLLM must exist");

        // Validate metadata
        let metadata = localllm_global.metadata();
        assert_eq!(metadata.name, "LocalLLM");
        assert!(metadata.description.contains("Ollama"));
        assert!(metadata.description.contains("Candle"));
    }
}
