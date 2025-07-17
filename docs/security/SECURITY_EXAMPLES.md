# Security Implementation Examples

## Overview

This document provides practical code examples demonstrating how to implement security features in LLMSpell tools. Each example includes both the vulnerable and secure versions to illustrate best practices.

## Input Validation Examples

### Example 1: Path Traversal Prevention

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - Vulnerable to path traversal
use std::fs;

pub async fn read_file(path: &str) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}
```

#### ✅ Secure Code
```rust
// DO THIS - Protected against path traversal
use std::fs;
use std::path::{Path, PathBuf};
use llmspell_utils::security::validation::PathValidator;

pub async fn read_file(path: &str) -> Result<String> {
    // Initialize validator with allowed directories
    let validator = PathValidator::new(vec![
        PathBuf::from("/var/llmspell/data"),
        PathBuf::from("/tmp/llmspell"),
    ]);
    
    // Validate and canonicalize the path
    let safe_path = validator.validate_path(path)?;
    
    // Additional security checks
    if safe_path.is_symlink() {
        return Err("Symlinks are not allowed".into());
    }
    
    // Check file size before reading
    let metadata = fs::metadata(&safe_path)?;
    if metadata.len() > 10_000_000 { // 10MB limit
        return Err("File too large".into());
    }
    
    let content = fs::read_to_string(safe_path)?;
    Ok(content)
}
```

### Example 2: Command Injection Prevention

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - Vulnerable to command injection
use std::process::Command;

pub async fn run_system_command(user_input: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("echo {}", user_input))
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

#### ✅ Secure Code
```rust
// DO THIS - Protected against command injection
use std::process::Command;
use llmspell_utils::security::validation::CommandValidator;

pub async fn run_system_command(user_input: &str) -> Result<String> {
    // Validate input
    let validator = CommandValidator::new();
    let safe_input = validator.validate_and_escape(user_input)?;
    
    // Use direct command execution, not shell
    let output = Command::new("echo")
        .arg(safe_input) // Passed as argument, not interpreted by shell
        .output()?;
    
    // Sanitize output
    let output_text = String::from_utf8_lossy(&output.stdout);
    let sanitized = validator.sanitize_output(output_text.to_string());
    
    Ok(sanitized)
}
```

### Example 3: SQL Injection Prevention

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - Vulnerable to SQL injection
pub async fn query_database(user_id: &str) -> Result<Vec<User>> {
    let query = format!("SELECT * FROM users WHERE id = '{}'", user_id);
    let results = db.execute(&query).await?;
    Ok(results)
}
```

#### ✅ Secure Code
```rust
// DO THIS - Protected against SQL injection
use sqlx::query_as;

pub async fn query_database(user_id: &str) -> Result<Vec<User>> {
    // Validate input format
    if !user_id.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err("Invalid user ID format".into());
    }
    
    // Use parameterized queries
    let results = query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        user_id
    )
    .fetch_all(&db_pool)
    .await?;
    
    Ok(results)
}
```

## Authentication Examples

### Example 4: API Key Validation

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - Weak API key validation
pub fn validate_api_key(key: &str) -> bool {
    key == "secret123" // Hardcoded key!
}
```

#### ✅ Secure Code
```rust
// DO THIS - Secure API key validation
use llmspell_utils::security::auth::{ApiKeyValidator, HashVerifier};
use std::time::{Duration, SystemTime};

pub struct SecureApiKeyValidator {
    validator: ApiKeyValidator,
    rate_limiter: RateLimiter,
}

impl SecureApiKeyValidator {
    pub async fn validate_api_key(&self, key: &str) -> Result<bool> {
        // Rate limit key validation attempts
        if !self.rate_limiter.check_rate("api_key_validation").await? {
            return Err("Rate limit exceeded".into());
        }
        
        // Validate key format
        if !self.validator.is_valid_format(key) {
            return Ok(false);
        }
        
        // Look up key hash in database
        let stored_hash = self.get_key_hash_from_db(key).await?;
        
        // Constant-time comparison
        let is_valid = HashVerifier::verify_constant_time(key, &stored_hash)?;
        
        // Check expiration
        if is_valid {
            let expiry = self.get_key_expiry(key).await?;
            if expiry < SystemTime::now() {
                return Ok(false);
            }
        }
        
        Ok(is_valid)
    }
}
```

### Example 5: Session Management

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - Insecure session management
pub fn create_session(user_id: &str) -> String {
    format!("session_{}", user_id) // Predictable session ID!
}
```

