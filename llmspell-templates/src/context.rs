//! Execution context for template operations

use std::path::PathBuf;
use std::sync::Arc;

/// Execution context providing access to all infrastructure for template execution
///
/// This struct provides templates with access to agents, tools, workflows, RAG, LLM providers,
/// state management, and session management. It's the primary dependency injection mechanism
/// for templates.
#[derive(Clone)]
pub struct ExecutionContext {
    /// State manager (optional, from llmspell-kernel)
    pub state_manager: Option<Arc<llmspell_kernel::state::StateManager>>,

    /// Session manager (optional, from llmspell-kernel)
    pub session_manager: Option<Arc<llmspell_kernel::sessions::manager::SessionManager>>,

    /// Tool registry (from llmspell-tools)
    pub tool_registry: Arc<llmspell_tools::ToolRegistry>,

    /// Agent factory registry (from llmspell-agents)
    pub agent_registry: Arc<llmspell_agents::FactoryRegistry>,

    /// Workflow factory (from llmspell-workflows)
    pub workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory>,

    /// RAG multi-tenant integration (optional, from llmspell-rag)
    pub rag: Option<Arc<llmspell_rag::multi_tenant_integration::MultiTenantRAG>>,

    /// Provider manager for LLM access (from llmspell-providers)
    pub providers: Arc<llmspell_providers::ProviderManager>,

    /// Kernel handle for REPL/interactive sessions (optional, Subtask 12.9.5)
    pub kernel_handle: Option<Arc<llmspell_kernel::api::KernelHandle>>,

    /// Session ID for scoped operations (optional)
    pub session_id: Option<String>,

    /// Output directory for artifacts (optional)
    pub output_dir: Option<PathBuf>,
}

impl ExecutionContext {
    /// Create a new execution context builder
    pub fn builder() -> ExecutionContextBuilder {
        ExecutionContextBuilder::default()
    }

    /// Get state manager
    pub fn state_manager(&self) -> Option<&Arc<llmspell_kernel::state::StateManager>> {
        self.state_manager.as_ref()
    }

    /// Get session manager
    pub fn session_manager(
        &self,
    ) -> Option<&Arc<llmspell_kernel::sessions::manager::SessionManager>> {
        self.session_manager.as_ref()
    }

    /// Get tool registry
    pub fn tool_registry(&self) -> &Arc<llmspell_tools::ToolRegistry> {
        &self.tool_registry
    }

    /// Get agent registry
    pub fn agent_registry(&self) -> &Arc<llmspell_agents::FactoryRegistry> {
        &self.agent_registry
    }

    /// Get workflow factory
    pub fn workflow_factory(&self) -> &Arc<dyn llmspell_workflows::WorkflowFactory> {
        &self.workflow_factory
    }

    /// Get RAG integration
    pub fn rag(&self) -> Option<&Arc<llmspell_rag::multi_tenant_integration::MultiTenantRAG>> {
        self.rag.as_ref()
    }

    /// Get provider manager
    pub fn providers(&self) -> &Arc<llmspell_providers::ProviderManager> {
        &self.providers
    }

    /// Get kernel handle for REPL/interactive sessions (Subtask 12.9.5)
    pub fn kernel_handle(&self) -> Option<&Arc<llmspell_kernel::api::KernelHandle>> {
        self.kernel_handle.as_ref()
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get output directory
    pub fn output_dir(&self) -> Option<&std::path::Path> {
        self.output_dir.as_deref()
    }

    /// Check if infrastructure component is available
    pub fn has_state(&self) -> bool {
        self.state_manager.is_some()
    }

    /// Check if session infrastructure is available
    pub fn has_sessions(&self) -> bool {
        self.session_manager.is_some()
    }

    /// Check if RAG is available
    pub fn has_rag(&self) -> bool {
        self.rag.is_some()
    }

    /// Require state manager or return error
    pub fn require_state(
        &self,
    ) -> crate::error::Result<&Arc<llmspell_kernel::state::StateManager>> {
        self.state_manager.as_ref().ok_or_else(|| {
            crate::error::TemplateError::InfrastructureUnavailable("state".to_string())
        })
    }

    /// Require session manager or return error
    pub fn require_sessions(
        &self,
    ) -> crate::error::Result<&Arc<llmspell_kernel::sessions::manager::SessionManager>> {
        self.session_manager.as_ref().ok_or_else(|| {
            crate::error::TemplateError::InfrastructureUnavailable("sessions".to_string())
        })
    }

    /// Require RAG or return error
    pub fn require_rag(
        &self,
    ) -> crate::error::Result<&Arc<llmspell_rag::multi_tenant_integration::MultiTenantRAG>> {
        self.rag.as_ref().ok_or_else(|| {
            crate::error::TemplateError::InfrastructureUnavailable("rag".to_string())
        })
    }
}

/// Builder for ExecutionContext
#[derive(Default)]
pub struct ExecutionContextBuilder {
    state_manager: Option<Arc<llmspell_kernel::state::StateManager>>,
    session_manager: Option<Arc<llmspell_kernel::sessions::manager::SessionManager>>,
    tool_registry: Option<Arc<llmspell_tools::ToolRegistry>>,
    agent_registry: Option<Arc<llmspell_agents::FactoryRegistry>>,
    workflow_factory: Option<Arc<dyn llmspell_workflows::WorkflowFactory>>,
    rag: Option<Arc<llmspell_rag::multi_tenant_integration::MultiTenantRAG>>,
    providers: Option<Arc<llmspell_providers::ProviderManager>>,
    kernel_handle: Option<Arc<llmspell_kernel::api::KernelHandle>>,
    session_id: Option<String>,
    output_dir: Option<PathBuf>,
}

impl ExecutionContextBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set state manager
    pub fn with_state_manager(
        mut self,
        state_manager: Arc<llmspell_kernel::state::StateManager>,
    ) -> Self {
        self.state_manager = Some(state_manager);
        self
    }

