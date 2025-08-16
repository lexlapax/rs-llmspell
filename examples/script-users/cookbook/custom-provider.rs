// ABOUTME: Custom provider implementation patterns for extending LLM provider support
// ABOUTME: Demonstrates how to implement custom providers following the llmspell provider trait

//! # Custom Provider Patterns
//! 
//! This module demonstrates how to implement custom LLM providers for llmspell,
//! including provider registration, request handling, streaming responses, and error management.
//! 
//! ## Key Patterns
//! 
//! 1. **Provider Implementation**: Core trait implementation for custom providers
//! 2. **Configuration Management**: Handling provider-specific configuration
//! 3. **Request Processing**: HTTP client setup and request formatting
//! 4. **Response Handling**: Parsing responses and error handling
//! 5. **Streaming Support**: Implementing real-time streaming responses
//! 6. **Authentication**: Various authentication patterns (API keys, OAuth, etc.)
//! 7. **Rate Limiting**: Implementing provider-specific rate limiting
//! 8. **Retry Logic**: Robust retry mechanisms with backoff strategies

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Configuration for custom LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProviderConfig {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout: Duration,
    pub max_retries: u32,
    pub rate_limit: Option<RateLimit>,
    pub default_model: String,
    pub supported_models: Vec<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub concurrent_requests: u32,
}

/// Request structure for LLM API calls
#[derive(Debug, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Response structure from LLM providers
#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub created: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Streaming response chunk
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub model: String,
    pub choices: Vec<StreamChoice>,
    pub created: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}

/// Error types for custom providers
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Provider unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Core trait for implementing custom LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider identification
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn supported_models(&self) -> &[String];
    
    /// Health check for provider availability
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError>;
    
    /// Make a completion request
    async fn complete(
        &self,
        request: LlmRequest,
    ) -> Result<LlmResponse, ProviderError>;
    
    /// Stream completion responses
    async fn stream_complete(
        &self,
        request: LlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamChunk, ProviderError>> + Unpin>, ProviderError>;
    
    /// Estimate token count for text
    fn estimate_tokens(&self, text: &str) -> u32;
    
    /// Get provider configuration
    fn config(&self) -> &CustomProviderConfig;
}

/// Provider health status
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub available_models: Vec<String>,
    pub rate_limit_remaining: Option<u32>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Pattern 1: OpenAI-Compatible Provider
/// 
/// This demonstrates implementing a provider for OpenAI-compatible APIs
/// (like LocalAI, Ollama, or other compatible services)
pub struct OpenAiCompatibleProvider {
    config: CustomProviderConfig,
    client: reqwest::Client,
    rate_limiter: RateLimiter,
}

impl OpenAiCompatibleProvider {
    pub fn new(config: CustomProviderConfig) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        let rate_limiter = RateLimiter::new(config.rate_limit.clone());
        
