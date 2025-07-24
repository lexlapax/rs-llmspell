//! ABOUTME: Data pipeline agent example demonstrating ETL operations with intelligent decision making
//! ABOUTME: Shows how agents can process data through transformation pipelines with conditional logic

use llmspell_agents::templates::{AgentTemplate, TemplateInstantiationParams, ToolAgentTemplate};
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use std::time::Instant;
use tracing::{info, Level};

/// Example demonstrating a data pipeline agent that processes data through
/// Extract-Transform-Load (ETL) operations with intelligent routing and validation.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Data Pipeline Agent Example");

    // Create a data pipeline agent with ETL capabilities
    let pipeline_template = ToolAgentTemplate::new();
    let pipeline_params = TemplateInstantiationParams::new("pipeline-001".to_string())
        .with_parameter("agent_name", "Data Pipeline Processor".into())
        .with_parameter(
            "allowed_tools",
            vec![
                "file_operations",
                "csv_analyzer",
                "json_processor",
                "data_validation",
                "text_manipulator",
                "hash_calculator",
                "archive_handler",
            ]
            .into(),
        )
        .with_parameter("tool_selection_strategy", "task_based".into())
        .with_parameter("enable_caching", true.into())
        .with_parameter("max_cache_size", 100.into())
        .with_parameter("enable_parallel_processing", true.into())
        .with_parameter("batch_size", 1000.into());

    let pipeline_result = pipeline_template.instantiate(pipeline_params).await?;
    let pipeline = pipeline_result.agent;

    // Example 1: Simple CSV to JSON Pipeline
    println!("\n=== Example 1: Simple CSV to JSON Pipeline ===");

    let csv_pipeline = AgentInput::text(
        "Process customer data pipeline:\n\
         1. Extract: Read customer_data.csv\n\
         2. Transform: Clean data (remove nulls, normalize dates)\n\
         3. Transform: Add calculated fields (age from birthdate, customer segment)\n\
         4. Load: Save as customer_processed.json",
    );

    let start = Instant::now();
    let csv_output = pipeline
        .execute(csv_pipeline, ExecutionContext::default())
        .await?;
    println!("Pipeline Result: {}", csv_output.text);
    println!("Execution Time: {:?}", start.elapsed());

    // Example 2: Multi-Source Data Integration
    println!("\n=== Example 2: Multi-Source Data Integration ===");

    let integration_pipeline = AgentInput::text(
        "Integrate data from multiple sources:\n\
         1. Extract: Read sales_data.csv, inventory.json, customers.xml\n\
         2. Transform: Join on product_id and customer_id\n\
         3. Transform: Calculate metrics (revenue, inventory turnover)\n\
         4. Validate: Check data consistency and completeness\n\
         5. Load: Create unified_dataset.parquet",
    );

    let integration_output = pipeline
        .execute(integration_pipeline, ExecutionContext::default())
        .await?;
    println!("Integration Result: {}", integration_output.text);

    // Example 3: Data Quality Pipeline
    println!("\n=== Example 3: Data Quality Pipeline ===");

    let quality_pipeline = AgentInput::text(
        "Execute data quality checks:\n\
         1. Profile: Analyze data_warehouse.csv for patterns\n\
         2. Validate: Check for:\n\
            - Missing values\n\
            - Duplicate records\n\
            - Invalid formats\n\
            - Outliers\n\
         3. Clean: Apply fixes based on rules\n\
         4. Report: Generate quality_report.html",
    );

    let quality_output = pipeline
        .execute(quality_pipeline, ExecutionContext::default())
        .await?;
    println!("Quality Check Result: {}", quality_output.text);

    // Example 4: Conditional Processing Pipeline
    println!("\n=== Example 4: Conditional Processing Pipeline ===");

    let conditional_pipeline = AgentInput::text(
        "Process files with conditional logic:\n\
         1. Scan input directory for data files\n\
         2. For each file:\n\
            - If CSV: Parse and validate against schema\n\
            - If JSON: Extract nested fields and flatten\n\
            - If XML: Convert to JSON format\n\
         3. Apply business rules:\n\
            - If sales > 1000: Flag as high-value\n\
            - If customer_age < 18: Apply youth discount\n\
         4. Route to appropriate output:\n\
            - High-value → priority_queue/\n\
            - Standard → standard_queue/\n\
            - Errors → error_queue/",
    );

    let conditional_output = pipeline
        .execute(conditional_pipeline, ExecutionContext::default())
        .await?;
    println!("Conditional Processing Result: {}", conditional_output.text);

    // Example 5: Real-time Stream Processing
    println!("\n=== Example 5: Real-time Stream Processing ===");

    let stream_pipeline = AgentInput::text(
        "Simulate real-time data stream processing:\n\
         1. Monitor: Watch events.json for new entries\n\
         2. Process: Apply transformations in micro-batches\n\
         3. Aggregate: Calculate rolling metrics (5-min window)\n\
         4. Alert: If threshold exceeded, trigger notification\n\
         5. Store: Append to time-series database",
    );

    let stream_output = pipeline
        .execute(stream_pipeline, ExecutionContext::default())
        .await?;
    println!("Stream Processing Result: {}", stream_output.text);

    // Example 6: Data Migration Pipeline
    println!("\n=== Example 6: Data Migration Pipeline ===");

    let migration_pipeline = AgentInput::text(
        "Migrate legacy data to new format:\n\
         1. Extract: Read from legacy_system/\n\
         2. Transform:\n\
            - Convert date formats (MM/DD/YYYY → ISO 8601)\n\
            - Map old field names to new schema\n\
            - Merge split records\n\
         3. Validate: Ensure no data loss\n\
         4. Load: Write to new_system/ with versioning\n\
         5. Verify: Compare checksums and record counts",
    );

    let migration_output = pipeline
        .execute(migration_pipeline, ExecutionContext::default())
        .await?;
    println!("Migration Result: {}", migration_output.text);

    // Example 7: Analytics Pipeline
    println!("\n=== Example 7: Analytics Pipeline ===");

    let analytics_pipeline = AgentInput::text(
        "Build analytics dataset:\n\
         1. Extract: Pull from multiple data sources\n\
         2. Transform:\n\
            - Denormalize for analytics\n\
            - Create derived features\n\
            - Apply business logic\n\
         3. Enrich:\n\
            - Add external data (weather, holidays)\n\
            - Calculate time-based features\n\
         4. Optimize:\n\
            - Partition by date\n\
            - Create indexes\n\
         5. Load: Push to analytics platform",
    );

    let analytics_output = pipeline
        .execute(analytics_pipeline, ExecutionContext::default())
        .await?;
    println!("Analytics Pipeline Result: {}", analytics_output.text);

    // Pipeline Design Patterns
    println!("\n=== Pipeline Design Patterns ===");
    println!("1. **Fan-out/Fan-in**: Split processing across parallel branches");
    println!("2. **Dead Letter Queue**: Handle failed records separately");
    println!("3. **Checkpointing**: Save progress for recovery");
    println!("4. **Backpressure**: Control flow rate to prevent overload");
    println!("5. **Circuit Breaker**: Stop pipeline on repeated failures");

    // Best Practices
    println!("\n=== Data Pipeline Best Practices ===");
    println!("1. **Idempotency**: Ensure pipelines can be safely re-run");
    println!("2. **Monitoring**: Track metrics at each stage");
    println!("3. **Error Handling**: Graceful degradation and recovery");
    println!("4. **Scalability**: Design for increasing data volumes");
    println!("5. **Documentation**: Clear data lineage and transformations");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::fixtures::create_test_context;

    #[tokio::test]
    async fn test_pipeline_agent_creation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-pipeline".to_string())
            .with_parameter("agent_name", "Test Pipeline".into())
            .with_parameter(
                "allowed_tools",
                vec!["csv_analyzer", "json_processor"].into(),
            );

        let result = template.instantiate(params).await.unwrap();
        assert!(!result.agent.metadata().name.is_empty());
    }

    #[tokio::test]
    async fn test_csv_processing() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("csv-pipeline".to_string())
            .with_parameter("agent_name", "CSV Processor".into())
            .with_parameter("allowed_tools", vec!["csv_analyzer"].into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Process CSV file and extract statistics");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_data_validation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("validation-pipeline".to_string())
            .with_parameter("agent_name", "Data Validator".into())
            .with_parameter("allowed_tools", vec!["data_validation"].into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Validate data quality and consistency");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_multi_format_pipeline() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("multi-format-pipeline".to_string())
            .with_parameter("agent_name", "Format Converter".into())
            .with_parameter(
                "allowed_tools",
                vec!["csv_analyzer", "json_processor", "text_manipulator"].into(),
            );

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Convert between CSV, JSON, and XML formats");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_parallel_processing() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("parallel-pipeline".to_string())
            .with_parameter("agent_name", "Parallel Processor".into())
            .with_parameter(
                "allowed_tools",
                vec!["file_operations", "hash_calculator"].into(),
            )
            .with_parameter("enable_parallel_processing", true.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Process multiple files in parallel");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }
}
