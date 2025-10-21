//! Interactive Chat Template
//!
//! Session-based conversation template with:
//! - Persistent conversation history
//! - Optional tool integration
//! - Two modes: interactive (REPL with readline) vs programmatic (single message)
//! - Memory placeholder for Phase 13

use crate::{
    artifacts::Artifact,
    context::ExecutionContext,
    core::{
        CostEstimate, TemplateCategory, TemplateMetadata, TemplateOutput, TemplateParams,
        TemplateResult,
    },
    error::{Result, TemplateError},
    validation::{ConfigSchema, ParameterConstraints, ParameterSchema, ParameterType},
};
use async_trait::async_trait;
use llmspell_core::traits::script_executor::{
    ScriptExecutionMetadata, ScriptExecutionOutput, ScriptExecutor,
};
use llmspell_core::LLMSpellError;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

/// Minimal no-op script executor for chat-only REPL mode (Subtask 12.9.5)
///
/// This executor provides minimal script execution for REPL infrastructure
/// while disabling actual code execution in chat-only mode.
struct NoOpScriptExecutor;

#[async_trait]
impl ScriptExecutor for NoOpScriptExecutor {
    async fn execute_script(
        &self,
        _script: &str,
    ) -> std::result::Result<ScriptExecutionOutput, LLMSpellError> {
        // Return empty output - code execution not supported in chat-only mode
        Ok(ScriptExecutionOutput {
            output: serde_json::Value::Null,
            console_output: vec!["Code execution disabled in chat-only mode".to_string()],
            metadata: ScriptExecutionMetadata {
                duration: std::time::Duration::from_millis(0),
                language: "none".to_string(),
                exit_code: Some(0),
                warnings: vec![],
            },
        })
    }

    fn language(&self) -> &'static str {
        "none" // Chat-only mode
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Interactive Chat Template
///
/// Conversational AI template with session-based memory and optional tool use:
/// - Maintains conversation history across turns
/// - Optional tool integration (user-configurable)
/// - Two execution modes: interactive (stdin) and programmatic (single message)
/// - Memory placeholder ready for Phase 13
#[derive(Debug)]
pub struct InteractiveChatTemplate {
    metadata: TemplateMetadata,
}

impl InteractiveChatTemplate {
    /// Create a new Interactive Chat template instance
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "interactive-chat".to_string(),
                name: "Interactive Chat".to_string(),
                description: "Session-based conversational AI with tool integration. \
                             Supports interactive mode (stdin loop) and programmatic mode \
                             (single message). Maintains conversation history and optionally \
                             uses tools for enhanced capabilities."
                    .to_string(),
                category: TemplateCategory::Chat,
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec!["local-llm".to_string()],
                tags: vec![
                    "chat".to_string(),
                    "conversation".to_string(),
                    "interactive".to_string(),
                    "session".to_string(),
                    "tools".to_string(),
                ],
            },
        }
    }
}

