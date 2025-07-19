//! ABOUTME: Lua Agent API implementation providing Agent.create() and agent methods
//! ABOUTME: Bridges between Lua scripts and Rust Agent implementations

use crate::agent_bridge::AgentBridge;
use crate::agent_conversion::{
    agent_output_to_lua_table, lua_table_to_agent_input, lua_value_to_json,
};
use crate::engine::types::AgentApiDefinition;
use crate::{ComponentRegistry, ProviderManager};
use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use llmspell_core::{
    traits::{
        agent::{Agent, AgentConfig, ConversationMessage},
        base_agent::BaseAgent,
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, Result,
};
use llmspell_providers::{ModelSpecifier, ProviderInstance};
use mlua::{Lua, Table, UserData, UserDataMethods};
use std::collections::HashMap;
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

    // Create agent bridge
    let bridge = Arc::new(AgentBridge::new(registry.clone()));

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
                let max_conversation_length: Option<usize> =
                    args.get("max_conversation_length").ok();
                let base_url: Option<String> = args.get("base_url").ok();
                let api_key: Option<String> = args.get("api_key").ok();

                // Create a basic agent configuration
                let agent_config = AgentConfig {
                    system_prompt,
                    temperature,
                    max_tokens,
                    max_conversation_length,
                };

                // Handle model specification with new syntax support
                let provider = if let Some(model_str) =
                    args.get::<_, Option<String>>("model").ok().flatten()
                {
                    // New syntax: "provider/model" or "model"
                    let model_spec = ModelSpecifier::parse(&model_str).map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Invalid model specification '{}': {}",
                            model_str, e
                        ))
                    })?;

                    providers
                        .as_ref()
                        .create_agent_from_spec(model_spec, base_url.as_deref(), api_key.as_deref())
                        .await
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to create agent from spec: {}",
                                e
                            ))
                        })?
                } else if let (Some(provider_name), Some(model_name)) = (
                    args.get::<_, Option<String>>("provider").ok().flatten(),
                    args.get::<_, Option<String>>("model_name").ok().flatten(),
                ) {
                    // Legacy syntax: separate provider and model_name fields
                    let model_spec = ModelSpecifier::with_provider(provider_name, model_name);
                    providers
                        .as_ref()
                        .create_agent_from_spec(model_spec, base_url.as_deref(), api_key.as_deref())
                        .await
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to create agent from legacy spec: {}",
                                e
                            ))
                        })?
                } else if let Some(provider_name) =
                    args.get::<_, Option<String>>("provider").ok().flatten()
                {
                    // Legacy syntax with just provider (use default model)
                    providers
                        .get_provider(Some(&provider_name))
                        .await
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to get provider '{}': {}",
                                provider_name, e
                            ))
                        })?
                } else {
                    // No provider specified, use default
                    providers.get_default_provider().await.map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to get default provider: {}", e))
                    })?
                };

                // Create a simple agent wrapper
                let agent: Arc<dyn Agent> = Arc::new(SimpleProviderAgent::new(
                    agent_config,
                    provider,
                    "default".to_string(), // This will be overridden by the provider's model
                ));

                // Create the Lua wrapper
                let wrapper = LuaAgentWrapper {
                    agent,
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

    // Add Agent.list() function to list available agent types
    let bridge_for_list = bridge.clone();
    let list_fn = lua
        .create_async_function(move |lua, _: ()| {
            let bridge = bridge_for_list.clone();
            async move {
                let types = bridge.list_agent_types().await;
                let list_table = lua.create_table()?;
                for (i, agent_type) in types.iter().enumerate() {
                    list_table.set(i + 1, agent_type.clone())?;
                }
                Ok(list_table)
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.list function: {}", e),
            source: None,
        })?;

    agent_table
        .set("list", list_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.list: {}", e),
            source: None,
        })?;

    // Add Agent.listTemplates() function
    let bridge_for_templates = bridge.clone();
    let list_templates_fn = lua
        .create_async_function(move |lua, _: ()| {
            let bridge = bridge_for_templates.clone();
            async move {
                let templates = bridge.list_templates().await;
                let list_table = lua.create_table()?;
                for (i, template) in templates.iter().enumerate() {
                    list_table.set(i + 1, template.clone())?;
                }
                Ok(list_table)
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.listTemplates function: {}", e),
            source: None,
        })?;

    agent_table
        .set("listTemplates", list_templates_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.listTemplates: {}", e),
            source: None,
        })?;

    // Add Agent.get() function to get an existing agent instance
    let bridge_for_get = bridge.clone();
    let registry_for_get = registry.clone();
    let providers_for_get = providers.clone();
    let get_fn = lua
        .create_async_function(move |_lua, name: String| {
            let bridge = bridge_for_get.clone();
            let registry = registry_for_get.clone();
            let providers = providers_for_get.clone();
            async move {
                if let Some(agent) = bridge.get_agent(&name).await {
                    let wrapper = LuaAgentWrapper {
                        agent,
                        _registry: registry,
                        _providers: providers,
                    };
                    Ok(Some(wrapper))
                } else {
                    Ok(None)
                }
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.get function: {}", e),
            source: None,
        })?;

    agent_table
        .set("get", get_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.get: {}", e),
            source: None,
        })?;

    // Add Agent.createFromTemplate() function
    let bridge_for_template = bridge.clone();
    let registry_for_template = registry.clone();
    let providers_for_template = providers.clone();
    let create_from_template_fn = lua
        .create_async_function(
            move |_lua, (instance_name, template_name, params): (String, String, Table)| {
                let bridge = bridge_for_template.clone();
                let registry = registry_for_template.clone();
                let providers = providers_for_template.clone();
                async move {
                    // Convert Lua table to HashMap
                    let mut parameters = HashMap::new();
                    for (key, value) in params.pairs::<String, mlua::Value>().flatten() {
                        if let Ok(json_value) = lua_value_to_json(value) {
                            parameters.insert(key, json_value);
                        }
                    }

                    // Create from template
                    bridge
                        .create_from_template(&instance_name, &template_name, parameters)
                        .await
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to create agent from template: {}",
                                e
                            ))
                        })?;

                    // Return the created agent
                    if let Some(agent) = bridge.get_agent(&instance_name).await {
                        let wrapper = LuaAgentWrapper {
                            agent,
                            _registry: registry,
                            _providers: providers,
                        };
                        Ok(wrapper)
                    } else {
                        Err(mlua::Error::RuntimeError(
                            "Failed to retrieve created agent".to_string(),
                        ))
                    }
                }
            },
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.createFromTemplate function: {}", e),
            source: None,
        })?;

    agent_table
        .set("createFromTemplate", create_from_template_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.createFromTemplate: {}", e),
            source: None,
        })?;

    // Add Agent.listInstances() function
    let bridge_for_instances = bridge.clone();
    let list_instances_fn = lua
        .create_async_function(move |lua, _: ()| {
            let bridge = bridge_for_instances.clone();
            async move {
                let instances = bridge.list_instances().await;
                let list_table = lua.create_table()?;
                for (i, instance) in instances.iter().enumerate() {
                    list_table.set(i + 1, instance.clone())?;
                }
                Ok(list_table)
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.listInstances function: {}", e),
            source: None,
        })?;

    agent_table
        .set("listInstances", list_instances_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.listInstances: {}", e),
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
    agent: Arc<dyn Agent>,
    _registry: Arc<ComponentRegistry>,
    _providers: Arc<ProviderManager>,
}

impl UserData for LuaAgentWrapper {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // execute method
        methods.add_async_method("execute", |lua, this, input: Table| async move {
            // Convert Lua table to AgentInput
            let agent_input = lua_table_to_agent_input(lua, input)?;
            let context = ExecutionContext::new();

            // Execute the agent
            let result = this
                .agent
                .execute(agent_input, context)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

            // Convert AgentOutput to Lua table
            let output_table = agent_output_to_lua_table(lua, result)?;

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
