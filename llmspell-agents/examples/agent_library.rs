//! ABOUTME: Agent library catalog demonstrating reusable agent components and templates
//! ABOUTME: Shows how to create, organize, and reuse agent templates for common patterns

use llmspell_agents::templates::{
    AgentTemplate, MonitorAgentTemplate, OrchestratorAgentTemplate, TemplateInstantiationParams,
    ToolAgentTemplate,
};
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use std::collections::HashMap;
use tracing::{info, Level};

/// Example demonstrating how to build a library of reusable agent templates
/// and patterns that can be customized for specific use cases.
#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Agent Library Catalog Example");

    // Create a catalog of agent templates
    let mut agent_catalog = HashMap::new();

    // Example 1: Customer Service Agent Templates
    println!("\n=== Example 1: Customer Service Agent Library ===");

    // Template 1.1: FAQ Bot
    let _faq_template = ToolAgentTemplate::new();
    let faq_params = TemplateInstantiationParams::new("faq-bot-template".to_string())
        .with_parameter("agent_name", "FAQ Assistant".into())
        .with_parameter(
            "allowed_tools",
            vec!["file_operations", "text_manipulator", "web_search"].into(),
        )
        .with_parameter("response_style", "friendly_professional".into())
        .with_parameter("max_response_length", 200.into());

    agent_catalog.insert("customer_service_faq", faq_params);

    // Template 1.2: Complaint Handler
    let _complaint_template = ToolAgentTemplate::new();
    let complaint_params =
        TemplateInstantiationParams::new("complaint-handler-template".to_string())
            .with_parameter("agent_name", "Complaint Resolution Specialist".into())
            .with_parameter(
                "allowed_tools",
                vec!["email_sender", "database_connector", "text_manipulator"].into(),
            )
            .with_parameter("escalation_threshold", 3.into())
            .with_parameter("sentiment_analysis", true.into());

    agent_catalog.insert("customer_service_complaints", complaint_params);

    println!("Customer Service Templates:");
    println!("- FAQ Bot: Handles common questions with knowledge base");
    println!("- Complaint Handler: Manages complaints with escalation");

    // Example 2: Development Team Agent Templates
    println!("\n=== Example 2: Development Team Agent Library ===");

    // Template 2.1: Code Reviewer
    let _reviewer_template = ToolAgentTemplate::new();
    let reviewer_params = TemplateInstantiationParams::new("code-reviewer-template".to_string())
        .with_parameter("agent_name", "Code Review Assistant".into())
        .with_parameter(
            "allowed_tools",
            vec!["file_operations", "diff_calculator", "process_executor"].into(),
        )
        .with_parameter(
            "review_criteria",
            vec!["style", "security", "performance"].into(),
        )
        .with_parameter("auto_fix_minor_issues", true.into());

    agent_catalog.insert("dev_code_reviewer", reviewer_params);

    // Template 2.2: Documentation Generator
    let _doc_template = ToolAgentTemplate::new();
    let doc_params = TemplateInstantiationParams::new("doc-generator-template".to_string())
        .with_parameter("agent_name", "Documentation Writer".into())
        .with_parameter(
            "allowed_tools",
            vec!["file_operations", "text_manipulator", "template_engine"].into(),
        )
        .with_parameter("doc_formats", vec!["markdown", "html", "pdf"].into())
        .with_parameter("include_examples", true.into());

    agent_catalog.insert("dev_doc_generator", doc_params);

    println!("Development Team Templates:");
    println!("- Code Reviewer: Automated code review with fix suggestions");
    println!("- Documentation Generator: Creates docs from code");

    // Example 3: Data Analysis Agent Templates
    println!("\n=== Example 3: Data Analysis Agent Library ===");

    // Template 3.1: ETL Pipeline
    let _etl_template = ToolAgentTemplate::new();
    let etl_params = TemplateInstantiationParams::new("etl-pipeline-template".to_string())
        .with_parameter("agent_name", "ETL Pipeline Processor".into())
        .with_parameter(
            "allowed_tools",
            vec![
                "csv_analyzer",
                "json_processor",
                "data_validation",
                "file_operations",
            ]
            .into(),
        )
        .with_parameter("batch_processing", true.into())
        .with_parameter("error_handling", "dead_letter_queue".into());

    agent_catalog.insert("data_etl_pipeline", etl_params);

    // Template 3.2: Report Generator
    let _report_template = ToolAgentTemplate::new();
    let report_params = TemplateInstantiationParams::new("report-generator-template".to_string())
        .with_parameter("agent_name", "Analytics Report Generator".into())
        .with_parameter(
            "allowed_tools",
            vec!["csv_analyzer", "calculator", "template_engine"].into(),
        )
        .with_parameter(
            "visualization_types",
            vec!["charts", "tables", "summaries"].into(),
        )
        .with_parameter("schedule", "daily".into());

    agent_catalog.insert("data_report_generator", report_params);

    println!("Data Analysis Templates:");
    println!("- ETL Pipeline: Configurable data processing pipeline");
    println!("- Report Generator: Automated analytics reporting");

    // Example 4: Security Agent Templates
    println!("\n=== Example 4: Security Agent Library ===");

    // Template 4.1: Threat Monitor
    let _threat_template = MonitorAgentTemplate::new();
    let threat_params = TemplateInstantiationParams::new("threat-monitor-template".to_string())
        .with_parameter("agent_name", "Security Threat Monitor".into())
        .with_parameter("monitoring_interval", 1.into())
        .with_parameter(
            "threat_sources",
            vec!["logs", "network", "filesystem"].into(),
        )
        .with_parameter("auto_quarantine", true.into());

    agent_catalog.insert("security_threat_monitor", threat_params);

    // Template 4.2: Compliance Auditor
    let _audit_template = ToolAgentTemplate::new();
    let audit_params = TemplateInstantiationParams::new("compliance-auditor-template".to_string())
        .with_parameter("agent_name", "Compliance Auditor".into())
        .with_parameter(
            "allowed_tools",
            vec!["file_operations", "data_validation", "hash_calculator"].into(),
        )
        .with_parameter("compliance_standards", vec!["GDPR", "HIPAA", "SOC2"].into())
        .with_parameter("generate_audit_trail", true.into());

    agent_catalog.insert("security_compliance_auditor", audit_params);

    println!("Security Templates:");
    println!("- Threat Monitor: Real-time security threat detection");
    println!("- Compliance Auditor: Automated compliance checking");

    // Example 5: Orchestration Templates
    println!("\n=== Example 5: Orchestration Agent Library ===");

    // Template 5.1: Workflow Orchestrator
    let _workflow_template = OrchestratorAgentTemplate::new();
    let workflow_params =
        TemplateInstantiationParams::new("workflow-orchestrator-template".to_string())
            .with_parameter("agent_name", "Workflow Orchestrator".into())
            .with_parameter("orchestration_strategy", "pipeline".into())
            .with_parameter("max_managed_agents", 10.into())
            .with_parameter("enable_rollback", true.into());

    agent_catalog.insert("orchestration_workflow", workflow_params);

    // Template 5.2: Load Balancer
    let _balancer_template = OrchestratorAgentTemplate::new();
    let balancer_params = TemplateInstantiationParams::new("load-balancer-template".to_string())
        .with_parameter("agent_name", "Load Balancer".into())
        .with_parameter("orchestration_strategy", "parallel".into())
        .with_parameter("balancing_algorithm", "round_robin".into())
        .with_parameter("health_check_interval", 5.into());

    agent_catalog.insert("orchestration_load_balancer", balancer_params);

    println!("Orchestration Templates:");
    println!("- Workflow Orchestrator: Complex workflow management");
    println!("- Load Balancer: Distributes work across agents");

    // Example 6: Using Templates from the Catalog
    println!("\n=== Example 6: Using Templates from Catalog ===");

    // Instantiate a customer service FAQ bot
    if let Some(faq_params) = agent_catalog.get("customer_service_faq") {
        let faq_template = ToolAgentTemplate::new();
        let custom_params = faq_params
            .clone()
            .with_parameter("knowledge_base", "products_faq.json".into())
            .with_parameter("language", "en-US".into());

        let faq_result = faq_template.instantiate(custom_params).await?;
        let faq_agent = faq_result.agent;

        let question = AgentInput::text("What is your return policy?");
        let answer = faq_agent
            .execute(question, ExecutionContext::default())
            .await?;
        println!("FAQ Bot Response: {}", answer.text);
    }

    // Example 7: Template Composition
    println!("\n=== Example 7: Template Composition ===");
    println!("Combining multiple templates for complex scenarios:");
    println!("1. Customer Support System = FAQ Bot + Complaint Handler + Escalation Monitor");
    println!("2. CI/CD Pipeline = Code Reviewer + Test Runner + Deployment Orchestrator");
    println!("3. Data Platform = ETL Pipeline + Report Generator + Data Quality Monitor");
    println!("4. Security Suite = Threat Monitor + Compliance Auditor + Incident Responder");

    // Template Customization Guidelines
    println!("\n=== Template Customization Guidelines ===");
    println!("1. **Base Configuration**: Start with standard template");
    println!("2. **Parameter Override**: Customize for specific needs");
    println!("3. **Tool Selection**: Add/remove tools as needed");
    println!("4. **Behavior Tuning**: Adjust thresholds and strategies");
    println!("5. **Integration Points**: Define how templates work together");

    // Best Practices for Agent Libraries
    println!("\n=== Agent Library Best Practices ===");
    println!("1. **Standardize Interfaces**: Consistent parameter naming");
    println!("2. **Version Templates**: Track template evolution");
    println!("3. **Document Patterns**: Clear usage examples");
    println!("4. **Test Templates**: Validate before catalog inclusion");
    println!("5. **Share Knowledge**: Export/import template collections");

    // Catalog Summary
    println!("\n=== Catalog Summary ===");
    println!("Total Templates: {}", agent_catalog.len());
    for (category, _) in &agent_catalog {
        println!("- {category}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::environment_helpers::create_test_context;

    #[tokio::test]
    async fn test_template_catalog() {
        let mut catalog = HashMap::new();

        // Add test template
        let params = TemplateInstantiationParams::new("test-template".to_string())
            .with_parameter("agent_name", "Test Agent".into());
        catalog.insert("test_category", params);

        assert_eq!(catalog.len(), 1);
        assert!(catalog.contains_key("test_category"));
    }

    #[tokio::test]
    async fn test_template_instantiation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("catalog-test".to_string())
            .with_parameter("agent_name", "Catalog Test Agent".into())
            .with_parameter("allowed_tools", vec!["calculator"].into());

        let result = template.instantiate(params).await.unwrap();
        assert!(!result.agent.metadata().name.is_empty());
    }

    #[tokio::test]
    async fn test_template_customization() {
        let base_params = TemplateInstantiationParams::new("base-template".to_string())
            .with_parameter("agent_name", "Base Agent".into());

        // Customize the base template
        let custom_params = base_params
            .with_parameter("custom_field", "custom_value".into())
            .with_parameter("override_field", 42.into());

        assert_eq!(custom_params.parameters.len(), 3);
    }

    #[tokio::test]
    async fn test_multi_template_composition() {
        // Create multiple templates
        let tool_template = ToolAgentTemplate::new();
        let monitor_template = MonitorAgentTemplate::new();
        let orchestrator_template = OrchestratorAgentTemplate::new();

        // Verify each can be instantiated
        let tool_params = TemplateInstantiationParams::new("tool-comp".to_string())
            .with_parameter("agent_name", "Tool Component".into());
        let _tool_result = tool_template.instantiate(tool_params).await.unwrap();

        let monitor_params = TemplateInstantiationParams::new("monitor-comp".to_string())
            .with_parameter("agent_name", "Monitor Component".into());
        let _monitor_result = monitor_template.instantiate(monitor_params).await.unwrap();

        let orchestrator_params = TemplateInstantiationParams::new("orchestrator-comp".to_string())
            .with_parameter("agent_name", "Orchestrator Component".into());
        let _orchestrator_result = orchestrator_template
            .instantiate(orchestrator_params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_catalog_usage() {
        let mut catalog = HashMap::new();

        // Add various templates
        let categories = vec![
            "customer_service",
            "development",
            "data_analysis",
            "security",
            "orchestration",
        ];

        for category in categories {
            let params = TemplateInstantiationParams::new(format!("{}-template", category))
                .with_parameter("agent_name", format!("{} Agent", category).into());
            catalog.insert(category, params);
        }

        // Test retrieval and usage
        if let Some(params) = catalog.get("customer_service") {
            let template = ToolAgentTemplate::new();
            let result = template.instantiate(params.clone()).await.unwrap();
            let agent = result.agent;

            let input = AgentInput::text("Test query");
            let output = agent.execute(input, create_test_context()).await.unwrap();
            assert!(!output.text.is_empty());
        }
    }
}
