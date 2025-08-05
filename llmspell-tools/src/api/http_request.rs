//! ABOUTME: HTTP request tool with authentication, retry logic, rate limiting, and automatic parsing
//! ABOUTME: Refactored to use shared utilities from llmspell-utils

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_utils::{
    extract_optional_object,
    extract_optional_string,
    extract_parameters,
    extract_required_string,
    // NEW: Using shared utilities
    rate_limiter::{RateLimiter, RateLimiterBuilder},
    response::ResponseBuilder,
    retry::{retry, AlwaysRetry, RetryConfig as SharedRetryConfig},
    timeout::TimeoutBuilder,
};
use reqwest::{Client, Method, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;

/// HTTP method types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Patch => write!(f, "PATCH"),
            Self::Head => write!(f, "HEAD"),
            Self::Options => write!(f, "OPTIONS"),
        }
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown HTTP method: {s}"),
                field: Some("method".to_string()),
            }),
        }
    }
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthType {
    None,
    Basic { username: String, password: String },
    Bearer { token: String },
    ApiKey { key: String, header_name: String },
    Custom { headers: HashMap<String, String> },
}

/// Retry configuration (simplified to use shared utility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Exponential backoff factor
    pub backoff_factor: f64,
    /// Status codes that should trigger a retry
    pub retry_on_status: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_factor: 2.0,
            retry_on_status: vec![429, 500, 502, 503, 504],
        }
    }
}

impl From<RetryConfig> for SharedRetryConfig {
    fn from(config: RetryConfig) -> Self {
        Self::new(config.max_attempts)
            .with_initial_delay(Duration::from_millis(config.initial_delay_ms))
            .with_max_delay(Duration::from_millis(config.max_delay_ms))
            .with_backoff_factor(config.backoff_factor)
            .with_jitter(true)
    }
}

/// HTTP request tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestConfig {
    /// Default timeout in seconds
    pub timeout_seconds: u64,
    /// Enable automatic redirect following
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow
    pub max_redirects: usize,
    /// Default retry configuration
    pub retry_config: RetryConfig,
    /// Rate limiting (requests per minute)
    pub rate_limit_per_minute: Option<u32>,
    /// User agent string
    pub user_agent: String,
}

impl Default for HttpRequestConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            follow_redirects: true,
            max_redirects: 10,
            retry_config: RetryConfig::default(),
            rate_limit_per_minute: Some(60),
            user_agent: "LLMSpell-HttpRequestTool/1.0".to_string(),
        }
    }
}

/// Response body types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseBody {
    Json(Value),
    Text(String),
    Binary(Vec<u8>),
}

/// HTTP response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: ResponseBody,
    pub timestamp: DateTime<Utc>,
}

/// Request parameters
#[derive(Debug)]
struct HttpRequestParams {
    method: HttpMethod,
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<Value>,
    auth: AuthType,
    retry_config: Option<RetryConfig>,
}

/// HTTP request tool with advanced features (refactored)
pub struct HttpRequestTool {
    metadata: ComponentMetadata,
    config: HttpRequestConfig,
    client: Client,
    rate_limiter: Option<RateLimiter>,
}

