//! Kernel command implementation - thin wrapper around kernel API
//!
//! This module provides CLI commands for kernel operations.
//! All kernel logic is in llmspell-kernel, this is just command handling.

use anyhow::{anyhow, Result};
use colored::Colorize;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{
    api::KernelServiceConfig, connect_to_kernel, daemon::DaemonConfig, execution::ExecutionConfig,
    monitoring::HealthThresholds, start_kernel_service_with_config,
};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tabled::{builder::Builder, settings::Style};
use tracing::{info, warn};

use crate::kernel_discovery::{self, KernelInfo, KernelMetrics, KernelStatus};

/// Handle kernel management commands
pub async fn handle_kernel_command(
    command: crate::cli::KernelCommands,
    runtime_config: LLMSpellConfig,
    _output_format: crate::cli::OutputFormat,
) -> Result<()> {
    use crate::cli::KernelCommands;

    match command {
        KernelCommands::Start {
            port,
            daemon,
            id,
            connection_file,
            log_file,
            pid_file,
            idle_timeout,
            max_clients,
            log_rotate_size,
            log_rotate_count,
        } => {
            // Build daemon configuration if daemon mode requested
            let daemon_config = if daemon {
                // Set default paths if not provided
                let default_pid_path = || {
                    dirs::runtime_dir()
                        .unwrap_or_else(|| PathBuf::from("/tmp"))
                        .join(format!(
                            "llmspell-kernel-{}.pid",
                            id.as_ref().unwrap_or(&format!("port-{}", port))
                        ))
                };

                let default_log_path = || {
                    dirs::state_dir()
                        .unwrap_or_else(|| {
                            dirs::home_dir()
                                .unwrap_or_else(|| PathBuf::from("/tmp"))
                                .join(".llmspell")
                        })
                        .join("logs")
                        .join(format!(
                            "kernel-{}.log",
                            id.as_ref().unwrap_or(&format!("port-{}", port))
                        ))
                };

                Some(DaemonConfig {
                    daemonize: true,
                    pid_file: Some(pid_file.unwrap_or_else(default_pid_path)),
                    working_dir: PathBuf::from("/"),
                    stdout_path: log_file.clone().or_else(|| Some(default_log_path())),
                    stderr_path: log_file.or_else(|| Some(default_log_path())),
                    close_stdin: true,
                    umask: Some(0o027),
                })
            } else {
                None
            };

            // Build execution config with daemon settings
            let exec_config = ExecutionConfig {
                runtime_config: serde_json::to_value(&runtime_config)
                    .ok()
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                io_config: llmspell_kernel::execution::IOConfig::default(),
                max_history: 1000,
                execution_timeout_secs: idle_timeout,
                monitor_agents: true,
                track_performance: true,
                daemon_mode: daemon,
                daemon_config: daemon_config.clone(),
                health_thresholds: Some(HealthThresholds::default()),
            };

            // Always use service mode (with ZeroMQ transport) for the start command
            // Service mode - start kernel that listens for connections
            info!(
                "Starting kernel service on port {} {}",
                port,
                if daemon { "in daemon mode" } else { "" }
            );

            // Create log directory if needed
            if let Some(ref config) = daemon_config {
                if let Some(ref log_path) = config.stdout_path {
                    if let Some(parent) = log_path.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                }
            }

            // Create real ScriptExecutor from llmspell-bridge
            let script_executor =
                llmspell_bridge::create_script_executor(runtime_config.clone()).await?;

            // Create service configuration
            let service_config = KernelServiceConfig {
                port,
                exec_config,
                kernel_id: id,
                connection_file_path: connection_file,
                max_clients,
                log_rotate_size,
                log_rotate_count,
                script_executor,
            };

            let service = start_kernel_service_with_config(service_config).await?;

            info!(
                "Kernel service started. Connection file: {:?}",
                service.connection_file()
            );
            service.run().await
        }

        KernelCommands::Connect { address } => {
            // Client mode - connect to existing kernel
            match address {
                Some(addr) => {
                    info!("Connecting to kernel at: {}", addr);
                    let mut client = connect_to_kernel(&addr).await?;
                    client.run_repl()
                }
                None => {
                    info!("No address provided, using default connection");
                    anyhow::bail!(
                        "No kernel address provided and auto-discovery not yet implemented"
                    )
                }
            }
        }

        KernelCommands::Stop {
            id,
            pid_file,
            all,
            force,
            timeout,
            no_cleanup,
        } => {
            // Validate arguments
            if id.is_some() && pid_file.is_some() {
                anyhow::bail!("Cannot specify both --id and --pid-file");
            }
            if all && (id.is_some() || pid_file.is_some()) {
                anyhow::bail!("Cannot use --all with --id or --pid-file");
            }
            if !all && id.is_none() && pid_file.is_none() {
                anyhow::bail!("Must specify --id, --pid-file, or --all");
            }

            // Find kernels to stop
            let kernels_to_stop = if all {
                info!("Discovering all running kernels...");
                kernel_discovery::discover_kernels()?
            } else if let Some(kernel_id) = id {
                info!("Finding kernel with ID: {}", kernel_id);
                vec![kernel_discovery::find_kernel_by_id(&kernel_id)?]
            } else if let Some(pid_file_path) = pid_file {
                info!("Reading PID from file: {}", pid_file_path.display());

                // Read PID from file
                let pid_str = fs::read_to_string(&pid_file_path)
                    .map_err(|e| anyhow!("Failed to read PID file: {}", e))?;
                let pid: u32 = pid_str
                    .trim()
                    .parse()
                    .map_err(|e| anyhow!("Invalid PID in file: {}", e))?;

                // Create minimal KernelInfo
                vec![KernelInfo {
                    id: format!("pid-{}", pid),
                    pid,
                    port: 0, // Unknown
                    connection_file: PathBuf::new(),
                    pid_file: Some(pid_file_path),
                    log_file: None,
                    status: kernel_discovery::KernelStatus::Unknown,
                    start_time: None,
                }]
            } else {
                vec![]
            };

            if kernels_to_stop.is_empty() {
                println!("No kernels found to stop");
                return Ok(());
            }

            // Stop each kernel
            let mut stopped_count = 0;
            let mut failed_count = 0;

            for kernel in kernels_to_stop {
                println!("Stopping kernel {} (PID: {})", kernel.id, kernel.pid);

                match stop_kernel_process(&kernel, force, timeout, no_cleanup).await {
                    Ok(()) => {
                        println!("✓ Kernel {} stopped successfully", kernel.id);
                        stopped_count += 1;
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to stop kernel {}: {}", kernel.id, e);
                        failed_count += 1;
                    }
                }
            }

            // Report results
            if failed_count == 0 {
                println!("\n✓ Successfully stopped {} kernel(s)", stopped_count);
                Ok(())
            } else {
                anyhow::bail!(
                    "Failed to stop {} kernel(s), succeeded with {} kernel(s)",
                    failed_count,
                    stopped_count
                )
            }
        }

        KernelCommands::Status {
            id,
            format,
            quiet,
            watch,
            interval,
        } => {
            // Parse output format from string
            let output_format = match format.as_str() {
                "json" => crate::cli::OutputFormat::Json,
                "yaml" => crate::cli::OutputFormat::Yaml,
                "table" | "pretty" => crate::cli::OutputFormat::Pretty,
                _ => crate::cli::OutputFormat::Text,
            };

            // Check for detailed mode based on format string
            let detailed = format.contains("detailed");

            // Handle watch mode
            if watch {
                loop {
                    // Clear screen for watch mode
                    print!("\x1B[2J\x1B[1;1H");

                    display_kernel_status(id.as_deref(), quiet, detailed, &output_format)?;

                    tokio::time::sleep(Duration::from_secs(interval)).await;
                }
            } else {
                display_kernel_status(id.as_deref(), quiet, detailed, &output_format)
            }
        }

        KernelCommands::InstallService {
            service_type,
            system,
            name,
            port,
            id,
            log_file,
            pid_file,
            enable,
            start,
            force,
        } => {
            let config = InstallServiceConfig {
                service_type,
                system,
                name,
                port,
                id,
                log_file,
                pid_file,
                enable,
                start,
                force,
            };
            install_kernel_service(config)
        }
    }
}

