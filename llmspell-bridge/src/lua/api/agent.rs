//! ABOUTME: Lua Agent API implementation providing Agent.create() and agent methods
//! ABOUTME: Bridges between Lua scripts and Rust Agent implementations

use mlua::{Lua, Result as LuaResult, Table, UserData, UserDataMethods};
use std::sync::Arc;
use llmspell_core::{
    traits::agent::Agent,
    types::{AgentInput, ExecutionContext},
};
use llmspell_core::error::LLMSpellError;
use crate::engine::types::AgentApiDefinition;
use crate::{ComponentRegistry, ProviderManager};

/// Inject the Agent API into the Lua environment
pub fn inject_agent_api(
    lua: &Lua,
    api_def: &AgentApiDefinition,
    registry: Arc<ComponentRegistry>,
    providers: Arc<ProviderManager>,
) -> Result<(), LLMSpellError> {
    // Create the Agent global table
    let agent_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Agent table: {}", e),
        source: None,
    })?;
    
    // Clone Arc for the closure
    let _registry_clone = registry.clone();
    let _providers_clone = providers.clone();
    
    // Create the Agent.create() function
    let create_fn = lua.create_function(move |_lua, args: Table| -> LuaResult<()> {
        // Extract configuration from Lua table
        let _system_prompt: Option<String> = args.get("system_prompt").ok();
        let _temperature: Option<f32> = args.get("temperature").ok();
        let _max_tokens: Option<usize> = args.get("max_tokens").ok();
        let _max_conversation_length: Option<usize> = args.get("max_conversation_length").ok();
        
        // TODO: Actually create agent from provider
        // For now, return error as we need provider integration
        Err(mlua::Error::RuntimeError("Agent creation not yet implemented - needs provider integration".to_string()))
    }).map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Agent.create function: {}", e),
        source: None,
    })?;
    
    // Add the create function to the Agent table
    agent_table.set(&api_def.constructor[..], create_fn).map_err(|e| LLMSpellError::Component {
        message: format!("Failed to set Agent.create: {}", e),
        source: None,
    })?;
    
    // Set the Agent table as a global
    lua.globals().set(&api_def.global_name[..], agent_table).map_err(|e| LLMSpellError::Component {
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
            let result = this.agent.execute(agent_input, context).await.map_err(|e| {
                mlua::Error::ExternalError(Arc::new(e))
            })?;
            
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