//! ABOUTME: Example demonstrating agent monitoring and observability setup
//! ABOUTME: Shows metrics collection, health checks, tracing, logging, and alerting

use llmspell_agents::agents::basic::BasicAgent;
use llmspell_agents::factory::{AgentConfig, ResourceLimits};
use llmspell_agents::{
    AlertCondition, AlertConfig, AlertManager, AlertRule, AlertSeverity, ConsoleLogExporter,
    ConsoleNotificationChannel, ConsoleTraceExporter, EventLogger, HealthMonitor, LogLevel,
    MetricRegistry, PerformanceMonitor, ThresholdOperator, TraceCollector, TraceSpan,
};
use llmspell_core::BaseAgent;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Agent Monitoring & Observability Setup\n");

    // Create an agent to monitor
    let agent_config = AgentConfig {
        name: "monitored-agent".to_string(),
        description: "Agent with comprehensive monitoring".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec![],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };
    let agent = Arc::new(BasicAgent::new(agent_config)?);

    // 1. Metrics Setup
    println!("1️⃣ Setting up Metrics Collection");
    println!("=================================");
    setup_metrics(&agent).await?;

    // 2. Health Monitoring
    println!("\n2️⃣ Setting up Health Monitoring");
    println!("================================");
    setup_health_monitoring(&agent).await?;

    // 3. Performance Tracking
    println!("\n3️⃣ Setting up Performance Tracking");
    println!("==================================");
    setup_performance_tracking(&agent).await?;

    // 4. Distributed Tracing
    println!("\n4️⃣ Setting up Distributed Tracing");
    println!("=================================");
    setup_tracing(&agent).await?;

    // 5. Event Logging
    println!("\n5️⃣ Setting up Event Logging");
    println!("===========================");
    setup_logging(&agent).await?;

    // 6. Alerting
    println!("\n6️⃣ Setting up Alerting Framework");
    println!("================================");
    setup_alerting(&agent).await?;

    // 7. Integrated Example
    println!("\n7️⃣ Integrated Monitoring Example");
    println!("================================");
    integrated_example(&agent).await?;

    Ok(())
}

async fn setup_metrics(agent: &Arc<BasicAgent>) -> Result<(), Box<dyn std::error::Error>> {
    let registry = MetricRegistry::new();

    // Get agent-specific metrics
    let metrics = registry.get_agent_metrics(&agent.metadata().id.to_string());

    // Simulate some activity
    metrics.requests_total.inc_by(100);
    metrics.requests_failed.inc_by(5);
    metrics.tool_invocations.inc_by(25);
    metrics.update_resources(150.0 * 1024.0 * 1024.0, 35.5); // 150MB, 35.5% CPU

    // Simulate request timing
    let timer = metrics.start_request();
    tokio::time::sleep(Duration::from_millis(50)).await;
    metrics.complete_request(timer, true);

    // Collect and display metrics
    let collected = registry.collect();
    println!("📊 Collected Metrics:");
    for (name, value) in collected.iter() {
        match value {
            llmspell_agents::MetricValue::Counter(v) => {
                println!("   {} = {} (counter)", name, v);
            }
            llmspell_agents::MetricValue::Gauge(v) => {
                println!("   {} = {:.2} (gauge)", name, v);
            }
            _ => {}
        }
    }

    Ok(())
}

async fn setup_health_monitoring(
    agent: &Arc<BasicAgent>,
) -> Result<(), Box<dyn std::error::Error>> {
    use llmspell_agents::AgentHealthCheck;

    // Create health monitor
    let mut monitor = HealthMonitor::new(
        Duration::from_secs(30), // Check interval
        Duration::from_secs(5),  // Check timeout
    );

    // Create agent health check
    let health_check = AgentHealthCheck::new(agent.metadata().clone())
        .with_min_memory(50 * 1024 * 1024) // 50MB minimum
        .with_max_cpu(90.0) // 90% max CPU
        .with_max_response_time(2000) // 2 second max response
        .with_min_success_rate(95.0); // 95% success rate

    monitor.register(Arc::new(health_check));

    // Perform health check
    let result = monitor.check_all().await?;
    println!("🏥 Health Check Result: {}", result.summary());

    // Display component health
    for (id, health) in &result.components {
        println!("   Component: {} - Status: {:?}", id, health.status);
        for indicator in &health.indicators {
            println!(
                "      - {}: {:?} {}",
                indicator.name,
                indicator.status,
                indicator.message.as_ref().unwrap_or(&String::new())
            );
        }
    }

    Ok(())
}

