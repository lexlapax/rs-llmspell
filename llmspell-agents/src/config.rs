//! ABOUTME: Default agent configurations and templates
//! ABOUTME: Provides pre-configured agent templates for common use cases

use crate::{builder::AgentBuilder, factory::AgentConfig};
use serde_json::json;
use std::collections::HashMap;

/// Default agent templates
pub struct DefaultTemplates;

impl DefaultTemplates {
    /// Create all default agent templates
    pub fn create_all() -> HashMap<String, AgentConfig> {
        let mut templates = HashMap::new();

        // Basic agent - minimal functionality
        templates.insert(
            "basic".to_string(),
            AgentBuilder::basic("basic-agent")
                .description("Basic agent with minimal capabilities for simple tasks")
                .max_execution_time_secs(60)
                .max_memory_mb(256)
                .build()
                .unwrap(),
        );

        // Tool orchestrator - can use all tools
        templates.insert(
            "tool-orchestrator".to_string(),
            AgentBuilder::tool_orchestrator("tool-orchestrator")
                .description("Orchestrates execution of multiple tools to accomplish complex tasks")
                .max_execution_time_secs(300)
                .max_tool_calls(50)
                .build()
                .unwrap(),
        );

        // Research agent - focused on information gathering
        templates.insert(
            "research".to_string(),
            AgentBuilder::new("research-agent", "research")
                .description(
                    "Specialized in gathering and analyzing information from various sources",
                )
                .allow_tools(vec![
                    "web_search".to_string(),
                    "web_scraper".to_string(),
                    "url_analyzer".to_string(),
                    "file_tool".to_string(),
                    "json_processor".to_string(),
                ])
                .custom("search_depth", json!(3))
                .custom("max_sources", json!(10))
                .max_execution_time_secs(600)
                .build()
                .unwrap(),
        );

        // Code assistant - helps with programming tasks
        templates.insert(
            "code-assistant".to_string(),
            AgentBuilder::new("code-assistant", "code-assistant")
                .description("Assists with code generation, analysis, and refactoring")
                .allow_tools(vec![
                    "file_tool".to_string(),
                    "grep_tool".to_string(),
                    "git_tool".to_string(),
                    "run_bash".to_string(),
                    "code_analyzer".to_string(),
                ])
                .custom(
                    "language_support",
                    json!(["rust", "python", "javascript", "typescript"]),
                )
                .max_execution_time_secs(300)
                .build()
                .unwrap(),
        );

        // Data processor - handles data transformation
        templates.insert(
            "data-processor".to_string(),
            AgentBuilder::new("data-processor", "data-processor")
                .description("Processes and transforms data using various tools")
                .allow_tools(vec![
                    "json_processor".to_string(),
                    "csv_analyzer".to_string(),
                    "data_validation".to_string(),
                    "calculator".to_string(),
                    "file_tool".to_string(),
                ])
                .custom("batch_size", json!(1000))
                .max_memory_mb(1024)
                .build()
                .unwrap(),
        );

        // Monitor agent - tracks system and process states
        templates.insert(
            "monitor".to_string(),
            AgentBuilder::new("monitor-agent", "monitor")
                .description("Monitors system resources, processes, and application states")
                .allow_tools(vec![
                    "process_manager".to_string(),
                    "resource_monitor".to_string(),
                    "webpage_monitor".to_string(),
                    "file_tool".to_string(),
                ])
                .custom("check_interval_secs", json!(60))
                .custom("alert_threshold", json!(0.8))
                .max_execution_time_secs(3600) // 1 hour
                .build()
                .unwrap(),
        );

        // Security analyst - focuses on security tasks
        templates.insert(
            "security-analyst".to_string(),
            AgentBuilder::new("security-analyst", "security")
                .description("Analyzes security aspects and validates safety measures")
                .allow_tools(vec![
                    "file_tool".to_string(),
                    "hash_generator".to_string(),
                    "data_validation".to_string(),
                    "json_processor".to_string(),
                    "process_manager".to_string(),
                ])
                .custom("scan_depth", json!("deep"))
                .custom("report_format", json!("detailed"))
                .build()
                .unwrap(),
        );

        // Workflow coordinator - manages workflow execution
        templates.insert(
            "workflow-coordinator".to_string(),
            AgentBuilder::workflow("workflow-coordinator")
                .description("Coordinates execution of complex workflows with multiple steps")
                .allow_all_tools()
                .custom("parallel_execution", json!(true))
                .custom("retry_failed_steps", json!(true))
                .max_execution_time_secs(1800) // 30 minutes
                .max_recursion_depth(5)
                .build()
                .unwrap(),
        );

        templates
    }

    /// Get a specific template configuration
    pub fn get(name: &str) -> Option<AgentConfig> {
        Self::create_all().remove(name)
    }

    /// List all available template names
    pub fn list() -> Vec<String> {
        Self::create_all().keys().cloned().collect()
    }
}

