//! ABOUTME: Webpage monitor tool for change detection and alerting
//! ABOUTME: Monitors web pages for changes and calculates differences

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
    params::{extract_optional_string, extract_parameters, extract_required_string},
    response::ResponseBuilder,
};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use similar::{DiffTag, TextDiff};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebpageMonitorTool {
    metadata: ComponentMetadata,
}

impl Default for WebpageMonitorTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebpageMonitorTool {
    #[must_use]
    pub fn new() -> Self {
        info!("Creating WebpageMonitorTool");
        Self {
            metadata: ComponentMetadata::new(
                "webpage-monitor".to_string(),
                "Monitor web pages for changes".to_string(),
            ),
        }
    }
}

#[async_trait]
impl Tool for WebpageMonitorTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "URL to monitor for changes".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "previous_content".to_string(),
            param_type: ParameterType::String,
            description: "Previous content to compare against (optional)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "selector".to_string(),
            param_type: ParameterType::String,
            description: "CSS selector to monitor specific content (optional)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "ignore_whitespace".to_string(),
            param_type: ParameterType::Boolean,
            description: "Ignore whitespace changes".to_string(),
            required: false,
            default: Some(json!(true)),
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Data
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted
    }
}

type MonitorParams = (String, Option<String>, Option<String>, bool, u64);

impl WebpageMonitorTool {
    fn extract_monitor_params(params: &Value) -> Result<MonitorParams> {
        let url = extract_required_string(params, "input")?;
        let previous_content = extract_optional_string(params, "previous_content");
        let selector = extract_optional_string(params, "selector");
        let ignore_whitespace = params
            .get("parameters")
            .and_then(|p| p.get("ignore_whitespace"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let timeout = params
            .get("parameters")
            .and_then(|p| p.get("timeout"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(30);

        debug!(
            url = %url,
            has_previous_content = previous_content.is_some(),
            has_selector = selector.is_some(),
            ignore_whitespace,
            timeout_secs = timeout,
            "Starting webpage monitoring with configuration"
        );

        Ok((
            url.to_string(),
            previous_content.map(String::from),
            selector.map(String::from),
            ignore_whitespace,
            timeout,
        ))
    }

    fn validate_url(url: &str) -> Result<()> {
        trace!(url = %url, "Validating URL format");
        if !url.starts_with("http://") && !url.starts_with("https://") {
            error!(
                url = %url,
                "Invalid URL format - must start with http:// or https://"
            );
            return Err(validation_error(
                "URL must start with http:// or https://",
                Some("input".to_string()),
            ));
        }
        trace!(url = %url, "URL format validation passed");
        Ok(())
    }

    fn build_initial_state_response(url: &str, current_content: &str) -> Value {
        json!({
            "url": url,
            "current_content": current_content,
            "has_changes": false,
            "message": "No previous content provided - returning current state for future comparison"
        })
    }

    fn build_comparison_response(
        url: &str,
        current_content: &str,
        prev_content: &str,
        selector: Option<&String>,
        ignore_whitespace: bool,
        changes: &[Value],
    ) -> Value {
        json!({
            "url": url,
            "current_content": current_content,
            "previous_content": prev_content,
            "has_changes": !changes.is_empty(),
            "changes": changes,
            "change_count": changes.len(),
            "selector_used": selector,
            "ignore_whitespace": ignore_whitespace
        })
    }
}

#[async_trait]
impl BaseAgent for WebpageMonitorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("WebpageMonitor error: {error}")))
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let start = Instant::now();
        info!(
            input_size = input.text.len(),
            has_params = !input.parameters.is_empty(),
            "Executing webpage monitor tool"
        );

        let params = extract_parameters(&input)?;
        let (url, previous_content, selector, ignore_whitespace, timeout) =
            Self::extract_monitor_params(params)?;

        // Validate URL
        Self::validate_url(&url)?;

        // Fetch current content
        debug!(
            url = %url,
            selector = ?selector,
            "Fetching current webpage content"
        );
        let current_content = self
            .fetch_content(&url, selector.as_deref(), timeout)
            .await?;

        trace!(
            url = %url,
            content_length = current_content.len(),
            "Current content fetched successfully"
        );

        // If no previous content provided, just return current state
        let Some(prev_content) = previous_content else {
            debug!(url = %url, "No previous content provided - returning current state");
            let result = Self::build_initial_state_response(&url, &current_content);

            let elapsed_ms = start.elapsed().as_millis();
            info!(
                url = %url,
                content_length = current_content.len(),
                duration_ms = elapsed_ms,
                "Webpage monitoring completed (initial state)"
            );

            let response = ResponseBuilder::success("monitor")
                .with_result(result)
                .build();

            return Ok(AgentOutput::text(
                serde_json::to_string_pretty(&response).unwrap(),
            ));
        };

        // Compare content
        debug!(
            url = %url,
            ignore_whitespace,
            previous_length = prev_content.len(),
            current_length = current_content.len(),
            "Comparing content for changes"
        );
        let changes = Self::compare_content(&prev_content, &current_content, ignore_whitespace);
        let has_changes = !changes.is_empty();

        debug!(
            url = %url,
            has_changes,
            change_count = changes.len(),
            "Content comparison completed"
        );

        let result = Self::build_comparison_response(
            &url,
            &current_content,
            &prev_content,
            selector.as_ref(),
            ignore_whitespace,
            &changes,
        );

        let elapsed_ms = start.elapsed().as_millis();
        info!(
            url = %url,
            has_changes,
            change_count = changes.len(),
            duration_ms = elapsed_ms,
            "Webpage monitoring completed"
        );

        let response = ResponseBuilder::success("monitor")
            .with_result(result)
            .build();

        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }
}

