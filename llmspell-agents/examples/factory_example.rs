//! ABOUTME: Example demonstrating agent factory usage
//! ABOUTME: Shows how to create agents using factory pattern and builder API

use anyhow::Result;
use llmspell_agents::{
    lifecycle::hooks::{LoggingHook, SecurityHook, ValidationHook},
    prelude::*,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Create provider manager
    let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());

    // Create a factory with hooks
    let mut factory = DefaultAgentFactory::new(provider_manager);

    // Add validation hook
    let validation_hook = ValidationHook::new()
        .with_execution_limits(10, 3600)
        .with_allowed_types(vec![
            "basic".to_string(),
            "tool-orchestrator".to_string(),
            "research".to_string(),
        ]);
    factory.add_hook(Arc::new(validation_hook));

    // Add logging hook
    let logging_hook = LoggingHook::new().with_level(tracing::Level::INFO);
    factory.add_hook(Arc::new(logging_hook));

    // Add security hook
    let security_hook = SecurityHook::new()
        .with_max_tool_access(10)
        .with_forbidden_tools(vec!["dangerous_tool".to_string()]);
    factory.add_hook(Arc::new(security_hook));

    // Example 1: Create agent using builder
    println!("\n=== Example 1: Agent Builder ===");

    let agent_config = AgentBuilder::new("my-agent", "research")
        .description("A research agent for gathering information")
        .allow_tools(vec!["web_search".to_string(), "file_tool".to_string()])
        .max_execution_time_secs(300)
        .max_memory_mb(512)
        .build()?;

    println!("Built agent config: {agent_config:?}");

    // Try to create agent
    match factory.create_agent(agent_config).await {
        Ok(agent) => {
            println!("Agent created successfully!");
            println!("Agent name: {}", agent.metadata().name);

            // Test the agent
            let input = llmspell_core::types::AgentInput::text("Hello from factory example!");
            let context = llmspell_core::ExecutionContext::default();

            match agent.execute(input, context).await {
                Ok(output) => println!("Agent response: {}", output.text),
                Err(e) => println!("Execution error: {e}"),
            }
        }
        Err(e) => println!("Creation error: {e}"),
    }

    // Example 2: Create from template
    println!("\n=== Example 2: Agent Templates ===");

    let templates = factory.list_templates();
    println!("Available templates: {templates:?}");

    // Try to create from template
    match factory.create_from_template("basic").await {
        Ok(agent) => {
            println!("Agent created from template!");

            // Test the templated agent
            let input = llmspell_core::types::AgentInput::text("Hello from template!");
            let context = llmspell_core::ExecutionContext::default();

            match agent.execute(input, context).await {
                Ok(output) => println!("Template agent response: {}", output.text),
                Err(e) => println!("Execution error: {e}"),
            }
        }
        Err(e) => println!("Template creation error: {e}"),
    }

    // Example 3: Using convenience builders
    println!("\n=== Example 3: Convenience Builders ===");

    let research_agent = AgentBuilder::basic("simple-agent")
        .max_tool_calls(10)
        .build()?;
    println!("Basic agent config: {research_agent:#?}");

    let orchestrator = AgentBuilder::tool_orchestrator("orchestrator")
        .max_execution_time_secs(600)
        .build()?;
    println!("Orchestrator config: {orchestrator:#?}");

    let llm_agent = AgentBuilder::llm("gpt-agent", "openai", "gpt-4")
        .temperature(0.8)
        .max_tokens(2000)
        .build()?;
    println!("LLM agent config: {llm_agent:#?}");

    // Example 4: Using factory registry
    println!("\n=== Example 4: Factory Registry ===");

    let registry = global_registry();

    // Register our factory
    registry
        .register_factory("default".to_string(), Arc::new(factory))
        .await?;

    println!(
        "Registered factories: {:?}",
        registry.list_factories().await
    );

    // Create agent through registry
    let config = AgentBuilder::basic("registry-agent").build()?;
    match registry.create_agent(config).await {
        Ok(agent) => {
            println!("Agent created through registry!");

            // Test registry agent
            let input = llmspell_core::types::AgentInput::text("Hello from registry!");
            let context = llmspell_core::ExecutionContext::default();

            match agent.execute(input, context).await {
                Ok(output) => println!("Registry agent response: {}", output.text),
                Err(e) => println!("Execution error: {e}"),
            }
        }
        Err(e) => println!("Registry creation error: {e}"),
    }

    // Example 5: Default templates
    println!("\n=== Example 5: Default Templates ===");
    println!("Note: DefaultTemplates has been removed. Use TemplateFactory from the templates module instead.");

    // TODO: Update this example to use the new templates module API
    // Example:
    // use llmspell_agents::templates::create_builtin_templates;
    // let factory = create_builtin_templates();
    // let templates = factory.list_templates();
    // for template_id in &templates {
    //     println!("  - {}", template_id);
    // }

    Ok(())
}
