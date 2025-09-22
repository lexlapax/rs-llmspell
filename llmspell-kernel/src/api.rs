//! High-level kernel API for embedded and client modes
//!
//! This module provides the public API for working with kernels,
//! abstracting away the complexity of protocols and transports.

use crate::execution::integrated::IntegratedKernel;
use crate::protocols::jupyter::JupyterProtocol;
use crate::traits::protocol::Protocol;
use crate::traits::{ChannelConfig, Transport, TransportConfig};
use crate::transport::inprocess::InProcessTransport;
use anyhow::Result;
use async_trait::async_trait;
use llmspell_config::LLMSpellConfig;
use llmspell_core::traits::script_executor::{
    ScriptExecutionMetadata, ScriptExecutionOutput, ScriptExecutor,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Handle for an embedded kernel running in-process
pub struct KernelHandle {
    kernel: IntegratedKernel<JupyterProtocol>,
    kernel_id: String,
    transport: Arc<InProcessTransport>,
    protocol: JupyterProtocol,
}

impl KernelHandle {
    /// Run the kernel until shutdown
    ///
    /// # Errors
    ///
    /// Returns an error if the kernel fails to run
    pub async fn run(self) -> Result<()> {
        info!("Running embedded kernel {}", self.kernel_id);
        self.kernel.run().await
    }

    /// Execute code and return result
    ///
    /// # Errors
    ///
    /// Returns an error if the execution fails or communication with kernel fails
    pub async fn execute(&mut self, code: &str) -> Result<String> {
        debug!("Executing code in kernel {}", self.kernel_id);

        // Create execute_request message
        let content = serde_json::json!({
            "code": code,
            "silent": false,
            "store_history": true,
            "user_expressions": {},
            "allow_stdin": false,
        });

        let request = self.protocol.create_request("execute_request", content)?;

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for execute_reply
        loop {
            if let Some(reply_parts) = self.transport.recv("shell").await? {
                if let Some(first_part) = reply_parts.first() {
                    // Parse reply and extract result
                    let reply_msg = self.protocol.parse_message(first_part)?;
                    if let Some(content) = reply_msg.get("content") {
                        // Extract execution result
                        return Ok(format!("Result: {content:?}"));
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Get the kernel ID
    pub fn kernel_id(&self) -> &str {
        &self.kernel_id
    }

    /// Get the transport for client connections
    pub fn transport(&self) -> Arc<InProcessTransport> {
        self.transport.clone()
    }

    /// Convert handle into the underlying kernel
    pub fn into_kernel(self) -> IntegratedKernel<JupyterProtocol> {
        self.kernel
    }
}

/// Handle for a client connection to a kernel
pub struct ClientHandle {
    protocol: JupyterProtocol,
    connection_string: String,
    transport: Box<dyn Transport>,
}

impl ClientHandle {
    /// Execute code on the remote kernel
    ///
    /// # Errors
    ///
    /// Returns an error if the execution fails or communication with kernel fails
    pub async fn execute(&mut self, code: &str) -> Result<String> {
        debug!("Sending execute request to kernel");

        // Create execute_request message
        let content = serde_json::json!({
            "code": code,
            "silent": false,
            "store_history": true,
            "user_expressions": {},
            "allow_stdin": false,
        });

        let request = self.protocol.create_request("execute_request", content)?;

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for execute_reply
        loop {
            if let Some(reply_parts) = self.transport.recv("shell").await? {
                if let Some(first_part) = reply_parts.first() {
                    // Parse reply and extract result
                    let reply_msg = self.protocol.parse_message(first_part)?;
                    if let Some(content) = reply_msg.get("content") {
                        // Extract execution result
                        return Ok(format!("Result: {content:?}"));
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Run interactive REPL connected to the kernel
    ///
    /// # Errors
    ///
    /// Returns an error if the REPL fails to start or connect
    pub fn run_repl(&mut self) -> Result<()> {
        info!("Starting REPL connected to {}", self.connection_string);
        // This would start an interactive REPL session
        // using the transport to communicate with the kernel
        Ok(())
    }
}

/// Handle for a kernel running in service mode
pub struct ServiceHandle {
    kernel: IntegratedKernel<JupyterProtocol>,
    port: u16,
    connection_file: PathBuf,
}

impl ServiceHandle {
    /// Run the kernel service until shutdown
    ///
    /// # Errors
    ///
    /// Returns an error if the kernel service fails to run
    pub async fn run(self) -> Result<()> {
        info!("Running kernel service on port {}", self.port);
        info!("Connection file: {:?}", self.connection_file);
        self.kernel.run().await
    }

    /// Get the connection file path
    pub fn connection_file(&self) -> &Path {
        &self.connection_file
    }
}

/// Start an embedded kernel with a custom script executor
///
/// This is used when the caller wants to provide a specific script executor
/// implementation, such as a real `ScriptRuntime` from llmspell-bridge.
///
/// # Errors
///
/// Returns an error if the kernel fails to start or transport setup fails
pub async fn start_embedded_kernel_with_executor(
    config: LLMSpellConfig,
    script_executor: Arc<dyn ScriptExecutor>,
) -> Result<KernelHandle> {
    let kernel_id = format!("embedded-{}", Uuid::new_v4());
    let session_id = format!("session-{}", Uuid::new_v4());

    info!("Starting embedded kernel {}", kernel_id);

    // Create Jupyter protocol
    let protocol = JupyterProtocol::new(session_id.clone(), kernel_id.clone());

    // Create in-process transport
    let transport = Arc::new(InProcessTransport::new());

    // Setup Jupyter 5-channel configuration
    let mut transport_config = TransportConfig {
        transport_type: "inprocess".to_string(),
        base_address: String::new(),
        channels: HashMap::new(),
        auth_key: None,
    };

    // Add all 5 Jupyter channels
    for channel in &["shell", "iopub", "stdin", "control", "heartbeat"] {
        transport_config.channels.insert(
            (*channel).to_string(),
            ChannelConfig {
                endpoint: String::new(),
                pattern: String::new(),
                options: HashMap::new(),
            },
        );
    }

    // Bind transport to channels
    let mut transport_mut = (*transport).clone();
    transport_mut.bind(&transport_config).await?;

    // Build execution config from LLMSpellConfig
    let exec_config = build_execution_config(&config);

    // Use the provided script executor
    // Create integrated kernel with the provided executor
    let mut kernel =
        IntegratedKernel::new(protocol.clone(), exec_config, session_id, script_executor).await?;

    // Set transport for message processing
    kernel.set_transport(Box::new(transport_mut));

    Ok(KernelHandle {
        kernel,
        kernel_id,
        transport,
        protocol,
    })
}

/// Start an embedded kernel that runs in-process
///
/// This is used when the CLI runs without --connect flag.
/// The kernel runs in the same process as the CLI.
///
/// # Errors
///
/// Returns an error if the kernel fails to start or transport setup fails
pub async fn start_embedded_kernel(config: LLMSpellConfig) -> Result<KernelHandle> {
    // Create a default executor for backward compatibility
    struct DefaultExecutor;

    #[async_trait]
    impl ScriptExecutor for DefaultExecutor {
        async fn execute_script(
            &self,
            _script: &str,
        ) -> Result<ScriptExecutionOutput, llmspell_core::error::LLMSpellError> {
            Ok(ScriptExecutionOutput {
                output: serde_json::json!(
                    "Default executor - use start_embedded_kernel_with_executor for real execution"
                ),
                console_output: vec![],
                metadata: ScriptExecutionMetadata {
                    duration: std::time::Duration::from_millis(0),
                    language: "lua".to_string(),
                    exit_code: Some(0),
                    warnings: vec![],
                },
            })
        }

        fn language(&self) -> &'static str {
            "lua"
        }
    }

    let script_executor = Arc::new(DefaultExecutor) as Arc<dyn ScriptExecutor>;
    start_embedded_kernel_with_executor(config, script_executor).await
}

/// Connect to an existing kernel service as a client
///
/// This is used when the CLI runs with --connect flag.
/// The CLI acts as a Jupyter client connecting to a remote kernel.
///
/// # Errors
///
/// Returns an error if connection fails or the connection string is invalid
pub async fn connect_to_kernel(connection_string: &str) -> Result<ClientHandle> {
    info!("Connecting to kernel at: {}", connection_string);

    // Create client protocol
    let protocol = JupyterProtocol::new_client();

    // Parse connection string to determine transport type
    let mut transport: Box<dyn Transport>;
    let mut transport_config = TransportConfig {
        transport_type: String::new(),
        base_address: String::new(),
        channels: HashMap::new(),
        auth_key: None,
    };

    if connection_string.starts_with("tcp://") {
        // TCP connection: tcp://host:port
        let addr = connection_string.trim_start_matches("tcp://");
        let parts: Vec<&str> = addr.split(':').collect();

        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid TCP connection string: {}",
                connection_string
            ));
        }

        transport_config.transport_type = "zeromq".to_string();
        transport_config.base_address = parts[0].to_string();

        let base_port: u16 = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid port: {}", parts[1]))?;

        // Setup 5 Jupyter channels with sequential ports
        transport_config.channels.insert(
            "shell".to_string(),
            ChannelConfig {
                endpoint: base_port.to_string(),
                pattern: "dealer".to_string(), // Client uses dealer
                options: HashMap::new(),
            },
        );
        transport_config.channels.insert(
            "iopub".to_string(),
            ChannelConfig {
                endpoint: (base_port + 1).to_string(),
                pattern: "sub".to_string(), // Client subscribes
                options: HashMap::new(),
            },
        );
        transport_config.channels.insert(
            "stdin".to_string(),
            ChannelConfig {
                endpoint: (base_port + 2).to_string(),
                pattern: "dealer".to_string(),
                options: HashMap::new(),
            },
        );
        transport_config.channels.insert(
            "control".to_string(),
            ChannelConfig {
                endpoint: (base_port + 3).to_string(),
                pattern: "dealer".to_string(),
                options: HashMap::new(),
            },
        );
        transport_config.channels.insert(
            "heartbeat".to_string(),
            ChannelConfig {
                endpoint: (base_port + 4).to_string(),
                pattern: "req".to_string(), // Client requests heartbeat
                options: HashMap::new(),
            },
        );

        transport = crate::traits::create_transport("zeromq")?;
    } else if std::path::Path::new(connection_string)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        // Connection file
        return Err(anyhow::anyhow!(
            "Connection file support not yet implemented. Use tcp:// format"
        ));
    } else {
        // Named kernel (e.g., "my-kernel")
        // This would look up the kernel in a registry
        return Err(anyhow::anyhow!(
            "Named kernel connections not yet implemented. Use tcp:// format"
        ));
    }

    // Connect the transport
    transport.connect(&transport_config).await?;

    Ok(ClientHandle {
        protocol,
        connection_string: connection_string.to_string(),
        transport,
    })
}

/// Start a kernel in service mode that listens for connections
///
/// This is used when starting a kernel as a service that other clients can connect to.
///
/// # Errors
///
/// Returns an error if the kernel service fails to start or bind to the port
pub async fn start_kernel_service(port: u16, config: LLMSpellConfig) -> Result<ServiceHandle> {
    // TODO: In subtask 9.4.6.4, this will be replaced with real ScriptRuntime from llmspell-bridge
    // For now, create a stub executor that will be replaced (same as in start_embedded_kernel)
    struct ServiceStubExecutor;

    #[async_trait]
    impl ScriptExecutor for ServiceStubExecutor {
        async fn execute_script(
            &self,
            _script: &str,
        ) -> Result<ScriptExecutionOutput, llmspell_core::error::LLMSpellError> {
            Ok(ScriptExecutionOutput {
                output: serde_json::json!("Service stub executor - will be replaced in 9.4.6.4"),
                console_output: vec![],
                metadata: ScriptExecutionMetadata {
                    duration: std::time::Duration::from_millis(0),
                    language: "stub".to_string(),
                    exit_code: Some(0),
                    warnings: vec![],
                },
            })
        }

        fn language(&self) -> &'static str {
            "stub"
        }
    }

    let kernel_id = format!("service-{}", Uuid::new_v4());
    let session_id = format!("session-{}", Uuid::new_v4());

    info!("Starting kernel service {} on port {}", kernel_id, port);

    // Create Jupyter protocol
    let protocol = JupyterProtocol::new(session_id.clone(), kernel_id.clone());

    // Build execution config
    let exec_config = build_execution_config(&config);

    let script_executor = Arc::new(ServiceStubExecutor) as Arc<dyn ScriptExecutor>;

    // Create integrated kernel
    let kernel = IntegratedKernel::new(protocol, exec_config, session_id, script_executor).await?;
    // Note: Service kernels don't need transport set here as they use external connections

    // Write connection file for clients
    let connection_file = write_connection_file(port, &kernel_id)?;

    Ok(ServiceHandle {
        kernel,
        port,
        connection_file,
    })
}

/// Build `ExecutionConfig` from `LLMSpellConfig`
fn build_execution_config(config: &LLMSpellConfig) -> crate::execution::ExecutionConfig {
    crate::execution::ExecutionConfig {
        runtime_config: serde_json::to_value(config)
            .ok()
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default()
            .into_iter()
            .collect(),
        io_config: crate::execution::IOConfig::default(),
        max_history: 1000,
        execution_timeout_secs: 300,
        monitor_agents: true,
        track_performance: true,
        daemon_mode: false,
        daemon_config: None,
    }
}

/// Write connection file for kernel service
fn write_connection_file(port: u16, kernel_id: &str) -> Result<PathBuf> {
    let connection_dir = dirs::runtime_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    let connection_file = connection_dir.join(format!("kernel-{kernel_id}.json"));

    let connection_info = serde_json::json!({
        "shell_port": port,
        "iopub_port": port + 1,
        "stdin_port": port + 2,
        "control_port": port + 3,
        "hb_port": port + 4,
        "ip": "127.0.0.1",
        "transport": "tcp",
        "signature_scheme": "hmac-sha256",
        "kernel_name": "llmspell",
    });

    std::fs::write(
        &connection_file,
        serde_json::to_string_pretty(&connection_info)?,
    )?;

    Ok(connection_file)
}
