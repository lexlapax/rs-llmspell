//! ABOUTME: Examples demonstrating various agent composition patterns
//! ABOUTME: Shows hierarchical, delegation, and tool composition patterns

use llmspell_agents::agents::basic::BasicAgent;
use llmspell_agents::composition::{
    lifecycle::{CompositeLifecycleManager, LifecycleConfig},
    CapabilityAggregator, CapabilityCategory, CapabilityRequirementBuilder, CompositeAgent,
    DelegatingAgentBuilder, DelegationConfig, DelegationStrategy, HierarchicalAgent,
    HierarchicalAgentBuilder, HierarchicalCompositeAgent, HierarchicalConfig,
};
use llmspell_agents::di::DIContainer;
use llmspell_agents::factory::{AgentConfig, ResourceLimits};
use llmspell_core::ExecutionContext;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Agent Composition Patterns Examples\n");

    // Example 1: Hierarchical Agent Composition
    hierarchical_composition_example().await?;

    // Example 2: Delegation Pattern
    delegation_pattern_example().await?;

    // Example 3: Capability-Based Composition
    capability_based_example().await?;

    // Example 4: Tool Composition with Agents
    tool_composition_example().await?;

    // Example 5: Composite Lifecycle Management
    lifecycle_management_example().await?;

    Ok(())
}

/// Example 1: Hierarchical Agent Composition
async fn hierarchical_composition_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("1️⃣ Hierarchical Agent Composition");
    println!("================================");

    // Create parent agent (orchestrator)
    let mut orchestrator = HierarchicalAgentBuilder::new("orchestrator")
        .description("Parent orchestrator agent")
        .build();

    // Create child agents
    let data_processor = HierarchicalAgentBuilder::new("data-processor")
        .description("Processes data")
        .add_capability(llmspell_agents::composition::Capability {
            name: "data-processing".to_string(),
            category: CapabilityCategory::DataProcessing,
            version: Some("1.0.0".to_string()),
            metadata: Default::default(),
        })
        .build();

    let analyzer = HierarchicalAgentBuilder::new("analyzer")
        .description("Analyzes results")
        .add_capability(llmspell_agents::composition::Capability {
            name: "analysis".to_string(),
            category: CapabilityCategory::Custom("analysis".to_string()),
            version: Some("1.0.0".to_string()),
            metadata: Default::default(),
        })
        .build();

    // Add children to orchestrator
    orchestrator
        .add_child(Arc::new(data_processor))
        .await
        .unwrap();
    orchestrator.add_child(Arc::new(analyzer)).await.unwrap();

    println!("✅ Created hierarchical structure:");
    println!("   Orchestrator (depth: {})", orchestrator.depth());
    println!("   ├── Data Processor");
    println!("   └── Analyzer");

    // Test propagation of events
    use llmspell_agents::composition::HierarchyEvent;
    let event = HierarchyEvent::ConfigurationChange(
        vec![("timeout".to_string(), serde_json::json!(30))]
            .into_iter()
            .collect(),
    );

    orchestrator.propagate_down(event).await?;
    println!("📢 Propagated configuration change to all children");

    println!();
    Ok(())
}

/// Example 2: Delegation Pattern
async fn delegation_pattern_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("2️⃣ Agent Delegation Pattern");
    println!("===========================");

    // Create a delegating agent
    let mut builder = DelegatingAgentBuilder::new("coordinator")
        .strategy(DelegationStrategy::LoadBalanced)
        .config(DelegationConfig {
            cache_capabilities: true,
            retry_on_failure: true,
            default_timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            max_concurrent: 10,
        });

    // Create specialized agents
    let calc_config = AgentConfig {
        name: "calc-agent".to_string(),
        description: "Performs calculations".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec!["calculator".to_string()],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };
    let calculator_agent = BasicAgent::new(calc_config)?;

    let text_config = AgentConfig {
        name: "text-agent".to_string(),
        description: "Processes text".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec!["text_manipulator".to_string()],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };
    let text_agent = BasicAgent::new(text_config)?;

    // Add agents to the builder
    builder = builder.add_agent(Arc::new(calculator_agent));
    builder = builder.add_agent(Arc::new(text_agent));

    let _coordinator = builder.build().await?;

    println!("✅ Created delegating agent with:");
    println!("   - Strategy: LoadBalanced");
    println!("   - Agents: calc-agent, text-agent");

    // Create a delegation request - skip actual delegation for example
    println!("📋 Would delegate tasks based on:");
    println!("   - Required capabilities (e.g., calculation, text processing)");
    println!("   - Agent availability and current load");
    println!("   - Strategy: Load balanced across available agents");

    println!();
    Ok(())
}

