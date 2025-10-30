//! Execution context for template operations

use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, trace};

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

    /// Provider configuration for smart dual-path resolution (Task 13.5.7d)
    pub provider_config: Arc<llmspell_config::providers::ProviderManagerConfig>,

    /// Kernel handle for REPL/interactive sessions (optional, Subtask 12.9.5)
    pub kernel_handle: Option<Arc<llmspell_kernel::api::KernelHandle>>,

    /// Session ID for scoped operations (optional)
    pub session_id: Option<String>,

    /// Output directory for artifacts (optional)
    pub output_dir: Option<PathBuf>,

    /// Memory manager for episodic and semantic memory (optional, Task 13.11.0)
    pub memory_manager: Option<Arc<dyn llmspell_memory::MemoryManager>>,

    /// Context bridge for memory-enhanced context assembly (optional, Task 13.11.1a)
    /// Uses ContextAssembler trait from llmspell-core for compile-time type safety
    pub context_bridge: Option<Arc<dyn llmspell_core::ContextAssembler>>,
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

    /// Get provider configuration by name (Task 13.5.7d)
    ///
    /// # Errors
    ///
    /// Returns error if provider name not found in configuration
    pub fn get_provider_config(
        &self,
        name: &str,
    ) -> crate::error::Result<llmspell_config::ProviderConfig> {
        self.provider_config
            .get_provider(name)
            .cloned()
            .ok_or_else(|| {
                crate::error::TemplateError::Config(format!("provider '{}' not found", name))
            })
    }

    /// Smart dual-path LLM config resolution: provider_name (preferred) OR model (ad-hoc)
    ///
    /// Supports three resolution paths (Task 13.5.7d):
    /// 1. `provider_name` param → centralized provider config (RECOMMENDED)
    /// 2. `model` param → ephemeral provider with inline overrides (backward compat)
    /// 3. Default provider → fallback from `ProviderManagerConfig`
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Both provider_name and model specified (mutually exclusive)
    /// - Neither specified and no default provider configured
    /// - Provider name not found in configuration
    pub fn resolve_llm_config(
        &self,
        params: &crate::TemplateParams,
    ) -> crate::error::Result<llmspell_config::ProviderConfig> {
        // 1. Check for provider_name (PREFERRED - centralized config)
        if let Some(provider_name) = params.get_optional::<String>("provider_name") {
            if params.contains("model") {
                return Err(crate::error::TemplateError::Config(
                    "Cannot specify both provider_name and model - use one or the other".into(),
                ));
            }
            return self.get_provider_config(&provider_name);
        }

        // 2. Check for model (AD-HOC - ephemeral provider)
        if let Some(model) = params.get_optional::<String>("model") {
            use llmspell_config::ProviderConfig;
            return Ok(ProviderConfig {
                name: "ephemeral".to_string(),
                provider_type: "ephemeral".to_string(),
                enabled: true,
                base_url: params.get_optional::<String>("base_url"),
                api_key_env: None,
                api_key: None,
                default_model: Some(model),
                max_tokens: params.get_optional::<u32>("max_tokens"),
                timeout_seconds: params.get_optional::<u64>("timeout_seconds"),
                temperature: params.get_optional::<f32>("temperature"),
                rate_limit: None,
                retry: None,
                max_retries: params.get_optional::<u32>("max_retries"),
                options: std::collections::HashMap::new(),
            });
        }

        // 3. Fallback to default provider
        self.provider_config
            .get_default_provider()
            .cloned()
            .ok_or_else(|| {
                crate::error::TemplateError::Config(
                    "No provider_name or model specified, and no default provider configured"
                        .into(),
                )
            })
    }

    /// Add memory manager to context (Task 13.11.0)
    pub fn with_memory(mut self, memory: Arc<dyn llmspell_memory::MemoryManager>) -> Self {
        debug!("ExecutionContext: Adding memory manager");
        self.memory_manager = Some(memory);
        self
    }

    /// Add context bridge to context (Task 13.11.1a)
    /// Uses ContextAssembler trait for compile-time type safety
    pub fn with_context_bridge(mut self, bridge: Arc<dyn llmspell_core::ContextAssembler>) -> Self {
        debug!("ExecutionContext: Adding context bridge");
        self.context_bridge = Some(bridge);
        self
    }

    /// Check if memory is available (Task 13.11.0)
    pub fn has_memory(&self) -> bool {
        self.memory_manager.is_some() && self.context_bridge.is_some()
    }

    /// Get memory manager if available (Task 13.11.0)
    pub fn memory_manager(&self) -> Option<Arc<dyn llmspell_memory::MemoryManager>> {
        trace!("ExecutionContext: Accessing memory manager");
        self.memory_manager.clone()
    }

    /// Get context bridge if available (Task 13.11.1a)
    /// Returns `Arc<dyn ContextAssembler>` for compile-time type safety
    pub fn context_bridge(&self) -> Option<Arc<dyn llmspell_core::ContextAssembler>> {
        trace!("ExecutionContext: Accessing context bridge");
        self.context_bridge.clone()
    }

    /// Require memory manager or return error (Task 13.11.0)
    pub fn require_memory(&self) -> crate::error::Result<Arc<dyn llmspell_memory::MemoryManager>> {
        self.memory_manager.clone().ok_or_else(|| {
            crate::error::TemplateError::InfrastructureUnavailable(
                "Memory manager not available in ExecutionContext".to_string(),
            )
        })
    }

    /// Require context bridge or return error (Task 13.11.1a)
    /// Returns `Arc<dyn ContextAssembler>` for compile-time type safety
    pub fn require_context_bridge(
        &self,
    ) -> crate::error::Result<Arc<dyn llmspell_core::ContextAssembler>> {
        self.context_bridge.clone().ok_or_else(|| {
            crate::error::TemplateError::InfrastructureUnavailable(
                "Context bridge not available in ExecutionContext".to_string(),
            )
        })
    }
}

