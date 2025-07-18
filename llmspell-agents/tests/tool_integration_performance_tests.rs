//! ABOUTME: Performance tests for tool integration infrastructure
//! ABOUTME: Ensures tool discovery, invocation, and composition meet performance requirements

use llmspell_agents::composition::{CompositionStep, DataFlow, ToolComposition};
use llmspell_agents::{
    AgentWrappedTool, InvocationConfig, ToolDiscoveryService, ToolInvoker, ToolManager,
    ToolManagerConfig, ToolSearchCriteria,
};
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, Result,
};
use llmspell_tools::{CalculatorTool, ToolRegistry};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance benchmark thresholds
const MAX_TOOL_DISCOVERY_TIME: Duration = Duration::from_millis(5);
const MAX_TOOL_INVOCATION_OVERHEAD: Duration = Duration::from_millis(5);
const MAX_AGENT_WRAPPING_TIME: Duration = Duration::from_millis(2);
const MAX_COMPOSITION_SETUP_TIME: Duration = Duration::from_millis(10);
const MAX_REGISTRY_OPERATIONS_TIME: Duration = Duration::from_millis(3);

/// Mock agent for performance testing
struct PerformanceTestAgent {
    metadata: ComponentMetadata,
    execution_time: Duration,
}

impl PerformanceTestAgent {
    fn new(name: &str, execution_time: Duration) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name.to_string(),
                format!("Performance test agent: {}", name),
            ),
            execution_time,
        }
    }
}

#[async_trait::async_trait]
impl BaseAgent for PerformanceTestAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Simulate work by sleeping for the specified duration
        tokio::time::sleep(self.execution_time).await;
        Ok(AgentOutput::text(format!("Processed: {}", input.text)))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Error: {}", error)))
    }
}

/// Mock fast tool for performance testing
struct FastMockTool {
    metadata: ComponentMetadata,
}

impl FastMockTool {
    fn new(name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(name.to_string(), format!("Fast mock tool: {}", name)),
        }
    }
}

#[async_trait::async_trait]
impl BaseAgent for FastMockTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Very fast execution - just echo input
        Ok(AgentOutput::text(format!("Fast: {}", input.text)))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Error: {}", error)))
    }
}

#[async_trait::async_trait]
impl Tool for FastMockTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Input text".to_string(),
            required: true,
            default: None,
        })
    }
}

#[tokio::test]
async fn test_tool_discovery_performance() {
    // Setup
    let registry = Arc::new(ToolRegistry::new());
    let discovery = ToolDiscoveryService::new(registry.clone());

    // Register some tools for discovery
    for i in 0..100 {
        let tool = FastMockTool::new(&format!("tool_{}", i));
        registry
            .register(format!("tool_{}", i), tool)
            .await
            .unwrap();
    }

    // Benchmark tool discovery
    let criteria = ToolSearchCriteria::new()
        .with_category("utility")
        .with_text_search("tool");

    let start = Instant::now();
    let tools = discovery.find_by_criteria(&criteria).await.unwrap();
    let discovery_time = start.elapsed();

    println!(
        "Tool discovery time: {:?} for {} tools",
        discovery_time,
        tools.len()
    );
    assert!(
        discovery_time < MAX_TOOL_DISCOVERY_TIME,
        "Tool discovery took {:?}, expected < {:?}",
        discovery_time,
        MAX_TOOL_DISCOVERY_TIME
    );
    assert!(!tools.is_empty(), "Should discover some tools");
}

#[tokio::test]
async fn test_tool_invocation_overhead() {
    // Setup
    let agent = Arc::new(PerformanceTestAgent::new(
        "fast_agent",
        Duration::from_micros(100),
    ));
    let wrapped_tool =
        AgentWrappedTool::new(agent.clone(), ToolCategory::Utility, SecurityLevel::Safe);
    let invoker = ToolInvoker::new(InvocationConfig::default());

    let context = ExecutionContext::new();
    let params = json!({"text": "test input"});

    // Measure baseline agent execution time
    let baseline_start = Instant::now();
    let _baseline_result = agent
        .execute(AgentInput::text("test input".to_string()), context.clone())
        .await
        .unwrap();
    let baseline_time = baseline_start.elapsed();

    // Measure tool invocation time
    let invocation_start = Instant::now();
    let _invocation_result = invoker
        .invoke(Arc::new(wrapped_tool), params, context)
        .await
        .unwrap();
    let invocation_time = invocation_start.elapsed();

    let overhead = invocation_time.saturating_sub(baseline_time);

    println!("Baseline execution: {:?}", baseline_time);
    println!("Tool invocation time: {:?}", invocation_time);
    println!("Invocation overhead: {:?}", overhead);

    assert!(
        overhead < MAX_TOOL_INVOCATION_OVERHEAD,
        "Tool invocation overhead was {:?}, expected < {:?}",
        overhead,
        MAX_TOOL_INVOCATION_OVERHEAD
    );
}