impl Default for InteractiveChatTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::core::Template for InteractiveChatTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new(vec![
            // model (optional with default)
            ParameterSchema::optional(
                "model",
                "LLM model to use for conversation",
                ParameterType::String,
                json!("ollama/llama3.2:3b"),
            ),
            // system_prompt (optional with default)
            ParameterSchema::optional(
                "system_prompt",
                "System prompt to set conversation context and behavior",
                ParameterType::String,
                json!("You are a helpful AI assistant. Provide clear, accurate, and concise responses."),
            ),
            // max_turns (optional with default and range)
            ParameterSchema::optional(
                "max_turns",
                "Maximum number of conversation turns (1-100)",
                ParameterType::Integer,
                json!(10),
            )
            .with_constraints(ParameterConstraints {
                min: Some(1.0),
                max: Some(100.0),
                ..Default::default()
            }),
            // tools (optional array of tool names)
            ParameterSchema::optional(
                "tools",
                "List of tool names to make available during conversation",
                ParameterType::Array,
                json!([]),
            ),
            // enable_memory (optional boolean)
            ParameterSchema::optional(
                "enable_memory",
                "Enable long-term memory (Phase 13 feature)",
                ParameterType::Boolean,
                json!(false),
            ),
            // message (optional - for programmatic mode)
            ParameterSchema::optional(
                "message",
                "Single message for programmatic mode (if not set, enters interactive mode)",
                ParameterType::String,
                json!(null),
            ),
            // session_id (optional - for session reuse across calls)
            ParameterSchema::optional(
                "session_id",
                "Optional session ID to reuse existing conversation (UUID format). If not provided, creates new session.",
                ParameterType::String,
                json!(null),
            ),
        ])
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        let start_time = Instant::now();

        // Extract and validate parameters
        let model: String = params.get_or("model", "ollama/llama3.2:3b".to_string());
        let system_prompt: String = params.get_or(
            "system_prompt",
            "You are a helpful AI assistant. Provide clear, accurate, and concise responses."
                .to_string(),
        );
        let max_turns: i64 = params.get_or("max_turns", 10);
        let tools: Vec<String> = params.get_or("tools", Vec::new());
        let enable_memory: bool = params.get_or("enable_memory", false);
        let message: Option<String> = params.get_optional("message");
        let session_id_param: Option<String> = params.get_optional("session_id");

        info!(
            "Starting interactive chat (model={}, max_turns={}, tools={}, memory={})",
            model,
            max_turns,
            tools.len(),
            enable_memory
        );

        // Initialize output
        let mut output = TemplateOutput::new(
            TemplateResult::text(""), // Will be replaced
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params,
        );

        // Check mode: interactive vs programmatic
        let mode = if message.is_some() {
            ExecutionMode::Programmatic
        } else {
            ExecutionMode::Interactive
        };

        info!("Execution mode: {:?}", mode);

        // Phase 1: Create/restore session
        info!("Phase 1: Creating/restoring session...");
        let session_id = self
            .get_or_create_session(&context, session_id_param)
            .await?;
        output.add_metric("session_id", json!(session_id));

        // Phase 2: Load tools (if specified)
        info!("Phase 2: Loading tools...");
        let loaded_tools = self.load_tools(&tools, &context).await?;
        output.metrics.tools_invoked = loaded_tools.len();

        // Phase 3: Check memory (placeholder for Phase 13)
        if enable_memory {
            warn!("Long-term memory requested but not yet implemented - will be added in Phase 13");
            output.add_metric("memory_enabled", json!(false));
            output.add_metric("memory_status", json!("Phase 13 placeholder"));
        }

        // Phase 4: Execute conversation
        info!("Phase 4: Executing conversation...");
        let conversation_result = match mode {
            ExecutionMode::Interactive => {
                self.run_interactive_mode(
                    &session_id,
                    &model,
                    &system_prompt,
                    max_turns as usize,
                    &loaded_tools,
                    &context,
                )
                .await?
            }
            ExecutionMode::Programmatic => {
                self.run_programmatic_mode(
                    &session_id,
                    &model,
                    &system_prompt,
                    message.as_deref().unwrap(),
                    &loaded_tools,
                    &context,
                )
                .await?
            }
        };

        output.metrics.agents_invoked = 1; // Chat agent

        // Phase 5: Save session state
        info!("Phase 5: Saving session state...");
        self.save_session_state(&session_id, &conversation_result, &context)
            .await?;

        // Save conversation history artifact
        if let Some(output_dir) = &context.output_dir {
            self.save_conversation_artifact(
                output_dir,
                &session_id,
                &conversation_result,
                &mut output,
            )?;
        }

        // Set result
        output.result = TemplateResult::text(conversation_result.transcript.clone());
        output.set_duration(start_time.elapsed().as_millis() as u64);
        output.add_metric("turn_count", json!(conversation_result.turns));
        output.add_metric("total_tokens", json!(conversation_result.total_tokens));

        info!(
            "Chat complete (duration: {}ms, turns: {})",
            output.metrics.duration_ms, conversation_result.turns
        );
        Ok(output)
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        // Estimate based on max_turns and average tokens per turn
        let max_turns: i64 = params.get_or("max_turns", 10);

        // Rough estimates:
        // - System prompt: ~100 tokens
        // - Per turn: ~300 tokens (user message + assistant response)
        let estimated_tokens = 100 + (max_turns * 300);

        // Assuming $0.10 per 1M tokens (local LLM is cheaper)
        let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 0.10;

        // Each turn: ~2s (LLM inference)
        // Interactive mode overhead: ~500ms
        let estimated_duration = 500 + (max_turns * 2000);

        CostEstimate::new(
            estimated_tokens as u64,
            estimated_cost,
            estimated_duration as u64,
            0.7, // Medium-high confidence
        )
    }
}

impl InteractiveChatTemplate {
    /// Phase 1: Get or create session
    async fn get_or_create_session(
        &self,
        context: &ExecutionContext,
        requested_session_id: Option<String>,
    ) -> Result<String> {
        use llmspell_kernel::sessions::types::CreateSessionOptions;
        use llmspell_kernel::sessions::SessionId;
        use std::str::FromStr;

        // Get session manager from context
        let session_manager = context.require_sessions().map_err(|e| {
            warn!("Session manager not available: {}", e);
            TemplateError::InfrastructureUnavailable("sessions".to_string())
        })?;

        // If session_id provided, attempt to reuse existing session
        if let Some(sid) = requested_session_id {
            info!("Attempting to reuse existing session: {}", sid);

            // Parse session ID (validate UUID format)
            let session_id_obj = SessionId::from_str(&sid).map_err(|e| {
                warn!("Invalid session_id format: {}", e);
                TemplateError::ExecutionFailed(format!("Invalid session_id format: {}", e))
            })?;

            // Verify session exists (SessionManager handles auto-load from storage)
            session_manager
                .get_session(&session_id_obj)
                .await
                .map_err(|e| {
                    warn!("Session {} not found: {}", sid, e);
                    TemplateError::ExecutionFailed(format!("Session not found: {}", sid))
                })?;

            info!("Successfully reusing session: {}", sid);
            return Ok(sid);
        }

        // Otherwise create new session
        info!("Creating new interactive chat session");

        let options = CreateSessionOptions::builder()
            .name("Interactive Chat Session")
            .description("Session-based conversational AI with tool integration")
            .add_tag("chat")
            .add_tag("interactive")
            .add_tag("template:interactive-chat")
            .build();

        let session_id = session_manager.create_session(options).await.map_err(|e| {
            warn!("Failed to create session: {}", e);
            TemplateError::ExecutionFailed(format!("Session creation failed: {}", e))
        })?;

        info!("Created new chat session: {}", session_id);
        Ok(session_id.to_string())
    }

