//! ABOUTME: API tester tool for REST API testing and validation
//! ABOUTME: Tests REST APIs with various HTTP methods and validates responses

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
    params::{
        extract_optional_object, extract_optional_string, extract_optional_u64, extract_parameters,
        extract_required_string,
    },
    response::ResponseBuilder,
    security::ssrf_protection::SsrfProtector,
};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTesterTool {
    metadata: ComponentMetadata,
}

impl Default for ApiTesterTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiTesterTool {
    #[must_use]
    pub fn new() -> Self {
        info!("Creating ApiTesterTool");
        Self {
            metadata: ComponentMetadata::new(
                "api-tester".to_string(),
                "Test and validate REST APIs".to_string(),
            ),
        }
    }
}

#[async_trait]
impl Tool for ApiTesterTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "API endpoint URL".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "method".to_string(),
            param_type: ParameterType::String,
            description: "HTTP method (GET, POST, PUT, DELETE, PATCH)".to_string(),
            required: false,
            default: Some(json!("GET")),
        })
        .with_parameter(ParameterDef {
            name: "headers".to_string(),
            param_type: ParameterType::Object,
            description: "HTTP headers as key-value pairs".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "body".to_string(),
            param_type: ParameterType::Object,
            description: "Request body (for POST, PUT, PATCH)".to_string(),
            required: false,
            default: None,
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
impl BaseAgent for ApiTesterTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("ApiTester error: {error}")))
    }

    #[allow(clippy::too_many_lines)]
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let start = Instant::now();
        info!(
            input_size = input.text.len(),
            has_params = !input.parameters.is_empty(),
            "Executing API tester tool"
        );

        let params = extract_parameters(&input)?;
        let url = extract_required_string(params, "input")?;
        let method_str = extract_optional_string(params, "method").unwrap_or("GET");
        let headers = extract_optional_object(params, "headers");
        let body = extract_optional_object(params, "body");
        let timeout = extract_optional_u64(params, "timeout").unwrap_or(30);

        debug!(
            url = %url,
            method = %method_str,
            has_headers = headers.is_some(),
            has_body = body.is_some(),
            timeout_seconds = timeout,
            "Preparing API test request"
        );

        // Validate URL with SSRF protection
        let ssrf_protector = SsrfProtector::new();
        if let Err(e) = ssrf_protector.validate_url(url) {
            error!(
                url = %url,
                error = %e,
                "URL validation failed (SSRF protection)"
            );
            return Err(validation_error(
                format!("URL validation failed: {e}"),
                Some("input".to_string()),
            ));
        }
        trace!(url = %url, "URL passed SSRF validation");

        // Parse HTTP method
        let method = match method_str.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            _ => {
                error!(
                    method = %method_str,
                    "Invalid HTTP method"
                );
                return Err(validation_error(
                    format!("Invalid HTTP method: {method_str}"),
                    Some("method".to_string()),
                ));
            }
        };

        // Build client using global runtime
        let client = create_io_bound_resource(move || {
            Client::builder()
                .timeout(Duration::from_secs(timeout))
                .user_agent("Mozilla/5.0 (compatible; LLMSpell-ApiTester/1.0)")
                .build()
                .unwrap_or_default()
        });

        // Build request
        let mut request = client.request(method.clone(), url);

        // Add headers
        if let Some(headers_map) = headers {
            let header_count = headers_map.len();
            trace!(header_count, "Adding request headers");
            for (key, value) in headers_map {
                if let Some(val_str) = value.as_str() {
                    trace!(header_key = %key, "Adding header");
                    request = request.header(key, val_str);
                }
            }
        }

        // Add body for methods that support it
        if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
            if let Some(body_data) = body {
                debug!("Adding JSON body to request");
                request = request.json(&body_data);
            }
        }

        // Execute request and measure time
        let request_start = Instant::now();
        info!(
            url = %url,
            method = %method_str,
            "Sending API test request"
        );

        let response = request.send().await.map_err(|e| {
            error!(
                url = %url,
                method = %method_str,
                error = %e,
                duration_ms = request_start.elapsed().as_millis(),
                "API request failed"
            );
            component_error(format!("Request failed: {e}"))
        })?;
        let request_duration = request_start.elapsed();

        // Extract response data
        let status = response.status();
        let headers = response.headers().clone();

        info!(
            url = %url,
            method = %method_str,
            status_code = status.as_u16(),
            success = status.is_success(),
            duration_ms = request_duration.as_millis(),
            "API request completed"
        );

        let mut response_headers = HashMap::new();
        for (name, value) in &headers {
            if let Ok(val) = value.to_str() {
                response_headers.insert(name.to_string(), val.to_string());
            }
        }

        // Try to parse response body
        let body_text = response.text().await.unwrap_or_default();
        let body_json: Option<Value> = serde_json::from_str(&body_text).ok();

        trace!(
            body_length = body_text.len(),
            is_json = body_json.is_some(),
            "Parsed response body"
        );

        let body_value = body_json.clone().unwrap_or_else(|| json!(body_text));
        let result = json!({
            "request": {
                "method": method_str,
                "url": url,
                "timeout": timeout,
            },
            "response": {
                "status_code": status.as_u16(),
                "status_text": status.canonical_reason().unwrap_or("Unknown"),
                "is_success": status.is_success(),
                "headers": response_headers,
                "body": body_value,
                "body_is_json": body_json.is_some(),
            },
            "timing": {
                "duration_ms": request_duration.as_millis(),
                "duration_secs": request_duration.as_secs_f64(),
            }
        });

        let response = ResponseBuilder::success("test").with_result(result).build();

        let elapsed_ms = start.elapsed().as_millis();
        debug!(
            url = %url,
            method = %method_str,
            status_code = status.as_u16(),
            total_duration_ms = elapsed_ms,
            "API test completed"
        );

        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }
}