#[tokio::test]
async fn test_agent_wrapping_performance() {
    let agent = Arc::new(PerformanceTestAgent::new(
        "wrap_test_agent",
        Duration::from_millis(1),
    ));

    // Benchmark agent wrapping
    let start = Instant::now();
    let _wrapped_tool = AgentWrappedTool::new(agent, ToolCategory::Utility, SecurityLevel::Safe);
    let wrapping_time = start.elapsed();

    println!("Agent wrapping time: {:?}", wrapping_time);
    assert!(
        wrapping_time < MAX_AGENT_WRAPPING_TIME,
        "Agent wrapping took {:?}, expected < {:?}",
        wrapping_time,
        MAX_AGENT_WRAPPING_TIME
    );
}

#[tokio::test]
async fn test_registry_operations_performance() {
    let registry = ToolRegistry::new();

    // Benchmark tool registration
    let registration_start = Instant::now();
    let tool = FastMockTool::new("perf_test_tool");
    registry
        .register("perf_test_tool".to_string(), tool)
        .await
        .unwrap();
    let registration_time = registration_start.elapsed();

    // Benchmark tool lookup
    let lookup_start = Instant::now();
    let _tool = registry.get_tool("perf_test_tool").await;
    let lookup_time = lookup_start.elapsed();

    // Benchmark tool info retrieval
    let info_start = Instant::now();
    let _info = registry.get_tool_info("perf_test_tool").await;
    let info_time = info_start.elapsed();

    let total_time = registration_time + lookup_time + info_time;

    println!("Registry registration: {:?}", registration_time);
    println!("Registry lookup: {:?}", lookup_time);
    println!("Registry info retrieval: {:?}", info_time);
    println!("Total registry operations: {:?}", total_time);

    assert!(
        total_time < MAX_REGISTRY_OPERATIONS_TIME,
        "Registry operations took {:?}, expected < {:?}",
        total_time,
        MAX_REGISTRY_OPERATIONS_TIME
    );
}

#[tokio::test]
async fn test_composition_setup_performance() {
    // Benchmark composition creation and setup
    let setup_start = Instant::now();

    let mut composition = ToolComposition::new("performance_test_composition");

    // Add multiple steps
    for i in 0..5 {
        composition.add_step(
            CompositionStep::new(format!("step_{}", i), format!("tool_{}", i))
                .with_input_mapping("input", DataFlow::Parameter("data".to_string())),
        );
    }

    let setup_time = setup_start.elapsed();

    println!("Composition setup time: {:?}", setup_time);
    assert!(
        setup_time < MAX_COMPOSITION_SETUP_TIME,
        "Composition setup took {:?}, expected < {:?}",
        setup_time,
        MAX_COMPOSITION_SETUP_TIME
    );
}

#[tokio::test]
async fn test_tool_manager_performance() {
    let registry = Arc::new(ToolRegistry::new());

    // Register a test tool
    let calculator = CalculatorTool::new();
    registry
        .register("calculator".to_string(), calculator)
        .await
        .unwrap();

    let manager = ToolManager::new(registry);

    // Benchmark tool discovery
    let discovery_start = Instant::now();
    let tools = manager.discover_tools(&Default::default()).await.unwrap();
    let discovery_time = discovery_start.elapsed();

    // Benchmark tool invocation
    let context = ExecutionContext::new();
    let params = json!({"expression": "2 + 2"});

    let invocation_start = Instant::now();
    let _result = manager
        .invoke_tool("calculator", params, context)
        .await
        .unwrap();
    let invocation_time = invocation_start.elapsed();

    println!("Manager discovery time: {:?}", discovery_time);
    println!("Manager invocation time: {:?}", invocation_time);

    assert!(
        discovery_time < MAX_TOOL_DISCOVERY_TIME,
        "Manager discovery took {:?}, expected < {:?}",
        discovery_time,
        MAX_TOOL_DISCOVERY_TIME
    );

    assert!(
        invocation_time < Duration::from_millis(50), // More lenient for actual tool execution
        "Manager invocation took {:?}, expected < 50ms",
        invocation_time
    );

    assert!(!tools.is_empty(), "Should discover tools");
}

