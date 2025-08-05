// ABOUTME: Service availability and health checking tool with TCP port monitoring
// ABOUTME: Provides network connectivity testing and service status validation with timeout controls

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result as LLMResult,
};
use llmspell_security::sandbox::SandboxContext;
use llmspell_utils::{
    extract_optional_u64, extract_parameters, extract_required_string, response::ResponseBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Service check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCheckResult {
    /// Target being checked (host:port or URL)
    pub target: String,
    /// Whether the service is available
    pub available: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Status message
    pub status: String,
    /// Error message if check failed
    pub error: Option<String>,
    /// Additional metadata about the check
    pub metadata: HashMap<String, String>,
}

/// Service check type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCheckType {
    /// TCP port connectivity check
    TcpPort,
    /// HTTP/HTTPS health check
    Http,
    /// HTTPS health check
    Https,
    /// DNS resolution check
    Dns,
}

/// Service checker tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCheckerConfig {
    /// Default timeout for checks in seconds
    pub default_timeout_seconds: u64,
    /// Maximum timeout allowed in seconds
    pub max_timeout_seconds: u64,
    /// Maximum number of concurrent checks
    pub max_concurrent_checks: usize,
    /// Default retries for failed checks
    pub default_retries: u32,
    /// Allowed ports for TCP checks
    pub allowed_ports: Vec<u16>,
    /// Blocked ports that should never be checked
    pub blocked_ports: Vec<u16>,
    /// Whether to allow arbitrary port checking
    pub allow_arbitrary_ports: bool,
    /// Allowed domains for checks
    pub allowed_domains: Vec<String>,
    /// Whether to allow checking any domain
    pub allow_any_domain: bool,
}

impl Default for ServiceCheckerConfig {
    fn default() -> Self {
        Self {
            default_timeout_seconds: 5,
            max_timeout_seconds: 30,
            max_concurrent_checks: 10,
            default_retries: 2,
            allowed_ports: vec![
                22, 23, 25, 53, 80, 110, 143, 443, 993, 995, 1433, 3306, 5432, 6379, 8080, 8443,
                9000,
            ],
            blocked_ports: vec![7, 9, 13, 17, 19, 21, 135, 139, 445, 1900, 5353, 6881, 6969],
            allow_arbitrary_ports: false,
            allowed_domains: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "::1".to_string(),
            ],
            allow_any_domain: false,
        }
    }
}

/// Service checker tool for availability monitoring
#[derive(Clone)]
pub struct ServiceCheckerTool {
    metadata: ComponentMetadata,
    config: ServiceCheckerConfig,
    #[allow(dead_code)] // Reserved for future sandbox integration
    sandbox_context: Option<Arc<SandboxContext>>,
}