async fn setup_performance_tracking(
    agent: &Arc<BasicAgent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = MetricRegistry::new();
    let metrics = registry.get_agent_metrics(&agent.metadata().id.to_string());

    // Create performance monitor
    let monitor = Arc::new(PerformanceMonitor::new(
        agent.metadata().id.to_string(),
        metrics.clone(),
        100,                    // Keep 100 snapshots
        Duration::from_secs(1), // Snapshot interval
    ));

    // Take some snapshots
    for i in 0..5 {
        // Simulate varying load
        metrics.requests_total.inc_by(20 + i * 5);
        metrics.requests_failed.inc_by(i);
        metrics.update_resources(
            (100.0 + (i as f64 * 10.0)) * 1024.0 * 1024.0, // Memory
            20.0 + (i as f64 * 5.0),                       // CPU
        );

        let snapshot = monitor.take_snapshot();
        println!(
            "📸 Snapshot {}: CPU={:.1}%, Memory={:.1}MB, Rate={:.1} req/s",
            i + 1,
            snapshot.resources.cpu_percent,
            snapshot.resources.memory_bytes as f64 / (1024.0 * 1024.0),
            snapshot.request_rate
        );

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Generate performance report
    let report = monitor.generate_report();
    println!("\n📈 Performance Report:");
    print!("{}", report.summary());

    Ok(())
}

async fn setup_tracing(agent: &Arc<BasicAgent>) -> Result<(), Box<dyn std::error::Error>> {
    let mut collector = TraceCollector::new(1000); // Keep 1000 completed spans
    collector.add_exporter(Box::new(ConsoleTraceExporter));
    let collector = Arc::new(collector);

    // Create a root span
    let root_span = TraceSpan::new_root(
        "process_request".to_string(),
        agent.metadata().id.to_string(),
    );
    let trace_id = root_span.trace_id.clone();

    // Start the span
    let root_handle = collector.start_span(root_span.clone());
    root_handle.add_tag("request_id".to_string(), "req-123".to_string());

    // Create child spans
    let child1 = root_span.new_child("validate_input".to_string());
    let child1_handle = collector.start_span(child1);
    tokio::time::sleep(Duration::from_millis(10)).await;
    child1_handle.complete_ok();

    let child2 = root_span.new_child("process_data".to_string());
    let child2_handle = collector.start_span(child2);
    tokio::time::sleep(Duration::from_millis(20)).await;
    child2_handle.complete_ok();

    let child3 = root_span.new_child("generate_response".to_string());
    let child3_handle = collector.start_span(child3);
    tokio::time::sleep(Duration::from_millis(15)).await;
    child3_handle.complete_ok();

    // Complete root span
    root_handle.complete_ok();

    // Analyze trace
    println!("🔗 Trace Analysis:");
    let trace = collector.get_trace(&trace_id);

    use llmspell_agents::TraceAnalyzer;
    let critical_path = TraceAnalyzer::critical_path(&trace);
    println!("   Critical Path: {} spans", critical_path.len());

    let stats = TraceAnalyzer::trace_stats(&trace);
    println!("   Total Spans: {}", stats.span_count);
    println!("   Total Duration: {:?}", stats.total_duration);
    println!("   Error Rate: {:.1}%", stats.error_rate);

    Ok(())
}

async fn setup_logging(agent: &Arc<BasicAgent>) -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = EventLogger::new(agent.metadata().id.to_string(), 1000);
    logger.set_level(LogLevel::Debug);
    logger.add_exporter(Box::new(ConsoleLogExporter));

    // Add filters
    use llmspell_agents::{ComponentFilter, RateLimitFilter};
    logger.add_filter(Box::new(ComponentFilter::new(vec![
        "auth".to_string(),
        "processing".to_string(),
        "output".to_string(),
    ])));
    logger.add_filter(Box::new(RateLimitFilter::new(100))); // 100 events/second

    // Log various events
    logger.debug("auth", "Starting authentication")?;
    logger.info("auth", "User authenticated successfully")?;
    logger.warn("processing", "High memory usage detected")?;

    use llmspell_agents::ErrorDetails;
    let error = ErrorDetails {
        error_type: "ValidationError".to_string(),
        message: "Invalid input format".to_string(),
        stack_trace: None,
        context: vec![("field".to_string(), "email".to_string())]
            .into_iter()
            .collect(),
    };
    logger.error("processing", "Validation failed", Some(error))?;

    // Get statistics
    let stats = logger.get_statistics();
    println!("📝 Logging Statistics:");
    println!("   Total Events: {}", stats.total_events);
    println!("   Buffer Utilization: {:.1}%", stats.buffer_utilization);
    println!("   Events by Level:");
    for (level, count) in &stats.level_counts {
        println!("      {:?}: {}", level, count);
    }

    Ok(())
}

