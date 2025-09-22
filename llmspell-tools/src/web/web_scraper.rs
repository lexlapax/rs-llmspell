//! ABOUTME: Web scraper tool for extracting content from web pages
//! ABOUTME: Supports HTML parsing, CSS selectors, and JavaScript rendering

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, Result,
};
use llmspell_kernel::runtime::create_io_bound_resource;
use llmspell_utils::{
    error_builders::llmspell::{component_error, validation_error},
    // error_handling::{ErrorContext, SafeErrorHandler}, // Available for production use
    params::{
        extract_optional_bool, extract_optional_object, extract_optional_string,
        extract_optional_u64, extract_parameters, extract_required_string,
    },
    response::ResponseBuilder,
    security::{input_sanitizer::InputSanitizer, ssrf_protection::SsrfProtector},
};
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, trace, warn};

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
    #[allow(dead_code)]
    client: Client,
}

impl Default for WebScraperTool {
    fn default() -> Self {
        info!(
            tool_name = "web-scraper",
            category = "Tool",
            phase = "Phase 3 (comprehensive instrumentation)",
            "Creating WebScraperTool"
        );

        Self::new(WebScraperConfig::default())
    }
}

#[derive(Debug)]
struct ScrapeOptions {
    timeout_secs: u64,
    extract_links: bool,
    extract_images: bool,
    extract_meta: bool,
}