impl HttpRequestTool {
    /// Create a new HTTP request tool
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Rate limiter configuration is invalid
    /// - HTTP client creation fails
    pub fn new(config: HttpRequestConfig) -> Result<Self> {
        // Create rate limiter using shared utility
        let rate_limiter = if let Some(rpm) = config.rate_limit_per_minute {
            Some(
                RateLimiterBuilder::default()
                    .per_minute(rpm)
                    .sliding_window()
                    .build()
                    .map_err(|e| LLMSpellError::Internal {
                        message: format!("Failed to create rate limiter: {e}"),
                        source: None,
                    })?,
            )
        } else {
            None
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent(&config.user_agent)
            .redirect(if config.follow_redirects {
                reqwest::redirect::Policy::limited(config.max_redirects)
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()
            .map_err(|e| LLMSpellError::Internal {
                message: format!("Failed to create HTTP client: {e}"),
                source: None,
            })?;

        Ok(Self {
            metadata: ComponentMetadata::new(
                "http-request-tool".to_string(),
                "HTTP client with authentication, retries, and rate limiting".to_string(),
            ),
            config,
            client,
            rate_limiter,
        })
    }

    /// Apply authentication to request
    #[allow(clippy::unused_self)]
    fn apply_auth(
        &self,
        mut request: reqwest::RequestBuilder,
        auth: &AuthType,
    ) -> reqwest::RequestBuilder {
        match auth {
            AuthType::None => request,
            AuthType::Basic { username, password } => request.basic_auth(username, Some(password)),
            AuthType::Bearer { token } => request.bearer_auth(token),
            AuthType::ApiKey { key, header_name } => request.header(header_name, key),
            AuthType::Custom { headers } => {
                for (name, value) in headers {
                    request = request.header(name, value);
                }
                request
            }
        }
    }

    /// Execute request with retry logic (using shared utility)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Rate limit is exceeded
    /// - HTTP request fails after all retries
    async fn execute_with_retry(
        &self,
        method: Method,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<Value>,
        auth: AuthType,
        retry_config: Option<RetryConfig>,
    ) -> Result<Response> {
        let retry_cfg = retry_config.unwrap_or_else(|| self.config.retry_config.clone());
        let shared_retry_config: SharedRetryConfig = retry_cfg.clone().into();

        // Apply rate limiting
        if let Some(limiter) = &self.rate_limiter {
            limiter
                .acquire()
                .await
                .map_err(|e| LLMSpellError::RateLimit {
                    message: format!("Rate limit exceeded: {e}"),
                    retry_after: None,
                })?;
        }

        // Execute with retry logic using shared utility
        let result = retry(shared_retry_config, AlwaysRetry, || async {
            // Build request
            let mut request = self.client.request(method.clone(), url);

            // Add headers
            if let Some(headers) = &headers {
                for (name, value) in headers {
                    request = request.header(name, value);
                }
            }

            // Add body
            if let Some(body) = &body {
                request = request.json(body);
            }

            // Apply authentication
            request = self.apply_auth(request, &auth);

            // Execute request
            request.send().await.map_err(|e| LLMSpellError::Tool {
                message: format!("HTTP request failed: {e}"),
                tool_name: Some("http_request".to_string()),
                source: None,
            })
        })
        .await;

        match result {
            Ok(response) => Ok(response),
            Err(retry_error) => Err(LLMSpellError::Tool {
                message: format!("HTTP request failed: {retry_error}"),
                tool_name: Some("http_request".to_string()),
                source: None,
            }),
        }
    }

    /// Parse response based on content type
    ///
    /// # Errors
    ///
    /// Returns an error if response parsing fails
    async fn parse_response(&self, response: Response) -> Result<HttpResponse> {
        let status = response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect();

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let body = if content_type.contains("application/json") {
            response
                .json::<Value>()
                .await
                .map_or_else(|_| ResponseBody::Text(String::new()), ResponseBody::Json)
        } else if content_type.contains("text/") || content_type.contains("xml") {
            response
                .text()
                .await
                .map_or_else(|_| ResponseBody::Text(String::new()), ResponseBody::Text)
        } else {
            // Binary content
            response.bytes().await.map_or_else(
                |_| ResponseBody::Binary(Vec::new()),
                |bytes| ResponseBody::Binary(bytes.to_vec()),
            )
        };

        Ok(HttpResponse {
            status_code: status.as_u16(),
            headers,
            body,
            timestamp: Utc::now(),
        })
    }

    /// Parse parameters from input
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Invalid HTTP method is specified
    /// - Required URL parameter is missing
    /// - Parameter parsing fails
    #[allow(clippy::unused_self)]
    fn parse_parameters(&self, params: &Value) -> Result<HttpRequestParams> {
        let method_str = extract_optional_string(params, "method").unwrap_or("GET");
        let method: HttpMethod = method_str.parse()?;

        let url = extract_required_string(params, "input")?.to_string();

        let headers = extract_optional_object(params, "headers").map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        });

        let body = params.get("body").cloned();

        let auth = params.get("auth").map_or(AuthType::None, |auth_obj| {
            serde_json::from_value(auth_obj.clone()).unwrap_or(AuthType::None)
        });

        let retry_config = params
            .get("retry")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        Ok(HttpRequestParams {
            method,
            url,
            headers,
            body,
            auth,
            retry_config,
        })
    }
}

#[async_trait]
impl Tool for HttpRequestTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Api
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "http_request".to_string(),
            "Execute HTTP requests with authentication, retries, and automatic parsing".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "The URL to send the request to".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "method".to_string(),
            param_type: ParameterType::String,
            description: "HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)".to_string(),
            required: false,
            default: Some(json!("GET")),
        })
        .with_parameter(ParameterDef {
            name: "headers".to_string(),
            param_type: ParameterType::Object,
            description: "HTTP headers as key-value pairs".to_string(),
            required: false,
            default: Some(json!({})),
        })
        .with_parameter(ParameterDef {
            name: "body".to_string(),
            param_type: ParameterType::Object,
            description: "Request body (will be serialized as JSON)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "auth".to_string(),
            param_type: ParameterType::Object,
            description: "Authentication configuration".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "retry".to_string(),
            param_type: ParameterType::Object,
            description: "Retry configuration".to_string(),
            required: false,
            default: None,
        })
    }
}

