//! Network REPL Server Implementation
//!
//! Provides TCP-based network access to the `InteractiveSession` REPL functionality.
//! Supports multiple concurrent sessions with protocol negotiation.

use crate::execution::IntegratedKernel;
use crate::protocols::jupyter::JupyterProtocol;
use crate::repl::{MetaCommand, ReplCommand};
use anyhow::{Context, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// REPL server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct REPLConfig {
    /// Server bind address
    pub bind_address: String,
    /// Server port
    pub port: u16,
    /// Maximum concurrent sessions
    pub max_sessions: usize,
    /// Session idle timeout (seconds)
    pub session_timeout_secs: u64,
    /// Enable authentication
    pub auth_enabled: bool,
    /// Default protocol mode
    pub default_protocol: REPLProtocol,
}

impl Default for REPLConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 9999,
            max_sessions: 100,
            session_timeout_secs: 3600,
            auth_enabled: false,
            default_protocol: REPLProtocol::Text,
        }
    }
}

/// Wire protocol modes for REPL communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum REPLProtocol {
    /// Simple text-based protocol (default)
    Text,
    /// Structured JSON-RPC 2.0 protocol
    JsonRpc,
    /// Efficient binary protocol (`MessagePack`)
    Binary,
}

/// Network session wrapper
struct NetworkSession {
    /// Unique session ID
    id: String,
    /// Current protocol mode
    protocol: Arc<RwLock<REPLProtocol>>,
    /// Session creation time
    created_at: std::time::Instant,
    /// Last activity time
    last_activity: Arc<RwLock<std::time::Instant>>,
    /// Flag to track if session is active
    active: Arc<AtomicBool>,
}

impl NetworkSession {
    /// Create a new network session
    fn new(protocol: REPLProtocol) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            protocol: Arc::new(RwLock::new(protocol)),
            created_at: std::time::Instant::now(),
            last_activity: Arc::new(RwLock::new(std::time::Instant::now())),
            active: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Update last activity timestamp
    async fn touch(&self) {
        *self.last_activity.write().await = std::time::Instant::now();
    }

    /// Check if session has timed out
    async fn is_expired(&self, timeout_secs: u64) -> bool {
        // Check if session is inactive
        if !self.active.load(std::sync::atomic::Ordering::Relaxed) {
            return true;
        }

        let last = *self.last_activity.read().await;
        last.elapsed().as_secs() > timeout_secs
    }

    /// Get session age in seconds
    fn age_secs(&self) -> u64 {
        self.created_at.elapsed().as_secs()
    }

    /// Mark session as inactive
    fn deactivate(&self) {
        self.active
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Network REPL Server
pub struct REPLServer {
    /// Server configuration
    config: REPLConfig,
    /// Active sessions
    sessions: Arc<DashMap<String, Arc<NetworkSession>>>,
    /// Kernel instance for creating new sessions
    kernel: Arc<IntegratedKernel<JupyterProtocol>>,
    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl REPLServer {
    /// Create a new REPL server
    pub fn new(config: REPLConfig, kernel: Arc<IntegratedKernel<JupyterProtocol>>) -> Self {
        Self {
            config,
            sessions: Arc::new(DashMap::new()),
            kernel,
            shutdown_tx: None,
        }
    }

    /// Start the REPL server
    ///
    /// # Errors
    ///
    /// Returns an error if binding to the configured address fails
    pub async fn start(&mut self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.bind_address, self.config.port)
            .parse()
            .context("Invalid bind address")?;

        let listener = TcpListener::bind(&addr)
            .await
            .with_context(|| format!("Failed to bind to {addr}"))?;

        info!("REPL server listening on {}", addr);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start session cleanup task
        let sessions = Arc::clone(&self.sessions);
        let timeout_secs = self.config.session_timeout_secs;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                Self::cleanup_expired_sessions(&sessions, timeout_secs).await;
            }
        });

        // Accept connections
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            if self.sessions.len() >= self.config.max_sessions {
                                warn!("Maximum sessions reached, rejecting connection from {}", addr);
                                continue;
                            }

