//! ABOUTME: Step execution engine for basic workflows
//! ABOUTME: Handles individual step execution with timeout, retry, and error handling

use super::hooks::{StepContext, WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext};
use super::traits::{ErrorStrategy, StepResult, StepType, WorkflowStep};
use super::types::{StepExecutionContext, WorkflowConfig};
use llmspell_core::{ComponentId, ComponentLookup, ComponentMetadata, LLMSpellError, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::field::Empty;
use tracing::{debug, error, info, instrument, warn};

/// Basic step executor for workflow steps
#[derive(Clone)]
pub struct StepExecutor {
    config: WorkflowConfig,
    /// Optional workflow executor for hook integration
    workflow_executor: Option<Arc<WorkflowExecutor>>,
    /// Optional component registry for looking up agents, tools, and workflows
    /// This is passed from the bridge layer during workflow creation
    registry: Option<Arc<dyn ComponentLookup>>,
}

impl StepExecutor {
    /// Create a new step executor with configuration
    pub fn new(config: WorkflowConfig) -> Self {
        Self {
            config,
            workflow_executor: None,
            registry: None,
        }
    }

    /// Create a new step executor with registry for component lookup
    pub fn new_with_registry(config: WorkflowConfig, registry: Arc<dyn ComponentLookup>) -> Self {
        Self {
            config,
            workflow_executor: None,
            registry: Some(registry),
        }
    }

    /// Create a new step executor with hook integration
    pub fn new_with_hooks(
        config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
    ) -> Self {
        Self {
            config,
            workflow_executor: Some(workflow_executor),
            registry: None,
        }
    }

    /// Create a new step executor with both hooks and registry
    pub fn new_with_hooks_and_registry(
        config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
        registry: Arc<dyn ComponentLookup>,
    ) -> Self {
        Self {
            config,
            workflow_executor: Some(workflow_executor),
            registry: Some(registry),
        }
    }

    /// Execute a single step with retry logic
    #[instrument(level = "info", skip(self, step, context), fields(
        step_name = %step.name,
        step_type = ?step.step_type,
        execution_count = Empty,
        retry_attempt = Empty
    ))]
    pub async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: StepExecutionContext,
    ) -> Result<StepResult> {
        self.execute_step_with_metadata(step, context, None, None)
            .await
    }

    /// Execute a single step with workflow metadata for hooks
    #[instrument(level = "info", skip_all, fields(
        step_name = %step.name,
        step_type = ?step.step_type,
        workflow_id = "",
        workflow_name = workflow_metadata.as_ref().map_or("", |m| m.name.as_str()),
        step_name = %step.name,
        step_type = ?step.step_type
    ))]
    pub async fn execute_step_with_metadata(
        &self,
        step: &WorkflowStep,
        context: StepExecutionContext,
        workflow_metadata: Option<ComponentMetadata>,
        workflow_type: Option<String>,
    ) -> Result<StepResult> {
        let start_time = Instant::now();
        let step_timeout = step.timeout.unwrap_or(self.config.default_step_timeout);

        debug!(
            "Executing step '{}' (id: {:?}) with timeout: {:?}",
            step.name, step.id, step_timeout
        );

        // Execute pre-step hooks if available
        if let (Some(workflow_executor), Some(metadata), Some(wf_type)) =
            (&self.workflow_executor, &workflow_metadata, &workflow_type)
        {
            let step_ctx = self.create_step_context(step, &context, None);
            let _ = workflow_executor
                .execute_step_hooks(
                    metadata.clone(),
                    context.workflow_state.clone(),
                    wf_type.clone(),
                    step_ctx,
                    true, // is_pre_execution
                )
                .await;
        }

        // Execute with timeout
        let result = timeout(
            step_timeout,
            self.execute_step_internal(
                step,
                &context,
                workflow_metadata.clone(),
                workflow_type.clone(),
            ),
        )
        .await;

        let duration = start_time.elapsed();

        let step_result = match result {
            Ok(Ok(output)) => {
                debug!(
                    "Step '{}' completed successfully in {:?}",
                    step.name, duration
                );
                StepResult::success(step.id, step.name.clone(), output, duration)
            }
            Ok(Err(err)) => {
                warn!(
                    "Step '{}' failed: {} (duration: {:?})",
                    step.name, err, duration
                );

                // Execute error hooks if available
                if let (Some(workflow_executor), Some(metadata), Some(wf_type)) =
                    (&self.workflow_executor, &workflow_metadata, &workflow_type)
                {
                    let component_id = llmspell_hooks::ComponentId::new(
                        llmspell_hooks::ComponentType::Workflow,
                        format!("workflow_{}", metadata.name),
                    );
                    let mut hook_ctx = WorkflowHookContext::new(
                        component_id,
                        metadata.clone(),
                        context.workflow_state.clone(),
                        wf_type.clone(),
                        WorkflowExecutionPhase::ErrorHandling,
                    );
                    let step_ctx = self.create_step_context(step, &context, Some(err.to_string()));
                    hook_ctx = hook_ctx.with_step_context(step_ctx);
                    let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
                }

                StepResult::failure(
                    step.id,
                    step.name.clone(),
                    err.to_string(),
                    duration,
                    context.retry_attempt,
                )
            }
            Err(_) => {
                error!("Step '{}' timed out after {:?}", step.name, step_timeout);
                StepResult::failure(
                    step.id,
                    step.name.clone(),
                    format!("Step timed out after {:?}", step_timeout),
                    duration,
                    context.retry_attempt,
                )
            }
        };

        // Execute post-step hooks if available
        if let (Some(workflow_executor), Some(metadata), Some(wf_type)) =
            (&self.workflow_executor, &workflow_metadata, &workflow_type)
        {
            let step_ctx = self.create_step_context_with_result(step, &context, &step_result);
            let _ = workflow_executor
                .execute_step_hooks(
                    metadata.clone(),
                    context.workflow_state.clone(),
                    wf_type.clone(),
                    step_ctx,
                    false, // is_pre_execution
                )
                .await;
        }

        Ok(step_result)
    }

    /// Execute a step with retry logic
    #[instrument(level = "info", skip(self, step, context, error_strategy), fields(
        step_name = %step.name
    ))]
    pub async fn execute_step_with_retry(
        &self,
        step: &WorkflowStep,
        context: StepExecutionContext,
        error_strategy: &ErrorStrategy,
    ) -> Result<StepResult> {
        self.execute_step_with_retry_and_metadata(step, context, error_strategy, None, None)
            .await
    }

    /// Execute a step with retry logic and workflow metadata
    pub async fn execute_step_with_retry_and_metadata(
        &self,
        step: &WorkflowStep,
        mut context: StepExecutionContext,
        error_strategy: &ErrorStrategy,
        workflow_metadata: Option<ComponentMetadata>,
        workflow_type: Option<String>,
    ) -> Result<StepResult> {
        let max_attempts = match error_strategy {
            ErrorStrategy::Retry { max_attempts, .. } => *max_attempts,
            _ => 1, // No retry for other strategies
        };

        let mut last_result = None;

        for attempt in 0..max_attempts {
            context = context.with_retry(attempt, max_attempts);

            debug!(
                "Attempting step '{}' (attempt {}/{})",
                step.name,
                attempt + 1,
                max_attempts
            );

            let result = self
                .execute_step_with_metadata(
                    step,
                    context.clone(),
                    workflow_metadata.clone(),
                    workflow_type.clone(),
                )
                .await?;

            if result.success {
                return Ok(result);
            }

            last_result = Some(result);

            // Don't wait after the last attempt
            if attempt < max_attempts - 1 {
                if let ErrorStrategy::Retry { backoff_ms, .. } = error_strategy {
                    let delay = if self.config.exponential_backoff {
                        Duration::from_millis(backoff_ms * 2_u64.pow(attempt))
                    } else {
                        Duration::from_millis(*backoff_ms)
                    };

                    debug!("Step '{}' failed, retrying in {:?}", step.name, delay);

                    tokio::time::sleep(delay).await;
                }
            }
        }

        // Return the last failure result with updated retry count
        let mut final_result = last_result.unwrap();
        final_result.retry_count = max_attempts;
        Ok(final_result)
    }

    /// Internal step execution logic with hook integration
    #[instrument(level = "debug", skip_all, fields(
        step_name = %step.name,
        step_type = ?step.step_type
    ))]
    async fn execute_step_internal(
        &self,
        step: &WorkflowStep,
        context: &StepExecutionContext,
        workflow_metadata: Option<ComponentMetadata>,
        workflow_type: Option<String>,
    ) -> Result<String> {
        // Get step type name for events
        let step_type_name = match &step.step_type {
            StepType::Tool { .. } => "tool",
            StepType::Agent { .. } => "agent",
            StepType::Workflow { .. } => "workflow",
        };

        // Emit step started event if events are available
        // Note: We only use this for reading events, actual execution context is created per step
        let exec_context_for_events = context.to_execution_context();
        if let Some(ref events) = exec_context_for_events.events {
            let _ = events
                .emit(
                    "workflow.step.started",
                    serde_json::json!({
                        "workflow_id": context.workflow_state.execution_id.to_string(),
                        "step_name": step.name,
                        "step_type": step_type_name,
                        "step_index": context.workflow_state.current_step,
                        "retry_attempt": context.retry_attempt,
                    }),
                )
                .await;
        }

        // Execute pre-execution hooks at the internal level (fine-grained hooks)
        if let (Some(ref workflow_executor), Some(ref metadata), Some(ref wf_type)) =
            (&self.workflow_executor, &workflow_metadata, &workflow_type)
        {
            let component_id = llmspell_hooks::ComponentId::new(
                llmspell_hooks::ComponentType::Workflow,
                format!("workflow_{}", metadata.name),
            );

            let mut hook_ctx = WorkflowHookContext::new(
                component_id,
                metadata.clone(),
                context.workflow_state.clone(),
                wf_type.clone(),
                WorkflowExecutionPhase::StepBoundary,
            );

            // Add step context to differentiate from outer hooks
            let step_ctx = self.create_step_context(step, context, None);
            hook_ctx = hook_ctx.with_step_context(step_ctx);

            // Add pattern context to indicate this is internal execution
            hook_ctx = hook_ctx.with_pattern_context(
                "execution_level".to_string(),
                serde_json::json!("internal_pre"),
            );

            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        // Execute the actual step based on its type
        let start_time = std::time::Instant::now();
        // DEBUG: Log the step type and details
        match &step.step_type {
            StepType::Agent { agent_id, input } => {
                debug!(
                    "DEBUG: Step '{}' is Agent type with agent_id: '{}', input: '{}'",
                    step.name, agent_id, input
                );
            }
            StepType::Tool { tool_name, .. } => {
                debug!(
                    "DEBUG: Step '{}' is Tool type with tool: '{}'",
                    step.name, tool_name
                );
            }
            StepType::Workflow { workflow_id, .. } => {
                debug!(
                    "DEBUG: Step '{}' is Workflow type with workflow_id: {:?}",
                    step.name, workflow_id
                );
            }
        }

        let result = match &step.step_type {
            StepType::Tool {
                tool_name,
                parameters,
            } => self.execute_tool_step(tool_name, parameters, context).await,
            StepType::Agent { agent_id, input } => {
                debug!(
                    "DEBUG: About to execute agent step for agent_id: '{}'",
                    agent_id
                );
                self.execute_agent_step(agent_id, input, context).await
            }
            StepType::Workflow { workflow_id, input } => {
                self.execute_workflow_step(*workflow_id, input, context)
                    .await
            }
        };

        let duration = start_time.elapsed();

        // Emit step completed or failed event
        if let Some(ref events) = exec_context_for_events.events {
            match &result {
                Ok(output) => {
                    let _ = events
                        .emit(
                            "workflow.step.completed",
                            serde_json::json!({
                                "workflow_id": context.workflow_state.execution_id.to_string(),
                                "step_name": step.name,
                                "step_type": step_type_name,
                                "step_index": context.workflow_state.current_step,
                                "duration_ms": duration.as_millis(),
                                "output_size": output.len(),
                                "retry_attempt": context.retry_attempt,
                            }),
                        )
                        .await;
                }
                Err(e) => {
                    let _ = events
                        .emit(
                            "workflow.step.failed",
                            serde_json::json!({
                                "workflow_id": context.workflow_state.execution_id.to_string(),
                                "step_name": step.name,
                                "step_type": step_type_name,
                                "step_index": context.workflow_state.current_step,
                                "error": e.to_string(),
                                "duration_ms": duration.as_millis(),
                                "retry_attempt": context.retry_attempt,
                            }),
                        )
                        .await;
                }
            }
        }

        // Execute post-execution hooks at the internal level
        if let (Some(ref workflow_executor), Some(ref metadata), Some(ref wf_type)) =
            (&self.workflow_executor, &workflow_metadata, &workflow_type)
        {
            let component_id = llmspell_hooks::ComponentId::new(
                llmspell_hooks::ComponentType::Workflow,
                format!("workflow_{}", metadata.name),
            );

            // Create hook context with result information
            let mut hook_ctx = WorkflowHookContext::new(
                component_id,
                metadata.clone(),
                context.workflow_state.clone(),
                wf_type.clone(),
                WorkflowExecutionPhase::StepBoundary,
            );

            // Include step context with result or error
            let step_ctx = if let Err(ref e) = result {
                self.create_step_context(step, context, Some(e.to_string()))
            } else {
                self.create_step_context(step, context, None)
            };
            hook_ctx = hook_ctx.with_step_context(step_ctx);

            // Add pattern context to indicate this is internal post-execution
            hook_ctx = hook_ctx.with_pattern_context(
                "execution_level".to_string(),
                serde_json::json!("internal_post"),
            );

            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        result
    }

    /// Execute a tool step
    async fn execute_tool_step(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
        context: &StepExecutionContext,
    ) -> Result<String> {
        debug!("Executing tool step: {}", tool_name);

        // Validate tool name
        if tool_name.is_empty() {
            return Err(LLMSpellError::Workflow {
                message: "Tool name cannot be empty".to_string(),
                step: Some("tool_execution".to_string()),
                source: None,
            });
        }

        // Get registry or fall back to mock execution
        let Some(ref registry) = self.registry else {
            // Fall back to mock execution for backward compatibility in tests
            warn!(
                "No registry available, using mock execution for tool: {}",
                tool_name
            );
            return self.execute_tool_step_mock(tool_name, parameters).await;
        };

        // Lookup tool from registry
        let tool = registry
            .get_tool(tool_name)
            .await
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Tool '{}' not found in registry", tool_name),
                source: None,
            })?;

        // Create AgentInput from parameters
        // Tools typically expect parameters as a JSON object or specific fields
        let mut agent_input = if let Some(text) = parameters.get("input").and_then(|v| v.as_str()) {
            llmspell_core::types::AgentInput::text(text)
        } else if let Some(text) = parameters.get("text").and_then(|v| v.as_str()) {
            llmspell_core::types::AgentInput::text(text)
        } else {
            // Use the entire parameters as text if no specific input field
            llmspell_core::types::AgentInput::text(parameters.to_string())
        };

        // Tools expect parameters to be wrapped in a "parameters" object
        // This is required by the extract_parameters utility function
        agent_input = agent_input.with_parameter("parameters".to_string(), parameters.clone());

        // Convert StepExecutionContext to ExecutionContext for BaseAgent execution
        let exec_context = context.to_execution_context();

        // Execute through BaseAgent trait with automatic event emission
        let output = tool.execute(agent_input, exec_context).await?;

        // Write output to state if state accessor is available
        if let Some(ref state_accessor) = context.state_accessor {
            let workflow_id = context.workflow_state.execution_id.to_string();

            // Use standardized state key functions
            let output_key = crate::types::state_keys::step_output(&workflow_id, tool_name);
            let metadata_key = crate::types::state_keys::step_metadata(&workflow_id, tool_name);

            // Store the output in state
            state_accessor.set(&output_key, serde_json::to_value(&output.text)?);

            // Store metadata
            state_accessor.set(&metadata_key, serde_json::to_value(&output.metadata)?);

            // Emit state change event if events are available
            let exec_context = context.to_execution_context();
            if let Some(ref events) = exec_context.events {
                if events.config().emit_state_events {
                    let _ = events
                        .emit(
                            "workflow.state.updated",
                            serde_json::json!({
                                "workflow_id": workflow_id,
                                "keys": [output_key, metadata_key],
                                "operation": "write",
                                "component": tool_name,
                            }),
                        )
                        .await;
                }
            }
        }

        Ok(output.text)
    }

    /// Mock execution fallback for tools (used when no registry is available)
    async fn execute_tool_step_mock(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
    ) -> Result<String> {
        // Mock execution based on tool name (for backward compatibility)
        let output = match tool_name {
            "calculator" => {
                let expression = parameters
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                format!("Calculator result for '{}': 42", expression)
            }
            "file_operations" => {
                let operation = parameters
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("read");
                format!("File operation '{}' completed", operation)
            }
            "json_processor" => {
                let default_input = serde_json::json!({});
                let input = parameters.get("input").unwrap_or(&default_input);
                format!("JSON processed: {}", input)
            }
            "email_handler" => {
                let priority = parameters
                    .get("priority")
                    .and_then(|v| v.as_str())
                    .unwrap_or("normal");
                format!("Email handled with priority: {}", priority)
            }
            "text_processor" => {
                let action = parameters
                    .get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("process");
                format!("Text processing completed: {}", action)
            }
            "mock_tool" | "tool" | "test_tool" => {
                // Generic mock tools for testing
                format!(
                    "Mock tool '{}' executed with parameters: {}",
                    tool_name, parameters
                )
            }
            "http_request" => {
                let method = parameters
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("GET");
                format!("HTTP {} request completed", method)
            }
            "data_processor" => {
                let operation = parameters
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("process");
                format!("Data processing operation '{}' completed", operation)
            }
            "csv_parser" => {
                format!("CSV data parsed with parameters: {}", parameters)
            }
            "text_parser" => {
                format!("Text parsed with parameters: {}", parameters)
            }
            "item_processor" => {
                let default_item = serde_json::json!("item");
                let item = parameters.get("item").unwrap_or(&default_item);
                format!("Item processed: {}", item)
            }
            _ => {
                // Unknown tools should fail
                return Err(LLMSpellError::Component {
                    message: format!("Tool '{}' not found in mock registry", tool_name),
                    source: None,
                });
            }
        };

        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(output)
    }

    /// Execute an agent step
    async fn execute_agent_step(
        &self,
        agent_name: &str, // Changed from ComponentId to String to use original agent name
        input: &str,
        context: &StepExecutionContext,
    ) -> Result<String> {
        debug!("Executing agent step: '{}'", agent_name);

        // Validate input
        if input.is_empty() {
            return Err(LLMSpellError::Workflow {
                message: "Agent input cannot be empty".to_string(),
                step: Some("agent_execution".to_string()),
                source: None,
            });
        }

        // Execute agent (mock or real) and get output text
        let output_text = if let Some(ref registry) = self.registry {
            // Real agent execution with registry
            debug!("DEBUG: Looking for agent with name: '{}'", agent_name);

            // Look up agent by its original name
            let agent = registry.get_agent(agent_name).await.ok_or_else(|| {
                error!("DEBUG: Agent '{}' not found in registry", agent_name);
                LLMSpellError::Component {
                    message: format!("Agent '{}' not found in registry", agent_name),
                    source: None,
                }
            })?;

            // Create AgentInput from the provided input string
            let agent_input = llmspell_core::types::AgentInput::text(input);

            // Convert StepExecutionContext to ExecutionContext for BaseAgent execution
            // This will preserve the state field if it was set in StepExecutionContext
            let mut exec_context = context.to_execution_context();

            info!(
                "execute_agent_step: context.state before conversion: {}, exec_context.state after conversion: {}",
                context.state.is_some(),
                exec_context.state.is_some()
            );

            // Override scope to Agent for this execution
            // Create a ComponentId from the agent name for the scope
            let agent_id = ComponentId::from_name(agent_name);
            exec_context.scope = llmspell_core::execution_context::ContextScope::Agent(agent_id);

            // Set workflow execution ID in session if not already set
            if exec_context.session_id.is_none() {
                exec_context.session_id = Some(context.workflow_state.execution_id.to_string());
            }

            // Execute through BaseAgent trait with automatic event emission
            let output = agent.execute(agent_input, exec_context.clone()).await?;

            // Extract output text for return and state writing
            output.text
        } else {
            // Fall back to mock execution for backward compatibility in tests
            warn!(
                "No registry available, using mock execution for agent: '{}'",
                agent_name
            );
            // Create a ComponentId for the mock function (temporary)
            let mock_id = ComponentId::from_name(agent_name);
            self.execute_agent_step_mock(mock_id, input).await?
        };

        // Write agent output to state if state is available
        // This happens for BOTH mock and real execution to ensure consistent behavior
        if let Some(ref state) = context.state {
            let workflow_id = context.workflow_state.execution_id.to_string();

            // Use standardized state key functions
            let output_key = crate::types::state_keys::agent_output(&workflow_id, agent_name);

            info!("Writing agent output to state with key: {}", output_key);
            info!("Agent output text: {:?}", output_text);

            // Store the output in state
            state
                .write(&output_key, serde_json::to_value(&output_text)?)
                .await?;

            info!("Successfully wrote agent output to state");
        } else {
            debug!("No state available in StepExecutionContext to write agent output");
        }

        Ok(output_text)
    }

    /// Mock execution fallback for agents (used when no registry is available)
    async fn execute_agent_step_mock(&self, agent_id: ComponentId, input: &str) -> Result<String> {
        // Mock agent execution
        let output = format!("Agent {:?} processed: {}", agent_id, input);

        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(20)).await;

        Ok(output)
    }

    /// Execute a nested workflow step
    async fn execute_workflow_step(
        &self,
        workflow_id: ComponentId,
        input: &serde_json::Value,
        context: &StepExecutionContext,
    ) -> Result<String> {
        debug!("Executing nested workflow step: {:?}", workflow_id);

        // Get registry or fall back to mock execution
        let Some(ref registry) = self.registry else {
            // Fall back to mock execution for backward compatibility in tests
            warn!(
                "No registry available, using mock execution for workflow: {:?}",
                workflow_id
            );
            return self.execute_workflow_step_mock(workflow_id, input).await;
        };

        // Try to lookup workflow by ID string representation
        let workflow_name = workflow_id.to_string();
        let workflow = registry.get_workflow(&workflow_name).await.ok_or_else(|| {
            LLMSpellError::Component {
                message: format!("Workflow '{}' not found in registry", workflow_name),
                source: None,
            }
        })?;

        // Create AgentInput from the provided input
        // Workflows can accept either structured JSON or text input
        let agent_input = if let Some(text) = input.get("input").and_then(|v| v.as_str()) {
            llmspell_core::types::AgentInput::text(text)
        } else if let Some(text) = input.as_str() {
            llmspell_core::types::AgentInput::text(text)
        } else {
            // Use the entire input as text if it's an object
            let mut agent_input = llmspell_core::types::AgentInput::text(input.to_string());

            // Add any object fields as parameters
            if let Some(obj) = input.as_object() {
                for (key, value) in obj {
                    agent_input = agent_input.with_parameter(key.clone(), value.clone());
                }
            }

            agent_input
        };

        // Create child context for nested workflow execution with inheritance
        let exec_context = context.create_child_context(
            &workflow_name,
            llmspell_core::execution_context::InheritancePolicy::Inherit,
        );

        // Execute through BaseAgent trait with automatic event emission
        let output = workflow.execute(agent_input, exec_context).await?;

        // Write output to state if state accessor is available
        if let Some(ref state_accessor) = context.state_accessor {
            let workflow_id = context.workflow_state.execution_id.to_string();

            // Use standardized state key functions
            let output_key =
                crate::types::state_keys::nested_workflow_output(&workflow_id, &workflow_name);
            let metadata_key =
                crate::types::state_keys::nested_workflow_metadata(&workflow_id, &workflow_name);

            // Store the output in state
            state_accessor.set(&output_key, serde_json::to_value(&output.text)?);

            // Store combined metadata including execution details
            let mut metadata = output.metadata;
            metadata
                .extra
                .insert("workflow_id".to_string(), serde_json::json!(workflow_name));
            metadata.extra.insert("input".to_string(), input.clone());
            metadata.extra.insert(
                "completed_at".to_string(),
                serde_json::json!(chrono::Utc::now().to_rfc3339()),
            );
            state_accessor.set(&metadata_key, serde_json::to_value(&metadata)?);

            // Emit state change event if events are available
            let child_exec_context = context.to_execution_context();
            if let Some(ref events) = child_exec_context.events {
                if events.config().emit_state_events {
                    let _ = events
                        .emit(
                            "workflow.state.updated",
                            serde_json::json!({
                                "workflow_id": workflow_id,
                                "keys": [output_key, metadata_key],
                                "operation": "write",
                                "component": workflow_name,
                            }),
                        )
                        .await;
                }
            }
        }

        Ok(output.text)
    }

    /// Mock execution fallback for workflows (used when no registry is available)
    async fn execute_workflow_step_mock(
        &self,
        workflow_id: ComponentId,
        input: &serde_json::Value,
    ) -> Result<String> {
        // Mock implementation for backward compatibility
        let output = format!(
            "Nested workflow executed: {} with input: {}",
            workflow_id, input
        );

        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(output)
    }

    /// Create a StepContext for hooks
    fn create_step_context(
        &self,
        step: &WorkflowStep,
        context: &StepExecutionContext,
        error: Option<String>,
    ) -> StepContext {
        let step_type = match &step.step_type {
            StepType::Tool { .. } => "tool",
            StepType::Agent { .. } => "agent",
            StepType::Workflow { .. } => "workflow",
        };

        StepContext {
            name: step.name.clone(),
            index: context.workflow_state.current_step,
            step_type: step_type.to_string(),
            input: Some(serde_json::to_value(&step.step_type).unwrap_or(serde_json::Value::Null)),
            output: error.map(serde_json::Value::String),
            duration_ms: None,
        }
    }

    /// Create a StepContext with result for post-execution hooks
    fn create_step_context_with_result(
        &self,
        step: &WorkflowStep,
        context: &StepExecutionContext,
        result: &StepResult,
    ) -> StepContext {
        let step_type = match &step.step_type {
            StepType::Tool { .. } => "tool",
            StepType::Agent { .. } => "agent",
            StepType::Workflow { .. } => "workflow",
        };

        StepContext {
            name: step.name.clone(),
            index: context.workflow_state.current_step,
            step_type: step_type.to_string(),
            input: Some(serde_json::to_value(&step.step_type).unwrap_or(serde_json::Value::Null)),
            output: Some(serde_json::Value::String(result.output.clone())),
            duration_ms: Some({
                #[allow(clippy::cast_possible_truncation)]
                let duration_ms_u64 = result.duration.as_millis() as u64;
                duration_ms_u64
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WorkflowState;
    #[tokio::test]
    async fn test_step_executor_tool_execution() {
        let config = WorkflowConfig::default();
        let executor = StepExecutor::new(config);

        let step = WorkflowStep::new(
            "calculator_test".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let context = StepExecutionContext::new(WorkflowState::new(), None);
        let result = executor.execute_step(&step, context).await.unwrap();

        assert!(result.success);
        assert!(result.output.contains("Calculator result"));
        assert_eq!(result.retry_count, 0);
    }
    #[tokio::test]
    async fn test_step_executor_agent_execution() {
        let config = WorkflowConfig::default();
        let executor = StepExecutor::new(config);

        let agent_id = "test_agent".to_string();
        let step = WorkflowStep::new(
            "agent_test".to_string(),
            StepType::Agent {
                agent_id,
                input: "Process this data".to_string(),
            },
        );

        let context = StepExecutionContext::new(WorkflowState::new(), None);
        let result = executor.execute_step(&step, context).await.unwrap();

        assert!(result.success);
        assert!(result.output.contains("Agent"));
        assert!(result.output.contains("processed"));
    }
    #[tokio::test]
    async fn test_step_executor_with_retry() {
        let config = WorkflowConfig {
            exponential_backoff: false, // Use fixed delay for faster test
            ..Default::default()
        };
        let executor = StepExecutor::new(config);

        // Create a step that will fail (empty tool name)
        let step = WorkflowStep::new(
            "failing_test".to_string(),
            StepType::Tool {
                tool_name: "".to_string(), // This will cause failure
                parameters: serde_json::json!({}),
            },
        );

        let context = StepExecutionContext::new(WorkflowState::new(), None);
        let error_strategy = ErrorStrategy::Retry {
            max_attempts: 3,
            backoff_ms: 10, // Short delay for test
        };

        let result = executor
            .execute_step_with_retry(&step, context, &error_strategy)
            .await
            .unwrap();

        assert!(!result.success);
        assert_eq!(result.retry_count, 3);
        assert!(result.error.is_some());
    }
    #[tokio::test]
    async fn test_step_executor_timeout() {
        let config = WorkflowConfig::default();
        let executor = StepExecutor::new(config);

        let step = WorkflowStep::new(
            "timeout_test".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"operation": "add", "values": [1, 1]}),
            },
        )
        .with_timeout(Duration::from_millis(1)); // Very short timeout

        let context = StepExecutionContext::new(WorkflowState::new(), None);
        let result = executor.execute_step(&step, context).await.unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("timed out"));
    }
}