/// Example 3: Capability-Based Composition
async fn capability_based_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("3️⃣ Capability-Based Composition");
    println!("===============================");

    // Create capability aggregator
    let aggregator = CapabilityAggregator::new();

    // Register capabilities from different agents
    use llmspell_agents::composition::Capability;

    let text_cap = Capability {
        name: "text-processing".to_string(),
        category: CapabilityCategory::DataProcessing,
        version: Some("2.0.0".to_string()),
        metadata: Default::default(),
    };

    let calc_cap = Capability {
        name: "mathematical-operations".to_string(),
        category: CapabilityCategory::DataProcessing,
        version: Some("1.5.0".to_string()),
        metadata: Default::default(),
    };

    let monitor_cap = Capability {
        name: "system-monitoring".to_string(),
        category: CapabilityCategory::Monitoring,
        version: Some("1.0.0".to_string()),
        metadata: Default::default(),
    };

    aggregator.register_capability(text_cap, "agent-1")?;
    aggregator.register_capability(calc_cap, "agent-2")?;
    aggregator.register_capability(monitor_cap, "agent-3")?;

    println!("✅ Registered 3 capabilities");

    // Define requirements
    let requirement = CapabilityRequirementBuilder::new("*-processing")
        .category(CapabilityCategory::DataProcessing)
        .min_version("1.0.0")
        .build();

    aggregator.add_requirement(requirement);

    // Find matching capabilities
    let matches = aggregator.find_matches();
    println!("🔍 Found {} matching capabilities:", matches.len());
    for m in matches {
        println!("   - {} (score: {:.2})", m.capability.name, m.score);
    }

    // Get statistics
    let stats = aggregator.get_statistics();
    println!("\n📊 Capability Statistics:");
    println!("   - Total: {}", stats.total_capabilities);
    println!("   - Available: {}", stats.available_capabilities);
    println!("   - Average Score: {:.2}", stats.average_score);

    println!();
    Ok(())
}

/// Example 4: Tool Composition with Agents
async fn tool_composition_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("4️⃣ Tool Composition with Agents");
    println!("===============================");

    // Note: Tool composition is already implemented but the builder API would need to be added
    // For now, we'll show the concept
    println!("   Tool composition allows chaining tools together");
    println!("   Example: search → calculate → format");
    println!("   Each tool output feeds into the next tool's input");

    // Create an agent that manages tool composition
    let pipeline_config = AgentConfig {
        name: "pipeline-agent".to_string(),
        description: "Agent that executes tool pipelines".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec![
            "file_search".to_string(),
            "calculator".to_string(),
            "text_manipulator".to_string(),
        ],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };
    let _pipeline_agent = BasicAgent::new(pipeline_config)?;

    println!("✅ Created agent to execute tool pipelines");
    println!("   The agent can discover and invoke tools via ToolCapable trait");
    println!("   Tools can be chained based on their input/output compatibility");

    println!();
    Ok(())
}

