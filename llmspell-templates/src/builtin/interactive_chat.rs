//! Interactive Chat Template
//!
//! Session-based conversation template with:
//! - Persistent conversation history
//! - Optional tool integration
//! - Two modes: interactive (stdin loop) vs programmatic (single message)
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
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};

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
        let session_id = self.get_or_create_session(&context).await?;
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
    async fn get_or_create_session(&self, _context: &ExecutionContext) -> Result<String> {
        // TODO: Implement actual session management integration
        // For now, generate a new session ID
        warn!("Session management not yet implemented - generating new session ID");

        let session_id = format!("chat-{}", uuid::Uuid::new_v4());
        info!("Created session: {}", session_id);

        Ok(session_id)
    }

    /// Phase 2: Load tools from registry
    async fn load_tools(
        &self,
        tool_names: &[String],
        _context: &ExecutionContext,
    ) -> Result<Vec<String>> {
        // TODO: Implement actual tool loading from context.tool_registry
        // For now, return placeholder
        if !tool_names.is_empty() {
            warn!(
                "Tool integration not yet implemented - requested tools: {:?}",
                tool_names
            );
        }

        Ok(tool_names.to_vec())
    }

    /// Phase 4a: Run interactive mode (stdin loop)
    async fn run_interactive_mode(
        &self,
        _session_id: &str,
        _model: &str,
        _system_prompt: &str,
        max_turns: usize,
        _tools: &[String],
        _context: &ExecutionContext,
    ) -> Result<ConversationResult> {
        // TODO: Implement actual interactive stdin loop with agent
        // For now, return placeholder
        warn!("Interactive mode not yet fully implemented - using placeholder conversation");

        Ok(ConversationResult {
            transcript: format!(
                "# Interactive Chat Session\n\n\
                 [This is a placeholder for interactive mode]\n\n\
                 System: Ready for conversation (max {} turns)\n\
                 System: Type your message and press Enter\n\
                 System: Type 'exit' or 'quit' to end conversation\n\n\
                 [In production, this would show actual conversation history]\n",
                max_turns
            ),
            turns: 0,
            total_tokens: 0,
        })
    }

    /// Phase 4b: Run programmatic mode (single message)
    async fn run_programmatic_mode(
        &self,
        _session_id: &str,
        model: &str,
        system_prompt: &str,
        user_message: &str,
        _tools: &[String],
        _context: &ExecutionContext,
    ) -> Result<ConversationResult> {
        // TODO: Implement actual agent execution with LLM provider
        // For now, return placeholder response
        warn!("Programmatic mode not yet fully implemented - using placeholder response");

        let response = format!(
            "[Placeholder response from {}]\n\n\
             System Prompt: {}\n\n\
             User: {}\n\n\
             Assistant: This is a placeholder response. In production, this would be \
             an actual LLM-generated response based on the system prompt and user message.",
            model, system_prompt, user_message
        );

        // Estimate tokens (rough)
        let estimated_tokens = (system_prompt.len() + user_message.len() + response.len()) / 4;

        Ok(ConversationResult {
            transcript: response,
            turns: 1,
            total_tokens: estimated_tokens,
        })
    }

    /// Phase 5: Save session state
    async fn save_session_state(
        &self,
        session_id: &str,
        _result: &ConversationResult,
        _context: &ExecutionContext,
    ) -> Result<()> {
        // TODO: Implement actual session state persistence
        // For now, just log
        warn!(
            "Session state persistence not yet implemented - session: {}",
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

    #[tokio::test]
    async fn test_get_or_create_session_placeholder() {
        let template = InteractiveChatTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available
            return;
        }
        let context = context.unwrap();

        let session_id = template.get_or_create_session(&context).await;
        assert!(session_id.is_ok());
        let session_id = session_id.unwrap();
        assert!(session_id.starts_with("chat-"));
    }

    #[tokio::test]
    async fn test_load_tools_placeholder() {
        let template = InteractiveChatTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available
            return;
        }
        let context = context.unwrap();

        let tool_names = vec!["search".to_string(), "calculator".to_string()];
        let loaded = template.load_tools(&tool_names, &context).await;
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[tokio::test]
    async fn test_programmatic_mode_placeholder() {
        let template = InteractiveChatTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available
            return;
        }
        let context = context.unwrap();

        let result = template
            .run_programmatic_mode(
                "test-session",
                "ollama/llama3.2:3b",
                "You are helpful",
                "Hello, how are you?",
                &[],
                &context,
            )
            .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.turns, 1);
        assert!(result.total_tokens > 0);
        assert!(result.transcript.contains("Hello, how are you?"));
    }
}