#[tokio::test]
async fn test_concurrent_tool_operations() {
    let registry = Arc::new(ToolRegistry::new());

    // Register tools concurrently
    let mut registration_handles = Vec::new();
    for i in 0..10 {
        let registry = registry.clone();
        let handle = tokio::spawn(async move {
            let tool = FastMockTool::new(&format!("concurrent_tool_{}", i));
            registry
                .register(format!("concurrent_tool_{}", i), tool)
                .await
                .unwrap();
        });
        registration_handles.push(handle);
    }

    let registration_start = Instant::now();
    for handle in registration_handles {
        handle.await.unwrap();
    }
    let concurrent_registration_time = registration_start.elapsed();

    // Perform concurrent lookups
    let mut lookup_handles = Vec::new();
    for i in 0..10 {
        let registry = registry.clone();
        let handle =
            tokio::spawn(async move { registry.get_tool(&format!("concurrent_tool_{}", i)).await });
        lookup_handles.push(handle);
    }

    let lookup_start = Instant::now();
    for handle in lookup_handles {
        let _tool = handle.await.unwrap();
    }
    let concurrent_lookup_time = lookup_start.elapsed();

    println!(
        "Concurrent registration time: {:?}",
        concurrent_registration_time
    );
    println!("Concurrent lookup time: {:?}", concurrent_lookup_time);

    // More lenient timing for concurrent operations
    assert!(
        concurrent_registration_time < Duration::from_millis(50),
        "Concurrent registration took {:?}, expected < 50ms",
        concurrent_registration_time
    );

    assert!(
        concurrent_lookup_time < Duration::from_millis(20),
        "Concurrent lookup took {:?}, expected < 20ms",
        concurrent_lookup_time
    );
}

#[tokio::test]
async fn test_memory_efficiency() {
    // Test that we don't have memory leaks in tool operations
    let registry = Arc::new(ToolRegistry::new());

    // Register and unregister tools multiple times
    for iteration in 0..100 {
        let tool = FastMockTool::new(&format!("memory_test_{}", iteration));
        registry
            .register(format!("memory_test_{}", iteration), tool)
            .await
            .unwrap();

        // Immediately unregister to test cleanup
        registry
            .unregister_tool(&format!("memory_test_{}", iteration))
            .await
            .unwrap();
    }

    // Registry should be empty after cleanup
    let stats = registry.get_statistics().await;
    assert_eq!(
        stats.total_tools, 0,
        "Registry should be clean after unregistering all tools"
    );
}

#[tokio::test]
async fn test_error_handling_performance() {
    let registry = Arc::new(ToolRegistry::new());
    let manager = ToolManager::new(registry);

    let context = ExecutionContext::new();
    let params = json!({"invalid": "parameters"});

    // Benchmark error handling performance
    let error_start = Instant::now();
    let result = manager
        .invoke_tool("nonexistent_tool", params, context)
        .await;
    let error_time = error_start.elapsed();

    println!("Error handling time: {:?}", error_time);

    // Error handling should be fast
    assert!(
        error_time < Duration::from_millis(5),
        "Error handling took {:?}, expected < 5ms",
        error_time
    );

    // Should return an error
    assert!(result.is_err(), "Should return error for nonexistent tool");
}

/// Integration performance test combining multiple operations
#[tokio::test]
async fn test_full_integration_performance() {
    let registry = Arc::new(ToolRegistry::new());

    // Setup: Register calculator tool
    let calculator = CalculatorTool::new();
    registry
        .register("calculator".to_string(), calculator)
        .await
        .unwrap();

    // Setup: Create and register agent-wrapped tool
    let agent = Arc::new(PerformanceTestAgent::new(
        "integration_agent",
        Duration::from_micros(500),
    ));
    let wrapped_tool = AgentWrappedTool::new(agent, ToolCategory::Utility, SecurityLevel::Safe);
    registry
        .register("agent_tool".to_string(), wrapped_tool)
        .await
        .unwrap();

    let manager = ToolManager::new(registry.clone());

    // Full integration test
    let integration_start = Instant::now();

    // 1. Discover tools
    let tools = manager.discover_tools(&Default::default()).await.unwrap();

    // 2. Invoke calculator tool
    let calc_result = manager
        .invoke_tool(
            "calculator",
            json!({"expression": "10 * 5"}),
            ExecutionContext::new(),
        )
        .await
        .unwrap();

    // 3. Invoke agent tool
    let agent_result = manager
        .invoke_tool(
            "agent_tool",
            json!({"text": "integration test"}),
            ExecutionContext::new(),
        )
        .await
        .unwrap();

    let total_integration_time = integration_start.elapsed();

    println!("Full integration time: {:?}", total_integration_time);
    println!("Discovered {} tools", tools.len());
    println!("Calculator result: {}", calc_result.text);
    println!("Agent result: {}", agent_result.text);

    // Full integration should complete quickly
    assert!(
        total_integration_time < Duration::from_millis(100),
        "Full integration took {:?}, expected < 100ms",
        total_integration_time
    );

    assert_eq!(tools.len(), 2, "Should discover both tools");
    assert!(
        calc_result.text.contains("50"),
        "Calculator should compute 10 * 5 = 50"
    );
    assert!(
        agent_result.text.contains("integration test"),
        "Agent should process input"
    );
}