        Ok(Self {
            config,
            client,
            rate_limiter,
        })
    }
    
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Add API key if available
        if let Some(api_key) = &self.config.api_key {
            headers.insert(
                "Authorization",
                format!("Bearer {}", api_key).parse().unwrap(),
            );
        }
        
        // Add custom headers
        for (key, value) in &self.config.headers {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(header_value) = reqwest::header::HeaderValue::from_str(value) {
                    headers.insert(header_name, header_value);
                }
            }
        }
        
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }
    
    async fn make_request_with_retry<T, F, Fut>(
        &self,
        request_fn: F,
    ) -> Result<T, ProviderError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, ProviderError>>,
    {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts <= self.config.max_retries {
            // Check rate limit
            self.rate_limiter.wait_if_needed().await?;
            
            match request_fn().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    
                    if attempts <= self.config.max_retries {
                        // Exponential backoff
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempts - 1));
                        sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            ProviderError::ServiceUnavailable("Max retries exceeded".to_string())
        }))
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn supported_models(&self) -> &[String] {
        &self.config.supported_models
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        let start = Instant::now();
        
        let health_url = format!("{}/v1/models", self.config.base_url);
        let response = self.client
            .get(&health_url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        let status = match response.status() {
            reqwest::StatusCode::OK => HealthStatus::Healthy,
            reqwest::StatusCode::TOO_MANY_REQUESTS => HealthStatus::Degraded,
            _ => HealthStatus::Unhealthy,
        };
        
        let rate_limit_remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());
        
        Ok(ProviderHealth {
            status,
            latency_ms,
            available_models: self.config.supported_models.clone(),
            rate_limit_remaining,
            message: Some("OpenAI-compatible provider health check".to_string()),
        })
    }
    
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, ProviderError> {
        self.make_request_with_retry(|| async {
            let url = format!("{}/v1/chat/completions", self.config.base_url);
            
            let response = self.client
                .post(&url)
                .headers(self.build_headers())
                .json(&request)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
            
            if !response.status().is_success() {
                return Err(self.handle_error_response(response).await);
            }
            
            let llm_response: LlmResponse = response
                .json()
                .await
                .map_err(|e| ProviderError::ParseError(e.to_string()))?;
            
            Ok(llm_response)
        }).await
    }
    
    async fn stream_complete(
        &self,
        mut request: LlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamChunk, ProviderError>> + Unpin>, ProviderError> {
        request.stream = true;
        
        let url = format!("{}/v1/chat/completions", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }
        
        // In a real implementation, this would parse Server-Sent Events
        // For now, we'll create a simple mock stream
        let stream = futures::stream::iter(vec![
            Ok(StreamChunk {
                id: "chatcmpl-123".to_string(),
                model: request.model.clone(),
                choices: vec![StreamChoice {
                    index: 0,
                    delta: Delta {
                        role: Some("assistant".to_string()),
                        content: Some("Hello".to_string()),
                    },
                    finish_reason: None,
                }],
                created: chrono::Utc::now().timestamp() as u64,
            }),
            Ok(StreamChunk {
                id: "chatcmpl-123".to_string(),
                model: request.model,
                choices: vec![StreamChoice {
                    index: 0,
                    delta: Delta {
                        role: None,
                        content: Some(" world!".to_string()),
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                created: chrono::Utc::now().timestamp() as u64,
            }),
        ]);
        
        Ok(Box::new(Box::pin(stream)))
    }
    
    fn estimate_tokens(&self, text: &str) -> u32 {
        // Simple token estimation (in production, use proper tokenizer)
        (text.len() as f32 / 4.0).ceil() as u32
    }
    
    fn config(&self) -> &CustomProviderConfig {
        &self.config
    }
}

impl OpenAiCompatibleProvider {
    async fn handle_error_response(&self, response: reqwest::Response) -> ProviderError {
        match response.status() {
            reqwest::StatusCode::UNAUTHORIZED => {
                ProviderError::AuthenticationError("Invalid API key".to_string())
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                ProviderError::RateLimitError("Rate limit exceeded".to_string())
            }
            reqwest::StatusCode::BAD_REQUEST => {
                let error_text = response.text().await.unwrap_or_default();
                ProviderError::InvalidRequest(error_text)
            }
            reqwest::StatusCode::SERVICE_UNAVAILABLE => {
                ProviderError::ServiceUnavailable("Service temporarily unavailable".to_string())
            }
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                ProviderError::NetworkError(format!("HTTP {}: {}", response.status(), error_text))
            }
        }
    }
}

/// Pattern 2: Rate Limiter Implementation
/// 
/// This demonstrates how to implement rate limiting for custom providers
pub struct RateLimiter {
    config: Option<RateLimit>,
    request_times: tokio::sync::Mutex<Vec<Instant>>,
    concurrent_requests: tokio::sync::Semaphore,
}

impl RateLimiter {
    pub fn new(config: Option<RateLimit>) -> Self {
        let concurrent_limit = config.as_ref()
            .map(|c| c.concurrent_requests as usize)
            .unwrap_or(10);
        
        Self {
            config,
            request_times: tokio::sync::Mutex::new(Vec::new()),
            concurrent_requests: tokio::sync::Semaphore::new(concurrent_limit),
        }
    }
    
    pub async fn wait_if_needed(&self) -> Result<(), ProviderError> {
        let Some(config) = &self.config else {
            return Ok(());
        };
        
        // Acquire semaphore for concurrent request limiting
        let _permit = self.concurrent_requests.acquire().await
            .map_err(|_| ProviderError::ServiceUnavailable("Too many concurrent requests".to_string()))?;
        
        let mut request_times = self.request_times.lock().await;
        let now = Instant::now();
        
        // Clean up old requests (older than 1 hour)
        request_times.retain(|&time| now.duration_since(time) < Duration::from_secs(3600));
        
        // Check requests per minute
        let minute_ago = now - Duration::from_secs(60);
        let recent_requests = request_times.iter()
            .filter(|&&time| time > minute_ago)
            .count() as u32;
        
        if recent_requests >= config.requests_per_minute {
            let wait_time = Duration::from_secs(60) - now.duration_since(request_times[0]);
            sleep(wait_time).await;
        }
        
        // Check requests per hour
        let hour_ago = now - Duration::from_secs(3600);
        let hourly_requests = request_times.iter()
            .filter(|&&time| time > hour_ago)
            .count() as u32;
        
        if hourly_requests >= config.requests_per_hour {
            return Err(ProviderError::RateLimitError(
                "Hourly rate limit exceeded".to_string(),
            ));
        }
        
        // Record this request
        request_times.push(now);
        
        Ok(())
    }
}

/// Pattern 3: Custom Authentication Provider
/// 
/// This demonstrates implementing custom authentication mechanisms
pub struct CustomAuthProvider {
    config: CustomProviderConfig,
    client: reqwest::Client,
    auth_token: tokio::sync::RwLock<Option<AuthToken>>,
}

#[derive(Debug, Clone)]
struct AuthToken {
    token: String,
    expires_at: Instant,
    token_type: String,
}

impl CustomAuthProvider {
    pub fn new(config: CustomProviderConfig) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        Ok(Self {
            config,
            client,
            auth_token: tokio::sync::RwLock::new(None),
        })
    }
    
    async fn ensure_valid_token(&self) -> Result<String, ProviderError> {
        // Check if we have a valid token
        {
            let token_guard = self.auth_token.read().await;
            if let Some(token) = &*token_guard {
                if token.expires_at > Instant::now() {
                    return Ok(format!("{} {}", token.token_type, token.token));
                }
            }
        }
        
        // Need to refresh token
        self.refresh_token().await
    }
    
    async fn refresh_token(&self) -> Result<String, ProviderError> {
        let auth_url = format!("{}/auth/token", self.config.base_url);
        
        let auth_request = serde_json::json!({
            "grant_type": "client_credentials",
            "client_id": self.config.api_key.as_ref().unwrap_or(&String::new()),
            "scope": "llm.complete"
        });
        
        let response = self.client
            .post(&auth_url)
            .json(&auth_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::AuthenticationError(
                "Failed to refresh token".to_string(),
            ));
        }
        
        let auth_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;
        
        let token = auth_response["access_token"]
            .as_str()
            .ok_or_else(|| ProviderError::ParseError("Missing access_token".to_string()))?;
        
        let expires_in = auth_response["expires_in"]
            .as_u64()
            .unwrap_or(3600);
        
        let token_type = auth_response["token_type"]
            .as_str()
            .unwrap_or("Bearer");
        
        let auth_token = AuthToken {
            token: token.to_string(),
            expires_at: Instant::now() + Duration::from_secs(expires_in - 60), // 1 minute buffer
            token_type: token_type.to_string(),
        };
        
        let auth_header = format!("{} {}", auth_token.token_type, auth_token.token);
        
        // Store the new token
        {
            let mut token_guard = self.auth_token.write().await;
            *token_guard = Some(auth_token);
        }
        
        Ok(auth_header)
    }
}