#### ✅ Secure Code
```rust
// DO THIS - Secure session management
use llmspell_utils::security::crypto::{generate_secure_token, hash_token};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct SecureSessionManager {
    sessions: RwLock<HashMap<String, SessionData>>,
    config: SessionConfig,
}

impl SecureSessionManager {
    pub async fn create_session(&self, user_id: &str) -> Result<String> {
        // Generate cryptographically secure token
        let token = generate_secure_token(32)?;
        
        // Hash token for storage
        let token_hash = hash_token(&token)?;
        
        // Create session data
        let session = SessionData {
            user_id: user_id.to_string(),
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            ip_address: self.get_client_ip()?,
        };
        
        // Store with expiration
        let mut sessions = self.sessions.write().await;
        sessions.insert(token_hash, session);
        
        // Schedule cleanup
        self.schedule_cleanup(token_hash.clone());
        
        Ok(token)
    }
    
    pub async fn validate_session(&self, token: &str) -> Result<Option<String>> {
        let token_hash = hash_token(token)?;
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(&token_hash) {
            // Check expiration
            if session.is_expired(&self.config) {
                return Ok(None);
            }
            
            // Verify IP hasn't changed (optional)
            if self.config.bind_to_ip && session.ip_address != self.get_client_ip()? {
                return Ok(None);
            }
            
            Ok(Some(session.user_id.clone()))
        } else {
            Ok(None)
        }
    }
}
```

## Output Sanitization Examples

### Example 6: Error Message Sanitization

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - Exposes internal details
pub async fn process_file(path: &str) -> Result<String> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read {}: {}", path, e).into()), // Leaks path!
    }
}
```

#### ✅ Secure Code
```rust
// DO THIS - Generic error messages
use llmspell_utils::security::output::ErrorSanitizer;
use tracing::error;

pub async fn process_file(path: &str) -> Result<String> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) => {
            // Log detailed error internally
            error!("File read failed - Path: {}, Error: {:?}", path, e);
            
            // Return generic error to user
            let sanitizer = ErrorSanitizer::new();
            Err(sanitizer.sanitize_file_error(e).into())
        }
    }
}
```

### Example 7: Data Redaction

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - Returns sensitive data
pub async fn get_system_info() -> Result<SystemInfo> {
    Ok(SystemInfo {
        hostname: hostname::get()?.to_string_lossy().to_string(),
        username: whoami::username(),
        home_dir: dirs::home_dir().unwrap().to_string_lossy().to_string(),
        env_vars: std::env::vars().collect(), // Exposes all env vars!
    })
}
```

#### ✅ Secure Code
```rust
// DO THIS - Redacts sensitive information
use llmspell_utils::security::output::DataRedactor;

pub async fn get_system_info() -> Result<SystemInfo> {
    let redactor = DataRedactor::new();
    
    Ok(SystemInfo {
        hostname: redactor.redact_hostname(hostname::get()?.to_string_lossy()),
        username: redactor.redact_username(whoami::username()),
        home_dir: "[REDACTED]".to_string(),
        env_vars: redactor.filter_env_vars(std::env::vars()),
    })
}

impl DataRedactor {
    fn filter_env_vars(&self, vars: impl Iterator<Item = (String, String)>) -> HashMap<String, String> {
        vars.filter(|(key, _)| {
            // Only include safe environment variables
            !key.contains("KEY") && 
            !key.contains("SECRET") && 
            !key.contains("PASSWORD") &&
            !key.contains("TOKEN")
        })
        .map(|(k, v)| (k, self.redact_if_sensitive(v)))
        .collect()
    }
}
```

## Resource Management Examples

### Example 8: Memory Protection

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - No memory limits
pub async fn process_large_data(size: usize) -> Result<Vec<u8>> {
    let mut data = Vec::with_capacity(size); // Could allocate unlimited memory!
    // ... process data ...
    Ok(data)
}
```

#### ✅ Secure Code
```rust
// DO THIS - Memory limits enforced
use llmspell_utils::security::limits::{MemoryGuard, ResourceLimits};

pub async fn process_large_data(size: usize) -> Result<Vec<u8>> {
    // Check size limit first
    const MAX_SIZE: usize = 100 * 1024 * 1024; // 100MB
    if size > MAX_SIZE {
        return Err("Requested size exceeds limit".into());
    }
    
    // Use memory guard
    let _guard = MemoryGuard::new(size)?;
    
    // Allocate with fallible allocation
    let mut data = Vec::new();
    data.try_reserve(size)
        .map_err(|_| "Failed to allocate memory")?;
    
    // Process with periodic memory checks
    for chunk in 0..size/1024 {
        if MemoryGuard::check_system_memory()? < 0.1 { // Less than 10% free
            return Err("System memory low".into());
        }
        // ... process chunk ...
    }
    
    Ok(data)
}
```

### Example 9: Rate Limiting

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - No rate limiting
pub async fn handle_request(user_id: &str) -> Result<Response> {
    // Process immediately
    process_expensive_operation().await
}
```

