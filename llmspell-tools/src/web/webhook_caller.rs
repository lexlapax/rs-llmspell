//! ABOUTME: Webhook caller tool for invoking webhooks with retry logic
//! ABOUTME: Calls webhooks with configurable retries and timeout

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
    error_builders::llmspell::validation_error,
    params::{extract_optional_object, extract_parameters, extract_required_string},
    response::ResponseBuilder,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookCallerTool {
    metadata: ComponentMetadata,
}

impl Default for WebhookCallerTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebhookCallerTool {
    pub fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "webhook-caller".to_string(),
                "Invoke webhooks with retry logic".to_string(),
            ),
        }
    }
}

#[async_trait]
impl Tool for WebhookCallerTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Webhook URL to call".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "payload".to_string(),
            param_type: ParameterType::Object,
            description: "JSON payload to send to webhook".to_string(),
            required: false,
            default: Some(json!({})),
        })
        .with_parameter(ParameterDef {
            name: "headers".to_string(),
            param_type: ParameterType::Object,
            description: "HTTP headers to include".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "max_retries".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum number of retry attempts".to_string(),
            required: false,
            default: Some(json!(3)),
        })
        .with_parameter(ParameterDef {
            name: "timeout".to_string(),
            param_type: ParameterType::Number,
            description: "Request timeout in seconds".to_string(),
            required: false,
            default: Some(json!(30)),
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
impl BaseAgent for WebhookCallerTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("WebhookCaller error: {}", error)))
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params = extract_parameters(&input)?;
        let url = extract_required_string(params, "input")?;
        let payload = extract_optional_object(params, "payload");
        let headers = extract_optional_object(params, "headers");
        let max_retries = params
            .get("parameters")
            .and_then(|p| p.get("max_retries"))
            .and_then(|r| r.as_u64())
            .unwrap_or(3) as u32;
        let timeout = params
            .get("parameters")
            .and_then(|p| p.get("timeout"))
            .and_then(|t| t.as_u64())
            .unwrap_or(30);

        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(validation_error(
                "URL must start with http:// or https://",
                Some("input".to_string()),
            ));
        }

        // Build client
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .user_agent("Mozilla/5.0 (compatible; LLMSpell-WebhookCaller/1.0)")
            .build()
            .unwrap_or_default();

        // Simple retry implementation
        let mut last_error = None;
        let mut retry_count = 0;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                // Wait before retry (exponential backoff)
                let delay = Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
                retry_count += 1;
            }

            let mut request = client.post(url);

            // Add payload
            if let Some(payload_data) = &payload {
                request = request.json(payload_data);
            } else {
                // Default empty JSON payload
                request = request.json(&json!({}));
            }

            // Add headers
            if let Some(headers_map) = headers {
                for (key, value) in headers_map {
                    if let Some(val_str) = value.as_str() {
                        request = request.header(key, val_str);
                    }
                }
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    let response_headers = response.headers().clone();
                    let body_text = response.text().await.unwrap_or_default();

                    // Only retry on 5xx errors
                    if status.is_server_error() && attempt < max_retries {
                        last_error = Some(format!(
                            "Server error: {} {}",
                            status.as_u16(),
                            status.canonical_reason().unwrap_or("Unknown")
                        ));
                        continue;
                    }

                    let body_json: Option<Value> = serde_json::from_str(&body_text).ok();

                    let mut headers_map = serde_json::Map::new();
                    for (name, value) in response_headers.iter() {
                        if let Ok(val) = value.to_str() {
                            headers_map.insert(name.to_string(), json!(val));
                        }
                    }

                    let result = json!({
                        "success": status.is_success(),
                        "webhook_url": url,
                        "response": {
                            "status_code": status.as_u16(),
                            "status_text": status.canonical_reason().unwrap_or("Unknown"),
                            "headers": headers_map,
                            "body": body_json.as_ref().unwrap_or(&json!(body_text)),
                            "body_is_json": body_json.is_some(),
                        },
                        "retry_count": retry_count,
                    });

                    let response = ResponseBuilder::success("call").with_result(result).build();

                    return Ok(AgentOutput::text(
                        serde_json::to_string_pretty(&response).unwrap(),
                    ));
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    if attempt < max_retries {
                        continue;
                    }
                }
            }
        }

        // All retries exhausted
        let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
        let error_result = json!({
            "success": false,
            "webhook_url": url,
            "error": error_msg,
            "max_retries": max_retries,
            "retry_count": retry_count,
        });

        let response = ResponseBuilder::error("call", error_msg)
            .with_result(error_result)
            .build();

        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }
}