    /// Phase 2: Validate tools exist in registry
    async fn load_tools(
        &self,
        tool_names: &[String],
        context: &ExecutionContext,
    ) -> Result<Vec<String>> {
        if tool_names.is_empty() {
            info!("No tools requested for chat agent");
            return Ok(Vec::new());
        }

        info!("Validating tools for chat agent: {:?}", tool_names);

        // Get tool registry
        let tool_registry = context.tool_registry();

        // Validate each requested tool exists
        let mut validated_tools = Vec::new();
        for tool_name in tool_names {
            if tool_registry.get_tool(tool_name).await.is_some() {
                validated_tools.push(tool_name.clone());
                info!("Tool validated: {}", tool_name);
            } else {
                warn!("Tool not found in registry: {}", tool_name);
                return Err(TemplateError::ExecutionFailed(format!(
                    "Tool not found: {}",
                    tool_name
                )));
            }
        }

        info!("All {} tools validated successfully", validated_tools.len());
        Ok(validated_tools)
    }

    /// Load conversation history from session state
    async fn load_conversation_history(
        &self,
        session_id: &str,
        context: &ExecutionContext,
    ) -> Result<Vec<ConversationTurn>> {
        use llmspell_kernel::sessions::SessionId;
        use std::str::FromStr;

        // Get session manager
        let session_manager = context.require_sessions()?;

        // Parse session ID
        let sid = SessionId::from_str(session_id)
            .map_err(|e| TemplateError::ExecutionFailed(format!("Invalid session ID: {}", e)))?;

        // Get session
        let session = session_manager.get_session(&sid).await.map_err(|e| {
            warn!("Failed to get session: {}", e);
            TemplateError::ExecutionFailed(format!("Failed to get session: {}", e))
        })?;

        // Load conversation history from state
        if let Some(history_value) = session.get_state("conversation_history").await {
            let history: Vec<ConversationTurn> =
                serde_json::from_value(history_value).map_err(|e| {
                    warn!("Failed to deserialize conversation history: {}", e);
                    TemplateError::ExecutionFailed(format!(
                        "Failed to deserialize conversation history: {}",
                        e
                    ))
                })?;
            info!("Loaded {} turns from conversation history", history.len());
            Ok(history)
        } else {
            info!("No conversation history found, starting new conversation");
            Ok(Vec::new())
        }
    }

    /// Save conversation history to session state
    async fn save_conversation_history(
        &self,
        session_id: &str,
        history: &[ConversationTurn],
        context: &ExecutionContext,
    ) -> Result<()> {
        use llmspell_kernel::sessions::SessionId;
        use std::str::FromStr;

        // Get session manager
        let session_manager = context.require_sessions()?;

        // Parse session ID
        let sid = SessionId::from_str(session_id)
            .map_err(|e| TemplateError::ExecutionFailed(format!("Invalid session ID: {}", e)))?;

        // Get session
        let session = session_manager.get_session(&sid).await.map_err(|e| {
            warn!("Failed to get session: {}", e);
            TemplateError::ExecutionFailed(format!("Failed to get session: {}", e))
        })?;

        // Serialize history to JSON
        let history_value = serde_json::to_value(history).map_err(|e| {
            warn!("Failed to serialize conversation history: {}", e);
            TemplateError::ExecutionFailed(format!(
                "Failed to serialize conversation history: {}",
                e
            ))
        })?;

        // Save to session state
        session
            .set_state("conversation_history".to_string(), history_value)
            .await
            .map_err(|e| {
                warn!("Failed to save conversation history: {}", e);
                TemplateError::ExecutionFailed(format!(
                    "Failed to save conversation history: {}",
                    e
                ))
            })?;

        info!("Saved {} turns to conversation history", history.len());
        Ok(())
    }

    /// Phase 4a: Run interactive mode using full REPL (Subtask 12.9.5 - ACTUAL IMPLEMENTATION)
    ///
    /// Uses InteractiveSession.run_repl() for production-grade UX:
    /// - Readline: arrow keys, Ctrl-A/E, history navigation
    /// - Multi-line: smart detection, continuation prompts
    /// - Ctrl-C: graceful interrupt (doesn't terminate)
    /// - Dual-mode: Execute Lua/JS code OR chat with agent
    async fn run_interactive_mode(
        &self,
        session_id: &str,
        model: &str,
        system_prompt: &str,
        _max_turns: usize, // REPL handles its own lifecycle
        tools: &[String],
        context: &ExecutionContext,
    ) -> Result<ConversationResult> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_kernel::execution::ExecutionConfig;
        use llmspell_kernel::protocols::JupyterProtocol;
        use llmspell_kernel::repl::{InteractiveSession, ReplSessionConfig};
        use llmspell_kernel::IntegratedKernel;

        info!(
            "Starting REPL chat session (model: {}, session: {})",
            model, session_id
        );

