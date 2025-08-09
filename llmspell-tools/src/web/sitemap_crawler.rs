//! ABOUTME: Sitemap crawler tool for parsing and analyzing sitemaps
//! ABOUTME: Parses XML sitemaps and extracts URLs with metadata

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, Result,
};
use llmspell_utils::{
    error_builders::llmspell::{component_error, validation_error},
    params::{
        extract_optional_bool, extract_optional_u64, extract_parameters, extract_required_string,
    },
    response::ResponseBuilder,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::pin::Pin;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapCrawlerTool {
    metadata: ComponentMetadata,
}

impl Default for SitemapCrawlerTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SitemapCrawlerTool {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "sitemap-crawler".to_string(),
                "Parse and analyze website sitemaps".to_string(),
            ),
        }
    }
}

#[async_trait]
impl Tool for SitemapCrawlerTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Sitemap URL to parse".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "follow_sitemaps".to_string(),
            param_type: ParameterType::Boolean,
            description: "Follow sitemap index files to parse child sitemaps".to_string(),
            required: false,
            default: Some(json!(true)),
        })
        .with_parameter(ParameterDef {
            name: "max_urls".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum number of URLs to return (default: 1000)".to_string(),
            required: false,
            default: Some(json!(1000)),
        })
        .with_parameter(ParameterDef {
            name: "timeout".to_string(),
            param_type: ParameterType::Number,
            description: "Request timeout in seconds (default: 30)".to_string(),
            required: false,
            default: Some(json!(30)),
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Data
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }
}

#[async_trait]
impl BaseAgent for SitemapCrawlerTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("SitemapCrawler error: {error}")))
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params = extract_parameters(&input)?;
        let url = extract_required_string(params, "input")?;
        let follow_sitemaps = extract_optional_bool(params, "follow_sitemaps").unwrap_or(true);
        #[allow(clippy::cast_possible_truncation)]
        let max_urls = extract_optional_u64(params, "max_urls").unwrap_or(1000) as usize;
        let timeout_secs = extract_optional_u64(params, "timeout").unwrap_or(30);

        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(validation_error(
                "URL must start with http:// or https://",
                Some("input".to_string()),
            ));
        }

        let mut all_urls = Vec::new();
        let mut visited_sitemaps = HashSet::new();
        let mut stats = SitemapStats::new();

        let options = CrawlOptions {
            follow_sitemaps,
            max_urls,
            timeout_secs,
        };

        self.crawl_sitemap(
            url,
            &options,
            &mut all_urls,
            &mut visited_sitemaps,
            &mut stats,
        )
        .await?;

        let result = json!({
            "sitemap_url": url,
            "urls_found": all_urls.len(),
            "max_urls": max_urls,
            "follow_sitemaps": follow_sitemaps,
            "urls": all_urls,
            "stats": {
                "sitemaps_processed": stats.sitemaps_processed,
                "index_files_found": stats.index_files_found,
                "total_urls_discovered": stats.total_urls_discovered,
                "urls_with_metadata": stats.urls_with_metadata
            }
        });

        let response = if all_urls.is_empty() && stats.sitemaps_processed == 0 {
            ResponseBuilder::error("crawl", "No sitemaps could be processed or no URLs found")
                .build()
        } else {
            ResponseBuilder::success("crawl")
                .with_result(result)
                .build()
        };

        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }
}

#[derive(Debug, Clone)]
struct SitemapStats {
    sitemaps_processed: u32,
    index_files_found: u32,
    total_urls_discovered: u32,
    urls_with_metadata: u32,
}

impl SitemapStats {
    const fn new() -> Self {
        Self {
            sitemaps_processed: 0,
            index_files_found: 0,
            total_urls_discovered: 0,
            urls_with_metadata: 0,
        }
    }
}

struct CrawlOptions {
    follow_sitemaps: bool,
    max_urls: usize,
    timeout_secs: u64,
}

