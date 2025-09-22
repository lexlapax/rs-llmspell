//! ABOUTME: Google Custom Search API provider implementation
//! ABOUTME: Requires API key and Search Engine ID from Google Cloud Console
use tracing::instrument;

use super::{ProviderConfig, SearchOptions, SearchProvider, SearchResult, SearchType};
use async_trait::async_trait;
use llmspell_core::{LLMSpellError, Result};
use llmspell_kernel::runtime::create_io_bound_resource;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, info};

/// Google Custom Search provider
pub struct GoogleSearchProvider {
    client: Client,
    api_key: Option<String>,
    search_engine_id: Option<String>,
}

impl GoogleSearchProvider {
    #[must_use]
    pub fn new(config: ProviderConfig) -> Self {
        let search_engine_id = config
            .additional_config
            .get("search_engine_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        Self {
            client: create_io_bound_resource(Client::new),
            api_key: config.api_key,
            search_engine_id,
        }
    }
}

/// Google Custom Search API response
#[derive(Debug, Deserialize)]
struct GoogleSearchResponse {
    items: Option<Vec<GoogleSearchItem>>,
    #[serde(rename = "searchInformation")]
    search_information: Option<SearchInformation>,
}

#[derive(Debug, Deserialize)]
struct GoogleSearchItem {
    title: String,
    link: String,
    snippet: Option<String>,
    #[serde(rename = "htmlSnippet")]
    html_snippet: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchInformation {
    #[serde(rename = "totalResults")]
    total_results: Option<String>,
    #[serde(rename = "searchTime")]
    search_time: Option<f64>,
}

#[async_trait]
impl SearchProvider for GoogleSearchProvider {
    fn name(&self) -> &'static str {
        "google"
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some() && self.search_engine_id.is_some()
    }

    fn rate_limit(&self) -> Option<u32> {
        Some(100) // 100 queries per day for free tier
    }

    #[instrument(skip(self))]
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| LLMSpellError::Configuration {
                message: "Google API key not configured".to_string(),
                source: None,
            })?;

        let search_engine_id =
            self.search_engine_id
                .as_ref()
                .ok_or_else(|| LLMSpellError::Configuration {
                    message: "Google Search Engine ID not configured".to_string(),
                    source: None,
                })?;

        let url = "https://www.googleapis.com/customsearch/v1";
        let num_results = options.max_results.min(10).to_string();
        let mut params = vec![
            ("key", api_key.as_str()),
            ("cx", search_engine_id.as_str()),
            ("q", query),
            ("num", num_results.as_str()), // Google limits to 10 per request
        ];

        if options.safe_search {
            params.push(("safe", "active"));
        }

        if let Some(lang) = &options.language {
            params.push(("lr", lang));
        }

        // Handle different search types
        let news_query;
        match options.search_type {
            SearchType::Images => params.push(("searchType", "image")),
            SearchType::News => {
                // Google Custom Search doesn't have a direct news filter
                // We can add "news" to the query as a workaround
                news_query = format!("{query} news");
                params[2] = ("q", news_query.as_str());
            }
            SearchType::Web => {} // Default
        }

        debug!("Searching Google for: {}", query);

        let response = self
            .client
            .get(url)
            .query(&params)
            .send()
            .await
            .map_err(|e| LLMSpellError::Network {
                message: format!("Google API request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(LLMSpellError::Network {
                message: format!("Google API returned status {status}: {error_body}"),
                source: None,
            });
        }

        let google_response: GoogleSearchResponse =
            response.json().await.map_err(|e| LLMSpellError::Network {
                message: format!("Failed to parse Google response: {e}"),
                source: Some(Box::new(e)),
            })?;

        let mut results = Vec::new();

        if let Some(items) = google_response.items {
            for (rank, item) in items.into_iter().enumerate() {
                let snippet = item
                    .snippet
                    .or(item.html_snippet)
                    .unwrap_or_else(|| "No description available".to_string());

                results.push(SearchResult {
                    title: item.title,
                    url: item.link,
                    snippet,
                    provider: self.name().to_string(),
                    rank: rank + 1,
                });
            }
        }

        if let Some(search_info) = google_response.search_information {
            info!(
                "Google search completed in {:.2}s, total results: {}",
                search_info.search_time.unwrap_or(0.0),
                search_info.total_results.unwrap_or_default()
            );
        }

        Ok(results)
    }

    fn metadata(&self) -> Value {
        let mut meta = serde_json::json!({
            "name": self.name(),
            "available": self.is_available(),
            "rate_limit": self.rate_limit(),
        });

        if self.is_available() {
            meta["search_engine_id"] =
                Value::String(self.search_engine_id.as_ref().unwrap().clone());
        }

        meta
    }
}
