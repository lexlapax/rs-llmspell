//! ABOUTME: Example usage of agent templates demonstrating creation, customization, and instantiation
//! ABOUTME: Shows how to use built-in templates and create custom agent instances from templates

use anyhow::Result;
use llmspell_agents::templates::{
    AgentTemplate, MonitorAgentTemplate, OrchestratorAgentTemplate, TemplateFactory,
    TemplateInstantiationParams, ToolAgentTemplate,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Agent Templates Usage Examples\n");

    // Example 1: Using Tool Agent Template
    example_tool_agent().await?;

    // Example 2: Using Orchestrator Agent Template
    example_orchestrator_agent().await?;

    // Example 3: Using Monitor Agent Template
    example_monitor_agent().await?;

    // Example 4: Using Template Factory
    example_template_factory().await?;

    // Example 5: Template Validation
    example_template_validation().await?;

    Ok(())
}

/// Example 1: Creating a Tool Agent from template
async fn example_tool_agent() -> Result<()> {
    println!("=== Example 1: Tool Agent Template ===\n");

    // Create the template
    let template = ToolAgentTemplate::new();

    println!("Template: {}", template.schema().metadata.name);
    println!("Category: {:?}", template.category());
    println!("Complexity: {:?}", template.complexity());
    println!("Required tools: {:?}", template.required_tools());
    println!();

    // Create instantiation parameters
    let params = TemplateInstantiationParams::new("example-tool-agent".to_string())
        .with_parameter("agent_name", "Example Tool Agent".into())
        .with_parameter("max_tools", 10.into())
        .with_parameter("execution_mode", "sequential".into())
        .with_parameter("enable_caching", true.into())
        .with_parameter("cache_ttl", 300.into());

    // Validate parameters
    match template.validate_parameters(&params).await {
        Ok(_) => println!("Parameters validated successfully"),
        Err(e) => println!("Parameter validation failed: {}", e),
    }

    // Instantiate the agent (in real usage, this would create an actual agent)
    match template.instantiate(params).await {
        Ok(result) => {
            println!("Agent instantiated successfully!");
            println!("Agent created successfully!");
            println!("Template used: {}", result.template_schema.metadata.name);
            println!(
                "Applied config: {:?}",
                result.applied_config.keys().collect::<Vec<_>>()
            );
        }
        Err(e) => {
            println!(
                "Agent instantiation would create a real agent (mock error: {})",
                e
            );
        }
    }

    println!();
    Ok(())
}

/// Example 2: Creating an Orchestrator Agent from template
async fn example_orchestrator_agent() -> Result<()> {
    println!("=== Example 2: Orchestrator Agent Template ===\n");

    // Use the enterprise variant
    let template = OrchestratorAgentTemplate::enterprise();

    println!("Template: {}", template.schema().metadata.name);
    println!("Description: {}", template.schema().metadata.description);
    println!();

    // Create parameters for enterprise orchestrator
    let params = TemplateInstantiationParams::new("enterprise-orchestrator".to_string())
        .with_parameter("agent_name", "Enterprise Workflow Orchestrator".into())
        .with_parameter("max_managed_agents", 50.into())
        .with_parameter("orchestration_strategy", "parallel".into())
        .with_parameter("enable_health_monitoring", true.into())
        .with_parameter("max_concurrent_workflows", 10.into())
        .with_parameter("enable_rollback", true.into())
        .with_config_override("audit_logging", true.into())
        .with_environment("LOG_LEVEL", "INFO");

    // Show parameter info
    println!("Instantiating with parameters:");
    for (key, value) in &params.parameters {
        println!("  {}: {}", key, value);
    }
    println!();

    // Attempt instantiation
    match template.instantiate(params).await {
        Ok(result) => {
            println!("Orchestrator created successfully!");
            println!(
                "Applied parameters: {} parameters",
                result.applied_parameters.len()
            );
        }
        Err(e) => {
            println!(
                "Mock orchestrator creation (would create real agent): {}",
                e
            );
        }
    }

    println!();
    Ok(())
}

/// Example 3: Creating a Monitor Agent from template
async fn example_monitor_agent() -> Result<()> {
    println!("=== Example 3: Monitor Agent Template ===\n");

    // Use different monitor variants
    let variants = vec![
        ("System Monitor", MonitorAgentTemplate::system_monitor()),
        (
            "Application Monitor",
            MonitorAgentTemplate::application_monitor(),
        ),
        ("Lightweight Monitor", MonitorAgentTemplate::lightweight()),
    ];

    for (name, template) in variants {
        println!("--- {} ---", name);
        println!("Complexity: {:?}", template.complexity());

        let resources = &template.schema().resource_requirements;
        if let Some(memory) = resources.memory {
            println!("Memory requirement: {} MB", memory / 1024 / 1024);
        }
        if let Some(cpu) = resources.cpu {
            println!("CPU requirement: {}%", cpu);
        }

        println!("Optional tools: {:?}", template.optional_tools());
        println!();
    }

    // Create a system monitor
    let template = MonitorAgentTemplate::system_monitor();
    let params = TemplateInstantiationParams::new("system-monitor-01".to_string())
        .with_parameter("agent_name", "Production System Monitor".into())
        .with_parameter("monitoring_interval", 15.into())
        .with_parameter("cpu_threshold", 75.0.into())
        .with_parameter("memory_threshold", 80.0.into())
        .with_parameter("disk_threshold", 85.0.into())
        .with_parameter("max_alerts_per_minute", 5.into());

    match template.instantiate(params).await {
        Ok(result) => {
            println!("Monitor agent created!");
            println!("Applied config items: {}", result.applied_config.len());
        }
        Err(e) => {
            println!("Mock monitor creation: {}", e);
        }
    }

    println!();
    Ok(())
}

