# Security Development Guide for rs-llmspell

✅ **CURRENT**: Updated for Phase 7 with correct APIs and implementation patterns.

This comprehensive guide provides security guidelines, best practices, and code examples for developers contributing to rs-llmspell. Following these guidelines ensures tools are secure, reliable, and resistant to common attack vectors.

## Implementation Status

### ✅ **Currently Implemented (Phase 3.3)**
- **Security Levels**: Safe, Restricted, Privileged classification  
- **Resource Limits**: Memory, CPU, file size, operation count limits
- **File Sandbox**: Path validation and access control
- **Basic Input Validation**: Path and basic sanitization
- **Tool Security Requirements**: Security level and requirements declaration

### 📋 **Planned Enhancements (Phase 4+)**
- **Advanced Input Validators**: URL, command, and content validation
- **Network Sandbox**: Domain-based network access control
- **Security Monitoring**: Real-time security violation tracking
- **Advanced Authentication**: Role-based access control
- **Audit Logging**: Comprehensive security event logging

## Core Security Principles

### 1. Security by Design
- Consider security from the initial design phase
- Threat model your tool before implementation
- Document security considerations in code comments

### 2. Least Privilege
- Request only the minimum permissions needed
- Limit file system access to specific directories
- Restrict network access to required domains

### 3. Defense in Depth
- Implement multiple layers of security
- Don't rely on a single security control
- Assume other defenses may fail

### 4. Fail Secure
- Default to denying access
- Handle errors gracefully without exposing internals
- Maintain security even in error conditions

## Tool Security Requirements

### Security Requirements Declaration

Every tool MUST declare its security requirements:

```rust
use llmspell_core::traits::tool::{Tool, SecurityRequirements, SecurityLevel, ResourceLimits};

impl Tool for YourTool {
    // ✅ CORRECT: Basic security level (most tools only need this)
    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted  // Safe, Restricted, or Privileged
    }

    // ✅ CORRECT: Optional - custom security requirements
    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::restricted()
            .with_file_access("/tmp/tool-workspace")
            .with_network_access("api.example.com")
            .with_env_access("HOME")
    }

    // ✅ CORRECT: Optional - custom resource limits  
    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(100 * 1024 * 1024) // 100MB
            .with_cpu_limit(5000) // 5 seconds
            .with_network_limit(1024 * 1024) // 1MB/s
    }
}
```

### Security Levels

1. **Safe**: No file/network access, memory-only operations
2. **Restricted** (Default): Limited file/network access, strict sandboxing  
3. **Privileged**: Extended access for system tools (requires review)

## Input Validation

### Required Validations

**CRITICAL SECURITY UPDATE**: All filesystem-accessing tools MUST use bridge-provided sandbox.

```rust
use llmspell_security::sandbox::FileSandbox;
use std::sync::Arc;

impl YourTool {
    // ✅ MANDATORY: Tools with file access must accept bridge-provided sandbox
    pub fn new(config: YourToolConfig, sandbox: Arc<FileSandbox>) -> Self {
        Self {
            config,
            sandbox, // Store bridge-provided sandbox
        }
    }

    fn validate_input(&self, input: &str) -> LLMResult<String> {
        // ✅ CORRECT: Use bridge-provided sandbox (NEVER create your own)
        let validated_path = self.sandbox.validate_path(Path::new(input))?;
        
        // ✅ CORRECT: Basic input sanitization
        let sanitized = input.trim();
        
        // Additional validation as needed
        if sanitized.is_empty() {
            return Err(LLMError::validation("Input cannot be empty"));
        }
        
        Ok(sanitized.to_string())
    }
}
```

### Path Security

```rust
use std::path::{Path, PathBuf};
use llmspell_utils::security::path::PathValidator;

fn safe_file_operation(user_path: &str) -> Result<PathBuf> {
    let validator = PathValidator::new(vec!["/allowed/path"]);
    
    // Canonicalize and validate
    let safe_path = validator.validate_path(user_path)?;
    
    // Additional checks
    if safe_path.is_symlink() {
        return Err("Symlinks not allowed".into());
    }
    
    Ok(safe_path)
}
```

**Path Security Rules**:
1. Always canonicalize paths
2. Check against allowed directories
3. Reject symlinks unless explicitly allowed
4. Prevent directory traversal
5. Validate file extensions
6. Check file size limits

