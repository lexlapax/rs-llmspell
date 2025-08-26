//! ABOUTME: `SerpApi` search provider implementation
//! ABOUTME: Supports multiple search engines through a unified API (Google, Bing, `DuckDuckGo`, etc.)

use super::{ProviderConfig, SearchOptions, SearchProvider, SearchResult, SearchType};
use async_trait::async_trait;
use llmspell_core::{LLMSpellError, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, info, warn};

/// `SerpApi` provider - supports multiple search engines
pub struct SerpApiProvider {
    client: Client,
    api_key: Option<String>,
    default_engine: String,
}

impl SerpApiProvider {
    #[must_use]
    pub fn new(config: ProviderConfig) -> Self {
        let default_engine = config
            .additional_config
            .get("default_engine")
            .and_then(|v| v.as_str())
            .unwrap_or("google")
            .to_string();

        Self {
            client: Client::new(),
            api_key: config.api_key,
            default_engine,
        }
    }
}

/// `SerpApi` response structure (varies by engine, this is common subset)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SerpApiResponse {
    search_metadata: Option<SearchMetadata>,
    search_parameters: Option<SearchParameters>,
    search_information: Option<SearchInformation>,

    // Different result types based on search type
    organic_results: Option<Vec<OrganicResult>>,
    news_results: Option<Vec<NewsResult>>,
    images_results: Option<Vec<ImageResult>>,

    // Error information
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SearchMetadata {
    id: Option<String>,
    status: Option<String>,
    json_endpoint: Option<String>,
    created_at: Option<String>,
    processed_at: Option<String>,
    total_time_taken: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SearchParameters {
    engine: Option<String>,
    q: Option<String>,
    num: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SearchInformation {
    total_results: Option<String>,
    time_taken_displayed: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OrganicResult {
    position: Option<i32>,
    title: Option<String>,
    link: Option<String>,
    snippet: Option<String>,
    displayed_link: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NewsResult {
    position: Option<i32>,
    title: Option<String>,
    link: Option<String>,
    snippet: Option<String>,
    source: Option<String>,
    date: Option<String>,
    thumbnail: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ImageResult {
    position: Option<i32>,
    title: Option<String>,
    link: Option<String>,
    original: Option<String>,
    thumbnail: Option<String>,
    source: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
}

#[async_trait]
impl SearchProvider for SerpApiProvider {
    fn name(&self) -> &'static str {
        "serpapi"
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn rate_limit(&self) -> Option<u32> {
        Some(100) // Base plan: 100 searches/month
    }

    #[allow(clippy::too_many_lines)]
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| LLMSpellError::Configuration {
                message: "SerpApi key not configured".to_string(),
                source: None,
            })?;

        let url = "https://serpapi.com/search";
        let num_results = options.max_results.min(100).to_string();
        let mut params = vec![
            ("api_key", api_key.as_str()),
            ("q", query),
            ("num", num_results.as_str()),
            ("engine", &self.default_engine),
        ];

        if options.safe_search {
            params.push(("safe", "active"));
        }

        if let Some(lang) = &options.language {
            params.push(("hl", lang));
        }

        // Handle different search types
        match options.search_type {
            SearchType::News => {
                params.push(("tbm", "nws")); // Google-style news search
            }
            SearchType::Images => {
                params.push(("tbm", "isch")); // Google-style image search
            }
            SearchType::Web => {} // Default
        }

        debug!(
            "Searching via SerpApi ({}) for: {}",
            self.default_engine, query
        );

        let response = self
            .client
            .get(url)
            .query(&params)
            .send()
            .await
            .map_err(|e| LLMSpellError::Network {
                message: format!("SerpApi request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(LLMSpellError::Network {
                message: format!("SerpApi returned status {status}: {error_body}"),
                source: None,
            });
        }

        let serpapi_response: SerpApiResponse =
            response.json().await.map_err(|e| LLMSpellError::Network {
                message: format!("Failed to parse SerpApi response: {e}"),
                source: Some(Box::new(e)),
            })?;

        // Check for API errors
        if let Some(error) = serpapi_response.error {
            return Err(LLMSpellError::Network {
                message: format!("SerpApi error: {error}"),
                source: None,
            });
        }

        let mut results = Vec::new();

        // Process results based on search type
        match options.search_type {
            SearchType::Web => {
                if let Some(organic_results) = serpapi_response.organic_results {
                    for result in organic_results {
                        if let (Some(title), Some(link)) = (result.title, result.link) {
                            results.push(SearchResult {
                                title,
                                url: link,
                                snippet: result
                                    .snippet
                                    .unwrap_or_else(|| "No description available".to_string()),
                                provider: format!("{} ({})", self.name(), self.default_engine),
                                rank: {
                                    #[allow(
                                        clippy::cast_possible_truncation,
                                        clippy::cast_sign_loss,
                                        clippy::cast_possible_wrap
                                    )]
                                    let fallback_rank = results.len() as i32 + 1;
                                    #[allow(clippy::cast_sign_loss)]
                                    let position =
                                        result.position.unwrap_or(fallback_rank) as usize;
                                    position
                                },
                            });
                        }
                    }
                }
            }
            SearchType::News => {
                if let Some(news_results) = serpapi_response.news_results {
                    for result in news_results {
                        if let (Some(title), Some(link)) = (result.title, result.link) {
                            results.push(SearchResult {
                                title,
                                url: link,
                                snippet: result
                                    .snippet
                                    .unwrap_or_else(|| "No description available".to_string()),
                                provider: format!("{} ({})", self.name(), self.default_engine),
                                rank: {
                                    #[allow(
                                        clippy::cast_possible_truncation,
                                        clippy::cast_sign_loss,
                                        clippy::cast_possible_wrap
                                    )]
                                    let fallback_rank = results.len() as i32 + 1;
                                    #[allow(clippy::cast_sign_loss)]
                                    let position =
                                        result.position.unwrap_or(fallback_rank) as usize;
                                    position
                                },
                            });
                        }
                    }
                }
            }
            SearchType::Images => {
                if let Some(images_results) = serpapi_response.images_results {
                    for result in images_results {
                        if let Some(original) = result.original {
                            results.push(SearchResult {
                                title: result.title.unwrap_or_else(|| "Image".to_string()),
                                url: original,
                                snippet: result
                                    .source
                                    .unwrap_or_else(|| "Image result".to_string()),
                                provider: format!("{} ({})", self.name(), self.default_engine),
                                rank: {
                                    #[allow(
                                        clippy::cast_possible_truncation,
                                        clippy::cast_sign_loss,
                                        clippy::cast_possible_wrap
                                    )]
                                    let fallback_rank = results.len() as i32 + 1;
                                    #[allow(clippy::cast_sign_loss)]
                                    let position =
                                        result.position.unwrap_or(fallback_rank) as usize;
                                    position
                                },
                            });
                        }
                    }
                }
            }
        }

        if let Some(metadata) = serpapi_response.search_metadata {
            if let Some(time) = metadata.total_time_taken {
                info!("SerpApi search completed in {:.2}s", time);
            }
        }

        if results.is_empty() {
            warn!("SerpApi returned no results for query: {}", query);
        }

        Ok(results)
    }

    fn metadata(&self) -> Value {
        let mut meta = serde_json::json!({
            "name": self.name(),
            "available": self.is_available(),
            "rate_limit": self.rate_limit(),
            "default_engine": self.default_engine,
        });

        // List supported engines
        meta["supported_engines"] =
            serde_json::json!(["google", "bing", "duckduckgo", "yahoo", "yandex", "baidu"]);

        meta
    }
}
