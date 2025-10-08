// ABOUTME: Standardized Tool API specification for script engines
// ABOUTME: Defines the common interface that all script engines must implement for Tool access

//! # Standardized Tool API
//!
//! This module defines the standard API that all script engines (Lua, JavaScript, Python)
//! must implement for tool access. This ensures consistency across different scripting
//! environments.
//!
//! ## Global Object: `Tool`
//!
//! The Tool global object provides access to all registered tools in the system.
//!
//! ### Core Methods
//!
//! #### `Tool.list() -> Array<ToolInfo>`
//! Returns a list of all available tools with their metadata.
//! ```js
//! const tools = Tool.list();
//! // Returns: [{ name: "calculator", description: "...", version: "..." }, ...]
//! ```
//!
//! #### `Tool.get(name: string) -> ToolInstance | null`
//! Retrieves a specific tool by name, returning a tool instance or null if not found.
//! ```js
//! const calc = Tool.get("calculator");
//! if (calc) {
//!     const result = calc.execute({ expression: "2 + 2" });
//! }
//! ```
//!
//! #### `Tool.execute(name: string, params: object) -> object`
//! Directly invokes a tool by name with the given parameters.
//! ```js
//! const result = Tool.execute("calculator", { expression: "2 + 2" });
//! ```
//!
//! #### `Tool.exists(name: string) -> boolean`
//! Checks if a tool with the given name exists.
//! ```js
//! if (Tool.exists("web-searcher")) {
//!     // Use web search tool
//! }
//! ```
//!
//! #### `Tool.categories() -> Array<string>`
//! Returns all available tool categories.
//! ```js
//! const categories = Tool.categories();
//! // Returns: ["utility", "data_processing", "file_system", "web", ...]
//! ```
//!
//! #### `Tool.discover(filter?: { category?: string, tag?: string }) -> Array<ToolInfo>`
//! Discovers tools matching the optional filter criteria.
//! ```js
//! const webTools = Tool.discover({ category: "web" });
//! const searchTools = Tool.discover({ tag: "search" });
//! ```
//!
//! ### Direct Tool Access
//!
//! Tools can be accessed directly via the Tool object using dot notation:
//! ```js
//! const result = Tool.calculator.execute({ expression: "2 + 2" });
//! ```
//!
//! ### Tool Instance Methods
//!
//! Each tool instance returned by `Tool.get()` or accessed directly has:
//!
//! #### `execute(params: object) -> object`
//! Executes the tool with the given parameters.
//!
//! #### `getSchema() -> ToolSchema`
//! Returns the tool's parameter schema.
//!
//! ## Data Types
//!
//! ### `ToolInfo`
//! ```typescript
//! interface ToolInfo {
//!     name: string;
//!     description: string;
//!     category: string;
//!     version: string;
//! }
//! ```
//!
//! ### `ToolSchema`
//! ```typescript
//! interface ToolSchema {
//!     name: string;
//!     description: string;
//!     parameters: ParameterDef[];
//! }
//!
//! interface ParameterDef {
//!     name: string;
//!     type: string;
//!     description: string;
//!     required: boolean;
//!     default?: any;
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - All tool executions are subject to security constraints
//! - File system tools respect sandbox boundaries
//! - Network tools respect allowed domains
//! - Resource limits are enforced per tool execution
//!
//! ## Error Handling
//!
//! Tool methods throw errors on failure:
//! - Tool not found
//! - Invalid parameters
//! - Security violations
//! - Resource limit exceeded
//! - Tool execution failure
//!
//! ## Implementation Notes
//!
//! - All methods are synchronous from the script's perspective
//! - Async operations are handled internally by the bridge
//! - Tool discovery is cached for performance
//! - Direct tool access (`Tool.tool_name`) uses lazy loading

use llmspell_core::traits::tool::ToolSchema;
use serde::{Deserialize, Serialize};

/// Standard tool information exposed to scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool category
    pub category: String,
    /// Tool version
    pub version: String,
}

/// Standard tool schema exposed to scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardToolSchema {
    /// Schema name
    pub name: String,
    /// Schema description
    pub description: String,
    /// Parameters
    pub parameters: Vec<StandardParameterDef>,
}

/// Standard parameter definition exposed to scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardParameterDef {
    /// Parameter name
    pub name: String,
    /// Parameter type as string
    #[serde(rename = "type")]
    pub param_type: String,
    /// Parameter description
    pub description: String,
    /// Whether required
    pub required: bool,
    /// Default value if any
    pub default: Option<serde_json::Value>,
}

/// Convert from core `ToolSchema` to standard schema
#[must_use]
pub fn to_standard_schema(schema: ToolSchema) -> StandardToolSchema {
    StandardToolSchema {
        name: schema.name,
        description: schema.description,
        parameters: schema
            .parameters
            .into_iter()
            .map(|p| StandardParameterDef {
                name: p.name,
                param_type: format!("{:?}", p.param_type).to_lowercase(),
                description: p.description,
                required: p.required,
                default: p.default,
            })
            .collect(),
    }
}

/// Standard tool API trait that all script engines must implement
#[async_trait::async_trait]
pub trait StandardToolApi {
    /// List all available tools
    async fn list_tools(&self) -> Vec<StandardToolInfo>;

    /// Get a specific tool's information
    async fn get_tool(&self, name: &str) -> Option<StandardToolInfo>;

    /// Invoke a tool with parameters
    async fn invoke_tool(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String>;

    /// Check if a tool exists
    async fn tool_exists(&self, name: &str) -> bool;

    /// Get all tool categories
    async fn get_categories(&self) -> Vec<String>;

    /// Discover tools with optional filters
    async fn discover_tools(
        &self,
        category: Option<String>,
        tag: Option<String>,
    ) -> Vec<StandardToolInfo>;

    /// Get a tool's schema
    async fn get_tool_schema(&self, name: &str) -> Option<StandardToolSchema>;
}
