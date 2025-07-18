//! ABOUTME: URL analyzer tool for validation and metadata extraction
//! ABOUTME: Analyzes URLs for validity, structure, and extracts metadata

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
    error_builders::llmspell::validation_error,
    params::{extract_optional_bool, extract_parameters, extract_required_string},
    response::ResponseBuilder,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use url::{Host, Url};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlAnalyzerTool {
    metadata: ComponentMetadata,
}

impl Default for UrlAnalyzerTool {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlAnalyzerTool {
    pub fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "url-analyzer".to_string(),
                "Analyze and validate URLs, extract metadata".to_string(),
            ),
        }
    }

    async fn fetch_url_metadata(&self, url: &Url) -> Result<Value> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; LLMSpell/1.0)")
            .build()
            .unwrap_or_default();

        // HEAD request to get headers without body
        let response = client.head(url.to_string()).send().await.map_err(|e| {
            llmspell_utils::error_builders::llmspell::component_error(format!(
                "Failed to fetch URL metadata: {}",
                e
            ))
        })?;

        let status = response.status();
        let headers = response.headers();

        let mut metadata = json!({
            "status_code": status.as_u16(),
            "status_text": status.canonical_reason().unwrap_or("Unknown"),
            "is_success": status.is_success(),
            "is_redirect": status.is_redirection(),
        });

        // Extract useful headers
        if let Some(content_type) = headers.get("content-type") {
            metadata["content_type"] = json!(content_type.to_str().unwrap_or(""));
        }
        if let Some(content_length) = headers.get("content-length") {
            if let Ok(length) = content_length.to_str().unwrap_or("").parse::<u64>() {
                metadata["content_length"] = json!(length);
            }
        }
        if let Some(server) = headers.get("server") {
            metadata["server"] = json!(server.to_str().unwrap_or(""));
        }
        if let Some(last_modified) = headers.get("last-modified") {
            metadata["last_modified"] = json!(last_modified.to_str().unwrap_or(""));
        }
        if let Some(etag) = headers.get("etag") {
            metadata["etag"] = json!(etag.to_str().unwrap_or(""));
        }

        // Check for redirect
        if status.is_redirection() {
            if let Some(location) = headers.get("location") {
                metadata["redirect_location"] = json!(location.to_str().unwrap_or(""));
            }
        }

        Ok(metadata)
    }
}

#[async_trait]
impl Tool for UrlAnalyzerTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "URL to analyze".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "fetch_metadata".to_string(),
            param_type: ParameterType::Boolean,
            description: "Fetch HTTP metadata (headers, status) for the URL".to_string(),
            required: false,
            default: Some(json!(false)),
        })
        .with_parameter(ParameterDef {
            name: "decode_params".to_string(),
            param_type: ParameterType::Boolean,
            description: "URL decode query parameters".to_string(),
            required: false,
            default: Some(json!(true)),
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
impl BaseAgent for UrlAnalyzerTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("UrlAnalyzer error: {}", error)))
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params = extract_parameters(&input)?;
        let url_str = extract_required_string(params, "input")?;
        let fetch_metadata = extract_optional_bool(params, "fetch_metadata").unwrap_or(false);
        let _decode_params = extract_optional_bool(params, "decode_params").unwrap_or(true);

        // Parse and validate URL
        let url = match Url::parse(url_str) {
            Ok(u) => u,
            Err(e) => {
                // Check if it's a relative URL error
                if e.to_string().contains("relative URL without a base") {
                    return Err(validation_error(
                        "Relative URLs are not supported. Please provide an absolute URL with scheme (http:// or https://)".to_string(),
                        Some("input".to_string()),
                    ));
                }
                return Err(validation_error(
                    format!("Invalid URL: {}", e),
                    Some("input".to_string()),
                ));
            }
        };

        // Validate scheme - only allow HTTP and HTTPS
        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(validation_error(
                format!(
                    "Unsupported URL scheme '{}'. Only http:// and https:// are supported.",
                    url.scheme()
                ),
                Some("input".to_string()),
            ));
        }

        let mut result = json!({
            "valid": true,
            "original": url_str,
            "normalized": url.to_string(),
            "scheme": url.scheme(),
            "host": url.host_str(),
            "port": url.port(),
            "path": url.path(),
            "query": url.query(),
            "fragment": url.fragment(),
            "query_params": {}
        });

        // Add auth info if present
        if !url.username().is_empty() {
            result["username"] = json!(url.username());
            result["has_auth"] = json!(true);
            if url.password().is_some() {
                result["has_password"] = json!(true);
            }
        }

        // Extract host details
        if let Some(host) = url.host() {
            let host_info = match host {
                Host::Domain(domain) => json!({
                    "type": "domain",
                    "value": domain,
                    "is_ip": false,
                }),
                Host::Ipv4(ip) => json!({
                    "type": "ipv4",
                    "value": ip.to_string(),
                    "is_ip": true,
                }),
                Host::Ipv6(ip) => json!({
                    "type": "ipv6",
                    "value": ip.to_string(),
                    "is_ip": true,
                }),
            };
            result["host_details"] = host_info;
        }

        // Parse query parameters
        if url.query().is_some() {
            let mut query_params = serde_json::Map::new();
            for (k, v) in url.query_pairs() {
                let key = k.to_string();
                let value = v.to_string(); // url.query_pairs() handles decoding
                query_params.insert(key, json!(value));
            }
            result["query_params"] = json!(query_params);
        }

        // Fetch metadata if requested
        if fetch_metadata && (url.scheme() == "http" || url.scheme() == "https") {
            match self.fetch_url_metadata(&url).await {
                Ok(metadata) => {
                    result["metadata"] = metadata;
                }
                Err(e) => {
                    result["metadata_error"] = json!(e.to_string());
                }
            }
        }

        let response = ResponseBuilder::success("analyze")
            .with_result(result)
            .build();

        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }
}
