//! ABOUTME: llmspell-bridge - Language-agnostic script runtime with bridge pattern
//! ABOUTME: Supports multiple script engines (Lua, JavaScript, Python) through `ScriptEngineBridge`
//!
//! # `LLMSpell` Bridge
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
//! 1. **`ScriptEngineBridge` Trait**: Defines the common interface for all script engines
//! 2. **Language Implementations**: Concrete implementations for each scripting language
//! 3. **`ScriptRuntime`**: High-level runtime that manages engines and provides the user API
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use llmspell_bridge::ScriptRuntime;
//! use llmspell_config::LLMSpellConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a runtime with Lua engine
//! let runtime = ScriptRuntime::new_with_lua(LLMSpellConfig::default()).await?;
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
//! ## Lua Integration
//!
//! ### Creating Agents
//!
//! ```lua
//! -- Create an agent with OpenAI provider
//! local agent = Agent.create({
//!     name = "assistant",
//!     provider = "openai",
//!     model = "gpt-4",
//!     temperature = 0.7
//! })
//!
//! -- Execute the agent
//! local response = agent:execute("What is the capital of France?")
//! print(response.text)
//!
//! -- Using streaming
//! local stream = agent:execute_stream("Tell me a story")
//! for chunk in stream do
//!     io.write(chunk.content)
//! end
//! ```
//!
//! ### Using Tools
//!
//! ```lua
//! -- List available tools
//! local tools = Tool.list()
//! for _, tool in ipairs(tools) do
//!     print(tool.name, tool.description)
//! end
//!
//! -- Execute a tool
//! local result = Tool.execute("file_reader", {
//!     path = "/tmp/data.txt"
//! })
//!
//! -- Create custom tool wrapper
//! local calculator = Tool.wrap("calculator")
//! local sum = calculator({operation = "add", a = 5, b = 3})
//! ```
//!
//! ### Building Workflows
//!
//! ```lua
//! -- Create a sequential workflow
//! local workflow = Workflow.sequential({
//!     name = "data_pipeline",
//!     steps = {
//!         {tool = "file_reader", params = {path = "input.txt"}},
//!         {tool = "text_processor", params = {operation = "uppercase"}},
//!         {tool = "file_writer", params = {path = "output.txt"}}
//!     }
//! })
//!
//! -- Execute workflow
//! local result = workflow:execute()
//!
//! -- Parallel workflow
//! local parallel = Workflow.parallel({
//!     name = "multi_search",
//!     steps = {
//!         {tool = "web_search", params = {query = "rust programming"}},
//!         {tool = "arxiv_search", params = {query = "machine learning"}},
//!         {tool = "news_search", params = {query = "technology"}}
//!     }
//! })
//! ```
//!
//! ### Session Management
//!
//! ```lua
//! -- Create or load session
//! local session = Session.load("user-123") or Session.create({
//!     id = "user-123",
//!     metadata = {user = "alice", created = os.time()}
//! })
//!
//! -- Store artifacts
//! session:store_artifact("conversation", conversation_history)
//! session:store_artifact("settings", user_preferences)
//!
//! -- Save session
//! session:save()
//!
//! -- List all sessions
//! local sessions = Session.list()
//! ```
//!
//! ## JavaScript Support (Planned - Phase 5)
//!
//! ```javascript
//! // Create agent
//! const agent = await Agent.create({
//!     name: "assistant",
//!     provider: "anthropic",
//!     model: "claude-3-opus"
//! });
//!
//! // Execute with async/await
//! const response = await agent.execute("Explain quantum computing");
//! console.log(response.text);
//!
//! // Tool execution
//! const result = await Tool.execute("web_search", {
//!     query: "latest AI news"
//! });
//! ```
//!
//! ## Python Support (Planned - Phase 9)
//!
//! ```python
//! # Create agent
//! agent = Agent.create(
//!     name="assistant",
//!     provider="openai",
//!     model="gpt-4"
//! )
//!
//! # Execute
//! response = agent.execute("What is machine learning?")
//! print(response.text)
//!
//! # Use with async
//! async def process():
//!     async for chunk in agent.execute_stream("Tell me about Python"):
//!         print(chunk.content, end="")
//! ```
//!
//! ## Cross-Language Compatibility
//!
//! All scripting languages share the same underlying Rust implementations, ensuring:
//!
//! - **Consistent Behavior**: Same results across languages
//! - **Shared State**: Sessions and artifacts accessible from any language
//! - **Unified Security**: Same security policies apply to all scripts
//! - **Performance**: Native Rust performance for all operations
//!
//! ## Configuration
//!
//! The runtime can be configured through `LLMSpellConfig`:
//!
//! ```rust,no_run
//! use llmspell_bridge::ScriptRuntime;
//! use llmspell_config::LLMSpellConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut config = LLMSpellConfig::default();
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
//! # use llmspell_bridge::ScriptRuntime;
//! # use llmspell_config::LLMSpellConfig;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let runtime = ScriptRuntime::new_with_lua(LLMSpellConfig::default()).await?;
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
pub mod config_bridge;
pub mod conversion;
pub mod debug_bridge;
pub mod discovery;
pub mod engine;
pub mod providers;
pub mod providers_discovery;
pub mod registry;
pub mod runtime;
pub mod state_adapter;
pub mod storage;
pub mod tools;

// Event bridge modules
pub mod event_bridge;
pub mod event_bus_adapter;
pub mod event_serialization;

// Global injection infrastructure
pub mod globals;

// Hook bridge module
pub mod hook_bridge;

// Session and artifact bridges
pub mod artifact_bridge;
pub mod session_bridge;

// Agent bridge modules
pub mod agent_bridge;
pub mod agents;
pub mod monitoring;

// Workflow modules (consolidated)
pub mod orchestration;
pub mod workflow_performance;
pub mod workflows; // Includes WorkflowBridge, WorkflowRegistry, and StandardizedWorkflowFactory (consolidated)

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

pub use llmspell_config::LLMSpellConfig;
pub use providers::ProviderManager;
pub use registry::ComponentRegistry;
pub use runtime::ScriptRuntime;
