//! ABOUTME: Brave Search API provider implementation
//! ABOUTME: Requires API key from <https://brave.com/search/api/>

use super::{ProviderConfig, SearchOptions, SearchProvider, SearchResult, SearchType};
use async_trait::async_trait;
use llmspell_core::{LLMSpellError, Result};
use reqwest::{header, Client};
use serde::Deserialize;
use tracing::{debug, info};

/// Brave Search provider
pub struct BraveSearchProvider {
    client: Client,
    api_key: Option<String>,
}

impl BraveSearchProvider {
    pub fn new(config: ProviderConfig) -> Self {
        // Create client with custom headers
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            api_key: config.api_key,
        }
    }
}

/// Brave Search API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BraveSearchResponse {
    #[serde(rename = "type")]
    response_type: Option<String>,
    web: Option<WebResults>,
    news: Option<NewsResults>,
    images: Option<ImageResults>,
    query: Option<QueryInfo>,
}

#[derive(Debug, Deserialize)]
struct WebResults {
    results: Vec<WebResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WebResult {
    title: String,
    url: String,
    description: Option<String>,
    age: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsResults {
    results: Vec<NewsResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NewsResult {
    title: String,
    url: String,
    description: Option<String>,
    age: Option<String>,
    #[serde(rename = "meta_url")]
    meta_url: Option<MetaUrl>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MetaUrl {
    favicon: Option<String>,
    hostname: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageResults {
    results: Vec<ImageResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ImageResult {
    title: Option<String>,
    url: String,
    source: Option<String>,
    thumbnail: Option<Thumbnail>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Thumbnail {
    src: String,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct QueryInfo {
    original: String,
    altered: Option<String>,
    spellcheck_off: Option<bool>,
}

#[async_trait]
impl SearchProvider for BraveSearchProvider {
    fn name(&self) -> &'static str {
        "brave"
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn rate_limit(&self) -> Option<u32> {
        Some(2000) // 2000 queries per month for free tier
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| LLMSpellError::Configuration {
                message: "Brave API key not configured".to_string(),
                source: None,
            })?;

        let url = "https://api.search.brave.com/res/v1/web/search";
        let count = options.max_results.min(20).to_string();
        let mut params = vec![
            ("q", query),
            ("count", count.as_str()), // Brave limits to 20
        ];

        if options.safe_search {
            params.push(("safesearch", "strict"));
        }

        if let Some(lang) = &options.language {
            params.push(("search_lang", lang));
        }

        // Handle different search types
        let _search_type_param = match options.search_type {
            SearchType::News => "news",
            SearchType::Images => "images",
            SearchType::Web => "search",
        };

        // For news and images, use different endpoints
        let final_url = match options.search_type {
            SearchType::News => "https://api.search.brave.com/res/v1/news/search",
            SearchType::Images => "https://api.search.brave.com/res/v1/images/search",
            SearchType::Web => url,
        };

        debug!("Searching Brave for: {}", query);

        let response = self
            .client
            .get(final_url)
            .header("X-Subscription-Token", api_key)
            .query(&params)
            .send()
            .await
            .map_err(|e| LLMSpellError::Network {
                message: format!("Brave API request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(LLMSpellError::Network {
                message: format!("Brave API returned status {status}: {error_body}"),
                source: None,
            });
        }

        let brave_response: BraveSearchResponse =
            response.json().await.map_err(|e| LLMSpellError::Network {
                message: format!("Failed to parse Brave response: {e}"),
                source: Some(Box::new(e)),
            })?;

        let mut results = Vec::new();

        // Process results based on search type
        match options.search_type {
            SearchType::Web => {
                if let Some(web_results) = brave_response.web {
                    for (rank, result) in web_results.results.into_iter().enumerate() {
                        results.push(SearchResult {
                            title: result.title,
                            url: result.url,
                            snippet: result
                                .description
                                .unwrap_or_else(|| "No description available".to_string()),
                            provider: self.name().to_string(),
                            rank: rank + 1,
                        });
                    }
                }
            }
            SearchType::News => {
                if let Some(news_results) = brave_response.news {
                    for (rank, result) in news_results.results.into_iter().enumerate() {
                        results.push(SearchResult {
                            title: result.title,
                            url: result.url,
                            snippet: result
                                .description
                                .unwrap_or_else(|| "No description available".to_string()),
                            provider: self.name().to_string(),
                            rank: rank + 1,
                        });
                    }
                }
            }
            SearchType::Images => {
                if let Some(image_results) = brave_response.images {
                    for (rank, result) in image_results.results.into_iter().enumerate() {
                        results.push(SearchResult {
                            title: result.title.unwrap_or_else(|| "Image".to_string()),
                            url: result.url,
                            snippet: result.source.unwrap_or_else(|| "Image result".to_string()),
                            provider: self.name().to_string(),
                            rank: rank + 1,
                        });
                    }
                }
            }
        }

        if let Some(query_info) = brave_response.query {
            if let Some(altered) = query_info.altered {
                info!(
                    "Brave altered query from '{}' to '{}'",
                    query_info.original, altered
                );
            }
        }

        Ok(results)
    }
}
