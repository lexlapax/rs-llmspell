//! ABOUTME: `DuckDuckGo` search provider implementation
//! ABOUTME: Uses HTML scraping for actual web results (no API key required)
//!
//! Primary method: HTML scraping of `html.duckduckgo.com`
//! Fallback: Instant Answer API for knowledge queries
use tracing::instrument;

use super::{SearchOptions, SearchProvider, SearchResult, SearchType};
use async_trait::async_trait;
use llmspell_core::{LLMSpellError, Result};
use llmspell_kernel::runtime::create_io_bound_resource;
use reqwest::Client;
use scraper::{Html, Selector};
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
        Some(300) // 5 req/sec (conservative, unofficial limit is ~20/sec)
    }

    #[allow(clippy::too_many_lines)]
    #[instrument(skip(self))]
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        // DuckDuckGo HTML scraping only supports web search
        if options.search_type != SearchType::Web {
            return Err(LLMSpellError::Validation {
                message: "DuckDuckGo only supports web search".to_string(),
                field: Some("search_type".to_string()),
            });
        }

        debug!("Searching DuckDuckGo (HTML) for: {}", query);

        // Try HTML scraping first (actual web results)
        match self.search_html(query, options).await {
            Ok(results) if !results.is_empty() => {
                info!(
                    "DuckDuckGo HTML scraping returned {} results",
                    results.len()
                );
                return Ok(results);
            }
            Ok(_) => {
                warn!("DuckDuckGo HTML scraping returned no results, trying Instant Answer API");
            }
            Err(e) => {
                warn!(
                    "DuckDuckGo HTML scraping failed: {}, trying Instant Answer API",
                    e
                );
            }
        }

        // Fallback to Instant Answer API (knowledge queries)
        self.search_instant_answer(query, options).await
    }
}

impl DuckDuckGoProvider {
    /// Search using HTML scraping (primary method for web results)
    async fn search_html(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        debug!("Fetching DuckDuckGo HTML: {}", url);

        // Use browser-like headers to avoid CAPTCHA
        let response = self
            .client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://duckduckgo.com/")
            .send()
            .await
            .map_err(|e| LLMSpellError::Network {
                message: format!("DuckDuckGo HTML request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        if !response.status().is_success() {
            return Err(LLMSpellError::Network {
                message: format!("DuckDuckGo returned status: {}", response.status()),
                source: None,
            });
        }

        let html_text = response.text().await.map_err(|e| LLMSpellError::Network {
            message: format!("Failed to read DuckDuckGo HTML: {e}"),
            source: Some(Box::new(e)),
        })?;

        // Check for CAPTCHA
        if html_text.contains("anomaly-modal") || html_text.contains("captcha") {
            return Err(LLMSpellError::Network {
                message: "DuckDuckGo CAPTCHA detected - anti-bot protection triggered".to_string(),
                source: None,
            });
        }

        self.parse_html_results(&html_text, options.max_results)
    }

    /// Parse HTML and extract search results
    fn parse_html_results(&self, html: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        let document = Html::parse_document(html);

        // Selectors for DuckDuckGo HTML structure
        let result_selector = Selector::parse(
            ".result.results_links.results_links_deep.web-result",
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create result selector: {e:?}"),
            source: None,
        })?;

        let link_selector =
            Selector::parse(".result__a").map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create link selector: {e:?}"),
                source: None,
            })?;

        let snippet_selector =
            Selector::parse(".result__snippet").map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create snippet selector: {e:?}"),
                source: None,
            })?;

        let mut results = Vec::new();
        let mut rank = 1;

        for result_elem in document.select(&result_selector).take(max_results) {
            // Extract title and URL from link
            let link = result_elem.select(&link_selector).next();
            if let Some(link_elem) = link {
                let title = link_elem
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                let url = link_elem.value().attr("href").unwrap_or("");

                // Extract snippet
                let snippet = result_elem
                    .select(&snippet_selector)
                    .next()
                    .map(|s| s.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default();

                // DuckDuckGo uses relative URLs like /l/?uddg=...
                // Need to extract actual URL from redirect parameter
                let actual_url = if url.starts_with("/l/?") {
                    // Extract uddg parameter which contains the actual URL
                    url.split("uddg=")
                        .nth(1)
                        .and_then(|s| s.split('&').next())
                        .and_then(|encoded| urlencoding::decode(encoded).ok())
                        .map(|decoded| decoded.to_string())
                        .unwrap_or_else(|| url.to_string())
                } else {
                    url.to_string()
                };

                if !title.is_empty() && !actual_url.is_empty() {
                    results.push(SearchResult {
                        title,
                        url: actual_url,
                        snippet,
                        provider: self.name().to_string(),
                        rank,
                    });
                    rank += 1;
                }
            }
        }

        debug!("Parsed {} results from DuckDuckGo HTML", results.len());
        Ok(results)
    }

    /// Search using Instant Answer API (fallback for knowledge queries)
    async fn search_instant_answer(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
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

        debug!("Searching DuckDuckGo Instant Answer API for: {}", query);

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
            warn!(
                "DuckDuckGo API returned empty response for query: {}",
                query
            );
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
            warn!("DuckDuckGo API returned no results for query: {}", query);
        }

        Ok(results)
    }
}