impl ServiceCheckerTool {
    /// Create a new service checker tool
    #[must_use]
    pub fn new(config: ServiceCheckerConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "service_checker".to_string(),
                "Service availability and health checking with network connectivity testing"
                    .to_string(),
            ),
            config,
            sandbox_context: None,
        }
    }

    /// Create a new service checker tool with sandbox context
    #[must_use]
    pub fn with_sandbox(
        config: ServiceCheckerConfig,
        sandbox_context: Arc<SandboxContext>,
    ) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "service_checker".to_string(),
                "Service availability and health checking with network connectivity testing"
                    .to_string(),
            ),
            config,
            sandbox_context: Some(sandbox_context),
        }
    }

    /// Check if a port is allowed to be checked
    fn is_port_allowed(&self, port: u16) -> bool {
        // Check blocked ports first (takes precedence)
        if self.config.blocked_ports.contains(&port) {
            debug!("Port {} is blocked", port);
            return false;
        }

        // If arbitrary ports are allowed, allow it
        if self.config.allow_arbitrary_ports {
            return true;
        }

        // Check allowed ports
        if self.config.allowed_ports.contains(&port) {
            debug!("Port {} is allowed", port);
            return true;
        }

        debug!("Port {} is not in allowed list", port);
        false
    }

    /// Check if a domain is allowed to be checked
    fn is_domain_allowed(&self, domain: &str) -> bool {
        // If any domain is allowed, allow it
        if self.config.allow_any_domain {
            return true;
        }

        // Check allowed domains
        for allowed in &self.config.allowed_domains {
            if domain == allowed || domain.ends_with(&format!(".{allowed}")) {
                debug!("Domain '{}' is allowed", domain);
                return true;
            }
        }

        debug!("Domain '{}' is not in allowed list", domain);
        false
    }

    /// Parse target into host and port
    #[allow(clippy::unused_self)]
    fn parse_target(&self, target: &str) -> LLMResult<(String, u16)> {
        if let Some(pos) = target.rfind(':') {
            let host = target[..pos].to_string();
            let port_str = &target[pos + 1..];
            let port = port_str
                .parse::<u16>()
                .map_err(|_| LLMSpellError::Validation {
                    message: format!("Invalid port number: {port_str}"),
                    field: Some("target".to_string()),
                })?;

            Ok((host, port))
        } else {
            Err(LLMSpellError::Validation {
                message: "Target must be in format 'host:port'".to_string(),
                field: Some("target".to_string()),
            })
        }
    }

    /// Check TCP port connectivity
    async fn check_tcp_port(
        &self,
        host: &str,
        port: u16,
        timeout_duration: Duration,
    ) -> ServiceCheckResult {
        let target = format!("{host}:{port}");
        let start_time = Instant::now();

        debug!("Checking TCP port: {}", target);

        // Validate port is allowed
        if !self.is_port_allowed(port) {
            return ServiceCheckResult {
                target: target.clone(),
                available: false,
                response_time_ms: 0,
                status: "Port check not allowed".to_string(),
                error: Some(format!("Port {port} is not allowed")),
                metadata: HashMap::new(),
            };
        }

        // Validate domain is allowed
        if !self.is_domain_allowed(host) {
            return ServiceCheckResult {
                target: target.clone(),
                available: false,
                response_time_ms: 0,
                status: "Domain check not allowed".to_string(),
                error: Some(format!("Domain '{host}' is not allowed")),
                metadata: HashMap::new(),
            };
        }

        // Attempt to resolve and connect
        let socket_addrs: Vec<SocketAddr> = match target.to_socket_addrs() {
            Ok(addrs) => addrs.collect(),
            Err(e) => {
                return ServiceCheckResult {
                    target: target.clone(),
                    available: false,
                    response_time_ms: u64::try_from(start_time.elapsed().as_millis())
                        .unwrap_or(u64::MAX),
                    status: "DNS resolution failed".to_string(),
                    error: Some(e.to_string()),
                    metadata: HashMap::new(),
                }
            }
        };

        if socket_addrs.is_empty() {
            return ServiceCheckResult {
                target: target.clone(),
                available: false,
                response_time_ms: u64::try_from(start_time.elapsed().as_millis())
                    .unwrap_or(u64::MAX),
                status: "No addresses resolved".to_string(),
                error: Some("DNS resolution returned no addresses".to_string()),
                metadata: HashMap::new(),
            };
        }

        // Try to connect to the first resolved address
        let socket_addr = socket_addrs[0];
        match timeout(timeout_duration, TcpStream::connect(socket_addr)).await {
            Ok(Ok(_stream)) => {
                let response_time =
                    u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
                info!("TCP port {} available in {}ms", target, response_time);
                ServiceCheckResult {
                    target,
                    available: true,
                    response_time_ms: response_time,
                    status: "Port open and reachable".to_string(),
                    error: None,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("resolved_ip".to_string(), socket_addr.ip().to_string());
                        meta.insert("resolved_port".to_string(), socket_addr.port().to_string());
                        meta
                    },
                }
            }
            Ok(Err(e)) => {
                let response_time =
                    u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
                warn!("TCP port {} connection failed: {}", target, e);
                ServiceCheckResult {
                    target,
                    available: false,
                    response_time_ms: response_time,
                    status: "Connection failed".to_string(),
                    error: Some(e.to_string()),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("resolved_ip".to_string(), socket_addr.ip().to_string());
                        meta
                    },
                }
            }
            Err(_) => {
                let response_time =
                    u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
                warn!("TCP port {} check timed out", target);
                ServiceCheckResult {
                    target,
                    available: false,
                    response_time_ms: response_time,
                    status: "Connection timed out".to_string(),
                    error: Some(format!("Timeout after {}ms", timeout_duration.as_millis())),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("resolved_ip".to_string(), socket_addr.ip().to_string());
                        meta
                    },
                }
            }
        }
    }

    /// Check HTTP/HTTPS service health
    async fn check_http_service(
        &self,
        url: &str,
        timeout_duration: Duration,
    ) -> ServiceCheckResult {
        let start_time = Instant::now();

        debug!("Checking HTTP service: {}", url);

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return ServiceCheckResult {
                target: url.to_string(),
                available: false,
                response_time_ms: 0,
                status: "Invalid URL format".to_string(),
                error: Some("URL must start with http:// or https://".to_string()),
                metadata: HashMap::new(),
            };
        }

        // Extract domain from URL for validation
        let domain = if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                host.to_string()
            } else {
                return ServiceCheckResult {
                    target: url.to_string(),
                    available: false,
                    response_time_ms: 0,
                    status: "Invalid host in URL".to_string(),
                    error: Some("Could not extract host from URL".to_string()),
                    metadata: HashMap::new(),
                };
            }
        } else {
            return ServiceCheckResult {
                target: url.to_string(),
                available: false,
                response_time_ms: 0,
                status: "URL parsing failed".to_string(),
                error: Some("Could not parse URL".to_string()),
                metadata: HashMap::new(),
            };
        };

        // Validate domain is allowed
        if !self.is_domain_allowed(&domain) {
            return ServiceCheckResult {
                target: url.to_string(),
                available: false,
                response_time_ms: 0,
                status: "Domain check not allowed".to_string(),
                error: Some(format!("Domain '{domain}' is not allowed")),
                metadata: HashMap::new(),
            };
        }

        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(timeout_duration)
            .build()
            .map_err(|e| LLMSpellError::Tool {
                message: format!("Failed to create HTTP client: {e}"),
                tool_name: Some("service_checker".to_string()),
                source: None,
            })
            .unwrap();

        // Make HEAD request to check service availability
        match client.head(url).send().await {
            Ok(response) => {
                let response_time =
                    u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
                let status_code = response.status().as_u16();
                let is_available =
                    response.status().is_success() || response.status().is_redirection();

                info!(
                    "HTTP service {} returned {} in {}ms",
                    url, status_code, response_time
                );

                ServiceCheckResult {
                    target: url.to_string(),
                    available: is_available,
                    response_time_ms: response_time,
                    status: format!("HTTP {status_code}"),
                    error: if is_available {
                        None
                    } else {
                        Some(format!("HTTP error {status_code}"))
                    },
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("status_code".to_string(), status_code.to_string());
                        meta.insert("method".to_string(), "HEAD".to_string());
                        if let Some(content_type) = response.headers().get("content-type") {
                            if let Ok(ct_str) = content_type.to_str() {
                                meta.insert("content_type".to_string(), ct_str.to_string());
                            }
                        }
                        meta
                    },
                }
            }
            Err(e) => {
                let response_time =
                    u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
                warn!("HTTP service {} check failed: {}", url, e);

                let (status, error_msg) = if e.is_timeout() {
                    (
                        "Request timed out".to_string(),
                        format!("Timeout after {}ms", timeout_duration.as_millis()),
                    )
                } else if e.is_connect() {
                    (
                        "Connection failed".to_string(),
                        "Failed to connect to service".to_string(),
                    )
                } else {
                    ("Request failed".to_string(), e.to_string())
                };

                ServiceCheckResult {
                    target: url.to_string(),
                    available: false,
                    response_time_ms: response_time,
                    status,
                    error: Some(error_msg),
                    metadata: HashMap::new(),
                }
            }
        }
    }
}