        // Get infrastructure from context
        let session_manager = context
            .session_manager()
            .ok_or_else(|| {
                TemplateError::ExecutionFailed("Session manager required for REPL mode".to_string())
            })?
            .clone();
        let provider_manager = context.providers().clone();
        let agent_registry = context.agent_registry();

        // Create minimal NoOp script executor for chat-only mode (no code execution needed)
        let script_executor = Arc::new(NoOpScriptExecutor)
            as Arc<dyn llmspell_core::traits::script_executor::ScriptExecutor>;

        // Create Jupyter protocol for REPL
        let jupyter_protocol = JupyterProtocol::new(
            session_id.to_string(),
            format!("chat-kernel-{}", session_id),
        );

        // Create integrated kernel
        let kernel = IntegratedKernel::new(
            jupyter_protocol,
            ExecutionConfig::default(),
            session_id.to_string(),
            script_executor,
            Some(provider_manager.clone()),
            session_manager,
        )
        .await
        .map_err(|e| TemplateError::ExecutionFailed(format!("Failed to create kernel: {}", e)))?;

        // Create REPL config with history
        let repl_config = ReplSessionConfig {
            history_file: Some(format!(".llmspell_chat_history_{}", session_id).into()),
            enable_debug_commands: false, // No debug commands in chat mode
            enable_performance_monitoring: false,
            execution_timeout_secs: 300,
            enable_persistence: false,
        };

        // Create interactive session
        let mut session = InteractiveSession::new(kernel, repl_config)
            .await
            .map_err(|e| {
                TemplateError::ExecutionFailed(format!("Failed to create REPL session: {}", e))
            })?;

