//! Debug command implementation using DebugBridge
//!
//! Provides dedicated debug command that uses the Bridge Pattern architecture
//! from llmspell-engine for hybrid local/protocol debugging support.

use crate::cli::{OutputFormat, ScriptEngine};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_engine::{DebugBridge, DebugConfig, DebugMode, LocalDebugConfig, PerformanceConfig};
use std::path::PathBuf;
use std::sync::Arc;

/// Handle the debug command using DebugBridge architecture
///
/// Creates a DebugBridge in local mode for current CLI usage, with Task 9.7
/// protocol mode prepared for future kernel-hub architecture transition.
pub async fn handle_debug_command(
    script: PathBuf,
    args: Vec<String>,
    engine: ScriptEngine,
    config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(
        "🐛 Starting debug session for: {} (engine: {})",
        script.display(),
        engine.as_str()
    );

    // Read script content
    let script_content = tokio::fs::read_to_string(&script)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read script file {}: {}", script.display(), e))?;

    // Create DebugBridge configuration in local mode (current)
    // Task 9.7: This will switch to protocol mode when CLI adopts kernel-hub architecture
    // TODO: ARCHITECTURAL ISSUE - Consolidate two DebugConfig structs
    // Currently: config.debug (logging) vs config.engine.debug (features)
    // Should be: single comprehensive DebugConfig
    let debug_config = DebugConfig {
        mode: DebugMode::Local(LocalDebugConfig {
            script_path: Some(script.clone()),
            enable_breakpoints: config.engine.debug.breakpoints_enabled,
            enable_stepping: config.engine.debug.step_debugging_enabled,
            enable_variable_inspection: config.engine.debug.variable_inspection_enabled,
            enable_stack_navigation: true, // Not in engine config, default enabled
            performance: PerformanceConfig {
                init_target_ms: 10, // <10ms initialization target
                state_target_ms: 1, // <1ms state operations target
                monitoring_enabled: config.debug.performance.enabled,
            },
        }),
        performance: PerformanceConfig {
            init_target_ms: 10,
            state_target_ms: 1,
            monitoring_enabled: config.debug.performance.enabled,
        },
    };

    // Create DebugBridge with performance monitoring
    let debug_bridge = DebugBridge::new(debug_config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create DebugBridge: {}", e))?;

    // Wire Protocol Adapters - Protocol-First Unification Architecture
    // Create and register debug infrastructure adapters for protocol-based access
    {
        use llmspell_bridge::debug_adapters::*;
        use llmspell_bridge::debug_state_cache::SharedDebugStateCache;
        use llmspell_bridge::execution_bridge::ExecutionManager;
        use llmspell_bridge::execution_context::SharedExecutionContext;
        use llmspell_bridge::lua::stack_navigator_impl::LuaStackNavigator;
        use llmspell_bridge::variable_inspector::SharedVariableInspector;

        // Create shared state cache for debug infrastructure
        let state_cache = Arc::new(SharedDebugStateCache::new());

        // Create execution manager
        let execution_manager = Arc::new(ExecutionManager::new(state_cache.clone()));

        // Create execution context for variable inspector
        let execution_context = Arc::new(tokio::sync::RwLock::new(SharedExecutionContext::new()));

        // Create variable inspector
        let variable_inspector = Arc::new(SharedVariableInspector::new(
            state_cache.clone(),
            execution_context,
        ));

        // Create stack navigator (using Lua implementation for now)
        let stack_navigator = Arc::new(LuaStackNavigator::new());

        // Create session manager (stub for now)
        let session_adapter = Arc::new(DebugSessionManagerAdapter::new());

        // Create protocol adapters wrapping the debug components
        let exec_adapter = Arc::new(ExecutionManagerAdapter::new(
            execution_manager,
            String::from("cli-session"),
        ));
        let var_adapter = Arc::new(VariableInspectorAdapter::new(variable_inspector));
        let stack_adapter = Arc::new(StackNavigatorAdapter::new(stack_navigator));

        // Register adapters with DebugBridge
        debug_bridge
            .register_capability("execution_manager".to_string(), exec_adapter)
            .await;
        debug_bridge
            .register_capability("variable_inspector".to_string(), var_adapter)
            .await;
        debug_bridge
            .register_capability("stack_navigator".to_string(), stack_adapter)
            .await;
        debug_bridge
            .register_capability("session_manager".to_string(), session_adapter)
            .await;

        tracing::debug!("Registered {} protocol adapters", 4);
    }

    // Log Bridge capabilities
    let capabilities = debug_bridge.capabilities();
    tracing::debug!("DebugBridge capabilities: {:?}", capabilities);

    // Discover registered capabilities
    let discovered = debug_bridge.discover_capabilities().await;
    tracing::debug!("Discovered capabilities: {:?}", discovered);

    // Start local debug session
    let debug_session = debug_bridge
        .debug_local(&script_content)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start debug session: {}", e))?;

    // Create debug runtime with the session and capabilities
    let capabilities = debug_bridge.get_capability_registry();
    let debug_runtime =
        llmspell_bridge::DebugRuntime::new(config.clone(), debug_session.clone(), capabilities)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create debug runtime: {}", e))?;

    println!("🔧 Debug session started: {}", debug_session.session_id);
    println!("📝 Available debug commands:");
    println!("   .break <line>     - Set breakpoint at line");
    println!("   .step             - Step to next line");
    println!("   .continue         - Continue execution");
    println!("   .locals           - Show local variables");
    println!("   .stack            - Show call stack");
    println!("   .help             - Show debug help");
    println!("   .quit             - Exit debug session");
    println!();

    // Get performance stats before moving debug_bridge
    let (init_time, state_time) = debug_bridge.get_performance_stats().await;

    // Start interactive debug REPL with session and runtime
    start_debug_repl(
        debug_bridge,
        debug_session,
        debug_runtime,
        args,
        output_format,
    )
    .await?;

    // Display performance statistics
    if let Some(init) = init_time {
        tracing::info!("Debug initialization: {}ms", init.as_millis());
    }
    if let Some(state) = state_time {
        tracing::info!("Average state operation: {}ms", state.as_millis());
    }

    println!("🏁 Debug session completed");
    Ok(())
}

/// Start interactive debug REPL using DebugBridge and Runtime
///
/// Provides interactive debugging interface with commands like .break, .step, etc.
/// Uses the DebugRuntime for actual script execution with debug hooks.
async fn start_debug_repl(
    debug_bridge: DebugBridge,
    mut debug_session: llmspell_engine::DebugSession,
    mut debug_runtime: llmspell_bridge::DebugRuntime,
    args: Vec<String>,
    output_format: OutputFormat,
) -> Result<()> {
    use rustyline::DefaultEditor;

    let mut rl =
        DefaultEditor::new().map_err(|e| anyhow::anyhow!("Failed to create REPL: {}", e))?;

    // Set up history (similar to main REPL)
    let history_path = dirs::home_dir()
        .map(|home| home.join(".llmspell_debug_history"))
        .unwrap_or_else(|| PathBuf::from(".llmspell_debug_history"));

    if let Err(e) = rl.load_history(&history_path) {
        tracing::debug!("Could not load debug history: {}", e);
    }

    println!("Debug REPL started. Type .help for available commands.");
    println!("Type .run to execute the script with debugging enabled.");

    // Mark session as active
    debug_session.state = llmspell_engine::DebugSessionState::Active;

    loop {
        match rl.readline("debug> ") {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // Add to history
                rl.add_history_entry(line)
                    .map_err(|e| anyhow::anyhow!("History error: {}", e))?;

                // Handle debug commands
                match handle_debug_command_input(
                    &debug_bridge,
                    &mut debug_session,
                    &mut debug_runtime,
                    line,
                    &args,
                    output_format,
                )
                .await
                {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                    }
                    Err(e) => {
                        // Use enhanced error reporting for debug command errors
                        let enhanced_error = debug_bridge.create_enhanced_error(
                            &e.to_string(),
                            &debug_session.script_content,
                            None, // No specific line for command errors
                            None, // No column information
                            None, // No file path for interactive input
                        );
                        let formatted_error = debug_bridge.format_enhanced_error(&enhanced_error);
                        eprintln!("{}", formatted_error);
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("^C - Use .quit to exit debug session");
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("EOF - Exiting debug session");
                break;
            }
            Err(err) => {
                eprintln!("REPL error: {}", err);
                break;
            }
        }
    }

    // Save history
    if let Err(e) = rl.save_history(&history_path) {
        tracing::debug!("Could not save debug history: {}", e);
    }

    // Mark session as completed
    debug_session.state = llmspell_engine::DebugSessionState::Completed;

    Ok(())
}

/// Handle individual debug command input
///
/// Processes debug commands like .break, .step, .continue, .run, etc.
/// Returns Ok(true) to continue REPL, Ok(false) to exit.
async fn handle_debug_command_input(
    debug_bridge: &DebugBridge,
    debug_session: &mut llmspell_engine::DebugSession,
    debug_runtime: &mut llmspell_bridge::DebugRuntime,
    input: &str,
    _args: &[String],
    _output_format: OutputFormat,
) -> Result<bool> {
    // Handle debug dot commands
    if input.starts_with('.') {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];

        match command {
            ".help" => {
                print_debug_help();
                Ok(true)
            }
            ".quit" | ".exit" => {
                println!("Exiting debug session...");
                Ok(false)
            }
            ".run" => {
                println!("🚀 Executing script with debug hooks...");
                match debug_runtime.execute().await {
                    Ok(output) => {
                        println!("✅ Script execution completed");
                        if !output.output.is_null() {
                            println!("Result: {:?}", output.output);
                        }
                        if !output.console_output.is_empty() {
                            println!("Console output:");
                            for line in &output.console_output {
                                println!("  {}", line);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Execution failed: {}", e);
                    }
                }
                Ok(true)
            }
            ".break" => {
                if parts.len() < 2 {
                    println!("Usage: .break <line_number>");
                } else {
                    match parts[1].parse::<u32>() {
                        Ok(line) => {
                            println!("Setting breakpoint at line {}", line);
                            // Send breakpoint request through the runtime
                            let request = llmspell_core::debug::DebugRequest::SetBreakpoints {
                                source: "<debug_script>".to_string(),
                                breakpoints: vec![(line, None)],
                            };
                            match debug_runtime.process_debug_command(request).await {
                                Ok(response) => {
                                    println!("Breakpoint set: {:?}", response);
                                }
                                Err(e) => {
                                    eprintln!("Failed to set breakpoint: {}", e);
                                }
                            }
                        }
                        Err(_) => {
                            println!("Error: Invalid line number '{}'", parts[1]);
                        }
                    }
                }
                Ok(true)
            }
            ".step" => {
                println!("Stepping to next line...");
                debug_runtime.step_over().await;
                Ok(true)
            }
            ".continue" => {
                println!("Continuing execution...");
                debug_runtime.resume().await;
                Ok(true)
            }
            ".locals" => {
                println!("Local variables:");
                // TODO: Integrate with VariableInspector via DebugBridge
                // This will show actual variable values from the existing infrastructure
                println!(
                    "  (variable inspection will be integrated with existing VariableInspector)"
                );
                Ok(true)
            }
            ".stack" => {
                println!("Call stack:");
                // TODO: Integrate with StackNavigator via DebugBridge
                // This will show actual stack frames from the existing infrastructure
                println!("  (stack navigation will be integrated with existing StackNavigator)");
                Ok(true)
            }
            ".sessions" => {
                let sessions = debug_bridge.get_sessions().await;
                println!("Active debug sessions: {}", sessions.len());
                for (id, session) in sessions {
                    println!(
                        "  {}: {:?} (started {}s ago)",
                        id,
                        session.state,
                        session.start_time.elapsed().as_secs()
                    );
                }
                Ok(true)
            }
            ".capabilities" => {
                let caps = debug_bridge.capabilities();
                println!("DebugBridge capabilities:");
                for cap in caps {
                    println!("  - {}", cap);
                }
                Ok(true)
            }
            ".stats" => {
                let (init_time, state_time) = debug_bridge.get_performance_stats().await;
                println!("Performance statistics:");
                if let Some(init) = init_time {
                    println!("  Average initialization time: {}ms", init.as_millis());
                }
                if let Some(state) = state_time {
                    println!("  Average state operation time: {}ms", state.as_millis());
                }
                println!(
                    "  Task 9.7 protocol ready: {}",
                    debug_bridge.is_protocol_ready()
                );
                Ok(true)
            }
            _ => {
                println!(
                    "Unknown debug command: {}. Type .help for available commands.",
                    command
                );
                Ok(true)
            }
        }
    } else {
        // Handle regular Lua code execution in debug mode
        println!("Executing in debug context: {}", input);

        // TODO: Execute code through DebugBridge with debug hooks
        // This would integrate with the existing ScriptRuntime with debug support
        // For now, simulate error handling with enhanced error reporting

        // Example: If there were a syntax error, show enhanced error display
        if input.contains("syntax_error_demo") {
            let enhanced_error = debug_bridge.create_enhanced_error(
                "Syntax error: unexpected symbol near 'syntax_error_demo'",
                &debug_session.script_content,
                Some(1),  // Line number would come from actual parser
                Some(10), // Column would come from actual parser
                None,     // Interactive session has no file path
            );
            let formatted_error = debug_bridge.format_enhanced_error(&enhanced_error);
            println!("{}", formatted_error);
            return Ok(true);
        }

        println!("  (code execution will be integrated with existing debug-enabled ScriptRuntime)");
        println!("  💡 Type 'syntax_error_demo' to see enhanced error reporting in action");

        Ok(true)
    }
}

/// Print debug help information
fn print_debug_help() {
    println!("Debug Commands:");
    println!("  .help             - Show this help message");
    println!("  .quit, .exit      - Exit debug session");
    println!("  .break <line>     - Set breakpoint at line number");
    println!("  .step             - Step to next line of execution");
    println!("  .continue         - Continue execution until next breakpoint");
    println!("  .locals           - Show local variables in current scope");
    println!("  .stack            - Show call stack");
    println!("  .sessions         - Show active debug sessions");
    println!("  .capabilities     - Show DebugBridge capabilities");
    println!("  .stats            - Show performance statistics");
    println!();
    println!("Code Execution:");
    println!("  Any non-command input will be executed in debug context");
    println!("  Example: print('debug test')");
    println!();
    println!("Enhanced Error Reporting:");
    println!("  ✅ Source context with line highlighting");
    println!("  ✅ Error type classification and suggestions");
    println!("  ✅ Documentation references");
    println!("  💡 Try 'syntax_error_demo' to see enhanced errors");
    println!();
    println!("Integration Status:");
    println!("  ✅ DebugBridge architecture with Bridge Pattern");
    println!("  ✅ Local debugging mode (current)");
    println!("  ✅ Protocol debugging mode prepared (Task 9.7)");
    println!("  ✅ Enhanced error reporting with source context");
    println!("  🔄 ExecutionManager integration (TODO)");
    println!("  🔄 VariableInspector integration (TODO)");
    println!("  🔄 StackNavigator integration (TODO)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_debug_command_with_simple_script() {
        let temp_dir = tempdir().unwrap();
        let script_path = temp_dir.path().join("test_debug.lua");

        // Create a simple test script
        tokio::fs::write(&script_path, "print('Debug test script')")
            .await
            .unwrap();

        let _config = LLMSpellConfig::default();

        // Test that debug command can be created (without running full REPL)
        let debug_config = DebugConfig {
            mode: DebugMode::Local(LocalDebugConfig::default()),
            performance: PerformanceConfig::default(),
        };

        let debug_bridge = DebugBridge::new(debug_config).await.unwrap();
        let script_content = tokio::fs::read_to_string(&script_path).await.unwrap();
        let session = debug_bridge.debug_local(&script_content).await.unwrap();

        assert!(!session.session_id.is_empty());
        assert_eq!(
            session.state,
            llmspell_engine::DebugSessionState::Initialized
        );
    }

    #[tokio::test]
    async fn test_debug_bridge_capabilities() {
        let config = DebugConfig {
            mode: DebugMode::Local(LocalDebugConfig::default()),
            performance: PerformanceConfig::default(),
        };

        let debug_bridge = DebugBridge::new(config).await.unwrap();
        let capabilities = debug_bridge.capabilities();

        assert!(capabilities.contains(&"local_debugging".to_string()));
        assert!(capabilities.contains(&"breakpoints".to_string()));
        assert!(capabilities.contains(&"stepping".to_string()));
        assert!(capabilities.contains(&"variable_inspection".to_string()));
        assert!(capabilities.contains(&"stack_navigation".to_string()));
    }
}