### Regex Validation

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
    
    Regex::new(pattern)
        .map_err(|_| LLMSpellError::InvalidParameter(
            "Invalid regex pattern".to_string()
        ))
}
```

## Code Examples: Secure vs Insecure

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
    let validator = PathValidator::new(vec!["/workspace", "/tmp/llmspell"]);
    
    // Validate and canonicalize path
    let safe_path = validator.validate_path(path)?;
    
    // Additional safety checks
    if safe_path.extension().map_or(false, |ext| ext == "exe") {
        return Err(LLMSpellError::SecurityViolation(
            "Executable files not allowed".into()
        ));
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

pub async fn run_command(user_input: &str) -> Result<String> {
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

pub async fn run_command(user_input: &str) -> Result<String> {
    // Validate input
    let validator = CommandValidator::new();
    let safe_input = validator.sanitize_argument(user_input)?;
    
    // Use array form, never shell interpretation
    let output = Command::new("echo")
        .arg(safe_input)
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

### Example 3: SQL Injection Prevention

#### ❌ Vulnerable Code
```rust
// DON'T DO THIS - Vulnerable to SQL injection
pub async fn query_database(user_id: &str) -> Result<Vec<User>> {
    let query = format!("SELECT * FROM users WHERE id = '{}'", user_id);
    db.execute(&query).await
}
```

#### ✅ Secure Code
```rust
// DO THIS - Protected against SQL injection
pub async fn query_database(user_id: &str) -> Result<Vec<User>> {
    // Use parameterized queries
    let query = "SELECT * FROM users WHERE id = $1";
    db.query(query, &[&user_id]).await
}
```

## Output Sanitization

### Preventing Information Disclosure

```rust
use llmspell_utils::security::output::OutputSanitizer;

fn sanitize_output(output: String) -> String {
    let sanitizer = OutputSanitizer::new();
    
    // Remove sensitive patterns
    let output = sanitizer.remove_paths(output);
    let output = sanitizer.remove_secrets(output);
    let output = sanitizer.remove_internal_ips(output);
    
    // Generic error messages
    match result {
        Err(e) if e.is_internal() => "Operation failed".to_string(),
        Err(e) => sanitizer.sanitize_error(e),
        Ok(output) => sanitizer.sanitize_output(output),
    }
}
```

### Output Filtering Rules

1. Never expose full file paths
2. Redact API keys and secrets
3. Remove stack traces in production
4. Sanitize error messages
5. Filter internal IP addresses
6. Remove debug information

## Resource Management

### Preventing DoS Attacks

```rust
use std::time::Duration;
use llmspell_utils::security::limits::{ResourceLimits, ResourceMonitor};

impl YourTool {
    fn with_limits<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>
    {
        let limits = ResourceLimits {
            max_memory: 100 * 1024 * 1024, // 100MB
            max_cpu_time: Duration::from_secs(30),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_open_files: 10,
        };
        
        let monitor = ResourceMonitor::new(limits);
        monitor.execute(operation)
    }
}
```

### Execution Time Limits

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

## Network Security

### Safe HTTP Requests

```rust
use llmspell_utils::security::network::{HttpClient, NetworkPolicy};

async fn safe_http_request(url: &str) -> Result<String> {
    let policy = NetworkPolicy {
        allowed_domains: vec!["api.example.com"],
        allowed_schemes: vec!["https"],
        max_redirects: 5,
        timeout: Duration::from_secs(30),
    };
    
    let client = HttpClient::with_policy(policy);
    
    // Validate URL first
    client.validate_url(url)?;
    
    // Make request with built-in protections
    let response = client.get(url).await?;
    
    Ok(response.text().await?)
}
```

### Network Security Rules

1. Use HTTPS for external requests
2. Validate URLs before use
3. Implement domain whitelisting
4. Set request timeouts
5. Limit redirect following
6. Prevent SSRF attacks

## Error Handling

### Secure Error Handling Pattern

```rust
use llmspell_core::{Result, LLMSpellError};

impl YourTool {
    fn operation(&self) -> Result<String> {
        match self.internal_operation() {
            Ok(result) => Ok(result),
            Err(e) => {
                // Log detailed error internally
                tracing::error!("Operation failed: {:?}", e);
                
                // Return generic error to user
                Err(LLMSpellError::ToolExecutionError(
                    "Operation failed".into()
                ))
            }
        }
    }
}
```

### Error Handling Rules

1. Log detailed errors internally
2. Return generic errors to users
3. Never expose stack traces
4. Sanitize error messages
5. Handle all error cases
6. Fail to a secure state

## Threat Modeling for Tools

### STRIDE Methodology Application

For each tool, consider:

- **S**poofing: Can inputs impersonate other sources?
- **T**ampering: Can data be modified maliciously?
- **R**epudiation: Can actions be denied?
- **I**nformation Disclosure: Can sensitive data leak?
- **D**enial of Service: Can resources be exhausted?
- **E**levation of Privilege: Can permissions be escalated?

### Tool-Specific Threats

#### File System Tools
**Threats**: Path traversal, symlink attacks, file disclosure
**Mitigations**: 
- Canonical path resolution
- Symlink detection
- Sandbox restrictions
- Size limits

#### Network Tools
**Threats**: SSRF, data exfiltration, DNS rebinding
**Mitigations**:
- URL validation
- Domain whitelisting
- Request timeouts
- Response filtering

#### Code Execution Tools
**Threats**: Command injection, sandbox escape, resource exhaustion
**Mitigations**:
- Argument sanitization
- Process isolation
- Resource limits
- Execution timeouts

## Security Testing

### Required Security Tests

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_path_traversal() {
        let tool = YourTool::new();
        let attacks = vec![
            "../../../etc/passwd",
            "..\\..\\windows\\system32",
            "/etc/passwd",
            "C:\\Windows\\System32",
        ];
        
        for attack in attacks {
            assert!(tool.validate_input(attack).is_err());
        }
    }
    
    #[test]
    fn test_injection_attacks() {
        // Test SQL injection
        // Test command injection
        // Test script injection
    }
    
    #[test]
    fn test_dos_protection() {
        // Test rate limiting
        // Test resource limits
        // Test timeout handling
    }
}
```