impl WebpageMonitorTool {
    async fn fetch_page(url: &str, timeout_secs: u64) -> Result<String> {
        let client = Self::create_http_client(timeout_secs);
        let response = Self::send_http_request(&client, url).await?;
        let body = Self::process_http_response(response, url).await?;

        trace!(
            url = %url,
            body_length = body.len(),
            "Response body read successfully"
        );

        Ok(body)
    }

    fn create_http_client(timeout_secs: u64) -> Client {
        create_io_bound_resource(move || {
            Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .user_agent("Mozilla/5.0 (compatible; LLMSpell-WebpageMonitor/1.0)")
                .build()
                .unwrap_or_default()
        })
    }

    async fn send_http_request(client: &Client, url: &str) -> Result<reqwest::Response> {
        let request_start = Instant::now();
        trace!(url = %url, "Sending HTTP GET request");

        let response = client.get(url).send().await.map_err(|e| {
            error!(
                url = %url,
                error = %e,
                duration_ms = request_start.elapsed().as_millis(),
                "Failed to fetch URL"
            );
            component_error(format!("Failed to fetch URL: {e}"))
        })?;

        debug!(
            url = %url,
            status_code = response.status().as_u16(),
            success = response.status().is_success(),
            duration_ms = request_start.elapsed().as_millis(),
            "HTTP request completed"
        );

        Ok(response)
    }

    async fn process_http_response(response: reqwest::Response, url: &str) -> Result<String> {
        let status = response.status();

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

        let body = response.text().await.map_err(|e| {
            error!(
                url = %url,
                error = %e,
                "Failed to read response body"
            );
            component_error(format!("Failed to read response body: {e}"))
        })?;

        Ok(body)
    }

    fn extract_with_selector(document: &Html, selector_str: &str, url: &str) -> Result<String> {
        debug!(
            url = %url,
            selector = %selector_str,
            "Parsing HTML and extracting content with CSS selector"
        );

        let css_selector = Selector::parse(selector_str).map_err(|e| {
            error!(
                url = %url,
                selector = %selector_str,
                error = ?e,
                "Invalid CSS selector"
            );
            validation_error(
                format!("Invalid CSS selector: {e}"),
                Some("selector".to_string()),
            )
        })?;

        let elements: Vec<String> = document
            .select(&css_selector)
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .collect();

        if elements.is_empty() {
            error!(
                url = %url,
                selector = %selector_str,
                "No elements found for CSS selector"
            );
            return Err(validation_error(
                format!("No elements found for selector: {selector_str}"),
                Some("selector".to_string()),
            ));
        }

        let result = elements.join("\n");
        debug!(
            url = %url,
            selector = %selector_str,
            element_count = elements.len(),
            content_length = result.len(),
            "Content extracted successfully with selector"
        );

        Ok(result)
    }

    fn extract_full_text(document: &Html, url: &str) -> String {
        debug!(url = %url, "Parsing HTML and extracting full text content");

        let body_selector = Selector::parse("body").unwrap();
        document.select(&body_selector).next().map_or_else(
            || {
                let text = document
                    .root_element()
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                debug!(
                    url = %url,
                    content_length = text.len(),
                    "Full text content extracted from root element"
                );
                text
            },
            |body_element| {
                let text: String = body_element.text().collect::<Vec<_>>().join(" ");
                let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
                debug!(
                    url = %url,
                    content_length = cleaned.len(),
                    "Full text content extracted from body element"
                );
                cleaned
            },
        )
    }

