//! Tool source abstraction for local and remote tools
//!
//! This module provides the foundation for tool discovery from multiple sources,
//! preparing for future MCP (Phase 12) and A2A (Phase 18) protocol support.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Tool source type for discovery and execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ToolSource {
    /// Local tools from ComponentRegistry
    #[default]
    Local,

    /// MCP (Model Context Protocol) tools - Phase 12
    #[cfg(feature = "mcp")]
    MCP {
        /// MCP server address
        server: String,
        /// Optional authentication token
        token: Option<String>,
    },

    /// A2A (Agent-to-Agent) tools - Phase 18
    #[cfg(feature = "a2a")]
    A2A {
        /// A2A node identifier
        node: String,
        /// Cluster namespace
        namespace: Option<String>,
    },
}

impl ToolSource {
    /// Parse tool source from string
    pub fn parse(s: &str) -> Result<Self> {
        if s == "local" || s.is_empty() {
            return Ok(ToolSource::Local);
        }

        #[cfg(feature = "mcp")]
        if let Some(server) = s.strip_prefix("mcp:") {
            return Ok(ToolSource::MCP {
                server: server.to_string(),
                token: None,
            });
        }

        #[cfg(feature = "a2a")]
        if let Some(node) = s.strip_prefix("a2a:") {
            let parts: Vec<&str> = node.split('/').collect();
            return Ok(ToolSource::A2A {
                node: parts[0].to_string(),
                namespace: parts.get(1).map(|s| s.to_string()),
            });
        }

        anyhow::bail!("Invalid tool source format: {}", s)
    }
}

/// Tool information returned by resolvers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub security_level: String,
    pub source: ToolSource,
    pub capabilities: Vec<String>,
}

/// Capability matcher for tool discovery
#[derive(Debug, Default)]
pub struct CapabilityMatcher {
    pub search_terms: Vec<String>,
    pub categories: Vec<String>,
    pub min_security: Option<String>,
}

impl CapabilityMatcher {
    /// Create a new capability matcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Add search terms
    pub fn with_search_terms(mut self, terms: Vec<String>) -> Self {
        self.search_terms = terms;
        self
    }

    /// Add category filters
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.categories = categories;
        self
    }

    /// Set minimum security level
    pub fn with_min_security(mut self, level: String) -> Self {
        self.min_security = Some(level);
        self
    }

    /// Check if a tool matches the criteria
    pub fn matches(&self, tool: &ToolInfo) -> bool {
        // Check search terms
        if !self.search_terms.is_empty() {
            let matches_search = self.search_terms.iter().any(|term| {
                let term_lower = term.to_lowercase();
                tool.name.to_lowercase().contains(&term_lower)
                    || tool.description.to_lowercase().contains(&term_lower)
                    || tool
                        .capabilities
                        .iter()
                        .any(|cap| cap.to_lowercase().contains(&term_lower))
            });
            if !matches_search {
                return false;
            }
        }

        // Check categories
        if !self.categories.is_empty() && !self.categories.contains(&tool.category) {
            return false;
        }

        // All criteria matched
        true
    }
}

/// Tool resolver trait for different sources (simplified for now)
pub trait ToolResolver {
    /// List all available tools
    fn list(&self) -> Result<Vec<String>>;

    /// Search tools by capability matcher
    fn search(&self, matcher: CapabilityMatcher) -> Result<Vec<ToolInfo>>;

    /// Get detailed tool information
    fn info(&self, name: &str) -> Result<Option<ToolInfo>>;
}

/// Local tool resolver (placeholder for ComponentRegistry integration)
#[derive(Default)]
pub struct LocalToolResolver {
    // TODO: Add ComponentRegistry reference when available
}

impl LocalToolResolver {
    /// Create a new local tool resolver
    pub fn new() -> Self {
        Self::default()
    }
}

impl ToolResolver for LocalToolResolver {
    fn list(&self) -> Result<Vec<String>> {
        // TODO: List from ComponentRegistry
        Ok(vec![
            "calculator".to_string(),
            "file_operations".to_string(),
            "web_scraper".to_string(),
        ])
    }

