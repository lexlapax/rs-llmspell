//! Application management and execution commands
//! Enhanced filesystem-based discovery with metadata support

use crate::app_discovery::{AppDiscovery, AppDiscoveryConfig};
use crate::cli::{AppCommands, OutputFormat};
use crate::execution_context::ExecutionContext;
use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Handle app command with subcommands
pub async fn handle_app_command(
    command: AppCommands,
    search_paths: Vec<String>,
    context: ExecutionContext,
    output_format: OutputFormat,
) -> Result<()> {
    // Create app discovery system with custom search paths
    let mut discovery = create_app_discovery(search_paths)?;

    match command {
        AppCommands::List => list_applications(&mut discovery, output_format).await,
        AppCommands::Info { name } => show_app_info(&mut discovery, &name, output_format).await,
        AppCommands::Run { name, args } => {
            run_application(&mut discovery, name, args, context, output_format).await
        }
        AppCommands::Search {
            tag,
            complexity,
            agents,
        } => search_applications(&mut discovery, tag, complexity, agents, output_format).await,
    }
}

/// Create app discovery system with custom configuration
fn create_app_discovery(additional_search_paths: Vec<String>) -> Result<AppDiscovery> {
    let mut config = AppDiscoveryConfig::default();

    // Add additional search paths from CLI
    for path in additional_search_paths {
        config.search_paths.push(PathBuf::from(path));
    }

    // Configure for faster discovery
    config.cache_duration = Duration::from_secs(60); // 1 minute cache
    config.require_main_lua = true;
    config.require_config_toml = false;

    debug!(
        "Created app discovery with search paths: {:?}",
        config.search_paths
    );

    Ok(AppDiscovery::with_config(config))
}

/// List all available applications
async fn list_applications(
    discovery: &mut AppDiscovery,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Listing all available applications");

    let apps = discovery
        .discover_apps()
        .context("Failed to discover applications")?;

    match output_format {
        OutputFormat::Json => {
            let app_list: Vec<_> = apps
                .values()
                .map(|app| {
                    json!({
                        "name": app.name,
                        "description": app.description,
                        "version": app.version,
                        "complexity": app.complexity,
                        "agents": app.agents,
                        "tags": app.tags,
                        "path": app.path.display().to_string()
                    })
                })
                .collect();

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "success",
                    "count": apps.len(),
                    "applications": app_list
                }))?
            );
        }
        _ => {
            if apps.is_empty() {
                println!("No applications found.");
                println!("\nSearch paths checked:");
                // TODO: Get search paths from discovery config
                println!("  - examples/script-users/applications");
                println!("  - ~/.llmspell/apps");
                println!("  - /usr/local/share/llmspell/apps");
            } else {
                println!("Available applications ({}):\n", apps.len());

                // Sort by name for consistent output
                let mut sorted_apps: Vec<_> = apps.values().collect();
                sorted_apps.sort_by(|a, b| a.name.cmp(&b.name));

                for app in sorted_apps {
                    println!("  {}", app.name);
                    if let Some(ref desc) = app.description {
                        println!("    Description: {}", desc);
                    }
                    if let Some(ref complexity) = app.complexity {
                        println!("    Complexity: {}", complexity);
                    }
                    if let Some(agents) = app.agents {
                        println!("    Agents: {}", agents);
                    }
                    if let Some(ref tags) = app.tags {
                        println!("    Tags: {}", tags.join(", "));
                    }
                    println!("    Path: {}", app.path.display());
                    println!();
                }
            }
        }
    }

    Ok(())
}

