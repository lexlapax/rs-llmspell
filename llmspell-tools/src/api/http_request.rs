//! ABOUTME: HTTP request tool with authentication, retry logic, rate limiting, and automatic parsing
//! ABOUTME: Provides comprehensive HTTP client capabilities for API interactions

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use reqwest::{Client, Method, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};
use tracing::{debug, info, warn};

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

/// Retry configuration
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

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Maximum requests per window
    max_requests: u32,
    /// Time window in seconds
    window_seconds: u64,
    /// Current request count
    request_count: Arc<Mutex<u32>>,
    /// Window start time
    window_start: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            request_count: Arc::new(Mutex::new(0)),
            window_start: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub async fn acquire(&self) -> Result<()> {
        let mut count = self.request_count.lock().await;
        let mut start = self.window_start.lock().await;

        // Check if window has expired
        if start.elapsed().as_secs() >= self.window_seconds {
            *count = 0;
            *start = Instant::now();
        }

        // Check rate limit
        if *count >= self.max_requests {
            let remaining = self.window_seconds - start.elapsed().as_secs();
            return Err(LLMSpellError::RateLimit {
                message: format!("Rate limit exceeded. Try again in {} seconds", remaining),
                retry_after: Some(remaining),
            });
        }

        *count += 1;
        Ok(())
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

/// HTTP request tool with advanced features
pub struct HttpRequestTool {
    metadata: ComponentMetadata,
    config: HttpRequestConfig,
    client: Client,
    rate_limiter: Option<RateLimiter>,
}

impl HttpRequestTool {
    pub fn new(config: HttpRequestConfig) -> Result<Self> {
        let rate_limiter = config
            .rate_limit_per_minute
            .map(|rpm| RateLimiter::new(rpm, 60));

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

    /// Execute request with retry logic
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
        let mut attempt = 0;
        let mut delay_ms = retry_cfg.initial_delay_ms;

        loop {
            attempt += 1;
            debug!(
                "HTTP request attempt {}/{}",
                attempt, retry_cfg.max_attempts
            );

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
            match request.send().await {
                Ok(response) => {
                    let status = response.status();

                    // Check if we should retry based on status code
                    if attempt < retry_cfg.max_attempts
                        && retry_cfg.retry_on_status.contains(&status.as_u16())
                    {
                        warn!(
                            "Request failed with status {}. Retrying after {} ms",
                            status, delay_ms
                        );
                        sleep(Duration::from_millis(delay_ms)).await;
                        delay_ms = (delay_ms as f64 * retry_cfg.backoff_factor) as u64;
                        delay_ms = delay_ms.min(retry_cfg.max_delay_ms);
                        continue;
                    }

                    return Ok(response);
                }
                Err(e) => {
                    if attempt >= retry_cfg.max_attempts {
                        return Err(LLMSpellError::Tool {
                            message: format!(
                                "HTTP request failed after {} attempts: {}",
                                attempt, e
                            ),
                            tool_name: Some("http_request".to_string()),
                            source: None,
                        });
                    }

                    warn!("Request error: {}. Retrying after {} ms", e, delay_ms);
                    sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms = (delay_ms as f64 * retry_cfg.backoff_factor) as u64;
                    delay_ms = delay_ms.min(retry_cfg.max_delay_ms);
                }
            }
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
        let method_str = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
        let method: HttpMethod = method_str.parse()?;

        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing required parameter 'url'".to_string(),
                field: Some("url".to_string()),
            })?
            .to_string();

        let headers = params
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
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

#[derive(Debug)]
struct HttpRequestParams {
    method: HttpMethod,
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<Value>,
    auth: AuthType,
    retry_config: Option<RetryConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    body: ResponseBody,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ResponseBody {
    Json(Value),
    Text(String),
    Binary(Vec<u8>),
}

impl Default for HttpRequestTool {
    fn default() -> Self {
        Self::new(HttpRequestConfig::default()).expect("Default config should be valid")
    }
}

#[async_trait]
impl BaseAgent for HttpRequestTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params =
            input
                .parameters
                .get("parameters")
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing parameters".to_string(),
                    field: Some("parameters".to_string()),
                })?;

        let request_params = self.parse_parameters(params)?;

        info!(
            "Executing HTTP {} request to {}",
            request_params.method, request_params.url
        );

        // Check rate limit
        if let Some(limiter) = &self.rate_limiter {
            limiter.acquire().await?;
        }

        // Convert method
        let method = match request_params.method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Patch => Method::PATCH,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Options => Method::OPTIONS,
        };

        // Execute request with retry
        let response = self
            .execute_with_retry(
                method,
                &request_params.url,
                request_params.headers,
                request_params.body,
                request_params.auth,
                request_params.retry_config,
            )
            .await?;

        // Parse response
        let http_response = self.parse_response(response).await?;

        // Create output
        let output_value = serde_json::to_value(&http_response)?;
        let output_text = serde_json::to_string_pretty(&output_value)?;

        // Create metadata
        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata.extra.insert(
            "status_code".to_string(),
            Value::Number(http_response.status_code.into()),
        );
        metadata.extra.insert(
            "method".to_string(),
            Value::String(request_params.method.to_string()),
        );
        metadata
            .extra
            .insert("url".to_string(), Value::String(request_params.url));

        Ok(AgentOutput::text(output_text).with_metadata(metadata))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
        }

        // Validate URL is present
        if let Some(params) = input.parameters.get("parameters") {
            if params.get("url").is_none() {
                return Err(LLMSpellError::Validation {
                    message: "Missing required parameter 'url'".to_string(),
                    field: Some("url".to_string()),
                });
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("HTTP request error: {}", error)))
    }
}

#[async_trait]
impl Tool for HttpRequestTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Api
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Privileged
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "http_request".to_string(),
            description: "Execute HTTP requests with authentication, retries, and rate limiting"
                .to_string(),
            parameters: vec![
                ParameterDef {
                    name: "method".to_string(),
                    description: "HTTP method: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS"
                        .to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(serde_json::json!("GET")),
                },
                ParameterDef {
                    name: "url".to_string(),
                    description: "URL to request".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ParameterDef {
                    name: "headers".to_string(),
                    description: "HTTP headers as key-value pairs".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "body".to_string(),
                    description: "Request body (for POST, PUT, PATCH)".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "auth".to_string(),
                    description: "Authentication configuration".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "retry".to_string(),
                    description: "Retry configuration".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
            ],
            returns: Some(ParameterType::Object),
        }
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements {
            level: SecurityLevel::Privileged,
            file_permissions: vec![],
            network_permissions: vec!["*".to_string()],
            env_permissions: vec![],
            custom_requirements: HashMap::new(),
        }
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(100 * 1024 * 1024) // 100MB
            .with_cpu_limit(self.config.timeout_seconds * 1000) // Convert to ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_parsing() {
        assert_eq!("GET".parse::<HttpMethod>().unwrap(), HttpMethod::Get);
        assert_eq!("post".parse::<HttpMethod>().unwrap(), HttpMethod::Post);
        assert_eq!("PUT".parse::<HttpMethod>().unwrap(), HttpMethod::Put);
        assert!("INVALID".parse::<HttpMethod>().is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, 1); // 2 requests per second

        // First two should succeed
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());

        // Third should fail
        assert!(limiter.acquire().await.is_err());

        // Wait for window to expire
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Should succeed again
        assert!(limiter.acquire().await.is_ok());
    }

    #[tokio::test]
    async fn test_http_request_tool_creation() {
        let config = HttpRequestConfig::default();
        let tool = HttpRequestTool::new(config).unwrap();

        assert_eq!(tool.metadata().name, "http-request-tool");
    }
}
