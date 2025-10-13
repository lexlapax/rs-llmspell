//! Template command implementation - sends template requests to kernel
//!
//! This module provides CLI commands for template operations.
//! All template logic is executed in the kernel which has ComponentRegistry access.

use anyhow::{anyhow, Result};
use serde_json::json;
use tracing::{info, instrument, trace};

use crate::cli::{OutputFormat, TemplateCommands};
use crate::execution_context::ExecutionContext;
use crate::output::OutputFormatter;
use llmspell_config::LLMSpellConfig;

/// Handle template management commands by sending requests to kernel
#[instrument(skip(runtime_config), fields(command_type))]
pub async fn handle_template_command(
    command: TemplateCommands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling template command");

    // Resolve execution context (connects to kernel or creates embedded)
    let context = ExecutionContext::resolve(
        None, // No connect string
        None, // No port
        None, // No daemon config
        runtime_config.clone(),
    )
    .await?;

    match context {
        ExecutionContext::Embedded { handle, config } => {
            trace!("Using embedded context");
            // For embedded mode, send template requests to embedded kernel
            handle_template_embedded(command, handle, config, output_format).await
        }
        ExecutionContext::Connected { handle, address } => {
            trace!("Using connected context at address: {}", address);
            // For connected mode, send template requests to remote kernel
            handle_template_remote(command, handle, address, output_format).await
        }
    }
}

