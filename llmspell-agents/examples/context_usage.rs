//! ABOUTME: Example demonstrating enhanced ExecutionContext usage
//! ABOUTME: Shows hierarchical contexts, inheritance, shared memory, and event integration

use async_trait::async_trait;
use llmspell_agents::context::event_integration::{EventHandler, LoggingEventHandler};
use llmspell_agents::context::inheritance::FieldTransform;
use llmspell_agents::context::shared_memory::MemoryPermission;
use llmspell_agents::context::{
    ContextEvent, ContextEventBus, HierarchicalContext, InheritanceRules, SharedMemoryManager,
};
use llmspell_core::execution_context::{
    ContextScope, ExecutionContext, ExecutionContextBuilder, InheritancePolicy,
};
use llmspell_core::{ComponentId, LLMSpellError, Result};
use serde_json::json;
use std::sync::Arc;

/// Custom event handler for demonstration
struct CustomEventHandler;

#[async_trait]
impl EventHandler for CustomEventHandler {
    async fn handle(&self, event: ContextEvent, context: ExecutionContext) -> Result<()> {
        println!("Custom handler received event: {}", event.event_type);
        println!("  From context: {:?}", context.scope);
        println!("  Event payload: {}", event.payload);
        Ok(())
    }

    fn event_types(&self) -> Vec<String> {
        vec!["agent_started".to_string(), "task_completed".to_string()]
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Enhanced ExecutionContext Examples ===\n");

    // Example 1: Basic Context Creation
    println!("1. Basic Context Creation:");
    basic_context_example()?;
    println!();

    // Example 2: Hierarchical Contexts
    println!("2. Hierarchical Context Management:");
    hierarchical_context_example().await?;
    println!();

    // Example 3: Context Inheritance
    println!("3. Context Inheritance Rules:");
    inheritance_example()?;
    println!();

    // Example 4: Shared Memory
    println!("4. Shared Memory for Inter-Agent Communication:");
    shared_memory_example().await?;
    println!();

    // Example 5: Event Integration
    println!("5. Event Bus Integration:");
    event_integration_example().await?;
    println!();

    // Example 6: Complete Workflow
    println!("6. Complete Agent Workflow with Context:");
    complete_workflow_example().await?;

    Ok(())
}

fn basic_context_example() -> Result<()> {
    // Create context with builder
    let context = ExecutionContextBuilder::new()
        .conversation_id("conv-123".to_string())
        .user_id("user-456".to_string())
        .session_id("session-789".to_string())
        .scope(ContextScope::Session("session-789".to_string()))
        .data("language".to_string(), json!("en"))
        .data("theme".to_string(), json!("dark"))
        .build();

    println!("  Created context: {}", context.id);
    println!("  Scope: {:?}", context.scope);
    println!("  Language: {:?}", context.get("language"));
    println!("  Theme: {:?}", context.get("theme"));

    Ok(())
}

async fn hierarchical_context_example() -> Result<()> {
    let mut hierarchy = HierarchicalContext::new();

    // Create root context for a workflow
    let workflow_ctx = ExecutionContext::new()
        .with_scope(ContextScope::Workflow("workflow-1".to_string()))
        .with_data(
            "workflow_name".to_string(),
            json!("Data Processing Pipeline"),
        )
        .with_data("max_retries".to_string(), json!(3));

    let workflow_id = hierarchy.create_root("main_workflow".to_string(), workflow_ctx)?;
    println!("  Created workflow context: {}", workflow_id);

    // Create child contexts for agents
    let agent1_ctx = hierarchy.create_child(
        &workflow_id,
        ContextScope::Agent(ComponentId::from_name("data-extractor")),
        InheritancePolicy::Inherit,
    )?;
    println!("  Created agent context: {}", agent1_ctx.id);
    println!(
        "  Agent inherited workflow_name: {:?}",
        agent1_ctx.get("workflow_name")
    );

    let agent2_ctx = hierarchy.create_child(
        &workflow_id,
        ContextScope::Agent(ComponentId::from_name("data-transformer")),
        InheritancePolicy::Copy,
    )?;
    println!("  Created another agent context: {}", agent2_ctx.id);

    // Get hierarchy stats
    let stats = hierarchy.stats();
    println!("  Total contexts: {}", stats.total_contexts);
    println!("  Max depth: {}", stats.max_depth);

    Ok(())
}

fn inheritance_example() -> Result<()> {
    // Set up inheritance rules
    let rules = InheritanceRules::new()
        .always_inherit("api_key".to_string())
        .never_inherit("temp_data".to_string())
        .conditional_inherit(
            "cache_enabled".to_string(),
            vec![InheritancePolicy::Inherit],
        )
        .with_transform(
            "task_id".to_string(),
            FieldTransform::Prefix("child_".to_string()),
        )
        .max_depth(5);

    // Create parent context
    let parent = ExecutionContext::new()
        .with_data("api_key".to_string(), json!("secret-key-123"))
        .with_data("temp_data".to_string(), json!("temporary"))
        .with_data("cache_enabled".to_string(), json!(true))
        .with_data("task_id".to_string(), json!("task-001"));

    // Create child with Inherit policy
    let mut child = parent.create_child(
        ContextScope::Agent(ComponentId::from_name("child-agent")),
        InheritancePolicy::Inherit,
    );

    // Apply inheritance rules
    rules
        .apply(&parent, &mut child)
        .map_err(|e| LLMSpellError::Component {
            message: e,
            source: None,
        })?;

    println!("  Parent task_id: {:?}", parent.get("task_id"));
    println!("  Child task_id: {:?}", child.get("task_id"));
    println!("  Child has api_key: {}", child.get("api_key").is_some());
    println!(
        "  Child has temp_data: {}",
        child.get("temp_data").is_none()
    );

    Ok(())
}

async fn shared_memory_example() -> Result<()> {
    let memory_manager = SharedMemoryManager::new();

    let agent1 = ComponentId::from_name("agent-1");
    let agent2 = ComponentId::from_name("agent-2");

    // Create a shared memory region
    memory_manager.create_region(
        "workflow-state".to_string(),
        ContextScope::Workflow("workflow-1".to_string()),
        agent1.clone(),
    )?;

    // Get the region and grant permissions
    if let Some(region) = memory_manager.get_region("workflow-state") {
        // Grant read permission to agent2
        region.grant_permission(agent2.clone(), MemoryPermission::Read);

        // Agent1 writes data
        region.set(
            "current_step".to_string(),
            json!("data_extraction"),
            &agent1,
        )?;
        region.set("progress".to_string(), json!(0.25), &agent1)?;

        // Agent2 reads data
        let current_step = region.get("current_step", &agent2)?;
        println!("  Agent2 read current_step: {:?}", current_step);

        // Subscribe to changes
        let mut receiver = region.subscribe();

        // Make a change
        region.set("progress".to_string(), json!(0.50), &agent1)?;

        // Check for notification
        if let Ok(change) = receiver.try_recv() {
            println!(
                "  Received change notification: {} changed {}",
                change.changed_by, change.key
            );
        }
    }

    Ok(())
}

async fn event_integration_example() -> Result<()> {
    let event_bus = ContextEventBus::new();

    // Subscribe handlers
    let logging_handler = Arc::new(LoggingEventHandler);
    let custom_handler = Arc::new(CustomEventHandler);

    let _sub1 = event_bus
        .subscribe(ContextScope::Global, vec![], logging_handler)
        .await?;

    let _sub2 = event_bus
        .subscribe(
            ContextScope::Agent(ComponentId::from_name("agent-1")),
            vec!["agent_started".to_string()],
            custom_handler,
        )
        .await?;

    // Publish events
    let event1 = ContextEvent::new(
        "agent_started".to_string(),
        json!({
            "agent_name": "data-processor",
            "config": {"max_retries": 3}
        }),
        ContextScope::Agent(ComponentId::from_name("agent-1")),
    );

    event_bus.publish(event1).await?;

    // Publish targeted event
    let event2 = ContextEvent::new(
        "task_completed".to_string(),
        json!({
            "task": "data_extraction",
            "records_processed": 1000
        }),
        ContextScope::Global,
    )
    .with_target(ContextScope::Agent(ComponentId::from_name("agent-1")));

    event_bus.publish(event2).await?;

    // Wait for handlers to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Get stats
    let stats = event_bus.stats().await;
    println!(
        "  Event bus stats - Subscriptions: {}, Events: {}",
        stats.subscription_count, stats.event_count
    );

    Ok(())
}

async fn complete_workflow_example() -> Result<()> {
    // Set up infrastructure
    let mut hierarchy = HierarchicalContext::new();
    let memory_manager = SharedMemoryManager::new();
    let event_bus = ContextEventBus::new();

    // Create workflow context
    let workflow_ctx = ExecutionContext::new()
        .with_scope(ContextScope::Workflow("etl-pipeline".to_string()))
        .with_data(
            "pipeline_config".to_string(),
            json!({
                "source": "database",
                "destination": "data_lake",
                "batch_size": 1000
            }),
        );

    let workflow_id = hierarchy.create_root("etl".to_string(), workflow_ctx)?;

    // Create shared memory for pipeline state
    memory_manager.create_region(
        "pipeline-state".to_string(),
        ContextScope::Workflow("etl-pipeline".to_string()),
        ComponentId::from_name("pipeline-controller"),
    )?;

    // Create agent contexts
    let extractor_ctx = hierarchy.create_child(
        &workflow_id,
        ContextScope::Agent(ComponentId::from_name("extractor")),
        InheritancePolicy::Inherit,
    )?;

    let transformer_ctx = hierarchy.create_child(
        &workflow_id,
        ContextScope::Agent(ComponentId::from_name("transformer")),
        InheritancePolicy::Inherit,
    )?;

    // Simulate workflow execution
    println!("  Starting ETL Pipeline...");

    // Extractor starts
    event_bus
        .publish(ContextEvent::new(
            "agent_started".to_string(),
            json!({"agent": "extractor", "phase": "initialization"}),
            extractor_ctx.scope.clone(),
        ))
        .await?;

    // Update shared state
    if let Some(region) = memory_manager.get_region("pipeline-state") {
        let extractor_id = ComponentId::from_name("extractor");
        region.grant_permission(
            extractor_id.clone(),
            llmspell_agents::context::shared_memory::MemoryPermission::ReadWrite,
        );

        region.set(
            "current_phase".to_string(),
            json!("extraction"),
            &extractor_id,
        )?;
        region.set("records_extracted".to_string(), json!(0), &extractor_id)?;
    }

    println!("  Pipeline initialized with contexts:");
    println!("    - Workflow: {}", workflow_id);
    println!("    - Extractor: {}", extractor_ctx.id);
    println!("    - Transformer: {}", transformer_ctx.id);

    Ok(())
}