                            let kernel = Arc::clone(&self.kernel);
                            let sessions = Arc::clone(&self.sessions);
                            let config = self.config.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(stream, addr, kernel, sessions, config).await {
                                    error!("Connection error from {}: {}", addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Accept error: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("REPL server shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a client connection
    async fn handle_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        kernel: Arc<IntegratedKernel<JupyterProtocol>>,
        sessions: Arc<DashMap<String, Arc<NetworkSession>>>,
        config: REPLConfig,
    ) -> Result<()> {
        info!("New connection from {}", addr);

        // Send welcome message and protocol negotiation
        let welcome = format!(
            "LLMSpell REPL Server v{}\r\n\
             Protocols: text, jsonrpc, binary\r\n\
             Type 'PROTOCOL <mode>' to switch protocols\r\n\
             Type '.help' for commands\r\n\
             Session ID: ",
            env!("CARGO_PKG_VERSION")
        );
        stream.write_all(welcome.as_bytes()).await?;

        // Create new session
        let network_session = Arc::new(NetworkSession::new(config.default_protocol));
        let session_id = network_session.id.clone();

        // Send session ID
        stream
            .write_all(format!("{session_id}\r\n> ").as_bytes())
            .await?;
        stream.flush().await?;

        // Store session
        sessions.insert(session_id.clone(), Arc::clone(&network_session));

        // Handle session I/O
        let (reader, writer) = stream.into_split();
        let reader = BufReader::new(reader);
        Self::handle_session(network_session, kernel, reader, writer).await?;

        // Mark session as inactive and clean up
        if let Some(entry) = sessions.get(&session_id) {
            entry.deactivate();
            info!(
                "Session {} disconnected after {} seconds",
                session_id,
                entry.age_secs()
            );
        }
        sessions.remove(&session_id);

        Ok(())
    }

    /// Handle session I/O
    async fn handle_session(
        session: Arc<NetworkSession>,
        kernel: Arc<IntegratedKernel<JupyterProtocol>>,
        mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
        mut writer: tokio::net::tcp::OwnedWriteHalf,
    ) -> Result<()> {
        let mut line = String::new();

        loop {
            line.clear();

            // Read input with timeout
            let read_result = tokio::time::timeout(
                std::time::Duration::from_secs(600),
                reader.read_line(&mut line),
            )
            .await;

            match read_result {
                Ok(Ok(0)) => {
                    // EOF - client disconnected
                    break;
                }
                Ok(Ok(_)) => {
                    session.touch().await;
                    let input = line.trim();

                    // Handle protocol switching
                    if input.starts_with("PROTOCOL ") {
                        let new_protocol = input.trim_start_matches("PROTOCOL ").trim();
                        if let Some(protocol) = Self::parse_protocol(new_protocol) {
                            *session.protocol.write().await = protocol;
                            writer.write_all(b"Protocol switched\r\n> ").await?;
                            writer.flush().await?;
                            continue;
                        }
                        writer.write_all(b"Unknown protocol\r\n> ").await?;
                        writer.flush().await?;
                        continue;
                    }

                    // Process command based on current protocol
                    let current_protocol = *session.protocol.read().await;
                    let response = match current_protocol {
                        REPLProtocol::Text => Self::handle_text_command(&kernel, input).await?,
                        REPLProtocol::JsonRpc => {
                            Self::handle_jsonrpc_command(&kernel, input).await?
                        }
                        REPLProtocol::Binary => Self::handle_binary_command(&kernel, input),
                    };

                    // Send response
                    writer.write_all(response.as_bytes()).await?;
                    writer.write_all(b"\r\n> ").await?;
                    writer.flush().await?;

                    // Check for exit command
                    if let Ok(ReplCommand::Meta(MetaCommand::Exit)) = ReplCommand::parse(input) {
                        break;
                    }
                }
                Ok(Err(e)) => {
                    error!("Read error: {}", e);
                    break;
                }
                Err(_) => {
                    // Timeout - send keepalive
                    writer.write_all(b"\r\n").await?;
                    writer.flush().await?;
                }
            }
        }

        Ok(())
    }

    /// Handle text protocol command
    async fn handle_text_command(
        kernel: &Arc<IntegratedKernel<JupyterProtocol>>,
        input: &str,
    ) -> Result<String> {
        // Parse the command
        match ReplCommand::parse(input) {
            Ok(ReplCommand::Execute(code)) => {
                // Execute code through kernel's script executor
                let executor = kernel.get_script_executor();
                match executor.execute_script(&code).await {
                    Ok(result) => {
                        // Convert the output value to string
                        let output_str = format!("{}", result.output);
                        // Include console output if any
                        if result.console_output.is_empty() {
                            Ok(output_str)
                        } else {
                            Ok(format!(
                                "{}
                        {}",
                                result.console_output.join(
                                    "
                        "
                                ),
                                output_str
                            ))
                        }
                    }
                    Err(e) => Ok(format!("Error: {e}")),
                }
            }
            Ok(ReplCommand::Meta(MetaCommand::Help)) => Ok("Available commands:\n\
                   .help    - Show this help\n\
                   .exit    - Exit the session\n\
                   .clear   - Clear screen\n\
                   .history - Show command history"
                .to_string()),
            Ok(ReplCommand::Meta(MetaCommand::Exit)) => Ok("Goodbye!".to_string()),
            Ok(ReplCommand::Meta(_)) => Ok("Command acknowledged".to_string()),
            Ok(ReplCommand::Debug(_)) => {
                Ok("Debug commands not supported in network mode".to_string())
            }
            Ok(ReplCommand::Empty) => Ok(String::new()),
            Err(e) => Ok(format!("Parse error: {e}")),
        }
    }

    /// Handle JSON-RPC protocol command
    #[allow(clippy::too_many_lines)]
    async fn handle_jsonrpc_command(
        kernel: &Arc<IntegratedKernel<JupyterProtocol>>,
        input: &str,
    ) -> Result<String> {
        #[derive(Deserialize)]
        struct JsonRpcRequest {
            jsonrpc: String,
            method: String,
            params: Option<serde_json::Value>,
            id: Option<serde_json::Value>,
        }

        #[derive(Serialize)]
        struct JsonRpcResponse {
            jsonrpc: String,
            result: Option<serde_json::Value>,
            error: Option<JsonRpcError>,
            id: Option<serde_json::Value>,
        }

        #[derive(Serialize)]
        struct JsonRpcError {
            code: i32,
            message: String,
            data: Option<serde_json::Value>,
        }

        // Parse JSON-RPC request
        let request = match serde_json::from_str::<JsonRpcRequest>(input) {
            Ok(r) => {
                // Validate JSON-RPC version
                if r.jsonrpc != "2.0" {
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32600,
                            message: format!(
                                "Invalid Request: JSON-RPC version must be 2.0, got {}",
                                r.jsonrpc
                            ),
                            data: None,
                        }),
                        id: None,
                    };
                    return Ok(serde_json::to_string(&error_response)?);
                }
                r
            }
            Err(e) => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {e}"),
                        data: None,
                    }),
                    id: None,
                };
                return Ok(serde_json::to_string(&error_response)?);
            }
        };

        // Handle method
        let result = if request.method.as_str() == "execute" {
            if let Some(serde_json::Value::String(code)) = request.params {
                let executor = kernel.get_script_executor();
                match executor.execute_script(&code).await {
                    Ok(result) => {
                        let output_str = format!("{}", result.output);
                        let full_output = if result.console_output.is_empty() {
                            output_str
                        } else {
                            format!(
                                "{}
                        {}",
                                result.console_output.join(
                                    "
                        "
                                ),
                                output_str
                            )
                        };
                        serde_json::json!({"output": full_output})
                    }
                    Err(e) => {
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32603,
                                message: format!("Execution error: {e}"),
                                data: None,
                            }),
                            id: request.id,
                        };
                        return Ok(serde_json::to_string(&error_response)?);
                    }
                }
            } else {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: None,
                    }),
                    id: request.id,
                };
                return Ok(serde_json::to_string(&error_response)?);
            }
        } else {
            let error_response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
                id: request.id,
            };
            return Ok(serde_json::to_string(&error_response)?);
        };

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        };

        Ok(serde_json::to_string(&response)?)
    }

    /// Handle binary protocol command (`MessagePack`)
    fn handle_binary_command(
        _kernel: &Arc<IntegratedKernel<JupyterProtocol>>,
        _input: &str,
    ) -> String {
        // For now, return a placeholder
        // Full implementation would use rmp_serde for MessagePack encoding/decoding
        "Binary protocol not yet implemented".to_string()
    }

    /// Parse protocol string
    fn parse_protocol(s: &str) -> Option<REPLProtocol> {
        match s.to_lowercase().as_str() {
            "text" => Some(REPLProtocol::Text),
            "jsonrpc" | "json-rpc" | "json" => Some(REPLProtocol::JsonRpc),
            "binary" | "msgpack" | "messagepack" => Some(REPLProtocol::Binary),
            _ => None,
        }
    }

    /// Clean up expired sessions
    async fn cleanup_expired_sessions(
        sessions: &Arc<DashMap<String, Arc<NetworkSession>>>,
        timeout_secs: u64,
    ) {
        let mut expired = Vec::new();

        for entry in sessions.iter() {
            if entry.value().is_expired(timeout_secs).await {
                expired.push(entry.key().clone());
            }
        }

        for id in expired {
            sessions.remove(&id);
            debug!("Cleaned up expired session: {}", id);
        }
    }

    /// Shutdown the server
    ///
    /// # Errors
    ///
    /// Returns an error if sending the shutdown signal fails
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // Clear all sessions
        self.sessions.clear();

        info!("REPL server shutdown complete");
        Ok(())
    }

    /// Get server statistics
    pub fn get_stats(&self) -> ServerStats {
        ServerStats {
            active_sessions: self.sessions.len(),
            max_sessions: self.config.max_sessions,
            bind_address: self.config.bind_address.clone(),
            port: self.config.port,
        }
    }
}