async fn setup_alerting(agent: &Arc<BasicAgent>) -> Result<(), Box<dyn std::error::Error>> {
    let manager = AlertManager::new(AlertConfig::default());

    // Register notification channel
    manager.register_channel("console".to_string(), Arc::new(ConsoleNotificationChannel));

    // Define alert rules
    let high_cpu_rule = AlertRule {
        id: "high-cpu".to_string(),
        name: "High CPU Usage".to_string(),
        description: "CPU usage exceeded threshold".to_string(),
        severity: AlertSeverity::Warning,
        condition: AlertCondition::MetricThreshold {
            metric_name: "agent.monitored-agent.cpu_percent".to_string(),
            operator: ThresholdOperator::GreaterThan,
            threshold: 80.0,
            duration: Duration::from_secs(60),
        },
        cooldown: Duration::from_secs(300),
        enabled: true,
        channels: vec!["console".to_string()],
    };
    manager.register_rule(high_cpu_rule);

    let high_error_rate_rule = AlertRule {
        id: "high-errors".to_string(),
        name: "High Error Rate".to_string(),
        description: "Error rate exceeded 5%".to_string(),
        severity: AlertSeverity::Critical,
        condition: AlertCondition::ErrorRate {
            rate_percent: 5.0,
            duration: Duration::from_secs(60),
        },
        cooldown: Duration::from_secs(300),
        enabled: true,
        channels: vec!["console".to_string()],
    };
    manager.register_rule(high_error_rate_rule);

    // Create metrics for evaluation
    let registry = MetricRegistry::new();
    let agent_metrics = registry.get_agent_metrics(&agent.metadata().id.to_string());
    agent_metrics.update_resources(200.0 * 1024.0 * 1024.0, 85.0); // Trigger high CPU
    agent_metrics.requests_total.inc_by(100);
    agent_metrics.requests_failed.inc_by(7); // 7% error rate

    // Evaluate rules
    use llmspell_agents::{AlertContext, MonitoringHealthStatus as HealthStatus};
    let metrics = registry.collect();
    let context = AlertContext {
        metrics: &metrics,
        health: Some(&HealthStatus::Degraded),
        performance_violations: &[],
        agent_id: &agent.metadata().id.to_string(),
    };

    manager.evaluate_rules(context).await?;

    // Display active alerts
    let active_alerts = manager.get_active_alerts();
    println!("🚨 Active Alerts: {}", active_alerts.len());
    for alert in &active_alerts {
        println!(
            "   {} [{}] {} - {}",
            alert.severity.color(),
            alert.severity,
            alert.title,
            alert.description
        );
    }

    // Get alert statistics
    let stats = manager.get_statistics();
    println!("\n📊 Alert Statistics:");
    println!("   Active Alerts: {}", stats.active_count);
    println!("   Total Triggered: {}", stats.total_triggered);

    Ok(())
}

async fn integrated_example(agent: &Arc<BasicAgent>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running integrated monitoring example...");

    // Create all monitoring components
    let registry = Arc::new(MetricRegistry::new());
    let metrics = registry.get_agent_metrics(&agent.metadata().id.to_string());
    let logger = Arc::new(EventLogger::new(agent.metadata().id.to_string(), 1000));
    let collector = Arc::new(TraceCollector::new(1000));

    // Simulate agent activity with full monitoring
    for i in 0..3 {
        // Start trace
        let span = TraceSpan::new_root(format!("request-{}", i), agent.metadata().id.to_string());
        let span_handle = collector.start_span(span);

        // Log start
        logger.info("request", &format!("Processing request {}", i))?;

        // Update metrics
        let timer = metrics.start_request();

        // Simulate processing
        tokio::time::sleep(Duration::from_millis(50 + i * 10)).await;

        // Random failure
        let success = i != 1;
        metrics.complete_request(timer, success);

        if success {
            span_handle.complete_ok();
            logger.info("request", &format!("Request {} completed successfully", i))?;
        } else {
            span_handle.complete_error();
            logger.error("request", &format!("Request {} failed", i), None)?;
        }
    }

    // Display integrated view
    println!("\n🎯 Integrated Monitoring Summary:");
    println!(
        "   Requests: {} total, {} failed",
        metrics.requests_total.get(),
        metrics.requests_failed.get()
    );
    println!("   Active Traces: {}", collector.active_span_count());
    println!("   Log Events: {}", logger.get_statistics().total_events);

    Ok(())
}
