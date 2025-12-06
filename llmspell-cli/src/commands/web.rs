use crate::cli::WebCommands;
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::start_embedded_kernel_with_executor;
use llmspell_web::{config::WebConfig, server::WebServer};
use std::sync::Arc;
use llmspell_bridge::ScriptRuntime;

/// Handle web commands
pub async fn handle_web_command(
    command: WebCommands,
    config: LLMSpellConfig,
) -> Result<()> {
    match command {
        WebCommands::Start {
            port,
            host,
            daemon: _, // Handled by main.rs entry point
            pid_file: _,
            log_file: _,
        } => {
            // 1. Create Runtime
            let runtime = ScriptRuntime::new(config.clone())
                .await
                .expect("Failed to create runtime");
            let executor = Arc::new(runtime);

            // 2. Start Kernel
            let kernel_handle = start_embedded_kernel_with_executor(config, executor)
                .await
                .expect("Failed to start kernel");

            // 3. Configure Web Server
            let web_config = WebConfig {
                port,
                host,
                ..Default::default()
            };

            println!("Starting web server on http://{}:{}", web_config.host, web_config.port);

            // 4. Run Server
            WebServer::run(web_config, kernel_handle).await?;

            Ok(())
        }

        WebCommands::Stop { pid_file } => {
            use llmspell_kernel::daemon::PidFile;
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            use std::path::PathBuf;

            let pid_path = pid_file.unwrap_or_else(|| {
                PathBuf::from("/tmp").join("llmspell-kernel-web.pid")
            });

            let pid_file = PidFile::new(pid_path.clone());
            if pid_file.is_running()? {
                let pid = pid_file.read_pid()?;
                println!("Stopping web server (PID: {}, PID file: {:?})", pid, pid_path);
                
                // Send SIGTERM
                kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;
                println!("Signal sent.");
                
                // Optional: remove pid file, but process usually does it or OS handles it?
                // PidFile::new(pid_path).remove()?; // Actually PidFile owns file, maybe leave cleanup to process shutdown
            } else {
                println!("Web server is not running (PID file: {:?})", pid_path);
            }
            Ok(())
        }

        WebCommands::Status { pid_file } => {
            use llmspell_kernel::daemon::PidFile;
            use std::path::PathBuf;

            let pid_path = pid_file.unwrap_or_else(|| {
                PathBuf::from("/tmp").join("llmspell-kernel-web.pid")
            });

            let pid_file = PidFile::new(pid_path.clone());
            if pid_file.is_running()? {
                let pid = pid_file.read_pid()?;
                println!("Web server is running (PID: {}, PID file: {:?})", pid, pid_path);
            } else {
                println!("Web server is not running (PID file: {:?})", pid_path);
            }
            Ok(())
        }
    }
}
