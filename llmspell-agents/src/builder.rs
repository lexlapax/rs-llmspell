//! ABOUTME: Agent builder with fluent API for easy agent configuration
//! ABOUTME: Provides chainable methods for building agent configurations

use crate::factory::{AgentConfig, ModelConfig, ResourceLimits};
use anyhow::Result;
use serde_json::Value;

/// Builder for creating agent configurations with a fluent API
#[derive(Debug, Clone)]
pub struct AgentBuilder {
    config: AgentConfig,
}

impl AgentBuilder {
    /// Create a new agent builder with required fields
    pub fn new(name: impl Into<String>, agent_type: impl Into<String>) -> Self {
        Self {
            config: AgentConfig {
                name: name.into(),
                description: String::new(),
                agent_type: agent_type.into(),
                model: None,
                allowed_tools: vec![],
                custom_config: serde_json::Map::new(),
                resource_limits: ResourceLimits::default(),
            },
        }
    }

    /// Set the agent description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.config.description = description.into();
        self
    }

    /// Configure the model for LLM-based agents
    pub fn with_model(mut self, provider: impl Into<String>, model_id: impl Into<String>) -> Self {
        self.config.model = Some(ModelConfig {
            provider: provider.into(),
            model_id: model_id.into(),
            temperature: None,
            max_tokens: None,
            settings: serde_json::Map::new(),
        });
        self
    }

    /// Set model temperature
    #[must_use]
    pub const fn temperature(mut self, temperature: f32) -> Self {
        if let Some(model) = &mut self.config.model {
            model.temperature = Some(temperature);
        }
        self
    }

    /// Set maximum tokens for model
    #[must_use]
    pub const fn max_tokens(mut self, max_tokens: u32) -> Self {
        if let Some(model) = &mut self.config.model {
            model.max_tokens = Some(max_tokens);
        }
        self
    }

    /// Add a model setting
    pub fn model_setting(mut self, key: impl Into<String>, value: Value) -> Self {
        if let Some(model) = &mut self.config.model {
            model.settings.insert(key.into(), value);
        }
        self
    }

    /// Allow access to specific tools
    pub fn allow_tool(mut self, tool_id: impl Into<String>) -> Self {
        self.config.allowed_tools.push(tool_id.into());
        self
    }

    /// Allow access to multiple tools
    #[must_use]
    pub fn allow_tools(mut self, tool_ids: Vec<String>) -> Self {
        self.config.allowed_tools.extend(tool_ids);
        self
    }

    /// Allow access to all tools
    #[must_use]
    pub fn allow_all_tools(mut self) -> Self {
        self.config.allowed_tools = vec!["*".to_string()];
        self
    }

    /// Add a custom configuration parameter
    pub fn custom(mut self, key: impl Into<String>, value: Value) -> Self {
        self.config.custom_config.insert(key.into(), value);
        self
    }

    /// Set maximum execution time
    #[must_use]
    pub const fn max_execution_time_secs(mut self, secs: u64) -> Self {
        self.config.resource_limits.max_execution_time_secs = secs;
        self
    }

    /// Set maximum memory usage
    #[must_use]
    pub const fn max_memory_mb(mut self, mb: u64) -> Self {
        self.config.resource_limits.max_memory_mb = mb;
        self
    }

    /// Set maximum number of tool calls
    #[must_use]
    pub const fn max_tool_calls(mut self, calls: u32) -> Self {
        self.config.resource_limits.max_tool_calls = calls;
        self
    }

    /// Set maximum recursion depth
    #[must_use]
    pub const fn max_recursion_depth(mut self, depth: u8) -> Self {
        self.config.resource_limits.max_recursion_depth = depth;
        self
    }

    /// Set all resource limits at once
    #[must_use]
    pub const fn resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.config.resource_limits = limits;
        self
    }

    /// Build the agent configuration
    pub fn build(self) -> Result<AgentConfig> {
        // Validate the configuration
        if self.config.name.is_empty() {
            anyhow::bail!("Agent name cannot be empty");
        }

        if self.config.agent_type.is_empty() {
            anyhow::bail!("Agent type cannot be empty");
        }

        Ok(self.config)
    }
}