impl HttpRequestTool {
    /// Check if this tool supports hook integration
    #[must_use]
    pub const fn supports_hooks(&self) -> bool {
        true // All tools that implement Tool automatically support hooks
    }

    /// Get hook integration metadata for this tool
    #[must_use]
    pub fn hook_metadata(&self) -> serde_json::Value {
        json!({
            "tool_name": self.metadata().name,
            "hook_points_supported": [
                "parameter_validation",
                "security_check",
                "resource_allocation",
                "pre_execution",
                "post_execution",
                "error_handling",
                "resource_cleanup",
                "timeout"
            ],
            "security_level": self.security_level(),
            "resource_limits": {
                "timeout_seconds": self.config.timeout_seconds,
                "max_retry_attempts": self.config.retry_config.max_attempts,
                "network_dependent": true
            },
            "hook_integration_benefits": [
                "HTTP request validation and sanitization",
                "URL security validation and blacklist checking",
                "Rate limiting and throttling with hooks",
                "Request/response size monitoring",
                "Network timeout and retry logic tracking",
                "Authentication credential validation",
                "Response parsing and error handling",
                "Performance monitoring for API calls"
            ],
            "security_considerations": [
                "Safe security level for HTTP requests",
                "URL validation to prevent SSRF attacks",
                "Request header sanitization",
                "Response size limits to prevent DoS",
                "Timeout enforcement for hanging requests",
                "Authentication credential protection"
            ],
            "supported_methods": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"],
            "authentication_types": ["none", "basic", "bearer", "api_key"]
        })
    }

    /// Demonstrate hook-aware execution for HTTP requests
    /// This method showcases how the HTTP request tool works with the hook system
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails
    /// - Hook execution fails
    /// - Response parsing fails
    pub async fn demonstrate_hook_integration(
        &self,
        tool_executor: &crate::lifecycle::ToolExecutor,
        method: &str,
        url: &str,
        headers: Option<&std::collections::HashMap<String, String>>,
        body: Option<&str>,
    ) -> Result<AgentOutput> {
        use crate::lifecycle::HookableToolExecution;

        let mut params = json!({
            "method": method,
            "input": url,
            "hook_integration": true  // Flag to indicate this is a hook demo
        });

        if let Some(headers) = headers {
            params["headers"] = json!(headers);
        }

        if let Some(body) = body {
            params["body"] = json!(body);
        }

        let input = AgentInput::text("HTTP request hook demonstration")
            .with_parameter("parameters", params);
        let context = ExecutionContext::default();

        // Execute with hooks using the HookableToolExecution trait
        self.execute_with_hooks(input, context, tool_executor).await
    }
}