/// Environment-based configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load agent configuration from environment variables
    pub fn from_env(prefix: &str) -> Option<AgentConfig> {
        use std::env;

        let name = env::var(format!("{}_NAME", prefix)).ok()?;
        let agent_type = env::var(format!("{}_TYPE", prefix)).ok()?;

        let mut builder = AgentBuilder::new(name, agent_type);

        // Load description if present
        if let Ok(desc) = env::var(format!("{}_DESCRIPTION", prefix)) {
            builder = builder.description(desc);
        }

        // Load model configuration if present
        if let Ok(provider) = env::var(format!("{}_MODEL_PROVIDER", prefix)) {
            if let Ok(model_id) = env::var(format!("{}_MODEL_ID", prefix)) {
                builder = builder.with_model(provider, model_id);

                if let Ok(temp) = env::var(format!("{}_MODEL_TEMPERATURE", prefix)) {
                    if let Ok(temp_f32) = temp.parse::<f32>() {
                        builder = builder.temperature(temp_f32);
                    }
                }

                if let Ok(tokens) = env::var(format!("{}_MODEL_MAX_TOKENS", prefix)) {
                    if let Ok(tokens_u32) = tokens.parse::<u32>() {
                        builder = builder.max_tokens(tokens_u32);
                    }
                }
            }
        }

        // Load allowed tools
        if let Ok(tools) = env::var(format!("{}_ALLOWED_TOOLS", prefix)) {
            let tool_list: Vec<String> = tools.split(',').map(|s| s.trim().to_string()).collect();
            builder = builder.allow_tools(tool_list);
        }

        // Load resource limits
        if let Ok(exec_time) = env::var(format!("{}_MAX_EXECUTION_SECS", prefix)) {
            if let Ok(secs) = exec_time.parse::<u64>() {
                builder = builder.max_execution_time_secs(secs);
            }
        }

        if let Ok(memory) = env::var(format!("{}_MAX_MEMORY_MB", prefix)) {
            if let Ok(mb) = memory.parse::<u64>() {
                builder = builder.max_memory_mb(mb);
            }
        }

        builder.build().ok()
    }

    /// Load agent configuration from a JSON file
    pub fn from_json_file(path: &std::path::Path) -> anyhow::Result<AgentConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: AgentConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    // Future: Add YAML support when needed
    // pub fn from_yaml_file(path: &std::path::Path) -> anyhow::Result<AgentConfig> {
    //     let content = std::fs::read_to_string(path)?;
    //     let config: AgentConfig = serde_yaml::from_str(&content)?;
    //     Ok(config)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_templates() {
        let templates = DefaultTemplates::create_all();

        // Check all expected templates exist
        assert!(templates.contains_key("basic"));
        assert!(templates.contains_key("tool-orchestrator"));
        assert!(templates.contains_key("research"));
        assert!(templates.contains_key("code-assistant"));
        assert!(templates.contains_key("data-processor"));
        assert!(templates.contains_key("monitor"));
        assert!(templates.contains_key("security-analyst"));
        assert!(templates.contains_key("workflow-coordinator"));

        // Verify specific template configurations
        let research = &templates["research"];
        assert_eq!(research.agent_type, "research");
        assert!(research.allowed_tools.contains(&"web_search".to_string()));
        assert_eq!(research.custom_config.get("search_depth"), Some(&json!(3)));

        let orchestrator = &templates["tool-orchestrator"];
        assert_eq!(orchestrator.allowed_tools, vec!["*"]);
    }

    #[test]
    fn test_template_list() {
        let list = DefaultTemplates::list();
        assert!(list.len() >= 8);
        assert!(list.contains(&"basic".to_string()));
    }

    #[test]
    fn test_get_template() {
        let basic = DefaultTemplates::get("basic");
        assert!(basic.is_some());
        assert_eq!(basic.unwrap().agent_type, "basic");

        let nonexistent = DefaultTemplates::get("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_env_config_loader() {
        use std::env;

        // Set up test environment variables
        env::set_var("TEST_AGENT_NAME", "env-agent");
        env::set_var("TEST_AGENT_TYPE", "custom");
        env::set_var("TEST_AGENT_DESCRIPTION", "Loaded from env");
        env::set_var("TEST_AGENT_ALLOWED_TOOLS", "tool1,tool2,tool3");
        env::set_var("TEST_AGENT_MAX_EXECUTION_SECS", "120");

        let config = ConfigLoader::from_env("TEST_AGENT").unwrap();

        assert_eq!(config.name, "env-agent");
        assert_eq!(config.agent_type, "custom");
        assert_eq!(config.description, "Loaded from env");
        assert_eq!(config.allowed_tools.len(), 3);
        assert_eq!(config.resource_limits.max_execution_time_secs, 120);

        // Clean up
        env::remove_var("TEST_AGENT_NAME");
        env::remove_var("TEST_AGENT_TYPE");
        env::remove_var("TEST_AGENT_DESCRIPTION");
        env::remove_var("TEST_AGENT_ALLOWED_TOOLS");
        env::remove_var("TEST_AGENT_MAX_EXECUTION_SECS");
    }
}