#[async_trait]
impl LlmProvider for CustomAuthProvider {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn supported_models(&self) -> &[String] {
        &self.config.supported_models
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        let start = Instant::now();
        
        let auth_header = self.ensure_valid_token().await?;
        
        let health_url = format!("{}/health", self.config.base_url);
        let response = self.client
            .get(&health_url)
            .header("Authorization", auth_header)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        let status = if response.status().is_success() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };
        
        Ok(ProviderHealth {
            status,
            latency_ms,
            available_models: self.config.supported_models.clone(),
            rate_limit_remaining: None,
            message: Some("Custom auth provider health check".to_string()),
        })
    }
    
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, ProviderError> {
        let auth_header = self.ensure_valid_token().await?;
        let url = format!("{}/completions", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::ServiceUnavailable(
                format!("Request failed with status: {}", response.status()),
            ));
        }
        
        let llm_response: LlmResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;
        
        Ok(llm_response)
    }
    
    async fn stream_complete(
        &self,
        _request: LlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamChunk, ProviderError>> + Unpin>, ProviderError> {
        // Simplified implementation
        Err(ProviderError::ServiceUnavailable(
            "Streaming not implemented for this provider".to_string(),
        ))
    }
    
    fn estimate_tokens(&self, text: &str) -> u32 {
        (text.len() as f32 / 4.0).ceil() as u32
    }
    
    fn config(&self) -> &CustomProviderConfig {
        &self.config
    }
}

