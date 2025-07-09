//! ABOUTME: Web search tool implementation with multiple provider support
//! ABOUTME: Provides rate-limited web search with Google, Bing, and DuckDuckGo

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use llmspell_utils::{extract_optional_string, extract_parameters, extract_required_string};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Search provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchProvider {
    Google,
    Bing,
    DuckDuckGo,
}

impl std::fmt::Display for SearchProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchProvider::Google => write!(f, "google"),
            SearchProvider::Bing => write!(f, "bing"),
            SearchProvider::DuckDuckGo => write!(f, "duckduckgo"),
        }
    }
}

impl std::str::FromStr for SearchProvider {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "google" => Ok(SearchProvider::Google),
            "bing" => Ok(SearchProvider::Bing),
            "duckduckgo" | "ddg" => Ok(SearchProvider::DuckDuckGo),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown search provider: {}", s),
                field: Some("provider".to_string()),
            }),
        }
    }
}

/// Search result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub provider: SearchProvider,
    pub rank: usize,
}

/// Web search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchConfig {
    /// Default search provider
    pub default_provider: SearchProvider,
    /// API keys for different providers
    pub api_keys: HashMap<String, String>,
    /// Maximum results per search
    pub max_results: usize,
    /// Rate limit (searches per minute)
    pub rate_limit: u32,
    /// Enable safe search
    pub safe_search: bool,
    /// Language preference
    pub language: Option<String>,
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        Self {
            default_provider: SearchProvider::DuckDuckGo, // No API key required
            api_keys: HashMap::new(),
            max_results: 10,
            rate_limit: 60,
            safe_search: true,
            language: Some("en".to_string()),
        }
    }
}

/// Rate limiter for search requests
struct RateLimiter {
    requests: Vec<std::time::Instant>,
    limit: u32,
    window: std::time::Duration,
}

impl RateLimiter {
    fn new(limit: u32) -> Self {
        Self {
            requests: Vec::new(),
            limit,
            window: std::time::Duration::from_secs(60), // 1 minute window
        }
    }

    fn check_and_record(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        let window_start = now - self.window;

        // Remove old requests
        self.requests.retain(|&time| time > window_start);

        // Check if we're under the limit
        if self.requests.len() >= self.limit as usize {
            let wait_time = self.requests[0] + self.window - now;
            return Err(LLMSpellError::Resource {
                message: format!(
                    "Rate limit exceeded. Please wait {} seconds",
                    wait_time.as_secs()
                ),
                resource_type: Some("search_api".to_string()),
                source: None,
            });
        }

        // Record new request
        self.requests.push(now);
        Ok(())
    }
}

