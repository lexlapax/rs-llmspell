//! ABOUTME: Multi-agent coordinator example demonstrating hierarchical agent coordination
//! ABOUTME: Shows how multiple specialized agents can work together on complex tasks

use llmspell_agents::templates::{
    AgentTemplate, MonitorAgentTemplate, OrchestratorAgentTemplate, TemplateInstantiationParams,
    ToolAgentTemplate,
};
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use std::time::Instant;
use tracing::{info, Level};

/// Example demonstrating multi-agent coordination where specialized agents
/// work together under an orchestrator to accomplish complex research tasks.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Multi-Agent Coordinator Example");

    // Create the main orchestrator agent that will coordinate other agents
    let orchestrator_template = OrchestratorAgentTemplate::new();
    let orchestrator_params = TemplateInstantiationParams::new("coordinator-001".to_string())
        .with_parameter("agent_name", "Research Coordinator".into())
        .with_parameter("orchestration_strategy", "parallel".into())
        .with_parameter("max_managed_agents", 5.into())
        .with_parameter("max_concurrent_workflows", 3.into())
        .with_parameter("enable_health_monitoring", true.into())
        .with_parameter(
            "managed_agent_types",
            vec!["tool_agent", "monitor_agent"].into(),
        );

    let orchestrator_result = orchestrator_template
        .instantiate(orchestrator_params)
        .await?;
    let orchestrator = orchestrator_result.agent;

    // Create specialized agents that will be coordinated

    // 1. Data Collection Agent - specialized in gathering information
    let data_agent_template = ToolAgentTemplate::new();
    let data_agent_params = TemplateInstantiationParams::new("data-agent-001".to_string())
        .with_parameter("agent_name", "Data Collector".into())
        .with_parameter(
            "allowed_tools",
            vec!["file_operations", "web_search"].into(),
        )
        .with_parameter("tool_selection_strategy", "capability_based".into());

    let data_agent_result = data_agent_template.instantiate(data_agent_params).await?;
    let _data_agent = data_agent_result.agent;

    // 2. Analysis Agent - specialized in processing and analyzing data
    let analysis_agent_template = ToolAgentTemplate::new();
    let analysis_agent_params = TemplateInstantiationParams::new("analysis-agent-001".to_string())
        .with_parameter("agent_name", "Data Analyzer".into())
        .with_parameter(
            "allowed_tools",
            vec!["json_processor", "csv_analyzer", "text_manipulator"].into(),
        )
        .with_parameter("enable_caching", true.into());

    let analysis_agent_result = analysis_agent_template
        .instantiate(analysis_agent_params)
        .await?;
    let _analysis_agent = analysis_agent_result.agent;

    // 3. Monitoring Agent - tracks the health and progress of other agents
    let monitor_template = MonitorAgentTemplate::new();
    let monitor_params = TemplateInstantiationParams::new("monitor-agent-001".to_string())
        .with_parameter("agent_name", "System Monitor".into())
        .with_parameter("monitoring_interval", 5.into())
        .with_parameter("alert_threshold", 0.8.into())
        .with_parameter("enable_performance_tracking", true.into());

    let monitor_result = monitor_template.instantiate(monitor_params).await?;
    let _monitor_agent = monitor_result.agent;

    // Example 1: Parallel Task Execution
    println!("\n=== Example 1: Parallel Task Execution ===");
    println!("Demonstrating how multiple agents work on different aspects simultaneously");

    let research_task = AgentInput::text(
        "Research the topic 'sustainable energy solutions' by:\n\
         1. Collecting recent articles and data (Data Agent)\n\
         2. Analyzing trends and patterns (Analysis Agent)\n\
         3. Monitoring resource usage during execution (Monitor Agent)",
    );

    let start = Instant::now();

    // In a real implementation, the orchestrator would delegate to sub-agents
    let orchestrator_output = orchestrator
        .execute(research_task, ExecutionContext::default())
        .await?;

    println!("Orchestrator Result: {}", orchestrator_output.text);
    println!("Execution Time: {:?}", start.elapsed());

    // Example 2: Sequential Pipeline with Dependencies
    println!("\n=== Example 2: Sequential Pipeline with Dependencies ===");
    println!("Showing how agents can depend on outputs from previous agents");

    let pipeline_task = AgentInput::text(
        "Process customer feedback data:\n\
         1. First, load feedback from 'feedback.csv' (Data Agent)\n\
         2. Then, analyze sentiment and categorize issues (Analysis Agent)\n\
         3. Finally, generate performance report (Monitor Agent)",
    );

    let pipeline_output = orchestrator
        .execute(pipeline_task, ExecutionContext::default())
        .await?;
    println!("Pipeline Result: {}", pipeline_output.text);

    // Example 3: Dynamic Agent Allocation
    println!("\n=== Example 3: Dynamic Agent Allocation ===");
    println!("Demonstrating how the coordinator allocates agents based on task requirements");

    let dynamic_task = AgentInput::text(
        "Handle incoming support tickets:\n\
         - If technical issue: assign to specialized technical agent\n\
         - If data request: assign to data agent\n\
         - If performance issue: assign to monitor agent\n\
         - Track all assignments and outcomes",
    );

    let dynamic_output = orchestrator
        .execute(dynamic_task, ExecutionContext::default())
        .await?;
    println!("Dynamic Allocation Result: {}", dynamic_output.text);

    // Example 4: Consensus Building
    println!("\n=== Example 4: Consensus Building ===");
    println!("Multiple agents analyze the same data and reach consensus");

    let consensus_task = AgentInput::text(
        "Analyze market data and provide investment recommendation:\n\
         - Each agent analyzes independently\n\
         - Compare and reconcile different conclusions\n\
         - Provide unified recommendation with confidence score",
    );

    let consensus_output = orchestrator
        .execute(consensus_task, ExecutionContext::default())
        .await?;
    println!("Consensus Result: {}", consensus_output.text);

    // Example 5: Error Recovery with Agent Redundancy
    println!("\n=== Example 5: Error Recovery with Agent Redundancy ===");
    println!("Showing how the system handles agent failures");

    let recovery_task = AgentInput::text(
        "Process critical data with fault tolerance:\n\
         - Primary agent attempts task\n\
         - If failure, secondary agent takes over\n\
         - Monitor tracks all attempts and recovery actions",
    );

    let recovery_output = orchestrator
        .execute(recovery_task, ExecutionContext::default())
        .await?;
    println!("Recovery Result: {}", recovery_output.text);

    // Performance Analysis
    println!("\n=== Multi-Agent Coordination Benefits ===");
    println!("1. Parallel Processing: Multiple agents work simultaneously");
    println!("2. Specialization: Each agent focuses on its strengths");
    println!("3. Fault Tolerance: System continues despite individual failures");
    println!("4. Scalability: Easy to add new specialized agents");
    println!("5. Flexibility: Dynamic task allocation based on requirements");

    // Best Practices
    println!("\n=== Best Practices for Multi-Agent Systems ===");
    println!("1. Clear Communication Protocols: Define how agents share information");
    println!("2. Resource Management: Monitor and limit resource usage");
    println!("3. Error Handling: Implement robust error recovery strategies");
    println!("4. Performance Monitoring: Track individual and system performance");
    println!("5. Task Decomposition: Break complex tasks into agent-appropriate subtasks");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::environment_helpers::create_test_context;

    #[tokio::test]
    async fn test_multi_agent_setup() {
        // Create orchestrator
        let orchestrator_template = OrchestratorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-coordinator".to_string())
            .with_parameter("agent_name", "Test Coordinator".into())
            .with_parameter("max_managed_agents", 3.into());

        let result = orchestrator_template.instantiate(params).await.unwrap();
        assert!(!result.agent.metadata().name.is_empty());
    }

    #[tokio::test]
    async fn test_agent_coordination() {
        // Create multiple agents
        let mut agents = std::collections::HashMap::new();

        // Create data agent
        let data_template = ToolAgentTemplate::new();
        let data_params = TemplateInstantiationParams::new("data-test".to_string())
            .with_parameter("agent_name", "Data Test".into())
            .with_parameter("allowed_tools", vec!["file_operations"].into());
        let data_result = data_template.instantiate(data_params).await.unwrap();
        agents.insert("data", data_result.agent);

        // Create analysis agent
        let analysis_template = ToolAgentTemplate::new();
        let analysis_params = TemplateInstantiationParams::new("analysis-test".to_string())
            .with_parameter("agent_name", "Analysis Test".into())
            .with_parameter("allowed_tools", vec!["json_processor"].into());
        let analysis_result = analysis_template
            .instantiate(analysis_params)
            .await
            .unwrap();
        agents.insert("analysis", analysis_result.agent);

        // Verify all agents created
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let orchestrator_template = OrchestratorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("parallel-test".to_string())
            .with_parameter("agent_name", "Parallel Test".into())
            .with_parameter("orchestration_strategy", "parallel".into())
            .with_parameter("max_concurrent_workflows", 5.into());

        let result = orchestrator_template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Execute multiple tasks in parallel");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_agent_health_monitoring() {
        let monitor_template = MonitorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("health-test".to_string())
            .with_parameter("agent_name", "Health Monitor".into())
            .with_parameter("monitoring_interval", 5.into())
            .with_parameter("enable_performance_tracking", true.into());

        let result = monitor_template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Check system health");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }
}