/// Stop a kernel process with graceful shutdown and cleanup
async fn stop_kernel_process(
    kernel: &KernelInfo,
    force: bool,
    timeout: u64,
    no_cleanup: bool,
) -> Result<()> {
    // Check if process is alive
    if !kernel_discovery::is_process_alive(kernel.pid) {
        info!("Kernel {} (PID {}) is not running", kernel.id, kernel.pid);

        // Still clean up files if requested
        if !no_cleanup {
            cleanup_kernel_files(kernel)?;
        }

        return Ok(());
    }

    // If force mode, skip graceful shutdown
    if force {
        warn!("Force killing kernel {} (PID {})", kernel.id, kernel.pid);
        signal::kill(Pid::from_raw(kernel.pid as i32), Signal::SIGKILL)
            .map_err(|e| anyhow!("Failed to send SIGKILL: {}", e))?;
    } else {
        // Send SIGTERM for graceful shutdown
        info!(
            "Sending SIGTERM to kernel {} (PID {})",
            kernel.id, kernel.pid
        );
        signal::kill(Pid::from_raw(kernel.pid as i32), Signal::SIGTERM)
            .map_err(|e| anyhow!("Failed to send SIGTERM: {}", e))?;

        // Wait for graceful shutdown
        let deadline = Instant::now() + Duration::from_secs(timeout);
        let mut last_check = Instant::now();

        while Instant::now() < deadline {
            if !kernel_discovery::is_process_alive(kernel.pid) {
                info!("Kernel {} stopped gracefully", kernel.id);
                break;
            }

            // Print progress every 5 seconds
            if Instant::now() - last_check > Duration::from_secs(5) {
                let remaining = (deadline - Instant::now()).as_secs();
                println!(
                    "  Waiting for graceful shutdown... {} seconds remaining",
                    remaining
                );
                last_check = Instant::now();
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Force kill if still alive
        if kernel_discovery::is_process_alive(kernel.pid) {
            warn!(
                "Kernel {} didn't shutdown gracefully within {} seconds, sending SIGKILL",
                kernel.id, timeout
            );
            signal::kill(Pid::from_raw(kernel.pid as i32), Signal::SIGKILL)
                .map_err(|e| anyhow!("Failed to send SIGKILL: {}", e))?;

            // Wait briefly for SIGKILL to take effect
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    // Verify process is dead
    if kernel_discovery::is_process_alive(kernel.pid) {
        return Err(anyhow!(
            "Failed to stop kernel {} (PID {})",
            kernel.id,
            kernel.pid
        ));
    }

    // Clean up files unless --no-cleanup was specified
    if !no_cleanup {
        cleanup_kernel_files(kernel)?;
    }

    Ok(())
}

/// Clean up kernel-related files (connection file, PID file)
fn cleanup_kernel_files(kernel: &KernelInfo) -> Result<()> {
    // Clean up connection file
    if !kernel.connection_file.as_os_str().is_empty() && kernel.connection_file.exists() {
        info!(
            "Removing connection file: {}",
            kernel.connection_file.display()
        );
        fs::remove_file(&kernel.connection_file)
            .map_err(|e| anyhow!("Failed to remove connection file: {}", e))?;
    }

    // Clean up PID file
    if let Some(ref pid_file) = kernel.pid_file {
        if pid_file.exists() {
            info!("Removing PID file: {}", pid_file.display());
            fs::remove_file(pid_file).map_err(|e| anyhow!("Failed to remove PID file: {}", e))?;
        }
    }

    // Note: We intentionally do NOT delete log files as they may be useful for debugging
    if let Some(ref log_file) = kernel.log_file {
        info!("Preserving log file for debugging: {}", log_file.display());
    }

    Ok(())
}

/// Display kernel status information
fn display_kernel_status(
    kernel_id: Option<&str>,
    quiet: bool,
    detailed: bool,
    output_format: &crate::cli::OutputFormat,
) -> Result<()> {
    use crate::cli::OutputFormat;

    // Discover kernels
    let kernels = if let Some(id) = kernel_id {
        // Find specific kernel
        vec![kernel_discovery::find_kernel_by_id(id)?]
    } else {
        // Find all kernels
        kernel_discovery::discover_kernels()?
    };

    if kernels.is_empty() {
        if !quiet {
            println!("No kernels currently running");
        }
        return Ok(());
    }

    // Collect metrics for each kernel
    let mut kernel_data = Vec::new();
    for kernel in &kernels {
        let metrics = kernel_discovery::get_kernel_metrics(kernel).ok();
        kernel_data.push((kernel, metrics));
    }

    match output_format {
        OutputFormat::Pretty => {
            if detailed {
                display_detailed_table(&kernel_data)?;
            } else {
                display_summary_table(&kernel_data)?;
            }
        }
        OutputFormat::Json => {
            display_json(&kernel_data)?;
        }
        OutputFormat::Yaml => {
            // Convert to YAML format
            println!(
                "{}",
                serde_yaml::to_string(
                    &kernel_data
                        .iter()
                        .map(|(k, m)| serde_json::json!({
                            "kernel": k,
                            "metrics": m,
                        }))
                        .collect::<Vec<_>>()
                )?
            );
        }
        OutputFormat::Text => {
            display_simple(&kernel_data, quiet)?;
        }
    }

    Ok(())
}

/// Display kernels in a summary table
fn display_summary_table(kernel_data: &[(&KernelInfo, Option<KernelMetrics>)]) -> Result<()> {
    let mut builder = Builder::default();

    // Add header
    builder.push_record(["ID", "PID", "Port", "Status", "CPU%", "Memory", "Uptime"]);

    for (kernel, metrics) in kernel_data {
        let status_str = format_status(&kernel.status);

        if let Some(m) = metrics {
            builder.push_record([
                &kernel.id,
                &kernel.pid.to_string(),
                &kernel.port.to_string(),
                &status_str,
                &format!("{:.1}", m.cpu_percent),
                &format_memory(m.memory_bytes),
                &format_duration(&m.uptime),
            ]);
        } else {
            builder.push_record([
                &kernel.id,
                &kernel.pid.to_string(),
                &kernel.port.to_string(),
                &status_str,
                "N/A",
                "N/A",
                "N/A",
            ]);
        }
    }

    let table = builder.build().with(Style::rounded()).to_string();
    println!("{}", table);

    Ok(())
}

/// Display kernels in a detailed table
fn display_detailed_table(kernel_data: &[(&KernelInfo, Option<KernelMetrics>)]) -> Result<()> {
    for (kernel, metrics) in kernel_data {
        println!("\n{}", "═".repeat(60));
        println!("Kernel: {}", kernel.id.bold());
        println!("{}", "─".repeat(60));

        let mut builder = Builder::default();
        builder.push_record(["Property", "Value"]);

        builder.push_record(["Process ID", &kernel.pid.to_string()]);
        builder.push_record(["Port", &kernel.port.to_string()]);
        builder.push_record(["Status", &format_status(&kernel.status)]);
        builder.push_record([
            "Connection File",
            &kernel.connection_file.display().to_string(),
        ]);

        if let Some(pid_file) = &kernel.pid_file {
            builder.push_record(["PID File", &pid_file.display().to_string()]);
        }

        if let Some(log_file) = &kernel.log_file {
            builder.push_record(["Log File", &log_file.display().to_string()]);
        }

        if let Some(m) = metrics {
            builder.push_record(["", ""]);
            builder.push_record(["CPU Usage", &format!("{:.1}%", m.cpu_percent)]);
            builder.push_record([
                "Memory Usage",
                &format!(
                    "{} ({:.1}%)",
                    format_memory(m.memory_bytes),
                    m.memory_percent
                ),
            ]);
            builder.push_record(["Open Files", &m.open_files.to_string()]);
            builder.push_record(["Active Connections", &m.active_connections.to_string()]);
            builder.push_record(["Uptime", &format_duration(&m.uptime)]);

            if let Some(last_activity) = m.last_activity {
                let elapsed = std::time::SystemTime::now()
                    .duration_since(last_activity)
                    .unwrap_or(Duration::ZERO);
                builder.push_record([
                    "Last Activity",
                    &format!("{} ago", format_duration(&elapsed)),
                ]);
            }
        }

        let table = builder.build().with(Style::rounded()).to_string();
        println!("{}", table);
    }

    Ok(())
}

/// Display kernels as JSON
fn display_json(kernel_data: &[(&KernelInfo, Option<KernelMetrics>)]) -> Result<()> {
    #[derive(serde::Serialize)]
    struct KernelStatusOutput {
        kernel: KernelInfo,
        metrics: Option<KernelMetrics>,
    }

    let output: Vec<KernelStatusOutput> = kernel_data
        .iter()
        .map(|(k, m)| KernelStatusOutput {
            kernel: (*k).clone(),
            metrics: m.clone(),
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Display kernels in simple format
fn display_simple(kernel_data: &[(&KernelInfo, Option<KernelMetrics>)], quiet: bool) -> Result<()> {
    for (kernel, metrics) in kernel_data {
        if quiet {
            println!("{}", kernel.id);
        } else {
            print!(
                "{}: PID={} Port={} Status={}",
                kernel.id,
                kernel.pid,
                kernel.port,
                format_status(&kernel.status)
            );

            if let Some(m) = metrics {
                print!(
                    " CPU={:.1}% Memory={} Uptime={}",
                    m.cpu_percent,
                    format_memory(m.memory_bytes),
                    format_duration(&m.uptime)
                );
            }

            println!();
        }
    }

    Ok(())
}

/// Format kernel status with color
fn format_status(status: &KernelStatus) -> String {
    match status {
        KernelStatus::Healthy => "Healthy".green().to_string(),
        KernelStatus::Busy => "Busy".yellow().to_string(),
        KernelStatus::Idle => "Idle".blue().to_string(),
        KernelStatus::ShuttingDown => "Shutting Down".red().to_string(),
        KernelStatus::Unknown => "Unknown".dimmed().to_string(),
    }
}

/// Format memory size in human-readable format
fn format_memory(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

/// Format duration in human-readable format
fn format_duration(duration: &Duration) -> String {
    let secs = duration.as_secs();

    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Configuration for installing a kernel service
struct InstallServiceConfig {
    service_type: Option<crate::cli::ServiceType>,
    system: bool,
    name: String,
    port: u16,
    id: Option<String>,
    log_file: Option<PathBuf>,
    pid_file: Option<PathBuf>,
    enable: bool,
    start: bool,
    force: bool,
}

/// Install kernel as system/user service
fn install_kernel_service(config: InstallServiceConfig) -> Result<()> {
    use std::env;
    use std::os::unix::fs::PermissionsExt;

    // Detect platform and service type
    let detected_service = detect_service_type(config.service_type)?;
    let service_info = get_service_info(detected_service, config.system, &config.name)?;

    // Get binary path
    let binary_path =
        env::current_exe().map_err(|e| anyhow!("Failed to get executable path: {}", e))?;

    // Resolve paths with defaults
    let kernel_id = config
        .id
        .unwrap_or_else(|| format!("{}-{}", config.name, config.port));

    let pid_file = config.pid_file.unwrap_or_else(|| {
        if config.system {
            PathBuf::from(format!("/var/run/{}.pid", config.name))
        } else {
            dirs::runtime_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(format!("{}.pid", config.name))
        }
    });

    let log_file = config.log_file.unwrap_or_else(|| {
        if config.system {
            PathBuf::from(format!("/var/log/{}.log", config.name))
        } else {
            dirs::state_dir()
                .unwrap_or_else(|| {
                    dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("/tmp"))
                        .join(".llmspell")
                })
                .join("logs")
                .join(format!("{}.log", config.name))
        }
    });

    // Generate service file content
    let service_content = generate_service_file(
        detected_service,
        &service_info,
        &binary_path,
        config.port,
        &kernel_id,
        &pid_file,
        &log_file,
    )?;

    // Check if service already exists
    let service_path = service_info.install_dir.join(&service_info.service_file);
    if service_path.exists() && !config.force {
        return Err(anyhow!(
            "Service file already exists at: {}\nUse --force to override",
            service_path.display()
        ));
    }

    // Create directory if needed
    if !service_info.install_dir.exists() {
        info!(
            "Creating service directory: {}",
            service_info.install_dir.display()
        );
        fs::create_dir_all(&service_info.install_dir)
            .map_err(|e| anyhow!("Failed to create service directory: {}", e))?;
    }

    // Write service file
    info!("Writing service file: {}", service_path.display());
    fs::write(&service_path, service_content)
        .map_err(|e| anyhow!("Failed to write service file: {}", e))?;

    // Set appropriate permissions
    #[cfg(unix)]
    {
        let metadata = fs::metadata(&service_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o644); // rw-r--r--
        fs::set_permissions(&service_path, permissions)?;
    }

    println!("✓ Service file installed: {}", service_path.display());

    // Print post-installation instructions
    print_post_install_instructions(
        detected_service,
        &config.name,
        config.system,
        config.enable,
        config.start,
    )?;

    // Optionally enable and start service
    if config.enable || config.start {
        manage_service(
            detected_service,
            &config.name,
            config.system,
            config.enable,
            config.start,
        )?;
    }

    Ok(())
}

/// Detect service type based on platform
fn detect_service_type(
    service_type: Option<crate::cli::ServiceType>,
) -> Result<crate::cli::ServiceType> {
    use crate::cli::ServiceType;
    use std::env;

    match service_type {
        Some(ServiceType::Auto) | None => match env::consts::OS {
            "linux" => Ok(ServiceType::Systemd),
            "macos" => Ok(ServiceType::Launchd),
            os => Err(anyhow!("Unsupported platform: {}", os)),
        },
        Some(t) => Ok(t),
    }
}

/// Service installation information
struct ServiceInfo {
    install_dir: PathBuf,
    service_file: String,
    user: String,
    group: String,
}

/// Get service installation information
fn get_service_info(
    service_type: crate::cli::ServiceType,
    system: bool,
    name: &str,
) -> Result<ServiceInfo> {
    use crate::cli::ServiceType;

    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;

    let user = env::var("USER").unwrap_or_else(|_| "nobody".to_string());
    let group = env::var("USER").unwrap_or_else(|_| "nobody".to_string());

    match service_type {
        ServiceType::Systemd => Ok(ServiceInfo {
            install_dir: if system {
                PathBuf::from("/etc/systemd/system")
            } else {
                home.join(".config/systemd/user")
            },
            service_file: format!("{}.service", name),
            user,
            group,
        }),
        ServiceType::Launchd => Ok(ServiceInfo {
            install_dir: if system {
                PathBuf::from("/Library/LaunchDaemons")
            } else {
                home.join("Library/LaunchAgents")
            },
            service_file: format!("com.llmspell.{}.plist", name),
            user,
            group,
        }),
        ServiceType::Auto => unreachable!("Auto should be resolved to specific type"),
    }
}

/// Generate service file content
fn generate_service_file(
    service_type: crate::cli::ServiceType,
    service_info: &ServiceInfo,
    binary_path: &Path,
    port: u16,
    kernel_id: &str,
    pid_file: &Path,
    log_file: &Path,
) -> Result<String> {
    use crate::cli::ServiceType;

    match service_type {
        ServiceType::Systemd => Ok(format!(
            r#"[Unit]
Description=LLMSpell Kernel Service ({})
After=network.target
Documentation=https://github.com/llmspell/llmspell

[Service]
Type=forking
PIDFile={}
ExecStart={} kernel start --daemon --port {} --id {} --pid-file {} --log-file {}
ExecStop={} kernel stop --pid-file {}
ExecReload=/bin/kill -USR1 $MAINPID
Restart=on-failure
RestartSec=5s
User={}
Group={}

# Resource limits
LimitNOFILE=65536

# Security hardening
PrivateTmp=true
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
"#,
            kernel_id,
            pid_file.display(),
            binary_path.display(),
            port,
            kernel_id,
            pid_file.display(),
            log_file.display(),
            binary_path.display(),
            pid_file.display(),
            service_info.user,
            service_info.group,
        )),
        ServiceType::Launchd => Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.llmspell.{}</string>

    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>kernel</string>
        <string>start</string>
        <string>--daemon</string>
        <string>--port</string>
        <string>{}</string>
        <string>--id</string>
        <string>{}</string>
        <string>--pid-file</string>
        <string>{}</string>
        <string>--log-file</string>
        <string>{}</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
        <key>Crashed</key>
        <true/>
    </dict>

    <key>StandardOutPath</key>
    <string>{}</string>

    <key>StandardErrorPath</key>
    <string>{}</string>

    <key>ThrottleInterval</key>
    <integer>5</integer>

    <key>UserName</key>
    <string>{}</string>
</dict>
</plist>"#,
            kernel_id,
            binary_path.display(),
            port,
            kernel_id,
            pid_file.display(),
            log_file.display(),
            log_file.display(),
            log_file.display(),
            service_info.user,
        )),
        _ => unreachable!(),
    }
}

/// Print post-installation instructions
fn print_post_install_instructions(
    service_type: crate::cli::ServiceType,
    name: &str,
    system: bool,
    enable: bool,
    start: bool,
) -> Result<()> {
    use crate::cli::ServiceType;

    println!("\n{}", "═".repeat(60));
    println!("Post-Installation Instructions");
    println!("{}", "─".repeat(60));

    match service_type {
        ServiceType::Systemd => {
            if !enable && !start {
                println!("To enable the service to start at boot:");
                if system {
                    println!("  sudo systemctl enable {}", name);
                } else {
                    println!("  systemctl --user enable {}", name);
                }
                println!();
            }

            if !start {
                println!("To start the service immediately:");
                if system {
                    println!("  sudo systemctl start {}", name);
                } else {
                    println!("  systemctl --user start {}", name);
                }
                println!();
            }

            println!("To check service status:");
            if system {
                println!("  sudo systemctl status {}", name);
            } else {
                println!("  systemctl --user status {}", name);
            }

            println!("\nTo view logs:");
            if system {
                println!("  sudo journalctl -u {} -f", name);
            } else {
                println!("  journalctl --user -u {} -f", name);
            }

            println!("\nTo stop the service:");
            if system {
                println!("  sudo systemctl stop {}", name);
            } else {
                println!("  systemctl --user stop {}", name);
            }

            println!("\nTo uninstall the service:");
            if system {
                println!("  sudo systemctl disable {}", name);
                println!("  sudo rm /etc/systemd/system/{}.service", name);
                println!("  sudo systemctl daemon-reload");
            } else {
                println!("  systemctl --user disable {}", name);
                println!("  rm ~/.config/systemd/user/{}.service", name);
                println!("  systemctl --user daemon-reload");
            }
        }
        ServiceType::Launchd => {
            let service_label = format!("com.llmspell.{}", name);

            if !enable && !start {
                println!("To load (enable) the service:");
                if system {
                    println!(
                        "  sudo launchctl load /Library/LaunchDaemons/{}.plist",
                        service_label
                    );
                } else {
                    println!(
                        "  launchctl load ~/Library/LaunchAgents/{}.plist",
                        service_label
                    );
                }
                println!();
            }

            if !start {
                println!("To start the service immediately:");
                if system {
                    println!("  sudo launchctl start {}", service_label);
                } else {
                    println!("  launchctl start {}", service_label);
                }
                println!();
            }

            println!("To check service status:");
            if system {
                println!("  sudo launchctl list | grep {}", name);
            } else {
                println!("  launchctl list | grep {}", name);
            }

            println!("\nTo view logs:");
            println!("  tail -f /var/log/system.log | grep {}", name);

            println!("\nTo stop the service:");
            if system {
                println!("  sudo launchctl stop {}", service_label);
            } else {
                println!("  launchctl stop {}", service_label);
            }

            println!("\nTo unload (disable) the service:");
            if system {
                println!(
                    "  sudo launchctl unload /Library/LaunchDaemons/{}.plist",
                    service_label
                );
            } else {
                println!(
                    "  launchctl unload ~/Library/LaunchAgents/{}.plist",
                    service_label
                );
            }

            println!("\nTo uninstall the service:");
            if system {
                println!(
                    "  sudo launchctl unload /Library/LaunchDaemons/{}.plist",
                    service_label
                );
                println!("  sudo rm /Library/LaunchDaemons/{}.plist", service_label);
            } else {
                println!(
                    "  launchctl unload ~/Library/LaunchAgents/{}.plist",
                    service_label
                );
                println!("  rm ~/Library/LaunchAgents/{}.plist", service_label);
            }
        }
        _ => {}
    }

    println!("{}", "═".repeat(60));
    Ok(())
}

/// Manage service (enable/start)
fn manage_service(
    service_type: crate::cli::ServiceType,
    name: &str,
    system: bool,
    enable: bool,
    start: bool,
) -> Result<()> {
    use crate::cli::ServiceType;
    use std::process::Command;

    match service_type {
        ServiceType::Systemd => {
            // Reload daemon to pick up new service file
            let reload_output = if system {
                Command::new("sudo")
                    .args(["systemctl", "daemon-reload"])
                    .output()?
            } else {
                Command::new("systemctl")
                    .args(["--user", "daemon-reload"])
                    .output()?
            };

            if !reload_output.status.success() {
                warn!(
                    "Failed to reload systemd daemon: {}",
                    String::from_utf8_lossy(&reload_output.stderr)
                );
            }

            if enable {
                println!("Enabling service...");
                let output = if system {
                    Command::new("sudo")
                        .args(["systemctl", "enable", name])
                        .output()?
                } else {
                    Command::new("systemctl")
                        .args(["--user", "enable", name])
                        .output()?
                };

                if output.status.success() {
                    println!("✓ Service enabled");
                } else {
                    warn!(
                        "Failed to enable service: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }

            if start {
                println!("Starting service...");
                let output = if system {
                    Command::new("sudo")
                        .args(["systemctl", "start", name])
                        .output()?
                } else {
                    Command::new("systemctl")
                        .args(["--user", "start", name])
                        .output()?
                };

                if output.status.success() {
                    println!("✓ Service started");
                } else {
                    return Err(anyhow!(
                        "Failed to start service: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }
        ServiceType::Launchd => {
            let service_label = format!("com.llmspell.{}", name);
            let plist_path = if system {
                format!("/Library/LaunchDaemons/{}.plist", service_label)
            } else {
                dirs::home_dir()
                    .ok_or_else(|| anyhow!("Could not determine home directory"))?
                    .join("Library/LaunchAgents")
                    .join(format!("{}.plist", service_label))
                    .to_string_lossy()
                    .to_string()
            };

            if enable || start {
                println!("Loading service...");
                let output = if system {
                    Command::new("sudo")
                        .args(["launchctl", "load", &plist_path])
                        .output()?
                } else {
                    Command::new("launchctl")
                        .args(["load", &plist_path])
                        .output()?
                };

                if output.status.success() {
                    println!("✓ Service loaded");
                } else {
                    // It might already be loaded
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.contains("already loaded") {
                        warn!("Failed to load service: {}", stderr);
                    }
                }
            }

            if start {
                println!("Starting service...");
                let output = if system {
                    Command::new("sudo")
                        .args(["launchctl", "start", &service_label])
                        .output()?
                } else {
                    Command::new("launchctl")
                        .args(["start", &service_label])
                        .output()?
                };

                if output.status.success() {
                    println!("✓ Service started");
                } else {
                    // Check if it's already running
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.contains("already running") {
                        return Err(anyhow!("Failed to start service: {}", stderr));
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}
