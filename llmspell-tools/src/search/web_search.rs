//! ABOUTME: Enhanced web search tool implementation with multiple provider support
//! ABOUTME: Supports DuckDuckGo, Google, Brave, SerpApi, and SerperDev with rate limiting

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use llmspell_utils::{
    extract_optional_string, extract_parameters, extract_required_string,
    rate_limiter::{RateLimiter, RateLimiterBuilder},
    response::ResponseBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::search::providers::{
    BraveSearchProvider, DuckDuckGoProvider, GoogleSearchProvider, ProviderConfig, SearchOptions,
    SearchProvider, SearchResult, SearchType, SerpApiProvider, SerperDevProvider,
};

/// Web search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchConfig {
    /// Default search provider
    pub default_provider: String,
    /// API keys and configuration for different providers
    pub providers: HashMap<String, ProviderConfig>,
    /// Maximum results per search
    pub max_results: usize,
    /// Enable safe search
    pub safe_search: bool,
    /// Language preference
    pub language: Option<String>,
    /// Provider fallback chain
    pub fallback_chain: Vec<String>,
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        // Default fallback chain prioritizes free/high-limit providers
        let fallback_chain = vec![
            "duckduckgo".to_string(), // No API key required
            "serperdev".to_string(),  // 2,500/month free
            "brave".to_string(),      // 2,000/month free
            "google".to_string(),     // 100/day free
            "serpapi".to_string(),    // 100/month free
        ];

        Self {
            default_provider: "duckduckgo".to_string(),
            providers: HashMap::new(),
            max_results: 10,
            safe_search: true,
            language: Some("en".to_string()),
            fallback_chain,
        }
    }
}

impl WebSearchConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        let mut providers = HashMap::new();

        // Google configuration
        if let Ok(api_key) = std::env::var("WEBSEARCH_GOOGLE_API_KEY") {
            let additional_config =
                if let Ok(engine_id) = std::env::var("WEBSEARCH_GOOGLE_SEARCH_ENGINE_ID") {
                    serde_json::json!({
                        "search_engine_id": engine_id
                    })
                } else {
                    serde_json::Value::Null
                };
            let google_config = ProviderConfig {
                api_key: Some(api_key),
                additional_config,
            };
            providers.insert("google".to_string(), google_config);
        }

        // Brave configuration
        if let Ok(api_key) = std::env::var("WEBSEARCH_BRAVE_API_KEY") {
            let brave_config = ProviderConfig {
                api_key: Some(api_key),
                ..Default::default()
            };
            providers.insert("brave".to_string(), brave_config);
        }

        // SerpApi configuration
        if let Ok(api_key) = std::env::var("WEBSEARCH_SERPAPI_API_KEY") {
            let serpapi_config = ProviderConfig {
                api_key: Some(api_key),
                ..Default::default()
            };
            providers.insert("serpapi".to_string(), serpapi_config);
        }

        // SerperDev configuration
        if let Ok(api_key) = std::env::var("WEBSEARCH_SERPERDEV_API_KEY") {
            let serperdev_config = ProviderConfig {
                api_key: Some(api_key),
                ..Default::default()
            };
            providers.insert("serperdev".to_string(), serperdev_config);
        }

        // DuckDuckGo doesn't need configuration
        providers.insert("duckduckgo".to_string(), ProviderConfig::default());

        config.providers = providers;

        // Override default provider if specified
        if let Ok(default) = std::env::var("WEBSEARCH_DEFAULT_PROVIDER") {
            config.default_provider = default;
        }

        config
    }
}

/// Provider wrapper with rate limiting
struct ProviderWrapper {
    provider: Box<dyn SearchProvider>,
    rate_limiter: Option<RateLimiter>,
}

/// Enhanced web search tool implementation
pub struct WebSearchTool {
    metadata: ComponentMetadata,
    config: WebSearchConfig,
    providers: Arc<Mutex<HashMap<String, ProviderWrapper>>>,
}

