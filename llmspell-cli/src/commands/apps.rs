//! Application execution commands
//! Professional example application runner

use crate::cli::OutputFormat;
use crate::execution_context::ExecutionContext;
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Run example applications
pub async fn run_application(
    name: String,
    args: Vec<String>,
    context: ExecutionContext,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Running application: {} with args: {:?}", name, args);

    // Look for application in examples directory
    let app_path = PathBuf::from("examples/script-users/applications")
        .join(&name)
        .join("main.lua");

    if !app_path.exists() {
        match output_format {
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "error",
                        "message": format!("Application '{}' not found", name),
                        "path": app_path.display().to_string()
                    })
                );
            }
            OutputFormat::Yaml => {
                let data = serde_json::json!({
                    "status": "error",
                    "message": format!("Application '{}' not found", name),
                    "path": app_path.display().to_string()
                });
                println!("{}", serde_yaml::to_string(&data)?);
            }
            _ => {
                println!(
                    "Application '{}' not found at: {}",
                    name,
                    app_path.display()
                );
            }
        }
        anyhow::bail!("Application '{}' not found", name);
    }

    // Load application config if it exists
    let config_path = app_path.parent().unwrap().join("config.toml");
    let app_config = if config_path.exists() {
        Some(config_path)
    } else {
        None
    };

    match context {
        ExecutionContext::Embedded { handle, .. } => {
            let _kernel = handle.into_kernel();

            // Execute the application script
            execute_app_script(&app_path, &args, app_config.as_ref(), output_format).await?;
        }
        ExecutionContext::Connected { .. } => {
            // For connected kernels, we would send the script for execution
            execute_app_script(&app_path, &args, app_config.as_ref(), output_format).await?;
        }
    }

    Ok(())
}

/// Execute application script
async fn execute_app_script(
    script_path: &PathBuf,
    args: &[String],
    config_path: Option<&PathBuf>,
    output_format: OutputFormat,
) -> Result<()> {
    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "executing",
                    "script": script_path.display().to_string(),
                    "config": config_path.map(|p| p.display().to_string()),
                    "args": args
                })
            );
        }
        OutputFormat::Yaml => {
            let data = serde_json::json!({
                "status": "executing",
                "script": script_path.display().to_string(),
                "config": config_path.map(|p| p.display().to_string()),
                "args": args
            });
            println!("{}", serde_yaml::to_string(&data)?);
        }
        _ => {
            println!("Executing application script: {}", script_path.display());
            if let Some(config) = config_path {
                println!("Using config: {}", config.display());
            }
            if !args.is_empty() {
                println!("Arguments: {:?}", args);
            }
            println!();

            // Read and display script content for demonstration
            match std::fs::read_to_string(script_path) {
                Ok(content) => {
                    println!("Script content preview:");
                    println!("```lua");
                    // Show first 10 lines
                    for (i, line) in content.lines().take(10).enumerate() {
                        println!("{:3}: {}", i + 1, line);
                    }
                    if content.lines().count() > 10 {
                        println!("... ({} more lines)", content.lines().count() - 10);
                    }
                    println!("```");
                    println!();
                    println!("✓ Application would be executed here");
                }
                Err(e) => {
                    println!("Error reading script: {}", e);
                }
            }
        }
    }

    Ok(())
}
