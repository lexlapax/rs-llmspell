//! ABOUTME: Web search tool implementation with multiple provider support
//! ABOUTME: Refactored to use shared rate limiter from llmspell-utils

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ExecutionContext,
    ComponentMetadata, LLMSpellError, Result,
};
use llmspell_utils::{
    extract_optional_string,
    extract_parameters,
    extract_required_string,
    // NEW: Using shared rate limiter
    rate_limiter::{RateLimiter, RateLimiterBuilder},
    response::ResponseBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{instrument, debug, error, info, trace, warn};

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

/// Web search tool implementation (refactored)
pub struct WebSearchTool {
    metadata: ComponentMetadata,
    config: WebSearchConfig,
    rate_limiter: RateLimiter,
}

impl WebSearchTool {
    /// Create a new web search tool
    pub fn new(config: WebSearchConfig) -> Result<Self> {
        info!(
            tool_name = "web-search-tool",
            default_provider = %config.default_provider,
            supported_providers = 3, // Google, Bing, DuckDuckGo
            max_results = config.max_results,
            rate_limit_per_minute = config.rate_limit,
            safe_search = config.safe_search,
            language = ?config.language,
            api_keys_count = config.api_keys.len(),
            security_level = "Safe",
            category = "Web",
            phase = "Phase 3 (comprehensive instrumentation)",
            "Creating WebSearchTool with configuration"
        );

        // Create rate limiter using shared utility
        let rate_limiter_creation_start = Instant::now();
        let rate_limiter = RateLimiterBuilder::default()
            .per_minute(config.rate_limit)
            .sliding_window()
            .build()
            .map_err(|e| {
                let rate_limiter_creation_duration_ms = rate_limiter_creation_start.elapsed().as_millis();
                error!(
                    rate_limit_per_minute = config.rate_limit,
                    rate_limiter_creation_duration_ms,
                    error = %e,
                    "Failed to create rate limiter"
                );
                LLMSpellError::Internal {
                    message: format!("Failed to create rate limiter: {}", e),
                    source: None,
                }
            })?;

        let rate_limiter_creation_duration_ms = rate_limiter_creation_start.elapsed().as_millis();
        debug!(
            rate_limit_per_minute = config.rate_limit,
            rate_limiter_creation_duration_ms,
            "Rate limiter created successfully"
        );

        Ok(Self {
            metadata: ComponentMetadata::new(
                "web-search-tool".to_string(),
                "Search the web using multiple providers".to_string(),
            ),
            config,
            rate_limiter,
        })
    }

    /// Perform search with a specific provider
    #[instrument(skip(self))]
    async fn search_with_provider(
        &self,
        query: &str,
        provider: SearchProvider,
        num_results: usize,
    ) -> Result<Vec<SearchResult>> {
        // Apply rate limiting
        self.rate_limiter
            .acquire()
            .await
            .map_err(|e| LLMSpellError::RateLimit {
                message: format!("Search rate limit exceeded: {}", e),
                retry_after: None,
            })?;

        info!("Searching with {} for: {}", provider, query);

        match provider {
            SearchProvider::Google => self.search_google(query, num_results).await,
            SearchProvider::Bing => self.search_bing(query, num_results).await,
            SearchProvider::DuckDuckGo => self.search_duckduckgo(query, num_results).await,
        }
    }

    /// Search using Google (requires API key)
    #[instrument(skip(self))]
    async fn search_google(&self, query: &str, _num_results: usize) -> Result<Vec<SearchResult>> {
        let _api_key =
            self.config
                .api_keys
                .get("google")
                .ok_or_else(|| LLMSpellError::Configuration {
                    message: "Google API key not configured".to_string(),
                    source: None,
                })?;

        // Mock implementation - would use actual Google Custom Search API
        warn!("Google search not fully implemented - returning mock results");
        Ok(vec![SearchResult {
            title: format!("Google result for: {}", query),
            url: "https://example.com".to_string(),
            snippet: "Mock Google search result".to_string(),
            provider: SearchProvider::Google,
            rank: 1,
        }])
    }

    /// Search using Bing (requires API key)
    #[instrument(skip(self))]
    async fn search_bing(&self, query: &str, _num_results: usize) -> Result<Vec<SearchResult>> {
        let _api_key =
            self.config
                .api_keys
                .get("bing")
                .ok_or_else(|| LLMSpellError::Configuration {
                    message: "Bing API key not configured".to_string(),
                    source: None,
                })?;

        // Mock implementation - would use actual Bing Search API
        warn!("Bing search not fully implemented - returning mock results");
        Ok(vec![SearchResult {
            title: format!("Bing result for: {}", query),
            url: "https://example.com".to_string(),
            snippet: "Mock Bing search result".to_string(),
            provider: SearchProvider::Bing,
            rank: 1,
        }])
    }

    /// Search using DuckDuckGo (no API key required)
    #[instrument(skip(self))]
    async fn search_duckduckgo(
        &self,
        query: &str,
        num_results: usize,
    ) -> Result<Vec<SearchResult>> {
        // Mock implementation - would use DuckDuckGo Instant Answer API
        warn!("DuckDuckGo search not fully implemented - returning mock results");

        let mut results = Vec::new();
        for i in 0..num_results.min(5) {
            results.push(SearchResult {
                title: format!("DuckDuckGo result {} for: {}", i + 1, query),
                url: format!("https://example{}.com", i + 1),
                snippet: format!("Mock DuckDuckGo search result {}", i + 1),
                provider: SearchProvider::DuckDuckGo,
                rank: i + 1,
            });
        }

        Ok(results)
    }

    /// Parse search parameters
    fn parse_parameters(
        &self,
        params: &serde_json::Value,
    ) -> Result<(String, SearchProvider, usize)> {
        let query = extract_required_string(params, "input")?.to_string();

        let provider = if let Some(provider_str) = extract_optional_string(params, "provider") {
            provider_str.parse()?
        } else {
            self.config.default_provider
        };

        let num_results = params
            .get("num_results")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(self.config.max_results)
            .min(self.config.max_results);

        Ok((query, provider, num_results))
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
            "web-searcher".to_string(),
            "Search the web using Google, Bing, or DuckDuckGo".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
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
        .with_parameter(ParameterDef {
            name: "num_results".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum number of results to return".to_string(),
            required: false,
            default: Some(serde_json::json!(10)),
        })
    }
}

#[async_trait]
impl BaseAgent for WebSearchTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[instrument(skip(_context, input, self), fields(tool = %self.metadata().name))]
    async fn execute_impl(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let execute_start = Instant::now();
        info!(
            tool_name = %self.metadata().name,
            input_text_length = input.text.len(),
            has_parameters = !input.parameters.is_empty(),
            "Starting WebSearchTool execution"
        );

        // Get parameters using shared utility
        let params = match extract_parameters(&input) {
            Ok(params) => {
                debug!(
                    param_count = params.as_object().map(|o| o.len()).unwrap_or(0),
                    "Successfully extracted parameters"
                );
                params
            }
            Err(e) => {
                error!(
                    error = %e,
                    input_text_length = input.text.len(),
                    "Failed to extract parameters"
                );
                return Err(e);
            }
        };

        // Parse parameters
        let parameter_parsing_start = Instant::now();
        let (query, provider, num_results) = match self.parse_parameters(params) {
            Ok(parsed) => {
                let parameter_parsing_duration_ms = parameter_parsing_start.elapsed().as_millis();
                debug!(
                    query = %parsed.0,
                    provider = %parsed.1,
                    num_results = parsed.2,
                    parameter_parsing_duration_ms,
                    "Successfully parsed search parameters"
                );
                parsed
            }
            Err(e) => {
                let parameter_parsing_duration_ms = parameter_parsing_start.elapsed().as_millis();
                error!(
                    error = %e,
                    parameter_parsing_duration_ms,
                    "Failed to parse search parameters"
                );
                return Err(e);
            }
        };

        info!(
            query = %query,
            provider = %provider,
            requested_results = num_results,
            max_allowed_results = self.config.max_results,
            safe_search = self.config.safe_search,
            rate_limit = self.config.rate_limit,
            "Executing web search with validated parameters"
        );

        trace!(
            query_length = query.len(),
            query_preview = %query.chars().take(100).collect::<String>(),
            "Search query details"
        );

        // Perform search
        let search_start = Instant::now();
        let results = match self.search_with_provider(&query, provider, num_results).await {
            Ok(results) => {
                let search_duration_ms = search_start.elapsed().as_millis();
                debug!(
                    provider = %provider,
                    query = %query,
                    returned_results = results.len(),
                    requested_results = num_results,
                    search_duration_ms,
                    "Search completed successfully"
                );
                results
            }
            Err(e) => {
                let search_duration_ms = search_start.elapsed().as_millis();
                error!(
                    provider = %provider,
                    query = %query,
                    requested_results = num_results,
                    search_duration_ms,
                    error = %e,
                    "Search failed"
                );
                return Err(e);
            }
        };

        // Create success message
        let message = format!(
            "Found {} results for '{}' using {}",
            results.len(),
            query,
            provider
        );

        // Build response
        let response_building_start = Instant::now();
        let response = ResponseBuilder::success("search")
            .with_message(message)
            .with_result(serde_json::json!({
                "query": query,
                "provider": provider.to_string(),
                "count": results.len(),
                "results": results,
            }))
            .build();

        let response_building_duration_ms = response_building_start.elapsed().as_millis();
        let total_execution_duration_ms = execute_start.elapsed().as_millis();

        info!(
            query = %query,
            provider = %provider,
            results_count = results.len(),
            requested_results = num_results,
            total_duration_ms = total_execution_duration_ms,
            search_duration_ms = search_start.elapsed().as_millis(),
            response_building_duration_ms,
            success = true,
            "WebSearchTool execution completed successfully"
        );

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    #[instrument(skip(self))]
    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Web search error: {}", error)))
    }
}
