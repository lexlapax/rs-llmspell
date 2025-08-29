//! ABOUTME: Apps command implementation for running embedded example applications
//! ABOUTME: Provides single-binary distribution of example apps

use crate::cli::{AppsSubcommand, OutputFormat, ScriptEngine};
use crate::embedded_resources::{cleanup_temp_dir, extract_app, list_apps};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use serde_json::json;

/// Execute an embedded application
pub async fn execute_apps_command(
    app: Option<AppsSubcommand>,
    engine: ScriptEngine,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    match app {
        None | Some(AppsSubcommand::List) => {
            // List all available applications
            list_available_apps(output_format)
        }
        Some(AppsSubcommand::FileOrganizer { args }) => {
            run_embedded_app(
                "file-organizer",
                engine,
                runtime_config,
                args,
                output_format,
            )
            .await
        }
        Some(AppsSubcommand::ResearchCollector { args }) => {
            run_embedded_app(
                "research-collector",
                engine,
                runtime_config,
                args,
                output_format,
            )
            .await
        }
        Some(AppsSubcommand::ContentCreator { args }) => {
            run_embedded_app(
                "content-creator",
                engine,
                runtime_config,
                args,
                output_format,
            )
            .await
        }
        Some(AppsSubcommand::CommunicationManager { args }) => {
            run_embedded_app(
                "communication-manager",
                engine,
                runtime_config,
                args,
                output_format,
            )
            .await
        }
        Some(AppsSubcommand::ProcessOrchestrator { args }) => {
            run_embedded_app(
                "process-orchestrator",
                engine,
                runtime_config,
                args,
                output_format,
            )
            .await
        }
        Some(AppsSubcommand::CodeReviewAssistant { args }) => {
            run_embedded_app(
                "code-review-assistant",
                engine,
                runtime_config,
                args,
                output_format,
            )
            .await
        }
        Some(AppsSubcommand::WebappCreator { args }) => {
            run_embedded_app(
                "webapp-creator",
                engine,
                runtime_config,
                args,
                output_format,
            )
            .await
        }
        Some(AppsSubcommand::KnowledgeBase { args }) => {
            run_embedded_app(
                "knowledge-base",
                engine,
                runtime_config,
                args,
                output_format,
            )
            .await
        }
        Some(AppsSubcommand::PersonalAssistant { args }) => {
            run_embedded_app(
                "personal-assistant",
                engine,
                runtime_config,
                args,
                output_format,
            )
            .await
        }
    }
}

/// List all available embedded applications
fn list_available_apps(output_format: OutputFormat) -> Result<()> {
    let apps = list_apps();

    match output_format {
        OutputFormat::Json => {
            let json_apps: Vec<_> = apps
                .iter()
                .map(|app| {
                    json!({
                        "name": app.name,
                        "description": app.description,
                        "complexity": app.complexity,
                        "agents": app.agents,
                    })
                })
                .collect();

            let output = json!({
                "applications": json_apps,
                "total": apps.len(),
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Pretty | OutputFormat::Text => {
            println!("🚀 Available LLMSpell Applications\n");
            println!(
                "{:<25} {:<15} {:<10} Description",
                "Application", "Complexity", "Agents"
            );
            println!("{}", "-".repeat(90));

            for app in apps {
                println!(
                    "{:<25} {:<15} {:<10} {}",
                    app.name, app.complexity, app.agents, app.description
                );
            }

            println!("\n✨ Run an application with: llmspell apps <app-name>");
            println!("📚 Example: llmspell apps file-organizer");
        }
    }

    Ok(())
}

/// Run an embedded application
async fn run_embedded_app(
    app_name: &str,
    engine: ScriptEngine,
    mut runtime_config: LLMSpellConfig,
    args: Vec<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Extract the application to a temporary directory
    let (lua_path, config_path) = extract_app(app_name)?;

    // Load the embedded config and merge with runtime config
    let app_config = LLMSpellConfig::load_with_discovery(Some(&config_path)).await?;

    // Merge configs (runtime config takes precedence for API keys)
    if runtime_config.providers.providers.is_empty() {
        runtime_config.providers = app_config.providers;
    }

    // Use the app's tools configuration
    runtime_config.tools = app_config.tools;

    // Notify user
    match output_format {
        OutputFormat::Json => {
            let output = json!({
                "status": "starting",
                "application": app_name,
                "script": lua_path.to_string_lossy(),
                "config": config_path.to_string_lossy(),
            });
            println!("{}", serde_json::to_string(&output)?);
        }
        _ => {
            println!("🚀 Starting {} application...", app_name);
            println!("📄 Script: {}", lua_path.display());
            println!("⚙️  Config: {}", config_path.display());
        }
    }

    // Run the script
    let result = crate::commands::run::execute_script_file(
        lua_path.clone(),
        engine,
        runtime_config,
        false, // No streaming for embedded apps
        args,
        output_format,
    )
    .await;

    // Clean up temp directory
    if let Some(parent) = lua_path.parent() {
        let _ = cleanup_temp_dir(parent);
    }

    result
}