/// Web search tool implementation
pub struct WebSearchTool {
    metadata: ComponentMetadata,
    config: WebSearchConfig,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

impl WebSearchTool {
    /// Create a new web search tool
    pub fn new(config: WebSearchConfig) -> Self {
        let rate_limit = config.rate_limit;
        Self {
            metadata: ComponentMetadata::new(
                "web-search-tool".to_string(),
                "Search the web using multiple providers".to_string(),
            ),
            config,
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new(rate_limit))),
        }
    }

    /// Set API key for a provider
    pub fn with_api_key(mut self, provider: &str, api_key: String) -> Self {
        self.config.api_keys.insert(provider.to_string(), api_key);
        self
    }

    /// Search using a specific provider
    async fn search_with_provider(
        &self,
        query: &str,
        provider: SearchProvider,
    ) -> Result<Vec<SearchResult>> {
        // Check rate limit
        {
            let mut limiter = self.rate_limiter.write().await;
            limiter.check_and_record()?;
        }

        info!("Searching '{}' using provider: {}", query, provider);

        match provider {
            SearchProvider::Google => self.search_google(query).await,
            SearchProvider::Bing => self.search_bing(query).await,
            SearchProvider::DuckDuckGo => self.search_duckduckgo(query).await,
        }
    }

    /// Search using Google
    async fn search_google(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Check if we have an API key
        let _api_key =
            self.config
                .api_keys
                .get("google")
                .ok_or_else(|| LLMSpellError::Configuration {
                    message: "Google API key not configured".to_string(),
                    source: None,
                })?;

        // TODO: In a real implementation, this would use the Google Custom Search API
        // For now, we'll return mock results
        warn!("Google search is not yet implemented, returning mock results");
        Ok(self.create_mock_results(query, SearchProvider::Google))
    }

    /// Search using Bing
    async fn search_bing(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Check if we have an API key
        let _api_key =
            self.config
                .api_keys
                .get("bing")
                .ok_or_else(|| LLMSpellError::Configuration {
                    message: "Bing API key not configured".to_string(),
                    source: None,
                })?;

        // TODO: In a real implementation, this would use the Bing Web Search API
        warn!("Bing search is not yet implemented, returning mock results");
        Ok(self.create_mock_results(query, SearchProvider::Bing))
    }

    /// Search using DuckDuckGo (no API key required)
    async fn search_duckduckgo(&self, query: &str) -> Result<Vec<SearchResult>> {
        // TODO: In a real implementation, this would use DuckDuckGo's instant answer API
        // or scrape their HTML (respecting robots.txt)
        warn!("DuckDuckGo search is not yet implemented, returning mock results");
        Ok(self.create_mock_results(query, SearchProvider::DuckDuckGo))
    }

    /// Create mock search results for testing
    fn create_mock_results(&self, query: &str, provider: SearchProvider) -> Vec<SearchResult> {
        (0..self.config.max_results.min(5))
            .map(|i| SearchResult {
                title: format!("Result {} for: {}", i + 1, query),
                url: format!("https://example.com/result{}", i + 1),
                snippet: format!(
                    "This is a mock search result for '{}' from {}. In a real implementation, \
                     this would contain actual search snippet text.",
                    query, provider
                ),
                provider,
                rank: i + 1,
            })
            .collect()
    }

    /// Format results as text
    fn format_results(&self, results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "No results found.".to_string();
        }

        results
            .iter()
            .map(|r| format!("{}. {}\n   {}\n   {}", r.rank, r.title, r.url, r.snippet))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Parse parameters from input
    fn parse_parameters(
        &self,
        params: &serde_json::Value,
    ) -> Result<(String, Option<SearchProvider>)> {
        // Get query
        let query = extract_required_string(params, "query")?.to_string();

        // Get optional provider
        let provider = extract_optional_string(params, "provider")
            .map(|s| s.parse::<SearchProvider>())
            .transpose()?;

        Ok((query, provider))
    }
}