// ============================================================================
// Context Assembly Helper (Task 13.11.2)
// ============================================================================

/// Message for LLM context (compatible with provider format)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
}

/// Assemble context from memory for template execution
///
/// Uses ContextAssembler trait from ExecutionContext to retrieve and format
/// context chunks from episodic/semantic memory using hybrid retrieval.
///
/// # Arguments
///
/// * `context_assembler` - ContextAssembler implementation (from ExecutionContext.context_bridge())
/// * `query` - Query string for context retrieval
/// * `session_id` - Session ID for episodic memory filtering
/// * `context_budget` - Maximum tokens for assembled context (default: 2000)
///
/// # Returns
///
/// Vec of ContextMessage prepended to LLM input, or empty vec on failure.
///
/// # Errors
///
/// Returns empty vec with warning if assembly fails (graceful degradation).
///
/// # Example
///
/// ```ignore
/// let messages = if let Some(bridge) = context.context_bridge() {
///     assemble_template_context(&bridge, "topic", "session-123", 2000).await
/// } else {
///     vec![]
/// };
/// ```
pub async fn assemble_template_context(
    context_assembler: &Arc<dyn llmspell_core::ContextAssembler>,
    query: &str,
    session_id: &str,
    context_budget: i64,
) -> Vec<ContextMessage> {
    use tracing::{debug, info, warn};

    info!(
        "Assembling context: session={}, budget={}, query={}",
        session_id,
        context_budget,
        query.chars().take(50).collect::<String>()
    );

    // Call context assembler with hybrid strategy
    let result = context_assembler
        .assemble(query, "hybrid", context_budget as usize, Some(session_id))
        .await;

    match result {
        Ok(ctx_json) => {
            // Extract chunks from JSON
            let chunks = ctx_json["chunks"].as_array();
            let token_count = ctx_json["token_count"].as_u64().unwrap_or(0);
            let formatted = ctx_json["formatted"].as_str().unwrap_or("");

            if let Some(chunks) = chunks {
                debug!("Assembled {} chunks, {} tokens", chunks.len(), token_count);

                // Convert to ContextMessage format
                let messages: Vec<ContextMessage> = vec![ContextMessage {
                    role: "system".to_string(),
                    content: format!(
                        "Previous context from memory ({} chunks, {} tokens):\n\n{}",
                        chunks.len(),
                        token_count,
                        formatted
                    ),
                }];

                info!("Context ready: {} messages", messages.len());
                messages
            } else {
                warn!("No chunks in assembled context, proceeding without memory");
                vec![]
            }
        }
        Err(e) => {
            warn!("Context assembly failed: {}, continuing without context", e);
            vec![]
        }
    }
}

