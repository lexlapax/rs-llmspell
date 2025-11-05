//! ABOUTME: Storage migration CLI commands
//! ABOUTME: Provides plan-based migration workflow for storage backends

use crate::cli::OutputFormat;
use anyhow::{anyhow, Context, Result};
use llmspell_config::LLMSpellConfig;
use llmspell_storage::backends::SledBackend;
use llmspell_storage::migration::{MigrationEngine, MigrationPlan, MigrationSource, MigrationTarget};
use llmspell_storage::traits::StorageBackend;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "postgres")]
use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};

/// Storage subcommands (imported from cli.rs)
pub use crate::cli::{StorageCommands, MigrateAction};

/// Execute storage command
pub async fn handle_storage_command(
    command: StorageCommands,
    _config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        StorageCommands::Migrate { action } => handle_migrate(action, output_format).await,
        StorageCommands::Info { backend } => handle_info(backend, output_format).await,
        StorageCommands::Validate {
            backend,
            components,
        } => handle_validate(backend, components, output_format).await,
    }
}

/// Handle migrate commands
async fn handle_migrate(action: MigrateAction, output_format: OutputFormat) -> Result<()> {
    match action {
        MigrateAction::Plan {
            from,
            to,
            components,
            output,
        } => generate_plan(&from, &to, &components, &output, output_format).await,

        MigrateAction::Execute { plan, dry_run } => {
            execute_migration(&plan, dry_run, output_format).await
        }
    }
}

/// Generate migration plan
async fn generate_plan(
    from: &str,
    to: &str,
    components: &str,
    output: &PathBuf,
    output_format: OutputFormat,
) -> Result<()> {
    // Parse components
    let component_list: Vec<String> = components
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Validate backends
    if from != "sled" {
        return Err(anyhow!(
            "Phase 1: Only 'sled' is supported as source backend"
        ));
    }
    if to != "postgres" {
        return Err(anyhow!(
            "Phase 1: Only 'postgres' is supported as target backend"
        ));
    }

    // Validate components (Phase 1)
    for component in &component_list {
        match component.as_str() {
            "agent_state" | "workflow_state" | "sessions" => {}
            _ => {
                return Err(anyhow!(
                    "Phase 1: Only agent_state, workflow_state, sessions are supported. Got: {}",
                    component
                ));
            }
        }
    }

    // Create plan
    let mut plan = MigrationPlan::new(from, to, component_list);

    // Populate estimated counts from source
    let source = create_source_backend(from)?;
    for component in &mut plan.components {
        let count = count_records(&*source, &component.name).await?;
        component.estimated_count = count;
    }

    // Save plan
    plan.save(output)
        .with_context(|| format!("Failed to save migration plan to {:?}", output))?;

    // Output result
    match output_format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "success": true,
                "plan_file": output,
                "components": plan.components.len(),
                "estimated_records": plan.components.iter().map(|c| c.estimated_count).sum::<usize>(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            println!("Migration plan generated: {:?}", output);
            println!("\nPlan summary:");
            println!("  Source: {} → Target: {}", plan.source.backend, plan.target.backend);
            println!("  Components: {}", plan.components.len());
            for component in &plan.components {
                println!(
                    "    - {}: {} records (batch size: {})",
                    component.name, component.estimated_count, component.batch_size
                );
            }
            println!("\nNext steps:");
            println!("  1. Review plan: cat {:?}", output);
            println!("  2. Dry-run: llmspell storage migrate execute --plan {:?} --dry-run", output);
            println!("  3. Execute: llmspell storage migrate execute --plan {:?}", output);
        }
    }

    Ok(())
}

/// Execute migration
async fn execute_migration(
    plan_path: &PathBuf,
    dry_run: bool,
    output_format: OutputFormat,
) -> Result<()> {
    // Load plan
    let plan = MigrationPlan::from_file(plan_path)
        .with_context(|| format!("Failed to load migration plan from {:?}", plan_path))?;

    // Create source and target backends
    let source = create_source_backend(&plan.source.backend)?;
    let target = create_target_backend(&plan.target.backend).await?;

    // Create migration engine
    let engine = MigrationEngine::new(source, target, plan.clone());

    // Execute migration
    match output_format {
        OutputFormat::Text | OutputFormat::Pretty => {
            if dry_run {
                println!("[DRY-RUN] Migration validation starting...");
            } else {
                println!("Migration starting...");
            }
        }
        _ => {}
    }

    let report = engine
        .execute(dry_run)
        .await
        .context("Migration execution failed")?;

    // Output result
    match output_format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "success": report.success,
                "dry_run": dry_run,
                "components": report.components,
                "source_count": report.source_count,
                "target_count": report.target_count,
                "duration_seconds": report.duration.num_seconds(),
                "records_per_second": report.records_per_second,
                "validation_results": report.validation_results,
                "errors": report.errors,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            println!("\n{}", report.format());
        }
    }

    if !report.success {
        return Err(anyhow!("Migration failed"));
    }

    Ok(())
}

