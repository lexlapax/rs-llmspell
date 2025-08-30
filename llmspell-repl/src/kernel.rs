//! Core kernel service implementation
//!
//! The LLMSpellKernel wraps the ScriptRuntime from llmspell-bridge and provides
//! multi-client debugging and REPL capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::Result;

use crate::channels::KernelChannels;
use crate::client::ConnectedClient;

/// Kernel execution state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelState {
    /// Kernel is idle and ready for commands
    Idle,
    /// Kernel is executing code
    Busy,
    /// Kernel is starting up
    Starting,
    /// Kernel is shutting down
    Stopping,
}

/// Configuration for kernel startup
#[derive(Debug, Clone)]
pub struct KernelConfig {
    /// Unique kernel identifier
    pub kernel_id: Option<String>,
    /// IP address to bind to
    pub ip: String,
    /// Port range start for allocating channels
    pub port_range_start: u16,
    /// Enable debug mode
    pub debug_enabled: bool,
    /// Maximum number of clients
    pub max_clients: usize,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            kernel_id: None,
            ip: "127.0.0.1".to_string(),
            port_range_start: 5555,
            debug_enabled: false,
            max_clients: 10,
        }
    }
}

/// Main kernel service that manages script execution and debugging
pub struct LLMSpellKernel {
    /// Unique kernel identifier
    pub kernel_id: String,
    
    // Script runtime will be integrated from llmspell-bridge
    // For now, using placeholder to allow compilation
    // pub runtime: Arc<llmspell_bridge::ScriptRuntime>,
    
    /// Connected clients
    pub clients: Arc<RwLock<HashMap<String, ConnectedClient>>>,
    
    /// Communication channels
    pub channels: KernelChannels,
    
    /// Current execution state
    pub execution_state: Arc<RwLock<KernelState>>,
    
    /// Kernel configuration
    pub config: KernelConfig,
    
    // Debug components will be added in Phase 9.2
    // pub debugger: Arc<Debugger>,
    // pub profiler: Arc<PerformanceProfiler>,
    // pub tracer: Arc<DistributedTracer>,
}

impl LLMSpellKernel {
    /// Start a new kernel with the given configuration
    pub async fn start(mut config: KernelConfig) -> Result<Self> {
        // Generate kernel ID if not provided
        if config.kernel_id.is_none() {
            config.kernel_id = Some(Uuid::new_v4().to_string());
        }
        let kernel_id = config.kernel_id.clone().unwrap();
        
        tracing::info!("Starting LLMSpell kernel {}", kernel_id);
        
        // Script runtime will be created from llmspell-bridge
        // let runtime = Arc::new(llmspell_bridge::ScriptRuntime::new().await?);
        
        // Initialize channels
        let channels = KernelChannels::new(&config.ip, config.port_range_start).await?;
        
        // Create kernel instance
        let kernel = Self {
            kernel_id: kernel_id.clone(),
            // runtime will be added when integrating with llmspell-bridge
            clients: Arc::new(RwLock::new(HashMap::new())),
            channels,
            execution_state: Arc::new(RwLock::new(KernelState::Starting)),
            config,
        };
        
        // Set state to idle
        *kernel.execution_state.write().await = KernelState::Idle;
        
        tracing::info!("Kernel {} started successfully", kernel_id);
        Ok(kernel)
    }
    
    /// Run the kernel event loop
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Kernel {} entering main event loop", self.kernel_id);
        
        // Start channel listeners
        self.channels.start_listeners().await?;
        
        // Main event loop will be implemented to handle:
        // - Shell channel requests (execute, complete, inspect, etc.)
        // - Control channel commands (interrupt, shutdown, etc.)
        // - Stdin channel input requests
        // - Heartbeat monitoring
        // - Broadcasting to IOPub channel
        
        // For now, just wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        
        Ok(())
    }
    
    /// Shutdown the kernel gracefully
    pub async fn shutdown(self) -> Result<()> {
        tracing::info!("Shutting down kernel {}", self.kernel_id);
        
        // Set state to stopping
        *self.execution_state.write().await = KernelState::Stopping;
        
        // Disconnect all clients
        let clients = self.clients.write().await;
        for (client_id, _client) in clients.iter() {
            tracing::info!("Disconnecting client {}", client_id);
        }
        drop(clients);
        
        // Stop channels
        self.channels.stop().await?;
        
        // Clean up resources
        // TODO: Remove connection file
        
        tracing::info!("Kernel {} shutdown complete", self.kernel_id);
        Ok(())
    }
    
    /// Add a new client connection
    pub async fn add_client(&self, client: ConnectedClient) -> Result<()> {
        let mut clients = self.clients.write().await;
        
        if clients.len() >= self.config.max_clients {
            anyhow::bail!("Maximum number of clients ({}) reached", self.config.max_clients);
        }
        
        let client_id = client.client_id.clone();
        clients.insert(client_id.clone(), client);
        
        tracing::info!("Client {} connected to kernel {}", client_id, self.kernel_id);
        Ok(())
    }
    
    /// Remove a client connection
    pub async fn remove_client(&self, client_id: &str) -> Result<()> {
        let mut clients = self.clients.write().await;
        
        if clients.remove(client_id).is_some() {
            tracing::info!("Client {} disconnected from kernel {}", client_id, self.kernel_id);
        }
        
        Ok(())
    }
}