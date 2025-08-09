//! ABOUTME: Monitoring agent example demonstrating system health tracking and alerting
//! ABOUTME: Shows how agents can monitor other agents, resources, and generate alerts

use llmspell_agents::templates::{
    AgentTemplate, MonitorAgentTemplate, TemplateInstantiationParams, ToolAgentTemplate,
};
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use std::time::{Duration, Instant};
use tracing::{info, warn, Level};

/// Example demonstrating a monitoring agent that tracks system health,
/// agent performance, and generates alerts based on thresholds.
#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Monitoring Agent Example");

    // Create the monitoring agent with custom configuration
    let monitor_template = MonitorAgentTemplate::new();
    let monitor_params = TemplateInstantiationParams::new("monitor-001".to_string())
        .with_parameter("agent_name", "System Health Monitor".into())
        .with_parameter("monitoring_interval", 5.into()) // Check every 5 seconds
        .with_parameter("alert_threshold", 0.75.into()) // Alert at 75% resource usage
        .with_parameter("enable_performance_tracking", true.into())
        .with_parameter("enable_health_checks", true.into())
        .with_parameter("enable_resource_tracking", true.into())
        .with_parameter("history_retention", 3600.into()) // Keep 1 hour of history
        .with_parameter(
            "monitored_agents",
            vec!["data-processor", "analyzer", "orchestrator"].into(),
        );

    let monitor_result = monitor_template.instantiate(monitor_params).await?;
    let monitor = monitor_result.agent;

    // Create some agents to monitor (for demonstration)
    let tool_template = ToolAgentTemplate::new();
    let tool_params = TemplateInstantiationParams::new("data-processor".to_string())
        .with_parameter("agent_name", "Data Processor".into())
        .with_parameter(
            "allowed_tools",
            vec!["file_operations", "json_processor"].into(),
        );
    let _tool_agent = tool_template.instantiate(tool_params).await?.agent;

    // Example 1: Basic Health Check
    println!("\n=== Example 1: Basic Health Check ===");

    let health_check = AgentInput::text("Check system health status");
    let health_output = monitor
        .execute(health_check, ExecutionContext::default())
        .await?;

    println!("Health Status:\n{}", health_output.text);

    // Example 2: Resource Monitoring
    println!("\n=== Example 2: Resource Monitoring ===");

    let resource_check = AgentInput::text(
        "Monitor resource usage:\n\
         - CPU utilization\n\
         - Memory consumption\n\
         - Disk I/O\n\
         - Network activity",
    );

    let resource_output = monitor
        .execute(resource_check, ExecutionContext::default())
        .await?;
    println!("Resource Report:\n{}", resource_output.text);

    // Example 3: Agent Performance Tracking
    println!("\n=== Example 3: Agent Performance Tracking ===");

    let performance_check = AgentInput::text(
        "Analyze agent performance metrics:\n\
         - Response times\n\
         - Success rates\n\
         - Error rates\n\
         - Throughput",
    );

    let performance_output = monitor
        .execute(performance_check, ExecutionContext::default())
        .await?;
    println!("Performance Analysis:\n{}", performance_output.text);

    // Example 4: Alert Generation
    println!("\n=== Example 4: Alert Generation ===");

    // Simulate high resource usage scenario
    let alert_scenario = AgentInput::text(
        "Simulate alert conditions:\n\
         - CPU usage: 85%\n\
         - Memory usage: 90%\n\
         - Agent 'data-processor' not responding\n\
         Generate appropriate alerts and recommendations.",
    );

    let alert_output = monitor
        .execute(alert_scenario, ExecutionContext::default())
        .await?;
    println!("Alert Status:\n{}", alert_output.text);

    // Check if alerts were generated (mock check based on output content)
    if alert_output.text.contains("alert") || alert_output.text.contains("Alert") {
        warn!("⚠️  ALERTS GENERATED - Immediate attention required!");
    }

    // Example 5: Historical Trend Analysis
    println!("\n=== Example 5: Historical Trend Analysis ===");

    let trend_analysis = AgentInput::text(
        "Analyze historical trends:\n\
         - Resource usage over last hour\n\
         - Performance degradation patterns\n\
         - Recurring issues\n\
         - Predictive analysis",
    );

    let trend_output = monitor
        .execute(trend_analysis, ExecutionContext::default())
        .await?;
    println!("Trend Analysis:\n{}", trend_output.text);

    // Example 6: Real-time Monitoring Simulation
    println!("\n=== Example 6: Real-time Monitoring Simulation ===");
    println!("Starting 30-second monitoring session...\n");

    let start_time = Instant::now();
    let monitoring_duration = Duration::from_secs(30);

    // Simulate real-time monitoring
    while start_time.elapsed() < monitoring_duration {
        // Quick health check
        let quick_check = AgentInput::text("Quick health pulse check");
        let pulse_output = monitor
            .execute(quick_check, ExecutionContext::default())
            .await?;

        // Extract key metrics (in a real implementation)
        let elapsed = start_time.elapsed().as_secs();
        println!(
            "[{:02}s] Status: {} | CPU: Mock% | Memory: Mock% | Alerts: 0",
            elapsed,
            if pulse_output.text.contains("healthy") {
                "✓ Healthy"
            } else {
                "⚠ Warning"
            }
        );

        // Wait before next check
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    // Example 7: Generate Monitoring Report
    println!("\n=== Example 7: Comprehensive Monitoring Report ===");

    let report_request = AgentInput::text(
        "Generate comprehensive monitoring report including:\n\
         - Executive summary\n\
         - Key metrics and KPIs\n\
         - Critical alerts and issues\n\
         - Resource utilization charts\n\
         - Performance benchmarks\n\
         - Recommendations for optimization",
    );

    let report_output = monitor
        .execute(report_request, ExecutionContext::default())
        .await?;
    println!("Monitoring Report:\n{}", report_output.text);

    // Best Practices Summary
    println!("\n=== Monitoring Best Practices ===");
    println!("1. Set Appropriate Thresholds: Balance sensitivity vs alert fatigue");
    println!("2. Monitor Key Metrics: Focus on business-critical indicators");
    println!("3. Enable Trend Analysis: Detect issues before they become critical");
    println!("4. Automate Responses: Set up automatic remediation for common issues");
    println!("5. Regular Reviews: Periodically review and adjust monitoring strategy");

    // Configuration Tips
    println!("\n=== Configuration Tips ===");
    println!("- Monitoring Interval: Lower for critical systems (1-5s)");
    println!("- Alert Thresholds: Start conservative and tune based on patterns");
    println!("- History Retention: Balance storage vs analysis needs");
    println!("- Aggregation: Use appropriate time windows for different metrics");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::environment_helpers::create_test_context;

    #[tokio::test]
    async fn test_monitor_agent_creation() {
        let template = MonitorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-monitor".to_string())
            .with_parameter("agent_name", "Test Monitor".into())
            .with_parameter("monitoring_interval", 10.into());

        let result = template.instantiate(params).await.unwrap();
        assert!(!result.agent.metadata().name.is_empty());
    }

    #[tokio::test]
    async fn test_health_check() {
        let template = MonitorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("health-monitor".to_string())
            .with_parameter("agent_name", "Health Monitor".into())
            .with_parameter("enable_health_checks", true.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Check health");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_alert_generation() {
        let template = MonitorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("alert-monitor".to_string())
            .with_parameter("agent_name", "Alert Monitor".into())
            .with_parameter("alert_threshold", 0.5.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Check for alerts with high resource usage");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_performance_tracking() {
        let template = MonitorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("perf-monitor".to_string())
            .with_parameter("agent_name", "Performance Monitor".into())
            .with_parameter("enable_performance_tracking", true.into())
            .with_parameter("history_retention", 7200.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Analyze performance metrics");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_resource_monitoring() {
        let template = MonitorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("resource-monitor".to_string())
            .with_parameter("agent_name", "Resource Monitor".into())
            .with_parameter("enable_resource_tracking", true.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Monitor CPU and memory usage");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }
}
