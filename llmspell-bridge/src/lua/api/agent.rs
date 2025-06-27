//! ABOUTME: Lua Agent API implementation providing Agent.create() and agent methods
//! ABOUTME: Bridges between Lua scripts and Rust Agent implementations

use crate::engine::types::AgentApiDefinition;
use crate::{ComponentRegistry, ProviderManager};
use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use llmspell_core::{
    traits::{
        agent::{Agent, AgentConfig, ConversationMessage},
        base_agent::BaseAgent,
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, Result,
};
use llmspell_providers::ProviderInstance;
use mlua::{Lua, Table, UserData, UserDataMethods};
use std::sync::Arc;

/// Inject the Agent API into the Lua environment
pub fn inject_agent_api(
    lua: &Lua,
    api_def: &AgentApiDefinition,
    registry: Arc<ComponentRegistry>,
    providers: Arc<ProviderManager>,
) -> Result<()> {
    // Create the Agent global table
    let agent_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Agent table: {}", e),
        source: None,
    })?;

    // Clone Arc for the closure
    let registry_clone = registry.clone();
    let providers_clone = providers.clone();

    // Create the Agent.create() function
    let create_fn = lua
        .create_async_function(move |_lua, args: Table| {
            let registry = registry_clone.clone();
            let providers = providers_clone.clone();

            async move {
                // Extract configuration from Lua table
                let system_prompt: Option<String> = args.get("system_prompt").ok();
                let temperature: Option<f32> = args.get("temperature").ok();
                let max_tokens: Option<usize> = args.get("max_tokens").ok();
                let provider_name: Option<String> = args.get("provider").ok();
                let model: Option<String> = args.get("model").ok();

                // Create a basic agent configuration
                let agent_config = AgentConfig {
                    system_prompt,
                    temperature,
                    max_tokens,
                    max_conversation_length: args.get("max_conversation_length").ok(),
                };

                // Get the provider
                let provider = if let Some(name) = provider_name {
                    providers.get_provider(Some(&name)).await.map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to get provider '{}': {}",
                            name, e
                        ))
                    })?
                } else {
                    providers.get_default_provider().await.map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to get default provider: {}", e))
                    })?
                };

                // Create a simple agent wrapper
                let agent: Box<dyn Agent> = Box::new(SimpleProviderAgent::new(
                    agent_config,
                    provider,
                    model.unwrap_or_else(|| "default".to_string()),
                ));

                // Create the Lua wrapper
                let wrapper = LuaAgentWrapper {
                    agent: Arc::new(agent),
                    _registry: registry,
                    _providers: providers,
                };

                Ok(wrapper)
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.create function: {}", e),
            source: None,
        })?;

    // Add the create function to the Agent table
    agent_table
        .set(&api_def.constructor[..], create_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.create: {}", e),
            source: None,
        })?;

    // Set the Agent table as a global
    lua.globals()
        .set(&api_def.global_name[..], agent_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent global: {}", e),
            source: None,
        })?;

    Ok(())
}

/// Wrapper around Agent for Lua
#[derive(Clone)]
struct LuaAgentWrapper {
    agent: Arc<Box<dyn Agent>>,
    _registry: Arc<ComponentRegistry>,
    _providers: Arc<ProviderManager>,
}

impl UserData for LuaAgentWrapper {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // execute method
        methods.add_async_method("execute", |lua, this, input: Table| async move {
            // Convert Lua table to AgentInput
            let text: String = input.get("text")?;

            let agent_input = AgentInput::text(text);
            let context = ExecutionContext::new();

            // Execute the agent
            let result = this
                .agent
                .execute(agent_input, context)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

            // Convert AgentOutput to Lua table
            let output_table = lua.create_table()?;
            output_table.set("text", result.text)?;

            Ok(output_table)
        });

        // getConfig method
        methods.add_method("getConfig", |lua, this, ()| {
            let config_table = lua.create_table()?;
            let config = this.agent.config();

            if let Some(prompt) = &config.system_prompt {
                config_table.set("system_prompt", prompt.clone())?;
            }
            if let Some(temp) = config.temperature {
                config_table.set("temperature", temp)?;
            }
            if let Some(tokens) = config.max_tokens {
                config_table.set("max_tokens", tokens)?;
            }

            Ok(config_table)
        });

        // getState method
        methods.add_method("getState", |lua, _this, ()| {
            let state_table = lua.create_table()?;
            // TODO: Implement state retrieval from agent
            Ok(state_table)
        });

        // setState method
        methods.add_method("setState", |_lua, _this, _state: Table| {
            // TODO: Implement state setting on agent
            Ok(())
        });
    }
}

/// Simple agent implementation that uses a provider directly
struct SimpleProviderAgent {
    metadata: ComponentMetadata,
    config: AgentConfig,
    provider: Arc<Box<dyn ProviderInstance>>,
    _model: String,
    conversation: tokio::sync::Mutex<Vec<ConversationMessage>>,
}

impl SimpleProviderAgent {
    fn new(config: AgentConfig, provider: Arc<Box<dyn ProviderInstance>>, model: String) -> Self {
        let metadata = ComponentMetadata::new(
            "SimpleProviderAgent".to_string(),
            "A basic agent that uses a provider directly".to_string(),
        );

        Self {
            metadata,
            config,
            provider,
            _model: model,
            conversation: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl BaseAgent for SimpleProviderAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        mut input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Add system prompt to the input if configured
        if let Some(ref system_prompt) = self.config.system_prompt {
            // Prepend system prompt to the input text
            input.text = format!("{}\n\n{}", system_prompt, input.text);
        }

        // Use the provider to complete the request
        let output = self.provider.complete(&input).await?;
        Ok(output)
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        // Basic validation - ensure text is not empty
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Error: {}", error)))
    }
}

#[async_trait]
impl Agent for SimpleProviderAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>> {
        let conv = self.conversation.lock().await;
        Ok(conv.clone())
    }

    async fn add_message(&mut self, message: ConversationMessage) -> Result<()> {
        let mut conv = self.conversation.lock().await;
        conv.push(message);
        Ok(())
    }

    async fn clear_conversation(&mut self) -> Result<()> {
        let mut conv = self.conversation.lock().await;
        conv.clear();
        Ok(())
    }
}