/// Handle info command
async fn handle_info(backend: String, output_format: OutputFormat) -> Result<()> {
    match backend.as_str() {
        "sled" => {
            let sled = SledBackend::new()?;
            let chars = sled.characteristics();

            match output_format {
                OutputFormat::Json => {
                    let json = serde_json::json!({
                        "backend": "sled",
                        "persistent": chars.persistent,
                        "transactional": chars.transactional,
                        "supports_prefix_scan": chars.supports_prefix_scan,
                        "supports_atomic_ops": chars.supports_atomic_ops,
                        "avg_read_latency_us": chars.avg_read_latency_us,
                        "avg_write_latency_us": chars.avg_write_latency_us,
                    });
                    println!("{}", serde_json::to_string_pretty(&json)?);
                }
                OutputFormat::Text | OutputFormat::Pretty => {
                    println!("Backend: Sled");
                    println!("Persistent: {}", chars.persistent);
                    println!("Transactional: {}", chars.transactional);
                    println!("Prefix Scan: {}", chars.supports_prefix_scan);
                    println!("Atomic Ops: {}", chars.supports_atomic_ops);
                    println!("Read Latency: {}μs", chars.avg_read_latency_us);
                    println!("Write Latency: {}μs", chars.avg_write_latency_us);
                }
            }
        }
        "postgres" => {
            // TODO: Get connection string from config
            return Err(anyhow!("PostgreSQL info requires configuration"));
        }
        _ => {
            return Err(anyhow!("Unknown backend: {}", backend));
        }
    }

    Ok(())
}

/// Handle validate command
async fn handle_validate(
    backend: String,
    components: String,
    output_format: OutputFormat,
) -> Result<()> {
    let component_list: Vec<String> = components
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    match backend.as_str() {
        "sled" => {
            let sled = SledBackend::new()?;
            for component in component_list {
                let count = count_records(&sled, &component).await?;
                match output_format {
                    OutputFormat::Json => {
                        let json = serde_json::json!({
                            "component": component,
                            "count": count,
                        });
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    }
                    OutputFormat::Text | OutputFormat::Pretty => {
                        println!("{}: {} records", component, count);
                    }
                }
            }
        }
        "postgres" => {
            // TODO: PostgreSQL validation requires connection
            return Err(anyhow!("PostgreSQL validation requires configuration"));
        }
        _ => {
            return Err(anyhow!("Unknown backend: {}", backend));
        }
    }

    Ok(())
}

/// Create source backend
fn create_source_backend(backend: &str) -> Result<Arc<dyn MigrationSource>> {
    match backend {
        "sled" => {
            let sled = SledBackend::new()?;
            Ok(Arc::new(sled))
        }
        _ => Err(anyhow!("Unsupported source backend: {}", backend)),
    }
}

/// Create target backend
#[cfg(feature = "postgres")]
async fn create_target_backend(backend: &str) -> Result<Arc<dyn MigrationTarget>> {
    match backend {
        "postgres" => {
            // Use default connection string for now
            // TODO: Get from config
            let connection_string =
                "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";
            let config = PostgresConfig::new(connection_string);
            let postgres = PostgresBackend::new(config)
                .await
                .context("Failed to connect to PostgreSQL")?;
            Ok(Arc::new(postgres))
        }
        _ => Err(anyhow!("Unsupported target backend: {}", backend)),
    }
}

/// Create target backend (fallback when postgres feature not enabled)
#[cfg(not(feature = "postgres"))]
async fn create_target_backend(backend: &str) -> Result<Arc<dyn MigrationTarget>> {
    Err(anyhow!(
        "Target backend '{}' requires 'postgres' feature to be enabled. \
         Please rebuild with --features postgres",
        backend
    ))
}

/// Count records for a component
async fn count_records(backend: &dyn MigrationSource, component: &str) -> Result<usize> {
    backend.count(component).await
}
