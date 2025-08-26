//! ABOUTME: Search provider abstraction and implementations
//! ABOUTME: Defines common interface for all search providers (Google, Bing, `DuckDuckGo`, `SerpApi`, `SerperDev`)

use async_trait::async_trait;
use llmspell_core::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod brave;
pub mod duckduckgo;
pub mod google;
pub mod serpapi;
pub mod serperdev;

// Re-export providers
pub use brave::BraveSearchProvider;
pub use duckduckgo::DuckDuckGoProvider;
pub use google::GoogleSearchProvider;
pub use serpapi::SerpApiProvider;
pub use serperdev::SerperDevProvider;

/// Search result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub provider: String,
    pub rank: usize,
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub max_results: usize,
    pub safe_search: bool,
    pub language: Option<String>,
    pub search_type: SearchType,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_results: 10,
            safe_search: true,
            language: Some("en".to_string()),
            search_type: SearchType::Web,
        }
    }
}

/// Search types supported by providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SearchType {
    #[default]
    Web,
    News,
    Images,
}

/// Common trait for all search providers
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Check if the provider is available (e.g., has API key configured)
    fn is_available(&self) -> bool;

    /// Get rate limit information (requests per minute)
    fn rate_limit(&self) -> Option<u32>;

    /// Perform a search
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>>;

    /// Get provider-specific metadata
    fn metadata(&self) -> Value {
        serde_json::json!({
            "name": self.name(),
            "available": self.is_available(),
            "rate_limit": self.rate_limit(),
        })
    }
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub additional_config: Value,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            additional_config: Value::Null,
        }
    }
}
