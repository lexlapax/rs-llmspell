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
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown HTTP method: {}", s),
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
        SharedRetryConfig::new(config.max_attempts)
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
    pub fn new(config: HttpRequestConfig) -> Result<Self> {
        // Create rate limiter using shared utility
        let rate_limiter = if let Some(rpm) = config.rate_limit_per_minute {
            Some(
                RateLimiterBuilder::default()
                    .per_minute(rpm)
                    .sliding_window()
                    .build()
                    .map_err(|e| LLMSpellError::Internal {
                        message: format!("Failed to create rate limiter: {}", e),
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
                message: format!("Failed to create HTTP client: {}", e),
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
                    message: format!("Rate limit exceeded: {}", e),
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
                message: format!("HTTP request failed: {}", e),
                tool_name: Some("http_request".to_string()),
                source: None,
            })
        })
        .await;

        match result {
            Ok(response) => Ok(response),
            Err(retry_error) => Err(LLMSpellError::Tool {
                message: format!("HTTP request failed: {}", retry_error),
                tool_name: Some("http_request".to_string()),
                source: None,
            }),
        }
    }

    /// Parse response based on content type
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
            match response.json::<Value>().await {
                Ok(json) => ResponseBody::Json(json),
                Err(_) => ResponseBody::Text(String::new()),
            }
        } else if content_type.contains("text/") || content_type.contains("xml") {
            match response.text().await {
                Ok(text) => ResponseBody::Text(text),
                Err(_) => ResponseBody::Text(String::new()),
            }
        } else {
            // Binary content
            match response.bytes().await {
                Ok(bytes) => ResponseBody::Binary(bytes.to_vec()),
                Err(_) => ResponseBody::Binary(Vec::new()),
            }
        };

        Ok(HttpResponse {
            status_code: status.as_u16(),
            headers,
            body,
            timestamp: Utc::now(),
        })
    }

    /// Parse parameters from input
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

        let auth = if let Some(auth_obj) = params.get("auth") {
            serde_json::from_value(auth_obj.clone()).unwrap_or(AuthType::None)
        } else {
            AuthType::None
        };

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
                message: format!("Request timeout: {}", e),
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
        Ok(AgentOutput::text(format!("HTTP request error: {}", error)))
    }
}