/// Convenience builders for common agent types
impl AgentBuilder {
    /// Create a builder for a basic agent
    pub fn basic(name: impl Into<String>) -> Self {
        Self::new(name, "basic").description("Basic agent with minimal capabilities")
    }

    /// Create a builder for a tool orchestrator agent
    pub fn tool_orchestrator(name: impl Into<String>) -> Self {
        Self::new(name, "tool-orchestrator")
            .description("Agent that orchestrates tool execution")
            .allow_all_tools()
    }

    /// Create a builder for an LLM agent
    pub fn llm(
        name: impl Into<String>,
        provider: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self::new(name.into(), "llm")
            .description("LLM-powered agent")
            .with_model(provider, model)
    }

    /// Create a builder for a workflow agent
    pub fn workflow(name: impl Into<String>) -> Self {
        Self::new(name, "workflow").description("Agent that executes workflows")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_basic_builder() {
        let config = AgentBuilder::new("test-agent", "basic")
            .description("A test agent")
            .build()
            .unwrap();

        assert_eq!(config.name, "test-agent");
        assert_eq!(config.agent_type, "basic");
        assert_eq!(config.description, "A test agent");
    }
    #[test]
    fn test_fluent_api() {
        let config = AgentBuilder::new("complex-agent", "llm")
            .description("Complex LLM agent")
            .with_model("openai", "gpt-4")
            .temperature(0.7)
            .max_tokens(2000)
            .model_setting("top_p", json!(0.9))
            .allow_tools(vec!["calculator".to_string(), "file_tool".to_string()])
            .custom("retry_count", json!(3))
            .max_execution_time_secs(600)
            .max_memory_mb(1024)
            .build()
            .unwrap();

        assert_eq!(config.name, "complex-agent");
        assert!(config.model.is_some());

        let model = config.model.unwrap();
        assert_eq!(model.provider, "openai");
        assert_eq!(model.model_id, "gpt-4");
        assert_eq!(model.temperature, Some(0.7));
        assert_eq!(model.max_tokens, Some(2000));
        assert_eq!(model.settings.get("top_p"), Some(&json!(0.9)));

        assert_eq!(config.allowed_tools.len(), 2);
        assert_eq!(config.custom_config.get("retry_count"), Some(&json!(3)));
        assert_eq!(config.resource_limits.max_execution_time_secs, 600);
        assert_eq!(config.resource_limits.max_memory_mb, 1024);
    }
    #[test]
    fn test_convenience_builders() {
        // Test basic builder
        let basic = AgentBuilder::basic("my-basic").build().unwrap();
        assert_eq!(basic.agent_type, "basic");
        assert!(basic.description.contains("Basic"));

        // Test tool orchestrator builder
        let orchestrator = AgentBuilder::tool_orchestrator("my-orchestrator")
            .build()
            .unwrap();
        assert_eq!(orchestrator.agent_type, "tool-orchestrator");
        assert_eq!(orchestrator.allowed_tools, vec!["*"]);

        // Test LLM builder
        let llm = AgentBuilder::llm("my-llm", "anthropic", "claude-3")
            .build()
            .unwrap();
        assert_eq!(llm.agent_type, "llm");
        assert!(llm.model.is_some());
        assert_eq!(llm.model.as_ref().unwrap().provider, "anthropic");

        // Test workflow builder
        let workflow = AgentBuilder::workflow("my-workflow").build().unwrap();
        assert_eq!(workflow.agent_type, "workflow");
    }
    #[test]
    fn test_validation() {
        // Empty name should fail
        let result = AgentBuilder::new("", "basic").build();
        assert!(result.is_err());

        // Empty type should fail
        let result = AgentBuilder::new("test", "").build();
        assert!(result.is_err());
    }
}
