//! RAG system commands
//!
//! Provides CLI commands for managing the RAG (Retrieval-Augmented Generation) system
//! through the kernel using custom protocol messages (RagRequest/RagReply).

use crate::cli::{OutputFormat, RagCommands};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::jupyter::protocol::RagOperation;
use std::fs;
use std::path::PathBuf;

/// Handle RAG system commands
pub async fn handle_rag_command(
    command: RagCommands,
    config: LLMSpellConfig,
    output_format: OutputFormat,
    connect: Option<String>, // Connection string for external kernel
) -> Result<()> {
    // Connect to kernel to access RAG pipeline
    let kernel = super::create_kernel_connection(config, connect).await?;

    match command {
        RagCommands::Ingest {
            id,
            content,
            metadata,
            scope,
        } => ingest_document(kernel, id, content, metadata, scope, output_format).await,
        RagCommands::Search {
            query,
            limit,
            threshold,
            scope,
        } => search_documents(kernel, query, limit, threshold, scope, output_format).await,
        RagCommands::Stats { scope } => show_stats(kernel, scope, output_format).await,
        RagCommands::Clear { scope, confirm } => clear_data(kernel, scope, confirm).await,
        RagCommands::Index {
            path,
            recursive,
            pattern,
            scope,
        } => index_directory(kernel, path, recursive, pattern, scope, output_format).await,
    }
}

/// Ingest a document into the RAG system
async fn ingest_document(
    mut kernel: Box<dyn crate::kernel_client::KernelConnectionTrait>,
    id: String,
    content: String,
    metadata: Option<String>,
    scope: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Read content from file if prefixed with @
    let doc_content = if let Some(file_path) = content.strip_prefix('@') {
        fs::read_to_string(file_path)?
    } else {
        content
    };

    // Parse metadata if provided
    let metadata_value = if let Some(meta_str) = metadata {
        Some(serde_json::from_str(&meta_str)?)
    } else {
        None
    };

    // Create the RAG operation
    let operation = RagOperation::Ingest {
        path: id.clone(),
        content: Some(doc_content),
        metadata: metadata_value,
        chunk_size: 512, // Default chunk size
        recursive: false,
    };

    // Send RAG request to kernel
    let reply = kernel
        .rag_request(serde_json::to_value(operation)?, scope)
        .await?;

    // Process the reply
    if let Some(data) = reply.get("data") {
        match output_format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
            OutputFormat::Text | OutputFormat::Pretty => {
                if let Some(chunks) = data.get("chunks_stored") {
                    println!("✓ Document '{}' ingested successfully", id);
                    println!("  Chunks stored: {}", chunks);
                    if let Some(time) = data.get("storage_time_ms") {
                        println!("  Storage time: {}ms", time);
                    }
                } else {
                    println!("Document ingested: {}", id);
                }
            }
        }
    } else if let Some(error) = reply.get("error") {
        anyhow::bail!("Ingestion failed: {}", error);
    }

    Ok(())
}

/// Search for relevant documents
async fn search_documents(
    mut kernel: Box<dyn crate::kernel_client::KernelConnectionTrait>,
    query: String,
    limit: usize,
    threshold: f32,
    scope: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Create the RAG operation
    let operation = RagOperation::Search {
        query: query.clone(),
        limit,
        threshold: Some(threshold),
        metadata_filter: None,
    };

    // Send RAG request to kernel
    let reply = kernel
        .rag_request(serde_json::to_value(operation)?, scope)
        .await?;

    // Process the reply
    if let Some(data) = reply.get("data") {
        match output_format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
            OutputFormat::Text | OutputFormat::Pretty => {
                if let Some(results) = data.get("results").and_then(|r| r.as_array()) {
                    println!("Search results for: \"{}\"", query);
                    println!("Found {} results", results.len());
                    println!();

                    for (i, result) in results.iter().enumerate() {
                        if let (Some(content), Some(score)) = (
                            result.get("content").and_then(|c| c.as_str()),
                            result.get("score").and_then(|s| s.as_f64()),
                        ) {
                            println!("{}. [Score: {:.3}]", i + 1, score);
                            // Truncate content for display
                            let display_content = if content.len() > 200 {
                                format!("{}...", &content[..200])
                            } else {
                                content.to_string()
                            };
                            println!("   {}", display_content);

                            // Show metadata if present
                            if let Some(metadata) = result.get("metadata") {
                                if !metadata.is_null() {
                                    println!("   Metadata: {}", metadata);
                                }
                            }
                            println!();
                        }
                    }

                    if let Some(time) = data.get("search_time_ms") {
                        println!("Search completed in {}ms", time);
                    }
                } else {
                    println!("No results found for query: \"{}\"", query);
                }
            }
        }
    } else if let Some(error) = reply.get("error") {
        anyhow::bail!("Search failed: {}", error);
    }

    Ok(())
}

