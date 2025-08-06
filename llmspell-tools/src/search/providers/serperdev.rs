//! ABOUTME: Serper.dev search provider implementation
//! ABOUTME: Modern Google Search API with excellent free tier (2,500 searches/month)

use super::{ProviderConfig, SearchOptions, SearchProvider, SearchResult, SearchType};
use async_trait::async_trait;
use llmspell_core::{LLMSpellError, Result};
use reqwest::{header, Client};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info};

/// Serper.dev provider
pub struct SerperDevProvider {
    client: Client,
    api_key: Option<String>,
}

impl SerperDevProvider {
    pub fn new(config: ProviderConfig) -> Self {
        // Create client with JSON content type
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
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

/// Serper.dev search response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SerperResponse {
    #[serde(rename = "searchParameters")]
    search_parameters: Option<SearchParameters>,
    organic: Option<Vec<OrganicResult>>,
    news: Option<Vec<NewsResult>>,
    images: Option<Vec<ImageResult>>,
    #[serde(rename = "knowledgeGraph")]
    knowledge_graph: Option<KnowledgeGraph>,
    #[serde(rename = "answerBox")]
    answer_box: Option<AnswerBox>,
    credits: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SearchParameters {
    q: String,
    #[serde(rename = "type")]
    search_type: Option<String>,
    num: Option<i32>,
    page: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OrganicResult {
    title: String,
    link: String,
    snippet: Option<String>,
    position: Option<i32>,
    date: Option<String>,
    #[serde(rename = "sitelinks")]
    site_links: Option<Vec<SiteLink>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SiteLink {
    title: String,
    link: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NewsResult {
    title: String,
    link: String,
    snippet: Option<String>,
    date: Option<String>,
    source: Option<String>,
    #[serde(rename = "imageUrl")]
    image_url: Option<String>,
    position: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ImageResult {
    title: Option<String>,
    #[serde(rename = "imageUrl")]
    image_url: String,
    link: String,
    source: Option<String>,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: Option<String>,
    position: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KnowledgeGraph {
    title: Option<String>,
    #[serde(rename = "type")]
    entity_type: Option<String>,
    description: Option<String>,
    #[serde(rename = "descriptionSource")]
    description_source: Option<String>,
    #[serde(rename = "descriptionLink")]
    description_link: Option<String>,
    attributes: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnswerBox {
    snippet: Option<String>,
    title: Option<String>,
    link: Option<String>,
}

#[async_trait]
impl SearchProvider for SerperDevProvider {
    fn name(&self) -> &'static str {
        "serperdev"
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn rate_limit(&self) -> Option<u32> {
        Some(2500) // 2,500 searches per month for free tier
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| LLMSpellError::Configuration {
                message: "Serper.dev API key not configured".to_string(),
                source: None,
            })?;

        // Determine endpoint based on search type
        let endpoint = match options.search_type {
            SearchType::News => "https://google.serper.dev/news",
            SearchType::Images => "https://google.serper.dev/images",
            SearchType::Web => "https://google.serper.dev/search",
        };

        // Build request body
        let mut body = json!({
            "q": query,
            "num": options.max_results.min(100),
        });

        if let Some(lang) = &options.language {
            body["hl"] = json!(lang);
        }

        if options.safe_search {
            body["safe"] = json!("active");
        }

        debug!("Searching Serper.dev for: {}", query);

        let response = self
            .client
            .post(endpoint)
            .header("X-API-KEY", api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| LLMSpellError::Network {
                message: format!("Serper.dev API request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(LLMSpellError::Network {
                message: format!("Serper.dev API returned status {status}: {error_body}"),
                source: None,
            });
        }

        let serper_response: SerperResponse =
            response.json().await.map_err(|e| LLMSpellError::Network {
                message: format!("Failed to parse Serper.dev response: {e}"),
                source: Some(Box::new(e)),
            })?;

        let mut results = Vec::new();
        let mut rank = 1;

        // Add answer box result if available (always first)
        if let Some(answer_box) = serper_response.answer_box {
            if let Some(snippet) = answer_box.snippet {
                results.push(SearchResult {
                    title: answer_box.title.unwrap_or_else(|| "Answer".to_string()),
                    url: answer_box.link.unwrap_or_else(|| query.to_string()),
                    snippet,
                    provider: format!("{} (answer)", self.name()),
                    rank,
                });
                rank += 1;
            }
        }

        // Process results based on search type
        match options.search_type {
            SearchType::Web => {
                if let Some(organic_results) = serper_response.organic {
                    for result in organic_results {
                        results.push(SearchResult {
                            title: result.title,
                            url: result.link,
                            snippet: result
                                .snippet
                                .unwrap_or_else(|| "No description available".to_string()),
                            provider: self.name().to_string(),
                            rank: {
                                #[allow(
                                    clippy::cast_possible_truncation,
                                    clippy::cast_sign_loss,
                                    clippy::cast_possible_wrap
                                )]
                                let rank_i32 = rank as i32;
                                #[allow(clippy::cast_sign_loss)]
                                let position = result.position.unwrap_or(rank_i32) as usize;
                                position
                            },
                        });
                        rank += 1;
                    }
                }
            }
            SearchType::News => {
                if let Some(news_results) = serper_response.news {
                    for result in news_results {
                        let mut snippet = result
                            .snippet
                            .unwrap_or_else(|| "No description available".to_string());
                        if let Some(source) = &result.source {
                            snippet = format!("{source} - {snippet}");
                        }

                        results.push(SearchResult {
                            title: result.title,
                            url: result.link,
                            snippet,
                            provider: self.name().to_string(),
                            rank: {
                                #[allow(
                                    clippy::cast_possible_truncation,
                                    clippy::cast_sign_loss,
                                    clippy::cast_possible_wrap
                                )]
                                let rank_i32 = rank as i32;
                                #[allow(clippy::cast_sign_loss)]
                                let position = result.position.unwrap_or(rank_i32) as usize;
                                position
                            },
                        });
                        rank += 1;
                    }
                }
            }
            SearchType::Images => {
                if let Some(image_results) = serper_response.images {
                    for result in image_results {
                        results.push(SearchResult {
                            title: result.title.unwrap_or_else(|| "Image".to_string()),
                            url: result.image_url,
                            snippet: result.source.unwrap_or_else(|| result.link.clone()),
                            provider: self.name().to_string(),
                            rank: {
                                #[allow(
                                    clippy::cast_possible_truncation,
                                    clippy::cast_sign_loss,
                                    clippy::cast_possible_wrap
                                )]
                                let rank_i32 = rank as i32;
                                #[allow(clippy::cast_sign_loss)]
                                let position = result.position.unwrap_or(rank_i32) as usize;
                                position
                            },
                        });
                        rank += 1;
                    }
                }
            }
        }

        // Log credits remaining if available
        if let Some(credits) = serper_response.credits {
            info!("Serper.dev credits remaining: {}", credits);
        }

        Ok(results)
    }

    fn metadata(&self) -> Value {
        serde_json::json!({
            "name": self.name(),
            "available": self.is_available(),
            "rate_limit": self.rate_limit(),
            "features": [
                "answer_box",
                "knowledge_graph",
                "site_links",
                "real_time_results",
                "structured_data"
            ]
        })
    }
}
