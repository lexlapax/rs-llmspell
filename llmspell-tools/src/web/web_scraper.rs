//! ABOUTME: Web scraper tool for extracting content from web pages
//! ABOUTME: Supports HTML parsing, CSS selectors, and JavaScript rendering

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, Result,
};
use llmspell_utils::{
    error_builders::llmspell::{component_error, validation_error},
    params::{
        extract_optional_bool, extract_optional_object, extract_parameters, extract_required_string,
    },
    response::ResponseBuilder,
};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebScraperConfig {
    /// Default timeout in seconds
    pub default_timeout: u64,
    /// User agent string
    pub user_agent: String,
}

impl Default for WebScraperConfig {
    fn default() -> Self {
        Self {
            default_timeout: 30,
            user_agent: "Mozilla/5.0 (compatible; LLMSpell/1.0)".to_string(),
        }
    }
}

/// Web scraper tool for extracting content from web pages
#[derive(Debug)]
pub struct WebScraperTool {
    metadata: ComponentMetadata,
    #[allow(dead_code)]
    config: WebScraperConfig,
    client: Client,
}

impl Default for WebScraperTool {
    fn default() -> Self {
        Self::new(WebScraperConfig::default())
    }
}

impl WebScraperTool {
    /// Create a new web scraper tool
    pub fn new(config: WebScraperConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.default_timeout))
            .user_agent(&config.user_agent)
            .build()
            .unwrap_or_default();

        Self {
            metadata: ComponentMetadata::new(
                "web-scraper".to_string(),
                "Scrape and extract content from web pages using CSS selectors".to_string(),
            ),
            config,
            client,
        }
    }

    async fn scrape_page(
        &self,
        url: &str,
        selectors: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        // Fetch the page
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| component_error(format!("Failed to fetch URL: {}", e)))?;

        if !response.status().is_success() {
            return Err(component_error(format!(
                "HTTP error: {} - {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let html_content = response
            .text()
            .await
            .map_err(|e| component_error(format!("Failed to read response: {}", e)))?;

        // Parse HTML
        let document = Html::parse_document(&html_content);
        let mut result = HashMap::new();

        if let Some(selectors) = selectors {
            // Extract specific elements using CSS selectors
            for (name, selector_str) in selectors {
                match Selector::parse(&selector_str) {
                    Ok(selector) => {
                        let elements: Vec<String> = document
                            .select(&selector)
                            .map(|el| {
                                // Try to get text content, fallback to HTML
                                let text =
                                    el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                                if text.is_empty() {
                                    el.html()
                                } else {
                                    text
                                }
                            })
                            .collect();

                        if elements.len() == 1 {
                            result.insert(name, json!(elements[0]));
                        } else if !elements.is_empty() {
                            result.insert(name, json!(elements));
                        }
                    }
                    Err(e) => {
                        return Err(validation_error(
                            format!("Invalid CSS selector '{}': {:?}", selector_str, e),
                            Some("selectors".to_string()),
                        ));
                    }
                }
            }
        } else {
            // Extract common metadata if no selectors provided
            let title_selector = Selector::parse("title").unwrap();
            if let Some(title) = document.select(&title_selector).next() {
                result.insert("title".to_string(), json!(title.text().collect::<String>()));
            }

            let meta_description = Selector::parse("meta[name=\"description\"]").unwrap();
            if let Some(desc) = document.select(&meta_description).next() {
                if let Some(content) = desc.value().attr("content") {
                    result.insert("description".to_string(), json!(content));
                }
            }

            // Extract all text content
            let body_selector = Selector::parse("body").unwrap();
            if let Some(body) = document.select(&body_selector).next() {
                let text: String = body.text().collect::<Vec<_>>().join(" ");
                let cleaned_text = text.split_whitespace().collect::<Vec<_>>().join(" ");
                result.insert("text".to_string(), json!(cleaned_text));
            }

            // Extract all links
            let link_selector = Selector::parse("a[href]").unwrap();
            let links: Vec<String> = document
                .select(&link_selector)
                .filter_map(|el| el.value().attr("href"))
                .map(|href| href.to_string())
                .collect();
            if !links.is_empty() {
                result.insert("links".to_string(), json!(links));
            }

            // Extract all images
            let img_selector = Selector::parse("img[src]").unwrap();
            let images: Vec<String> = document
                .select(&img_selector)
                .filter_map(|el| el.value().attr("src"))
                .map(|src| src.to_string())
                .collect();
            if !images.is_empty() {
                result.insert("images".to_string(), json!(images));
            }
        }

        Ok(json!(result))
    }
}

#[async_trait]
impl Tool for WebScraperTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "URL to scrape".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "selectors".to_string(),
            param_type: ParameterType::Object,
            description: "CSS selectors as key-value pairs (e.g., {\"title\": \"h1\", \"content\": \".article-body\"})".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "wait_for_js".to_string(),
            param_type: ParameterType::Boolean,
            description: "Wait for JavaScript to load (requires headless browser, not yet implemented)".to_string(),
            required: false,
            default: Some(serde_json::json!(false)),
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Data
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted
    }
}

#[async_trait]
impl BaseAgent for WebScraperTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        // Validation is done in execute method when extracting parameters
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("WebScraper error: {}", error)))
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params = extract_parameters(&input)?;
        let url = extract_required_string(params, "input")?;

        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(validation_error(
                "URL must start with http:// or https://",
                Some("input".to_string()),
            ));
        }

        let selectors = if let Some(obj) = extract_optional_object(params, "selectors") {
            serde_json::from_value::<HashMap<String, String>>(serde_json::Value::Object(
                obj.clone(),
            ))
            .ok()
        } else {
            None
        };

        let wait_for_js = extract_optional_bool(params, "wait_for_js").unwrap_or(false);

        if wait_for_js {
            return Err(component_error(
                "JavaScript rendering not yet implemented. Please use selectors for static content."
            ));
        }

        let result = self.scrape_page(url, selectors).await?;

        let response = ResponseBuilder::success("scrape")
            .with_result(json!({
                "url": url,
                "content": result
            }))
            .with_metadata("timestamp", json!(chrono::Utc::now().to_rfc3339()))
            .build();

        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_tool_schema() {
        let tool = WebScraperTool::new(WebScraperConfig::default());
        let schema = tool.schema();
        assert_eq!(schema.name, "web-scraper");
        assert_eq!(schema.parameters.len(), 3);
        assert_eq!(schema.parameters[0].name, "input");
        assert!(schema.parameters[0].required);
    }

    #[tokio::test]
    async fn test_url_validation() {
        let tool = WebScraperTool::new(WebScraperConfig::default());
        let mut params = HashMap::new();
        params.insert(
            "parameters".to_string(),
            json!({
                "input": "not-a-url"
            }),
        );
        let input = AgentInput {
            text: "".to_string(),
            media: vec![],
            context: None,
            parameters: params,
            output_modalities: vec![],
        };
        let context = ExecutionContext::default();

        let result = tool.execute(input, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_js_wait_not_implemented() {
        let tool = WebScraperTool::new(WebScraperConfig::default());
        let mut params = HashMap::new();
        params.insert(
            "parameters".to_string(),
            json!({
                "input": "https://example.com",
                "wait_for_js": true
            }),
        );
        let input = AgentInput {
            text: "".to_string(),
            media: vec![],
            context: None,
            parameters: params,
            output_modalities: vec![],
        };
        let context = ExecutionContext::default();

        let result = tool.execute(input, context).await;
        assert!(result.is_err());
    }
}