impl WebScraperTool {
    /// Create a new web scraper tool
    #[must_use]
    pub fn new(config: WebScraperConfig) -> Self {
        info!(
            timeout_secs = config.default_timeout,
            user_agent = %config.user_agent,
            "Creating WebScraperTool"
        );
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

    // #[instrument(skip(self))] // Disabled - method not found
    async fn fetch_page(url: &str, timeout_secs: u64) -> Result<String> {
        let client = create_io_bound_resource(move || {
            Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .user_agent("Mozilla/5.0 (compatible; LLMSpell/1.0)")
                .build()
                .unwrap_or_default()
        });

        let request_start = Instant::now();
        info!(url = %url, "Fetching web page");

        let response = client.get(url).send().await.map_err(|e| {
            error!(
                url = %url,
                error = %e,
                duration_ms = request_start.elapsed().as_millis(),
                "Failed to fetch URL"
            );
            component_error(format!("Failed to fetch URL: {e}"))
        })?;

        let status = response.status();
        debug!(
            url = %url,
            status_code = status.as_u16(),
            success = status.is_success(),
            duration_ms = request_start.elapsed().as_millis(),
            "HTTP request completed"
        );

        if !status.is_success() {
            error!(
                url = %url,
                status_code = status.as_u16(),
                "HTTP error response"
            );
            return Err(component_error(format!(
                "HTTP error: {} - {}",
                status,
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        response.text().await.map_err(|e| {
            error!(
                url = %url,
                error = %e,
                "Failed to read response body"
            );
            component_error(format!("Failed to read response: {e}"))
        })
    }

    fn extract_element_text(el: ElementRef) -> String {
        let text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if text.is_empty() {
            el.html()
        } else {
            text
        }
    }

    fn process_single_selector(
        document: &Html,
        selector_str: &str,
        url: &str,
    ) -> Result<Vec<String>> {
        debug!(
            url = %url,
            selector = %selector_str,
            "Processing single CSS selector"
        );

        let selector = Selector::parse(selector_str).map_err(|e| {
            error!(
                url = %url,
                selector = %selector_str,
                error = ?e,
                "Invalid CSS selector"
            );
            validation_error(
                format!("Invalid CSS selector '{selector_str}': {e:?}"),
                Some("selector".to_string()),
            )
        })?;

        let elements: Vec<String> = document
            .select(&selector)
            .map(Self::extract_element_text)
            .collect();

        trace!(
            url = %url,
            selector = %selector_str,
            element_count = elements.len(),
            "Extracted elements with single selector"
        );

        Ok(elements)
    }

    fn process_selector(
        document: &Html,
        name: &str,
        selector_str: &str,
        url: &str,
        result: &mut HashMap<String, Value>,
    ) -> Result<()> {
        trace!(
            url = %url,
            selector_name = %name,
            selector = %selector_str,
            "Processing selector"
        );

        let selector = Selector::parse(selector_str).map_err(|e| {
            error!(
                url = %url,
                selector_name = %name,
                selector = %selector_str,
                error = ?e,
                "Invalid CSS selector"
            );
            validation_error(
                format!("Invalid CSS selector '{selector_str}': {e:?}"),
                Some("selectors".to_string()),
            )
        })?;

        let elements: Vec<String> = document
            .select(&selector)
            .map(Self::extract_element_text)
            .collect();

        trace!(
            url = %url,
            selector_name = %name,
            element_count = elements.len(),
            "Extracted elements for selector"
        );

        if elements.len() == 1 {
            result.insert(name.to_string(), json!(elements[0]));
        } else if !elements.is_empty() {
            result.insert(name.to_string(), json!(elements));
        }

        Ok(())
    }

    fn extract_basic_content(document: &Html, url: &str, result: &mut HashMap<String, Value>) {
        debug!(url = %url, "Extracting basic content (title and text)");

        // Extract title
        let title_selector = Selector::parse("title").unwrap();
        if let Some(title) = document.select(&title_selector).next() {
            let title_text = title.text().collect::<String>();
            trace!(
                url = %url,
                title_length = title_text.len(),
                "Extracted page title"
            );
            result.insert("title".to_string(), json!(title_text));
        }

        // Extract all text content
        let body_selector = Selector::parse("body").unwrap();
        if let Some(body) = document.select(&body_selector).next() {
            let text: String = body.text().collect::<Vec<_>>().join(" ");
            let cleaned_text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            trace!(
                url = %url,
                text_length = cleaned_text.len(),
                "Extracted body text"
            );
            result.insert("text".to_string(), json!(cleaned_text));
        }
    }

    fn extract_links(document: &Html, url: &str, result: &mut HashMap<String, Value>) {
        debug!(url = %url, "Extracting links from page");

        let link_selector = Selector::parse("a[href]").unwrap();
        let links: Vec<String> = document
            .select(&link_selector)
            .filter_map(|el| el.value().attr("href"))
            .map(std::string::ToString::to_string)
            .collect();

        trace!(
            url = %url,
            link_count = links.len(),
            "Extracted links"
        );
        result.insert("links".to_string(), json!(links));
    }

    fn extract_images(document: &Html, url: &str, result: &mut HashMap<String, Value>) {
        debug!(url = %url, "Extracting images from page");

        let img_selector = Selector::parse("img[src]").unwrap();
        let images: Vec<String> = document
            .select(&img_selector)
            .filter_map(|el| el.value().attr("src"))
            .map(std::string::ToString::to_string)
            .collect();

        trace!(
            url = %url,
            image_count = images.len(),
            "Extracted images"
        );
        result.insert("images".to_string(), json!(images));
    }

    fn extract_metadata(document: &Html, url: &str, result: &mut HashMap<String, Value>) {
        debug!(url = %url, "Extracting metadata from page");

        let mut metadata = HashMap::new();

        // Extract meta description
        let meta_description = Selector::parse("meta[name=\"description\"]").unwrap();
        if let Some(desc) = document.select(&meta_description).next() {
            if let Some(content) = desc.value().attr("content") {
                metadata.insert("description".to_string(), json!(content));
            }
        }

        // Extract all meta tags
        let meta_selector = Selector::parse("meta").unwrap();
        let mut meta_tags = HashMap::new();

        for meta in document.select(&meta_selector) {
            if let Some(name) = meta.value().attr("name") {
                if let Some(content) = meta.value().attr("content") {
                    meta_tags.insert(name.to_string(), json!(content));
                }
            }
            if let Some(property) = meta.value().attr("property") {
                if let Some(content) = meta.value().attr("content") {
                    meta_tags.insert(property.to_string(), json!(content));
                }
            }
        }

        if !meta_tags.is_empty() {
            metadata.insert("meta_tags".to_string(), json!(meta_tags));
        }

        trace!(
            url = %url,
            meta_tag_count = meta_tags.len(),
            has_description = metadata.contains_key("description"),
            "Extracted metadata"
        );

        if !metadata.is_empty() {
            result.insert("metadata".to_string(), json!(metadata));
        }
    }

    #[instrument(skip(options, selectors, self))]
    async fn scrape_page(
        &self,
        url: &str,
        options: &ScrapeOptions,
        selectors: Option<HashMap<String, String>>,
        single_selector: Option<String>,
    ) -> Result<Value> {
        let scrape_start = Instant::now();
        debug!(
            url = %url,
            timeout_secs = options.timeout_secs,
            extract_links = options.extract_links,
            extract_images = options.extract_images,
            extract_meta = options.extract_meta,
            has_selectors = selectors.is_some(),
            has_single_selector = single_selector.is_some(),
            "Starting web scraping operation"
        );

        let document = self.fetch_and_parse_page(url, options.timeout_secs).await?;
        let mut result = HashMap::new();

        Self::process_content_selectors(&document, url, selectors, single_selector, &mut result)?;
        Self::extract_optional_content(&document, url, options, &mut result);

        let total_duration = scrape_start.elapsed();
        debug!(
            url = %url,
            result_keys = result.len(),
            total_duration_ms = total_duration.as_millis(),
            "Web scraping operation completed"
        );

        Ok(json!(result))
    }

    // #[instrument(skip(self))] // Disabled - method not found
    async fn fetch_and_parse_page(&self, url: &str, timeout_secs: u64) -> Result<Html> {
        let html_content = Self::fetch_page(url, timeout_secs).await?;

        let parse_start = Instant::now();
        trace!(
            url = %url,
            content_length = html_content.len(),
            "Parsing HTML document"
        );
        let document = Html::parse_document(&html_content);

        debug!(
            url = %url,
            parse_duration_ms = parse_start.elapsed().as_millis(),
            "HTML parsing completed"
        );

        Ok(document)
    }

    fn process_content_selectors(
        document: &Html,
        url: &str,
        selectors: Option<HashMap<String, String>>,
        single_selector: Option<String>,
        result: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if let Some(selector_str) = single_selector {
            let elements = Self::process_single_selector(document, &selector_str, url)?;
            result.insert("selected_content".to_string(), json!(elements));
        } else if let Some(selectors) = selectors {
            debug!(
                url = %url,
                selector_count = selectors.len(),
                "Processing multiple CSS selectors"
            );
            for (name, selector_str) in selectors {
                Self::process_selector(document, &name, &selector_str, url, result)?;
            }
        } else {
            Self::extract_basic_content(document, url, result);
        }
        Ok(())
    }

    fn extract_optional_content(
        document: &Html,
        url: &str,
        options: &ScrapeOptions,
        result: &mut HashMap<String, Value>,
    ) {
        if options.extract_links {
            Self::extract_links(document, url, result);
        }

        if options.extract_images {
            Self::extract_images(document, url, result);
        }

        if options.extract_meta {
            Self::extract_metadata(document, url, result);
        }
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
        .with_parameter(ParameterDef {
            name: "extract_links".to_string(),
            param_type: ParameterType::Boolean,
            description: "Extract all links from the page".to_string(),
            required: false,
            default: Some(serde_json::json!(false)),
        })
        .with_parameter(ParameterDef {
            name: "extract_images".to_string(),
            param_type: ParameterType::Boolean,
            description: "Extract all images from the page".to_string(),
            required: false,
            default: Some(serde_json::json!(false)),
        })
        .with_parameter(ParameterDef {
            name: "extract_meta".to_string(),
            param_type: ParameterType::Boolean,
            description: "Extract meta tags and metadata".to_string(),
            required: false,
            default: Some(serde_json::json!(false)),
        })
        .with_parameter(ParameterDef {
            name: "selector".to_string(),
            param_type: ParameterType::String,
            description: "Single CSS selector to extract content".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "timeout".to_string(),
            param_type: ParameterType::Number,
            description: "Request timeout in seconds".to_string(),
            required: false,
            default: Some(serde_json::json!(30)),
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Data
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted
    }
}

type ScrapeParams = (
    String,
    ScrapeOptions,
    Option<HashMap<String, String>>,
    Option<String>,
);

impl WebScraperTool {
    fn validate_url_security(url: &str) -> Result<()> {
        Self::validate_ssrf_protection(url)?;
        Self::validate_input_sanitization(url)?;
        Ok(())
    }

    fn validate_ssrf_protection(url: &str) -> Result<()> {
        trace!(url = %url, "Validating URL with SSRF protection");
        let ssrf_protector = SsrfProtector::new();
        ssrf_protector.validate_url(url).map_err(|e| {
            error!(url = %url, error = %e, "URL failed SSRF validation");
            validation_error(
                format!("URL validation failed: {e}"),
                Some("input".to_string()),
            )
        })?;
        trace!(url = %url, "URL passed SSRF validation");
        Ok(())
    }

    fn validate_input_sanitization(url: &str) -> Result<()> {
        trace!(url = %url, "Validating URL with input sanitizer");
        let sanitizer = InputSanitizer::new();
        let validation_report = sanitizer.validate(url);

        if !validation_report.is_safe {
            warn!(
                url = %url,
                issue_count = validation_report.issues.len(),
                "URL failed input validation checks"
            );
            Self::check_critical_security_issues(url, &validation_report.issues)?;
        }

        Ok(())
    }

    fn check_critical_security_issues(
        url: &str,
        issues: &[llmspell_utils::security::input_sanitizer::ValidationIssue],
    ) -> Result<()> {
        for issue in issues {
            if matches!(
                issue.severity,
                llmspell_utils::security::input_sanitizer::Severity::High
                    | llmspell_utils::security::input_sanitizer::Severity::Critical
            ) {
                error!(
                    url = %url,
                    issue = ?issue.message,
                    severity = ?issue.severity,
                    "Critical security issue detected in URL"
                );
                return Err(validation_error(
                    format!("URL contains potential security risk: {:?}", issue.message),
                    Some("input".to_string()),
                ));
            }
        }
        Ok(())
    }

    fn extract_scrape_params(params: &Value) -> Result<ScrapeParams> {
        let url = extract_required_string(params, "input")?;

        let extract_links = extract_optional_bool(params, "extract_links").unwrap_or(false);
        let extract_images = extract_optional_bool(params, "extract_images").unwrap_or(false);
        let extract_meta = extract_optional_bool(params, "extract_meta").unwrap_or(false);
        let single_selector = extract_optional_string(params, "selector");
        let timeout = extract_optional_u64(params, "timeout").unwrap_or(30);

        debug!(
            url = %url,
            extract_links,
            extract_images,
            extract_meta,
            has_single_selector = single_selector.is_some(),
            timeout_secs = timeout,
            "Starting web scraping with options"
        );

        let selectors = extract_optional_object(params, "selectors").and_then(|obj| {
            serde_json::from_value::<HashMap<String, String>>(serde_json::Value::Object(
                obj.clone(),
            ))
            .ok()
        });

        if let Some(ref selectors) = selectors {
            debug!(
                url = %url,
                selector_count = selectors.len(),
                "Using custom CSS selectors"
            );
        }

        let wait_for_js = extract_optional_bool(params, "wait_for_js").unwrap_or(false);
        if wait_for_js {
            warn!(url = %url, "JavaScript rendering requested but not implemented");
            return Err(component_error(
                "JavaScript rendering not yet implemented. Please use selectors for static content."
            ));
        }

        let options = ScrapeOptions {
            timeout_secs: timeout,
            extract_links,
            extract_images,
            extract_meta,
        };

        Ok((
            url.to_string(),
            options,
            selectors,
            single_selector.map(std::string::ToString::to_string),
        ))
    }
}

#[async_trait]
impl BaseAgent for WebScraperTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    // #[instrument(skip(self))] // Disabled - method not found
    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        // Validation is done in execute method when extracting parameters
        Ok(())
    }

    // #[instrument(skip(self))] // Disabled - method not found
    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        // In production, we would use SafeErrorHandler to sanitize the error
        // For now, we'll keep the existing behavior but add a comment
        // let handler = SafeErrorHandler::new(true); // true for production mode
        // let context = ErrorContext::new()
        //     .with_operation("web_scraping")
        //     .with_resource(url);
        // let safe_response = handler.handle_llmspell_error(&error, context);
        Ok(AgentOutput::text(format!("WebScraper error: {error}")))
    }

    #[instrument(skip(_context, input, self), fields(tool = %self.metadata().name))]
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let start = Instant::now();
        info!(
            input_size = input.text.len(),
            has_params = !input.parameters.is_empty(),
            "Executing web scraper tool"
        );

        let params = extract_parameters(&input)?;

        // Extract and validate parameters
        let (url, options, selectors, single_selector) = Self::extract_scrape_params(params)?;

        // Validate URL security
        Self::validate_url_security(&url)?;

        let result = self
            .scrape_page(&url, &options, selectors, single_selector)
            .await?;

        let elapsed_ms = start.elapsed().as_millis();
        debug!(
            url = %url,
            duration_ms = elapsed_ms,
            "Web scraping completed successfully"
        );

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
        assert_eq!(schema.parameters.len(), 8);
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
            text: String::new(),
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
            text: String::new(),
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
