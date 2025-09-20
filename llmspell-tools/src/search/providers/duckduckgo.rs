//! ABOUTME: `DuckDuckGo` search provider implementation
//! ABOUTME: Uses `DuckDuckGo` Instant Answer API (no API key required)
use tracing::instrument;

use super::{SearchOptions, SearchProvider, SearchResult, SearchType};
use async_trait::async_trait;
use llmspell_core::{LLMSpellError, Result};
use llmspell_kernel::runtime::create_io_bound_resource;
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, info, warn};

/// `DuckDuckGo` search provider
pub struct DuckDuckGoProvider {
    client: Client,
}

impl DuckDuckGoProvider {
    /// Create a new `DuckDuckGo` search provider with global runtime
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: create_io_bound_resource(Client::new),
        }
    }
}

impl Default for DuckDuckGoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchProvider for DuckDuckGoProvider {
    fn name(&self) -> &'static str {
        "duckduckgo"
    }

    fn is_available(&self) -> bool {
        true // No API key required
    }

    fn rate_limit(&self) -> Option<u32> {
        None // No official rate limit, but we should be respectful
    }

    #[allow(clippy::too_many_lines)]
    #[instrument(skip(self))]
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        // DuckDuckGo Instant Answer API only supports web search
        if options.search_type != SearchType::Web {
            return Err(LLMSpellError::Validation {
                message: "DuckDuckGo only supports web search".to_string(),
                field: Some("search_type".to_string()),
            });
        }

        let url = "https://api.duckduckgo.com/";
        let mut params = vec![
            ("q", query),
            ("format", "json"),
            ("no_html", "1"),
            ("skip_disambig", "1"),
        ];

        if options.safe_search {
            params.push(("safe_search", "1"));
        }

        debug!("Searching DuckDuckGo for: {}", query);
        info!("DuckDuckGo search query: '{}', params: {:?}", query, params);

        let response = self
            .client
            .get(url)
            .query(&params)
            .send()
            .await
            .map_err(|e| LLMSpellError::Network {
                message: format!("DuckDuckGo API request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        if !response.status().is_success() {
            return Err(LLMSpellError::Network {
                message: format!("DuckDuckGo API returned status: {}", response.status()),
                source: None,
            });
        }

        let response_text = response.text().await.map_err(|e| LLMSpellError::Network {
            message: format!("Failed to get DuckDuckGo response text: {e}"),
            source: Some(Box::new(e)),
        })?;

        if response_text.is_empty() {
            warn!("DuckDuckGo returned empty response for query: {}", query);
            return Ok(vec![]);
        }

        let response_json: Value = serde_json::from_str(&response_text).map_err(|e| {
            warn!(
                "Failed to parse DuckDuckGo JSON. Response text: {}",
                response_text
            );
            LLMSpellError::Network {
                message: format!("Failed to parse DuckDuckGo response: {e}"),
                source: Some(Box::new(e)),
            }
        })?;

        let mut results = Vec::new();
        let mut rank = 1;

        // Add abstract result if available
        if let (Some(text), Some(url)) = (
            response_json.get("Abstract").and_then(|v| v.as_str()),
            response_json.get("AbstractURL").and_then(|v| v.as_str()),
        ) {
            if !text.is_empty() {
                results.push(SearchResult {
                    title: response_json
                        .get("AbstractSource")
                        .and_then(|v| v.as_str())
                        .unwrap_or("DuckDuckGo Abstract")
                        .to_string(),
                    url: url.to_string(),
                    snippet: text.to_string(),
                    provider: self.name().to_string(),
                    rank,
                });
                rank += 1;
            }
        }

        // Add instant answer results
        if let Some(instant_results) = response_json.get("Results").and_then(|v| v.as_array()) {
            for result in instant_results
                .iter()
                .take(options.max_results.saturating_sub(results.len()))
            {
                if let (Some(text), Some(url)) = (
                    result.get("Text").and_then(|v| v.as_str()),
                    result.get("FirstURL").and_then(|v| v.as_str()),
                ) {
                    results.push(SearchResult {
                        title: text.to_string(),
                        url: url.to_string(),
                        snippet: text.to_string(),
                        provider: self.name().to_string(),
                        rank,
                    });
                    rank += 1;
                }
            }
        }

        // Add related topics
        if let Some(topics) = response_json
            .get("RelatedTopics")
            .and_then(|v| v.as_array())
        {
            for topic in topics
                .iter()
                .take(options.max_results.saturating_sub(results.len()))
            {
                if let (Some(text), Some(url)) = (
                    topic.get("Text").and_then(|v| v.as_str()),
                    topic.get("FirstURL").and_then(|v| v.as_str()),
                ) {
                    results.push(SearchResult {
                        title: text.split('.').next().unwrap_or(text).to_string(),
                        url: url.to_string(),
                        snippet: text.to_string(),
                        provider: self.name().to_string(),
                        rank,
                    });
                    rank += 1;
                }
            }
        }

        if results.is_empty() {
            warn!("DuckDuckGo returned no results for query: {}", query);
        }

        Ok(results)
    }
}