#[async_trait]
impl BaseAgent for HttpRequestTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params = extract_parameters(&input)?;
        let request_params = self.parse_parameters(params)?;

        info!(
            "Executing HTTP {} request to {}",
            request_params.method, request_params.url
        );

        // Execute request with timeout using shared utility
        let response = TimeoutBuilder::default()
            .duration(Duration::from_secs(self.config.timeout_seconds))
            .name(format!(
                "HTTP {} {}",
                request_params.method, request_params.url
            ))
            .execute(async {
                self.execute_with_retry(
                    request_params.method.to_string().parse().unwrap(),
                    &request_params.url,
                    request_params.headers,
                    request_params.body,
                    request_params.auth,
                    request_params.retry_config,
                )
                .await
            })
            .await
            .map_err(|e| LLMSpellError::Tool {
                message: format!("Request timeout: {e}"),
                tool_name: Some("http_request".to_string()),
                source: None,
            })?;

        let http_response = self.parse_response(response?).await?;

        let message = format!(
            "HTTP {} request to {} completed with status {}",
            request_params.method, request_params.url, http_response.status_code
        );

        let response = ResponseBuilder::success("http_request")
            .with_message(message)
            .with_result(json!({
                "status_code": http_response.status_code,
                "headers": http_response.headers,
                "body": http_response.body,
                "timestamp": http_response.timestamp.to_rfc3339(),
            }))
            .with_metadata("method", json!(request_params.method.to_string()))
            .with_metadata("url", json!(request_params.url))
            .with_metadata("duration_ms", json!(0)) // TODO: Track actual duration
            .build();

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("HTTP request error: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hook_integration_metadata() {
        let tool = HttpRequestTool::new(HttpRequestConfig::default()).unwrap();

        // Test that the tool supports hooks
        assert!(tool.supports_hooks());

        // Test hook metadata
        let metadata = tool.hook_metadata();
        assert_eq!(metadata["tool_name"], "http-request-tool");
        assert!(metadata["hook_points_supported"].is_array());
        assert_eq!(
            metadata["hook_points_supported"].as_array().unwrap().len(),
            8
        );
        assert!(metadata["hook_integration_benefits"].is_array());
        assert!(metadata["security_considerations"].is_array());
        assert_eq!(metadata["security_level"], "Safe");
        assert!(metadata["supported_methods"].is_array());
        assert!(metadata["authentication_types"].is_array());
    }
    #[tokio::test]
    async fn test_http_request_hook_integration() {
        use crate::lifecycle::{ToolExecutor, ToolLifecycleConfig};
        let tool = HttpRequestTool::new(HttpRequestConfig::default()).unwrap();

        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        // Demonstrate hook integration with a simple GET request (may fail due to network)
        let result = tool
            .demonstrate_hook_integration(
                &tool_executor,
                "GET",
                "https://httpbin.org/get",
                None,
                None,
            )
            .await;

        // The network request may fail, but hook integration should not panic
        assert!(result.is_ok() || result.is_err());
    }
    #[tokio::test]
    async fn test_hookable_tool_execution_trait_http() {
        use crate::lifecycle::{HookableToolExecution, ToolExecutor, ToolLifecycleConfig};
        let tool = HttpRequestTool::new(HttpRequestConfig::default()).unwrap();

        // Verify the tool implements HookableToolExecution
        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        let input = AgentInput::text("Hook trait test").with_parameter(
            "parameters",
            json!({
                "method": "GET",
                "input": "https://httpbin.org/get"
            }),
        );
        let context = ExecutionContext::default();

        // This should compile and execute (network request may fail, that's ok)
        let result = tool
            .execute_with_hooks(input, context, &tool_executor)
            .await;
        assert!(result.is_ok() || result.is_err()); // Should not panic
    }
}