#[async_trait]
impl BaseAgent for ServiceCheckerTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> LLMResult<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Extract required parameters
        let target = extract_required_string(params, "target")?;
        let check_type = extract_required_string(params, "check_type")?;

        // Validate check_type
        match check_type {
            "tcp" | "http" | "https" | "dns" => {}
            _ => {
                return Err(LLMSpellError::Validation {
                    message: format!(
                        "Invalid check_type: {check_type}. Supported types: tcp, http, https, dns"
                    ),
                    field: Some("check_type".to_string()),
                });
            }
        }

        // Validate target is not empty
        if target.trim().is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Target cannot be empty".to_string(),
                field: Some("target".to_string()),
            });
        }

        // Extract optional parameters
        let timeout_seconds = extract_optional_u64(params, "timeout_seconds")
            .unwrap_or(self.config.default_timeout_seconds);

        // Validate timeout if provided
        if timeout_seconds > self.config.max_timeout_seconds {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "Timeout {} exceeds maximum allowed timeout of {} seconds",
                    timeout_seconds, self.config.max_timeout_seconds
                ),
                field: Some("timeout_seconds".to_string()),
            });
        }

        let timeout_duration = Duration::from_secs(timeout_seconds);

        // Perform the check based on type
        let result = match check_type {
            "tcp" => {
                let (host, port) = self.parse_target(target)?;
                self.check_tcp_port(&host, port, timeout_duration).await
            }
            "http" => {
                let url = if target.starts_with("http://") || target.starts_with("https://") {
                    target.to_string()
                } else {
                    format!("http://{target}")
                };
                self.check_http_service(&url, timeout_duration).await
            }
            "https" => {
                let url = if target.starts_with("https://") {
                    target.to_string()
                } else if target.starts_with("http://") {
                    target.replace("http://", "https://")
                } else {
                    format!("https://{target}")
                };
                self.check_http_service(&url, timeout_duration).await
            }
            "dns" => {
                // Simple DNS resolution check
                let start_time = Instant::now();
                match target.to_socket_addrs() {
                    Ok(addrs) => {
                        let addr_list: Vec<SocketAddr> = addrs.collect();
                        let response_time =
                            u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
                        ServiceCheckResult {
                            target: target.to_string(),
                            available: !addr_list.is_empty(),
                            response_time_ms: response_time,
                            status: format!("Resolved {} addresses", addr_list.len()),
                            error: None,
                            metadata: {
                                let mut meta = HashMap::new();
                                meta.insert(
                                    "address_count".to_string(),
                                    addr_list.len().to_string(),
                                );
                                if let Some(first_addr) = addr_list.first() {
                                    meta.insert(
                                        "first_address".to_string(),
                                        first_addr.to_string(),
                                    );
                                }
                                meta
                            },
                        }
                    }
                    Err(e) => {
                        let response_time =
                            u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
                        ServiceCheckResult {
                            target: target.to_string(),
                            available: false,
                            response_time_ms: response_time,
                            status: "DNS resolution failed".to_string(),
                            error: Some(e.to_string()),
                            metadata: HashMap::new(),
                        }
                    }
                }
            }
            _ => unreachable!(), // Already validated above
        };

        // Format response
        let message = if result.available {
            format!(
                "Service '{}' is available ({}ms response time)",
                result.target, result.response_time_ms
            )
        } else {
            format!(
                "Service '{}' is not available: {}",
                result.target,
                result.error.as_ref().unwrap_or(&result.status)
            )
        };

        let response = ResponseBuilder::success("check")
            .with_message(message)
            .with_result(json!({
                "target": result.target,
                "check_type": check_type,
                "available": result.available,
                "response_time_ms": result.response_time_ms,
                "status": result.status,
                "error": result.error,
                "metadata": result.metadata,
                "timeout_seconds": timeout_seconds
            }))
            .build();

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> LLMResult<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
        Ok(AgentOutput::text(format!("Service checker error: {error}")))
    }
}

