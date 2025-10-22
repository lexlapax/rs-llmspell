//! ABOUTME: Bing Search API provider implementation
//! ABOUTME: Requires API key from Microsoft Azure <https://azure.microsoft.com/en-us/services/cognitive-services/bing-web-search-api/>
use tracing::instrument;

use super::{ProviderConfig, SearchOptions, SearchProvider, SearchResult, SearchType};
use async_trait::async_trait;
use llmspell_core::{LLMSpellError, Result};
use llmspell_kernel::runtime::create_io_bound_resource;
use reqwest::{header, Client};
use serde::Deserialize;
use tracing::{debug, info, warn};

/// Bing Search provider
pub struct BingSearchProvider {
    client: Client,
    api_key: Option<String>,
}

impl BingSearchProvider {
    #[must_use]
    pub fn new(config: ProviderConfig) -> Self {
        // Create client with custom headers
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
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

/// Bing Search API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BingSearchResponse {
    #[serde(rename = "webPages")]
    web_pages: Option<WebPages>,
    news: Option<NewsValues>,
    images: Option<ImagesValues>,
    #[serde(rename = "queryContext")]
    query_context: Option<QueryContext>,
}

#[derive(Debug, Deserialize)]
struct WebPages {
    value: Vec<WebPage>,
    #[serde(rename = "totalEstimatedMatches")]
    #[allow(dead_code)]
    total_estimated_matches: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WebPage {
    name: String,
    url: String,
    snippet: String,
    #[serde(rename = "dateLastCrawled")]
    date_last_crawled: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsValues {
    value: Vec<NewsArticle>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NewsArticle {
    name: String,
    url: String,
    description: String,
    #[serde(rename = "datePublished")]
    date_published: Option<String>,
    provider: Option<Vec<NewsProvider>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NewsProvider {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImagesValues {
    value: Vec<BingImage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BingImage {
    name: String,
    #[serde(rename = "contentUrl")]
    content_url: String,
    #[serde(rename = "hostPageUrl")]
    host_page_url: String,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct QueryContext {
    #[serde(rename = "originalQuery")]
    original_query: String,
    #[serde(rename = "alteredQuery")]
    altered_query: Option<String>,
}

#[async_trait]
impl SearchProvider for BingSearchProvider {
    fn name(&self) -> &'static str {
        "bing"
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn rate_limit(&self) -> Option<u32> {
        Some(1000) // 1000 queries per month for free tier
    }

    #[allow(clippy::too_many_lines)]
    #[instrument(skip(self))]
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| LLMSpellError::Configuration {
                message: "Bing API key not configured".to_string(),
                source: None,
            })?;

        // Build URL based on search type
        let base_url = match options.search_type {
            SearchType::Web => "https://api.bing.microsoft.com/v7.0/search",
            SearchType::News => "https://api.bing.microsoft.com/v7.0/news/search",
            SearchType::Images => "https://api.bing.microsoft.com/v7.0/images/search",
        };

        let count = options.max_results.min(50).to_string(); // Bing allows up to 50

        // Declare market outside if block for lifetime
        let market: String;
        let mut params = vec![("q", query), ("count", count.as_str())];

        if options.safe_search {
            params.push(("safeSearch", "Strict"));
        } else {
            params.push(("safeSearch", "Off"));
        }

        if let Some(lang) = &options.language {
            // Bing uses market codes like "en-US"
            market = format!("{}-US", lang.to_uppercase());
            params.push(("mkt", &market));
        }

        debug!("Searching Bing for: {}", query);

        let response = self
            .client
            .get(base_url)
            .header("Ocp-Apim-Subscription-Key", api_key)
            .query(&params)
            .send()
            .await
            .map_err(|e| LLMSpellError::Network {
                message: format!("Bing API request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(LLMSpellError::Network {
                message: format!("Bing API returned status {status}: {error_body}"),
                source: None,
            });
        }

        let bing_response: BingSearchResponse =
            response.json().await.map_err(|e| LLMSpellError::Network {
                message: format!("Failed to parse Bing response: {e}"),
                source: Some(Box::new(e)),
            })?;

        let mut results = Vec::new();

        // Process results based on search type
        match options.search_type {
            SearchType::Web => {
                if let Some(web_pages) = bing_response.web_pages {
                    for (rank, page) in web_pages.value.into_iter().enumerate() {
                        results.push(SearchResult {
                            title: page.name,
                            url: page.url,
                            snippet: page.snippet,
                            provider: self.name().to_string(),
                            rank: rank + 1,
                        });
                    }
                }
            }
            SearchType::News => {
                if let Some(news) = bing_response.news {
                    for (rank, article) in news.value.into_iter().enumerate() {
                        results.push(SearchResult {
                            title: article.name,
                            url: article.url,
                            snippet: article.description,
                            provider: self.name().to_string(),
                            rank: rank + 1,
                        });
                    }
                }
            }
            SearchType::Images => {
                if let Some(images) = bing_response.images {
                    for (rank, image) in images.value.into_iter().enumerate() {
                        results.push(SearchResult {
                            title: image.name,
                            url: image.content_url,
                            snippet: format!("Image from {}", image.host_page_url),
                            provider: self.name().to_string(),
                            rank: rank + 1,
                        });
                    }
                }
            }
        }

        // Log query alteration if it occurred
        if let Some(query_context) = bing_response.query_context {
            if let Some(altered) = query_context.altered_query {
                if altered != query_context.original_query {
                    info!(
                        "Bing altered query from '{}' to '{}'",
                        query_context.original_query, altered
                    );
                }
            }
        }

        if results.is_empty() {
            warn!("Bing returned no results for query: {}", query);
        } else {
            info!(
                "Bing search returned {} results for query: '{}'",
                results.len(),
                query
            );
        }

        Ok(results)
    }
}
