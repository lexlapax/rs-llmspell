//! ABOUTME: llmspell-bridge - Language-agnostic script runtime with bridge pattern
//! ABOUTME: Supports multiple script engines (Lua, JavaScript, Python) through ScriptEngineBridge
//! 
//! # LLMSpell Bridge
//! 
//! The bridge crate provides a language-agnostic runtime for executing scripts that
//! interact with LLM agents, tools, and workflows. It implements the Bridge pattern
//! to support multiple scripting languages through a common interface.
//! 
//! ## Key Features
//! 
//! - **Multi-Language Support**: Execute scripts in Lua (Phase 1), JavaScript (Phase 5), 
//!   and Python (Phase 9)
//! - **Streaming Execution**: Support for streaming responses from LLM providers
//! - **Provider Integration**: Access multiple LLM providers through a unified API
//! - **Security Controls**: Fine-grained security settings for script execution
//! - **Component Registry**: Dynamic registration of agents, tools, and workflows
//! 
//! ## Architecture
//! 
//! The bridge uses a three-layer architecture:
//! 
//! 1. **ScriptEngineBridge Trait**: Defines the common interface for all script engines
//! 2. **Language Implementations**: Concrete implementations for each scripting language
//! 3. **ScriptRuntime**: High-level runtime that manages engines and provides the user API
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! use llmspell_bridge::{ScriptRuntime, RuntimeConfig};
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a runtime with Lua engine
//! let runtime = ScriptRuntime::new_with_lua(RuntimeConfig::default()).await?;
//! 
//! // Execute a simple script
//! let output = runtime.execute_script(r#"
//!     print("Hello from Lua!")
//!     return { message = "Script completed" }
//! "#).await?;
//! 
//! println!("Output: {:?}", output.output);
//! # Ok(())
//! # }
//! ```
//! 
//! ## Configuration
//! 
//! The runtime can be configured through `RuntimeConfig`:
//! 
//! ```rust,no_run
//! use llmspell_bridge::{RuntimeConfig, ScriptRuntime};
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut config = RuntimeConfig::default();
//! 
//! // Configure security settings
//! config.runtime.security.allow_file_access = false;
//! config.runtime.security.allow_network_access = true;
//! config.runtime.security.max_memory_bytes = Some(50_000_000); // 50MB
//! 
//! // Set default engine
//! config.default_engine = "lua".to_string();
//! 
//! let runtime = ScriptRuntime::new_with_engine_name("lua", config).await?;
//! # Ok(())
//! # }
//! ```
//! 
//! ## Provider Access
//! 
//! Scripts can access LLM providers configured in the runtime:
//! 
//! ```rust,no_run
//! # use llmspell_bridge::{ScriptRuntime, RuntimeConfig};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let runtime = ScriptRuntime::new_with_lua(RuntimeConfig::default()).await?;
//! 
//! let script = r#"
//!     -- List available providers
//!     local providers = Provider.list()
//!     for _, p in ipairs(providers) do
//!         print("Provider: " .. p.name)
//!     end
//!     
//!     return providers
//! "#;
//! 
//! let output = runtime.execute_script(script).await?;
//! # Ok(())
//! # }
//! ```

// Core modules
pub mod engine;
pub mod providers;
pub mod registry;
pub mod runtime;

// Language-specific implementations (feature-gated)
#[cfg(feature = "lua")]
pub mod lua;

#[cfg(feature = "javascript")]
pub mod javascript;

// Re-exports for convenience
pub use engine::{
    register_engine_plugin, unregister_engine_plugin, EngineFactory, EngineFeatures, EngineInfo,
    ExecutionContext, ScriptEngineBridge, ScriptEnginePlugin, ScriptMetadata, ScriptOutput,
    ScriptStream, SecurityContext,
};

pub use providers::{ProviderManager, ProviderManagerConfig};
pub use registry::ComponentRegistry;
pub use runtime::{RuntimeConfig, ScriptRuntime};