#### ✅ Secure Code
```rust
// DO THIS - Rate limiting implemented
use llmspell_utils::security::ratelimit::{RateLimiter, RateLimitConfig};
use std::time::Duration;

pub struct SecureRequestHandler {
    rate_limiter: RateLimiter,
}

impl SecureRequestHandler {
    pub async fn handle_request(&self, user_id: &str) -> Result<Response> {
        // Check rate limit
        let key = format!("user:{}", user_id);
        if !self.rate_limiter.check_and_update(&key).await? {
            return Err(LLMSpellError::RateLimitExceeded {
                retry_after: Duration::from_secs(60),
            });
        }
        
        // Apply backpressure if system is under load
        if self.system_load_high().await? {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Process with timeout
        tokio::time::timeout(
            Duration::from_secs(30),
            process_expensive_operation()
        )
        .await
        .map_err(|_| "Operation timed out".into())?
    }
}
```

## Network Security Examples

### Example 10: SSRF Prevention

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - Vulnerable to SSRF
pub async fn fetch_url(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    Ok(response.text().await?)
}
```

#### ✅ Secure Code
```rust
// DO THIS - Protected against SSRF
use llmspell_utils::security::network::{UrlValidator, SafeHttpClient};
use std::net::IpAddr;

pub async fn fetch_url(url: &str) -> Result<String> {
    // Validate URL
    let validator = UrlValidator::new()
        .allowed_schemes(vec!["https"])
        .allowed_ports(vec![443])
        .deny_private_ips()
        .deny_local_addresses();
    
    let safe_url = validator.validate(url)?;
    
    // Create client with security settings
    let client = SafeHttpClient::builder()
        .timeout(Duration::from_secs(30))
        .max_redirects(5)
        .user_agent("LLMSpell/1.0")
        .dns_resolver(|host| {
            // Custom DNS resolution to prevent DNS rebinding
            resolve_with_validation(host)
        })
        .build()?;
    
    // Make request with additional checks
    let response = client
        .get(&safe_url)
        .send()
        .await?;
    
    // Validate response
    if response.content_length().unwrap_or(0) > 10_000_000 {
        return Err("Response too large".into());
    }
    
    Ok(response.text().await?)
}
```

## Complete Tool Example

### Example 11: Secure File Reader Tool

```rust
use llmspell_core::{BaseAgent, Tool, Result, LLMSpellError};
use llmspell_core::types::{AgentInput, AgentOutput, ExecutionContext};
use llmspell_utils::security::{
    validation::{InputValidator, PathValidator},
    output::OutputSanitizer,
    limits::ResourceGuard,
    auth::PermissionChecker,
};
use std::path::PathBuf;
use serde_json::json;

pub struct SecureFileReaderTool {
    path_validator: PathValidator,
    input_validator: InputValidator,
    output_sanitizer: OutputSanitizer,
    allowed_paths: Vec<PathBuf>,
}

impl SecureFileReaderTool {
    pub fn new() -> Self {
        Self {
            path_validator: PathValidator::new(vec![
                PathBuf::from("/var/llmspell/data"),
                PathBuf::from("/tmp/llmspell"),
            ]),
            input_validator: InputValidator::default(),
            output_sanitizer: OutputSanitizer::default(),
            allowed_paths: vec![
                PathBuf::from("/var/llmspell/data"),
                PathBuf::from("/tmp/llmspell"),
            ],
        }
    }
}

