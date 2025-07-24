//! ABOUTME: Research agent example demonstrating information gathering and synthesis
//! ABOUTME: Shows how agents can conduct research using web and file resources to generate reports

use llmspell_agents::templates::{AgentTemplate, TemplateInstantiationParams, ToolAgentTemplate};
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use std::time::Instant;
use tracing::{info, Level};

/// Example demonstrating a research agent that gathers information from multiple sources,
/// analyzes data, and synthesizes findings into comprehensive reports.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Research Agent Example");

    // Create a research agent with information gathering capabilities
    let research_template = ToolAgentTemplate::new();
    let research_params = TemplateInstantiationParams::new("research-001".to_string())
        .with_parameter("agent_name", "Research Assistant".into())
        .with_parameter(
            "allowed_tools",
            vec![
                "web_search",
                "web_scraper",
                "file_operations",
                "text_manipulator",
                "json_processor",
                "csv_analyzer",
                "sitemap_crawler",
                "url_analyzer",
            ]
            .into(),
        )
        .with_parameter("tool_selection_strategy", "intelligent".into())
        .with_parameter("enable_caching", true.into())
        .with_parameter("search_depth", 3.into())
        .with_parameter("max_sources", 10.into());

    let research_result = research_template.instantiate(research_params).await?;
    let researcher = research_result.agent;

    // Example 1: Topic Research
    println!("\n=== Example 1: Topic Research ===");

    let topic_research = AgentInput::text(
        "Research the topic 'Quantum Computing Applications in Healthcare':\n\
         1. Search for recent developments (last 2 years)\n\
         2. Find key players and institutions\n\
         3. Identify practical applications\n\
         4. Analyze challenges and limitations\n\
         5. Create a summary report with citations",
    );

    let start = Instant::now();
    let topic_output = researcher
        .execute(topic_research, ExecutionContext::default())
        .await?;
    println!("Research Summary: {}", topic_output.text);
    println!("Research Time: {:?}", start.elapsed());

    // Example 2: Competitive Analysis
    println!("\n=== Example 2: Competitive Analysis ===");

    let competitive_research = AgentInput::text(
        "Conduct competitive analysis for 'AI Writing Assistants':\n\
         1. Identify top 5 competitors\n\
         2. Compare features and pricing\n\
         3. Analyze market positioning\n\
         4. Review user feedback and ratings\n\
         5. Identify gaps and opportunities\n\
         6. Create comparison matrix",
    );

    let competitive_output = researcher
        .execute(competitive_research, ExecutionContext::default())
        .await?;
    println!("Competitive Analysis: {}", competitive_output.text);

    // Example 3: Technical Documentation Research
    println!("\n=== Example 3: Technical Documentation Research ===");

    let tech_research = AgentInput::text(
        "Research and summarize documentation for 'Rust async programming':\n\
         1. Find official documentation\n\
         2. Locate best practices guides\n\
         3. Identify common patterns\n\
         4. Find performance optimization tips\n\
         5. Collect code examples\n\
         6. Create learning roadmap",
    );

    let tech_output = researcher
        .execute(tech_research, ExecutionContext::default())
        .await?;
    println!("Technical Research Summary: {}", tech_output.text);

    // Example 4: Data-Driven Research
    println!("\n=== Example 4: Data-Driven Research ===");

    let data_research = AgentInput::text(
        "Research and analyze data on 'Remote Work Trends 2024':\n\
         1. Find statistical data and surveys\n\
         2. Analyze CSV/JSON datasets if available\n\
         3. Identify key metrics and KPIs\n\
         4. Compare pre/post pandemic trends\n\
         5. Create visualizable insights\n\
         6. Generate executive summary",
    );

    let data_output = researcher
        .execute(data_research, ExecutionContext::default())
        .await?;
    println!("Data Analysis Results: {}", data_output.text);

    // Example 5: Multi-Source Integration
    println!("\n=== Example 5: Multi-Source Integration ===");

    let integration_research = AgentInput::text(
        "Research 'Sustainable Energy Solutions' from multiple perspectives:\n\
         1. Academic papers and research\n\
         2. Industry reports and whitepapers\n\
         3. Government policies and regulations\n\
         4. News and recent developments\n\
         5. Case studies and implementations\n\
         6. Synthesize findings across all sources",
    );

    let integration_output = researcher
        .execute(integration_research, ExecutionContext::default())
        .await?;
    println!("Integrated Research Report: {}", integration_output.text);

    // Example 6: Real-time Information Tracking
    println!("\n=== Example 6: Real-time Information Tracking ===");

    let tracking_research = AgentInput::text(
        "Track real-time information on 'Cryptocurrency Market':\n\
         1. Monitor current prices and trends\n\
         2. Find breaking news and updates\n\
         3. Analyze social media sentiment\n\
         4. Track regulatory changes\n\
         5. Identify emerging patterns\n\
         6. Generate hourly summary",
    );

    let tracking_output = researcher
        .execute(tracking_research, ExecutionContext::default())
        .await?;
    println!("Real-time Tracking Report: {}", tracking_output.text);

    // Example 7: Deep Dive Research
    println!("\n=== Example 7: Deep Dive Research ===");

    let deep_research = AgentInput::text(
        "Conduct deep research on 'CRISPR Gene Editing Ethics':\n\
         1. Historical development timeline\n\
         2. Scientific breakthroughs and limitations\n\
         3. Ethical considerations and debates\n\
         4. Regulatory frameworks worldwide\n\
         5. Future implications and scenarios\n\
         6. Expert opinions and consensus\n\
         7. Create comprehensive report with bibliography",
    );

    let deep_output = researcher
        .execute(deep_research, ExecutionContext::default())
        .await?;
    println!("Deep Research Report: {}", deep_output.text);

    // Research Methodologies
    println!("\n=== Research Methodologies ===");
    println!("1. **Systematic Review**: Comprehensive search with defined criteria");
    println!("2. **Cross-Reference Validation**: Verify information across sources");
    println!("3. **Temporal Analysis**: Track changes and trends over time");
    println!("4. **Source Credibility**: Evaluate reliability of information");
    println!("5. **Synthesis Framework**: Structured approach to combine findings");

    // Best Practices
    println!("\n=== Research Agent Best Practices ===");
    println!("1. **Source Diversity**: Use multiple types of sources");
    println!("2. **Bias Detection**: Identify and account for source bias");
    println!("3. **Citation Management**: Properly track and cite sources");
    println!("4. **Update Frequency**: Regular refreshing of cached data");
    println!("5. **Quality over Quantity**: Focus on relevant, high-quality sources");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::fixtures::create_test_context;

    #[tokio::test]
    async fn test_research_agent_creation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-researcher".to_string())
            .with_parameter("agent_name", "Test Researcher".into())
            .with_parameter(
                "allowed_tools",
                vec!["web_search", "text_manipulator"].into(),
            );

        let result = template.instantiate(params).await.unwrap();
        assert!(!result.agent.metadata().name.is_empty());
    }

    #[tokio::test]
    async fn test_web_research() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("web-researcher".to_string())
            .with_parameter("agent_name", "Web Researcher".into())
            .with_parameter("allowed_tools", vec!["web_search", "web_scraper"].into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Search for information about Rust programming");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_data_analysis_research() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("data-researcher".to_string())
            .with_parameter("agent_name", "Data Researcher".into())
            .with_parameter(
                "allowed_tools",
                vec!["csv_analyzer", "json_processor"].into(),
            );

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Analyze dataset and extract insights");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_multi_source_research() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("multi-researcher".to_string())
            .with_parameter("agent_name", "Multi-Source Researcher".into())
            .with_parameter(
                "allowed_tools",
                vec!["web_search", "file_operations", "text_manipulator"].into(),
            )
            .with_parameter("max_sources", 5.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Research topic from multiple sources");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_research_caching() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("cached-researcher".to_string())
            .with_parameter("agent_name", "Cached Researcher".into())
            .with_parameter("allowed_tools", vec!["web_search"].into())
            .with_parameter("enable_caching", true.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        // First request
        let input = AgentInput::text("Search for cached topic");
        let output1 = agent
            .execute(input.clone(), create_test_context())
            .await
            .unwrap();

        // Second request (should use cache)
        let output2 = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output1.text.is_empty());
        assert!(!output2.text.is_empty());
    }
}