/// Server statistics
#[derive(Debug, Serialize)]
pub struct ServerStats {
    /// Number of active sessions
    pub active_sessions: usize,
    /// Maximum allowed sessions
    pub max_sessions: usize,
    /// Server bind address
    pub bind_address: String,
    /// Server port
    pub port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{ExecutionConfig, IntegratedKernel};
    use crate::protocols::jupyter::JupyterProtocol;
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_repl_server_creation() {
        let config = REPLConfig::default();
        let script_executor = Arc::new(
            ScriptRuntime::new_with_lua(LLMSpellConfig::default())
                .await
                .unwrap(),
        );
        let protocol = JupyterProtocol::new("test-session".to_string(), "test-kernel".to_string());
        let exec_config = ExecutionConfig::default();
        let kernel = Arc::new(
            IntegratedKernel::new(
                protocol,
                exec_config,
                "test-session".to_string(),
                script_executor,
            )
            .await
            .unwrap(),
        );

        let server = REPLServer::new(config, kernel);
        let stats = server.get_stats();

        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.max_sessions, 100);
        assert_eq!(stats.port, 9999);
    }

    #[test]
    fn test_protocol_parsing() {
        assert_eq!(REPLServer::parse_protocol("text"), Some(REPLProtocol::Text));
        assert_eq!(
            REPLServer::parse_protocol("jsonrpc"),
            Some(REPLProtocol::JsonRpc)
        );
        assert_eq!(
            REPLServer::parse_protocol("binary"),
            Some(REPLProtocol::Binary)
        );
        assert_eq!(REPLServer::parse_protocol("invalid"), None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_session_expiration() {
        let session = NetworkSession::new(REPLProtocol::Text);

        // Session should not be expired initially
        assert!(!session.is_expired(60).await);

        // Touch and verify not expired
        session.touch().await;
        assert!(!session.is_expired(60).await);
    }
}