#[async_trait::async_trait]
impl Tool for SecureFileReaderTool {
    fn metadata(&self) -> llmspell_core::ToolMetadata {
        llmspell_core::ToolMetadata {
            name: "SecureFileReader".to_string(),
            description: "Securely reads files with comprehensive protection".to_string(),
            version: "1.0.0".to_string(),
            author: "Security Team".to_string(),
            tags: vec!["file", "read", "secure"],
            parameters_schema: json!({
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to file to read"
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["utf-8", "ascii"],
                        "default": "utf-8"
                    }
                }
            }),
        }
    }
    
    async fn execute(&self, input: AgentInput, ctx: ExecutionContext) -> Result<AgentOutput> {
        // 1. Check permissions
        let checker = PermissionChecker::from_context(&ctx);
        checker.require_permission("file:read")?;
        
        // 2. Validate input parameters
        let path = input.parameters
            .get("parameters")
            .and_then(|p| p.get("path"))
            .and_then(|p| p.as_str())
            .ok_or_else(|| LLMSpellError::validation("Missing path parameter"))?;
        
        // 3. Resource limits
        let _guard = ResourceGuard::new()
            .memory_limit(50 * 1024 * 1024) // 50MB
            .time_limit(Duration::from_secs(10))
            .acquire()?;
        
        // 4. Validate path security
        let safe_path = self.path_validator.validate_path(path)?;
        
        // Additional security checks
        if safe_path.is_symlink() {
            return Err(LLMSpellError::validation("Symlinks not allowed"));
        }
        
        // 5. Check file size
        let metadata = tokio::fs::metadata(&safe_path).await
            .map_err(|e| {
                tracing::error!("Failed to read metadata for {:?}: {}", safe_path, e);
                LLMSpellError::execution("Failed to access file")
            })?;
        
        const MAX_SIZE: u64 = 10 * 1024 * 1024; // 10MB
        if metadata.len() > MAX_SIZE {
            return Err(LLMSpellError::validation("File too large"));
        }
        
        // 6. Read file with timeout
        let content = tokio::time::timeout(
            Duration::from_secs(5),
            tokio::fs::read_to_string(&safe_path)
        )
        .await
        .map_err(|_| LLMSpellError::timeout("File read timeout"))?
        .map_err(|e| {
            tracing::error!("Failed to read file {:?}: {}", safe_path, e);
            LLMSpellError::execution("Failed to read file")
        })?;
        
        // 7. Sanitize output
        let sanitized_content = self.output_sanitizer.sanitize(&content);
        
        // 8. Build response
        let response = json!({
            "success": true,
            "content": sanitized_content,
            "metadata": {
                "size": metadata.len(),
                "readable": true,
                "encoding": "utf-8"
            }
        });
        
        Ok(AgentOutput::success(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_path_traversal_blocked() {
        let tool = SecureFileReaderTool::new();
        let malicious_paths = vec![
            "../../../etc/passwd",
            "/etc/passwd",
            "../../sensitive.txt",
            "/var/llmspell/data/../../../etc/passwd",
        ];
        
        for path in malicious_paths {
            let input = AgentInput::from_params(json!({
                "parameters": { "path": path }
            }));
            
            let result = tool.execute(input, ExecutionContext::new()).await;
            assert!(result.is_err(), "Should block path: {}", path);
        }
    }
    
    #[tokio::test]
    async fn test_valid_file_read() {
        // Create test file
        let test_path = "/tmp/llmspell/test.txt";
        tokio::fs::create_dir_all("/tmp/llmspell").await.unwrap();
        tokio::fs::write(test_path, "Hello, World!").await.unwrap();
        
        let tool = SecureFileReaderTool::new();
        let input = AgentInput::from_params(json!({
            "parameters": { "path": test_path }
        }));
        
        let result = tool.execute(input, ExecutionContext::new()).await;
        assert!(result.is_ok());
        
        // Cleanup
        tokio::fs::remove_file(test_path).await.unwrap();
    }
}
```

## Security Testing Examples

### Example 12: Security Test Suite

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    use llmspell_tools::tests::security::*;
    
    #[tokio::test]
    async fn test_injection_protection() {
        let tool = YourTool::new();
        let test_cases = all_injection_tests();
        
        for test_case in test_cases {
            let input = create_test_input(test_case.payload);
            let result = tool.execute(input, ExecutionContext::new()).await;
            
            match test_case.expected_behavior {
                ExpectedBehavior::Reject => {
                    assert!(result.is_err(), 
                        "Failed to reject: {}", test_case.name);
                }
                ExpectedBehavior::Sanitize => {
                    assert!(result.is_ok());
                    let output = result.unwrap();
                    assert!(!contains_malicious_pattern(&output.text),
                        "Failed to sanitize: {}", test_case.name);
                }
                _ => {}
            }
        }
    }
    
    #[tokio::test]
    async fn test_resource_limits() {
        let tool = YourTool::new();
        
        // Test memory limit
        let large_input = create_test_input(json!({
            "data": "x".repeat(100_000_000) // 100MB
        }));
        
        let result = tool.execute(large_input, ExecutionContext::new()).await;
        assert!(result.is_err());
        
        // Test timeout
        let slow_input = create_test_input(json!({
            "operation": "sleep",
            "duration": 60
        }));
        
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            tool.execute(slow_input, ExecutionContext::new())
        ).await;
        
        assert!(result.is_err()); // Should timeout
    }
}
```

## Conclusion

These examples demonstrate practical security implementations for LLMSpell tools. Key takeaways:

1. **Always validate input** - Never trust user-provided data
2. **Use parameterized operations** - Avoid string concatenation for commands/queries
3. **Implement defense in depth** - Multiple security layers
4. **Fail securely** - Default to denying access
5. **Log security events** - But don't expose sensitive data
6. **Test security thoroughly** - Include security tests in your test suite

Remember: Security is not a feature, it's a requirement. Every tool should implement these patterns to maintain the security posture of the LLMSpell platform.