    fn search(&self, matcher: CapabilityMatcher) -> Result<Vec<ToolInfo>> {
        // TODO: Search ComponentRegistry
        let all_tools = vec![
            ToolInfo {
                name: "calculator".to_string(),
                description: "Mathematical calculations".to_string(),
                category: "utility".to_string(),
                security_level: "safe".to_string(),
                source: ToolSource::Local,
                capabilities: vec!["math".to_string(), "calculation".to_string()],
            },
            ToolInfo {
                name: "file_operations".to_string(),
                description: "File system operations".to_string(),
                category: "filesystem".to_string(),
                security_level: "restricted".to_string(),
                source: ToolSource::Local,
                capabilities: vec!["read".to_string(), "write".to_string()],
            },
        ];

        Ok(all_tools
            .into_iter()
            .filter(|tool| matcher.matches(tool))
            .collect())
    }

    fn info(&self, name: &str) -> Result<Option<ToolInfo>> {
        // TODO: Get from ComponentRegistry
        Ok(Some(ToolInfo {
            name: name.to_string(),
            description: format!("Tool {} - placeholder", name),
            category: "utility".to_string(),
            security_level: "safe".to_string(),
            source: ToolSource::Local,
            capabilities: vec![],
        }))
    }
}

// Future MCP implementation stub
#[cfg(feature = "mcp")]
pub mod mcp {
    use super::*;

    /// MCP tool resolver for Phase 12
    #[allow(dead_code)]
    pub struct MCPToolResolver {
        server: String,
        token: Option<String>,
    }

    impl MCPToolResolver {
        pub fn new(server: String, token: Option<String>) -> Self {
            Self { server, token }
        }
    }

    impl ToolResolver for MCPToolResolver {
        fn list(&self) -> Result<Vec<String>> {
            // Phase 12: Connect to MCP server and list tools
            anyhow::bail!("MCP support will be implemented in Phase 12")
        }

        fn search(&self, _matcher: CapabilityMatcher) -> Result<Vec<ToolInfo>> {
            // Phase 12: Search MCP tools
            anyhow::bail!("MCP support will be implemented in Phase 12")
        }

        fn info(&self, _name: &str) -> Result<Option<ToolInfo>> {
            // Phase 12: Get MCP tool info
            anyhow::bail!("MCP support will be implemented in Phase 12")
        }
    }
}

// Future A2A implementation stub
#[cfg(feature = "a2a")]
pub mod a2a {
    use super::*;

    /// A2A tool resolver for Phase 18
    #[allow(dead_code)]
    pub struct A2AToolResolver {
        node: String,
        namespace: Option<String>,
    }

    impl A2AToolResolver {
        pub fn new(node: String, namespace: Option<String>) -> Self {
            Self { node, namespace }
        }
    }

    impl ToolResolver for A2AToolResolver {
        fn list(&self) -> Result<Vec<String>> {
            // Phase 18: List distributed tools
            anyhow::bail!("A2A support will be implemented in Phase 18")
        }

        fn search(&self, _matcher: CapabilityMatcher) -> Result<Vec<ToolInfo>> {
            // Phase 18: Search A2A tools
            anyhow::bail!("A2A support will be implemented in Phase 18")
        }

        fn info(&self, _name: &str) -> Result<Option<ToolInfo>> {
            // Phase 18: Get A2A tool info
            anyhow::bail!("A2A support will be implemented in Phase 18")
        }
    }
}

/// Registry for multiple tool resolvers
#[derive(Default)]
pub struct ToolResolverRegistry {
    local_resolver: LocalToolResolver,
    #[cfg(feature = "mcp")]
    #[allow(dead_code)]
    mcp_resolvers: std::collections::HashMap<String, mcp::MCPToolResolver>,
    #[cfg(feature = "a2a")]
    #[allow(dead_code)]
    a2a_resolvers: std::collections::HashMap<String, a2a::A2AToolResolver>,
}

impl ToolResolverRegistry {
    /// Create a new resolver registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Get local resolver
    pub fn local(&self) -> &LocalToolResolver {
        &self.local_resolver
    }
}