#[async_trait]
impl BaseAgent for WebSearchTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Parse parameters
        let (query, provider) = self.parse_parameters(params)?;
        let search_provider = provider.unwrap_or(self.config.default_provider);

        debug!(
            "Executing web search: query='{}', provider={}",
            query, search_provider
        );

        // Perform search
        let results = self.search_with_provider(&query, search_provider).await?;

        // Format results
        let formatted = self.format_results(&results);

        // Return as JSON and text
        let output_json = serde_json::json!({
            "query": query,
            "provider": search_provider.to_string(),
            "count": results.len(),
            "results": results,
        });

        // Create metadata with the search results
        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata
            .extra
            .insert("output_json".to_string(), output_json);

        Ok(AgentOutput::text(formatted).with_metadata(metadata))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Web search error: {}", error)))
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Web
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "web_search".to_string(),
            "Search the web using Google, Bing, or DuckDuckGo".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "query".to_string(),
            param_type: ParameterType::String,
            description: "The search query".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "provider".to_string(),
            param_type: ParameterType::String,
            description: "Search provider: google, bing, or duckduckgo (optional)".to_string(),
            required: false,
            default: Some(serde_json::json!("duckduckgo")),
        })
        .with_returns(ParameterType::Object)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
            .with_network_access("*.google.com")
            .with_network_access("*.bing.com")
            .with_network_access("*.duckduckgo.com")
            .with_env_access("GOOGLE_SEARCH_API_KEY")
            .with_env_access("BING_SEARCH_API_KEY")
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_network_limit(10 * 1024 * 1024) // 10MB/s
            .with_cpu_limit(5000) // 5 seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_input(text: &str, params: serde_json::Value) -> AgentInput {
        AgentInput {
            text: text.to_string(),
            media: vec![],
            context: None,
            parameters: {
                let mut map = HashMap::new();
                map.insert("parameters".to_string(), params);
                map
            },
            output_modalities: vec![],
        }
    }

    #[test]
    fn test_search_provider_parsing() {
        assert_eq!(
            "google".parse::<SearchProvider>().unwrap(),
            SearchProvider::Google
        );
        assert_eq!(
            "bing".parse::<SearchProvider>().unwrap(),
            SearchProvider::Bing
        );
        assert_eq!(
            "duckduckgo".parse::<SearchProvider>().unwrap(),
            SearchProvider::DuckDuckGo
        );
        assert_eq!(
            "ddg".parse::<SearchProvider>().unwrap(),
            SearchProvider::DuckDuckGo
        );
        assert!("invalid".parse::<SearchProvider>().is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2); // 2 requests per minute

        // First two requests should succeed
        assert!(limiter.check_and_record().is_ok());
        assert!(limiter.check_and_record().is_ok());

        // Third request should fail
        assert!(limiter.check_and_record().is_err());
    }

    #[tokio::test]
    async fn test_web_search_tool_creation() {
        let config = WebSearchConfig::default();
        let tool = WebSearchTool::new(config);

        assert_eq!(tool.category(), ToolCategory::Web);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);

        let schema = tool.schema();
        assert_eq!(schema.name, "web_search");
        assert_eq!(schema.required_parameters(), vec!["query"]);
    }

    #[tokio::test]
    async fn test_parameter_parsing() {
        let config = WebSearchConfig::default();
        let tool = WebSearchTool::new(config);

        // Valid parameters
        let params = serde_json::json!({
            "query": "rust programming",
            "provider": "google"
        });
        let (query, provider) = tool.parse_parameters(&params).unwrap();
        assert_eq!(query, "rust programming");
        assert_eq!(provider, Some(SearchProvider::Google));

        // Missing query
        let params = serde_json::json!({
            "provider": "google"
        });
        assert!(tool.parse_parameters(&params).is_err());
    }

    #[tokio::test]
    async fn test_mock_search_results() {
        let config = WebSearchConfig {
            max_results: 3,
            ..Default::default()
        };
        let tool = WebSearchTool::new(config);

        let results = tool.create_mock_results("test query", SearchProvider::DuckDuckGo);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[0].provider, SearchProvider::DuckDuckGo);
    }

    #[tokio::test]
    async fn test_search_execution() {
        let config = WebSearchConfig::default();
        let tool = WebSearchTool::new(config);

        let input = create_test_input(
            "search for rust",
            serde_json::json!({
                "query": "rust programming language"
            }),
        );

        let context = ExecutionContext::with_conversation("test".to_string());
        let result = tool.execute(input, context).await.unwrap();

        assert!(result
            .text
            .contains("Result 1 for: rust programming language"));

        // Check metadata
        let metadata = result.metadata.extra.get("output_json").unwrap();
        assert_eq!(metadata["query"], "rust programming language");
        assert_eq!(metadata["provider"], "duckduckgo");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = WebSearchConfig {
            rate_limit: 2, // Very low limit for testing
            ..Default::default()
        };
        let tool = WebSearchTool::new(config);

        // First two searches should work
        for i in 0..2 {
            let input = create_test_input(
                "search",
                serde_json::json!({
                    "query": format!("test {}", i)
                }),
            );
            let context = ExecutionContext::with_conversation("test".to_string());
            assert!(tool.execute(input, context).await.is_ok());
        }

        // Third search should fail with rate limit
        let input = create_test_input(
            "search",
            serde_json::json!({
                "query": "test 3"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());
        let result = tool.execute(input, context).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            LLMSpellError::Resource { message, .. } => {
                assert!(message.contains("Rate limit exceeded"));
            }
            _ => panic!("Expected Resource error"),
        }
    }
}