### Fuzzing Integration

```rust
#[cfg(test)]
mod fuzz_tests {
    use quickcheck::{quickcheck, TestResult};
    
    #[quickcheck]
    fn fuzz_path_validation(path: String) -> TestResult {
        let tool = YourTool::new(Config::default());
        
        if path.is_empty() {
            return TestResult::discard();
        }
        
        let result = tool.validate_path(&path);
        
        // Should either succeed or fail gracefully
        match result {
            Ok(_) => TestResult::passed(),
            Err(LLMSpellError::SecurityViolation(_)) => TestResult::passed(),
            Err(LLMSpellError::InvalidParameter(_)) => TestResult::passed(),
            Err(_) => TestResult::failed(),
        }
    }
}
```

## Tool Security Checklist

Before releasing a tool, ensure:

### Design Phase
- [ ] Threat model completed
- [ ] Security requirements defined
- [ ] Permissions documented
- [ ] Risk assessment done

### Implementation Phase
- [ ] Input validation implemented
- [ ] Output sanitization added
- [ ] Resource limits configured
- [ ] Error handling secured
- [ ] Logging implemented

### Testing Phase
- [ ] Security tests written
- [ ] Fuzzing performed
- [ ] Code review completed
- [ ] Security scan passed

### Deployment Phase
- [ ] Security configuration reviewed
- [ ] Permissions verified
- [ ] Monitoring enabled
- [ ] Incident response plan ready

## Common Vulnerabilities to Avoid

### 1. Injection Flaws
- SQL injection
- Command injection
- Script injection
- Path injection
- LDAP injection

### 2. Broken Authentication
- Weak password storage
- Session fixation
- Missing MFA
- Insecure tokens

### 3. Sensitive Data Exposure
- Unencrypted data
- Verbose errors
- Debug info leaks
- Path disclosure

### 4. XML External Entity (XXE)
- External entity processing
- DTD processing
- XML bombs

### 5. Broken Access Control
- Missing authorization
- IDOR vulnerabilities
- Privilege escalation
- Directory traversal

### 6. Security Misconfiguration
- Default credentials
- Unnecessary features
- Verbose errors
- Open permissions

### 7. Cross-Site Scripting (XSS)
- Reflected XSS
- Stored XSS
- DOM XSS

### 8. Insecure Deserialization
- Object injection
- Remote code execution
- Denial of service

### 9. Using Components with Known Vulnerabilities
- Outdated dependencies
- Unpatched libraries
- EOL components

### 10. Insufficient Logging
- Missing audit trail
- No security events
- Poor monitoring

## Security Resources

### Documentation
- [OWASP Top 10](https://owasp.org/Top10/)
- [CWE/SANS Top 25](https://cwe.mitre.org/top25/)
- [NIST Guidelines](https://csrc.nist.gov/publications/sp)
- [Rust Security Guidelines](https://doc.rust-lang.org/reference/unsafety.html)

### Tools
- `cargo audit` - Dependency scanning
- `cargo-deny` - Supply chain security
- `clippy` - Security lints
- `cargo-fuzz` - Fuzz testing

### Training
- OWASP Security Training
- SANS Security Courses
- Secure Coding Practices

## Getting Help

### Security Team Contacts
- Security Questions: security@llmspell.org
- Vulnerability Reports: security@llmspell.org
- Security Reviews: review@llmspell.org

### Review Process
1. Submit tool for security review
2. Address feedback from security team
3. Pass automated security tests
4. Receive security approval

## Conclusion

Security is a shared responsibility. By following these guidelines, you help ensure that rs-llmspell remains a secure and reliable platform for LLM-driven automation. Remember: when in doubt, choose the more secure option, and always ask the security team for guidance.

Stay secure, and happy coding!