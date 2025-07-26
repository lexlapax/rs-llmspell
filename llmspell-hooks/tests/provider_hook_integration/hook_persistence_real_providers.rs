// ABOUTME: Example demonstrating hook persistence and replay with real LLM providers
// ABOUTME: Shows how hooks capture, persist, and replay interactions with OpenAI and Anthropic

use anyhow::Result;
use chrono::Utc;
use llmspell_agents::{
    anthropic::AnthropicAgent,
    openai::OpenAIAgent,
    traits::Agent,
};
use llmspell_hooks::{
    builtin::*,
    context::HookContext,
    executor::HookExecutor,
    replay::{HookReplay, ReplayOptions, Timeline},
    storage::ReplayableHookStorage,
    traits::Hook,
    types::{ComponentId, ComponentType, HookPoint},
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use uuid::Uuid;

/// Example configuration
struct ExampleConfig {
    openai_key: Option<String>,
    anthropic_key: Option<String>,
    use_mock: bool,
}

impl ExampleConfig {
    fn from_env() -> Self {
        Self {
            openai_key: std::env::var("OPENAI_API_KEY").ok(),
            anthropic_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            use_mock: std::env::var("USE_MOCK").is_ok(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    let config = ExampleConfig::from_env();
    
    println!("=== LLMSpell Hook Persistence Example ===\n");
    
    // Set up hook storage and replay system
    let storage_path = std::env::temp_dir().join("llmspell_hook_example");
    let storage = Arc::new(ReplayableHookStorage::new(&storage_path).await?);
    let replay = Arc::new(HookReplay::new(storage.clone()));
    let executor = Arc::new(HookExecutor::new());
    
    // Register all builtin hooks
    register_builtin_hooks(&executor);
    
    // Example 1: OpenAI interaction with hooks
    if config.openai_key.is_some() || config.use_mock {
        println!("## Example 1: OpenAI Chat with Hook Persistence\n");
        
        let correlation_id = Uuid::new_v4();
        let result = demonstrate_openai_hooks(
            &config,
            &executor,
            &storage,
            correlation_id,
        ).await?;
        
        println!("OpenAI Response: {}\n", result);
        
        // Show captured hooks
        print_captured_hooks(&storage, &correlation_id).await?;
        
        // Replay the interaction
        println!("\n### Replaying OpenAI Interaction\n");
        replay_interaction(&replay, &correlation_id).await?;
    }
    
    // Example 2: Anthropic interaction with hooks
    if config.anthropic_key.is_some() || config.use_mock {
        println!("\n## Example 2: Anthropic Chat with Hook Persistence\n");
        
        let correlation_id = Uuid::new_v4();
        let result = demonstrate_anthropic_hooks(
            &config,
            &executor,
            &storage,
            correlation_id,
        ).await?;
        
        println!("Anthropic Response: {}\n", result);
        
        // Show captured hooks
        print_captured_hooks(&storage, &correlation_id).await?;
        
        // Timeline reconstruction
        println!("\n### Timeline Reconstruction\n");
        reconstruct_timeline(&storage, &correlation_id).await?;
    }
    
    // Example 3: Multi-provider workflow with correlation
    if (config.openai_key.is_some() && config.anthropic_key.is_some()) || config.use_mock {
        println!("\n## Example 3: Multi-Provider Workflow\n");
        
        let workflow_id = Uuid::new_v4();
        demonstrate_multi_provider_workflow(
            &config,
            &executor,
            &storage,
            workflow_id,
        ).await?;
        
        // Show correlated events
        println!("\n### Correlated Events Across Providers\n");
        show_correlated_events(&storage, &workflow_id).await?;
    }
    
    // Example 4: Hook replay with modifications
    println!("\n## Example 4: Replay with Modifications\n");
    demonstrate_replay_modifications(&storage, &replay).await?;
    
    println!("\n=== Example Complete ===");
    Ok(())
}

/// Register all builtin hooks
fn register_builtin_hooks(executor: &HookExecutor) {
    executor.register_hook(Arc::new(LoggingHook::new()) as Arc<dyn Hook>);
    executor.register_hook(Arc::new(MetricsHook::new()) as Arc<dyn Hook>);
    executor.register_hook(Arc::new(SecurityHook::new()) as Arc<dyn Hook>);
    executor.register_hook(Arc::new(CachingHook::new()) as Arc<dyn Hook>);
    executor.register_hook(Arc::new(RateLimitHook::new()) as Arc<dyn Hook>);
    executor.register_hook(Arc::new(CostTrackingHook::new()) as Arc<dyn Hook>);
    executor.register_hook(Arc::new(RetryHook::new()) as Arc<dyn Hook>);
    executor.register_hook(Arc::new(DebuggingHook::new()) as Arc<dyn Hook>);
}

/// Demonstrate OpenAI hooks
async fn demonstrate_openai_hooks(
    config: &ExampleConfig,
    executor: &HookExecutor,
    storage: &Arc<ReplayableHookStorage>,
    correlation_id: Uuid,
) -> Result<String> {
    // Create OpenAI agent
    let agent = if let Some(key) = &config.openai_key {
        OpenAIAgent::builder()
            .api_key(key.clone())
            .model("gpt-3.5-turbo")
            .temperature(0.7)
            .build()?
    } else {
        // Mock agent for testing
        return Ok("Mock OpenAI response: Hello from mock GPT!".to_string());
    };
    
    // Create hook context
    let component_id = ComponentId::new(ComponentType::Agent, "openai-example".to_string());
    let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);
    context.correlation_id = correlation_id;
    
    // Add request data
    context.insert_data(
        "request".to_string(),
        json!({
            "model": "gpt-3.5-turbo",
            "messages": [{
                "role": "user",
                "content": "Write a haiku about hooks in software"
            }],
            "temperature": 0.7,
            "max_tokens": 50
        }),
    );
    
    // Execute pre-hooks
    executor.execute(&mut context).await?;
    persist_replayable_hooks(executor, storage, &context).await?;
    
    // Execute agent
    let start_time = std::time::Instant::now();
    let response = agent.execute(&json!({
        "messages": [{
            "role": "user",
            "content": "Write a haiku about hooks in software"
        }]
    })).await?;
    let execution_time = start_time.elapsed();
    
    // Update context with response
    context.point = HookPoint::AfterAgentExecution;
    context.insert_data("response".to_string(), response.clone());
    context.insert_data(
        "usage".to_string(),
        json!({
            "prompt_tokens": 20,
            "completion_tokens": 30,
            "total_tokens": 50
        }),
    );
    context.insert_metadata("execution_time_ms".to_string(), execution_time.as_millis().to_string());
    
    // Execute post-hooks
    executor.execute(&mut context).await?;
    persist_replayable_hooks(executor, storage, &context).await?;
    
    Ok(response.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("No content")
        .to_string())
}

/// Demonstrate Anthropic hooks
async fn demonstrate_anthropic_hooks(
    config: &ExampleConfig,
    executor: &HookExecutor,
    storage: &Arc<ReplayableHookStorage>,
    correlation_id: Uuid,
) -> Result<String> {
    // Create Anthropic agent
    let agent = if let Some(key) = &config.anthropic_key {
        AnthropicAgent::builder()
            .api_key(key.clone())
            .model("claude-3-haiku-20240307")
            .temperature(0.5)
            .build()?
    } else {
        // Mock agent for testing
        return Ok("Mock Anthropic response: Hello from mock Claude!".to_string());
    };
    
    // Create hook context
    let component_id = ComponentId::new(ComponentType::Agent, "anthropic-example".to_string());
    let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);
    context.correlation_id = correlation_id;
    
    // Add request data
    context.insert_data(
        "request".to_string(),
        json!({
            "model": "claude-3-haiku-20240307",
            "messages": [{
                "role": "user",
                "content": "Explain hook patterns in 2 sentences"
            }],
            "temperature": 0.5,
            "max_tokens": 100
        }),
    );
    
    // Execute hooks and agent (similar to OpenAI example)
    executor.execute(&mut context).await?;
    persist_replayable_hooks(executor, storage, &context).await?;
    
    let response = if !config.use_mock {
        agent.execute(&json!({
            "messages": [{
                "role": "user",
                "content": "Explain hook patterns in 2 sentences"
            }]
        })).await?
    } else {
        json!({
            "content": "Hook patterns allow code injection at specific points. They enable extensibility without modifying core logic."
        })
    };
    
    context.point = HookPoint::AfterAgentExecution;
    context.insert_data("response".to_string(), response.clone());
    
    executor.execute(&mut context).await?;
    persist_replayable_hooks(executor, storage, &context).await?;
    
    Ok(response.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("No content")
        .to_string())
}

/// Demonstrate multi-provider workflow
async fn demonstrate_multi_provider_workflow(
    config: &ExampleConfig,
    executor: &HookExecutor,
    storage: &Arc<ReplayableHookStorage>,
    workflow_id: Uuid,
) -> Result<()> {
    println!("Starting multi-provider analysis workflow...\n");
    
    // Step 1: OpenAI generates questions
    let mut openai_context = HookContext::new(
        HookPoint::BeforeAgentExecution,
        ComponentId::new(ComponentType::Agent, "openai-questioner".to_string()),
    );
    openai_context.correlation_id = workflow_id;
    openai_context.insert_metadata("workflow_step".to_string(), "1_generate_questions".to_string());
    
    executor.execute(&mut openai_context).await?;
    persist_replayable_hooks(executor, storage, &openai_context).await?;
    
    println!("✓ OpenAI generated analysis questions");
    
    // Step 2: Anthropic provides answers
    let mut anthropic_context = HookContext::new(
        HookPoint::BeforeAgentExecution,
        ComponentId::new(ComponentType::Agent, "anthropic-analyzer".to_string()),
    );
    anthropic_context.correlation_id = workflow_id;
    anthropic_context.insert_metadata("workflow_step".to_string(), "2_analyze_questions".to_string());
    anthropic_context.insert_metadata("triggered_by".to_string(), "openai-questioner".to_string());
    
    executor.execute(&mut anthropic_context).await?;
    persist_replayable_hooks(executor, storage, &anthropic_context).await?;
    
    println!("✓ Anthropic analyzed the questions");
    
    // Step 3: OpenAI summarizes
    let mut summary_context = HookContext::new(
        HookPoint::BeforeAgentExecution,
        ComponentId::new(ComponentType::Agent, "openai-summarizer".to_string()),
    );
    summary_context.correlation_id = workflow_id;
    summary_context.insert_metadata("workflow_step".to_string(), "3_summarize_results".to_string());
    
    executor.execute(&mut summary_context).await?;
    persist_replayable_hooks(executor, storage, &summary_context).await?;
    
    println!("✓ OpenAI created final summary");
    
    Ok(())
}

/// Persist replayable hooks
async fn persist_replayable_hooks(
    executor: &HookExecutor,
    storage: &Arc<ReplayableHookStorage>,
    context: &HookContext,
) -> Result<()> {
    for hook in executor.get_hooks() {
        if let Some(replayable) = hook.as_any().downcast_ref::<dyn ReplayableHook>() {
            if replayable.is_replayable() {
                let serialized = replayable.serialize_context(context)?;
                storage.store(
                    &context.correlation_id.to_string(),
                    &replayable.replay_id(),
                    &serialized,
                ).await?;
            }
        }
    }
    Ok(())
}

/// Print captured hooks
async fn print_captured_hooks(
    storage: &Arc<ReplayableHookStorage>,
    correlation_id: &Uuid,
) -> Result<()> {
    let hooks = storage.load_by_correlation_id(&correlation_id.to_string()).await?;
    
    println!("### Captured Hooks ({} total):", hooks.len());
    for (replay_id, _) in hooks.iter().take(5) {
        let hook_name = replay_id.split(':').next().unwrap_or("Unknown");
        println!("  - {}", hook_name);
    }
    if hooks.len() > 5 {
        println!("  ... and {} more", hooks.len() - 5);
    }
    
    Ok(())
}

/// Replay an interaction
async fn replay_interaction(
    replay: &Arc<HookReplay>,
    correlation_id: &Uuid,
) -> Result<()> {
    let options = ReplayOptions::default();
    let contexts = replay.replay_by_correlation_id(&correlation_id.to_string(), options).await?;
    
    println!("Replayed {} hook contexts:", contexts.len());
    for (idx, context) in contexts.iter().enumerate().take(3) {
        println!("  {}. {} at {:?}", 
            idx + 1,
            context.component_id.name,
            context.point
        );
    }
    
    Ok(())
}

/// Reconstruct timeline
async fn reconstruct_timeline(
    storage: &Arc<ReplayableHookStorage>,
    correlation_id: &Uuid,
) -> Result<()> {
    let timeline = Timeline::reconstruct_from_correlation_id(
        storage,
        &correlation_id.to_string(),
    ).await?;
    
    println!("Timeline has {} events over {:?}", 
        timeline.events.len(),
        timeline.total_duration()
    );
    
    for event in timeline.events.iter().take(5) {
        println!("  {} - {} ({:?})",
            event.timestamp.format("%H:%M:%S%.3f"),
            event.component_name,
            event.hook_point
        );
    }
    
    Ok(())
}

/// Show correlated events
async fn show_correlated_events(
    storage: &Arc<ReplayableHookStorage>,
    workflow_id: &Uuid,
) -> Result<()> {
    let events = storage.load_by_correlation_id(&workflow_id.to_string()).await?;
    
    let mut components = std::collections::HashSet::new();
    let mut steps = std::collections::HashSet::new();
    
    for (_, data) in &events {
        if let Ok(context) = serde_json::from_slice::<HookContext>(data) {
            components.insert(context.component_id.name.clone());
            if let Some(step) = context.metadata.get("workflow_step") {
                steps.insert(step.clone());
            }
        }
    }
    
    println!("Workflow involved {} components:", components.len());
    for component in &components {
        println!("  - {}", component);
    }
    
    println!("\nWorkflow steps:");
    let mut sorted_steps: Vec<_> = steps.into_iter().collect();
    sorted_steps.sort();
    for step in &sorted_steps {
        println!("  - {}", step);
    }
    
    Ok(())
}

/// Demonstrate replay with modifications
async fn demonstrate_replay_modifications(
    storage: &Arc<ReplayableHookStorage>,
    replay: &Arc<HookReplay>,
) -> Result<()> {
    // Find a correlation ID to replay
    let correlation_ids = storage.list_correlation_ids().await?;
    if let Some(correlation_id) = correlation_ids.first() {
        println!("Replaying correlation {} with modifications...", correlation_id);
        
        // Replay with modified temperature
        let mut options = ReplayOptions::default();
        options.modify_context = Some(Box::new(|ctx| {
            if let Some(request) = ctx.data.get_mut("request") {
                if let Some(obj) = request.as_object_mut() {
                    obj.insert("temperature".to_string(), json!(0.1));
                    obj.insert("modified".to_string(), json!(true));
                }
            }
        }));
        
        let contexts = replay.replay_by_correlation_id(correlation_id, options).await?;
        
        println!("Modified {} contexts during replay", contexts.len());
        
        // Check modifications
        for context in contexts.iter().take(2) {
            if let Some(request) = context.data.get("request") {
                if let Some(modified) = request.get("modified") {
                    println!("  ✓ Context was modified: temperature set to {}", 
                        request.get("temperature").unwrap_or(&json!(null))
                    );
                }
            }
        }
    } else {
        println!("No previous interactions to replay");
    }
    
    Ok(())
}