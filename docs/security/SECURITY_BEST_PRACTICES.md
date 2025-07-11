# ABOUTME: Security best practices guide for rs-llmspell development
# ABOUTME: Comprehensive security guidelines for tool development and deployment

# Security Best Practices for rs-llmspell

This guide provides comprehensive security best practices for developing, deploying, and using rs-llmspell tools.

## Table of Contents

1. [Tool Development Security](#tool-development-security)
2. [Input Validation](#input-validation)
3. [Resource Management](#resource-management)
4. [Sandbox Implementation](#sandbox-implementation)
5. [Error Handling](#error-handling)
6. [Deployment Security](#deployment-security)
7. [Monitoring and Logging](#monitoring-and-logging)
8. [Security Testing](#security-testing)

## Tool Development Security

### Secure Design Principles

#### 1. Principle of Least Privilege
```rust
// ✅ Good: Minimal required permissions
impl Tool for MyTool {
    fn security_requirements() -> SecurityRequirements {
        SecurityRequirements::default()
            .with_security_level(SecurityLevel::Restricted)
            .with_file_access("/tmp/tool-workspace") // Only specific directory
    }
}

// ❌ Bad: Excessive permissions
impl Tool for MyTool {
    fn security_requirements() -> SecurityRequirements {
        SecurityRequirements::default()
            .with_security_level(SecurityLevel::Privileged)
            .with_file_access("/") // Root access
            .with_network_access()
            .with_process_execution()
    }
}
```

#### 2. Defense in Depth
Implement multiple layers of security controls:
- Input validation at tool boundary
- Sandbox enforcement during execution
- Resource limits throughout operation
- Output sanitization before return

#### 3. Fail Secure
When security checks fail, default to denying access:
```rust
// ✅ Good: Secure failure
fn validate_path(&self, path: &Path) -> Result<PathBuf> {
    match self.sandbox.validate_path(path) {
        Ok(canonical_path) => {
            if self.is_path_allowed(&canonical_path) {
                Ok(canonical_path)
            } else {
                Err(LLMSpellError::SecurityViolation(
                    "Path access denied".to_string()
                ))
            }
        }
        Err(_) => Err(LLMSpellError::SecurityViolation(
            "Invalid path".to_string()
        ))
    }
}
```

### Security Requirements Declaration

Every tool MUST declare its security requirements:
```rust
impl Tool for YourTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "your-tool".to_string(),
            description: "Tool description".to_string(),
            // ... other fields
        }
    }
    
    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::default()
            .with_security_level(SecurityLevel::Restricted)
            .with_resource_limits(ResourceLimits {
                max_memory_bytes: 100 * 1024 * 1024, // 100MB
                max_cpu_time_ms: 5000, // 5 seconds
                max_network_bps: 1024 * 1024, // 1MB/s
                max_file_ops_per_sec: 100,
                custom_limits: HashMap::new(),
            })
    }
}
```

## Input Validation

### 1. Validate All Inputs
```rust
fn validate_parameters(&self, params: &Value) -> Result<ValidatedParams> {
    // Type validation
    let operation = params.get("operation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LLMSpellError::InvalidParameter(
            "operation must be a string".to_string()
        ))?;
    
    // Enum validation
    let operation = match operation {
        "read" | "write" | "delete" => operation,
        _ => return Err(LLMSpellError::InvalidParameter(
            "invalid operation".to_string()
        ))
    };
    
    // Size limits
    if let Some(content) = params.get("content").and_then(|v| v.as_str()) {
        if content.len() > MAX_CONTENT_SIZE {
            return Err(LLMSpellError::InvalidParameter(
                "content too large".to_string()
            ));
        }
    }
    
    Ok(ValidatedParams { operation, /* ... */ })
}
```

### 2. Path Validation
```rust
fn validate_file_path(&self, path: &str) -> Result<PathBuf> {
    // Reject dangerous patterns
    if path.contains("..") || path.contains("~") {
        return Err(LLMSpellError::SecurityViolation(
            "Path traversal attempted".to_string()
        ));
    }
    
    // Canonicalize and validate
    let canonical = std::fs::canonicalize(path)
        .map_err(|_| LLMSpellError::InvalidParameter(
            "Invalid file path".to_string()
        ))?;
    
    // Check against sandbox
    self.sandbox.validate_path(&canonical)
}
```

### 3. Regex Validation
```rust
fn validate_regex_pattern(&self, pattern: &str) -> Result<Regex> {
    // Check pattern complexity
    if pattern.len() > MAX_PATTERN_LENGTH {
        return Err(LLMSpellError::InvalidParameter(
            "Pattern too complex".to_string()
        ));
    }
    
    // Reject dangerous patterns
    let dangerous_patterns = [
        r"\([^)]*\)\*",  // Nested repetition
        r"\([^)]*\)\+",  // Nested repetition
        r"(.+)+",        // Exponential backtracking
    ];
    
    for dangerous in &dangerous_patterns {
        if Regex::new(dangerous).unwrap().is_match(pattern) {
            return Err(LLMSpellError::SecurityViolation(
                "Potentially dangerous regex pattern".to_string()
            ));
        }
    }
    
    // Compile with timeout
    Regex::new(pattern)
        .map_err(|_| LLMSpellError::InvalidParameter(
            "Invalid regex pattern".to_string()
        ))
}
```

## Resource Management

### 1. Memory Limits
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

struct ResourceTracker {
    memory_used: AtomicUsize,
    max_memory: usize,
}

impl ResourceTracker {
    fn allocate(&self, size: usize) -> Result<()> {
        let current = self.memory_used.load(Ordering::Relaxed);
        if current + size > self.max_memory {
            return Err(LLMSpellError::ResourceExhausted(
                "Memory limit exceeded".to_string()
            ));
        }
        self.memory_used.store(current + size, Ordering::Relaxed);
        Ok(())
    }
    
    fn deallocate(&self, size: usize) {
        let current = self.memory_used.load(Ordering::Relaxed);
        self.memory_used.store(current.saturating_sub(size), Ordering::Relaxed);
    }
}
```

### 2. Execution Time Limits
```rust
use tokio::time::{timeout, Duration};

async fn execute_with_timeout<F, T>(&self, operation: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let max_duration = Duration::from_millis(
        self.security_requirements().resource_limits.max_cpu_time_ms
    );
    
    timeout(max_duration, operation)
        .await
        .map_err(|_| LLMSpellError::ResourceExhausted(
            "Operation timed out".to_string()
        ))?
}
```

### 3. File Size Limits
```rust
fn write_file_with_limits(&self, path: &Path, content: &str) -> Result<()> {
    // Check content size
    if content.len() > self.config.max_file_size {
        return Err(LLMSpellError::InvalidParameter(
            "File content too large".to_string()
        ));
    }
    
    // Check available disk space
    if let Ok(metadata) = std::fs::metadata(path.parent().unwrap()) {
        // Implementation depends on platform
        // Consider using a crate like `fs2` for cross-platform disk usage
    }
    
    std::fs::write(path, content)
        .map_err(|e| LLMSpellError::IoError(e.to_string()))
}
```

## Sandbox Implementation

### 1. File System Sandbox
```rust
use std::path::{Path, PathBuf};

pub struct FileSandbox {
    allowed_paths: Vec<PathBuf>,
    denied_patterns: Vec<String>,
}

impl FileSandbox {
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // Resolve to canonical path
        let canonical = path.canonicalize()
            .map_err(|_| LLMSpellError::SecurityViolation(
                "Cannot resolve path".to_string()
            ))?;
        
        // Check if path is within allowed directories
        let is_allowed = self.allowed_paths.iter().any(|allowed| {
            canonical.starts_with(allowed)
        });
        
        if !is_allowed {
            return Err(LLMSpellError::SecurityViolation(
                "Path outside sandbox".to_string()
            ));
        }
        
        // Check against denied patterns
        let path_str = canonical.to_string_lossy();
        for pattern in &self.denied_patterns {
            if path_str.contains(pattern) {
                return Err(LLMSpellError::SecurityViolation(
                    "Path matches denied pattern".to_string()
                ));
            }
        }
        
        Ok(canonical)
    }
}
```

### 2. Network Sandbox
```rust
pub struct NetworkSandbox {
    allowed_hosts: Vec<String>,
    allowed_ports: Vec<u16>,
    max_request_size: usize,
}

impl NetworkSandbox {
    pub fn validate_url(&self, url: &str) -> Result<url::Url> {
        let parsed = url::Url::parse(url)
            .map_err(|_| LLMSpellError::InvalidParameter(
                "Invalid URL".to_string()
            ))?;
        
        // Check scheme
        match parsed.scheme() {
            "http" | "https" => {},
            _ => return Err(LLMSpellError::SecurityViolation(
                "Unsupported URL scheme".to_string()
            ))
        }
        
        // Check host
        if let Some(host) = parsed.host_str() {
            if !self.is_host_allowed(host) {
                return Err(LLMSpellError::SecurityViolation(
                    "Host not allowed".to_string()
                ));
            }
        }
        
        // Check port
        if let Some(port) = parsed.port() {
            if !self.allowed_ports.contains(&port) {
                return Err(LLMSpellError::SecurityViolation(
                    "Port not allowed".to_string()
                ));
            }
        }
        
        Ok(parsed)
    }
}
```

## Error Handling

### 1. Secure Error Messages
```rust
// ✅ Good: Generic error message
fn handle_file_error(&self, error: std::io::Error) -> LLMSpellError {
    tracing::error!("File operation failed: {}", error); // Log details
    LLMSpellError::IoError("File operation failed".to_string()) // Generic message
}

// ❌ Bad: Exposes internal details
fn handle_file_error(&self, error: std::io::Error) -> LLMSpellError {
    LLMSpellError::IoError(format!("Failed to read /etc/secret/config: {}", error))
}
```

### 2. Error Classification
```rust
#[derive(Debug, Clone)]
pub enum SecurityErrorType {
    InputValidation,
    AccessControl,
    ResourceExhaustion,
    SandboxViolation,
    ConfigurationError,
}

impl LLMSpellError {
    pub fn security_violation(
        error_type: SecurityErrorType,
        message: String
    ) -> Self {
        // Log security events for monitoring
        tracing::warn!(
            security_event = true,
            error_type = ?error_type,
            "Security violation: {}", message
        );
        
        LLMSpellError::SecurityViolation(message)
    }
}
```

## Deployment Security

### 1. Configuration Security
```toml
# config/security.toml
[security]
default_security_level = "Restricted"
enable_privileged_tools = false
max_concurrent_operations = 100

[sandboxes.file_system]
allowed_paths = ["/tmp/llmspell", "/workspace"]
denied_patterns = ["secret", "private", ".ssh"]
max_file_size = "10MB"

[sandboxes.network]
allowed_hosts = ["api.example.com", "*.trusted-domain.com"]
allowed_ports = [80, 443]
max_request_size = "1MB"
```

### 2. Environment Hardening
```bash
# Container security
docker run --security-opt=no-new-privileges \
           --cap-drop=ALL \
           --read-only \
           --tmpfs /tmp \
           --user 1000:1000 \
           rs-llmspell

# Process isolation
systemd-run --uid=llmspell \
            --gid=llmspell \
            --no-new-privileges \
            --private-tmp \
            rs-llmspell
```

## Monitoring and Logging

### 1. Security Event Logging
```rust
use tracing::{event, Level};

fn log_security_event(&self, event_type: &str, details: &str) {
    event!(
        Level::WARN,
        security_event = true,
        event_type = event_type,
        tool_name = self.metadata.name,
        details = details,
        "Security event logged"
    );
}

// Usage
self.log_security_event("path_traversal_attempt", &format!("Path: {}", path));
```

### 2. Metrics Collection
```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref SECURITY_VIOLATIONS: Counter = Counter::new(
        "security_violations_total",
        "Total number of security violations"
    ).unwrap();
    
    static ref OPERATION_DURATION: Histogram = Histogram::new(
        "tool_operation_duration_seconds",
        "Time spent executing tool operations"
    ).unwrap();
}

fn record_security_violation(&self, violation_type: &str) {
    SECURITY_VIOLATIONS.inc();
    tracing::warn!(
        violation_type = violation_type,
        "Security violation detected"
    );
}
```

## Security Testing

### 1. Unit Test Security Patterns
```rust
#[tokio::test]
async fn test_path_traversal_prevention() {
    let tool = YourTool::new(Config::default());
    
    let malicious_paths = vec![
        "../../../etc/passwd",
        "/etc/shadow",
        "~/.ssh/id_rsa",
        "/proc/self/environ",
    ];
    
    for path in malicious_paths {
        let input = AgentInput::text("read").with_parameter(
            "parameters",
            json!({ "path": path })
        );
        
        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err(), "Path traversal should be blocked: {}", path);
    }
}
```

### 2. Fuzzing Integration
```rust
#[cfg(test)]
mod fuzz_tests {
    use quickcheck::{quickcheck, TestResult};
    
    #[quickcheck]
    fn fuzz_path_validation(path: String) -> TestResult {
        let tool = YourTool::new(Config::default());
        
        // Skip empty paths
        if path.is_empty() {
            return TestResult::discard();
        }
        
        let result = tool.validate_path(&path);
        
        // Should either succeed or fail gracefully
        match result {
            Ok(_) => TestResult::passed(),
            Err(LLMSpellError::SecurityViolation(_)) => TestResult::passed(),
            Err(LLMSpellError::InvalidParameter(_)) => TestResult::passed(),
            Err(_) => TestResult::failed(), // Unexpected error type
        }
    }
}
```

### 3. Integration Security Tests
```rust
#[tokio::test]
async fn test_resource_exhaustion_protection() {
    let tool = YourTool::new(Config::default());
    
    // Test large input
    let large_input = "A".repeat(100_000_000); // 100MB
    let input = AgentInput::text("process").with_parameter(
        "parameters",
        json!({ "data": large_input })
    );
    
    let start = Instant::now();
    let result = tool.execute(input, ExecutionContext::default()).await;
    let elapsed = start.elapsed();
    
    // Should either reject large input or complete quickly
    assert!(
        result.is_err() || elapsed < Duration::from_secs(5),
        "Tool should protect against resource exhaustion"
    );
}
```

## Conclusion

Following these security best practices ensures that rs-llmspell tools are resilient against common attack vectors and operate within secure boundaries. Regular security testing and monitoring are essential for maintaining a strong security posture.

### Security Checklist for New Tools

- [ ] Security requirements properly declared
- [ ] Input validation implemented
- [ ] Resource limits enforced
- [ ] Sandbox integration completed
- [ ] Error handling secures sensitive information
- [ ] Security tests written and passing
- [ ] Documentation includes security considerations
- [ ] Code review includes security evaluation

### Resources

- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)
- [Rust Security Guidelines](https://doc.rust-lang.org/reference/unsafety.html)
- [Container Security Best Practices](https://kubernetes.io/docs/concepts/security/)

---
*Generated as part of Task 2.10.2 - Security Validation*  
*🤖 Generated with [Claude Code](https://claude.ai/code)*