/// Example 4: Using Template Factory
async fn example_template_factory() -> Result<()> {
    println!("=== Example 4: Template Factory ===\n");

    // Create factory and register templates
    let mut factory = TemplateFactory::new();

    // Register built-in templates
    factory.register_template(Box::new(ToolAgentTemplate::new()))?;
    factory.register_template(Box::new(ToolAgentTemplate::batch_processor()))?;
    factory.register_template(Box::new(OrchestratorAgentTemplate::new()))?;
    factory.register_template(Box::new(OrchestratorAgentTemplate::simple()))?;
    factory.register_template(Box::new(MonitorAgentTemplate::new()))?;

    println!("Registered {} templates", factory.template_count());
    println!("Available templates:");
    for template_id in factory.list_templates() {
        if let Some(template) = factory.get_template(&template_id) {
            println!("  - {} ({})", template_id, template.schema().metadata.name);
        }
    }
    println!();

    // Search for templates
    let search_terms = vec!["tool", "monitor", "workflow"];
    for term in search_terms {
        let found = factory.find_templates(term);
        println!("Templates matching '{}': {}", term, found.len());
        for template in found {
            println!("  - {}", template.schema().metadata.id);
        }
    }
    println!();

    // Get templates by category
    use llmspell_agents::templates::schema::TemplateCategory;
    let categories = vec![
        TemplateCategory::ToolExecution,
        TemplateCategory::Orchestration,
        TemplateCategory::Monitoring,
    ];

    for category in categories {
        let templates = factory.get_templates_by_category(&category);
        println!("{:?} templates: {}", category, templates.len());
    }

    Ok(())
}

/// Example 5: Template parameter validation
async fn example_template_validation() -> Result<()> {
    println!("\n=== Example 5: Template Validation ===\n");

    let template = ToolAgentTemplate::new();

    // Test various parameter validations
    let test_cases = vec![
        (
            "Valid parameters",
            TemplateInstantiationParams::new("test-1".to_string())
                .with_parameter("agent_name", "Valid Agent".into())
                .with_parameter("max_tools", 5.into())
                .with_parameter("execution_mode", "sequential".into()),
            true,
        ),
        (
            "Missing required parameter",
            TemplateInstantiationParams::new("test-2".to_string())
                .with_parameter("max_tools", 5.into()),
            false,
        ),
        (
            "Invalid enum value",
            TemplateInstantiationParams::new("test-3".to_string())
                .with_parameter("agent_name", "Test".into())
                .with_parameter("execution_mode", "invalid_mode".into()),
            false,
        ),
        (
            "Value out of range",
            TemplateInstantiationParams::new("test-4".to_string())
                .with_parameter("agent_name", "Test".into())
                .with_parameter("max_tools", 200.into()), // Max is 100
            false,
        ),
        (
            "Wrong parameter type",
            TemplateInstantiationParams::new("test-5".to_string())
                .with_parameter("agent_name", "Test".into())
                .with_parameter("enable_caching", "yes".into()), // Should be boolean
            false,
        ),
    ];

    for (description, params, should_pass) in test_cases {
        print!("{}: ", description);
        match template.validate_parameters(&params).await {
            Ok(_) => {
                if should_pass {
                    println!("✓ Passed");
                } else {
                    println!("✗ Expected to fail but passed");
                }
            }
            Err(e) => {
                if !should_pass {
                    println!("✓ Failed as expected: {}", e);
                } else {
                    println!("✗ Unexpected failure: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Example showing how to create a custom template configuration
#[allow(dead_code)]
async fn example_custom_configuration() -> Result<()> {
    println!("\n=== Custom Template Configuration ===\n");

    // Create a highly customized tool agent
    let template = ToolAgentTemplate::specialized(vec![
        "file_reader".to_string(),
        "data_processor".to_string(),
        "text_analyzer".to_string(),
    ]);

    let params = TemplateInstantiationParams::new("custom-processor".to_string())
        .with_parameter("agent_name", "Custom Data Processor".into())
        .with_parameter("max_tools", 20.into())
        .with_parameter("execution_mode", "parallel".into())
        .with_parameter("enable_caching", true.into())
        .with_parameter("cache_ttl", 600.into())
        .with_parameter("enable_retry", true.into())
        .with_parameter("max_retries", 5.into())
        .with_parameter("retry_delay", 2.into())
        .with_parameter("timeout_seconds", 120.into())
        .with_parameter("enable_logging", true.into())
        .with_parameter("log_level", "debug".into())
        // Custom configuration overrides
        .with_config_override("batch_size", 100.into())
        .with_config_override("parallel_workers", 4.into())
        .with_config_override("memory_limit_mb", 512.into())
        // Environment variables
        .with_environment("DATA_DIR", "/tmp/data")
        .with_environment("OUTPUT_FORMAT", "json");

    // Show all the customization
    println!("Custom parameters: {}", params.parameters.len());
    println!("Config overrides: {}", params.config_overrides.len());
    println!("Environment vars: {}", params.environment.len());

    match template.instantiate(params).await {
        Ok(result) => {
            println!("\nCustom agent created successfully!");
            println!("Applied configuration includes:");
            for (key, _) in result.applied_config.iter().take(5) {
                println!("  - {}", key);
            }
            if result.applied_config.len() > 5 {
                println!("  ... and {} more", result.applied_config.len() - 5);
            }
        }
        Err(e) => {
            println!("\nMock creation (would create real custom agent): {}", e);
        }
    }

    Ok(())
}