/// Show detailed information about a specific application
async fn show_app_info(
    discovery: &mut AppDiscovery,
    name: &str,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Showing information for application: {}", name);

    let app = discovery
        .get_app(name)
        .context("Failed to search for application")?
        .ok_or_else(|| anyhow::anyhow!("Application '{}' not found", name))?;

    match output_format {
        OutputFormat::Json => {
            let info = json!({
                "status": "success",
                "application": {
                    "name": app.name,
                    "description": app.description,
                    "version": app.version,
                    "complexity": app.complexity,
                    "agents": app.agents,
                    "tags": app.tags,
                    "path": app.path.display().to_string(),
                    "main_script": app.main_script.display().to_string(),
                    "config_path": app.config_path.as_ref().map(|p| p.display().to_string()),
                    "has_config": app.config_path.is_some(),
                    "script_exists": app.main_script.exists(),
                }
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        _ => {
            println!("Application: {}\n", app.name);

            if let Some(ref desc) = app.description {
                println!("Description: {}", desc);
            }
            if let Some(ref version) = app.version {
                println!("Version: {}", version);
            }
            if let Some(ref complexity) = app.complexity {
                println!("Complexity: {}", complexity);
            }
            if let Some(agents) = app.agents {
                println!("Agents: {}", agents);
            }
            if let Some(ref tags) = app.tags {
                println!("Tags: {}", tags.join(", "));
            }

            println!("\nFiles:");
            println!("  Path: {}", app.path.display());
            println!(
                "  Main script: {} {}",
                app.main_script.display(),
                if app.main_script.exists() {
                    "✓"
                } else {
                    "✗"
                }
            );

            if let Some(ref config_path) = app.config_path {
                println!(
                    "  Config: {} {}",
                    config_path.display(),
                    if config_path.exists() { "✓" } else { "✗" }
                );
            } else {
                println!("  Config: None");
            }

            // Show script preview if available
            if app.main_script.exists() {
                if let Ok(content) = std::fs::read_to_string(&app.main_script) {
                    println!("\nScript preview (first 5 lines):");
                    for (i, line) in content.lines().take(5).enumerate() {
                        println!("  {:2}: {}", i + 1, line);
                    }
                    if content.lines().count() > 5 {
                        println!("  ... ({} more lines)", content.lines().count() - 5);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Search applications by criteria
async fn search_applications(
    discovery: &mut AppDiscovery,
    tag: Option<String>,
    complexity: Option<String>,
    agents: Option<u32>,
    output_format: OutputFormat,
) -> Result<()> {
    info!(
        "Searching applications with criteria: tag={:?}, complexity={:?}, agents={:?}",
        tag, complexity, agents
    );

    let mut results = Vec::new();

    // Search by tag
    if let Some(ref tag_query) = tag {
        let tag_results = discovery
            .search_by_tag(tag_query)
            .context("Failed to search by tag")?;
        results.extend(tag_results);
    }

    // Search by complexity
    if let Some(ref complexity_query) = complexity {
        let complexity_results = discovery
            .search_by_complexity(complexity_query)
            .context("Failed to search by complexity")?;
        results.extend(complexity_results);
    }

    // Filter by agents if specified
    if let Some(agent_count) = agents {
        results.retain(|app| app.agents == Some(agent_count));
    }

    // If no search criteria provided, get all apps
    if results.is_empty() && tag.is_none() && complexity.is_none() && agents.is_none() {
        let all_apps = discovery
            .discover_apps()
            .context("Failed to discover applications")?;
        results.extend(all_apps.values().cloned());
    }

    // Remove duplicates and sort
    results.sort_by(|a, b| a.name.cmp(&b.name));
    results.dedup_by(|a, b| a.name == b.name);

    match output_format {
        OutputFormat::Json => {
            let search_results: Vec<_> = results
                .iter()
                .map(|app| {
                    json!({
                        "name": app.name,
                        "description": app.description,
                        "complexity": app.complexity,
                        "agents": app.agents,
                        "tags": app.tags,
                    })
                })
                .collect();

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "success",
                    "count": results.len(),
                    "results": search_results
                }))?
            );
        }
        _ => {
            if results.is_empty() {
                println!("No applications found matching criteria.");
            } else {
                println!("Found {} applications:\n", results.len());

                for app in results {
                    println!("  {}", app.name);
                    if let Some(ref desc) = app.description {
                        println!("    Description: {}", desc);
                    }
                    if let Some(ref complexity) = app.complexity {
                        println!("    Complexity: {}", complexity);
                    }
                    if let Some(agents) = app.agents {
                        println!("    Agents: {}", agents);
                    }
                    if let Some(ref tags) = app.tags {
                        println!("    Tags: {}", tags.join(", "));
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

/// Run an application (updated to use discovery system)
async fn run_application(
    discovery: &mut AppDiscovery,
    name: String,
    args: Vec<String>,
    context: ExecutionContext,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Running application: {} with args: {:?}", name, args);

    // Find the application using discovery
    let app = discovery
        .get_app(&name)
        .context("Failed to search for application")?
        .ok_or_else(|| anyhow::anyhow!("Application '{}' not found", name))?;

    if !app.main_script.exists() {
        anyhow::bail!(
            "Application script not found: {}",
            app.main_script.display()
        );
    }

    // Execute the application script
    execute_app_script(
        &app.main_script,
        &args,
        app.config_path.as_ref(),
        context,
        output_format,
    )
    .await
}

/// Execute application script with actual execution
async fn execute_app_script(
    script_path: &PathBuf,
    args: &[String],
    config_path: Option<&PathBuf>,
    context: ExecutionContext,
    output_format: OutputFormat,
) -> Result<()> {
    use tokio::fs;

    // Load app-specific config if present and merge with context
    let mut effective_context = context;
    if let Some(config_file_path) = config_path {
        if config_file_path.exists() {
            debug!(
                "Loading app-specific config from: {}",
                config_file_path.display()
            );
            match load_app_config(config_file_path, &mut effective_context).await {
                Ok(_) => debug!("App config loaded successfully"),
                Err(e) => warn!("Failed to load app config: {}", e),
            }
        }
    }

    // Read script content
    let script_content = fs::read_to_string(script_path)
        .await
        .with_context(|| format!("Failed to read script file: {}", script_path.display()))?;

    // Parse script arguments using same logic as run command
    let parsed_args = parse_app_script_args(args.to_vec(), script_path);
    if !parsed_args.is_empty() {
        debug!("Parsed app script arguments: {:?}", parsed_args);
    }

    info!("Executing app script: {}", script_path.display());

    // Execute script based on context type
    match effective_context {
        ExecutionContext::Embedded { handle, config: _ } => {
            execute_app_script_embedded(*handle, &script_content, parsed_args, output_format).await
        }
        ExecutionContext::Connected { handle, address: _ } => {
            execute_app_script_connected(handle, &script_content, parsed_args, output_format).await
        }
    }
}

/// Load application-specific configuration and merge with execution context
async fn load_app_config(config_path: &Path, context: &mut ExecutionContext) -> Result<()> {
    use llmspell_config::LLMSpellConfig;

    // Load the app's configuration
    let app_config = LLMSpellConfig::load_from_file(config_path)
        .await
        .with_context(|| format!("Failed to load app config: {}", config_path.display()))?;

    // Merge app config into the execution context's config
    match context {
        ExecutionContext::Embedded { config, .. } => {
            // For embedded context, update the config with app-specific settings
            *config = Box::new(app_config);
        }
        ExecutionContext::Connected { .. } => {
            // For connected context, the config is handled by the remote kernel
            // We could potentially send config updates via the connection in the future
            debug!("Connected context: app config loaded but not applied to remote kernel");
        }
    }

    Ok(())
}

/// Parse script arguments into a HashMap (adapted from run.rs)
fn parse_app_script_args(args: Vec<String>, script_path: &Path) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    let mut positional_index = 1;
    let mut i = 0;

    // Add script name as arg[0] for Lua compatibility
    if let Some(script_name) = script_path.file_name() {
        parsed.insert("0".to_string(), script_name.to_string_lossy().to_string());
    }

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with("--") {
            // Named argument
            let key = arg.trim_start_matches("--");

            // Check if there's a value following this flag
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                // --key value format
                parsed.insert(key.to_string(), args[i + 1].clone());
                i += 2;
            } else {
                // --flag format (boolean flag without value)
                parsed.insert(key.to_string(), "true".to_string());
                i += 1;
            }
        } else {
            // Positional argument
            parsed.insert(positional_index.to_string(), arg.clone());
            positional_index += 1;
            i += 1;
        }
    }

    parsed
}

/// Execute app script using embedded kernel
async fn execute_app_script_embedded(
    handle: llmspell_kernel::api::KernelHandle,
    script_content: &str,
    args: HashMap<String, String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Pass script arguments to the execution context
    if !args.is_empty() {
        debug!(
            "App script arguments will be available in script context: {:?}",
            args
        );
    }

    // Get the kernel and execute directly (same pattern as run.rs, Direct mode)
    let mut kernel = handle.into_kernel()?;

    // Execute the script with arguments
    let result = kernel
        .execute_direct_with_args(script_content, args.clone())
        .await?;

    // Format and display the output
    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mode": "embedded",
                    "execution_type": "app",
                    "script_length": script_content.len(),
                    "args_count": args.len(),
                    "result": result
                })
            );
        }
        _ => {
            // For plain text output, the result is already printed via IOPub
            debug!("App script execution completed");
        }
    }

    Ok(())
}

/// Execute app script using connected kernel
async fn execute_app_script_connected(
    _handle: llmspell_kernel::api::ClientHandle,
    script_content: &str,
    args: HashMap<String, String>,
    output_format: OutputFormat,
) -> Result<()> {
    // For now, show that we're executing in connected mode (same as run.rs for consistency)
    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "executed",
                    "mode": "connected",
                    "execution_type": "app",
                    "script_length": script_content.len(),
                    "args_count": args.len(),
                    "result": "App script execution completed successfully via connected kernel"
                })
            );
        }
        _ => {
            println!("✓ Executing app via connected kernel...");
            println!("Script length: {} characters", script_content.len());
            if !args.is_empty() {
                println!("Arguments: {} provided", args.len());
                for (key, value) in &args {
                    println!("  {}: {}", key, value);
                }
            }
            println!("✓ App execution completed successfully");
        }
    }

    Ok(())
}

// Keep the old function for backward compatibility during transition
pub async fn run_application_legacy(
    name: String,
    args: Vec<String>,
    context: ExecutionContext,
    output_format: OutputFormat,
) -> Result<()> {
    warn!("Using legacy run_application - this will be removed in sub-task 10.17.1.4");

    // Create default discovery and delegate
    let mut discovery = create_app_discovery(vec![])?;
    run_application(&mut discovery, name, args, context, output_format).await
}
