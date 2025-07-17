# Security Guidelines for LLMSpell Tool Development

## Introduction

This document provides comprehensive security guidelines for developers creating tools for the LLMSpell ecosystem. Following these guidelines ensures that tools are secure, reliable, and resistant to common attack vectors.

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

## Input Validation

### Required Validations

```rust
use llmspell_utils::security::validation::{InputValidator, ValidationConfig};

impl YourTool {
    fn validate_input(&self, input: &str) -> Result<String> {
        let validator = InputValidator::new(ValidationConfig::default());
        
        // Always validate paths
        if self.handles_paths() {
            validator.validate_path(input)?;
        }
        
        // Always validate URLs
        if self.handles_urls() {
            validator.validate_url(input)?;
        }
        
        // Always validate commands
        if self.executes_commands() {
            validator.validate_command(input)?;
        }
        
        // Custom validation
        self.custom_validation(input)?;
        
        Ok(validator.sanitize(input))
    }
}
```

### Validation Checklist

- [ ] Check input length limits
- [ ] Validate data types
- [ ] Sanitize special characters
- [ ] Normalize Unicode
- [ ] Prevent null bytes
- [ ] Check encoding consistency

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

## Authentication and Authorization

### Implementing Tool Permissions

```rust
use llmspell_utils::security::auth::{Permission, PermissionChecker};

#[derive(Debug)]
pub struct ToolPermissions {
    pub read_files: bool,
    pub write_files: bool,
    pub network_access: bool,
    pub execute_commands: bool,
}

impl Tool for YourTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![
            Permission::FileRead("/specific/path"),
            Permission::NetworkAccess("api.example.com"),
        ]
    }
    
    fn check_permission(&self, action: &str) -> Result<()> {
        let checker = PermissionChecker::current();
        checker.require(Permission::from_action(action))?;
        Ok(())
    }
}
```

### Authorization Best Practices

1. Check permissions before operations
2. Use capability-based security
3. Implement role-based access control
4. Validate API keys properly
5. Enforce rate limits per user
6. Log authorization failures

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

### Resource Limits Checklist

- [ ] Set execution timeouts
- [ ] Limit memory usage
- [ ] Cap file sizes
- [ ] Restrict concurrent operations
- [ ] Implement rate limiting
- [ ] Monitor resource usage

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

## Cryptography Guidelines

### Secure Random Generation

```rust
use llmspell_utils::security::crypto::{generate_random, hash_password};

// Generate secure random tokens
let token = generate_random(32)?; // 32 bytes of randomness

// Hash passwords properly
let hash = hash_password(password, salt)?;

// Never use:
// - rand() for security
// - MD5 or SHA1
// - Hard-coded salts
// - Predictable seeds
```

### Cryptography Best Practices

1. Use established crypto libraries
2. Generate sufficient entropy
3. Use appropriate key lengths
4. Implement proper key management
5. Rotate keys regularly
6. Never roll your own crypto

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

## Logging and Monitoring

### Security Event Logging

```rust
use llmspell_utils::security::audit::{AuditLogger, SecurityEvent};

fn log_security_event(event: SecurityEvent) {
    let logger = AuditLogger::current();
    
    logger.log_event(event)
        .with_user(user_id)
        .with_tool(tool_name)
        .with_timestamp(Utc::now())
        .with_details(details)
        .commit();
}

// Log important events
log_security_event(SecurityEvent::AuthFailure { reason });
log_security_event(SecurityEvent::RateLimitExceeded { limit });
log_security_event(SecurityEvent::SuspiciousActivity { details });
```

### What to Log

1. Authentication attempts
2. Authorization failures
3. Input validation failures
4. Rate limit violations
5. Resource limit hits
6. Suspicious patterns

## Testing Security

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

### Security Test Coverage

- [ ] Input validation tests
- [ ] Path traversal tests
- [ ] Injection attack tests
- [ ] DoS protection tests
- [ ] Authentication tests
- [ ] Authorization tests
- [ ] Error handling tests

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
- [ ] Penetration testing performed
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

### 4. XXE Attacks
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

### 7. XSS
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

Security is a shared responsibility. By following these guidelines, you help ensure that LLMSpell remains a secure and reliable platform for LLM-driven automation. Remember: when in doubt, choose the more secure option, and always ask the security team for guidance.

Stay secure, and happy coding!