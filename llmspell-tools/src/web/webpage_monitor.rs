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
use std::time::Duration;

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

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params = extract_parameters(&input)?;
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

        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(validation_error(
                "URL must start with http:// or https://",
                Some("input".to_string()),
            ));
        }

        // Fetch current content
        let current_content = self.fetch_content(url, selector, timeout).await?;

        // If no previous content provided, just return current state
        let Some(prev_content) = previous_content else {
            let result = json!({
                "url": url,
                "current_content": current_content,
                "has_changes": false,
                "message": "No previous content provided - returning current state for future comparison"
            });

            let response = ResponseBuilder::success("monitor")
                .with_result(result)
                .build();

            return Ok(AgentOutput::text(
                serde_json::to_string_pretty(&response).unwrap(),
            ));
        };

        // Compare content
        let changes = self.compare_content(prev_content, &current_content, ignore_whitespace);
        let has_changes = !changes.is_empty();

        let result = json!({
            "url": url,
            "current_content": current_content,
            "previous_content": prev_content,
            "has_changes": has_changes,
            "changes": changes,
            "change_count": changes.len(),
            "selector_used": selector,
            "ignore_whitespace": ignore_whitespace
        });

        let response = ResponseBuilder::success("monitor")
            .with_result(result)
            .build();

        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }
}

impl WebpageMonitorTool {
    async fn fetch_content(
        &self,
        url: &str,
        selector: Option<&str>,
        timeout_secs: u64,
    ) -> Result<String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("Mozilla/5.0 (compatible; LLMSpell-WebpageMonitor/1.0)")
            .build()
            .unwrap_or_default();

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| component_error(format!("Failed to fetch URL: {e}")))?;

        if !response.status().is_success() {
            return Err(component_error(format!(
                "HTTP error: {} - {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| component_error(format!("Failed to read response body: {e}")))?;

        // If selector provided, extract specific content
        if let Some(sel) = selector {
            let document = Html::parse_document(&body);
            match Selector::parse(sel) {
                Ok(css_selector) => {
                    let elements: Vec<String> = document
                        .select(&css_selector)
                        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .collect();

                    if elements.is_empty() {
                        return Err(validation_error(
                            format!("No elements found for selector: {sel}"),
                            Some("selector".to_string()),
                        ));
                    }

                    Ok(elements.join("\n"))
                }
                Err(e) => Err(validation_error(
                    format!("Invalid CSS selector: {e}"),
                    Some("selector".to_string()),
                )),
            }
        } else {
            // Return full text content
            let document = Html::parse_document(&body);
            let body_selector = Selector::parse("body").unwrap();
            document.select(&body_selector).next().map_or_else(
                || {
                    Ok(document
                        .root_element()
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "))
                },
                |body_element| {
                    let text: String = body_element.text().collect::<Vec<_>>().join(" ");
                    Ok(text.split_whitespace().collect::<Vec<_>>().join(" "))
                },
            )
        }
    }

    fn compare_content(
        &self,
        old_content: &str,
        new_content: &str,
        ignore_whitespace: bool,
    ) -> Vec<Value> {
        let (old_text, new_text) = if ignore_whitespace {
            (
                old_content.split_whitespace().collect::<Vec<_>>().join(" "),
                new_content.split_whitespace().collect::<Vec<_>>().join(" "),
            )
        } else {
            (old_content.to_string(), new_content.to_string())
        };

        let diff = TextDiff::from_lines(&old_text, &new_text);
        let mut changes = Vec::new();

        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            for op in group {
                match op.tag() {
                    DiffTag::Delete => {
                        changes.push(json!({
                            "type": "deletion",
                            "old_line": op.old_range().start + 1,
                            "content": old_text.lines().nth(op.old_range().start).unwrap_or(""),
                            "group": idx
                        }));
                    }
                    DiffTag::Insert => {
                        changes.push(json!({
                            "type": "addition",
                            "new_line": op.new_range().start + 1,
                            "content": new_text.lines().nth(op.new_range().start).unwrap_or(""),
                            "group": idx
                        }));
                    }
                    DiffTag::Replace => {
                        changes.push(json!({
                            "type": "modification",
                            "old_line": op.old_range().start + 1,
                            "new_line": op.new_range().start + 1,
                            "old_content": old_text.lines().nth(op.old_range().start).unwrap_or(""),
                            "new_content": new_text.lines().nth(op.new_range().start).unwrap_or(""),
                            "group": idx
                        }));
                    }
                    DiffTag::Equal => {
                        // Skip unchanged content
                    }
                }
            }
        }

        changes
    }
}