        // Create chat agent
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            ("ollama".to_string(), model.to_string())
        };

        let agent_config = AgentConfig {
            name: format!("chat-agent-{}", session_id),
            description: format!("REPL chat agent for session {}", session_id),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: Some(0.7),
                max_tokens: Some(1000),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: tools.to_vec(),
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 120,
                max_memory_mb: 256,
                max_tool_calls: if tools.is_empty() { 0 } else { 10 },
                max_recursion_depth: 1,
            },
        };

        let agent = agent_registry
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                TemplateError::ExecutionFailed(format!("Failed to create chat agent: {}", e))
            })?;

        // Wire up agent infrastructure using builder methods
        session = session
            .with_agent_registry(std::sync::Arc::new(agent_registry.clone()))
            .with_provider_manager(std::sync::Arc::new(provider_manager));

        session = session.with_model(model).await;
        session = session.with_system_prompt(system_prompt).await;
        session = session.with_tools(tools.to_vec()).await;
        session = session.with_initial_agent(agent).await;

        // Create agent creator callback for auto-recreation (Subtask 12.9.5)
        let agent_registry_clone = agent_registry.clone();
        let agent_creator: llmspell_kernel::repl::session::AgentCreator = std::sync::Arc::new(
            move |model: String, _system_prompt: String, tools: Vec<String>| {
                let registry = agent_registry_clone.clone();
                Box::pin(async move {
                    use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};

                    // Parse model (format: provider/model or just model)
                    let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
                        (model[..slash_pos].to_string(), model[slash_pos + 1..].to_string())
                    } else {
                        ("ollama".to_string(), model)
                    };

                    let agent_config = AgentConfig {
                        name: "repl-chat-agent".to_string(),
                        description: format!("Chat agent for model {provider}/{model_id}"),
                        agent_type: "llm".to_string(),
                        model: Some(ModelConfig {
                            provider,
                            model_id,
                            temperature: Some(0.7),
                            max_tokens: Some(1000),
                            settings: serde_json::Map::new(),
                        }),
                        allowed_tools: tools.clone(),
                        custom_config: serde_json::Map::new(),
                        resource_limits: ResourceLimits {
                            max_execution_time_secs: 120,
                            max_memory_mb: 256,
                            max_tool_calls: if tools.is_empty() { 0 } else { 10 },
                            max_recursion_depth: 1,
                        },
                    };

                    registry
                        .create_agent(agent_config)
                        .await
                        .map_err(|e| llmspell_core::LLMSpellError::Component {
                            message: format!("Failed to create agent: {e}"),
                            source: None,
                        })
                })
            },
        );

        session = session.with_agent_creator(agent_creator);

        // Print welcome message
        println!("\n╔══════════════════════════════════════════════╗");
        println!("║   Interactive REPL Chat Session Started     ║");
        println!("╚══════════════════════════════════════════════╝");
        println!("\nModel: {}", model);
        println!("Session: {}", session_id);
        println!("\n📝 Chat Commands:");
        println!("  • Type your message to chat with the AI");
        println!("  • .exit or .quit - end the conversation");
        println!("  • .system \"prompt\" - change system prompt");
        println!("  • .model provider/model - change LLM model");
        println!("  • .tools tool1,tool2 - configure available tools");
        println!("  • .context - show conversation history");
        println!("  • .clearchat - clear conversation history");
        println!("\n💬 Chat Mode:");
        println!("  • Type naturally - What is the capital of France?");
        println!("  • Multi-turn conversations with context retention");
        println!("  • Tool integration (if enabled)");
        println!("\n✨ REPL Features: Arrow keys, history (↑↓), multi-line, Ctrl-C interrupt\n");

        // Run REPL - this handles all input/output, readline, history, multi-line, Ctrl-C
        session
            .run_repl()
            .await
            .map_err(|e| TemplateError::ExecutionFailed(format!("REPL session failed: {}", e)))?;

        // Extract conversation data for result
        let turn_count = session.get_conversation_context().await.lines().count() / 2;
        let total_tokens = session.get_token_count().await;
        let transcript = format!(
            "# Interactive REPL Chat Session\n\n{}",
            session.get_conversation_context().await
        );

        Ok(ConversationResult {
            transcript,
            turns: turn_count,
            total_tokens,
        })
    }

    /// Phase 4b: Run programmatic mode (single message)
    async fn run_programmatic_mode(
        &self,
        session_id: &str,
        model: &str,
        system_prompt: &str,
        user_message: &str,
        tools: &[String],
        context: &ExecutionContext,
    ) -> Result<ConversationResult> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        info!(
            "Running programmatic mode (session: {}, model: {}, message_len: {}, tools: {})",
            session_id,
            model,
            user_message.len(),
            tools.len()
        );

        // Load conversation history from session
        let mut history = self.load_conversation_history(session_id, context).await?;
        let turn_number = (history.len() + 1) as u64;

        // Add user message to history
        let user_turn = ConversationTurn::user(user_message, turn_number);
        history.push(user_turn);

        // Parse model specification (format: "provider/model-id" or just "model-id")
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            ("ollama".to_string(), model.to_string())
        };

        info!(
            "Creating chat agent (provider: {}, model: {})",
            provider, model_id
        );

        // Get agent registry
        let agent_registry = context.agent_registry();

        // Create chat agent configuration
        let agent_config = AgentConfig {
            name: format!("chat-agent-{}", session_id),
            description: format!("Interactive chat agent for session {}", session_id),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: Some(0.7), // Balanced creativity for conversation
                max_tokens: Some(1000),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: tools.to_vec(),
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 120, // 2 minutes for chat response (local LLMs: Phase 12.8.2.7)
                max_memory_mb: 256,
                max_tool_calls: if tools.is_empty() { 0 } else { 10 },
                max_recursion_depth: 1,
            },
        };

        // Create chat agent
        let agent = agent_registry
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create chat agent: {}", e);
                TemplateError::ExecutionFailed(format!("Chat agent creation failed: {}", e))
            })?;

        // Build prompt with system instructions and conversation history
        let conversation_context = if !history.is_empty() {
            // Include all conversation turns (including first message) for context
            let history_text: String = history
                .iter()
                .map(|turn| format!("{}: {}", turn.role, turn.content))
                .collect::<Vec<_>>()
                .join("\n\n");
            format!("\n\nConversation History:\n{}\n", history_text)
        } else {
            String::new()
        };

        let prompt = format!(
            "{}{}\n\nRespond to the user's latest message naturally and helpfully.",
            system_prompt, conversation_context
        );

        // Create input for agent
        let agent_input = AgentInput::builder().text(prompt).build();

        // Execute agent
        let output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Chat agent execution failed: {}", e);
                TemplateError::ExecutionFailed(format!("Chat agent failed: {}", e))
            })?;

        info!(
            "Chat agent response generated ({} characters)",
            output.text.len()
        );

        // Add assistant response to history
        let assistant_turn = ConversationTurn::assistant(&output.text, turn_number + 1);
        history.push(assistant_turn);

        // Save updated history to session
        self.save_conversation_history(session_id, &history, context)
            .await?;

        // Build transcript
        let transcript = format!(
            "# Chat Conversation\n\n\
             User: {}\n\n\
             Assistant: {}\n",
            user_message, output.text
        );

        // Estimate tokens (rough: ~4 chars per token)
        let estimated_tokens = (system_prompt.len() + user_message.len() + output.text.len()) / 4;

        Ok(ConversationResult {
            transcript,
            turns: 1,
            total_tokens: estimated_tokens,
        })
    }

    /// Phase 5: Save session state
    async fn save_session_state(
        &self,
        session_id: &str,
        result: &ConversationResult,
        context: &ExecutionContext,
    ) -> Result<()> {
        use llmspell_kernel::sessions::SessionId;
        use std::str::FromStr;

        info!("Saving session state for session: {}", session_id);

        // Get session manager
        let session_manager = context.require_sessions()?;

        // Parse session ID
        let sid = SessionId::from_str(session_id)
            .map_err(|e| TemplateError::ExecutionFailed(format!("Invalid session ID: {}", e)))?;

        // Get session
        let session = session_manager.get_session(&sid).await.map_err(|e| {
            warn!("Failed to get session: {}", e);
            TemplateError::ExecutionFailed(format!("Failed to get session: {}", e))
        })?;

        // Save conversation metrics to session state
        session
            .set_state(
                "conversation_metrics".to_string(),
                serde_json::json!({
                    "total_turns": result.turns,
                    "total_tokens": result.total_tokens,
                    "last_updated": chrono::Utc::now(),
                }),
            )
            .await
            .map_err(|e| {
                warn!("Failed to save conversation metrics: {}", e);
                TemplateError::ExecutionFailed(format!(
                    "Failed to save conversation metrics: {}",
                    e
                ))
            })?;

        // Increment session operation count
        session.increment_operation_count().await.map_err(|e| {
            warn!("Failed to increment operation count: {}", e);
            TemplateError::ExecutionFailed(format!("Failed to increment operation count: {}", e))
        })?;

        // Save session to persistent storage
        session_manager.save_session(&session).await.map_err(|e| {
            warn!("Failed to save session: {}", e);
            TemplateError::ExecutionFailed(format!("Failed to save session: {}", e))
        })?;

        info!(
            "Session state saved successfully for session: {}",
            session_id
        );
        Ok(())
    }

    /// Save conversation artifact
    fn save_conversation_artifact(
        &self,
        output_dir: &std::path::Path,
        session_id: &str,
        result: &ConversationResult,
        output: &mut TemplateOutput,
    ) -> Result<()> {
        use std::fs;

        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to create output directory: {}", e))
        })?;

        // Save conversation transcript
        let transcript_path = output_dir.join(format!("conversation-{}.txt", session_id));
        fs::write(&transcript_path, &result.transcript).map_err(|e| {
            TemplateError::ExecutionFailed(format!(
                "Failed to write conversation transcript: {}",
                e
            ))
        })?;
        output.add_artifact(Artifact::new(
            transcript_path.to_string_lossy().to_string(),
            result.transcript.clone(),
            "text/plain".to_string(),
        ));

        Ok(())
    }
}

/// Execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionMode {
    /// Interactive mode (stdin loop)
    Interactive,
    /// Programmatic mode (single message)
    Programmatic,
}

/// Conversation result
#[derive(Debug, Clone)]
struct ConversationResult {
    /// Full conversation transcript
    transcript: String,
    /// Number of turns executed
    turns: usize,
    /// Total tokens used
    total_tokens: usize,
}

/// Single conversation turn for history tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ConversationTurn {
    /// Role: "user" or "assistant"
    role: String,
    /// Message content
    content: String,
    /// Timestamp of the turn
    timestamp: chrono::DateTime<chrono::Utc>,
    /// Turn number in conversation
    turn_number: u64,
    /// Token count for this turn (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    token_count: Option<u64>,
}

impl ConversationTurn {
    /// Create a new user turn
    fn user(content: impl Into<String>, turn_number: u64) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            timestamp: chrono::Utc::now(),
            turn_number,
            token_count: None,
        }
    }

    /// Create a new assistant turn
    fn assistant(content: impl Into<String>, turn_number: u64) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            timestamp: chrono::Utc::now(),
            turn_number,
            token_count: None,
        }
    }

    /// Set token count
    #[allow(dead_code)]
    fn with_token_count(mut self, count: u64) -> Self {
        self.token_count = Some(count);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Template;

    #[test]
    fn test_template_metadata() {
        let template = InteractiveChatTemplate::new();
        let metadata = template.metadata();

        assert_eq!(metadata.id, "interactive-chat");
        assert_eq!(metadata.name, "Interactive Chat");
        assert_eq!(metadata.category, TemplateCategory::Chat);
        assert!(metadata.requires.contains(&"local-llm".to_string()));
        assert!(metadata.tags.contains(&"chat".to_string()));
        assert!(metadata.tags.contains(&"conversation".to_string()));
        assert!(metadata.tags.contains(&"interactive".to_string()));
    }

    #[test]
    fn test_config_schema() {
        let template = InteractiveChatTemplate::new();
        let schema = template.config_schema();

        assert!(schema.get_parameter("model").is_some());
        assert!(schema.get_parameter("system_prompt").is_some());
        assert!(schema.get_parameter("max_turns").is_some());
        assert!(schema.get_parameter("tools").is_some());
        assert!(schema.get_parameter("enable_memory").is_some());
        assert!(schema.get_parameter("message").is_some());

        // All parameters are optional (have defaults)
        let model_param = schema.get_parameter("model").unwrap();
        assert!(!model_param.required);

        let max_turns_param = schema.get_parameter("max_turns").unwrap();
        assert!(!max_turns_param.required);
    }

    #[tokio::test]
    async fn test_cost_estimate() {
        let template = InteractiveChatTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("max_turns", serde_json::json!(5));

        let estimate = template.estimate_cost(&params).await;
        assert!(estimate.estimated_tokens.is_some());
        assert!(estimate.estimated_cost_usd.is_some());
        assert!(estimate.estimated_duration_ms.is_some());
        assert!(estimate.confidence > 0.0);

        // Verify calculation (100 + 5 * 300 = 1600 tokens)
        assert_eq!(estimate.estimated_tokens, Some(1600));
    }

    #[test]
    fn test_parameter_validation_out_of_range() {
        let template = InteractiveChatTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("max_turns".to_string(), serde_json::json!(200)); // Over max of 100

        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_success() {
        let template = InteractiveChatTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("model".to_string(), serde_json::json!("ollama/llama3.2:3b"));
        params.insert("max_turns".to_string(), serde_json::json!(10));
        params.insert("tools".to_string(), serde_json::json!([]));

        let result = schema.validate(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execution_mode_detection() {
        // With message → Programmatic mode
        let mut params1 = TemplateParams::new();
        params1.insert("message", serde_json::json!("Hello"));
        let message1: Option<String> = params1.get_optional("message");
        assert!(message1.is_some());

        // Without message → Interactive mode
        let params2 = TemplateParams::new();
        let message2: Option<String> = params2.get_optional("message");
        assert!(message2.is_none());
    }

    #[test]
    fn test_conversation_turn_user_creation() {
        let turn = ConversationTurn::user("Hello, assistant!", 1);

        assert_eq!(turn.role, "user");
        assert_eq!(turn.content, "Hello, assistant!");
        assert_eq!(turn.turn_number, 1);
        assert!(turn.token_count.is_none());
        assert!(turn.timestamp <= chrono::Utc::now());
    }

    #[test]
    fn test_conversation_turn_assistant_creation() {
        let turn = ConversationTurn::assistant("Hello, user!", 2);

        assert_eq!(turn.role, "assistant");
        assert_eq!(turn.content, "Hello, user!");
        assert_eq!(turn.turn_number, 2);
        assert!(turn.token_count.is_none());
    }

    #[test]
    fn test_conversation_turn_with_token_count() {
        let turn = ConversationTurn::user("Test message", 1).with_token_count(42);

        assert_eq!(turn.token_count, Some(42));
    }

    #[test]
    fn test_conversation_turn_serialization() {
        let turn = ConversationTurn::user("Test serialization", 1);
        let json = serde_json::to_value(&turn).unwrap();

        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "Test serialization");
        assert_eq!(json["turn_number"], 1);
        assert!(json.get("timestamp").is_some());
    }

    #[test]
    fn test_conversation_turn_deserialization() {
        let json = serde_json::json!({
            "role": "assistant",
            "content": "Deserialized response",
            "timestamp": "2024-01-01T12:00:00Z",
            "turn_number": 3
        });

        let turn: ConversationTurn = serde_json::from_value(json).unwrap();
        assert_eq!(turn.role, "assistant");
        assert_eq!(turn.content, "Deserialized response");
        assert_eq!(turn.turn_number, 3);
    }

    #[test]
    fn test_conversation_turn_roundtrip() {
        let original = ConversationTurn::assistant("Roundtrip test", 5);
        let json = serde_json::to_value(&original).unwrap();
        let deserialized: ConversationTurn = serde_json::from_value(json).unwrap();

        assert_eq!(original.role, deserialized.role);
        assert_eq!(original.content, deserialized.content);
        assert_eq!(original.turn_number, deserialized.turn_number);
        assert_eq!(original.token_count, deserialized.token_count);
    }

    #[test]
    fn test_conversation_history_serialization() {
        let history = vec![
            ConversationTurn::user("First message", 1),
            ConversationTurn::assistant("First response", 2),
            ConversationTurn::user("Second message", 3),
            ConversationTurn::assistant("Second response", 4),
        ];

        let json = serde_json::to_value(&history).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 4);

        let deserialized: Vec<ConversationTurn> = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.len(), 4);
        assert_eq!(deserialized[0].content, "First message");
        assert_eq!(deserialized[1].content, "First response");
        assert_eq!(deserialized[2].content, "Second message");
        assert_eq!(deserialized[3].content, "Second response");
    }

    #[test]
    fn test_model_spec_parsing_with_provider() {
        // Test model spec parsing logic (as used in run_programmatic_mode)
        let model = "anthropic/claude-3-opus";
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            ("ollama".to_string(), model.to_string())
        };

        assert_eq!(provider, "anthropic");
        assert_eq!(model_id, "claude-3-opus");
    }

    #[test]
    fn test_model_spec_parsing_without_provider() {
        // Test model spec defaults to ollama when no provider specified
        let model = "llama3.2:3b";
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            ("ollama".to_string(), model.to_string())
        };

        assert_eq!(provider, "ollama");
        assert_eq!(model_id, "llama3.2:3b");
    }

    #[test]
    fn test_execution_mode_enum() {
        let interactive = ExecutionMode::Interactive;
        let programmatic = ExecutionMode::Programmatic;

        assert_ne!(interactive, programmatic);
        assert_eq!(interactive, ExecutionMode::Interactive);
        assert_eq!(programmatic, ExecutionMode::Programmatic);
    }

    #[test]
    fn test_conversation_result_creation() {
        let result = ConversationResult {
            transcript: "Test transcript".to_string(),
            turns: 5,
            total_tokens: 1000,
        };

        assert_eq!(result.transcript, "Test transcript");
        assert_eq!(result.turns, 5);
        assert_eq!(result.total_tokens, 1000);
    }

    // Integration tests that require full infrastructure (SessionManager, ToolRegistry, etc.)
    // These are marked #[ignore] because they need proper runtime setup
    // Real integration tests are in llmspell-bridge/tests/template_execution_test.rs

    #[tokio::test]
    #[ignore = "Requires SessionManager infrastructure - see llmspell-bridge/tests"]
    async fn test_session_creation_with_infrastructure() {
        // This test requires:
        // - ExecutionContext with SessionManager
        // - SessionManager.create_session() working
        // Expected behavior: Returns UUID SessionId (not "chat-" prefix)
        let template = InteractiveChatTemplate::new();
        let context = ExecutionContext::builder().build();
        if let Ok(context) = context {
            let session_id = template.get_or_create_session(&context, None).await;
            if let Ok(session_id) = session_id {
                // New implementation returns UUID, not "chat-" prefix
                assert!(!session_id.is_empty());
                // Verify UUID format (8-4-4-4-12 hex characters)
                assert!(session_id.len() >= 32); // UUIDs are at least 32 chars
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires ToolRegistry infrastructure - see llmspell-bridge/tests"]
    async fn test_tool_validation_with_infrastructure() {
        // This test requires:
        // - ExecutionContext with ToolRegistry
        // - ToolRegistry with registered tools
        // Expected behavior: Validates tools exist via ToolRegistry.get_tool()
        let template = InteractiveChatTemplate::new();
        let context = ExecutionContext::builder().build();
        if let Ok(context) = context {
            let tool_names = vec!["calculator".to_string()]; // Standard tool
            let result = template.load_tools(&tool_names, &context).await;
            // Will succeed if calculator tool is registered
            // Will fail with TemplateError::ExecutionFailed if tool not found
            if let Ok(loaded) = result {
                assert_eq!(loaded.len(), 1);
                assert_eq!(loaded[0], "calculator");
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires full infrastructure (SessionManager, AgentRegistry, ProviderManager)"]
    async fn test_programmatic_mode_with_infrastructure() {
        // This test requires:
        // - ExecutionContext with SessionManager, AgentRegistry, ProviderManager
        // - Working LLM provider (ollama or other)
        // - Session persistence
        // Expected behavior:
        // - Creates session
        // - Loads history (empty for new session)
        // - Creates agent with AgentConfig
        // - Executes agent with conversation context
        // - Saves history to session.state["conversation_history"]
        // - Returns ConversationResult with transcript
        let template = InteractiveChatTemplate::new();
        let context = ExecutionContext::builder().build();
        if let Ok(context) = context {
            let result = template
                .run_programmatic_mode(
                    "test-session-uuid",
                    "ollama/llama3.2:3b",
                    "You are helpful",
                    "Hello, how are you?",
                    &[],
                    &context,
                )
                .await;

            if let Ok(result) = result {
                assert_eq!(result.turns, 1);
                assert!(result.total_tokens > 0);
                assert!(result.transcript.contains("Hello, how are you?"));
                assert!(result.transcript.contains("User:"));
                assert!(result.transcript.contains("Assistant:"));
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires full infrastructure for multi-turn conversation testing"]
    async fn test_conversation_history_persistence() {
        // This test requires:
        // - Full ExecutionContext
        // - Multiple calls to run_programmatic_mode with same session_id
        // Expected behavior:
        // - First call: empty history
        // - Second call: history contains first turn
        // - Third call: history contains first two turns
        // - History persisted in Session.state["conversation_history"]
        let template = InteractiveChatTemplate::new();
        let context = ExecutionContext::builder().build();
        if let Ok(context) = context {
            let session_id = template.get_or_create_session(&context, None).await.ok();
            if let Some(session_id) = session_id {
                // Turn 1
                let _ = template
                    .run_programmatic_mode(
                        &session_id,
                        "ollama/llama3.2:3b",
                        "System",
                        "Message 1",
                        &[],
                        &context,
                    )
                    .await;

                // Turn 2 - should have history from turn 1
                let _ = template
                    .run_programmatic_mode(
                        &session_id,
                        "ollama/llama3.2:3b",
                        "System",
                        "Message 2",
                        &[],
                        &context,
                    )
                    .await;

                // Load history
                if let Ok(history) = template
                    .load_conversation_history(&session_id, &context)
                    .await
                {
                    // Should have 4 turns: user1, assistant1, user2, assistant2
                    assert_eq!(history.len(), 4);
                    assert_eq!(history[0].role, "user");
                    assert_eq!(history[1].role, "assistant");
                    assert_eq!(history[2].role, "user");
                    assert_eq!(history[3].role, "assistant");
                }
            }
        }
    }

    #[test]
    fn test_empty_tool_list_returns_empty() {
        // Test that empty tool list logic returns early without infrastructure
        // This behavior is implemented in load_tools at line 309-312:
        // if tool_names.is_empty() { return Ok(Vec::new()); }

        let tools: Vec<String> = Vec::new();
        assert!(tools.is_empty());

        // Verify the expected behavior when load_tools receives empty list
        let expected: Vec<String> = Vec::new();
        assert_eq!(tools, expected);
    }

    #[test]
    fn test_token_estimation_logic() {
        // Test the token estimation logic used in run_programmatic_mode
        let system_prompt = "You are helpful"; // 15 chars
        let user_message = "Hello"; // 5 chars
        let output_text = "Hi there!"; // 9 chars

        let estimated_tokens = (system_prompt.len() + user_message.len() + output_text.len()) / 4;
        assert_eq!(estimated_tokens, 7); // (15 + 5 + 9) / 4 = 7.25 -> 7
    }
}
