//! Lua global objects for scripting
//!
//! This module provides Lua bindings for all `LLMSpell` functionality,
//! making it accessible through global objects in Lua scripts.
//!
//! # Available Globals
//!
//! ## Agent
//! ```lua
//! -- Create and interact with LLM agents
//! local agent = Agent.create({name = "assistant", provider = "openai"})
//! local response = agent:execute("Hello!")
//! ```
//!
//! ## Tool
//! ```lua
//! -- Execute tools and query capabilities
//! local result = Tool.execute("calculator", {operation = "add", a = 5, b = 3})
//! local tools = Tool.list()
//! ```
//!
//! ## Workflow
//! ```lua
//! -- Build and execute workflows
//! local workflow = Workflow.sequential({name = "pipeline", steps = {...}})
//! local result = workflow:execute()
//! ```
//!
//! ## Session
//! ```lua
//! -- Manage sessions and artifacts
//! local session = Session.create({id = "user-123"})
//! session:store_artifact("data", {key = "value"})
//! session:save()
//! ```
//!
//! ## State
//! ```lua
//! -- Global state management
//! State.set("config", {theme = "dark"})
//! local config = State.get("config")
//! ```
//!
//! ## Event
//! ```lua
//! -- Event publishing and subscription
//! Event.publish("user.login", {user_id = "123"})
//! Event.subscribe("user.*", function(event) print(event) end)
//! ```
//!
//! ## Hook
//! ```lua
//! -- Hook registration and management
//! Hook.register("before_execute", function(ctx) return true end)
//! ```
//!
//! # Thread Safety
//!
//! All global objects are thread-safe and can be used from multiple Lua coroutines
//! simultaneously. The underlying Rust implementations handle synchronization.

pub mod agent;
pub mod args;
pub mod artifact;
pub mod config;
pub mod debug;
pub mod event;
pub mod hook;
pub mod json;
pub mod provider;
pub mod rag;
pub mod replay;
pub mod session;
pub mod state;
pub mod streaming;
pub mod tool;
pub mod workflow;

pub use agent::inject_agent_global;
pub use args::inject_args_global;
pub use artifact::inject_artifact_global;
pub use debug::inject_debug_global;
pub use event::inject_event_global;
pub use hook::inject_hook_global;
pub use json::inject_json_global;
pub use provider::inject_provider_global;
pub use rag::inject_rag_global;
pub use replay::inject_replay_global;
pub use session::inject_session_global;
pub use state::inject_state_global;
pub use streaming::inject_streaming_global;
pub use tool::inject_tool_global;
pub use workflow::inject_workflow_global;