/// Show RAG system statistics
async fn show_stats(
    mut kernel: Box<dyn crate::kernel_client::KernelConnectionTrait>,
    scope: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Create the RAG operation
    let operation = RagOperation::Stats { detailed: false };

    // Send RAG request to kernel
    tracing::debug!("Sending stats request to kernel with scope: {:?}", scope);
    let reply = kernel
        .rag_request(serde_json::to_value(operation)?, scope.clone())
        .await?;

    tracing::debug!("Received stats reply: {:?}", reply);

    // Process the reply
    if let Some(data) = reply.get("data") {
        match output_format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
            OutputFormat::Text | OutputFormat::Pretty => {
                if let Some(stats) = data.get("stats") {
                    println!("RAG System Statistics");
                    println!("====================");

                    if let Some(scope_str) = &scope {
                        println!("Scope: {}", scope_str);
                    } else {
                        println!("Scope: Global");
                    }

                    if let Some(vectors) = stats.get("vectors_stored") {
                        println!("Vectors stored: {}", vectors);
                    }
                    if let Some(memory) = stats.get("memory_usage_bytes") {
                        let mb = memory.as_u64().unwrap_or(0) as f64 / 1_048_576.0;
                        println!("Memory usage: {:.2} MB", mb);
                    }
                    if let Some(cache_hits) = stats.get("cache_hits") {
                        println!("Cache hits: {}", cache_hits);
                    }
                    if let Some(cache_misses) = stats.get("cache_misses") {
                        println!("Cache misses: {}", cache_misses);
                    }
                    if let Some(hit_rate) = stats.get("cache_hit_rate") {
                        println!(
                            "Cache hit rate: {:.1}%",
                            hit_rate.as_f64().unwrap_or(0.0) * 100.0
                        );
                    }
                    if let Some(cost) = stats.get("estimated_cost_usd") {
                        println!("Estimated cost: ${:.4}", cost.as_f64().unwrap_or(0.0));
                    }
                } else {
                    println!("No statistics available");
                }
            }
        }
    } else if let Some(error) = reply.get("error") {
        anyhow::bail!("Failed to get stats: {}", error);
    }

    Ok(())
}

/// Clear RAG data
async fn clear_data(
    mut kernel: Box<dyn crate::kernel_client::KernelConnectionTrait>,
    scope: Option<String>,
    confirm: bool,
) -> Result<()> {
    // Require confirmation for destructive operation
    if !confirm {
        println!("Warning: This will permanently delete RAG data.");
        if scope.is_some() {
            println!("Scope: {}", scope.as_ref().unwrap());
        } else {
            println!("Scope: Global (ALL DATA)");
        }
        println!("\nTo confirm, run with --confirm flag");
        return Ok(());
    }

    // Create the RAG operation
    let operation = RagOperation::Clear {
        scope: scope.clone(),
        confirm,
    };

    // Send RAG request to kernel
    let reply = kernel
        .rag_request(serde_json::to_value(operation)?, scope.clone())
        .await?;

    // Process the reply
    if let Some(data) = reply.get("data") {
        if let Some(deleted) = data.get("vectors_deleted") {
            println!("✓ Cleared {} vectors", deleted);
            if let Some(scope_str) = scope {
                println!("  Scope: {}", scope_str);
            } else {
                println!("  Scope: Global");
            }
        } else {
            println!("✓ RAG data cleared");
        }
    } else if let Some(error) = reply.get("error") {
        anyhow::bail!("Clear operation failed: {}", error);
    }

    Ok(())
}

/// Index files or directories
async fn index_directory(
    mut kernel: Box<dyn crate::kernel_client::KernelConnectionTrait>,
    path: PathBuf,
    _recursive: bool,
    pattern: Option<String>,
    scope: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Create the RAG operation
    let operation = RagOperation::Index {
        action: llmspell_kernel::jupyter::protocol::IndexAction::List,
    };

    // Send RAG request to kernel
    let reply = kernel
        .rag_request(serde_json::to_value(operation)?, scope)
        .await?;

    // Process the reply
    match output_format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&reply)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            if let Some(error) = reply.get("error") {
                if error.as_str() == Some("Index operation not yet implemented via kernel") {
                    println!("Note: Directory indexing is not yet implemented.");
                    println!("\nTo index files, use the ingest command with individual files:");
                    println!("  llmspell rag ingest <id> @<file_path>");

                    if let Some(pattern_str) = pattern {
                        println!(
                            "\nTo find files matching pattern '{}', you can use:",
                            pattern_str
                        );
                        println!(
                            "  find {} -name \"{}\" | while read f; do",
                            path.display(),
                            pattern_str
                        );
                        println!("    llmspell rag ingest \"$f\" \"@$f\"");
                        println!("  done");
                    }
                } else {
                    anyhow::bail!("Index operation failed: {}", error);
                }
            } else {
                println!("Index operation completed");
            }
        }
    }

    Ok(())
}