impl SitemapCrawlerTool {
    #[allow(clippy::too_many_arguments)]
    fn crawl_sitemap<'a>(
        &'a self,
        url: &'a str,
        options: &'a CrawlOptions,
        all_urls: &'a mut Vec<Value>,
        visited_sitemaps: &'a mut HashSet<String>,
        stats: &'a mut SitemapStats,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a + Send>> {
        Box::pin(async move {
            // Prevent infinite loops
            if visited_sitemaps.contains(url) {
                return Ok(());
            }
            visited_sitemaps.insert(url.to_string());

            // Stop if we've reached max URLs
            if all_urls.len() >= options.max_urls {
                return Ok(());
            }

            let client = Client::builder()
                .timeout(Duration::from_secs(options.timeout_secs))
                .user_agent("Mozilla/5.0 (compatible; LLMSpell-SitemapCrawler/1.0)")
                .build()
                .unwrap_or_default();

            let response = client
                .get(url)
                .send()
                .await
                .map_err(|e| component_error(format!("Failed to fetch sitemap: {e}")))?;

            if !response.status().is_success() {
                return Err(component_error(format!(
                    "HTTP error: {} - {}",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("Unknown")
                )));
            }

            let xml_content = response
                .text()
                .await
                .map_err(|e| component_error(format!("Failed to read sitemap content: {e}")))?;

            stats.sitemaps_processed += 1;

            // Try to parse as sitemap index first
            if options.follow_sitemaps {
                if let Ok(index_urls) = self.parse_sitemap_index(&xml_content) {
                    if !index_urls.is_empty() {
                        stats.index_files_found += 1;
                        // Recursively crawl child sitemaps
                        for index_url in index_urls {
                            if all_urls.len() >= options.max_urls {
                                break;
                            }
                            self.crawl_sitemap(
                                &index_url,
                                options,
                                all_urls,
                                visited_sitemaps,
                                stats,
                            )
                            .await?;
                        }
                        return Ok(());
                    }
                }
            }

            // Parse as regular sitemap
            let urls = self.parse_sitemap(&xml_content);
            #[allow(clippy::cast_possible_truncation)]
            let urls_len_u32 = urls.len() as u32;
            stats.total_urls_discovered += urls_len_u32;

            for url_entry in urls {
                if all_urls.len() >= options.max_urls {
                    break;
                }
                if url_entry
                    .get("has_metadata")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                {
                    stats.urls_with_metadata += 1;
                }
                all_urls.push(url_entry);
            }

            Ok(())
        })
    }

    #[allow(clippy::unused_self)]
    fn parse_sitemap_index(&self, xml_content: &str) -> std::result::Result<Vec<String>, ()> {
        // Simple XML parsing for sitemap index
        let mut urls = Vec::new();

        // Look for <sitemap><loc>...</loc></sitemap> patterns
        let lines: Vec<&str> = xml_content.lines().collect();
        let mut in_sitemap = false;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.contains("<sitemap>") {
                in_sitemap = true;
            } else if trimmed.contains("</sitemap>") {
                in_sitemap = false;
            } else if in_sitemap && trimmed.contains("<loc>") {
                // Extract URL from <loc>URL</loc>
                if let Some(start) = trimmed.find("<loc>") {
                    if let Some(end) = trimmed.find("</loc>") {
                        let url = &trimmed[start + 5..end];
                        urls.push(url.to_string());
                    }
                }
            }
        }

        if urls.is_empty() {
            Err(())
        } else {
            Ok(urls)
        }
    }

    #[allow(clippy::unused_self)]
    fn parse_sitemap(&self, xml_content: &str) -> Vec<Value> {
        let mut urls = Vec::new();
        let lines: Vec<&str> = xml_content.lines().collect();

        let mut in_url = false;
        let mut current_url = None;
        let mut current_lastmod = None;
        let mut current_changefreq = None;
        let mut current_priority: Option<String> = None;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.contains("<url>") {
                in_url = true;
                current_url = None;
                current_lastmod = None;
                current_changefreq = None;
                current_priority = None;
            } else if trimmed.contains("</url>") {
                if let Some(url) = current_url.take() {
                    let mut url_entry = json!({
                        "url": url,
                        "has_metadata": false
                    });

                    let mut has_metadata = false;
                    if let Some(lastmod) = current_lastmod.take() {
                        url_entry["lastmod"] = json!(lastmod);
                        has_metadata = true;
                    }
                    if let Some(changefreq) = current_changefreq.take() {
                        url_entry["changefreq"] = json!(changefreq);
                        has_metadata = true;
                    }
                    if let Some(priority) = current_priority.take() {
                        if let Ok(p) = priority.parse::<f64>() {
                            url_entry["priority"] = json!(p);
                            has_metadata = true;
                        }
                    }

                    url_entry["has_metadata"] = json!(has_metadata);
                    urls.push(url_entry);
                }
                in_url = false;
            } else if in_url {
                if trimmed.contains("<loc>") {
                    if let Some(start) = trimmed.find("<loc>") {
                        if let Some(end) = trimmed.find("</loc>") {
                            current_url = Some(trimmed[start + 5..end].to_string());
                        }
                    }
                } else if trimmed.contains("<lastmod>") {
                    if let Some(start) = trimmed.find("<lastmod>") {
                        if let Some(end) = trimmed.find("</lastmod>") {
                            current_lastmod = Some(trimmed[start + 9..end].to_string());
                        }
                    }
                } else if trimmed.contains("<changefreq>") {
                    if let Some(start) = trimmed.find("<changefreq>") {
                        if let Some(end) = trimmed.find("</changefreq>") {
                            current_changefreq = Some(trimmed[start + 12..end].to_string());
                        }
                    }
                } else if trimmed.contains("<priority>") {
                    if let Some(start) = trimmed.find("<priority>") {
                        if let Some(end) = trimmed.find("</priority>") {
                            current_priority = Some(trimmed[start + 10..end].to_string());
                        }
                    }
                }
            }
        }

        urls
    }
}