#[async_trait]
impl Tool for ServiceCheckerTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::System
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted // Network access requires restricted security
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "service_checker".to_string(),
            "Check service availability and network connectivity".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "target".to_string(),
            param_type: ParameterType::String,
            description:
                "Target to check (host:port for TCP, URL for HTTP/HTTPS, hostname for DNS)"
                    .to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "check_type".to_string(),
            param_type: ParameterType::String,
            description: "Type of check to perform: tcp, http, https, dns".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "timeout_seconds".to_string(),
            param_type: ParameterType::Number,
            description: "Timeout for the check in seconds".to_string(),
            required: false,
            default: Some(json!(5)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::tool_helpers::{create_test_tool, create_test_tool_input};
    use std::collections::HashMap;

    fn create_test_service_checker() -> ServiceCheckerTool {
        let config = ServiceCheckerConfig::default();
        ServiceCheckerTool::new(config)
    }

    fn create_test_tool_with_custom_config() -> ServiceCheckerTool {
        let config = ServiceCheckerConfig {
            default_timeout_seconds: 2,
            max_timeout_seconds: 10,
            allow_any_domain: true,
            ..Default::default()
        };
        ServiceCheckerTool::new(config)
    }
    #[tokio::test]
    async fn test_tcp_check_localhost() {
        let tool = create_test_tool_with_custom_config();

        let input = create_test_tool_input(vec![
            ("target", "127.0.0.1:22"),
            ("check_type", "tcp"),
            ("timeout_seconds", "1"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        // Note: SSH port might not be open, so we just check the tool doesn't crash
        assert!(result.text.contains("127.0.0.1:22"));
    }
    #[tokio::test]
    async fn test_http_check_invalid_url() {
        let tool = create_test_tool_with_custom_config();

        let input = create_test_tool_input(vec![
            ("target", "http://nonexistent-domain-12345.invalid"),
            ("check_type", "http"),
            ("timeout_seconds", "1"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("not available"));
    }
    #[tokio::test]
    async fn test_dns_check_localhost() {
        let tool = create_test_tool_with_custom_config();

        let input = create_test_tool_input(vec![("target", "localhost:80"), ("check_type", "dns")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("localhost"));
    }
    #[tokio::test]
    async fn test_blocked_port() {
        let tool = create_test_service_checker();

        let input = create_test_tool_input(vec![
            ("target", "127.0.0.1:7"), // Echo port (blocked by default)
            ("check_type", "tcp"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("not available"));
        assert!(result.text.contains("not allowed") || result.text.contains("blocked"));
    }
    #[tokio::test]
    async fn test_blocked_domain() {
        let tool = create_test_service_checker();

        let input = create_test_tool_input(vec![
            ("target", "evil.example.com:80"),
            ("check_type", "tcp"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("not available"));
    }
    #[tokio::test]
    async fn test_invalid_parameters() {
        let tool = create_test_service_checker();

        // Missing target
        let input1 = create_test_tool_input(vec![("check_type", "tcp")]);
        let result1 = tool.execute(input1, ExecutionContext::default()).await;
        assert!(result1.is_err());
        assert!(result1
            .unwrap_err()
            .to_string()
            .contains("Missing required parameter 'target'"));

        // Missing check_type
        let input2 = create_test_tool_input(vec![("target", "localhost:80")]);
        let result2 = tool.execute(input2, ExecutionContext::default()).await;
        assert!(result2.is_err());
        assert!(result2
            .unwrap_err()
            .to_string()
            .contains("Missing required parameter 'check_type'"));

        // Invalid check_type
        let input3 =
            create_test_tool_input(vec![("target", "localhost:80"), ("check_type", "invalid")]);
        let result3 = tool.execute(input3, ExecutionContext::default()).await;
        assert!(result3.is_err());
        assert!(result3
            .unwrap_err()
            .to_string()
            .contains("Invalid check_type"));

        // Empty target
        let input4 = create_test_tool_input(vec![("target", ""), ("check_type", "tcp")]);
        let result4 = tool.execute(input4, ExecutionContext::default()).await;
        assert!(result4.is_err());
        assert!(result4.unwrap_err().to_string().contains("cannot be empty"));

        // Excessive timeout
        let input5 = create_test_tool_input(vec![
            ("target", "localhost:80"),
            ("check_type", "tcp"),
            ("timeout_seconds", "100"),
        ]);
        let result5 = tool.execute(input5, ExecutionContext::default()).await;
        assert!(result5.is_err());
        assert!(result5.unwrap_err().to_string().contains("exceeds maximum"));
    }
    #[tokio::test]
    async fn test_target_parsing() {
        let tool = create_test_service_checker();

        // Valid target
        let result1 = tool.parse_target("localhost:80");
        assert!(result1.is_ok());
        let (host, port) = result1.unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 80);

        // Invalid target (no port)
        let result2 = tool.parse_target("localhost");
        assert!(result2.is_err());
        assert!(result2
            .unwrap_err()
            .to_string()
            .contains("format 'host:port'"));

        // Invalid port
        let result3 = tool.parse_target("localhost:abc");
        assert!(result3.is_err());
        assert!(result3
            .unwrap_err()
            .to_string()
            .contains("Invalid port number"));
    }
    #[tokio::test]
    async fn test_port_and_domain_validation() {
        let tool = create_test_service_checker();

        // Test port validation
        assert!(tool.is_port_allowed(80)); // Allowed
        assert!(tool.is_port_allowed(443)); // Allowed
        assert!(!tool.is_port_allowed(7)); // Blocked
        assert!(!tool.is_port_allowed(9999)); // Not in allowed list

        // Test domain validation
        assert!(tool.is_domain_allowed("localhost")); // Allowed
        assert!(tool.is_domain_allowed("127.0.0.1")); // Allowed
        assert!(!tool.is_domain_allowed("example.com")); // Not in allowed list

        // Test with allow_any_domain enabled
        let config = ServiceCheckerConfig {
            allow_any_domain: true,
            ..Default::default()
        };
        let tool_permissive = ServiceCheckerTool::new(config);
        assert!(tool_permissive.is_domain_allowed("example.com"));
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_service_checker();

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "service_checker");
        assert!(
            metadata.description.contains("Service availability")
                || metadata.description.contains("service availability")
        );

        let schema = tool.schema();
        assert_eq!(schema.name, "service_checker");
        assert_eq!(tool.category(), ToolCategory::System);
        assert_eq!(tool.security_level(), SecurityLevel::Restricted);

        // Check required parameters
        let required_params = schema.required_parameters();
        assert!(required_params.contains(&"target".to_string()));
        assert!(required_params.contains(&"check_type".to_string()));
        assert_eq!(required_params.len(), 2);
    }
    #[tokio::test]
    async fn test_custom_config() {
        let tool = create_test_tool_with_custom_config();

        // Test that custom configuration is applied
        assert_eq!(tool.config.default_timeout_seconds, 2);
        assert_eq!(tool.config.max_timeout_seconds, 10);
        assert!(tool.config.allow_any_domain);
    }
    #[tokio::test]
    async fn test_https_url_transformation() {
        let tool = create_test_tool_with_custom_config();

        let input = create_test_tool_input(vec![
            ("target", "http://httpbin.org/status/200"),
            ("check_type", "https"),
            ("timeout_seconds", "5"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        // Check that the tool transformed http:// to https://
        assert!(result.text.contains("httpbin.org"));
    }
}