    async fn fetch_content(
        &self,
        url: &str,
        selector: Option<&str>,
        timeout_secs: u64,
    ) -> Result<String> {
        let fetch_start = Instant::now();
        debug!(
            url = %url,
            timeout_secs,
            has_selector = selector.is_some(),
            "Starting content fetch"
        );

        // Fetch the page
        let body = Self::fetch_page(url, timeout_secs).await?;

        // Parse HTML
        let parse_start = Instant::now();
        let document = Html::parse_document(&body);
        trace!(
            url = %url,
            parse_duration_ms = parse_start.elapsed().as_millis(),
            "HTML document parsed"
        );

        // Extract content based on selector
        let result = if let Some(sel) = selector {
            Self::extract_with_selector(&document, sel, url)?
        } else {
            Self::extract_full_text(&document, url)
        };

        debug!(
            url = %url,
            content_length = result.len(),
            fetch_duration_ms = fetch_start.elapsed().as_millis(),
            "Content extraction completed"
        );

        Ok(result)
    }

    fn compare_content(
        old_content: &str,
        new_content: &str,
        ignore_whitespace: bool,
    ) -> Vec<Value> {
        let compare_start = Instant::now();
        trace!(
            old_length = old_content.len(),
            new_length = new_content.len(),
            ignore_whitespace,
            "Starting content comparison"
        );

        let (old_text, new_text) =
            Self::normalize_content_for_comparison(old_content, new_content, ignore_whitespace);
        let changes = Self::compute_text_diff_changes(&old_text, &new_text);

        debug!(
            total_changes = changes.len(),
            compare_duration_ms = compare_start.elapsed().as_millis(),
            "Content comparison completed"
        );

        changes
    }

    fn normalize_content_for_comparison(
        old_content: &str,
        new_content: &str,
        ignore_whitespace: bool,
    ) -> (String, String) {
        if ignore_whitespace {
            let old_normalized = old_content.split_whitespace().collect::<Vec<_>>().join(" ");
            let new_normalized = new_content.split_whitespace().collect::<Vec<_>>().join(" ");
            trace!(
                old_normalized_length = old_normalized.len(),
                new_normalized_length = new_normalized.len(),
                "Content normalized for whitespace comparison"
            );
            (old_normalized, new_normalized)
        } else {
            (old_content.to_string(), new_content.to_string())
        }
    }

    fn compute_text_diff_changes(old_text: &str, new_text: &str) -> Vec<Value> {
        let diff_start = Instant::now();
        let diff = TextDiff::from_lines(old_text, new_text);
        let mut changes = Vec::new();

        trace!(
            diff_duration_ms = diff_start.elapsed().as_millis(),
            "Text diff calculation completed"
        );

        let mut change_stats = (0, 0, 0); // deletions, additions, modifications

        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            for op in group {
                Self::process_diff_operation(
                    op,
                    idx,
                    old_text,
                    new_text,
                    &mut changes,
                    &mut change_stats,
                );
            }
        }

        debug!(
            deletions = change_stats.0,
            additions = change_stats.1,
            modifications = change_stats.2,
            "Diff operation statistics"
        );

        changes
    }

    fn process_diff_operation(
        op: &similar::DiffOp,
        group_idx: usize,
        old_text: &str,
        new_text: &str,
        changes: &mut Vec<Value>,
        change_stats: &mut (i32, i32, i32),
    ) {
        match op.tag() {
            DiffTag::Delete => {
                change_stats.0 += 1;
                changes.push(json!({
                    "type": "deletion",
                    "old_line": op.old_range().start + 1,
                    "content": old_text.lines().nth(op.old_range().start).unwrap_or(""),
                    "group": group_idx
                }));
            }
            DiffTag::Insert => {
                change_stats.1 += 1;
                changes.push(json!({
                    "type": "addition",
                    "new_line": op.new_range().start + 1,
                    "content": new_text.lines().nth(op.new_range().start).unwrap_or(""),
                    "group": group_idx
                }));
            }
            DiffTag::Replace => {
                change_stats.2 += 1;
                changes.push(json!({
                    "type": "modification",
                    "old_line": op.old_range().start + 1,
                    "new_line": op.new_range().start + 1,
                    "old_content": old_text.lines().nth(op.old_range().start).unwrap_or(""),
                    "new_content": new_text.lines().nth(op.new_range().start).unwrap_or(""),
                    "group": group_idx
                }));
            }
            DiffTag::Equal => {
                // Skip unchanged content
            }
        }
    }
}
