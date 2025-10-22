//! ABOUTME: Tavily AI Search API provider implementation
//! ABOUTME: Requires API key from <https://tavily.com> - AI-optimized search for RAG/LLM workflows
use tracing::instrument;

use super::{ProviderConfig, SearchOptions, SearchProvider, SearchResult, SearchType};
use async_trait::async_trait;
use llmspell_core::{LLMSpellError, Result};
use llmspell_kernel::runtime::create_io_bound_resource;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Tavily Search provider - AI-optimized for RAG/LLM workflows
pub struct TavilySearchProvider {
    client: Client,
    api_key: Option<String>,
}

impl TavilySearchProvider {
    #[must_use]
    pub fn new(config: ProviderConfig) -> Self {
        // Create client with custom headers
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = create_io_bound_resource(|| {
            Client::builder()
                .default_headers(headers)
                .build()
                .unwrap_or_else(|_| Client::new())
        });

        Self {
            client,
            api_key: config.api_key,
        }
    }
}

/// Tavily Search API request
#[derive(Debug, Serialize)]
struct TavilySearchRequest {
    api_key: String,
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_depth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_answer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_raw_content: Option<bool>,
}

/// Tavily Search API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TavilySearchResponse {
    results: Vec<TavilyResult>,
    #[serde(default)]
    answer: Option<String>,
    #[serde(default)]
    query: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f32,
    #[serde(default)]
    published_date: Option<String>,
    #[serde(default)]
    raw_content: Option<String>,
}

#[async_trait]
impl SearchProvider for TavilySearchProvider {
    fn name(&self) -> &'static str {
        "tavily"
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn rate_limit(&self) -> Option<u32> {
        Some(1000) // 1000 queries per month for free tier
    }

    #[instrument(skip(self))]
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| LLMSpellError::Configuration {
                message: "Tavily API key not configured".to_string(),
                source: None,
            })?;

        // Tavily only supports web search (optimized for LLM/RAG)
        if options.search_type != SearchType::Web {
            warn!(
                "Tavily only supports web search, ignoring search_type: {:?}",
                options.search_type
            );
        }

        let url = "https://api.tavily.com/search";

        // Tavily search depth: "basic" (faster) or "advanced" (more thorough)
        // Use "advanced" for better quality RAG results
        let search_depth = Some("advanced".to_string());

        let request_body = TavilySearchRequest {
            api_key: api_key.clone(),
            query: query.to_string(),
            max_results: Some(options.max_results.min(20)), // Tavily recommends max 20
            search_depth,
            include_answer: Some(true), // Get AI-generated answer if available
            include_raw_content: Some(false), // Don't need full page content
        };

        debug!("Searching Tavily for: {}", query);

        let response = self
            .client
            .post(url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LLMSpellError::Network {
                message: format!("Tavily API request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(LLMSpellError::Network {
                message: format!("Tavily API returned status {status}: {error_body}"),
                source: None,
            });
        }

        let tavily_response: TavilySearchResponse =
            response.json().await.map_err(|e| LLMSpellError::Network {
                message: format!("Failed to parse Tavily response: {e}"),
                source: Some(Box::new(e)),
            })?;

        // Log AI-generated answer if available (useful for debugging)
        if let Some(answer) = &tavily_response.answer {
            info!("Tavily AI answer: {}", answer);
        }

        // Convert Tavily results to SearchResult format
        let results: Vec<SearchResult> = tavily_response
            .results
            .into_iter()
            .enumerate()
            .map(|(rank, result)| SearchResult {
                title: result.title,
                url: result.url,
                snippet: result.content, // Tavily returns "content" optimized for LLMs
                provider: self.name().to_string(),
                rank: rank + 1,
            })
            .collect();

        info!(
            "Tavily search returned {} results for query: '{}'",
            results.len(),
            query
        );

        Ok(results)
    }
}
