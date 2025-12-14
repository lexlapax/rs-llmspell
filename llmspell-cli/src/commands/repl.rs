//! ABOUTME: REPL command implementation for interactive scripting
//! ABOUTME: Provides an interactive read-eval-print loop

use crate::cli::{OutputFormat, ScriptEngine};
use crate::execution_context::ExecutionContext;
use anyhow::Result;
use llmspell_kernel::api::{ClientHandle, KernelHandle};
use llmspell_kernel::repl::{InteractiveSession, ReplSessionConfig};
use std::path::PathBuf;

/// Start interactive REPL session
pub async fn start_repl(
    engine: ScriptEngine,
    context: ExecutionContext,
    history: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<()> {
    println!("LLMSpell REPL - {} engine", engine.as_str());
    println!("Type '.exit' or press Ctrl+D to quit");
    println!();

    // Create session configuration
    let mut session_config = ReplSessionConfig::default();
    if let Some(path) = history {
        session_config.history_file = Some(path);
    }

    // Execute based on context type
    match context {
        ExecutionContext::Embedded { handle, config: _ } => {
            start_embedded_repl(*handle, session_config, engine, output_format).await?;
        }
        ExecutionContext::Connected { handle, address: _ } => {
            start_connected_repl(handle, session_config, engine, output_format).await?;
        }
    }

    Ok(())
}

/// Start REPL with embedded kernel
async fn start_embedded_repl(
    handle: KernelHandle,
    session_config: ReplSessionConfig,
    _engine: ScriptEngine,
    _output_format: OutputFormat,
) -> Result<()> {
    // Create interactive session (Direct mode)
    let kernel = handle.into_kernel()?;
    let mut session = InteractiveSession::new(kernel, session_config).await?;

    // Run REPL loop
    session.run_repl().await?;

    Ok(())
}

/// Start REPL with connected kernel
async fn start_connected_repl(
    _handle: ClientHandle,
    _session_config: ReplSessionConfig,
    engine: ScriptEngine,
    output_format: OutputFormat,
) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpStream;

    // TODO: Get the connection address from the client handle
    // For now, use localhost:9999 as the default REPL server address
    let repl_address = "127.0.0.1:9999";

    println!("Connecting to REPL server at {}...", repl_address);

    // Connect to REPL server
    let stream = TcpStream::connect(&repl_address).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Read and display welcome message
    let mut line = String::new();
    loop {
        reader.read_line(&mut line).await?;
        print!("{}", line);
        if line.contains("> ") {
            break;
        }
        line.clear();
    }

    // Set up readline for local input
    use rustyline::DefaultEditor;
    let mut rl = DefaultEditor::new()?;

    // Main REPL client loop
    loop {
        // Read user input
        let input = match rl.readline("") {
            Ok(line) => {
                rl.add_history_entry(&line)?;
                line
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("Disconnecting...");
                writer.write_all(b".exit\n").await?;
                break;
            }
            Err(err) => {
                println!("Error reading input: {}", err);
                break;
            }
        };

        // Send to server
        writer.write_all(format!("{}\n", input).as_bytes()).await?;
        writer.flush().await?;

        // Read response
        line.clear();
        loop {
            let bytes_read = reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                println!("Server disconnected");
                return Ok(());
            }

            print!("{}", line);

            // Check if we've received the prompt
            if line.contains("> ") {
                break;
            }
            line.clear();
        }

        // Check for exit command
        if input.trim() == ".exit" {
            break;
        }
    }

    if output_format == OutputFormat::Json {
        println!(
            "{}",
            serde_json::json!({
                "status": "repl_disconnected",
                "mode": "connected",
                "engine": engine.as_str(),
            })
        );
    }

    Ok(())
}