/// Pattern 4: Provider Registry and Factory
/// 
/// This demonstrates how to register and manage multiple custom providers
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn LlmProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
    
    pub fn register_provider(&mut self, provider: Box<dyn LlmProvider>) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }
    
    pub fn get_provider(&self, name: &str) -> Option<&dyn LlmProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }
    
    pub fn list_providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
    
    pub async fn health_check_all(&self) -> HashMap<String, ProviderHealth> {
        let mut results = HashMap::new();
        
        for (name, provider) in &self.providers {
            match provider.health_check().await {
                Ok(health) => {
                    results.insert(name.clone(), health);
                }
                Err(_) => {
                    results.insert(name.clone(), ProviderHealth {
                        status: HealthStatus::Unhealthy,
                        latency_ms: 0,
                        available_models: vec![],
                        rate_limit_remaining: None,
                        message: Some("Health check failed".to_string()),
                    });
                }
            }
        }
        
        results
    }
}

/// Example usage and testing patterns
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    #[tokio::test]
    async fn test_openai_compatible_provider() {
        let config = CustomProviderConfig {
            name: "test-provider".to_string(),
            base_url: "https://api.example.com".to_string(),
            api_key: Some("test-key".to_string()),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            rate_limit: Some(RateLimit {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                concurrent_requests: 5,
            }),
            default_model: "gpt-3.5-turbo".to_string(),
            supported_models: vec!["gpt-3.5-turbo".to_string(), "gpt-4".to_string()],
            headers: HashMap::new(),
        };
        
        let provider = OpenAiCompatibleProvider::new(config).unwrap();
        assert_eq!(provider.name(), "test-provider");
        assert_eq!(provider.supported_models(), &["gpt-3.5-turbo", "gpt-4"]);
        
        let tokens = provider.estimate_tokens("Hello world");
        assert!(tokens > 0);
    }
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let rate_limit = RateLimit {
            requests_per_minute: 2,
            requests_per_hour: 10,
            concurrent_requests: 1,
        };
        
        let limiter = RateLimiter::new(Some(rate_limit));
        
        // First request should succeed
        assert!(limiter.wait_if_needed().await.is_ok());
        
        // Second request should succeed
        assert!(limiter.wait_if_needed().await.is_ok());
        
        // Third request should be rate limited (but we won't test the delay)
    }
    
    #[tokio::test]
    async fn test_provider_registry() {
        let mut registry = ProviderRegistry::new();
        
        let config = CustomProviderConfig {
            name: "test-provider".to_string(),
            base_url: "https://api.example.com".to_string(),
            api_key: Some("test-key".to_string()),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            rate_limit: None,
            default_model: "test-model".to_string(),
            supported_models: vec!["test-model".to_string()],
            headers: HashMap::new(),
        };
        
        let provider = OpenAiCompatibleProvider::new(config).unwrap();
        registry.register_provider(Box::new(provider));
        
        assert!(registry.get_provider("test-provider").is_some());
        assert_eq!(registry.list_providers(), vec!["test-provider"]);
    }
}

/// Key Takeaways for Custom Provider Implementation:
///
/// 1. **Trait Implementation**: Implement the core `LlmProvider` trait with all required methods
/// 2. **Configuration Management**: Use strongly-typed configuration structures
/// 3. **Error Handling**: Implement comprehensive error types and handling
/// 4. **Rate Limiting**: Implement provider-specific rate limiting and backoff
/// 5. **Authentication**: Support various authentication patterns (API keys, OAuth, custom)
/// 6. **Retry Logic**: Implement robust retry mechanisms with exponential backoff
/// 7. **Health Monitoring**: Provide health check capabilities for monitoring
/// 8. **Streaming Support**: Implement streaming responses for real-time interactions
/// 9. **Token Estimation**: Provide accurate token counting for cost estimation
/// 10. **Testing**: Write comprehensive tests for all provider functionality
///
/// This pattern allows for easy extension of llmspell with new LLM providers
/// while maintaining consistency and reliability across all implementations.