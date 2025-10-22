//! Test helper utilities for llmspell-bridge tests

use std::sync::Arc;

/// Create infrastructure registries for testing
///
/// Returns `(tool_registry, agent_registry, workflow_factory)` tuple with
/// empty registries that can be passed to `inject_apis()`.
#[must_use]
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