    /// Set session manager
    pub fn with_session_manager(
        mut self,
        session_manager: Arc<llmspell_kernel::sessions::manager::SessionManager>,
    ) -> Self {
        self.session_manager = Some(session_manager);
        self
    }

    /// Set tool registry
    pub fn with_tool_registry(mut self, tool_registry: Arc<llmspell_tools::ToolRegistry>) -> Self {
        self.tool_registry = Some(tool_registry);
        self
    }

    /// Set agent registry
    pub fn with_agent_registry(
        mut self,
        agent_registry: Arc<llmspell_agents::FactoryRegistry>,
    ) -> Self {
        self.agent_registry = Some(agent_registry);
        self
    }

    /// Set workflow factory
    pub fn with_workflow_factory(
        mut self,
        workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory>,
    ) -> Self {
        self.workflow_factory = Some(workflow_factory);
        self
    }

    /// Set RAG integration
    pub fn with_rag(
        mut self,
        rag: Arc<llmspell_rag::multi_tenant_integration::MultiTenantRAG>,
    ) -> Self {
        self.rag = Some(rag);
        self
    }

    /// Set provider manager
    pub fn with_providers(mut self, providers: Arc<llmspell_providers::ProviderManager>) -> Self {
        self.providers = Some(providers);
        self
    }

    /// Set kernel handle (Subtask 12.9.5)
    pub fn with_kernel_handle(
        mut self,
        kernel_handle: Arc<llmspell_kernel::api::KernelHandle>,
    ) -> Self {
        self.kernel_handle = Some(kernel_handle);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set output directory
    pub fn with_output_dir(mut self, output_dir: PathBuf) -> Self {
        self.output_dir = Some(output_dir);
        self
    }

    /// Build the execution context
    ///
    /// # Errors
    ///
    /// Returns error if required components are missing (tool_registry, agent_registry, workflow_factory, providers)
    pub fn build(self) -> crate::error::Result<ExecutionContext> {
        Ok(ExecutionContext {
            state_manager: self.state_manager,
            session_manager: self.session_manager,
            tool_registry: self.tool_registry.ok_or_else(|| {
                crate::error::TemplateError::InfrastructureUnavailable(
                    "tool_registry is required".to_string(),
                )
            })?,
            agent_registry: self.agent_registry.ok_or_else(|| {
                crate::error::TemplateError::InfrastructureUnavailable(
                    "agent_registry is required".to_string(),
                )
            })?,
            workflow_factory: self.workflow_factory.ok_or_else(|| {
                crate::error::TemplateError::InfrastructureUnavailable(
                    "workflow_factory is required".to_string(),
                )
            })?,
            rag: self.rag,
            providers: self.providers.ok_or_else(|| {
                crate::error::TemplateError::InfrastructureUnavailable(
                    "providers is required".to_string(),
                )
            })?,
            kernel_handle: self.kernel_handle,
            session_id: self.session_id,
            output_dir: self.output_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_builder_requires_core_components() {
        let builder = ExecutionContextBuilder::new();

        // Should fail without required components
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_has_infrastructure_checks() {
        // This test would need actual infrastructure instances
        // For now, we just verify the methods exist
        let builder = ExecutionContextBuilder::new();

        // Verify builder methods exist
        let _ = builder
            .with_session_id("test")
            .with_output_dir(PathBuf::from("/tmp"));
    }
}
