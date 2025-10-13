//! Template command implementation
//!
//! This module provides CLI commands for template operations - listing, info, execution, search, and schema.

use anyhow::{anyhow, Result};
use serde_json::json;
use std::path::PathBuf;
use tracing::{info, instrument, trace};

use crate::cli::{OutputFormat, TemplateCommands};
use crate::output::OutputFormatter;
use llmspell_config::LLMSpellConfig;
use llmspell_templates::{
    core::TemplateResult, registry::global_registry, ExecutionContext, TemplateCategory,
    TemplateParams,
};

/// Handle template management commands
#[instrument(skip(runtime_config))]
pub async fn handle_template_command(
    command: TemplateCommands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling template command: {:?}", command);

    match command {
        TemplateCommands::List { category, format } => {
            handle_list(category, format.unwrap_or(output_format)).await
        }
        TemplateCommands::Info { name, show_schema } => {
            handle_info(name, show_schema, output_format).await
        }
        TemplateCommands::Exec {
            name,
            params,
            output,
        } => handle_exec(name, params, output, runtime_config, output_format).await,
        TemplateCommands::Search { query, category } => {
            handle_search(query, category, output_format).await
        }
        TemplateCommands::Schema { name } => handle_schema(name, output_format).await,
    }
}

/// Handle template list command
async fn handle_list(category: Option<String>, output_format: OutputFormat) -> Result<()> {
    info!("Listing templates");
    let registry = global_registry();

    // Get templates by category if specified
    let metadata_list = if let Some(cat_str) = category {
        let category = parse_category(&cat_str)?;
        registry.discover_by_category(&category)
    } else {
        registry.list_metadata()
    };

    // Format output
    let formatter = OutputFormatter::new(output_format);

    match output_format {
        OutputFormat::Json => {
            let json_output = json!({
                "templates": metadata_list.iter().map(|m| {
                    json!({
                        "id": m.id,
                        "name": m.name,
                        "description": m.description,
                        "category": format!("{}", m.category),
                        "version": m.version,
                        "tags": m.tags,
                    })
                }).collect::<Vec<_>>()
            });
            formatter.print_json(&json_output)?;
        }
        OutputFormat::Pretty | OutputFormat::Text => {
            if metadata_list.is_empty() {
                println!("No templates found");
            } else {
                println!("\nAvailable Templates:");
                println!("{}", "=".repeat(80));
                for metadata in metadata_list {
                    println!("\n  {} ({})", metadata.name, metadata.id);
                    println!("  Category: {}", metadata.category);
                    println!("  Version:  {}", metadata.version);
                    println!("  Description: {}", metadata.description);
                    if !metadata.tags.is_empty() {
                        println!("  Tags: {}", metadata.tags.join(", "));
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}

/// Handle template info command
async fn handle_info(name: String, show_schema: bool, output_format: OutputFormat) -> Result<()> {
    info!("Getting info for template: {}", name);
    let registry = global_registry();

    // Get template
    let template = registry.get(&name)?;
    let metadata = template.metadata();

    // Format output
    let formatter = OutputFormatter::new(output_format);

    match output_format {
        OutputFormat::Json => {
            let mut json_output = json!({
                "id": metadata.id,
                "name": metadata.name,
                "description": metadata.description,
                "category": format!("{}", metadata.category),
                "version": metadata.version,
                "author": metadata.author,
                "requires": metadata.requires,
                "tags": metadata.tags,
            });

            if show_schema {
                let schema = template.config_schema();
                json_output["schema"] = serde_json::to_value(schema)?;
            }

            formatter.print_json(&json_output)?;
        }
        OutputFormat::Pretty | OutputFormat::Text => {
            println!("\nTemplate: {} ({})", metadata.name, metadata.id);
            println!("{}", "=".repeat(80));
            println!("Category:    {}", metadata.category);
            println!("Version:     {}", metadata.version);
            if let Some(author) = &metadata.author {
                println!("Author:      {}", author);
            }
            println!("Description: {}", metadata.description);

            if !metadata.requires.is_empty() {
                println!("\nRequires:");
                for req in &metadata.requires {
                    println!("  - {}", req);
                }
            }

            if !metadata.tags.is_empty() {
                println!("\nTags: {}", metadata.tags.join(", "));
            }

            if show_schema {
                println!("\nParameter Schema:");
                println!("{}", "-".repeat(80));
                let schema = template.config_schema();
                for param in &schema.parameters {
                    println!("\n  {} ({:?})", param.name, param.param_type);
                    println!("    {}", param.description);
                    println!("    Required: {}", param.required);
                    if let Some(default) = &param.default {
                        println!("    Default:  {}", default);
                    }
                    if let Some(constraints) = &param.constraints {
                        if constraints.min.is_some()
                            || constraints.max.is_some()
                            || constraints.min_length.is_some()
                            || constraints.max_length.is_some()
                        {
                            print!("    Constraints:");
                            if let Some(min) = constraints.min {
                                print!(" min={}", min);
                            }
                            if let Some(max) = constraints.max {
                                print!(" max={}", max);
                            }
                            if let Some(min_len) = constraints.min_length {
                                print!(" min_length={}", min_len);
                            }
                            if let Some(max_len) = constraints.max_length {
                                print!(" max_length={}", max_len);
                            }
                            println!();
                        }
                    }
                }
            }

            println!();
        }
    }

    Ok(())
}

/// Handle template exec command
async fn handle_exec(
    name: String,
    params: Vec<(String, String)>,
    output_dir: Option<PathBuf>,
    runtime_config: LLMSpellConfig,
    _output_format: OutputFormat,
) -> Result<()> {
    info!("Executing template: {}", name);
    let registry = global_registry();

    // Get template
    let template = registry.get(&name)?;

    // Parse parameters
    let mut template_params = TemplateParams::new();
    for (key, value) in params {
        // Try parsing as JSON first, fallback to string
        let json_value = serde_json::from_str(&value).unwrap_or_else(|_| json!(value));
        template_params.insert(key, json_value);
    }

    // Validate parameters
    template.validate(&template_params)?;

    // Build execution context
    let context = build_execution_context(runtime_config, output_dir.clone()).await?;

    // Execute template
    info!("Starting template execution");
    let start = std::time::Instant::now();
    let output = template.execute(template_params, context).await?;
    let duration = start.elapsed();

    // Display results
    println!(
        "\n✓ Template execution completed in {:.2}s",
        duration.as_secs_f64()
    );
    println!("{}", "=".repeat(80));

    // Show result
    match output.result {
        TemplateResult::Text(text) => {
            println!("\nResult:\n{}", text);
        }
        TemplateResult::Structured(value) => {
            println!("\nResult (JSON):");
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        TemplateResult::File(path) => {
            println!("\nResult file: {}", path.display());
        }
        TemplateResult::Multiple(results) => {
            println!("\nMultiple results ({} items)", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("\n  Result {}:", i + 1);
                match result {
                    TemplateResult::Text(text) => {
                        println!("    {}", text);
                    }
                    TemplateResult::File(path) => {
                        println!("    File: {}", path.display());
                    }
                    _ => {}
                }
            }
        }
    }

    // Show artifacts
    if !output.artifacts.is_empty() {
        println!("\nArtifacts ({}):", output.artifacts.len());
        for artifact in &output.artifacts {
            println!(
                "  - {} ({}, {} bytes)",
                artifact.filename,
                artifact.mime_type,
                artifact.size()
            );

            // Write artifacts to output directory if specified
            if let Some(output_path) = &output_dir {
                let artifact_path = artifact.write_to_file(output_path)?;
                println!("    Written to: {}", artifact_path.display());
            }
        }
    }

    // Show metrics
    println!("\nMetrics:");
    println!(
        "  Duration:      {:.2}s",
        output.metrics.duration_ms as f64 / 1000.0
    );
    if let Some(tokens) = output.metrics.tokens_used {
        println!("  Tokens:        {}", tokens);
    }
    if let Some(cost) = output.metrics.cost_usd {
        println!("  Cost:          ${:.4}", cost);
    }
    if output.metrics.agents_invoked > 0 {
        println!("  Agents:        {}", output.metrics.agents_invoked);
    }
    if output.metrics.tools_invoked > 0 {
        println!("  Tools:         {}", output.metrics.tools_invoked);
    }
    if output.metrics.rag_queries > 0 {
        println!("  RAG queries:   {}", output.metrics.rag_queries);
    }

    println!();
    Ok(())
}

/// Handle template search command
async fn handle_search(
    query: Vec<String>,
    category: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Searching templates with query: {:?}", query);
    let registry = global_registry();

    // Join query words
    let query_str = query.join(" ");

    // Search templates
    let mut results = registry.search(&query_str);

    // Filter by category if specified
    if let Some(cat_str) = category {
        let category = parse_category(&cat_str)?;
        results.retain(|m| m.category == category);
    }

    // Format output
    let formatter = OutputFormatter::new(output_format);

    match output_format {
        OutputFormat::Json => {
            let json_output = json!({
                "query": query_str,
                "results": results.iter().map(|m| {
                    json!({
                        "id": m.id,
                        "name": m.name,
                        "description": m.description,
                        "category": format!("{}", m.category),
                        "tags": m.tags,
                    })
                }).collect::<Vec<_>>()
            });
            formatter.print_json(&json_output)?;
        }
        OutputFormat::Pretty | OutputFormat::Text => {
            if results.is_empty() {
                println!("No templates found matching '{}'", query_str);
            } else {
                println!("\nSearch Results ({} templates):", results.len());
                println!("{}", "=".repeat(80));
                for metadata in results {
                    println!("\n  {} ({})", metadata.name, metadata.id);
                    println!("  Category: {}", metadata.category);
                    println!("  Description: {}", metadata.description);
                }
                println!();
            }
        }
    }

    Ok(())
}

/// Handle template schema command
async fn handle_schema(name: String, _output_format: OutputFormat) -> Result<()> {
    info!("Getting schema for template: {}", name);
    let registry = global_registry();

    // Get template
    let template = registry.get(&name)?;
    let schema = template.config_schema();

    // Always output as JSON for schema
    let formatter = OutputFormatter::new(OutputFormat::Json);
    let schema_json = serde_json::to_value(schema)?;
    formatter.print_json(&schema_json)?;

    Ok(())
}

/// Parse category string to enum
fn parse_category(s: &str) -> Result<TemplateCategory> {
    match s.to_lowercase().as_str() {
        "research" => Ok(TemplateCategory::Research),
        "chat" => Ok(TemplateCategory::Chat),
        "analysis" => Ok(TemplateCategory::Analysis),
        "codegen" => Ok(TemplateCategory::CodeGen),
        "document" => Ok(TemplateCategory::Document),
        "workflow" => Ok(TemplateCategory::Workflow),
        _ => Err(anyhow!("Invalid category: {}. Valid categories: Research, Chat, Analysis, CodeGen, Document, Workflow", s)),
    }
}

/// Build ExecutionContext from runtime config
async fn build_execution_context(
    _config: LLMSpellConfig,
    output_dir: Option<PathBuf>,
) -> Result<ExecutionContext> {
    use llmspell_agents::FactoryRegistry as AgentRegistry;
    use llmspell_providers::ProviderManager;
    use llmspell_tools::ToolRegistry;
    use llmspell_workflows::WorkflowFactory;
    use std::sync::Arc;

    // Initialize core components (minimal setup for templates)
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(AgentRegistry::new());

    // Create workflow factory
    let workflow_factory: Arc<dyn WorkflowFactory> =
        Arc::new(llmspell_workflows::DefaultWorkflowFactory::new());

    // Initialize provider manager
    let provider_manager = Arc::new(ProviderManager::new());

    // Build ExecutionContext with required components only
    let mut builder = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager);

    // Add output directory if specified
    if let Some(output_path) = output_dir {
        builder = builder.with_output_dir(output_path);
    }

    Ok(builder.build()?)
}
