//! Example demonstrating the use of builder patterns for configuration objects

use llmspell_agents::factory::{AgentConfig, ModelConfig};
use llmspell_sessions::config::SessionManagerConfig;
use llmspell_workflows::types::WorkflowConfig;
use std::time::Duration;

fn main() {
    // Example 1: Building a SessionManagerConfig
    let session_config = SessionManagerConfig::builder()
        .max_active_sessions(500)
        .auto_persist(true)
        .persist_interval_secs(60)
        .enable_compression(true)
        .compression_level(6)
        .build();

    println!("Session config: {session_config:#?}");

    // Example 2: Building a WorkflowConfig
    let workflow_config = WorkflowConfig::builder()
        .max_execution_time(Some(Duration::from_secs(600)))
        .default_step_timeout(Duration::from_secs(60))
        .max_retry_attempts(5)
        .retry_delay_ms(2000)
        .exponential_backoff(true)
        .continue_on_error(false)
        .build();

    println!("\nWorkflow config: {workflow_config:#?}");

    // Example 3: Building an AgentConfig
    let agent_config = AgentConfig::builder("my-agent")
        .description("An example AI agent")
        .agent_type("llm")
        .model(ModelConfig {
            provider: "openai".to_string(),
            model_id: "gpt-4".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            settings: serde_json::Map::new(),
        })
        .allow_tool("calculator")
        .allow_tool("web_search")
        .max_execution_time_secs(300)
        .max_memory_mb(1024)
        .build();

    println!("\nAgent config: {agent_config:#?}");

    // Example 4: Using defaults with minimal configuration
    let minimal_session = SessionManagerConfig::builder()
        .max_active_sessions(100)
        .build();

    println!("\nMinimal session config uses defaults: {minimal_session:#?}");

    // Example 5: Chaining multiple configurations
    let workflow = WorkflowConfig::builder()
        .max_retry_attempts(3)
        .continue_on_error(true)
        .build();

    let agent = AgentConfig::builder("resilient-agent")
        .description("An agent that handles errors gracefully")
        .agent_type("basic")
        .max_tool_calls(50)
        .build();

    println!("\nResilient workflow: {workflow:#?}");
    println!("Resilient agent: {agent:#?}");
}