/// Handle template commands in embedded mode (kernel in same process)
async fn handle_template_embedded(
    command: TemplateCommands,
    mut handle: Box<llmspell_kernel::api::KernelHandle>,
    _config: Box<LLMSpellConfig>,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling template command in embedded mode");

    match command {
        TemplateCommands::List { category, format } => {
            info!("Listing templates via kernel message protocol");

            // Create template_request message for list command
            let request_content = json!({
                "command": "list",
                "category": category,
            });

            // Send request to kernel and wait for response
            let response = handle.send_template_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Template list error: {}", error));
            }

            // Extract templates from response
            let templates = response
                .get("templates")
                .and_then(|t| t.as_array())
                .ok_or_else(|| anyhow!("Invalid response format"))?;

            // Format output
            let fmt = format.unwrap_or(output_format);
            let formatter = OutputFormatter::new(fmt);

            match fmt {
                OutputFormat::Json => {
                    formatter.print_json(&json!({"templates": templates}))?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    if templates.is_empty() {
                        println!("No templates found");
                    } else {
                        println!("\nAvailable Templates:");
                        println!("{}", "=".repeat(80));
                        for template in templates {
                            if let Some(name) = template.get("name").and_then(|n| n.as_str()) {
                                if let Some(id) = template.get("id").and_then(|i| i.as_str()) {
                                    println!("\n  {} ({})", name, id);
                                }
                                if let Some(category) =
                                    template.get("category").and_then(|c| c.as_str())
                                {
                                    println!("  Category: {}", category);
                                }
                                if let Some(version) =
                                    template.get("version").and_then(|v| v.as_str())
                                {
                                    println!("  Version:  {}", version);
                                }
                                if let Some(description) =
                                    template.get("description").and_then(|d| d.as_str())
                                {
                                    println!("  Description: {}", description);
                                }
                                if let Some(tags) = template.get("tags").and_then(|t| t.as_array())
                                {
                                    let tag_strings: Vec<String> = tags
                                        .iter()
                                        .filter_map(|t| t.as_str().map(String::from))
                                        .collect();
                                    if !tag_strings.is_empty() {
                                        println!("  Tags: {}", tag_strings.join(", "));
                                    }
                                }
                            }
                        }
                        println!();
                    }
                }
            }
            Ok(())
        }

        TemplateCommands::Info { name, show_schema } => {
            info!("Getting info for template: {} via kernel", name);

            // Create template_request message for info command
            let request_content = json!({
                "command": "info",
                "name": name,
                "show_schema": show_schema,
            });

            // Send request to kernel and wait for response
            let response = handle.send_template_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Template info error: {}", error));
            }

            // Get template data from response
            let template = response
                .get("template")
                .ok_or_else(|| anyhow!("Invalid response format"))?;

            // Format output
            let formatter = OutputFormatter::new(output_format);

            match output_format {
                OutputFormat::Json => {
                    formatter.print_json(template)?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    if let Some(name) = template.get("name").and_then(|n| n.as_str()) {
                        if let Some(id) = template.get("id").and_then(|i| i.as_str()) {
                            println!("\nTemplate: {} ({})", name, id);
                        } else {
                            println!("\nTemplate: {}", name);
                        }
                    }
                    println!("{}", "=".repeat(80));

                    if let Some(category) = template.get("category").and_then(|c| c.as_str()) {
                        println!("Category:    {}", category);
                    }
                    if let Some(version) = template.get("version").and_then(|v| v.as_str()) {
                        println!("Version:     {}", version);
                    }
                    if let Some(author) = template.get("author").and_then(|a| a.as_str()) {
                        println!("Author:      {}", author);
                    }
                    if let Some(description) = template.get("description").and_then(|d| d.as_str())
                    {
                        println!("Description: {}", description);
                    }

                    if let Some(requires) = template.get("requires").and_then(|r| r.as_array()) {
                        if !requires.is_empty() {
                            println!("\nRequires:");
                            for req in requires {
                                if let Some(req_str) = req.as_str() {
                                    println!("  - {}", req_str);
                                }
                            }
                        }
                    }

                    if let Some(tags) = template.get("tags").and_then(|t| t.as_array()) {
                        let tag_strings: Vec<String> = tags
                            .iter()
                            .filter_map(|t| t.as_str().map(String::from))
                            .collect();
                        if !tag_strings.is_empty() {
                            println!("\nTags: {}", tag_strings.join(", "));
                        }
                    }

                    if show_schema {
                        if let Some(schema) = template.get("schema") {
                            println!("\nParameter Schema:");
                            println!("{}", "-".repeat(80));
                            formatter.print_json(schema)?;
                        }
                    }

                    println!();
                }
            }
            Ok(())
        }

        TemplateCommands::Exec {
            name,
            params,
            output,
        } => {
            info!("Executing template: {} via kernel", name);

            // Convert params Vec<(String, String)> to JSON object
            let mut params_obj = serde_json::Map::new();
            for (key, value) in params {
                // Try parsing as JSON first, fallback to string
                let json_value = serde_json::from_str(&value).unwrap_or_else(|_| json!(value));
                params_obj.insert(key, json_value);
            }

            // Create template_request message for exec command
            let request_content = json!({
                "command": "exec",
                "name": name,
                "params": params_obj,
            });

            // Send request to kernel and wait for response
            let start = std::time::Instant::now();
            let response = handle.send_template_request(request_content).await?;
            let duration = start.elapsed();

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Template execution error: {}", error));
            }

            // Display results
            println!(
                "\n✓ Template execution completed in {:.2}s",
                duration.as_secs_f64()
            );
            println!("{}", "=".repeat(80));

            // Show result
            if let Some(result) = response.get("result") {
                if let Some(result_type) = result.get("type").and_then(|t| t.as_str()) {
                    match result_type {
                        "text" => {
                            if let Some(text) = result.get("value").and_then(|v| v.as_str()) {
                                println!("\nResult:\n{}", text);
                            }
                        }
                        "structured" => {
                            if let Some(value) = result.get("value") {
                                println!("\nResult (JSON):");
                                println!("{}", serde_json::to_string_pretty(value)?);
                            }
                        }
                        "file" => {
                            if let Some(path) = result.get("path").and_then(|p| p.as_str()) {
                                println!("\nResult file: {}", path);
                            }
                        }
                        "multiple" => {
                            if let Some(results) = result.get("results").and_then(|r| r.as_array())
                            {
                                println!("\nMultiple results ({} items)", results.len());
                                for (i, res) in results.iter().enumerate() {
                                    println!("\n  Result {}:", i + 1);
                                    if let Some(res_type) = res.get("type").and_then(|t| t.as_str())
                                    {
                                        match res_type {
                                            "text" => {
                                                if let Some(text) =
                                                    res.get("value").and_then(|v| v.as_str())
                                                {
                                                    println!("    {}", text);
                                                }
                                            }
                                            "file" => {
                                                if let Some(path) =
                                                    res.get("path").and_then(|p| p.as_str())
                                                {
                                                    println!("    File: {}", path);
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Show artifacts
            if let Some(artifacts) = response.get("artifacts").and_then(|a| a.as_array()) {
                if !artifacts.is_empty() {
                    println!("\nArtifacts ({}):", artifacts.len());
                    for artifact in artifacts {
                        if let Some(filename) = artifact.get("filename").and_then(|f| f.as_str()) {
                            let mime_type = artifact
                                .get("mime_type")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown");
                            let size = artifact.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
                            println!("  - {} ({}, {} bytes)", filename, mime_type, size);

                            // Note: Actual artifact writing would need additional kernel support
                            if output.is_some() {
                                println!("    (Artifact writing to disk requires additional implementation)");
                            }
                        }
                    }
                }
            }

            // Show metrics
            if let Some(metrics) = response.get("metrics") {
                println!("\nMetrics:");
                if let Some(duration_ms) = metrics.get("duration_ms").and_then(|d| d.as_f64()) {
                    println!("  Duration:      {:.2}s", duration_ms / 1000.0);
                }
                if let Some(tokens) = metrics.get("tokens_used").and_then(|t| t.as_u64()) {
                    println!("  Tokens:        {}", tokens);
                }
                if let Some(cost) = metrics.get("cost_usd").and_then(|c| c.as_f64()) {
                    println!("  Cost:          ${:.4}", cost);
                }
                if let Some(agents) = metrics.get("agents_invoked").and_then(|a| a.as_u64()) {
                    if agents > 0 {
                        println!("  Agents:        {}", agents);
                    }
                }
                if let Some(tools) = metrics.get("tools_invoked").and_then(|t| t.as_u64()) {
                    if tools > 0 {
                        println!("  Tools:         {}", tools);
                    }
                }
                if let Some(rag_queries) = metrics.get("rag_queries").and_then(|r| r.as_u64()) {
                    if rag_queries > 0 {
                        println!("  RAG queries:   {}", rag_queries);
                    }
                }
            }

            println!();
            Ok(())
        }

        TemplateCommands::Search { query, category } => {
            info!("Searching templates with query: {:?} via kernel", query);

            // Join query words
            let query_str = query.join(" ");

            // Create template_request message for search command
            let request_content = json!({
                "command": "search",
                "query": query_str,
                "category": category,
            });

            // Send request to kernel and wait for response
            let response = handle.send_template_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Template search error: {}", error));
            }

            // Extract results from response
            let results = response
                .get("results")
                .and_then(|r| r.as_array())
                .ok_or_else(|| anyhow!("Invalid response format"))?;

            // Format output
            let formatter = OutputFormatter::new(output_format);

            match output_format {
                OutputFormat::Json => {
                    formatter.print_json(&json!({
                        "query": query_str,
                        "results": results
                    }))?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    if results.is_empty() {
                        println!("No templates found matching '{}'", query_str);
                    } else {
                        println!("\nSearch Results ({} templates):", results.len());
                        println!("{}", "=".repeat(80));
                        for template in results {
                            if let Some(name) = template.get("name").and_then(|n| n.as_str()) {
                                if let Some(id) = template.get("id").and_then(|i| i.as_str()) {
                                    println!("\n  {} ({})", name, id);
                                }
                                if let Some(category) =
                                    template.get("category").and_then(|c| c.as_str())
                                {
                                    println!("  Category: {}", category);
                                }
                                if let Some(description) =
                                    template.get("description").and_then(|d| d.as_str())
                                {
                                    println!("  Description: {}", description);
                                }
                            }
                        }
                        println!();
                    }
                }
            }
            Ok(())
        }

        TemplateCommands::Schema { name } => {
            info!("Getting schema for template: {} via kernel", name);

            // Create template_request message for schema command
            let request_content = json!({
                "command": "schema",
                "name": name,
            });

            // Send request to kernel and wait for response
            let response = handle.send_template_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Template schema error: {}", error));
            }

            // Always output as JSON for schema
            let formatter = OutputFormatter::new(OutputFormat::Json);
            if let Some(schema) = response.get("schema") {
                formatter.print_json(schema)?;
            } else {
                formatter.print_json(&response)?;
            }
            Ok(())
        }
    }
}

/// Handle template commands in connected mode (remote kernel)
async fn handle_template_remote(
    command: TemplateCommands,
    mut handle: llmspell_kernel::api::ClientHandle,
    address: String,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling template command in connected mode to {}", address);

    // For now, replicate the same logic as embedded mode
    // The only difference is we're using ClientHandle instead of KernelHandle
    // The message protocol is identical

    match command {
        TemplateCommands::List { category, format } => {
            info!("Listing templates via remote kernel");

            let request_content = json!({
                "command": "list",
                "category": category,
            });

            let response = handle.send_template_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Template list error: {}", error));
            }

            let templates = response
                .get("templates")
                .and_then(|t| t.as_array())
                .ok_or_else(|| anyhow!("Invalid response format"))?;

            let fmt = format.unwrap_or(output_format);
            let formatter = OutputFormatter::new(fmt);

            match fmt {
                OutputFormat::Json => {
                    formatter.print_json(&json!({"templates": templates}))?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    if templates.is_empty() {
                        println!("No templates found");
                    } else {
                        println!("\nAvailable Templates:");
                        println!("{}", "=".repeat(80));
                        for template in templates {
                            if let Some(name) = template.get("name").and_then(|n| n.as_str()) {
                                if let Some(id) = template.get("id").and_then(|i| i.as_str()) {
                                    println!("\n  {} ({})", name, id);
                                }
                                if let Some(category) =
                                    template.get("category").and_then(|c| c.as_str())
                                {
                                    println!("  Category: {}", category);
                                }
                                if let Some(description) =
                                    template.get("description").and_then(|d| d.as_str())
                                {
                                    println!("  Description: {}", description);
                                }
                            }
                        }
                        println!();
                    }
                }
            }
            Ok(())
        }

        // For brevity, the other commands follow the same pattern
        // In a real implementation, we'd duplicate all command handling for connected mode
        _ => {
            // For now, show a helpful message
            Err(anyhow!("Template command not yet fully implemented for connected mode. Please use embedded mode for now."))
        }
    }
}