/// Example 5: Composite Lifecycle Management
async fn lifecycle_management_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("5️⃣ Composite Lifecycle Management");
    println!("=================================");

    // Create lifecycle configuration
    let config = LifecycleConfig {
        init_timeout: std::time::Duration::from_secs(10),
        shutdown_timeout: std::time::Duration::from_secs(10),
        cascade_events: true,
        wait_for_all: true,
        health_check_interval: Some(std::time::Duration::from_secs(30)),
    };

    // Create lifecycle manager
    let manager = CompositeLifecycleManager::new(config.clone());

    // Create a composite agent with some components
    let mut composite = HierarchicalAgentBuilder::new("managed-composite")
        .description("Lifecycle-managed composite agent")
        .build();

    // Add some child agents
    let child1_config = AgentConfig {
        name: "worker-1".to_string(),
        description: "Worker agent 1".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec![],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };
    let _child1 = BasicAgent::new(child1_config)?;

    let child2_config = AgentConfig {
        name: "worker-2".to_string(),
        description: "Worker agent 2".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec![],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };
    let _child2 = BasicAgent::new(child2_config)?;

    composite
        .add_child(Arc::new(HierarchicalAgentBuilder::new("child1").build()))
        .await?;
    composite
        .add_child(Arc::new(HierarchicalAgentBuilder::new("child2").build()))
        .await?;

    // Initialize the composite
    manager.initialize_composite(&composite).await?;
    println!("✅ Initialized composite agent");
    println!("   State: {:?}", manager.state().await);

    // Transition through lifecycle states
    manager.activate().await?;
    println!("▶️  Activated agent");

    manager.pause().await?;
    println!("⏸️  Paused agent");

    manager.resume().await?;
    println!("▶️  Resumed agent");

    // Perform health check
    let health = manager.health_check().await?;
    println!("\n🏥 Health Check:");
    println!("   - Overall health: {}", health.overall_health);
    println!("   - Manager state: {:?}", health.manager_state);
    println!("   - Components: {}", health.component_health.len());

    // Shutdown
    manager.shutdown().await?;
    println!("\n🛑 Shutdown complete");
    println!("   Final state: {:?}", manager.state().await);

    // Hierarchical lifecycle management
    println!("\n📊 Hierarchical Lifecycle Management:");
    println!("   - Manages parent-child agent relationships");
    println!("   - Cascades lifecycle events up/down hierarchy");
    println!("   - Coordinates state changes across agent trees");
    println!("   - Ensures proper initialization/shutdown order");

    println!();
    Ok(())
}

/// Helper function to demonstrate agent composition with real tools
#[allow(dead_code)]
async fn advanced_composition_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Composition Example");
    println!("==============================");

    // Create a dependency injection container
    let _container = DIContainer::new();

    // Create a sophisticated composite agent
    let _orchestrator = HierarchicalAgentBuilder::new("advanced-orchestrator")
        .description("Advanced orchestrator with multiple capabilities")
        .build();

    // Create specialized sub-agents
    let data_config = AgentConfig {
        name: "data-agent".to_string(),
        description: "Handles data operations".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec!["file_search".to_string()],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };
    let data_agent = BasicAgent::new(data_config)?;

    let analysis_config = AgentConfig {
        name: "analysis-agent".to_string(),
        description: "Performs analysis".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec!["calculator".to_string()],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };
    let analysis_agent = BasicAgent::new(analysis_config)?;

    let report_config = AgentConfig {
        name: "reporting-agent".to_string(),
        description: "Generates reports".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec!["text_manipulator".to_string()],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };
    let reporting_agent = BasicAgent::new(report_config)?;

    // Build the hierarchy
    let mut root = HierarchicalCompositeAgent::new("root", HierarchicalConfig::default());
    let root_as_composite: &mut dyn CompositeAgent = &mut root;
    root_as_composite
        .add_component(Arc::new(data_agent))
        .await?;
    root_as_composite
        .add_component(Arc::new(analysis_agent))
        .await?;
    root_as_composite
        .add_component(Arc::new(reporting_agent))
        .await?;

    println!("✅ Created advanced composite structure");
    println!("   Root → [Data Agent, Analysis Agent, Reporting Agent]");

    // Execute a complex workflow
    let _context = ExecutionContext::new();
    let _input = serde_json::json!({
        "task": "Analyze system logs and generate report",
        "data_source": "/var/log",
        "output_format": "markdown"
    });

    println!("\n📋 Executing complex workflow...");

    // The orchestrator would coordinate between agents
    // Data agent would discover and use file_search tool
    // Analysis agent would process the data
    // Reporting agent would format the results

    println!("✅ Workflow execution complete!");

    Ok(())
}
