//! Test helper utilities for llmspell-bridge tests

use llmspell_bridge::engine::bridge::ApiDependencies;
use llmspell_bridge::{ComponentRegistry, ProviderManager};
use llmspell_config::providers::ProviderManagerConfig;
use std::sync::Arc;

/// Create infrastructure registries for testing
///
/// Returns `(tool_registry, agent_registry, workflow_factory)` tuple with
/// empty registries that can be passed to `inject_apis()`.
#[must_use]
#[allow(dead_code)] // Used in other test files
pub fn create_test_infrastructure() -> (
    Arc<llmspell_tools::ToolRegistry>,
    Arc<llmspell_agents::FactoryRegistry>,
    Arc<dyn llmspell_workflows::WorkflowFactory>,
) {
    let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());
    let agent_registry = Arc::new(llmspell_agents::FactoryRegistry::new());
    let workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory> =
        Arc::new(llmspell_workflows::factory::DefaultWorkflowFactory::new());
    (tool_registry, agent_registry, workflow_factory)
}

/// Create API dependencies for testing
///
/// Returns fully configured `ApiDependencies` struct that can be passed to `inject_apis()`.
/// This is a convenience wrapper around `create_test_infrastructure()` that creates all
/// required dependencies and bundles them into the struct.
///
/// # Panics
///
/// Panics if `ProviderManager` creation fails (e.g., invalid default configuration).
#[must_use]
#[allow(dead_code)] // Used in test files
pub async fn create_test_api_deps() -> ApiDependencies {
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(
        ProviderManager::new(provider_config)
            .await
            .expect("Failed to create provider manager"),
    );
    let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

    ApiDependencies::new(
        registry,
        providers,
        tool_registry,
        agent_registry,
        workflow_factory,
    )
}

/// Execute test function with tokio runtime context
///
/// Provides runtime context needed for async operations in Lua tests.
/// Use this wrapper for any test that creates Lua environments with
/// Memory/Context/RAG globals that perform async operations.
///
/// # Why This Is Needed
///
/// When bridges (`MemoryBridge`, `ContextBridge`) call async methods, they use
/// `block_on_async()` which requires an active tokio runtime context.
/// Tests that directly create their own `tokio::runtime::Runtime` have this context,
/// but integration tests that just call Lua functions don't.
///
/// This helper provides the runtime context by entering the global IO runtime,
/// allowing Lua calls to bridge methods to work correctly.
///
/// # Example
///
/// ```no_run
/// # use llmspell_bridge::tests::test_helpers::with_runtime_context;
/// fn test_context_assemble() {
///     with_runtime_context(|| {
///         // Setup Lua environment
///         // Lua calls to Memory.episodic.add, Context.assemble, etc. will work
///     })
/// }
/// ```
///
/// # Design Rationale
///
/// This approach was chosen over alternatives:
/// - ❌ `#[tokio::test]`: Philosophically wrong for sync Lua tests
/// - ❌ Dependency injection: Against project architecture, breaks API
/// - ❌ Restore runtime field to bridges: Architectural regression
/// - ✅ Runtime context wrapper: Clean, reusable, production-realistic
#[allow(dead_code)] // Used in context_global_test, memory_context_integration_test
pub fn with_runtime_context<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = llmspell_kernel::global_io_runtime().enter();
    f()
}