// ============================================================================
// Memory Storage Helper (Task 13.11.3)
// ============================================================================

/// Store template execution in episodic memory
///
/// Stores both input and output as separate episodic entries for future context retrieval.
/// Uses MemoryManager from ExecutionContext to persist template interactions.
///
/// # Arguments
///
/// * `memory_manager` - MemoryManager implementation (from ExecutionContext.memory_manager())
/// * `session_id` - Session ID for episodic grouping
/// * `template_id` - Template identifier
/// * `input_summary` - Summary of template input
/// * `output_summary` - Summary of template output
/// * `metadata` - Additional metadata (duration, params, etc.)
///
/// # Returns
///
/// Ok(()) on success, or error if storage fails
///
/// # Errors
///
/// Returns error if episodic memory storage fails (logged as warning)
///
/// # Example
///
/// ```ignore
/// if let Some(memory) = context.memory_manager() {
///     store_template_execution(
///         &memory,
///         "session-123",
///         "research-assistant",
///         "Research topic: AI",
///         "Found 10 sources",
///         json!({"duration_ms": 1500})
///     ).await.ok(); // Don't fail execution on storage error
/// }
/// ```
pub async fn store_template_execution(
    memory_manager: &Arc<dyn llmspell_memory::MemoryManager>,
    session_id: &str,
    template_id: &str,
    input_summary: &str,
    output_summary: &str,
    metadata: serde_json::Value,
) -> crate::error::Result<()> {
    use llmspell_memory::EpisodicEntry;
    use tracing::{debug, info, warn};

    debug!(
        "Storing template execution in memory: template={}, session={}",
        template_id, session_id
    );

    // Store input (user role)
    let mut input_entry = EpisodicEntry::new(
        session_id.to_string(),
        "user".to_string(),
        format!("Template: {} - Input: {}", template_id, input_summary),
    );
    input_entry.metadata = serde_json::json!({
        "template_id": template_id,
        "type": "template_input",
        "metadata": metadata,
    });

    memory_manager
        .episodic()
        .add(input_entry)
        .await
        .map_err(|e| {
            warn!("Failed to store template input in memory: {}", e);
            crate::error::TemplateError::ExecutionFailed(format!("Memory storage failed: {}", e))
        })?;

    // Store output (assistant role)
    let mut output_entry = EpisodicEntry::new(
        session_id.to_string(),
        "assistant".to_string(),
        format!("Template: {} - Output: {}", template_id, output_summary),
    );
    output_entry.metadata = serde_json::json!({
        "template_id": template_id,
        "type": "template_output",
        "metadata": metadata,
    });

    memory_manager
        .episodic()
        .add(output_entry)
        .await
        .map_err(|e| {
            warn!("Failed to store template output in memory: {}", e);
            crate::error::TemplateError::ExecutionFailed(format!("Memory storage failed: {}", e))
        })?;

    info!(
        "Template execution stored in memory: session={}, template={}",
        session_id, template_id
    );
    Ok(())
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
    provider_config: Option<Arc<llmspell_config::providers::ProviderManagerConfig>>,
    kernel_handle: Option<Arc<llmspell_kernel::api::KernelHandle>>,
    session_id: Option<String>,
    output_dir: Option<PathBuf>,
    memory_manager: Option<Arc<dyn llmspell_memory::MemoryManager>>,
    context_bridge: Option<Arc<dyn llmspell_core::ContextAssembler>>,
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

    /// Set provider configuration (Task 13.5.7d)
    pub fn with_provider_config(
        mut self,
        provider_config: Arc<llmspell_config::providers::ProviderManagerConfig>,
    ) -> Self {
        self.provider_config = Some(provider_config);
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

    /// Set memory manager (Task 13.11.0)
    pub fn with_memory_manager(
        mut self,
        memory_manager: Arc<dyn llmspell_memory::MemoryManager>,
    ) -> Self {
        self.memory_manager = Some(memory_manager);
        self
    }

    /// Set context bridge (Task 13.11.1a)
    /// Uses ContextAssembler trait for compile-time type safety
    pub fn with_context_bridge(
        mut self,
        context_bridge: Arc<dyn llmspell_core::ContextAssembler>,
    ) -> Self {
        self.context_bridge = Some(context_bridge);
        self
    }

    /// Build the execution context
    ///
    /// # Errors
    ///
    /// Returns error if required components are missing (tool_registry, agent_registry, workflow_factory, providers)
    pub fn build(self) -> crate::error::Result<ExecutionContext> {
        debug!(
            "Building ExecutionContext with memory={}, context_bridge={}",
            self.memory_manager.is_some(),
            self.context_bridge.is_some()
        );

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
            provider_config: self.provider_config.ok_or_else(|| {
                crate::error::TemplateError::InfrastructureUnavailable(
                    "provider_config is required".to_string(),
                )
            })?,
            kernel_handle: self.kernel_handle,
            session_id: self.session_id,
            output_dir: self.output_dir,
            memory_manager: self.memory_manager,
            context_bridge: self.context_bridge,
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

    #[tokio::test]
    async fn test_execution_context_memory_manager_field() {
        // Create in-memory MemoryManager for testing
        let memory = Arc::new(
            llmspell_memory::DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager"),
        );

        // Create minimal ExecutionContext with memory manager
        let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());
        let agent_registry = Arc::new(llmspell_agents::FactoryRegistry::new());
        let workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory> =
            Arc::new(llmspell_workflows::DefaultWorkflowFactory::new());
        let providers = Arc::new(llmspell_providers::ProviderManager::new());
        let provider_config =
            Arc::new(llmspell_config::providers::ProviderManagerConfig::default());

        let context = ExecutionContextBuilder::new()
            .with_tool_registry(tool_registry)
            .with_agent_registry(agent_registry)
            .with_workflow_factory(workflow_factory)
            .with_providers(providers)
            .with_provider_config(provider_config)
            .with_memory_manager(memory.clone())
            .build()
            .expect("Failed to build context");

        assert!(context.memory_manager().is_some());
    }

    #[tokio::test]
    async fn test_execution_context_require_memory() {
        // Create context WITHOUT memory
        let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());
        let agent_registry = Arc::new(llmspell_agents::FactoryRegistry::new());
        let workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory> =
            Arc::new(llmspell_workflows::DefaultWorkflowFactory::new());
        let providers = Arc::new(llmspell_providers::ProviderManager::new());
        let provider_config =
            Arc::new(llmspell_config::providers::ProviderManagerConfig::default());

        let context = ExecutionContextBuilder::new()
            .with_tool_registry(tool_registry)
            .with_agent_registry(agent_registry)
            .with_workflow_factory(workflow_factory)
            .with_providers(providers)
            .with_provider_config(provider_config)
            .build()
            .expect("Failed to build context");

        assert!(!context.has_memory());
        assert!(context.require_memory().is_err());
        assert!(context.require_context_bridge().is_err());
    }
}
