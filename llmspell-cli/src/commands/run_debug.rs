//! Debug execution support for run command

use crate::cli::OutputFormat;
use crate::commands::run::parse_script_args;
use crate::kernel::{
    CliKernelDiscovery, DebugExecutionHandle, KernelConnectionBuilder, KernelConnectionTrait,
};
use crate::output::format_output;
use anyhow::Result;
use llmspell_bridge::{
    circuit_breaker::ExponentialBackoffBreaker,
    diagnostics_bridge::DiagnosticsBridge,
    engine::{ScriptMetadata, ScriptOutput},
};
use llmspell_config::LLMSpellConfig;
use std::path::PathBuf;
use tracing::info;

/// Execute script in debug mode via kernel connection
pub async fn execute_script_debug(
    script_content: String,
    script_path: PathBuf,
    runtime_config: LLMSpellConfig,
    args: Vec<String>,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Starting debug execution for: {}", script_path.display());

    // Generate debug session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Create kernel connection with debug support
    let mut kernel = create_debug_kernel(runtime_config).await?;

    // Check if debug is supported
    if !kernel.supports_debug() {
        anyhow::bail!("Debug mode not supported by current kernel configuration");
    }

    // Execute script with debug support
    let debug_handle = kernel
        .execute_script_debug(&script_content, args.clone(), session_id.clone())
        .await?;

    // Handle debug execution
    handle_debug_execution(debug_handle, output_format).await?;

    // Disconnect kernel
    kernel.disconnect().await?;

    Ok(())
}

/// Create a kernel connection configured for debugging
async fn create_debug_kernel(
    _runtime_config: LLMSpellConfig,
) -> Result<Box<dyn KernelConnectionTrait>> {
    let mut kernel = KernelConnectionBuilder::new()
        .discovery(Box::new(CliKernelDiscovery::new()))
        .circuit_breaker(Box::new(ExponentialBackoffBreaker::default()))
        .diagnostics(DiagnosticsBridge::builder().build())
        .build();

    // Connect to kernel or start new one
    kernel.connect_or_start().await?;

    Ok(Box::new(kernel))
}

/// Handle debug execution events
async fn handle_debug_execution(
    handle: DebugExecutionHandle,
    _output_format: OutputFormat,
) -> Result<()> {
    info!("Debug session started: {}", handle.session_id);

    // Get shared context for monitoring
    let context = handle.shared_context.read().await;
    if let Some(correlation_id) = &context.correlation_id {
        info!("Correlation ID: {}", correlation_id);
    }

    // In a real implementation, this would:
    // 1. Monitor debug events from the kernel
    // 2. Handle breakpoints, step operations, etc.
    // 3. Display variables and stack traces
    // 4. Process debug commands from user input

    // For now, just indicate debug mode is active
    println!(
        "Debug execution completed for session: {}",
        handle.session_id
    );
    println!("Use debug commands to interact with the session");

    Ok(())
}

/// Execute script in non-debug mode (traditional execution)
pub async fn execute_script_nondebug(
    script_content: String,
    script_path: PathBuf,
    _engine: crate::cli::ScriptEngine, // Engine selection handled by kernel
    runtime_config: LLMSpellConfig,
    _stream: bool, // Streaming handled differently in kernel mode
    args: Vec<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Parse script arguments
    let parsed_args = parse_script_args(args, &script_path);
    if !parsed_args.is_empty() {
        tracing::debug!("Parsed script arguments: {:?}", parsed_args);
    }

    // Create kernel connection instead of direct runtime
    let mut kernel = crate::commands::create_kernel_connection(runtime_config).await?;

    // Execute script via kernel connection
    // TODO: Add support for script arguments in kernel protocol
    let result = kernel.execute(&script_content).await?;

    // Create ScriptOutput from kernel result
    let script_output = ScriptOutput {
        output: result,
        console_output: vec![], // TODO: Get console output from kernel
        metadata: ScriptMetadata {
            engine: "kernel".to_string(),
            execution_time_ms: 0, // TODO: Get timing from kernel
            memory_usage_bytes: None,
            warnings: vec![],
        },
    };

    // Format and display the result
    println!("{}", format_output(&script_output, output_format)?);

    Ok(())
}