impl WebSearchTool {
    /// Create a new web search tool
    pub fn new(config: WebSearchConfig) -> Result<Self> {
        let mut providers = HashMap::new();

        // Initialize DuckDuckGo (always available)
        let ddg_provider = DuckDuckGoProvider::new();
        providers.insert(
            "duckduckgo".to_string(),
            ProviderWrapper {
                provider: Box::new(ddg_provider),
                rate_limiter: None, // No official rate limit
            },
        );

        // Initialize configured providers
        for (name, provider_config) in &config.providers {
            let wrapper = match name.as_str() {
                "google" => {
                    let provider = GoogleSearchProvider::new(provider_config.clone());
                    if provider.is_available() {
                        Some(ProviderWrapper {
                            provider: Box::new(provider),
                            rate_limiter: Some(
                                RateLimiterBuilder::default()
                                    .custom(100, Duration::from_secs(24 * 60 * 60)) // 100 per day
                                    .sliding_window()
                                    .build()
                                    .map_err(|e| LLMSpellError::Internal {
                                        message: format!("Failed to create rate limiter: {}", e),
                                        source: None,
                                    })?,
                            ),
                        })
                    } else {
                        None
                    }
                }
                "brave" => {
                    let provider = BraveSearchProvider::new(provider_config.clone());
                    if provider.is_available() {
                        Some(ProviderWrapper {
                            provider: Box::new(provider),
                            rate_limiter: Some(
                                RateLimiterBuilder::default()
                                    .per_minute(60) // ~2000/month spread evenly
                                    .sliding_window()
                                    .build()
                                    .map_err(|e| LLMSpellError::Internal {
                                        message: format!("Failed to create rate limiter: {}", e),
                                        source: None,
                                    })?,
                            ),
                        })
                    } else {
                        None
                    }
                }
                "serpapi" => {
                    let provider = SerpApiProvider::new(provider_config.clone());
                    if provider.is_available() {
                        Some(ProviderWrapper {
                            provider: Box::new(provider),
                            rate_limiter: Some(
                                RateLimiterBuilder::default()
                                    .per_minute(2) // ~100/month conservative
                                    .sliding_window()
                                    .build()
                                    .map_err(|e| LLMSpellError::Internal {
                                        message: format!("Failed to create rate limiter: {}", e),
                                        source: None,
                                    })?,
                            ),
                        })
                    } else {
                        None
                    }
                }
                "serperdev" => {
                    let provider = SerperDevProvider::new(provider_config.clone());
                    if provider.is_available() {
                        Some(ProviderWrapper {
                            provider: Box::new(provider),
                            rate_limiter: Some(
                                RateLimiterBuilder::default()
                                    .per_minute(80) // ~2500/month spread evenly
                                    .sliding_window()
                                    .build()
                                    .map_err(|e| LLMSpellError::Internal {
                                        message: format!("Failed to create rate limiter: {}", e),
                                        source: None,
                                    })?,
                            ),
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(wrapper) = wrapper {
                providers.insert(name.clone(), wrapper);
                info!("Initialized search provider: {}", name);
            } else if name != "duckduckgo" {
                warn!("Search provider {} not available (missing API key?)", name);
            }
        }

        Ok(Self {
            metadata: ComponentMetadata::new(
                "web-search-tool".to_string(),
                "Search the web using multiple providers with fallback support".to_string(),
            ),
            config,
            providers: Arc::new(Mutex::new(providers)),
        })
    }

    /// Create from environment configuration
    pub fn from_env() -> Result<Self> {
        Self::new(WebSearchConfig::from_env())
    }

    /// Perform search with fallback support
    async fn search_with_fallback(
        &self,
        query: &str,
        requested_provider: Option<&str>,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let providers = self.providers.lock().await;

        // Determine provider order
        let provider_chain = if let Some(requested) = requested_provider {
            // If specific provider requested, try it first
            let mut chain = vec![requested.to_string()];
            // Then add fallback chain excluding the requested one
            for fallback in &self.config.fallback_chain {
                if fallback != requested {
                    chain.push(fallback.clone());
                }
            }
            chain
        } else {
            // Use default provider first, then fallback chain
            let mut chain = vec![self.config.default_provider.clone()];
            for fallback in &self.config.fallback_chain {
                if fallback != &self.config.default_provider {
                    chain.push(fallback.clone());
                }
            }
            chain
        };

        // Try each provider in order
        let mut last_error = None;

        for provider_name in provider_chain {
            if let Some(wrapper) = providers.get(&provider_name) {
                // Check rate limit if applicable
                if let Some(rate_limiter) = &wrapper.rate_limiter {
                    match rate_limiter.try_acquire().await {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Rate limit exceeded for {}: {}", provider_name, e);
                            continue;
                        }
                    }
                }

                // Try the search
                info!("Searching with provider: {}", provider_name);
                match wrapper.provider.search(query, &options).await {
                    Ok(results) => {
                        if !results.is_empty() {
                            return Ok(results);
                        } else {
                            warn!("Provider {} returned no results", provider_name);
                        }
                    }
                    Err(e) => {
                        error!("Search failed with provider {}: {}", provider_name, e);
                        last_error = Some(e);
                    }
                }
            } else {
                warn!(
                    "Provider '{}' not found, skipping to next provider",
                    provider_name
                );
            }
        }

        // All providers failed
        Err(last_error.unwrap_or_else(|| LLMSpellError::Network {
            message: "All search providers failed or returned no results".to_string(),
            source: None,
        }))
    }

    /// Parse search type from string
    fn parse_search_type(type_str: Option<&str>) -> SearchType {
        match type_str {
            Some("news") => SearchType::News,
            Some("images") => SearchType::Images,
            _ => SearchType::Web,
        }
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
            "Search the web using multiple providers with automatic fallback".to_string(),
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
            description:
                "Search provider: google, brave, duckduckgo, serpapi, or serperdev (optional)"
                    .to_string(),
            required: false,
            default: Some(serde_json::json!(self.config.default_provider)),
        })
        .with_parameter(ParameterDef {
            name: "max_results".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum number of results to return (1-100)".to_string(),
            required: false,
            default: Some(serde_json::json!(self.config.max_results)),
        })
        .with_parameter(ParameterDef {
            name: "search_type".to_string(),
            param_type: ParameterType::String,
            description: "Type of search: web, news, or images".to_string(),
            required: false,
            default: Some(serde_json::json!("web")),
        })
        .with_parameter(ParameterDef {
            name: "safe_search".to_string(),
            param_type: ParameterType::Boolean,
            description: "Enable safe search filtering".to_string(),
            required: false,
            default: Some(serde_json::json!(self.config.safe_search)),
        })
        .with_parameter(ParameterDef {
            name: "language".to_string(),
            param_type: ParameterType::String,
            description: "Language code (e.g., en, es, fr)".to_string(),
            required: false,
            default: self.config.language.as_ref().map(|l| serde_json::json!(l)),
        })
    }
}

#[async_trait]
impl BaseAgent for WebSearchTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Get parameters
        let params = extract_parameters(&input)?;

        // Parse required parameters
        let query = extract_required_string(params, "input")?.to_string();

        // Parse optional parameters
        let provider = extract_optional_string(params, "provider");
        let max_results = params
            .get("max_results")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(self.config.max_results);
        let search_type = Self::parse_search_type(extract_optional_string(params, "search_type"));
        let safe_search = params
            .get("safe_search")
            .and_then(|v| v.as_bool())
            .unwrap_or(self.config.safe_search);
        let language = extract_optional_string(params, "language")
            .map(|s| s.to_string())
            .or_else(|| self.config.language.clone());

        debug!(
            "Executing web search: query='{}', provider={:?}, type={:?}, results={}",
            query, provider, search_type, max_results
        );

        // Build search options
        let options = SearchOptions {
            max_results,
            safe_search,
            language,
            search_type,
        };

        // Perform search with fallback
        let results = self.search_with_fallback(&query, provider, options).await?;

        // Create response
        let provider_used = results
            .first()
            .map(|r| r.provider.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let message = format!(
            "Found {} results for '{}' using {}",
            results.len(),
            query,
            provider_used
        );

        let response = ResponseBuilder::success("search")
            .with_message(&message)
            .with_result(serde_json::json!({
                "query": query,
                "provider": provider_used,
                "count": results.len(),
                "results": results,
            }))
            .build();

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
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
        let response = ResponseBuilder::error("search", error.to_string()).build();
        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }
}
